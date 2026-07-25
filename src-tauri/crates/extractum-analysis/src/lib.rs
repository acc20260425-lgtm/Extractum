mod chat;
mod corpus;
mod domain;
mod groups;
mod models;
mod report;
mod state;
mod store;
mod templates;
#[cfg(test)]
mod test_schema;
#[cfg(test)]
mod tests;
mod trace;

use domain::{
    now_secs, ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS, ANALYSIS_RUN_TYPE_REPORT,
    ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
};
#[cfg(test)]
use domain::{
    validate_chat_role, validate_chat_turns, ANALYSIS_STATUS_COMPLETED, ANALYSIS_STATUS_FAILED,
    TEMPLATE_KIND_REPORT,
};

pub use chat::{
    clear_analysis_chat_messages_in_pool as clear_analysis_chat_messages, complete_analysis_chat,
    execute_analysis_chat, list_analysis_chat_messages_in_pool as list_analysis_chat_messages,
    prepare_analysis_chat, publish_analysis_chat_execution_error,
    publish_analysis_chat_persistence_error, AnalysisChatCompletionTicket,
    AnalysisChatExecutionTicket, AskAnalysisRunQuestionRequest,
};
pub use corpus::snapshot::list_run_snapshot_messages_page as list_analysis_run_messages;
pub use corpus::source_resolution::resolve_analysis_telegram_history_scope;
pub use corpus::{
    preflight_analysis_corpus, AnalysisCorpusMessage, AnalysisCorpusReader, AnalysisCorpusRequest,
    AnalysisPortFuture, AnalysisRunPreflight, AnalysisRunPreflightLimits, YoutubeCorpusMode,
};
pub use groups::{
    create_analysis_source_group_in_pool as create_analysis_source_group,
    delete_analysis_source_group_in_pool as delete_analysis_source_group,
    get_analysis_source_group_record, load_analysis_source_group_for_enrichment,
    load_analysis_source_groups_for_enrichment,
    update_analysis_source_group_in_pool as update_analysis_source_group, AnalysisSourceGroupInput,
    AnalysisSourceGroupRecord,
};
pub use models::{
    AnalysisChatEvent, AnalysisChatMessage, AnalysisChatRun, AnalysisChatTurn,
    AnalysisChunkSummaryEvent, AnalysisEventSink, AnalysisForeignLabelMatch,
    AnalysisForeignLabelRef, AnalysisForeignLabels, AnalysisProjectLabel, AnalysisPromptTemplate,
    AnalysisRunDetail, AnalysisRunEvent, AnalysisRunMessage, AnalysisRunMessageCursor,
    AnalysisRunMessagesPage, AnalysisRunSummary, AnalysisScopeKind, AnalysisSnapshotState,
    AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceKind, AnalysisSourceLabel,
    AnalysisSourceOption, AnalysisTraceData, AnalysisTraceRef, ResolvedAnalysisScope,
};
pub use report::{
    capture_analysis_corpus, execute_analysis_report, finalize_analysis_report_execution,
    mark_interrupted_analysis_runs, prepare_analysis_report, prepare_analysis_report_execution,
    request_analysis_run_cancel_in_pool as request_analysis_run_cancel, AnalysisExecutionError,
    AnalysisReportExecutionTicket, AnalysisReportPreparationTicket, AnalysisReportScopeTicket,
    StartAnalysisReportRequest,
};
pub use state::{AnalysisReportCancellationWait, AnalysisState};
pub use store::{
    analysis_run_ids_depending_on_sources, delete_analysis_run, delete_project_analysis_runs,
    load_analysis_chat_run, load_analysis_run_diagnostics, load_project_analysis_run_aggregates,
    prepare_active_analysis_run_summaries, prepare_analysis_run_detail,
    prepare_analysis_run_summaries, prepare_legacy_analysis_chat_run, AnalysisChatRunEnrichment,
    AnalysisRunDetailEnrichment, AnalysisRunDiagnosticCount, AnalysisRunListFilters,
    AnalysisRunSummaryEnrichment, ProjectAnalysisRunAggregate,
};
pub use templates::{
    create_analysis_prompt_template_in_pool as create_analysis_prompt_template,
    delete_analysis_prompt_template_in_pool as delete_analysis_prompt_template,
    list_analysis_prompt_templates_in_pool as list_analysis_prompt_templates,
    update_analysis_prompt_template_in_pool as update_analysis_prompt_template,
};
pub use trace::{
    get_analysis_run_trace_in_pool as get_analysis_run_trace,
    resolve_analysis_trace_refs_in_pool as resolve_analysis_trace_refs,
};
