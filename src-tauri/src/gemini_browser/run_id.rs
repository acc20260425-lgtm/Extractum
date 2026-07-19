use super::domain_error::{GeminiBrowserError, GeminiBrowserResult};

pub(crate) fn safe_run_id(run_id: &str) -> GeminiBrowserResult<String> {
    let candidate = run_id.trim();
    if candidate.is_empty() {
        return Err(GeminiBrowserError::validation("run_id cannot be empty"));
    }
    if candidate
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        Ok(candidate.to_string())
    } else {
        Err(GeminiBrowserError::validation(
            "run_id can only contain ASCII letters, numbers, dashes, and underscores",
        ))
    }
}
