mod avatar;
mod peer;

use extractum_core::error::AppResult;

use super::dto::PeerDescriptor;

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
