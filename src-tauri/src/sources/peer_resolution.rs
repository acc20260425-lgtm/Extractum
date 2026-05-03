use grammers_client::{peer::Peer, tl};
use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::compression::{compress_json_bytes, decompress_bytes};
use super::avatar::{cache_source_avatar, peer_photo_bytes_with_timeout};
use super::types::{
    SourceSyncTarget, TelegramSourceInfo, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
    TELEGRAM_KIND_SUPERGROUP, TELEGRAM_SOURCE_TYPE,
};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SourcePeerResolutionStrategy {
    Username,
    Dialog,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
struct SourcePeerIdentity {
    strategy: SourcePeerResolutionStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    access_hash: Option<i64>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(super) struct SourceMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    peer_identity: Option<SourcePeerIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) avatar_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    added_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    access_hash: Option<i64>,
}

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
    pub(super) telegram_source_kind: String,
    pub(super) is_member: bool,
    pub(super) username: Option<String>,
    access_hash: Option<i64>,
    pub(super) avatar_bytes: Option<Vec<u8>>,
}

pub(crate) struct ResolvedSyncPeer {
    pub(crate) peer: PeerRef,
    pub(crate) refreshed_metadata_zstd: Option<Vec<u8>>,
}

fn parse_username(input: &str) -> String {
    let s = input.trim();
    if let Some(rest) = s.strip_prefix("https://t.me/") {
        return rest.split('/').next().unwrap_or(rest).to_string();
    }
    if let Some(rest) = s.strip_prefix("t.me/") {
        return rest.split('/').next().unwrap_or(rest).to_string();
    }
    s.trim_start_matches('@').to_string()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ManualTelegramSourceRef {
    Username(String),
    NumericId(i64),
}

fn unsupported_manual_source_ref_message(source_ref: &str) -> String {
    format!(
        "Unsupported manual Telegram source reference '{}'. Use @username or t.me/name for public sources. For private Telegram sources, add them from the account's dialogs.",
        source_ref
    )
}

fn unsupported_private_manual_source_ref_message(source_ref: &str) -> String {
    format!(
        "Unsupported private Telegram source reference '{}'. Private invite links and internal t.me/c links are not supported for manual add. Add this source from the account's dialogs instead.",
        source_ref
    )
}

fn parse_supported_manual_telegram_source_ref(
    source_ref: &str,
) -> Result<ManualTelegramSourceRef, String> {
    let trimmed = source_ref.trim();
    if trimmed.is_empty() {
        return Err("Telegram source reference cannot be empty".to_string());
    }

    if let Ok(source_id) = trimmed.parse::<i64>() {
        return Ok(ManualTelegramSourceRef::NumericId(source_id));
    }

    if let Some(rest) = trimmed.strip_prefix('@') {
        let username = rest.trim();
        if username.is_empty() || username.contains('/') || username.starts_with('+') {
            return Err(unsupported_manual_source_ref_message(source_ref));
        }
        return Ok(ManualTelegramSourceRef::Username(username.to_string()));
    }

    if let Some(rest) = trimmed
        .strip_prefix("https://t.me/")
        .or_else(|| trimmed.strip_prefix("http://t.me/"))
        .or_else(|| trimmed.strip_prefix("t.me/"))
    {
        let path = rest.trim_matches('/');
        let first_segment = path.split('/').next().unwrap_or(path).trim();
        if first_segment.is_empty() {
            return Err(unsupported_manual_source_ref_message(source_ref));
        }
        if first_segment.eq_ignore_ascii_case("joinchat")
            || first_segment.eq_ignore_ascii_case("c")
            || first_segment.starts_with('+')
        {
            return Err(unsupported_private_manual_source_ref_message(source_ref));
        }
        return Ok(ManualTelegramSourceRef::Username(first_segment.to_string()));
    }

    let username = parse_username(trimmed);
    if !username.is_empty()
        && !username.contains('/')
        && !username.starts_with('+')
        && !username.chars().all(|char| char.is_ascii_digit())
    {
        return Ok(ManualTelegramSourceRef::Username(username));
    }

    Err(unsupported_manual_source_ref_message(source_ref))
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

fn add_source_resolution_strategy(
    source_ref: &str,
    telegram_source_kind: Option<&str>,
) -> SourcePeerResolutionStrategy {
    if telegram_source_kind.is_some() {
        return SourcePeerResolutionStrategy::Dialog;
    }

    let username = parse_username(source_ref);
    if username.is_empty() || username.chars().all(|char| char.is_ascii_digit()) {
        SourcePeerResolutionStrategy::Dialog
    } else {
        SourcePeerResolutionStrategy::Username
    }
}

pub(super) fn source_metadata_for_added_source(
    source_ref: &str,
    telegram_source_kind: Option<&str>,
    resolved: &ResolvedTelegramSource,
    avatar_cache_key: Option<String>,
) -> SourceMetadata {
    SourceMetadata {
        peer_identity: Some(SourcePeerIdentity {
            strategy: add_source_resolution_strategy(source_ref, telegram_source_kind),
            username: resolved.username.clone(),
            access_hash: resolved.access_hash,
        }),
        avatar_cache_key,
        ..SourceMetadata::default()
    }
}

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
            if source.telegram_source_kind == TELEGRAM_KIND_GROUP =>
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
    expected_kind: Option<&str>,
) -> Result<ResolvedTelegramSource, String> {
    let peer = client
        .resolve_username(username)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Telegram source '{}' not found", source_ref))?;

    let mut source = resolved_telegram_source_from_peer(&peer)
        .ok_or_else(|| "Not a Telegram channel, group, or supergroup".to_string())?;
    validate_expected_telegram_source_kind(&source, expected_kind)?;
    source.avatar_bytes = peer_photo_bytes_with_timeout(client, &peer).await;
    Ok(source)
}

fn dialog_lookup_not_found_message(source_ref: &str, expected_kind: Option<&str>) -> String {
    if expected_kind.is_some() {
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

async fn resolve_telegram_source_from_dialogs(
    client: &grammers_client::Client,
    source_id: i64,
    source_ref: &str,
    expected_kind: Option<&str>,
) -> Result<ResolvedTelegramSource, String> {
    let mut dialogs = client.iter_dialogs();
    let mut found_wrong_kind = false;
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if dialog.peer().id().bare_id() == source_id {
            if let Some(source) = resolved_telegram_source_from_peer(dialog.peer()) {
                if telegram_source_kind_matches(&source, expected_kind)? {
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
        return Err(format!(
            "Telegram source '{}' was found, but it has a different Telegram source kind than the requested source kind",
            source_ref
        ));
    }

    Err(dialog_lookup_not_found_message(source_ref, expected_kind))
}

pub(super) async fn resolve_telegram_source(
    client: &grammers_client::Client,
    source_ref: &str,
    expected_kind: Option<&str>,
) -> Result<ResolvedTelegramSource, String> {
    let trimmed = source_ref.trim();
    if expected_kind.is_none() {
        match parse_supported_manual_telegram_source_ref(trimmed)? {
            ManualTelegramSourceRef::Username(username) => {
                return resolve_telegram_source_by_username(
                    client,
                    &username,
                    source_ref,
                    expected_kind,
                )
                .await
            }
            ManualTelegramSourceRef::NumericId(source_id) => {
                return resolve_telegram_source_from_dialogs(
                    client,
                    source_id,
                    source_ref,
                    expected_kind,
                )
                .await
            }
        }
    }

    let username = parse_username(trimmed);
    if !username.is_empty() && !username.chars().all(|char| char.is_ascii_digit()) {
        return resolve_telegram_source_by_username(client, &username, source_ref, expected_kind)
            .await;
    }

    let Ok(source_id) = trimmed.parse::<i64>() else {
        return Err(format!("Telegram source '{}' not found", source_ref));
    };

    resolve_telegram_source_from_dialogs(client, source_id, source_ref, expected_kind).await
}

fn telegram_source_kind_matches(
    source: &ResolvedTelegramSource,
    expected_kind: Option<&str>,
) -> Result<bool, String> {
    let Some(expected_kind) = expected_kind else {
        return Ok(true);
    };

    ensure_supported_telegram_source_kind(expected_kind)?;
    Ok(source.telegram_source_kind == expected_kind)
}

fn validate_expected_telegram_source_kind(
    source: &ResolvedTelegramSource,
    expected_kind: Option<&str>,
) -> Result<(), String> {
    if telegram_source_kind_matches(source, expected_kind)? {
        Ok(())
    } else {
        Err(format!(
            "Resolved Telegram source has a different Telegram source kind than the requested source kind: expected '{}', got '{}'",
            expected_kind.unwrap_or("unknown"),
            source.telegram_source_kind
        ))
    }
}

fn ensure_supported_telegram_source_kind(kind: &str) -> Result<(), String> {
    match kind {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP | TELEGRAM_KIND_GROUP => Ok(()),
        other => Err(format!("Unsupported telegram_source_kind '{other}'")),
    }
}

fn resolved_telegram_source_from_peer(peer: &Peer) -> Option<ResolvedTelegramSource> {
    telegram_source_info_from_peer(peer).map(|source| ResolvedTelegramSource {
        external_id: source.id.to_string(),
        title: source.title,
        telegram_source_kind: source.telegram_source_kind,
        is_member: source.is_member,
        username: source.username,
        access_hash: peer_access_hash(peer),
        avatar_bytes: None,
    })
}

pub(super) fn telegram_source_info_from_peer(peer: &Peer) -> Option<TelegramSourceInfo> {
    match peer {
        Peer::Channel(channel) => Some(TelegramSourceInfo {
            id: channel.id().bare_id(),
            title: channel.title().to_string(),
            username: channel.username().map(|value| value.to_string()),
            telegram_source_kind: TELEGRAM_KIND_CHANNEL.to_string(),
            is_member: !channel.raw.left,
            photo_data_url: None,
        }),
        Peer::Group(group) => Some(TelegramSourceInfo {
            id: group.id().bare_id(),
            title: group.title().unwrap_or("Untitled group").to_string(),
            username: group.username().map(|value| value.to_string()),
            telegram_source_kind: telegram_group_kind(group).to_string(),
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

pub(super) fn encode_source_metadata(metadata: &SourceMetadata) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(&metadata.normalized()).map_err(|e| e.to_string())?;
    compress_json_bytes(&json)
}

pub(super) fn decode_source_metadata(bytes: Option<&[u8]>) -> Result<SourceMetadata, String> {
    let Some(bytes) = bytes else {
        return Ok(SourceMetadata::default());
    };
    let decoded = decompress_bytes(bytes)?;
    serde_json::from_slice::<SourceMetadata>(&decoded)
        .map(|metadata| metadata.normalized())
        .map_err(|e| e.to_string())
}

async fn resolve_source_peer(
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
) -> Result<PeerRef, String> {
    if source.source_type != TELEGRAM_SOURCE_TYPE {
        return Err(format!(
            "Source {} has unsupported source_type '{}'",
            source.id, source.source_type
        ));
    }

    let telegram_source_id = source.external_id.parse::<i64>().map_err(|_| {
        format!(
            "Invalid external_id '{}' for source {}",
            source.external_id, source.id
        )
    })?;

    let metadata = decode_source_metadata(source.metadata_zstd.as_deref())?;
    for step in source_peer_resolution_plan(&metadata) {
        match step {
            SourcePeerResolutionStep::Username => {
                let Some(username) = metadata
                    .peer_identity
                    .as_ref()
                    .and_then(|identity| identity.username.as_deref())
                else {
                    continue;
                };

                if let Some(peer) = client
                    .resolve_username(username)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    return peer_ref_for_source_kind(
                        &peer,
                        &source.telegram_source_kind,
                        source.id,
                    );
                }
            }
            SourcePeerResolutionStep::StoredPeerIdentity => {
                if let Some(peer_ref) =
                    source_peer_ref_from_identity(source, telegram_source_id, &metadata)?
                {
                    return Ok(peer_ref);
                }
            }
            SourcePeerResolutionStep::DialogScan => {
                let mut dialogs = client.iter_dialogs();
                while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
                    if dialog.peer().id().bare_id() == telegram_source_id {
                        return peer_ref_for_source_kind(
                            dialog.peer(),
                            &source.telegram_source_kind,
                            source.id,
                        );
                    }
                }
            }
        }
    }

    Err(source_peer_resolution_failure(source, &metadata))
}

fn source_peer_ref_from_identity(
    source: &SourceSyncTarget,
    telegram_source_id: i64,
    metadata: &SourceMetadata,
) -> Result<Option<PeerRef>, String> {
    let Some(access_hash) = metadata
        .peer_identity
        .as_ref()
        .and_then(|identity| identity.access_hash)
    else {
        return Ok(None);
    };

    match source.telegram_source_kind.as_str() {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP => Ok(Some(PeerRef {
            id: PeerId::channel(telegram_source_id),
            auth: PeerAuth::from_hash(access_hash),
        })),
        TELEGRAM_KIND_GROUP => Ok(None),
        other => Err(format!(
            "Source {} has unsupported telegram_source_kind '{}'",
            source.id, other
        )),
    }
}

fn peer_ref_for_source_kind(
    peer: &Peer,
    telegram_source_kind: &str,
    source_id: i64,
) -> Result<PeerRef, String> {
    match (telegram_source_kind, peer) {
        (TELEGRAM_KIND_CHANNEL, Peer::Channel(channel)) => Ok(channel.raw.clone().into()),
        (TELEGRAM_KIND_SUPERGROUP, Peer::Group(group)) if group.is_megagroup() => {
            Ok(group.raw.clone().into())
        }
        (TELEGRAM_KIND_GROUP, Peer::Group(group)) if !group.is_megagroup() => {
            Ok(group.raw.clone().into())
        }
        (TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP | TELEGRAM_KIND_GROUP, _) => Err(
            format!(
                "Source {} resolved to a different Telegram source kind than the requested source kind",
                source_id
            ),
        ),
        (other, _) => Err(format!(
            "Source {} has unsupported telegram_source_kind '{}'",
            source_id, other
        )),
    }
}

pub(crate) async fn resolve_and_refresh_peer(
    handle: &AppHandle,
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
    account_id: i64,
) -> Result<ResolvedSyncPeer, String> {
    let peer = resolve_source_peer(client, source).await?;
    let refreshed_metadata_zstd =
        refresh_source_avatar_cache(handle, client, source, account_id, peer).await;

    Ok(ResolvedSyncPeer {
        peer,
        refreshed_metadata_zstd,
    })
}

async fn refresh_source_avatar_cache(
    handle: &AppHandle,
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
    account_id: i64,
    peer_ref: PeerRef,
) -> Option<Vec<u8>> {
    let peer = client.resolve_peer(peer_ref).await.ok()?;
    let bytes = peer_photo_bytes_with_timeout(client, &peer).await?;
    let cache_key = cache_source_avatar(
        handle,
        account_id,
        &source.telegram_source_kind,
        &source.external_id,
        &bytes,
    )
    .ok()
    .flatten()?;

    let mut metadata = decode_source_metadata(source.metadata_zstd.as_deref()).ok()?;
    metadata.avatar_cache_key = Some(cache_key);
    encode_source_metadata(&metadata).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_json_bytes;

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
    fn source_metadata_roundtrip_preserves_peer_identity() {
        let original = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: Some("example".to_string()),
                access_hash: Some(42),
            }),
            avatar_cache_key: Some("1_channel_42.jpg".to_string()),
            ..SourceMetadata::default()
        };

        let encoded = encode_source_metadata(&original).expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(decoded, original);
    }

    #[test]
    fn parse_username_accepts_username_and_t_me_links() {
        assert_eq!(parse_username("@example"), "example");
        assert_eq!(parse_username("t.me/example"), "example");
        assert_eq!(parse_username("https://t.me/example/42"), "example");
    }

    #[test]
    fn parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids() {
        assert_eq!(
            parse_supported_manual_telegram_source_ref("@example"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("t.me/example"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("https://t.me/example/42"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("12345"),
            Ok(ManualTelegramSourceRef::NumericId(12345))
        );
    }

    #[test]
    fn parse_supported_manual_telegram_source_ref_rejects_private_links() {
        for source_ref in [
            "https://t.me/+AAAAAE-example",
            "t.me/joinchat/AAAAAE-example",
            "https://t.me/c/12345/67",
        ] {
            let error = parse_supported_manual_telegram_source_ref(source_ref)
                .expect_err("private/manual ref should be rejected");
            assert!(error.contains("not supported for manual add"));
            assert!(error.contains("dialogs"));
        }
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
    fn validate_expected_telegram_source_kind_reports_requested_and_actual_kind() {
        let source = ResolvedTelegramSource {
            external_id: "123".to_string(),
            title: "Example".to_string(),
            telegram_source_kind: TELEGRAM_KIND_SUPERGROUP.to_string(),
            is_member: true,
            username: Some("example".to_string()),
            access_hash: Some(42),
            avatar_bytes: None,
        };

        let error = validate_expected_telegram_source_kind(&source, Some(TELEGRAM_KIND_CHANNEL))
            .expect_err("expected kind mismatch");

        assert!(error.contains("requested source kind"));
        assert!(error.contains(TELEGRAM_KIND_CHANNEL));
        assert!(error.contains(TELEGRAM_KIND_SUPERGROUP));
    }

    #[test]
    fn peer_ref_from_identity_uses_channel_access_hash() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_CHANNEL.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
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

        assert_eq!(peer_ref.id.bare_id(), 12345);
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_identity_uses_supergroup_access_hash() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_SUPERGROUP.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
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

        assert_eq!(peer_ref.id.bare_id(), 12345);
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_identity_ignores_small_groups_without_supported_identity() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_GROUP.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
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
    fn source_peer_resolution_failure_explains_small_group_dialog_dependency() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_GROUP.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
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
