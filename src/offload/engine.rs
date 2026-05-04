use crate::offload::*;
use crossbeam_channel::{bounded, Sender};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MiB
const CHANNEL_BOUND: usize = 16;

enum ChunkMessage {
    Data(Vec<u8>),
    End,
}

pub fn scan_source(
    source: &Path,
    options: &OffloadOptions,
    progress: &mut dyn FnMut(u64, u64),
) -> anyhow::Result<SourceScan> {
    use walkdir::WalkDir;

    let mut files = Vec::new();
    let mut total_size = 0u64;
    let mut total_files = 0u64;

    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let relative = path.strip_prefix(source)?;
        let rel_str = relative.to_string_lossy().replace('\\', "/");

        if options.ignore_hidden_system && is_hidden_or_system(path) {
            continue;
        }
        if !options.ignore_patterns.is_empty() && should_ignore(&rel_str, &options.ignore_patterns) {
            continue;
        }

        let size = entry.metadata()?.len();
        total_size += size;
        total_files += 1;

        // Compute blake3 hash
        let mut file = File::open(path)?;
        let mut hasher = blake3::Hasher::new();
        let mut buf = vec![0u8; CHUNK_SIZE];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let hash = hasher.finalize().to_hex().to_string();

        files.push(FileEntry {
            relative_path: rel_str,
            size,
            source_blake3: hash,
        });

        progress(total_files, total_size);
    }

    Ok(SourceScan {
        files,
        total_size,
        total_files,
    })
}

pub fn offload_files(
    source: &Path,
    scan: &SourceScan,
    destinations: &[DestinationConfig],
    verify: bool,
    cancel_flag: &AtomicBool,
    progress: &mut dyn FnMut(OffloadProgress),
) -> anyhow::Result<Vec<DestinationResult>> {
    let mut results: Vec<DestinationResult> = destinations
        .iter()
        .map(|d| DestinationResult {
            config: d.clone(),
            state: DestinationState::Pending,
            files_copied: 0,
            files_verified: 0,
            files_failed: 0,
            bytes_copied: 0,
            final_error: None,
        })
        .collect();

    let overall_files_total = scan.files.len() as u64;
    let overall_bytes_total = scan.total_size;
    let mut overall_files_completed = 0u64;
    let mut overall_bytes_completed = 0u64;

    for (file_idx, file_entry) in scan.files.iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            for r in &mut results {
                if r.state != DestinationState::Complete {
                    r.state = DestinationState::Failed;
                    if r.final_error.is_none() {
                        r.final_error = Some("Cancelled by user".into());
                    }
                }
            }
            return Ok(results);
        }

        let src_path = source.join(&file_entry.relative_path);

        // Copy file to all destinations
        let copy_result = copy_file_fanout(
            &src_path,
            &file_entry.relative_path,
            destinations,
            cancel_flag,
        );

        match copy_result {
            Ok(dest_hashes) => {
                for (idx, hash) in dest_hashes.iter().enumerate() {
                    if let Some(h) = hash {
                        results[idx].files_copied += 1;
                        results[idx].bytes_copied += file_entry.size;
                        results[idx].state = DestinationState::Copying;

                        if verify {
                            results[idx].state = DestinationState::Verifying;
                            let dest_path = destinations[idx]
                                .path
                                .join(&file_entry.relative_path);
                            match verify_file(&dest_path, &file_entry.source_blake3) {
                                Ok(()) => {
                                    results[idx].files_verified += 1;
                                }
                                Err(e) => {
                                    results[idx].files_failed += 1;
                                    results[idx].final_error = Some(format!(
                                        "{}: verify failed - {}",
                                        file_entry.relative_path, e
                                    ));
                                }
                            }
                        }
                    } else {
                        results[idx].files_failed += 1;
                        results[idx].state = DestinationState::Failed;
                        if results[idx].final_error.is_none() {
                            results[idx].final_error = Some(format!(
                                "{}: copy failed",
                                file_entry.relative_path
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                for r in &mut results {
                    r.files_failed += 1;
                    r.state = DestinationState::Failed;
                    if r.final_error.is_none() {
                        r.final_error = Some(format!(
                            "{}: copy failed - {}",
                            file_entry.relative_path, e
                        ));
                    }
                }
            }
        }

        overall_files_completed += 1;
        overall_bytes_completed += file_entry.size;

        let dest_progress: Vec<DestinationProgress> = results
            .iter()
            .enumerate()
            .map(|(i, r)| DestinationProgress {
                index: i,
                state: r.state,
                files_completed: r.files_copied + r.files_verified,
                files_total: overall_files_total,
                bytes_completed: r.bytes_copied,
                bytes_total: overall_bytes_total,
                current_file: file_entry.relative_path.clone(),
                error: r.final_error.clone(),
            })
            .collect();

        progress(OffloadProgress {
            phase: if verify { "verifying".into() } else { "copying".into() },
            overall_files_completed,
            overall_files_total,
            overall_bytes_completed,
            overall_bytes_total,
            current_file: file_entry.relative_path.clone(),
            destinations: dest_progress,
        });
    }

    for r in &mut results {
        if r.state != DestinationState::Failed && r.files_failed == 0 {
            r.state = DestinationState::Complete;
        }
    }

    Ok(results)
}

fn copy_file_fanout(
    src_path: &Path,
    relative_path: &str,
    destinations: &[DestinationConfig],
    cancel_flag: &AtomicBool,
) -> anyhow::Result<Vec<Option<String>>> {
    let mut src_file = File::open(src_path)
        .map_err(|e| anyhow::anyhow!("Open source {}: {}", src_path.display(), e))?;

    let mut senders: Vec<Sender<ChunkMessage>> = Vec::with_capacity(destinations.len());
    let mut handles = Vec::with_capacity(destinations.len());

    for dest in destinations {
        let dest_path = dest.path.join(relative_path);
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let (tx, rx) = bounded::<ChunkMessage>(CHANNEL_BOUND);
        senders.push(tx);

        let handle = std::thread::spawn(move || -> anyhow::Result<String> {
            let mut file = File::create(&dest_path)
                .map_err(|e| anyhow::anyhow!("Create {}: {}", dest_path.display(), e))?;
            let mut hasher = blake3::Hasher::new();

            for msg in rx {
                match msg {
                    ChunkMessage::Data(bytes) => {
                        file.write_all(&bytes)?;
                        hasher.update(&bytes);
                    }
                    ChunkMessage::End => break,
                }
            }
            file.sync_data()?;
            Ok(hasher.finalize().to_hex().to_string())
        });
        handles.push(handle);
    }

    let mut buf = vec![0u8; CHUNK_SIZE];
    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            for sender in &senders {
                let _ = sender.send(ChunkMessage::End);
            }
            return Err(anyhow::anyhow!("Cancelled by user"));
        }

        let n = src_file.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let chunk = ChunkMessage::Data(buf[..n].to_vec());
        for sender in &senders {
            sender.send(chunk.clone())
                .map_err(|_| anyhow::anyhow!("Destination writer disconnected"))?;
        }
    }

    for sender in &senders {
        sender.send(ChunkMessage::End)?;
    }

    let mut dest_hashes = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.join() {
            Ok(Ok(hash)) => dest_hashes.push(Some(hash)),
            Ok(Err(e)) => {
                dest_hashes.push(None);
                eprintln!("Destination writer error: {}", e);
            }
            Err(_) => {
                dest_hashes.push(None);
                eprintln!("Destination writer thread panicked");
            }
        }
    }

    Ok(dest_hashes)
}

fn verify_file(dest_path: &Path, expected_blake3: &str) -> anyhow::Result<()> {
    let mut file = File::open(dest_path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; CHUNK_SIZE];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let actual = hasher.finalize().to_hex().to_string();
    if actual != expected_blake3 {
        anyhow::bail!(
            "Hash mismatch\n  expected: {}\n  actual:   {}",
            expected_blake3,
            actual
        );
    }
    Ok(())
}

fn is_hidden_or_system(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let attr = meta.file_attributes();
            if (attr & 0x2) != 0 || (attr & 0x4) != 0 {
                return true;
            }
        }
    }
    if let Some(name) = path.file_name() {
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "$RECYCLE.BIN" || name == "System Volume Information" {
            return true;
        }
    }
    false
}

fn should_ignore(rel_path: &str, patterns: &[String]) -> bool {
    let basename = Path::new(rel_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    for pattern in patterns {
        let pat = pattern.trim();
        if pat.is_empty() {
            continue;
        }
        if glob_match(pat, &basename) || glob_match(pat, rel_path) {
            return true;
        }
    }
    false
}

fn glob_match(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return text == pattern;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return text == pattern;
    }
    if parts[0].is_empty() {
        if parts.len() == 2 {
            return text.ends_with(parts[1]);
        }
        if parts.last().map_or(true, |p| p.is_empty()) {
            let middle = &pattern[1..pattern.len() - 1];
            return text.contains(middle);
        }
    }
    if parts.last().map_or(false, |p| p.is_empty()) {
        return text.starts_with(parts[0]);
    }
    if parts.len() == 3 && parts[0].is_empty() && parts[2].is_empty() {
        return text.contains(parts[1]);
    }
    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        match text[pos..].find(part) {
            Some(idx) => {
                if i == 0 && idx != 0 {
                    return false;
                }
                pos += idx + part.len();
            }
            None => return false,
        }
    }
    if !pattern.ends_with('*') && pos != text.len() {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_exact_match() {
        assert!(glob_match("hello.txt", "hello.txt"));
        assert!(!glob_match("hello.txt", "hello2.txt"));
    }

    #[test]
    fn glob_prefix_wildcard() {
        assert!(glob_match("*.txt", "hello.txt"));
        assert!(glob_match("*.log", "error.log"));
        assert!(!glob_match("*.txt", "hello.log"));
    }

    #[test]
    fn glob_suffix_wildcard() {
        assert!(glob_match("hello.*", "hello.txt"));
        assert!(glob_match("hello.*", "hello.log"));
        assert!(!glob_match("hello.*", "world.txt"));
    }

    #[test]
    fn glob_contains_wildcard() {
        assert!(glob_match("*test*", "mytestfile"));
        assert!(glob_match("*tmp*", "temp_tmp_file"));
        assert!(!glob_match("*test*", "nothing"));
    }

    #[test]
    fn glob_both_ends_wildcard() {
        assert!(glob_match("*middle*", "amiddleb"));
        assert!(!glob_match("*middle*", "nothing"));
    }

    #[test]
    fn should_ignore_basename_glob() {
        let patterns = vec!["*.txt".to_string()];
        assert!(should_ignore("folder/hello.txt", &patterns));
        assert!(!should_ignore("folder/hello.log", &patterns));
    }

    #[test]
    fn should_ignore_path_wildcard() {
        let patterns = vec!["*temp*".to_string()];
        assert!(should_ignore("some/temp/file.txt", &patterns));
    }

    #[test]
    fn should_ignore_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(!should_ignore("anything.txt", &patterns));
    }

    #[test]
    fn is_hidden_dotfile() {
        use std::path::Path;
        assert!(is_hidden_or_system(Path::new(".hidden")));
        assert!(!is_hidden_or_system(Path::new("visible")));
    }

    #[test]
    fn is_hidden_system_folders() {
        use std::path::Path;
        assert!(is_hidden_or_system(Path::new("$RECYCLE.BIN/something")));
        assert!(is_hidden_or_system(Path::new("System Volume Information/something")));
    }

    #[test]
    fn verify_file_matching_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        let data = b"hello world test data for verification\n";
        std::fs::write(&path, data).unwrap();

        let hash = blake3::hash(data).to_hex().to_string();
        assert!(verify_file(&path, &hash).is_ok());
    }

    #[test]
    fn verify_file_mismatched_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"correct data").unwrap();

        let wrong_hash = blake3::hash(b"different data").to_hex().to_string();
        assert!(verify_file(&path, &wrong_hash).is_err());
    }
}
