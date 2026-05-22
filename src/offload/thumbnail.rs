//! Thumbnail extraction via ffmpeg.
//!
//! For each clip we ask ffmpeg to seek a small amount into the file and
//! decode a single frame, scaled down to a fixed width and re-encoded as
//! JPEG. Outputs are content-addressed: the filename is derived from the
//! source clip's already-computed hash + algorithm, so re-extracting for
//! the same content is a no-op and so the cache safely survives source
//! rename / move.
//!
//! This module is a soft dependency on ffmpeg in the same way `ffprobe.rs`
//! is on ffprobe — discovery is at runtime and missing binaries downgrade
//! to None instead of an error.

use crate::offload::ffprobe::discover_ffmpeg;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Output JPEG width. Heights are computed by ffmpeg from the source AR.
pub const THUMB_WIDTH: u32 = 320;
/// Seek offset, expressed as an HH:MM:SS.mmm string passed to `-ss`.
/// One second balances "past the leader/black-frames" with "still works
/// on very short clips" — ffmpeg will pick the closest decodable frame.
pub const SEEK_OFFSET: &str = "00:00:01.0";

/// Path inside `cache_dir` for a thumbnail keyed by content hash. Includes
/// the algorithm name so different algorithms never collide on the same
/// physical file (e.g. an XXH3-64 hash that happens to be 16 hex chars).
pub fn thumb_path(cache_dir: &Path, algorithm: &str, hash: &str) -> PathBuf {
    let safe_algo = algorithm
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let safe_hash = hash
        .chars()
        .take(64)
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>();
    cache_dir.join(format!("{}-{}.jpg", safe_algo, safe_hash))
}

/// Extract a thumbnail JPEG into the cache directory. Returns the cached
/// path on success, or an error describing why ffmpeg failed. Idempotent:
/// if the cached file already exists, returns immediately.
pub fn extract(
    media: &Path,
    cache_dir: &Path,
    algorithm: &str,
    hash: &str,
) -> anyhow::Result<PathBuf> {
    if hash.is_empty() {
        anyhow::bail!("thumbnail cache key requires a non-empty source hash");
    }
    let ffmpeg = discover_ffmpeg()
        .ok_or_else(|| anyhow::anyhow!("ffmpeg not found on PATH or fallback locations"))?;
    std::fs::create_dir_all(cache_dir)
        .map_err(|e| anyhow::anyhow!("create cache dir {}: {}", cache_dir.display(), e))?;

    let out = thumb_path(cache_dir, algorithm, hash);
    if out.exists() {
        return Ok(out);
    }

    let filter = format!("scale={}:-1", THUMB_WIDTH);
    let status = Command::new(&ffmpeg)
        .args(["-y", "-v", "error", "-ss", SEEK_OFFSET, "-i"])
        .arg(media)
        .args(["-frames:v", "1", "-vf", &filter, "-q:v", "5"])
        .arg(&out)
        .status()
        .map_err(|e| anyhow::anyhow!("spawn ffmpeg: {}", e))?;

    if !status.success() {
        // Clean up partial output so a retry doesn't think we succeeded.
        let _ = std::fs::remove_file(&out);
        anyhow::bail!("ffmpeg exited with status {}", status);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_path_includes_algorithm_and_hash() {
        let cache = Path::new("/tmp/seder-thumbs");
        let p = thumb_path(cache, "BLAKE3", "deadbeef");
        assert_eq!(p, Path::new("/tmp/seder-thumbs/BLAKE3-deadbeef.jpg"));
    }

    #[test]
    fn cache_path_sanitizes_algorithm_separators() {
        let p = thumb_path(Path::new("/c"), "XXH3-64", "abc");
        assert!(p.to_string_lossy().contains("XXH3_64-abc"));
    }

    #[test]
    fn cache_path_truncates_overly_long_hashes() {
        let long = "a".repeat(200);
        let p = thumb_path(Path::new("/c"), "SHA1", &long);
        let s = p.to_string_lossy();
        // 64 chars max for the hash component
        let suffix = s.rsplit_once('-').unwrap().1.trim_end_matches(".jpg");
        assert_eq!(suffix.len(), 64);
    }

    #[test]
    fn cache_path_strips_non_alphanumeric_from_hash() {
        let p = thumb_path(Path::new("/c"), "MD5", "ab/cd ef");
        assert!(p.to_string_lossy().ends_with("MD5-abcdef.jpg"));
    }

    #[test]
    fn extract_errors_when_hash_is_empty() {
        let cache = std::env::temp_dir().join("seder-thumb-empty");
        let result = extract(Path::new("/nonexistent.mov"), &cache, "BLAKE3", "");
        assert!(result.is_err());
    }
}
