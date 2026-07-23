#[path = "store/read_model.rs"]
mod app_read_model;
#[path = "store/owned_read_model.rs"]
mod read_model;
mod runs;
mod setup;
mod snapshot;

pub(crate) use self::app_read_model::{
    get_analysis_run_in_pool, list_active_analysis_runs_in_pool, list_analysis_runs_in_pool,
    resolve_legacy_analysis_chat_run_in_pool,
};
pub(crate) use self::read_model::{
    analysis_run_exists, load_analysis_run_status, load_analysis_run_trace_data,
    resolve_run_scope_label, AnalysisRunListFilters,
};
pub(crate) use self::runs::{
    delete_saved_run, find_active_duplicate_run, insert_analysis_run, set_run_status,
    AnalysisRunInsert, DuplicateRunLookup,
};
pub(crate) use self::setup::{
    ensure_builtin_report_template, ensure_sources_exist, fetch_prompt_template, fetch_source_group,
};
#[allow(unused_imports)]
pub(crate) use self::snapshot::{
    capture_run_snapshot, mark_run_capture_failed, persist_run_snapshot, sanitize_provider_error,
    sanitize_snapshot_error,
};

#[cfg(test)]
mod tests;
