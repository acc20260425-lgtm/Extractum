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
        || lower.contains("is missing")
        || lower.contains("missing source_id")
        || lower.contains("missing source_group_id")
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
