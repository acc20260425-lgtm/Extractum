use std::sync::Arc;

use sqlx::{Pool, Sqlite, SqlitePool};

use extractum_core::error::{AppError, AppResult};
use extractum_llm::{
    run_llm_stream_with_profile, LlmChatRequest, LlmMessage, LlmRequestError, LlmRequestKind,
    LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState, ResolvedLlmProfile,
};

use super::corpus::{load_run_snapshot_messages, AnalysisCorpusMessage};
use super::domain::{now_secs, validate_chat_role, validate_chat_turns, ANALYSIS_STATUS_COMPLETED};
use super::models::{
    AnalysisChatEvent, AnalysisChatMessage, AnalysisChatRun, AnalysisChatTurn, AnalysisEventSink,
    AnalysisRunDetail,
};
use super::report::AnalysisExecutionError;
use super::store::{analysis_run_exists, resolve_run_scope_label};

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
    corpus: &'a [AnalysisCorpusMessage],
) -> Vec<&'a AnalysisCorpusMessage> {
    let terms = chat_search_terms(question);
    if terms.is_empty() {
        return corpus.iter().rev().take(6).collect();
    }

    let mut scored = corpus
        .iter()
        .filter_map(|message| {
            let haystack = message.content().to_ascii_lowercase();
            let score = terms
                .iter()
                .map(|term| usize::from(haystack.contains(term)))
                .sum::<usize>();
            (score > 0).then_some((score, message.published_at(), message))
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

fn history_scope_label_from_metadata(metadata_zstd: &[u8]) -> Option<&'static str> {
    let bytes = extractum_core::compression::decompress_bytes(metadata_zstd).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    match value.get("history_scope").and_then(|value| value.as_str()) {
        Some("migrated") => Some("Migrated small-group history"),
        Some("current") => Some("Current supergroup history"),
        _ => None,
    }
}

fn format_chat_context_messages(messages: &[&AnalysisCorpusMessage]) -> String {
    if messages.is_empty() {
        return "No additional local source document matches were found for the current question."
            .to_string();
    }

    messages
        .iter()
        .map(|message| {
            let history_scope = message
                .metadata_zstd()
                .and_then(history_scope_label_from_metadata)
                .unwrap_or("Current supergroup history");
            format!(
                "[{ref}] Date: {published_at}\nHistory scope: {history_scope}\nAuthor: {author}\nExcerpt:\n{excerpt}",
                ref = message.reference(),
                published_at = message.published_at(),
                history_scope = history_scope,
                author = message.author().unwrap_or("unknown"),
                excerpt = clip_excerpt(message.content(), 420)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

fn ensure_completed_chat_context(
    run: &AnalysisRunDetail,
    snapshot: &[AnalysisCorpusMessage],
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
    context_messages: &'a [&'a AnalysisCorpusMessage],
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
        max_output_tokens: None,
    }
}

fn analysis_chat_request_metadata(
    request: &LlmChatRequest,
    profile_id: String,
    provider: String,
    run_id: i64,
) -> LlmRequestMetadata {
    LlmRequestMetadata {
        request_id: request.request_id.clone(),
        profile_id,
        provider,
        kind: LlmRequestKind::AnalysisChat,
        priority: LlmRequestPriority::Interactive,
        owner_run_id: Some(run_id),
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

    fn publish(self, sink: &dyn AnalysisEventSink) {
        sink.publish_chat(self.event);
    }
}

pub struct AskAnalysisRunQuestionRequest {
    run_id: i64,
    question: String,
    model_override: Option<String>,
    profile_id: Option<String>,
}

impl AskAnalysisRunQuestionRequest {
    pub fn new(
        run_id: i64,
        question: String,
        model_override: Option<String>,
        profile_id: Option<String>,
    ) -> AppResult<Self> {
        let question = question.trim().to_string();
        if question.is_empty() {
            return Err(AppError::validation("Question cannot be empty"));
        }
        Ok(Self {
            run_id,
            question,
            model_override,
            profile_id,
        })
    }
}

pub struct AnalysisChatExecutionTicket {
    request: LlmChatRequest,
    profile_id: String,
    run_id: i64,
    question: String,
}

impl AnalysisChatExecutionTicket {
    pub fn request_id(&self) -> &str {
        &self.request.request_id
    }

    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }
}

pub struct AnalysisChatCompletionTicket {
    request_id: String,
    run_id: i64,
    question: String,
    answer: String,
}

impl AnalysisChatCompletionTicket {
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn run_id(&self) -> i64 {
        self.run_id
    }
}

pub async fn prepare_analysis_chat(
    pool: &SqlitePool,
    request: AskAnalysisRunQuestionRequest,
    run: AnalysisChatRun,
) -> AppResult<AnalysisChatExecutionTicket> {
    let AskAnalysisRunQuestionRequest {
        run_id,
        question,
        model_override,
        profile_id,
    } = request;
    let run = run.into_detail();
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
    let corpus = load_run_snapshot_messages(pool, run.id).await?;
    ensure_completed_chat_context(&run, &corpus)?;
    let context_messages = find_chat_context_messages(&question, &corpus);
    let profile_id = profile_id.unwrap_or_else(|| run.provider_profile.clone());
    let history = load_chat_messages_from_pool(pool, run_id)
        .await?
        .into_iter()
        .map(|message| AnalysisChatTurn {
            role: message.role,
            content: message.content,
        })
        .collect::<Vec<_>>();
    validate_chat_turns(&history)?;
    let llm_request = build_chat_request(ChatRequestParams {
        run: &run,
        profile_id: profile_id.clone(),
        scope_label: &scope_label,
        history: &history,
        question: &question,
        report_markdown: &report_markdown,
        context_messages: &context_messages,
        model_override,
    });
    Ok(AnalysisChatExecutionTicket {
        request: llm_request,
        profile_id,
        run_id,
        question,
    })
}

pub async fn execute_analysis_chat(
    scheduler: Arc<LlmSchedulerState>,
    sink: Arc<dyn AnalysisEventSink>,
    ticket: AnalysisChatExecutionTicket,
    resolved_profile: ResolvedLlmProfile,
) -> Result<AnalysisChatCompletionTicket, AnalysisExecutionError> {
    let AnalysisChatExecutionTicket {
        request,
        profile_id: _,
        run_id,
        question,
    } = ticket;
    let request_id = request.request_id.clone();
    let request_meta = analysis_chat_request_metadata(
        &request,
        resolved_profile.profile_id().to_string(),
        resolved_profile.provider().as_str().to_string(),
        run_id,
    );
    let queued_sink = sink.clone();
    let started_sink = sink.clone();
    let delta_sink = sink;
    let queued_request_id = request_id.clone();
    let started_request_id = request_id.clone();
    let delta_request_id = request_id.clone();
    let scheduled_request = request;
    let scheduled_profile = resolved_profile;

    match scheduler
        .as_ref()
        .run_request(
            request_meta,
            move |position| {
                ChatEvent::new(queued_request_id.clone(), run_id, "queued")
                    .queue_position(position)
                    .message(format!("Answer queued at position {position}..."))
                    .publish(queued_sink.as_ref());
            },
            move |control| async move {
                ChatEvent::new(started_request_id, run_id, "started")
                    .message("Preparing grounded answer...".to_string())
                    .publish(started_sink.as_ref());

                control
                    .run_cancellable(run_llm_stream_with_profile(
                        &scheduled_request,
                        &scheduled_profile,
                        |delta| {
                            ChatEvent::new(delta_request_id.clone(), run_id, "delta")
                                .delta(delta.to_string())
                                .publish(delta_sink.as_ref());
                        },
                    ))
                    .await
            },
        )
        .await
    {
        Ok(completion) => Ok(AnalysisChatCompletionTicket {
            request_id,
            run_id,
            question,
            answer: completion.text,
        }),
        Err(LlmRequestError::Failed(error)) => {
            Err(AnalysisExecutionError::Failed(error.to_string()))
        }
        Err(LlmRequestError::Cancelled) => Err(AnalysisExecutionError::Cancelled(
            "Answer cancelled.".to_string(),
        )),
    }
}

pub async fn complete_analysis_chat(
    pool: &SqlitePool,
    sink: &dyn AnalysisEventSink,
    completion: AnalysisChatCompletionTicket,
) -> AppResult<()> {
    persist_chat_exchange(
        pool,
        completion.run_id,
        &completion.question,
        &completion.answer,
    )
    .await?;
    ChatEvent::new(completion.request_id, completion.run_id, "completed")
        .message("Answer completed.".to_string())
        .publish(sink);
    Ok(())
}

pub fn publish_analysis_chat_execution_error(
    sink: &dyn AnalysisEventSink,
    request_id: &str,
    run_id: i64,
    error: &AnalysisExecutionError,
) {
    match error {
        AnalysisExecutionError::Cancelled(message) => {
            ChatEvent::new(request_id.to_string(), run_id, "cancelled")
                .message(message.clone())
                .publish(sink);
        }
        AnalysisExecutionError::CaptureFailed(error) | AnalysisExecutionError::Failed(error) => {
            ChatEvent::new(request_id.to_string(), run_id, "failed")
                .error(error.clone())
                .publish(sink);
        }
    }
}

pub fn publish_analysis_chat_persistence_error(
    sink: &dyn AnalysisEventSink,
    request_id: &str,
    run_id: i64,
    error: &AppError,
) {
    ChatEvent::new(request_id.to_string(), run_id, "failed")
        .error(completed_chat_persistence_failure_message(error))
        .publish(sink);
}

pub async fn list_analysis_chat_messages_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<AnalysisChatMessage>> {
    if !analysis_run_exists(pool, run_id).await? {
        return Err(AppError::not_found(format!(
            "Analysis run {run_id} not found"
        )));
    }
    load_chat_messages_from_pool(pool, run_id).await
}

pub async fn clear_analysis_chat_messages_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<()> {
    if !analysis_run_exists(pool, run_id).await? {
        return Err(AppError::not_found(format!(
            "Analysis run {run_id} not found"
        )));
    }

    sqlx::query("DELETE FROM analysis_chat_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
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

fn completed_chat_persistence_failure_message(error: &impl std::fmt::Display) -> String {
    format!("Answer completed but chat history could not be saved: {error}")
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};

    use super::super::corpus::AnalysisCorpusMessage;
    use super::super::models::{
        AnalysisChatEvent, AnalysisChatRun, AnalysisEventSink, AnalysisRunDetail, AnalysisRunEvent,
    };
    use super::{
        analysis_chat_request_metadata, build_chat_request, complete_analysis_chat,
        completed_chat_persistence_failure_message, ensure_completed_chat_context,
        clear_analysis_chat_messages_in_pool, execute_analysis_chat, format_chat_context_messages,
        list_analysis_chat_messages_in_pool, prepare_analysis_chat,
        publish_analysis_chat_execution_error, publish_analysis_chat_persistence_error,
        AskAnalysisRunQuestionRequest, ChatRequestParams,
    };
    use extractum_core::compression::compress_text;
    use extractum_llm::{
        LlmProviderAccess, LlmRequestKind, LlmSchedulerState, ProviderKind, ResolvedLlmProfile,
    };

    fn read_http_request(stream: &mut TcpStream) {
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let read = stream.read(&mut chunk).expect("read chat request");
            assert!(read > 0, "chat request ended before headers");
            buffer.extend_from_slice(&chunk[..read]);
            if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                break position + 4;
            }
        };
        let headers = String::from_utf8_lossy(&buffer[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);
        let body_read = buffer.len().saturating_sub(header_end);
        if content_length > body_read {
            let mut body_tail = vec![0_u8; content_length - body_read];
            stream
                .read_exact(&mut body_tail)
                .expect("read chat request body");
        }
    }

    fn start_chat_completion_server(
        completion: &'static str,
    ) -> (String, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind chat completion server");
        let base_url = format!(
            "http://{}/v1",
            listener.local_addr().expect("chat completion address")
        );
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept chat completion request");
            read_http_request(&mut stream);
            let payload = serde_json::json!({
                "choices": [{"delta": {"content": completion}}],
            });
            let body = format!("data: {payload}\n\ndata: [DONE]\n\n");
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            stream
                .write_all(response.as_bytes())
                .expect("write chat completion response");
        });
        (base_url, server)
    }

    struct RecordingChatEventSink {
        pool: sqlx::SqlitePool,
        events: Mutex<Vec<AnalysisChatEvent>>,
        rows_seen_at_completed: Mutex<Vec<(String, String)>>,
    }

    impl RecordingChatEventSink {
        fn new(pool: sqlx::SqlitePool) -> Self {
            Self {
                pool,
                events: Mutex::new(Vec::new()),
                rows_seen_at_completed: Mutex::new(Vec::new()),
            }
        }

        fn take_events(&self) -> Vec<AnalysisChatEvent> {
            std::mem::take(&mut *self.events.lock().expect("chat events lock"))
        }

        fn rows_seen_at_completed(&self) -> Vec<(String, String)> {
            self.rows_seen_at_completed
                .lock()
                .expect("completed rows lock")
                .clone()
        }
    }

    impl AnalysisEventSink for RecordingChatEventSink {
        fn publish_run(&self, _event: AnalysisRunEvent) {}

        fn publish_chat(&self, event: AnalysisChatEvent) {
            if event.kind == "completed" {
                let pool = self.pool.clone();
                let rows = std::thread::spawn(move || {
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("completed-event runtime")
                        .block_on(async move {
                            sqlx::query_as::<_, (String, String)>(
                                "SELECT role, content FROM analysis_chat_messages ORDER BY id ASC",
                            )
                            .fetch_all(&pool)
                            .await
                            .expect("load rows at completed event")
                        })
                })
                .join()
                .expect("completed-event persistence witness");
                *self
                    .rows_seen_at_completed
                    .lock()
                    .expect("completed rows lock") = rows;
            }
            self.events.lock().expect("chat events lock").push(event);
        }
    }

    fn sample_run() -> AnalysisRunDetail {
        AnalysisRunDetail {
            id: 42,
            run_type: "report".to_string(),
            scope_type: "single_source".to_string(),
            source_id: Some(3),
            source_title: Some("Source".to_string()),
            source_group_id: None,
            source_group_name: None,
            project_id: None,
            project_name: None,
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
            telegram_history_scope: "current".to_string(),
            status: "completed".to_string(),
            result_markdown: Some("Saved report".to_string()),
            error: None,
            has_trace_data: true,
            snapshot_state: Some(super::super::models::AnalysisSnapshotState::Captured),
            snapshot_captured_at: Some("2026-05-18T10:00:00Z".to_string()),
            snapshot_error: None,
            created_at: 1_710_000_500,
            completed_at: Some(1_710_000_600),
            scope_label_snapshot: Some("Source".to_string()),
            snapshot_message_count: 1,
        }
    }

    fn sample_message() -> AnalysisCorpusMessage {
        AnalysisCorpusMessage::new(
            9,
            3,
            "abc".to_string(),
            1_710_000_000,
            Some("analyst".to_string()),
            "A matching source document excerpt".to_string(),
            "s3-i9".to_string(),
            Some("telegram_message".to_string()),
            Some("telegram".to_string()),
            None,
            None,
        )
    }

    #[test]
    fn chat_context_labels_migrated_history_scope_from_metadata() {
        let message = AnalysisCorpusMessage::new(
            9,
            3,
            "abc".to_string(),
            1_710_000_000,
            Some("analyst".to_string()),
            "A matching source document excerpt".to_string(),
            "s3-i9".to_string(),
            Some("telegram_message".to_string()),
            Some("telegram".to_string()),
            None,
            Some(
                extractum_core::compression::compress_json_bytes(
                    br#"{"history_scope":"migrated"}"#,
                )
                .expect("compress metadata"),
            ),
        );

        let text = format_chat_context_messages(&[&message]);

        assert!(text.contains("History scope: Migrated small-group history"));
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
    fn analysis_chat_request_metadata_uses_run_owner() {
        let request = build_chat_request(ChatRequestParams {
            run: &sample_run(),
            profile_id: "default".to_string(),
            scope_label: "Source",
            history: &[],
            question: "What changed?",
            report_markdown: "Saved report",
            context_messages: &[],
            model_override: None,
        });
        let metadata = analysis_chat_request_metadata(
            &request,
            "default".to_string(),
            "gemini".to_string(),
            42,
        );

        assert_eq!(metadata.kind, LlmRequestKind::AnalysisChat);
        assert_eq!(metadata.owner_run_id, Some(42));
    }

    #[test]
    fn empty_chat_context_uses_source_document_wording() {
        let text = format_chat_context_messages(&[]);

        assert!(text.contains("source document"));
        assert!(!text.contains("message"));
    }

    #[tokio::test]
    async fn chat_persistence_failure_keeps_completed_answer_failure_message() {
        let error = extractum_core::error::AppError::database("insert failed");

        assert_eq!(
            completed_chat_persistence_failure_message(&error),
            "Answer completed but chat history could not be saved: Database error: insert failed"
        );
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect failure-event pool");
        sqlx::query("CREATE TABLE analysis_runs (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("create failure-event runs");
        sqlx::query(
            "CREATE TABLE analysis_chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create failure-event chat messages");
        let list_error = match list_analysis_chat_messages_in_pool(&pool, 77).await {
            Ok(_) => panic!("missing run list should fail"),
            Err(error) => error,
        };
        let clear_error = clear_analysis_chat_messages_in_pool(&pool, 77)
            .await
            .expect_err("missing run clear should fail");
        for error in [list_error, clear_error] {
            assert_eq!(error.kind, extractum_core::error::AppErrorKind::NotFound);
            assert_eq!(error.message, "Analysis run 77 not found");
        }
        let sink = RecordingChatEventSink::new(pool);
        publish_analysis_chat_persistence_error(&sink, "request-1", 77, &error);
        publish_analysis_chat_execution_error(
            &sink,
            "request-2",
            77,
            &super::AnalysisExecutionError::Failed("provider failed".to_string()),
        );
        publish_analysis_chat_execution_error(
            &sink,
            "request-3",
            77,
            &super::AnalysisExecutionError::Cancelled("Answer cancelled.".to_string()),
        );
        assert_eq!(
            sink.take_events()
                .into_iter()
                .map(|event| serde_json::to_value(event).expect("serialize failure event"))
                .collect::<Vec<_>>(),
            vec![
                serde_json::json!({
                    "request_id": "request-1",
                    "run_id": 77,
                    "kind": "failed",
                    "queue_position": null,
                    "delta": null,
                    "message": null,
                    "error": "Answer completed but chat history could not be saved: Database error: insert failed",
                }),
                serde_json::json!({
                    "request_id": "request-2",
                    "run_id": 77,
                    "kind": "failed",
                    "queue_position": null,
                    "delta": null,
                    "message": null,
                    "error": "provider failed",
                }),
                serde_json::json!({
                    "request_id": "request-3",
                    "run_id": 77,
                    "kind": "cancelled",
                    "queue_position": null,
                    "delta": null,
                    "message": "Answer cancelled.",
                    "error": null,
                }),
            ],
        );
    }

    #[tokio::test]
    async fn chat_execution_persists_turns_before_completed_event() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect chat runtime pool");
        for statement in [
            "CREATE TABLE analysis_runs (id INTEGER PRIMARY KEY)",
            "CREATE TABLE analysis_chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            "CREATE TABLE analysis_run_messages (
                run_id INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                author TEXT,
                published_at INTEGER NOT NULL,
                ref TEXT NOT NULL,
                content_zstd BLOB NOT NULL,
                item_kind TEXT,
                source_type TEXT,
                source_subtype TEXT,
                metadata_zstd BLOB
            )",
        ] {
            sqlx::query(statement)
                .execute(&pool)
                .await
                .expect("create chat runtime schema");
        }
        sqlx::query("INSERT INTO analysis_runs (id) VALUES (77)")
            .execute(&pool)
            .await
            .expect("insert chat runtime run");
        sqlx::query(
            "INSERT INTO analysis_run_messages (
                run_id, item_id, source_id, external_id, author, published_at, ref,
                content_zstd, item_kind, source_type, source_subtype, metadata_zstd
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(77_i64)
        .bind(1_i64)
        .bind(2_i64)
        .bind("source-item-1")
        .bind("analyst")
        .bind(1_750_000_000_i64)
        .bind("s2-i1")
        .bind(compress_text("Grounding source text").expect("compress chat source"))
        .bind("telegram_message")
        .bind("telegram")
        .bind(Option::<String>::None)
        .bind(Option::<Vec<u8>>::None)
        .execute(&pool)
        .await
        .expect("insert chat runtime snapshot");
        let mut run = sample_run();
        run.id = 77;
        run.provider_profile = "research".to_string();
        let request = AskAnalysisRunQuestionRequest::new(
            77,
            "  What changed?  ".to_string(),
            None,
            Some("research".to_string()),
        )
        .expect("construct chat request");
        let ticket = prepare_analysis_chat(&pool, request, AnalysisChatRun::new(run))
            .await
            .expect("prepare chat execution");
        let request_id = ticket.request_id().to_string();
        assert!(request_id.starts_with("analysis-chat-77-"));
        assert_eq!(ticket.profile_id(), "research");
        let (base_url, server) = start_chat_completion_server("Grounded answer [s2-i1].");
        let profile = ResolvedLlmProfile::new(
            "research".to_string(),
            "test-model".to_string(),
            LlmProviderAccess::new(
                ProviderKind::OpenAiCompatible,
                "secret-key".to_string().into(),
                base_url,
            ),
        );
        let sink = Arc::new(RecordingChatEventSink::new(pool.clone()));

        let completion = execute_analysis_chat(
            Arc::new(LlmSchedulerState::new()),
            sink.clone(),
            ticket,
            profile,
        )
        .await
        .expect("execute chat");
        complete_analysis_chat(&pool, sink.as_ref(), completion)
            .await
            .expect("complete chat");

        let events = sink.take_events();
        assert_eq!(
            events
                .iter()
                .map(|event| serde_json::to_value(event).expect("serialize chat event"))
                .collect::<Vec<_>>(),
            vec![
                serde_json::json!({
                    "request_id": request_id.clone(),
                    "run_id": 77,
                    "kind": "queued",
                    "queue_position": 1,
                    "delta": null,
                    "message": "Answer queued at position 1...",
                    "error": null,
                }),
                serde_json::json!({
                    "request_id": request_id.clone(),
                    "run_id": 77,
                    "kind": "started",
                    "queue_position": null,
                    "delta": null,
                    "message": "Preparing grounded answer...",
                    "error": null,
                }),
                serde_json::json!({
                    "request_id": request_id.clone(),
                    "run_id": 77,
                    "kind": "delta",
                    "queue_position": null,
                    "delta": "Grounded answer [s2-i1].",
                    "message": null,
                    "error": null,
                }),
                serde_json::json!({
                    "request_id": request_id,
                    "run_id": 77,
                    "kind": "completed",
                    "queue_position": null,
                    "delta": null,
                    "message": "Answer completed.",
                    "error": null,
                }),
            ],
            "RED: CP4 chat persistence before completed"
        );
        assert!(events
            .iter()
            .all(|event| event.request_id == request_id && event.run_id == 77));
        let expected_rows = vec![
            ("user".to_string(), "What changed?".to_string()),
            (
                "assistant".to_string(),
                "Grounded answer [s2-i1].".to_string(),
            ),
        ];
        assert_eq!(sink.rows_seen_at_completed(), expected_rows);
        let persisted: Vec<(String, String)> =
            sqlx::query_as("SELECT role, content FROM analysis_chat_messages ORDER BY id ASC")
                .fetch_all(&pool)
                .await
                .expect("load persisted chat rows");
        assert_eq!(persisted, expected_rows);
        let listed = list_analysis_chat_messages_in_pool(&pool, 77)
            .await
            .expect("list persisted chat rows");
        assert_eq!(
            listed
                .iter()
                .map(|message| (message.role.as_str(), message.content.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("user", "What changed?"),
                ("assistant", "Grounded answer [s2-i1]."),
            ]
        );
        clear_analysis_chat_messages_in_pool(&pool, 77)
            .await
            .expect("clear persisted chat rows");
        assert!(
            list_analysis_chat_messages_in_pool(&pool, 77)
                .await
                .expect("list cleared chat rows")
                .is_empty()
        );
        server.join().expect("chat completion server");
    }
}
