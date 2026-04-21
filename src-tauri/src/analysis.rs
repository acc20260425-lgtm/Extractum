use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use std::io::Cursor;
use tauri::AppHandle;

use crate::db::get_pool;

const TEMPLATE_KIND_REPORT: &str = "report";
const TEMPLATE_KIND_CHAT: &str = "chat";
const DEFAULT_REPORT_TEMPLATE_NAME: &str = "Default report";

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
    pub prompt_template_version: i64,
    pub provider_profile: String,
    pub provider: String,
    pub model: String,
    pub status: String,
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

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
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
        prompt_template_version: row.prompt_template_version,
        provider_profile: row.provider_profile,
        provider: row.provider,
        model: row.model,
        status: row.status,
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
        WHERE runs.id = ?
        "#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
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
