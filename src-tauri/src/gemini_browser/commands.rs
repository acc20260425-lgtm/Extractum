use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};

use super::{
    cdp_chrome, chrome_cdp_profile_dir, create_queued_run, finish_run, list_runs, mark_running,
    path_string, profile_dir, run_dir, runs_dir, sidecar, GeminiBrowserProviderConfig,
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
    sidecar::status(
        &handle,
        &state,
        path_string(&profile_dir(&handle)?),
        browser_config,
        state.active_run_id().await,
        state.queue_depth().await,
    )
    .await
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
    Ok(cdp_chrome::start_chrome_result(&spec))
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

    let runs_root = runs_dir(&handle)?;
    create_queued_run(
        &runs_root,
        &request.run_id,
        &request.source,
        &request.prompt,
    )?;
    let queue_position = state.enqueue(request.clone()).await;
    emit_run_event(
        &handle,
        GeminiBrowserRunEvent {
            run_id: request.run_id.clone(),
            status: GeminiBrowserRunStatus::Queued,
            message: Some("Queued".to_string()),
            queue_position: Some(queue_position),
        },
    );

    let next = state
        .pop_next()
        .await
        .ok_or_else(|| AppError::internal("Gemini browser queue unexpectedly empty"))?;
    let _token = state.start_run(next.run_id.clone()).await;
    mark_running(&runs_root, &next.run_id)?;
    emit_run_event(
        &handle,
        GeminiBrowserRunEvent {
            run_id: next.run_id.clone(),
            status: GeminiBrowserRunStatus::Running,
            message: Some("Running".to_string()),
            queue_position: None,
        },
    );

    let artifact_dir = path_string(&run_dir(&handle, &next.run_id)?);
    let result = match sidecar::send_single(
        &handle,
        &state,
        next.clone(),
        path_string(&profile_dir(&handle)?),
        artifact_dir,
        browser_config,
    )
    .await
    {
        Ok(result) => result,
        Err(_) => sidecar::sidecar_unavailable_result(next.clone()),
    };
    finish_run(&runs_root, &next.run_id, result.clone())?;
    state.finish_run(&next.run_id).await;
    emit_run_event(
        &handle,
        GeminiBrowserRunEvent {
            run_id: next.run_id,
            status: result.status.clone(),
            message: result.message.clone(),
            queue_position: None,
        },
    );
    Ok(result)
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
