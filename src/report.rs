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
        "Files:     {}\n",
        report.source_scan.total_files
    ));
    out.push_str(&format!(
        "Size:      {}\n\n",
        format_bytes(report.source_scan.total_size)
    ));

    for (idx, dest) in report.destination_results.iter().enumerate() {
        out.push_str(&format!("Destination {}: {}\n", idx + 1, dest.config.path.display()));
        let status = match dest.state {
            DestinationState::Complete => "PASS",
            DestinationState::Failed => "FAIL",
            _ => "INCOMPLETE",
        };
        out.push_str(&format!("  Status:   {}\n", status));
        out.push_str(&format!("  Copied:   {}\n", dest.files_copied));
        out.push_str(&format!("  Verified: {}\n", dest.files_verified));
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
    out.push_str("destination,path,status,copied,verified,failed,error\n");
    for dest in &report.destination_results {
        let status = match dest.state {
            DestinationState::Complete => "PASS",
            DestinationState::Failed => "FAIL",
            _ => "INCOMPLETE",
        };
        let error = dest.final_error.as_deref().unwrap_or("");
        out.push_str(&format!(
            "\"{}\",\"{}\",{},{},{},{},\"{}\"\n",
            dest.config.label.as_deref().unwrap_or(""),
            dest.config.path.display(),
            status,
            dest.files_copied,
            dest.files_verified,
            dest.files_failed,
            error.replace('"', "\"\"")
        ));
    }
    out
}

pub fn report_mhl(report: &OffloadReport, destination_index: usize) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<hashlist version=\"2.0\" xmlns=\"urn:ASC:MHL:v2.0\">\n");
    out.push_str("  <generator>\n");
    out.push_str("    <name>SEDER DIT Tool</name>\n");
    out.push_str("    <version>0.0.1</version>\n");
    out.push_str(&format!("    <date>{}</date>\n", report.timestamp));
    out.push_str("  </generator>\n");
    out.push_str("  <process>transfer</process>\n");

    if report.destination_results.len() > destination_index {
        for file in &report.source_scan.files {
            out.push_str("  <hash>\n");
            out.push_str(&format!(
                "    <file>{}</file>\n",
                xml_escape(&file.relative_path)
            ));
            out.push_str(&format!("    <size>{}</size>\n", file.size));
            out.push_str("    <hashmethod>blake3</hashmethod>\n");
            out.push_str(&format!(
                "    <hashvalue>{}</hashvalue>\n",
                file.source_blake3
            ));
            out.push_str("  </hash>\n");
        }
    }

    out.push_str("</hashlist>\n");
    out
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
                        source_blake3: "abc123hash".into(),
                    },
                    FileEntry {
                        relative_path: "clip002.mxf".into(),
                        size: 2048 * 1024,
                        source_blake3: "def456hash".into(),
                    },
                ],
                total_size: 3 * 1024 * 1024,
                total_files: 2,
            },
            destination_results: vec![
                DestinationResult {
                    config: DestinationConfig {
                        path: PathBuf::from("/Volumes/BACKUP01"),
                        label: Some("Backup A".into()),
                    },
                    state: DestinationState::Complete,
                    files_copied: 2,
                    files_verified: 2,
                    files_failed: 0,
                    bytes_copied: 3 * 1024 * 1024,
                    final_error: None,
                },
            ],
            timestamp: "2026-05-04 12:00:00".into(),
            warnings: vec![],
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
        assert!(csv.starts_with("destination,path,status"));
    }

    #[test]
    fn report_mhl_contains_hash() {
        let report = make_test_report();
        let mhl = report_mhl(&report, 0);
        assert!(mhl.contains("abc123hash"));
        assert!(mhl.contains("urn:ASC:MHL:v2.0"));
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
