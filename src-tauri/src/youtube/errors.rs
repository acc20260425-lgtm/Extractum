use crate::error::AppError;

pub(crate) fn invalid_youtube_url(reason: impl Into<String>) -> AppError {
    AppError::validation(format!("Invalid YouTube URL: {}", reason.into()))
}

pub(crate) fn classify_ytdlp_failure(stderr: &str) -> AppError {
    let message = concise_provider_message(stderr);
    let lower = message.to_ascii_lowercase();
    let prefixed = format!("YouTube preview failed: {message}");

    if lower.contains("private")
        || lower.contains("sign in")
        || lower.contains("log in")
        || lower.contains("login required")
        || lower.contains("authenticate")
        || lower.contains("cookies")
        || lower.contains("member")
        || lower.contains("age-restricted")
        || lower.contains("age restricted")
        || lower.contains("confirm your age")
        || lower.contains("geo-restricted")
        || lower.contains("geo restricted")
        || lower.contains("not available in your country")
        || lower.contains("not available in your region")
    {
        AppError::auth(prefixed)
    } else if lower.contains("unavailable")
        || lower.contains("deleted")
        || lower.contains("not found")
        || lower.contains("removed")
    {
        AppError::not_found(prefixed)
    } else if lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("http error")
        || lower.contains("rate limit")
        || lower.contains("too many requests")
    {
        AppError::network(prefixed)
    } else {
        AppError::validation(prefixed)
    }
}

fn concise_provider_message(stderr: &str) -> String {
    let message = stderr
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("yt-dlp failed");

    let message = message.strip_prefix("ERROR:").unwrap_or(message).trim();

    if message.len() > 300 {
        format!("{}...", &message[..300])
    } else {
        message.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_ytdlp_failure, invalid_youtube_url};

    #[test]
    fn invalid_youtube_url_maps_to_validation_error() {
        let error = invalid_youtube_url("unsupported host");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("unsupported host"));
    }

    #[test]
    fn ytdlp_private_failures_map_to_auth_error() {
        let error = classify_ytdlp_failure("This video is private. Use --cookies to authenticate.");

        assert_eq!(error.kind, crate::error::AppErrorKind::Auth);
        assert!(error.message.contains("private"));
    }

    #[test]
    fn ytdlp_network_failures_map_to_network_error() {
        let error = classify_ytdlp_failure("ERROR: timed out while downloading webpage");

        assert_eq!(error.kind, crate::error::AppErrorKind::Network);
    }

    #[test]
    fn ytdlp_deleted_failures_map_to_not_found_error() {
        let error = classify_ytdlp_failure("ERROR: Video unavailable. This video has been deleted");

        assert_eq!(error.kind, crate::error::AppErrorKind::NotFound);
    }
}
