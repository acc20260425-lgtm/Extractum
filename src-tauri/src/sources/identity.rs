#![allow(dead_code)]

use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::Serialize;

use crate::error::{AppError, AppResult};

use super::types::{now_secs, TelegramSourceKind, TELEGRAM_SOURCE_TYPE};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceIdentity {
    pub(crate) id: i64,
    pub(crate) source_type: String,
    pub(crate) source_subtype: String,
    pub(crate) account_id: Option<i64>,
    pub(crate) external_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TelegramPeerKind {
    Channel,
    Chat,
}

impl TelegramPeerKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Channel => "channel",
            Self::Chat => "chat",
        }
    }

    pub(crate) fn parse(value: &str) -> AppResult<Self> {
        match value {
            "channel" => Ok(Self::Channel),
            "chat" => Ok(Self::Chat),
            other => Err(AppError::validation(format!(
                "Unsupported Telegram peer_kind '{other}'"
            ))),
        }
    }

    pub(crate) fn from_source_subtype(subtype: TelegramSourceKind) -> Self {
        match subtype {
            TelegramSourceKind::Channel | TelegramSourceKind::Supergroup => Self::Channel,
            TelegramSourceKind::Group => Self::Chat,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TelegramResolutionStrategy {
    Username,
    Dialog,
    LegacyMetadata,
    Unknown,
}

impl TelegramResolutionStrategy {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Username => "username",
            Self::Dialog => "dialog",
            Self::LegacyMetadata => "legacy_metadata",
            Self::Unknown => "unknown",
        }
    }

    pub(crate) fn parse(value: &str) -> AppResult<Self> {
        match value {
            "username" => Ok(Self::Username),
            "dialog" => Ok(Self::Dialog),
            "legacy_metadata" => Ok(Self::LegacyMetadata),
            "unknown" => Ok(Self::Unknown),
            other => Err(AppError::validation(format!(
                "Unsupported Telegram resolution_strategy '{other}'"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelegramSourceIdentity {
    pub(crate) source_id: i64,
    pub(crate) account_id: i64,
    pub(crate) source_subtype: TelegramSourceKind,
    pub(crate) peer_kind: TelegramPeerKind,
    pub(crate) peer_id: i64,
    pub(crate) resolution_strategy: TelegramResolutionStrategy,
    pub(crate) username: Option<String>,
    pub(crate) access_hash: Option<i64>,
    pub(crate) avatar_cache_key: Option<String>,
}

impl TelegramSourceIdentity {
    pub(crate) fn peer_ref(&self) -> AppResult<Option<PeerRef>> {
        match (self.peer_kind, self.source_subtype, self.access_hash) {
            (
                TelegramPeerKind::Channel,
                TelegramSourceKind::Channel | TelegramSourceKind::Supergroup,
                Some(access_hash),
            ) => Ok(Some(PeerRef {
                id: PeerId::channel(self.peer_id),
                auth: PeerAuth::from_hash(access_hash),
            })),
            (TelegramPeerKind::Chat, TelegramSourceKind::Group, _) => Ok(None),
            _ => Err(AppError::validation(format!(
                "Source {} has inconsistent Telegram typed identity",
                self.source_id
            ))),
        }
    }
}

pub(crate) fn canonical_telegram_external_id(value: &str) -> AppResult<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| AppError::validation("Malformed Telegram external_id for source identity"))?;
    if parsed < 0 || parsed.to_string() != value {
        return Err(AppError::validation(
            "Malformed Telegram external_id for source identity",
        ));
    }
    Ok(parsed)
}

pub(crate) fn normalize_telegram_username(value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    let stripped = raw
        .strip_prefix("https://t.me/")
        .or_else(|| raw.strip_prefix("http://t.me/"))
        .or_else(|| raw.strip_prefix("t.me/"))
        .unwrap_or(raw)
        .trim_start_matches('@')
        .split(['/', '?'])
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

pub(crate) fn ensure_telegram_source_type(identity: &SourceIdentity) -> AppResult<()> {
    if identity.source_type == TELEGRAM_SOURCE_TYPE {
        Ok(())
    } else {
        Err(AppError::validation(format!(
            "Source {} is not a Telegram source",
            identity.id
        )))
    }
}

pub(crate) fn identity_updated_at() -> i64 {
    now_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_kind_matches_telegram_subtype() {
        assert_eq!(
            TelegramPeerKind::from_source_subtype(TelegramSourceKind::Channel),
            TelegramPeerKind::Channel
        );
        assert_eq!(
            TelegramPeerKind::from_source_subtype(TelegramSourceKind::Supergroup),
            TelegramPeerKind::Channel
        );
        assert_eq!(
            TelegramPeerKind::from_source_subtype(TelegramSourceKind::Group),
            TelegramPeerKind::Chat
        );
    }

    #[test]
    fn canonical_external_id_rejects_malformed_values() {
        for value in ["+123", "-123", "00123", "123 ", "12a3", ""] {
            assert!(
                canonical_telegram_external_id(value).is_err(),
                "{value} should be rejected"
            );
        }
        assert_eq!(canonical_telegram_external_id("123").unwrap(), 123);
    }

    #[test]
    fn username_normalization_removes_url_and_at_syntax() {
        assert_eq!(
            normalize_telegram_username(Some("https://t.me/Example_User?x=1")).as_deref(),
            Some("example_user")
        );
        assert_eq!(
            normalize_telegram_username(Some("@MixedCase")).as_deref(),
            Some("mixedcase")
        );
        assert_eq!(normalize_telegram_username(Some("  ")), None);
    }
}
