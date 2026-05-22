/// Classify common digital cinema and post-production media files.
///
/// The mapping is extension-based (case-insensitive). It is intentionally
/// conservative — anything we do not recognise falls into `MediaKind::Other`
/// rather than being guessed. Callers use the result to drive per-format
/// stats in reports and UI badges; nothing in the copy pipeline branches on
/// this value, so misclassification only affects display, not integrity.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKind {
    /// RED RAW (.r3d)
    R3d,
    /// ARRI RAW (.ari, .arx)
    Arri,
    /// Blackmagic RAW (.braw)
    Braw,
    /// Canon Cinema RAW Light (.crm)
    CanonRaw,
    /// Cinema DNG (.dng inside a CinemaDNG bundle)
    CinemaDng,
    /// Material Exchange Format (.mxf)
    Mxf,
    /// QuickTime / ProRes / DNxHR container (.mov)
    Mov,
    /// MPEG-4 container (.mp4, .m4v)
    Mp4,
    /// MPEG-2 Transport Stream (.mts, .m2ts, .ts)
    MpegTs,
    /// Audio (.wav, .bwf, .aif, .aiff, .flac, .mp3)
    Audio,
    /// Subtitle / caption sidecar (.srt, .ass, .vtt, .scc)
    Subtitle,
    /// XML / JSON / sidecar metadata (.xml, .json, .cdl, .ale, .edl, .ccc)
    Sidecar,
    /// Anything else.
    Other,
}

impl MediaKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MediaKind::R3d => "R3D",
            MediaKind::Arri => "ARRI",
            MediaKind::Braw => "BRAW",
            MediaKind::CanonRaw => "Canon RAW",
            MediaKind::CinemaDng => "CinemaDNG",
            MediaKind::Mxf => "MXF",
            MediaKind::Mov => "MOV",
            MediaKind::Mp4 => "MP4",
            MediaKind::MpegTs => "MPEG-TS",
            MediaKind::Audio => "Audio",
            MediaKind::Subtitle => "Subtitle",
            MediaKind::Sidecar => "Sidecar",
            MediaKind::Other => "Other",
        }
    }
}

/// Classify by file path. Looks only at the extension; lowercase comparison.
pub fn classify(path: &str) -> MediaKind {
    let ext = path
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "r3d" => MediaKind::R3d,
        "ari" | "arx" => MediaKind::Arri,
        "braw" => MediaKind::Braw,
        "crm" => MediaKind::CanonRaw,
        "dng" => MediaKind::CinemaDng,
        "mxf" => MediaKind::Mxf,
        "mov" => MediaKind::Mov,
        "mp4" | "m4v" => MediaKind::Mp4,
        "mts" | "m2ts" | "ts" => MediaKind::MpegTs,
        "wav" | "bwf" | "aif" | "aiff" | "flac" | "mp3" => MediaKind::Audio,
        "srt" | "ass" | "vtt" | "scc" => MediaKind::Subtitle,
        "xml" | "json" | "cdl" | "ale" | "edl" | "ccc" => MediaKind::Sidecar,
        _ => MediaKind::Other,
    }
}

#[derive(Debug, Default, Clone)]
pub struct FormatBreakdown {
    /// (kind, file count, total bytes), sorted descending by byte size.
    pub entries: Vec<(MediaKind, u64, u64)>,
}

impl FormatBreakdown {
    /// Aggregate a slice of (relative_path, size) pairs into a breakdown.
    pub fn from_files<I: IntoIterator<Item = (String, u64)>>(files: I) -> Self {
        use std::collections::HashMap;
        let mut counts: HashMap<MediaKind, (u64, u64)> = HashMap::new();
        for (path, size) in files {
            let kind = classify(&path);
            let entry = counts.entry(kind).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += size;
        }
        let mut entries: Vec<(MediaKind, u64, u64)> =
            counts.into_iter().map(|(k, (c, b))| (k, c, b)).collect();
        entries.sort_by(|a, b| b.2.cmp(&a.2).then(b.1.cmp(&a.1)));
        Self { entries }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_extensions() {
        assert_eq!(classify("A001_C001.R3D"), MediaKind::R3d);
        assert_eq!(classify("ALEX_001.ari"), MediaKind::Arri);
        assert_eq!(classify("clip.braw"), MediaKind::Braw);
        assert_eq!(classify("scene1.MXF"), MediaKind::Mxf);
        assert_eq!(classify("a.mov"), MediaKind::Mov);
        assert_eq!(classify("a.mp4"), MediaKind::Mp4);
        assert_eq!(classify("a.m4v"), MediaKind::Mp4);
        assert_eq!(classify("a.mts"), MediaKind::MpegTs);
        assert_eq!(classify("a.wav"), MediaKind::Audio);
        assert_eq!(classify("a.flac"), MediaKind::Audio);
        assert_eq!(classify("notes.xml"), MediaKind::Sidecar);
        assert_eq!(classify("notes.ALE"), MediaKind::Sidecar);
        assert_eq!(classify("captions.srt"), MediaKind::Subtitle);
    }

    #[test]
    fn unknown_or_extensionless_is_other() {
        assert_eq!(classify("README"), MediaKind::Other);
        assert_eq!(classify("file.unknownext"), MediaKind::Other);
        assert_eq!(classify(""), MediaKind::Other);
    }

    #[test]
    fn classification_is_case_insensitive() {
        assert_eq!(classify("X.R3D"), MediaKind::R3d);
        assert_eq!(classify("X.r3d"), MediaKind::R3d);
        assert_eq!(classify("X.MoV"), MediaKind::Mov);
    }

    #[test]
    fn breakdown_aggregates_and_sorts_by_bytes() {
        let files = vec![
            ("a.mxf".to_string(), 100),
            ("b.mxf".to_string(), 200),
            ("c.mov".to_string(), 500),
            ("notes.xml".to_string(), 10),
        ];
        let bd = FormatBreakdown::from_files(files);
        assert_eq!(bd.entries.len(), 3);
        assert_eq!(bd.entries[0].0, MediaKind::Mov);
        assert_eq!(bd.entries[0].1, 1);
        assert_eq!(bd.entries[0].2, 500);
        assert_eq!(bd.entries[1].0, MediaKind::Mxf);
        assert_eq!(bd.entries[1].1, 2);
        assert_eq!(bd.entries[1].2, 300);
        assert_eq!(bd.entries[2].0, MediaKind::Sidecar);
    }

    #[test]
    fn breakdown_empty_when_no_files() {
        let bd = FormatBreakdown::from_files(std::iter::empty::<(String, u64)>());
        assert!(bd.is_empty());
    }
}
