use serde::Serialize;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;
use crate::telegram_impl::{
    LiveMessage, PeerDescriptor, TelegramClientHandle, TelegramMessageDraft,
};

use super::identity_repair::{require_source_identity_ready, SourceIdentityRepairState};
use super::items::insert_telegram_source_item;
use super::peer_resolution::resolve_and_refresh_peer;
use super::refresh_forum_topics;
use super::settings::{
    initial_sync_policy_label, load_sync_settings_from_pool, InitialSyncMode, SyncSettingsRecord,
    SECONDS_PER_DAY,
};
use super::store::load_source;
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

struct TelegramBatchLoopPage<M> {
    messages: Vec<M>,
    is_terminal: bool,
    next_offset_id: i32,
    next_offset_date: i32,
}

trait TelegramBatchLoopMessage {
    fn message_id(&self) -> i64;
    fn published_at(&self) -> i64;
    fn into_draft(self, source_title: Option<&str>) -> AppResult<Option<TelegramMessageDraft>>;
}

trait TelegramBatchLoopBackend {
    type Message: TelegramBatchLoopMessage;

    async fn fetch_message_batch(
        &mut self,
        offset_id: i32,
        offset_date: i32,
        limit: usize,
    ) -> AppResult<TelegramBatchLoopPage<Self::Message>>;

    async fn persist_draft(&mut self, draft: TelegramMessageDraft) -> AppResult<bool>;
}

async fn run_telegram_batch_loop<B: TelegramBatchLoopBackend>(
    backend: &mut B,
    source_title: Option<&str>,
    sync_policy: &SyncPolicy,
) -> AppResult<IngestOutcome> {
    let mut outcome = IngestOutcome {
        inserted: 0,
        skipped: 0,
        max_message_id: sync_policy.previous_last_sync,
    };
    let mut remaining = sync_policy
        .initial_sync_settings
        .as_ref()
        .and_then(|settings| {
            (settings.initial_sync_mode == InitialSyncMode::RecentMessages)
                .then_some(settings.initial_sync_value as usize)
        });
    let mut offset_id = 0_i32;
    let mut offset_date = 0_i32;

    loop {
        if remaining == Some(0) {
            return Ok(outcome);
        }
        let limit = remaining.map(|remaining| remaining.min(100)).unwrap_or(100);
        let page = backend
            .fetch_message_batch(offset_id, offset_date, limit)
            .await?;

        for message in page.messages {
            if remaining == Some(0) {
                break;
            }
            if let Some(remaining) = remaining.as_mut() {
                *remaining -= 1;
            }

            let message_id = message.message_id();
            if message_id <= sync_policy.previous_last_sync {
                return Ok(outcome);
            }
            if sync_policy
                .initial_sync_cutoff
                .is_some_and(|cutoff| message.published_at() < cutoff)
            {
                return Ok(outcome);
            }

            outcome.max_message_id = outcome.max_message_id.max(message_id);
            let Some(draft) = message.into_draft(source_title)? else {
                outcome.skipped += 1;
                continue;
            };
            if backend.persist_draft(draft).await? {
                outcome.inserted += 1;
            } else {
                outcome.skipped += 1;
            }
        }

        if page.is_terminal || remaining == Some(0) {
            return Ok(outcome);
        }
        offset_id = page.next_offset_id;
        offset_date = page.next_offset_date;
    }
}

impl TelegramBatchLoopMessage for LiveMessage {
    fn message_id(&self) -> i64 {
        LiveMessage::message_id(self)
    }

    fn published_at(&self) -> i64 {
        LiveMessage::published_at(self)
    }

    fn into_draft(self, source_title: Option<&str>) -> AppResult<Option<TelegramMessageDraft>> {
        LiveMessage::into_draft(self, source_title)
    }
}

struct LiveBatchLoopBackend<'a> {
    pool: &'a sqlx::Pool<sqlx::Sqlite>,
    client: &'a TelegramClientHandle,
    peer: &'a PeerDescriptor,
    source_id: i64,
}

impl TelegramBatchLoopBackend for LiveBatchLoopBackend<'_> {
    type Message = LiveMessage;

    async fn fetch_message_batch(
        &mut self,
        offset_id: i32,
        offset_date: i32,
        limit: usize,
    ) -> AppResult<TelegramBatchLoopPage<Self::Message>> {
        let mut batch = self
            .client
            .fetch_message_batch(self.peer, offset_id, offset_date, limit)
            .await?;
        Ok(TelegramBatchLoopPage {
            messages: batch.take_messages(),
            is_terminal: batch.is_terminal(),
            next_offset_id: batch.next_offset_id(),
            next_offset_date: batch.next_offset_date(),
        })
    }

    async fn persist_draft(&mut self, draft: TelegramMessageDraft) -> AppResult<bool> {
        insert_telegram_source_item(self.pool, self.source_id, draft).await
    }
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
    client: &TelegramClientHandle,
    peer: &PeerDescriptor,
    source: &SourceSyncTarget,
    sync_policy: &SyncPolicy,
) -> AppResult<IngestOutcome> {
    let mut backend = LiveBatchLoopBackend {
        pool,
        client,
        peer,
        source_id: source.id,
    };
    run_telegram_batch_loop(&mut backend, source.title.as_deref(), sync_policy).await
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

    let client_handle = state.authorized_client(account_id).await?;
    let resolved_peer =
        resolve_and_refresh_peer(&handle, &pool, &client_handle, &source, account_id).await?;
    let forum_topic_warnings =
        refresh_forum_topics(&pool, &client_handle, &resolved_peer.descriptor, &source).await;
    let sync_policy = determine_sync_policy(&pool, &source).await?;
    let ingest = persist_items(
        &pool,
        &client_handle,
        &resolved_peer.descriptor,
        &source,
        &sync_policy,
    )
    .await?;
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
        determine_sync_policy, finalize_sync, run_telegram_batch_loop, sync_provider_for_source,
        IngestOutcome, SyncPolicy, SyncProvider, TelegramBatchLoopBackend,
        TelegramBatchLoopMessage, TelegramBatchLoopPage,
    };
    use crate::error::{AppError, AppResult};
    use crate::sources::settings::{InitialSyncMode, SyncSettingsRecord};
    use crate::sources::store::load_source;
    use crate::sources::test_support::memory_pool_with_sources;
    use crate::sources::types::{SourceSyncTarget, TELEGRAM_KIND_CHANNEL, TELEGRAM_SOURCE_TYPE};
    use crate::telegram_impl::{
        TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity,
        ITEM_KIND_TELEGRAM_MESSAGE,
    };
    use std::collections::VecDeque;

    enum FakeMessageMapping {
        Draft,
        Skip,
        Fail(&'static str),
    }

    struct FakeBatchMessage {
        id: i64,
        published_at: i64,
        mapping: FakeMessageMapping,
    }

    impl TelegramBatchLoopMessage for FakeBatchMessage {
        fn message_id(&self) -> i64 {
            self.id
        }

        fn published_at(&self) -> i64 {
            self.published_at
        }

        fn into_draft(
            self,
            _source_title: Option<&str>,
        ) -> AppResult<Option<TelegramMessageDraft>> {
            match self.mapping {
                FakeMessageMapping::Draft => Ok(Some(fake_draft(self.id, self.published_at))),
                FakeMessageMapping::Skip => Ok(None),
                FakeMessageMapping::Fail(message) => Err(AppError::internal(message)),
            }
        }
    }

    struct FakeBatchBackend {
        pages: VecDeque<AppResult<TelegramBatchLoopPage<FakeBatchMessage>>>,
        requests: Vec<(i32, i32, usize)>,
        persisted: Vec<i64>,
        fail_persist_id: Option<i64>,
    }

    impl TelegramBatchLoopBackend for FakeBatchBackend {
        type Message = FakeBatchMessage;

        async fn fetch_message_batch(
            &mut self,
            offset_id: i32,
            offset_date: i32,
            limit: usize,
        ) -> AppResult<TelegramBatchLoopPage<Self::Message>> {
            self.requests.push((offset_id, offset_date, limit));
            self.pages
                .pop_front()
                .unwrap_or_else(|| Err(AppError::internal("unexpected extra batch fetch")))
        }

        async fn persist_draft(&mut self, draft: TelegramMessageDraft) -> AppResult<bool> {
            let message_id = draft
                .telegram_identity
                .as_ref()
                .expect("fake draft identity")
                .telegram_message_id;
            if self.fail_persist_id == Some(message_id) {
                return Err(AppError::internal("fake persistence failure"));
            }
            self.persisted.push(message_id);
            Ok(true)
        }
    }

    fn fake_draft(message_id: i64, published_at: i64) -> TelegramMessageDraft {
        TelegramMessageDraft {
            telegram_identity: Some(TelegramMessageIdentity {
                history_peer_kind: "channel".to_string(),
                history_peer_id: 900,
                telegram_message_id: message_id,
                migration_domain: None,
                is_migrated_history: false,
            }),
            telegram_context: TelegramItemContext::default(),
            content: Some(format!("message {message_id}")),
            content_kind: "text_only",
            author: None,
            published_at,
            raw_data: Vec::new(),
            item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
            media: None,
        }
    }

    fn fake_message(id: i64, mapping: FakeMessageMapping) -> FakeBatchMessage {
        FakeBatchMessage {
            id,
            published_at: id * 10,
            mapping,
        }
    }

    fn fake_page(
        messages: Vec<FakeBatchMessage>,
        is_terminal: bool,
        next_offset_id: i32,
        next_offset_date: i32,
    ) -> TelegramBatchLoopPage<FakeBatchMessage> {
        TelegramBatchLoopPage {
            messages,
            is_terminal,
            next_offset_id,
            next_offset_date,
        }
    }

    fn recent_messages_policy(limit: i64) -> SyncPolicy {
        SyncPolicy {
            previous_last_sync: 0,
            initial_sync_settings: Some(SyncSettingsRecord {
                initial_sync_mode: InitialSyncMode::RecentMessages,
                initial_sync_value: limit,
            }),
            initial_sync_policy_applied: Some(format!("last {limit} messages")),
            initial_sync_cutoff: None,
        }
    }

    fn assert_outcome(outcome: IngestOutcome, inserted: i64, skipped: i64, max_message_id: i64) {
        assert_eq!(
            (outcome.inserted, outcome.skipped, outcome.max_message_id),
            (inserted, skipped, max_message_id)
        );
    }

    #[tokio::test]
    async fn telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error() {
        let mut persistence_failure = FakeBatchBackend {
            pages: VecDeque::from([Ok(fake_page(
                vec![
                    fake_message(10, FakeMessageMapping::Draft),
                    fake_message(9, FakeMessageMapping::Draft),
                ],
                false,
                9,
                90,
            ))]),
            requests: Vec::new(),
            persisted: Vec::new(),
            fail_persist_id: Some(9),
        };
        let error = run_telegram_batch_loop(
            &mut persistence_failure,
            Some("Source"),
            &recent_messages_policy(10),
        )
        .await
        .err()
        .expect("second persistence fails");
        assert!(error.message.contains("persistence"));
        assert_eq!(persistence_failure.persisted, vec![10]);
        assert_eq!(persistence_failure.requests.len(), 1);

        let mut budget = FakeBatchBackend {
            pages: VecDeque::from([
                Ok(fake_page(
                    vec![
                        fake_message(12, FakeMessageMapping::Skip),
                        fake_message(10, FakeMessageMapping::Draft),
                    ],
                    false,
                    10,
                    100,
                )),
                Ok(fake_page(
                    vec![
                        fake_message(8, FakeMessageMapping::Draft),
                        fake_message(7, FakeMessageMapping::Draft),
                    ],
                    true,
                    10,
                    100,
                )),
            ]),
            requests: Vec::new(),
            persisted: Vec::new(),
            fail_persist_id: None,
        };
        let outcome =
            run_telegram_batch_loop(&mut budget, Some("Source"), &recent_messages_policy(3))
                .await
                .expect("bounded batch loop");
        assert_outcome(outcome, 2, 1, 12);
        assert_eq!(budget.requests, vec![(0, 0, 3), (10, 100, 1)]);
        assert_eq!(budget.persisted, vec![10, 8]);

        let mut conversion_failure = FakeBatchBackend {
            pages: VecDeque::from([
                Ok(fake_page(
                    vec![fake_message(
                        7,
                        FakeMessageMapping::Fail("fake conversion failure"),
                    )],
                    false,
                    7,
                    70,
                )),
                Err(AppError::internal(
                    "must not fetch after conversion failure",
                )),
            ]),
            requests: Vec::new(),
            persisted: Vec::new(),
            fail_persist_id: None,
        };
        let error = run_telegram_batch_loop(
            &mut conversion_failure,
            Some("Source"),
            &recent_messages_policy(5),
        )
        .await
        .err()
        .expect("conversion failure");
        assert!(error.message.contains("conversion"));
        assert_eq!(conversion_failure.requests.len(), 1);

        let mut timeout = FakeBatchBackend {
            pages: VecDeque::from([
                Ok(fake_page(
                    vec![fake_message(6, FakeMessageMapping::Draft)],
                    false,
                    6,
                    60,
                )),
                Err(AppError::network("timeout")),
                Err(AppError::internal("must not fetch after timeout")),
            ]),
            requests: Vec::new(),
            persisted: Vec::new(),
            fail_persist_id: None,
        };
        let error =
            run_telegram_batch_loop(&mut timeout, Some("Source"), &recent_messages_policy(5))
                .await
                .err()
                .expect("timeout stops loop");
        assert_eq!(error.message, "timeout");
        assert_eq!(timeout.persisted, vec![6]);
        assert_eq!(timeout.requests, vec![(0, 0, 5), (6, 60, 4)]);
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
            is_member: true,
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
            is_member: true,
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
            is_member: false,
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
