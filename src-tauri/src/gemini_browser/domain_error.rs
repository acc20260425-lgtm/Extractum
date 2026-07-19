use std::fmt;

pub(crate) type GeminiBrowserResult<T> = Result<T, GeminiBrowserError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeminiBrowserErrorKind {
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
pub(crate) struct GeminiBrowserError {
    kind: GeminiBrowserErrorKind,
    message: String,
}

impl GeminiBrowserError {
    pub(crate) fn kind(&self) -> GeminiBrowserErrorKind {
        self.kind
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    fn new(kind: GeminiBrowserErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Validation, message)
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::NotFound, message)
    }

    pub(crate) fn conflict(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Conflict, message)
    }

    pub(crate) fn persistence(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Persistence, message)
    }

    pub(crate) fn protocol(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Protocol, message)
    }

    pub(crate) fn transport(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Transport, message)
    }

    pub(crate) fn browser(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Browser, message)
    }

    pub(crate) fn timeout(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Timeout, message)
    }

    pub(crate) fn cancellation(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Cancellation, message)
    }

    pub(crate) fn invariant(message: impl Into<String>) -> Self {
        Self::new(GeminiBrowserErrorKind::Invariant, message)
    }
}

impl fmt::Display for GeminiBrowserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GeminiBrowserError {}
