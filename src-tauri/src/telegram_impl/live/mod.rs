mod avatar;
mod messages;
mod peer;
mod topics;

use extractum_core::error::AppResult;

use super::dto::{ForumTopicSnapshot, PeerDescriptor};
use super::session::TelegramSession;

pub use messages::{LiveMessage, LiveMessageBatch};
pub use peer::DialogListing;

pub(super) fn dialog_listing(
    client: &grammers_client::Client,
    avatar_budget_ms: u64,
) -> DialogListing {
    DialogListing::new(client.clone(), avatar_budget_ms)
}

pub(super) async fn resolve_dialog_peer(
    client: &grammers_client::Client,
    peer_id: i64,
    expected_subtype: Option<&str>,
) -> AppResult<PeerDescriptor> {
    peer::resolve_dialog_peer(client, peer_id, expected_subtype).await
}

pub(super) async fn resolve_username(
    client: &grammers_client::Client,
    username: &str,
    expected_subtype: Option<&str>,
) -> AppResult<Option<PeerDescriptor>> {
    peer::resolve_username(client, username, expected_subtype).await
}

pub(super) async fn peer_avatar_bytes(
    client: &grammers_client::Client,
    descriptor: &PeerDescriptor,
) -> Option<Vec<u8>> {
    peer::peer_avatar_bytes(client, descriptor).await
}

pub(super) async fn fetch_message_batch(
    client: &grammers_client::Client,
    session: &TelegramSession,
    descriptor: &PeerDescriptor,
    offset_id: i32,
    offset_date: i32,
    limit: usize,
) -> AppResult<LiveMessageBatch> {
    let peer = peer::peer_ref_from_descriptor(descriptor)?;
    messages::fetch_message_batch(client, session, peer, offset_id, offset_date, limit).await
}

pub(super) async fn fetch_forum_topics(
    client: &grammers_client::Client,
    descriptor: &PeerDescriptor,
) -> AppResult<Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>> {
    let peer = peer::peer_ref_from_descriptor(descriptor)?;
    topics::fetch_forum_topics(client, peer).await
}
