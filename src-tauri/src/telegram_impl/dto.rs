use super::media::TelegramMediaPayload;

pub const ITEM_KIND_TELEGRAM_MESSAGE: &str = "telegram_message";
pub(crate) const TELEGRAM_PEER_KIND_CHANNEL: &str = "channel";
pub(crate) const TELEGRAM_PEER_KIND_CHAT: &str = "chat";
pub(crate) const TELEGRAM_PEER_KIND_USER: &str = "user";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramMessageIdentity {
    /// Telegram history/origin peer for this message, not necessarily the current source peer.
    pub history_peer_kind: String,
    pub history_peer_id: i64,
    pub telegram_message_id: i64,
    pub migration_domain: Option<String>,
    pub is_migrated_history: bool,
}

impl TelegramMessageIdentity {
    pub fn validate(&self) -> extractum_core::error::AppResult<()> {
        if !matches!(
            self.history_peer_kind.as_str(),
            TELEGRAM_PEER_KIND_CHANNEL | TELEGRAM_PEER_KIND_CHAT | TELEGRAM_PEER_KIND_USER
        ) {
            return Err(extractum_core::error::AppError::validation(format!(
                "Unsupported Telegram history peer kind '{}'",
                self.history_peer_kind
            )));
        }
        if self.history_peer_id <= 0 {
            return Err(extractum_core::error::AppError::validation(
                "Telegram history peer id must be positive",
            ));
        }
        if self.telegram_message_id <= 0 {
            return Err(extractum_core::error::AppError::validation(
                "Telegram message id must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TelegramItemContext {
    pub reply_to_msg_id: Option<i64>,
    pub reply_to_peer_kind: Option<String>,
    pub reply_to_peer_id: Option<String>,
    pub reply_to_top_id: Option<i64>,
    pub reaction_count: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TelegramMessageDraft {
    pub telegram_identity: Option<TelegramMessageIdentity>,
    pub telegram_context: TelegramItemContext,
    pub content: Option<String>,
    pub content_kind: &'static str,
    pub author: Option<String>,
    pub published_at: i64,
    pub raw_data: Vec<u8>,
    pub item_kind: String,
    pub media: Option<TelegramMediaPayload>,
}

#[cfg(test)]
mod tests {
    use super::{
        TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity,
        ITEM_KIND_TELEGRAM_MESSAGE, TELEGRAM_PEER_KIND_CHANNEL,
    };

    #[test]
    fn telegram_message_draft_has_single_persistence_shape() {
        let draft = TelegramMessageDraft {
            telegram_identity: Some(TelegramMessageIdentity {
                history_peer_kind: "channel".to_string(),
                history_peer_id: 12345,
                telegram_message_id: 42,
                migration_domain: None,
                is_migrated_history: false,
            }),
            telegram_context: TelegramItemContext {
                reply_to_msg_id: Some(41),
                reply_to_peer_kind: Some("channel".to_string()),
                reply_to_peer_id: Some("12345".to_string()),
                reply_to_top_id: Some(40),
                reaction_count: Some(7),
            },
            content: Some("message body".to_string()),
            content_kind: "text_only",
            author: Some("Alice".to_string()),
            published_at: 1_700_000_000,
            raw_data: br#"{"id":42}"#.to_vec(),
            item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
            media: None,
        };

        assert_eq!(
            (
                draft
                    .telegram_identity
                    .as_ref()
                    .map(|identity| identity.telegram_message_id),
                draft.telegram_context.reply_to_msg_id,
                draft.content.as_deref(),
                draft.content_kind,
                draft.author.as_deref(),
                draft.published_at,
                draft.raw_data.as_slice(),
                draft.item_kind.as_str(),
                draft.media.as_ref().map(|media| media.kind.as_str()),
            ),
            (
                Some(42),
                Some(41),
                Some("message body"),
                "text_only",
                Some("Alice"),
                1_700_000_000,
                br#"{"id":42}"#.as_slice(),
                ITEM_KIND_TELEGRAM_MESSAGE,
                None,
            )
        );
    }

    #[test]
    fn telegram_item_kind_constant_matches_persisted_wire_value() {
        assert_eq!(ITEM_KIND_TELEGRAM_MESSAGE, "telegram_message");
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
        let invalid_kind_error = invalid_kind.validate().expect_err("reject kind");
        assert_eq!(
            invalid_kind_error.kind,
            extractum_core::error::AppErrorKind::Validation
        );
        assert_eq!(
            invalid_kind_error.message,
            "Unsupported Telegram history peer kind 'supergroup'"
        );
        assert_eq!(
            serde_json::to_string(&invalid_kind_error).expect("serialize invalid kind"),
            r#"{"kind":"validation","message":"Unsupported Telegram history peer kind 'supergroup'"}"#
        );
        let validation_kind = invalid_kind_error.kind;

        let invalid_peer_id = TelegramMessageIdentity {
            history_peer_kind: TELEGRAM_PEER_KIND_CHANNEL.to_string(),
            history_peer_id: 0,
            telegram_message_id: 1,
            migration_domain: None,
            is_migrated_history: false,
        };
        let invalid_peer_id_error = invalid_peer_id.validate().expect_err("reject peer id");
        assert_eq!(invalid_peer_id_error.kind, validation_kind);
        assert_eq!(
            invalid_peer_id_error.message,
            "Telegram history peer id must be positive"
        );
        assert_eq!(
            serde_json::to_string(&invalid_peer_id_error).expect("serialize invalid peer id"),
            r#"{"kind":"validation","message":"Telegram history peer id must be positive"}"#
        );

        let invalid_message = TelegramMessageIdentity {
            history_peer_kind: TELEGRAM_PEER_KIND_CHANNEL.to_string(),
            history_peer_id: 1,
            telegram_message_id: 0,
            migration_domain: None,
            is_migrated_history: false,
        };
        let invalid_message_error = invalid_message.validate().expect_err("reject message id");
        assert_eq!(
            invalid_message_error.kind,
            extractum_core::error::AppErrorKind::Validation
        );
        assert_eq!(
            invalid_message_error.message,
            "Telegram message id must be positive"
        );
        assert_eq!(
            serde_json::to_string(&invalid_message_error).expect("serialize invalid message id"),
            r#"{"kind":"validation","message":"Telegram message id must be positive"}"#
        );

        let multiple_invalid_ids = TelegramMessageIdentity {
            history_peer_kind: TELEGRAM_PEER_KIND_CHANNEL.to_string(),
            history_peer_id: 0,
            telegram_message_id: 0,
            migration_domain: None,
            is_migrated_history: false,
        };
        let id_precedence_error = multiple_invalid_ids
            .validate()
            .expect_err("reject peer id before message id");
        assert_eq!(id_precedence_error.kind, validation_kind);
        assert_eq!(
            id_precedence_error.message,
            "Telegram history peer id must be positive"
        );
        assert_eq!(
            serde_json::to_string(&id_precedence_error).expect("serialize id precedence error"),
            r#"{"kind":"validation","message":"Telegram history peer id must be positive"}"#
        );

        let multiple_invalid = TelegramMessageIdentity {
            history_peer_kind: "supergroup".to_string(),
            history_peer_id: 0,
            telegram_message_id: 0,
            migration_domain: None,
            is_migrated_history: false,
        };
        let precedence_error = multiple_invalid
            .validate()
            .expect_err("reject multiple invalid values in precedence order");
        assert_eq!(precedence_error.kind, validation_kind);
        assert_eq!(
            precedence_error.message,
            "Unsupported Telegram history peer kind 'supergroup'"
        );
        assert_eq!(
            serde_json::to_string(&precedence_error).expect("serialize precedence error"),
            r#"{"kind":"validation","message":"Unsupported Telegram history peer kind 'supergroup'"}"#
        );
    }
}
