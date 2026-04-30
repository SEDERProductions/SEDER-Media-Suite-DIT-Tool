use super::*;
use std::fs;
use tempfile::tempdir;

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn recursively_scans_files_and_folders() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("card/a.mov"), "aaa");
    let scan = scan_folder(
        dir.path(),
        &ScanOptions {
            ignore_hidden_system: true,
            ignore_patterns: vec![],
            checksum: false,
        },
    )
    .unwrap();
    assert!(scan.files.contains_key("card/a.mov"));
    assert!(scan.folders.contains("card"));
}

#[test]
fn detects_nested_relative_path_matches() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("A001/clip.mov"), "same");
    write(&b.path().join("A001/clip.mov"), "same");
    let report = compare_folders(a.path(), b.path(), CompareMode::PathSize, true, vec![]).unwrap();
    assert_eq!(report.rows[0].relative_path, "A001/clip.mov");
    assert_eq!(report.rows[0].status, FileStatus::Matching);
}

#[test]
fn detects_files_only_in_a_and_b() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("only-a.mov"), "a");
    write(&b.path().join("only-b.mov"), "b");
    let report = compare_folders(a.path(), b.path(), CompareMode::PathSize, true, vec![]).unwrap();
    assert!(report
        .rows
        .iter()
        .any(|row| row.status == FileStatus::OnlyInA));
    assert!(report
        .rows
        .iter()
        .any(|row| row.status == FileStatus::OnlyInB));
}

#[test]
fn detects_changed_and_matching_files() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("changed.mov"), "a");
    write(&b.path().join("changed.mov"), "bb");
    write(&a.path().join("same.mov"), "ok");
    write(&b.path().join("same.mov"), "ok");
    let report = compare_folders(a.path(), b.path(), CompareMode::PathSize, true, vec![]).unwrap();
    assert!(report
        .rows
        .iter()
        .any(|row| row.relative_path == "changed.mov" && row.status == FileStatus::Changed));
    assert!(report
        .rows
        .iter()
        .any(|row| row.relative_path == "same.mov" && row.status == FileStatus::Matching));
}

#[test]
fn ignores_system_files() {
    let dir = tempdir().unwrap();
    write(&dir.path().join(".DS_Store"), "hidden");
    write(&dir.path().join("clip.mov"), "clip");
    let scan = scan_folder(
        dir.path(),
        &ScanOptions {
            ignore_hidden_system: true,
            ignore_patterns: vec![],
            checksum: false,
        },
    )
    .unwrap();
    assert!(!scan.files.contains_key(".DS_Store"));
    assert!(scan.files.contains_key("clip.mov"));
}

#[test]
fn checksum_comparison_detects_same_size_changes() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("clip.mov"), "abcd");
    write(&b.path().join("clip.mov"), "wxyz");
    let report = compare_folders(
        a.path(),
        b.path(),
        CompareMode::PathSizeChecksum,
        true,
        vec![],
    )
    .unwrap();
    assert_eq!(report.rows[0].status, FileStatus::Changed);
    assert!(report.rows[0].checksum_a.is_some());
    assert!(report.rows[0].xxh64_a.is_some());
}

#[test]
fn exports_reports() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("clip.mov"), "abcd");
    write(&b.path().join("clip.mov"), "abcd");
    let report = compare_folders(
        a.path(),
        b.path(),
        CompareMode::PathSizeChecksum,
        true,
        vec![],
    )
    .unwrap();
    assert!(report_txt(&report, "Folder Compare").contains("Matching"));
    assert!(report_csv(&report).contains("clip.mov"));

    let dit = DitReport {
        metadata: DitMetadata {
            project_name: "Project".into(),
            shoot_date: "2026-04-29".into(),
            card_name: "A001".into(),
            camera_id: "A".into(),
            source_path: a.path().display().to_string(),
            destination_path: b.path().display().to_string(),
            checksum_method: ChecksumMethod::Blake3,
        },
        comparison: report,
        timestamp: "now".into(),
        compare_mode: CompareMode::PathSizeChecksum,
    };
    assert!(dit_txt(&dit).contains("Offload Report"));
    assert!(dit_csv(&dit).contains("project_name"));
    assert!(dit_mhl(&dit).contains("<xxh64"));
    assert!(!dit_mhl(&dit).contains("blake3"));
    assert!(!dit_mhl(&dit).contains("BLAKE3"));
}

#[test]
fn dit_mhl_exports_destination_xxh64_without_blake3() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("A001/clip.mov"), "abcd");
    write(&b.path().join("A001/clip.mov"), "abcd");
    let report = create_dit_report(
        a.path(),
        b.path(),
        DitMetadata {
            project_name: "Project".into(),
            shoot_date: "2026-04-29".into(),
            card_name: "A001".into(),
            camera_id: "A".into(),
            source_path: a.path().display().to_string(),
            destination_path: b.path().display().to_string(),
            checksum_method: ChecksumMethod::Blake3,
        },
        true,
    )
    .unwrap();

    let row = report
        .comparison
        .rows
        .iter()
        .find(|row| row.relative_path == "A001/clip.mov")
        .unwrap();
    let mhl = dit_mhl(&report);

    assert_eq!(pass_fail(&report.comparison), "PASS");
    assert!(row.checksum_b.as_ref().unwrap().len() > row.xxh64_b.as_ref().unwrap().len());
    assert!(mhl.contains("<hashlist xmlns=\"urn:ASC:MHL:v2.0\" version=\"2.0\">"));
    assert!(mhl.contains("<process>transfer</process>"));
    assert!(mhl.contains("<path size=\"4\">A001/clip.mov</path>"));
    assert!(mhl.contains(&format!(">{}</xxh64>", row.xxh64_b.as_ref().unwrap())));
    assert!(!mhl.contains(row.checksum_b.as_ref().unwrap()));
    assert!(!mhl.contains("<blake3>"));
    assert!(!mhl.contains("BLAKE3"));
}

#[test]
fn dit_mhl_escapes_xml_values_and_keeps_failed_status() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("A&B/clip \"one\".mov"), "abcd");
    write(&b.path().join("A&B/clip \"one\".mov"), "wxyz");
    let report = create_dit_report(
        a.path(),
        b.path(),
        DitMetadata {
            project_name: "Project & Show".into(),
            shoot_date: "2026-04-29".into(),
            card_name: "A<001>".into(),
            camera_id: "A\"Cam\"".into(),
            source_path: a.path().display().to_string(),
            destination_path: b.path().display().to_string(),
            checksum_method: ChecksumMethod::Blake3,
        },
        true,
    )
    .unwrap();
    let mhl = dit_mhl(&report);

    assert_eq!(pass_fail(&report.comparison), "FAIL");
    assert!(mhl.contains("Project: Project &amp; Show"));
    assert!(mhl.contains("card: A&lt;001&gt;"));
    assert!(mhl.contains("camera: A&quot;Cam&quot;"));
    assert!(mhl.contains("A&amp;B/clip &quot;one&quot;.mov"));
}

#[test]
fn creates_mode_aware_dit_reports_without_checksums() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("A001/clip.mov"), "same");
    write(&b.path().join("A001/clip.mov"), "same");

    let report = create_dit_report_with_mode(
        a.path(),
        b.path(),
        DitMetadata {
            project_name: "Project".into(),
            shoot_date: "2026-04-29".into(),
            card_name: "A001".into(),
            camera_id: "A".into(),
            source_path: a.path().display().to_string(),
            destination_path: b.path().display().to_string(),
            checksum_method: ChecksumMethod::Blake3,
        },
        CompareMode::PathSize,
        true,
        vec![],
    )
    .unwrap();

    assert_eq!(report.compare_mode, CompareMode::PathSize);
    assert_eq!(pass_fail(&report.comparison), "PASS");
    assert!(report.comparison.rows[0].checksum_a.is_none());
    assert!(dit_txt(&report).contains("Compare mode: Path + Size"));
    assert!(dit_txt(&report).contains("Checksum method: Not used"));
}

#[test]
fn parses_comma_and_newline_ignore_patterns() {
    assert_eq!(
        parse_ignore_patterns(".DS_Store, Thumbs.db\n.proxy\r\n"),
        vec![".DS_Store", "Thumbs.db", ".proxy"]
    );
}

#[test]
fn progress_reports_scan_and_completion_phases() {
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    write(&a.path().join("A001/clip.mov"), "same");
    write(&b.path().join("A001/clip.mov"), "same");
    let mut phases = Vec::new();

    let report = create_dit_report_with_progress(
        a.path(),
        b.path(),
        DitMetadata {
            project_name: "Project".into(),
            shoot_date: "2026-04-29".into(),
            card_name: "A001".into(),
            camera_id: "A".into(),
            source_path: a.path().display().to_string(),
            destination_path: b.path().display().to_string(),
            checksum_method: ChecksumMethod::Blake3,
        },
        CompareMode::PathSizeChecksum,
        true,
        vec![],
        &mut |update| phases.push(update.phase),
    )
    .unwrap();

    assert_eq!(pass_fail(&report.comparison), "PASS");
    assert!(phases.contains(&"scan_a"));
    assert!(phases.contains(&"scan_b"));
    assert!(phases.contains(&"complete"));
}
