use grammers_session::types::{PeerKind, PeerRef};
use serde::Serialize;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::media::extract_item_payload;
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;

use super::identity_repair::{require_source_identity_ready, SourceIdentityRepairState};
use super::items::{
    build_raw_payload, extract_telegram_context, insert_telegram_source_item, message_author,
    SourceItemInsert,
};
use super::peer_resolution::resolve_and_refresh_peer;
use super::refresh_forum_topics;
use super::settings::{
    initial_sync_policy_label, load_sync_settings_from_pool, InitialSyncMode, SyncSettingsRecord,
    SECONDS_PER_DAY,
};
use super::store::load_source;
use super::types::{
    now_secs, SourceSyncTarget, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,
    TELEGRAM_PEER_KIND_CHANNEL, TELEGRAM_PEER_KIND_CHAT, TELEGRAM_PEER_KIND_USER,
};

#[derive(Serialize)]
pub struct SyncResult {
    pub inserted: i64,
    pub skipped: i64,
    pub last_message_id: Option<i64>,
    pub initial_sync_policy_applied: Option<String>,
    pub warnings: Vec<String>,
}

struct SyncPolicy {
    previous_last_sync: i64,
    initial_sync_settings: Option<SyncSettingsRecord>,
    initial_sync_policy_applied: Option<String>,
    initial_sync_cutoff: Option<i64>,
}

struct IngestOutcome {
    inserted: i64,
    skipped: i64,
    max_message_id: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SyncProvider {
    Telegram,
}

fn sync_provider_for_source(source: &SourceSyncTarget) -> AppResult<SyncProvider> {
    match source.source_type.as_str() {
        crate::sources::types::TELEGRAM_SOURCE_TYPE => Ok(SyncProvider::Telegram),
        other => Err(AppError::validation(format!(
            "Source {} with source_type '{}' is not syncable",
            source.id, other
        ))),
    }
}

async fn determine_sync_policy(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &SourceSyncTarget,
) -> AppResult<SyncPolicy> {
    let previous_last_sync = source.last_sync_state.unwrap_or(0);
    let initial_sync_settings = if previous_last_sync == 0 {
        Some(load_sync_settings_from_pool(pool).await?)
    } else {
        None
    };
    let initial_sync_policy_applied = initial_sync_settings
        .as_ref()
        .map(initial_sync_policy_label);
    let initial_sync_cutoff =
        initial_sync_settings
            .as_ref()
            .and_then(|settings| match settings.initial_sync_mode {
                InitialSyncMode::RecentDays => {
                    Some(now_secs() - settings.initial_sync_value * SECONDS_PER_DAY)
                }
                InitialSyncMode::RecentMessages => None,
            });

    Ok(SyncPolicy {
        previous_last_sync,
        initial_sync_settings,
        initial_sync_policy_applied,
        initial_sync_cutoff,
    })
}

async fn persist_items(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &grammers_client::Client,
    peer: PeerRef,
    source: &SourceSyncTarget,
    sync_policy: &SyncPolicy,
) -> AppResult<IngestOutcome> {
    let mut inserted = 0_i64;
    let mut skipped = 0_i64;
    let mut max_message_id = sync_policy.previous_last_sync;
    let mut messages = if let Some(settings) = sync_policy.initial_sync_settings.as_ref() {
        match settings.initial_sync_mode {
            InitialSyncMode::RecentMessages => client
                .iter_messages(peer)
                .limit(settings.initial_sync_value as usize),
            InitialSyncMode::RecentDays => client.iter_messages(peer),
        }
    } else {
        client.iter_messages(peer)
    };

    while let Some(message) = messages
        .next()
        .await
        .map_err(|e| AppError::network(e.to_string()))?
    {
        let message_id = i64::from(message.id());
        if sync_policy.previous_last_sync > 0 && message_id <= sync_policy.previous_last_sync {
            break;
        }
        let published_at = message.date().timestamp();
        if let Some(cutoff) = sync_policy.initial_sync_cutoff {
            if published_at < cutoff {
                break;
            }
        }

        if message_id > max_message_id {
            max_message_id = message_id;
        }

        let item_payload = match extract_item_payload(&message) {
            Some(payload) => payload,
            None => {
                skipped += 1;
                continue;
            }
        };

        let author = message_author(&message);
        let telegram_context = extract_telegram_context(&message);
        let raw_data = build_raw_payload(&message, &source.title, &author, &item_payload)?;

        let identity = fallback_message_identity(peer, message_id)?;
        let telegram_identity = Some(identity.clone());
        let inserted_item = insert_telegram_source_item(
            pool,
            source.id,
            identity,
            SourceItemInsert {
                external_id: message_id.to_string(),
                item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
                author,
                published_at,
                payload: item_payload,
                raw_data,
                telegram_context,
                telegram_identity,
            },
        )
        .await?;

        if inserted_item {
            inserted += 1;
        } else {
            skipped += 1;
        }
    }

    Ok(IngestOutcome {
        inserted,
        skipped,
        max_message_id,
    })
}

fn fallback_message_identity(
    fallback_peer: grammers_session::types::PeerRef,
    telegram_message_id: i64,
) -> AppResult<TelegramMessageIdentity> {
    let history_peer_kind = match fallback_peer.id.kind() {
        PeerKind::User => TELEGRAM_PEER_KIND_USER,
        PeerKind::Chat => TELEGRAM_PEER_KIND_CHAT,
        PeerKind::Channel => TELEGRAM_PEER_KIND_CHANNEL,
    }
    .to_string();
    let history_peer_id = fallback_peer.id.bare_id().ok_or_else(|| {
        AppError::validation("Telegram self-user peer cannot be used as message history peer")
    })?;

    Ok(TelegramMessageIdentity {
        history_peer_kind,
        history_peer_id,
        telegram_message_id,
        migration_domain: None,
        is_migrated_history: false,
    })
}

#[cfg(test)]
fn fallback_message_identity_for_test(
    fallback_peer: grammers_session::types::PeerRef,
    telegram_message_id: i64,
) -> TelegramMessageIdentity {
    fallback_message_identity(fallback_peer, telegram_message_id).expect("valid fallback peer")
}

pub(crate) async fn finalize_sync(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &SourceSyncTarget,
    previous_last_sync: i64,
    max_message_id: i64,
    refreshed_avatar_cache_key: Option<String>,
) -> AppResult<Option<i64>> {
    let sync_completed_at = now_secs();
    let last_sync_state = if max_message_id > previous_last_sync {
        Some(max_message_id)
    } else {
        source.last_sync_state
    };

    sqlx::query("UPDATE sources SET last_sync_state = ?, last_synced_at = ? WHERE id = ?")
        .bind(last_sync_state)
        .bind(sync_completed_at)
        .bind(source.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    if let Some(cache_key) = refreshed_avatar_cache_key {
        sqlx::query(
            "UPDATE telegram_sources SET avatar_cache_key = ?, updated_at = strftime('%s','now'), identity_refreshed_at = strftime('%s','now') WHERE source_id = ?",
        )
        .bind(cache_key)
        .bind(source.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;
    }

    Ok(last_sync_state)
}

#[tauri::command]
pub async fn sync_source(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, TelegramState>,
    ingest_locks: tauri::State<'_, SourceIngestLocks>,
    source_id: i64,
) -> AppResult<SyncResult> {
    require_source_identity_ready(repair_state.inner()).await?;
    let _ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::Sync)
        .await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;

    let provider = sync_provider_for_source(&source)?;
    match provider {
        SyncProvider::Telegram => sync_telegram_source(handle, state, source).await,
    }
}

async fn sync_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    source: SourceSyncTarget,
) -> AppResult<SyncResult> {
    let pool = get_pool(&handle).await?;
    let source_id = source.id;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;

    let runtime = crate::telegram::get_authorized_runtime(&state, account_id).await?;
    let client = runtime.client;
    let resolved_peer =
        resolve_and_refresh_peer(&handle, &pool, &client, &source, account_id).await?;
    let forum_topic_warnings =
        refresh_forum_topics(&pool, &client, resolved_peer.peer, &source).await;
    let sync_policy = determine_sync_policy(&pool, &source).await?;
    let ingest = persist_items(&pool, &client, resolved_peer.peer, &source, &sync_policy).await?;
    let last_sync_state = finalize_sync(
        &pool,
        &source,
        sync_policy.previous_last_sync,
        ingest.max_message_id,
        resolved_peer.refreshed_avatar_cache_key,
    )
    .await?;

    Ok(SyncResult {
        inserted: ingest.inserted,
        skipped: ingest.skipped,
        last_message_id: last_sync_state,
        initial_sync_policy_applied: sync_policy.initial_sync_policy_applied,
        warnings: forum_topic_warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        determine_sync_policy, fallback_message_identity_for_test, finalize_sync,
        sync_provider_for_source, SyncProvider,
    };
    use crate::sources::store::load_source;
    use crate::sources::test_support::memory_pool_with_sources;
    use crate::sources::types::{SourceSyncTarget, TELEGRAM_KIND_CHANNEL, TELEGRAM_SOURCE_TYPE};

    #[test]
    fn fallback_peer_identity_uses_telegram_history_peer_vocabulary() {
        use grammers_session::types::{PeerAuth, PeerId, PeerRef};

        let identity = fallback_message_identity_for_test(
            PeerRef {
                id: PeerId::channel(12345).expect("valid channel peer id"),
                auth: PeerAuth::from_hash(99),
            },
            42,
        );

        assert_eq!(identity.history_peer_kind, "channel");
        assert_eq!(identity.history_peer_id, 12345);
        assert_eq!(identity.telegram_message_id, 42);
        assert_eq!(identity.migration_domain, None);
        assert!(!identity.is_migrated_history);
    }

    #[tokio::test]
    async fn determine_sync_policy_only_applies_initial_settings_on_first_sync() {
        let pool = memory_pool_with_sources().await;
        let source = SourceSyncTarget {
            id: 1,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some(TELEGRAM_KIND_CHANNEL.to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };

        let initial = determine_sync_policy(&pool, &source)
            .await
            .expect("determine initial policy");
        assert_eq!(initial.previous_last_sync, 0);
        assert_eq!(
            initial.initial_sync_policy_applied.as_deref(),
            Some("last 500 messages")
        );
        assert!(initial.initial_sync_settings.is_some());
        assert_eq!(initial.initial_sync_cutoff, None);

        let incremental = determine_sync_policy(
            &pool,
            &SourceSyncTarget {
                last_sync_state: Some(77),
                ..source
            },
        )
        .await
        .expect("determine incremental policy");
        assert_eq!(incremental.previous_last_sync, 77);
        assert!(incremental.initial_sync_settings.is_none());
        assert!(incremental.initial_sync_policy_applied.is_none());
        assert_eq!(incremental.initial_sync_cutoff, None);
    }

    #[test]
    fn sync_provider_accepts_telegram_sources() {
        let source = SourceSyncTarget {
            id: 1,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some(TELEGRAM_KIND_CHANNEL.to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            last_sync_state: None,
        };

        assert_eq!(
            sync_provider_for_source(&source).unwrap(),
            SyncProvider::Telegram
        );
    }

    #[test]
    fn sync_provider_rejects_manual_youtube_video_sources() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: "youtube".to_string(),
            source_subtype: Some("video".to_string()),
            account_id: None,
            external_id: "dQw4w9WgXcQ".to_string(),
            title: Some("Demo video".to_string()),
            last_sync_state: None,
        };

        let error = sync_provider_for_source(&source).expect_err("manual video is not syncable");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("Source 7"));
        assert!(error.message.contains("youtube"));
        assert!(error.message.contains("not syncable"));
    }

    #[tokio::test]
    async fn finalize_sync_updates_source_state_and_typed_avatar_cache() {
        let pool = memory_pool_with_sources().await;
        sqlx::query(
            r#"
            INSERT INTO sources (
                id,
                source_type,
                source_subtype,
                account_id,
                external_id,
                title,
                metadata_zstd,
                last_sync_state,
                last_synced_at,
                is_active,
                is_member,
                created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(TELEGRAM_SOURCE_TYPE)
        .bind(TELEGRAM_KIND_CHANNEL)
        .bind(1_i64)
        .bind("12345")
        .bind("Example")
        .bind(None::<Vec<u8>>)
        .bind(5_i64)
        .bind(10_i64)
        .bind(1_i64)
        .bind(1_i64)
        .bind(20_i64)
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash, avatar_cache_key
            )
            VALUES (1, 1, 'channel', 'channel', 12345, 'username', 'before', 77, 'old.jpg')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert typed identity");

        let source = load_source(&pool, 1).await.expect("load source");

        let last_sync_state = finalize_sync(
            &pool,
            &source,
            5,
            9,
            Some("1_channel_12345.jpg".to_string()),
        )
        .await
        .expect("finalize sync");
        assert_eq!(last_sync_state, Some(9));

        let row: (Option<i64>, Option<i64>, Option<Vec<u8>>, Option<String>) = sqlx::query_as(
            r#"
            SELECT s.last_sync_state, s.last_synced_at, s.metadata_zstd, ts.avatar_cache_key
            FROM sources s
            JOIN telegram_sources ts ON ts.source_id = s.id
            WHERE s.id = ?
            "#,
        )
        .bind(1_i64)
        .fetch_one(&pool)
        .await
        .expect("reload updated source");

        assert_eq!(row.0, Some(9));
        assert!(row.1.is_some());
        assert_eq!(row.2, None);
        assert_eq!(row.3.as_deref(), Some("1_channel_12345.jpg"));
    }

    #[tokio::test]
    async fn finalize_sync_preserves_existing_legacy_metadata_blob() {
        let pool = memory_pool_with_sources().await;
        let legacy_blob = crate::compression::compress_json_bytes(
            br#"{"peer_identity":{"strategy":"username","username":"legacy"}}"#,
        )
        .expect("compress legacy metadata");
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
            )
            VALUES (1, ?, ?, 1, '12345', 'Example', ?, 5, 10, 1, 1, 20)
            "#,
        )
        .bind(TELEGRAM_SOURCE_TYPE)
        .bind(TELEGRAM_KIND_CHANNEL)
        .bind(&legacy_blob)
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash, avatar_cache_key
            )
            VALUES (1, 1, 'channel', 'channel', 12345, 'username', 'before', 77, 'old.jpg')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert typed identity");

        let source = load_source(&pool, 1).await.expect("load source");
        finalize_sync(&pool, &source, 5, 9, Some("new.jpg".to_string()))
            .await
            .expect("finalize sync");

        let row: (Option<Vec<u8>>, Option<String>) = sqlx::query_as(
            r#"
            SELECT s.metadata_zstd, ts.avatar_cache_key
            FROM sources s
            JOIN telegram_sources ts ON ts.source_id = s.id
            WHERE s.id = 1
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("load row");

        assert_eq!(row.0.as_deref(), Some(legacy_blob.as_slice()));
        assert_eq!(row.1.as_deref(), Some("new.jpg"));
    }
}
