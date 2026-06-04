use crate::offload::ffprobe;
use crate::offload::hash::ChecksumAlgo;
use crate::offload::media::{classify, MediaKind};
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
    let mut ignored_paths: Vec<String> = Vec::new();
    let mut total_size = 0u64;
    let mut total_files = 0u64;
    let mut buf = vec![0u8; CHUNK_SIZE];

    // Discover ffprobe once per scan; if not available, treat
    // extract_metadata as a no-op for this run.
    let ffprobe_path = if options.extract_metadata {
        ffprobe::discover()
    } else {
        None
    };

    let ignore_glob = build_ignore_glob(&options.ignore_patterns);

    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let relative = path.strip_prefix(source)?;
        let rel_str = relative.to_string_lossy().replace('\\', "/");

        if options.ignore_hidden_system && is_hidden_or_system(path) {
            ignored_paths.push(rel_str);
            continue;
        }
        if let Some(ref gs) = ignore_glob {
            let basename = Path::new(&rel_str)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&rel_str);
            if gs.is_match(rel_str.as_str()) || gs.is_match(basename) {
                ignored_paths.push(rel_str);
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

        // Best-effort metadata probe for recognised media kinds.
        let metadata = if let Some(ref ffprobe_bin) = ffprobe_path {
            let kind = classify(&rel_str);
            if matches!(
                kind,
                MediaKind::R3d
                    | MediaKind::Arri
                    | MediaKind::Braw
                    | MediaKind::CanonRaw
                    | MediaKind::CinemaDng
                    | MediaKind::Mxf
                    | MediaKind::Mov
                    | MediaKind::Mp4
                    | MediaKind::MpegTs
                    | MediaKind::Audio
            ) {
                ffprobe::probe(path, ffprobe_bin).ok()
            } else {
                None
            }
        } else {
            None
        };

        files.push(FileEntry {
            relative_path: rel_str,
            size,
            source_hash: hash,
            algorithm: options.algorithm,
            metadata,
        });

        progress(total_files, total_size);
    }

    Ok(SourceScan {
        files,
        total_size,
        total_files,
        ignored_paths,
    })
}

/// Build the glob ignore set from a list of patterns. Returns `None` when
/// there are no patterns (or none compile). Shared by `scan_source` and
/// `walk_media` so both honour the exact same ignore semantics.
fn build_ignore_glob(patterns: &[String]) -> Option<globset::GlobSet> {
    let mut builder = GlobSetBuilder::new();
    let mut added = false;
    for p in patterns {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            if let Ok(glob) = Glob::new(trimmed) {
                builder.add(glob);
                added = true;
            }
        }
    }
    if !added {
        return None;
    }
    // build() only fails on pathological inputs; degrade to no-ignore rather
    // than panic (release builds use panic=abort, so a panic would crash).
    builder.build().ok()
}

/// A single media file discovered by [`walk_media`], described without any
/// hashing or I/O beyond a `stat`. Drives the pre-offload media browser,
/// which needs a fast listing rather than integrity checksums.
#[derive(Debug, Clone)]
pub struct MediaListEntry {
    pub relative_path: String,
    pub absolute_path: String,
    pub size: u64,
    pub kind: MediaKind,
}

/// Walk `source` applying the same hidden/system + glob-ignore rules as
/// [`scan_source`], but WITHOUT opening or hashing files. Invokes `visit`
/// once per kept file. This is the cheap counterpart used to populate the
/// thumbnail/media browser before (or instead of) a full verified offload.
pub fn walk_media(
    source: &Path,
    options: &OffloadOptions,
    mut visit: impl FnMut(MediaListEntry),
) -> anyhow::Result<()> {
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

    let ignore_glob = build_ignore_glob(&options.ignore_patterns);

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
        let kind = classify(&rel_str);
        visit(MediaListEntry {
            relative_path: rel_str,
            absolute_path: path.to_string_lossy().replace('\\', "/"),
            size,
            kind,
        });
    }

    Ok(())
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

        let mut copy_file_status: Vec<FileTransferStatus> =
            vec![FileTransferStatus::None; destinations.len()];
        let mut verify_file_status: Vec<FileTransferStatus> =
            vec![FileTransferStatus::None; destinations.len()];
        let mut verification_performed_for_file = false;

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
                                        verify_file_status[idx] = FileTransferStatus::Verified;
                                        verification_performed_for_file = true;
                                    }
                                    Err(e) => {
                                        results[idx].files_failed += 1;
                                        results[idx].final_error = Some(format!(
                                            "{}: verify failed - {}",
                                            file_entry.relative_path, e
                                        ));
                                        verify_file_status[idx] = FileTransferStatus::Failed;
                                        verification_performed_for_file = true;
                                    }
                                }
                            } else {
                                copy_file_status[idx] = FileTransferStatus::Copied;
                            }
                        }
                        FileCopyStatus::Skipped => {
                            results[idx].files_skipped += 1;
                            results[idx].state = DestinationState::Copying;
                            copy_file_status[idx] = FileTransferStatus::Skipped;
                        }
                        FileCopyStatus::Failed(err) => {
                            results[idx].files_failed += 1;
                            results[idx].state = DestinationState::Failed;
                            if results[idx].final_error.is_none() {
                                results[idx].final_error =
                                    Some(format!("{}: {}", file_entry.relative_path, err));
                            }
                            copy_file_status[idx] = FileTransferStatus::Failed;
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
                    copy_file_status[idx] = FileTransferStatus::Failed;
                }
            }
        }

        overall_bytes_completed += file_entry.size;

        let copy_dest_progress: Vec<DestinationProgress> = results
            .iter()
            .enumerate()
            .map(|(i, r)| DestinationProgress {
                index: i,
                state: r.state,
                // Copy-phase progress counts terminal copy outcomes only.
                files_completed: r.files_copied + r.files_skipped + r.files_failed,
                files_total: overall_files_total,
                bytes_completed: r.bytes_copied,
                bytes_total: overall_bytes_total,
                current_file: file_entry.relative_path.clone(),
                last_file_status: copy_file_status[i],
                error: r.final_error.clone(),
            })
            .collect();

        progress(OffloadProgress {
            phase: "copying".into(),
            overall_files_completed,
            overall_files_total,
            overall_bytes_completed,
            overall_bytes_total,
            current_file: file_entry.relative_path.clone(),
            destinations: copy_dest_progress,
            warnings: warnings.clone(),
        });

        if verify && verification_performed_for_file {
            let verify_dest_progress: Vec<DestinationProgress> = results
                .iter()
                .enumerate()
                .map(|(i, r)| DestinationProgress {
                    index: i,
                    state: r.state,
                    // Verify-phase progress should count only verify outcomes and failed copies
                    // to avoid double-counting copied+verified for the same source file.
                    files_completed: r.files_verified + r.files_skipped + r.files_failed,
                    files_total: overall_files_total,
                    bytes_completed: r.bytes_copied,
                    bytes_total: overall_bytes_total,
                    current_file: file_entry.relative_path.clone(),
                    last_file_status: verify_file_status[i],
                    error: r.final_error.clone(),
                })
                .collect();

            progress(OffloadProgress {
                phase: "verifying".into(),
                overall_files_completed,
                overall_files_total,
                overall_bytes_completed,
                overall_bytes_total,
                current_file: file_entry.relative_path.clone(),
                destinations: verify_dest_progress,
                warnings: warnings.clone(),
            });
        }
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
            if let Err(e) = std::fs::create_dir_all(parent) {
                result[idx] =
                    FileCopyStatus::Failed(format!("Create directory {}: {}", parent.display(), e));
                warnings.push(format!(
                    "Destination {} failed to create {}: {}",
                    idx + 1,
                    parent.display(),
                    e
                ));
                continue;
            }
        }

        if dest_path.is_symlink() {
            let msg = format!(
                "Destination path is a symlink, refusing to follow: {}",
                dest_path.display()
            );
            result[idx] = FileCopyStatus::Failed(msg.clone());
            warnings.push(format!("Destination {}: {}", idx + 1, msg));
            continue;
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
        // No active writers (all destinations either skipped or failed during setup)
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
                    dest_idx + 1,
                    relative_path
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
                warnings.push(format!("Destination {} writer error: {}", dest_idx + 1, e));
            }
            Err(_) => {
                result[dest_idx] = FileCopyStatus::Failed("Writer thread panicked".into());
                warnings.push(format!(
                    "Destination {} writer thread panicked",
                    dest_idx + 1
                ));
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
    fn walk_media_lists_without_hashing_and_classifies() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("A001.mxf"), b"video").unwrap();
        std::fs::write(temp.path().join("notes.wav"), b"audio").unwrap();
        std::fs::create_dir(temp.path().join(".hidden")).unwrap();
        std::fs::write(temp.path().join(".hidden").join("h.mxf"), b"x").unwrap();

        let mut found: Vec<(String, MediaKind, u64)> = Vec::new();
        walk_media(temp.path(), &OffloadOptions::default(), |e| {
            found.push((e.relative_path, e.kind, e.size));
        })
        .unwrap();

        // Hidden dir excluded by default; two visible files remain.
        assert_eq!(found.len(), 2);
        assert!(found
            .iter()
            .any(|(p, k, _)| p == "A001.mxf" && *k == MediaKind::Mxf));
        assert!(found
            .iter()
            .any(|(p, k, _)| p == "notes.wav" && *k == MediaKind::Audio));
        assert!(found.iter().all(|(p, _, _)| p != ".hidden/h.mxf"));
    }

    #[test]
    fn walk_media_respects_glob_ignore() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("keep.mxf"), b"a").unwrap();
        std::fs::write(temp.path().join("drop.wav"), b"b").unwrap();

        let options = OffloadOptions {
            ignore_patterns: vec!["*.wav".to_string()],
            ..OffloadOptions::default()
        };
        let mut paths: Vec<String> = Vec::new();
        walk_media(temp.path(), &options, |e| paths.push(e.relative_path)).unwrap();

        assert_eq!(paths, vec!["keep.mxf".to_string()]);
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

    #[test]
    fn offload_files_emits_copying_then_verifying_when_verify_enabled() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("clip.mxf"), b"abc123").unwrap();

        let scan = scan_source(src.path(), &OffloadOptions::default(), &mut |_, _| {}).unwrap();
        let destinations = vec![DestinationConfig {
            path: dst.path().to_path_buf(),
            label: Some("A".into()),
        }];
        let cancel = AtomicBool::new(false);
        let mut warnings = Vec::new();
        let mut phases = Vec::new();

        offload_files(
            src.path(),
            &scan,
            &destinations,
            true,
            &cancel,
            &mut |p| phases.push(p.phase),
            false,
            false,
            &mut warnings,
        )
        .unwrap();

        assert_eq!(phases, vec!["copying".to_string(), "verifying".to_string()]);
    }

    #[test]
    fn offload_files_progress_files_completed_never_exceeds_total_for_copy_and_verify() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("a.mxf"), b"aaa").unwrap();
        std::fs::write(src.path().join("b.mxf"), b"bbb").unwrap();

        let scan = scan_source(src.path(), &OffloadOptions::default(), &mut |_, _| {}).unwrap();
        let destinations = vec![DestinationConfig {
            path: dst.path().to_path_buf(),
            label: Some("A".into()),
        }];
        let cancel = AtomicBool::new(false);
        let mut warnings = Vec::new();
        let mut progress_events = Vec::new();

        offload_files(
            src.path(),
            &scan,
            &destinations,
            true,
            &cancel,
            &mut |p| progress_events.push(p),
            false,
            false,
            &mut warnings,
        )
        .unwrap();

        assert!(progress_events.iter().any(|p| p.phase == "copying"));
        assert!(progress_events.iter().any(|p| p.phase == "verifying"));

        for event in progress_events {
            for dest in event.destinations {
                assert!(
                    dest.files_completed <= dest.files_total,
                    "phase={}, completed={}, total={}",
                    event.phase,
                    dest.files_completed,
                    dest.files_total
                );
            }
        }
    }

    #[test]
    fn offload_files_does_not_emit_verifying_for_skipped_files() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("clip.mxf"), b"abc123").unwrap();
        std::fs::write(dst.path().join("clip.mxf"), b"already-there").unwrap();

        let scan = scan_source(src.path(), &OffloadOptions::default(), &mut |_, _| {}).unwrap();
        let destinations = vec![DestinationConfig {
            path: dst.path().to_path_buf(),
            label: Some("A".into()),
        }];
        let cancel = AtomicBool::new(false);
        let mut warnings = Vec::new();
        let mut phases = Vec::new();

        offload_files(
            src.path(),
            &scan,
            &destinations,
            true,
            &cancel,
            &mut |p| phases.push(p.phase),
            false,
            true,
            &mut warnings,
        )
        .unwrap();

        assert_eq!(phases, vec!["copying".to_string()]);
    }
}
