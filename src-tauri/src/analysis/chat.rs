use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_profile_for_backend, run_llm_stream_with_profile, LlmChatRequest, LlmMessage,
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState,
};

use super::corpus::load_run_corpus_messages;
use super::models::{
    AnalysisChatEvent, AnalysisChatMessage, AnalysisChatTurn, AnalysisRunDetail, CorpusMessage,
};
use super::store::{fetch_run_row, map_run_detail, resolve_run_scope_label};
use super::{
    emit_analysis_chat_event, now_secs, validate_chat_role, validate_chat_turns,
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
    profile_id: String,
    scope_label: &str,
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
                scope_label,
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
        profile_id: Some(profile_id),
        messages,
        model_override,
    }
}

fn emit_chat_event(
    handle: &AppHandle,
    request_id: String,
    run_id: i64,
    kind: &str,
    queue_position: Option<usize>,
    delta: Option<String>,
    message: Option<String>,
    error: Option<String>,
) {
    emit_analysis_chat_event(
        handle,
        &AnalysisChatEvent {
            request_id,
            run_id,
            kind: kind.to_string(),
            queue_position,
            delta,
            message,
            error,
        },
    );
}

async fn load_chat_messages_from_pool(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> Result<Vec<AnalysisChatMessage>, String> {
    sqlx::query_as(
        r#"
        SELECT id, run_id, role, content, created_at
        FROM analysis_chat_messages
        WHERE run_id = ?
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())
}

async fn persist_chat_exchange(
    pool: &Pool<Sqlite>,
    run_id: i64,
    user_question: &str,
    assistant_answer: &str,
) -> Result<(), String> {
    validate_chat_role("user")?;
    validate_chat_role("assistant")?;

    let now = now_secs();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(run_id)
    .bind("user")
    .bind(user_question)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(run_id)
    .bind("assistant")
    .bind(assistant_answer)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_analysis_chat_messages(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Vec<AnalysisChatMessage>> {
    let pool = get_pool(&handle).await?;
    let exists = fetch_run_row(&pool, run_id).await?.is_some();
    if !exists {
        return Err(AppError::not_found(format!(
            "Analysis run {run_id} not found"
        )));
    }
    Ok(load_chat_messages_from_pool(&pool, run_id).await?)
}

#[tauri::command]
pub async fn clear_analysis_chat_messages(handle: AppHandle, run_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let exists = fetch_run_row(&pool, run_id).await?.is_some();
    if !exists {
        return Err(AppError::not_found(format!(
            "Analysis run {run_id} not found"
        )));
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
) -> AppResult<String> {
    let question = question.trim().to_string();
    if question.is_empty() {
        return Err(AppError::validation("Question cannot be empty"));
    }

    let pool = get_pool(&handle).await?;
    let run_row = fetch_run_row(&pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;
    let run = map_run_detail(run_row);
    let scope_label = resolve_run_scope_label(&run);

    if run.status != ANALYSIS_STATUS_COMPLETED {
        return Err(AppError::validation(
            "Open a completed analysis run before asking follow-up questions",
        ));
    }

    let report_markdown = run
        .result_markdown
        .clone()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| {
            AppError::conflict("The selected analysis run does not have a saved report")
        })?;

    let corpus = load_run_corpus_messages(&pool, &run).await?;
    let context_messages = find_chat_context_messages(&question, &corpus);
    let effective_profile_id = profile_id.unwrap_or_else(|| run.provider_profile.clone());
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
        effective_profile_id.clone(),
        &scope_label,
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
            match resolve_profile_for_backend(&app_handle, Some(effective_profile_id.as_str())).await
            {
                Ok(profile) => profile,
                Err(error) => {
                    emit_chat_event(
                        &app_handle,
                        emitted_request_id.clone(),
                        run_id,
                        "failed",
                        None,
                        None,
                        None,
                        Some(error),
                    );
                    return;
                }
            };

        let scheduler = app_handle.state::<LlmSchedulerState>();
        let request_meta = LlmRequestMetadata {
            request_id: request.request_id.clone(),
            profile_id: resolved_profile.profile_id.clone(),
            provider: resolved_profile.provider.as_str().to_string(),
            kind: LlmRequestKind::AnalysisChat,
            priority: LlmRequestPriority::Interactive,
            owner_run_id: None,
        };
        let queued_handle = app_handle.clone();
        let started_handle = app_handle.clone();
        let delta_handle = app_handle.clone();
        let completed_handle = app_handle.clone();
        let failed_handle = app_handle.clone();
        let cancelled_handle = app_handle.clone();
        let queued_request_id = emitted_request_id.clone();
        let started_request_id = emitted_request_id.clone();
        let delta_request_id = emitted_request_id.clone();
        let completed_request_id = emitted_request_id.clone();
        let failed_request_id = emitted_request_id.clone();
        let cancelled_request_id = emitted_request_id.clone();
        let scheduled_request = request.clone();
        let scheduled_profile = resolved_profile.clone();

        match scheduler
            .run_request(
                request_meta,
                move |position| {
                    emit_chat_event(
                        &queued_handle,
                        queued_request_id.clone(),
                        run_id,
                        "queued",
                        Some(position),
                        None,
                        Some(format!("Answer queued at position {position}...")),
                        None,
                    );
                },
                move |control| async move {
                    emit_chat_event(
                        &started_handle,
                        started_request_id,
                        run_id,
                        "started",
                        None,
                        None,
                        Some("Preparing grounded answer...".to_string()),
                        None,
                    );

                    control
                        .run_cancellable(run_llm_stream_with_profile(
                            &scheduled_request,
                            &scheduled_profile,
                            |delta| {
                                emit_chat_event(
                                    &delta_handle,
                                    delta_request_id.clone(),
                                    run_id,
                                    "delta",
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
            Ok(completion) => {
                if let Ok(pool) = get_pool(&app_handle).await {
                    let _ = persist_chat_exchange(&pool, run_id, &question, &completion.text).await;
                }

                emit_chat_event(
                    &completed_handle,
                    completed_request_id,
                    run_id,
                    "completed",
                    None,
                    None,
                    Some("Answer completed.".to_string()),
                    None,
                );
            }
            Err(LlmRequestError::Failed(error)) => emit_chat_event(
                &failed_handle,
                failed_request_id,
                run_id,
                "failed",
                None,
                None,
                None,
                Some(error),
            ),
            Err(LlmRequestError::Cancelled) => emit_chat_event(
                &cancelled_handle,
                cancelled_request_id,
                run_id,
                "cancelled",
                None,
                None,
                Some("Answer cancelled.".to_string()),
                None,
            ),
        }
    });

    Ok(request_id)
}
