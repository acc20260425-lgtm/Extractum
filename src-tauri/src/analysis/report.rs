use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_effective_model, resolve_model_input_token_limit_for_backend,
    resolve_profile_for_backend, ResolvedLlmProfile,
};

use super::corpus::{
    preflight_analysis_run, preflight_limit_error, resolve_analysis_sources, AnalysisRunPreflight,
    AnalysisRunPreflightLimits, AnalysisSourceResolutionError, CorpusLoadRequest,
    YoutubeCorpusMode,
};
use super::events::emit_analysis_event;
use super::models::{AnalysisChunkSummaryEvent, AnalysisPromptTemplate, AnalysisRunEvent};
use super::store::{
    fetch_prompt_template, fetch_source_group, find_active_duplicate_run, insert_analysis_run,
    set_run_status, AnalysisRunInsert, DuplicateRunLookup,
};
use super::trace::{build_trace_data, compress_trace_data};
use super::{
    now_secs, AnalysisState, ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED,
    ANALYSIS_STATUS_RUNNING, TEMPLATE_KIND_REPORT,
};

mod capture;
mod lifecycle;
mod phases;
mod requests;

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

pub(crate) struct StartAnalysisReportRequest {
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: String,
    pub(crate) prompt_template_id: i64,
    pub(crate) model_override: Option<String>,
    pub(crate) profile_id: Option<String>,
    pub(crate) youtube_corpus_mode: Option<String>,
    pub(crate) include_migrated_history: bool,
}

pub(crate) fn resolve_analysis_telegram_history_scope(
    include_migrated_history: bool,
    source_type: &str,
) -> AppResult<(&'static str, bool)> {
    if include_migrated_history && source_type != crate::sources::TELEGRAM_SOURCE_TYPE {
        return Err(AppError::validation(
            "Migrated historical scope can be included only for Telegram analysis",
        ));
    }
    if include_migrated_history {
        return Ok((
            crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED,
            true,
        ));
    }
    Ok((
        crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT,
        false,
    ))
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

struct ReportRunInput {
    run_id: i64,
    scope_label: String,
    corpus_request: CorpusLoadRequest,
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
    if preflight.message_count == 0 {
        return Err(AppError::validation(
            "No synced source documents were found for the selected analysis scope and period",
        ));
    }

    if let Some(error) = preflight_limit_error(preflight) {
        return Err(AppError::validation(error));
    }

    Ok(())
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

    RunEvent::new(run_id, "started", "load_items")
        .message(format!(
            "Preflight passed: {} documents, {} estimated chunks, {} estimated input characters.",
            input.preflight.message_count,
            input.preflight.estimated_chunks,
            input.preflight.estimated_input_chars
        ))
        .emit(&handle);

    let corpus =
        capture_report_corpus(&pool, run_id, &input.scope_label, &input.corpus_request).await?;
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
        resolve_model_input_token_limit_for_backend(&resolved_profile, &effective_model).await;
    let chunk_target_chars = chunk_target_chars_for_model_input_limit(model_input_token_limit);
    let youtube_corpus_mode = YoutubeCorpusMode::from_wire(youtube_corpus_mode.as_deref())
        .map_err(AppError::validation)?;

    let (scope_type, resolved_source_id, resolved_group_id, resolved_project_id, scope_label) =
        if let Some(source_id) = source_id {
            let source_exists =
                sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)")
                    .bind(source_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(AppError::database)?;
            if source_exists == 0 {
                return Err(AppError::not_found(format!("Source {source_id} not found")));
            }

            let source_title =
                sqlx::query_scalar::<_, Option<String>>("SELECT title FROM sources WHERE id = ?")
                    .bind(source_id)
                    .fetch_optional(&pool)
                    .await
                    .map_err(AppError::database)?
                    .flatten()
                    .filter(|title| !title.trim().is_empty())
                    .unwrap_or_else(|| format!("Source {source_id}"));

            (
                ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
                Some(source_id),
                None,
                None,
                source_title,
            )
        } else if let Some(group_id) = source_group_id {
            let group = fetch_source_group(&pool, group_id).await?.ok_or_else(|| {
                AppError::not_found(format!("Analysis source group {group_id} not found"))
            })?;

            if group.members.is_empty() {
                return Err(AppError::validation(
                    "The selected source group does not contain any sources",
                ));
            }

            (
                ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
                None,
                Some(group.id),
                None,
                group.name.clone(),
            )
        } else {
            let project_id = project_id.expect("validated project_id");
            let project = crate::projects::get_project_in_pool(&pool, project_id)
                .await?
                .ok_or_else(|| AppError::not_found(format!("Project {project_id} not found")))?;
            let source_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM project_sources WHERE project_id = ?")
                    .bind(project_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(AppError::database)?;
            if source_count == 0 {
                return Err(AppError::validation("Project does not contain any sources"));
            }

            (
                ANALYSIS_SCOPE_TYPE_PROJECT,
                None,
                None,
                Some(project.id),
                project.name.clone(),
            )
        };

    let resolved_sources = resolve_analysis_sources(
        &pool,
        resolved_source_id,
        resolved_group_id,
        resolved_project_id,
    )
    .await
    .map_err(AnalysisSourceResolutionError::into_app_error)?;
    let (telegram_history_scope, include_migrated_history) =
        resolve_analysis_telegram_history_scope(
            include_migrated_history,
            &resolved_sources.source_type,
        )?;
    let corpus_request = CorpusLoadRequest {
        source_type: resolved_sources.source_type.clone(),
        source_ids: resolved_sources.source_ids.clone(),
        period_from,
        period_to,
        youtube_corpus_mode,
        include_migrated_history,
    };

    let preflight = preflight_analysis_run(
        &pool,
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
            source_id: resolved_source_id,
            source_group_id: resolved_group_id,
            project_id: resolved_project_id,
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
            source_id: resolved_source_id,
            source_group_id: resolved_group_id,
            project_id: resolved_project_id,
            period_from,
            period_to,
            output_language: &output_language,
            prompt_template: &prompt_template,
            provider_profile: resolved_profile.profile_id(),
            provider: resolved_profile.provider().as_str(),
            model: &effective_model,
            youtube_corpus_mode,
            telegram_history_scope,
            scope_label_snapshot: Some(&scope_label),
        },
    )
    .await?;

    state.insert_active_report_run(run_id).await;

    let app_handle = handle.clone();
    tokio::spawn(async move {
        match run_report_pipeline(
            app_handle.clone(),
            ReportRunInput {
                run_id,
                scope_label,
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
        app_handle
            .state::<AnalysisState>()
            .remove_active_report_run(run_id)
            .await;
    });

    Ok(run_id)
}

#[cfg(test)]
mod tests;
