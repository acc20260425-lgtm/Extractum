mod chat;
mod groups;
mod models;
mod report;
mod store;
mod templates;
mod trace;

use std::io::Cursor;

use tauri::{AppHandle, Emitter};

use crate::db::get_pool;
use self::models::{
    AnalysisChatEvent, AnalysisChatTurn, AnalysisRunDetail, AnalysisRunEvent, AnalysisRunRow,
    AnalysisRunSummary, AnalysisSourceOption, AnalysisTraceData, AnalysisTraceRef,
};
use self::store::{fetch_run_row, load_corpus_messages, map_run_detail, map_run_summary, resolve_run_source_ids};
use self::trace::{build_trace_refs, decode_trace_data, normalize_ref};

pub use self::chat::{
    ask_analysis_run_question, clear_analysis_chat_messages, list_analysis_chat_messages,
};
pub use self::groups::{
    create_analysis_source_group, delete_analysis_source_group, list_analysis_source_groups,
    update_analysis_source_group,
};
pub use self::report::start_analysis_report;
pub use self::templates::{
    create_analysis_prompt_template, delete_analysis_prompt_template,
    list_analysis_prompt_templates, update_analysis_prompt_template,
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
    Ok(fetch_run_row(&pool, run_id).await?.map(map_run_detail))
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

#[cfg(test)]
mod tests {
    use super::{
        decode_trace_data, AnalysisTraceData, AnalysisTraceRef, TEMPLATE_KIND_REPORT,
    };
    use super::groups::normalize_source_group_input;
    use super::store::ensure_builtin_report_template;
    use super::trace::compress_trace_data;

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
