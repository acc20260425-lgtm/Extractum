mod avatar;
pub(crate) mod identity;
pub(crate) mod identity_repair;
mod items;
mod legacy_metadata_cleanup;
mod peer_resolution;
mod settings;
mod store;
mod sync;
#[cfg(test)]
pub(crate) mod test_support;
mod topics;
mod types;

pub use items::list_source_items;
#[allow(unused_imports)]
pub use items::{ForumTopicFilter, ItemRecord, ListSourceItemsRequest};
pub(crate) use legacy_metadata_cleanup::{
    audit_legacy_telegram_source_metadata, clear_legacy_telegram_source_metadata,
};
pub use settings::{get_sync_settings, save_sync_settings};
#[allow(unused_imports)]
pub use settings::{InitialSyncMode, SaveSyncSettingsRequest, SyncSettingsRecord};
#[allow(unused_imports)]
pub use store::AddTelegramSourceRequest;
pub use store::{add_telegram_source, delete_source, list_sources, list_telegram_sources};
pub use sync::sync_source;
#[allow(unused_imports)]
pub use sync::SyncResult;
pub use topics::list_source_forum_topics;
pub(crate) use topics::refresh_forum_topics;
#[allow(unused_imports)]
pub use topics::SourceForumTopicRecord;
#[allow(unused_imports)]
pub use types::{SourceRecord, SourceType, TelegramSourceInfo, TelegramSourceKind};

pub(crate) use identity_repair::{require_source_identity_ready, SourceIdentityRepairState};
#[allow(unused_imports)]
pub(crate) use items::{
    insert_source_item, insert_telegram_source_item, insert_telegram_source_item_outcome,
    insert_telegram_source_item_with_observation,
    insert_telegram_source_item_with_observation_in_context, upsert_youtube_comment_item,
    upsert_youtube_transcript_item, SourceItemInsert, TelegramInsertContext, TelegramItemContext,
    TelegramItemInsertOutcome,
};
pub(crate) use peer_resolution::{resolve_and_refresh_peer, ResolvedSyncPeer};
pub(crate) use store::{
    load_source, load_source_record, upsert_youtube_playlist_source, upsert_youtube_video_source,
};
pub(crate) use sync::finalize_sync;
pub(crate) use types::{
    SourceSyncTarget, StoredItemRow, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,
    MIGRATED_HISTORY_STATUS_AVAILABLE, MIGRATED_HISTORY_STATUS_UNAVAILABLE, TELEGRAM_KIND_CHANNEL,
    TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
};
