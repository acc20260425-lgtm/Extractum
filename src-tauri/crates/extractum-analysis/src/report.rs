use std::sync::Arc;

use sqlx::SqlitePool;

use extractum_core::error::{AppError, AppResult};
use extractum_llm::{LlmSchedulerState, ResolvedLlmProfile};

use super::corpus::{
    preflight_analysis_corpus as preflight_analysis_run, preflight_limit_error,
    resolve_analysis_telegram_history_scope, AnalysisCorpusReader, AnalysisCorpusRequest,
    AnalysisRunPreflight, AnalysisRunPreflightLimits, YoutubeCorpusMode,
};
use super::domain::{
    now_secs, ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED,
    ANALYSIS_STATUS_RUNNING, TEMPLATE_KIND_REPORT,
};
use super::models::{
    AnalysisChunkSummaryEvent, AnalysisEventSink, AnalysisPromptTemplate, AnalysisRunEvent,
    AnalysisScopeKind, ResolvedAnalysisScope,
};
use super::store::{
    fetch_prompt_template, find_active_duplicate_run, insert_analysis_run, set_run_status,
    AnalysisRunInsert, DuplicateRunLookup,
};
use super::trace::{build_trace_data, compress_trace_data};
use super::AnalysisState;

#[path = "report/capture.rs"]
mod capture;
mod lifecycle;
#[path = "report/phases.rs"]
mod phases;
#[path = "report/requests.rs"]
mod requests;

pub use self::capture::capture_analysis_corpus;
use self::lifecycle::{
    mark_interrupted_analysis_runs as mark_interrupted_analysis_runs_in_store,
    request_analysis_run_cancel_for_pool,
};
#[rustfmt::skip]
#[cfg(test)] use self::phases::{finish_map_phase, run_analysis_step_with_cancel};
use self::phases::{run_map_phase, run_reduce_phase, ReportPipelineContext};
#[rustfmt::skip]
#[cfg(test)] use self::requests::{
    build_map_request, build_reduce_request, extract_json_payload, parse_chunk_summary,
    ReduceRequestParams,
};
use self::requests::{chunk_messages, chunk_target_chars_for_model_input_limit};

pub(super) const INTERRUPTED_RUN_MESSAGE: &str =
    "Analysis run was interrupted when the app was restarted.";
const CANCELLED_RUN_MESSAGE: &str = "Analysis run cancelled.";

pub async fn mark_interrupted_analysis_runs(pool: &SqlitePool) -> AppResult<()> {
    mark_interrupted_analysis_runs_in_store(pool).await
}

pub async fn request_analysis_run_cancel_in_pool(
    pool: &SqlitePool,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    sink: &dyn AnalysisEventSink,
    run_id: i64,
) -> AppResult<()> {
    let status = request_analysis_run_cancel_for_pool(pool, state, scheduler, run_id).await?;
    RunEvent::new(run_id, "progress", &status)
        .message("Cancelling analysis run...".to_string())
        .publish(sink);
    Ok(())
}

pub struct StartAnalysisReportRequest {
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template_id: i64,
    model_override: Option<String>,
    profile_id: Option<String>,
    youtube_corpus_mode: Option<String>,
    include_migrated_history: bool,
}

impl StartAnalysisReportRequest {
    fn validated_output_language(
        period_from: i64,
        period_to: i64,
        output_language: String,
    ) -> AppResult<String> {
        if period_from > period_to {
            return Err(AppError::validation(
                "period_from must be less than or equal to period_to",
            ));
        }

        let output_language = output_language.trim().to_string();
        if output_language.is_empty() {
            return Err(AppError::validation("Output language cannot be empty"));
        }

        Ok(output_language)
    }

    #[expect(clippy::too_many_arguments)]
    pub fn from_command(
        source_id: Option<i64>,
        source_group_id: Option<i64>,
        project_id: Option<i64>,
        period_from: i64,
        period_to: i64,
        output_language: String,
        prompt_template_id: i64,
        model_override: Option<String>,
        profile_id: Option<String>,
        youtube_corpus_mode: Option<String>,
        include_migrated_history: bool,
    ) -> AppResult<Self> {
        let output_language =
            Self::validated_output_language(period_from, period_to, output_language)?;
        match (source_id, source_group_id, project_id) {
            (Some(source_id), None, None) => Self::for_source(
                source_id,
                period_from,
                period_to,
                output_language,
                prompt_template_id,
                model_override,
                profile_id,
                youtube_corpus_mode,
                include_migrated_history,
            ),
            (None, Some(source_group_id), None) => Self::for_source_group(
                source_group_id,
                period_from,
                period_to,
                output_language,
                prompt_template_id,
                model_override,
                profile_id,
                youtube_corpus_mode,
                include_migrated_history,
            ),
            (None, None, Some(project_id)) => Self::for_project(
                project_id,
                period_from,
                period_to,
                output_language,
                prompt_template_id,
                model_override,
                profile_id,
                youtube_corpus_mode,
                include_migrated_history,
            ),
            _ => Err(AppError::validation("Select exactly one analysis scope")),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn for_source(
        source_id: i64,
        period_from: i64,
        period_to: i64,
        output_language: String,
        prompt_template_id: i64,
        model_override: Option<String>,
        profile_id: Option<String>,
        youtube_corpus_mode: Option<String>,
        include_migrated_history: bool,
    ) -> AppResult<Self> {
        let output_language =
            Self::validated_output_language(period_from, period_to, output_language)?;
        Ok(Self {
            source_id: Some(source_id),
            source_group_id: None,
            project_id: None,
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            model_override,
            profile_id,
            youtube_corpus_mode,
            include_migrated_history,
        })
    }

    #[expect(clippy::too_many_arguments)]
    pub fn for_source_group(
        source_group_id: i64,
        period_from: i64,
        period_to: i64,
        output_language: String,
        prompt_template_id: i64,
        model_override: Option<String>,
        profile_id: Option<String>,
        youtube_corpus_mode: Option<String>,
        include_migrated_history: bool,
    ) -> AppResult<Self> {
        let output_language =
            Self::validated_output_language(period_from, period_to, output_language)?;
        Ok(Self {
            source_id: None,
            source_group_id: Some(source_group_id),
            project_id: None,
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            model_override,
            profile_id,
            youtube_corpus_mode,
            include_migrated_history,
        })
    }

    #[expect(clippy::too_many_arguments)]
    pub fn for_project(
        project_id: i64,
        period_from: i64,
        period_to: i64,
        output_language: String,
        prompt_template_id: i64,
        model_override: Option<String>,
        profile_id: Option<String>,
        youtube_corpus_mode: Option<String>,
        include_migrated_history: bool,
    ) -> AppResult<Self> {
        let output_language =
            Self::validated_output_language(period_from, period_to, output_language)?;
        Ok(Self {
            source_id: None,
            source_group_id: None,
            project_id: Some(project_id),
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            model_override,
            profile_id,
            youtube_corpus_mode,
            include_migrated_history,
        })
    }
}

pub struct AnalysisReportPreparationTicket {
    request: StartAnalysisReportRequest,
    prompt_template: AnalysisPromptTemplate,
}

impl AnalysisReportPreparationTicket {
    pub fn requested_profile_id(&self) -> Option<&str> {
        self.request.profile_id.as_deref()
    }

    pub fn model_override(&self) -> Option<&str> {
        self.request.model_override.as_deref()
    }

    pub fn resolve_youtube_corpus_mode(self) -> AppResult<AnalysisReportScopeTicket> {
        let scope = match (
            self.request.source_id,
            self.request.source_group_id,
            self.request.project_id,
        ) {
            (Some(scope_id), None, None) => (AnalysisScopeKind::SingleSource, scope_id),
            (None, Some(scope_id), None) => (AnalysisScopeKind::SourceGroup, scope_id),
            (None, None, Some(scope_id)) => (AnalysisScopeKind::Project, scope_id),
            _ => return Err(AppError::validation("Select exactly one analysis scope")),
        };
        let youtube_corpus_mode =
            YoutubeCorpusMode::from_wire(self.request.youtube_corpus_mode.as_deref())
                .map_err(AppError::validation)?;

        Ok(AnalysisReportScopeTicket {
            scope_kind: scope.0,
            scope_id: scope.1,
            period_from: self.request.period_from,
            period_to: self.request.period_to,
            output_language: self.request.output_language,
            prompt_template: self.prompt_template,
            model_override: self.request.model_override,
            youtube_corpus_mode,
            include_migrated_history: self.request.include_migrated_history,
        })
    }
}

pub struct AnalysisReportScopeTicket {
    scope_kind: AnalysisScopeKind,
    scope_id: i64,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template: AnalysisPromptTemplate,
    model_override: Option<String>,
    youtube_corpus_mode: YoutubeCorpusMode,
    include_migrated_history: bool,
}

impl AnalysisReportScopeTicket {
    pub fn scope_kind(&self) -> AnalysisScopeKind {
        self.scope_kind
    }

    pub fn scope_id(&self) -> i64 {
        self.scope_id
    }

    pub fn youtube_corpus_mode(&self) -> YoutubeCorpusMode {
        self.youtube_corpus_mode
    }
}

pub async fn prepare_analysis_report(
    pool: &SqlitePool,
    request: StartAnalysisReportRequest,
) -> AppResult<AnalysisReportPreparationTicket> {
    let prompt_template = fetch_prompt_template(pool, request.prompt_template_id).await?;
    if prompt_template.template_kind != TEMPLATE_KIND_REPORT {
        return Err(AppError::validation(
            "Selected prompt template is not a report template",
        ));
    }

    Ok(AnalysisReportPreparationTicket {
        request,
        prompt_template,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalysisExecutionError {
    Cancelled(String),
    CaptureFailed(String),
    Failed(String),
}

type ReportRunError = AnalysisExecutionError;

pub struct AnalysisReportExecutionTicket {
    input: ReportRunInput,
}

impl AnalysisReportExecutionTicket {
    pub fn run_id(&self) -> i64 {
        self.input.run_id
    }
}

// This public cross-crate entry point retains its established call shape in Wave 0.
#[allow(clippy::too_many_arguments)]
pub async fn prepare_analysis_report_execution(
    pool: &SqlitePool,
    state: &AnalysisState,
    reader: &dyn AnalysisCorpusReader,
    preparation: AnalysisReportScopeTicket,
    scope: ResolvedAnalysisScope,
    resolved_profile: ResolvedLlmProfile,
    effective_model: String,
    model_input_token_limit: Option<usize>,
) -> AppResult<AnalysisReportExecutionTicket> {
    let AnalysisReportScopeTicket {
        period_from,
        period_to,
        output_language,
        prompt_template,
        model_override,
        youtube_corpus_mode,
        include_migrated_history,
        ..
    } = preparation;
    let scope_type = match scope.scope_kind() {
        AnalysisScopeKind::SingleSource => ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
        AnalysisScopeKind::SourceGroup => ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
        AnalysisScopeKind::Project => ANALYSIS_SCOPE_TYPE_PROJECT,
    };
    let (telegram_history_scope, include_migrated_history) =
        resolve_analysis_telegram_history_scope(include_migrated_history, scope.source_kind())?;
    let corpus_request = AnalysisCorpusRequest::new(
        scope.source_kind(),
        scope.source_ids().to_vec(),
        period_from,
        period_to,
        youtube_corpus_mode,
        include_migrated_history,
    )?;
    let chunk_target_chars = chunk_target_chars_for_model_input_limit(model_input_token_limit);
    let preflight = preflight_analysis_run(
        reader,
        &corpus_request,
        chunk_target_chars,
        AnalysisRunPreflightLimits::default(),
    )
    .await?;
    validate_report_preflight(&preflight)?;

    if let Some(existing_run_id) = find_active_duplicate_run(
        pool,
        &DuplicateRunLookup {
            scope_type,
            source_id: scope.source_id(),
            source_group_id: scope.source_group_id(),
            project_id: scope.project_id(),
            period_from,
            period_to,
            output_language: &output_language,
            prompt_template_id: prompt_template.id,
            provider_profile: resolved_profile.profile_id(),
            model: &effective_model,
            youtube_corpus_mode,
            telegram_history_scope,
        },
    )
    .await?
    {
        let active_run_ids = state.active_report_run_ids().await;
        if active_run_ids.contains(&existing_run_id) {
            return Err(AppError::conflict(format!(
                "An identical analysis report is already queued or running (run {existing_run_id})"
            )));
        }

        set_run_status(
            pool,
            existing_run_id,
            ANALYSIS_STATUS_CANCELLED,
            None,
            None,
            Some(INTERRUPTED_RUN_MESSAGE),
            Some(now_secs()),
        )
        .await?;
    }

    let run_id = insert_analysis_run(
        pool,
        &AnalysisRunInsert {
            scope_type,
            source_id: scope.source_id(),
            source_group_id: scope.source_group_id(),
            project_id: scope.project_id(),
            period_from,
            period_to,
            output_language: &output_language,
            prompt_template: &prompt_template,
            provider_profile: resolved_profile.profile_id(),
            provider: resolved_profile.provider().as_str(),
            model: &effective_model,
            youtube_corpus_mode,
            telegram_history_scope,
            scope_label_snapshot: Some(scope.scope_label_snapshot()),
        },
    )
    .await?;
    state.insert_active_report_run(run_id).await;

    Ok(AnalysisReportExecutionTicket {
        input: ReportRunInput {
            run_id,
            scope,
            corpus_request,
            period_from,
            period_to,
            output_language,
            prompt_template,
            model_override,
            resolved_profile,
            chunk_target_chars,
            preflight,
        },
    })
}

pub async fn execute_analysis_report(
    pool: &SqlitePool,
    state: &AnalysisState,
    scheduler: Arc<LlmSchedulerState>,
    reader: &dyn AnalysisCorpusReader,
    sink: Arc<dyn AnalysisEventSink>,
    ticket: AnalysisReportExecutionTicket,
) -> Result<(), AnalysisExecutionError> {
    let input = ticket.input;
    if state.is_report_run_cancelled(input.run_id).await {
        return Err(AnalysisExecutionError::Cancelled(
            CANCELLED_RUN_MESSAGE.to_string(),
        ));
    }
    run_report_pipeline_with_capabilities(pool, state, scheduler, reader, sink, input).await
}

pub async fn finalize_analysis_report_execution(
    pool: Option<&SqlitePool>,
    state: &AnalysisState,
    sink: &dyn AnalysisEventSink,
    run_id: i64,
    outcome: Result<(), AnalysisExecutionError>,
) {
    match outcome {
        Ok(()) => {}
        Err(AnalysisExecutionError::Failed(error)) => {
            let sanitized_error =
                super::store::sanitize_provider_error("Report run failed", &error);
            if let Some(pool) = pool {
                let _ = set_run_status(
                    pool,
                    run_id,
                    super::domain::ANALYSIS_STATUS_FAILED,
                    None,
                    None,
                    Some(&sanitized_error),
                    Some(now_secs()),
                )
                .await;
            }
            RunEvent::new(run_id, "failed", "persist")
                .message("Report run failed.".to_string())
                .error(sanitized_error)
                .publish(sink);
        }
        Err(AnalysisExecutionError::CaptureFailed(error)) => {
            self::lifecycle::persist_capture_failure_event(pool, run_id, &error, now_secs())
                .await
                .publish(sink);
        }
        Err(AnalysisExecutionError::Cancelled(message)) => {
            if let Some(pool) = pool {
                let _ = set_run_status(
                    pool,
                    run_id,
                    ANALYSIS_STATUS_CANCELLED,
                    None,
                    None,
                    Some(&message),
                    Some(now_secs()),
                )
                .await;
            }
            RunEvent::new(run_id, "cancelled", "persist")
                .message(message)
                .publish(sink);
        }
    }
    state.remove_active_report_run(run_id).await;
}

#[cfg(test)]
mod tests;

pub(super) struct RunEvent {
    event: AnalysisRunEvent,
}

impl RunEvent {
    pub(super) fn new(run_id: i64, kind: &str, phase: &str) -> Self {
        Self {
            event: AnalysisRunEvent {
                run_id,
                request_id: None,
                kind: kind.to_string(),
                phase: phase.to_string(),
                queue_position: None,
                message: None,
                progress_current: None,
                progress_total: None,
                delta: None,
                chunk_summary: None,
                error: None,
            },
        }
    }

    fn request_id(mut self, request_id: String) -> Self {
        self.event.request_id = Some(request_id);
        self
    }

    fn queue_position(mut self, queue_position: usize) -> Self {
        self.event.queue_position = Some(queue_position);
        self
    }

    pub(super) fn message(mut self, message: String) -> Self {
        self.event.message = Some(message);
        self
    }

    fn progress(mut self, current: i64, total: i64) -> Self {
        self.event.progress_current = Some(current);
        self.event.progress_total = Some(total);
        self
    }

    fn delta(mut self, delta: String) -> Self {
        self.event.delta = Some(delta);
        self
    }

    fn chunk_summary(mut self, chunk_summary: AnalysisChunkSummaryEvent) -> Self {
        self.event.chunk_summary = Some(chunk_summary);
        self
    }

    pub(super) fn error(mut self, error: String) -> Self {
        self.event.error = Some(error);
        self
    }

    pub(super) fn publish(self, sink: &dyn AnalysisEventSink) {
        sink.publish_run(self.event);
    }
}

fn started_load_items_event(run_id: i64, preflight: &AnalysisRunPreflight) -> RunEvent {
    RunEvent::new(run_id, "started", "load_items").message(format!(
        "Preflight passed: {} documents, {} estimated chunks, {} estimated input characters.",
        preflight.message_count(),
        preflight.estimated_chunks(),
        preflight.estimated_input_chars()
    ))
}

struct ReportRunInput {
    run_id: i64,
    scope: ResolvedAnalysisScope,
    corpus_request: AnalysisCorpusRequest,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template: AnalysisPromptTemplate,
    model_override: Option<String>,
    resolved_profile: ResolvedLlmProfile,
    chunk_target_chars: usize,
    preflight: AnalysisRunPreflight,
}

fn validate_report_preflight(preflight: &AnalysisRunPreflight) -> AppResult<()> {
    if preflight.message_count() == 0 {
        return Err(AppError::validation(
            "No synced source documents were found for the selected analysis scope and period",
        ));
    }

    if let Some(error) = preflight_limit_error(preflight) {
        return Err(AppError::validation(error));
    }

    Ok(())
}

async fn run_report_pipeline_with_capabilities(
    pool: &SqlitePool,
    state: &AnalysisState,
    scheduler: Arc<LlmSchedulerState>,
    reader: &dyn AnalysisCorpusReader,
    sink: Arc<dyn AnalysisEventSink>,
    input: ReportRunInput,
) -> Result<(), ReportRunError> {
    let run_id = input.run_id;
    set_run_status(
        pool,
        run_id,
        ANALYSIS_STATUS_RUNNING,
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|error| ReportRunError::Failed(error.to_string()))?;

    started_load_items_event(run_id, &input.preflight).publish(sink.as_ref());
    let corpus = capture_analysis_corpus(
        pool,
        reader,
        run_id,
        input.scope.scope_label_snapshot(),
        &input.corpus_request,
    )
    .await?;
    if state.is_report_run_cancelled(run_id).await {
        return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
    }

    RunEvent::new(run_id, "progress", "chunking")
        .message(format!(
            "Loaded {} source documents. Preparing chunks...",
            corpus.len()
        ))
        .publish(sink.as_ref());

    let chunks = chunk_messages(&corpus, input.chunk_target_chars);
    let ctx = ReportPipelineContext {
        pool: pool.clone(),
        state,
        scheduler,
        sink,
        resolved_profile: input.resolved_profile.clone(),
        run_id,
    };

    ctx.ensure_not_cancelled().await?;
    let chunk_summaries = run_map_phase(&ctx, chunks).await?;
    ctx.ensure_not_cancelled().await?;

    let reduce_result = run_reduce_phase(&ctx, &input, &chunk_summaries).await?;
    ctx.ensure_not_cancelled().await?;
    let trace_data = build_trace_data(&reduce_result.completion.text, &corpus);
    let compressed_trace = compress_trace_data(&trace_data)
        .map_err(|error| ReportRunError::Failed(error.to_string()))?;

    ctx.emit(
        RunEvent::new(run_id, "progress", "persist")
            .request_id(reduce_result.request_id.clone())
            .message("Saving report...".to_string()),
    );

    set_run_status(
        &ctx.pool,
        run_id,
        ANALYSIS_STATUS_COMPLETED,
        Some(&reduce_result.completion.text),
        Some(&compressed_trace),
        None,
        Some(now_secs()),
    )
    .await
    .map_err(|error| ReportRunError::Failed(error.to_string()))?;

    ctx.emit(
        RunEvent::new(run_id, "completed", "persist")
            .request_id(reduce_result.request_id)
            .message(format!(
                "Report completed with {} cited references.",
                trace_data.refs.len()
            )),
    );

    Ok(())
}
