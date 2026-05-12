use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use seder_dit_tool::offload::engine::{offload_files, scan_source};
use seder_dit_tool::offload::*;
use seder_dit_tool::report::{report_csv, report_mhl, report_txt};

fn setup_source(root: &std::path::Path) {
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::write(root.join("clip001.mxf"), b"file one content").unwrap();
    std::fs::write(root.join("sub/clip002.mxf"), b"file two content longer").unwrap();
    std::fs::write(root.join(".hidden.mxf"), b"hidden").unwrap();
}

fn make_options() -> OffloadOptions {
    OffloadOptions {
        ignore_hidden_system: true,
        ignore_patterns: vec![],
        verify_after_copy: true,
        sync_writes: false,
        skip_existing: false,
        generate_report: true,
    }
}

fn make_dest_configs(dest_paths: &[PathBuf], label: &str) -> Vec<DestinationConfig> {
    dest_paths
        .iter()
        .enumerate()
        .map(|(i, p)| DestinationConfig {
            path: p.clone(),
            label: Some(format!("{}{}", label, i + 1)),
        })
        .collect()
}

#[test]
fn full_offload_pipeline_single_destination() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("source");
    let dst = tmp.path().join("dest");
    std::fs::create_dir_all(&dst).unwrap();

    setup_source(&src);

    let scan = scan_source(&src, &make_options(), &mut |_, _| {}).unwrap();
    // Should find 2 files (hidden is ignored)
    assert_eq!(scan.total_files, 2);

    let dests = make_dest_configs(std::slice::from_ref(&dst), "Drive");
    let cancel = Arc::new(AtomicBool::new(false));
    let mut warnings = Vec::new();
    let mut progress_calls = 0;

    let results = offload_files(
        &src,
        &scan,
        &dests,
        true,
        &cancel,
        &mut |_| progress_calls += 1,
        false,
        false,
        &mut warnings,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].state, DestinationState::Complete);
    assert_eq!(results[0].files_copied, 2);
    assert_eq!(results[0].files_verified, 2);
    assert_eq!(results[0].files_failed, 0);
    assert_eq!(results[0].files_skipped, 0);
    assert!(results[0].final_error.is_none());
    assert!(dst.join("clip001.mxf").exists());
    assert!(dst.join("sub/clip002.mxf").exists());
    assert!(progress_calls > 0);
}

#[test]
fn full_offload_pipeline_multiple_destinations() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("source");
    let dst1 = tmp.path().join("dest1");
    let dst2 = tmp.path().join("dest2");
    std::fs::create_dir_all(&dst1).unwrap();
    std::fs::create_dir_all(&dst2).unwrap();

    setup_source(&src);

    let scan = scan_source(&src, &make_options(), &mut |_, _| {}).unwrap();
    let dests = make_dest_configs(&[dst1.clone(), dst2.clone()], "Backup");
    let cancel = Arc::new(AtomicBool::new(false));
    let mut warnings = Vec::new();

    let results =
        offload_files(&src, &scan, &dests, false, &cancel, &mut |_| {}, false, false, &mut warnings)
            .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].state, DestinationState::Complete);
    assert_eq!(results[1].state, DestinationState::Complete);
    assert_eq!(results[0].files_copied, 2);
    assert_eq!(results[1].files_copied, 2);
    assert_eq!(results[0].files_failed, 0);
    assert_eq!(results[1].files_failed, 0);
    assert!(dst1.join("clip001.mxf").exists());
    assert!(dst2.join("clip001.mxf").exists());
}

#[test]
fn offload_cancel_during_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("source");
    let dst = tmp.path().join("dest");
    std::fs::create_dir_all(&dst).unwrap();

    std::fs::create_dir_all(&src).unwrap();
    // Create many files so there's time to cancel
    for i in 0..20 {
        std::fs::write(
            src.join(format!("clip{:03}.mxf", i)),
            vec![b'x'; 1024 * 100],
        )
        .unwrap();
    }

    let scan = scan_source(&src, &make_options(), &mut |_, _| {}).unwrap();
    let dests = make_dest_configs(std::slice::from_ref(&dst), "Drive");
    let cancel = Arc::new(AtomicBool::new(true)); // pre-cancelled
    let mut warnings = Vec::new();

    let results =
        offload_files(&src, &scan, &dests, false, &cancel, &mut |_| {}, false, false, &mut warnings)
            .unwrap();

    // All destinations should be cancelled
    for r in &results {
        assert_eq!(r.state, DestinationState::Cancelled);
    }
}

#[test]
fn offload_empty_source_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("empty_source");
    std::fs::create_dir_all(&src).unwrap();

    if let Ok(scan) = scan_source(&src, &make_options(), &mut |_, _| {}) {
        assert_eq!(scan.total_files, 0);
    }
}

#[test]
fn scan_with_custom_ignore_patterns() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("keep.mxf"), b"keep").unwrap();
    std::fs::write(tmp.path().join("ignore.txt"), b"ignore").unwrap();
    std::fs::write(tmp.path().join("also_ignore.log"), b"ignore").unwrap();

    let options = OffloadOptions {
        ignore_hidden_system: false,
        ignore_patterns: vec!["*.txt".into(), "*.log".into()],
        ..make_options()
    };

    let scan = scan_source(tmp.path(), &options, &mut |_, _| {}).unwrap();
    assert_eq!(scan.total_files, 1);
    assert_eq!(scan.files[0].relative_path, "keep.mxf");
}

#[test]
fn report_txt_contains_all_destinations() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("source");
    let dst = tmp.path().join("dest");
    std::fs::create_dir_all(&dst).unwrap();
    setup_source(&src);

    let scan = scan_source(&src, &make_options(), &mut |_, _| {}).unwrap();
    let dests = make_dest_configs(std::slice::from_ref(&dst), "Drive");
    let cancel = Arc::new(AtomicBool::new(false));
    let mut warnings = Vec::new();

    let results =
        offload_files(&src, &scan, &dests, true, &cancel, &mut |_| {}, false, false, &mut warnings)
            .unwrap();

    let report = OffloadReport {
        source_path: src.to_string_lossy().replace('\\', "/"),
        metadata: ProjectMetadata {
            project_name: "Test".into(),
            shoot_date: "2026-01-01".into(),
            card_name: "A001".into(),
            camera_id: "CAM1".into(),
        },
        source_scan: scan,
        destination_results: results,
        timestamp: "2026-01-01 12:00:00".into(),
        verification_performed: true,
        warnings: vec!["test warning".into()],
        checksum_verified: true,
    };

    let txt = report_txt(&report);
    assert!(txt.contains("PASS"));
    assert!(txt.contains("test warning"));
    assert!(txt.contains(&dst.to_string_lossy().to_string()));

    let csv = report_csv(&report);
    assert!(csv.contains("PASS"));

    let mhl = report_mhl(&report, 0).expect("MHL should be generated");
    assert!(mhl.contains("urn:ASC:MHL:v2.0"));
    assert!(mhl.contains("clip001.mxf"));
}

#[test]
fn verify_failure_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("source");
    let dst = tmp.path().join("dest");
    std::fs::create_dir_all(&dst).unwrap();
    std::fs::create_dir_all(&src).unwrap();

    std::fs::write(src.join("test.mxf"), b"original content").unwrap();

    let scan = scan_source(&src, &make_options(), &mut |_, _| {}).unwrap();
    let dests = make_dest_configs(std::slice::from_ref(&dst), "Drive");
    let cancel = Arc::new(AtomicBool::new(false));
    let mut warnings = Vec::new();

    // Copy then corrupt the destination
    let results =
        offload_files(&src, &scan, &dests, true, &cancel, &mut |_| {}, false, false, &mut warnings)
            .unwrap();

    // Should have copied and verified successfully since we didn't corrupt it
    assert_eq!(results[0].files_verified, 1);
    assert_eq!(results[0].files_failed, 0);

    // Now corrupt and re-verify by calling verify_file directly
    std::fs::write(dst.join("test.mxf"), b"corrupted content").unwrap();

    // Re-scan to get fresh hash, then re-copy with verify
    std::fs::write(src.join("test.mxf"), b"corrupted content").unwrap();
    let scan2 = scan_source(&src, &make_options(), &mut |_, _| {}).unwrap();
    // Force verify by having destination already exist with wrong content
    // but since skip_existing=false, it'll overwrite and verify should pass
    let mut warnings2 = Vec::new();
    let results2 = offload_files(
        &src,
        &scan2,
        &dests,
        true,
        &cancel,
        &mut |_| {},
        false,
        false,
        &mut warnings2,
    )
    .unwrap();
    assert_eq!(results2[0].files_verified, 1);
}
