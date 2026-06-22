use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::error::{AppError, AppResult};

use super::jobs::{
    cancel_gemini_browser_job, enqueue_gemini_browser_job, GeminiBrowserArtifactMode,
    GeminiBrowserJob, GeminiBrowserJobRuntime, GeminiBrowserWaiterReceiver, QueuedGeminiBrowserJob,
};
use super::{
    cdp_chrome, chrome_cdp_profile_dir, create_queued_run, finish_run, list_runs, path_string,
    profile_dir, recorded_run_dir, runs_dir, sidecar, GeminiBrowserProviderConfig,
    GeminiBrowserProviderStatus, GeminiBrowserRunEvent, GeminiBrowserRunLogSummary,
    GeminiBrowserRunRequest, GeminiBrowserRunResult, GeminiBrowserRunStatus,
    GeminiBrowserStartChromeResult, GeminiBrowserState,
};

pub const GEMINI_BROWSER_RUN_EVENT: &str = "gemini-browser://run";

fn emit_run_event(handle: &AppHandle, event: GeminiBrowserRunEvent) {
    let _ = handle.emit(GEMINI_BROWSER_RUN_EVENT, event);
}

#[tauri::command]
pub async fn gemini_bridge_status(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
    provider_status(&handle, &state, browser_config).await
}

pub(crate) async fn provider_status(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
    let active_run_id = state.active_run_id().await;
    let queue_depth = 0;
    let browser_profile_dir = path_string(&profile_dir(handle)?);
    let live_status = sidecar::status(
        handle,
        state,
        browser_profile_dir,
        browser_config,
        active_run_id,
        queue_depth,
    );

    provider_status_core(
        state,
        live_status,
        std::time::Duration::from_millis(250),
        || state.status_snapshot(handle),
    )
    .await
}

#[derive(Debug)]
struct SendSinglePromptEnqueueHandoff {
    queued_event: GeminiBrowserRunEvent,
    waiter: GeminiBrowserWaiterReceiver,
}

async fn send_single_prompt_enqueue_core<Enqueue, EnqueueFut, EmitEvent>(
    runs_root: &std::path::Path,
    runtime: &GeminiBrowserJobRuntime,
    request: GeminiBrowserRunRequest,
    browser_config: Option<GeminiBrowserProviderConfig>,
    enqueue: Enqueue,
    mut emit_event: EmitEvent,
) -> AppResult<SendSinglePromptEnqueueHandoff>
where
    Enqueue: FnOnce(GeminiBrowserJob) -> EnqueueFut,
    EnqueueFut: std::future::Future<Output = AppResult<QueuedGeminiBrowserJob>>,
    EmitEvent: FnMut(GeminiBrowserRunEvent),
{
    let artifact_mode = GeminiBrowserArtifactMode::from_wire(Some(&request.artifact_mode))?;

    runtime.ensure_worker_ready_for_enqueue().await?;
    reject_duplicate_existing_run_or_waiter(runtime, runs_root, &request.run_id).await?;
    create_queued_run(runs_root, &request.run_id, &request.source, &request.prompt)?;
    let waiter = runtime.register_waiter(&request.run_id)?;

    let queued = match enqueue(GeminiBrowserJob {
        run_id: request.run_id.clone(),
        prompt: request.prompt.clone(),
        source: request.source.clone(),
        artifact_mode,
        browser_config,
    })
    .await
    {
        Ok(queued) => queued,
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
            finish_run(runs_root, &request.run_id, failed.clone())?;
            emit_event(GeminiBrowserRunEvent {
                run_id: request.run_id,
                status: GeminiBrowserRunStatus::Failed,
                message: failed.message,
                queue_position: None,
            });
            return Err(error);
        }
    };

    let queued_event = GeminiBrowserRunEvent {
        run_id: queued.run_id,
        status: GeminiBrowserRunStatus::Queued,
        message: Some("Queued".to_string()),
        queue_position: queued.queue_position,
    };
    emit_event(queued_event.clone());

    Ok(SendSinglePromptEnqueueHandoff {
        queued_event,
        waiter,
    })
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
    state: &GeminiBrowserState,
    live_status: Fut,
    timeout: std::time::Duration,
    fallback_status: impl FnOnce() -> AppResult<GeminiBrowserProviderStatus>,
) -> AppResult<GeminiBrowserProviderStatus>
where
    Fut: std::future::Future<Output = AppResult<GeminiBrowserProviderStatus>>,
{
    match tokio::time::timeout(timeout, live_status).await {
        Ok(Ok(status)) => {
            state.set_status_snapshot(status.clone());
            Ok(status)
        }
        Ok(Err(_)) | Err(_) => fallback_status(),
    }
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
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserStartChromeResult> {
    let spec = cdp_chrome::build_chrome_cdp_launch_spec(
        cdp_chrome::find_chrome_executable(),
        chrome_cdp_profile_dir(&handle)?,
        browser_config.as_ref(),
    )?;
    cdp_chrome::spawn_chrome_cdp(&spec)?;
    cdp_chrome::wait_for_cdp_endpoint(&spec.cdp_endpoint).await?;
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
        |event| {
            let _ = state.update_status_snapshot(handle, |status| {
                status.status =
                    GeminiBrowserState::provider_status_kind_for_run_status(&event.status);
                status.active_run_id = None;
                status.queue_depth = 0;
                status.latest_message = event.message.clone();
                status.manual_action = None;
            });
            emit_run_event(handle, event);
        },
    )
    .await?;

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
    sidecar::stop(&handle, &state).await
}

#[tauri::command]
pub async fn gemini_bridge_list_runs(
    handle: AppHandle,
    limit: Option<usize>,
) -> AppResult<GeminiBrowserRunLogSummary> {
    list_runs(&runs_dir(&handle)?, limit.unwrap_or(20))
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
            &state,
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
        let events = Arc::new(Mutex::new(Vec::<GeminiBrowserRunEvent>::new()));
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
            {
                let events = events.clone();
                move |event| events.lock().push(event)
            },
        )
        .await
        .expect("handoff succeeds");

        assert!(*observed_queued_log.lock());
        let jobs = captured_jobs.lock();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].run_id, request.run_id);
        assert_eq!(jobs[0].prompt, request.prompt);
        assert_eq!(jobs[0].source, request.source);
        assert_eq!(jobs[0].artifact_mode, GeminiBrowserArtifactMode::Reduced);
        assert_eq!(jobs[0].browser_config, browser_config);
        assert_eq!(handoff.queued_event.status, GeminiBrowserRunStatus::Queued);
        assert_eq!(events.lock().as_slice(), [handoff.queued_event.clone()]);
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

        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            request,
            None,
            |_job| async { panic!("enqueue should not be called") },
            |_event| {},
        )
        .await
        .expect_err("duplicate run id rejected");

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

        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            request,
            None,
            |_job| async { panic!("enqueue should not be called") },
            |_event| {},
        )
        .await
        .expect_err("duplicate terminal run id rejected");

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

        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            request,
            None,
            |_job| async { panic!("enqueue should not be called") },
            |_event| {},
        )
        .await
        .expect_err("duplicate waiter rejected");

        assert!(error.to_string().contains("active waiter"));
    }

    #[tokio::test]
    async fn send_single_prompt_marks_run_failed_when_enqueue_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let request = test_request("run-enqueue-fails");
        let events = Arc::new(Mutex::new(Vec::<GeminiBrowserRunEvent>::new()));

        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            request.clone(),
            None,
            |_job| async { Err(AppError::internal("push failed")) },
            {
                let events = events.clone();
                move |event| events.lock().push(event)
            },
        )
        .await
        .expect_err("enqueue error returned");

        assert_eq!(error.to_string(), "push failed");
        assert!(!runtime.has_waiter(&request.run_id));
        let run = read_run_by_id(temp.path(), &request.run_id);
        assert_eq!(run.status, GeminiBrowserRunStatus::Failed);
        assert!(run
            .result
            .as_ref()
            .and_then(|result| result.message.as_deref())
            .unwrap_or("")
            .contains("Gemini Browser job enqueue failed: push failed"));
        assert_eq!(
            events.lock().last().map(|event| event.status.clone()),
            Some(GeminiBrowserRunStatus::Failed)
        );
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
            |_event| {},
        )
        .await
        .expect_err("invalid artifact mode rejected");

        assert!(error
            .to_string()
            .contains("unsupported Gemini Browser artifact_mode"));
        assert!(list_runs(temp.path(), 10)
            .expect("list runs")
            .runs
            .is_empty());
        assert!(!runtime.has_waiter(&request.run_id));
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
