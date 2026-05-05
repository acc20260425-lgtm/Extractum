use crate::error::{AppError, AppResult};

pub(super) fn parse_username(input: &str) -> String {
    let s = input.trim();
    if let Some(rest) = s.strip_prefix("https://t.me/") {
        return rest.split('/').next().unwrap_or(rest).to_string();
    }
    if let Some(rest) = s.strip_prefix("t.me/") {
        return rest.split('/').next().unwrap_or(rest).to_string();
    }
    s.trim_start_matches('@').to_string()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ManualTelegramSourceRef {
    Username(String),
    NumericId(i64),
}

fn unsupported_manual_source_ref_message(source_ref: &str) -> String {
    format!(
        "Unsupported manual Telegram source reference '{}'. Use @username or t.me/name for public sources. For private Telegram sources, add them from the account's dialogs.",
        source_ref
    )
}

fn unsupported_private_manual_source_ref_message(source_ref: &str) -> String {
    format!(
        "Unsupported private Telegram source reference '{}'. Private invite links and internal t.me/c links are not supported for manual add. Add this source from the account's dialogs instead.",
        source_ref
    )
}

pub(super) fn parse_supported_manual_telegram_source_ref(
    source_ref: &str,
) -> AppResult<ManualTelegramSourceRef> {
    let trimmed = source_ref.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(
            "Telegram source reference cannot be empty",
        ));
    }

    if let Ok(source_id) = trimmed.parse::<i64>() {
        return Ok(ManualTelegramSourceRef::NumericId(source_id));
    }

    if let Some(rest) = trimmed.strip_prefix('@') {
        let username = rest.trim();
        if username.is_empty() || username.contains('/') || username.starts_with('+') {
            return Err(AppError::validation(unsupported_manual_source_ref_message(
                source_ref,
            )));
        }
        return Ok(ManualTelegramSourceRef::Username(username.to_string()));
    }

    if let Some(rest) = trimmed
        .strip_prefix("https://t.me/")
        .or_else(|| trimmed.strip_prefix("http://t.me/"))
        .or_else(|| trimmed.strip_prefix("t.me/"))
    {
        let path = rest.trim_matches('/');
        let first_segment = path.split('/').next().unwrap_or(path).trim();
        if first_segment.is_empty() {
            return Err(AppError::validation(unsupported_manual_source_ref_message(
                source_ref,
            )));
        }
        if first_segment.eq_ignore_ascii_case("joinchat")
            || first_segment.eq_ignore_ascii_case("c")
            || first_segment.starts_with('+')
        {
            return Err(AppError::validation(
                unsupported_private_manual_source_ref_message(source_ref),
            ));
        }
        return Ok(ManualTelegramSourceRef::Username(first_segment.to_string()));
    }

    let username = parse_username(trimmed);
    if !username.is_empty()
        && !username.contains('/')
        && !username.starts_with('+')
        && !username.chars().all(|char| char.is_ascii_digit())
    {
        return Ok(ManualTelegramSourceRef::Username(username));
    }

    Err(AppError::validation(unsupported_manual_source_ref_message(
        source_ref,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppErrorKind;

    #[test]
    fn parse_username_accepts_username_and_t_me_links() {
        assert_eq!(parse_username("@example"), "example");
        assert_eq!(parse_username("t.me/example"), "example");
        assert_eq!(parse_username("https://t.me/example/42"), "example");
    }

    #[test]
    fn parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids() {
        assert_eq!(
            parse_supported_manual_telegram_source_ref("@example"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("t.me/example"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("https://t.me/example/42"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("12345"),
            Ok(ManualTelegramSourceRef::NumericId(12345))
        );
    }

    #[test]
    fn parse_supported_manual_telegram_source_ref_rejects_private_links() {
        for source_ref in [
            "https://t.me/+AAAAAE-example",
            "t.me/joinchat/AAAAAE-example",
            "https://t.me/c/12345/67",
        ] {
            let error = parse_supported_manual_telegram_source_ref(source_ref)
                .expect_err("private/manual ref should be rejected");
            assert!(error.message.contains("not supported for manual add"));
            assert!(error.message.contains("dialogs"));
        }
    }

    #[test]
    fn parse_supported_manual_telegram_source_ref_rejects_empty_refs_as_validation() {
        let error = parse_supported_manual_telegram_source_ref("   ")
            .expect_err("empty ref should be rejected");

        assert_eq!(error.kind, AppErrorKind::Validation);
    }
}
