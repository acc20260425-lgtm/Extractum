use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};

use super::{
    create_queued_run, finish_run, list_runs, mark_running, path_string, profile_dir, runs_dir,
    sidecar, GeminiBrowserProviderStatus, GeminiBrowserRunEvent, GeminiBrowserRunLogSummary,
    GeminiBrowserRunRequest, GeminiBrowserRunResult, GeminiBrowserRunStatus, GeminiBrowserState,
};

pub const GEMINI_BROWSER_RUN_EVENT: &str = "gemini-browser://run";

fn emit_run_event(handle: &AppHandle, event: GeminiBrowserRunEvent) {
    let _ = handle.emit(GEMINI_BROWSER_RUN_EVENT, event);
}

#[tauri::command]
pub async fn gemini_bridge_status(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<GeminiBrowserProviderStatus> {
    sidecar::status(
        path_string(&profile_dir(&handle)?),
        state.active_run_id().await,
        state.queue_depth().await,
    )
    .await
}

#[tauri::command]
pub async fn gemini_bridge_open_browser(
    handle: AppHandle,
) -> AppResult<GeminiBrowserProviderStatus> {
    sidecar::open_browser(&handle, path_string(&profile_dir(&handle)?)).await
}

#[tauri::command]
pub async fn gemini_bridge_send_single(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    run_id: String,
    prompt: String,
    source: Option<String>,
    artifact_mode: Option<String>,
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
    create_queued_run(&runs_root, &request.run_id, &request.source, &request.prompt)?;
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

    let result = sidecar::send_single_stub(next.clone()).await?;
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
pub async fn gemini_bridge_resume() -> AppResult<()> {
    Ok(())
}

#[tauri::command]
pub async fn gemini_bridge_stop(state: State<'_, GeminiBrowserState>) -> AppResult<()> {
    state.request_stop().await;
    Ok(())
}

#[tauri::command]
pub async fn gemini_bridge_list_runs(
    handle: AppHandle,
    limit: Option<usize>,
) -> AppResult<GeminiBrowserRunLogSummary> {
    list_runs(&runs_dir(&handle)?, limit.unwrap_or(20))
}
