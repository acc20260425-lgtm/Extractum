use std::collections::{HashMap, HashSet};

use sqlx::SqlitePool;

use crate::compression::decompress_text;
use crate::error::{AppError, AppResult};

pub(crate) async fn build_youtube_summary_canonical_result(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<serde_json::Value> {
    let (pack_id, pack_version, output_language): (String, String, String) = sqlx::query_as(
        "SELECT pack_id, pack_version, output_language FROM prompt_pack_runs WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let source_rows = sqlx::query_as::<_, (i64, String, String, Option<String>)>(
        "SELECT id, source_ref_id, video_id, title
         FROM prompt_pack_run_source_snapshots
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut limitations = build_base_limitations(pool, run_id).await?;
    let mut quality_flags = build_base_quality_flags(pool, run_id).await?;
    let graph_outcome = load_complete_intermediate_graph_for_result(pool, run_id).await?;
    let use_graph_entities = matches!(graph_outcome, GraphLoadOutcome::Complete(_));

    let mut source_refs = Vec::new();
    let mut videos = Vec::new();
    let mut claims = Vec::new();
    let mut evidence = Vec::new();
    for (source_index, (snapshot_id, source_ref_id, source_video_id, title)) in
        source_rows.iter().enumerate()
    {
        source_refs.push(serde_json::json!({
            "source_ref_id": source_ref_id,
            "source_snapshot_id": snapshot_id,
            "title": title
        }));
        let parsed = load_latest_parsed_output(pool, run_id, *snapshot_id).await?;
        videos.push(serde_json::json!({
            "video_id": format!("video_{}", source_index + 1),
            "source_ref_id": source_ref_id,
            "provider_video_id": source_video_id,
            "title": title,
            "summary_text": parsed
                .get("video_candidate")
                .and_then(|value| value.get("summary_text"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
        }));
        if !use_graph_entities {
            if let Some(candidate_claims) = parsed
                .get("claim_candidates")
                .and_then(|value| value.as_array())
            {
                for candidate in candidate_claims {
                    claims.push(serde_json::json!({
                        "claim_id": format!("claim_{}", claims.len() + 1),
                        "source_ref_id": source_ref_id,
                        "text": candidate.get("text").and_then(serde_json::Value::as_str).unwrap_or("")
                    }));
                }
            }
            if let Some(candidate_evidence) = parsed
                .get("evidence_fragment_candidates")
                .and_then(|value| value.as_array())
            {
                for candidate in candidate_evidence {
                    evidence.push(serde_json::json!({
                        "evidence_id": format!("evidence_{}", evidence.len() + 1),
                        "source_ref_id": source_ref_id,
                        "text": candidate.get("text").and_then(serde_json::Value::as_str).unwrap_or("")
                    }));
                }
            }
        }
    }

    match graph_outcome {
        GraphLoadOutcome::Complete(graph) => {
            claims = graph
                .get("claims")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            evidence = graph
                .get("evidence")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
        }
        GraphLoadOutcome::NoGraph => {}
    }

    if !use_graph_entities && evidence.is_empty() && !claims.is_empty() {
        evidence.push(serde_json::json!({
            "evidence_id": "evidence_1",
            "claim_id": "claim_1",
            "text": "Derived from transcript analysis"
        }));
    }

    let synthesis =
        load_latest_run_stage_parsed_output(pool, run_id, "youtube_summary/synthesis").await?;
    let synthesis_stage_status =
        load_run_stage_status(pool, run_id, "youtube_summary/synthesis").await?;
    let synthesis_candidate = synthesis
        .as_ref()
        .and_then(|value| value.get("synthesis_candidate"));
    let canonical_synthesis = if videos.len() > 1 {
        synthesis_candidate
            .map(|candidate| build_canonical_synthesis(candidate, &videos))
            .transpose()?
            .unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    match (
        videos.len(),
        canonical_synthesis.is_null(),
        synthesis_stage_status.as_deref(),
    ) {
        (1, true, _) => {
            limitations.push(
                "Synthesis is not applicable to a single-video YouTube Summary run.".to_string(),
            );
            push_quality_flag(
                &mut quality_flags,
                "synthesis_not_applicable_single_video",
                "info",
            );
        }
        (count, true, Some("failed")) if count > 1 => {
            limitations.push(
                "The synthesis stage failed, so the report only includes per-video analysis."
                    .to_string(),
            );
            push_quality_flag(&mut quality_flags, "synthesis_failed", "warning");
        }
        (count, true, Some("skipped")) if count > 1 => {
            limitations.push(
                "The synthesis stage was skipped because fewer than two videos produced usable transcript analysis."
                    .to_string(),
            );
            push_quality_flag(
                &mut quality_flags,
                "synthesis_skipped_insufficient_successes",
                "warning",
            );
        }
        _ => {}
    }
    let sections = synthesis_candidate
        .and_then(|candidate| candidate.get("summary_text"))
        .and_then(serde_json::Value::as_str)
        .map(|summary| {
            vec![serde_json::json!({
                "section_id": "section_summary",
                "title": "Summary",
                "body": summary
            })]
        })
        .unwrap_or_default();

    Ok(serde_json::json!({
        "schema_version": "1.0",
        "result_id": format!("result_{run_id}"),
        "run_id": run_id,
        "pack_id": pack_id,
        "pack_version": pack_version,
        "stage": "youtube_summary/transcript_analysis",
        "created_at": crate::time::now_rfc3339_utc(),
        "output_language": output_language,
        "metadata": {},
        "run_context": {},
        "outputs": {
            "sections": sections,
            "pack_data": {
                "youtube_summary": {
                    "videos": videos,
                    "synthesis": canonical_synthesis
                }
            }
        },
        "source_refs": source_refs,
        "claims": claims,
        "evidence": evidence,
        "warnings": [],
        "limitations": limitations,
        "quality_flags": quality_flags,
        "audit_refs": []
    }))
}

enum GraphLoadOutcome {
    Complete(serde_json::Value),
    NoGraph,
}

struct IntermediateGraphRow {
    source_snapshot_id: i64,
    source_ref_id: String,
    content_zstd: Vec<u8>,
}

async fn load_complete_intermediate_graph_for_result(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<GraphLoadOutcome> {
    let expected_sources = sqlx::query_as::<_, (i64, String)>(
        "SELECT snapshots.id, snapshots.source_ref_id
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.run_id = ?
           AND stages.stage_name = 'youtube_summary/transcript_analysis'
           AND stages.stage_status = 'succeeded'
         ORDER BY snapshots.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let graph_rows = load_intermediate_graph_rows(pool, run_id).await?;
    if graph_rows.is_empty() {
        return Ok(GraphLoadOutcome::NoGraph);
    }
    let graph_row_sources = graph_rows
        .iter()
        .map(|row| (row.source_snapshot_id, row.source_ref_id.clone()))
        .collect::<Vec<_>>();
    if graph_row_sources != expected_sources {
        return Err(AppError::internal(format!(
            "incomplete intermediate_entities graph for prompt pack run {run_id}"
        )));
    }
    let merged = merge_result_graph_rows(graph_rows)?;
    let merged_sources = graph_source_keys(&merged)?;
    if merged_sources != expected_sources {
        return Err(AppError::internal(format!(
            "intermediate_entities graph sources do not match successful transcript stages for prompt pack run {run_id}"
        )));
    }
    Ok(GraphLoadOutcome::Complete(merged))
}

async fn load_intermediate_graph_rows(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<IntermediateGraphRow>> {
    sqlx::query_as::<_, (i64, String, Vec<u8>)>(
        "SELECT snapshots.id, snapshots.source_ref_id, artifacts.content_zstd
         FROM prompt_pack_run_source_snapshots snapshots
         JOIN prompt_pack_stage_runs stages
           ON stages.run_id = snapshots.run_id
          AND stages.source_snapshot_id = snapshots.id
          AND stages.stage_name = 'youtube_summary/transcript_analysis'
          AND stages.stage_status = 'succeeded'
         JOIN prompt_pack_stage_artifacts artifacts
           ON artifacts.stage_run_id = stages.id
          AND artifacts.artifact_kind = 'intermediate_entities'
          AND artifacts.id = (
              SELECT latest.id
              FROM prompt_pack_stage_artifacts latest
              WHERE latest.stage_run_id = stages.id
                AND latest.artifact_kind = 'intermediate_entities'
              ORDER BY latest.attempt_number DESC, latest.artifact_index DESC, latest.id DESC
              LIMIT 1
          )
         WHERE snapshots.run_id = ?
         ORDER BY snapshots.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(source_snapshot_id, source_ref_id, content_zstd)| IntermediateGraphRow {
                    source_snapshot_id,
                    source_ref_id,
                    content_zstd,
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

fn merge_result_graph_rows(rows: Vec<IntermediateGraphRow>) -> AppResult<serde_json::Value> {
    let mut merged = serde_json::json!({
        "sources": [],
        "claims": [],
        "evidence": []
    });
    for row in rows {
        let text = decompress_text(&row.content_zstd).map_err(AppError::internal)?;
        let graph: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| AppError::internal(format!("parse intermediate entities: {error}")))?;
        append_graph_array(&mut merged, &graph, "sources")?;
        append_graph_array(&mut merged, &graph, "claims")?;
        append_graph_array(&mut merged, &graph, "evidence")?;
    }
    Ok(merged)
}

fn append_graph_array(
    merged: &mut serde_json::Value,
    graph: &serde_json::Value,
    key: &str,
) -> AppResult<()> {
    let graph_items = graph
        .get(key)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| AppError::internal(format!("intermediate graph {key} must be an array")))?;
    let merged_items = merged
        .get_mut(key)
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| AppError::internal(format!("merged graph {key} missing")))?;
    merged_items.extend(graph_items.iter().cloned());
    Ok(())
}

fn graph_source_keys(graph: &serde_json::Value) -> AppResult<Vec<(i64, String)>> {
    let sources = graph
        .get("sources")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| AppError::internal("intermediate graph sources must be an array"))?;
    let mut seen = HashSet::new();
    let mut keys = Vec::new();
    for source in sources {
        let source_snapshot_id = source
            .get("source_snapshot_id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                AppError::internal("intermediate graph source missing source_snapshot_id")
            })?;
        let source_ref_id = source
            .get("source_ref_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| AppError::internal("intermediate graph source missing source_ref_id"))?
            .to_string();
        if !seen.insert((source_snapshot_id, source_ref_id.clone())) {
            return Err(AppError::internal(format!(
                "duplicate intermediate graph source ({source_snapshot_id}, {source_ref_id})"
            )));
        }
        keys.push((source_snapshot_id, source_ref_id));
    }
    Ok(keys)
}

async fn load_latest_parsed_output(
    pool: &SqlitePool,
    run_id: i64,
    source_snapshot_id: i64,
) -> AppResult<serde_json::Value> {
    let bytes = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT artifacts.content_zstd
         FROM prompt_pack_stage_artifacts artifacts
         JOIN prompt_pack_stage_runs stages ON stages.id = artifacts.stage_run_id
         WHERE artifacts.run_id = ?
           AND stages.source_snapshot_id = ?
           AND artifacts.artifact_kind = 'parsed_output'
         ORDER BY artifacts.attempt_number DESC, artifacts.artifact_index DESC
         LIMIT 1",
    )
    .bind(run_id)
    .bind(source_snapshot_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    let Some(bytes) = bytes else {
        return Ok(serde_json::json!({}));
    };
    let text = decompress_text(&bytes).map_err(AppError::internal)?;
    serde_json::from_str(&text)
        .map_err(|error| AppError::internal(format!("parse parsed_output artifact: {error}")))
}

async fn load_latest_run_stage_parsed_output(
    pool: &SqlitePool,
    run_id: i64,
    stage_name: &str,
) -> AppResult<Option<serde_json::Value>> {
    let bytes = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT artifacts.content_zstd
         FROM prompt_pack_stage_artifacts artifacts
         JOIN prompt_pack_stage_runs stages ON stages.id = artifacts.stage_run_id
         WHERE artifacts.run_id = ?
           AND stages.stage_name = ?
           AND artifacts.artifact_kind = 'parsed_output'
         ORDER BY artifacts.attempt_number DESC, artifacts.artifact_index DESC
         LIMIT 1",
    )
    .bind(run_id)
    .bind(stage_name)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    let Some(bytes) = bytes else {
        return Ok(None);
    };
    let text = decompress_text(&bytes).map_err(AppError::internal)?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|error| AppError::internal(format!("parse parsed_output artifact: {error}")))
}

async fn load_run_stage_status(
    pool: &SqlitePool,
    run_id: i64,
    stage_name: &str,
) -> AppResult<Option<String>> {
    sqlx::query_scalar(
        "SELECT stage_status FROM prompt_pack_stage_runs
         WHERE run_id = ? AND stage_name = ?
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(run_id)
    .bind(stage_name)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

fn build_canonical_synthesis(
    candidate: &serde_json::Value,
    videos: &[serde_json::Value],
) -> AppResult<serde_json::Value> {
    let source_to_video = videos
        .iter()
        .filter_map(|video| {
            Some((
                video.get("source_ref_id")?.as_str()?.to_string(),
                video.get("video_id")?.as_str()?.to_string(),
            ))
        })
        .collect::<HashMap<_, _>>();
    let video_to_source = videos
        .iter()
        .filter_map(|video| {
            Some((
                video.get("video_id")?.as_str()?.to_string(),
                video.get("source_ref_id")?.as_str()?.to_string(),
            ))
        })
        .collect::<HashMap<_, _>>();

    let cross_video_themes = candidate
        .get("cross_video_themes")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, theme)| {
            let source_refs = ref_strings(theme.get("source_refs"));
            let video_refs = source_refs
                .iter()
                .filter_map(|source_ref| source_to_video.get(source_ref).cloned())
                .collect::<Vec<_>>();
            serde_json::json!({
                "theme_id": format!("theme_{}", index + 1),
                "theme_text": theme.get("theme_text").and_then(serde_json::Value::as_str).unwrap_or(""),
                "video_refs": video_refs,
                "claim_refs": ref_strings(theme.get("claim_refs")),
                "evidence_refs": ref_strings(theme.get("evidence_refs"))
            })
        })
        .collect::<Vec<_>>();

    let common_claims = candidate
        .get("common_claims")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, claim)| {
            let source_refs = ref_strings(claim.get("source_refs"));
            let video_refs = source_refs
                .iter()
                .filter_map(|source_ref| source_to_video.get(source_ref).cloned())
                .collect::<Vec<_>>();
            serde_json::json!({
                "common_claim_id": format!("common_claim_{}", index + 1),
                "summary_text": claim
                    .get("summary_text")
                    .or_else(|| claim.get("text"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                "video_refs": video_refs,
                "claim_refs": ref_strings(claim.get("claim_refs")),
                "evidence_refs": ref_strings(claim.get("evidence_refs"))
            })
        })
        .collect::<Vec<_>>();

    let contradictions = candidate
        .get("contradictions_across_videos")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, contradiction)| {
            let source_refs = ref_strings(contradiction.get("source_refs"));
            let video_refs = source_refs
                .iter()
                .filter_map(|source_ref| source_to_video.get(source_ref).cloned())
                .collect::<Vec<_>>();
            serde_json::json!({
                "contradiction_id": format!("contradiction_{}", index + 1),
                "description": contradiction.get("description").and_then(serde_json::Value::as_str).unwrap_or(""),
                "video_refs": video_refs,
                "relation_refs": ref_strings(contradiction.get("relation_refs")),
                "claim_refs": ref_strings(contradiction.get("claim_refs")),
                "evidence_refs": ref_strings(contradiction.get("evidence_refs"))
            })
        })
        .collect::<Vec<_>>();

    let mut claim_refs = Vec::new();
    let mut relation_refs = Vec::new();
    let mut evidence_refs = Vec::new();
    let mut source_refs = Vec::new();
    extend_unique_refs_from_items(&mut claim_refs, &cross_video_themes, "claim_refs");
    extend_unique_refs_from_items(&mut claim_refs, &common_claims, "claim_refs");
    extend_unique_refs_from_items(&mut claim_refs, &contradictions, "claim_refs");
    extend_unique_refs_from_items(&mut relation_refs, &contradictions, "relation_refs");
    extend_unique_refs_from_items(&mut evidence_refs, &cross_video_themes, "evidence_refs");
    extend_unique_refs_from_items(&mut evidence_refs, &common_claims, "evidence_refs");
    extend_unique_refs_from_items(&mut evidence_refs, &contradictions, "evidence_refs");
    extend_unique_source_refs_from_video_refs(
        &mut source_refs,
        &cross_video_themes,
        &video_to_source,
    );
    extend_unique_source_refs_from_video_refs(&mut source_refs, &common_claims, &video_to_source);
    extend_unique_source_refs_from_video_refs(&mut source_refs, &contradictions, &video_to_source);

    Ok(serde_json::json!({
        "cross_video_themes": cross_video_themes,
        "common_claims": common_claims,
        "contradictions_across_videos": contradictions,
        "claim_refs": claim_refs,
        "relation_refs": relation_refs,
        "evidence_refs": evidence_refs,
        "source_refs": source_refs
    }))
}

async fn build_base_quality_flags(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<serde_json::Value>> {
    let total_sources: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_run_source_snapshots WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    let successful_transcripts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_stage_runs
         WHERE run_id = ?
           AND stage_name = 'youtube_summary/transcript_analysis'
           AND stage_status = 'succeeded'",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    let mut flags = Vec::new();
    if successful_transcripts < total_sources {
        push_quality_flag(&mut flags, "partial_result", "warning");
    }
    Ok(flags)
}

async fn build_base_limitations(_pool: &SqlitePool, _run_id: i64) -> AppResult<Vec<String>> {
    Ok(Vec::new())
}

fn push_quality_flag(flags: &mut Vec<serde_json::Value>, flag: &str, severity: &str) {
    if !flags
        .iter()
        .any(|value| value["flag"].as_str() == Some(flag))
    {
        flags.push(serde_json::json!({
            "flag": flag,
            "severity": severity
        }));
    }
}

fn ref_strings(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn push_unique_ref(target: &mut Vec<String>, value: String) {
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}

fn extend_unique_refs_from_items(
    target: &mut Vec<String>,
    items: &[serde_json::Value],
    field: &str,
) {
    for item in items {
        for value in ref_strings(item.get(field)) {
            push_unique_ref(target, value);
        }
    }
}

fn extend_unique_source_refs_from_video_refs(
    target: &mut Vec<String>,
    items: &[serde_json::Value],
    video_to_source: &HashMap<String, String>,
) {
    for item in items {
        for video_ref in ref_strings(item.get("video_refs")) {
            if let Some(source_ref) = video_to_source.get(&video_ref) {
                push_unique_ref(target, source_ref.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::build_youtube_summary_canonical_result;
    use crate::compression::compress_text;
    use crate::error::AppResult;
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::test_schema::prompt_pack_test_pool;
    use crate::prompt_packs::youtube_summary::gem_analysis::assemble_gem_analysis_transcript_output;
    use crate::prompt_packs::youtube_summary::outputs::execute_transcript_analysis_stage_with_completion;
    use crate::prompt_packs::youtube_summary::test_support::{
        test_pool_with_frozen_youtube_summary_run, transcript_analysis_stage_id,
    };
    use crate::prompt_packs::youtube_summary::LlmCompletion;

    #[tokio::test]
    async fn build_canonical_result_assigns_backend_owned_ids() {
        let pool = test_pool_with_successful_stage_artifacts().await;

        let result = build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert_eq!(result["pack_id"], "youtube_summary");
        assert_eq!(result["run_id"], 42);
        assert_eq!(result["source_refs"][0]["source_ref_id"], "source_ref_1");
        assert_eq!(result["claims"][0]["claim_id"], "claim_1");
        assert_eq!(result["evidence"][0]["evidence_id"], "evidence_1");
        assert_eq!(
            result["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["video_id"],
            "video_1",
        );
        assert_eq!(
            result["outputs"]["pack_data"]["youtube_summary"]["synthesis"],
            serde_json::Value::Null,
        );
        assert!(result.get("sources").is_none());
        assert!(result.get("pack_data").is_none());
    }

    #[tokio::test]
    async fn gem_analysis_final_output_builds_canonical_single_video_result() {
        let pool = test_pool_with_frozen_youtube_summary_run().await;
        let stage_id = transcript_analysis_stage_id(&pool, 1).await;
        let output =
            assemble_gem_analysis_transcript_output("# Gem-анализ\n\n### Section\nContent")
                .expect("assembled output");
        execute_transcript_analysis_stage_with_completion(
            &pool,
            stage_id,
            LlmCompletion {
                text: output,
                input_tokens: Some(10),
                output_tokens: Some(20),
                latency_ms: 30,
            },
        )
        .await
        .expect("persist transcript stage");

        let canonical = build_youtube_summary_canonical_result(&pool, 1)
            .await
            .expect("canonical");

        assert!(
            canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["summary_text"]
                .as_str()
                .unwrap()
                .starts_with("# Gem-анализ")
        );
    }

    #[tokio::test]
    async fn build_canonical_result_uses_current_created_at() {
        use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

        let pool = test_pool_with_successful_stage_artifacts().await;
        let before = OffsetDateTime::now_utc() - Duration::seconds(5);

        let result = build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        let after = OffsetDateTime::now_utc() + Duration::seconds(5);
        let created_at = result["created_at"].as_str().expect("created_at");
        let parsed = OffsetDateTime::parse(created_at, &Rfc3339).expect("parse created_at");

        assert_ne!(created_at, "2026-06-14T00:00:00Z");
        assert!(
            parsed >= before && parsed <= after,
            "expected {created_at} to be between {before} and {after}"
        );
    }

    #[tokio::test]
    async fn build_canonical_result_includes_synthesis_output() {
        let pool = test_pool_with_two_successful_stage_artifacts().await;
        insert_synthesis_parsed_output(&pool, 42, "Combined summary")
            .await
            .expect("insert synthesis output");

        let result = super::build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert_eq!(
            result["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"][0]
                ["theme_text"],
            "Shared theme",
        );
        assert_eq!(
            result["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["source_refs"],
            serde_json::json!(["source_ref_1", "source_ref_2"]),
        );
        assert_eq!(result["outputs"]["sections"][0]["title"], "Summary");
    }

    #[tokio::test]
    async fn build_canonical_result_preserves_synthesis_common_claim_text() {
        let pool = test_pool_with_two_successful_stage_artifacts().await;
        insert_synthesis_parsed_output_with_common_claim_text(
            &pool,
            42,
            "Both videos show NotebookLM automating AI workflows.",
        )
        .await
        .expect("insert synthesis output");

        let result = super::build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert_eq!(
            result["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["common_claims"][0]
                ["summary_text"],
            "Both videos show NotebookLM automating AI workflows.",
        );
    }

    #[tokio::test]
    async fn build_canonical_result_marks_single_video_synthesis_not_applicable() {
        let pool = test_pool_with_successful_stage_artifacts().await;
        insert_isolated_result_builder_synthesis_stage_status(&pool, 42, "skipped")
            .await
            .expect("insert synthesis status");

        let result = super::build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert!(result["outputs"]["pack_data"]["youtube_summary"]["synthesis"].is_null());
        assert!(has_quality_flag(
            &result,
            "synthesis_not_applicable_single_video"
        ));
    }

    #[tokio::test]
    async fn build_canonical_result_marks_multi_video_synthesis_failed() {
        let pool = test_pool_with_two_successful_stage_artifacts().await;
        insert_isolated_result_builder_synthesis_stage_status(&pool, 42, "failed")
            .await
            .expect("insert synthesis status");

        let result = super::build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert!(result["outputs"]["pack_data"]["youtube_summary"]["synthesis"].is_null());
        assert!(has_quality_flag(&result, "synthesis_failed"));
        assert!(result["limitations"]
            .as_array()
            .expect("limitations")
            .iter()
            .any(|value| value
                .as_str()
                .unwrap_or("")
                .contains("synthesis stage failed")));
    }

    #[tokio::test]
    async fn build_canonical_result_marks_multi_video_synthesis_skipped_insufficient_successes() {
        let pool = test_pool_with_two_sources_one_successful_stage_artifact().await;
        insert_isolated_result_builder_synthesis_stage_status(&pool, 42, "skipped")
            .await
            .expect("insert synthesis status");

        let result = super::build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert!(result["outputs"]["pack_data"]["youtube_summary"]["synthesis"].is_null());
        assert!(has_quality_flag(
            &result,
            "synthesis_skipped_insufficient_successes"
        ));
    }

    #[tokio::test]
    async fn build_canonical_result_keeps_partial_result_flag_when_synthesis_is_skipped() {
        let pool = test_pool_with_two_sources_one_successful_stage_artifact().await;
        insert_isolated_result_builder_synthesis_stage_status(&pool, 42, "skipped")
            .await
            .expect("insert synthesis status");

        let result = super::build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert!(has_quality_flag(&result, "partial_result"));
        assert!(has_quality_flag(
            &result,
            "synthesis_skipped_insufficient_successes"
        ));
    }

    #[tokio::test]
    async fn build_canonical_result_uses_intermediate_graph_claims_and_evidence() {
        let pool = test_pool_with_successful_stage_artifacts().await;
        insert_intermediate_entities_artifact(
            &pool,
            42,
            1001,
            501,
            "source_ref_1",
            "Graph claim",
            "Graph evidence",
            1,
        )
        .await
        .expect("graph artifact");

        let result = build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert_eq!(result["claims"][0]["text"], "Graph claim");
        assert_eq!(result["evidence"][0]["text"], "Graph evidence");
    }

    #[tokio::test]
    async fn build_canonical_result_rejects_incomplete_intermediate_graph() {
        let pool = test_pool_with_two_successful_stage_artifacts().await;
        insert_intermediate_entities_artifact(
            &pool,
            42,
            1001,
            501,
            "source_ref_1",
            "Graph first",
            "Graph evidence first",
            1,
        )
        .await
        .expect("graph artifact");

        let error = build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect_err("incomplete graph rejected");

        assert!(error
            .message
            .contains("incomplete intermediate_entities graph"));
    }

    fn has_quality_flag(result: &serde_json::Value, flag: &str) -> bool {
        result["quality_flags"]
            .as_array()
            .expect("quality flags")
            .iter()
            .any(|value| value["flag"].as_str() == Some(flag))
    }

    async fn test_pool_with_successful_stage_artifacts() -> sqlx::SqlitePool {
        let pool = prompt_pack_test_pool().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed");
        sqlx::query(
            "INSERT INTO prompt_pack_runs (
                id, pack_version_id, pack_id, pack_version, schema_version,
                run_status, result_status, output_language, control_preset,
                evidence_mode, include_comments, created_at, updated_at
             )
             VALUES (42, 1, 'youtube_summary', '1.0.0', '1.0',
                'running', 'none', 'en', 'standard', 'standard', 0,
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert run");
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             )
             VALUES (901, 'youtube', 'video', 'provider-video-1', 'Video', 1, 0, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            "INSERT INTO prompt_pack_run_source_snapshots (
                id, run_id, source_id, source_ref_id, video_id, title, created_at
             )
             VALUES (501, 42, 901, 'source_ref_1', 'provider-video-1', 'Video', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert snapshot");
        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
                created_at, updated_at
             )
             VALUES (1001, 42, 501, 'youtube_summary/transcript_analysis', 20, 'succeeded',
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert stage");
        let parsed = serde_json::json!({
            "video_candidate": { "summary_text": "Summary" },
            "claim_candidates": [{ "text": "Claim" }],
            "evidence_fragment_candidates": [{ "text": "Evidence" }],
            "warning_candidates": []
        });
        sqlx::query(
            "INSERT INTO prompt_pack_stage_artifacts (
                run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
                content_type, content_hash, content_zstd, redaction_state, created_at
             )
             VALUES (42, 1001, 'parsed_output', 1, 3, 'application/json', 'sha384-test', ?, 'none',
                '2026-06-14T00:00:00Z')",
        )
        .bind(compress_text(&parsed.to_string()).expect("compress"))
        .execute(&pool)
        .await
        .expect("insert parsed artifact");
        pool
    }

    async fn insert_isolated_result_builder_synthesis_stage_status(
        pool: &sqlx::SqlitePool,
        run_id: i64,
        status: &str,
    ) -> sqlx::Result<()> {
        let run_exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_runs WHERE id = ?")
                .bind(run_id)
                .fetch_one(pool)
                .await?;
        assert_eq!(run_exists, 1, "result-builder fixture must own the run");

        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
                created_at, updated_at
             )
             VALUES (2001, ?, NULL, 'youtube_summary/synthesis', 103, ?,
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .bind(run_id)
        .bind(status)
        .execute(pool)
        .await?;
        let owned_stage_exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM prompt_pack_stage_runs stages
             JOIN prompt_pack_runs runs ON runs.id = stages.run_id
             WHERE stages.id = 2001 AND runs.id = ?",
        )
        .bind(run_id)
        .fetch_one(pool)
        .await?;
        assert_eq!(
            owned_stage_exists, 1,
            "synthesis stage must belong to fixture run"
        );
        Ok(())
    }

    async fn insert_synthesis_parsed_output(
        pool: &sqlx::SqlitePool,
        run_id: i64,
        summary: &str,
    ) -> sqlx::Result<()> {
        insert_isolated_result_builder_synthesis_stage_status(pool, run_id, "succeeded").await?;
        let parsed = serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/synthesis",
            "synthesis_candidate": {
                "summary_text": summary,
                "cross_video_themes": [
                    {
                        "theme_text": "Shared theme",
                        "source_refs": ["source_ref_1", "source_ref_2"],
                        "claim_refs": [],
                        "evidence_refs": []
                    }
                ],
                "common_claims": [],
                "contradictions_across_videos": []
            },
            "limitations": [],
            "warning_candidates": []
        });
        sqlx::query(
            "INSERT INTO prompt_pack_stage_artifacts (
                run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
                content_type, content_hash, content_zstd, redaction_state, created_at
             )
             VALUES (?, 2001, 'parsed_output', 1, 3, 'application/json', 'sha384-synthesis', ?, 'none',
                '2026-06-14T00:00:00Z')",
        )
        .bind(run_id)
        .bind(compress_text(&parsed.to_string()).expect("compress synthesis"))
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn insert_synthesis_parsed_output_with_common_claim_text(
        pool: &sqlx::SqlitePool,
        run_id: i64,
        claim_text: &str,
    ) -> sqlx::Result<()> {
        insert_isolated_result_builder_synthesis_stage_status(pool, run_id, "succeeded").await?;
        let parsed = serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/synthesis",
            "synthesis_candidate": {
                "summary_text": "Combined summary",
                "cross_video_themes": [],
                "common_claims": [
                    {
                        "text": claim_text,
                        "source_refs": ["source_ref_1", "source_ref_2"],
                        "claim_refs": [],
                        "evidence_refs": []
                    }
                ],
                "contradictions_across_videos": []
            },
            "limitations": [],
            "warning_candidates": []
        });
        sqlx::query(
            "INSERT INTO prompt_pack_stage_artifacts (
                run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
                content_type, content_hash, content_zstd, redaction_state, created_at
             )
             VALUES (?, 2001, 'parsed_output', 1, 3, 'application/json', 'sha384-synthesis-common-claim', ?, 'none',
                '2026-06-14T00:00:00Z')",
        )
        .bind(run_id)
        .bind(compress_text(&parsed.to_string()).expect("compress synthesis"))
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn insert_intermediate_entities_artifact(
        pool: &sqlx::SqlitePool,
        run_id: i64,
        stage_run_id: i64,
        source_snapshot_id: i64,
        source_ref_id: &str,
        claim: &str,
        evidence: &str,
        attempt_number: i64,
    ) -> AppResult<()> {
        let claim_ref = format!("{source_ref_id}_claim_1");
        let evidence_ref = format!("{source_ref_id}_evidence_1");
        let graph = serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "graph_kind": "youtube_summary_intermediate_entities",
            "run_id": run_id,
            "attempt_number": attempt_number,
            "sources": [{
                "source_ref_id": source_ref_id,
                "source_snapshot_id": source_snapshot_id,
                "title": null
            }],
            "segments": [],
            "key_points": [],
            "quotes": [],
            "claims": [{
                "claim_id": claim_ref.clone(),
                "source_ref_id": source_ref_id,
                "text": claim,
                "material_refs": []
            }],
            "evidence": [{
                "evidence_id": evidence_ref.clone(),
                "source_ref_id": source_ref_id,
                "text": evidence,
                "material_refs": [],
                "quote_ref": null
            }],
            "warnings": [],
            "allowed_refs": {
                "source_refs": [source_ref_id],
                "segment_refs": [],
                "key_point_refs": [],
                "quote_refs": [],
                "claim_refs": [claim_ref],
                "evidence_refs": [evidence_ref]
            }
        });
        crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
            pool,
            run_id,
            stage_run_id,
            "intermediate_entities",
            attempt_number,
            5,
            &graph.to_string(),
        )
        .await
    }

    async fn test_pool_with_two_successful_stage_artifacts() -> sqlx::SqlitePool {
        let pool = test_pool_with_successful_stage_artifacts().await;
        insert_second_source_snapshot_and_optional_parsed_output(&pool, true).await;
        pool
    }

    async fn test_pool_with_two_sources_one_successful_stage_artifact() -> sqlx::SqlitePool {
        let pool = test_pool_with_successful_stage_artifacts().await;
        insert_second_source_snapshot_and_optional_parsed_output(&pool, false).await;
        pool
    }

    async fn insert_second_source_snapshot_and_optional_parsed_output(
        pool: &sqlx::SqlitePool,
        include_parsed_output: bool,
    ) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             )
             VALUES (902, 'youtube', 'video', 'provider-video-2', 'Video 2', 1, 0, 1)",
        )
        .execute(pool)
        .await
        .expect("insert second source");
        sqlx::query(
            "INSERT INTO prompt_pack_run_source_snapshots (
                id, run_id, source_id, source_ref_id, video_id, title, created_at
             )
             VALUES (502, 42, 902, 'source_ref_2', 'provider-video-2', 'Video 2', '2026-06-14T00:00:00Z')",
        )
        .execute(pool)
        .await
        .expect("insert second snapshot");

        if include_parsed_output {
            sqlx::query(
                "INSERT INTO prompt_pack_stage_runs (
                    id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
                    created_at, updated_at
                 )
                 VALUES (1002, 42, 502, 'youtube_summary/transcript_analysis', 20, 'succeeded',
                    '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
            )
            .execute(pool)
            .await
            .expect("insert second stage");
            let parsed = serde_json::json!({
                "video_candidate": { "summary_text": "Second summary" },
                "claim_candidates": [{ "text": "Second claim" }],
                "evidence_fragment_candidates": [{ "text": "Second evidence" }],
                "warning_candidates": []
            });
            sqlx::query(
                "INSERT INTO prompt_pack_stage_artifacts (
                    run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
                    content_type, content_hash, content_zstd, redaction_state, created_at
                 )
                 VALUES (42, 1002, 'parsed_output', 1, 3, 'application/json', 'sha384-test-2', ?, 'none',
                    '2026-06-14T00:00:00Z')",
            )
            .bind(compress_text(&parsed.to_string()).expect("compress second parsed"))
            .execute(pool)
            .await
            .expect("insert second parsed artifact");
        }
    }
}
