use tauri::AppHandle;

use crate::db::get_pool;
use crate::llm::{
    resolve_effective_model, resolve_profile_for_backend, run_llm_collect_with_profile,
    run_llm_stream_with_profile, LlmChatRequest, LlmMessage,
};

use super::models::{AnalysisPromptTemplate, AnalysisRunEvent, ChunkSummary, CorpusMessage};
use super::store::{
    fetch_prompt_template, fetch_source_group, find_active_duplicate_run, insert_analysis_run,
    load_corpus_messages, set_run_status,
};
use super::trace::{build_trace_data, compress_trace_data, normalize_ref};
use super::{
    emit_analysis_event, now_secs, ANALYSIS_CHUNK_TARGET_CHARS, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_COMPLETED, ANALYSIS_STATUS_FAILED,
    ANALYSIS_STATUS_RUNNING, TEMPLATE_KIND_REPORT,
};

fn chunk_messages(messages: &[CorpusMessage], max_chars: usize) -> Vec<Vec<CorpusMessage>> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_chars = 0usize;

    for message in messages {
        let estimated_len =
            message.content.len() + message.r#ref.len() + message.author.as_deref().unwrap_or("").len() + 64;

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

fn build_map_request(chunk_index: usize, total_chunks: usize, messages: &[CorpusMessage]) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!("analysis-map-{}-{}", now_secs(), chunk_index),
        profile_id: None,
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
        request_id: format!("analysis-reduce-{}", now_secs()),
        profile_id: None,
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
) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    set_run_status(&pool, run_id, ANALYSIS_STATUS_RUNNING, None, None, None, None).await?;

    emit_analysis_event(
        &handle,
        &AnalysisRunEvent {
            run_id,
            kind: "started".to_string(),
            phase: "load_items".to_string(),
            message: Some("Loading synced messages from local storage...".to_string()),
            progress_current: None,
            progress_total: None,
            delta: None,
            error: None,
        },
    );

    let corpus = load_corpus_messages(&pool, &source_ids, period_from, period_to).await?;
    if corpus.is_empty() {
        return Err("No synced messages were found for the selected analysis scope and period".to_string());
    }

    emit_analysis_event(
        &handle,
        &AnalysisRunEvent {
            run_id,
            kind: "progress".to_string(),
            phase: "chunking".to_string(),
            message: Some(format!("Loaded {} messages. Preparing chunks...", corpus.len())),
            progress_current: None,
            progress_total: None,
            delta: None,
            error: None,
        },
    );

    let chunks = chunk_messages(&corpus, ANALYSIS_CHUNK_TARGET_CHARS);
    let resolved_profile = resolve_profile_for_backend(&handle, profile_id.as_deref()).await?;
    let mut chunk_summaries = Vec::new();

    for (index, chunk) in chunks.iter().enumerate() {
        emit_analysis_event(
            &handle,
            &AnalysisRunEvent {
                run_id,
                kind: "progress".to_string(),
                phase: "map".to_string(),
                message: Some(format!("Analyzing chunk {} of {}...", index + 1, chunks.len())),
                progress_current: Some((index + 1) as i64),
                progress_total: Some(chunks.len() as i64),
                delta: None,
                error: None,
            },
        );

        let request = build_map_request(index + 1, chunks.len(), chunk);
        let completion = run_llm_collect_with_profile(&request, &resolved_profile).await?;
        chunk_summaries.push(parse_chunk_summary(&completion.text)?);
    }

    emit_analysis_event(
        &handle,
        &AnalysisRunEvent {
            run_id,
            kind: "progress".to_string(),
            phase: "reduce".to_string(),
            message: Some("Writing final report...".to_string()),
            progress_current: None,
            progress_total: None,
            delta: None,
            error: None,
        },
    );

    let reduce_request = build_reduce_request(
        &scope_label,
        &output_language,
        &prompt_template,
        period_from,
        period_to,
        &chunk_summaries,
        model_override.clone(),
    );

    let completion = run_llm_stream_with_profile(&reduce_request, &resolved_profile, |delta| {
        emit_analysis_event(
            &handle,
            &AnalysisRunEvent {
                run_id,
                kind: "delta".to_string(),
                phase: "reduce".to_string(),
                message: None,
                progress_current: None,
                progress_total: None,
                delta: Some(delta.to_string()),
                error: None,
            },
        );
    })
    .await?;

    let trace_data = build_trace_data(&completion.text, &corpus);
    let compressed_trace = compress_trace_data(&trace_data)?;

    emit_analysis_event(
        &handle,
        &AnalysisRunEvent {
            run_id,
            kind: "progress".to_string(),
            phase: "persist".to_string(),
            message: Some("Saving report...".to_string()),
            progress_current: None,
            progress_total: None,
            delta: None,
            error: None,
        },
    );

    set_run_status(
        &pool,
        run_id,
        ANALYSIS_STATUS_COMPLETED,
        Some(&completion.text),
        Some(&compressed_trace),
        None,
        Some(now_secs()),
    )
    .await?;

    emit_analysis_event(
        &handle,
        &AnalysisRunEvent {
            run_id,
            kind: "completed".to_string(),
            phase: "persist".to_string(),
            message: Some(format!(
                "Report completed with {} cited references.",
                trace_data.refs.len()
            )),
            progress_current: None,
            progress_total: None,
            delta: None,
            error: None,
        },
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

    emit_analysis_event(
        handle,
        &AnalysisRunEvent {
            run_id,
            kind: "failed".to_string(),
            phase: "persist".to_string(),
            message: Some("Report run failed.".to_string()),
            progress_current: None,
            progress_total: None,
            delta: None,
            error: Some(error),
        },
    );
}

#[tauri::command]
pub async fn start_analysis_report(
    handle: AppHandle,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template_id: i64,
    model_override: Option<String>,
    profile_id: Option<String>,
) -> Result<i64, String> {
    if period_from > period_to {
        return Err("period_from must be less than or equal to period_to".to_string());
    }

    let output_language = output_language.trim().to_string();
    if output_language.is_empty() {
        return Err("Output language cannot be empty".to_string());
    }

    if source_id.is_some() == source_group_id.is_some() {
        return Err("Select either a source or a source group".to_string());
    }

    let pool = get_pool(&handle).await?;
    let prompt_template = fetch_prompt_template(&pool, prompt_template_id).await?;
    if prompt_template.template_kind != TEMPLATE_KIND_REPORT {
        return Err("Selected prompt template is not a report template".to_string());
    }

    let resolved_profile = resolve_profile_for_backend(&handle, profile_id.as_deref()).await?;
    let effective_model = resolve_effective_model(&resolved_profile, model_override.as_deref())?;

    let (scope_type, resolved_source_id, resolved_group_id, scope_label, source_ids) =
        if let Some(source_id) = source_id {
            let source_exists = sqlx::query_scalar::<_, i64>(
                "SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)",
            )
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
            if source_exists == 0 {
                return Err(format!("Source {source_id} not found"));
            }

            let source_title = sqlx::query_scalar::<_, Option<String>>(
                "SELECT title FROM sources WHERE id = ?",
            )
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
            let group = fetch_source_group(&pool, group_id)
                .await?
                .ok_or_else(|| format!("Analysis source group {group_id} not found"))?;

            if group.members.is_empty() {
                return Err("The selected source group does not contain any sources".to_string());
            }

            (
                ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
                None,
                Some(group.id),
                group.name.clone(),
                group.members.into_iter().map(|member| member.source_id).collect::<Vec<_>>(),
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
        return Err(format!(
            "An identical analysis report is already queued or running (run {existing_run_id})"
        ));
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

    let app_handle = handle.clone();
    tokio::spawn(async move {
        if let Err(error) = run_report_pipeline(
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
            fail_run(&app_handle, run_id, error).await;
        }
    });

    Ok(run_id)
}

#[cfg(test)]
mod tests {
    use super::{extract_json_payload, parse_chunk_summary};

    const SAMPLE_JSON: &str = r#"{"summary":"Brief","topics":["sync"],"notable_points":["Point"],"candidate_refs":["s1-m2"]}"#;

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
}
