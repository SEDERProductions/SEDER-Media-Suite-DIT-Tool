use std::path::Path;

/// Returns a stable per-volume identifier.
pub fn volume_id(path: &Path) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(path).map(|m| m.dev()).ok()
    }
    #[cfg(windows)]
    {
        // TODO: Use GetVolumeInformationW via windows-sys
        // For now, return None (same-volume detection disabled on Windows)
        let _ = path;
        None
    }
}

pub fn are_same_volume(a: &Path, b: &Path) -> bool {
    match (volume_id(a), volume_id(b)) {
        (Some(aid), Some(bid)) => aid == bid,
        _ => false,
    }
}
