use grammers_client::{peer::Peer, tl};
use grammers_session::types::PeerRef;
#[cfg(test)]
use grammers_session::types::{PeerAuth, PeerId};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use super::avatar::{cache_source_avatar, peer_photo_bytes_with_timeout};
use super::identity::{
    load_telegram_source_identity, TelegramResolutionStrategy, TelegramSourceIdentity,
};
#[cfg(test)]
use super::types::TELEGRAM_SOURCE_TYPE;
use super::types::{
    SourceSyncTarget, TelegramSourceInfo, TelegramSourceKind, TELEGRAM_KIND_CHANNEL,
    TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
};
use crate::compression::decompress_bytes;
use crate::error::{AppError, AppResult};

use manual_ref::{
    parse_supported_manual_telegram_source_ref, parse_username, ManualTelegramSourceRef,
};

mod manual_ref;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum SourcePeerResolutionStrategy {
    Username,
    Dialog,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(super) struct SourcePeerIdentity {
    pub(super) strategy: SourcePeerResolutionStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) access_hash: Option<i64>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(super) struct SourceMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) peer_identity: Option<SourcePeerIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) avatar_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) added_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) access_hash: Option<i64>,
}

#[cfg(test)]
impl SourcePeerIdentity {
    fn has_username(&self) -> bool {
        self.username
            .as_deref()
            .is_some_and(|username| !username.trim().is_empty())
    }
}

impl SourceMetadata {
    fn normalized(&self) -> Self {
        let mut normalized = self.clone();

        if normalized.peer_identity.is_none() {
            normalized.peer_identity = legacy_peer_identity(
                normalized.username.clone(),
                normalized.added_from.clone(),
                normalized.access_hash,
            );
        }

        normalized.username = None;
        normalized.added_from = None;
        normalized.access_hash = None;
        normalized
    }
}

pub(super) struct ResolvedTelegramSource {
    pub(super) external_id: String,
    pub(super) title: String,
    pub(super) source_subtype: String,
    pub(super) is_member: bool,
    pub(super) username: Option<String>,
    pub(super) access_hash: Option<i64>,
    pub(super) avatar_bytes: Option<Vec<u8>>,
}

pub(crate) struct ResolvedSyncPeer {
    pub(crate) peer: PeerRef,
    pub(crate) refreshed_avatar_cache_key: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourcePeerResolutionStep {
    Username,
    StoredPeerIdentity,
    DialogScan,
}

fn legacy_peer_identity(
    username: Option<String>,
    added_from: Option<String>,
    access_hash: Option<i64>,
) -> Option<SourcePeerIdentity> {
    if username.is_none() && access_hash.is_none() {
        return None;
    }

    let strategy = match added_from
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("dialog") => SourcePeerResolutionStrategy::Dialog,
        Some("username") => SourcePeerResolutionStrategy::Username,
        _ if username.is_some() => SourcePeerResolutionStrategy::Username,
        _ => SourcePeerResolutionStrategy::Dialog,
    };

    Some(SourcePeerIdentity {
        strategy,
        username,
        access_hash,
    })
}

pub(super) fn add_source_resolution_strategy(
    source_ref: &str,
    source_subtype: Option<&str>,
) -> SourcePeerResolutionStrategy {
    if source_subtype.is_some() {
        return SourcePeerResolutionStrategy::Dialog;
    }

    let username = parse_username(source_ref);
    if username.is_empty() || username.chars().all(|char| char.is_ascii_digit()) {
        SourcePeerResolutionStrategy::Dialog
    } else {
        SourcePeerResolutionStrategy::Username
    }
}

#[cfg(test)]
fn source_peer_resolution_plan(metadata: &SourceMetadata) -> Vec<SourcePeerResolutionStep> {
    let Some(identity) = metadata.peer_identity.as_ref() else {
        return vec![SourcePeerResolutionStep::DialogScan];
    };

    let mut plan = Vec::new();
    match identity.strategy {
        SourcePeerResolutionStrategy::Username => {
            if identity.has_username() {
                plan.push(SourcePeerResolutionStep::Username);
            }
        }
        SourcePeerResolutionStrategy::Dialog => {
            if identity.access_hash.is_some() {
                plan.push(SourcePeerResolutionStep::StoredPeerIdentity);
            }
            if identity.has_username() {
                plan.push(SourcePeerResolutionStep::Username);
            }
        }
    }

    plan.push(SourcePeerResolutionStep::DialogScan);
    plan
}

#[cfg(test)]
fn source_peer_resolution_failure(source: &SourceSyncTarget, metadata: &SourceMetadata) -> String {
    match metadata
        .peer_identity
        .as_ref()
        .map(|identity| identity.strategy)
    {
        Some(SourcePeerResolutionStrategy::Username) => {
            let username = metadata
                .peer_identity
                .as_ref()
                .and_then(|identity| identity.username.as_deref())
                .unwrap_or("unknown");
            format!(
                "Source {} could not be resolved from stored username '{}' or compatibility dialog scanning. If the public username changed or the source became private, re-add it from the account's dialogs.",
                source.id, username
            )
        }
        Some(SourcePeerResolutionStrategy::Dialog)
            if source.source_subtype.as_deref() == Some(TELEGRAM_KIND_GROUP) =>
        {
            format!(
                "Source {} could not be resolved from dialogs. Small Telegram groups still depend on dialog availability; if this group disappeared from the account's dialogs, re-add it from that account.",
                source.id
            )
        }
        Some(SourcePeerResolutionStrategy::Dialog) => format!(
            "Source {} could not be resolved from stored peer identity or dialogs. If this private Telegram source disappeared from the account's dialogs, re-add it from that account.",
            source.id
        ),
        None => format!(
            "Source {} could not be resolved from compatibility dialog scanning. If this is a private Telegram source, re-add it from the account's dialogs.",
            source.id
        ),
    }
}

async fn resolve_telegram_source_by_username(
    client: &grammers_client::Client,
    username: &str,
    source_ref: &str,
    expected_subtype: Option<&str>,
) -> AppResult<ResolvedTelegramSource> {
    let peer = client
        .resolve_username(username)
        .await
        .map_err(|e| AppError::network(e.to_string()))?
        .ok_or_else(|| {
            AppError::not_found(format!("Telegram source '{}' not found", source_ref))
        })?;

    let mut source = resolved_telegram_source_from_peer(&peer)
        .ok_or_else(|| AppError::validation("Not a Telegram channel, group, or supergroup"))?;
    validate_expected_telegram_source_subtype(&source, expected_subtype)?;
    source.avatar_bytes = peer_photo_bytes_with_timeout(client, &peer).await;
    Ok(source)
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

async fn resolve_telegram_source_from_dialogs(
    client: &grammers_client::Client,
    source_id: i64,
    source_ref: &str,
    expected_subtype: Option<&str>,
) -> AppResult<ResolvedTelegramSource> {
    let mut dialogs = client.iter_dialogs();
    let mut found_wrong_kind = false;
    while let Some(dialog) = dialogs
        .next()
        .await
        .map_err(|e| AppError::network(e.to_string()))?
    {
        if dialog.peer().id().bare_id() == Some(source_id) {
            if let Some(source) = resolved_telegram_source_from_peer(dialog.peer()) {
                if telegram_source_subtype_matches(&source, expected_subtype)? {
                    let mut source = source;
                    source.avatar_bytes =
                        peer_photo_bytes_with_timeout(client, dialog.peer()).await;
                    return Ok(source);
                }
                found_wrong_kind = true;
            }
        }
    }

    if found_wrong_kind {
        return Err(AppError::validation(format!(
            "Telegram source '{}' was found, but it has a different source subtype than the requested source subtype",
            source_ref
        )));
    }

    Err(dialog_lookup_not_found_error(source_ref, expected_subtype))
}

pub(super) async fn resolve_telegram_source(
    client: &grammers_client::Client,
    source_ref: &str,
    expected_subtype: Option<&str>,
) -> AppResult<ResolvedTelegramSource> {
    let trimmed = source_ref.trim();
    if expected_subtype.is_none() {
        match parse_supported_manual_telegram_source_ref(trimmed)? {
            ManualTelegramSourceRef::Username(username) => {
                return resolve_telegram_source_by_username(
                    client,
                    &username,
                    source_ref,
                    expected_subtype,
                )
                .await
            }
            ManualTelegramSourceRef::NumericId(source_id) => {
                return resolve_telegram_source_from_dialogs(
                    client,
                    source_id,
                    source_ref,
                    expected_subtype,
                )
                .await
            }
        }
    }

    let username = parse_username(trimmed);
    if !username.is_empty() && !username.chars().all(|char| char.is_ascii_digit()) {
        return resolve_telegram_source_by_username(
            client,
            &username,
            source_ref,
            expected_subtype,
        )
        .await;
    }

    let Ok(source_id) = trimmed.parse::<i64>() else {
        return Err(AppError::not_found(format!(
            "Telegram source '{}' not found",
            source_ref
        )));
    };

    resolve_telegram_source_from_dialogs(client, source_id, source_ref, expected_subtype).await
}

fn telegram_source_subtype_matches(
    source: &ResolvedTelegramSource,
    expected_subtype: Option<&str>,
) -> AppResult<bool> {
    let Some(expected_subtype) = expected_subtype else {
        return Ok(true);
    };

    TelegramSourceKind::parse(expected_subtype)?;
    Ok(source.source_subtype == expected_subtype)
}

fn validate_expected_telegram_source_subtype(
    source: &ResolvedTelegramSource,
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

fn resolved_telegram_source_from_peer(peer: &Peer) -> Option<ResolvedTelegramSource> {
    telegram_source_info_from_peer(peer).map(|source| ResolvedTelegramSource {
        external_id: source.id.to_string(),
        title: source.title,
        source_subtype: source.source_subtype,
        is_member: source.is_member,
        username: source.username,
        access_hash: peer_access_hash(peer),
        avatar_bytes: None,
    })
}

pub(super) fn telegram_source_info_from_peer(peer: &Peer) -> Option<TelegramSourceInfo> {
    match peer {
        Peer::Channel(channel) => Some(TelegramSourceInfo {
            id: channel.id().bare_id()?,
            title: channel.title().to_string(),
            username: channel.username().map(|value| value.to_string()),
            source_subtype: TELEGRAM_KIND_CHANNEL.to_string(),
            is_member: !channel.raw.left,
            photo_data_url: None,
        }),
        Peer::Group(group) => Some(TelegramSourceInfo {
            id: group.id().bare_id()?,
            title: group.title().unwrap_or("Untitled group").to_string(),
            username: group.username().map(|value| value.to_string()),
            source_subtype: telegram_group_kind(group).to_string(),
            is_member: telegram_group_is_member(group),
            photo_data_url: None,
        }),
        Peer::User(_) => None,
    }
}

fn telegram_group_kind(group: &grammers_client::peer::Group) -> &'static str {
    if group.is_megagroup() {
        TELEGRAM_KIND_SUPERGROUP
    } else {
        TELEGRAM_KIND_GROUP
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

pub(super) fn decode_source_metadata(bytes: Option<&[u8]>) -> AppResult<SourceMetadata> {
    let Some(bytes) = bytes else {
        return Ok(SourceMetadata::default());
    };
    let decoded = decompress_bytes(bytes).map_err(AppError::internal)?;
    serde_json::from_slice::<SourceMetadata>(&decoded)
        .map(|metadata| metadata.normalized())
        .map_err(|e| AppError::internal(e.to_string()))
}

#[cfg(test)]
fn telegram_source_id_from_sync_target(source: &SourceSyncTarget) -> AppResult<i64> {
    if source.source_type != TELEGRAM_SOURCE_TYPE {
        let subtype = source.source_subtype.as_deref().unwrap_or("unknown");
        return Err(AppError::validation(format!(
            "Source {} has unsupported source_type '{}' and source_subtype '{}'",
            source.id, source.source_type, subtype
        )));
    }

    source.external_id.parse::<i64>().map_err(|_| {
        AppError::validation(format!(
            "Invalid external_id '{}' for source {}",
            source.external_id, source.id
        ))
    })
}

fn typed_peer_resolution_plan(
    identity: &TelegramSourceIdentity,
) -> AppResult<Vec<SourcePeerResolutionStep>> {
    let mut plan = Vec::new();

    if identity.peer_ref()?.is_some() {
        plan.push(SourcePeerResolutionStep::StoredPeerIdentity);
    }

    match identity.resolution_strategy {
        TelegramResolutionStrategy::Username => {
            if identity
                .username
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                plan.push(SourcePeerResolutionStep::Username);
            }
        }
        TelegramResolutionStrategy::Dialog
        | TelegramResolutionStrategy::LegacyMetadata
        | TelegramResolutionStrategy::Unknown => {
            if identity
                .username
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                plan.push(SourcePeerResolutionStep::Username);
            }
        }
    }

    plan.push(SourcePeerResolutionStep::DialogScan);
    Ok(plan)
}

async fn resolve_source_peer_from_typed_identity(
    client: &grammers_client::Client,
    source_id: i64,
    identity: &TelegramSourceIdentity,
) -> AppResult<PeerRef> {
    for step in typed_peer_resolution_plan(identity)? {
        match step {
            SourcePeerResolutionStep::Username => {
                let Some(username) = identity.username.as_deref() else {
                    continue;
                };

                if let Some(peer) = client
                    .resolve_username(username)
                    .await
                    .map_err(|e| AppError::network(e.to_string()))?
                {
                    return peer_ref_for_typed_identity(&peer, source_id, identity);
                }
            }
            SourcePeerResolutionStep::StoredPeerIdentity => {
                if let Some(peer_ref) = identity.peer_ref()? {
                    return Ok(peer_ref);
                }
            }
            SourcePeerResolutionStep::DialogScan => {
                let mut dialogs = client.iter_dialogs();
                while let Some(dialog) = dialogs
                    .next()
                    .await
                    .map_err(|e| AppError::network(e.to_string()))?
                {
                    if dialog.peer().id().bare_id() == Some(identity.peer_id) {
                        return peer_ref_for_typed_identity(dialog.peer(), source_id, identity);
                    }
                }
            }
        }
    }

    Err(AppError::not_found(typed_peer_resolution_failure(
        source_id, identity,
    )))
}

fn typed_peer_resolution_failure(source_id: i64, identity: &TelegramSourceIdentity) -> String {
    match identity.resolution_strategy {
        TelegramResolutionStrategy::Username => {
            let username = identity.username.as_deref().unwrap_or("unknown");
            format!(
                "Source {source_id} could not be resolved from stored username '{username}' or typed dialog scanning. If the public username changed or the source became private, re-add it from the account's dialogs."
            )
        }
        TelegramResolutionStrategy::Dialog
        | TelegramResolutionStrategy::LegacyMetadata
        | TelegramResolutionStrategy::Unknown
            if identity.source_subtype == TelegramSourceKind::Group =>
        {
            format!(
                "Source {source_id} could not be resolved from dialogs. Small Telegram groups still depend on dialog availability; if this group disappeared from the account's dialogs, re-add it from that account."
            )
        }
        TelegramResolutionStrategy::Dialog
        | TelegramResolutionStrategy::LegacyMetadata
        | TelegramResolutionStrategy::Unknown => format!(
            "Source {source_id} could not be resolved from typed peer identity or dialogs. If this private Telegram source disappeared from the account's dialogs, re-add it from that account."
        ),
    }
}

#[cfg(test)]
fn source_peer_ref_from_identity(
    source: &SourceSyncTarget,
    telegram_source_id: i64,
    metadata: &SourceMetadata,
) -> AppResult<Option<PeerRef>> {
    let Some(access_hash) = metadata
        .peer_identity
        .as_ref()
        .and_then(|identity| identity.access_hash)
    else {
        return Ok(None);
    };

    match source.source_subtype.as_deref() {
        Some(TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP) => Ok(Some(PeerRef {
            id: PeerId::channel(telegram_source_id).ok_or_else(|| {
                AppError::validation(format!(
                    "Source {} has invalid Telegram channel peer id {}",
                    source.id, telegram_source_id
                ))
            })?,
            auth: PeerAuth::from_hash(access_hash),
        })),
        Some(TELEGRAM_KIND_GROUP) => Ok(None),
        Some(other) => Err(AppError::validation(format!(
            "Source {} has unsupported source_subtype '{}'",
            source.id, other
        ))),
        None => Err(AppError::validation(format!(
            "Source {} is missing source_subtype",
            source.id
        ))),
    }
}

fn peer_ref_for_source_subtype(
    peer: &Peer,
    source_subtype: &str,
    source_id: i64,
) -> AppResult<PeerRef> {
    match (source_subtype, peer) {
        (TELEGRAM_KIND_CHANNEL, Peer::Channel(channel)) => Ok(channel.raw.clone().into()),
        (TELEGRAM_KIND_SUPERGROUP, Peer::Group(group)) if group.is_megagroup() => {
            Ok(group.raw.clone().into())
        }
        (TELEGRAM_KIND_GROUP, Peer::Group(group)) if !group.is_megagroup() => {
            Ok(group.raw.clone().into())
        }
        (TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP | TELEGRAM_KIND_GROUP, _) => Err(
            AppError::validation(format!(
                "Source {} resolved to a different Telegram source subtype than the requested source subtype",
                source_id
            )),
        ),
        (other, _) => Err(AppError::validation(format!(
            "Source {} has unsupported source_subtype '{}'",
            source_id, other
        ))),
    }
}

fn peer_ref_for_typed_identity(
    peer: &Peer,
    source_id: i64,
    identity: &TelegramSourceIdentity,
) -> AppResult<PeerRef> {
    peer_ref_for_source_subtype(peer, identity.source_subtype.as_str(), source_id)
}

pub(crate) async fn resolve_and_refresh_peer(
    handle: &AppHandle,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
    account_id: i64,
) -> AppResult<ResolvedSyncPeer> {
    let identity = load_telegram_source_identity(pool, source.id).await?;
    let peer = resolve_source_peer_from_typed_identity(client, source.id, &identity).await?;
    let refreshed_avatar_cache_key =
        refresh_source_avatar_cache(handle, client, source, &identity, account_id, peer).await;

    Ok(ResolvedSyncPeer {
        peer,
        refreshed_avatar_cache_key,
    })
}

async fn refresh_source_avatar_cache(
    handle: &AppHandle,
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
    identity: &TelegramSourceIdentity,
    account_id: i64,
    peer_ref: PeerRef,
) -> Option<String> {
    let peer = client.resolve_peer(peer_ref).await.ok()?;
    let bytes = peer_photo_bytes_with_timeout(client, &peer).await?;
    cache_source_avatar(
        handle,
        account_id,
        identity.source_subtype.as_str(),
        &source.external_id,
        &bytes,
    )
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_json_bytes;
    use crate::error::AppErrorKind;
    use crate::sources::identity::{
        TelegramPeerKind, TelegramResolutionStrategy, TelegramSourceIdentity,
    };

    #[test]
    fn typed_identity_builds_channel_peer_ref_when_access_hash_exists() {
        let identity = TelegramSourceIdentity {
            source_id: 101,
            account_id: 1,
            source_subtype: TelegramSourceKind::Channel,
            peer_kind: TelegramPeerKind::Channel,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Username,
            username: Some("example".to_string()),
            access_hash: Some(77),
            avatar_cache_key: None,
        };

        assert!(identity.peer_ref().expect("peer ref check").is_some());
    }

    #[test]
    fn typed_identity_rejects_subtype_peer_kind_mismatch() {
        let identity = TelegramSourceIdentity {
            source_id: 101,
            account_id: 1,
            source_subtype: TelegramSourceKind::Group,
            peer_kind: TelegramPeerKind::Channel,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Dialog,
            username: None,
            access_hash: Some(77),
            avatar_cache_key: None,
        };

        let error = identity.peer_ref().expect_err("mismatch is invalid");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    }

    #[test]
    fn typed_identity_plan_allows_username_resolution_without_access_hash() {
        let identity = TelegramSourceIdentity {
            source_id: 101,
            account_id: 1,
            source_subtype: TelegramSourceKind::Channel,
            peer_kind: TelegramPeerKind::Channel,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Username,
            username: Some("example".to_string()),
            access_hash: None,
            avatar_cache_key: None,
        };

        assert_eq!(
            typed_peer_resolution_plan(&identity).expect("typed plan"),
            vec![
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan
            ]
        );
    }

    #[test]
    fn typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists() {
        let identity = TelegramSourceIdentity {
            source_id: 101,
            account_id: 1,
            source_subtype: TelegramSourceKind::Channel,
            peer_kind: TelegramPeerKind::Channel,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Username,
            username: Some("example".to_string()),
            access_hash: Some(77),
            avatar_cache_key: None,
        };

        assert_eq!(
            typed_peer_resolution_plan(&identity).expect("typed plan"),
            vec![
                SourcePeerResolutionStep::StoredPeerIdentity,
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan
            ]
        );
    }

    #[test]
    fn typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists() {
        let identity = TelegramSourceIdentity {
            source_id: 101,
            account_id: 1,
            source_subtype: TelegramSourceKind::Channel,
            peer_kind: TelegramPeerKind::Channel,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Dialog,
            username: Some("example".to_string()),
            access_hash: Some(77),
            avatar_cache_key: None,
        };

        assert_eq!(
            typed_peer_resolution_plan(&identity).expect("typed plan"),
            vec![
                SourcePeerResolutionStep::StoredPeerIdentity,
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan
            ]
        );
    }

    #[test]
    fn typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists() {
        let identity = TelegramSourceIdentity {
            source_id: 101,
            account_id: 1,
            source_subtype: TelegramSourceKind::Supergroup,
            peer_kind: TelegramPeerKind::Channel,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Dialog,
            username: Some("example".to_string()),
            access_hash: Some(77),
            avatar_cache_key: None,
        };

        assert_eq!(
            typed_peer_resolution_plan(&identity).expect("typed plan"),
            vec![
                SourcePeerResolutionStep::StoredPeerIdentity,
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan
            ]
        );
    }

    #[test]
    fn typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan() {
        let identity = TelegramSourceIdentity {
            source_id: 101,
            account_id: 1,
            source_subtype: TelegramSourceKind::Group,
            peer_kind: TelegramPeerKind::Chat,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Dialog,
            username: None,
            access_hash: Some(77),
            avatar_cache_key: None,
        };

        assert_eq!(
            typed_peer_resolution_plan(&identity).expect("typed plan"),
            vec![SourcePeerResolutionStep::DialogScan]
        );
    }

    #[test]
    fn typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing() {
        let identity = TelegramSourceIdentity {
            source_id: 101,
            account_id: 1,
            source_subtype: TelegramSourceKind::Channel,
            peer_kind: TelegramPeerKind::Channel,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Dialog,
            username: Some("example".to_string()),
            access_hash: None,
            avatar_cache_key: None,
        };

        assert_eq!(
            typed_peer_resolution_plan(&identity).expect("typed plan"),
            vec![
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan
            ]
        );
    }

    #[test]
    fn source_metadata_decodes_old_username_only_payloads() {
        let encoded = compress_json_bytes(br#"{"username":"example"}"#).expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(
            decoded.peer_identity,
            Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Username,
                username: Some("example".to_string()),
                access_hash: None,
            })
        );
        assert_eq!(decoded.username, None);
        assert_eq!(decoded.added_from, None);
        assert_eq!(decoded.access_hash, None);
        assert_eq!(decoded.avatar_cache_key, None);
    }

    #[test]
    fn source_metadata_decodes_old_dialog_payloads_into_peer_identity() {
        let encoded = compress_json_bytes(
            br#"{"username":"example","added_from":"dialog","access_hash":42,"avatar_cache_key":"1_channel_42.jpg"}"#,
        )
        .expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(
            decoded.peer_identity,
            Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: Some("example".to_string()),
                access_hash: Some(42),
            })
        );
        assert_eq!(decoded.username, None);
        assert_eq!(decoded.added_from, None);
        assert_eq!(decoded.access_hash, None);
        assert_eq!(
            decoded.avatar_cache_key.as_deref(),
            Some("1_channel_42.jpg")
        );
    }

    #[test]
    fn source_metadata_decodes_typed_peer_identity_payloads() {
        let expected = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: Some("example".to_string()),
                access_hash: Some(42),
            }),
            avatar_cache_key: Some("1_channel_42.jpg".to_string()),
            ..SourceMetadata::default()
        };

        let encoded = compress_json_bytes(
            br#"{"peer_identity":{"strategy":"dialog","username":"example","access_hash":42},"avatar_cache_key":"1_channel_42.jpg"}"#,
        )
        .expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(decoded, expected);
    }

    #[test]
    fn source_metadata_decode_failures_are_internal() {
        let error =
            decode_source_metadata(Some(b"not zstd metadata")).expect_err("decode should fail");

        assert_eq!(error.kind, AppErrorKind::Internal);
    }

    #[test]
    fn dialog_lookup_misses_are_not_found() {
        let error = dialog_lookup_not_found_error("12345", None);

        assert_eq!(error.kind, AppErrorKind::NotFound);
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
    fn add_source_resolution_strategy_distinguishes_username_and_dialog_flows() {
        assert_eq!(
            add_source_resolution_strategy("@example", None),
            SourcePeerResolutionStrategy::Username
        );
        assert_eq!(
            add_source_resolution_strategy("t.me/example", None),
            SourcePeerResolutionStrategy::Username
        );
        assert_eq!(
            add_source_resolution_strategy("12345", None),
            SourcePeerResolutionStrategy::Dialog
        );
        assert_eq!(
            add_source_resolution_strategy("@example", Some(TELEGRAM_KIND_CHANNEL)),
            SourcePeerResolutionStrategy::Dialog
        );
    }

    #[test]
    fn source_peer_resolution_plan_prefers_explicit_strategy_order() {
        let dialog_metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: Some("example".to_string()),
                access_hash: Some(42),
            }),
            ..SourceMetadata::default()
        };
        assert_eq!(
            source_peer_resolution_plan(&dialog_metadata),
            vec![
                SourcePeerResolutionStep::StoredPeerIdentity,
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan,
            ]
        );

        let username_metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Username,
                username: Some("example".to_string()),
                access_hash: Some(42),
            }),
            ..SourceMetadata::default()
        };
        assert_eq!(
            source_peer_resolution_plan(&username_metadata),
            vec![
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan,
            ]
        );
    }

    #[test]
    fn validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype() {
        let source = ResolvedTelegramSource {
            external_id: "123".to_string(),
            title: "Example".to_string(),
            source_subtype: TELEGRAM_KIND_SUPERGROUP.to_string(),
            is_member: true,
            username: Some("example".to_string()),
            access_hash: Some(42),
            avatar_bytes: None,
        };

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
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some(TELEGRAM_KIND_CHANNEL.to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };
        let metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: Some(67890),
            }),
            ..SourceMetadata::default()
        };

        let peer_ref = source_peer_ref_from_identity(&source, 12345, &metadata)
            .expect("metadata peer ref")
            .expect("peer ref");

        assert_eq!(peer_ref.id.bare_id(), Some(12345));
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_identity_uses_supergroup_access_hash() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some(TELEGRAM_KIND_SUPERGROUP.to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };
        let metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: Some(67890),
            }),
            ..SourceMetadata::default()
        };

        let peer_ref = source_peer_ref_from_identity(&source, 12345, &metadata)
            .expect("metadata peer ref")
            .expect("peer ref");

        assert_eq!(peer_ref.id.bare_id(), Some(12345));
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_identity_ignores_small_groups_without_supported_identity() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some(TELEGRAM_KIND_GROUP.to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };
        let metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: Some(67890),
            }),
            ..SourceMetadata::default()
        };

        let peer_ref =
            source_peer_ref_from_identity(&source, 12345, &metadata).expect("metadata peer ref");

        assert!(peer_ref.is_none());
    }

    #[test]
    fn peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some("unsupported".to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };
        let metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: Some(67890),
            }),
            ..SourceMetadata::default()
        };

        let error = source_peer_ref_from_identity(&source, 12345, &metadata)
            .expect_err("unsupported kind should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
    }

    #[test]
    fn source_peer_input_rejects_unsupported_source_type_as_validation() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: "rss".to_string(),
            source_subtype: Some("feed".to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };

        let error = telegram_source_id_from_sync_target(&source)
            .expect_err("unsupported source type should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
    }

    #[test]
    fn source_peer_input_rejects_malformed_external_id_as_validation() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some(TELEGRAM_KIND_CHANNEL.to_string()),
            account_id: Some(1),
            external_id: "not-a-number".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };

        let error = telegram_source_id_from_sync_target(&source)
            .expect_err("malformed external id should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
    }

    #[test]
    fn source_peer_resolution_failure_explains_small_group_dialog_dependency() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some(TELEGRAM_KIND_GROUP.to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };
        let metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: None,
            }),
            ..SourceMetadata::default()
        };

        let message = source_peer_resolution_failure(&source, &metadata);
        assert!(message.contains("Small Telegram groups"));
        assert!(message.contains("dialogs"));
    }
}
