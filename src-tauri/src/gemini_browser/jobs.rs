use apalis::prelude::{IntervalStrategy, StrategyBuilder, WorkerBuilderExt};
use apalis_sqlite::TaskBuilderExt;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

pub(crate) const GEMINI_BROWSER_QUEUE_NAME: &str = "gemini-browser";
const GEMINI_BROWSER_QUEUE_POLL_INTERVAL_MS: u64 = 100;
const DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS: u64 = 20 * 60;
pub(crate) type GeminiBrowserWaiterResult =
    crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>;
type GeminiBrowserWaiterSender = tokio::sync::oneshot::Sender<GeminiBrowserWaiterResult>;
pub(crate) type GeminiBrowserWaiterReceiver =
    tokio::sync::oneshot::Receiver<GeminiBrowserWaiterResult>;
type GeminiBrowserApalisTask<IdType> =
    apalis::prelude::Task<GeminiBrowserJob, apalis_sqlite::SqliteContext, IdType>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApalisQueueInspectionMode {
    Supported,
    DegradedRunLogOnly,
}

pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
    ApalisQueueInspectionMode::DegradedRunLogOnly
}

pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
    mode: ApalisQueueInspectionMode,
) -> bool {
    matches!(mode, ApalisQueueInspectionMode::Supported)
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueuedGeminiBrowserJob {
    pub run_id: String,
    pub queue_position: Option<usize>,
}

pub(crate) trait GeminiBrowserApalisStorageAccess {
    fn pool(&self) -> &sqlx::SqlitePool;
}

impl<T, C, F> GeminiBrowserApalisStorageAccess for apalis_sqlite::SqliteStorage<T, C, F> {
    fn pool(&self) -> &sqlx::SqlitePool {
        self.pool()
    }
}

pub(crate) struct GeminiBrowserJobRuntime {
    waiters: parking_lot::Mutex<std::collections::HashMap<String, GeminiBrowserWaiterSender>>,
    cancelled_runs: parking_lot::Mutex<std::collections::HashSet<String>>,
    worker_status: tokio::sync::watch::Sender<GeminiBrowserWorkerStatus>,
    waiter_timeout: std::time::Duration,
    worker_execution_timeout: std::time::Duration,
    worker_hard_guard_timeout: std::time::Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GeminiBrowserWorkerStatus {
    Starting,
    Ready {
        started_at: String,
    },
    Failed {
        started_at: Option<String>,
        error: String,
    },
}

impl Default for GeminiBrowserJobRuntime {
    fn default() -> Self {
        let (worker_status, _) = tokio::sync::watch::channel(GeminiBrowserWorkerStatus::Starting);
        Self {
            waiters: parking_lot::Mutex::new(std::collections::HashMap::new()),
            cancelled_runs: parking_lot::Mutex::new(std::collections::HashSet::new()),
            worker_status,
            worker_execution_timeout: std::time::Duration::from_secs(
                DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS,
            ),
            waiter_timeout: std::time::Duration::from_secs(
                DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS + 5,
            ),
            worker_hard_guard_timeout: std::time::Duration::from_secs(
                DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS + 15,
            ),
        }
    }
}

impl GeminiBrowserJobRuntime {
    #[cfg(test)]
    pub(crate) fn new_for_test(worker_execution_timeout: std::time::Duration) -> Self {
        Self::new_for_test_with_timeouts(
            worker_execution_timeout + std::time::Duration::from_millis(50),
            worker_execution_timeout,
            worker_execution_timeout + std::time::Duration::from_millis(100),
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_timeouts(
        waiter_timeout: std::time::Duration,
        worker_execution_timeout: std::time::Duration,
        worker_hard_guard_timeout: std::time::Duration,
    ) -> Self {
        assert!(
            worker_execution_timeout < waiter_timeout,
            "worker execution timeout must be lower than waiter timeout"
        );
        assert!(
            waiter_timeout < worker_hard_guard_timeout,
            "waiter timeout must be lower than hard guard timeout"
        );
        Self::new_with_timeouts(
            waiter_timeout,
            worker_execution_timeout,
            worker_hard_guard_timeout,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_waiter_timeout_test(waiter_timeout: std::time::Duration) -> Self {
        Self::new_with_timeouts(
            waiter_timeout,
            waiter_timeout + std::time::Duration::from_millis(50),
            waiter_timeout + std::time::Duration::from_millis(100),
        )
    }

    fn new_with_timeouts(
        waiter_timeout: std::time::Duration,
        worker_execution_timeout: std::time::Duration,
        worker_hard_guard_timeout: std::time::Duration,
    ) -> Self {
        let (worker_status, _) = tokio::sync::watch::channel(GeminiBrowserWorkerStatus::Starting);
        Self {
            waiters: parking_lot::Mutex::new(std::collections::HashMap::new()),
            cancelled_runs: parking_lot::Mutex::new(std::collections::HashSet::new()),
            worker_status,
            waiter_timeout,
            worker_execution_timeout,
            worker_hard_guard_timeout,
        }
    }

    pub(crate) fn register_waiter(
        &self,
        run_id: &str,
    ) -> crate::error::AppResult<GeminiBrowserWaiterReceiver> {
        let mut waiters = self.waiters.lock();
        if waiters.contains_key(run_id) {
            return Err(crate::error::AppError::conflict(format!(
                "Run '{run_id}' already has an active Gemini Browser waiter"
            )));
        }

        let (sender, receiver) = tokio::sync::oneshot::channel();
        waiters.insert(run_id.to_string(), sender);
        Ok(receiver)
    }

    pub(crate) fn complete_waiter(&self, run_id: &str, result: GeminiBrowserWaiterResult) {
        if let Some(sender) = self.waiters.lock().remove(run_id) {
            let _ = sender.send(result);
        }
    }

    pub(crate) fn remove_waiter(&self, run_id: &str) {
        self.waiters.lock().remove(run_id);
    }

    pub(crate) fn has_waiter(&self, run_id: &str) -> bool {
        self.waiters.lock().contains_key(run_id)
    }

    #[cfg(test)]
    pub(crate) fn has_waiter_for_test(&self, run_id: &str) -> bool {
        self.has_waiter(run_id)
    }

    #[cfg(test)]
    pub(crate) fn worker_status_for_test(&self) -> GeminiBrowserWorkerStatus {
        self.worker_status.borrow().clone()
    }

    pub(crate) async fn wait_for_registered_result(
        &self,
        run_id: &str,
        receiver: GeminiBrowserWaiterReceiver,
    ) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult> {
        match tokio::time::timeout(self.waiter_timeout, receiver).await {
            Ok(Ok(result)) => {
                self.remove_waiter(run_id);
                result
            }
            Ok(Err(_)) => {
                self.remove_waiter(run_id);
                Err(crate::error::AppError::internal(
                    "Gemini Browser worker channel closed unexpectedly",
                ))
            }
            Err(_) => {
                self.remove_waiter(run_id);
                Err(crate::error::AppError::internal(
                    "Gemini Browser job timed out waiting for worker result",
                ))
            }
        }
    }

    pub(crate) fn worker_execution_timeout(&self) -> std::time::Duration {
        self.worker_execution_timeout
    }

    pub(crate) fn worker_hard_guard_timeout(&self) -> std::time::Duration {
        self.worker_hard_guard_timeout
    }

    pub(crate) fn mark_worker_ready(&self, started_at: String) {
        self.worker_status
            .send_replace(GeminiBrowserWorkerStatus::Ready { started_at });
    }

    pub(crate) fn mark_worker_failed(&self, error: impl Into<String>) {
        self.worker_status
            .send_replace(GeminiBrowserWorkerStatus::Failed {
                started_at: None,
                error: error.into(),
            });
    }

    pub(crate) async fn ensure_worker_ready_for_enqueue(&self) -> crate::error::AppResult<()> {
        self.ensure_worker_ready_for_enqueue_with_timeout(std::time::Duration::from_secs(5))
            .await
    }

    pub(crate) async fn ensure_worker_ready_for_enqueue_with_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> crate::error::AppResult<()> {
        match Self::worker_status_enqueue_result(self.worker_status.borrow().clone()) {
            WorkerReadinessDecision::Ready => return Ok(()),
            WorkerReadinessDecision::Failed(error) => return Err(error),
            WorkerReadinessDecision::Starting => {}
        }

        let mut receiver = self.worker_status.subscribe();
        let wait_for_ready = async move {
            loop {
                receiver.changed().await.map_err(|_| {
                    crate::error::AppError::internal(
                        "Gemini Browser worker status channel closed unexpectedly",
                    )
                })?;
                match Self::worker_status_enqueue_result(receiver.borrow().clone()) {
                    WorkerReadinessDecision::Ready => return Ok(()),
                    WorkerReadinessDecision::Failed(error) => return Err(error),
                    WorkerReadinessDecision::Starting => {}
                }
            }
        };

        tokio::time::timeout(timeout, wait_for_ready)
            .await
            .unwrap_or_else(|_| {
                Err(crate::error::AppError::internal(
                    "Gemini Browser worker is still starting",
                ))
            })
    }

    fn worker_status_enqueue_result(status: GeminiBrowserWorkerStatus) -> WorkerReadinessDecision {
        match status {
            GeminiBrowserWorkerStatus::Starting => WorkerReadinessDecision::Starting,
            GeminiBrowserWorkerStatus::Ready { .. } => WorkerReadinessDecision::Ready,
            GeminiBrowserWorkerStatus::Failed { error, .. } => {
                WorkerReadinessDecision::Failed(crate::error::AppError::internal(format!(
                    "Gemini Browser worker failed to start: {error}"
                )))
            }
        }
    }

    pub(crate) fn request_cancel(&self, run_id: &str) {
        self.cancelled_runs.lock().insert(run_id.to_string());
    }

    pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
        self.cancelled_runs.lock().contains(run_id)
    }

    pub(crate) fn clear_cancelled(&self, run_id: &str) {
        self.cancelled_runs.lock().remove(run_id);
    }
}

enum WorkerReadinessDecision {
    Starting,
    Ready,
    Failed(crate::error::AppError),
}

enum GeminiBrowserWorkerEntryDecision {
    Execute(crate::gemini_browser::GeminiBrowserRun),
    Acknowledged,
    Terminal {
        run: crate::gemini_browser::GeminiBrowserRun,
        result: crate::gemini_browser::GeminiBrowserRunResult,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GeminiBrowserArtifactMode {
    Reduced,
    Full,
}

impl GeminiBrowserArtifactMode {
    pub(crate) fn from_wire(value: Option<&str>) -> crate::error::AppResult<Self> {
        match value.unwrap_or("reduced") {
            "reduced" => Ok(Self::Reduced),
            "full" => Ok(Self::Full),
            other => Err(crate::error::AppError::validation(format!(
                "unsupported Gemini Browser artifact_mode '{other}'"
            ))),
        }
    }

    pub(crate) fn as_wire(&self) -> &'static str {
        match self {
            Self::Reduced => "reduced",
            Self::Full => "full",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct GeminiBrowserJob {
    pub run_id: String,
    pub prompt: String,
    pub source: String,
    pub artifact_mode: GeminiBrowserArtifactMode,
    pub browser_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
}

impl GeminiBrowserJob {
    pub(crate) fn run_request(&self) -> crate::gemini_browser::GeminiBrowserRunRequest {
        crate::gemini_browser::GeminiBrowserRunRequest {
            run_id: self.run_id.clone(),
            prompt: self.prompt.clone(),
            source: self.source.clone(),
            artifact_mode: self.artifact_mode.as_wire().to_string(),
        }
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

    cancel_gemini_browser_job_core(
        &runtime,
        &state,
        &runs_root,
        run_id,
        |run| emit_gemini_browser_run_change_event(handle, run),
        |result| update_terminal_status_snapshot_best_effort(handle, &state, result),
        || stop_active_gemini_browser_sidecar(handle, &state),
    )
    .await
}

async fn cancel_gemini_browser_job_core<
    EmitEvent,
    BeforeEmitTerminalSnapshot,
    StopActive,
    StopFut,
>(
    runtime: &GeminiBrowserJobRuntime,
    state: &crate::gemini_browser::GeminiBrowserState,
    runs_root: &std::path::Path,
    run_id: &str,
    mut emit_event: EmitEvent,
    mut before_emit_terminal_snapshot: BeforeEmitTerminalSnapshot,
    stop_active: StopActive,
) -> crate::error::AppResult<()>
where
    EmitEvent: FnMut(&crate::gemini_browser::GeminiBrowserRun),
    BeforeEmitTerminalSnapshot: FnMut(&crate::gemini_browser::GeminiBrowserRunResult),
    StopActive: FnOnce() -> StopFut,
    StopFut: std::future::Future<Output = crate::error::AppResult<()>>,
{
    runtime.request_cancel(run_id);

    if state.active_run_id().await.as_deref() == Some(run_id) {
        state.request_stop().await;
        stop_active().await?;
        return Ok(());
    }

    let Some(run) = run_log_entry_by_id(runs_root, run_id)? else {
        return Ok(());
    };

    if run.status == crate::gemini_browser::GeminiBrowserRunStatus::Queued {
        let result = cancelled_run_result_for_id(run_id);
        let cancelled_run = crate::gemini_browser::finish_run(runs_root, run_id, result.clone())?;
        runtime.complete_waiter(run_id, Ok(result.clone()));
        before_emit_terminal_snapshot(&result);
        emit_event(&cancelled_run);
        runtime.clear_cancelled(run_id);
        return Ok(());
    }

    if run.status.is_terminal() {
        runtime.clear_cancelled(run_id);
        return Ok(());
    }

    if run.status == crate::gemini_browser::GeminiBrowserRunStatus::Running {
        let result = crate::gemini_browser::GeminiBrowserRunResult {
            run_id: run_id.to_string(),
            status: crate::gemini_browser::GeminiBrowserRunStatus::Failed,
            text: None,
            message: Some("Gemini Browser run was running without an active sidecar".to_string()),
            manual_action: None,
            artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 0,
            debug_summary: None,
        };
        let failed_run = crate::gemini_browser::finish_run(runs_root, run_id, result.clone())?;
        runtime.complete_waiter(run_id, Ok(result.clone()));
        before_emit_terminal_snapshot(&result);
        emit_event(&failed_run);
        runtime.clear_cancelled(run_id);
    }

    Ok(())
}

async fn stop_active_gemini_browser_sidecar(
    handle: &tauri::AppHandle,
    state: &crate::gemini_browser::GeminiBrowserState,
) -> crate::error::AppResult<()> {
    crate::gemini_browser::sidecar::stop(handle, state).await
}

fn run_log_entry_by_id(
    runs_root: &std::path::Path,
    run_id: &str,
) -> crate::error::AppResult<Option<crate::gemini_browser::GeminiBrowserRun>> {
    Ok(crate::gemini_browser::list_runs(runs_root, usize::MAX)?
        .runs
        .into_iter()
        .find(|run| run.run_id == run_id))
}

fn run_log_is_cancelled(
    runs_root: &std::path::Path,
    run_id: &str,
) -> crate::error::AppResult<bool> {
    Ok(run_log_entry_by_id(runs_root, run_id)?
        .is_some_and(|run| run.status == crate::gemini_browser::GeminiBrowserRunStatus::Cancelled))
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
    start_gemini_browser_job_worker_core(runtime.inner(), || {
        let setup_handle = handle.clone();
        async move {
            let pool = crate::db::get_pool(&setup_handle).await?;
            setup_gemini_browser_apalis_storage(&pool).await?;
            let runs_root = crate::gemini_browser::runs_dir(&setup_handle)?;
            reconcile_gemini_browser_run_log_at_startup(
                &runs_root,
                apalis_queue_inspection_mode(),
                |_run_id| Ok(None),
            )?;
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
}

async fn process_gemini_browser_job(
    job: GeminiBrowserJob,
    handle: apalis::prelude::Data<tauri::AppHandle>,
) -> Result<(), apalis::prelude::BoxDynError> {
    let handle = &*handle;
    let runtime = handle.state::<GeminiBrowserJobRuntime>();
    let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    let runs_root = crate::gemini_browser::runs_dir(handle)?;

    match reconcile_gemini_browser_worker_entry(&runtime, &state, &runs_root, &job).await? {
        GeminiBrowserWorkerEntryDecision::Execute(running_run) => {
            update_running_status_snapshot_best_effort(handle, &state, &job.run_id);
            emit_gemini_browser_run_change_event(handle, &running_run);
        }
        GeminiBrowserWorkerEntryDecision::Acknowledged => return Ok(()),
        GeminiBrowserWorkerEntryDecision::Terminal { run, result } => {
            update_terminal_status_snapshot_best_effort(handle, &state, &result);
            emit_gemini_browser_run_change_event(handle, &run);
            return Ok(());
        }
    }

    let request = job.run_request();
    let fallback_request = request.clone();
    let browser_profile_dir =
        crate::gemini_browser::path_string(&crate::gemini_browser::profile_dir(handle)?);
    let artifact_dir =
        crate::gemini_browser::path_string(&crate::gemini_browser::run_dir(handle, &job.run_id)?);
    let browser_config = job.browser_config.clone();
    let sidecar_future = async {
        Ok(
            match crate::gemini_browser::sidecar::send_single(
                handle,
                &state,
                request,
                browser_profile_dir,
                artifact_dir,
                browser_config,
            )
            .await
            {
                Ok(result) => result,
                Err(_error) => {
                    crate::gemini_browser::sidecar::sidecar_unavailable_result(fallback_request)
                }
            },
        )
    };

    let result = run_job_with_execution_timeout(
        handle,
        &runtime,
        &state,
        &runs_root,
        job.clone(),
        sidecar_future,
    )
    .await?;

    if !is_worker_timeout_result(&job, &result) {
        finish_completed_job(handle, &runtime, &state, &runs_root, &job, result).await?;
    }

    Ok(())
}

fn reconcile_gemini_browser_run_log_at_startup<Lookup>(
    runs_root: &std::path::Path,
    mode: ApalisQueueInspectionMode,
    apalis_status_for_run: Lookup,
) -> crate::error::AppResult<()>
where
    Lookup:
        Fn(&str) -> crate::error::AppResult<Option<crate::gemini_browser::GeminiBrowserRunStatus>>,
{
    let runs = crate::gemini_browser::list_runs(runs_root, usize::MAX)?.runs;
    for run in runs {
        if run.status.is_terminal() {
            continue;
        }

        if run.status == crate::gemini_browser::GeminiBrowserRunStatus::Running {
            let result = if startup_reconciliation_checks_queued_runs_against_apalis(mode) {
                match apalis_status_for_run(&run.run_id)? {
                    Some(status) if status.is_terminal() => {
                        terminal_apalis_state_result(&run.run_id, status)
                    }
                    _ => interrupted_worker_result(&run.run_id),
                }
            } else {
                interrupted_worker_result(&run.run_id)
            };
            crate::gemini_browser::finish_run(runs_root, &run.run_id, result)?;
            continue;
        }

        if run.status == crate::gemini_browser::GeminiBrowserRunStatus::Queued
            && startup_reconciliation_checks_queued_runs_against_apalis(mode)
        {
            match apalis_status_for_run(&run.run_id)? {
                Some(crate::gemini_browser::GeminiBrowserRunStatus::Queued) => {}
                Some(crate::gemini_browser::GeminiBrowserRunStatus::Running) => {
                    crate::gemini_browser::finish_run(
                        runs_root,
                        &run.run_id,
                        failed_run_result_for_id(
                            &run.run_id,
                            "Gemini Browser queue state was running without an active sidecar",
                        ),
                    )?;
                }
                Some(status) if status.is_terminal() => {
                    crate::gemini_browser::finish_run(
                        runs_root,
                        &run.run_id,
                        terminal_apalis_state_result(&run.run_id, status),
                    )?;
                }
                Some(_) => {}
                None => {
                    crate::gemini_browser::finish_run(
                        runs_root,
                        &run.run_id,
                        failed_run_result_for_id(
                            &run.run_id,
                            "Gemini Browser queued job was missing from Apalis storage",
                        ),
                    )?;
                }
            }
        }
    }

    Ok(())
}

async fn reconcile_gemini_browser_worker_entry(
    runtime: &GeminiBrowserJobRuntime,
    state: &crate::gemini_browser::GeminiBrowserState,
    runs_root: &std::path::Path,
    job: &GeminiBrowserJob,
) -> crate::error::AppResult<GeminiBrowserWorkerEntryDecision> {
    if runtime.is_cancelled(&job.run_id) {
        let result = cancelled_run_result(job);
        let run = crate::gemini_browser::finish_run(runs_root, &job.run_id, result.clone())?;
        runtime.complete_waiter(&job.run_id, Ok(result.clone()));
        runtime.clear_cancelled(&job.run_id);
        return Ok(GeminiBrowserWorkerEntryDecision::Terminal { run, result });
    }

    let Some(run) = run_log_entry_by_id(runs_root, &job.run_id)? else {
        runtime.clear_cancelled(&job.run_id);
        return Ok(GeminiBrowserWorkerEntryDecision::Acknowledged);
    };

    if run.status.is_terminal() {
        if let Some(result) = run.result {
            runtime.complete_waiter(&job.run_id, Ok(result));
        }
        runtime.clear_cancelled(&job.run_id);
        return Ok(GeminiBrowserWorkerEntryDecision::Acknowledged);
    }

    if run.status == crate::gemini_browser::GeminiBrowserRunStatus::Running {
        if state.active_run_id().await.as_deref() == Some(job.run_id.as_str()) {
            return Ok(GeminiBrowserWorkerEntryDecision::Acknowledged);
        }
        let result = interrupted_worker_result(&job.run_id);
        let run = crate::gemini_browser::finish_run(runs_root, &job.run_id, result.clone())?;
        runtime.complete_waiter(&job.run_id, Ok(result.clone()));
        runtime.clear_cancelled(&job.run_id);
        return Ok(GeminiBrowserWorkerEntryDecision::Terminal { run, result });
    }

    if run.status != crate::gemini_browser::GeminiBrowserRunStatus::Queued {
        runtime.clear_cancelled(&job.run_id);
        return Ok(GeminiBrowserWorkerEntryDecision::Acknowledged);
    }

    let running_run = crate::gemini_browser::mark_running(runs_root, &job.run_id)?;
    let _token = state.start_run(job.run_id.clone()).await;
    Ok(GeminiBrowserWorkerEntryDecision::Execute(running_run))
}

async fn run_job_with_execution_timeout<Fut>(
    handle: &tauri::AppHandle,
    runtime: &GeminiBrowserJobRuntime,
    state: &crate::gemini_browser::GeminiBrowserState,
    runs_dir: &std::path::Path,
    job: GeminiBrowserJob,
    sidecar_future: Fut,
) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
where
    Fut: std::future::Future<
        Output = crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
    >,
{
    match tokio::time::timeout(runtime.worker_execution_timeout(), sidecar_future).await {
        Ok(result) => result,
        Err(_elapsed) => finish_timed_out_job(handle, runtime, state, runs_dir, job).await,
    }
}

async fn finish_timed_out_job(
    handle: &tauri::AppHandle,
    runtime: &GeminiBrowserJobRuntime,
    state: &crate::gemini_browser::GeminiBrowserState,
    runs_dir: &std::path::Path,
    job: GeminiBrowserJob,
) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult> {
    let should_stop_sidecar = state.active_run_id().await.as_deref() == Some(job.run_id.as_str());
    let result = timeout_failed_run_result(&job, runtime.worker_execution_timeout());
    let timed_out_run = crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
    state.finish_run(&job.run_id).await;
    runtime.complete_waiter(&job.run_id, Ok(result.clone()));
    runtime.clear_cancelled(&job.run_id);
    update_terminal_status_snapshot_best_effort(handle, state, &result);
    emit_gemini_browser_run_change_event(handle, &timed_out_run);

    if should_stop_sidecar {
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            crate::gemini_browser::sidecar::stop(handle, state),
        )
        .await;
    }

    Ok(result)
}

async fn finish_completed_job(
    handle: &tauri::AppHandle,
    runtime: &GeminiBrowserJobRuntime,
    state: &crate::gemini_browser::GeminiBrowserState,
    runs_dir: &std::path::Path,
    job: &GeminiBrowserJob,
    result: crate::gemini_browser::GeminiBrowserRunResult,
) -> crate::error::AppResult<()> {
    let completed_run = crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
    state.finish_run(&job.run_id).await;
    runtime.complete_waiter(&job.run_id, Ok(result.clone()));
    runtime.clear_cancelled(&job.run_id);
    update_terminal_status_snapshot_best_effort(handle, state, &result);
    emit_gemini_browser_run_change_event(handle, &completed_run);
    Ok(())
}

fn update_running_status_snapshot(
    handle: &tauri::AppHandle,
    state: &crate::gemini_browser::GeminiBrowserState,
    run_id: &str,
) -> crate::error::AppResult<()> {
    state.update_status_snapshot(handle, |status| {
        status.status = crate::gemini_browser::GeminiBrowserProviderStatusKind::Running;
        status.active_run_id = Some(run_id.to_string());
        status.queue_depth = 0;
        status.latest_message = Some("Running".to_string());
        status.manual_action = None;
    })
}

fn update_running_status_snapshot_best_effort(
    handle: &tauri::AppHandle,
    state: &crate::gemini_browser::GeminiBrowserState,
    run_id: &str,
) {
    if let Err(error) = update_running_status_snapshot(handle, state, run_id) {
        eprintln!("Gemini Browser running status snapshot update failed: {error}");
    }
}

fn update_terminal_status_snapshot(
    handle: &tauri::AppHandle,
    state: &crate::gemini_browser::GeminiBrowserState,
    result: &crate::gemini_browser::GeminiBrowserRunResult,
) -> crate::error::AppResult<()> {
    state.update_status_snapshot(handle, |status| {
        status.status =
            crate::gemini_browser::GeminiBrowserState::provider_status_kind_for_run_status(
                &result.status,
            );
        status.active_run_id = None;
        status.queue_depth = 0;
        status.latest_message = result.message.clone();
        status.manual_action = result.manual_action.clone();
    })
}

fn update_terminal_status_snapshot_best_effort(
    handle: &tauri::AppHandle,
    state: &crate::gemini_browser::GeminiBrowserState,
    result: &crate::gemini_browser::GeminiBrowserRunResult,
) {
    if let Err(error) = update_terminal_status_snapshot(handle, state, result) {
        eprintln!("Gemini Browser terminal status snapshot update failed: {error}");
    }
}

fn emit_gemini_browser_run_change_event(
    handle: &tauri::AppHandle,
    run: &crate::gemini_browser::GeminiBrowserRun,
) {
    crate::gemini_browser::commands::emit_run_change_event_core(run, |event| {
        handle
            .emit(
                crate::gemini_browser::commands::GEMINI_BROWSER_RUN_CHANGE_EVENT,
                event,
            )
            .map_err(|error| error.to_string())
    });
}

fn cancelled_run_result(job: &GeminiBrowserJob) -> crate::gemini_browser::GeminiBrowserRunResult {
    cancelled_run_result_for_id(&job.run_id)
}

fn cancelled_run_result_for_id(run_id: &str) -> crate::gemini_browser::GeminiBrowserRunResult {
    crate::gemini_browser::GeminiBrowserRunResult {
        run_id: run_id.to_string(),
        status: crate::gemini_browser::GeminiBrowserRunStatus::Cancelled,
        text: None,
        message: Some("Cancelled".to_string()),
        manual_action: None,
        artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 0,
        debug_summary: None,
    }
}

fn interrupted_worker_result(run_id: &str) -> crate::gemini_browser::GeminiBrowserRunResult {
    failed_run_result_for_id(
        run_id,
        "Gemini Browser worker was interrupted before completion",
    )
}

fn failed_run_result_for_id(
    run_id: &str,
    message: &str,
) -> crate::gemini_browser::GeminiBrowserRunResult {
    crate::gemini_browser::GeminiBrowserRunResult {
        run_id: run_id.to_string(),
        status: crate::gemini_browser::GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some(message.to_string()),
        manual_action: None,
        artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 0,
        debug_summary: None,
    }
}

fn terminal_apalis_state_result(
    run_id: &str,
    status: crate::gemini_browser::GeminiBrowserRunStatus,
) -> crate::gemini_browser::GeminiBrowserRunResult {
    match status {
        crate::gemini_browser::GeminiBrowserRunStatus::Cancelled => {
            cancelled_run_result_for_id(run_id)
        }
        crate::gemini_browser::GeminiBrowserRunStatus::Failed
        | crate::gemini_browser::GeminiBrowserRunStatus::Timeout
        | crate::gemini_browser::GeminiBrowserRunStatus::BrowserCrashed
        | crate::gemini_browser::GeminiBrowserRunStatus::Blocked
        | crate::gemini_browser::GeminiBrowserRunStatus::NeedsLogin
        | crate::gemini_browser::GeminiBrowserRunStatus::NeedsManualAction => {
            crate::gemini_browser::GeminiBrowserRunResult {
                run_id: run_id.to_string(),
                status,
                text: None,
                message: Some("Gemini Browser Apalis job failed before completion".to_string()),
                manual_action: None,
                artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
                elapsed_ms: 0,
                debug_summary: None,
            }
        }
        _ => failed_run_result_for_id(
            run_id,
            "Gemini Browser Apalis job completed before run log captured a result",
        ),
    }
}

fn is_worker_timeout_result(
    job: &GeminiBrowserJob,
    result: &crate::gemini_browser::GeminiBrowserRunResult,
) -> bool {
    result.run_id == job.run_id
        && result.status == crate::gemini_browser::GeminiBrowserRunStatus::Failed
        && result
            .message
            .as_deref()
            .is_some_and(|message| message.starts_with("Gemini Browser job timed out after "))
}

async fn start_gemini_browser_job_worker_core<Setup, SetupFut, RunFut, RunError>(
    runtime: &GeminiBrowserJobRuntime,
    setup_worker: Setup,
) -> crate::error::AppResult<()>
where
    Setup: FnOnce() -> SetupFut,
    SetupFut: std::future::Future<Output = crate::error::AppResult<RunFut>>,
    RunFut: std::future::Future<Output = Result<(), RunError>>,
    RunError: std::fmt::Display,
{
    let worker = match setup_worker().await {
        Ok(worker) => worker,
        Err(error) => {
            runtime.mark_worker_failed(error.to_string());
            return Err(error);
        }
    };

    runtime.mark_worker_ready(current_time_rfc3339());

    let error = match worker.await {
        Ok(()) => {
            crate::error::AppError::internal("Gemini Browser job worker stopped unexpectedly")
        }
        Err(error) => {
            let message = error.to_string();
            if message.trim().is_empty() {
                crate::error::AppError::internal("Gemini Browser job worker stopped unexpectedly")
            } else {
                crate::error::AppError::internal(message)
            }
        }
    };
    runtime.mark_worker_failed(error.to_string());
    Err(error)
}

fn current_time_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc().to_string())
}

fn timeout_failed_run_result(
    job: &GeminiBrowserJob,
    timeout: std::time::Duration,
) -> crate::gemini_browser::GeminiBrowserRunResult {
    crate::gemini_browser::GeminiBrowserRunResult {
        run_id: job.run_id.clone(),
        status: crate::gemini_browser::GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some(format!(
            "Gemini Browser job timed out after {}s",
            timeout.as_secs()
        )),
        manual_action: None,
        artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
        elapsed_ms: timeout.as_millis().try_into().unwrap_or(u64::MAX),
        debug_summary: None,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    };

    use apalis::prelude::{
        BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    };
    use apalis_sqlite::SqliteStorage;
    use parking_lot::Mutex;
    use sqlx::{
        sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
        SqlitePool,
    };
    use tokio::sync::oneshot;

    use super::{
        apalis_queue_inspection_mode, build_gemini_browser_task, cancel_gemini_browser_job_core,
        cancelled_run_result, cancelled_run_result_for_id, enqueue_gemini_browser_job_to_storage,
        gemini_browser_queue_config, open_gemini_browser_job_storage,
        reconcile_gemini_browser_run_log_at_startup, reconcile_gemini_browser_worker_entry,
        run_log_is_cancelled, run_status_for_queue_state, setup_gemini_browser_apalis_storage,
        start_gemini_browser_job_worker_core,
        startup_reconciliation_checks_queued_runs_against_apalis, timeout_failed_run_result,
        ApalisQueueInspectionMode, GeminiBrowserArtifactMode, GeminiBrowserJob,
        GeminiBrowserJobRuntime, GeminiBrowserWorkerEntryDecision, GeminiBrowserWorkerStatus,
        GEMINI_BROWSER_QUEUE_NAME,
    };

    const PRODUCT_TABLES: [&str; 4] = [
        "prompt_pack_runs",
        "prompt_pack_stage_runs",
        "prompt_pack_versions",
        "projects",
    ];

    #[test]
    fn gemini_browser_job_serializes_queue_payload() {
        let job = GeminiBrowserJob {
            run_id: "run-1".to_string(),
            prompt: "hello".to_string(),
            source: "settings_test".to_string(),
            artifact_mode: GeminiBrowserArtifactMode::Reduced,
            browser_config: Some(crate::gemini_browser::GeminiBrowserProviderConfig {
                mode: crate::gemini_browser::GeminiBrowserProviderMode::CdpAttach,
                cdp_endpoint: Some("http://127.0.0.1:9222".to_string()),
            }),
        };

        let json = serde_json::to_string(&job).expect("serialize job");
        let decoded: GeminiBrowserJob = serde_json::from_str(&json).expect("decode job");

        assert_eq!(decoded, job);
        assert_eq!(decoded.artifact_mode.as_wire(), "reduced");
        assert_eq!(
            decoded
                .browser_config
                .as_ref()
                .and_then(|config| config.cdp_endpoint.as_deref()),
            Some("http://127.0.0.1:9222")
        );
    }

    #[test]
    fn apalis_storage_uses_shared_main_extractum_db_identity() {
        let config_dir = std::path::PathBuf::from("config");

        assert_eq!(crate::db::APP_IDENTIFIER, "org.ai.extractum");
        assert_eq!(crate::db::DB_FILENAME, "extractum.db");
        assert_eq!(crate::db::DB_URL, "sqlite:extractum.db");
        assert_eq!(
            crate::db::db_path_from_config_dir(&config_dir),
            config_dir.join("org.ai.extractum").join("extractum.db")
        );
    }

    #[tokio::test]
    async fn apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");

        let product_schema_before = table_sql_by_name(&pool).await;
        for table in PRODUCT_TABLES {
            assert!(
                product_schema_before.contains_key(table),
                "missing product table before Apalis setup: {table}"
            );
        }
        for table in ["Jobs", "Workers"] {
            assert!(
                product_schema_before.contains_key(table),
                "missing app-managed Apalis table before Apalis setup: {table}"
            );
        }

        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");

        let product_schema_after = table_sql_by_name(&pool).await;
        for table in PRODUCT_TABLES {
            assert!(
                product_schema_after.contains_key(table),
                "missing product table after Apalis setup: {table}"
            );
            assert_eq!(
                product_schema_after.get(table),
                product_schema_before.get(table),
                "product table schema changed after Apalis setup: {table}"
            );
        }
        for table in ["Jobs", "Workers"] {
            assert!(
                product_schema_after.contains_key(table),
                "missing app-managed Apalis table after Apalis setup: {table}"
            );
            assert_eq!(
                product_schema_after.get(table),
                product_schema_before.get(table),
                "Apalis table schema changed after storage setup: {table}"
            );
        }

        let mut storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        storage
            .push(GeminiBrowserJob {
                run_id: "run-apalis-smoke".to_string(),
                prompt: "hello".to_string(),
                source: "settings_test".to_string(),
                artifact_mode: GeminiBrowserArtifactMode::Reduced,
                browser_config: None,
            })
            .await
            .expect("push Apalis job");

        let (processed_tx, processed_rx) = oneshot::channel::<String>();
        let processed_tx = Arc::new(Mutex::new(Some(processed_tx)));
        let worker = WorkerBuilder::new("gemini-browser")
            .backend(storage)
            .concurrency(1)
            .data(processed_tx)
            .build(process_one_job);

        tokio::time::timeout(Duration::from_secs(5), worker.run())
            .await
            .expect("worker stops before timeout")
            .expect("worker run succeeds");

        let processed_run_id = tokio::time::timeout(Duration::from_secs(5), processed_rx)
            .await
            .expect("processed run id before timeout")
            .expect("processed sender remains open");
        assert_eq!(processed_run_id, "run-apalis-smoke");
    }

    #[tokio::test]
    async fn apalis_storage_preserves_existing_sqlx_migration_history_table() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        sqlx::raw_sql(
            "CREATE TABLE _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create migration history fixture");
        sqlx::query(
            "INSERT INTO _sqlx_migrations (
                version,
                description,
                success,
                checksum,
                execution_time
            ) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(42_i64)
        .bind("fixture migration")
        .bind(true)
        .bind(vec![1_u8, 2, 3])
        .bind(7_i64)
        .execute(&pool)
        .await
        .expect("seed migration history fixture");

        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");

        let row = sqlx::query_as::<_, (i64, String, bool, Vec<u8>, i64)>(
            "SELECT version, description, success, checksum, execution_time
             FROM _sqlx_migrations
             WHERE version = ?",
        )
        .bind(42_i64)
        .fetch_one(&pool)
        .await
        .expect("read seeded migration row");

        assert_eq!(
            row,
            (
                42_i64,
                "fixture migration".to_string(),
                true,
                vec![1_u8, 2, 3],
                7_i64
            )
        );
    }

    #[tokio::test]
    async fn apalis_storage_shares_extractum_db_without_locking_app_pool() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");

        let mut storage = open_gemini_browser_job_storage(&pool)
            .await
            .expect("open Apalis SQLite storage");

        let project_count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
            .fetch_one(&pool)
            .await
            .expect("read product table while Apalis storage exists");

        storage
            .push(GeminiBrowserJob {
                run_id: "run-apalis-locking".to_string(),
                prompt: "hello".to_string(),
                source: "settings_test".to_string(),
                artifact_mode: GeminiBrowserArtifactMode::Reduced,
                browser_config: None,
            })
            .await
            .expect("push Apalis job while app pool remains open");

        let project_count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
            .fetch_one(&pool)
            .await
            .expect("read product table after Apalis push");

        assert_eq!(project_count_after, project_count_before);

        let tables = table_sql_by_name(&pool).await;
        for table in PRODUCT_TABLES {
            assert!(
                tables.contains_key(table),
                "missing product table after Apalis push: {table}"
            );
        }

        assert_eq!(GEMINI_BROWSER_QUEUE_NAME, "gemini-browser");
    }

    #[tokio::test]
    async fn enqueue_duplicate_run_id_returns_conflict() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");

        let mut storage = open_gemini_browser_job_storage(&pool)
            .await
            .expect("open Apalis SQLite storage");

        let first = enqueue_gemini_browser_job_to_storage(
            &mut storage,
            test_job("run-duplicate-idempotency"),
        )
        .await
        .expect("first enqueue succeeds");
        assert_eq!(first.run_id, "run-duplicate-idempotency");
        assert_eq!(first.queue_position, None);

        let stored_jobs = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT job_type, idempotency_key FROM Jobs ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .expect("read Apalis jobs after first enqueue");
        assert_eq!(
            stored_jobs,
            vec![(
                GEMINI_BROWSER_QUEUE_NAME.to_string(),
                Some("run-duplicate-idempotency".to_string())
            )]
        );

        let error = enqueue_gemini_browser_job_to_storage(
            &mut storage,
            test_job("run-duplicate-idempotency"),
        )
        .await
        .expect_err("duplicate run id returns conflict");

        assert_eq!(error.kind, crate::error::AppErrorKind::Conflict);
        assert_eq!(
            error.message,
            "Gemini Browser job with this run_id is already queued or running"
        );
    }

    #[tokio::test]
    async fn enqueue_persists_job_before_worker_startup() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");

        let mut storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("run-before-worker"))
            .await
            .expect("enqueue before worker startup");

        let (processed_tx, processed_rx) = oneshot::channel::<String>();
        let processed_tx = Arc::new(Mutex::new(Some(processed_tx)));
        let worker = WorkerBuilder::new("gemini-browser")
            .backend(storage)
            .concurrency(1)
            .data(processed_tx)
            .build(process_one_job);

        tokio::time::timeout(Duration::from_secs(5), worker.run())
            .await
            .expect("worker stops before timeout")
            .expect("worker run succeeds");

        let processed_run_id = tokio::time::timeout(Duration::from_secs(5), processed_rx)
            .await
            .expect("processed run id before timeout")
            .expect("processed sender remains open");
        assert_eq!(processed_run_id, "run-before-worker");
    }

    #[tokio::test]
    async fn worker_picks_up_job_quickly_after_idle() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");

        let storage = SqliteStorage::new_with_config(&pool, &gemini_browser_queue_config());
        let mut enqueue_storage = storage.clone();
        let (processed_tx, processed_rx) = oneshot::channel::<String>();
        let processed_tx = Arc::new(Mutex::new(Some(processed_tx)));
        let worker = WorkerBuilder::new("gemini-browser-idle-pickup")
            .backend(storage)
            .concurrency(1)
            .data(processed_tx)
            .build(process_one_job);
        let worker_task = tokio::spawn(async move { worker.run().await });

        tokio::time::sleep(Duration::from_secs(2)).await;
        enqueue_gemini_browser_job_to_storage(
            &mut enqueue_storage,
            test_job("run-after-idle-pickup"),
        )
        .await
        .expect("enqueue after worker idle");

        let processed_run_id = tokio::time::timeout(Duration::from_millis(500), processed_rx)
            .await
            .expect("worker picks up idle job without long polling backoff")
            .expect("processed sender remains open");
        assert_eq!(processed_run_id, "run-after-idle-pickup");

        tokio::time::timeout(Duration::from_secs(5), worker_task)
            .await
            .expect("worker task stops before timeout")
            .expect("worker task joins")
            .expect("worker run succeeds");
    }

    #[tokio::test]
    async fn restart_worker_processes_pending_job_after_runtime_restart() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;
        let runs_dir = Arc::new(temp_dir.path().join("runs"));
        let job = test_job("run-restart-pending");

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");
        crate::gemini_browser::create_queued_run(
            runs_dir.as_ref(),
            &job.run_id,
            &job.source,
            &job.prompt,
        )
        .expect("create queued run");

        let first_runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let mut enqueue_storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut enqueue_storage, job.clone())
            .await
            .expect("enqueue before restart");
        drop(first_runtime);

        let runtime = Arc::new(GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(
            1,
        )));
        let state = Arc::new(crate::gemini_browser::GeminiBrowserState::new());
        let executions = Arc::new(AtomicUsize::new(0));
        let storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        let worker = WorkerBuilder::new("gemini-browser-restart-pending")
            .backend(storage)
            .concurrency(1)
            .data(runtime)
            .data(state)
            .data(runs_dir.clone())
            .data(executions.clone())
            .build(restart_recovery_test_handler);

        tokio::time::timeout(Duration::from_secs(5), worker.run())
            .await
            .expect("worker stops before timeout")
            .expect("worker run succeeds");

        assert_eq!(executions.load(Ordering::SeqCst), 1);
        assert_eq!(
            read_run_by_id(runs_dir.as_ref(), &job.run_id).status,
            crate::gemini_browser::GeminiBrowserRunStatus::Ok
        );
    }

    #[test]
    fn restart_reconciliation_degraded_leaves_queued_run_log_records() {
        let temp = tempfile::tempdir().expect("temp dir");
        let job = test_job("run-restart-degraded-queued");
        crate::gemini_browser::create_queued_run(
            temp.path(),
            &job.run_id,
            &job.source,
            &job.prompt,
        )
        .expect("create queued run");

        reconcile_gemini_browser_run_log_at_startup(
            temp.path(),
            ApalisQueueInspectionMode::DegradedRunLogOnly,
            |_run_id| Ok(None),
        )
        .expect("reconcile degraded startup");

        assert_eq!(
            read_run_by_id(temp.path(), &job.run_id).status,
            crate::gemini_browser::GeminiBrowserRunStatus::Queued
        );
    }

    #[test]
    fn restart_reconciliation_matrix_handles_supported_apalis_states() {
        let temp = tempfile::tempdir().expect("temp dir");
        for run_id in [
            "run-restart-queued-present",
            "run-restart-queued-running",
            "run-restart-queued-missing",
            "run-restart-running-missing",
            "run-restart-running-killed",
            "run-restart-terminal-cancelled",
        ] {
            crate::gemini_browser::create_queued_run(temp.path(), run_id, "settings_test", "hello")
                .expect("create queued run");
        }
        crate::gemini_browser::mark_running(temp.path(), "run-restart-running-missing")
            .expect("mark running missing");
        crate::gemini_browser::mark_running(temp.path(), "run-restart-running-killed")
            .expect("mark running killed");
        crate::gemini_browser::finish_run(
            temp.path(),
            "run-restart-terminal-cancelled",
            cancelled_run_result_for_id("run-restart-terminal-cancelled"),
        )
        .expect("finish cancelled");

        reconcile_gemini_browser_run_log_at_startup(
            temp.path(),
            ApalisQueueInspectionMode::Supported,
            |run_id| {
                Ok(match run_id {
                    "run-restart-queued-present" => {
                        Some(crate::gemini_browser::GeminiBrowserRunStatus::Queued)
                    }
                    "run-restart-queued-running" => {
                        Some(crate::gemini_browser::GeminiBrowserRunStatus::Running)
                    }
                    "run-restart-running-killed" => {
                        Some(crate::gemini_browser::GeminiBrowserRunStatus::Failed)
                    }
                    "run-restart-terminal-cancelled" => {
                        Some(crate::gemini_browser::GeminiBrowserRunStatus::Queued)
                    }
                    _ => None,
                })
            },
        )
        .expect("reconcile supported startup");

        assert_eq!(
            read_run_by_id(temp.path(), "run-restart-queued-present").status,
            crate::gemini_browser::GeminiBrowserRunStatus::Queued
        );
        assert_run_status_message(
            temp.path(),
            "run-restart-queued-running",
            crate::gemini_browser::GeminiBrowserRunStatus::Failed,
            "Gemini Browser queue state was running without an active sidecar",
        );
        assert_run_status_message(
            temp.path(),
            "run-restart-queued-missing",
            crate::gemini_browser::GeminiBrowserRunStatus::Failed,
            "Gemini Browser queued job was missing from Apalis storage",
        );
        assert_run_status_message(
            temp.path(),
            "run-restart-running-missing",
            crate::gemini_browser::GeminiBrowserRunStatus::Failed,
            "Gemini Browser worker was interrupted before completion",
        );
        assert_run_status_message(
            temp.path(),
            "run-restart-running-killed",
            crate::gemini_browser::GeminiBrowserRunStatus::Failed,
            "Gemini Browser Apalis job failed before completion",
        );
        assert_eq!(
            read_run_by_id(temp.path(), "run-restart-terminal-cancelled").status,
            crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn restart_worker_entry_skips_terminal_cancelled_run_log() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let state = crate::gemini_browser::GeminiBrowserState::new();
        let job = test_job("run-restart-terminal-cancelled-entry");
        let executions = Arc::new(AtomicUsize::new(0));
        crate::gemini_browser::create_queued_run(
            temp.path(),
            &job.run_id,
            &job.source,
            &job.prompt,
        )
        .expect("create queued run");
        crate::gemini_browser::finish_run(
            temp.path(),
            &job.run_id,
            cancelled_run_result_for_id(&job.run_id),
        )
        .expect("finish cancelled");

        let decision = run_reconciled_job_for_test(
            temp.path(),
            &runtime,
            &state,
            job.clone(),
            executions.clone(),
            || async { Ok(ok_run_result(&job.run_id, "should not run")) },
        )
        .await
        .expect("ack terminal cancelled");

        assert_eq!(decision, TestWorkerEntryDecision::Acknowledged);
        assert_eq!(executions.load(Ordering::SeqCst), 0);
        assert_eq!(
            read_run_by_id(temp.path(), &job.run_id).status,
            crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn restart_worker_entry_acknowledges_missing_run_log_without_sidecar() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let state = crate::gemini_browser::GeminiBrowserState::new();
        let job = test_job("run-restart-missing-run-log-entry");
        let executions = Arc::new(AtomicUsize::new(0));

        let decision = run_reconciled_job_for_test(
            temp.path(),
            &runtime,
            &state,
            job,
            executions.clone(),
            || async {
                Ok(ok_run_result(
                    "run-restart-missing-run-log-entry",
                    "should not run",
                ))
            },
        )
        .await
        .expect("ack missing run log");

        assert_eq!(decision, TestWorkerEntryDecision::Acknowledged);
        assert_eq!(executions.load(Ordering::SeqCst), 0);
        assert!(crate::gemini_browser::list_runs(temp.path(), 10)
            .expect("list runs")
            .runs
            .is_empty());
    }

    #[test]
    fn degraded_apalis_queue_inspection_leaves_queued_run_log_records_for_worker_entry() {
        assert_eq!(
            apalis_queue_inspection_mode(),
            ApalisQueueInspectionMode::DegradedRunLogOnly
        );
        assert!(!startup_reconciliation_checks_queued_runs_against_apalis(
            apalis_queue_inspection_mode()
        ));
    }

    #[tokio::test]
    async fn worker_status_blocks_enqueue_when_startup_failed() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        runtime.mark_worker_failed("storage open failed");

        let error = runtime
            .ensure_worker_ready_for_enqueue()
            .await
            .expect_err("worker failed");

        assert!(error.to_string().contains("storage open failed"));
    }

    #[tokio::test]
    async fn worker_status_allows_enqueue_after_ready() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        runtime.mark_worker_ready("2026-06-22T00:00:00Z".to_string());

        runtime
            .ensure_worker_ready_for_enqueue()
            .await
            .expect("worker ready");
    }

    #[tokio::test]
    async fn worker_status_times_out_while_starting() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));

        let error = runtime
            .ensure_worker_ready_for_enqueue_with_timeout(Duration::from_millis(1))
            .await
            .expect_err("still starting");

        assert!(error.to_string().contains("worker is still starting"));
    }

    #[tokio::test]
    async fn waiter_receives_terminal_worker_result() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let receiver = runtime
            .register_waiter("run-waiter-1")
            .expect("register waiter");
        let result = ok_run_result("run-waiter-1", "answer");

        runtime.complete_waiter("run-waiter-1", Ok(result.clone()));

        assert_eq!(
            receiver.await.expect("waiter open").expect("worker result"),
            result
        );
    }

    #[tokio::test]
    async fn wait_for_result_removes_waiter_on_timeout() {
        let runtime =
            GeminiBrowserJobRuntime::new_for_waiter_timeout_test(Duration::from_millis(1));
        let receiver = runtime
            .register_waiter("run-timeout")
            .expect("register waiter");
        let error = runtime
            .wait_for_registered_result("run-timeout", receiver)
            .await
            .expect_err("timeout error");

        assert!(error
            .to_string()
            .contains("timed out waiting for worker result"));
        assert!(!runtime.has_waiter_for_test("run-timeout"));
    }

    #[tokio::test]
    async fn wait_for_result_removes_waiter_when_worker_channel_closes() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let receiver = runtime
            .register_waiter("run-channel-closed")
            .expect("register waiter");

        runtime.remove_waiter("run-channel-closed");

        let error = runtime
            .wait_for_registered_result("run-channel-closed", receiver)
            .await
            .expect_err("closed channel error");

        assert!(error
            .to_string()
            .contains("Gemini Browser worker channel closed unexpectedly"));
        assert!(!runtime.has_waiter_for_test("run-channel-closed"));
    }

    #[tokio::test]
    async fn register_waiter_rejects_duplicate_run_id() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let _first = runtime
            .register_waiter("run-duplicate")
            .expect("first waiter");

        let error = runtime
            .register_waiter("run-duplicate")
            .expect_err("duplicate waiter");

        assert!(error
            .to_string()
            .contains("already has an active Gemini Browser waiter"));
    }

    #[tokio::test]
    async fn complete_waiter_ignores_dropped_receiver() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let receiver = runtime
            .register_waiter("run-dropped-receiver")
            .expect("register waiter");
        drop(receiver);

        runtime.complete_waiter(
            "run-dropped-receiver",
            Ok(ok_run_result("run-dropped-receiver", "late answer")),
        );

        assert!(!runtime.has_waiter_for_test("run-dropped-receiver"));
    }

    #[test]
    fn runtime_tracks_and_clears_cancelled_run_ids() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));

        assert!(!runtime.is_cancelled("run-cancel"));
        runtime.request_cancel("run-cancel");
        assert!(runtime.is_cancelled("run-cancel"));
        runtime.clear_cancelled("run-cancel");
        assert!(!runtime.is_cancelled("run-cancel"));
    }

    #[tokio::test]
    async fn worker_handler_marks_run_running_and_terminal() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let receiver = runtime
            .register_waiter("run-worker-1")
            .expect("register waiter");
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let job = test_job("run-worker-1");

        crate::gemini_browser::create_queued_run(
            temp.path(),
            &job.run_id,
            &job.source,
            &job.prompt,
        )
        .expect("create queued run");

        let result =
            run_job_with_executor_for_test(temp.path(), &runtime, job, events.clone(), || async {
                Ok(ok_run_result("run-worker-1", "answer"))
            })
            .await
            .expect("run job");

        assert_eq!(
            result.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Ok
        );
        assert_eq!(
            receiver.await.expect("waiter open").expect("worker result"),
            result
        );

        let runs = crate::gemini_browser::list_runs(temp.path(), 10)
            .expect("list runs")
            .runs;
        assert_eq!(
            runs[0].status,
            crate::gemini_browser::GeminiBrowserRunStatus::Ok
        );
        assert_eq!(events.lock().as_slice(), ["running", "ok"]);
    }

    #[tokio::test]
    async fn worker_handler_converts_executor_error_to_terminal_failed_result() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let receiver = runtime
            .register_waiter("run-worker-error")
            .expect("register waiter");
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let job = test_job("run-worker-error");

        crate::gemini_browser::create_queued_run(
            temp.path(),
            &job.run_id,
            &job.source,
            &job.prompt,
        )
        .expect("create queued run");

        let result =
            run_job_with_executor_for_test(temp.path(), &runtime, job, events.clone(), || async {
                Err(crate::error::AppError::internal("sidecar unavailable"))
            })
            .await
            .expect("worker returns success to Apalis after terminal run result");

        assert_eq!(
            result.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Failed
        );
        assert!(result
            .message
            .as_deref()
            .unwrap_or("")
            .contains("sidecar unavailable"));
        assert_eq!(
            receiver.await.expect("waiter open").expect("worker result"),
            result
        );
        assert_eq!(events.lock().as_slice(), ["running", "failed"]);

        let runs = crate::gemini_browser::list_runs(temp.path(), 10)
            .expect("list runs")
            .runs;
        assert_eq!(
            runs[0].status,
            crate::gemini_browser::GeminiBrowserRunStatus::Failed
        );
    }

    #[tokio::test]
    async fn apalis_sqlite_status_probe_documents_actual_status_values() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");

        let mut success_storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut success_storage, test_job("run-status-ok"))
            .await
            .expect("enqueue success probe job");
        let queued = apalis_job_status_for_run_id(&pool, "run-status-ok").await;

        let (started_tx, started_rx) = oneshot::channel::<()>();
        let (release_tx, release_rx) = oneshot::channel::<()>();
        let worker = WorkerBuilder::new("gemini-browser-status-ok")
            .backend(success_storage)
            .concurrency(1)
            .data(Arc::new(Mutex::new(Some(started_tx))))
            .data(Arc::new(Mutex::new(Some(release_rx))))
            .build(blocking_success_probe_handler);
        let worker_task = tokio::spawn(worker.run());

        tokio::time::timeout(Duration::from_secs(5), started_rx)
            .await
            .expect("worker starts before timeout")
            .expect("worker start signal remains open");
        let running = apalis_job_status_for_run_id(&pool, "run-status-ok").await;

        release_tx.send(()).expect("release worker");
        worker_task
            .await
            .expect("worker task joins")
            .expect("worker run succeeds");
        let completed = apalis_job_status_for_run_id(&pool, "run-status-ok").await;

        let mut failed_storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut failed_storage, test_job("run-status-failed"))
            .await
            .expect("enqueue failed probe job");
        let worker = WorkerBuilder::new("gemini-browser-status-failed")
            .backend(failed_storage)
            .concurrency(1)
            .build(failing_probe_handler);
        tokio::time::timeout(Duration::from_secs(5), worker.run())
            .await
            .expect("failed worker stops before timeout")
            .expect("failed worker run succeeds");
        let failed = apalis_job_status_for_run_id(&pool, "run-status-failed").await;

        // apalis-sqlite 1.0.0-rc.8 stores these status strings in Jobs.status.
        // With max_attempts(1), a handler error exhausts retries and becomes Killed.
        assert_eq!(
            (
                queued.as_str(),
                running.as_str(),
                completed.as_str(),
                failed.as_str()
            ),
            ("Pending", "Running", "Done", "Killed")
        );
        assert_eq!(
            run_status_for_queue_state(&queued),
            Some(crate::gemini_browser::GeminiBrowserRunStatus::Queued)
        );
        assert_eq!(
            run_status_for_queue_state(&running),
            Some(crate::gemini_browser::GeminiBrowserRunStatus::Running)
        );
        assert_eq!(
            run_status_for_queue_state(&completed),
            Some(crate::gemini_browser::GeminiBrowserRunStatus::Ok)
        );
        assert_eq!(
            run_status_for_queue_state(&failed),
            Some(crate::gemini_browser::GeminiBrowserRunStatus::Failed)
        );
        assert_eq!(
            run_status_for_queue_state("Failed"),
            Some(crate::gemini_browser::GeminiBrowserRunStatus::Failed)
        );
    }

    #[test]
    fn gemini_browser_jobs_are_built_with_one_total_attempt() {
        let task: apalis_sqlite::SqliteTask<GeminiBrowserJob> =
            build_gemini_browser_task(test_job("run-one-attempt"));

        assert_eq!(task.parts.ctx.max_attempts(), 1);
        assert_eq!(
            task.parts.idempotency_key.as_deref(),
            Some("run-one-attempt")
        );
    }

    #[tokio::test]
    async fn failed_gemini_browser_job_is_not_retried() {
        assert_failed_gemini_browser_job_is_not_retried().await;
    }

    #[tokio::test]
    async fn failed_gemini_browser_job_retry_is_not_attempted() {
        assert_failed_gemini_browser_job_is_not_retried().await;
    }

    #[tokio::test]
    async fn cancel_gemini_browser_job_cancels_queued_run_and_waiter() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let state = crate::gemini_browser::GeminiBrowserState::new();
        let job = test_job("run-cancel-queued");
        crate::gemini_browser::create_queued_run(
            temp.path(),
            &job.run_id,
            &job.source,
            &job.prompt,
        )
        .expect("create queued run");
        let waiter = runtime
            .register_waiter(&job.run_id)
            .expect("register waiter");
        runtime.request_cancel(&job.run_id);
        let events = Arc::new(Mutex::new(Vec::<
            crate::gemini_browser::GeminiBrowserRunChangeEvent,
        >::new()));

        cancel_gemini_browser_job_core(
            &runtime,
            &state,
            temp.path(),
            &job.run_id,
            {
                let events = events.clone();
                move |run| {
                    events
                        .lock()
                        .push(crate::gemini_browser::commands::run_change_event_from_run(
                            run,
                        ))
                }
            },
            |_result| {},
            || async { Ok(()) },
        )
        .await
        .expect("cancel queued job");

        let waiter_result = waiter
            .await
            .expect("waiter channel open")
            .expect("cancelled result");
        assert_eq!(
            waiter_result.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
        );
        assert!(!runtime.is_cancelled(&job.run_id));
        assert_eq!(
            read_run_by_id(temp.path(), &job.run_id).status,
            crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
        );
        let cancelled_run = read_run_by_id(temp.path(), &job.run_id);
        assert_eq!(
            events.lock().as_slice(),
            [crate::gemini_browser::GeminiBrowserRunChangeEvent {
                run_id: job.run_id.clone(),
                run_updated_at: cancelled_run.updated_at.clone(),
            }]
        );

        let executed = Arc::new(AtomicUsize::new(0));
        let result = run_job_with_cancellation_check_for_test(
            temp.path(),
            &runtime,
            job,
            executed.clone(),
            || async { Ok(ok_run_result("run-cancel-queued", "should not run")) },
        )
        .await
        .expect("worker acknowledges queued cancellation");
        assert_eq!(
            result.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
        );
        assert_eq!(executed.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn cancel_missing_run_does_not_emit_change_event() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let state = crate::gemini_browser::GeminiBrowserState::new();
        let events = Arc::new(Mutex::new(Vec::<
            crate::gemini_browser::GeminiBrowserRunChangeEvent,
        >::new()));

        cancel_gemini_browser_job_core(
            &runtime,
            &state,
            temp.path(),
            "missing-run",
            {
                let events = events.clone();
                move |run| {
                    events
                        .lock()
                        .push(crate::gemini_browser::commands::run_change_event_from_run(
                            run,
                        ))
                }
            },
            |_result| {},
            || async { Ok(()) },
        )
        .await
        .expect("missing run is acknowledged");

        assert!(events.lock().is_empty());
    }

    #[tokio::test]
    async fn cancel_queued_run_updates_terminal_snapshot_before_change_event() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let state = crate::gemini_browser::GeminiBrowserState::new();
        let job = test_job("run-cancel-order");
        crate::gemini_browser::create_queued_run(
            temp.path(),
            &job.run_id,
            &job.source,
            &job.prompt,
        )
        .expect("create queued run");
        runtime.request_cancel(&job.run_id);

        let order = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let run_id_for_emit = job.run_id.clone();

        cancel_gemini_browser_job_core(
            &runtime,
            &state,
            temp.path(),
            &job.run_id,
            {
                let order = order.clone();
                move |run| {
                    assert_eq!(run.run_id, run_id_for_emit);
                    order.lock().push("emit");
                }
            },
            {
                let order = order.clone();
                move |result| {
                    assert_eq!(
                        result.status,
                        crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
                    );
                    order.lock().push("snapshot");
                }
            },
            || async { Ok(()) },
        )
        .await
        .expect("cancel queued job");

        assert_eq!(order.lock().as_slice(), ["snapshot", "emit"]);
    }

    #[tokio::test]
    async fn cancel_gemini_browser_job_requests_stop_for_active_run() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        let state = crate::gemini_browser::GeminiBrowserState::new();
        let job = test_job("run-cancel-active");
        crate::gemini_browser::create_queued_run(
            temp.path(),
            &job.run_id,
            &job.source,
            &job.prompt,
        )
        .expect("create queued run");
        crate::gemini_browser::mark_running(temp.path(), &job.run_id).expect("mark running");
        let token = state.start_run(job.run_id.clone()).await;
        let stopped = Arc::new(AtomicUsize::new(0));

        cancel_gemini_browser_job_core(
            &runtime,
            &state,
            temp.path(),
            &job.run_id,
            |_run| {},
            |_result| {},
            {
                let stopped = stopped.clone();
                || async move {
                    stopped.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        )
        .await
        .expect("cancel active job");

        assert!(token.is_cancelled());
        assert_eq!(stopped.load(Ordering::SeqCst), 1);
        assert!(runtime.is_cancelled(&job.run_id));
        assert_eq!(
            read_run_by_id(temp.path(), &job.run_id).status,
            crate::gemini_browser::GeminiBrowserRunStatus::Running
        );

        let result = run_job_with_executor_for_test(
            temp.path(),
            &runtime,
            job,
            Arc::new(Mutex::new(Vec::new())),
            || async {
                Ok(crate::gemini_browser::GeminiBrowserRunResult {
                    run_id: "run-cancel-active".to_string(),
                    status: crate::gemini_browser::GeminiBrowserRunStatus::Cancelled,
                    text: None,
                    message: Some("Cancelled".to_string()),
                    manual_action: None,
                    artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
                    elapsed_ms: 0,
                    debug_summary: None,
                })
            },
        )
        .await
        .expect("worker writes terminal cancelled result");
        assert_eq!(
            result.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn worker_startup_failure_marks_runtime_failed() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));

        let error = start_gemini_browser_job_worker_core(&runtime, || async {
            Err::<std::future::Pending<Result<(), BoxDynError>>, _>(
                crate::error::AppError::internal("open storage failed"),
            )
        })
        .await
        .expect_err("startup fails");

        assert!(error.to_string().contains("open storage failed"));
        assert!(matches!(
            runtime.worker_status_for_test(),
            GeminiBrowserWorkerStatus::Failed { error, .. }
                if error.contains("open storage failed")
        ));
        let readiness_error = runtime
            .ensure_worker_ready_for_enqueue()
            .await
            .expect_err("worker readiness reports startup failure");
        assert!(readiness_error.to_string().contains("open storage failed"));
    }

    #[tokio::test]
    async fn worker_run_failure_marks_runtime_failed() {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));

        let error = start_gemini_browser_job_worker_core(&runtime, || async {
            Ok(async {
                Err::<(), BoxDynError>(
                    std::io::Error::new(std::io::ErrorKind::Other, "worker loop failed").into(),
                )
            })
        })
        .await
        .expect_err("worker loop failure is returned");

        assert!(error.to_string().contains("worker loop failed"));
        assert!(matches!(
            runtime.worker_status_for_test(),
            GeminiBrowserWorkerStatus::Failed { error, .. }
                if error.contains("worker loop failed")
        ));
        let readiness_error = runtime
            .ensure_worker_ready_for_enqueue()
            .await
            .expect_err("worker readiness reports run failure");
        assert!(readiness_error.to_string().contains("worker loop failed"));
    }

    #[tokio::test]
    async fn worker_timeout_marks_run_failed_and_processes_next_job() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");

        let runs_dir = Arc::new(temp_dir.path().join("runs"));
        for run_id in ["run-timeout-first", "run-timeout-second"] {
            let job = test_job(run_id);
            crate::gemini_browser::create_queued_run(
                &runs_dir,
                &job.run_id,
                &job.source,
                &job.prompt,
            )
            .expect("create queued run");
        }

        let mut storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("run-timeout-first"))
            .await
            .expect("enqueue first job");
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("run-timeout-second"))
            .await
            .expect("enqueue second job");

        let runtime = Arc::new(GeminiBrowserJobRuntime::new_for_test_with_timeouts(
            Duration::from_millis(50),
            Duration::from_millis(25),
            Duration::from_millis(250),
        ));
        let first_receiver = runtime
            .register_waiter("run-timeout-first")
            .expect("register first waiter");
        let state = Arc::new(crate::gemini_browser::GeminiBrowserState::new());
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let stop_attempts = Arc::new(Mutex::new(Vec::<String>::new()));
        let executions = Arc::new(AtomicUsize::new(0));

        let worker = WorkerBuilder::new("gemini-browser-timeout-release")
            .backend(storage)
            .concurrency(1)
            .layer(tower::timeout::TimeoutLayer::new(
                runtime.worker_hard_guard_timeout(),
            ))
            .data(runtime.clone())
            .data(state)
            .data(runs_dir.clone())
            .data(TimeoutTestEvents(events.clone()))
            .data(TimeoutTestStopAttempts(stop_attempts.clone()))
            .data(executions)
            .build(timeout_release_test_handler);
        let worker_task = tokio::spawn(worker.run());

        wait_for_test_event(&events, "run-timeout-first:running").await;
        let started = Instant::now();
        let first_result = runtime
            .wait_for_registered_result("run-timeout-first", first_receiver)
            .await
            .expect("caller receives worker timeout result");

        assert_eq!(
            first_result.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Failed
        );
        assert!(first_result
            .message
            .as_deref()
            .unwrap_or("")
            .contains("timed out"));
        assert!(
            started.elapsed() < runtime.worker_hard_guard_timeout(),
            "run log should be terminal before hard guard deadline"
        );

        worker_task
            .await
            .expect("worker task joins")
            .expect("worker run succeeds");

        let first_run = read_run_by_id(&runs_dir, "run-timeout-first");
        assert_eq!(
            first_run.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Failed
        );
        assert!(first_run
            .result
            .as_ref()
            .and_then(|result| result.message.as_deref())
            .unwrap_or("")
            .contains("timed out"));

        let second_run = read_run_by_id(&runs_dir, "run-timeout-second");
        assert_eq!(
            second_run.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Ok
        );

        assert_eq!(
            events.lock().as_slice(),
            [
                "run-timeout-first:running",
                "run-timeout-first:failed",
                "run-timeout-first:stop",
                "run-timeout-second:running",
                "run-timeout-second:ok",
            ]
        );
        assert_eq!(stop_attempts.lock().as_slice(), ["run-timeout-first:stop"]);
    }

    #[tokio::test]
    async fn worker_timeout_clears_active_and_cancelled_state() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");

        let runs_dir = Arc::new(temp_dir.path().join("runs"));
        let job = test_job("run-timeout-first");
        crate::gemini_browser::create_queued_run(&runs_dir, &job.run_id, &job.source, &job.prompt)
            .expect("create queued run");

        let mut storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut storage, job)
            .await
            .expect("enqueue timeout job");

        let runtime = Arc::new(GeminiBrowserJobRuntime::new_for_test_with_timeouts(
            Duration::from_millis(50),
            Duration::from_millis(25),
            Duration::from_millis(250),
        ));
        let state = Arc::new(crate::gemini_browser::GeminiBrowserState::new());
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let stop_attempts = Arc::new(Mutex::new(Vec::<String>::new()));
        let executions = Arc::new(AtomicUsize::new(0));

        let worker = WorkerBuilder::new("gemini-browser-timeout-cleanup")
            .backend(storage)
            .concurrency(1)
            .layer(tower::timeout::TimeoutLayer::new(
                runtime.worker_hard_guard_timeout(),
            ))
            .data(runtime.clone())
            .data(state.clone())
            .data(runs_dir.clone())
            .data(TimeoutTestEvents(events.clone()))
            .data(TimeoutTestStopAttempts(stop_attempts.clone()))
            .data(executions)
            .build(timeout_cleanup_test_handler);
        let worker_task = tokio::spawn(worker.run());

        wait_for_test_event(&events, "run-timeout-first:running").await;
        runtime.request_cancel("run-timeout-first");

        tokio::time::timeout(Duration::from_secs(5), worker_task)
            .await
            .expect("worker stops before timeout")
            .expect("worker task joins")
            .expect("worker run succeeds");

        assert_eq!(state.active_run_id().await, None);
        assert!(!runtime.is_cancelled("run-timeout-first"));
        assert_eq!(stop_attempts.lock().as_slice(), ["run-timeout-first:stop"]);
    }

    async fn assert_failed_gemini_browser_job_is_not_retried() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = sqlite_file_pool(&db_path).await;

        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply app migrations");
        setup_gemini_browser_apalis_storage(&pool)
            .await
            .expect("setup Apalis SQLite storage");

        let mut storage = SqliteStorage::new_in_queue(&pool, GEMINI_BROWSER_QUEUE_NAME);
        enqueue_gemini_browser_job_to_storage(&mut storage, test_job("run-no-retry"))
            .await
            .expect("enqueue no-retry job");

        let executions = Arc::new(AtomicUsize::new(0));
        let worker = WorkerBuilder::new("gemini-browser-no-retry")
            .backend(storage)
            .concurrency(1)
            .data(executions.clone())
            .build(failing_no_retry_probe_handler);
        tokio::time::timeout(Duration::from_secs(5), worker.run())
            .await
            .expect("worker stops before timeout")
            .expect("worker run succeeds");

        assert_eq!(executions.load(Ordering::SeqCst), 1);

        let row = sqlx::query_as::<_, (String, i64, i64)>(
            "SELECT status, attempts, max_attempts
             FROM Jobs
             WHERE job_type = ? AND idempotency_key = ?",
        )
        .bind(GEMINI_BROWSER_QUEUE_NAME)
        .bind("run-no-retry")
        .fetch_one(&pool)
        .await
        .expect("read no-retry job row");

        assert!(
            matches!(row.0.as_str(), "Failed" | "Killed"),
            "expected terminal failed/killed Apalis status, got {}",
            row.0
        );
        assert_eq!(row.1, 1);
        assert_eq!(row.2, 1);
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

    async fn restart_recovery_test_handler(
        job: GeminiBrowserJob,
        runtime: Data<Arc<GeminiBrowserJobRuntime>>,
        state: Data<Arc<crate::gemini_browser::GeminiBrowserState>>,
        runs_dir: Data<Arc<PathBuf>>,
        executions: Data<Arc<AtomicUsize>>,
        worker: WorkerContext,
    ) -> Result<(), BoxDynError> {
        run_reconciled_job_for_test(
            runs_dir.as_ref(),
            runtime.as_ref(),
            state.as_ref(),
            job.clone(),
            (*executions).clone(),
            || async { Ok(ok_run_result(&job.run_id, "answer after restart")) },
        )
        .await?;
        worker.stop()?;
        Ok(())
    }

    async fn blocking_success_probe_handler(
        _job: GeminiBrowserJob,
        started: Data<Arc<Mutex<Option<oneshot::Sender<()>>>>>,
        release: Data<Arc<Mutex<Option<oneshot::Receiver<()>>>>>,
        worker: WorkerContext,
    ) -> Result<(), BoxDynError> {
        if let Some(sender) = started.lock().take() {
            let _ = sender.send(());
        }
        let receiver = release.lock().take().expect("release receiver available");
        let _ = receiver.await;
        worker.stop()?;
        Ok(())
    }

    async fn failing_probe_handler(
        _job: GeminiBrowserJob,
        worker: WorkerContext,
    ) -> Result<(), BoxDynError> {
        worker.stop()?;
        Err(std::io::Error::new(std::io::ErrorKind::Other, "probe failure").into())
    }

    async fn failing_no_retry_probe_handler(
        _job: GeminiBrowserJob,
        executions: Data<Arc<AtomicUsize>>,
        worker: WorkerContext,
    ) -> Result<(), BoxDynError> {
        executions.fetch_add(1, Ordering::SeqCst);
        worker.stop()?;
        Err(std::io::Error::new(std::io::ErrorKind::Other, "probe failure").into())
    }

    async fn timeout_release_test_handler(
        job: GeminiBrowserJob,
        runtime: Data<Arc<GeminiBrowserJobRuntime>>,
        state: Data<Arc<crate::gemini_browser::GeminiBrowserState>>,
        runs_dir: Data<Arc<PathBuf>>,
        events: Data<TimeoutTestEvents>,
        stop_attempts: Data<TimeoutTestStopAttempts>,
        executions: Data<Arc<AtomicUsize>>,
        worker: WorkerContext,
    ) -> Result<(), BoxDynError> {
        let result = if job.run_id == "run-timeout-first" {
            run_test_job_with_execution_timeout(
                runtime.as_ref(),
                state.as_ref(),
                runs_dir.as_ref(),
                job,
                events.0.as_ref(),
                stop_attempts.0.as_ref(),
                async {
                    std::future::pending::<
                        crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
                    >()
                    .await
                },
            )
            .await?
        } else {
            let result = ok_run_result(&job.run_id, "second answer");
            run_test_job_with_execution_timeout(
                runtime.as_ref(),
                state.as_ref(),
                runs_dir.as_ref(),
                job,
                events.0.as_ref(),
                stop_attempts.0.as_ref(),
                async { Ok(result) },
            )
            .await?
        };

        if result.run_id == "run-timeout-second" {
            worker.stop()?;
        }
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn timeout_cleanup_test_handler(
        job: GeminiBrowserJob,
        runtime: Data<Arc<GeminiBrowserJobRuntime>>,
        state: Data<Arc<crate::gemini_browser::GeminiBrowserState>>,
        runs_dir: Data<Arc<PathBuf>>,
        events: Data<TimeoutTestEvents>,
        stop_attempts: Data<TimeoutTestStopAttempts>,
        executions: Data<Arc<AtomicUsize>>,
        worker: WorkerContext,
    ) -> Result<(), BoxDynError> {
        run_test_job_with_execution_timeout(
            runtime.as_ref(),
            state.as_ref(),
            runs_dir.as_ref(),
            job,
            events.0.as_ref(),
            stop_attempts.0.as_ref(),
            async {
                std::future::pending::<
                    crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
                >()
                .await
            },
        )
        .await?;

        executions.fetch_add(1, Ordering::SeqCst);
        worker.stop()?;
        Ok(())
    }

    #[derive(Clone)]
    struct TimeoutTestEvents(Arc<Mutex<Vec<String>>>);

    #[derive(Clone)]
    struct TimeoutTestStopAttempts(Arc<Mutex<Vec<String>>>);

    #[derive(Debug, PartialEq, Eq)]
    enum TestWorkerEntryDecision {
        Executed,
        Acknowledged,
        Terminal(crate::gemini_browser::GeminiBrowserRunStatus),
    }

    async fn run_reconciled_job_for_test<F, Fut>(
        runs_dir: &Path,
        runtime: &GeminiBrowserJobRuntime,
        state: &crate::gemini_browser::GeminiBrowserState,
        job: GeminiBrowserJob,
        executions: Arc<AtomicUsize>,
        executor: F,
    ) -> crate::error::AppResult<TestWorkerEntryDecision>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<
            Output = crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
        >,
    {
        match reconcile_gemini_browser_worker_entry(runtime, state, runs_dir, &job).await? {
            GeminiBrowserWorkerEntryDecision::Execute(_) => {
                executions.fetch_add(1, Ordering::SeqCst);
                let result = executor().await?;
                crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
                state.finish_run(&job.run_id).await;
                runtime.complete_waiter(&job.run_id, Ok(result));
                runtime.clear_cancelled(&job.run_id);
                Ok(TestWorkerEntryDecision::Executed)
            }
            GeminiBrowserWorkerEntryDecision::Acknowledged => {
                Ok(TestWorkerEntryDecision::Acknowledged)
            }
            GeminiBrowserWorkerEntryDecision::Terminal { result, .. } => {
                Ok(TestWorkerEntryDecision::Terminal(result.status))
            }
        }
    }

    async fn run_test_job_with_execution_timeout<Fut>(
        runtime: &GeminiBrowserJobRuntime,
        state: &crate::gemini_browser::GeminiBrowserState,
        runs_dir: &Path,
        job: GeminiBrowserJob,
        events: &Mutex<Vec<String>>,
        stop_attempts: &Mutex<Vec<String>>,
        sidecar_future: Fut,
    ) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
    where
        Fut: std::future::Future<
            Output = crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
        >,
    {
        crate::gemini_browser::mark_running(runs_dir, &job.run_id)?;
        let _token = state.start_run(job.run_id.clone()).await;
        events.lock().push(format!("{}:running", job.run_id));

        let result = match tokio::time::timeout(runtime.worker_execution_timeout(), sidecar_future)
            .await
        {
            Ok(result) => result?,
            Err(_elapsed) => {
                let result = timeout_failed_run_result(&job, runtime.worker_execution_timeout());
                crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
                runtime.complete_waiter(&job.run_id, Ok(result.clone()));
                events.lock().push(format!("{}:failed", job.run_id));
                state.finish_run(&job.run_id).await;
                runtime.clear_cancelled(&job.run_id);
                stop_attempts.lock().push(format!("{}:stop", job.run_id));
                events.lock().push(format!("{}:stop", job.run_id));
                return Ok(result);
            }
        };

        crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
        state.finish_run(&job.run_id).await;
        runtime.complete_waiter(&job.run_id, Ok(result.clone()));
        events.lock().push(format!(
            "{}:{}",
            job.run_id,
            format!("{:?}", result.status).to_lowercase()
        ));
        runtime.clear_cancelled(&job.run_id);
        Ok(result)
    }

    async fn run_job_with_executor_for_test<F, Fut>(
        runs_dir: &Path,
        runtime: &GeminiBrowserJobRuntime,
        job: GeminiBrowserJob,
        events: Arc<Mutex<Vec<String>>>,
        executor: F,
    ) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<
            Output = crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
        >,
    {
        crate::gemini_browser::mark_running(runs_dir, &job.run_id)?;
        events.lock().push("running".to_string());

        let result = match executor().await {
            Ok(result) => result,
            Err(error) => crate::gemini_browser::GeminiBrowserRunResult {
                run_id: job.run_id.clone(),
                status: crate::gemini_browser::GeminiBrowserRunStatus::Failed,
                text: None,
                message: Some(error.to_string()),
                manual_action: None,
                artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
                elapsed_ms: 0,
                debug_summary: None,
            },
        };
        crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
        events
            .lock()
            .push(format!("{:?}", result.status).to_lowercase());
        runtime.complete_waiter(&job.run_id, Ok(result.clone()));
        Ok(result)
    }

    async fn run_job_with_cancellation_check_for_test<F, Fut>(
        runs_dir: &Path,
        runtime: &GeminiBrowserJobRuntime,
        job: GeminiBrowserJob,
        executions: Arc<AtomicUsize>,
        executor: F,
    ) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<
            Output = crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
        >,
    {
        if runtime.is_cancelled(&job.run_id) || run_log_is_cancelled(runs_dir, &job.run_id)? {
            let result = cancelled_run_result(&job);
            crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
            runtime.complete_waiter(&job.run_id, Ok(result.clone()));
            runtime.clear_cancelled(&job.run_id);
            return Ok(result);
        }

        executions.fetch_add(1, Ordering::SeqCst);
        run_job_with_executor_for_test(
            runs_dir,
            runtime,
            job,
            Arc::new(Mutex::new(Vec::new())),
            executor,
        )
        .await
    }

    async fn sqlite_file_pool(db_path: &Path) -> SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("connect sqlite file pool")
    }

    fn test_job(run_id: &str) -> GeminiBrowserJob {
        GeminiBrowserJob {
            run_id: run_id.to_string(),
            prompt: "hello".to_string(),
            source: "settings_test".to_string(),
            artifact_mode: GeminiBrowserArtifactMode::Reduced,
            browser_config: None,
        }
    }

    fn ok_run_result(run_id: &str, text: &str) -> crate::gemini_browser::GeminiBrowserRunResult {
        crate::gemini_browser::GeminiBrowserRunResult {
            run_id: run_id.to_string(),
            status: crate::gemini_browser::GeminiBrowserRunStatus::Ok,
            text: Some(text.to_string()),
            message: Some("done".to_string()),
            manual_action: None,
            artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 10,
            debug_summary: None,
        }
    }

    fn read_run_by_id(runs_dir: &Path, run_id: &str) -> crate::gemini_browser::GeminiBrowserRun {
        crate::gemini_browser::list_runs(runs_dir, 20)
            .expect("list runs")
            .runs
            .into_iter()
            .find(|run| run.run_id == run_id)
            .unwrap_or_else(|| panic!("missing run log for {run_id}"))
    }

    fn assert_run_status_message(
        runs_dir: &Path,
        run_id: &str,
        expected_status: crate::gemini_browser::GeminiBrowserRunStatus,
        expected_message: &str,
    ) {
        let run = read_run_by_id(runs_dir, run_id);
        assert_eq!(run.status, expected_status);
        assert_eq!(
            run.result
                .as_ref()
                .and_then(|result| result.message.as_deref()),
            Some(expected_message)
        );
    }

    async fn wait_for_test_event(events: &Mutex<Vec<String>>, expected: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if events.lock().iter().any(|event| event == expected) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        panic!("timed out waiting for test event {expected}");
    }

    async fn table_sql_by_name(pool: &SqlitePool) -> BTreeMap<String, String> {
        sqlx::query_as::<_, (String, String)>(
            "SELECT name, sql FROM sqlite_master WHERE type = 'table' ORDER BY name",
        )
        .fetch_all(pool)
        .await
        .expect("read sqlite table schema")
        .into_iter()
        .collect()
    }

    async fn apalis_job_status_for_run_id(pool: &SqlitePool, run_id: &str) -> String {
        sqlx::query_scalar::<_, String>(
            "SELECT status
             FROM Jobs
             WHERE job_type = ? AND idempotency_key = ?",
        )
        .bind(GEMINI_BROWSER_QUEUE_NAME)
        .bind(run_id)
        .fetch_one(pool)
        .await
        .expect("read Apalis job status")
    }
}
