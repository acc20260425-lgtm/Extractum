use extractum_core::error::AppError;
use grammers_mtsender::InvocationError;

pub(super) fn is_non_forum_topic_refresh_error(error: &str) -> bool {
    error.contains("CHANNEL_FORUM_MISSING") || error.contains("CHANNEL_MONOFORUM_UNSUPPORTED")
}

pub(super) fn is_channel_private_error(error: &AppError) -> bool {
    error
        .message
        .to_ascii_uppercase()
        .contains("CHANNEL_PRIVATE")
}

pub(super) fn should_fallback_export_dc_error(error: &InvocationError) -> bool {
    match error {
        InvocationError::InvalidDc
        | InvocationError::Io(_)
        | InvocationError::Transport(_)
        | InvocationError::Authentication(_)
        | InvocationError::Dropped => true,
        InvocationError::Session(_) | InvocationError::Rpc(_) | InvocationError::Deserialize(_) => {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use extractum_core::error::AppError;

    use super::{is_channel_private_error, is_non_forum_topic_refresh_error};

    #[test]
    fn non_forum_topic_refresh_errors_are_detected() {
        assert!(is_non_forum_topic_refresh_error(
            "Rpc error 400: CHANNEL_FORUM_MISSING"
        ));
        assert!(is_non_forum_topic_refresh_error(
            "Rpc error 400: CHANNEL_MONOFORUM_UNSUPPORTED"
        ));
        assert!(!is_non_forum_topic_refresh_error(
            "Rpc error 400: CHANNEL_PRIVATE"
        ));
    }

    #[test]
    fn channel_private_detection_reads_rpc_name_from_error_message() {
        assert!(is_channel_private_error(&AppError::network(
            "Rpc error 400: CHANNEL_PRIVATE"
        )));
        assert!(!is_channel_private_error(&AppError::network(
            "Rpc error 400: TAKEOUT_INVALID"
        )));
    }
}
