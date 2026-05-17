use serde::Serialize;

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

#[derive(Serialize)]
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
}

#[derive(sqlx::FromRow)]
pub(super) struct StoredItemRow {
    pub(super) id: i64,
    pub(super) source_id: i64,
    pub(super) external_id: String,
    pub(super) item_kind: String,
    pub(super) author: Option<String>,
    pub(super) published_at: i64,
    pub(super) content_kind: String,
    pub(super) has_media: bool,
    pub(super) media_kind: Option<String>,
    pub(super) content_zstd: Option<Vec<u8>>,
    pub(super) media_metadata_zstd: Option<Vec<u8>>,
    pub(super) raw_data_zstd: Option<Vec<u8>>,
    pub(super) forum_topic_id: Option<i64>,
    pub(super) forum_topic_title: Option<String>,
    pub(super) forum_topic_top_message_id: Option<i64>,
    pub(super) reply_to_msg_id: Option<i64>,
    pub(super) reply_to_peer_kind: Option<String>,
    pub(super) reply_to_peer_id: Option<String>,
    pub(super) reply_to_top_id: Option<i64>,
    pub(super) reaction_count: Option<i64>,
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

pub(super) fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
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
