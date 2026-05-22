//! Proxy / transcode generation via ffmpeg.
//!
//! Proxies are post-verification: we hand ffmpeg a verified source file
//! and ask for a smaller editable copy in a chosen preset format. As with
//! `thumbnail.rs` and `ffprobe.rs`, ffmpeg is a soft dependency — without
//! it, this module returns Err immediately rather than blocking the
//! offload pipeline.

use crate::offload::ffprobe::discover_ffmpeg;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyPreset {
    /// Apple ProRes 422 Proxy at 1080p, .mov container.
    #[default]
    ProResProxy1080,
    /// H.264 720p, .mp4 container — small "review" proxies.
    H264_720,
    /// Avid DNxHR Low Bandwidth, .mov container.
    DnxhrLb,
}

impl ProxyPreset {
    pub fn as_str(self) -> &'static str {
        match self {
            ProxyPreset::ProResProxy1080 => "ProRes Proxy 1080p",
            ProxyPreset::H264_720 => "H.264 720p",
            ProxyPreset::DnxhrLb => "DNxHR LB",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "PRORES" | "PRORES_PROXY" | "PRORESPROXY" | "PRORES PROXY 1080P" => {
                Some(ProxyPreset::ProResProxy1080)
            }
            "H264" | "H264_720" | "H.264 720P" => Some(ProxyPreset::H264_720),
            "DNXHR" | "DNXHR_LB" | "DNXHR LB" => Some(ProxyPreset::DnxhrLb),
            _ => None,
        }
    }

    /// File extension (no dot) for this preset's output container.
    pub fn extension(self) -> &'static str {
        match self {
            ProxyPreset::ProResProxy1080 => "mov",
            ProxyPreset::H264_720 => "mp4",
            ProxyPreset::DnxhrLb => "mov",
        }
    }

    fn ffmpeg_args(self) -> Vec<&'static str> {
        match self {
            ProxyPreset::ProResProxy1080 => vec![
                "-c:v",
                "prores_ks",
                "-profile:v",
                "0",
                "-vf",
                "scale=-2:1080",
                "-c:a",
                "pcm_s16le",
            ],
            ProxyPreset::H264_720 => vec![
                "-c:v",
                "libx264",
                "-preset",
                "fast",
                "-crf",
                "23",
                "-vf",
                "scale=-2:720",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                "-movflags",
                "+faststart",
            ],
            ProxyPreset::DnxhrLb => vec![
                "-c:v",
                "dnxhd",
                "-profile:v",
                "dnxhr_lb",
                "-vf",
                "scale=-2:720,format=yuv422p",
                "-c:a",
                "pcm_s16le",
            ],
        }
    }
}

/// Where the proxy file should land given the chosen preset and the
/// caller's preferred output root. Layout: `proxies_root/<preset>/<filename>.<ext>`.
pub fn proxy_path(proxies_root: &Path, preset: ProxyPreset, source: &Path) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("clip");
    let preset_dir = preset.as_str().replace(' ', "_");
    proxies_root
        .join(preset_dir)
        .join(format!("{}.{}", stem, preset.extension()))
}

/// Transcode `media` into `proxies_root` using the chosen preset.
/// Skips work if the output already exists.
pub fn transcode(
    media: &Path,
    proxies_root: &Path,
    preset: ProxyPreset,
) -> anyhow::Result<PathBuf> {
    let ffmpeg = discover_ffmpeg()
        .ok_or_else(|| anyhow::anyhow!("ffmpeg not found on PATH or fallback locations"))?;
    let out = proxy_path(proxies_root, preset, media);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("create proxy dir {}: {}", parent.display(), e))?;
    }
    if out.exists() {
        return Ok(out);
    }

    let mut cmd = Command::new(&ffmpeg);
    cmd.args(["-y", "-v", "error", "-i"]).arg(media);
    for a in preset.ffmpeg_args() {
        cmd.arg(a);
    }
    cmd.arg(&out);

    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("spawn ffmpeg: {}", e))?;
    if !status.success() {
        let _ = std::fs::remove_file(&out);
        anyhow::bail!("ffmpeg exited with status {}", status);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_parse_roundtrip() {
        for p in [
            ProxyPreset::ProResProxy1080,
            ProxyPreset::H264_720,
            ProxyPreset::DnxhrLb,
        ] {
            assert_eq!(ProxyPreset::parse(p.as_str()), Some(p));
        }
    }

    #[test]
    fn preset_parse_aliases() {
        assert_eq!(
            ProxyPreset::parse("prores"),
            Some(ProxyPreset::ProResProxy1080)
        );
        assert_eq!(ProxyPreset::parse("h264"), Some(ProxyPreset::H264_720));
        assert_eq!(ProxyPreset::parse("dnxhr"), Some(ProxyPreset::DnxhrLb));
        assert_eq!(ProxyPreset::parse("nope"), None);
    }

    #[test]
    fn output_path_layout() {
        let root = Path::new("/proxies");
        let p = proxy_path(root, ProxyPreset::H264_720, Path::new("clip001.mxf"));
        assert_eq!(p, Path::new("/proxies/H.264_720p/clip001.mp4"));

        let p = proxy_path(root, ProxyPreset::ProResProxy1080, Path::new("a.r3d"));
        assert_eq!(p, Path::new("/proxies/ProRes_Proxy_1080p/a.mov"));

        let p = proxy_path(root, ProxyPreset::DnxhrLb, Path::new("b.mov"));
        assert_eq!(p, Path::new("/proxies/DNxHR_LB/b.mov"));
    }

    #[test]
    fn extensions_match_containers() {
        assert_eq!(ProxyPreset::ProResProxy1080.extension(), "mov");
        assert_eq!(ProxyPreset::H264_720.extension(), "mp4");
        assert_eq!(ProxyPreset::DnxhrLb.extension(), "mov");
    }

    #[test]
    fn ffmpeg_args_are_non_empty_and_include_codec() {
        for p in [
            ProxyPreset::ProResProxy1080,
            ProxyPreset::H264_720,
            ProxyPreset::DnxhrLb,
        ] {
            let args = p.ffmpeg_args();
            assert!(!args.is_empty());
            assert!(args.contains(&"-c:v"), "preset {:?} missing -c:v", p);
        }
    }
}
