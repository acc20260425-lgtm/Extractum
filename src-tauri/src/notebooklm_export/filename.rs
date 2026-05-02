use std::path::{Path, PathBuf};

const RESERVED_WINDOWS_NAMES: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

pub(crate) fn sanitize_path_component(input: &str, fallback: &str) -> String {
    let mut output = String::with_capacity(input.len().min(96));
    let mut last_was_separator = false;

    for ch in input.trim().chars() {
        let normalized = if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            Some(ch.to_ascii_lowercase())
        } else {
            Some('_')
        };

        if let Some(ch) = normalized {
            if ch == '_' {
                if last_was_separator {
                    continue;
                }
                last_was_separator = true;
            } else {
                last_was_separator = false;
            }
            output.push(ch);
        }

        if output.len() >= 96 {
            break;
        }
    }

    let trimmed = output.trim_matches(['_', '.', ' ']).to_string();
    let candidate = if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed
    };

    if is_rejected_component(&candidate) {
        fallback.to_string()
    } else {
        candidate
    }
}

pub(crate) fn is_rejected_component(component: &str) -> bool {
    let trimmed = component.trim();
    trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains(['/', '\\'])
        || RESERVED_WINDOWS_NAMES
            .iter()
            .any(|reserved| trimmed.eq_ignore_ascii_case(reserved))
}

pub(crate) fn ensure_child_path(base: &Path, component: &str) -> Option<PathBuf> {
    if is_rejected_component(component) {
        return None;
    }

    let path = base.join(component);
    path.starts_with(base).then_some(path)
}

#[cfg(test)]
mod tests {
    use super::{ensure_child_path, is_rejected_component, sanitize_path_component};
    use std::path::Path;

    #[test]
    fn sanitizes_unsafe_filename_parts() {
        assert_eq!(
            sanitize_path_component(" My: Source / Name ", "source"),
            "my_source_name"
        );
        assert_eq!(sanitize_path_component("..", "source"), "source");
        assert_eq!(sanitize_path_component("CON", "source"), "source");
        assert_eq!(
            sanitize_path_component("Тестовый источник", "source"),
            "source"
        );
    }

    #[test]
    fn rejects_reserved_components() {
        assert!(is_rejected_component(".."));
        assert!(is_rejected_component("a/b"));
        assert!(is_rejected_component("nul"));
    }

    #[test]
    fn child_paths_stay_under_base() {
        let base = Path::new("export");
        assert!(ensure_child_path(base, "safe.md").is_some());
        assert!(ensure_child_path(base, "../nope").is_none());
    }
}
