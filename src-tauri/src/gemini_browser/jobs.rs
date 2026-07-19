use apalis::prelude::{IntervalStrategy, StrategyBuilder, WorkerBuilderExt};
use apalis_sqlite::TaskBuilderExt;
use tauri::Manager;

use super::executor::{app_error_to_domain, AppStatusObserver, DomainErrorContext};
use crate::gemini_browser::executor::{domain_error_to_app, AppBrowserExecutor};
use extractum_gemini_browser::{
    cancel_run, ensure_startup_reconciled, execute_delivered_job, run_registered_worker,
    DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
    GeminiBrowserJobRuntime, QueueInspectionSnapshot, QueuedGeminiBrowserJob, ReconciliationAction,
    StartupReconciliationSnapshot,
};

pub(crate) const GEMINI_BROWSER_QUEUE_NAME: &str = "gemini-browser";
const GEMINI_BROWSER_QUEUE_POLL_INTERVAL_MS: u64 = 100;
type GeminiBrowserApalisTask<IdType> =
    apalis::prelude::Task<GeminiBrowserJob, apalis_sqlite::SqliteContext, IdType>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApalisQueueInspectionMode {
    #[cfg(test)]
    Supported,
    DegradedRunLogOnly,
}

pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
    ApalisQueueInspectionMode::DegradedRunLogOnly
}

pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
    mode: ApalisQueueInspectionMode,
) -> bool {
    match mode {
        #[cfg(test)]
        ApalisQueueInspectionMode::Supported => true,
        ApalisQueueInspectionMode::DegradedRunLogOnly => false,
    }
}

#[cfg(test)]
pub(crate) fn run_status_for_queue_state(
    state: &str,
) -> Option<crate::gemini_browser::GeminiBrowserRunStatus> {
    match state {
        "Pending" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Queued),
        "Running" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Running),
        "Done" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Ok),
        "Failed" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Failed),
        "Killed" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Failed),
        _ => None,
    }
}

pub(crate) trait GeminiBrowserApalisStorageAccess {
    fn pool(&self) -> &sqlx::SqlitePool;
}

impl<T, C, F> GeminiBrowserApalisStorageAccess for apalis_sqlite::SqliteStorage<T, C, F> {
    fn pool(&self) -> &sqlx::SqlitePool {
        self.pool()
    }
}

pub(crate) async fn setup_gemini_browser_apalis_storage(
    pool: &sqlx::SqlitePool,
) -> crate::error::AppResult<()> {
    sqlx::query("PRAGMA journal_mode = 'WAL';")
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;
    sqlx::query("PRAGMA temp_store = 2;")
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;
    sqlx::query("PRAGMA synchronous = NORMAL;")
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;
    sqlx::query("PRAGMA cache_size = 64000;")
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;

    let mut migrations = apalis_sqlite::SqliteStorage::migrations();
    migrations.set_ignore_missing(true);
    migrations
        .run(pool)
        .await
        .map_err(crate::error::AppError::database)?;

    Ok(())
}

pub(crate) async fn open_gemini_browser_job_storage(
    pool: &sqlx::SqlitePool,
) -> crate::error::AppResult<
    impl apalis::prelude::TaskSink<GeminiBrowserJob>
        + apalis::prelude::Backend<
            Args = GeminiBrowserJob,
            Context = apalis_sqlite::SqliteContext,
            Error = sqlx::Error,
            IdType: Send + Sync + 'static,
        > + GeminiBrowserApalisStorageAccess
        + Clone
        + Send
        + 'static,
> {
    setup_gemini_browser_apalis_storage(pool).await?;
    Ok(apalis_sqlite::SqliteStorage::new_with_config(
        pool,
        &gemini_browser_queue_config(),
    ))
}

fn gemini_browser_queue_config() -> apalis_sqlite::Config {
    let poll_strategy = StrategyBuilder::new()
        .apply(IntervalStrategy::new(std::time::Duration::from_millis(
            GEMINI_BROWSER_QUEUE_POLL_INTERVAL_MS,
        )))
        .build();
    apalis_sqlite::Config::new(GEMINI_BROWSER_QUEUE_NAME).with_poll_interval(poll_strategy)
}

pub(crate) async fn enqueue_gemini_browser_job_to_storage<S>(
    storage: &mut S,
    job: GeminiBrowserJob,
) -> crate::error::AppResult<QueuedGeminiBrowserJob>
where
    S: apalis::prelude::TaskSink<GeminiBrowserJob>
        + apalis::prelude::Backend<
            Args = GeminiBrowserJob,
            Context = apalis_sqlite::SqliteContext,
            Error = sqlx::Error,
        > + GeminiBrowserApalisStorageAccess,
    S::IdType: Send + 'static,
{
    let run_id = job.run_id.clone();
    if gemini_browser_job_idempotency_exists(storage.pool(), &run_id).await? {
        return Err(crate::error::AppError::conflict(
            "Gemini Browser job with this run_id is already queued or running",
        ));
    }

    let task = build_gemini_browser_task::<S::IdType>(job);
    storage
        .push_task(task)
        .await
        .map_err(|error| map_enqueue_error(&run_id, error))?;

    Ok(QueuedGeminiBrowserJob {
        run_id,
        queue_position: None,
    })
}

fn build_gemini_browser_task<IdType>(job: GeminiBrowserJob) -> GeminiBrowserApalisTask<IdType> {
    let run_id = job.run_id.clone();
    apalis::prelude::TaskBuilder::<_, apalis_sqlite::SqliteContext, IdType>::new(job)
        .with_idempotency_key(&run_id)
        .max_attempts(1)
        .build()
}

async fn gemini_browser_job_idempotency_exists(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> crate::error::AppResult<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM Jobs
         WHERE job_type = ? AND idempotency_key = ?",
    )
    .bind(GEMINI_BROWSER_QUEUE_NAME)
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(crate::error::AppError::database)?;

    Ok(count > 0)
}

pub(crate) async fn enqueue_gemini_browser_job(
    handle: &tauri::AppHandle,
    job: GeminiBrowserJob,
) -> crate::error::AppResult<QueuedGeminiBrowserJob> {
    let pool = crate::db::get_pool(handle).await?;
    let mut storage = open_gemini_browser_job_storage(&pool).await?;
    enqueue_gemini_browser_job_to_storage(&mut storage, job).await
}

pub(crate) async fn cancel_gemini_browser_job(
    handle: &tauri::AppHandle,
    run_id: &str,
) -> crate::error::AppResult<()> {
    let runtime = handle.state::<GeminiBrowserJobRuntime>();
    let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    let runs_root = crate::gemini_browser::runs_dir(handle)?;

    let executor = AppBrowserExecutor::new(handle, &state);
    cancel_run(
        &runs_root,
        &runtime,
        state.domain(),
        &executor,
        &AppStatusObserver,
        run_id,
    )
    .await
    .map(|_| ())
    .map_err(domain_error_to_app)
}

fn map_enqueue_error(_run_id: &str, error: impl std::fmt::Display) -> crate::error::AppError {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("unique constraint")
        || message.contains("constraint failed")
        || message.contains("idempotency")
        || message.contains("duplicate")
        || message.contains("already exists")
    {
        return crate::error::AppError::conflict(
            "Gemini Browser job with this run_id is already queued or running",
        );
    }

    crate::error::AppError::internal(format!("Gemini Browser job enqueue failed: {error}"))
}

pub(crate) async fn start_gemini_browser_job_worker(
    handle: tauri::AppHandle,
) -> crate::error::AppResult<()> {
    let runtime = handle.state::<GeminiBrowserJobRuntime>();
    run_registered_worker(runtime.inner(), || {
        let setup_handle = handle.clone();
        async move {
            let pool = crate::db::get_pool(&setup_handle)
                .await
                .map_err(|error| app_error_to_domain(error, DomainErrorContext::Persistence))?;
            setup_gemini_browser_apalis_storage(&pool)
                .await
                .map_err(|error| app_error_to_domain(error, DomainErrorContext::Persistence))?;
            ensure_gemini_browser_startup_reconciled(&setup_handle)
                .await
                .map_err(|error| app_error_to_domain(error, DomainErrorContext::Persistence))?;
            let storage = apalis_sqlite::SqliteStorage::new_with_config(
                &pool,
                &gemini_browser_queue_config(),
            );
            let worker_runtime = setup_handle.state::<GeminiBrowserJobRuntime>();
            let timeout_layer =
                tower::timeout::TimeoutLayer::new(worker_runtime.worker_hard_guard_timeout());
            let worker = apalis::prelude::WorkerBuilder::new(GEMINI_BROWSER_QUEUE_NAME)
                .backend(storage)
                .concurrency(1)
                .layer(timeout_layer)
                .data(setup_handle)
                .build(process_gemini_browser_job);
            Ok(async move { worker.run().await })
        }
    })
    .await
    .map_err(domain_error_to_app)
}

pub(crate) async fn ensure_gemini_browser_startup_reconciled(
    handle: &tauri::AppHandle,
) -> crate::error::AppResult<()> {
    let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    let load_handle = handle.clone();
    let apply_handle = handle.clone();
    ensure_startup_reconciled(
        state.domain(),
        move || async move {
            let runs_root = crate::gemini_browser::runs_dir(&load_handle)
                .map_err(|error| app_error_to_domain(error, DomainErrorContext::Persistence))?;
            Ok(StartupReconciliationSnapshot {
                runs: extractum_gemini_browser::list_runs(&runs_root, usize::MAX)?.runs,
                queue: QueueInspectionSnapshot::Unavailable,
            })
        },
        move |actions| async move {
            let runs_root = crate::gemini_browser::runs_dir(&apply_handle)
                .map_err(|error| app_error_to_domain(error, DomainErrorContext::Persistence))?;
            for action in actions {
                let ReconciliationAction::Finish { run_id, result } = action;
                extractum_gemini_browser::finish_run(&runs_root, &run_id, result)?;
            }
            Ok(())
        },
    )
    .await
    .map_err(domain_error_to_app)
}

async fn process_gemini_browser_job(
    job: GeminiBrowserJob,
    handle: apalis::prelude::Data<tauri::AppHandle>,
) -> Result<(), apalis::prelude::BoxDynError> {
    let handle = &*handle;
    let runtime = handle.state::<GeminiBrowserJobRuntime>();
    let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    let executor = AppBrowserExecutor::new(handle, &state);
    let outcome = execute_delivered_job(
        &runtime,
        state.domain(),
        &executor,
        &AppStatusObserver,
        DeliveredJobInput {
            runs_dir: crate::gemini_browser::runs_dir(handle)?,
            browser_profile_dir: crate::gemini_browser::path_string(
                &crate::gemini_browser::profile_dir(handle)?,
            ),
            artifact_dir: crate::gemini_browser::path_string(&crate::gemini_browser::run_dir(
                handle,
                &job.run_id,
            )?),
            job,
        },
    )
    .await
    .map_err(|error| -> apalis::prelude::BoxDynError { Box::new(domain_error_to_app(error)) })?;
    if let DeliveryOutcome::Cancelled { stop_error, .. }
    | DeliveryOutcome::TimedOut { stop_error, .. } = &outcome
    {
        if let Some(error) = stop_error {
            eprintln!("Gemini Browser stop diagnostic: {error}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod app_tests {
    use std::{
        path::Path,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    use apalis::prelude::{
        BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    };
    use apalis_sqlite::SqliteStorage;
    use parking_lot::Mutex;
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use tokio::sync::oneshot;

    use super::*;

    fn test_job(run_id: &str) -> GeminiBrowserJob {
        GeminiBrowserJob {
            run_id: run_id.to_string(),
            prompt: "hello".to_string(),
            source: "settings_test".to_string(),
            artifact_mode: GeminiBrowserArtifactMode::Reduced,
            browser_config: None,
        }
    }

    async fn sqlite_file_pool(path: &Path) -> sqlx::SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("connect sqlite pool")
    }

    async fn migrated_pool(temp: &tempfile::TempDir) -> sqlx::SqlitePool {
        let pool = sqlite_file_pool(&temp.path().join("extractum.db")).await;
        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        pool
    }

    async fn process_one_job(
        job: GeminiBrowserJob,
        processed: Data<Arc<Mutex<Option<oneshot::Sender<String>>>>>,
        worker: WorkerContext,
    ) -> Result<(), BoxDynError> {
        if let Some(sender) = processed.lock().take() {
            let _ = sender.send(job.run_id);
        }
        worker.stop()?;
        Ok(())
    }

    async fn failing_job(
        _job: GeminiBrowserJob,
        executions: Data<Arc<AtomicUsize>>,
        worker: WorkerContext,
    ) -> Result<(), BoxDynError> {
        executions.fetch_add(1, Ordering::SeqCst);
        worker.stop()?;
        Err(std::io::Error::other("probe failure").into())
    }

    async fn run_storage_once(pool: &sqlx::SqlitePool, worker_name: &str) -> String {
        let storage = SqliteStorage::new_in_queue(pool, GEMINI_BROWSER_QUEUE_NAME);
        let (tx, rx) = oneshot::channel();
        let worker = WorkerBuilder::new(worker_name)
            .backend(storage)
            .concurrency(1)
            .data(Arc::new(Mutex::new(Some(tx))))
            .build(process_one_job);
        tokio::time::timeout(Duration::from_secs(5), worker.run())
            .await
            .expect("worker timeout")
            .expect("worker run");
        tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .expect("result timeout")
            .expect("result channel")
    }

    async fn job_row(pool: &sqlx::SqlitePool, run_id: &str) -> (String, i64, i64) {
        sqlx::query_as(
            "SELECT status, attempts, max_attempts FROM Jobs WHERE job_type = ? AND idempotency_key = ?",
        )
        .bind(GEMINI_BROWSER_QUEUE_NAME)
        .bind(run_id)
        .fetch_one(pool)
        .await
        .expect("job row")
    }

    #[test]
    fn apalis_storage_uses_shared_main_extractum_db_identity() {
        let config = std::path::PathBuf::from("config");
        assert_eq!(crate::db::DB_URL, "sqlite:extractum.db");
        assert_eq!(
            crate::db::db_path_from_config_dir(&config),
            config
                .join(crate::db::APP_IDENTIFIER)
                .join(crate::db::DB_FILENAME)
        );
    }

    #[tokio::test]
    async fn apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup storage");
        for table in ["projects", "Jobs", "Workers"] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("schema");
            assert_eq!(exists, 1, "missing table {table}");
        }
        let mut storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("schema-worker"))
            .await
            .expect("enqueue");
        assert_eq!(
            run_storage_once(&pool, "schema-worker").await,
            "schema-worker"
        );
    }

    #[tokio::test]
    async fn apalis_storage_preserves_existing_sqlx_migration_history_table() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("history before");
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup storage");
        let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("history after");
        assert_eq!(after, before);
    }

    #[tokio::test]
    async fn apalis_storage_shares_extractum_db_without_locking_app_pool() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        let mut storage = open_gemini_browser_job_storage(&pool)
            .await
            .expect("storage");
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("shared-db"))
            .await
            .expect("enqueue");
        let projects: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
            .fetch_one(&pool)
            .await
            .expect("app query");
        assert_eq!(projects, 0);
    }

    #[tokio::test]
    async fn enqueue_duplicate_run_id_returns_conflict() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        let mut storage = open_gemini_browser_job_storage(&pool)
            .await
            .expect("storage");
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("duplicate"))
            .await
            .expect("first enqueue");
        let error = enqueue_gemini_browser_job_to_storage(&mut storage, test_job("duplicate"))
            .await
            .expect_err("duplicate conflict");
        assert_eq!(error.kind, crate::error::AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn enqueue_persists_job_before_worker_startup() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        let mut storage = open_gemini_browser_job_storage(&pool)
            .await
            .expect("storage");
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("before-worker"))
            .await
            .expect("enqueue");
        assert_eq!(job_row(&pool, "before-worker").await.0, "Pending");
    }

    #[tokio::test]
    async fn worker_picks_up_job_quickly_after_idle() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup");
        let storage = SqliteStorage::new_with_config(&pool, &gemini_browser_queue_config());
        let mut enqueue = storage.clone();
        let (tx, rx) = oneshot::channel::<String>();
        let worker = WorkerBuilder::new("idle-pickup")
            .backend(storage)
            .concurrency(1)
            .data(Arc::new(Mutex::new(Some(tx))))
            .build(process_one_job);
        let task = tokio::spawn(worker.run());
        tokio::time::sleep(Duration::from_millis(150)).await;
        enqueue_gemini_browser_job_to_storage(&mut enqueue, test_job("after-idle"))
            .await
            .expect("enqueue");
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(1), rx)
                .await
                .expect("pickup")
                .expect("channel"),
            "after-idle"
        );
        task.await.expect("join").expect("worker");
    }

    #[tokio::test]
    async fn restart_worker_processes_pending_job_after_runtime_restart() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        let mut before = open_gemini_browser_job_storage(&pool)
            .await
            .expect("storage");
        enqueue_gemini_browser_job_to_storage(&mut before, test_job("restart-pending"))
            .await
            .expect("enqueue");
        drop(before);
        assert_eq!(
            run_storage_once(&pool, "restart-worker").await,
            "restart-pending"
        );
    }

    #[tokio::test]
    async fn apalis_sqlite_status_probe_documents_actual_status_values() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        let mut storage = open_gemini_browser_job_storage(&pool)
            .await
            .expect("storage");
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("status-ok"))
            .await
            .expect("enqueue");
        assert_eq!(job_row(&pool, "status-ok").await.0, "Pending");
        run_storage_once(&pool, "status-worker").await;
        assert_eq!(job_row(&pool, "status-ok").await.0, "Done");
        assert_eq!(
            run_status_for_queue_state("Killed"),
            Some(crate::gemini_browser::GeminiBrowserRunStatus::Failed)
        );
    }

    #[test]
    fn gemini_browser_jobs_are_built_with_one_total_attempt() {
        let task: apalis_sqlite::SqliteTask<GeminiBrowserJob> =
            build_gemini_browser_task(test_job("one-attempt"));
        assert_eq!(task.parts.ctx.max_attempts(), 1);
        assert_eq!(task.parts.idempotency_key.as_deref(), Some("one-attempt"));
    }

    #[tokio::test]
    async fn failed_gemini_browser_job_is_not_retried() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pool = migrated_pool(&temp).await;
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup");
        let mut storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("no-retry"))
            .await
            .expect("enqueue");
        let executions = Arc::new(AtomicUsize::new(0));
        let worker = WorkerBuilder::new("no-retry")
            .backend(storage)
            .concurrency(1)
            .data(executions.clone())
            .build(failing_job);
        tokio::time::timeout(Duration::from_secs(5), worker.run())
            .await
            .expect("worker timeout")
            .expect("worker run");
        let row = job_row(&pool, "no-retry").await;
        assert!(matches!(row.0.as_str(), "Failed" | "Killed"));
        assert_eq!((row.1, row.2, executions.load(Ordering::SeqCst)), (1, 1, 1));
    }
}
