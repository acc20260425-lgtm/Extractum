use crate::error::AppError;

pub(crate) fn invalid_youtube_url(reason: impl Into<String>) -> AppError {
    AppError::validation(format!("Invalid YouTube URL: {}", reason.into()))
}

#[cfg(test)]
mod tests {
    use super::invalid_youtube_url;

    #[test]
    fn invalid_youtube_url_maps_to_validation_error() {
        let error = invalid_youtube_url("unsupported host");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("unsupported host"));
    }
}
