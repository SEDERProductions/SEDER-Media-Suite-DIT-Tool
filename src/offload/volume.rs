use std::path::{Component, Path};

/// Returns a stable per-volume identifier.
pub fn volume_id(path: &Path) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(path).map(|m| m.dev()).ok()
    }
    #[cfg(windows)]
    {
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if let Some(prefix) = canonical.components().next() {
            if let Component::Prefix(prefix) = prefix {
                let s = prefix.as_os_str().to_string_lossy().to_ascii_uppercase();
                return Some(fxhash(&s));
            }
        }
        None
    }
}

fn fxhash(s: &str) -> u64 {
    let mut h: u64 = 0;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    h
}

pub fn are_same_volume(a: &Path, b: &Path) -> bool {
    match (volume_id(a), volume_id(b)) {
        (Some(aid), Some(bid)) => aid == bid,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_volume_returns_true_for_same_path() {
        assert!(are_same_volume(Path::new("."), Path::new(".")));
    }

    #[test]
    fn volume_id_returns_some_for_cwd() {
        assert!(volume_id(Path::new(".")).is_some());
    }

    #[test]
    fn nonexistent_path_returns_none_on_windows() {
        // On Unix, metadata on a nonexistent path fails
        // On Windows, we canonicalize, which also fails for nonexistent paths
        let result = volume_id(Path::new("/nonexistent/path/xyz123"));
        // Both platforms should return None for nonexistent paths
        assert!(result.is_none());
    }
}
