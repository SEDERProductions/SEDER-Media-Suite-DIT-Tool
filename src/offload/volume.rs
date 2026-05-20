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
        use std::path::Component;
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

#[cfg(windows)]
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

/// Best-effort detection of LTFS-mounted volumes. Parses platform-native
/// mount metadata: `/proc/mounts` on Linux, `mount` output on macOS, and
/// GetVolumeInformationW filesystem name on Windows. Returns true if any
/// mount line covering `path` advertises an "ltfs" filesystem type.
pub fn is_ltfs_volume(path: &Path) -> bool {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let canonical_str = canonical.to_string_lossy().into_owned();

    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/mounts") {
            return parse_mounts_lookup_ltfs(&contents, &canonical_str);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("mount").output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                return parse_macos_mount_lookup_ltfs(&s, &canonical_str);
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        // GetVolumeInformationW returns a filesystem name; LTFS drivers on
        // Windows expose "LTFS" there. The crate's existing logic stays in
        // sync with std without pulling winapi for one call: shell out to
        // PowerShell when present, otherwise just return false.
        if let Ok(output) = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-Volume -FilePath '{}').FileSystemType",
                    canonical_str.replace('\'', "''")
                ),
            ])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                return s.to_ascii_uppercase().contains("LTFS");
            }
        }
    }

    let _ = canonical_str;
    false
}

#[cfg(target_os = "linux")]
fn parse_mounts_lookup_ltfs(mounts: &str, target: &str) -> bool {
    for line in mounts.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 3 {
            continue;
        }
        let mount_point = cols[1];
        let fs_type = cols[2];
        if target.starts_with(mount_point) && fs_type.eq_ignore_ascii_case("ltfs") {
            return true;
        }
    }
    false
}

#[cfg(target_os = "macos")]
fn parse_macos_mount_lookup_ltfs(mount_output: &str, target: &str) -> bool {
    // mount output: `/dev/disk2s1 on /Volumes/LTO (ltfs, local, nodev, ...)`
    for line in mount_output.lines() {
        if let Some(on_idx) = line.find(" on ") {
            let after = &line[on_idx + 4..];
            if let Some(paren_idx) = after.find(" (") {
                let mount_point = &after[..paren_idx];
                let opts = &after[paren_idx + 2..];
                if target.starts_with(mount_point)
                    && opts
                        .split([',', ' '])
                        .any(|t| t.eq_ignore_ascii_case("ltfs"))
                {
                    return true;
                }
            }
        }
    }
    false
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

    #[test]
    fn ltfs_detection_does_not_crash_on_arbitrary_paths() {
        // We can't assert true/false portably here — LTFS volumes are
        // not present in CI. Just guarantee the function is safe to
        // call on any path.
        let _ = is_ltfs_volume(Path::new("."));
        let _ = is_ltfs_volume(Path::new("/nonexistent"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_ltfs_line_from_proc_mounts() {
        let sample = concat!(
            "rootfs / rootfs rw 0 0\n",
            "/dev/sg1 /mnt/lto ltfs rw,relatime 0 0\n",
            "tmpfs /tmp tmpfs rw 0 0\n",
        );
        assert!(parse_mounts_lookup_ltfs(sample, "/mnt/lto/clip.mxf"));
        assert!(!parse_mounts_lookup_ltfs(sample, "/tmp/foo"));
        assert!(!parse_mounts_lookup_ltfs(sample, "/etc/hosts"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn proc_mounts_treats_fs_type_case_insensitively() {
        let sample = "/dev/sg0 /mnt/tape LTFS rw 0 0\n";
        assert!(parse_mounts_lookup_ltfs(sample, "/mnt/tape/data.mxf"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn parses_ltfs_line_from_macos_mount() {
        let sample = "/dev/disk2s1 on /Volumes/LTO (ltfs, local, nodev)\n\
                      /dev/disk1s1 on / (apfs, local, journaled)\n";
        assert!(parse_macos_mount_lookup_ltfs(sample, "/Volumes/LTO/clip"));
        assert!(!parse_macos_mount_lookup_ltfs(sample, "/Users/x"));
    }
}
