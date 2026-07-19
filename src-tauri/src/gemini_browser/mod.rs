mod browser_executor;
mod cdp_chrome;
mod cdp_contract;
mod commands;
mod domain_error;
mod execution;
mod executor;
mod jobs;
mod paths;
mod portable_state;
mod protocol;
mod reconciliation;
mod run_id;
mod run_log;
mod runtime;
mod sidecar;
mod sidecar_launch;
mod state;
mod status;
mod submission;
mod types;

pub(crate) use cdp_chrome::shutdown_cdp_chrome;
pub use commands::{
    gemini_bridge_get_run, gemini_bridge_list_runs, gemini_bridge_open_browser,
    gemini_bridge_open_run_folder, gemini_bridge_resume, gemini_bridge_send_single,
    gemini_bridge_start_cdp_chrome, gemini_bridge_status, gemini_bridge_status_snapshot,
    gemini_bridge_stop,
};
pub(crate) use commands::{provider_status, send_single_prompt};
pub(crate) use jobs::{cancel_gemini_browser_job, start_gemini_browser_job_worker};
#[cfg(test)]
pub(crate) use jobs::{
    enqueue_gemini_browser_job_to_storage, open_gemini_browser_job_storage,
    setup_gemini_browser_apalis_storage,
};
pub(crate) use paths::{chrome_cdp_profile_dir, path_string, profile_dir, run_dir, runs_dir};
pub(crate) use runtime::GeminiBrowserJobRuntime;
#[cfg(test)]
pub(crate) use runtime::{GeminiBrowserArtifactMode, GeminiBrowserJob};
pub(crate) use sidecar::shutdown_sidecar;
pub use state::GeminiBrowserState;
pub use types::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
    GeminiBrowserSidecarResponse, GeminiBrowserStartChromeResult,
};
#[cfg(test)]
pub(crate) use types::{GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary};

pub(crate) fn create_queued_run(
    runs_dir: &std::path::Path,
    run_id: &str,
    source: &str,
    prompt: &str,
) -> crate::error::AppResult<GeminiBrowserRun> {
    run_log::create_queued_run(runs_dir, run_id, source, prompt)
        .map_err(executor::domain_error_to_app)
}

pub(crate) fn mark_running(
    runs_dir: &std::path::Path,
    run_id: &str,
) -> crate::error::AppResult<GeminiBrowserRun> {
    run_log::mark_running(runs_dir, run_id).map_err(executor::domain_error_to_app)
}

pub(crate) fn finish_run(
    runs_dir: &std::path::Path,
    run_id: &str,
    result: GeminiBrowserRunResult,
) -> crate::error::AppResult<GeminiBrowserRun> {
    run_log::finish_run(runs_dir, run_id, result).map_err(executor::domain_error_to_app)
}

pub(crate) fn list_runs(
    runs_dir: &std::path::Path,
    limit: usize,
) -> crate::error::AppResult<GeminiBrowserRunLogSummary> {
    run_log::list_runs(runs_dir, limit).map_err(executor::domain_error_to_app)
}

pub(crate) fn read_run(
    runs_dir: &std::path::Path,
    run_id: &str,
) -> crate::error::AppResult<GeminiBrowserRun> {
    run_log::read_run(runs_dir, run_id).map_err(executor::domain_error_to_app)
}

pub(crate) fn recorded_run_dir(
    runs_dir: &std::path::Path,
    run_id: &str,
) -> crate::error::AppResult<std::path::PathBuf> {
    run_log::recorded_run_dir(runs_dir, run_id).map_err(executor::domain_error_to_app)
}
