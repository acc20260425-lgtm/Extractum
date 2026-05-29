use serde::Serialize;

pub(super) use crate::time::now_secs;

pub(crate) const TELEGRAM_SOURCE_TYPE: &str = "telegram";
pub(crate) const YOUTUBE_SOURCE_TYPE: &str = "youtube";
pub(crate) const RSS_SOURCE_TYPE: &str = "rss";
pub(crate) const FORUM_SOURCE_TYPE: &str = "forum";
pub(crate) const TELEGRAM_KIND_CHANNEL: &str = "channel";
pub(crate) const TELEGRAM_KIND_SUPERGROUP: &str = "supergroup";
pub(crate) const TELEGRAM_KIND_GROUP: &str = "group";
pub(crate) const TELEGRAM_PEER_KIND_CHANNEL: &str = "channel";
pub(crate) const TELEGRAM_PEER_KIND_CHAT: &str = "chat";
pub(crate) const TELEGRAM_PEER_KIND_USER: &str = "user";
#[allow(dead_code)]
pub(crate) const TELEGRAM_MIGRATION_DOMAIN_MIGRATED_FROM_CHAT: &str = "migrated_from_chat";
#[allow(dead_code)]
pub(crate) const MIGRATED_HISTORY_STATUS_NONE: &str = "none";
#[allow(dead_code)]
pub(crate) const MIGRATED_HISTORY_STATUS_AVAILABLE: &str = "available";
#[allow(dead_code)]
pub(crate) const MIGRATED_HISTORY_STATUS_UNAVAILABLE: &str = "unavailable";
#[allow(dead_code)]
pub(crate) const TELEGRAM_HISTORY_SCOPE_CURRENT: &str = "current";
#[allow(dead_code)]
pub(crate) const TELEGRAM_HISTORY_SCOPE_MIGRATED: &str = "migrated";
#[allow(dead_code)]
pub(crate) const TELEGRAM_HISTORY_SCOPE_MERGED: &str = "merged";
#[allow(dead_code)]
pub(crate) const TELEGRAM_HISTORY_SCOPE_LABEL_CURRENT: &str = "Current supergroup history";
#[allow(dead_code)]
pub(crate) const TELEGRAM_HISTORY_SCOPE_LABEL_MIGRATED: &str = "Migrated small-group history";
#[allow(dead_code)]
pub(crate) const ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT: &str = "current";
#[allow(dead_code)]
pub(crate) const ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED: &str =
    "current_plus_migrated";
#[allow(dead_code)]
pub(crate) const TELEGRAM_MESSAGE_HISTORY_SCOPE_CURRENT: &str = "current";
#[allow(dead_code)]
pub(crate) const TELEGRAM_MESSAGE_HISTORY_SCOPE_MIGRATED: &str = "migrated";
#[allow(dead_code)]
pub(crate) const NOTEBOOKLM_HISTORY_SCOPE_CURRENT_SUPERGROUP: &str = "current_supergroup_history";
#[allow(dead_code)]
pub(crate) const NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP: &str =
    "migrated_small_group_history";
pub(crate) const ITEM_KIND_TELEGRAM_MESSAGE: &str = "telegram_message";
pub(crate) const ITEM_KIND_YOUTUBE_TRANSCRIPT: &str = "youtube_transcript";
pub(crate) const ITEM_KIND_YOUTUBE_COMMENT: &str = "youtube_comment";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Telegram,
    Youtube,
    Rss,
    Forum,
}

impl SourceType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Telegram => TELEGRAM_SOURCE_TYPE,
            Self::Youtube => YOUTUBE_SOURCE_TYPE,
            Self::Rss => RSS_SOURCE_TYPE,
            Self::Forum => FORUM_SOURCE_TYPE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramSourceKind {
    Channel,
    Supergroup,
    Group,
}

impl TelegramSourceKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Channel => TELEGRAM_KIND_CHANNEL,
            Self::Supergroup => TELEGRAM_KIND_SUPERGROUP,
            Self::Group => TELEGRAM_KIND_GROUP,
        }
    }

    pub(crate) fn from_source_subtype(value: &str) -> crate::error::AppResult<Self> {
        match value {
            TELEGRAM_KIND_CHANNEL => Ok(Self::Channel),
            TELEGRAM_KIND_SUPERGROUP => Ok(Self::Supergroup),
            TELEGRAM_KIND_GROUP => Ok(Self::Group),
            other => Err(crate::error::AppError::validation(format!(
                "Unsupported Telegram source_subtype '{other}'"
            ))),
        }
    }

    pub(crate) fn parse(value: &str) -> crate::error::AppResult<Self> {
        Self::from_source_subtype(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub(crate) enum TelegramHistoryScope {
    Current,
    Migrated,
    Merged,
}

#[allow(dead_code)]
impl TelegramHistoryScope {
    pub(crate) fn from_optional(value: Option<Self>) -> Self {
        value.unwrap_or(Self::Current)
    }

    pub(crate) fn as_wire(self) -> &'static str {
        match self {
            Self::Current => TELEGRAM_HISTORY_SCOPE_CURRENT,
            Self::Migrated => TELEGRAM_HISTORY_SCOPE_MIGRATED,
            Self::Merged => TELEGRAM_HISTORY_SCOPE_MERGED,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub(crate) struct SourceItemsCursor {
    pub(crate) published_at: i64,
    pub(crate) history_scope_order: i64,
    pub(crate) history_peer_kind: String,
    pub(crate) history_peer_id: i64,
    pub(crate) telegram_message_id: i64,
    pub(crate) item_id: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
struct SourceItemsCursorEnvelope {
    version: u8,
    cursor: SourceItemsCursor,
}

#[allow(dead_code)]
impl SourceItemsCursor {
    pub(crate) fn encode_opaque(&self) -> crate::error::AppResult<String> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        let envelope = SourceItemsCursorEnvelope {
            version: 1,
            cursor: self.clone(),
        };
        let json = serde_json::to_vec(&envelope)
            .map_err(|error| crate::error::AppError::internal(error.to_string()))?;
        Ok(URL_SAFE_NO_PAD.encode(json))
    }

    pub(crate) fn decode_opaque(value: &str) -> crate::error::AppResult<Self> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        let json = URL_SAFE_NO_PAD
            .decode(value)
            .map_err(|_| crate::error::AppError::validation("Invalid source item cursor"))?;
        let envelope: SourceItemsCursorEnvelope = serde_json::from_slice(&json)
            .map_err(|_| crate::error::AppError::validation("Invalid source item cursor"))?;
        if envelope.version != 1 {
            return Err(crate::error::AppError::validation(
                "Unsupported source item cursor",
            ));
        }
        Ok(envelope.cursor)
    }
}

#[derive(Serialize)]
pub struct TelegramSourceInfo {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub source_subtype: String,
    pub is_member: bool,
    pub photo_data_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelegramMessageIdentity {
    /// Telegram history/origin peer for this message, not necessarily the current source peer.
    pub(crate) history_peer_kind: String,
    pub(crate) history_peer_id: i64,
    pub(crate) telegram_message_id: i64,
    pub(crate) migration_domain: Option<String>,
    pub(crate) is_migrated_history: bool,
}

impl TelegramMessageIdentity {
    pub(crate) fn validate(&self) -> crate::error::AppResult<()> {
        if !matches!(
            self.history_peer_kind.as_str(),
            TELEGRAM_PEER_KIND_CHANNEL | TELEGRAM_PEER_KIND_CHAT | TELEGRAM_PEER_KIND_USER
        ) {
            return Err(crate::error::AppError::validation(format!(
                "Unsupported Telegram history peer kind '{}'",
                self.history_peer_kind
            )));
        }
        if self.history_peer_id <= 0 {
            return Err(crate::error::AppError::validation(
                "Telegram history peer id must be positive",
            ));
        }
        if self.telegram_message_id <= 0 {
            return Err(crate::error::AppError::validation(
                "Telegram message id must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct TelegramSourcePeerIdentity {
    pub(crate) peer_kind: String,
    pub(crate) peer_id: i64,
}

#[derive(Debug, Serialize)]
pub struct SourceRecord {
    pub id: i64,
    pub source_type: String,
    pub source_subtype: String,
    pub account_id: Option<i64>,
    pub external_id: String,
    pub title: Option<String>,
    pub last_sync_state: Option<i64>,
    pub last_synced_at: Option<i64>,
    pub is_member: bool,
    pub is_active: bool,
    pub created_at: i64,
    pub telegram_username: Option<String>,
    pub avatar_data_url: Option<String>,
    pub migrated_history_status: String,
    pub migrated_history_detected_at: Option<i64>,
    pub migrated_history_refreshed_at: Option<i64>,
    pub migrated_history_row_count: i64,
    pub migrated_history_import_completed: bool,
}

#[derive(sqlx::FromRow)]
pub(crate) struct SourceSyncTarget {
    pub(crate) id: i64,
    pub(crate) source_type: String,
    pub(crate) source_subtype: Option<String>,
    pub(crate) account_id: Option<i64>,
    pub(crate) external_id: String,
    pub(crate) title: Option<String>,
    pub(crate) last_sync_state: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub(super) struct SourceRecordRow {
    pub(super) id: i64,
    pub(super) source_type: String,
    pub(super) source_subtype: Option<String>,
    pub(super) account_id: Option<i64>,
    pub(super) external_id: String,
    pub(super) title: Option<String>,
    #[allow(dead_code)]
    pub(super) metadata_zstd: Option<Vec<u8>>,
    pub(super) last_sync_state: Option<i64>,
    pub(super) last_synced_at: Option<i64>,
    pub(super) is_active: bool,
    pub(super) is_member: bool,
    pub(super) created_at: i64,
    pub(super) telegram_username: Option<String>,
    pub(super) telegram_avatar_cache_key: Option<String>,
    pub(super) migrated_history_status: Option<String>,
    pub(super) migrated_history_detected_at: Option<i64>,
    pub(super) migrated_history_refreshed_at: Option<i64>,
    pub(super) migrated_history_row_count: i64,
    pub(super) migrated_history_import_completed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub(crate) struct StoredItemRow {
    pub(crate) id: i64,
    pub(crate) source_id: i64,
    pub(crate) external_id: String,
    pub(crate) item_kind: String,
    pub(crate) author: Option<String>,
    pub(crate) published_at: i64,
    pub(crate) content_kind: String,
    pub(crate) has_media: bool,
    pub(crate) media_kind: Option<String>,
    pub(crate) content_zstd: Option<Vec<u8>>,
    pub(crate) media_metadata_zstd: Option<Vec<u8>>,
    pub(crate) has_raw_data: bool,
    pub(crate) forum_topic_id: Option<i64>,
    pub(crate) forum_topic_title: Option<String>,
    pub(crate) forum_topic_top_message_id: Option<i64>,
    pub(crate) reply_to_msg_id: Option<i64>,
    pub(crate) reply_to_peer_kind: Option<String>,
    pub(crate) reply_to_peer_id: Option<String>,
    pub(crate) reply_to_top_id: Option<i64>,
    pub(crate) reaction_count: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub(super) struct SourceForumTopicRow {
    pub(super) topic_id: i64,
    pub(super) top_message_id: i64,
    pub(super) title: String,
    pub(super) icon_color: Option<i64>,
    pub(super) icon_emoji_id: Option<i64>,
    pub(super) is_closed: bool,
    pub(super) is_pinned: bool,
    pub(super) is_hidden: bool,
    pub(super) is_deleted: bool,
    pub(super) sort_order: Option<i64>,
    pub(super) message_count: i64,
}

#[cfg(test)]
mod tests {
    use super::{
        SourceType, TelegramMessageIdentity, TelegramSourceKind, ITEM_KIND_TELEGRAM_MESSAGE,
        ITEM_KIND_YOUTUBE_COMMENT, ITEM_KIND_YOUTUBE_TRANSCRIPT, TELEGRAM_PEER_KIND_CHANNEL,
    };

    #[test]
    fn source_type_serializes_supported_provider_values() {
        assert_eq!(
            serde_json::to_string(&SourceType::Telegram).expect("serialize"),
            "\"telegram\""
        );
        assert_eq!(
            serde_json::to_string(&SourceType::Youtube).expect("serialize"),
            "\"youtube\""
        );
        assert_eq!(
            serde_json::to_string(&SourceType::Rss).expect("serialize"),
            "\"rss\""
        );
        assert_eq!(
            serde_json::to_string(&SourceType::Forum).expect("serialize"),
            "\"forum\""
        );
    }

    #[test]
    fn telegram_source_subtype_parses_supported_values() {
        assert_eq!(
            TelegramSourceKind::parse("channel").unwrap(),
            TelegramSourceKind::Channel
        );
        assert_eq!(
            TelegramSourceKind::parse("supergroup").unwrap(),
            TelegramSourceKind::Supergroup
        );
        assert_eq!(
            TelegramSourceKind::parse("group").unwrap(),
            TelegramSourceKind::Group
        );
    }

    #[test]
    fn telegram_source_subtype_parses_from_canonical_source_subtype() {
        assert_eq!(
            TelegramSourceKind::from_source_subtype("channel").unwrap(),
            TelegramSourceKind::Channel
        );
        assert_eq!(
            TelegramSourceKind::from_source_subtype("supergroup").unwrap(),
            TelegramSourceKind::Supergroup
        );
        assert_eq!(
            TelegramSourceKind::from_source_subtype("group").unwrap(),
            TelegramSourceKind::Group
        );
    }

    #[test]
    fn telegram_source_subtype_rejects_unsupported_source_subtype() {
        let error =
            TelegramSourceKind::from_source_subtype("video").expect_err("unsupported subtype");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    }

    #[test]
    fn telegram_source_subtype_rejects_unknown_values_as_validation() {
        let error = TelegramSourceKind::parse("user").expect_err("unsupported kind");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    }

    #[test]
    fn telegram_source_subtype_serializes_as_existing_wire_value() {
        let value = serde_json::to_string(&TelegramSourceKind::Supergroup).expect("serialize");
        assert_eq!(value, "\"supergroup\"");
    }

    #[test]
    fn item_kind_constants_match_persisted_wire_values() {
        assert_eq!(ITEM_KIND_TELEGRAM_MESSAGE, "telegram_message");
        assert_eq!(ITEM_KIND_YOUTUBE_TRANSCRIPT, "youtube_transcript");
        assert_eq!(ITEM_KIND_YOUTUBE_COMMENT, "youtube_comment");
    }

    #[test]
    fn telegram_message_identity_validation_rejects_invalid_values() {
        let invalid_kind = TelegramMessageIdentity {
            history_peer_kind: "supergroup".to_string(),
            history_peer_id: 1,
            telegram_message_id: 1,
            migration_domain: None,
            is_migrated_history: false,
        };
        assert_eq!(
            invalid_kind.validate().expect_err("reject kind").kind,
            crate::error::AppErrorKind::Validation
        );

        let invalid_message = TelegramMessageIdentity {
            history_peer_kind: TELEGRAM_PEER_KIND_CHANNEL.to_string(),
            history_peer_id: 1,
            telegram_message_id: 0,
            migration_domain: None,
            is_migrated_history: false,
        };
        assert_eq!(
            invalid_message.validate().expect_err("reject id").kind,
            crate::error::AppErrorKind::Validation
        );
    }
}
