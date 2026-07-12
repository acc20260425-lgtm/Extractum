use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

use crate::error::{AppError, AppResult};
use crate::external_process::ExternalProcessShutdownState;

use super::jobs::{
    cancel_gemini_browser_job, enqueue_gemini_browser_job, GeminiBrowserArtifactMode,
    GeminiBrowserJob, GeminiBrowserJobRuntime, GeminiBrowserWaiterReceiver, QueuedGeminiBrowserJob,
};
use super::{
    cdp_chrome, chrome_cdp_profile_dir, create_queued_run, finish_run, list_runs, path_string,
    profile_dir, read_run, recorded_run_dir, runs_dir, sidecar, GeminiBrowserProviderConfig,
    GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind, GeminiBrowserRun,
    GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserRunStatus, GeminiBrowserStartChromeResult, GeminiBrowserState,
};

#[tauri::command]
pub async fn gemini_bridge_status(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
    provider_status(&handle, &state, browser_config).await
}

#[tauri::command]
pub async fn gemini_bridge_status_snapshot(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<GeminiBrowserProviderStatus> {
    super::jobs::ensure_gemini_browser_startup_reconciled(&handle).await?;
    let active_run_id = state.active_run_id().await;
    provider_status_snapshot_read_core(
        &runs_dir(&handle)?,
        || state.status_snapshot(&handle),
        active_run_id,
        |expected, snapshot| Ok(state.set_status_snapshot_if_current(expected, snapshot)),
    )
}

pub(crate) async fn provider_status(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
    let browser_profile_dir = path_string(&profile_dir(handle)?);
    provider_status_read_core(
        || super::jobs::ensure_gemini_browser_startup_reconciled(handle),
        || state.active_run_id(),
        |active_run_id| {
            sidecar::status(
                handle,
                state,
                browser_profile_dir,
                browser_config,
                active_run_id,
                0,
            )
        },
        std::time::Duration::from_millis(250),
        || state.status_snapshot(handle),
    )
    .await
}

#[derive(Debug)]
struct SendSinglePromptEnqueueHandoff {
    waiter: GeminiBrowserWaiterReceiver,
}

#[derive(Debug)]
enum SendSinglePromptEnqueueError {
    App(AppError),
    EnqueueFailed {
        run_id: String,
        source: AppError,
        failed_result: GeminiBrowserRunResult,
    },
}

impl From<AppError> for SendSinglePromptEnqueueError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}

async fn send_single_prompt_enqueue_core<Enqueue, EnqueueFut>(
    runs_root: &std::path::Path,
    runtime: &GeminiBrowserJobRuntime,
    request: GeminiBrowserRunRequest,
    browser_config: Option<GeminiBrowserProviderConfig>,
    enqueue: Enqueue,
) -> Result<SendSinglePromptEnqueueHandoff, SendSinglePromptEnqueueError>
where
    Enqueue: FnOnce(GeminiBrowserJob) -> EnqueueFut,
    EnqueueFut: std::future::Future<Output = AppResult<QueuedGeminiBrowserJob>>,
{
    let artifact_mode = GeminiBrowserArtifactMode::from_wire(Some(&request.artifact_mode))?;

    runtime.ensure_worker_ready_for_enqueue().await?;
    reject_duplicate_existing_run_or_waiter(runtime, runs_root, &request.run_id).await?;
    let queued_run =
        create_queued_run(runs_root, &request.run_id, &request.source, &request.prompt)?;
    let waiter = runtime.register_waiter(&request.run_id)?;

    match enqueue(GeminiBrowserJob {
        run_id: request.run_id.clone(),
        prompt: request.prompt.clone(),
        source: request.source.clone(),
        artifact_mode,
        browser_config,
    })
    .await
    {
        Ok(_queued) => {}
        Err(error) => {
            runtime.remove_waiter(&request.run_id);
            let failed = GeminiBrowserRunResult {
                run_id: request.run_id.clone(),
                status: GeminiBrowserRunStatus::Failed,
                text: None,
                message: Some(format!("Gemini Browser job enqueue failed: {error}")),
                manual_action: None,
                artifacts: super::GeminiBrowserArtifactRefs::default(),
                elapsed_ms: 0,
                debug_summary: None,
            };
            let _failed_run = finish_run(runs_root, &request.run_id, failed.clone())?;
            return Err(SendSinglePromptEnqueueError::EnqueueFailed {
                run_id: request.run_id.clone(),
                source: error,
                failed_result: failed,
            });
        }
    };

    let _queued_run = queued_run;

    Ok(SendSinglePromptEnqueueHandoff { waiter })
}

async fn reject_duplicate_existing_run_or_waiter(
    runtime: &GeminiBrowserJobRuntime,
    runs_root: &std::path::Path,
    run_id: &str,
) -> AppResult<()> {
    if runtime.has_waiter(run_id) {
        return Err(AppError::conflict(format!(
            "Gemini Browser run_id '{run_id}' already has an active waiter"
        )));
    }

    if run_log_has_any_run(runs_root, run_id).await? {
        return Err(AppError::conflict(format!(
            "Gemini Browser run_id '{run_id}' already exists"
        )));
    }

    Ok(())
}

async fn run_log_has_any_run(runs_root: &std::path::Path, run_id: &str) -> AppResult<bool> {
    Ok(list_runs(runs_root, usize::MAX)?
        .runs
        .into_iter()
        .any(|run| run.run_id == run_id))
}

async fn provider_status_core<Fut>(
    live_status: Fut,
    timeout: std::time::Duration,
    fallback_status: impl FnOnce() -> AppResult<GeminiBrowserProviderStatus>,
) -> AppResult<GeminiBrowserProviderStatus>
where
    Fut: std::future::Future<Output = AppResult<GeminiBrowserProviderStatus>>,
{
    match tokio::time::timeout(timeout, live_status).await {
        Ok(Ok(status)) => Ok(status),
        Ok(Err(_)) | Err(_) => fallback_status(),
    }
}

const STATUS_SNAPSHOT_RUN_SCAN_LIMIT: usize = 200;
const STATUS_SNAPSHOT_ACTIVITY_GRACE_MINUTES: i64 = 30;

async fn provider_status_read_core<Gate, GateFut, Active, ActiveFut, Live, LiveFut>(
    ensure_reconciled: Gate,
    active_run_id: Active,
    live_status: Live,
    timeout: std::time::Duration,
    fallback_status: impl FnOnce() -> AppResult<GeminiBrowserProviderStatus>,
) -> AppResult<GeminiBrowserProviderStatus>
where
    Gate: FnOnce() -> GateFut,
    GateFut: std::future::Future<Output = AppResult<()>>,
    Active: FnOnce() -> ActiveFut,
    ActiveFut: std::future::Future<Output = Option<String>>,
    Live: FnOnce(Option<String>) -> LiveFut,
    LiveFut: std::future::Future<Output = AppResult<GeminiBrowserProviderStatus>>,
{
    ensure_reconciled().await?;
    let active_run_id = active_run_id().await;
    provider_status_core(live_status(active_run_id), timeout, fallback_status).await
}

fn provider_status_snapshot_core(
    fallback_status: impl FnOnce() -> AppResult<GeminiBrowserProviderStatus>,
) -> AppResult<GeminiBrowserProviderStatus> {
    fallback_status()
}

fn provider_status_snapshot_read_core(
    runs_root: &std::path::Path,
    mut read_snapshot: impl FnMut() -> AppResult<GeminiBrowserProviderStatus>,
    active_run_id: Option<String>,
    mut write_snapshot_if_current: impl FnMut(
        &GeminiBrowserProviderStatus,
        GeminiBrowserProviderStatus,
    ) -> AppResult<bool>,
) -> AppResult<GeminiBrowserProviderStatus> {
    let snapshot = provider_status_snapshot_core(|| read_snapshot())?;
    let reconciled =
        status_snapshot_from_reconciled_run_log(runs_root, snapshot.clone(), active_run_id)?;
    if write_snapshot_if_current(&snapshot, reconciled.clone())? {
        Ok(reconciled)
    } else {
        read_snapshot()
    }
}

fn status_snapshot_from_reconciled_run_log(
    runs_root: &std::path::Path,
    snapshot: GeminiBrowserProviderStatus,
    active_run_id: Option<String>,
) -> AppResult<GeminiBrowserProviderStatus> {
    status_snapshot_from_reconciled_run_log_at(
        runs_root,
        snapshot,
        active_run_id,
        OffsetDateTime::now_utc(),
    )
}

fn status_snapshot_from_reconciled_run_log_at(
    runs_root: &std::path::Path,
    mut snapshot: GeminiBrowserProviderStatus,
    active_run_id: Option<String>,
    now: OffsetDateTime,
) -> AppResult<GeminiBrowserProviderStatus> {
    let runs = list_runs(runs_root, STATUS_SNAPSHOT_RUN_SCAN_LIMIT)?.runs;
    let fresh_queued_count = runs
        .iter()
        .filter(|run| {
            run.status == GeminiBrowserRunStatus::Queued && run_log_activity_is_fresh(run, now)
        })
        .count();
    let stale_queued_count = runs
        .iter()
        .filter(|run| {
            run.status == GeminiBrowserRunStatus::Queued && !run_log_activity_is_fresh(run, now)
        })
        .count();

    if let Some(active_run_id) = active_run_id {
        snapshot.status = GeminiBrowserProviderStatusKind::Running;
        snapshot.active_run_id = Some(active_run_id);
        snapshot.queue_depth = fresh_queued_count;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some("Running".to_string());
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }

    if fresh_queued_count > 0 {
        snapshot.status = GeminiBrowserProviderStatusKind::Running;
        snapshot.active_run_id = None;
        snapshot.queue_depth = fresh_queued_count;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some("Queued".to_string());
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }

    snapshot.active_run_id = None;
    snapshot.queue_depth = 0;
    if stale_queued_count > 0 && snapshot.status == GeminiBrowserProviderStatusKind::Running {
        snapshot.status = GeminiBrowserProviderStatusKind::NotStarted;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some(
                "Gemini browser has stale queued run-log entries; waiting for cleanup.".to_string(),
            );
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }

    if snapshot.status == GeminiBrowserProviderStatusKind::Running {
        if let Some(latest) = runs.first().and_then(|run| run.result.as_ref()) {
            snapshot.status =
                GeminiBrowserState::provider_status_kind_for_run_status(&latest.status);
            snapshot.latest_message = latest.message.clone();
            snapshot.manual_action = latest.manual_action.clone();
        } else {
            snapshot.status = GeminiBrowserProviderStatusKind::NotStarted;
            snapshot.latest_message = Some("Gemini browser sidecar is not running.".to_string());
            snapshot.manual_action = None;
        }
    }
    Ok(snapshot)
}

fn run_log_activity_is_fresh(run: &GeminiBrowserRun, now: OffsetDateTime) -> bool {
    let Ok(updated_at) = OffsetDateTime::parse(&run.updated_at, &Rfc3339) else {
        return false;
    };
    now - updated_at <= Duration::minutes(STATUS_SNAPSHOT_ACTIVITY_GRACE_MINUTES)
}

fn get_run_core(runs_root: &std::path::Path, run_id: &str) -> AppResult<GeminiBrowserRun> {
    read_run(runs_root, run_id)
}

#[tauri::command]
pub async fn gemini_bridge_open_browser(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
    sidecar::open_browser(
        &handle,
        &state,
        path_string(&profile_dir(&handle)?),
        browser_config,
    )
    .await
}

#[tauri::command]
pub async fn gemini_bridge_start_cdp_chrome(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserStartChromeResult> {
    let spec = cdp_chrome::build_chrome_cdp_launch_spec(
        cdp_chrome::find_chrome_executable(),
        chrome_cdp_profile_dir(&handle)?,
        browser_config.as_ref(),
    )?;
    let shutdown = handle
        .state::<ExternalProcessShutdownState>()
        .inner()
        .clone();
    let permit = shutdown
        .try_admit()
        .map_err(|_| AppError::internal("Application is shutting down"))?;
    let spawn_spec = spec.clone();
    let process = tokio::task::spawn_blocking(move || cdp_chrome::spawn_chrome_cdp(&spawn_spec))
        .await
        .map_err(|_| AppError::internal("Chrome launch task did not complete"))??;
    {
        let mut cdp_process = state.cdp_chrome_process().await;
        *cdp_process = Some(process);
    }
    drop(permit);

    if let Err(error) = cdp_chrome::wait_for_cdp_endpoint(&spec.cdp_endpoint).await {
        let process = state.cdp_chrome_process().await.take();
        if let Some(mut process) = process {
            let _ = tokio::task::spawn_blocking(move || process.shutdown()).await;
        }
        return Err(error);
    }
    Ok(cdp_chrome::start_chrome_result(&spec))
}

#[tauri::command]
pub(crate) async fn send_single_prompt(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    run_id: String,
    prompt: String,
    source: Option<String>,
    artifact_mode: Option<String>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserRunResult> {
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::validation("prompt cannot be empty"));
    }
    let request = GeminiBrowserRunRequest {
        run_id,
        prompt,
        source: source.unwrap_or_else(|| "settings_test".to_string()),
        artifact_mode: artifact_mode.unwrap_or_else(|| "reduced".to_string()),
    };

    let runs_root = runs_dir(handle)?;
    let runtime = handle.state::<GeminiBrowserJobRuntime>();
    let handoff = send_single_prompt_enqueue_core(
        &runs_root,
        &runtime,
        request.clone(),
        browser_config.clone(),
        |job| enqueue_gemini_browser_job(handle, job),
    )
    .await;
    let handoff = match handoff {
        Ok(handoff) => handoff,
        Err(SendSinglePromptEnqueueError::EnqueueFailed {
            run_id,
            source,
            failed_result,
        }) => {
            debug_assert_eq!(failed_result.run_id, run_id);
            if let Err(error) = state.update_status_snapshot(handle, |status| {
                status.status =
                    GeminiBrowserState::provider_status_kind_for_run_status(&failed_result.status);
                status.active_run_id = None;
                status.queue_depth = 0;
                status.latest_message = failed_result.message.clone();
                status.manual_action = failed_result.manual_action.clone();
            }) {
                eprintln!("Gemini Browser enqueue failure status snapshot update failed: {error}");
            }
            return Err(source);
        }
        Err(SendSinglePromptEnqueueError::App(error)) => return Err(error),
    };

    if let Err(error) = state.update_status_snapshot(handle, |status| {
        status.status = GeminiBrowserProviderStatusKind::Running;
        status.active_run_id = None;
        status.queue_depth = 1;
        status.latest_message = Some("Queued".to_string());
        status.manual_action = None;
    }) {
        eprintln!("Gemini Browser queued status snapshot update failed: {error}");
    }

    runtime
        .wait_for_registered_result(&request.run_id, handoff.waiter)
        .await
}

#[tauri::command]
pub async fn gemini_bridge_send_single(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    run_id: String,
    prompt: String,
    source: Option<String>,
    artifact_mode: Option<String>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserRunResult> {
    send_single_prompt(
        &handle,
        &state,
        run_id,
        prompt,
        source,
        artifact_mode,
        browser_config,
    )
    .await
}

#[tauri::command]
pub async fn gemini_bridge_resume(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
    sidecar::resume(
        &handle,
        &state,
        path_string(&profile_dir(&handle)?),
        browser_config,
    )
    .await
}

#[tauri::command]
pub async fn gemini_bridge_stop(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<()> {
    if let Some(run_id) = state.active_run_id().await {
        return cancel_gemini_browser_job(&handle, &run_id).await;
    }
    state.request_stop().await;
    let process = state.cdp_chrome_process().await.take();
    if let Some(mut process) = process {
        tokio::task::spawn_blocking(move || process.shutdown())
            .await
            .map_err(|_| AppError::internal("Chrome shutdown task did not complete"))?
            .map_err(|error| AppError::internal(format!("Failed to stop Chrome: {error}")))?;
    }
    sidecar::stop(&handle, &state).await
}

#[tauri::command]
pub async fn gemini_bridge_list_runs(
    handle: AppHandle,
    limit: Option<usize>,
) -> AppResult<GeminiBrowserRunLogSummary> {
    super::jobs::ensure_gemini_browser_startup_reconciled(&handle).await?;
    list_runs(&runs_dir(&handle)?, limit.unwrap_or(20))
}

#[tauri::command]
pub async fn gemini_bridge_get_run(
    handle: AppHandle,
    run_id: String,
) -> AppResult<GeminiBrowserRun> {
    super::jobs::ensure_gemini_browser_startup_reconciled(&handle).await?;
    get_run_core(&runs_dir(&handle)?, &run_id)
}

#[tauri::command]
pub async fn gemini_bridge_open_run_folder(handle: AppHandle, run_id: String) -> AppResult<()> {
    let dir = recorded_run_dir(&runs_dir(&handle)?, &run_id)?;
    handle
        .opener()
        .open_path(path_string(&dir), None::<&str>)
        .map_err(|error| {
            AppError::internal(format!("Failed to open Gemini browser run folder: {error}"))
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        sync::Arc,
        time::{Duration, Instant},
    };

    use parking_lot::Mutex;

    use super::super::jobs::{
        GeminiBrowserArtifactMode, GeminiBrowserJob, GeminiBrowserJobRuntime,
        QueuedGeminiBrowserJob,
    };
    use super::super::run_log::read_run;
    use super::super::{mark_running, GeminiBrowserProviderStatusKind};
    use super::*;

    #[tokio::test]
    async fn provider_status_uses_cached_snapshot_when_sidecar_is_busy() {
        let state = GeminiBrowserState::new();
        state.set_status_snapshot(GeminiBrowserProviderStatus {
            status: super::super::GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-busy".to_string()),
            queue_depth: 1,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("Running".to_string()),
        });

        let started = Instant::now();
        let status = provider_status_core(
            async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(GeminiBrowserProviderStatus {
                    status: super::super::GeminiBrowserProviderStatusKind::Ready,
                    manual_action: None,
                    active_run_id: None,
                    queue_depth: 0,
                    browser_profile_dir: "profile-dir".to_string(),
                    latest_message: Some("Ready".to_string()),
                })
            },
            Duration::from_millis(25),
            || {
                state
                    .status_snapshot_for_test()
                    .ok_or_else(|| AppError::internal("expected cached Gemini Browser status"))
            },
        )
        .await
        .expect("cached status");

        assert!(started.elapsed() < Duration::from_millis(200));
        assert_eq!(
            status.status,
            super::super::GeminiBrowserProviderStatusKind::Running
        );
        assert_eq!(status.active_run_id.as_deref(), Some("run-busy"));
    }

    #[tokio::test]
    async fn provider_status_live_probe_does_not_mutate_cached_snapshot() {
        let state = GeminiBrowserState::new();
        state.set_status_snapshot(GeminiBrowserProviderStatus {
            status: super::super::GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-cached".to_string()),
            queue_depth: 1,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("Cached running".to_string()),
        });

        let returned = provider_status_core(
            async {
                Ok(GeminiBrowserProviderStatus {
                    status: super::super::GeminiBrowserProviderStatusKind::Ready,
                    manual_action: None,
                    active_run_id: None,
                    queue_depth: 0,
                    browser_profile_dir: "profile-dir".to_string(),
                    latest_message: Some("Live ready".to_string()),
                })
            },
            Duration::from_millis(25),
            || {
                state
                    .status_snapshot_for_test()
                    .ok_or_else(|| AppError::internal("expected cached Gemini Browser status"))
            },
        )
        .await
        .expect("live status returned");

        assert_eq!(
            returned.status,
            super::super::GeminiBrowserProviderStatusKind::Ready
        );
        assert_eq!(returned.latest_message.as_deref(), Some("Live ready"));

        let cached = state
            .status_snapshot_for_test()
            .expect("cached status remains present");
        assert_eq!(
            cached.status,
            super::super::GeminiBrowserProviderStatusKind::Running
        );
        assert_eq!(cached.active_run_id.as_deref(), Some("run-cached"));
        assert_eq!(cached.latest_message.as_deref(), Some("Cached running"));
    }

    #[test]
    fn status_snapshot_core_returns_cached_status_without_polling_live_sidecar() {
        let cached = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-cached".to_string()),
            queue_depth: 1,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("Cached".to_string()),
        };

        let returned =
            provider_status_snapshot_core(|| Ok(cached.clone())).expect("snapshot succeeds");

        assert_eq!(returned, cached);
    }

    #[test]
    fn provider_status_snapshot_from_reconciled_runs_does_not_keep_stale_running_snapshot() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let runs_root = temp.path();
        create_queued_run(runs_root, "run-stale", "settings_test", "hello").expect("create queued");
        mark_running(runs_root, "run-stale").expect("mark running");

        let stale = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-stale".to_string()),
            queue_depth: 1,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("stale running".to_string()),
        };

        let reconciled = status_snapshot_from_reconciled_run_log(runs_root, stale, None)
            .expect("derive reconciled snapshot");

        assert_eq!(
            reconciled.status,
            GeminiBrowserProviderStatusKind::NotStarted
        );
        assert_eq!(reconciled.active_run_id, None);
        assert_eq!(reconciled.queue_depth, 0);
    }

    #[test]
    fn provider_status_snapshot_from_reconciled_runs_preserves_live_active_run() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let runs_root = temp.path();
        create_queued_run(runs_root, "run-live", "settings_test", "hello").expect("create queued");
        mark_running(runs_root, "run-live").expect("mark running");

        let stale = GeminiBrowserProviderStatus {
            latest_message: Some("Worker is submitting prompt".to_string()),
            ..GeminiBrowserState::not_started_status("profile-dir".to_string())
        };

        let reconciled =
            status_snapshot_from_reconciled_run_log(runs_root, stale, Some("run-live".to_string()))
                .expect("derive reconciled snapshot");

        assert_eq!(reconciled.status, GeminiBrowserProviderStatusKind::Running);
        assert_eq!(reconciled.active_run_id.as_deref(), Some("run-live"));
        assert_eq!(
            reconciled.latest_message.as_deref(),
            Some("Worker is submitting prompt")
        );
    }

    #[test]
    fn provider_status_snapshot_from_reconciled_runs_ignores_stale_queued_rows() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let runs_root = temp.path();
        create_queued_run(runs_root, "run-stale-queued", "settings_test", "hello")
            .expect("create queued");
        let mut stale_run = read_run(runs_root, "run-stale-queued").expect("read run");
        stale_run.updated_at = "2026-06-22T00:00:00Z".to_string();
        std::fs::write(
            runs_root.join("run-stale-queued").join("result.json"),
            serde_json::to_string_pretty(&stale_run).expect("serialize run"),
        )
        .expect("write stale run");

        let reconciled = status_snapshot_from_reconciled_run_log_at(
            runs_root,
            GeminiBrowserState::not_started_status("profile-dir".to_string()),
            None,
            time::OffsetDateTime::parse(
                "2026-06-22T00:31:00Z",
                &time::format_description::well_known::Rfc3339,
            )
            .expect("parse time"),
        )
        .expect("derive reconciled snapshot");

        assert_eq!(
            reconciled.status,
            GeminiBrowserProviderStatusKind::NotStarted
        );
        assert_eq!(reconciled.active_run_id, None);
        assert_eq!(reconciled.queue_depth, 0);
    }

    #[test]
    fn provider_status_snapshot_read_core_writes_reconciled_snapshot_back() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let runs_root = temp.path();
        create_queued_run(runs_root, "run-stale", "settings_test", "hello").expect("create queued");
        mark_running(runs_root, "run-stale").expect("mark running");

        let stale = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-stale".to_string()),
            queue_depth: 1,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("stale running".to_string()),
        };
        let mut written = None;

        let returned = provider_status_snapshot_read_core(
            runs_root,
            || Ok(stale.clone()),
            None,
            |expected, snapshot| {
                assert_eq!(expected.status, GeminiBrowserProviderStatusKind::Running);
                written = Some(snapshot);
                Ok(true)
            },
        )
        .expect("snapshot read succeeds");

        let written = written.expect("snapshot write-back");
        assert_eq!(returned, written);
        assert_eq!(written.status, GeminiBrowserProviderStatusKind::NotStarted);
        assert_eq!(written.active_run_id, None);
    }

    #[test]
    fn provider_status_snapshot_read_core_skips_stale_write_back_when_snapshot_changed() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let stale = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-stale".to_string()),
            queue_depth: 1,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("stale running".to_string()),
        };
        let newer = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-newer".to_string()),
            queue_depth: 0,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("newer worker snapshot".to_string()),
        };
        let mut attempted = None;
        let mut reads = 0;

        let returned = provider_status_snapshot_read_core(
            temp.path(),
            || {
                reads += 1;
                if reads == 1 {
                    Ok(stale.clone())
                } else {
                    Ok(newer.clone())
                }
            },
            None,
            |expected, snapshot| {
                attempted = Some((expected.clone(), snapshot.clone()));
                Ok(false)
            },
        )
        .expect("snapshot read succeeds");

        let (_expected, _attempted_snapshot) = attempted.expect("conditional write attempted");
        assert_eq!(returned, newer);
    }

    #[test]
    fn get_run_core_returns_exact_run_from_log() {
        let temp = tempfile::tempdir().expect("create temp dir");
        create_queued_run(temp.path(), "run-detail", "settings_test", "hello").expect("create run");

        let run = get_run_core(temp.path(), "run-detail").expect("get run");

        assert_eq!(run.run_id, "run-detail");
    }

    #[tokio::test]
    async fn provider_status_read_core_waits_for_startup_reconciliation_before_live_status() {
        let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let gate_order = order.clone();
        let active_order = order.clone();
        let live_order = order.clone();

        let status = provider_status_read_core(
            || async move {
                gate_order.lock().unwrap().push("gate");
                Ok(())
            },
            || async move {
                active_order.lock().unwrap().push("active_run_id");
                None
            },
            |_active_run_id| async move {
                live_order.lock().unwrap().push("live_status");
                Ok(GeminiBrowserState::not_started_status(
                    "profile-dir".to_string(),
                ))
            },
            std::time::Duration::from_millis(250),
            || {
                Ok(GeminiBrowserState::not_started_status(
                    "fallback-dir".to_string(),
                ))
            },
        )
        .await
        .expect("status read succeeds");

        assert_eq!(status.browser_profile_dir, "profile-dir");
        assert_eq!(
            order.lock().unwrap().as_slice(),
            ["gate", "active_run_id", "live_status"]
        );
    }

    #[tokio::test]
    async fn send_single_prompt_handoff_writes_run_log_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let request = test_request("run-handoff");
        let browser_config = Some(GeminiBrowserProviderConfig {
            mode: super::super::GeminiBrowserProviderMode::CdpAttach,
            cdp_endpoint: Some("http://127.0.0.1:9222".to_string()),
        });
        let captured_jobs = Arc::new(Mutex::new(Vec::<GeminiBrowserJob>::new()));
        let observed_queued_log = Arc::new(Mutex::new(false));
        let runs_dir = temp.path().to_path_buf();

        let handoff = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            request.clone(),
            browser_config.clone(),
            {
                let captured_jobs = captured_jobs.clone();
                let observed_queued_log = observed_queued_log.clone();
                move |job| {
                    let runs = list_runs(&runs_dir, 10).expect("list runs before enqueue");
                    *observed_queued_log.lock() = runs.runs.iter().any(|run| {
                        run.run_id == job.run_id && run.status == GeminiBrowserRunStatus::Queued
                    });
                    captured_jobs.lock().push(job);
                    async {
                        Ok(QueuedGeminiBrowserJob {
                            run_id: "run-handoff".to_string(),
                            queue_position: Some(1),
                        })
                    }
                }
            },
        )
        .await
        .expect("handoff succeeds");

        let _handoff = handoff;
        assert!(*observed_queued_log.lock());
        let jobs = captured_jobs.lock();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].run_id, request.run_id);
        assert_eq!(jobs[0].prompt, request.prompt);
        assert_eq!(jobs[0].source, request.source);
        assert_eq!(jobs[0].artifact_mode, GeminiBrowserArtifactMode::Reduced);
        assert_eq!(jobs[0].browser_config, browser_config);
        let queued_run = read_run_by_id(temp.path(), &request.run_id);
        assert_eq!(queued_run.status, GeminiBrowserRunStatus::Queued);
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let request = test_request("run-duplicate");
        create_queued_run(
            temp.path(),
            &request.run_id,
            &request.source,
            &request.prompt,
        )
        .expect("create duplicate queued run");

        let error =
            send_single_prompt_enqueue_core(temp.path(), &runtime, request, None, |_job| async {
                panic!("enqueue should not be called")
            })
            .await
            .expect_err("duplicate run id rejected");

        let error = unwrap_enqueue_app_error(error);
        assert!(error.to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_terminal_run_id_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let request = test_request("run-duplicate-terminal");
        create_queued_run(
            temp.path(),
            &request.run_id,
            &request.source,
            &request.prompt,
        )
        .expect("create duplicate queued run");
        finish_run(temp.path(), &request.run_id, ok_result(&request.run_id))
            .expect("finish duplicate run");

        let error =
            send_single_prompt_enqueue_core(temp.path(), &runtime, request, None, |_job| async {
                panic!("enqueue should not be called")
            })
            .await
            .expect_err("duplicate terminal run id rejected");

        let error = unwrap_enqueue_app_error(error);
        assert!(error.to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_waiter_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let request = test_request("run-duplicate-waiter");
        let _waiter = runtime
            .register_waiter(&request.run_id)
            .expect("register waiter");

        let error =
            send_single_prompt_enqueue_core(temp.path(), &runtime, request, None, |_job| async {
                panic!("enqueue should not be called")
            })
            .await
            .expect_err("duplicate waiter rejected");

        let error = unwrap_enqueue_app_error(error);
        assert!(error.to_string().contains("active waiter"));
    }

    #[tokio::test]
    async fn send_single_prompt_marks_run_failed_when_enqueue_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let request = test_request("run-enqueue-fails");

        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            request.clone(),
            None,
            |_job| async { Err(AppError::internal("push failed")) },
        )
        .await
        .expect_err("enqueue error returned");

        let (source, failed_result) = match error {
            SendSinglePromptEnqueueError::EnqueueFailed {
                run_id,
                source,
                failed_result,
            } => {
                assert_eq!(run_id, request.run_id);
                (source, failed_result)
            }
            other => panic!("unexpected enqueue error {other:?}"),
        };
        assert_eq!(source.to_string(), "push failed");
        assert_eq!(failed_result.run_id, request.run_id);
        assert!(!runtime.has_waiter(&request.run_id));
        let run = read_run_by_id(temp.path(), &request.run_id);
        assert_eq!(run.run_id, request.run_id);
        assert_eq!(run.status, GeminiBrowserRunStatus::Failed);
        assert!(run
            .result
            .as_ref()
            .and_then(|result| result.message.as_deref())
            .unwrap_or("")
            .contains("Gemini Browser job enqueue failed: push failed"));
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_invalid_artifact_mode_before_side_effects() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let mut request = test_request("run-invalid-artifact");
        request.artifact_mode = "invalid".to_string();

        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            request.clone(),
            None,
            |_job| async { panic!("enqueue should not be called") },
        )
        .await
        .expect_err("invalid artifact mode rejected");

        let error = unwrap_enqueue_app_error(error);
        assert!(error
            .to_string()
            .contains("unsupported Gemini Browser artifact_mode"));
        assert!(list_runs(temp.path(), 10)
            .expect("list runs")
            .runs
            .is_empty());
        assert!(!runtime.has_waiter(&request.run_id));
    }

    #[tokio::test]
    async fn failed_run_log_transition_returns_app_error_without_side_effects() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let mut request = test_request("bad/run-id");
        request.run_id = "../bad".to_string();

        let error =
            send_single_prompt_enqueue_core(temp.path(), &runtime, request, None, |_job| async {
                panic!("enqueue should not be called")
            })
            .await
            .expect_err("invalid run id rejected before side effects");

        let error = unwrap_enqueue_app_error(error);
        assert!(!error.to_string().is_empty());
        assert!(list_runs(temp.path(), 10)
            .expect("list runs")
            .runs
            .is_empty());
    }

    fn unwrap_enqueue_app_error(error: SendSinglePromptEnqueueError) -> AppError {
        match error {
            SendSinglePromptEnqueueError::App(error) => error,
            other => panic!("unexpected enqueue error {other:?}"),
        }
    }

    fn ready_runtime() -> GeminiBrowserJobRuntime {
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
        runtime.mark_worker_ready("2026-06-22T00:00:00Z".to_string());
        runtime
    }

    fn test_request(run_id: &str) -> GeminiBrowserRunRequest {
        GeminiBrowserRunRequest {
            run_id: run_id.to_string(),
            prompt: "hello Gemini".to_string(),
            source: "settings_test".to_string(),
            artifact_mode: "reduced".to_string(),
        }
    }

    fn ok_result(run_id: &str) -> GeminiBrowserRunResult {
        GeminiBrowserRunResult {
            run_id: run_id.to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("ok".to_string()),
            message: Some("done".to_string()),
            manual_action: None,
            artifacts: super::super::GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 1,
            debug_summary: None,
        }
    }

    fn read_run_by_id(runs_dir: &Path, run_id: &str) -> super::super::GeminiBrowserRun {
        list_runs(runs_dir, 20)
            .expect("list runs")
            .runs
            .into_iter()
            .find(|run| run.run_id == run_id)
            .unwrap_or_else(|| panic!("missing run {run_id}"))
    }
}
