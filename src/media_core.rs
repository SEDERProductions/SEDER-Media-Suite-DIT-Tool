use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use twox_hash::XxHash64;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumMethod {
    Blake3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareMode {
    PathSize,
    PathSizeModified,
    PathSizeChecksum,
}

impl CompareMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::PathSize => "Path + Size",
            Self::PathSizeModified => "Path + Size + Modified Time",
            Self::PathSizeChecksum => "Path + Size + Checksum",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChecksums {
    pub blake3: String,
    pub xxh64: String,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub relative_path: String,
    pub size: u64,
    pub modified: Option<u64>,
    pub checksums: Option<FileChecksums>,
}

#[derive(Debug, Clone)]
pub struct FolderEntry {
    pub relative_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub files: BTreeMap<String, FileEntry>,
    pub folders: BTreeSet<String>,
    // u64: max ~18 EB, adequate for any real storage device
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub ignore_hidden_system: bool,
    pub ignore_patterns: Vec<String>,
    pub checksum: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Matching,
    Changed,
    OnlyInA,
    OnlyInB,
}

#[derive(Debug, Clone)]
pub struct ComparisonRow {
    pub relative_path: String,
    pub status: FileStatus,
    pub size_a: Option<u64>,
    pub size_b: Option<u64>,
    pub checksum_a: Option<String>,
    pub checksum_b: Option<String>,
    pub xxh64_a: Option<String>,
    pub xxh64_b: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CompareReport {
    pub rows: Vec<ComparisonRow>,
    pub folders_only_in_a: Vec<String>,
    pub folders_only_in_b: Vec<String>,
    pub total_files: usize,
    pub total_folders: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub struct DitMetadata {
    pub project_name: String,
    pub shoot_date: String,
    pub card_name: String,
    pub camera_id: String,
    pub source_path: String,
    pub destination_path: String,
    pub checksum_method: ChecksumMethod,
}

#[derive(Debug, Clone)]
pub struct DitReport {
    pub metadata: DitMetadata,
    pub comparison: CompareReport,
    pub timestamp: String,
    pub compare_mode: CompareMode,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub phase: &'static str,
    pub processed_files: u64,
    pub processed_bytes: u64,
    pub status: String,
}

fn is_hidden_or_system(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            name.starts_with('.')
                || matches!(name, "Thumbs.db" | "desktop.ini")
                || matches!(name, ".DS_Store" | ".Spotlight-V100" | ".Trashes")
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone)]
struct IgnorePattern {
    raw: String,
    is_glob: bool,
}

#[derive(Debug, Clone, Default)]
struct IgnoreMatcher {
    patterns: Vec<IgnorePattern>,
}

impl IgnoreMatcher {
    fn new(patterns: &[String]) -> Self {
        Self {
            patterns: patterns
                .iter()
                .map(|raw| IgnorePattern {
                    is_glob: raw.contains('*') || raw.contains('?'),
                    raw: raw.clone(),
                })
                .collect(),
        }
    }

    fn matches(&self, path: &Path) -> bool {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let normalized = path.to_string_lossy().replace('\\', "/");
        self.patterns.iter().any(|pattern| {
            if pattern.is_glob {
                wildcard_match(&pattern.raw, name) || wildcard_match(&pattern.raw, &normalized)
            } else {
                name == pattern.raw
            }
        })
    }
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let text = text.chars().collect::<Vec<_>>();
    let mut previous = vec![false; text.len() + 1];
    previous[0] = true;

    for pattern_char in pattern {
        let mut next = vec![false; text.len() + 1];
        match pattern_char {
            '*' => {
                next[0] = previous[0];
                for index in 1..=text.len() {
                    next[index] = next[index - 1] || previous[index];
                }
            }
            '?' => {
                for (slot, value) in next.iter_mut().skip(1).zip(previous.iter()) {
                    *slot = *value;
                }
            }
            literal => {
                for index in 1..=text.len() {
                    next[index] = previous[index - 1] && literal == text[index - 1];
                }
            }
        }
        previous = next;
    }

    previous[text.len()]
}

fn should_ignore(path: &Path, options: &ScanOptions, matcher: &IgnoreMatcher) -> bool {
    (options.ignore_hidden_system && is_hidden_or_system(path)) || matcher.matches(path)
}

fn relative(root: &Path, path: &Path) -> Result<String> {
    let stripped = path.strip_prefix(root)?;
    let utf8 = stripped.to_str().with_context(|| {
        format!(
            "Path contains invalid UTF-8 and cannot be compared: {}",
            path.display()
        )
    })?;
    Ok(utf8.replace('\\', "/"))
}

pub fn checksum_file_set(path: &Path) -> Result<FileChecksums> {
    let mut blake3_hasher = blake3::Hasher::new();
    let mut xxh64_hasher = XxHash64::with_seed(0);
    let mut file =
        File::open(path).with_context(|| format!("Unable to read {}", path.display()))?;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        blake3_hasher.update(chunk);
        xxh64_hasher.write(chunk);
    }
    Ok(FileChecksums {
        blake3: blake3_hasher.finalize().to_hex().to_string(),
        xxh64: format!("{:016x}", xxh64_hasher.finish()),
    })
}

pub fn parse_ignore_patterns(value: &str) -> Vec<String> {
    value
        .split([',', '\n', '\r'])
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn scan_folder(root: &Path, options: &ScanOptions) -> Result<ScanResult> {
    let mut noop = |_update: ProgressUpdate| {};
    scan_folder_with_progress(root, options, "scan", &mut noop)
}

pub fn scan_folder_with_progress<F>(
    root: &Path,
    options: &ScanOptions,
    phase: &'static str,
    progress: &mut F,
) -> Result<ScanResult>
where
    F: FnMut(ProgressUpdate) + ?Sized,
{
    if !root.is_dir() {
        anyhow::bail!("Folder does not exist: {}", root.display());
    }
    let matcher = IgnoreMatcher::new(&options.ignore_patterns);
    let mut result = ScanResult::default();
    let mut processed_files = 0_u64;
    let mut processed_bytes = 0_u64;
    progress(ProgressUpdate {
        phase,
        processed_files,
        processed_bytes,
        status: format!("Scanning {}", root.display()),
    });
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| entry.depth() == 0 || !should_ignore(entry.path(), options, &matcher))
    {
        let entry = entry?;
        if entry.depth() == 0 || should_ignore(entry.path(), options, &matcher) {
            continue;
        }
        let rel = relative(root, entry.path())?;
        if entry.file_type().is_dir() {
            result.folders.insert(rel);
            continue;
        }
        // Treat regular files and symlinks as files; std::fs::metadata follows
        // symlinks so size and mtime come from the resolved target.
        if entry.file_type().is_file() || entry.file_type().is_symlink() {
            let metadata = std::fs::metadata(entry.path()).with_context(|| {
                format!(
                    "Unable to stat {} (broken symlink?)",
                    entry.path().display()
                )
            })?;
            let modified = metadata
                .modified()
                .with_context(|| format!("Cannot read mtime for {rel}"))?
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs());
            let size = metadata.len();
            if options.checksum {
                progress(ProgressUpdate {
                    phase,
                    processed_files,
                    processed_bytes,
                    status: format!("Checksumming {rel}"),
                });
            }
            let checksums = if options.checksum {
                Some(checksum_file_set(entry.path())?)
            } else {
                None
            };
            result.total_size += size;
            processed_files += 1;
            processed_bytes += size;
            result.files.insert(
                rel.clone(),
                FileEntry {
                    relative_path: rel,
                    size,
                    modified,
                    checksums,
                },
            );
            if processed_files.is_multiple_of(64) {
                progress(ProgressUpdate {
                    phase,
                    processed_files,
                    processed_bytes,
                    status: format!("Scanned {processed_files} files"),
                });
            }
        }
    }
    progress(ProgressUpdate {
        phase,
        processed_files,
        processed_bytes,
        status: format!("Scanned {processed_files} files in {}", root.display()),
    });
    Ok(result)
}

fn files_match(a: &FileEntry, b: &FileEntry, mode: CompareMode) -> bool {
    match mode {
        CompareMode::PathSize => a.size == b.size,
        CompareMode::PathSizeModified => a.size == b.size && a.modified == b.modified,
        CompareMode::PathSizeChecksum => {
            a.size == b.size
                && a.checksums.as_ref().map(|checksums| &checksums.blake3)
                    == b.checksums.as_ref().map(|checksums| &checksums.blake3)
        }
    }
}

pub fn compare_scans(a: &ScanResult, b: &ScanResult, mode: CompareMode) -> CompareReport {
    let mut keys = BTreeSet::new();
    keys.extend(a.files.keys().cloned());
    keys.extend(b.files.keys().cloned());
    let total_files = keys.len();
    let rows = keys
        .into_iter()
        .map(|key| {
            let left = a.files.get(&key);
            let right = b.files.get(&key);
            let status = match (left, right) {
                (Some(l), Some(r)) if files_match(l, r, mode) => FileStatus::Matching,
                (Some(_), Some(_)) => FileStatus::Changed,
                (Some(_), None) => FileStatus::OnlyInA,
                (None, Some(_)) => FileStatus::OnlyInB,
                (None, None) => {
                    unreachable!("key set is built from a ∪ b, at least one side exists")
                }
            };
            ComparisonRow {
                relative_path: key,
                status,
                size_a: left.map(|entry| entry.size),
                size_b: right.map(|entry| entry.size),
                checksum_a: left.and_then(|entry| {
                    entry
                        .checksums
                        .as_ref()
                        .map(|checksums| checksums.blake3.clone())
                }),
                checksum_b: right.and_then(|entry| {
                    entry
                        .checksums
                        .as_ref()
                        .map(|checksums| checksums.blake3.clone())
                }),
                xxh64_a: left.and_then(|entry| {
                    entry
                        .checksums
                        .as_ref()
                        .map(|checksums| checksums.xxh64.clone())
                }),
                xxh64_b: right.and_then(|entry| {
                    entry
                        .checksums
                        .as_ref()
                        .map(|checksums| checksums.xxh64.clone())
                }),
            }
        })
        .collect::<Vec<_>>();

    let mut unique_folders = a.folders.clone();
    unique_folders.extend(b.folders.iter().cloned());

    CompareReport {
        rows,
        folders_only_in_a: a.folders.difference(&b.folders).cloned().collect(),
        folders_only_in_b: b.folders.difference(&a.folders).cloned().collect(),
        total_files,
        total_folders: unique_folders.len(),
        total_size: a.total_size + b.total_size,
    }
}

pub fn compare_folders_with_progress<F>(
    a: &Path,
    b: &Path,
    mode: CompareMode,
    ignore_hidden_system: bool,
    ignore_patterns: Vec<String>,
    progress: &mut F,
) -> Result<CompareReport>
where
    F: FnMut(ProgressUpdate) + ?Sized,
{
    let checksum = mode == CompareMode::PathSizeChecksum;
    let options = ScanOptions {
        ignore_hidden_system,
        ignore_patterns,
        checksum,
    };
    let left = scan_folder_with_progress(a, &options, "scan_a", progress)?;
    let right = scan_folder_with_progress(b, &options, "scan_b", progress)?;
    progress(ProgressUpdate {
        phase: "compare",
        processed_files: (left.files.len() + right.files.len()) as u64,
        processed_bytes: left.total_size + right.total_size,
        status: "Building comparison report".into(),
    });
    let report = compare_scans(&left, &right, mode);
    progress(ProgressUpdate {
        phase: "complete",
        processed_files: report.total_files as u64,
        processed_bytes: report.total_size,
        status: "Comparison complete".into(),
    });
    Ok(report)
}

pub fn create_dit_report_with_progress<F>(
    source: &Path,
    destination: &Path,
    metadata: DitMetadata,
    mode: CompareMode,
    ignore_hidden_system: bool,
    ignore_patterns: Vec<String>,
    progress: &mut F,
) -> Result<DitReport>
where
    F: FnMut(ProgressUpdate) + ?Sized,
{
    let comparison = compare_folders_with_progress(
        source,
        destination,
        mode,
        ignore_hidden_system,
        ignore_patterns,
        progress,
    )?;
    Ok(DitReport {
        metadata,
        comparison,
        timestamp: current_timestamp(),
        compare_mode: mode,
    })
}

pub fn current_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    unix_to_utc_datetime(secs)
}

fn current_mhl_datetime() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    unix_to_utc_datetime(secs)
}

fn unix_to_utc_datetime(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let day_secs = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = day_secs / 3_600;
    let minute = (day_secs % 3_600) / 60;
    let second = day_secs % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u64, u64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m as u64, d as u64)
}

pub fn pass_fail(report: &CompareReport) -> &'static str {
    if report
        .rows
        .iter()
        .all(|row| row.status == FileStatus::Matching)
        && report.folders_only_in_a.is_empty()
        && report.folders_only_in_b.is_empty()
    {
        "PASS"
    } else {
        "FAIL"
    }
}

pub fn compare_summary(report: &CompareReport) -> (usize, usize, usize, usize) {
    let only_a = report
        .rows
        .iter()
        .filter(|row| row.status == FileStatus::OnlyInA)
        .count();
    let only_b = report
        .rows
        .iter()
        .filter(|row| row.status == FileStatus::OnlyInB)
        .count();
    let changed = report
        .rows
        .iter()
        .filter(|row| row.status == FileStatus::Changed)
        .count();
    let matching = report
        .rows
        .iter()
        .filter(|row| row.status == FileStatus::Matching)
        .count();
    (only_a, only_b, changed, matching)
}

pub fn report_txt(report: &CompareReport, title: &str) -> String {
    let (only_a, only_b, changed, matching) = compare_summary(report);
    let mut out = format!(
        "{title}\nGenerated: {}\nTotal files: {}\nTotal folders: {}\nTotal size: {}\nOnly in A: {}\nOnly in B: {}\nChanged: {}\nMatching: {}\n\n",
        current_timestamp(),
        report.total_files,
        report.total_folders,
        report.total_size,
        only_a,
        only_b,
        changed,
        matching
    );
    for row in &report.rows {
        out.push_str(&format!(
            "{:?}\t{}\t{:?}\t{:?}\n",
            row.status, row.relative_path, row.size_a, row.size_b
        ));
    }
    for folder in &report.folders_only_in_a {
        out.push_str(&format!("Folder only in A\t{folder}\n"));
    }
    for folder in &report.folders_only_in_b {
        out.push_str(&format!("Folder only in B\t{folder}\n"));
    }
    out
}

pub fn report_csv(report: &CompareReport) -> String {
    let mut out = String::from(
        "\"status\",\"relative_path\",\"size_a\",\"size_b\",\"checksum_a\",\"checksum_b\"\n",
    );
    for row in &report.rows {
        out.push_str(&format!(
            "\"{:?}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            row.status,
            row.relative_path.replace('"', "\"\""),
            row.size_a.map(|v| v.to_string()).unwrap_or_default(),
            row.size_b.map(|v| v.to_string()).unwrap_or_default(),
            row.checksum_a.clone().unwrap_or_default(),
            row.checksum_b.clone().unwrap_or_default()
        ));
    }
    for folder in &report.folders_only_in_a {
        out.push_str(&format!(
            "\"FolderOnlyInA\",\"{}\",\"\",\"\",\"\",\"\"\n",
            folder.replace('"', "\"\"")
        ));
    }
    for folder in &report.folders_only_in_b {
        out.push_str(&format!(
            "\"FolderOnlyInB\",\"{}\",\"\",\"\",\"\",\"\"\n",
            folder.replace('"', "\"\"")
        ));
    }
    out
}

pub fn dit_txt(report: &DitReport) -> String {
    let meta = &report.metadata;
    let checksum_line = if report.compare_mode == CompareMode::PathSizeChecksum {
        format!("Checksum method: {:?}", meta.checksum_method)
    } else {
        "Checksum method: Not used".to_string()
    };
    format!(
        "SEDER Media Suite DIT Tool Offload Report\nProject: {}\nShoot date: {}\nCard name: {}\nCamera ID: {}\nSource: {}\nDestination: {}\nCompare mode: {}\n{}\nStatus: {}\nTimestamp: {}\n\n{}",
        meta.project_name,
        meta.shoot_date,
        meta.card_name,
        meta.camera_id,
        meta.source_path,
        meta.destination_path,
        report.compare_mode.label(),
        checksum_line,
        pass_fail(&report.comparison),
        report.timestamp,
        report_txt(&report.comparison, "Verification Results")
    )
}

pub fn dit_csv(report: &DitReport) -> String {
    let meta = &report.metadata;
    let mut out = String::from("\"field\",\"value\"\n");
    for (field, value) in [
        ("project_name", meta.project_name.as_str()),
        ("shoot_date", meta.shoot_date.as_str()),
        ("card_name", meta.card_name.as_str()),
        ("camera_id", meta.camera_id.as_str()),
        ("source_path", meta.source_path.as_str()),
        ("destination_path", meta.destination_path.as_str()),
        ("compare_mode", report.compare_mode.label()),
        (
            "checksum_method",
            if report.compare_mode == CompareMode::PathSizeChecksum {
                "Blake3"
            } else {
                ""
            },
        ),
        ("status", pass_fail(&report.comparison)),
        ("timestamp", report.timestamp.as_str()),
    ] {
        out.push_str(&format!(
            "\"{}\",\"{}\"\n",
            field,
            value.replace('"', "\"\"")
        ));
    }
    out.push('\n');
    out.push_str(&report_csv(&report.comparison));
    out
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn dit_mhl(report: &DitReport) -> String {
    let meta = &report.metadata;
    let created_at = current_mhl_datetime();
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<hashlist xmlns=\"urn:ASC:MHL:v2.0\" version=\"2.0\">\n");
    out.push_str("  <creatorinfo>\n");
    out.push_str(&format!(
        "    <creationdate>{}</creationdate>\n",
        created_at
    ));
    out.push_str("    <hostname>localhost</hostname>\n");
    out.push_str("    <tool version=\"0.1.0\">SEDER Media Suite DIT Tool</tool>\n");
    out.push_str(&format!(
        "    <comment>{}</comment>\n",
        xml_escape(&format!(
            "Project: {}; shoot date: {}; card: {}; camera: {}; xxh64 written for ASC MHL compatibility.",
            meta.project_name, meta.shoot_date, meta.card_name, meta.camera_id
        ))
    ));
    out.push_str("  </creatorinfo>\n");
    out.push_str("  <processinfo>\n");
    out.push_str("    <process>transfer</process>\n");
    out.push_str("  </processinfo>\n");
    let rows = report
        .comparison
        .rows
        .iter()
        .filter(|row| row.status != FileStatus::OnlyInA && row.xxh64_b.is_some())
        .collect::<Vec<_>>();
    let has_hashes = !rows.is_empty();
    if has_hashes {
        out.push_str("  <hashes>\n");
    }
    for row in rows {
        let Some(xxh64) = &row.xxh64_b else {
            continue;
        };
        out.push_str("    <hash>\n");
        out.push_str(&format!(
            "      <path size=\"{}\">{}</path>\n",
            row.size_b.map(|size| size.to_string()).unwrap_or_default(),
            xml_escape(&row.relative_path)
        ));
        out.push_str(&format!(
            "      <xxh64 action=\"original\" hashdate=\"{}\">{}</xxh64>\n",
            created_at,
            xml_escape(xxh64)
        ));
        out.push_str("    </hash>\n");
    }
    if has_hashes {
        out.push_str("  </hashes>\n");
    }
    out.push_str("</hashlist>\n");
    out
}

#[cfg(test)]
mod tests;
