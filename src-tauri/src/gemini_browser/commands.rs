use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::error::{AppError, AppResult};
use crate::external_process::ExternalProcessShutdownState;

use super::executor::{
    app_error_to_domain, domain_error_to_app, AppBrowserExecutor, AppStatusObserver,
    DomainErrorContext,
};
use super::jobs::{cancel_gemini_browser_job, enqueue_gemini_browser_job};
use super::{
    cdp_chrome, chrome_cdp_profile_dir, list_runs, path_string, profile_dir, read_run,
    recorded_run_dir, runs_dir, GeminiBrowserProviderConfig, GeminiBrowserProviderStatus,
    GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserStartChromeResult, GeminiBrowserState,
};
use extractum_gemini_browser::{
    build_chrome_cdp_launch_spec, open_provider, read_provider_status,
    read_reconciled_status_snapshot, resume_provider, start_chrome_result, submit_and_wait,
    BrowserExecutor, BrowserSessionContext, BrowserStopReason, GeminiBrowserJobRuntime,
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
    read_reconciled_status_snapshot(
        state.domain(),
        &runs_dir(&handle)?,
        path_string(&profile_dir(&handle)?),
    )
    .map_err(domain_error_to_app)
}

pub(crate) async fn provider_status(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
    let browser_profile_dir = path_string(&profile_dir(handle)?);
    let executor = AppBrowserExecutor::new(handle, state);
    super::jobs::ensure_gemini_browser_startup_reconciled(handle).await?;
    read_provider_status(
        state.domain(),
        &executor,
        BrowserSessionContext {
            browser_profile_dir,
            browser_config,
        },
        0,
        std::time::Duration::from_millis(250),
    )
    .await
    .map_err(domain_error_to_app)
}

fn get_run_core(runs_root: &std::path::Path, run_id: &str) -> AppResult<GeminiBrowserRun> {
    read_run(runs_root, run_id).map_err(domain_error_to_app)
}

#[tauri::command]
pub async fn gemini_bridge_open_browser(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
    let executor = AppBrowserExecutor::new(&handle, &state);
    open_provider(
        &executor,
        &AppStatusObserver,
        BrowserSessionContext {
            browser_profile_dir: path_string(&profile_dir(&handle)?),
            browser_config,
        },
    )
    .await
    .map_err(domain_error_to_app)
}

#[tauri::command]
pub async fn gemini_bridge_start_cdp_chrome(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserStartChromeResult> {
    let chrome_path = cdp_chrome::find_chrome_executable();
    let spec =
        build_chrome_cdp_launch_spec(chrome_cdp_profile_dir(&handle)?, browser_config.as_ref())
            .map_err(domain_error_to_app)?;
    let shutdown = handle
        .state::<ExternalProcessShutdownState>()
        .inner()
        .clone();
    let permit = shutdown
        .try_admit()
        .map_err(|_| AppError::internal("Application is shutting down"))?;
    let spawn_spec = spec.clone();
    let process = tokio::task::spawn_blocking(move || {
        cdp_chrome::spawn_chrome_cdp(&chrome_path, &spawn_spec)
    })
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
    Ok(start_chrome_result(&spec))
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
    submit_and_wait(
        &runs_root,
        &runtime,
        state.domain(),
        &AppStatusObserver,
        request.clone(),
        browser_config.clone(),
        |job| async move {
            enqueue_gemini_browser_job(handle, job)
                .await
                .map_err(|error| app_error_to_domain(error, DomainErrorContext::Persistence))
        },
    )
    .await
    .map_err(domain_error_to_app)
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
    let executor = AppBrowserExecutor::new(&handle, &state);
    resume_provider(
        &executor,
        &AppStatusObserver,
        BrowserSessionContext {
            browser_profile_dir: path_string(&profile_dir(&handle)?),
            browser_config,
        },
    )
    .await
    .map_err(domain_error_to_app)
}

#[tauri::command]
pub async fn gemini_bridge_stop(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<()> {
    if let Some(run_id) = state.active_run_id().await {
        return cancel_gemini_browser_job(&handle, &run_id).await;
    }
    AppBrowserExecutor::new(&handle, &state)
        .stop(BrowserStopReason::Requested)
        .await
        .map_err(domain_error_to_app)
}

#[tauri::command]
pub async fn gemini_bridge_list_runs(
    handle: AppHandle,
    limit: Option<usize>,
) -> AppResult<GeminiBrowserRunLogSummary> {
    super::jobs::ensure_gemini_browser_startup_reconciled(&handle).await?;
    list_runs(&runs_dir(&handle)?, limit.unwrap_or(20)).map_err(domain_error_to_app)
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
    let dir = recorded_run_dir(&runs_dir(&handle)?, &run_id).map_err(domain_error_to_app)?;
    handle
        .opener()
        .open_path(path_string(&dir), None::<&str>)
        .map_err(|error| {
            AppError::internal(format!("Failed to open Gemini browser run folder: {error}"))
        })?;
    Ok(())
}
