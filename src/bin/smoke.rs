// End-to-end smoke test for the SEDER DIT offload core.
//
// The Qt UI talks to the Rust core exclusively through the C ABI in
// `src/ffi.rs`; this binary drives the exact same Rust entrypoints that
// `seder_offload_start` calls (scan_source, offload_files, the report
// formatters, the template engine, the checkpoint store) so we can
// exercise the full pipeline in a non-graphical environment.
//
// Run with:  cargo run --bin smoke --release

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use seder_dit_tool::offload::engine::{offload_files, scan_source};
use seder_dit_tool::offload::*;
use seder_dit_tool::report::{
    report_ale, report_csv, report_metadata_json, report_mhl, report_txt,
};

fn main() -> anyhow::Result<()> {
    // Hand-rolled temp dir to avoid pulling `tempfile` as a runtime dep.
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let root = std::env::temp_dir().join(format!("seder-dit-smoke-{nonce}"));
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir_all(&root)?;

    let src = root.join("source");
    let dst1 = root.join("dest1");
    let dst2 = root.join("dest2");
    std::fs::create_dir_all(src.join("A001_C001"))?;
    std::fs::write(src.join("A001_C001/clip001.mxf"), vec![0u8; 256 * 1024])?;
    std::fs::write(src.join("A001_C001/clip002.mxf"), vec![1u8; 512 * 1024])?;
    std::fs::write(src.join("notes.xml"), b"<clip>meta</clip>")?;
    std::fs::write(src.join(".DS_Store"), b"macos-junk")?;
    std::fs::create_dir_all(&dst1)?;
    std::fs::create_dir_all(&dst2)?;

    println!("=== SEDER DIT smoke test (Rust entrypoints Qt calls via FFI) ===");
    println!("tmp root : {}", root.display());
    println!("source   : {}", src.display());
    println!("dest1    : {}", dst1.display());
    println!("dest2    : {}", dst2.display());

    // ---- 1) Scan --------------------------------------------------------
    let scan = scan_source(
        &src,
        &OffloadOptions {
            ignore_hidden_system: true,
            ignore_patterns: vec!["*.tmp".into()],
            verify_after_copy: true,
            sync_writes: false,
            skip_existing: false,
            generate_report: true,
            algorithm: ChecksumAlgo::Blake3,
            extract_metadata: false,
        },
        &mut |files, bytes| {
            println!("  scan: files={files} bytes={bytes}");
        },
    )?;
    println!(
        "scan: {} files, {} bytes, ignored {} ({} hidden by walker filter at depth 1)",
        scan.total_files,
        scan.total_size,
        scan.ignored_paths.len(),
        // .DS_Store is filtered by the walker, not pushed to ignored_paths
        // — see engine::scan_source filter_entry branch.
        if scan.ignored_paths.is_empty() { 1 } else { 0 }
    );
    assert_eq!(scan.total_files, 3);

    // ---- 2) Offload -----------------------------------------------------
    let dests = vec![
        DestinationConfig {
            path: dst1.clone(),
            label: Some("Primary".into()),
        },
        DestinationConfig {
            path: dst2.clone(),
            label: Some("Backup".into()),
        },
    ];
    let cancel = Arc::new(AtomicBool::new(false));
    let mut warnings: Vec<String> = Vec::new();
    let mut last_phase = String::new();
    let mut progress_calls = 0u32;
    let t0 = Instant::now();
    let results = offload_files(
        &src,
        &scan,
        &dests,
        true,
        &cancel,
        &mut |p: OffloadProgress| {
            progress_calls += 1;
            if p.phase != last_phase {
                println!("  phase: {} -> {}", last_phase, p.phase);
                last_phase = p.phase.clone();
            }
        },
        false,
        false,
        &mut warnings,
    )?;
    let dt = t0.elapsed();
    println!(
        "offload: {} ms, {} progress events, {} warnings",
        dt.as_millis(),
        progress_calls,
        warnings.len()
    );

    for (i, r) in results.iter().enumerate() {
        println!(
            "  dest[{}] state={:?} copied={} verified={} skipped={} failed={} bytes={}",
            i,
            r.state,
            r.files_copied,
            r.files_verified,
            r.files_skipped,
            r.files_failed,
            r.bytes_copied
        );
        assert_eq!(r.state, DestinationState::Complete);
        assert_eq!(r.files_copied, 3);
        assert_eq!(r.files_verified, 3);
        assert_eq!(r.files_failed, 0);
    }

    // ---- 3) Build the report exactly the way ffi.rs does ---------------
    let report = OffloadReport {
        source_path: src.to_string_lossy().replace('\\', "/"),
        metadata: ProjectMetadata {
            project_name: "SmokeTest".into(),
            shoot_date: "2026-06-17".into(),
            card_name: "A001".into(),
            camera_id: "CAM-01".into(),
        },
        source_scan: scan.clone(),
        destination_results: results.clone(),
        timestamp: "2026-06-17 12:00:00".into(),
        verification_performed: true,
        warnings: warnings.clone(),
        checksum_verified: true,
    };
    let txt = report_txt(&report);
    let csv = report_csv(&report);
    let mhl = report_mhl(&report, 0).map_err(anyhow::Error::msg)?;
    let ale = report_ale(&report);
    let md_json = report_metadata_json(&report).map_err(anyhow::Error::msg)?;

    std::fs::write(root.join("report.txt"), &txt)?;
    std::fs::write(root.join("report.csv"), &csv)?;
    std::fs::write(root.join("report.mhl"), &mhl)?;
    std::fs::write(root.join("report.ale"), &ale)?;
    std::fs::write(root.join("report.metadata.json"), &md_json)?;

    println!("\n--- TXT report ({} bytes) ---", txt.len());
    println!("{txt}");
    println!("--- CSV report ({} bytes) ---", csv.len());
    println!("{csv}");
    println!("--- MHL v2.0 head ---");
    for line in mhl.lines().take(8) {
        println!("  {line}");
    }
    println!("  ... ({} lines total)", mhl.lines().count());
    println!("--- ALE head ---");
    for line in ale.lines().take(6) {
        println!("  {line}");
    }

    // ---- 4) Verify the destination trees are real on disk --------------
    println!("\n--- destination tree ---");
    for d in [&dst1, &dst2] {
        let d: &PathBuf = d;
        println!("{}:", d.display());
        let mut entries: Vec<PathBuf> = walkdir::WalkDir::new(d)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect();
        entries.sort();
        for p in entries {
            let rel = p.strip_prefix(d)?.to_string_lossy();
            let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            let expected = std::fs::read(src.join(&*rel))?;
            let actual = std::fs::read(&p)?;
            let same = expected == actual;
            println!("  {rel:<35} {size:>9} bytes  match={same}");
            assert!(same, "byte mismatch for {rel}");
        }
    }

    println!("\nALL OK");
    // Clean up our tmp tree on success.
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
