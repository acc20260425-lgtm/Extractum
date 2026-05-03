mod avatar;
mod items;
mod peer_resolution;
mod settings;
mod store;
mod sync;
mod topics;
mod types;

pub use self::items::{get_items, ForumTopicFilter, ItemRecord};
pub use self::settings::{
    get_sync_settings, save_sync_settings, InitialSyncMode, SyncSettingsRecord,
};
pub use self::store::{add_telegram_source, delete_source, list_sources, list_telegram_sources};
pub use self::sync::{sync_source, SyncResult};
pub use self::topics::{list_source_forum_topics, SourceForumTopicRecord};
pub use self::types::{SourceRecord, TelegramSourceInfo};

pub(crate) use self::items::{insert_source_item, SourceItemInsert, TelegramItemContext};
pub(crate) use self::peer_resolution::{resolve_and_refresh_peer, ResolvedSyncPeer};
pub(crate) use self::store::load_source;
pub(crate) use self::sync::finalize_sync;
pub(crate) use self::types::SourceSyncTarget;
