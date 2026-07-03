mod read_model;
mod runs;
mod setup;
mod snapshot;

pub(crate) use self::read_model::{
    fetch_run_row, list_analysis_run_summaries, map_run_detail, map_run_summary,
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
