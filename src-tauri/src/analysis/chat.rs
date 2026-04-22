use tauri::AppHandle;

use crate::db::get_pool;
use crate::llm::{
    resolve_profile_for_backend, run_llm_stream_with_profile, LlmChatRequest, LlmMessage,
};

use super::models::{
    AnalysisChatEvent, AnalysisChatMessage, AnalysisChatTurn, AnalysisRunDetail, CorpusMessage,
};
use super::store::{
    fetch_run_row, load_chat_messages_from_pool, load_corpus_messages, map_run_detail,
    persist_chat_exchange, resolve_run_source_ids,
};
use super::{
    emit_analysis_chat_event, now_secs, validate_chat_turns, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
    ANALYSIS_STATUS_COMPLETED,
};

fn chat_search_terms(question: &str) -> Vec<String> {
    const STOP_WORDS: &[&str] = &[
        "the",
        "and",
        "for",
        "with",
        "that",
        "this",
        "from",
        "into",
        "about",
        "what",
        "when",
        "where",
        "which",
        "have",
        "has",
        "were",
        "will",
        "would",
        "could",
        "should",
        "РєР°Рє",
        "С‡С‚Рѕ",
        "СЌС‚Рѕ",
        "РґР»СЏ",
        "РїСЂРѕ",
        "РёР»Рё",
        "РµСЃР»Рё",
        "РєРѕРіРґР°",
        "РєР°РєРёРµ",
        "РєР°РєРѕР№",
        "РіРґРµ",
        "РїРѕСЃР»Рµ",
        "РЅР°Рґ",
        "РїРѕРґ",
        "РµС‰С‘",
        "also",
        "over",
    ];

    let mut terms = question
        .split(|c: char| !c.is_alphanumeric())
        .map(|part| part.trim().to_ascii_lowercase())
        .filter(|part| part.len() >= 3 && !STOP_WORDS.contains(&part.as_str()))
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms.truncate(8);
    terms
}

fn find_chat_context_messages<'a>(
    question: &str,
    corpus: &'a [CorpusMessage],
) -> Vec<&'a CorpusMessage> {
    let terms = chat_search_terms(question);
    if terms.is_empty() {
        return corpus.iter().rev().take(6).collect();
    }

    let mut scored = corpus
        .iter()
        .filter_map(|message| {
            let haystack = message.content.to_ascii_lowercase();
            let score = terms
                .iter()
                .map(|term| usize::from(haystack.contains(term)))
                .sum::<usize>();
            (score > 0).then_some((score, message.published_at, message))
        })
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));

    scored
        .into_iter()
        .take(8)
        .map(|(_, _, message)| message)
        .collect()
}

fn clip_excerpt(content: &str, max_chars: usize) -> String {
    if content.chars().count() <= max_chars {
        return content.to_string();
    }

    let clipped = content.chars().take(max_chars).collect::<String>();
    format!("{clipped}...")
}

fn format_chat_context_messages(messages: &[&CorpusMessage]) -> String {
    if messages.is_empty() {
        return "No additional local message matches were found for the current question."
            .to_string();
    }

    messages
        .iter()
        .map(|message| {
            format!(
                "[{ref}] Date: {published_at}\nAuthor: {author}\nExcerpt:\n{excerpt}",
                ref = message.r#ref,
                published_at = message.published_at,
                author = message.author.as_deref().unwrap_or("unknown"),
                excerpt = clip_excerpt(&message.content, 420)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

fn build_chat_request(
    run: &AnalysisRunDetail,
    history: &[AnalysisChatTurn],
    question: &str,
    report_markdown: &str,
    context_messages: &[&CorpusMessage],
    model_override: Option<String>,
) -> LlmChatRequest {
    let mut messages = vec![
        LlmMessage {
            role: "system".to_string(),
            content: format!(
                "You answer follow-up questions about a saved Telegram analysis report.\nAnswer in {}.\nUse markdown only.\nGround every important claim in the saved report or the provided message excerpts.\nWhen referring to message evidence, cite refs like [s12-m845].\nDo not invent facts beyond the saved report and provided excerpts.",
                run.output_language
            ),
        },
        LlmMessage {
            role: "user".to_string(),
            content: format!(
                "Saved report scope: {}\nSaved report period: {} to {}\n\nSaved report markdown:\n\n{}\n\nAdditional local message matches for the current question:\n\n{}",
                if run.scope_type == ANALYSIS_SCOPE_TYPE_SOURCE_GROUP {
                    run.source_group_name
                        .clone()
                        .unwrap_or_else(|| format!("Group {}", run.source_group_id.unwrap_or_default()))
                } else {
                    run.source_title
                        .clone()
                        .unwrap_or_else(|| format!("Source {}", run.source_id.unwrap_or_default()))
                },
                run.period_from,
                run.period_to,
                report_markdown,
                format_chat_context_messages(context_messages)
            ),
        },
    ];

    messages.extend(history.iter().map(|turn| LlmMessage {
        role: turn.role.clone(),
        content: turn.content.clone(),
    }));

    messages.push(LlmMessage {
        role: "user".to_string(),
        content: question.trim().to_string(),
    });

    LlmChatRequest {
        request_id: format!("analysis-chat-{}-{}", run.id, now_secs()),
        profile_id: Some(run.provider_profile.clone()),
        messages,
        model_override,
    }
}

#[tauri::command]
pub async fn list_analysis_chat_messages(
    handle: AppHandle,
    run_id: i64,
) -> Result<Vec<AnalysisChatMessage>, String> {
    let pool = get_pool(&handle).await?;
    let exists = fetch_run_row(&pool, run_id).await?.is_some();
    if !exists {
        return Err(format!("Analysis run {run_id} not found"));
    }
    load_chat_messages_from_pool(&pool, run_id).await
}

#[tauri::command]
pub async fn clear_analysis_chat_messages(handle: AppHandle, run_id: i64) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    let exists = fetch_run_row(&pool, run_id).await?.is_some();
    if !exists {
        return Err(format!("Analysis run {run_id} not found"));
    }

    sqlx::query("DELETE FROM analysis_chat_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn ask_analysis_run_question(
    handle: AppHandle,
    run_id: i64,
    question: String,
    model_override: Option<String>,
    profile_id: Option<String>,
) -> Result<String, String> {
    let question = question.trim().to_string();
    if question.is_empty() {
        return Err("Question cannot be empty".to_string());
    }

    let pool = get_pool(&handle).await?;
    let run = fetch_run_row(&pool, run_id)
        .await?
        .map(map_run_detail)
        .ok_or_else(|| format!("Analysis run {run_id} not found"))?;

    if run.status != ANALYSIS_STATUS_COMPLETED {
        return Err("Open a completed analysis run before asking follow-up questions".to_string());
    }

    let report_markdown = run
        .result_markdown
        .clone()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| "The selected analysis run does not have a saved report".to_string())?;

    let source_ids = resolve_run_source_ids(&pool, &run).await?;
    let corpus = load_corpus_messages(&pool, &source_ids, run.period_from, run.period_to).await?;
    let context_messages = find_chat_context_messages(&question, &corpus);
    let history = load_chat_messages_from_pool(&pool, run_id)
        .await?
        .into_iter()
        .map(|message| AnalysisChatTurn {
            role: message.role,
            content: message.content,
        })
        .collect::<Vec<_>>();
    validate_chat_turns(&history)?;
    let request = build_chat_request(
        &run,
        &history,
        &question,
        &report_markdown,
        &context_messages,
        model_override.clone(),
    );

    let request_id = request.request_id.clone();
    let emitted_request_id = request_id.clone();
    let app_handle = handle.clone();
    tokio::spawn(async move {
        let resolved_profile =
            match resolve_profile_for_backend(&app_handle, profile_id.as_deref()).await {
                Ok(profile) => profile,
                Err(error) => {
                    emit_analysis_chat_event(
                        &app_handle,
                        &AnalysisChatEvent {
                            request_id: emitted_request_id.clone(),
                            run_id,
                            kind: "failed".to_string(),
                            delta: None,
                            message: None,
                            error: Some(error),
                        },
                    );
                    return;
                }
            };

        emit_analysis_chat_event(
            &app_handle,
            &AnalysisChatEvent {
                request_id: emitted_request_id.clone(),
                run_id,
                kind: "started".to_string(),
                delta: None,
                message: Some("Preparing grounded answer...".to_string()),
                error: None,
            },
        );

        match run_llm_stream_with_profile(&request, &resolved_profile, |delta| {
            emit_analysis_chat_event(
                &app_handle,
                &AnalysisChatEvent {
                    request_id: emitted_request_id.clone(),
                    run_id,
                    kind: "delta".to_string(),
                    delta: Some(delta.to_string()),
                    message: None,
                    error: None,
                },
            );
        })
        .await
        {
            Ok(completion) => {
                if let Ok(pool) = get_pool(&app_handle).await {
                    let _ = persist_chat_exchange(&pool, run_id, &question, &completion.text).await;
                }

                emit_analysis_chat_event(
                    &app_handle,
                    &AnalysisChatEvent {
                        request_id: emitted_request_id.clone(),
                        run_id,
                        kind: "completed".to_string(),
                        delta: None,
                        message: Some("Answer completed.".to_string()),
                        error: None,
                    },
                )
            }
            Err(error) => emit_analysis_chat_event(
                &app_handle,
                &AnalysisChatEvent {
                    request_id: emitted_request_id.clone(),
                    run_id,
                    kind: "failed".to_string(),
                    delta: None,
                    message: None,
                    error: Some(error),
                },
            ),
        }
    });

    Ok(request_id)
}
