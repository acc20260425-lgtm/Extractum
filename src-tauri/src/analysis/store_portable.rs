#[path = "store/read_model.rs"]
mod read_model;
#[path = "store/runs.rs"]
mod runs;
#[path = "store/setup.rs"]
mod setup;
#[path = "store/snapshot.rs"]
mod snapshot;

#[allow(unused_imports)]
pub use self::read_model::{
    load_analysis_chat_run, prepare_active_analysis_run_summaries, prepare_analysis_run_detail,
    prepare_analysis_run_summaries, prepare_legacy_analysis_chat_run, AnalysisChatRunEnrichment,
    AnalysisRunDetailEnrichment, AnalysisRunListFilters, AnalysisRunSummaryEnrichment,
};
#[allow(unused_imports)]
pub use self::runs::{
    analysis_run_ids_depending_on_sources, delete_analysis_run, delete_project_analysis_runs,
    load_analysis_run_diagnostics, load_project_analysis_run_aggregates,
    AnalysisRunDiagnosticCount, ProjectAnalysisRunAggregate,
};

pub(crate) use self::read_model::{
    analysis_run_exists, load_analysis_run_status, resolve_run_scope_label,
};
pub(crate) use self::runs::{
    find_active_duplicate_run, insert_analysis_run, set_run_status, AnalysisRunInsert,
    DuplicateRunLookup,
};
pub(crate) use self::setup::{ensure_builtin_report_template, fetch_prompt_template};
#[allow(unused_imports)]
pub(crate) use self::snapshot::{
    capture_run_snapshot, mark_run_capture_failed, persist_run_snapshot, sanitize_provider_error,
    sanitize_snapshot_error,
};

#[cfg(test)]
#[path = "store/tests/mod.rs"]
mod tests;
