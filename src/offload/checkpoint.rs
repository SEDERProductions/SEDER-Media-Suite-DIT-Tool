//! Crash-recovery checkpoint format.
//!
//! Records enough state to let the UI ask "you were partway through
//! offloading <source> to <destinations> last time — resume?" after a
//! crash, power loss, or forced quit. The format is JSON for easy
//! debugging and a single source-of-truth file at a well-known path
//! under the app's data directory.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Checkpoint {
    /// ISO-8601 timestamp of when the checkpoint was last written.
    pub updated_at: String,
    /// Absolute path to the source folder.
    pub source: String,
    /// Absolute destination paths the user had configured.
    pub destinations: Vec<String>,
    /// Relative paths that have already been verified.
    pub completed: Vec<String>,
    /// Total files in the scan, for progress reconstruction.
    pub total_files: u64,
    /// Project metadata at the time of checkpoint.
    pub project_name: String,
    pub shoot_date: String,
    pub card_name: String,
    pub camera_id: String,
}

/// Default checkpoint file location: <state_dir>/checkpoint.json
pub fn default_path(state_dir: &Path) -> PathBuf {
    state_dir.join("checkpoint.json")
}

pub fn save(state_dir: &Path, cp: &Checkpoint) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(state_dir)
        .map_err(|e| anyhow::anyhow!("create state dir {}: {}", state_dir.display(), e))?;
    let path = default_path(state_dir);
    let json = serde_json::to_string_pretty(cp)
        .map_err(|e| anyhow::anyhow!("serialize checkpoint: {}", e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| anyhow::anyhow!("write checkpoint tmp: {}", e))?;
    std::fs::rename(&tmp, &path).map_err(|e| anyhow::anyhow!("rename checkpoint: {}", e))?;
    Ok(path)
}

pub fn load(state_dir: &Path) -> Option<Checkpoint> {
    let path = default_path(state_dir);
    let contents = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&contents).ok()
}

pub fn clear(state_dir: &Path) -> anyhow::Result<()> {
    let path = default_path(state_dir);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(anyhow::anyhow!("remove checkpoint: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cp() -> Checkpoint {
        Checkpoint {
            updated_at: "2026-05-20 10:00:00".into(),
            source: "/source".into(),
            destinations: vec!["/dest1".into(), "/dest2".into()],
            completed: vec!["A001/clip001.r3d".into()],
            total_files: 42,
            project_name: "Project".into(),
            shoot_date: "2026-05-20".into(),
            card_name: "A001".into(),
            camera_id: "CAM-01".into(),
        }
    }

    #[test]
    fn roundtrip_via_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        save(dir.path(), &cp()).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert_eq!(loaded, cp());
    }

    #[test]
    fn load_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(dir.path()).is_none());
    }

    #[test]
    fn clear_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        clear(dir.path()).unwrap();
        save(dir.path(), &cp()).unwrap();
        clear(dir.path()).unwrap();
        assert!(load(dir.path()).is_none());
        clear(dir.path()).unwrap();
    }

    #[test]
    fn save_uses_atomic_rename_via_tmp() {
        let dir = tempfile::tempdir().unwrap();
        save(dir.path(), &cp()).unwrap();
        assert!(default_path(dir.path()).exists());
        // The tmp should have been renamed away.
        assert!(!default_path(dir.path()).with_extension("json.tmp").exists());
    }
}
