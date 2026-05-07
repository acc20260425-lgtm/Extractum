use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorKind {
    Validation,
    NotFound,
    Auth,
    Network,
    Conflict,
    Internal,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AppError {
    pub kind: AppErrorKind,
    pub message: String,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(kind: AppErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Validation, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::NotFound, message)
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Auth, message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Network, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Conflict, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Internal, message)
    }

    pub fn database(error: impl std::fmt::Display) -> Self {
        Self::internal(format!("Database error: {error}"))
    }

    pub fn telegram_network(error: impl std::fmt::Display) -> Self {
        Self::network(format!("Telegram request failed: {error}"))
    }

    pub fn llm_network(error: impl std::fmt::Display) -> Self {
        Self::network(format!("LLM request failed: {error}"))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        classify_message(message)
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        classify_message(&message)
    }
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.message
    }
}

fn classify_message(message: &str) -> AppError {
    let normalized = message.trim();
    let lower = normalized.to_ascii_lowercase();

    let kind = if lower.contains("not found")
        || lower.contains("was not found")
        || lower.contains("is missing")
        || lower.contains("missing source_id")
        || lower.contains("missing source_group_id")
        || lower.contains("could not be resolved")
    {
        AppErrorKind::NotFound
    } else if lower.contains("already queued")
        || lower.contains("already running")
        || lower.contains("already exists")
        || lower.contains("duplicate")
        || lower.contains("conflict")
        || lower.contains("cannot be edited directly")
        || lower.contains("cannot be deleted")
        || lower.contains("unique constraint failed")
    {
        AppErrorKind::Conflict
    } else if lower.contains("not authenticated")
        || lower.contains("not initialized")
        || lower.contains("call tg_send_code first")
        || lower.contains("sign in")
        || lower.contains("api key")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("phone_code")
        || lower.contains("session_password")
    {
        AppErrorKind::Auth
    } else if lower.contains("cannot be empty")
        || lower.contains("invalid")
        || lower.contains("unsupported")
        || lower.contains("select either")
        || lower.contains("select at least")
        || lower.contains("at least one")
        || lower.contains("must be")
        || lower.contains("required")
        || lower.contains("not a broadcast channel")
        || lower.contains("requested source kind")
        || lower.contains("different telegram source kind")
        || lower.contains("pass either")
    {
        AppErrorKind::Validation
    } else if lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("dns")
        || lower.contains("socket")
        || lower.contains("transport")
        || lower.contains("http ")
        || lower.contains("gemini request failed")
    {
        AppErrorKind::Network
    } else {
        AppErrorKind::Internal
    };

    AppError::new(
        kind,
        if normalized.is_empty() {
            "Unknown error".to_string()
        } else {
            normalized.to_string()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{classify_message, AppError, AppErrorKind};

    #[test]
    fn classify_message_treats_dialog_lookup_misses_as_not_found() {
        let error =
            classify_message("Telegram source '123' was not found in this account's dialogs");

        assert_eq!(error.kind, AppErrorKind::NotFound);
    }

    #[test]
    fn classify_message_treats_resolution_failures_as_not_found() {
        let error = classify_message(
            "Source 7 could not be resolved from stored username, peer identity metadata, or dialogs",
        );

        assert_eq!(error.kind, AppErrorKind::NotFound);
    }

    #[test]
    fn classify_message_treats_source_kind_mismatches_as_validation() {
        let error = classify_message(
            "Resolved Telegram source has a different Telegram source kind than the requested source kind",
        );

        assert_eq!(error.kind, AppErrorKind::Validation);
    }

    #[test]
    fn database_helper_maps_to_internal() {
        let error = AppError::database("connection closed");

        assert_eq!(error.kind, AppErrorKind::Internal);
        assert_eq!(error.message, "Database error: connection closed");
    }

    #[test]
    fn telegram_network_helper_maps_to_network() {
        let error = AppError::telegram_network("transport disconnected");

        assert_eq!(error.kind, AppErrorKind::Network);
        assert_eq!(
            error.message,
            "Telegram request failed: transport disconnected"
        );
    }

    #[test]
    fn llm_network_helper_maps_to_network() {
        let error = AppError::llm_network("timeout");

        assert_eq!(error.kind, AppErrorKind::Network);
        assert_eq!(error.message, "LLM request failed: timeout");
    }
}
