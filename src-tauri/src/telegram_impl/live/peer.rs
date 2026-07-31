use extractum_core::error::{AppError, AppResult};
use grammers_client::{client::DialogIter, peer::Peer, tl};
use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use tokio::time::{Duration, Instant};

use super::super::dto::PeerDescriptor;
use super::avatar::peer_photo_bytes_with_timeout;

enum ListedPeer {
    Grammers(Peer),
    #[cfg(test)]
    Test(TestPeer),
}

enum DialogListingBackend {
    Grammers {
        client: grammers_client::Client,
        dialogs: DialogIter,
    },
    #[cfg(test)]
    Test(TestPeerTransport),
}

pub struct DialogListing {
    backend: DialogListingBackend,
    avatar_budget_started_at: Instant,
    avatar_budget: Duration,
}

impl DialogListing {
    pub(super) fn new(client: grammers_client::Client, avatar_budget_ms: u64) -> Self {
        let dialogs = client.iter_dialogs();
        Self {
            backend: DialogListingBackend::Grammers { client, dialogs },
            avatar_budget_started_at: Instant::now(),
            avatar_budget: Duration::from_millis(avatar_budget_ms),
        }
    }

    pub async fn next(&mut self) -> AppResult<Option<PeerDescriptor>> {
        loop {
            let Some(peer) = self.backend.next_peer().await? else {
                return Ok(None);
            };
            let Some(mut descriptor) = self.backend.descriptor(&peer) else {
                continue;
            };
            if self.avatar_budget_started_at.elapsed() < self.avatar_budget {
                descriptor.avatar_bytes = self.backend.avatar(&peer).await;
            }
            return Ok(Some(descriptor));
        }
    }
}

impl DialogListingBackend {
    async fn next_peer(&mut self) -> AppResult<Option<ListedPeer>> {
        match self {
            Self::Grammers { dialogs, .. } => dialogs
                .next()
                .await
                .map(|dialog| dialog.map(|dialog| ListedPeer::Grammers(dialog.peer().clone())))
                .map_err(|error| AppError::network(error.to_string())),
            #[cfg(test)]
            Self::Test(transport) => transport
                .next_dialog_peer()
                .await
                .map(|peer| peer.map(ListedPeer::Test)),
        }
    }

    fn descriptor(&self, peer: &ListedPeer) -> Option<PeerDescriptor> {
        match peer {
            ListedPeer::Grammers(peer) => peer_descriptor_from_peer(peer),
            #[cfg(test)]
            ListedPeer::Test(peer) => peer.descriptor.clone(),
        }
    }

    async fn avatar(&mut self, peer: &ListedPeer) -> Option<Vec<u8>> {
        match (self, peer) {
            (Self::Grammers { client, .. }, ListedPeer::Grammers(peer)) => {
                peer_photo_bytes_with_timeout(client, peer).await
            }
            #[cfg(test)]
            (Self::Test(transport), ListedPeer::Test(peer)) => transport.peer_avatar(peer).await,
            #[cfg(test)]
            _ => unreachable!("dialog-listing backend returned a mismatched peer"),
        }
    }
}

trait PeerDescriptorBackend {
    type Peer;

    fn descriptor(&self, peer: &Self::Peer) -> Option<PeerDescriptor>;
    async fn peer_avatar(&mut self, peer: &Self::Peer) -> Option<Vec<u8>>;
}

trait UsernamePeerBackend: PeerDescriptorBackend {
    async fn resolve_username_peer(&mut self, username: &str) -> AppResult<Option<Self::Peer>>;
}

trait DialogPeerBackend: PeerDescriptorBackend {
    async fn next_dialog_peer(&mut self) -> AppResult<Option<Self::Peer>>;
    fn bare_id(&self, peer: &Self::Peer) -> Option<i64>;
}

struct LiveUsernameBackend<'a> {
    client: &'a grammers_client::Client,
}

impl PeerDescriptorBackend for LiveUsernameBackend<'_> {
    type Peer = Peer;

    fn descriptor(&self, peer: &Self::Peer) -> Option<PeerDescriptor> {
        peer_descriptor_from_peer(peer)
    }

    async fn peer_avatar(&mut self, peer: &Self::Peer) -> Option<Vec<u8>> {
        peer_photo_bytes_with_timeout(self.client, peer).await
    }
}

impl UsernamePeerBackend for LiveUsernameBackend<'_> {
    async fn resolve_username_peer(&mut self, username: &str) -> AppResult<Option<Self::Peer>> {
        self.client
            .resolve_username(username)
            .await
            .map_err(|error| AppError::network(error.to_string()))
    }
}

struct LiveDialogBackend<'a> {
    client: &'a grammers_client::Client,
    dialogs: DialogIter,
}

impl PeerDescriptorBackend for LiveDialogBackend<'_> {
    type Peer = Peer;

    fn descriptor(&self, peer: &Self::Peer) -> Option<PeerDescriptor> {
        peer_descriptor_from_peer(peer)
    }

    async fn peer_avatar(&mut self, peer: &Self::Peer) -> Option<Vec<u8>> {
        peer_photo_bytes_with_timeout(self.client, peer).await
    }
}

impl DialogPeerBackend for LiveDialogBackend<'_> {
    async fn next_dialog_peer(&mut self) -> AppResult<Option<Self::Peer>> {
        self.dialogs
            .next()
            .await
            .map(|dialog| dialog.map(|dialog| dialog.peer().clone()))
            .map_err(|error| AppError::network(error.to_string()))
    }

    fn bare_id(&self, peer: &Self::Peer) -> Option<i64> {
        peer.id().bare_id()
    }
}

async fn resolve_username_descriptor<B>(
    backend: &mut B,
    username: &str,
    expected_subtype: Option<&str>,
) -> AppResult<Option<PeerDescriptor>>
where
    B: UsernamePeerBackend,
{
    let Some(peer) = backend.resolve_username_peer(username).await? else {
        return Ok(None);
    };
    let mut descriptor = backend
        .descriptor(&peer)
        .ok_or_else(|| AppError::validation("Not a Telegram channel, group, or supergroup"))?;
    validate_expected_telegram_source_subtype(&descriptor, expected_subtype)?;
    descriptor.avatar_bytes = backend.peer_avatar(&peer).await;
    Ok(Some(descriptor))
}

async fn resolve_dialog_descriptor<B>(
    backend: &mut B,
    peer_id: i64,
    expected_subtype: Option<&str>,
) -> AppResult<PeerDescriptor>
where
    B: DialogPeerBackend,
{
    let mut found_wrong_kind = false;
    while let Some(peer) = backend.next_dialog_peer().await? {
        if backend.bare_id(&peer) != Some(peer_id) {
            continue;
        }
        let Some(mut descriptor) = backend.descriptor(&peer) else {
            continue;
        };
        if telegram_source_subtype_matches(&descriptor, expected_subtype)? {
            descriptor.avatar_bytes = backend.peer_avatar(&peer).await;
            return Ok(descriptor);
        }
        found_wrong_kind = true;
    }

    if found_wrong_kind {
        return Err(AppError::validation(format!(
            "Telegram source '{peer_id}' was found, but it has a different source subtype than the requested source subtype"
        )));
    }

    Err(dialog_lookup_not_found_error(
        &peer_id.to_string(),
        expected_subtype,
    ))
}

pub(super) async fn resolve_username(
    client: &grammers_client::Client,
    username: &str,
    expected_subtype: Option<&str>,
) -> AppResult<Option<PeerDescriptor>> {
    resolve_username_descriptor(
        &mut LiveUsernameBackend { client },
        username,
        expected_subtype,
    )
    .await
}

pub(super) async fn resolve_dialog_peer(
    client: &grammers_client::Client,
    peer_id: i64,
    expected_subtype: Option<&str>,
) -> AppResult<PeerDescriptor> {
    resolve_dialog_descriptor(
        &mut LiveDialogBackend {
            client,
            dialogs: client.iter_dialogs(),
        },
        peer_id,
        expected_subtype,
    )
    .await
}

pub(super) async fn peer_avatar_bytes(
    client: &grammers_client::Client,
    descriptor: &PeerDescriptor,
) -> Option<Vec<u8>> {
    let peer_ref = peer_ref_from_descriptor(descriptor).ok()?;
    let peer = client.resolve_peer(peer_ref).await.ok()?;
    peer_photo_bytes_with_timeout(client, &peer).await
}

fn dialog_lookup_not_found_message(source_ref: &str, expected_subtype: Option<&str>) -> String {
    if expected_subtype.is_some() {
        format!(
            "Telegram source '{}' was not found in this account's dialogs",
            source_ref
        )
    } else {
        format!(
            "Telegram source '{}' was not found in this account's dialogs. Numeric manual adds only work for sources that are still visible in that account's dialogs. For private Telegram sources, add them from the account's dialogs instead.",
            source_ref
        )
    }
}

fn dialog_lookup_not_found_error(source_ref: &str, expected_subtype: Option<&str>) -> AppError {
    AppError::not_found(dialog_lookup_not_found_message(
        source_ref,
        expected_subtype,
    ))
}

fn telegram_source_subtype_matches(
    source: &PeerDescriptor,
    expected_subtype: Option<&str>,
) -> AppResult<bool> {
    let Some(expected_subtype) = expected_subtype else {
        return Ok(true);
    };

    if !matches!(expected_subtype, "channel" | "supergroup" | "group") {
        return Err(AppError::validation(format!(
            "Unsupported Telegram source_subtype '{expected_subtype}'"
        )));
    }
    Ok(source.source_subtype == expected_subtype)
}

fn validate_expected_telegram_source_subtype(
    source: &PeerDescriptor,
    expected_subtype: Option<&str>,
) -> AppResult<()> {
    if telegram_source_subtype_matches(source, expected_subtype)? {
        Ok(())
    } else {
        Err(AppError::validation(format!(
            "Resolved Telegram source has a different source subtype than the requested source subtype: requested {}, actual {}",
            expected_subtype.unwrap_or("unknown"),
            source.source_subtype
        )))
    }
}

fn peer_descriptor_from_peer(peer: &Peer) -> Option<PeerDescriptor> {
    match peer {
        Peer::Channel(channel) => Some(PeerDescriptor {
            external_id: channel.id().bare_id()?.to_string(),
            title: channel.title().to_string(),
            source_subtype: "channel".to_string(),
            is_member: !channel.raw.left,
            username: channel.username().map(str::to_string),
            access_hash: channel.raw.access_hash,
            avatar_bytes: None,
        }),
        Peer::Group(group) => Some(PeerDescriptor {
            external_id: group.id().bare_id()?.to_string(),
            title: group.title().unwrap_or("Untitled group").to_string(),
            source_subtype: telegram_group_kind(group).to_string(),
            is_member: telegram_group_is_member(group),
            username: group.username().map(str::to_string),
            access_hash: peer_access_hash(peer),
            avatar_bytes: None,
        }),
        Peer::User(_) => None,
    }
}

fn telegram_group_kind(group: &grammers_client::peer::Group) -> &'static str {
    if group.is_megagroup() {
        "supergroup"
    } else {
        "group"
    }
}

fn telegram_group_is_member(group: &grammers_client::peer::Group) -> bool {
    match &group.raw {
        tl::enums::Chat::Chat(chat) => !chat.left && !chat.deactivated,
        tl::enums::Chat::Channel(channel) => !channel.left,
        tl::enums::Chat::Empty(_)
        | tl::enums::Chat::Forbidden(_)
        | tl::enums::Chat::ChannelForbidden(_) => false,
    }
}

fn peer_access_hash(peer: &Peer) -> Option<i64> {
    match peer {
        Peer::Channel(channel) => channel.raw.access_hash,
        Peer::Group(group) => match &group.raw {
            tl::enums::Chat::Channel(channel) => channel.access_hash,
            tl::enums::Chat::ChannelForbidden(channel) => Some(channel.access_hash),
            tl::enums::Chat::Empty(_)
            | tl::enums::Chat::Chat(_)
            | tl::enums::Chat::Forbidden(_) => None,
        },
        Peer::User(_) => None,
    }
}

fn peer_ref_from_descriptor(descriptor: &PeerDescriptor) -> AppResult<PeerRef> {
    let peer_id = descriptor
        .external_id
        .parse::<i64>()
        .ok()
        .filter(|peer_id| *peer_id > 0)
        .ok_or_else(|| {
            AppError::validation(format!(
                "Invalid Telegram peer id '{}'",
                descriptor.external_id
            ))
        })?;

    match descriptor.source_subtype.as_str() {
        "channel" | "supergroup" => {
            let access_hash = descriptor.access_hash.ok_or_else(|| {
                AppError::validation(format!(
                    "Telegram {} {} is missing an access hash",
                    descriptor.source_subtype, descriptor.external_id
                ))
            })?;
            Ok(PeerRef {
                id: PeerId::channel(peer_id).ok_or_else(|| {
                    AppError::validation(format!("Invalid Telegram channel peer id {peer_id}"))
                })?,
                auth: PeerAuth::from_hash(access_hash),
            })
        }
        "group" => Ok(PeerRef {
            id: PeerId::chat(peer_id).ok_or_else(|| {
                AppError::validation(format!("Invalid Telegram group peer id {peer_id}"))
            })?,
            auth: PeerAuth::default(),
        }),
        other => Err(AppError::validation(format!(
            "Unsupported Telegram source_subtype '{other}'"
        ))),
    }
}

#[cfg(test)]
const TEST_AVATAR_TIMEOUT_MS: u64 = 750;

#[cfg(test)]
#[derive(Clone)]
struct TestPeer {
    label: &'static str,
    bare_id: i64,
    descriptor: Option<PeerDescriptor>,
    avatar_bytes: Option<Vec<u8>>,
    avatar_delay: Duration,
}

#[cfg(test)]
struct TestPeerTransport {
    dialogs: std::collections::VecDeque<TestPeer>,
    username_result: Option<TestPeer>,
    events: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

#[cfg(test)]
impl PeerDescriptorBackend for TestPeerTransport {
    type Peer = TestPeer;

    fn descriptor(&self, peer: &Self::Peer) -> Option<PeerDescriptor> {
        peer.descriptor.clone()
    }

    async fn peer_avatar(&mut self, peer: &Self::Peer) -> Option<Vec<u8>> {
        self.events
            .lock()
            .expect("event lock")
            .push(format!("avatar:{}:{TEST_AVATAR_TIMEOUT_MS}", peer.label));
        tokio::time::sleep(peer.avatar_delay).await;
        peer.avatar_bytes.clone()
    }
}

#[cfg(test)]
impl DialogPeerBackend for TestPeerTransport {
    async fn next_dialog_peer(&mut self) -> AppResult<Option<Self::Peer>> {
        let peer = self.dialogs.pop_front();
        if let Some(peer) = peer.as_ref() {
            self.events
                .lock()
                .expect("event lock")
                .push(format!("next:{}", peer.label));
        }
        Ok(peer)
    }

    fn bare_id(&self, peer: &Self::Peer) -> Option<i64> {
        Some(peer.bare_id)
    }
}

#[cfg(test)]
impl UsernamePeerBackend for TestPeerTransport {
    async fn resolve_username_peer(&mut self, username: &str) -> AppResult<Option<Self::Peer>> {
        self.events
            .lock()
            .expect("event lock")
            .push(format!("username:{username}"));
        Ok(self.username_result.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use grammers_session::types::PeerKind;
    use tokio::time::{Duration, Instant};

    use super::*;

    const TELEGRAM_KIND_CHANNEL: &str = "channel";
    const TELEGRAM_KIND_SUPERGROUP: &str = "supergroup";
    const TELEGRAM_KIND_GROUP: &str = "group";

    fn descriptor(id: i64, title: &str, source_subtype: &str) -> PeerDescriptor {
        PeerDescriptor {
            external_id: id.to_string(),
            title: title.to_string(),
            source_subtype: source_subtype.to_string(),
            is_member: true,
            username: None,
            access_hash: Some(id * 10),
            avatar_bytes: None,
        }
    }

    fn fake_peer(
        label: &'static str,
        bare_id: i64,
        descriptor: Option<PeerDescriptor>,
        avatar_bytes: Option<Vec<u8>>,
        avatar_delay_ms: u64,
    ) -> TestPeer {
        TestPeer {
            label,
            bare_id,
            descriptor,
            avatar_bytes,
            avatar_delay: Duration::from_millis(avatar_delay_ms),
        }
    }

    #[test]
    fn typed_identity_builds_channel_peer_ref_when_access_hash_exists() {
        let descriptor = descriptor(12345, "Channel", TELEGRAM_KIND_CHANNEL);

        let peer_ref = peer_ref_from_descriptor(&descriptor).expect("channel peer ref");

        assert_eq!(peer_ref.id.kind(), PeerKind::Channel);
        assert_eq!(peer_ref.id.bare_id(), Some(12345));
        assert_eq!(peer_ref.auth.hash(), 123450);
    }

    #[test]
    fn typed_identity_rejects_subtype_peer_kind_mismatch() {
        let mut descriptor = descriptor(12345, "Supergroup", TELEGRAM_KIND_SUPERGROUP);
        descriptor.access_hash = None;

        let error = peer_ref_from_descriptor(&descriptor).expect_err("missing channel hash");

        assert_eq!(error.kind, extractum_core::error::AppErrorKind::Validation);
    }

    #[test]
    fn dialog_lookup_misses_are_not_found() {
        let error = dialog_lookup_not_found_error("12345", None);

        assert_eq!(error.kind, extractum_core::error::AppErrorKind::NotFound);
        assert!(error
            .message
            .contains("not found in this account's dialogs"));
    }

    #[test]
    fn dialog_lookup_not_found_message_explains_numeric_manual_limit() {
        let message = dialog_lookup_not_found_message("12345", None);
        assert!(message.contains("not found in this account's dialogs"));
        assert!(message.contains("Numeric manual adds only work"));
        assert!(message.contains("private Telegram sources"));
    }

    #[test]
    fn validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype() {
        let source = descriptor(123, "Example", TELEGRAM_KIND_SUPERGROUP);

        let error = validate_expected_telegram_source_subtype(&source, Some(TELEGRAM_KIND_CHANNEL))
            .expect_err("expected subtype mismatch");

        assert!(error.message.contains("requested source subtype"));
        let legacy_key = ["telegram", "source", "kind"].join("_");
        assert!(!error.message.contains(&legacy_key));
        assert!(error.message.contains(TELEGRAM_KIND_CHANNEL));
        assert!(error.message.contains(TELEGRAM_KIND_SUPERGROUP));
    }

    #[test]
    fn peer_ref_from_identity_uses_channel_access_hash() {
        let mut descriptor = descriptor(12345, "Channel", TELEGRAM_KIND_CHANNEL);
        descriptor.access_hash = Some(67890);

        let peer_ref = peer_ref_from_descriptor(&descriptor).expect("channel peer ref");

        assert_eq!(peer_ref.id.kind(), PeerKind::Channel);
        assert_eq!(peer_ref.id.bare_id(), Some(12345));
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_identity_uses_supergroup_access_hash() {
        let mut descriptor = descriptor(12345, "Supergroup", TELEGRAM_KIND_SUPERGROUP);
        descriptor.access_hash = Some(67890);

        let peer_ref = peer_ref_from_descriptor(&descriptor).expect("supergroup peer ref");

        assert_eq!(peer_ref.id.kind(), PeerKind::Channel);
        assert_eq!(peer_ref.id.bare_id(), Some(12345));
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_identity_ignores_small_groups_without_supported_identity() {
        let mut descriptor = descriptor(12345, "Small group", TELEGRAM_KIND_GROUP);
        descriptor.access_hash = Some(67890);

        let peer_ref = peer_ref_from_descriptor(&descriptor).expect("ambient chat peer ref");

        assert_eq!(peer_ref.id.kind(), PeerKind::Chat);
        assert_eq!(peer_ref.id.bare_id(), Some(12345));
        assert_eq!(peer_ref.auth, PeerAuth::default());
    }

    #[test]
    fn peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation() {
        let unsupported = descriptor(12345, "Unsupported", "unsupported");
        let unsupported_error =
            peer_ref_from_descriptor(&unsupported).expect_err("unsupported subtype");
        assert_eq!(
            unsupported_error.kind,
            extractum_core::error::AppErrorKind::Validation
        );

        let invalid_id = PeerDescriptor {
            external_id: "not-a-number".to_string(),
            ..descriptor(12345, "Invalid", TELEGRAM_KIND_CHANNEL)
        };
        let invalid_id_error =
            peer_ref_from_descriptor(&invalid_id).expect_err("invalid numeric id");
        assert_eq!(
            invalid_id_error.kind,
            extractum_core::error::AppErrorKind::Validation
        );
    }

    #[tokio::test(start_paused = true)]
    async fn dialog_listing_preserves_dialog_avatar_interleaving_and_budget() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let channel = fake_peer(
            "channel",
            101,
            Some(descriptor(101, "Channel", "channel")),
            Some(vec![0x01]),
            20,
        );
        let user = fake_peer("user", 202, None, Some(vec![0x02]), 0);
        let group = fake_peer(
            "group",
            303,
            Some(descriptor(303, "Group", "group")),
            Some(vec![0x03]),
            0,
        );
        let mut listing = DialogListing {
            backend: DialogListingBackend::Test(TestPeerTransport {
                dialogs: VecDeque::from([channel, user, group]),
                username_result: None,
                events: Arc::clone(&events),
            }),
            avatar_budget_started_at: Instant::now(),
            avatar_budget: Duration::from_millis(5),
        };

        let first = listing
            .next()
            .await
            .expect("first dialog")
            .expect("channel");
        assert_eq!(first.external_id, "101");
        assert_eq!(first.avatar_bytes, Some(vec![0x01]));

        let second = listing.next().await.expect("second dialog").expect("group");
        assert_eq!(second.external_id, "303");
        assert_eq!(second.avatar_bytes, None);
        assert_eq!(listing.next().await.expect("true EOF"), None);
        assert_eq!(
            *events.lock().expect("event lock"),
            vec![
                "next:channel",
                "avatar:channel:750",
                "next:user",
                "next:group",
            ]
        );
    }

    #[tokio::test]
    async fn resolution_primitives_preserve_username_dialog_and_subtype_outcomes() {
        let username_events = Arc::new(Mutex::new(Vec::new()));
        let username_peer = fake_peer(
            "username-channel",
            101,
            Some(descriptor(101, "Username channel", "channel")),
            Some(vec![0x11]),
            0,
        );
        let mut username_transport = TestPeerTransport {
            dialogs: VecDeque::new(),
            username_result: Some(username_peer),
            events: Arc::clone(&username_events),
        };
        let username =
            resolve_username_descriptor(&mut username_transport, "example", Some("channel"))
                .await
                .expect("username resolution")
                .expect("username peer");
        assert_eq!(username.source_subtype, "channel");
        assert_eq!(username.avatar_bytes, Some(vec![0x11]));
        assert_eq!(
            *username_events.lock().expect("event lock"),
            vec!["username:example", "avatar:username-channel:750"]
        );

        let dialog_events = Arc::new(Mutex::new(Vec::new()));
        let wrong_kind = fake_peer(
            "wrong-channel",
            303,
            Some(descriptor(303, "Wrong channel", "channel")),
            Some(vec![0x22]),
            0,
        );
        let matching_group = fake_peer(
            "matching-group",
            303,
            Some(descriptor(303, "Matching group", "group")),
            Some(vec![0x33]),
            0,
        );
        let mut dialog_transport = TestPeerTransport {
            dialogs: VecDeque::from([wrong_kind.clone(), matching_group]),
            username_result: None,
            events: Arc::clone(&dialog_events),
        };
        let dialog = resolve_dialog_descriptor(&mut dialog_transport, 303, Some("group"))
            .await
            .expect("dialog resolution");
        assert_eq!(dialog.source_subtype, "group");
        assert_eq!(dialog.avatar_bytes, Some(vec![0x33]));
        assert_eq!(
            *dialog_events.lock().expect("event lock"),
            vec![
                "next:wrong-channel",
                "next:matching-group",
                "avatar:matching-group:750",
            ]
        );

        let mut wrong_subtype_transport = TestPeerTransport {
            dialogs: VecDeque::from([wrong_kind]),
            username_result: None,
            events: Arc::new(Mutex::new(Vec::new())),
        };
        let wrong_subtype =
            resolve_dialog_descriptor(&mut wrong_subtype_transport, 303, Some("group"))
                .await
                .expect_err("wrong subtype");
        assert_eq!(
            wrong_subtype.kind,
            extractum_core::error::AppErrorKind::Validation
        );

        let mut missing_transport = TestPeerTransport {
            dialogs: VecDeque::new(),
            username_result: None,
            events: Arc::new(Mutex::new(Vec::new())),
        };
        let missing = resolve_dialog_descriptor(&mut missing_transport, 404, Some("group"))
            .await
            .expect_err("missing dialog peer");
        assert_eq!(missing.kind, extractum_core::error::AppErrorKind::NotFound);
    }
}
