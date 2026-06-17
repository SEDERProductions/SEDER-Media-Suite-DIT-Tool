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
            "{},{},{},{},{},{},{},{},{}\n",
            csv_field(dest.config.label.as_deref().unwrap_or("")),
            csv_field(&dest.config.path.display().to_string()),
            csv_field(verification_mode),
            csv_field(status),
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

/// Emit an Avid Log Exchange (.ale) sidecar listing each clip. Columns
/// follow the minimal Avid convention: Name, Tape, Start, FPS, Duration.
/// Tape is derived from the project's card name when available, falling
/// back to the file's stem. Frame rate and duration use the metadata
/// populated by ffprobe; rows without metadata still appear with
/// blanks so the offload can be imported as a manifest.
pub fn report_ale(report: &OffloadReport) -> String {
    let mut out = String::new();
    out.push_str("Heading\n");
    out.push_str("FIELD_DELIM\tTABS\n");
    out.push_str("VIDEO_FORMAT\t1080\n");
    out.push_str("AUDIO_FORMAT\t48khz\n");
    out.push_str(&format!(
        "FPS\t{}\n",
        report
            .source_scan
            .files
            .iter()
            .find_map(|f| f.metadata.as_ref().map(|m| m.fps_display()))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "24".into())
    ));
    out.push('\n');

    out.push_str("Column\n");
    out.push_str("Name\tTape\tStart\tFPS\tDuration\n\n");

    out.push_str("Data\n");
    let tape = if !report.metadata.card_name.is_empty() {
        report.metadata.card_name.clone()
    } else {
        String::new()
    };
    for f in &report.source_scan.files {
        let name = std::path::Path::new(&f.relative_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&f.relative_path)
            .to_string();
        let start = f
            .metadata
            .as_ref()
            .and_then(|m| m.timecode.clone())
            .unwrap_or_default();
        let fps = f
            .metadata
            .as_ref()
            .map(|m| m.fps_display())
            .unwrap_or_default();
        let duration = f
            .metadata
            .as_ref()
            .map(|m| seconds_to_timecode(m.duration_seconds, m.fps_num, m.fps_den))
            .unwrap_or_default();
        let row_tape = if tape.is_empty() {
            name.clone()
        } else {
            tape.clone()
        };
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            ale_field(&name),
            ale_field(&row_tape),
            ale_field(&start),
            ale_field(&fps),
            ale_field(&duration),
        ));
    }
    out
}

fn ale_field(s: &str) -> String {
    // Avid ALE uses tabs as delimiters; sanitize any embedded tab/CR/LF
    // by replacing with single spaces so the table layout stays intact.
    s.chars()
        .map(|c| {
            if c == '\t' || c == '\n' || c == '\r' {
                ' '
            } else {
                c
            }
        })
        .collect()
}

fn seconds_to_timecode(seconds: f64, fps_num: u32, fps_den: u32) -> String {
    if seconds <= 0.0 || fps_num == 0 || fps_den == 0 {
        return String::new();
    }
    let fps = fps_num as f64 / fps_den as f64;
    let total_frames = (seconds * fps).round() as u64;
    let frames_per_second = fps.round() as u64;
    if frames_per_second == 0 {
        return String::new();
    }
    let frames = total_frames % frames_per_second;
    let total_seconds = total_frames / frames_per_second;
    let s = total_seconds % 60;
    let m = (total_seconds / 60) % 60;
    let h = total_seconds / 3600;
    format!("{:02}:{:02}:{:02}:{:02}", h, m, s, frames)
}

/// Emit a JSON sidecar describing each scanned file plus its ffprobe
/// metadata when present. Returns Err on serialization failure (rare).
pub fn report_metadata_json(report: &OffloadReport) -> Result<String, String> {
    let entries: Vec<serde_json::Value> = report
        .source_scan
        .files
        .iter()
        .map(|f| {
            let media_kind = crate::offload::media::classify(&f.relative_path);
            serde_json::json!({
                "path": f.relative_path,
                "size": f.size,
                "hash_algorithm": f.algorithm.as_str(),
                "hash": f.source_hash,
                "media_kind": media_kind.as_str(),
                "metadata": f.metadata,
            })
        })
        .collect();
    let doc = serde_json::json!({
        "generator": {
            "name": "SEDER DIT Tool",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "timestamp": report.timestamp,
        "source": report.source_path,
        "project": report.metadata.project_name,
        "shoot_date": report.metadata.shoot_date,
        "card": report.metadata.card_name,
        "camera": report.metadata.camera_id,
        "files": entries,
        "ignored_paths": report.source_scan.ignored_paths,
    });
    serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())
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
                        metadata: None,
                    },
                    FileEntry {
                        relative_path: "clip002.mxf".into(),
                        size: 2048 * 1024,
                        source_hash: "def456hash".into(),
                        algorithm: ChecksumAlgo::Blake3,
                        metadata: None,
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
        report.verification_performed = false;

        let csv = report_csv(&report);

        // Each field is quoted exactly once: the outer quotes come from
        // csv_field() alone, and embedded quotes are doubled per RFC 4180.
        // A value ending in a quote (e.g. `Backup "A"`) legitimately yields
        // a `"""` run (doubled quote + field-closing quote), so we assert
        // the exact escaped forms rather than a blanket "no triple quote".
        assert!(csv.contains("\"Backup \"\"A\"\"\""));
        assert!(csv.contains("\"/Volumes/BACKUP, 01\""));
        assert!(csv.contains("\"Copy-only (Unverified)\""));
        assert!(csv.contains("\"COPIED (UNVERIFIED)\""));
        assert!(csv.contains("\"bad \"\"checksum\"\"\""));
        // Guard against the old double-wrapping bug: a field's value should
        // never be wrapped in two layers of quotes (e.g. `""Verified""`).
        assert!(!csv.contains("\"\"Verified\"\""));
        assert!(!csv.contains("\"\"Copy-only"));
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

    #[test]
    fn metadata_json_includes_files_and_ignored_paths() {
        let mut report = make_test_report();
        report.source_scan.ignored_paths = vec![".DS_Store".into()];
        let json = report_metadata_json(&report).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["project"], "Test Project");
        assert_eq!(v["files"][0]["path"], "clip001.mxf");
        assert_eq!(v["files"][0]["hash_algorithm"], "BLAKE3");
        assert_eq!(v["files"][0]["media_kind"], "MXF");
        assert!(v["files"][0]["metadata"].is_null());
        assert_eq!(v["ignored_paths"][0], ".DS_Store");
    }

    #[test]
    fn ale_export_has_required_sections() {
        let report = make_test_report();
        let ale = report_ale(&report);
        assert!(ale.starts_with("Heading\n"));
        assert!(ale.contains("FIELD_DELIM\tTABS"));
        assert!(ale.contains("\nColumn\n"));
        assert!(ale.contains("Name\tTape\tStart\tFPS\tDuration"));
        assert!(ale.contains("\nData\n"));
        // Names come from file stems
        assert!(ale.contains("clip001"));
        assert!(ale.contains("clip002"));
        // Tape from project card_name
        assert!(ale.contains("A001"));
    }

    #[test]
    fn ale_uses_metadata_fps_and_duration_when_present() {
        use crate::offload::ClipMetadata;
        let mut report = make_test_report();
        report.source_scan.files[0].metadata = Some(ClipMetadata {
            video_codec: "prores".into(),
            width: 1920,
            height: 1080,
            fps_num: 24000,
            fps_den: 1001,
            duration_seconds: 4.0,
            audio_codec: String::new(),
            audio_channels: 0,
            audio_sample_rate: 0,
            timecode: Some("01:00:00:00".into()),
            color_space: String::new(),
        });
        let ale = report_ale(&report);
        assert!(ale.contains("23.976"));
        assert!(ale.contains("01:00:00:00"));
    }

    #[test]
    fn seconds_to_timecode_basic() {
        assert_eq!(super::seconds_to_timecode(0.0, 24, 1), "");
        assert_eq!(super::seconds_to_timecode(10.0, 24, 1), "00:00:10:00");
        assert_eq!(super::seconds_to_timecode(3661.0, 24, 1), "01:01:01:00");
        assert_eq!(super::seconds_to_timecode(1.0, 0, 1), "");
    }

    #[test]
    fn metadata_json_serializes_clip_metadata_when_present() {
        use crate::offload::ClipMetadata;
        let mut report = make_test_report();
        report.source_scan.files[0].metadata = Some(ClipMetadata {
            video_codec: "prores".into(),
            width: 1920,
            height: 1080,
            fps_num: 24000,
            fps_den: 1001,
            duration_seconds: 10.5,
            audio_codec: "pcm_s16le".into(),
            audio_channels: 2,
            audio_sample_rate: 48000,
            timecode: Some("01:00:00:00".into()),
            color_space: "bt709".into(),
        });
        let json = report_metadata_json(&report).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["files"][0]["metadata"]["video_codec"], "prores");
        assert_eq!(v["files"][0]["metadata"]["width"], 1920);
        assert_eq!(v["files"][0]["metadata"]["timecode"], "01:00:00:00");
    }
}
