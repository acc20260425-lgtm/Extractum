use std::fmt;

pub type GeminiBrowserResult<T> = Result<T, GeminiBrowserError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeminiBrowserErrorKind {
    Validation,
    NotFound,
    Conflict,
    Persistence,
    Protocol,
    Transport,
    Browser,
    Timeout,
    Cancellation,
    Invariant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeminiBrowserError {
    kind: GeminiBrowserErrorKind,
    message: String,
}

impl GeminiBrowserError {
    pub fn kind(&self) -> GeminiBrowserErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    fn new(kind: GeminiBrowserErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Validation, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::NotFound, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Conflict, message)
    }

    pub fn persistence(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Persistence, message)
    }

    pub fn protocol(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Protocol, message)
    }

    pub fn transport(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Transport, message)
    }

    pub fn browser(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Browser, message)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Timeout, message)
    }

    pub fn cancellation(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Cancellation, message)
    }

    pub fn invariant(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Invariant, message)
    }
}

impl fmt::Display for GeminiBrowserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GeminiBrowserError {}
