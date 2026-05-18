use crate::offload::media::FormatBreakdown;
use crate::offload::{DestinationState, OffloadReport};

pub fn report_txt(report: &OffloadReport) -> String {
    let mut out = String::new();
    out.push_str("SEDER DIT Offload Report\n");
    out.push_str("========================\n\n");
    out.push_str(&format!("Timestamp: {}\n", report.timestamp));
    out.push_str(&format!("Project:   {}\n", report.metadata.project_name));
    out.push_str(&format!("Date:      {}\n", report.metadata.shoot_date));
    out.push_str(&format!("Card:      {}\n", report.metadata.card_name));
    out.push_str(&format!("Camera:    {}\n", report.metadata.camera_id));
    out.push_str(&format!("Source:    {}\n", report.source_path));
    out.push_str(&format!(
        "Verification Mode: {}\n",
        if report.verification_performed {
            "Verified"
        } else {
            "Copy-only (Unverified)"
        }
    ));
    out.push_str(&format!("Files:     {}\n", report.source_scan.total_files));
    out.push_str(&format!(
        "Size:      {}\n",
        format_bytes(report.source_scan.total_size)
    ));

    let breakdown = FormatBreakdown::from_files(
        report
            .source_scan
            .files
            .iter()
            .map(|f| (f.relative_path.clone(), f.size)),
    );
    if !breakdown.is_empty() {
        out.push_str("\nFormat breakdown:\n");
        for (kind, count, bytes) in &breakdown.entries {
            out.push_str(&format!(
                "  {:<10} {:>6} file(s)   {}\n",
                kind.as_str(),
                count,
                format_bytes(*bytes)
            ));
        }
    }

    if !report.source_scan.ignored_paths.is_empty() {
        out.push_str(&format!(
            "\nIgnored:   {} file(s) skipped by ignore rules\n",
            report.source_scan.ignored_paths.len()
        ));
    }
    out.push('\n');

    for (idx, dest) in report.destination_results.iter().enumerate() {
        out.push_str(&format!(
            "Destination {}: {}\n",
            idx + 1,
            dest.config.path.display()
        ));
        let status = match dest.state {
            DestinationState::Complete if report.verification_performed => "PASS",
            DestinationState::Complete => "COPIED (UNVERIFIED)",
            DestinationState::Failed => "FAIL",
            DestinationState::Cancelled => "CANCELLED",
            _ => "INCOMPLETE",
        };
        out.push_str(&format!("  Status:   {}\n", status));
        out.push_str(&format!("  Copied:   {}\n", dest.files_copied));
        out.push_str(&format!("  Verified: {}\n", dest.files_verified));
        out.push_str(&format!("  Skipped:  {}\n", dest.files_skipped));
        out.push_str(&format!("  Failed:   {}\n", dest.files_failed));
        if let Some(ref err) = dest.final_error {
            out.push_str(&format!("  Error:    {}\n", err));
        }
        out.push('\n');
    }

    if !report.warnings.is_empty() {
        out.push_str("Warnings:\n");
        out.push_str("---------\n");
        for w in &report.warnings {
            out.push_str(&format!("  * {}\n", w));
        }
        out.push('\n');
    }

    out
}

pub fn report_csv(report: &OffloadReport) -> String {
    let mut out = String::new();
    out.push_str(
        "destination,path,verification_mode,status,copied,verified,skipped,failed,error\n",
    );
    for dest in &report.destination_results {
        let status = match dest.state {
            DestinationState::Complete if report.verification_performed => "PASS",
            DestinationState::Complete => "COPIED (UNVERIFIED)",
            DestinationState::Failed => "FAIL",
            DestinationState::Cancelled => "CANCELLED",
            _ => "INCOMPLETE",
        };
        let verification_mode = if report.verification_performed {
            "Verified"
        } else {
            "Copy-only (Unverified)"
        };
        let error = dest.final_error.as_deref().unwrap_or("");
        out.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",{},{},{},{},{},\"{}\"\n",
            csv_field(dest.config.label.as_deref().unwrap_or("")),
            csv_field(&dest.config.path.display().to_string()),
            verification_mode,
            status,
            dest.files_copied,
            dest.files_verified,
            dest.files_skipped,
            dest.files_failed,
            csv_field(error)
        ));
    }
    out
}

pub fn report_mhl(report: &OffloadReport, destination_index: usize) -> Result<String, String> {
    if !report.checksum_verified {
        return Err("MHL export requires checksum verification (Verify after copy).".into());
    }
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<hashlist version=\"2.0\" xmlns=\"urn:ASC:MHL:v2.0\">\n");
    out.push_str("  <creatorinfo>\n");
    out.push_str("    <tool>\n");
    out.push_str("      <name>SEDER DIT Tool</name>\n");
    out.push_str(&format!(
        "      <version>{}</version>\n",
        env!("CARGO_PKG_VERSION")
    ));
    out.push_str("    </tool>\n");
    out.push_str(&format!(
        "    <creationdate>{}</creationdate>\n",
        xml_escape(&report.timestamp)
    ));
    if !report.metadata.project_name.is_empty() {
        out.push_str(&format!(
            "    <project>{}</project>\n",
            xml_escape(&report.metadata.project_name)
        ));
    }
    if !report.metadata.shoot_date.is_empty() {
        out.push_str(&format!(
            "    <shootdate>{}</shootdate>\n",
            xml_escape(&report.metadata.shoot_date)
        ));
    }
    if !report.metadata.card_name.is_empty() {
        out.push_str(&format!(
            "    <cardname>{}</cardname>\n",
            xml_escape(&report.metadata.card_name)
        ));
    }
    if !report.metadata.camera_id.is_empty() {
        out.push_str(&format!(
            "    <camera>{}</camera>\n",
            xml_escape(&report.metadata.camera_id)
        ));
    }
    out.push_str("  </creatorinfo>\n");
    out.push_str("  <generator>\n");
    out.push_str("    <name>SEDER DIT Tool</name>\n");
    out.push_str(&format!(
        "    <version>{}</version>\n",
        env!("CARGO_PKG_VERSION")
    ));
    out.push_str(&format!("    <date>{}</date>\n", report.timestamp));
    out.push_str("  </generator>\n");
    out.push_str("  <process>transfer</process>\n");

    if report.destination_results.len() > destination_index {
        for file in &report.source_scan.files {
            let method = file.algorithm.mhl_element_name();
            out.push_str("  <hash>\n");
            out.push_str(&format!(
                "    <file>{}</file>\n",
                xml_escape(&file.relative_path)
            ));
            out.push_str(&format!("    <size>{}</size>\n", file.size));
            out.push_str(&format!("    <hashmethod>{}</hashmethod>\n", method));
            out.push_str(&format!(
                "    <hashvalue>{}</hashvalue>\n",
                file.source_hash
            ));
            out.push_str("  </hash>\n");
        }
    }

    if !report.source_scan.ignored_paths.is_empty() {
        out.push_str("  <ignored>\n");
        for path in &report.source_scan.ignored_paths {
            out.push_str(&format!("    <path>{}</path>\n", xml_escape(path)));
        }
        out.push_str("  </ignored>\n");
    }

    out.push_str("</hashlist>\n");
    Ok(out)
}

fn csv_field(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn format_bytes(value: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if value == 0 {
        return "0 B".into();
    }
    let exp = (value as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let unit = UNITS[exp];
    let scaled = value as f64 / 1024f64.powi(exp as i32);
    if exp == 0 {
        format!("{} {}", value, unit)
    } else {
        format!("{:.2} {}", scaled, unit)
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offload::*;
    use std::path::PathBuf;

    fn make_test_report() -> OffloadReport {
        OffloadReport {
            source_path: "/Volumes/CARD01".into(),
            metadata: ProjectMetadata {
                project_name: "Test Project".into(),
                shoot_date: "2026-05-04".into(),
                card_name: "A001".into(),
                camera_id: "CAM-001".into(),
            },
            source_scan: SourceScan {
                files: vec![
                    FileEntry {
                        relative_path: "clip001.mxf".into(),
                        size: 1024 * 1024,
                        source_hash: "abc123hash".into(),
                        algorithm: ChecksumAlgo::Blake3,
                    },
                    FileEntry {
                        relative_path: "clip002.mxf".into(),
                        size: 2048 * 1024,
                        source_hash: "def456hash".into(),
                        algorithm: ChecksumAlgo::Blake3,
                    },
                ],
                total_size: 3 * 1024 * 1024,
                total_files: 2,
                ignored_paths: vec![],
            },
            destination_results: vec![DestinationResult {
                config: DestinationConfig {
                    path: PathBuf::from("/Volumes/BACKUP01"),
                    label: Some("Backup A".into()),
                },
                state: DestinationState::Complete,
                files_copied: 2,
                files_verified: 2,
                files_failed: 0,
                files_skipped: 0,
                bytes_copied: 3 * 1024 * 1024,
                final_error: None,
            }],
            timestamp: "2026-05-04 12:00:00".into(),
            verification_performed: true,
            warnings: vec![],
            checksum_verified: true,
        }
    }

    #[test]
    fn report_txt_contains_source_path() {
        let report = make_test_report();
        let txt = report_txt(&report);
        assert!(txt.contains("/Volumes/CARD01"));
    }

    #[test]
    fn report_txt_contains_destination() {
        let report = make_test_report();
        let txt = report_txt(&report);
        assert!(txt.contains("/Volumes/BACKUP01"));
        assert!(txt.contains("PASS"));
    }

    #[test]
    fn report_txt_contains_warnings() {
        let mut report = make_test_report();
        report.warnings = vec!["Test warning".into()];
        let txt = report_txt(&report);
        assert!(txt.contains("Test warning"));
    }

    #[test]
    fn report_csv_has_header() {
        let report = make_test_report();
        let csv = report_csv(&report);
        assert!(csv.starts_with(
            "destination,path,verification_mode,status,copied,verified,skipped,failed,error"
        ));
    }

    #[test]
    fn report_csv_escapes_quoted_fields() {
        let mut report = make_test_report();
        report.destination_results[0].config.label = Some("Backup \"A\"".into());
        report.destination_results[0].config.path = PathBuf::from("/Volumes/BACKUP, 01");
        report.destination_results[0].final_error = Some("bad \"checksum\"".into());

        let csv = report_csv(&report);

        assert!(csv.contains("\"Backup \"\"A\"\"\""));
        assert!(csv.contains("\"/Volumes/BACKUP, 01\""));
        assert!(csv.contains("\"bad \"\"checksum\"\"\""));
    }

    #[test]
    fn report_mhl_contains_hash() {
        let report = make_test_report();
        let mhl = report_mhl(&report, 0).expect("mhl should be generated");
        assert!(mhl.contains("abc123hash"));
        assert!(mhl.contains("urn:ASC:MHL:v2.0"));
        assert!(mhl.contains("<hashmethod>blake3</hashmethod>"));
    }

    #[test]
    fn report_mhl_emits_creatorinfo_with_project_metadata() {
        let report = make_test_report();
        let mhl = report_mhl(&report, 0).expect("mhl should be generated");
        assert!(mhl.contains("<creatorinfo>"));
        assert!(mhl.contains("<name>SEDER DIT Tool</name>"));
        assert!(mhl.contains("<project>Test Project</project>"));
        assert!(mhl.contains("<shootdate>2026-05-04</shootdate>"));
        assert!(mhl.contains("<cardname>A001</cardname>"));
        assert!(mhl.contains("<camera>CAM-001</camera>"));
    }

    #[test]
    fn report_mhl_emits_ignored_block_when_files_were_skipped() {
        let mut report = make_test_report();
        report.source_scan.ignored_paths = vec![
            ".DS_Store".into(),
            "Thumbs.db".into(),
            "sub/<weird>.txt".into(),
        ];
        let mhl = report_mhl(&report, 0).expect("mhl should be generated");
        assert!(mhl.contains("<ignored>"));
        assert!(mhl.contains("<path>.DS_Store</path>"));
        assert!(mhl.contains("<path>Thumbs.db</path>"));
        // XML escape on the weird path
        assert!(mhl.contains("&lt;weird&gt;"));
    }

    #[test]
    fn report_mhl_omits_ignored_block_when_nothing_skipped() {
        let report = make_test_report();
        let mhl = report_mhl(&report, 0).expect("mhl should be generated");
        assert!(!mhl.contains("<ignored>"));
    }

    #[test]
    fn report_txt_shows_format_breakdown() {
        let report = make_test_report();
        let txt = report_txt(&report);
        assert!(txt.contains("Format breakdown:"));
        assert!(txt.contains("MXF"));
    }

    #[test]
    fn report_txt_shows_ignored_count() {
        let mut report = make_test_report();
        report.source_scan.ignored_paths = vec![".DS_Store".into(), "Thumbs.db".into()];
        let txt = report_txt(&report);
        assert!(txt.contains("Ignored:"));
        assert!(txt.contains("2 file"));
    }

    #[test]
    fn report_mhl_uses_per_file_algorithm() {
        let mut report = make_test_report();
        report.source_scan.files[0].algorithm = ChecksumAlgo::Md5;
        report.source_scan.files[0].source_hash = "900150983cd24fb0d6963f7d28e17f72".into();
        report.source_scan.files[1].algorithm = ChecksumAlgo::Xxh3_64;
        report.source_scan.files[1].source_hash = "abcdefabcdef0123".into();

        let mhl = report_mhl(&report, 0).expect("mhl should be generated");
        assert!(mhl.contains("<hashmethod>md5</hashmethod>"));
        assert!(mhl.contains("<hashmethod>xxh3</hashmethod>"));
        assert!(mhl.contains("900150983cd24fb0d6963f7d28e17f72"));
        assert!(mhl.contains("abcdefabcdef0123"));
    }

    #[test]
    fn format_bytes_values() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn xml_escape_chars() {
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
    }
}
