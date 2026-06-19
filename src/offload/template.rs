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
    // Iterate over chars (not bytes) so non-ASCII template characters
    // (e.g. accented project names) round-trip through the engine
    // without being mangled by `u8 as char` truncation.
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Collect the token up to the matching '}'. Unknown or
            // unterminated tokens are left in place verbatim.
            let mut token = String::new();
            let mut closed = false;
            for next in chars.by_ref() {
                if next == '}' {
                    closed = true;
                    break;
                }
                token.push(next);
            }
            if closed {
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
                    continue;
                }
                // Unknown token: emit the original `{token}` literally so
                // the user can see what was misspelled.
                out.push('{');
                out.push_str(&token);
                out.push('}');
                continue;
            }
            // Unterminated `{` — emit the literal so the user sees it
            // and we don't silently swallow input.
            out.push('{');
            out.push_str(&token);
        } else {
            out.push(ch);
        }
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

    #[test]
    fn non_ascii_template_characters_round_trip() {
        // The old implementation indexed `template.as_bytes()` and pushed
        // each byte through `u8 as char`, which truncates multi-byte
        // UTF-8 sequences (e.g. `é` became `Ã©`). Verify the new char
        // iterator preserves them.
        assert_eq!(expand("café/{project}", &meta()), "café/Mountain Film");
    }

    #[test]
    fn non_ascii_metadata_round_trips() {
        // Token values can also be non-ASCII (project name in any
        // language). The substitution must not mangle them.
        let mut m = meta();
        m.project_name = "東京オリンピック".into();
        assert_eq!(
            expand("{project}/{card}", &m),
            "東京オリンピック/A001"
        );
    }

    #[test]
    fn unterminated_brace_is_preserved_literally() {
        // A `{` with no closing `}` should be visible to the user, not
        // silently dropped.
        assert_eq!(expand("{project}/{unfinished", &meta()), "Mountain Film/{unfinished");
    }
}
