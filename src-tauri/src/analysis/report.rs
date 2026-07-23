use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::llm::resolve_profile_for_backend;
use extractum_core::error::{AppError, AppResult};
use extractum_llm::{resolve_effective_model, resolve_model_input_token_limit, ResolvedLlmProfile};

use super::corpus::{
    preflight_analysis_corpus as preflight_analysis_run, preflight_limit_error,
    resolve_analysis_sources, resolve_analysis_telegram_history_scope, AnalysisCorpusRequest,
    AnalysisRunPreflight, AnalysisRunPreflightLimits, AnalysisSourceResolutionError,
    AppAnalysisCorpusReader, YoutubeCorpusMode,
};
use super::domain::{
    now_secs, ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED,
    ANALYSIS_STATUS_RUNNING, TEMPLATE_KIND_REPORT,
};
use super::events::emit_analysis_event;
use super::models::{
    AnalysisChunkSummaryEvent, AnalysisPromptTemplate, AnalysisRunEvent, AnalysisScopeKind,
    ResolvedAnalysisScope,
};
use super::store::{
    capture_run_snapshot, fetch_prompt_template, find_active_duplicate_run, insert_analysis_run,
    sanitize_snapshot_error, set_run_status, AnalysisRunInsert, DuplicateRunLookup,
};
use super::trace::{build_trace_data, compress_trace_data};
use super::AnalysisState;

mod capture;
mod lifecycle;
mod phases;
mod requests;

#[cfg(test)]
use self::capture::capture_report_corpus;

pub use self::lifecycle::cleanup_interrupted_analysis_runs;
#[rustfmt::skip]
#[cfg(test)] use self::lifecycle::request_analysis_run_cancel_for_pool;
use self::lifecycle::{cancel_run, fail_capture_run, fail_run};
#[allow(unused_imports)]
pub(crate) use self::lifecycle::{mark_interrupted_analysis_runs, request_analysis_run_cancel};
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
const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";

pub(crate) struct StartAnalysisReportRequest {
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReportRunError {
    Failed(String),
    CaptureFailed(String),
    Cancelled(String),
}

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

    pub(super) fn emit(self, handle: &AppHandle) {
        emit_analysis_event(handle, &self.event);
    }
}

fn started_load_items_event(run_id: i64, message: String) -> RunEvent {
    RunEvent::new(run_id, "started", "load_items").message(message)
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

async fn load_execution_corpus<F>(
    reader: &dyn super::corpus::AnalysisCorpusReader,
    request: &AnalysisCorpusRequest,
    preflight: &AnalysisRunPreflight,
    mut on_started: F,
) -> Result<Vec<super::corpus::AnalysisCorpusMessage>, ReportRunError>
where
    F: FnMut(String),
{
    on_started(format!(
        "Preflight passed: {} documents, {} estimated chunks, {} estimated input characters.",
        preflight.message_count(),
        preflight.estimated_chunks(),
        preflight.estimated_input_chars()
    ));
    let corpus = reader.load_corpus(request.clone()).await.map_err(|error| {
        ReportRunError::CaptureFailed(sanitize_snapshot_error(
            "Corpus preload failed",
            &error.to_string(),
        ))
    })?;
    if corpus.is_empty() {
        return Err(ReportRunError::CaptureFailed(
            SNAPSHOT_CAPTURE_FAILED_MESSAGE.to_string(),
        ));
    }
    Ok(corpus)
}

mod execution_capture {
    use super::*;

    pub(super) struct CaptureReportCorpusInput<'a, F> {
        pub(super) run_id: i64,
        pub(super) scope_label: &'a str,
        pub(super) reader: &'a dyn super::super::corpus::AnalysisCorpusReader,
        pub(super) request: &'a AnalysisCorpusRequest,
        pub(super) preflight: &'a AnalysisRunPreflight,
        pub(super) on_started: F,
    }

    pub(super) async fn capture_report_corpus<F>(
        pool: &sqlx::SqlitePool,
        input: CaptureReportCorpusInput<'_, F>,
    ) -> Result<Vec<super::super::corpus::AnalysisCorpusMessage>, ReportRunError>
    where
        F: FnMut(String),
    {
        let corpus = load_execution_corpus(
            input.reader,
            input.request,
            input.preflight,
            input.on_started,
        )
        .await?;
        capture_run_snapshot(pool, input.run_id, input.scope_label, &corpus)
            .await
            .map_err(|error| {
                ReportRunError::CaptureFailed(sanitize_snapshot_error(
                    SNAPSHOT_CAPTURE_FAILED_MESSAGE,
                    &error.to_string(),
                ))
            })
    }
}

async fn run_report_pipeline(
    handle: AppHandle,
    input: ReportRunInput,
) -> Result<(), ReportRunError> {
    let run_id = input.run_id;

    if handle
        .state::<AnalysisState>()
        .is_report_run_cancelled(run_id)
        .await
    {
        return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
    }

    let pool = get_pool(&handle)
        .await
        .map_err(|error| ReportRunError::Failed(error.to_string()))?;
    set_run_status(
        &pool,
        run_id,
        ANALYSIS_STATUS_RUNNING,
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|error| ReportRunError::Failed(error.to_string()))?;

    let reader = AppAnalysisCorpusReader::new(pool.clone());
    let capture = execution_capture::CaptureReportCorpusInput {
        run_id,
        scope_label: input.scope.scope_label_snapshot(),
        reader: &reader,
        request: &input.corpus_request,
        preflight: &input.preflight,
        on_started: |message| started_load_items_event(run_id, message).emit(&handle),
    };
    let corpus = execution_capture::capture_report_corpus(&pool, capture).await?;
    if handle
        .state::<AnalysisState>()
        .is_report_run_cancelled(run_id)
        .await
    {
        return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
    }

    RunEvent::new(run_id, "progress", "chunking")
        .message(format!(
            "Loaded {} source documents. Preparing chunks...",
            corpus.len()
        ))
        .emit(&handle);

    let chunks = chunk_messages(&corpus, input.chunk_target_chars);
    let ctx = ReportPipelineContext {
        handle,
        pool,
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

async fn await_report_terminal_and_cleanup<F, T>(
    state: &AnalysisState,
    run_id: i64,
    terminal: F,
) -> T
where
    F: std::future::Future<Output = T>,
{
    let result = terminal.await;
    state.remove_active_report_run(run_id).await;
    result
}

pub(crate) async fn start_analysis_report_run(
    handle: AppHandle,
    state: &AnalysisState,
    request: StartAnalysisReportRequest,
) -> AppResult<i64> {
    let StartAnalysisReportRequest {
        source_id,
        source_group_id,
        project_id,
        period_from,
        period_to,
        output_language,
        prompt_template_id,
        model_override,
        profile_id,
        youtube_corpus_mode,
        include_migrated_history,
    } = request;

    if period_from > period_to {
        return Err(AppError::validation(
            "period_from must be less than or equal to period_to",
        ));
    }

    let output_language = output_language.trim().to_string();
    if output_language.is_empty() {
        return Err(AppError::validation("Output language cannot be empty"));
    }

    let selected_count = [
        source_id.is_some(),
        source_group_id.is_some(),
        project_id.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if selected_count != 1 {
        return Err(AppError::validation("Select exactly one analysis scope"));
    }

    let pool = get_pool(&handle).await?;
    let prompt_template = fetch_prompt_template(&pool, prompt_template_id).await?;
    if prompt_template.template_kind != TEMPLATE_KIND_REPORT {
        return Err(AppError::validation(
            "Selected prompt template is not a report template",
        ));
    }

    let resolved_profile = resolve_profile_for_backend(&handle, profile_id.as_deref()).await?;
    let effective_model = resolve_effective_model(&resolved_profile, model_override.as_deref())?;
    let model_input_token_limit =
        resolve_model_input_token_limit(&resolved_profile, &effective_model).await;
    let chunk_target_chars = chunk_target_chars_for_model_input_limit(model_input_token_limit);
    let youtube_corpus_mode = YoutubeCorpusMode::from_wire(youtube_corpus_mode.as_deref())
        .map_err(AppError::validation)?;

    let resolved_sources = resolve_analysis_sources(&pool, source_id, source_group_id, project_id)
        .await
        .map_err(AnalysisSourceResolutionError::into_app_error)?;
    let resolved_scope = resolved_sources.into_scope();
    let scope_type = match resolved_scope.scope_kind() {
        AnalysisScopeKind::SingleSource => ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
        AnalysisScopeKind::SourceGroup => ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
        AnalysisScopeKind::Project => ANALYSIS_SCOPE_TYPE_PROJECT,
    };
    let source_kind = resolved_scope.source_kind();
    let (telegram_history_scope, include_migrated_history) =
        resolve_analysis_telegram_history_scope(include_migrated_history, source_kind)?;
    let corpus_request = AnalysisCorpusRequest::new(
        source_kind,
        resolved_scope.source_ids().to_vec(),
        period_from,
        period_to,
        youtube_corpus_mode,
        include_migrated_history,
    )?;

    let reader = AppAnalysisCorpusReader::new(pool.clone());
    let preflight = preflight_analysis_run(
        &reader,
        &corpus_request,
        chunk_target_chars,
        AnalysisRunPreflightLimits::default(),
    )
    .await?;

    validate_report_preflight(&preflight)?;

    if let Some(existing_run_id) = find_active_duplicate_run(
        &pool,
        &DuplicateRunLookup {
            scope_type,
            source_id: resolved_scope.source_id(),
            source_group_id: resolved_scope.source_group_id(),
            project_id: resolved_scope.project_id(),
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
            &pool,
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
        &pool,
        &AnalysisRunInsert {
            scope_type,
            source_id: resolved_scope.source_id(),
            source_group_id: resolved_scope.source_group_id(),
            project_id: resolved_scope.project_id(),
            period_from,
            period_to,
            output_language: &output_language,
            prompt_template: &prompt_template,
            provider_profile: resolved_profile.profile_id(),
            provider: resolved_profile.provider().as_str(),
            model: &effective_model,
            youtube_corpus_mode,
            telegram_history_scope,
            scope_label_snapshot: Some(resolved_scope.scope_label_snapshot()),
        },
    )
    .await?;

    state.insert_active_report_run(run_id).await;

    let app_handle = handle.clone();
    tokio::spawn(async move {
        let state = app_handle.state::<AnalysisState>();
        await_report_terminal_and_cleanup(state.inner(), run_id, async {
            match run_report_pipeline(
                app_handle.clone(),
                ReportRunInput {
                    run_id,
                    scope: resolved_scope,
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
            )
            .await
            {
                Ok(()) => {}
                Err(ReportRunError::Failed(error)) => fail_run(&app_handle, run_id, error).await,
                Err(ReportRunError::CaptureFailed(error)) => {
                    fail_capture_run(&app_handle, run_id, error).await
                }
                Err(ReportRunError::Cancelled(message)) => {
                    cancel_run(&app_handle, run_id, message).await
                }
            }
        })
        .await;
    });

    Ok(run_id)
}

#[cfg(test)]
mod tests;
