use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, QueryBuilder, Sqlite};
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
const ANALYSIS_CHAT_EVENT: &str = "analysis://chat";
const ANALYSIS_RUN_TYPE_REPORT: &str = "report";
const ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE: &str = "single_source";
const ANALYSIS_SCOPE_TYPE_SOURCE_GROUP: &str = "source_group";
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

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisSourceGroupMember {
    pub source_id: i64,
    pub source_title: Option<String>,
    pub item_count: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AnalysisSourceGroup {
    pub id: i64,
    pub name: String,
    pub members: Vec<AnalysisSourceGroupMember>,
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
    pub source_group_id: Option<i64>,
    pub source_group_name: Option<String>,
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
    pub source_group_id: Option<i64>,
    pub source_group_name: Option<String>,
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
    source_group_id: Option<i64>,
    source_group_name: Option<String>,
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

#[derive(FromRow)]
struct AnalysisSourceGroupRow {
    id: i64,
    name: String,
    created_at: i64,
    updated_at: i64,
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

#[derive(Serialize)]
pub struct AnalysisChatEvent {
    pub request_id: String,
    pub run_id: i64,
    pub kind: String,
    pub delta: Option<String>,
    pub message: Option<String>,
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisChatTurn {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisChatMessage {
    pub id: i64,
    pub run_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: i64,
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

fn emit_analysis_chat_event(handle: &AppHandle, event: &AnalysisChatEvent) {
    let _ = handle.emit(ANALYSIS_CHAT_EVENT, event);
}

fn validate_template_kind(template_kind: &str) -> Result<String, String> {
    let normalized = template_kind.trim().to_ascii_lowercase();
    match normalized.as_str() {
        TEMPLATE_KIND_REPORT | TEMPLATE_KIND_CHAT => Ok(normalized),
        _ => Err(format!("Unsupported template kind '{template_kind}'")),
    }
}

fn validate_chat_turns(history: &[AnalysisChatTurn]) -> Result<(), String> {
    for turn in history {
        match turn.role.as_str() {
            "user" | "assistant" => {}
            other => return Err(format!("Unsupported chat turn role '{other}'")),
        }
        if turn.content.trim().is_empty() {
            return Err("Chat turns cannot be empty".to_string());
        }
    }

    Ok(())
}

fn validate_chat_role(role: &str) -> Result<(), String> {
    match role {
        "user" | "assistant" => Ok(()),
        other => Err(format!("Unsupported chat role '{other}'")),
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

fn normalize_source_group_input(name: &str, source_ids: Vec<i64>) -> Result<(String, Vec<i64>), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Source group name cannot be empty".to_string());
    }

    let mut source_ids = source_ids
        .into_iter()
        .filter(|source_id| *source_id > 0)
        .collect::<Vec<_>>();
    source_ids.sort_unstable();
    source_ids.dedup();

    if source_ids.is_empty() {
        return Err("Select at least one source for the group".to_string());
    }

    Ok((name, source_ids))
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

async fn ensure_sources_exist(pool: &Pool<Sqlite>, source_ids: &[i64]) -> Result<(), String> {
    for source_id in source_ids {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)",
        )
        .bind(source_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        if exists == 0 {
            return Err(format!("Source {source_id} not found"));
        }
    }

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
        source_group_id: row.source_group_id,
        source_group_name: row.source_group_name,
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
        source_group_id: row.source_group_id,
        source_group_name: row.source_group_name,
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
            runs.source_group_id,
            groups.name AS source_group_name,
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
        LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
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

async fn fetch_source_group(pool: &Pool<Sqlite>, group_id: i64) -> Result<Option<AnalysisSourceGroup>, String> {
    let group = sqlx::query_as::<_, AnalysisSourceGroupRow>(
        r#"
        SELECT id, name, created_at, updated_at
        FROM analysis_source_groups
        WHERE id = ?
        "#,
    )
    .bind(group_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let Some(group) = group else {
        return Ok(None);
    };

    let members = sqlx::query_as::<_, AnalysisSourceGroupMember>(
        r#"
        SELECT
            sources.id AS source_id,
            sources.title AS source_title,
            COUNT(items.id) AS item_count
        FROM analysis_source_group_members members
        JOIN sources ON sources.id = members.source_id
        LEFT JOIN items ON items.source_id = sources.id
        WHERE members.group_id = ?
        GROUP BY sources.id, sources.title
        ORDER BY COALESCE(sources.title, ''), sources.id
        "#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Some(AnalysisSourceGroup {
        id: group.id,
        name: group.name,
        members,
        created_at: group.created_at,
        updated_at: group.updated_at,
    }))
}

async fn resolve_run_source_ids(pool: &Pool<Sqlite>, run: &AnalysisRunDetail) -> Result<Vec<i64>, String> {
    if run.scope_type == ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE {
        let source_id = run
            .source_id
            .ok_or_else(|| format!("Analysis run {} is missing source_id", run.id))?;
        return Ok(vec![source_id]);
    }

    if run.scope_type == ANALYSIS_SCOPE_TYPE_SOURCE_GROUP {
        let group_id = run
            .source_group_id
            .ok_or_else(|| format!("Analysis run {} is missing source_group_id", run.id))?;
        let group = fetch_source_group(pool, group_id)
            .await?
            .ok_or_else(|| format!("Analysis source group {group_id} not found"))?;
        return Ok(group.members.into_iter().map(|member| member.source_id).collect());
    }

    Err(format!("Unsupported analysis scope '{}'", run.scope_type))
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
    sqlx::query(
        r#"
        INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
        VALUES (?, ?, ?, ?), (?, ?, ?, ?)
        "#,
    )
    .bind(run_id)
    .bind("user")
    .bind(user_question)
    .bind(now)
    .bind(run_id)
    .bind("assistant")
    .bind(assistant_answer)
    .bind(now + 1)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

async fn insert_analysis_run(
    pool: &Pool<Sqlite>,
    scope_type: &str,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
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
            source_group_id,
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
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(scope_type)
    .bind(source_id)
    .bind(source_group_id)
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
    source_ids: &[i64],
    period_from: i64,
    period_to: i64,
) -> Result<Vec<CorpusMessage>, String> {
    if source_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT id, source_id, external_id, author, published_at, content_zstd FROM items WHERE published_at >= ",
    );
    query.push_bind(period_from);
    query.push(" AND published_at <= ");
    query.push_bind(period_to);
    query.push(" AND source_id IN (");

    {
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id);
        }
    }

    query.push(") ORDER BY published_at ASC, id ASC");

    let rows: Vec<StoredAnalysisItemRow> = query
        .build_query_as()
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

fn build_trace_refs(refs: &[String], corpus: &[CorpusMessage]) -> Vec<AnalysisTraceRef> {
    let mut trace_refs = Vec::new();

    for reference in refs {
        if let Some(message) = corpus.iter().find(|message| message.r#ref == *reference) {
            let excerpt = if message.content.len() > 480 {
                format!("{}...", &message.content[..480])
            } else {
                message.content.clone()
            };

            trace_refs.push(AnalysisTraceRef {
                r#ref: reference.clone(),
                item_id: message.item_id,
                source_id: message.source_id,
                external_id: message.external_id.clone(),
                published_at: message.published_at,
                excerpt,
            });
        }
    }

    trace_refs
}

fn build_trace_data(markdown: &str, corpus: &[CorpusMessage]) -> AnalysisTraceData {
    let refs = extract_cited_refs(markdown);
    let trace_refs = build_trace_refs(&refs, corpus);

    AnalysisTraceData { refs: trace_refs }
}

fn chat_search_terms(question: &str) -> Vec<String> {
    const STOP_WORDS: &[&str] = &[
        "the", "and", "for", "with", "that", "this", "from", "into", "about", "what", "when",
        "where", "which", "have", "has", "were", "will", "would", "could", "should", "как",
        "что", "это", "для", "про", "или", "если", "когда", "какие", "какой", "где", "после",
        "над", "под", "ещё", "also", "over",
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

fn find_chat_context_messages<'a>(question: &str, corpus: &'a [CorpusMessage]) -> Vec<&'a CorpusMessage> {
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

    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
    });

    scored.into_iter().take(8).map(|(_, _, message)| message).collect()
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
        return "No additional local message matches were found for the current question.".to_string();
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
    let run = get_analysis_run(handle.clone(), run_id)
        .await?
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
        let resolved_profile = match resolve_profile_for_backend(&app_handle, profile_id.as_deref()).await {
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
pub async fn list_analysis_source_groups(handle: AppHandle) -> Result<Vec<AnalysisSourceGroup>, String> {
    let pool = get_pool(&handle).await?;
    let rows = sqlx::query_as::<_, AnalysisSourceGroupRow>(
        r#"
        SELECT id, name, created_at, updated_at
        FROM analysis_source_groups
        ORDER BY updated_at DESC, id DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut groups = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(group) = fetch_source_group(&pool, row.id).await? {
            groups.push(group);
        }
    }

    Ok(groups)
}

#[tauri::command]
pub async fn create_analysis_source_group(
    handle: AppHandle,
    name: String,
    source_ids: Vec<i64>,
) -> Result<AnalysisSourceGroup, String> {
    let pool = get_pool(&handle).await?;
    let (name, source_ids) = normalize_source_group_input(&name, source_ids)?;
    ensure_sources_exist(&pool, &source_ids).await?;

    let now = now_secs();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let group_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO analysis_source_groups (name, created_at, updated_at)
        VALUES (?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(&name)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    for source_id in source_ids {
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(group_id)
        .bind(source_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    fetch_source_group(&pool, group_id)
        .await?
        .ok_or_else(|| format!("Analysis source group {group_id} not found after creation"))
}

#[tauri::command]
pub async fn update_analysis_source_group(
    handle: AppHandle,
    group_id: i64,
    name: String,
    source_ids: Vec<i64>,
) -> Result<AnalysisSourceGroup, String> {
    let pool = get_pool(&handle).await?;
    let (name, source_ids) = normalize_source_group_input(&name, source_ids)?;
    ensure_sources_exist(&pool, &source_ids).await?;

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS(SELECT 1 FROM analysis_source_groups WHERE id = ?)",
    )
    .bind(group_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;
    if exists == 0 {
        return Err(format!("Analysis source group {group_id} not found"));
    }

    let now = now_secs();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        UPDATE analysis_source_groups
        SET name = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&name)
    .bind(now)
    .bind(group_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM analysis_source_group_members WHERE group_id = ?")
        .bind(group_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for source_id in source_ids {
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(group_id)
        .bind(source_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    fetch_source_group(&pool, group_id)
        .await?
        .ok_or_else(|| format!("Analysis source group {group_id} not found after update"))
}

#[tauri::command]
pub async fn delete_analysis_source_group(handle: AppHandle, group_id: i64) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    let result = sqlx::query("DELETE FROM analysis_source_groups WHERE id = ?")
        .bind(group_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err(format!("Analysis source group {group_id} not found"));
    }

    Ok(())
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
    source_group_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<AnalysisRunSummary>, String> {
    let pool = get_pool(&handle).await?;
    let limit = limit.unwrap_or(20).clamp(1, 100);

    if source_id.is_some() && source_group_id.is_some() {
        return Err("Pass either source_id or source_group_id, not both".to_string());
    }

    let rows: Vec<AnalysisRunRow> = if let Some(source_id) = source_id {
        sqlx::query_as(
            r#"
            SELECT
                runs.id,
                runs.run_type,
                runs.scope_type,
                runs.source_id,
                sources.title AS source_title,
                runs.source_group_id,
                groups.name AS source_group_name,
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
            LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
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
    } else if let Some(source_group_id) = source_group_id {
        sqlx::query_as(
            r#"
            SELECT
                runs.id,
                runs.run_type,
                runs.scope_type,
                runs.source_id,
                sources.title AS source_title,
                runs.source_group_id,
                groups.name AS source_group_name,
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
            LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
            LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
            WHERE runs.source_group_id = ?
            ORDER BY runs.created_at DESC
            LIMIT ?
            "#,
        )
        .bind(source_group_id)
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
                runs.source_group_id,
                groups.name AS source_group_name,
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
            LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
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
pub async fn resolve_analysis_trace_refs(
    handle: AppHandle,
    run_id: i64,
    refs: Vec<String>,
) -> Result<Vec<AnalysisTraceRef>, String> {
    let mut normalized_refs = refs
        .into_iter()
        .filter_map(|reference| normalize_ref(&reference))
        .collect::<Vec<_>>();
    normalized_refs.sort();
    normalized_refs.dedup();

    if normalized_refs.is_empty() {
        return Ok(Vec::new());
    }

    let pool = get_pool(&handle).await?;
    let run = get_analysis_run(handle.clone(), run_id)
        .await?
        .ok_or_else(|| format!("Analysis run {run_id} not found"))?;

    let source_ids = resolve_run_source_ids(&pool, &run).await?;
    let corpus = load_corpus_messages(&pool, &source_ids, run.period_from, run.period_to).await?;
    Ok(build_trace_refs(&normalized_refs, &corpus))
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
    use super::{
        compress_trace_data, decode_trace_data, ensure_builtin_report_template,
        normalize_source_group_input, AnalysisTraceData, AnalysisTraceRef, TEMPLATE_KIND_REPORT,
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
                source_group_id INTEGER,
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

    #[test]
    fn source_group_input_is_trimmed_and_deduplicated() {
        let (name, source_ids) = normalize_source_group_input("  Core sources  ", vec![4, 2, 4, -1, 2])
            .expect("normalize source group");

        assert_eq!(name, "Core sources");
        assert_eq!(source_ids, vec![2, 4]);
    }
}
