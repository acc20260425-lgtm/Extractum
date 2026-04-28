use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tokio::task::JoinSet;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_effective_model, resolve_profile_for_backend, run_llm_collect_with_profile,
    run_llm_stream_with_profile, LlmChatRequest, LlmMessage, LlmRequestError, LlmRequestKind,
    LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState,
};

use super::corpus::load_corpus_messages;
use super::models::{
    AnalysisChunkSummaryEvent, AnalysisPromptTemplate, AnalysisRunEvent, ChunkSummary,
    CorpusMessage,
};
use super::store::{
    fetch_prompt_template, fetch_run_row, fetch_source_group, find_active_duplicate_run,
    insert_analysis_run, persist_run_snapshot, set_run_status,
};
use super::trace::{build_trace_data, compress_trace_data, normalize_ref};
use super::{
    emit_analysis_event, now_secs, AnalysisState, ANALYSIS_CHUNK_TARGET_CHARS,
    ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
    ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED, ANALYSIS_STATUS_FAILED,
    ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING, TEMPLATE_KIND_REPORT,
};

const INTERRUPTED_RUN_MESSAGE: &str = "Analysis run was interrupted when the app was restarted.";
const CANCELLED_RUN_MESSAGE: &str = "Analysis run cancelled.";

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReportRunError {
    Failed(String),
    Cancelled(String),
}

fn chunk_messages(messages: &[CorpusMessage], max_chars: usize) -> Vec<Vec<CorpusMessage>> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_chars = 0usize;

    for message in messages {
        let estimated_len = message.content.len()
            + message.r#ref.len()
            + message.author.as_deref().unwrap_or("").len()
            + 64;

        if !current.is_empty() && current_chars + estimated_len > max_chars {
            chunks.push(current);
            current = Vec::new();
            current_chars = 0;
        }

        current_chars += estimated_len;
        current.push(message.clone());
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn format_chunk_corpus(messages: &[CorpusMessage]) -> String {
    messages
        .iter()
        .map(|message| {
            format!(
                "[{ref}]\nDate: {published_at}\nAuthor: {author}\nContent:\n{content}",
                ref = message.r#ref,
                published_at = message.published_at,
                author = message.author.as_deref().unwrap_or("unknown"),
                content = message.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

fn build_map_request(
    run_id: i64,
    profile_id: String,
    chunk_index: usize,
    total_chunks: usize,
    messages: &[CorpusMessage],
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!("analysis-map-{run_id}-{chunk_index}-{}", now_secs()),
        profile_id: Some(profile_id),
        model_override: None,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "You analyze Telegram message excerpts. Return a strict JSON object only with keys: summary, topics, notable_points, candidate_refs. Do not wrap JSON in markdown fences. Use only refs that appear in the provided messages.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Chunk {chunk_index} of {total_chunks}.\nSummarize the messages below for later reduction.\n\nMessages:\n\n{}",
                    format_chunk_corpus(messages)
                ),
            },
        ],
    }
}

fn extract_json_payload(text: &str) -> Result<&str, String> {
    let mut search_from = 0usize;
    let mut saw_candidate = false;

    while let Some(relative_start) = text[search_from..].find('{') {
        let start = search_from + relative_start;
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaping = false;

        for (offset, character) in text[start..].char_indices() {
            if in_string {
                if escaping {
                    escaping = false;
                    continue;
                }
                match character {
                    '\\' => escaping = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }

            match character {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        saw_candidate = true;
                        let end = start + offset + character.len_utf8();
                        let candidate = &text[start..end];
                        if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
                            return Ok(candidate);
                        }
                        search_from = start + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if search_from <= start {
            return Err("LLM response contained malformed JSON boundaries".to_string());
        }
    }

    if saw_candidate {
        Err("LLM response did not contain a valid JSON object".to_string())
    } else {
        Err("LLM response did not contain JSON".to_string())
    }
}

fn parse_chunk_summary(text: &str) -> Result<ChunkSummary, String> {
    let payload = extract_json_payload(text)?;
    serde_json::from_str(payload).map_err(|e| format!("Failed to parse chunk summary JSON: {e}"))
}

fn summarize_chunk_for_reduce(summary: &ChunkSummary) -> String {
    let topics = if summary.topics.is_empty() {
        "- none".to_string()
    } else {
        summary
            .topics
            .iter()
            .map(|topic| format!("- {topic}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let notable_points = if summary.notable_points.is_empty() {
        "- none".to_string()
    } else {
        summary
            .notable_points
            .iter()
            .map(|point| format!("- {point}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let refs = if summary.candidate_refs.is_empty() {
        "- none".to_string()
    } else {
        summary
            .candidate_refs
            .iter()
            .filter_map(|candidate| normalize_ref(candidate))
            .map(|r| format!("- {r}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Summary:\n{}\n\nTopics:\n{}\n\nNotable points:\n{}\n\nCandidate refs:\n{}",
        summary.summary.trim(),
        topics,
        notable_points,
        refs
    )
}

fn build_reduce_request(
    run_id: i64,
    profile_id: String,
    scope_label: &str,
    output_language: &str,
    prompt_template: &AnalysisPromptTemplate,
    period_from: i64,
    period_to: i64,
    chunk_summaries: &[ChunkSummary],
    model_override: Option<String>,
) -> LlmChatRequest {
    let combined = chunk_summaries
        .iter()
        .enumerate()
        .map(|(index, summary)| {
            format!(
                "Chunk {} summary\n{}\n",
                index + 1,
                summarize_chunk_for_reduce(summary)
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n\n");

    LlmChatRequest {
        request_id: format!("analysis-reduce-{run_id}-{}", now_secs()),
        profile_id: Some(profile_id),
        model_override,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: format!(
                    "You write grounded markdown reports over already-summarized Telegram messages.\nAnswer in {output_language}.\nUse markdown only.\nEvery important conclusion must cite one or more refs like [s12-m845].\nDo not invent facts beyond the provided chunk summaries."
                ),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Analysis scope: {scope_label}\nPeriod: {period_from} to {period_to}\n\nUser report template:\n{template}\n\nChunk summaries:\n\n{combined}",
                    template = prompt_template.body
                ),
            },
        ],
    }
}

fn emit_run_event(
    handle: &AppHandle,
    run_id: i64,
    request_id: Option<String>,
    kind: &str,
    phase: &str,
    queue_position: Option<usize>,
    message: Option<String>,
    progress_current: Option<i64>,
    progress_total: Option<i64>,
    delta: Option<String>,
    chunk_summary: Option<AnalysisChunkSummaryEvent>,
    error: Option<String>,
) {
    emit_analysis_event(
        handle,
        &AnalysisRunEvent {
            run_id,
            request_id,
            kind: kind.to_string(),
            phase: phase.to_string(),
            queue_position,
            message,
            progress_current,
            progress_total,
            delta,
            chunk_summary,
            error,
        },
    );
}

fn finish_map_phase(
    ordered_summaries: Vec<Option<ChunkSummary>>,
    first_error: Option<ReportRunError>,
) -> Result<Vec<ChunkSummary>, ReportRunError> {
    if let Some(error) = first_error {
        return Err(error);
    }

    ordered_summaries.into_iter().collect::<Option<Vec<_>>>().ok_or_else(|| {
        ReportRunError::Failed("Some chunk summaries were not collected".to_string())
    })
}

async fn ensure_report_not_cancelled(
    handle: &AppHandle,
    run_id: i64,
) -> Result<(), ReportRunError> {
    if handle
        .state::<AnalysisState>()
        .is_report_run_cancelled(run_id)
        .await
    {
        return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
    }

    Ok(())
}

async fn cancel_report_children(handle: &AppHandle, run_id: i64) {
    handle
        .state::<LlmSchedulerState>()
        .cancel_run_requests(run_id)
        .await;
}

async fn run_report_pipeline(
    handle: AppHandle,
    run_id: i64,
    scope_label: String,
    source_ids: Vec<i64>,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template: AnalysisPromptTemplate,
    model_override: Option<String>,
    profile_id: Option<String>,
) -> Result<(), ReportRunError> {
    ensure_report_not_cancelled(&handle, run_id).await?;

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
    .map_err(ReportRunError::Failed)?;

    emit_run_event(
        &handle,
        run_id,
        None,
        "started",
        "load_items",
        None,
        Some("Loading synced messages from local storage...".to_string()),
        None,
        None,
        None,
        None,
        None,
    );

    let corpus = load_corpus_messages(&pool, &source_ids, period_from, period_to)
        .await
        .map_err(ReportRunError::Failed)?;
    if corpus.is_empty() {
        return Err(ReportRunError::Failed(
            "No synced messages were found for the selected analysis scope and period".to_string(),
        ));
    }
    ensure_report_not_cancelled(&handle, run_id).await?;

    emit_run_event(
        &handle,
        run_id,
        None,
        "progress",
        "chunking",
        None,
        Some(format!(
            "Loaded {} messages. Preparing chunks...",
            corpus.len()
        )),
        None,
        None,
        None,
        None,
        None,
    );

    let chunks = chunk_messages(&corpus, ANALYSIS_CHUNK_TARGET_CHARS);
    let resolved_profile = resolve_profile_for_backend(&handle, profile_id.as_deref())
        .await
        .map_err(ReportRunError::Failed)?;
    ensure_report_not_cancelled(&handle, run_id).await?;

    emit_run_event(
        &handle,
        run_id,
        None,
        "progress",
        "map",
        None,
        Some(format!(
            "Dispatching {} chunk analysis request{}...",
            chunks.len(),
            if chunks.len() == 1 { "" } else { "s" }
        )),
        Some(0),
        Some(chunks.len() as i64),
        None,
        None,
        None,
    );

    let completed_chunks = Arc::new(AtomicUsize::new(0));
    let mut join_set = JoinSet::new();
    let total_chunks = chunks.len();
    for (index, chunk) in chunks.into_iter().enumerate() {
        let task_handle = handle.clone();
        let task_profile = resolved_profile.clone();
        let task_profile_id = resolved_profile.profile_id.clone();
        let chunk_request =
            build_map_request(run_id, task_profile_id, index + 1, total_chunks, &chunk);
        let chunk_request_id = chunk_request.request_id.clone();
        let chunk_provider = task_profile.provider.as_str().to_string();
        let chunk_counter = completed_chunks.clone();
        let chunk_message_count = chunk.len() as i64;

        join_set.spawn(async move {
            let scheduler = task_handle.state::<LlmSchedulerState>();
            let request_meta = LlmRequestMetadata {
                request_id: chunk_request.request_id.clone(),
                profile_id: task_profile.profile_id.clone(),
                provider: chunk_provider.clone(),
                kind: LlmRequestKind::AnalysisReportMap,
                priority: LlmRequestPriority::Background,
                owner_run_id: Some(run_id),
            };
            let queued_handle = task_handle.clone();
            let started_handle = task_handle.clone();
            let failed_handle = task_handle.clone();
            let cancelled_handle = task_handle.clone();
            let queued_counter = chunk_counter.clone();
            let started_counter = chunk_counter.clone();
            let queued_request_id = chunk_request_id.clone();
            let started_request_id = chunk_request_id.clone();
            let failed_request_id = chunk_request_id.clone();
            let cancelled_request_id = chunk_request_id.clone();
            let scheduled_request = chunk_request.clone();
            let scheduled_profile = task_profile.clone();

            match scheduler
                .run_request(
                    request_meta,
                    move |position| {
                        emit_run_event(
                            &queued_handle,
                            run_id,
                            Some(queued_request_id.clone()),
                            "queued",
                            "map",
                            Some(position),
                            Some(format!(
                                "Chunk {} of {} queued at position {}...",
                                index + 1,
                                total_chunks,
                                position
                            )),
                            Some(queued_counter.load(Ordering::SeqCst) as i64),
                            Some(total_chunks as i64),
                            None,
                            None,
                            None,
                        );
                    },
                    move |control| async move {
                        emit_run_event(
                            &started_handle,
                            run_id,
                            Some(started_request_id),
                            "started",
                            "map",
                            None,
                            Some(format!(
                                "Analyzing chunk {} of {}...",
                                index + 1,
                                total_chunks
                            )),
                            Some(started_counter.load(Ordering::SeqCst) as i64),
                            Some(total_chunks as i64),
                            None,
                            None,
                            None,
                        );

                        control
                            .run_cancellable(run_llm_collect_with_profile(
                                &scheduled_request,
                                &scheduled_profile,
                            ))
                            .await
                    },
                )
                .await
            {
                Ok(completion) => {
                    let summary =
                        parse_chunk_summary(&completion.text).map_err(ReportRunError::Failed)?;
                    let completed = chunk_counter.fetch_add(1, Ordering::SeqCst) + 1;
                    emit_run_event(
                        &task_handle,
                        run_id,
                        Some(chunk_request_id.clone()),
                        "progress",
                        "map",
                        None,
                        Some(format!(
                            "Chunk {} of {} summarized.",
                            index + 1,
                            total_chunks
                        )),
                        Some(completed as i64),
                        Some(total_chunks as i64),
                        None,
                        Some(AnalysisChunkSummaryEvent {
                            index: (index + 1) as i64,
                            total: total_chunks as i64,
                            message_count: chunk_message_count,
                            summary: summary.summary.clone(),
                            topics: summary.topics.clone(),
                            notable_points: summary.notable_points.clone(),
                            candidate_refs: summary.candidate_refs.clone(),
                        }),
                        None,
                    );
                    Ok::<(usize, ChunkSummary), ReportRunError>((index, summary))
                }
                Err(LlmRequestError::Failed(error)) => {
                    emit_run_event(
                        &failed_handle,
                        run_id,
                        Some(failed_request_id),
                        "failed",
                        "map",
                        None,
                        Some(format!(
                            "Chunk {} of {} failed.",
                            index + 1,
                            total_chunks
                        )),
                        Some(chunk_counter.load(Ordering::SeqCst) as i64),
                        Some(total_chunks as i64),
                        None,
                        None,
                        Some(error.clone()),
                    );
                    Err(ReportRunError::Failed(error))
                }
                Err(LlmRequestError::Cancelled) => {
                    emit_run_event(
                        &cancelled_handle,
                        run_id,
                        Some(cancelled_request_id),
                        "cancelled",
                        "map",
                        None,
                        Some(format!(
                            "Chunk {} of {} cancelled.",
                            index + 1,
                            total_chunks
                        )),
                        Some(chunk_counter.load(Ordering::SeqCst) as i64),
                        Some(total_chunks as i64),
                        None,
                        None,
                        None,
                    );
                    Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()))
                }
            }
        });
    }

    let mut ordered_summaries = vec![None; total_chunks];
    let mut first_error: Option<ReportRunError> = None;
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(Ok((index, summary))) => {
                ordered_summaries[index] = Some(summary);
            }
            Ok(Err(error)) => {
                if first_error.is_none() {
                    first_error = Some(error.clone());
                    cancel_report_children(&handle, run_id).await;
                }
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(ReportRunError::Failed(format!(
                        "Chunk worker crashed: {error}"
                    )));
                    cancel_report_children(&handle, run_id).await;
                }
            }
        }
    }

    let chunk_summaries = finish_map_phase(ordered_summaries, first_error)?;
    ensure_report_not_cancelled(&handle, run_id).await?;

    emit_run_event(
        &handle,
        run_id,
        None,
        "progress",
        "reduce",
        None,
        Some("Writing final report...".to_string()),
        None,
        None,
        None,
        None,
        None,
    );

    let reduce_request = build_reduce_request(
        run_id,
        resolved_profile.profile_id.clone(),
        &scope_label,
        &output_language,
        &prompt_template,
        period_from,
        period_to,
        &chunk_summaries,
        model_override.clone(),
    );
    let reduce_request_id = reduce_request.request_id.clone();
    let reduce_provider = resolved_profile.provider.as_str().to_string();
    let scheduler = handle.state::<LlmSchedulerState>();
    let queued_handle = handle.clone();
    let started_handle = handle.clone();
    let delta_handle = handle.clone();
    let failed_handle = handle.clone();
    let cancelled_handle = handle.clone();
    let queued_request_id = reduce_request_id.clone();
    let started_request_id = reduce_request_id.clone();
    let delta_request_id = reduce_request_id.clone();
    let failed_request_id = reduce_request_id.clone();
    let cancelled_request_id = reduce_request_id.clone();
    let reduce_request_for_stream = reduce_request.clone();
    let reduce_profile = resolved_profile.clone();
    let completion = match scheduler
        .run_request(
            LlmRequestMetadata {
                request_id: reduce_request.request_id.clone(),
                profile_id: resolved_profile.profile_id.clone(),
                provider: reduce_provider.clone(),
                kind: LlmRequestKind::AnalysisReportReduce,
                priority: LlmRequestPriority::Background,
                owner_run_id: Some(run_id),
            },
            move |position| {
                emit_run_event(
                    &queued_handle,
                    run_id,
                    Some(queued_request_id.clone()),
                    "queued",
                    "reduce",
                    Some(position),
                    Some(format!("Final report queued at position {}...", position)),
                    None,
                    None,
                    None,
                    None,
                    None,
                );
            },
            move |control| async move {
                emit_run_event(
                    &started_handle,
                    run_id,
                    Some(started_request_id),
                    "started",
                    "reduce",
                    None,
                    Some("Writing final report...".to_string()),
                    None,
                    None,
                    None,
                    None,
                    None,
                );

                control
                    .run_cancellable(run_llm_stream_with_profile(
                        &reduce_request_for_stream,
                        &reduce_profile,
                        |delta| {
                            emit_run_event(
                                &delta_handle,
                                run_id,
                                Some(delta_request_id.clone()),
                                "delta",
                                "reduce",
                                None,
                                None,
                                None,
                                None,
                                Some(delta.to_string()),
                                None,
                                None,
                            );
                        },
                    ))
                    .await
            },
        )
        .await
    {
        Ok(completion) => completion,
        Err(LlmRequestError::Failed(error)) => {
            emit_run_event(
                &failed_handle,
                run_id,
                Some(failed_request_id),
                "failed",
                "reduce",
                None,
                Some("Final report generation failed.".to_string()),
                None,
                None,
                None,
                None,
                Some(error.clone()),
            );
            return Err(ReportRunError::Failed(error));
        }
        Err(LlmRequestError::Cancelled) => {
            emit_run_event(
                &cancelled_handle,
                run_id,
                Some(cancelled_request_id),
                "cancelled",
                "reduce",
                None,
                Some("Final report generation cancelled.".to_string()),
                None,
                None,
                None,
                None,
                None,
            );
            return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
        }
    };

    ensure_report_not_cancelled(&handle, run_id).await?;
    let trace_data = build_trace_data(&completion.text, &corpus);
    let compressed_trace = compress_trace_data(&trace_data).map_err(ReportRunError::Failed)?;

    emit_run_event(
        &handle,
        run_id,
        Some(reduce_request_id.clone()),
        "progress",
        "persist",
        None,
        Some("Saving report...".to_string()),
        None,
        None,
        None,
        None,
        None,
    );

    persist_run_snapshot(&pool, run_id, &scope_label, &corpus)
        .await
        .map_err(ReportRunError::Failed)?;

    set_run_status(
        &pool,
        run_id,
        ANALYSIS_STATUS_COMPLETED,
        Some(&completion.text),
        Some(&compressed_trace),
        None,
        Some(now_secs()),
    )
    .await
    .map_err(ReportRunError::Failed)?;

    emit_run_event(
        &handle,
        run_id,
        Some(reduce_request_id),
        "completed",
        "persist",
        None,
        Some(format!(
            "Report completed with {} cited references.",
            trace_data.refs.len()
        )),
        None,
        None,
        None,
        None,
        None,
    );

    Ok(())
}

async fn fail_run(handle: &AppHandle, run_id: i64, error: String) {
    if let Ok(pool) = get_pool(handle).await {
        let _ = set_run_status(
            &pool,
            run_id,
            ANALYSIS_STATUS_FAILED,
            None,
            None,
            Some(&error),
            Some(now_secs()),
        )
        .await;
    }

    emit_run_event(
        handle,
        run_id,
        None,
        "failed",
        "persist",
        None,
        Some("Report run failed.".to_string()),
        None,
        None,
        None,
        None,
        Some(error),
    );
}

async fn cancel_run(handle: &AppHandle, run_id: i64, message: String) {
    if let Ok(pool) = get_pool(handle).await {
        let _ = set_run_status(
            &pool,
            run_id,
            ANALYSIS_STATUS_CANCELLED,
            None,
            None,
            Some(&message),
            Some(now_secs()),
        )
        .await;
    }

    emit_run_event(
        handle,
        run_id,
        None,
        "cancelled",
        "persist",
        None,
        Some(message),
        None,
        None,
        None,
        None,
        None,
    );
}

pub async fn cleanup_interrupted_analysis_runs(handle: AppHandle) {
    if let Ok(pool) = get_pool(&handle).await {
        let _ = sqlx::query(
            r#"
            UPDATE analysis_runs
            SET status = ?, error = ?, completed_at = ?
            WHERE status IN (?, ?)
            "#,
        )
        .bind(ANALYSIS_STATUS_CANCELLED)
        .bind(INTERRUPTED_RUN_MESSAGE)
        .bind(now_secs())
        .bind(ANALYSIS_STATUS_QUEUED)
        .bind(ANALYSIS_STATUS_RUNNING)
        .execute(&pool)
        .await;
    }
}

#[tauri::command]
pub async fn cancel_analysis_run(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
    scheduler: tauri::State<'_, LlmSchedulerState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let run = fetch_run_row(&pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;

    if run.status != ANALYSIS_STATUS_QUEUED && run.status != ANALYSIS_STATUS_RUNNING {
        return Err(AppError::conflict(format!(
            "Analysis run {run_id} is not queued or running"
        )));
    }

    let requested = state.request_report_run_cancel(run_id).await;
    let cancelled_requests = scheduler.cancel_run_requests(run_id).await;
    if !requested && cancelled_requests == 0 {
        return Err(AppError::conflict(format!(
            "Analysis run {run_id} is no longer active"
        )));
    }

    emit_run_event(
        &handle,
        run_id,
        None,
        "progress",
        &run.status,
        None,
        Some("Cancelling analysis run...".to_string()),
        None,
        None,
        None,
        None,
        None,
    );

    Ok(())
}

#[tauri::command]
pub async fn start_analysis_report(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template_id: i64,
    model_override: Option<String>,
    profile_id: Option<String>,
) -> AppResult<i64> {
    if period_from > period_to {
        return Err(AppError::validation(
            "period_from must be less than or equal to period_to",
        ));
    }

    let output_language = output_language.trim().to_string();
    if output_language.is_empty() {
        return Err(AppError::validation("Output language cannot be empty"));
    }

    if source_id.is_some() == source_group_id.is_some() {
        return Err(AppError::validation(
            "Select either a source or a source group",
        ));
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

    let (scope_type, resolved_source_id, resolved_group_id, scope_label, source_ids) =
        if let Some(source_id) = source_id {
            let source_exists =
                sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)")
                    .bind(source_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
            if source_exists == 0 {
                return Err(AppError::not_found(format!("Source {source_id} not found")));
            }

            let source_title =
                sqlx::query_scalar::<_, Option<String>>("SELECT title FROM sources WHERE id = ?")
                    .bind(source_id)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| e.to_string())?
                    .flatten()
                    .filter(|title| !title.trim().is_empty())
                    .unwrap_or_else(|| format!("Source {source_id}"));

            (
                ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
                Some(source_id),
                None,
                source_title,
                vec![source_id],
            )
        } else {
            let group_id = source_group_id.expect("validated source_group_id");
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
                group.name.clone(),
                group
                    .members
                    .into_iter()
                    .map(|member| member.source_id)
                    .collect::<Vec<_>>(),
            )
        };

    if let Some(existing_run_id) = find_active_duplicate_run(
        &pool,
        scope_type,
        resolved_source_id,
        resolved_group_id,
        period_from,
        period_to,
        &output_language,
        prompt_template.id,
        &resolved_profile.profile_id,
        &effective_model,
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
        scope_type,
        resolved_source_id,
        resolved_group_id,
        period_from,
        period_to,
        &output_language,
        &prompt_template,
        &resolved_profile.profile_id,
        resolved_profile.provider.as_str(),
        &effective_model,
    )
    .await?;

    state.insert_active_report_run(run_id).await;

    let app_handle = handle.clone();
    tokio::spawn(async move {
        match run_report_pipeline(
            app_handle.clone(),
            run_id,
            scope_label,
            source_ids,
            period_from,
            period_to,
            output_language,
            prompt_template,
            model_override,
            profile_id,
        )
        .await
        {
            Ok(()) => {}
            Err(ReportRunError::Failed(error)) => fail_run(&app_handle, run_id, error).await,
            Err(ReportRunError::Cancelled(message)) => cancel_run(&app_handle, run_id, message).await,
        }
        app_handle
            .state::<AnalysisState>()
            .remove_active_report_run(run_id)
            .await;
    });

    Ok(run_id)
}

#[cfg(test)]
mod tests {
    use super::{
        build_map_request, build_reduce_request, extract_json_payload, finish_map_phase,
        parse_chunk_summary, ReportRunError,
    };
    use crate::analysis::models::{AnalysisPromptTemplate, ChunkSummary, CorpusMessage};

    const SAMPLE_JSON: &str =
        r#"{"summary":"Brief","topics":["sync"],"notable_points":["Point"],"candidate_refs":["s1-m2"]}"#;

    fn sample_chunk_summary(label: &str) -> ChunkSummary {
        ChunkSummary {
            summary: label.to_string(),
            topics: vec![format!("{label}-topic")],
            notable_points: vec![format!("{label}-point")],
            candidate_refs: vec![format!("{label}-ref")],
        }
    }

    fn sample_prompt_template() -> AnalysisPromptTemplate {
        AnalysisPromptTemplate {
            id: 7,
            name: "Report".to_string(),
            template_kind: "report".to_string(),
            body: "Write a concise report.".to_string(),
            version: 3,
            is_builtin: false,
            created_at: 1,
            updated_at: 1,
        }
    }

    fn sample_corpus_message() -> CorpusMessage {
        CorpusMessage {
            item_id: 1,
            source_id: 2,
            external_id: "42".to_string(),
            published_at: 1_700_000_000,
            author: Some("analyst".to_string()),
            content: "Important update from the source".to_string(),
            r#ref: "s2-m42".to_string(),
        }
    }

    #[test]
    fn extracts_json_with_text_before_and_after() {
        let response = format!("Preface\n{SAMPLE_JSON}\nTail");
        let payload = extract_json_payload(&response).expect("extract payload");

        assert_eq!(payload, SAMPLE_JSON);
    }

    #[test]
    fn extracts_json_inside_markdown_fence() {
        let response = format!("```json\n{SAMPLE_JSON}\n```");
        let payload = extract_json_payload(&response).expect("extract fenced payload");

        assert_eq!(payload, SAMPLE_JSON);
    }

    #[test]
    fn parse_chunk_summary_ignores_non_json_prefix_with_braces() {
        let summary = parse_chunk_summary(&format!("Note {{not json}}\n{SAMPLE_JSON}"))
            .expect("parse summary");

        assert_eq!(summary.summary, "Brief");
        assert_eq!(summary.topics, vec!["sync".to_string()]);
    }

    #[test]
    fn parse_chunk_summary_rejects_malformed_payload() {
        let error = parse_chunk_summary("```json\n{\"summary\": }\n```")
            .expect_err("malformed payload should fail");

        assert!(
            error.contains("Failed to parse chunk summary JSON")
                || error.contains("malformed JSON")
                || error.contains("valid JSON object")
        );
    }

    #[test]
    fn finish_map_phase_preserves_chunk_order_by_original_index() {
        let ordered = vec![
            Some(sample_chunk_summary("first")),
            Some(sample_chunk_summary("second")),
            Some(sample_chunk_summary("third")),
        ];

        let collected = finish_map_phase(ordered, None).expect("collect summaries");

        assert_eq!(collected[0].summary, "first");
        assert_eq!(collected[1].summary, "second");
        assert_eq!(collected[2].summary, "third");
    }

    #[test]
    fn finish_map_phase_rejects_missing_chunk_before_reduce() {
        let ordered = vec![Some(sample_chunk_summary("first")), None];

        let error = finish_map_phase(ordered, None).expect_err("missing chunk should fail");

        assert_eq!(
            error,
            ReportRunError::Failed("Some chunk summaries were not collected".to_string())
        );
    }

    #[test]
    fn finish_map_phase_propagates_map_error_without_starting_reduce() {
        let ordered = vec![Some(sample_chunk_summary("first"))];

        let error = finish_map_phase(
            ordered,
            Some(ReportRunError::Cancelled("Analysis run cancelled.".to_string())),
        )
        .expect_err("map cancellation should stop reduce");

        assert_eq!(
            error,
            ReportRunError::Cancelled("Analysis run cancelled.".to_string())
        );
    }

    #[test]
    fn build_map_request_keeps_run_scoped_request_and_profile() {
        let request = build_map_request(55, "default".to_string(), 2, 4, &[sample_corpus_message()]);

        assert!(request.request_id.starts_with("analysis-map-55-2-"));
        assert_eq!(request.profile_id.as_deref(), Some("default"));
        assert!(request.messages[1].content.contains("Chunk 2 of 4."));
    }

    #[test]
    fn build_reduce_request_keeps_run_scoped_request_and_profile() {
        let request = build_reduce_request(
            77,
            "profile-a".to_string(),
            "My scope",
            "Russian",
            &sample_prompt_template(),
            10,
            20,
            &[sample_chunk_summary("alpha"), sample_chunk_summary("beta")],
            Some("model-x".to_string()),
        );

        assert!(request.request_id.starts_with("analysis-reduce-77-"));
        assert_eq!(request.profile_id.as_deref(), Some("profile-a"));
        assert_eq!(request.model_override.as_deref(), Some("model-x"));
        assert!(request.messages[1].content.contains("Chunk 1 summary"));
        assert!(request.messages[1].content.contains("Chunk 2 summary"));
    }
}
