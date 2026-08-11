use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use super::avatar::cache_source_avatar;
use super::identity::{
    load_telegram_source_identity, TelegramPeerKind, TelegramResolutionStrategy,
    TelegramSourceIdentity,
};
use super::types::{SourceSyncTarget, TelegramSourceKind};
#[cfg(test)]
use super::types::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_SOURCE_TYPE};
use crate::compression::decompress_bytes;
use crate::error::{AppError, AppResult};
use crate::telegram_impl::{PeerDescriptor, TelegramClientHandle};

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

pub(crate) struct ResolvedSyncPeer {
    pub(crate) descriptor: PeerDescriptor,
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

pub(super) async fn resolve_telegram_source(
    client: &TelegramClientHandle,
    source_ref: &str,
    expected_subtype: Option<&str>,
) -> AppResult<PeerDescriptor> {
    let trimmed = source_ref.trim();
    if expected_subtype.is_none() {
        match parse_supported_manual_telegram_source_ref(trimmed)? {
            ManualTelegramSourceRef::Username(username) => {
                return client
                    .resolve_username(&username, expected_subtype)
                    .await?
                    .ok_or_else(|| {
                        AppError::not_found(format!("Telegram source '{}' not found", source_ref))
                    })
            }
            ManualTelegramSourceRef::NumericId(source_id) => {
                return client
                    .resolve_dialog_peer(source_id, expected_subtype)
                    .await
            }
        }
    }

    let username = parse_username(trimmed);
    if !username.is_empty() && !username.chars().all(|char| char.is_ascii_digit()) {
        return client
            .resolve_username(&username, expected_subtype)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!("Telegram source '{}' not found", source_ref))
            });
    }

    let Ok(source_id) = trimmed.parse::<i64>() else {
        return Err(AppError::not_found(format!(
            "Telegram source '{}' not found",
            source_ref
        )));
    };

    client
        .resolve_dialog_peer(source_id, expected_subtype)
        .await
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

    let has_stored_identity = match (identity.peer_kind, identity.source_subtype) {
        (
            TelegramPeerKind::Channel,
            TelegramSourceKind::Channel | TelegramSourceKind::Supergroup,
        ) => identity.access_hash.is_some(),
        (TelegramPeerKind::Chat, TelegramSourceKind::Group) => false,
        _ => {
            return Err(AppError::validation(format!(
                "Source {} has inconsistent Telegram typed identity",
                identity.source_id
            )))
        }
    };
    if has_stored_identity {
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

trait TypedPeerResolutionBackend {
    async fn resolve_username(
        &mut self,
        username: &str,
        expected_subtype: &str,
    ) -> AppResult<Option<PeerDescriptor>>;

    async fn resolve_dialog_peer(
        &mut self,
        peer_id: i64,
        expected_subtype: &str,
    ) -> AppResult<PeerDescriptor>;
}

struct LiveTypedPeerResolutionBackend<'a>(&'a TelegramClientHandle);

impl TypedPeerResolutionBackend for LiveTypedPeerResolutionBackend<'_> {
    async fn resolve_username(
        &mut self,
        username: &str,
        expected_subtype: &str,
    ) -> AppResult<Option<PeerDescriptor>> {
        self.0
            .resolve_username(username, Some(expected_subtype))
            .await
    }

    async fn resolve_dialog_peer(
        &mut self,
        peer_id: i64,
        expected_subtype: &str,
    ) -> AppResult<PeerDescriptor> {
        self.0
            .resolve_dialog_peer(peer_id, Some(expected_subtype))
            .await
    }
}

async fn resolve_source_peer_from_typed_identity(
    client: &TelegramClientHandle,
    source: &SourceSyncTarget,
    identity: &TelegramSourceIdentity,
) -> AppResult<(PeerDescriptor, bool)> {
    let mut backend = LiveTypedPeerResolutionBackend(client);
    resolve_source_peer_from_typed_identity_with(&mut backend, source, identity).await
}

async fn resolve_source_peer_from_typed_identity_with<B>(
    backend: &mut B,
    source: &SourceSyncTarget,
    identity: &TelegramSourceIdentity,
) -> AppResult<(PeerDescriptor, bool)>
where
    B: TypedPeerResolutionBackend,
{
    for step in typed_peer_resolution_plan(identity)? {
        match step {
            SourcePeerResolutionStep::Username => {
                let Some(username) = identity.username.as_deref() else {
                    continue;
                };

                if let Some(descriptor) = backend
                    .resolve_username(username, identity.source_subtype.as_str())
                    .await?
                {
                    return Ok((descriptor, false));
                }
            }
            SourcePeerResolutionStep::StoredPeerIdentity => {
                return Ok((
                    peer_descriptor_from_stored_identity(source, identity)?,
                    true,
                ));
            }
            SourcePeerResolutionStep::DialogScan => {
                match backend
                    .resolve_dialog_peer(identity.peer_id, identity.source_subtype.as_str())
                    .await
                {
                    Ok(descriptor) => return Ok((descriptor, false)),
                    Err(error) if error.kind == crate::error::AppErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
    }

    Err(AppError::not_found(typed_peer_resolution_failure(
        source.id, identity,
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

fn peer_descriptor_from_stored_identity(
    source: &SourceSyncTarget,
    identity: &TelegramSourceIdentity,
) -> AppResult<PeerDescriptor> {
    match (
        identity.peer_kind,
        identity.source_subtype,
        identity.access_hash,
    ) {
        (
            TelegramPeerKind::Channel,
            TelegramSourceKind::Channel | TelegramSourceKind::Supergroup,
            Some(_),
        ) => {}
        _ => {
            return Err(AppError::validation(format!(
                "Source {} does not have a reconstructible stored Telegram identity",
                source.id
            )))
        }
    }

    Ok(PeerDescriptor {
        external_id: source.external_id.clone(),
        title: source
            .title
            .clone()
            .unwrap_or_else(|| "Untitled Telegram source".to_string()),
        source_subtype: identity.source_subtype.as_str().to_string(),
        is_member: source.is_member,
        username: identity.username.clone(),
        access_hash: identity.access_hash,
        avatar_bytes: None,
    })
}

pub(crate) async fn resolve_and_refresh_peer(
    handle: &AppHandle,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &TelegramClientHandle,
    source: &SourceSyncTarget,
    account_id: i64,
) -> AppResult<ResolvedSyncPeer> {
    let identity = load_telegram_source_identity(pool, source.id).await?;
    let (descriptor, fetch_stored_avatar) =
        resolve_source_peer_from_typed_identity(client, source, &identity).await?;
    let refreshed_avatar_cache_key = refresh_source_avatar_cache(
        handle,
        client,
        source,
        &identity,
        account_id,
        &descriptor,
        fetch_stored_avatar,
    )
    .await;

    Ok(ResolvedSyncPeer {
        descriptor,
        refreshed_avatar_cache_key,
    })
}

async fn refresh_source_avatar_cache(
    handle: &AppHandle,
    client: &TelegramClientHandle,
    source: &SourceSyncTarget,
    identity: &TelegramSourceIdentity,
    account_id: i64,
    descriptor: &PeerDescriptor,
    fetch_stored_avatar: bool,
) -> Option<String> {
    let bytes = if fetch_stored_avatar {
        client.peer_avatar_bytes(descriptor).await?
    } else {
        descriptor.avatar_bytes.clone()?
    };
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

    #[derive(Default)]
    struct ExhaustedPeerResolver {
        events: Vec<String>,
    }

    impl TypedPeerResolutionBackend for ExhaustedPeerResolver {
        async fn resolve_username(
            &mut self,
            username: &str,
            expected_subtype: &str,
        ) -> AppResult<Option<PeerDescriptor>> {
            self.events
                .push(format!("username:{username}:{expected_subtype}"));
            Ok(None)
        }

        async fn resolve_dialog_peer(
            &mut self,
            peer_id: i64,
            expected_subtype: &str,
        ) -> AppResult<PeerDescriptor> {
            self.events
                .push(format!("dialog:{peer_id}:{expected_subtype}"));
            Err(AppError::not_found("deterministic dialog miss"))
        }
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
    fn source_peer_input_rejects_unsupported_source_type_as_validation() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: "rss".to_string(),
            source_subtype: Some("feed".to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            is_member: true,
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
            is_member: true,
            last_sync_state: None,
        };

        let error = telegram_source_id_from_sync_target(&source)
            .expect_err("malformed external id should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
    }

    #[tokio::test]
    async fn source_peer_resolution_failure_explains_small_group_dialog_dependency() {
        let username_source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some(TELEGRAM_KIND_CHANNEL.to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            is_member: true,
            last_sync_state: None,
        };
        let username_identity = TelegramSourceIdentity {
            source_id: username_source.id,
            account_id: 1,
            source_subtype: TelegramSourceKind::Channel,
            peer_kind: TelegramPeerKind::Channel,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Username,
            username: Some("example".to_string()),
            access_hash: None,
            avatar_cache_key: None,
        };
        let mut username_backend = ExhaustedPeerResolver::default();

        let username_error = resolve_source_peer_from_typed_identity_with(
            &mut username_backend,
            &username_source,
            &username_identity,
        )
        .await
        .expect_err("username and dialog exhaustion");

        assert_eq!(username_error.kind, AppErrorKind::NotFound);
        assert!(username_error.message.contains("stored username 'example'"));
        assert!(username_error.message.contains("typed dialog scanning"));
        assert_eq!(
            username_backend.events,
            vec!["username:example:channel", "dialog:12345:channel"]
        );

        let group_source = SourceSyncTarget {
            id: 8,
            source_subtype: Some(TELEGRAM_KIND_GROUP.to_string()),
            ..username_source
        };
        let group_identity = TelegramSourceIdentity {
            source_id: group_source.id,
            account_id: 1,
            source_subtype: TelegramSourceKind::Group,
            peer_kind: TelegramPeerKind::Chat,
            peer_id: 12345,
            resolution_strategy: TelegramResolutionStrategy::Dialog,
            username: None,
            access_hash: None,
            avatar_cache_key: None,
        };
        let mut group_backend = ExhaustedPeerResolver::default();

        let group_error = resolve_source_peer_from_typed_identity_with(
            &mut group_backend,
            &group_source,
            &group_identity,
        )
        .await
        .expect_err("small-group dialog miss");

        assert_eq!(group_error.kind, AppErrorKind::NotFound);
        assert!(group_error.message.contains("Small Telegram groups"));
        assert!(group_error.message.contains("dialogs"));
        assert_eq!(group_backend.events, vec!["dialog:12345:group"]);
    }
}
