use super::YoutubeSummaryRunExecutionOutcome;
#[cfg(test)]
use crate::compression::compress_text;
#[cfg(test)]
use crate::error::{AppError, AppResult};
#[cfg(test)]
use sqlx::SqlitePool;

pub(crate) fn terminal_message(status: &str) -> &'static str {
    match status {
        "complete" => "Completed",
        "partial" => "Completed with partial results",
        _ => "Failed",
    }
}

pub(crate) fn cancelled_outcome(
    run_id: i64,
    progress_current: i64,
    progress_total: i64,
) -> YoutubeSummaryRunExecutionOutcome {
    YoutubeSummaryRunExecutionOutcome {
        run_id,
        run_status: "cancelled".to_string(),
        progress_current,
        progress_total,
        message: "Cancelled".to_string(),
    }
}

#[cfg(test)]
pub(crate) async fn persist_minimal_execution_result(
    pool: &SqlitePool,
    run_id: i64,
    result_status: &str,
) -> AppResult<()> {
    let canonical = serde_json::json!({
        "schema_version": "1.0",
        "result_id": format!("result_{run_id}"),
        "run_id": run_id,
        "pack_id": "youtube_summary",
        "pack_version": "1.0.0",
        "stage": "youtube_summary/transcript_analysis",
        "created_at": now_string(),
        "output_language": "en",
        "metadata": {},
        "run_context": {},
        "outputs": { "pack_data": { "youtube_summary": { "videos": [] } } },
        "source_refs": [],
        "claims": [],
        "evidence": [],
        "warnings": [],
        "limitations": [],
        "quality_flags": [],
        "audit_refs": []
    });
    let canonical_json = canonical.to_string();
    let result_row_id: i64 = sqlx::query_scalar(
        "INSERT INTO prompt_pack_results (
            run_id, result_id, result_status, schema_version, canonical_hash,
            canonical_json_zstd, projection_updated_at, created_at, updated_at
         )
         VALUES (?, ?, ?, '1.0', ?, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(run_id)
    .bind(format!("result_{run_id}"))
    .bind(result_status)
    .bind(format!("sha384-{}", simple_hash(&canonical_json)))
    .bind(compress_text(&canonical_json).map_err(AppError::internal)?)
    .bind(now_string())
    .bind(now_string())
    .bind(now_string())
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if result_status == "partial" {
        sqlx::query(
            "INSERT INTO prompt_pack_result_warnings (
                result_row_id, run_id, warning_id, code, message
             )
             VALUES (?, ?, 'warning_1', 'partial_provider_failure', 'One or more videos failed')",
        )
        .bind(result_row_id)
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
        sqlx::query(
            "INSERT INTO prompt_pack_result_quality_flags (
                result_row_id, run_id, flag_id, severity, message
             )
             VALUES (?, ?, 'quality_flag_1', 'warning', 'Partial result')",
        )
        .bind(result_row_id)
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    }

    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = ?, result_status = ?, completed_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(result_status)
    .bind(result_status)
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) fn terminal_status_for_synthesis(
    successes: i64,
    failures: i64,
    transcript_total: i64,
    synthesis_status: &str,
) -> &'static str {
    if successes == 0 {
        return "failed";
    }
    if synthesis_status == "failed" {
        return "partial";
    }
    if failures > 0 || successes < transcript_total {
        return "partial";
    }
    if transcript_total > 1 && synthesis_status != "succeeded" {
        return "partial";
    }
    "complete"
}

#[cfg(test)]
fn now_string() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
fn simple_hash(value: &str) -> String {
    use sha2::{Digest, Sha384};
    Sha384::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
