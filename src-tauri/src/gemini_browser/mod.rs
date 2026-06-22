mod cdp_chrome;
mod commands;
mod jobs;
mod paths;
mod run_log;
mod sidecar;
mod sidecar_launch;
mod state;
mod types;

pub use commands::{
    gemini_bridge_list_runs, gemini_bridge_open_browser, gemini_bridge_open_run_folder,
    gemini_bridge_resume, gemini_bridge_send_single, gemini_bridge_start_cdp_chrome,
    gemini_bridge_status, gemini_bridge_stop,
};
pub(crate) use commands::{provider_status, send_single_prompt};
pub(crate) use jobs::{
    cancel_gemini_browser_job, start_gemini_browser_job_worker, GeminiBrowserJobRuntime,
};
pub(crate) use paths::{chrome_cdp_profile_dir, path_string, profile_dir, run_dir, runs_dir};
pub(crate) use run_log::{
    create_queued_run, finish_run, list_runs, mark_running, recorded_run_dir,
};
pub use state::GeminiBrowserState;
pub use types::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserDebugErrorStage,
    GeminiBrowserProviderConfig, GeminiBrowserProviderMode, GeminiBrowserProviderStatus,
    GeminiBrowserProviderStatusKind, GeminiBrowserRun, GeminiBrowserRunChangeEvent,
    GeminiBrowserRunDebugSummary, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest,
    GeminiBrowserRunResult, GeminiBrowserRunStatus, GeminiBrowserSidecarCommand,
    GeminiBrowserSidecarEnvelope, GeminiBrowserSidecarResponse, GeminiBrowserStartChromeResult,
};
