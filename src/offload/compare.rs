use crate::offload::engine::walk_media;
use crate::offload::hash::ChecksumAlgo;
use crate::offload::*;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareMode {
    PathSize,
    MTime,
    Checksum,
}

impl CompareMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "mtime" => CompareMode::MTime,
            "checksum" => CompareMode::Checksum,
            _ => CompareMode::PathSize,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareStatus {
    Match,
    MissingInDest,
    ExtraInDest,
    SizeMismatch,
    MTimeMismatch,
    ChecksumMismatch,
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompareEntry {
    pub path: String,
    pub status: CompareStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompareReport {
    pub mode: CompareMode,
    pub source: String,
    pub dest: String,
    pub matched: u64,
    pub differing: u64,
    pub missing_in_dest: u64,
    pub extra_in_dest: u64,
    /// Non-match entries only (matches omitted to keep the payload small).
    pub entries: Vec<CompareEntry>,
}

/// Compare `source` against `dest` using the given mode, applying the same
/// walk/ignore rules used by the offload engine.
pub fn compare(
    source: &Path,
    dest: &Path,
    mode: CompareMode,
    options: &OffloadOptions,
    algorithm: ChecksumAlgo,
) -> anyhow::Result<CompareReport> {
    // 1. Walk source into a map: rel_path -> (size, mtime_secs)
    let mut src_map: HashMap<String, (u64, i64)> = HashMap::new();
    walk_media(source, options, |e| {
        let mtime = mtime_secs(source.join(&e.relative_path));
        src_map.insert(e.relative_path, (e.size, mtime));
    })?;

    // 2. Walk dest into a map (use a permissive options set — don't apply
    //    source ignore rules to the destination tree).
    let dest_options = OffloadOptions {
        ignore_hidden_system: false,
        ignore_patterns: vec![],
        ..OffloadOptions::default()
    };
    let mut dst_map: HashMap<String, (u64, i64)> = HashMap::new();
    walk_media(dest, &dest_options, |e| {
        let mtime = mtime_secs(dest.join(&e.relative_path));
        dst_map.insert(e.relative_path, (e.size, mtime));
    })?;

    let mut entries: Vec<CompareEntry> = Vec::new();
    let mut matched: u64 = 0;
    let mut differing: u64 = 0;
    let mut missing_in_dest: u64 = 0;
    let mut extra_in_dest: u64 = 0;

    // 3. Compare each source file
    for (rel, (src_size, src_mtime)) in &src_map {
        match dst_map.get(rel) {
            None => {
                missing_in_dest += 1;
                entries.push(CompareEntry {
                    path: rel.clone(),
                    status: CompareStatus::MissingInDest,
                    detail: None,
                });
            }
            Some(&(dst_size, dst_mtime)) => {
                if *src_size != dst_size {
                    differing += 1;
                    entries.push(CompareEntry {
                        path: rel.clone(),
                        status: CompareStatus::SizeMismatch,
                        detail: Some(format!("src={src_size} dst={dst_size}")),
                    });
                    continue;
                }
                match mode {
                    CompareMode::PathSize => {
                        matched += 1;
                    }
                    CompareMode::MTime => {
                        // 2-second tolerance for FAT/exFAT granularity
                        if (src_mtime - dst_mtime).abs() > 2 {
                            differing += 1;
                            entries.push(CompareEntry {
                                path: rel.clone(),
                                status: CompareStatus::MTimeMismatch,
                                detail: Some(format!(
                                    "src_mtime={src_mtime} dst_mtime={dst_mtime}"
                                )),
                            });
                        } else {
                            matched += 1;
                        }
                    }
                    CompareMode::Checksum => {
                        let src_path = source.join(rel);
                        let dst_path = dest.join(rel);
                        let src_hash = hash_file(&src_path, algorithm);
                        let dst_hash = hash_file(&dst_path, algorithm);
                        match (src_hash, dst_hash) {
                            (Ok(sh), Ok(dh)) => {
                                if sh == dh {
                                    matched += 1;
                                } else {
                                    differing += 1;
                                    entries.push(CompareEntry {
                                        path: rel.clone(),
                                        status: CompareStatus::ChecksumMismatch,
                                        detail: None,
                                    });
                                }
                            }
                            (Err(e), _) | (_, Err(e)) => {
                                differing += 1;
                                entries.push(CompareEntry {
                                    path: rel.clone(),
                                    status: CompareStatus::Error,
                                    detail: Some(e.to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Extra files in dest
    for rel in dst_map.keys() {
        if !src_map.contains_key(rel) {
            extra_in_dest += 1;
            entries.push(CompareEntry {
                path: rel.clone(),
                status: CompareStatus::ExtraInDest,
                detail: None,
            });
        }
    }

    // Sort for stable output
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(CompareReport {
        mode,
        source: source.to_string_lossy().into_owned(),
        dest: dest.to_string_lossy().into_owned(),
        matched,
        differing,
        missing_in_dest,
        extra_in_dest,
        entries,
    })
}

fn hash_file(path: &Path, algorithm: ChecksumAlgo) -> anyhow::Result<String> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(path)?;
    let mut hasher = algorithm.new_hasher(); // Box<dyn FileHasher>
    let mut buf = vec![0u8; 1 << 20]; // 1 MiB
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize_hex())
}

fn mtime_secs(path: impl AsRef<Path>) -> i64 {
    path.as_ref()
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(dir: &Path, name: &str, content: &[u8]) {
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn compare_identical_path_size() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        write(src.path(), "a.mov", b"data");
        write(dst.path(), "a.mov", b"data");
        let report = compare(
            src.path(),
            dst.path(),
            CompareMode::PathSize,
            &OffloadOptions::default(),
            ChecksumAlgo::Blake3,
        )
        .unwrap();
        assert_eq!(report.matched, 1);
        assert_eq!(report.entries.len(), 0);
    }

    #[test]
    fn compare_missing_in_dest() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        write(src.path(), "a.mov", b"data");
        let report = compare(
            src.path(),
            dst.path(),
            CompareMode::PathSize,
            &OffloadOptions::default(),
            ChecksumAlgo::Blake3,
        )
        .unwrap();
        assert_eq!(report.missing_in_dest, 1);
        assert_eq!(report.entries[0].status, CompareStatus::MissingInDest);
    }

    #[test]
    fn compare_size_mismatch() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        write(src.path(), "a.mov", b"short");
        write(dst.path(), "a.mov", b"longer content here");
        let report = compare(
            src.path(),
            dst.path(),
            CompareMode::PathSize,
            &OffloadOptions::default(),
            ChecksumAlgo::Blake3,
        )
        .unwrap();
        assert_eq!(report.differing, 1);
        assert_eq!(report.entries[0].status, CompareStatus::SizeMismatch);
    }
}
