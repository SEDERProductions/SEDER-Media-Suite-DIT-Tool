/// Tiny path-template engine for destination folders.
///
/// Templates are arbitrary strings with `{token}` placeholders. Recognised
/// tokens (case-insensitive):
///
///   {project} {shoot_date} {date} {card} {card_name} {camera} {camera_id}
///
/// An unknown token is left in place verbatim so the user can see what
/// was misspelled. Forward slashes in the template create subdirectories;
/// after expansion we sanitize any sequence component that contains
/// filesystem-hostile characters by replacing them with underscores.
use crate::offload::ProjectMetadata;

const ILLEGAL_PER_COMPONENT: &[char] = &['<', '>', ':', '"', '\\', '|', '?', '*'];

/// Substitute tokens and sanitize each path component.
pub fn expand(template: &str, metadata: &ProjectMetadata) -> String {
    let raw = substitute(template, metadata);
    raw.split('/')
        .map(sanitize_component)
        .collect::<Vec<_>>()
        .join("/")
}

fn substitute(template: &str, metadata: &ProjectMetadata) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(close) = template[i..].find('}') {
                let token = &template[i + 1..i + close];
                let lower = token.to_ascii_lowercase();
                let replacement = match lower.as_str() {
                    "project" => Some(metadata.project_name.as_str()),
                    "shoot_date" | "date" => Some(metadata.shoot_date.as_str()),
                    "card" | "card_name" => Some(metadata.card_name.as_str()),
                    "camera" | "camera_id" => Some(metadata.camera_id.as_str()),
                    _ => None,
                };
                if let Some(value) = replacement {
                    out.push_str(value);
                    i += close + 1;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn sanitize_component(component: &str) -> String {
    let trimmed = component.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ILLEGAL_PER_COMPONENT.contains(&ch) || ch.is_control() {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    out
}

/// Convenience for previewing without producing a path: substitute only,
/// no sanitization. Used by the UI live-preview where the user benefits
/// from seeing exactly what they typed.
pub fn preview(template: &str, metadata: &ProjectMetadata) -> String {
    substitute(template, metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> ProjectMetadata {
        ProjectMetadata {
            project_name: "Mountain Film".into(),
            shoot_date: "2026-05-04".into(),
            card_name: "A001".into(),
            camera_id: "CAM-01".into(),
        }
    }

    #[test]
    fn substitutes_known_tokens() {
        assert_eq!(
            expand("{project}/{shoot_date}/{card}", &meta()),
            "Mountain Film/2026-05-04/A001"
        );
    }

    #[test]
    fn tokens_are_case_insensitive() {
        assert_eq!(
            expand("{Project}/{SHOOT_DATE}/{Card_Name}", &meta()),
            "Mountain Film/2026-05-04/A001"
        );
    }

    #[test]
    fn unknown_tokens_pass_through_verbatim() {
        assert_eq!(
            expand("{project}/{location}/{card}", &meta()),
            "Mountain Film/{location}/A001"
        );
    }

    #[test]
    fn aliases_resolve() {
        assert_eq!(expand("{date}", &meta()), "2026-05-04");
        assert_eq!(expand("{card_name}", &meta()), "A001");
        assert_eq!(expand("{camera_id}", &meta()), "CAM-01");
        assert_eq!(expand("{camera}", &meta()), "CAM-01");
    }

    #[test]
    fn sanitizes_illegal_filesystem_chars_per_component() {
        let mut m = meta();
        m.project_name = "Show: \"Pilot\" <2026>".into();
        assert_eq!(expand("{project}/{card}", &m), "Show_ _Pilot_ _2026_/A001");
    }

    #[test]
    fn empty_components_collapse_to_empty_strings_but_preserve_slashes() {
        let m = ProjectMetadata {
            project_name: "".into(),
            shoot_date: "2026-05-04".into(),
            card_name: "A001".into(),
            camera_id: "".into(),
        };
        assert_eq!(expand("{project}/{date}/{card}", &m), "/2026-05-04/A001");
    }

    #[test]
    fn preview_does_not_sanitize() {
        let mut m = meta();
        m.project_name = "Show: Pilot".into();
        assert_eq!(preview("{project}/{card}", &m), "Show: Pilot/A001");
    }

    #[test]
    fn no_tokens_means_no_change() {
        assert_eq!(expand("dailies/raw", &meta()), "dailies/raw");
    }
}
