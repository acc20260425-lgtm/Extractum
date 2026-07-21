use std::collections::HashSet;
use std::sync::OnceLock;

use extractum_core::compression::compress_text;
use extractum_core::error::{AppError, AppResult};
use extractum_core::time::now_rfc3339_utc;
use jsonschema::{error::ValidationErrorKind, ValidationError, Validator};
use sqlx::SqlitePool;

use super::assets::{SYNTHESIS_OUTPUT_SCHEMA_JSON, TRANSCRIPT_OUTPUT_SCHEMA_JSON};
use super::stage_io::TranscriptAnalysisStageInput;
use super::stage_output_normalization::{
    normalize_synthesis_output_for_runtime, normalize_transcript_analysis_output_for_schema,
};
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
    validate_transcript_analysis_output_schema(output)?;
    reject_backend_owned_ids(output, "$")?;
    let allowed_material_refs = input
        .allowed_material_refs
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    reject_unknown_material_refs(output, "$", &allowed_material_refs)?;
    Ok(())
}

fn validate_transcript_analysis_output_schema(
    output: &serde_json::Value,
) -> Result<(), PromptPackValidationError> {
    let output = normalize_transcript_analysis_output_for_schema(output);
    transcript_analysis_output_validator().and_then(|validator| {
        validator.validate(&output).map_err(|error| {
            let object_path = jsonschema_error_object_path(&error);
            PromptPackValidationError {
                message: format!("schema validation failed at {object_path}: {error}"),
                object_path: Some(object_path),
            }
        })
    })
}

fn transcript_analysis_output_validator() -> Result<&'static Validator, PromptPackValidationError> {
    static VALIDATOR: OnceLock<Result<Validator, String>> = OnceLock::new();
    VALIDATOR
        .get_or_init(|| {
            let schema: serde_json::Value = serde_json::from_str(TRANSCRIPT_OUTPUT_SCHEMA_JSON)
                .map_err(|error| {
                    format!("invalid transcript analysis output schema JSON: {error}")
                })?;
            jsonschema::validator_for(&schema)
                .map_err(|error| format!("invalid transcript analysis output schema: {error}"))
        })
        .as_ref()
        .map_err(|message| PromptPackValidationError {
            message: message.clone(),
            object_path: Some("$".to_string()),
        })
}

fn jsonschema_instance_path_to_object_path(instance_path: &str) -> String {
    if instance_path.is_empty() {
        return "$".to_string();
    }

    let mut object_path = "$".to_string();
    for segment in instance_path.trim_start_matches('/').split('/') {
        let segment = segment.replace("~1", "/").replace("~0", "~");
        if segment.chars().all(|character| character.is_ascii_digit()) {
            object_path.push('[');
            object_path.push_str(&segment);
            object_path.push(']');
        } else {
            object_path.push('.');
            object_path.push_str(&segment);
        }
    }
    object_path
}

fn jsonschema_error_object_path(error: &ValidationError<'_>) -> String {
    let object_path = jsonschema_instance_path_to_object_path(&error.instance_path().to_string());
    match error.kind() {
        ValidationErrorKind::Required { property } => property
            .as_str()
            .map(|property| jsonschema_child_object_path(&object_path, property))
            .unwrap_or(object_path),
        _ => object_path,
    }
}

fn jsonschema_child_object_path(parent_path: &str, property: &str) -> String {
    if parent_path == "$" {
        format!("$.{property}")
    } else {
        format!("{parent_path}.{property}")
    }
}

pub(crate) async fn quarantine_prompt_pack_validation_error(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    output: &serde_json::Value,
    error: PromptPackValidationError,
) -> AppResult<()> {
    let object_path = error.object_path.clone().unwrap_or_else(|| "$".to_string());
    let validation_message = error.message.clone();
    let candidate = value_at_path(output, &object_path).unwrap_or(output);
    let content = serde_json::to_string(candidate).unwrap_or_else(|_| "{}".to_string());
    sqlx::query(
        "INSERT INTO prompt_pack_result_quarantine_artifacts (
            run_id, stage_run_id, object_path, reason, content_json_zstd, created_at
         )
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(stage_run_id)
    .bind(&object_path)
    .bind(&validation_message)
    .bind(compress_text(&content).unwrap_or_default())
    .bind(now_rfc3339_utc())
    .execute(pool)
    .await
    .map_err(|db_error| {
        AppError::internal(format!(
            "quarantine prompt pack validation error `{validation_message}` failed: {db_error}"
        ))
    })?;
    Ok(())
}

pub(crate) fn validate_synthesis_output(
    output: &serde_json::Value,
    allowed_source_refs: &HashSet<String>,
) -> Result<(), PromptPackValidationError> {
    validate_synthesis_output_schema(output)?;
    let candidate = output
        .get("synthesis_candidate")
        .ok_or_else(|| PromptPackValidationError {
            message: "missing required key synthesis_candidate".to_string(),
            object_path: Some("$.synthesis_candidate".to_string()),
        })?;
    expect_non_empty_string_at(
        candidate,
        "summary_text",
        "$.synthesis_candidate.summary_text",
    )?;
    reject_non_empty_synthesis_candidate_ref_arrays(candidate, "$.synthesis_candidate")?;
    reject_backend_owned_ids(output, "$")?;
    reject_unknown_synthesis_source_refs(output, "$", allowed_source_refs)?;
    Ok(())
}

fn validate_synthesis_output_schema(
    output: &serde_json::Value,
) -> Result<(), PromptPackValidationError> {
    let output = normalize_synthesis_output_for_runtime(output);
    synthesis_output_validator().and_then(|validator| {
        validator.validate(&output).map_err(|error| {
            let object_path = jsonschema_error_object_path(&error);
            PromptPackValidationError {
                message: format!("schema validation failed at {object_path}: {error}"),
                object_path: Some(object_path),
            }
        })
    })
}

fn synthesis_output_validator() -> Result<&'static Validator, PromptPackValidationError> {
    static VALIDATOR: OnceLock<Result<Validator, String>> = OnceLock::new();
    VALIDATOR
        .get_or_init(|| {
            let schema: serde_json::Value = serde_json::from_str(SYNTHESIS_OUTPUT_SCHEMA_JSON)
                .map_err(|error| format!("invalid synthesis output schema JSON: {error}"))?;
            jsonschema::validator_for(&schema)
                .map_err(|error| format!("invalid synthesis output schema: {error}"))
        })
        .as_ref()
        .map_err(|message| PromptPackValidationError {
            message: message.clone(),
            object_path: Some("$".to_string()),
        })
}

pub(crate) fn validate_synthesis_output_with_allowed_refs(
    output: &serde_json::Value,
    allowed_source_refs: &HashSet<String>,
    allowed_claim_refs: &HashSet<String>,
    allowed_evidence_refs: &HashSet<String>,
) -> Result<(), PromptPackValidationError> {
    validate_synthesis_output(output, allowed_source_refs)?;
    let candidate = output
        .get("synthesis_candidate")
        .ok_or_else(|| PromptPackValidationError {
            message: "missing required key synthesis_candidate".to_string(),
            object_path: Some("$.synthesis_candidate".to_string()),
        })?;
    reject_direct_intermediate_refs(candidate, "$.synthesis_candidate")?;
    reject_unknown_refs_in_synthesis(
        candidate,
        "$.synthesis_candidate",
        "claim_refs",
        "claim_ref",
        allowed_claim_refs,
    )?;
    reject_unknown_refs_in_synthesis(
        candidate,
        "$.synthesis_candidate",
        "evidence_refs",
        "evidence_ref",
        allowed_evidence_refs,
    )?;
    Ok(())
}

pub(crate) async fn validate_and_quarantine_synthesis_output(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    output: &serde_json::Value,
) -> AppResult<()> {
    let empty_source_refs = HashSet::new();
    if let Err(error) = validate_synthesis_output(output, &empty_source_refs) {
        if !error.message.contains("unknown synthesis source ref") {
            return quarantine_synthesis_output(pool, run_id, stage_run_id, output, error).await;
        }
    }

    let allowed_source_refs = load_allowed_synthesis_source_refs(pool, run_id).await?;
    match validate_synthesis_output(output, &allowed_source_refs) {
        Ok(()) => Ok(()),
        Err(error) => quarantine_synthesis_output(pool, run_id, stage_run_id, output, error).await,
    }
}

async fn quarantine_synthesis_output(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    output: &serde_json::Value,
    error: PromptPackValidationError,
) -> AppResult<()> {
    let validation_message = error.message.clone();
    quarantine_prompt_pack_validation_error(pool, run_id, stage_run_id, output, error).await?;
    Err(AppError::validation(validation_message))
}

async fn load_allowed_synthesis_source_refs(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<HashSet<String>> {
    sqlx::query_scalar::<_, String>(
        "SELECT source_ref_id FROM prompt_pack_run_source_snapshots
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|refs| refs.into_iter().collect())
    .map_err(AppError::database)
}

fn expect_non_empty_string_at(
    output: &serde_json::Value,
    key: &str,
    object_path: &str,
) -> Result<(), PromptPackValidationError> {
    if output
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
    {
        Ok(())
    } else {
        Err(PromptPackValidationError {
            message: format!("{key} must be a non-empty string"),
            object_path: Some(object_path.to_string()),
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
        "section_id",
        "video_id",
        "synthesis_item_id",
        "theme_id",
        "common_claim_id",
        "contradiction_id",
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

fn reject_unknown_synthesis_source_refs(
    value: &serde_json::Value,
    path: &str,
    allowed_source_refs: &HashSet<String>,
) -> Result<(), PromptPackValidationError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let child_path = format!("{path}.{key}");
                if key == "source_refs" {
                    let refs = nested.as_array().ok_or_else(|| PromptPackValidationError {
                        message: "source_refs must be an array".to_string(),
                        object_path: Some(child_path.clone()),
                    })?;
                    for (index, item) in refs.iter().enumerate() {
                        let item_path = format!("{child_path}[{index}]");
                        let source_ref =
                            item.as_str().ok_or_else(|| PromptPackValidationError {
                                message: "source_refs entries must be strings".to_string(),
                                object_path: Some(item_path.clone()),
                            })?;
                        if !allowed_source_refs.contains(source_ref) {
                            return Err(PromptPackValidationError {
                                message: format!("unknown synthesis source ref {source_ref}"),
                                object_path: Some(item_path),
                            });
                        }
                    }
                } else {
                    reject_unknown_synthesis_source_refs(nested, &child_path, allowed_source_refs)?;
                }
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                reject_unknown_synthesis_source_refs(
                    nested,
                    &format!("{path}[{index}]"),
                    allowed_source_refs,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_non_empty_synthesis_candidate_ref_arrays(
    value: &serde_json::Value,
    path: &str,
) -> Result<(), PromptPackValidationError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let child_path = format!("{path}.{key}");
                if key == "relation_refs" {
                    let refs = nested.as_array().ok_or_else(|| PromptPackValidationError {
                        message: format!("{key} must be an array"),
                        object_path: Some(child_path.clone()),
                    })?;
                    if !refs.is_empty() {
                        return Err(PromptPackValidationError {
                            message: format!(
                                "{key} must be empty in synthesis_candidate until the backend exposes an allowed ref map"
                            ),
                            object_path: Some(child_path),
                        });
                    }
                } else {
                    reject_non_empty_synthesis_candidate_ref_arrays(nested, &child_path)?;
                }
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                reject_non_empty_synthesis_candidate_ref_arrays(
                    nested,
                    &format!("{path}[{index}]"),
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_direct_intermediate_refs(
    value: &serde_json::Value,
    path: &str,
) -> Result<(), PromptPackValidationError> {
    match value {
        serde_json::Value::Object(map) => {
            for forbidden in ["segment_refs", "key_point_refs", "quote_refs"] {
                if map.contains_key(forbidden) {
                    return Err(PromptPackValidationError {
                        message: format!(
                            "direct {forbidden} are not allowed in synthesis output v1"
                        ),
                        object_path: Some(format!("{path}.{forbidden}")),
                    });
                }
            }
            for (key, child) in map {
                reject_direct_intermediate_refs(child, &format!("{path}.{key}"))?;
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                reject_direct_intermediate_refs(child, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_unknown_refs_in_synthesis(
    value: &serde_json::Value,
    path: &str,
    key_to_check: &str,
    singular_name: &str,
    allowed_refs: &HashSet<String>,
) -> Result<(), PromptPackValidationError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let child_path = format!("{path}.{key}");
                if key == key_to_check {
                    let refs = nested.as_array().ok_or_else(|| PromptPackValidationError {
                        message: format!("{key_to_check} must be an array of strings"),
                        object_path: Some(child_path.clone()),
                    })?;
                    for (index, item) in refs.iter().enumerate() {
                        let item_path = format!("{child_path}[{index}]");
                        let reference = item.as_str().ok_or_else(|| PromptPackValidationError {
                            message: format!("{key_to_check} must be an array of strings"),
                            object_path: Some(item_path.clone()),
                        })?;
                        if !allowed_refs.contains(reference) {
                            return Err(PromptPackValidationError {
                                message: format!("unknown {singular_name} {reference}"),
                                object_path: Some(item_path),
                            });
                        }
                    }
                } else {
                    reject_unknown_refs_in_synthesis(
                        nested,
                        &child_path,
                        key_to_check,
                        singular_name,
                        allowed_refs,
                    )?;
                }
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                reject_unknown_refs_in_synthesis(
                    nested,
                    &format!("{path}[{index}]"),
                    key_to_check,
                    singular_name,
                    allowed_refs,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
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
                reject_unknown_material_refs(
                    child,
                    &format!("{path}.{key}"),
                    allowed_material_refs,
                )?;
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

fn value_at_path<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
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
        validate_and_quarantine_synthesis_output, validate_synthesis_output,
        validate_synthesis_output_with_allowed_refs, validate_transcript_analysis_output,
    };
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::stage_io::{extract_json_payload, TranscriptAnalysisStageInput};
    use crate::prompt_packs::test_schema::prompt_pack_test_pool;

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

        let error =
            validate_transcript_analysis_output(&input, &output).expect_err("final ids rejected");

        assert!(error.message.contains("claim_id"));
    }

    #[test]
    fn transcript_analysis_output_rejects_structural_schema_errors() {
        let input = test_stage_input_with_material_refs(["m_transcript_1"]);
        let output = serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/transcript_analysis",
            "video_candidate": "not an object",
            "claim_candidates": [],
            "evidence_fragment_candidates": [],
            "warning_candidates": []
        });

        let error = validate_transcript_analysis_output(&input, &output)
            .expect_err("schema structural error rejected");

        assert!(error.message.contains("video_candidate"));
        assert_eq!(error.object_path.as_deref(), Some("$.video_candidate"));
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
        let error =
            extract_json_payload("{\"a\":1}\n{\"b\":2}").expect_err("ambiguous JSON rejected");

        assert!(error.message.contains("multiple JSON objects"));
    }

    #[test]
    fn synthesis_output_validator_accepts_valid_output() {
        let output = valid_synthesis_output();

        validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect("valid synthesis output");
    }

    #[test]
    fn synthesis_output_validator_rejects_missing_summary_text() {
        let mut output = valid_synthesis_output();
        output["synthesis_candidate"]
            .as_object_mut()
            .expect("candidate")
            .remove("summary_text");

        let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect_err("missing summary rejected");

        assert!(error.message.contains("summary_text"));
        assert_eq!(
            error.object_path.as_deref(),
            Some("$.synthesis_candidate.summary_text")
        );
    }

    #[test]
    fn synthesis_output_validator_rejects_wrong_stage_io_version() {
        let mut output = valid_synthesis_output();
        output["stage_io_version"] = serde_json::json!("2.0");

        let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect_err("stage io version rejected");

        assert!(error.message.contains("stage_io_version"));
        assert_eq!(error.object_path.as_deref(), Some("$.stage_io_version"));
    }

    #[test]
    fn synthesis_output_validator_rejects_wrong_schema_version() {
        let mut output = valid_synthesis_output();
        output["schema_version"] = serde_json::json!("2.0");

        let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect_err("schema version rejected");

        assert!(error.message.contains("schema_version"));
        assert_eq!(error.object_path.as_deref(), Some("$.schema_version"));
    }

    #[test]
    fn synthesis_output_accepts_provider_string_items_for_readable_arrays() {
        let mut output = valid_synthesis_output();
        output["synthesis_candidate"]["common_claims"] = serde_json::json!(["Common claim"]);
        output["limitations"] = serde_json::json!(["Limitation"]);

        validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect("provider string items accepted");
    }

    #[test]
    fn synthesis_output_validator_rejects_wrong_stage() {
        let mut output = valid_synthesis_output();
        output["stage"] = serde_json::json!("youtube_summary/transcript_analysis");

        let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect_err("wrong stage rejected");

        assert!(error.message.contains("stage"));
        assert_eq!(error.object_path.as_deref(), Some("$.stage"));
    }

    #[test]
    fn synthesis_output_validator_rejects_non_array_fields() {
        let mut output = valid_synthesis_output();
        output["synthesis_candidate"]["cross_video_themes"] = serde_json::json!("not an array");

        let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect_err("array contract rejected");

        assert!(error.message.contains("cross_video_themes"));
        assert_eq!(
            error.object_path.as_deref(),
            Some("$.synthesis_candidate.cross_video_themes")
        );
    }

    #[test]
    fn synthesis_output_validator_rejects_structural_schema_errors() {
        let mut output = valid_synthesis_output();
        output["synthesis_candidate"] = serde_json::json!("not an object");

        let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect_err("schema structural error rejected");

        assert!(error.message.contains("synthesis_candidate"));
        assert_eq!(error.object_path.as_deref(), Some("$.synthesis_candidate"));
    }

    #[test]
    fn synthesis_output_validator_rejects_backend_owned_ids() {
        let mut output = valid_synthesis_output();
        output["synthesis_candidate"]["cross_video_themes"] = serde_json::json!([
            {
                "theme_id": "theme_1",
                "text": "Provider must not assign final IDs"
            }
        ]);

        let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect_err("backend ids rejected");

        assert!(error.message.contains("theme_id"));
    }

    #[test]
    fn synthesis_output_validator_rejects_unknown_source_ref() {
        let mut output = valid_synthesis_output();
        output["synthesis_candidate"]["cross_video_themes"][0]["source_refs"] =
            serde_json::json!(["source_ref_999"]);

        let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
            .expect_err("unknown source ref rejected");

        assert!(error.message.contains("source_ref_999"));
        assert_eq!(
            error.object_path.as_deref(),
            Some("$.synthesis_candidate.cross_video_themes[0].source_refs[0]")
        );
    }

    #[test]
    fn synthesis_output_validator_rejects_provider_authored_claim_ref() {
        let output = valid_synthesis_output_with_refs(
            serde_json::json!(["source_ref_1"]),
            serde_json::json!(["claim_999"]),
            serde_json::json!([]),
        );
        let allowed_claims = std::collections::HashSet::from(["source_ref_1_claim_1".to_string()]);
        let allowed_evidence =
            std::collections::HashSet::from(["source_ref_1_evidence_1".to_string()]);

        let error = validate_synthesis_output_with_allowed_refs(
            &output,
            &allowed_synthesis_source_refs(),
            &allowed_claims,
            &allowed_evidence,
        )
        .expect_err("provider-authored claim ref rejected");

        assert!(error.message.contains("unknown claim_ref claim_999"));
        assert_eq!(
            error.object_path.as_deref(),
            Some("$.synthesis_candidate.cross_video_themes[0].claim_refs[0]")
        );
    }

    #[test]
    fn synthesis_output_rejects_unknown_claim_ref() {
        let output = valid_synthesis_output_with_refs(
            serde_json::json!(["source_ref_1"]),
            serde_json::json!(["claim_999"]),
            serde_json::json!([]),
        );
        let allowed_sources = std::collections::HashSet::from(["source_ref_1".to_string()]);
        let allowed_claims = std::collections::HashSet::from(["source_ref_1_claim_1".to_string()]);
        let allowed_evidence =
            std::collections::HashSet::from(["source_ref_1_evidence_1".to_string()]);

        let error = validate_synthesis_output_with_allowed_refs(
            &output,
            &allowed_sources,
            &allowed_claims,
            &allowed_evidence,
        )
        .expect_err("unknown claim rejected");

        assert!(error.message.contains("unknown claim_ref claim_999"));
    }

    #[test]
    fn synthesis_output_rejects_direct_segment_key_point_or_quote_refs_inside_synthesis_candidate()
    {
        for key in ["segment_refs", "key_point_refs", "quote_refs"] {
            let mut output = valid_synthesis_output_with_refs(
                serde_json::json!(["source_ref_1"]),
                serde_json::json!([]),
                serde_json::json!([]),
            );
            output["synthesis_candidate"]["cross_video_themes"][0][key] =
                serde_json::json!(["not_allowed"]);

            let allowed_sources = std::collections::HashSet::from(["source_ref_1".to_string()]);
            let allowed_claims = std::collections::HashSet::new();
            let allowed_evidence = std::collections::HashSet::new();
            let error = validate_synthesis_output_with_allowed_refs(
                &output,
                &allowed_sources,
                &allowed_claims,
                &allowed_evidence,
            )
            .expect_err("direct intermediate ref rejected inside synthesis_candidate");

            assert!(error.message.contains(key));
        }
    }

    #[test]
    fn synthesis_output_rejects_non_array_or_non_string_ref_values() {
        let allowed_sources = std::collections::HashSet::from(["source_ref_1".to_string()]);
        let allowed_claims = std::collections::HashSet::from(["source_ref_1_claim_1".to_string()]);
        let allowed_evidence =
            std::collections::HashSet::from(["source_ref_1_evidence_1".to_string()]);

        let mut non_array = valid_synthesis_output_with_refs(
            serde_json::json!(["source_ref_1"]),
            serde_json::json!("source_ref_1_claim_1"),
            serde_json::json!([]),
        );
        let error = validate_synthesis_output_with_allowed_refs(
            &non_array,
            &allowed_sources,
            &allowed_claims,
            &allowed_evidence,
        )
        .expect_err("claim_refs string rejected");
        assert!(error
            .message
            .contains("claim_refs must be an array of strings"));

        non_array["synthesis_candidate"]["cross_video_themes"][0]["claim_refs"] =
            serde_json::json!(["source_ref_1_claim_1", 42]);
        let error = validate_synthesis_output_with_allowed_refs(
            &non_array,
            &allowed_sources,
            &allowed_claims,
            &allowed_evidence,
        )
        .expect_err("non-string claim ref rejected");
        assert!(error
            .message
            .contains("claim_refs must be an array of strings"));
    }

    #[tokio::test]
    async fn invalid_synthesis_output_is_written_to_quarantine_artifacts() {
        let pool = test_pool_with_synthesis_stage().await;
        let mut output = valid_synthesis_output();
        output["warning_candidates"] = serde_json::json!([
            {
                "source_ref_id": "source_ref_1",
                "text": "Provider must not assign backend source refs"
            }
        ]);

        validate_and_quarantine_synthesis_output(&pool, 42, 2001, &output)
            .await
            .expect_err("invalid synthesis rejected");

        let quarantine_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
             WHERE run_id = 42 AND stage_run_id = 2001",
        )
        .fetch_one(&pool)
        .await
        .expect("quarantine count");

        assert_eq!(quarantine_count, 1);
    }

    #[tokio::test]
    async fn synthesis_quarantine_artifact_uses_current_time() {
        let pool = test_pool_with_synthesis_stage().await;
        let mut output = valid_synthesis_output();
        output["warning_candidates"] = serde_json::json!([
            {
                "source_ref_id": "source_ref_1",
                "text": "Provider must not assign backend source refs"
            }
        ]);

        validate_and_quarantine_synthesis_output(&pool, 42, 2001, &output)
            .await
            .expect_err("invalid synthesis rejected");

        assert_quarantine_created_at_is_current(&pool, 2001).await;
    }

    #[tokio::test]
    async fn invalid_synthesis_output_with_unknown_source_ref_is_quarantined() {
        let pool = test_pool_with_synthesis_stage().await;
        let mut output = valid_synthesis_output();
        output["synthesis_candidate"]["cross_video_themes"][0]["source_refs"] =
            serde_json::json!(["source_ref_999"]);

        validate_and_quarantine_synthesis_output(&pool, 42, 2001, &output)
            .await
            .expect_err("unknown source ref rejected");

        let quarantine_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
             WHERE run_id = 42
               AND stage_run_id = 2001
               AND reason LIKE '%source_ref_999%'",
        )
        .fetch_one(&pool)
        .await
        .expect("quarantine count");

        assert_eq!(quarantine_count, 1);
    }

    #[tokio::test]
    async fn invalid_synthesis_output_surfaces_quarantine_write_failure() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite without migrations");
        let mut output = valid_synthesis_output();
        output["synthesis_candidate"]["cross_video_themes"] = serde_json::json!([
            {
                "theme_id": "theme_1",
                "theme_text": "Provider must not assign backend IDs"
            }
        ]);

        let error = validate_and_quarantine_synthesis_output(&pool, 42, 2001, &output)
            .await
            .expect_err("quarantine write failure is surfaced");

        assert!(error
            .message
            .contains("quarantine prompt pack validation error"));
    }

    fn valid_synthesis_output() -> serde_json::Value {
        valid_synthesis_output_with_refs(
            serde_json::json!(["source_ref_1", "source_ref_2"]),
            serde_json::json!([]),
            serde_json::json!([]),
        )
    }

    fn valid_synthesis_output_with_refs(
        source_refs: serde_json::Value,
        claim_refs: serde_json::Value,
        evidence_refs: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/synthesis",
            "synthesis_candidate": {
                "summary_text": "Combined summary",
                "cross_video_themes": [
                    {
                        "theme_text": "Shared theme",
                        "source_refs": source_refs,
                        "claim_refs": claim_refs,
                        "evidence_refs": evidence_refs
                    }
                ],
                "common_claims": [],
                "contradictions_across_videos": []
            },
            "limitations": [],
            "warning_candidates": []
        })
    }

    fn allowed_synthesis_source_refs() -> std::collections::HashSet<String> {
        ["source_ref_1", "source_ref_2"]
            .into_iter()
            .map(ToString::to_string)
            .collect()
    }

    async fn assert_quarantine_created_at_is_current(pool: &sqlx::SqlitePool, stage_run_id: i64) {
        use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

        let created_at: String = sqlx::query_scalar(
            "SELECT created_at FROM prompt_pack_result_quarantine_artifacts
             WHERE run_id = 42 AND stage_run_id = ?
             ORDER BY id DESC
             LIMIT 1",
        )
        .bind(stage_run_id)
        .fetch_one(pool)
        .await
        .expect("quarantine created_at");
        let parsed = OffsetDateTime::parse(&created_at, &Rfc3339).expect("parse created_at");
        let before = OffsetDateTime::now_utc() - Duration::seconds(5);
        let after = OffsetDateTime::now_utc() + Duration::seconds(5);

        assert_ne!(created_at, "2026-06-14T00:00:00Z");
        assert!(
            parsed >= before && parsed <= after,
            "expected {created_at} to be between {before} and {after}"
        );
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
            allowed_material_refs: material_refs
                .iter()
                .map(|value| value.to_string())
                .collect(),
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

    async fn test_pool_with_synthesis_stage() -> sqlx::SqlitePool {
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
             VALUES
                (901, 'youtube', 'video', 'provider-video-1', 'Video 1', 1, 0, 1),
                (902, 'youtube', 'video', 'provider-video-2', 'Video 2', 1, 0, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert sources");
        sqlx::query(
            "INSERT INTO prompt_pack_run_source_snapshots (
                id, run_id, source_id, source_ref_id, video_id, title, created_at
             )
             VALUES
                (501, 42, 901, 'source_ref_1', 'provider-video-1', 'Video 1', '2026-06-14T00:00:00Z'),
                (502, 42, 902, 'source_ref_2', 'provider-video-2', 'Video 2', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert snapshots");
        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
                created_at, updated_at
             )
             VALUES (2001, 42, NULL, 'youtube_summary/synthesis', 103, 'running',
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert synthesis stage");
        pool
    }
}
