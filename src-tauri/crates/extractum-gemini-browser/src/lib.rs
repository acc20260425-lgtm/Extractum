mod cdp;
mod error;
mod execution;
mod executor;
mod protocol;
mod reconciliation;
mod run_id;
mod run_log;
mod runtime;
mod sidecar_launch;
mod state;
mod status;
mod submission;
mod types;

pub use cdp::{build_chrome_cdp_launch_spec, start_chrome_result, ChromeCdpLaunchSpec};
pub use error::{GeminiBrowserError, GeminiBrowserErrorKind, GeminiBrowserResult};
pub use execution::{
    cancel_run, execute_delivered_job, CancelRunOutcome, DeliveredJobInput, DeliveryOutcome,
};
pub use executor::{
    BrowserExecutor, BrowserExecutorFuture, BrowserRunContext, BrowserSessionContext,
    BrowserStopReason,
};
pub use protocol::{classify_resume_response, GeminiBrowserJsonlCodec, ResumeSidecarOutcome};
pub use reconciliation::{
    ensure_startup_reconciled, reconcile_startup, NormalizedQueueState, QueueInspectionSnapshot,
    ReconciliationAction, StartupReconciliationSnapshot,
};
pub use run_id::safe_run_id;
pub use run_log::{
    create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
};
pub use runtime::{
    run_registered_worker, GeminiBrowserArtifactMode, GeminiBrowserJob, GeminiBrowserJobRuntime,
};
pub use sidecar_launch::{
    bundled_sidecar_path, dev_sidecar_script, resolve_launch_mode, GeminiBrowserBuildProfile,
    GeminiBrowserSidecarLaunch, GEMINI_BROWSER_SIDECAR_NAME,
};
pub use state::GeminiBrowserDomainState;
pub use status::{
    open_provider, read_provider_status, read_reconciled_status_snapshot, resume_provider,
    StatusObserver,
};
pub use submission::{submit_and_wait, QueuedGeminiBrowserJob};
pub use types::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserAnswerExtractionDebug,
    GeminiBrowserAnswerGrouping, GeminiBrowserArtifactRefs, GeminiBrowserCandidateRejectReason,
    GeminiBrowserDebugErrorStage, GeminiBrowserManualAction, GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRun, GeminiBrowserRunDebugSummary, GeminiBrowserRunLogSummary,
    GeminiBrowserRunRequest, GeminiBrowserRunResult, GeminiBrowserRunStatus,
    GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope, GeminiBrowserSidecarResponse,
    GeminiBrowserStartChromeResult,
};
