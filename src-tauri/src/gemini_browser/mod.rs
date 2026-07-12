mod cdp_chrome;
mod commands;
mod jobs;
mod paths;
mod run_log;
mod sidecar;
mod sidecar_launch;
mod state;
mod types;

pub(crate) use cdp_chrome::shutdown_cdp_chrome;
pub use commands::{
    gemini_bridge_get_run, gemini_bridge_list_runs, gemini_bridge_open_browser,
    gemini_bridge_open_run_folder, gemini_bridge_resume, gemini_bridge_send_single,
    gemini_bridge_start_cdp_chrome, gemini_bridge_status, gemini_bridge_status_snapshot,
    gemini_bridge_stop,
};
pub(crate) use commands::{provider_status, send_single_prompt};
pub(crate) use jobs::{
    cancel_gemini_browser_job, start_gemini_browser_job_worker, GeminiBrowserJobRuntime,
};
#[cfg(test)]
pub(crate) use jobs::{
    enqueue_gemini_browser_job_to_storage, open_gemini_browser_job_storage,
    setup_gemini_browser_apalis_storage, GeminiBrowserArtifactMode, GeminiBrowserJob,
};
pub(crate) use paths::{chrome_cdp_profile_dir, path_string, profile_dir, run_dir, runs_dir};
pub(crate) use run_log::{
    create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
};
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
