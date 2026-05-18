use crate::offload::hash::ChecksumAlgo;
use crate::offload::*;
use crossbeam_channel::{bounded, Sender};
use globset::{Glob, GlobSetBuilder};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MiB
const CHANNEL_BOUND: usize = 16;

const IO_RETRY_ATTEMPTS: u32 = 3;
const IO_RETRY_BASE_DELAY: Duration = Duration::from_millis(50);

fn is_transient_io_error(kind: io::ErrorKind) -> bool {
    matches!(
        kind,
        io::ErrorKind::Interrupted
            | io::ErrorKind::WouldBlock
            | io::ErrorKind::TimedOut
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::BrokenPipe
    )
}

fn retry_io<T, F>(mut op: F) -> io::Result<T>
where
    F: FnMut() -> io::Result<T>,
{
    let mut attempt: u32 = 0;
    loop {
        match op() {
            Ok(value) => return Ok(value),
            Err(err) => {
                if attempt + 1 >= IO_RETRY_ATTEMPTS || !is_transient_io_error(err.kind()) {
                    return Err(err);
                }
                std::thread::sleep(IO_RETRY_BASE_DELAY * (1u32 << attempt));
                attempt += 1;
            }
        }
    }
}

#[derive(Clone)]
enum ChunkMessage {
    Data(Vec<u8>),
    End,
}

#[derive(Debug)]
pub enum FileCopyStatus {
    Copied(String),
    Skipped,
    Failed(String),
}

pub fn scan_source(
    source: &Path,
    options: &OffloadOptions,
    progress: &mut dyn FnMut(u64, u64),
) -> anyhow::Result<SourceScan> {
    use walkdir::WalkDir;

    let walker = WalkDir::new(source)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if !options.ignore_hidden_system {
                return true;
            }
            if entry.depth() == 0 {
                return true;
            }
            !is_hidden_or_system(entry.path())
        });

    let mut files = Vec::new();
    let mut total_size = 0u64;
    let mut total_files = 0u64;
    let mut buf = vec![0u8; CHUNK_SIZE];

    let ignore_glob = if !options.ignore_patterns.is_empty() {
        let mut builder = GlobSetBuilder::new();
        for p in &options.ignore_patterns {
            let trimmed = p.trim();
            if !trimmed.is_empty() {
                if let Ok(glob) = Glob::new(trimmed) {
                    builder.add(glob);
                }
            }
        }
        Some(builder.build().unwrap())
    } else {
        None
    };

    for entry in walker {
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
        if let Some(ref gs) = ignore_glob {
            let basename = Path::new(&rel_str)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&rel_str);
            if gs.is_match(rel_str.as_str()) || gs.is_match(basename) {
                continue;
            }
        }

        let size = entry.metadata()?.len();
        total_size += size;
        total_files += 1;

        let mut file = File::open(path)?;
        let mut hasher = options.algorithm.new_hasher();
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let hash = hasher.finalize_hex();

        files.push(FileEntry {
            relative_path: rel_str,
            size,
            source_hash: hash,
            algorithm: options.algorithm,
        });

        progress(total_files, total_size);
    }

    Ok(SourceScan {
        files,
        total_size,
        total_files,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn offload_files(
    source: &Path,
    scan: &SourceScan,
    destinations: &[DestinationConfig],
    verify: bool,
    cancel_flag: &AtomicBool,
    progress: &mut dyn FnMut(OffloadProgress),
    sync_writes: bool,
    skip_existing: bool,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<DestinationResult>> {
    let mut results: Vec<DestinationResult> = destinations
        .iter()
        .map(|d| DestinationResult {
            config: d.clone(),
            state: DestinationState::Pending,
            files_copied: 0,
            files_verified: 0,
            files_failed: 0,
            files_skipped: 0,
            bytes_copied: 0,
            final_error: None,
        })
        .collect();

    let overall_files_total = scan.files.len() as u64;
    let overall_bytes_total = scan.total_size;
    let mut overall_bytes_completed = 0u64;
    let mut verify_buf = vec![0u8; CHUNK_SIZE];

    for (idx, file_entry) in scan.files.iter().enumerate() {
        let overall_files_completed = (idx + 1) as u64;
        if cancel_flag.load(Ordering::Relaxed) {
            for r in &mut results {
                if r.state != DestinationState::Complete {
                    r.state = DestinationState::Cancelled;
                    if r.final_error.is_none() {
                        r.final_error = Some("Cancelled by user".into());
                    }
                }
            }
            return Ok(results);
        }

        let src_path = source.join(&file_entry.relative_path);

        // Collect per-file warnings
        let mut file_warnings = Vec::new();

        // Copy file to all destinations
        let copy_result = copy_file_fanout(
            &src_path,
            &file_entry.relative_path,
            destinations,
            cancel_flag,
            sync_writes,
            skip_existing,
            file_entry.algorithm,
            &mut file_warnings,
        );

        warnings.extend(file_warnings);

        let mut dest_file_status: Vec<FileTransferStatus> =
            vec![FileTransferStatus::None; destinations.len()];

        match copy_result {
            Ok(statuses) => {
                for (idx, status) in statuses.iter().enumerate() {
                    match status {
                        FileCopyStatus::Copied(_) => {
                            results[idx].files_copied += 1;
                            results[idx].bytes_copied += file_entry.size;
                            results[idx].state = DestinationState::Copying;

                            if verify {
                                results[idx].state = DestinationState::Verifying;
                                let dest_path =
                                    destinations[idx].path.join(&file_entry.relative_path);
                                match verify_file(
                                    &dest_path,
                                    &file_entry.source_hash,
                                    file_entry.algorithm,
                                    &mut verify_buf,
                                ) {
                                    Ok(()) => {
                                        results[idx].files_verified += 1;
                                        dest_file_status[idx] = FileTransferStatus::Verified;
                                    }
                                    Err(e) => {
                                        results[idx].files_failed += 1;
                                        results[idx].final_error = Some(format!(
                                            "{}: verify failed - {}",
                                            file_entry.relative_path, e
                                        ));
                                        dest_file_status[idx] = FileTransferStatus::Failed;
                                    }
                                }
                            } else {
                                dest_file_status[idx] = FileTransferStatus::Copied;
                            }
                        }
                        FileCopyStatus::Skipped => {
                            results[idx].files_skipped += 1;
                            results[idx].state = DestinationState::Copying;
                            dest_file_status[idx] = FileTransferStatus::Skipped;
                        }
                        FileCopyStatus::Failed(err) => {
                            results[idx].files_failed += 1;
                            results[idx].state = DestinationState::Failed;
                            if results[idx].final_error.is_none() {
                                results[idx].final_error =
                                    Some(format!("{}: {}", file_entry.relative_path, err));
                            }
                            dest_file_status[idx] = FileTransferStatus::Failed;
                        }
                    }
                }
            }
            Err(e) => {
                for (idx, r) in results.iter_mut().enumerate() {
                    r.files_failed += 1;
                    r.state = DestinationState::Failed;
                    if r.final_error.is_none() {
                        r.final_error =
                            Some(format!("{}: copy failed - {}", file_entry.relative_path, e));
                    }
                    dest_file_status[idx] = FileTransferStatus::Failed;
                }
            }
        }

        overall_bytes_completed += file_entry.size;

        let dest_progress: Vec<DestinationProgress> = results
            .iter()
            .enumerate()
            .map(|(i, r)| DestinationProgress {
                index: i,
                state: r.state,
                files_completed: r.files_copied + r.files_verified + r.files_skipped,
                files_total: overall_files_total,
                bytes_completed: r.bytes_copied,
                bytes_total: overall_bytes_total,
                current_file: file_entry.relative_path.clone(),
                last_file_status: dest_file_status[i],
                error: r.final_error.clone(),
            })
            .collect();

        progress(OffloadProgress {
            phase: if verify {
                "verifying".into()
            } else {
                "copying".into()
            },
            overall_files_completed,
            overall_files_total,
            overall_bytes_completed,
            overall_bytes_total,
            current_file: file_entry.relative_path.clone(),
            destinations: dest_progress,
            warnings: warnings.clone(),
        });
    }

    for r in &mut results {
        if r.state != DestinationState::Failed
            && r.state != DestinationState::Cancelled
            && r.files_failed == 0
        {
            r.state = DestinationState::Complete;
        }
    }

    Ok(results)
}

#[allow(clippy::too_many_arguments)]
fn copy_file_fanout(
    src_path: &Path,
    relative_path: &str,
    destinations: &[DestinationConfig],
    cancel_flag: &AtomicBool,
    sync_writes: bool,
    skip_existing: bool,
    algorithm: ChecksumAlgo,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<FileCopyStatus>> {
    let dest_count = destinations.len();
    let mut result: Vec<FileCopyStatus> = Vec::with_capacity(dest_count);

    // Determine which destinations need a copy thread vs skip
    for (idx, dest) in destinations.iter().enumerate() {
        let dest_path = dest.path.join(relative_path);
        if skip_existing && dest_path.exists() {
            result.push(FileCopyStatus::Skipped);
        } else {
            // Temporary placeholder - will be filled by the actual copy
            result.push(FileCopyStatus::Failed("Not started".into()));
            _ = idx; // suppress unused warning
        }
    }

    // Spawn writer threads for destinations that need copying
    let mut senders: Vec<(usize, Sender<ChunkMessage>)> = Vec::new();
    let mut handles: Vec<(usize, std::thread::JoinHandle<anyhow::Result<String>>)> = Vec::new();

    for (idx, dest) in destinations.iter().enumerate() {
        if matches!(result[idx], FileCopyStatus::Skipped) {
            continue;
        }

        let dest_path = dest.path.join(relative_path);
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if dest_path.is_symlink() {
            anyhow::bail!(
                "Destination path is a symlink, refusing to follow: {}",
                dest_path.display()
            );
        }
        if dest_path.exists() {
            let msg = format!("Overwriting existing file: {}", dest_path.display());
            warnings.push(msg.clone());
            eprintln!("{}", msg);
        }

        let (tx, rx) = bounded::<ChunkMessage>(CHANNEL_BOUND);
        senders.push((idx, tx));

        let handle = std::thread::spawn(move || -> anyhow::Result<String> {
            let mut file = retry_io(|| File::create(&dest_path))
                .map_err(|e| anyhow::anyhow!("Create {}: {}", dest_path.display(), e))?;
            let mut hasher = algorithm.new_hasher();

            for msg in rx {
                match msg {
                    ChunkMessage::Data(bytes) => {
                        retry_io(|| file.write_all(&bytes))?;
                        hasher.update(&bytes);
                    }
                    ChunkMessage::End => break,
                }
            }
            if sync_writes {
                retry_io(|| file.sync_data())?;
            }
            Ok(hasher.finalize_hex())
        });
        handles.push((idx, handle));
    }

    if senders.is_empty() {
        // All destinations skipped, nothing to copy
        return Ok(result);
    }

    let mut src_file = retry_io(|| File::open(src_path))
        .map_err(|e| anyhow::anyhow!("Open source {}: {}", src_path.display(), e))?;

    let mut buf = vec![0u8; CHUNK_SIZE];
    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            for (_, sender) in &senders {
                let _ = sender.send(ChunkMessage::End);
            }
            return Err(anyhow::anyhow!("Cancelled by user"));
        }

        let n = match retry_io(|| src_file.read(&mut buf)) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                for (_, sender) in &senders {
                    let _ = sender.send(ChunkMessage::End);
                }
                return Err(anyhow::anyhow!("Read error {}: {}", src_path.display(), e));
            }
        };

        let chunk = ChunkMessage::Data(buf[..n].to_vec());
        // Gracefully handle individual destination failures
        let mut i = 0;
        while i < senders.len() {
            let (dest_idx, ref sender) = senders[i];
            if sender.send(chunk.clone()).is_ok() {
                i += 1;
            } else {
                // This destination disconnected - mark as failed and continue
                result[dest_idx] = FileCopyStatus::Failed("Destination writer disconnected".into());
                warnings.push(format!(
                    "Destination {} disconnected during copy of {}",
                    dest_idx, relative_path
                ));
                // Remove handle for this destination
                if let Some(pos) = handles.iter().position(|(idx, _)| *idx == dest_idx) {
                    handles.swap_remove(pos);
                }
                senders.swap_remove(i);
            }
        }
    }

    // Send End to remaining active senders
    for (_, sender) in &senders {
        let _ = sender.send(ChunkMessage::End);
    }

    // Collect results from remaining handles (indexed by dest_idx)
    for (dest_idx, handle) in handles {
        match handle.join() {
            Ok(Ok(hash)) => result[dest_idx] = FileCopyStatus::Copied(hash),
            Ok(Err(e)) => {
                result[dest_idx] = FileCopyStatus::Failed(format!("Writer error: {}", e));
                warnings.push(format!("Destination {} writer error: {}", dest_idx, e));
            }
            Err(_) => {
                result[dest_idx] = FileCopyStatus::Failed("Writer thread panicked".into());
                warnings.push(format!("Destination {} writer thread panicked", dest_idx));
            }
        }
    }

    Ok(result)
}

fn verify_file(
    dest_path: &Path,
    expected_hash: &str,
    algorithm: ChecksumAlgo,
    buf: &mut [u8],
) -> anyhow::Result<()> {
    let mut file = retry_io(|| File::open(dest_path))?;
    let mut hasher = algorithm.new_hasher();

    loop {
        let n = retry_io(|| file.read(buf))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let actual = hasher.finalize_hex();
    if actual != expected_hash {
        anyhow::bail!(
            "{} mismatch\n  expected: {}\n  actual:   {}",
            algorithm.as_str(),
            expected_hash,
            actual
        );
    }
    Ok(())
}

#[inline]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_source_ignores_hidden_directories_when_enabled() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join(".hidden_dir")).unwrap();
        std::fs::write(temp.path().join(".hidden_dir").join("clip.mxf"), b"hidden").unwrap();
        std::fs::write(temp.path().join("visible.mxf"), b"visible").unwrap();

        let mut progress_calls = 0;
        let scan = scan_source(temp.path(), &OffloadOptions::default(), &mut |_, _| {
            progress_calls += 1
        })
        .unwrap();

        assert_eq!(scan.total_files, 1);
        assert_eq!(scan.files[0].relative_path, "visible.mxf");
        assert_eq!(progress_calls, 1);
    }

    #[test]
    fn scan_source_keeps_hidden_directories_when_disabled() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join(".hidden_dir")).unwrap();
        std::fs::write(temp.path().join(".hidden_dir").join("clip.mxf"), b"hidden").unwrap();

        let options = OffloadOptions {
            ignore_hidden_system: false,
            ..OffloadOptions::default()
        };
        let scan = scan_source(temp.path(), &options, &mut |_, _| {}).unwrap();

        assert_eq!(scan.total_files, 1);
        assert_eq!(scan.files[0].relative_path, ".hidden_dir/clip.mxf");
    }

    #[test]
    fn is_hidden_dotfile() {
        use std::path::Path;
        assert!(is_hidden_or_system(Path::new(".hidden")));
        assert!(!is_hidden_or_system(Path::new("visible")));
    }

    #[test]
    #[cfg(windows)]
    fn is_hidden_system_folders() {
        use std::path::Path;
        assert!(is_hidden_or_system(Path::new("$RECYCLE.BIN/something")));
        assert!(is_hidden_or_system(Path::new(
            "System Volume Information/something"
        )));
    }

    #[test]
    fn verify_file_matching_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        let data = b"hello world test data for verification\n";
        std::fs::write(&path, data).unwrap();

        let hash = blake3::hash(data).to_hex().to_string();
        let mut buf = vec![0u8; CHUNK_SIZE];
        assert!(verify_file(&path, &hash, ChecksumAlgo::Blake3, &mut buf).is_ok());
    }

    #[test]
    fn verify_file_mismatched_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"correct data").unwrap();

        let wrong_hash = blake3::hash(b"different data").to_hex().to_string();
        let mut buf = vec![0u8; CHUNK_SIZE];
        assert!(verify_file(&path, &wrong_hash, ChecksumAlgo::Blake3, &mut buf).is_err());
    }

    #[test]
    fn verify_file_works_with_md5() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        let data = b"abc";
        std::fs::write(&path, data).unwrap();

        let mut buf = vec![0u8; CHUNK_SIZE];
        let md5_abc = "900150983cd24fb0d6963f7d28e17f72";
        assert!(verify_file(&path, md5_abc, ChecksumAlgo::Md5, &mut buf).is_ok());
        assert!(verify_file(&path, "deadbeef", ChecksumAlgo::Md5, &mut buf).is_err());
    }

    #[test]
    fn retry_io_succeeds_after_transient_interrupted() {
        use std::cell::Cell;
        let attempts = Cell::new(0u32);
        let result: io::Result<u32> = retry_io(|| {
            let n = attempts.get();
            attempts.set(n + 1);
            if n < 2 {
                Err(io::Error::new(io::ErrorKind::Interrupted, "try again"))
            } else {
                Ok(42)
            }
        });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.get(), 3);
    }

    #[test]
    fn retry_io_does_not_retry_non_transient() {
        use std::cell::Cell;
        let attempts = Cell::new(0u32);
        let result: io::Result<()> = retry_io(|| {
            attempts.set(attempts.get() + 1);
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "nope"))
        });
        assert!(result.is_err());
        assert_eq!(attempts.get(), 1);
    }

    #[test]
    fn retry_io_gives_up_after_max_attempts() {
        use std::cell::Cell;
        let attempts = Cell::new(0u32);
        let result: io::Result<()> = retry_io(|| {
            attempts.set(attempts.get() + 1);
            Err(io::Error::new(io::ErrorKind::TimedOut, "still down"))
        });
        assert!(result.is_err());
        assert_eq!(attempts.get(), IO_RETRY_ATTEMPTS);
    }

    #[test]
    fn transient_kinds_are_marked_transient() {
        for k in [
            io::ErrorKind::Interrupted,
            io::ErrorKind::WouldBlock,
            io::ErrorKind::TimedOut,
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::ConnectionAborted,
            io::ErrorKind::BrokenPipe,
        ] {
            assert!(is_transient_io_error(k), "{:?} should be transient", k);
        }
        for k in [
            io::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists,
            io::ErrorKind::InvalidInput,
        ] {
            assert!(!is_transient_io_error(k), "{:?} should not be transient", k);
        }
    }
}
