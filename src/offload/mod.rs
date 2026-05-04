use std::path::PathBuf;

pub mod engine;
pub mod progress;
pub mod verify;
pub mod volume;

#[derive(Debug, Clone)]
pub struct DestinationConfig {
    pub path: PathBuf,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub project_name: String,
    pub shoot_date: String,
    pub card_name: String,
    pub camera_id: String,
}

#[derive(Debug, Clone)]
pub struct OffloadOptions {
    pub ignore_hidden_system: bool,
    pub ignore_patterns: Vec<String>,
    pub verify_after_copy: bool,
}

impl Default for OffloadOptions {
    fn default() -> Self {
        Self {
            ignore_hidden_system: true,
            ignore_patterns: vec![],
            verify_after_copy: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OffloadRequest {
    pub source: PathBuf,
    pub destinations: Vec<DestinationConfig>,
    pub metadata: ProjectMetadata,
    pub options: OffloadOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationState {
    Pending = 0,
    Scanning = 1,
    Copying = 2,
    Verifying = 3,
    Complete = 4,
    Failed = 5,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub relative_path: String,
    pub size: u64,
    pub source_blake3: String,
}

#[derive(Debug, Clone)]
pub struct SourceScan {
    pub files: Vec<FileEntry>,
    pub total_size: u64,
    pub total_files: u64,
}

#[derive(Debug, Clone)]
pub struct DestinationResult {
    pub config: DestinationConfig,
    pub state: DestinationState,
    pub files_copied: u64,
    pub files_verified: u64,
    pub files_failed: u64,
    pub bytes_copied: u64,
    pub final_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DestinationProgress {
    pub index: usize,
    pub state: DestinationState,
    pub files_completed: u64,
    pub files_total: u64,
    pub bytes_completed: u64,
    pub bytes_total: u64,
    pub current_file: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OffloadProgress {
    pub phase: String,
    pub overall_files_completed: u64,
    pub overall_files_total: u64,
    pub overall_bytes_completed: u64,
    pub overall_bytes_total: u64,
    pub current_file: String,
    pub destinations: Vec<DestinationProgress>,
}

#[derive(Debug, Clone)]
pub struct OffloadReport {
    pub source_path: String,
    pub metadata: ProjectMetadata,
    pub source_scan: SourceScan,
    pub destination_results: Vec<DestinationResult>,
    pub timestamp: String,
    pub warnings: Vec<String>,
}
