use tauri::AppHandle;

use crate::error::{AppError, AppResult};

use super::{
    GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind, GeminiBrowserRunRequest,
    GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

pub(crate) async fn status(
    browser_profile_dir: String,
    active_run_id: Option<String>,
    queue_depth: usize,
) -> AppResult<GeminiBrowserProviderStatus> {
    Ok(GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::NotStarted,
        manual_action: None,
        active_run_id,
        queue_depth,
        browser_profile_dir,
        latest_message: Some("Gemini browser sidecar is not running yet.".to_string()),
    })
}

pub(crate) async fn open_browser(
    _handle: &AppHandle,
    browser_profile_dir: String,
) -> AppResult<GeminiBrowserProviderStatus> {
    Ok(GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::NotStarted,
        manual_action: None,
        active_run_id: None,
        queue_depth: 0,
        browser_profile_dir,
        latest_message: Some("Browser launch will be enabled when the sidecar is wired.".to_string()),
    })
}

pub(crate) async fn send_single_stub(
    request: GeminiBrowserRunRequest,
) -> AppResult<GeminiBrowserRunResult> {
    if request.prompt.trim().is_empty() {
        return Err(AppError::validation("prompt cannot be empty"));
    }
    Ok(GeminiBrowserRunResult {
        run_id: request.run_id,
        status: GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some("Gemini browser sidecar is not wired yet.".to_string()),
        manual_action: None,
        artifacts: Default::default(),
        elapsed_ms: 0,
    })
}
