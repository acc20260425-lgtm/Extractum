use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use std::io::Cursor;
use tauri::{AppHandle, Emitter};

use crate::db::get_pool;
use crate::llm::{
    resolve_effective_model, resolve_profile_for_backend, run_llm_collect_with_profile,
    run_llm_stream_with_profile, LlmChatRequest, LlmMessage,
};

const TEMPLATE_KIND_REPORT: &str = "report";
const TEMPLATE_KIND_CHAT: &str = "chat";
const DEFAULT_REPORT_TEMPLATE_NAME: &str = "Default report";
const ANALYSIS_RUN_EVENT: &str = "analysis://run";
const ANALYSIS_RUN_TYPE_REPORT: &str = "report";
const ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE: &str = "single_source";
const ANALYSIS_STATUS_QUEUED: &str = "queued";
const ANALYSIS_STATUS_RUNNING: &str = "running";
const ANALYSIS_STATUS_COMPLETED: &str = "completed";
const ANALYSIS_STATUS_FAILED: &str = "failed";
const ANALYSIS_CHUNK_TARGET_CHARS: usize = 16_000;

#[derive(Serialize, FromRow)]
pub struct AnalysisSourceOption {
    pub id: i64,
    pub account_id: Option<i64>,
    pub title: Option<String>,
    pub item_count: i64,
    pub last_synced_at: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisPromptTemplate {
    pub id: i64,
    pub name: String,
    pub template_kind: String,
    pub body: String,
    pub version: i64,
    pub is_builtin: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisTraceRef {
    pub r#ref: String,
    pub item_id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub published_at: i64,
    pub excerpt: String,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisTraceData {
    pub refs: Vec<AnalysisTraceRef>,
}

#[derive(Serialize)]
pub struct AnalysisRunSummary {
    pub id: i64,
    pub run_type: String,
    pub scope_type: String,
    pub source_id: Option<i64>,
    pub source_title: Option<String>,
    pub period_from: i64,
    pub period_to: i64,
    pub output_language: String,
    pub prompt_template_id: Option<i64>,
    pub prompt_template_name: Option<String>,
    pub prompt_template_version: i64,
    pub provider_profile: String,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub error: Option<String>,
    pub has_trace_data: bool,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Serialize)]
pub struct AnalysisRunDetail {
    pub id: i64,
    pub run_type: String,
    pub scope_type: String,
    pub source_id: Option<i64>,
    pub source_title: Option<String>,
    pub period_from: i64,
    pub period_to: i64,
    pub output_language: String,
    pub prompt_template_id: Option<i64>,
    pub prompt_template_name: Option<String>,
    pub prompt_template_version: i64,
    pub provider_profile: String,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub result_markdown: Option<String>,
    pub error: Option<String>,
    pub has_trace_data: bool,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(FromRow)]
struct AnalysisRunRow {
    id: i64,
    run_type: String,
    scope_type: String,
    source_id: Option<i64>,
    source_title: Option<String>,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template_id: Option<i64>,
    prompt_template_name: Option<String>,
    prompt_template_version: i64,
    provider_profile: String,
    provider: String,
    model: String,
    status: String,
    result_markdown: Option<String>,
    trace_data_zstd: Option<Vec<u8>>,
    error: Option<String>,
    created_at: i64,
    completed_at: Option<i64>,
}

#[derive(Serialize)]
pub struct AnalysisRunEvent {
    pub run_id: i64,
    pub kind: String,
    pub phase: String,
    pub message: Option<String>,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub delta: Option<String>,
    pub error: Option<String>,
}

#[derive(FromRow)]
struct StoredAnalysisItemRow {
    id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    content_zstd: Option<Vec<u8>>,
}

#[derive(Clone)]
struct CorpusMessage {
    item_id: i64,
    source_id: i64,
    external_id: String,
    published_at: i64,
    author: Option<String>,
    content: String,
    r#ref: String,
}

#[derive(Deserialize)]
struct ChunkSummary {
    summary: String,
    topics: Vec<String>,
    notable_points: Vec<String>,
    candidate_refs: Vec<String>,
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn emit_analysis_event(handle: &AppHandle, event: &AnalysisRunEvent) {
    let _ = handle.emit(ANALYSIS_RUN_EVENT, event);
}

fn validate_template_kind(template_kind: &str) -> Result<String, String> {
    let normalized = template_kind.trim().to_ascii_lowercase();
    match normalized.as_str() {
        TEMPLATE_KIND_REPORT | TEMPLATE_KIND_CHAT => Ok(normalized),
        _ => Err(format!("Unsupported template kind '{template_kind}'")),
    }
}

fn validate_template_input(
    name: &str,
    template_kind: &str,
    body: &str,
) -> Result<(String, String, String), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Template name cannot be empty".to_string());
    }

    let template_kind = validate_template_kind(template_kind)?;

    let body = body.trim().to_string();
    if body.is_empty() {
        return Err("Template body cannot be empty".to_string());
    }

    Ok((name, template_kind, body))
}

fn default_report_template_body() -> &'static str {
    r#"Create a grounded report over the provided Telegram messages.

Focus on:
- the main topics and recurring themes
- the most notable claims, updates, and shifts
- supporting examples from the source material

Always keep the report concise, readable, and useful for later follow-up analysis."#
}

fn decompress_text(bytes: &[u8]) -> Result<String, String> {
    let decoded = zstd::decode_all(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    String::from_utf8(decoded).map_err(|e| e.to_string())
}

async fn builtin_report_template_exists(pool: &Pool<Sqlite>) -> Result<bool, String> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM analysis_prompt_templates
            WHERE is_builtin = 1 AND template_kind = ?
        )
        "#,
    )
    .bind(TEMPLATE_KIND_REPORT)
    .fetch_one(pool)
    .await
    .map(|exists| exists != 0)
    .map_err(|e| e.to_string())
}

async fn ensure_builtin_report_template(pool: &Pool<Sqlite>) -> Result<(), String> {
    if builtin_report_template_exists(pool).await? {
        return Ok(());
    }

    let now = now_secs();
    sqlx::query(
        r#"
        INSERT INTO analysis_prompt_templates (
            name,
            template_kind,
            body,
            version,
            is_builtin,
            created_at,
            updated_at
        )
        VALUES (?, ?, ?, 1, 1, ?, ?)
        "#,
    )
    .bind(DEFAULT_REPORT_TEMPLATE_NAME)
    .bind(TEMPLATE_KIND_REPORT)
    .bind(default_report_template_body())
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[allow(dead_code)]
fn compress_trace_data(trace_data: &AnalysisTraceData) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(trace_data).map_err(|e| e.to_string())?;
    zstd::encode_all(Cursor::new(json), 3).map_err(|e| e.to_string())
}

fn decode_trace_data(bytes: Option<&[u8]>) -> Result<AnalysisTraceData, String> {
    let Some(bytes) = bytes else {
        return Ok(AnalysisTraceData::default());
    };

    let decoded = zstd::decode_all(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    serde_json::from_slice(&decoded).map_err(|e| e.to_string())
}

fn map_run_summary(row: AnalysisRunRow) -> AnalysisRunSummary {
    AnalysisRunSummary {
        id: row.id,
        run_type: row.run_type,
        scope_type: row.scope_type,
        source_id: row.source_id,
        source_title: row.source_title,
        period_from: row.period_from,
        period_to: row.period_to,
        output_language: row.output_language,
        prompt_template_id: row.prompt_template_id,
        prompt_template_name: row.prompt_template_name,
        prompt_template_version: row.prompt_template_version,
        provider_profile: row.provider_profile,
        provider: row.provider,
        model: row.model,
        status: row.status,
        error: row.error,
        has_trace_data: row.trace_data_zstd.is_some(),
        created_at: row.created_at,
        completed_at: row.completed_at,
    }
}

fn map_run_detail(row: AnalysisRunRow) -> AnalysisRunDetail {
    AnalysisRunDetail {
        id: row.id,
        run_type: row.run_type,
        scope_type: row.scope_type,
        source_id: row.source_id,
        source_title: row.source_title,
        period_from: row.period_from,
        period_to: row.period_to,
        output_language: row.output_language,
        prompt_template_id: row.prompt_template_id,
        prompt_template_name: row.prompt_template_name,
        prompt_template_version: row.prompt_template_version,
        provider_profile: row.provider_profile,
        provider: row.provider,
        model: row.model,
        status: row.status,
        result_markdown: row.result_markdown,
        error: row.error,
        has_trace_data: row.trace_data_zstd.is_some(),
        created_at: row.created_at,
        completed_at: row.completed_at,
    }
}

async fn fetch_run_row(pool: &Pool<Sqlite>, run_id: i64) -> Result<Option<AnalysisRunRow>, String> {
    sqlx::query_as(
        r#"
        SELECT
            runs.id,
            runs.run_type,
            runs.scope_type,
            runs.source_id,
            sources.title AS source_title,
            runs.period_from,
            runs.period_to,
            runs.output_language,
            runs.prompt_template_id,
            templates.name AS prompt_template_name,
            runs.prompt_template_version,
            runs.provider_profile,
            runs.provider,
            runs.model,
            runs.status,
            runs.result_markdown,
            runs.trace_data_zstd,
            runs.error,
            runs.created_at,
            runs.completed_at
        FROM analysis_runs runs
        LEFT JOIN sources ON sources.id = runs.source_id
        LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
        WHERE runs.id = ?
        "#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
}

async fn fetch_prompt_template(
    pool: &Pool<Sqlite>,
    template_id: i64,
) -> Result<AnalysisPromptTemplate, String> {
    ensure_builtin_report_template(pool).await?;

    sqlx::query_as(
        r#"
        SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
        FROM analysis_prompt_templates
        WHERE id = ?
        "#,
    )
    .bind(template_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Analysis prompt template {template_id} not found"))
}

async fn insert_analysis_run(
    pool: &Pool<Sqlite>,
    source_id: i64,
    period_from: i64,
    period_to: i64,
    output_language: &str,
    prompt_template: &AnalysisPromptTemplate,
    provider_profile: &str,
    provider: &str,
    model: &str,
) -> Result<i64, String> {
    let created_at = now_secs();
    sqlx::query_scalar(
        r#"
        INSERT INTO analysis_runs (
            run_type,
            scope_type,
            source_id,
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            prompt_template_version,
            provider_profile,
            provider,
            model,
            status,
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE)
    .bind(source_id)
    .bind(period_from)
    .bind(period_to)
    .bind(output_language)
    .bind(prompt_template.id)
    .bind(prompt_template.version)
    .bind(provider_profile)
    .bind(provider)
    .bind(model)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())
}

async fn set_run_status(
    pool: &Pool<Sqlite>,
    run_id: i64,
    status: &str,
    result_markdown: Option<&str>,
    trace_data_zstd: Option<&[u8]>,
    error: Option<&str>,
    completed_at: Option<i64>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET
            status = ?,
            result_markdown = COALESCE(?, result_markdown),
            trace_data_zstd = COALESCE(?, trace_data_zstd),
            error = ?,
            completed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(result_markdown)
    .bind(trace_data_zstd)
    .bind(error)
    .bind(completed_at)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

async fn load_corpus_messages(
    pool: &Pool<Sqlite>,
    source_id: i64,
    period_from: i64,
    period_to: i64,
) -> Result<Vec<CorpusMessage>, String> {
    let rows: Vec<StoredAnalysisItemRow> = sqlx::query_as(
        r#"
        SELECT id, source_id, external_id, author, published_at, content_zstd
        FROM items
        WHERE source_id = ? AND published_at >= ? AND published_at <= ?
        ORDER BY published_at ASC, id ASC
        "#,
    )
    .bind(source_id)
    .bind(period_from)
    .bind(period_to)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|row| {
            let content = decompress_text(
                row.content_zstd
                    .as_deref()
                    .ok_or_else(|| format!("Item {} is missing content", row.id))?,
            )?;

            Ok(CorpusMessage {
                item_id: row.id,
                source_id: row.source_id,
                external_id: row.external_id.clone(),
                published_at: row.published_at,
                author: row.author,
                r#ref: format!("s{}-m{}", row.source_id, row.external_id),
                content,
            })
        })
        .collect()
}

async fn load_source_title(pool: &Pool<Sqlite>, source_id: i64) -> Result<String, String> {
    sqlx::query_scalar::<_, Option<String>>("SELECT title FROM sources WHERE id = ?")
        .bind(source_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .flatten()
        .filter(|title| !title.trim().is_empty())
        .ok_or_else(|| format!("Source {source_id} title is not available"))
}

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
    let start = text.find('{').ok_or("LLM response did not contain JSON")?;
    let end = text.rfind('}').ok_or("LLM response did not contain a closing JSON object")?;
    if end < start {
        return Err("LLM response contained malformed JSON boundaries".to_string());
    }
    Ok(&text[start..=end])
}

fn parse_chunk_summary(text: &str) -> Result<ChunkSummary, String> {
    let payload = extract_json_payload(text)?;
    serde_json::from_str(payload).map_err(|e| format!("Failed to parse chunk summary JSON: {e}"))
}

fn normalize_ref(candidate: &str) -> Option<String> {
    let candidate = candidate.trim().trim_matches('[').trim_matches(']');
    let (source_part, message_part) = candidate.split_once("-m")?;
    if !source_part.starts_with('s') {
        return None;
    }
    let source_digits = &source_part[1..];
    if source_digits.is_empty()
        || message_part.is_empty()
        || !source_digits.chars().all(|c| c.is_ascii_digit())
        || !message_part.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }

    Some(format!("s{source_digits}-m{message_part}"))
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
    source_title: &str,
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
                    "Source title: {source_title}\nPeriod: {period_from} to {period_to}\n\nUser report template:\n{template}\n\nChunk summaries:\n\n{combined}",
                    template = prompt_template.body
                ),
            },
        ],
    }
}

fn extract_cited_refs(markdown: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut cursor = 0usize;

    while let Some(relative_start) = markdown[cursor..].find('[') {
        let start = cursor + relative_start;
        let Some(relative_end) = markdown[start + 1..].find(']') else {
            break;
        };
        let end = start + 1 + relative_end;
        let inside = &markdown[start + 1..end];
        for part in inside.split(',') {
            if let Some(reference) = normalize_ref(part) {
                if !refs.contains(&reference) {
                    refs.push(reference);
                }
            }
        }
        cursor = end + 1;
    }

    refs
}

fn build_trace_data(markdown: &str, corpus: &[CorpusMessage]) -> AnalysisTraceData {
    let refs = extract_cited_refs(markdown);
    let mut trace_refs = Vec::new();

    for reference in refs {
        if let Some(message) = corpus.iter().find(|message| message.r#ref == reference) {
            let excerpt = if message.content.len() > 480 {
                format!("{}...", &message.content[..480])
            } else {
                message.content.clone()
            };

            trace_refs.push(AnalysisTraceRef {
                r#ref: reference,
                item_id: message.item_id,
                source_id: message.source_id,
                external_id: message.external_id.clone(),
                published_at: message.published_at,
                excerpt,
            });
        }
    }

    AnalysisTraceData { refs: trace_refs }
}

async fn run_report_pipeline(
    handle: AppHandle,
    run_id: i64,
    source_id: i64,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template: AnalysisPromptTemplate,
    model_override: Option<String>,
    profile_id: Option<String>,
) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    set_run_status(
        &pool,
        run_id,
        ANALYSIS_STATUS_RUNNING,
        None,
        None,
        None,
        None,
    )
    .await?;

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

    let corpus = load_corpus_messages(&pool, source_id, period_from, period_to).await?;
    if corpus.is_empty() {
        return Err("No synced messages were found for the selected source and period".to_string());
    }

    let source_title = load_source_title(&pool, source_id)
        .await
        .unwrap_or_else(|_| format!("Source {source_id}"));

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
        &source_title,
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
pub async fn list_analysis_sources(handle: AppHandle) -> Result<Vec<AnalysisSourceOption>, String> {
    let pool = get_pool(&handle).await?;
    sqlx::query_as(
        r#"
        SELECT
            sources.id,
            sources.account_id,
            sources.title,
            COUNT(items.id) AS item_count,
            sources.last_synced_at
        FROM sources
        LEFT JOIN items ON items.source_id = sources.id
        GROUP BY sources.id, sources.account_id, sources.title, sources.last_synced_at
        ORDER BY sources.created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_analysis_prompt_templates(
    handle: AppHandle,
    template_kind: Option<String>,
) -> Result<Vec<AnalysisPromptTemplate>, String> {
    let pool = get_pool(&handle).await?;
    ensure_builtin_report_template(&pool).await?;

    if let Some(template_kind) = template_kind {
        let template_kind = validate_template_kind(&template_kind)?;
        sqlx::query_as(
            r#"
            SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
            FROM analysis_prompt_templates
            WHERE template_kind = ?
            ORDER BY is_builtin DESC, updated_at DESC, id DESC
            "#,
        )
        .bind(template_kind)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
    } else {
        sqlx::query_as(
            r#"
            SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
            FROM analysis_prompt_templates
            ORDER BY template_kind ASC, is_builtin DESC, updated_at DESC, id DESC
            "#,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn create_analysis_prompt_template(
    handle: AppHandle,
    name: String,
    template_kind: String,
    body: String,
) -> Result<AnalysisPromptTemplate, String> {
    let pool = get_pool(&handle).await?;
    let (name, template_kind, body) = validate_template_input(&name, &template_kind, &body)?;
    let now = now_secs();

    sqlx::query_as(
        r#"
        INSERT INTO analysis_prompt_templates (
            name,
            template_kind,
            body,
            version,
            is_builtin,
            created_at,
            updated_at
        )
        VALUES (?, ?, ?, 1, 0, ?, ?)
        RETURNING id, name, template_kind, body, version, is_builtin, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(template_kind)
    .bind(body)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_analysis_prompt_template(
    handle: AppHandle,
    template_id: i64,
    name: String,
    body: String,
) -> Result<AnalysisPromptTemplate, String> {
    let pool = get_pool(&handle).await?;
    let existing: AnalysisPromptTemplate = sqlx::query_as(
        r#"
        SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
        FROM analysis_prompt_templates
        WHERE id = ?
        "#,
    )
    .bind(template_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Analysis prompt template {template_id} not found"))?;

    if existing.is_builtin {
        return Err("Built-in templates cannot be edited directly".to_string());
    }

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Template name cannot be empty".to_string());
    }

    let body = body.trim().to_string();
    if body.is_empty() {
        return Err("Template body cannot be empty".to_string());
    }

    let now = now_secs();
    sqlx::query_as(
        r#"
        UPDATE analysis_prompt_templates
        SET
            name = ?,
            body = ?,
            version = version + 1,
            updated_at = ?
        WHERE id = ?
        RETURNING id, name, template_kind, body, version, is_builtin, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(body)
    .bind(now)
    .bind(template_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_analysis_prompt_template(
    handle: AppHandle,
    template_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    let template: Option<(i64, bool)> = sqlx::query_as(
        "SELECT id, is_builtin FROM analysis_prompt_templates WHERE id = ?",
    )
    .bind(template_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let Some((_, is_builtin)) = template else {
        return Err(format!("Analysis prompt template {template_id} not found"));
    };

    if is_builtin {
        return Err("Built-in templates cannot be deleted".to_string());
    }

    sqlx::query("DELETE FROM analysis_prompt_templates WHERE id = ?")
        .bind(template_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_analysis_runs(
    handle: AppHandle,
    source_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<AnalysisRunSummary>, String> {
    let pool = get_pool(&handle).await?;
    let limit = limit.unwrap_or(20).clamp(1, 100);

    let rows: Vec<AnalysisRunRow> = if let Some(source_id) = source_id {
        sqlx::query_as(
            r#"
            SELECT
                runs.id,
                runs.run_type,
                runs.scope_type,
                runs.source_id,
                sources.title AS source_title,
                runs.period_from,
                runs.period_to,
                runs.output_language,
                runs.prompt_template_id,
                templates.name AS prompt_template_name,
                runs.prompt_template_version,
                runs.provider_profile,
                runs.provider,
                runs.model,
                runs.status,
                runs.result_markdown,
                runs.trace_data_zstd,
                runs.error,
                runs.created_at,
                runs.completed_at
            FROM analysis_runs runs
            LEFT JOIN sources ON sources.id = runs.source_id
            LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
            WHERE runs.source_id = ?
            ORDER BY runs.created_at DESC
            LIMIT ?
            "#,
        )
        .bind(source_id)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            r#"
            SELECT
                runs.id,
                runs.run_type,
                runs.scope_type,
                runs.source_id,
                sources.title AS source_title,
                runs.period_from,
                runs.period_to,
                runs.output_language,
                runs.prompt_template_id,
                templates.name AS prompt_template_name,
                runs.prompt_template_version,
                runs.provider_profile,
                runs.provider,
                runs.model,
                runs.status,
                runs.result_markdown,
                runs.trace_data_zstd,
                runs.error,
                runs.created_at,
                runs.completed_at
            FROM analysis_runs runs
            LEFT JOIN sources ON sources.id = runs.source_id
            LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
            ORDER BY runs.created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    Ok(rows.into_iter().map(map_run_summary).collect())
}

#[tauri::command]
pub async fn get_analysis_run(
    handle: AppHandle,
    run_id: i64,
) -> Result<Option<AnalysisRunDetail>, String> {
    let pool = get_pool(&handle).await?;
    Ok(fetch_run_row(&pool, run_id)
        .await?
        .map(map_run_detail))
}

#[tauri::command]
pub async fn get_analysis_run_trace(
    handle: AppHandle,
    run_id: i64,
) -> Result<AnalysisTraceData, String> {
    let pool = get_pool(&handle).await?;
    let row = fetch_run_row(&pool, run_id)
        .await?
        .ok_or_else(|| format!("Analysis run {run_id} not found"))?;

    decode_trace_data(row.trace_data_zstd.as_deref())
}

#[tauri::command]
pub async fn start_analysis_report(
    handle: AppHandle,
    source_id: i64,
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

    let pool = get_pool(&handle).await?;
    let prompt_template = fetch_prompt_template(&pool, prompt_template_id).await?;
    if prompt_template.template_kind != TEMPLATE_KIND_REPORT {
        return Err("Selected prompt template is not a report template".to_string());
    }

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

    let resolved_profile = resolve_profile_for_backend(&handle, profile_id.as_deref()).await?;
    let effective_model = resolve_effective_model(&resolved_profile, model_override.as_deref())?;

    let run_id = insert_analysis_run(
        &pool,
        source_id,
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
            source_id,
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
    use super::{
        compress_trace_data, decode_trace_data, ensure_builtin_report_template, AnalysisTraceData,
        AnalysisTraceRef, TEMPLATE_KIND_REPORT,
    };

    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE analysis_prompt_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                template_kind TEXT NOT NULL,
                body TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                is_builtin BOOLEAN NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create templates");
        sqlx::query(
            r#"
            CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_type TEXT NOT NULL,
                scope_type TEXT NOT NULL,
                source_id INTEGER,
                period_from INTEGER NOT NULL,
                period_to INTEGER NOT NULL,
                output_language TEXT NOT NULL,
                prompt_template_id INTEGER,
                prompt_template_version INTEGER NOT NULL,
                provider_profile TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL,
                result_markdown TEXT,
                trace_data_zstd BLOB,
                error TEXT,
                created_at INTEGER NOT NULL,
                completed_at INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create runs");
        pool
    }

    #[tokio::test]
    async fn builtin_template_is_seeded_once() {
        let pool = memory_pool().await;
        ensure_builtin_report_template(&pool)
            .await
            .expect("seed builtin");
        ensure_builtin_report_template(&pool)
            .await
            .expect("seed builtin twice");

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM analysis_prompt_templates WHERE template_kind = ?",
        )
        .bind(TEMPLATE_KIND_REPORT)
        .fetch_one(&pool)
        .await
        .expect("count report templates");

        assert_eq!(count, 1);
    }

    #[test]
    fn trace_data_roundtrips_through_zstd() {
        let trace = AnalysisTraceData {
            refs: vec![AnalysisTraceRef {
                r#ref: "s12-m845".to_string(),
                item_id: 321,
                source_id: 12,
                external_id: "845".to_string(),
                published_at: 1_710_000_000,
                excerpt: "Example quote".to_string(),
            }],
        };

        let compressed = compress_trace_data(&trace).expect("compress");
        let decoded = decode_trace_data(Some(&compressed)).expect("decode");
        assert_eq!(decoded, trace);
    }
}
