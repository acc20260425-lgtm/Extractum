use std::collections::{BTreeSet, HashSet};

use sqlx::{Pool, Sqlite};

use crate::analysis::AnalysisState;
use crate::error::{AppError, AppResult};
use crate::llm::LlmSchedulerState;
use crate::source_ingest::SourceIngestLocks;
use crate::takeout_import::TakeoutImportState;
use crate::youtube::jobs::SourceJobState;

pub(crate) const ACCOUNT_DELETE_ACTIVE_WORK_CONFLICT_MESSAGE: &str =
    "Cannot delete account while source sync, import, source job, or analysis work is running for its sources. Stop or wait for the active work, then try again.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccountDeletionPlan {
    pub(crate) account_id: i64,
    pub(crate) owned_source_ids: Vec<i64>,
    #[cfg(test)]
    pub(crate) blocking_work: Vec<AccountDeletionBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AccountDeletionBlocker {
    SourceIngest {
        source_id: i64,
    },
    TakeoutImport {
        source_id: i64,
        job_id: String,
    },
    SourceJob {
        source_id: i64,
        related_source_id: Option<i64>,
        job_id: String,
    },
    AnalysisRun {
        run_id: i64,
    },
    LlmRequest {
        run_id: i64,
    },
}

pub(crate) async fn check_account_deletion(
    pool: &Pool<Sqlite>,
    account_id: i64,
    source_locks: &SourceIngestLocks,
    takeout_state: &TakeoutImportState,
    source_job_state: &SourceJobState,
    analysis_state: &AnalysisState,
    llm_scheduler: &LlmSchedulerState,
) -> AppResult<AccountDeletionPlan> {
    if !account_exists(pool, account_id).await? {
        return Err(AppError::not_found(format!("Account {account_id} not found")));
    }
    let owned_source_ids = load_owned_source_ids(pool, account_id).await?;
    let blocking_work = collect_account_deletion_blockers(
        &owned_source_ids,
        source_locks,
        takeout_state,
        source_job_state,
        pool,
        analysis_state,
        llm_scheduler,
    )
    .await?;
    if !blocking_work.is_empty() {
        return Err(AppError::conflict(
            ACCOUNT_DELETE_ACTIVE_WORK_CONFLICT_MESSAGE,
        ));
    }

    Ok(AccountDeletionPlan {
        account_id,
        owned_source_ids,
        #[cfg(test)]
        blocking_work,
    })
}

async fn collect_account_deletion_blockers(
    owned_source_ids: &[i64],
    source_locks: &SourceIngestLocks,
    takeout_state: &TakeoutImportState,
    source_job_state: &SourceJobState,
    pool: &Pool<Sqlite>,
    analysis_state: &AnalysisState,
    llm_scheduler: &LlmSchedulerState,
) -> AppResult<Vec<AccountDeletionBlocker>> {
    let mut blocking_work = Vec::new();
    for (source_id, _kind) in source_locks
        .active_kinds_for_sources(owned_source_ids)
        .await?
    {
        blocking_work.push(AccountDeletionBlocker::SourceIngest { source_id });
    }
    for job in takeout_state.active_jobs_for_sources(owned_source_ids).await {
        blocking_work.push(AccountDeletionBlocker::TakeoutImport {
            source_id: job.source_id,
            job_id: job.job_id,
        });
    }
    for job in source_job_state.active_jobs_for_sources(owned_source_ids).await {
        blocking_work.push(AccountDeletionBlocker::SourceJob {
            source_id: job.source_id,
            related_source_id: job.related_source_id,
            job_id: job.job_id,
        });
    }
    let active_run_ids = analysis_state.active_report_run_ids().await;
    for run_id in run_ids_depending_on_sources(pool, &active_run_ids, owned_source_ids).await? {
        blocking_work.push(AccountDeletionBlocker::AnalysisRun { run_id });
    }
    let llm_owner_run_ids = llm_scheduler.active_owner_run_ids().await;
    for run_id in run_ids_depending_on_sources(pool, &llm_owner_run_ids, owned_source_ids).await? {
        blocking_work.push(AccountDeletionBlocker::LlmRequest { run_id });
    }
    Ok(blocking_work)
}

async fn run_ids_depending_on_sources(
    pool: &Pool<Sqlite>,
    candidate_run_ids: &HashSet<i64>,
    owned_source_ids: &[i64],
) -> AppResult<BTreeSet<i64>> {
    if candidate_run_ids.is_empty() || owned_source_ids.is_empty() {
        return Ok(BTreeSet::new());
    }

    let owned = owned_source_ids.iter().copied().collect::<HashSet<_>>();
    let mut blocked = BTreeSet::new();
    let rows = sqlx::query_as::<_, AnalysisRunScopeRow>(
        "SELECT id, source_id, source_group_id FROM analysis_runs ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    for row in rows {
        if !candidate_run_ids.contains(&row.id) {
            continue;
        }
        if row
            .source_id
            .is_some_and(|source_id| owned.contains(&source_id))
        {
            blocked.insert(row.id);
            continue;
        }
        if let Some(group_id) = row.source_group_id {
            if group_has_owned_source(pool, group_id, &owned).await? {
                blocked.insert(row.id);
            }
        }
    }
    Ok(blocked)
}

async fn group_has_owned_source(
    pool: &Pool<Sqlite>,
    group_id: i64,
    owned_source_ids: &HashSet<i64>,
) -> AppResult<bool> {
    let source_ids = sqlx::query_scalar::<_, i64>(
        "SELECT source_id FROM analysis_source_group_members WHERE group_id = ?",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    Ok(source_ids
        .into_iter()
        .any(|source_id| owned_source_ids.contains(&source_id)))
}

#[derive(sqlx::FromRow)]
struct AnalysisRunScopeRow {
    id: i64,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
}

async fn account_exists(pool: &Pool<Sqlite>, account_id: i64) -> AppResult<bool> {
    sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM accounts WHERE id = ?)")
        .bind(account_id)
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
        .map_err(AppError::database)
}

async fn load_owned_source_ids(pool: &Pool<Sqlite>, account_id: i64) -> AppResult<Vec<i64>> {
    sqlx::query_scalar::<_, i64>("SELECT id FROM sources WHERE account_id = ? ORDER BY id ASC")
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::analysis::AnalysisState;
    use crate::error::{AppError, AppErrorKind};
    use crate::ingest_provenance::TAKEOUT_HISTORY_SCOPE_CURRENT;
    use crate::llm::{
        LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState,
    };
    use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
    use crate::takeout_import::TakeoutImportState;
    use crate::youtube::jobs::{SourceJobState, SourceJobType, YoutubeSyncOptions};
    use tokio::sync::Notify;
    use tokio::time::{timeout, Duration};

    async fn pool() -> sqlx::SqlitePool {
        let pool = crate::sources::test_support::memory_pool_with_sources().await;
        sqlx::query(
            "CREATE TABLE accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                label TEXT NOT NULL,
                api_id INTEGER NOT NULL,
                api_hash TEXT NOT NULL,
                phone TEXT,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create accounts");
        sqlx::query(
            "CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create groups");
        sqlx::query(
            "CREATE TABLE analysis_source_group_members (
                group_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (group_id, source_id)
            )",
        )
        .execute(&pool)
        .await
        .expect("create group members");
        sqlx::query(
            "CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                status TEXT NOT NULL,
                source_id INTEGER,
                source_group_id INTEGER,
                created_at INTEGER NOT NULL DEFAULT 1
            )",
        )
        .execute(&pool)
        .await
        .expect("create runs");
        pool
    }

    async fn insert_account(pool: &sqlx::SqlitePool, account_id: i64) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, created_at)
             VALUES (?, 'Account', 12345, '', 1)",
        )
        .bind(account_id)
        .execute(pool)
        .await
        .expect("insert account");
    }

    async fn insert_source(pool: &sqlx::SqlitePool, source_id: i64, account_id: Option<i64>) {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, account_id, external_id, title, metadata_zstd, created_at)
             VALUES (?, 'telegram', 'channel', ?, ?, 'Source', x'00', 1)",
        )
        .bind(source_id)
        .bind(account_id)
        .bind(format!("source-{source_id}"))
        .execute(pool)
        .await
        .expect("insert source");
    }

    async fn insert_run(
        pool: &sqlx::SqlitePool,
        run_id: i64,
        status: &str,
        source_id: Option<i64>,
        source_group_id: Option<i64>,
    ) {
        sqlx::query(
            "INSERT INTO analysis_runs (id, status, source_id, source_group_id, created_at)
             VALUES (?, ?, ?, ?, 1)",
        )
        .bind(run_id)
        .bind(status)
        .bind(source_id)
        .bind(source_group_id)
        .execute(pool)
        .await
        .expect("insert run");
    }

    async fn insert_group_member(pool: &sqlx::SqlitePool, group_id: i64, source_id: i64) {
        sqlx::query(
            "INSERT OR IGNORE INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
             VALUES (?, 'Group', 'telegram', 1, 1)",
        )
        .bind(group_id)
        .execute(pool)
        .await
        .expect("insert group");
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (?, ?, 1)",
        )
        .bind(group_id)
        .bind(source_id)
        .execute(pool)
        .await
        .expect("insert group member");
    }

    struct States {
        source_locks: SourceIngestLocks,
        takeout_state: TakeoutImportState,
        source_job_state: SourceJobState,
        analysis_state: AnalysisState,
        llm_scheduler: Arc<LlmSchedulerState>,
    }

    impl States {
        fn new() -> Self {
            Self {
                source_locks: SourceIngestLocks::new(),
                takeout_state: TakeoutImportState::new(),
                source_job_state: SourceJobState::new(),
                analysis_state: AnalysisState::new(),
                llm_scheduler: Arc::new(LlmSchedulerState::new()),
            }
        }

        async fn check(
            &self,
            pool: &sqlx::SqlitePool,
            account_id: i64,
        ) -> AppResult<AccountDeletionPlan> {
            check_account_deletion(
                pool,
                account_id,
                &self.source_locks,
                &self.takeout_state,
                &self.source_job_state,
                &self.analysis_state,
                self.llm_scheduler.as_ref(),
            )
            .await
        }
    }

    async fn start_scheduler_request(
        scheduler: Arc<LlmSchedulerState>,
        request_id: &str,
        owner_run_id: Option<i64>,
    ) -> Arc<Notify> {
        let release = Arc::new(Notify::new());
        let release_for_task = Arc::clone(&release);
        let request_id = request_id.to_string();
        let request_id_for_wait = request_id.clone();
        let scheduler_for_task = Arc::clone(&scheduler);
        tokio::spawn(async move {
            let _ = scheduler_for_task
                .run_request(
                    LlmRequestMetadata {
                        request_id,
                        profile_id: "default".to_string(),
                        provider: "gemini".to_string(),
                        kind: if owner_run_id.is_some() {
                            LlmRequestKind::AnalysisChat
                        } else {
                            LlmRequestKind::ProviderTest
                        },
                        priority: LlmRequestPriority::Interactive,
                        owner_run_id,
                    },
                    |_| {},
                    move |control| async move {
                        control
                            .run_cancellable(async move {
                                release_for_task.notified().await;
                                Ok::<_, AppError>("done")
                            })
                            .await
                    },
                )
                .await;
        });

        timeout(Duration::from_secs(1), async {
            loop {
                let snapshots = scheduler.request_snapshots().await;
                if snapshots
                    .iter()
                    .any(|snapshot| snapshot.request_id == request_id_for_wait)
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("scheduler request registered");

        release
    }

    #[tokio::test]
    async fn missing_account_returns_not_found() {
        let pool = pool().await;
        let states = States::new();

        let error = states.check(&pool, 404).await.expect_err("missing account");

        assert_eq!(error.kind, AppErrorKind::NotFound);
        assert_eq!(error.message, "Account 404 not found");
    }

    #[tokio::test]
    async fn existing_account_with_zero_sources_passes() {
        let pool = pool().await;
        insert_account(&pool, 11).await;
        let states = States::new();

        let plan = states.check(&pool, 11).await.expect("preflight");

        assert_eq!(plan.account_id, 11);
        assert!(plan.owned_source_ids.is_empty());
        assert!(plan.blocking_work.is_empty());
    }

    #[tokio::test]
    async fn source_ingest_lock_on_owned_source_blocks_without_deleting_rows() {
        let pool = pool().await;
        insert_account(&pool, 11).await;
        insert_source(&pool, 7, Some(11)).await;
        let states = States::new();
        let _guard = states
            .source_locks
            .try_acquire(7, SourceIngestKind::Sync)
            .await
            .expect("lock source");

        let error = states.check(&pool, 11).await.expect_err("blocked");

        assert_eq!(error.kind, AppErrorKind::Conflict);
        assert_eq!(error.message, ACCOUNT_DELETE_ACTIVE_WORK_CONFLICT_MESSAGE);
        let account_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE id = 11")
            .fetch_one(&pool)
            .await
            .expect("count account");
        let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE id = 7")
            .fetch_one(&pool)
            .await
            .expect("count source");
        assert_eq!(account_count, 1);
        assert_eq!(source_count, 1);
    }

    #[tokio::test]
    async fn active_takeout_job_on_owned_source_blocks() {
        let pool = pool().await;
        insert_account(&pool, 11).await;
        insert_source(&pool, 7, Some(11)).await;
        let states = States::new();
        states
            .takeout_state
            .create_job(7, 11, 100, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("takeout job");

        let error = states.check(&pool, 11).await.expect_err("blocked");

        assert_eq!(error.kind, AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn active_source_job_on_owned_source_blocks_but_unowned_job_does_not() {
        let pool = pool().await;
        insert_account(&pool, 11).await;
        insert_source(&pool, 7, Some(11)).await;
        insert_source(&pool, 8, None).await;
        let states = States::new();
        let options = YoutubeSyncOptions {
            metadata: true,
            transcripts: false,
            comments: false,
        };
        states
            .source_job_state
            .create_job(8, SourceJobType::YoutubeVideoFullSync, None, options.clone())
            .await
            .expect("unowned job");
        states
            .check(&pool, 11)
            .await
            .expect("unowned job ignored");

        states
            .source_job_state
            .create_job(7, SourceJobType::YoutubeVideoFullSync, None, options)
            .await
            .expect("owned job");

        let error = states.check(&pool, 11).await.expect_err("blocked");

        assert_eq!(error.kind, AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn active_direct_source_analysis_run_blocks_owned_source_only() {
        let pool = pool().await;
        insert_account(&pool, 11).await;
        insert_source(&pool, 7, Some(11)).await;
        insert_source(&pool, 8, None).await;
        insert_run(&pool, 70, "running", Some(8), None).await;
        let states = States::new();
        states.analysis_state.insert_active_report_run(70).await;
        states
            .check(&pool, 11)
            .await
            .expect("unowned active run ignored");

        insert_run(&pool, 71, "running", Some(7), None).await;
        states.analysis_state.insert_active_report_run(71).await;

        let error = states.check(&pool, 11).await.expect_err("blocked");

        assert_eq!(error.kind, AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn active_group_analysis_run_blocks_when_any_member_source_is_owned() {
        let pool = pool().await;
        insert_account(&pool, 11).await;
        insert_source(&pool, 7, Some(11)).await;
        insert_source(&pool, 8, None).await;
        insert_group_member(&pool, 90, 8).await;
        insert_run(&pool, 90, "running", None, Some(90)).await;
        let states = States::new();
        states.analysis_state.insert_active_report_run(90).await;
        states.check(&pool, 11).await.expect("unowned group ignored");

        insert_group_member(&pool, 90, 7).await;

        let error = states.check(&pool, 11).await.expect_err("blocked");

        assert_eq!(error.kind, AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn active_llm_request_for_completed_owned_run_blocks_but_provider_test_does_not() {
        let pool = pool().await;
        insert_account(&pool, 11).await;
        insert_source(&pool, 7, Some(11)).await;
        insert_run(&pool, 77, "completed", Some(7), None).await;
        let states = States::new();

        let provider_release =
            start_scheduler_request(Arc::clone(&states.llm_scheduler), "provider-test", None)
                .await;
        states
            .check(&pool, 11)
            .await
            .expect("provider test ignored");

        let chat_release =
            start_scheduler_request(Arc::clone(&states.llm_scheduler), "chat-77", Some(77)).await;
        let error = states.check(&pool, 11).await.expect_err("blocked");

        assert_eq!(error.kind, AppErrorKind::Conflict);
        provider_release.notify_waiters();
        chat_release.notify_waiters();
    }

    #[tokio::test]
    async fn blocker_collection_keeps_multiple_categories_for_internal_diagnostics() {
        let pool = pool().await;
        insert_account(&pool, 11).await;
        insert_source(&pool, 7, Some(11)).await;
        insert_run(&pool, 55, "running", Some(7), None).await;
        let states = States::new();
        let _guard = states
            .source_locks
            .try_acquire(7, SourceIngestKind::Sync)
            .await
            .expect("source lock");
        states.analysis_state.insert_active_report_run(55).await;

        let blockers = collect_account_deletion_blockers(
            &[7],
            &states.source_locks,
            &states.takeout_state,
            &states.source_job_state,
            &pool,
            &states.analysis_state,
            states.llm_scheduler.as_ref(),
        )
        .await
        .expect("blockers");

        assert!(blockers.contains(&AccountDeletionBlocker::SourceIngest { source_id: 7 }));
        assert!(blockers.contains(&AccountDeletionBlocker::AnalysisRun { run_id: 55 }));
    }
}
