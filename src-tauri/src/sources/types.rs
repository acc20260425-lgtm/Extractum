use serde::Serialize;

pub(crate) const TELEGRAM_SOURCE_TYPE: &str = "telegram";
pub(crate) const TELEGRAM_KIND_CHANNEL: &str = "channel";
pub(crate) const TELEGRAM_KIND_SUPERGROUP: &str = "supergroup";
pub(crate) const TELEGRAM_KIND_GROUP: &str = "group";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Telegram,
}

impl SourceType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Telegram => TELEGRAM_SOURCE_TYPE,
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

    pub(crate) fn parse(value: &str) -> crate::error::AppResult<Self> {
        match value {
            TELEGRAM_KIND_CHANNEL => Ok(Self::Channel),
            TELEGRAM_KIND_SUPERGROUP => Ok(Self::Supergroup),
            TELEGRAM_KIND_GROUP => Ok(Self::Group),
            other => Err(crate::error::AppError::validation(format!(
                "Unsupported telegram_source_kind '{other}'"
            ))),
        }
    }
}

#[derive(Serialize)]
pub struct TelegramSourceInfo {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub telegram_source_kind: String,
    pub is_member: bool,
    pub photo_data_url: Option<String>,
}

#[derive(Serialize)]
pub struct SourceRecord {
    pub id: i64,
    pub source_type: String,
    pub telegram_source_kind: String,
    pub account_id: Option<i64>,
    pub external_id: String,
    pub title: Option<String>,
    pub last_sync_state: Option<i64>,
    pub last_synced_at: Option<i64>,
    pub is_member: bool,
    pub is_active: bool,
    pub created_at: i64,
    pub avatar_data_url: Option<String>,
}

#[derive(sqlx::FromRow)]
pub(crate) struct SourceSyncTarget {
    pub(crate) id: i64,
    pub(crate) source_type: String,
    pub(crate) telegram_source_kind: String,
    pub(crate) account_id: Option<i64>,
    pub(crate) external_id: String,
    pub(crate) title: Option<String>,
    pub(crate) metadata_zstd: Option<Vec<u8>>,
    pub(crate) last_sync_state: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub(super) struct SourceRecordRow {
    pub(super) id: i64,
    pub(super) source_type: String,
    pub(super) telegram_source_kind: String,
    pub(super) account_id: Option<i64>,
    pub(super) external_id: String,
    pub(super) title: Option<String>,
    pub(super) metadata_zstd: Option<Vec<u8>>,
    pub(super) last_sync_state: Option<i64>,
    pub(super) last_synced_at: Option<i64>,
    pub(super) is_active: bool,
    pub(super) is_member: bool,
    pub(super) created_at: i64,
}

#[derive(sqlx::FromRow)]
pub(super) struct StoredItemRow {
    pub(super) id: i64,
    pub(super) source_id: i64,
    pub(super) external_id: String,
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
    use super::TelegramSourceKind;

    #[test]
    fn telegram_source_kind_parses_supported_values() {
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
    fn telegram_source_kind_rejects_unknown_values_as_validation() {
        let error = TelegramSourceKind::parse("user").expect_err("unsupported kind");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    }

    #[test]
    fn telegram_source_kind_serializes_as_existing_wire_value() {
        let value = serde_json::to_string(&TelegramSourceKind::Supergroup).expect("serialize");
        assert_eq!(value, "\"supergroup\"");
    }
}
