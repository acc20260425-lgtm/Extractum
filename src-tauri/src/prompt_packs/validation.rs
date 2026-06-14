use std::collections::HashSet;

use sqlx::SqlitePool;

use super::stage_io::TranscriptAnalysisStageInput;
use crate::compression::compress_text;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptPackValidationError {
    pub message: String,
    pub object_path: Option<String>,
}

impl std::fmt::Display for PromptPackValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PromptPackValidationError {}

pub(crate) fn validate_transcript_analysis_output(
    input: &TranscriptAnalysisStageInput,
    output: &serde_json::Value,
) -> Result<(), PromptPackValidationError> {
    expect_string(output, "stage_io_version", "1.0")?;
    expect_string(output, "schema_version", "1.0")?;
    expect_string(output, "stage", "youtube_summary/transcript_analysis")?;
    for key in [
        "video_candidate",
        "claim_candidates",
        "evidence_fragment_candidates",
        "warning_candidates",
    ] {
        if output.get(key).is_none() {
            return Err(PromptPackValidationError {
                message: format!("missing required key {key}"),
                object_path: Some(format!("$.{key}")),
            });
        }
    }

    reject_backend_owned_ids(output, "$")?;
    let allowed_material_refs = input
        .allowed_material_refs
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    reject_unknown_material_refs(output, "$", &allowed_material_refs)?;
    Ok(())
}

pub(crate) async fn validate_and_quarantine_transcript_analysis_output(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    input: &TranscriptAnalysisStageInput,
    output: &serde_json::Value,
) -> Result<(), PromptPackValidationError> {
    match validate_transcript_analysis_output(input, output) {
        Ok(()) => Ok(()),
        Err(error) => {
            let object_path = error
                .object_path
                .clone()
                .unwrap_or_else(|| "$".to_string());
            let candidate = value_at_path(output, &object_path).unwrap_or(output);
            let content = serde_json::to_string(candidate).unwrap_or_else(|_| "{}".to_string());
            let _ = sqlx::query(
                "INSERT INTO prompt_pack_result_quarantine_artifacts (
                    run_id, stage_run_id, object_path, reason, content_json_zstd, created_at
                 )
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(run_id)
            .bind(stage_run_id)
            .bind(&object_path)
            .bind(&error.message)
            .bind(compress_text(&content).unwrap_or_default())
            .bind("2026-06-14T00:00:00Z")
            .execute(pool)
            .await;
            Err(error)
        }
    }
}

fn expect_string(
    output: &serde_json::Value,
    key: &str,
    expected: &str,
) -> Result<(), PromptPackValidationError> {
    if output.get(key).and_then(serde_json::Value::as_str) == Some(expected) {
        Ok(())
    } else {
        Err(PromptPackValidationError {
            message: format!("{key} must be {expected}"),
            object_path: Some(format!("$.{key}")),
        })
    }
}

fn reject_backend_owned_ids(
    value: &serde_json::Value,
    path: &str,
) -> Result<(), PromptPackValidationError> {
    const FORBIDDEN: &[&str] = &[
        "claim_id",
        "evidence_id",
        "source_ref_id",
        "segment_id",
        "key_point_id",
        "quote_id",
        "action_item_id",
        "open_question_id",
    ];
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}.{key}");
                if FORBIDDEN.contains(&key.as_str()) {
                    return Err(PromptPackValidationError {
                        message: format!("LLM output must not assign backend-owned id {key}"),
                        object_path: Some(child_path),
                    });
                }
                reject_backend_owned_ids(child, &child_path)?;
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                reject_backend_owned_ids(child, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn reject_unknown_material_refs(
    value: &serde_json::Value,
    path: &str,
    allowed_material_refs: &HashSet<String>,
) -> Result<(), PromptPackValidationError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                if key == "material_refs" {
                    if let Some(refs) = child.as_array() {
                        for material_ref in refs.iter().filter_map(serde_json::Value::as_str) {
                            if !allowed_material_refs.contains(material_ref) {
                                return Err(PromptPackValidationError {
                                    message: format!("unknown material ref {material_ref}"),
                                    object_path: Some(path.to_string()),
                                });
                            }
                        }
                    }
                }
                reject_unknown_material_refs(child, &format!("{path}.{key}"), allowed_material_refs)?;
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                reject_unknown_material_refs(
                    child,
                    &format!("{path}[{index}]"),
                    allowed_material_refs,
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn value_at_path<'a>(
    value: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
    if path == "$" {
        return Some(value);
    }
    if path == "$.claim_candidates[0]" {
        return value.get("claim_candidates")?.get(0);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        validate_and_quarantine_transcript_analysis_output,
        validate_transcript_analysis_output,
    };
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::stage_io::{
        extract_json_payload, TranscriptAnalysisStageInput,
    };

    #[test]
    fn transcript_analysis_output_rejects_unknown_material_ref() {
        let input = test_stage_input_with_material_refs(["m_transcript_1"]);
        let output = serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/transcript_analysis",
            "video_candidate": {
                "summary_text": "Summary",
                "segment_candidates": [],
                "key_point_candidates": [],
                "quote_candidates": [],
                "action_item_candidates": [],
                "open_question_candidates": []
            },
            "claim_candidates": [
                {
                    "text": "Claim",
                    "material_refs": ["m_missing"]
                }
            ],
            "evidence_fragment_candidates": [],
            "warning_candidates": []
        });

        let error = validate_transcript_analysis_output(&input, &output)
            .expect_err("unknown material ref rejected");

        assert!(error.message.contains("m_missing"));
    }

    #[test]
    fn transcript_analysis_output_rejects_llm_assigned_final_ids() {
        let input = test_stage_input_with_material_refs(["m_transcript_1"]);
        let output = test_output_with_claim_id("claim_1");

        let error = validate_transcript_analysis_output(&input, &output)
            .expect_err("final ids rejected");

        assert!(error.message.contains("claim_id"));
    }

    #[test]
    fn extract_json_payload_accepts_fenced_json_object() {
        let text = "```json\n{\"stage_io_version\":\"1.0\",\"value\":1}\n```";

        let value = extract_json_payload(text).expect("json payload");

        assert_eq!(value["stage_io_version"], "1.0");
        assert_eq!(value["value"], 1);
    }

    #[test]
    fn extract_json_payload_accepts_leading_and_trailing_prose() {
        let text = "Here is the result:\n{\"stage_io_version\":\"1.0\",\"value\":1}\nDone.";

        let value = extract_json_payload(text).expect("json payload");

        assert_eq!(value["stage_io_version"], "1.0");
        assert_eq!(value["value"], 1);
    }

    #[test]
    fn extract_json_payload_rejects_malformed_braces() {
        let error = extract_json_payload("{\"stage_io_version\":\"1.0\"")
            .expect_err("malformed JSON rejected");

        assert!(error.message.contains("malformed"));
    }

    #[test]
    fn extract_json_payload_rejects_multiple_json_objects() {
        let error = extract_json_payload("{\"a\":1}\n{\"b\":2}")
            .expect_err("ambiguous JSON rejected");

        assert!(error.message.contains("multiple JSON objects"));
    }

    #[tokio::test]
    async fn invalid_candidate_is_written_to_quarantine_artifacts() {
        let pool = test_pool_with_transcript_analysis_stage().await;
        let input = test_stage_input_with_material_refs(["m_transcript_1"]);
        let output = serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/transcript_analysis",
            "video_candidate": {
                "summary_text": "Summary",
                "segment_candidates": [],
                "key_point_candidates": [],
                "quote_candidates": [],
                "action_item_candidates": [],
                "open_question_candidates": []
            },
            "claim_candidates": [
                {
                    "text": "Claim",
                    "material_refs": ["m_missing"]
                }
            ],
            "evidence_fragment_candidates": [],
            "warning_candidates": []
        });

        validate_and_quarantine_transcript_analysis_output(&pool, 42, 1001, &input, &output)
            .await
            .expect_err("invalid candidate rejected");

        let quarantine_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts \
             WHERE run_id = 42 AND stage_run_id = 1001 AND object_path = '$.claim_candidates[0]'",
        )
        .fetch_one(&pool)
        .await
        .expect("quarantine count");

        assert_eq!(quarantine_count, 1);
    }

    fn test_stage_input_with_material_refs<const N: usize>(
        material_refs: [&str; N],
    ) -> TranscriptAnalysisStageInput {
        TranscriptAnalysisStageInput {
            stage_io_version: "1.0".to_string(),
            schema_version: "1.0".to_string(),
            stage: "youtube_summary/transcript_analysis".to_string(),
            pack_id: "youtube_summary".to_string(),
            pack_version: "1.0.0".to_string(),
            run_id: 42,
            source_ref_id: "source_ref_1".to_string(),
            allowed_source_ref_ids: vec!["source_ref_1".to_string()],
            allowed_material_refs: material_refs.iter().map(|value| value.to_string()).collect(),
            transcript_segment_registry: vec![],
            comment_selection_policy: serde_json::json!({}),
            control_preset: "standard".to_string(),
            evidence_mode: "standard".to_string(),
            output_language: "en".to_string(),
        }
    }

    fn test_output_with_claim_id(claim_id: &str) -> serde_json::Value {
        serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/transcript_analysis",
            "video_candidate": {
                "summary_text": "Summary",
                "segment_candidates": [],
                "key_point_candidates": [],
                "quote_candidates": [],
                "action_item_candidates": [],
                "open_question_candidates": []
            },
            "claim_candidates": [
                {
                    "claim_id": claim_id,
                    "text": "Claim",
                    "material_refs": ["m_transcript_1"]
                }
            ],
            "evidence_fragment_candidates": [],
            "warning_candidates": []
        })
    }

    async fn test_pool_with_transcript_analysis_stage() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        seed_builtin_prompt_packs_in_pool(&pool).await.expect("seed");
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
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
                created_at, updated_at
             )
             VALUES (1001, 42, NULL, 'youtube_summary/transcript_analysis', 20, 'running',
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert stage");
        pool
    }
}
