use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::compression::{compress_text, decompress_text};
use crate::error::{AppError, AppResult};

pub(crate) const TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID: &str =
    "stage-io/youtube_summary_transcript_analysis_output";
pub(crate) const SYNTHESIS_OUTPUT_SCHEMA_ID: &str = "stage-io/youtube_summary_synthesis_output";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptAnalysisStageInput {
    pub stage_io_version: String,
    pub schema_version: String,
    pub stage: String,
    pub pack_id: String,
    pub pack_version: String,
    pub run_id: i64,
    pub source_ref_id: String,
    pub allowed_source_ref_ids: Vec<String>,
    pub allowed_material_refs: Vec<String>,
    pub transcript_segment_registry: Vec<TranscriptSegmentRegistryEntry>,
    pub comment_selection_policy: serde_json::Value,
    pub control_preset: String,
    pub evidence_mode: String,
    pub output_language: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegmentRegistryEntry {
    pub material_ref_id: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageRunForInput {
    pub id: i64,
    pub run_id: i64,
    pub source_snapshot_id: i64,
    pub source_ref_id: String,
}

pub(crate) async fn load_transcript_analysis_stage_for_source(
    pool: &SqlitePool,
    run_id: i64,
    source_ref_id: &str,
) -> AppResult<StageRunForInput> {
    sqlx::query_as::<_, (i64, i64, i64, String)>(
        "SELECT stages.id, stages.run_id, snapshots.id, snapshots.source_ref_id
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.run_id = ?
           AND stages.stage_name = 'youtube_summary/transcript_analysis'
           AND snapshots.source_ref_id = ?",
    )
    .bind(run_id)
    .bind(source_ref_id)
    .fetch_one(pool)
    .await
    .map(
        |(id, run_id, source_snapshot_id, source_ref_id)| StageRunForInput {
            id,
            run_id,
            source_snapshot_id,
            source_ref_id,
        },
    )
    .map_err(AppError::database)
}

pub(crate) async fn build_transcript_analysis_stage_input(
    pool: &SqlitePool,
    stage_run_id: i64,
) -> AppResult<TranscriptAnalysisStageInput> {
    let (
        run_id,
        pack_id,
        pack_version,
        output_language,
        control_preset,
        evidence_mode,
        source_snapshot_id,
        source_ref_id,
    ) = sqlx::query_as::<_, (i64, String, String, String, String, String, i64, String)>(
        "SELECT runs.id, runs.pack_id, runs.pack_version, runs.output_language,
                runs.control_preset, runs.evidence_mode,
                snapshots.id, snapshots.source_ref_id
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_runs runs ON runs.id = stages.run_id
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.id = ?",
    )
    .bind(stage_run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let allowed_source_ref_ids = sqlx::query_scalar::<_, String>(
        "SELECT source_ref_id FROM prompt_pack_run_source_snapshots
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let material_rows = sqlx::query_as::<_, (String, String, Vec<u8>)>(
        "SELECT material_ref_id, material_kind, text_zstd
         FROM prompt_pack_run_material_snapshots
         WHERE run_id = ? AND source_snapshot_id = ?
         ORDER BY sequence_index ASC, id ASC",
    )
    .bind(run_id)
    .bind(source_snapshot_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let allowed_material_refs = material_rows
        .iter()
        .map(|(material_ref_id, _, _)| material_ref_id.clone())
        .collect::<Vec<_>>();
    let transcript_segment_registry = material_rows
        .iter()
        .filter(|(_, material_kind, _)| material_kind == "transcript")
        .map(
            |(material_ref_id, _, text_zstd)| TranscriptSegmentRegistryEntry {
                material_ref_id: material_ref_id.clone(),
                text: decompress_text(text_zstd).unwrap_or_default(),
            },
        )
        .collect::<Vec<_>>();

    Ok(TranscriptAnalysisStageInput {
        stage_io_version: "1.0".to_string(),
        schema_version: "1.0".to_string(),
        stage: "youtube_summary/transcript_analysis".to_string(),
        pack_id,
        pack_version,
        run_id,
        source_ref_id,
        allowed_source_ref_ids,
        allowed_material_refs,
        transcript_segment_registry,
        comment_selection_policy: serde_json::json!({
            "comment_count_cap": 50,
            "comment_budget_ratio": 0.15,
            "comment_token_cap": 4000
        }),
        control_preset,
        evidence_mode,
        output_language,
    })
}

pub(crate) fn extract_json_payload(text: &str) -> AppResult<serde_json::Value> {
    let trimmed = text.trim();
    let candidate = if trimmed.starts_with("```json") && trimmed.ends_with("```") {
        trimmed
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else {
        trimmed
    };

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(candidate) {
        return Ok(value);
    }

    let mut ranges = Vec::new();
    let mut depth = 0_i64;
    let mut start = None;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in trimmed.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth < 0 {
                    return Err(AppError::validation("malformed JSON braces"));
                }
                if depth == 0 {
                    if let Some(start_index) = start.take() {
                        ranges.push((start_index, index + ch.len_utf8()));
                    }
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(AppError::validation("malformed JSON braces"));
    }
    if ranges.len() > 1 {
        return Err(AppError::validation(
            "multiple JSON objects in provider response",
        ));
    }
    let Some((start, end)) = ranges.first().copied() else {
        return Err(AppError::validation(
            "provider response did not contain a JSON object",
        ));
    };
    serde_json::from_str(&trimmed[start..end])
        .map_err(|error| AppError::validation(format!("malformed JSON payload: {error}")))
}

pub(crate) async fn insert_stage_artifact_in_pool(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    artifact_kind: &str,
    attempt_number: i64,
    artifact_index: i64,
    content: &str,
) -> AppResult<()> {
    insert_stage_artifact_with_executor(
        pool,
        run_id,
        stage_run_id,
        artifact_kind,
        attempt_number,
        artifact_index,
        content,
    )
    .await
}

pub(crate) async fn insert_stage_artifact_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: i64,
    stage_run_id: i64,
    artifact_kind: &str,
    attempt_number: i64,
    artifact_index: i64,
    content: &str,
) -> AppResult<()> {
    insert_stage_artifact_with_executor(
        &mut **tx,
        run_id,
        stage_run_id,
        artifact_kind,
        attempt_number,
        artifact_index,
        content,
    )
    .await
}

async fn insert_stage_artifact_with_executor<'e, E>(
    executor: E,
    run_id: i64,
    stage_run_id: i64,
    artifact_kind: &str,
    attempt_number: i64,
    artifact_index: i64,
    content: &str,
) -> AppResult<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let content_hash = format!("sha384-{}", sha384_hex(content.as_bytes()));
    let content_zstd = compress_text(content).map_err(AppError::internal)?;
    let created_at = crate::time::now_rfc3339_utc();
    sqlx::query(
        "INSERT INTO prompt_pack_stage_artifacts (
            run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
            content_type, content_hash, content_zstd, redaction_state, created_at
         )
         VALUES (?, ?, ?, ?, ?, 'application/json', ?, ?, 'none', ?)",
    )
    .bind(run_id)
    .bind(stage_run_id)
    .bind(artifact_kind)
    .bind(attempt_number)
    .bind(artifact_index)
    .bind(content_hash)
    .bind(content_zstd)
    .bind(created_at)
    .execute(executor)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn sha384_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha384};
    Sha384::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        build_transcript_analysis_stage_input, insert_stage_artifact_in_pool,
        load_transcript_analysis_stage_for_source,
    };
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::dto::StartYoutubeSummaryRunRequest;
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::youtube_summary::create_youtube_summary_run_skeleton_in_pool;

    #[tokio::test]
    async fn build_transcript_analysis_stage_input_uses_frozen_registries() {
        let (pool, run_id) = test_pool_with_frozen_youtube_summary_run().await;
        let stage = load_transcript_analysis_stage_for_source(&pool, run_id, "source_ref_1")
            .await
            .expect("stage");

        let input = build_transcript_analysis_stage_input(&pool, stage.id)
            .await
            .expect("input");

        assert_eq!(input.stage_io_version, "1.0");
        assert_eq!(input.stage, "youtube_summary/transcript_analysis");
        assert_eq!(input.pack_id, "youtube_summary");
        assert_eq!(input.source_ref_id, "source_ref_1");
        assert!(input
            .allowed_material_refs
            .iter()
            .all(|value| value.starts_with("m_")));
        assert!(!input.transcript_segment_registry.is_empty());
    }

    #[tokio::test]
    async fn insert_stage_artifact_uses_current_time() {
        use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

        let (pool, run_id) = test_pool_with_frozen_youtube_summary_run().await;
        let stage = load_transcript_analysis_stage_for_source(&pool, run_id, "source_ref_1")
            .await
            .expect("stage");
        let before = OffsetDateTime::now_utc() - Duration::seconds(5);

        insert_stage_artifact_in_pool(&pool, run_id, stage.id, "metrics", 1, 4, r#"{"ok":true}"#)
            .await
            .expect("insert artifact");

        let after = OffsetDateTime::now_utc() + Duration::seconds(5);
        let created_at: String = sqlx::query_scalar(
            "SELECT created_at FROM prompt_pack_stage_artifacts WHERE stage_run_id = ?",
        )
        .bind(stage.id)
        .fetch_one(&pool)
        .await
        .expect("artifact created_at");
        let parsed = OffsetDateTime::parse(&created_at, &Rfc3339).expect("parse created_at");

        assert_ne!(created_at, "2026-06-14T00:00:00Z");
        assert!(
            parsed >= before && parsed <= after,
            "expected {created_at} to be between {before} and {after}"
        );
    }

    async fn test_pool_with_frozen_youtube_summary_run() -> (sqlx::SqlitePool, i64) {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed");
        insert_youtube_video(&pool, 901, "v-ready").await;
        insert_transcript(&pool, 901, "Ready transcript").await;
        let request = StartYoutubeSummaryRunRequest {
            client_request_id: "req-stage-input".to_string(),
            project_id: None,
            source_ids: vec![901],
            profile_id: None,
            model_override: Some("test-model".to_string()),
            output_language: "en".to_string(),
            control_preset: "standard".to_string(),
            evidence_mode: "standard".to_string(),
            include_comments: false,
        };
        let run_id = create_youtube_summary_run_skeleton_in_pool(&pool, request, 1)
            .await
            .expect("run skeleton");
        (pool, run_id)
    }

    async fn insert_youtube_video(pool: &sqlx::SqlitePool, source_id: i64, video_id: &str) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             )
             VALUES (?, 'youtube', 'video', ?, ?, 1, 0, 1)",
        )
        .bind(source_id)
        .bind(video_id)
        .bind(format!("Video {video_id}"))
        .execute(pool)
        .await
        .expect("insert source");
        sqlx::query(
            "INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, description,
                video_form, availability_status
             )
             VALUES (?, ?, ?, ?, 'Description', 'regular', 'available')",
        )
        .bind(source_id)
        .bind(video_id)
        .bind(format!("https://www.youtube.com/watch?v={video_id}"))
        .bind(format!("Video {video_id}"))
        .execute(pool)
        .await
        .expect("insert video metadata");
    }

    async fn insert_transcript(pool: &sqlx::SqlitePool, source_id: i64, text: &str) {
        let item_id: i64 = sqlx::query_scalar(
            "INSERT INTO items (
                source_id, external_id, published_at, ingested_at, item_kind
             )
             VALUES (?, ?, 1, 1, 'youtube_transcript')
             RETURNING id",
        )
        .bind(source_id)
        .bind(format!("item-{source_id}"))
        .fetch_one(pool)
        .await
        .expect("insert transcript item");
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text
             )
             VALUES (?, ?, 0, 0, 1000, ?)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(text)
        .execute(pool)
        .await
        .expect("insert transcript segment");
    }
}
