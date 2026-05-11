use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_profile_for_backend, run_llm_stream_with_profile, LlmChatRequest, LlmMessage,
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState,
};

use super::corpus::load_run_snapshot_messages;
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
        "как",
        "что",
        "это",
        "для",
        "про",
        "или",
        "если",
        "когда",
        "какие",
        "какой",
        "где",
        "после",
        "над",
        "под",
        "ещё",
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
        return "No additional local source document matches were found for the current question."
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

fn ensure_completed_chat_context(
    run: &AnalysisRunDetail,
    snapshot: &[CorpusMessage],
) -> AppResult<()> {
    if run.status != ANALYSIS_STATUS_COMPLETED {
        return Err(AppError::validation(
            "Open a completed analysis run before asking follow-up questions",
        ));
    }

    if snapshot.is_empty() {
        return Err(AppError::conflict(
            "This completed analysis run has no saved snapshot context for follow-up chat",
        ));
    }

    Ok(())
}

struct ChatRequestParams<'a> {
    run: &'a AnalysisRunDetail,
    profile_id: String,
    scope_label: &'a str,
    history: &'a [AnalysisChatTurn],
    question: &'a str,
    report_markdown: &'a str,
    context_messages: &'a [&'a CorpusMessage],
    model_override: Option<String>,
}

fn build_chat_request(params: ChatRequestParams<'_>) -> LlmChatRequest {
    let mut messages = vec![
        LlmMessage {
            role: "system".to_string(),
            content: format!(
                "You answer follow-up questions about a saved source analysis report.\nAnswer in {}.\nUse markdown only.\nGround every important claim in the saved report or the provided source document excerpts.\nWhen referring to source evidence, cite refs like [s12-i845].\nDo not invent facts beyond the saved report and provided excerpts.",
                params.run.output_language
            ),
        },
        LlmMessage {
            role: "user".to_string(),
            content: format!(
                "Saved report scope: {}\nSaved report period: {} to {}\n\nSaved report markdown:\n\n{}\n\nAdditional local source document matches for the current question:\n\n{}",
                params.scope_label,
                params.run.period_from,
                params.run.period_to,
                params.report_markdown,
                format_chat_context_messages(params.context_messages)
            ),
        },
    ];

    messages.extend(params.history.iter().map(|turn| LlmMessage {
        role: turn.role.clone(),
        content: turn.content.clone(),
    }));

    messages.push(LlmMessage {
        role: "user".to_string(),
        content: params.question.trim().to_string(),
    });

    LlmChatRequest {
        request_id: format!("analysis-chat-{}-{}", params.run.id, now_secs()),
        profile_id: Some(params.profile_id),
        messages,
        model_override: params.model_override,
    }
}

struct ChatEvent {
    event: AnalysisChatEvent,
}

impl ChatEvent {
    fn new(request_id: String, run_id: i64, kind: &str) -> Self {
        Self {
            event: AnalysisChatEvent {
                request_id,
                run_id,
                kind: kind.to_string(),
                queue_position: None,
                delta: None,
                message: None,
                error: None,
            },
        }
    }

    fn queue_position(mut self, queue_position: usize) -> Self {
        self.event.queue_position = Some(queue_position);
        self
    }

    fn delta(mut self, delta: String) -> Self {
        self.event.delta = Some(delta);
        self
    }

    fn message(mut self, message: String) -> Self {
        self.event.message = Some(message);
        self
    }

    fn error(mut self, error: String) -> Self {
        self.event.error = Some(error);
        self
    }

    fn emit(self, handle: &AppHandle) {
        emit_analysis_chat_event(handle, &self.event);
    }
}

async fn load_chat_messages_from_pool(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> AppResult<Vec<AnalysisChatMessage>> {
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
    .map_err(AppError::database)
}

async fn persist_chat_exchange(
    pool: &Pool<Sqlite>,
    run_id: i64,
    user_question: &str,
    assistant_answer: &str,
) -> AppResult<()> {
    validate_chat_role("user")?;
    validate_chat_role("assistant")?;

    let now = now_secs();
    let mut tx = pool.begin().await.map_err(AppError::database)?;

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
    .map_err(AppError::database)?;

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
    .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)?;

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
    load_chat_messages_from_pool(&pool, run_id).await
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
        .map_err(AppError::database)?;

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

    let corpus = load_run_snapshot_messages(&pool, run.id)
        .await
        .map_err(AppError::database)?;
    ensure_completed_chat_context(&run, &corpus)?;
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
    let request = build_chat_request(ChatRequestParams {
        run: &run,
        profile_id: effective_profile_id.clone(),
        scope_label: &scope_label,
        history: &history,
        question: &question,
        report_markdown: &report_markdown,
        context_messages: &context_messages,
        model_override: model_override.clone(),
    });

    let request_id = request.request_id.clone();
    let emitted_request_id = request_id.clone();
    let app_handle = handle.clone();
    tokio::spawn(async move {
        let resolved_profile =
            match resolve_profile_for_backend(&app_handle, Some(effective_profile_id.as_str()))
                .await
            {
                Ok(profile) => profile,
                Err(error) => {
                    ChatEvent::new(emitted_request_id.clone(), run_id, "failed")
                        .error(String::from(error))
                        .emit(&app_handle);
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
                    ChatEvent::new(queued_request_id.clone(), run_id, "queued")
                        .queue_position(position)
                        .message(format!("Answer queued at position {position}..."))
                        .emit(&queued_handle);
                },
                move |control| async move {
                    ChatEvent::new(started_request_id, run_id, "started")
                        .message("Preparing grounded answer...".to_string())
                        .emit(&started_handle);

                    control
                        .run_cancellable(run_llm_stream_with_profile(
                            &scheduled_request,
                            &scheduled_profile,
                            |delta| {
                                ChatEvent::new(delta_request_id.clone(), run_id, "delta")
                                    .delta(delta.to_string())
                                    .emit(&delta_handle);
                            },
                        ))
                        .await
                },
            )
            .await
        {
            Ok(completion) => {
                let pool = match get_pool(&app_handle).await {
                    Ok(pool) => pool,
                    Err(error) => {
                        ChatEvent::new(failed_request_id, run_id, "failed")
                            .error(format!(
                                "Answer completed but chat history could not be saved: {error}"
                            ))
                            .emit(&failed_handle);
                        return;
                    }
                };

                if let Err(error) =
                    persist_chat_exchange(&pool, run_id, &question, &completion.text).await
                {
                    ChatEvent::new(failed_request_id, run_id, "failed")
                        .error(format!(
                            "Answer completed but chat history could not be saved: {error}"
                        ))
                        .emit(&failed_handle);
                    return;
                }

                ChatEvent::new(completed_request_id, run_id, "completed")
                    .message("Answer completed.".to_string())
                    .emit(&completed_handle);
            }
            Err(LlmRequestError::Failed(error)) => {
                ChatEvent::new(failed_request_id, run_id, "failed")
                    .error(error)
                    .emit(&failed_handle);
            }
            Err(LlmRequestError::Cancelled) => {
                ChatEvent::new(cancelled_request_id, run_id, "cancelled")
                    .message("Answer cancelled.".to_string())
                    .emit(&cancelled_handle);
            }
        }
    });

    Ok(request_id)
}

#[cfg(test)]
mod tests {
    use super::{
        build_chat_request, ensure_completed_chat_context, format_chat_context_messages,
        ChatRequestParams,
    };
    use crate::analysis::models::{AnalysisRunDetail, CorpusMessage};

    fn sample_run() -> AnalysisRunDetail {
        AnalysisRunDetail {
            id: 42,
            run_type: "report".to_string(),
            scope_type: "single_source".to_string(),
            source_id: Some(3),
            source_title: Some("Source".to_string()),
            source_group_id: None,
            source_group_name: None,
            scope_label: "Source".to_string(),
            period_from: 10,
            period_to: 20,
            output_language: "English".to_string(),
            prompt_template_id: Some(1),
            prompt_template_name: Some("Default".to_string()),
            prompt_template_version: 1,
            provider_profile: "default".to_string(),
            provider: "gemini".to_string(),
            model: "gemini-2.5-flash".to_string(),
            youtube_corpus_mode: "transcript_description".to_string(),
            status: "completed".to_string(),
            result_markdown: Some("Saved report".to_string()),
            error: None,
            has_trace_data: true,
            created_at: 1_710_000_500,
            completed_at: Some(1_710_000_600),
            scope_label_snapshot: Some("Source".to_string()),
        }
    }

    fn sample_message() -> CorpusMessage {
        CorpusMessage {
            item_id: 9,
            source_id: 3,
            external_id: "abc".to_string(),
            published_at: 1_710_000_000,
            author: Some("analyst".to_string()),
            content: "A matching source document excerpt".to_string(),
            r#ref: "s3-i9".to_string(),
            item_kind: Some("telegram_message".to_string()),
            source_type: Some("telegram".to_string()),
            source_subtype: None,
            metadata_zstd: None,
        }
    }

    #[test]
    fn completed_chat_context_requires_saved_snapshot_messages() {
        let error = ensure_completed_chat_context(&sample_run(), &[])
            .expect_err("missing snapshot rejects completed chat");

        assert_eq!(
            error.message,
            "This completed analysis run has no saved snapshot context for follow-up chat"
        );
    }

    #[test]
    fn completed_chat_context_accepts_saved_snapshot_messages() {
        ensure_completed_chat_context(&sample_run(), &[sample_message()])
            .expect("snapshot context enables completed chat");
    }

    #[test]
    fn build_chat_request_uses_provider_neutral_source_document_wording() {
        let message = sample_message();
        let context_messages = vec![&message];
        let request = build_chat_request(ChatRequestParams {
            run: &sample_run(),
            profile_id: "default".to_string(),
            scope_label: "Source",
            history: &[],
            question: "What changed?",
            report_markdown: "Saved report",
            context_messages: &context_messages,
            model_override: None,
        });

        assert!(request.messages[0]
            .content
            .contains("saved source analysis report"));
        assert!(request.messages[0]
            .content
            .contains("source document excerpts"));
        assert!(request.messages[0].content.contains("[s12-i845]"));
        assert!(request.messages[1]
            .content
            .contains("Additional local source document matches"));
    }

    #[test]
    fn empty_chat_context_uses_source_document_wording() {
        let text = format_chat_context_messages(&[]);

        assert!(text.contains("source document"));
        assert!(!text.contains("message"));
    }
}
