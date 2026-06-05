//! Folder comparison / verification.
//!
//! Compares two trees (typically a source card against an already-offloaded
//! destination) and reports, per relative path, whether the two sides match.
//! Three modes trade thoroughness for speed:
//!   * `PathSize`  — presence + byte size (fast).
//!   * `MTime`     — presence + size + modification time (2s tolerance for
//!     FAT/exFAT granularity).
//!   * `Checksum`  — presence + a full content hash (authoritative).
//!
//! The walk reuses `walk_media`, so the same hidden/system + glob ignore rules
//! as the offload apply.

use crate::offload::engine::walk_media;
use crate::offload::hash::ChecksumAlgo;
use crate::offload::OffloadOptions;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

const HASH_CHUNK: usize = 1024 * 1024;
/// Modification-time tolerance in seconds. FAT/exFAT store mtimes at 2s
/// resolution, so an exact match is not meaningful across filesystems.
const MTIME_TOLERANCE_SECS: u64 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareMode {
    PathSize,
    MTime,
    Checksum,
}

impl CompareMode {
    pub fn parse(s: &str) -> CompareMode {
        match s.trim().to_ascii_uppercase().as_str() {
            "MTIME" => CompareMode::MTime,
            "CHECKSUM" => CompareMode::Checksum,
            _ => CompareMode::PathSize,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareStatus {
    Match,
    MissingInDest,
    ExtraInDest,
    SizeMismatch,
    MTimeMismatch,
    ChecksumMismatch,
}

impl CompareStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CompareStatus::Match => "match",
            CompareStatus::MissingInDest => "missing_in_dest",
            CompareStatus::ExtraInDest => "extra_in_dest",
            CompareStatus::SizeMismatch => "size_mismatch",
            CompareStatus::MTimeMismatch => "mtime_mismatch",
            CompareStatus::ChecksumMismatch => "checksum_mismatch",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompareEntry {
    pub rel_path: String,
    pub status: CompareStatus,
}

#[derive(Debug, Clone, Default)]
pub struct CompareReport {
    pub entries: Vec<CompareEntry>,
    pub matched: u64,
    pub differing: u64,
    pub missing: u64,
    pub extra: u64,
}

#[derive(Clone)]
struct SideEntry {
    abs_path: String,
    size: u64,
}

fn collect(root: &Path, options: &OffloadOptions) -> anyhow::Result<BTreeMap<String, SideEntry>> {
    let mut map = BTreeMap::new();
    walk_media(root, options, |item| {
        map.insert(
            item.relative_path,
            SideEntry {
                abs_path: item.absolute_path,
                size: item.size,
            },
        );
    })?;
    Ok(map)
}

fn mtime_secs(path: &str) -> Option<u64> {
    std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn hash_file(path: &str, algo: ChecksumAlgo) -> anyhow::Result<String> {
    let mut f = std::fs::File::open(path)?;
    let mut hasher = algo.new_hasher();
    let mut buf = vec![0u8; HASH_CHUNK];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize_hex())
}

/// Compare `source` against `dest`. Errors only on an unreadable root; per-file
/// read errors degrade to a mismatch so one bad file doesn't abort the run.
pub fn compare(
    source: &Path,
    dest: &Path,
    mode: CompareMode,
    options: &OffloadOptions,
    algorithm: ChecksumAlgo,
) -> anyhow::Result<CompareReport> {
    let a = collect(source, options)?;
    let b = collect(dest, options)?;

    let mut report = CompareReport::default();

    // Union of relative paths, kept sorted by BTreeMap iteration.
    let mut keys: Vec<&String> = a.keys().collect();
    for k in b.keys() {
        if !a.contains_key(k) {
            keys.push(k);
        }
    }
    keys.sort();

    for key in keys {
        let status = match (a.get(key), b.get(key)) {
            (Some(_), None) => CompareStatus::MissingInDest,
            (None, Some(_)) => CompareStatus::ExtraInDest,
            (Some(sa), Some(sb)) => {
                if sa.size != sb.size {
                    CompareStatus::SizeMismatch
                } else {
                    match mode {
                        CompareMode::PathSize => CompareStatus::Match,
                        CompareMode::MTime => {
                            match (mtime_secs(&sa.abs_path), mtime_secs(&sb.abs_path)) {
                                (Some(ta), Some(tb)) => {
                                    let diff = ta.abs_diff(tb);
                                    if diff <= MTIME_TOLERANCE_SECS {
                                        CompareStatus::Match
                                    } else {
                                        CompareStatus::MTimeMismatch
                                    }
                                }
                                _ => CompareStatus::MTimeMismatch,
                            }
                        }
                        CompareMode::Checksum => {
                            let ha = hash_file(&sa.abs_path, algorithm).ok();
                            let hb = hash_file(&sb.abs_path, algorithm).ok();
                            match (ha, hb) {
                                (Some(ha), Some(hb)) if ha == hb => CompareStatus::Match,
                                _ => CompareStatus::ChecksumMismatch,
                            }
                        }
                    }
                }
            }
            (None, None) => continue,
        };

        match status {
            CompareStatus::Match => report.matched += 1,
            CompareStatus::MissingInDest => report.missing += 1,
            CompareStatus::ExtraInDest => report.extra += 1,
            _ => report.differing += 1,
        }
        report.entries.push(CompareEntry {
            rel_path: key.clone(),
            status,
        });
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> OffloadOptions {
        OffloadOptions::default()
    }

    #[test]
    fn parse_mode_is_case_insensitive_with_fallback() {
        assert_eq!(CompareMode::parse("mtime"), CompareMode::MTime);
        assert_eq!(CompareMode::parse("CHECKSUM"), CompareMode::Checksum);
        assert_eq!(CompareMode::parse("pathsize"), CompareMode::PathSize);
        assert_eq!(CompareMode::parse("nonsense"), CompareMode::PathSize);
    }

    #[test]
    fn detects_match_missing_extra_and_size() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        std::fs::write(a.path().join("same.mov"), b"hello").unwrap();
        std::fs::write(b.path().join("same.mov"), b"hello").unwrap();
        std::fs::write(a.path().join("only_a.mxf"), b"x").unwrap();
        std::fs::write(b.path().join("only_b.mxf"), b"y").unwrap();
        std::fs::write(a.path().join("diff.wav"), b"12345").unwrap();
        std::fs::write(b.path().join("diff.wav"), b"123").unwrap();

        let r = compare(
            a.path(),
            b.path(),
            CompareMode::PathSize,
            &opts(),
            ChecksumAlgo::Blake3,
        )
        .unwrap();

        assert_eq!(r.matched, 1, "same.mov matches");
        assert_eq!(r.missing, 1, "only_a missing in dest");
        assert_eq!(r.extra, 1, "only_b extra in dest");
        assert_eq!(r.differing, 1, "diff.wav size mismatch");
        let diff = r.entries.iter().find(|e| e.rel_path == "diff.wav").unwrap();
        assert_eq!(diff.status, CompareStatus::SizeMismatch);
    }

    #[test]
    fn checksum_mode_flags_same_size_different_content() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        // Same size, different bytes -> only checksum catches it.
        std::fs::write(a.path().join("c.mov"), b"AAAAA").unwrap();
        std::fs::write(b.path().join("c.mov"), b"BBBBB").unwrap();

        let path_size = compare(
            a.path(),
            b.path(),
            CompareMode::PathSize,
            &opts(),
            ChecksumAlgo::Blake3,
        )
        .unwrap();
        assert_eq!(
            path_size.matched, 1,
            "path/size mode sees equal sizes as a match"
        );

        let checksum = compare(
            a.path(),
            b.path(),
            CompareMode::Checksum,
            &opts(),
            ChecksumAlgo::Blake3,
        )
        .unwrap();
        assert_eq!(checksum.differing, 1);
        assert_eq!(checksum.entries[0].status, CompareStatus::ChecksumMismatch);
    }
}
