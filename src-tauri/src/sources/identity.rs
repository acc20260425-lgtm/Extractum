#![allow(dead_code)]

use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::Serialize;

use crate::error::{AppError, AppResult};

use super::types::{now_secs, SourceSyncTarget, TelegramSourceKind, TELEGRAM_SOURCE_TYPE};

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
            (
                TelegramPeerKind::Channel,
                TelegramSourceKind::Channel | TelegramSourceKind::Supergroup,
                None,
            ) => Ok(None),
            (TelegramPeerKind::Chat, TelegramSourceKind::Group, _) => Ok(None),
            _ => Err(AppError::validation(format!(
                "Source {} has inconsistent Telegram typed identity",
                self.source_id
            ))),
        }
    }
}

#[derive(sqlx::FromRow)]
struct TelegramSourceIdentityRow {
    source_id: i64,
    account_id: i64,
    source_subtype: String,
    peer_kind: String,
    peer_id: i64,
    resolution_strategy: String,
    username: Option<String>,
    access_hash: Option<i64>,
    avatar_cache_key: Option<String>,
}

pub(crate) async fn load_telegram_source_identity(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<TelegramSourceIdentity> {
    let row: TelegramSourceIdentityRow = sqlx::query_as(
        r#"
        SELECT source_id, account_id, source_subtype, peer_kind, peer_id,
               resolution_strategy, username, access_hash, avatar_cache_key
        FROM telegram_sources
        WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| {
        AppError::internal(format!(
            "Source {source_id} is missing Telegram typed identity after startup repair"
        ))
    })?;

    Ok(TelegramSourceIdentity {
        source_id: row.source_id,
        account_id: row.account_id,
        source_subtype: TelegramSourceKind::from_source_subtype(&row.source_subtype)?,
        peer_kind: TelegramPeerKind::parse(&row.peer_kind)?,
        peer_id: row.peer_id,
        resolution_strategy: TelegramResolutionStrategy::parse(&row.resolution_strategy)?,
        username: row.username,
        access_hash: row.access_hash,
        avatar_cache_key: row.avatar_cache_key,
    })
}

pub(crate) struct TelegramRuntimeSource {
    pub(crate) source: SourceSyncTarget,
    pub(crate) identity: TelegramSourceIdentity,
}

pub(crate) async fn load_telegram_runtime_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<TelegramRuntimeSource> {
    let source = crate::sources::store::load_source(pool, source_id).await?;
    if source.source_type != TELEGRAM_SOURCE_TYPE {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a Telegram source"
        )));
    }
    let identity = load_telegram_source_identity(pool, source_id).await?;
    Ok(TelegramRuntimeSource { source, identity })
}

pub(crate) fn canonical_telegram_external_id(value: &str) -> AppResult<i64> {
    if value.is_empty()
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(AppError::validation(
            "Malformed Telegram external_id for source identity",
        ));
    }

    let parsed = value
        .parse::<i64>()
        .map_err(|_| AppError::validation("Malformed Telegram external_id for source identity"))?;
    if parsed <= 0 || parsed.to_string() != value {
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
        for value in [
            "",
            "0",
            "00123",
            "-123",
            "+123",
            " 123",
            "123 ",
            "@name",
            "name",
            "telegram:123",
            "12a3",
            "１２３",
        ] {
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

    #[tokio::test]
    async fn load_telegram_identity_returns_typed_row() {
        let pool = crate::sources::test_support::memory_pool_with_sources().await;
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id,
                external_id, title, is_active, is_member, created_at
            )
            VALUES (101, 'telegram', 'channel', 1, '12345', 'source', 1, 1, 100)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash
            )
            VALUES (101, 1, 'channel', 'channel', 12345, 'username', 'example', 77)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert typed row");

        let identity = load_telegram_source_identity(&pool, 101)
            .await
            .expect("load typed identity");

        assert_eq!(identity.source_id, 101);
        assert_eq!(identity.source_subtype, TelegramSourceKind::Channel);
        assert_eq!(identity.peer_kind, TelegramPeerKind::Channel);
        assert_eq!(identity.username.as_deref(), Some("example"));
    }

    #[tokio::test]
    async fn load_telegram_runtime_source_pairs_source_with_typed_identity() {
        let pool = crate::sources::test_support::memory_pool_with_sources().await;
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id,
                external_id, title, is_active, is_member, created_at
            )
            VALUES (101, 'telegram', 'channel', 1, '12345', 'source', 1, 1, 100)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash
            )
            VALUES (101, 1, 'channel', 'channel', 12345, 'username', 'example', 77)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert typed row");

        let runtime_source = load_telegram_runtime_source(&pool, 101)
            .await
            .expect("load runtime source");

        assert_eq!(runtime_source.source.id, 101);
        assert_eq!(runtime_source.identity.peer_id, 12345);
    }
}
