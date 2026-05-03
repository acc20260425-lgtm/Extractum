mod avatar;
mod items;
mod peer_resolution;
mod settings;
mod store;
mod sync;
mod topics;
mod types;

pub use items::{get_items, ForumTopicFilter, ItemRecord};
pub use settings::{get_sync_settings, save_sync_settings, InitialSyncMode, SyncSettingsRecord};
pub use store::{add_telegram_source, delete_source, list_sources, list_telegram_sources};
pub use sync::{sync_source, SyncResult};
pub use topics::{list_source_forum_topics, SourceForumTopicRecord};
pub use types::{SourceRecord, TelegramSourceInfo};

pub(crate) use items::{insert_source_item, SourceItemInsert, TelegramItemContext};
pub(crate) use peer_resolution::{resolve_and_refresh_peer, ResolvedSyncPeer};
pub(crate) use store::load_source;
pub(crate) use sync::finalize_sync;
pub(crate) use types::SourceSyncTarget;
