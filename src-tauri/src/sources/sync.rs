use grammers_session::types::PeerRef;
use serde::Serialize;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::media::extract_item_payload;
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;

use super::items::{
    build_raw_payload, extract_telegram_context, insert_source_item, message_author,
    SourceItemInsert,
};
use super::peer_resolution::resolve_and_refresh_peer;
use super::settings::{
    initial_sync_policy_label, load_sync_settings_from_pool, InitialSyncMode, SyncSettingsRecord,
    SECONDS_PER_DAY,
};
use super::store::load_source;
use super::topics::refresh_forum_topics;
use super::types::{now_secs, SourceSyncTarget};

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

        let inserted_item = insert_source_item(
            pool,
            source.id,
            SourceItemInsert {
                external_id: message_id.to_string(),
                author,
                published_at,
                payload: item_payload,
                raw_data,
                telegram_context,
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

pub(crate) async fn finalize_sync(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &SourceSyncTarget,
    previous_last_sync: i64,
    max_message_id: i64,
    refreshed_metadata_zstd: Option<Vec<u8>>,
) -> AppResult<Option<i64>> {
    let sync_completed_at = now_secs();
    let last_sync_state = if max_message_id > previous_last_sync {
        Some(max_message_id)
    } else {
        source.last_sync_state
    };

    if let Some(metadata_zstd) = refreshed_metadata_zstd {
        sqlx::query(
            "UPDATE sources SET last_sync_state = ?, last_synced_at = ?, metadata_zstd = ? WHERE id = ?",
        )
        .bind(last_sync_state)
        .bind(sync_completed_at)
        .bind(metadata_zstd)
        .bind(source.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;
    } else {
        sqlx::query("UPDATE sources SET last_sync_state = ?, last_synced_at = ? WHERE id = ?")
            .bind(last_sync_state)
            .bind(sync_completed_at)
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
    state: tauri::State<'_, TelegramState>,
    ingest_locks: tauri::State<'_, SourceIngestLocks>,
    source_id: i64,
) -> AppResult<SyncResult> {
    let _ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::Sync)
        .await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;

    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;

    let runtime = crate::telegram::get_authorized_runtime(&state, account_id).await?;
    let client = runtime.client;
    let resolved_peer = resolve_and_refresh_peer(&handle, &client, &source, account_id).await?;
    let forum_topic_warnings =
        refresh_forum_topics(&pool, &client, resolved_peer.peer, &source).await;
    let sync_policy = determine_sync_policy(&pool, &source).await?;
    let ingest = persist_items(&pool, &client, resolved_peer.peer, &source, &sync_policy).await?;
    let last_sync_state = finalize_sync(
        &pool,
        &source,
        sync_policy.previous_last_sync,
        ingest.max_message_id,
        resolved_peer.refreshed_metadata_zstd,
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
    use super::{determine_sync_policy, finalize_sync};
    use crate::compression::{compress_json_bytes, decompress_bytes};
    use crate::sources::peer_resolution::decode_source_metadata;
    use crate::sources::store::load_source;
    use crate::sources::types::{
        SourceRecordRow, SourceSyncTarget, TELEGRAM_KIND_CHANNEL, TELEGRAM_SOURCE_TYPE,
    };

    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .expect("create app_settings");
        pool
    }

    async fn memory_pool_with_sources() -> sqlx::SqlitePool {
        let pool = memory_pool().await;
        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                source_type TEXT NOT NULL,
                telegram_source_kind TEXT NOT NULL,
                account_id INTEGER,
                external_id TEXT NOT NULL,
                title TEXT,
                metadata_zstd BLOB,
                last_sync_state INTEGER,
                last_synced_at INTEGER,
                is_active INTEGER NOT NULL DEFAULT 1,
                is_member INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create sources");
        pool
    }

    #[tokio::test]
    async fn determine_sync_policy_only_applies_initial_settings_on_first_sync() {
        let pool = memory_pool_with_sources().await;
        let source = SourceSyncTarget {
            id: 1,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_CHANNEL.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
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

    #[tokio::test]
    async fn finalize_sync_updates_source_state_and_metadata() {
        let pool = memory_pool_with_sources().await;
        let metadata_zstd = compress_json_bytes(
            br#"{"peer_identity":{"strategy":"username","username":"before"}}"#,
        )
        .expect("encode initial metadata");
        sqlx::query(
            r#"
            INSERT INTO sources (
                id,
                source_type,
                telegram_source_kind,
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
        .bind(metadata_zstd)
        .bind(5_i64)
        .bind(10_i64)
        .bind(1_i64)
        .bind(1_i64)
        .bind(20_i64)
        .execute(&pool)
        .await
        .expect("insert source");

        let source = load_source(&pool, 1).await.expect("load source");
        let updated_metadata_zstd = compress_json_bytes(
            br#"{"peer_identity":{"strategy":"username","username":"after"},"avatar_cache_key":"1_channel_12345.jpg"}"#,
        )
        .expect("encode updated metadata");

        let last_sync_state = finalize_sync(&pool, &source, 5, 9, Some(updated_metadata_zstd))
            .await
            .expect("finalize sync");
        assert_eq!(last_sync_state, Some(9));

        let row: SourceRecordRow = sqlx::query_as(
            "SELECT id, source_type, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources WHERE id = ?",
        )
        .bind(1_i64)
        .fetch_one(&pool)
        .await
        .expect("reload updated source");

        assert_eq!(row.last_sync_state, Some(9));
        assert!(row.last_synced_at.is_some());
        let metadata_bytes = row.metadata_zstd.as_deref().expect("updated metadata");
        let decoded_metadata =
            decode_source_metadata(Some(metadata_bytes)).expect("decode metadata");
        let raw_metadata: serde_json::Value =
            serde_json::from_slice(&decompress_bytes(metadata_bytes).expect("decompress metadata"))
                .expect("parse metadata");
        assert_eq!(
            raw_metadata
                .pointer("/peer_identity/username")
                .and_then(serde_json::Value::as_str),
            Some("after")
        );
        assert_eq!(
            decoded_metadata.avatar_cache_key.as_deref(),
            Some("1_channel_12345.jpg")
        );
    }
}
