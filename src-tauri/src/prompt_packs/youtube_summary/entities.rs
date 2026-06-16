use std::collections::{HashMap, HashSet};

use sqlx::SqlitePool;

use crate::compression::decompress_text;
use crate::error::{AppError, AppResult};
use crate::prompt_packs::stage_io::TranscriptAnalysisStageInput;
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, insert_stage_artifact_in_pool,
};
use crate::prompt_packs::validation::{
    quarantine_prompt_pack_validation_error, PromptPackValidationError,
};

pub(crate) const INTERMEDIATE_ENTITIES_ARTIFACT_KIND: &str = "intermediate_entities";
pub(crate) const YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND: &str =
    "youtube_summary_intermediate_entities";

pub(crate) fn build_source_intermediate_entities(
    input: &TranscriptAnalysisStageInput,
    source_snapshot_id: i64,
    title: Option<&str>,
    parsed: &serde_json::Value,
    attempt_number: i64,
) -> Result<serde_json::Value, PromptPackValidationError> {
    validate_graph_kind_for_pack(&input.pack_id)?;
    let allowed_material_refs = input
        .allowed_material_refs
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let mut warnings = Vec::new();

    let video_candidate = parsed.get("video_candidate").ok_or_else(|| {
        validation_error("missing required key video_candidate", "$.video_candidate")
    })?;

    let segments = build_segments(
        &input.source_ref_id,
        video_candidate,
        &allowed_material_refs,
    )?;
    let key_points = build_key_points(
        &input.source_ref_id,
        video_candidate,
        &allowed_material_refs,
        &segments.index_to_ref,
        &mut warnings,
    )?;
    let quotes = build_quotes(
        &input.source_ref_id,
        video_candidate,
        &allowed_material_refs,
        &segments.index_to_ref,
        &mut warnings,
    )?;
    let claims = build_claims(
        &input.source_ref_id,
        parsed,
        &allowed_material_refs,
        &mut warnings,
    )?;
    let evidence = build_evidence(
        &input.source_ref_id,
        parsed,
        &allowed_material_refs,
        &quotes.index_to_ref,
        &mut warnings,
    )?;

    Ok(serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "graph_kind": YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND,
        "run_id": input.run_id,
        "attempt_number": attempt_number,
        "sources": [{
            "source_ref_id": input.source_ref_id,
            "source_snapshot_id": source_snapshot_id,
            "title": title
        }],
        "segments": segments.items,
        "key_points": key_points.items,
        "quotes": quotes.items,
        "claims": claims,
        "evidence": evidence,
        "warnings": warnings,
        "allowed_refs": allowed_refs(
            &input.source_ref_id,
            &segments.items,
            &key_points.items,
            &quotes.items,
            &claims,
            &evidence
        )
    }))
}

pub(crate) async fn load_intermediate_entities_context_for_transcript_stage(
    pool: &SqlitePool,
    stage_run_id: i64,
) -> AppResult<(TranscriptAnalysisStageInput, i64, Option<String>)> {
    let input = build_transcript_analysis_stage_input(pool, stage_run_id).await?;
    let (source_snapshot_id, title): (i64, Option<String>) = sqlx::query_as(
        "SELECT snapshots.id, snapshots.title
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.id = ?",
    )
    .bind(stage_run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok((input, source_snapshot_id, title))
}

pub(crate) async fn build_or_quarantine_intermediate_entities_for_transcript_stage(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    parsed: &serde_json::Value,
    attempt_number: i64,
) -> AppResult<serde_json::Value> {
    let (input, source_snapshot_id, title) =
        load_intermediate_entities_context_for_transcript_stage(pool, stage_run_id).await?;
    match build_source_intermediate_entities(
        &input,
        source_snapshot_id,
        title.as_deref(),
        parsed,
        attempt_number,
    ) {
        Ok(graph) => Ok(graph),
        Err(error) => {
            let validation_message = error.message.clone();
            quarantine_prompt_pack_validation_error(pool, run_id, stage_run_id, parsed, error)
                .await?;
            Err(AppError::validation(validation_message))
        }
    }
}

pub(crate) async fn insert_intermediate_entities_artifact(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    graph: &serde_json::Value,
    attempt_number: i64,
) -> AppResult<()> {
    let content = serde_json::to_string(graph)
        .map_err(|error| AppError::internal(format!("serialize intermediate entities: {error}")))?;
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        INTERMEDIATE_ENTITIES_ARTIFACT_KIND,
        attempt_number,
        5,
        &content,
    )
    .await
}

pub(crate) async fn load_merged_intermediate_entities_for_run(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Option<serde_json::Value>> {
    let rows = sqlx::query_as::<_, (i64, i64, String, Vec<u8>)>(
        "SELECT snapshots.id, stages.id, snapshots.source_ref_id, artifacts.content_zstd
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
    .map_err(AppError::database)?;

    if rows.is_empty() {
        return Ok(None);
    }

    let mut merged = empty_run_graph(run_id);
    for (_source_snapshot_id, _stage_run_id, _source_ref_id, content_zstd) in rows {
        let text = decompress_text(&content_zstd).map_err(AppError::internal)?;
        let graph: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| AppError::internal(format!("parse intermediate entities: {error}")))?;
        merge_source_graph(&mut merged, &graph)?;
    }

    Ok(Some(merged))
}

pub(crate) struct AllowedSynthesisRefs {
    pub(crate) source_refs: HashSet<String>,
    pub(crate) claim_refs: HashSet<String>,
    pub(crate) evidence_refs: HashSet<String>,
}

impl AllowedSynthesisRefs {
    fn empty() -> Self {
        Self {
            source_refs: HashSet::new(),
            claim_refs: HashSet::new(),
            evidence_refs: HashSet::new(),
        }
    }
}

pub(crate) async fn load_required_allowed_refs_for_live_synthesis(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<AllowedSynthesisRefs> {
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
    let graph = load_merged_intermediate_entities_for_run(pool, run_id).await?;
    let Some(graph) = graph else {
        if !expected_sources.is_empty() {
            return Err(AppError::validation(
                "missing complete intermediate_entities graph for live synthesis",
            ));
        }
        return Ok(AllowedSynthesisRefs::empty());
    };

    let graph_sources = graph
        .get("sources")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let graph_source_keys = graph_sources
        .iter()
        .map(|source| {
            Ok((
                source
                    .get("source_snapshot_id")
                    .and_then(serde_json::Value::as_i64)
                    .ok_or_else(|| {
                        AppError::internal("intermediate graph source missing source_snapshot_id")
                    })?,
                source
                    .get("source_ref_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        AppError::internal("intermediate graph source missing source_ref_id")
                    })?
                    .to_string(),
            ))
        })
        .collect::<AppResult<Vec<_>>>()?;
    if graph_source_keys != expected_sources {
        return Err(AppError::validation(
            "missing complete intermediate_entities graph for live synthesis",
        ));
    }

    let allowed = graph
        .get("allowed_refs")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(AllowedSynthesisRefs {
        source_refs: string_set(allowed.get("source_refs")),
        claim_refs: string_set(allowed.get("claim_refs")),
        evidence_refs: string_set(allowed.get("evidence_refs")),
    })
}

#[derive(Clone)]
struct BuiltIndexedEntities {
    items: Vec<serde_json::Value>,
    index_to_ref: HashMap<usize, String>,
}

fn build_segments(
    source_ref_id: &str,
    video_candidate: &serde_json::Value,
    allowed_material_refs: &HashSet<String>,
) -> Result<BuiltIndexedEntities, PromptPackValidationError> {
    let candidates = candidate_array(
        video_candidate,
        "segment_candidates",
        "$.video_candidate.segment_candidates",
        true,
    )?;
    let mut items = Vec::new();
    let mut index_to_ref = HashMap::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let path = format!("$.video_candidate.segment_candidates[{index}]");
        let object = candidate_object(candidate, &path)?;
        reject_backend_owned_fields(object, &path)?;
        let material_refs = material_refs(
            object,
            &format!("{path}.material_refs"),
            allowed_material_refs,
        )?;
        let segment_ref = format!("{source_ref_id}_segment_{}", items.len() + 1);
        let title = optional_string_field(object, "title", &format!("{path}.title"))?;
        let summary_text =
            optional_string_field(object, "summary_text", &format!("{path}.summary_text"))?;

        index_to_ref.insert(index, segment_ref.clone());
        items.push(serde_json::json!({
            "segment_ref": segment_ref,
            "source_ref_id": source_ref_id,
            "order_index": items.len(),
            "title": title,
            "summary_text": summary_text,
            "material_refs": material_refs
        }));
    }

    Ok(BuiltIndexedEntities {
        items,
        index_to_ref,
    })
}

fn build_key_points(
    source_ref_id: &str,
    video_candidate: &serde_json::Value,
    allowed_material_refs: &HashSet<String>,
    segment_refs: &HashMap<usize, String>,
    warnings: &mut Vec<serde_json::Value>,
) -> Result<BuiltIndexedEntities, PromptPackValidationError> {
    let candidates = candidate_array(
        video_candidate,
        "key_point_candidates",
        "$.video_candidate.key_point_candidates",
        true,
    )?;
    let mut items = Vec::new();
    let mut index_to_ref = HashMap::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let path = format!("$.video_candidate.key_point_candidates[{index}]");
        let object = candidate_object(candidate, &path)?;
        reject_backend_owned_fields(object, &path)?;
        let material_refs = material_refs(
            object,
            &format!("{path}.material_refs"),
            allowed_material_refs,
        )?;
        let Some(text) = semantic_text(object, "key_point", index, warnings)? else {
            continue;
        };
        let segment_ref = optional_index_ref(
            object,
            "segment_candidate_index",
            &format!("{path}.segment_candidate_index"),
            segment_refs,
            "segment_candidate_index",
            "segment candidate",
        )?;
        let key_point_ref = format!("{source_ref_id}_key_point_{}", items.len() + 1);

        index_to_ref.insert(index, key_point_ref.clone());
        items.push(serde_json::json!({
            "key_point_ref": key_point_ref,
            "source_ref_id": source_ref_id,
            "text": text,
            "segment_ref": segment_ref,
            "material_refs": material_refs
        }));
    }

    Ok(BuiltIndexedEntities {
        items,
        index_to_ref,
    })
}

fn build_quotes(
    source_ref_id: &str,
    video_candidate: &serde_json::Value,
    allowed_material_refs: &HashSet<String>,
    segment_refs: &HashMap<usize, String>,
    warnings: &mut Vec<serde_json::Value>,
) -> Result<BuiltIndexedEntities, PromptPackValidationError> {
    let candidates = candidate_array(
        video_candidate,
        "quote_candidates",
        "$.video_candidate.quote_candidates",
        true,
    )?;
    let mut items = Vec::new();
    let mut index_to_ref = HashMap::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let path = format!("$.video_candidate.quote_candidates[{index}]");
        let object = candidate_object(candidate, &path)?;
        reject_backend_owned_fields(object, &path)?;
        let material_refs = material_refs(
            object,
            &format!("{path}.material_refs"),
            allowed_material_refs,
        )?;
        let Some(text) = semantic_text(object, "quote", index, warnings)? else {
            continue;
        };
        let segment_ref = optional_index_ref(
            object,
            "segment_candidate_index",
            &format!("{path}.segment_candidate_index"),
            segment_refs,
            "segment_candidate_index",
            "segment candidate",
        )?;
        let quote_ref = format!("{source_ref_id}_quote_{}", items.len() + 1);

        index_to_ref.insert(index, quote_ref.clone());
        items.push(serde_json::json!({
            "quote_ref": quote_ref,
            "source_ref_id": source_ref_id,
            "text": text,
            "segment_ref": segment_ref,
            "material_refs": material_refs
        }));
    }

    Ok(BuiltIndexedEntities {
        items,
        index_to_ref,
    })
}

fn build_claims(
    source_ref_id: &str,
    parsed: &serde_json::Value,
    allowed_material_refs: &HashSet<String>,
    warnings: &mut Vec<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, PromptPackValidationError> {
    let candidates = candidate_array(parsed, "claim_candidates", "$.claim_candidates", false)?;
    let mut items = Vec::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let path = format!("$.claim_candidates[{index}]");
        let object = candidate_object(candidate, &path)?;
        reject_backend_owned_fields(object, &path)?;
        let material_refs = material_refs(
            object,
            &format!("{path}.material_refs"),
            allowed_material_refs,
        )?;
        let Some(text) = semantic_text(object, "claim", index, warnings)? else {
            continue;
        };
        items.push(serde_json::json!({
            "claim_id": format!("{source_ref_id}_claim_{}", items.len() + 1),
            "source_ref_id": source_ref_id,
            "text": text,
            "material_refs": material_refs
        }));
    }

    Ok(items)
}

fn build_evidence(
    source_ref_id: &str,
    parsed: &serde_json::Value,
    allowed_material_refs: &HashSet<String>,
    quote_refs: &HashMap<usize, String>,
    warnings: &mut Vec<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, PromptPackValidationError> {
    let candidates = candidate_array(
        parsed,
        "evidence_fragment_candidates",
        "$.evidence_fragment_candidates",
        false,
    )?;
    let mut items = Vec::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let path = format!("$.evidence_fragment_candidates[{index}]");
        let object = candidate_object(candidate, &path)?;
        reject_backend_owned_fields(object, &path)?;
        let material_refs = material_refs(
            object,
            &format!("{path}.material_refs"),
            allowed_material_refs,
        )?;
        let Some(text) = semantic_text(object, "evidence", index, warnings)? else {
            continue;
        };
        let quote_ref = optional_index_ref(
            object,
            "quote_candidate_index",
            &format!("{path}.quote_candidate_index"),
            quote_refs,
            "quote_candidate_index",
            "quote candidate",
        )?;
        items.push(serde_json::json!({
            "evidence_id": format!("{source_ref_id}_evidence_{}", items.len() + 1),
            "source_ref_id": source_ref_id,
            "text": text,
            "quote_ref": quote_ref,
            "material_refs": material_refs
        }));
    }

    Ok(items)
}

fn candidate_array<'a>(
    parent: &'a serde_json::Value,
    key: &str,
    path: &str,
    missing_is_empty: bool,
) -> Result<Vec<&'a serde_json::Value>, PromptPackValidationError> {
    match parent.get(key) {
        Some(value) => value
            .as_array()
            .map(|items| items.iter().collect())
            .ok_or_else(|| validation_error(format!("{key} must be an array"), path)),
        None if missing_is_empty => Ok(Vec::new()),
        None => Err(validation_error(
            format!("missing required key {key}"),
            path,
        )),
    }
}

fn candidate_object<'a>(
    candidate: &'a serde_json::Value,
    path: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, PromptPackValidationError> {
    candidate
        .as_object()
        .ok_or_else(|| validation_error("candidate item must be an object", path))
}

fn reject_backend_owned_fields(
    object: &serde_json::Map<String, serde_json::Value>,
    path: &str,
) -> Result<(), PromptPackValidationError> {
    for (key, value) in object {
        let child_path = format!("{path}.{key}");
        if matches!(
            key.as_str(),
            "segment_ref"
                | "key_point_ref"
                | "quote_ref"
                | "claim_id"
                | "evidence_id"
                | "source_ref_id"
        ) || key.ends_with("_id")
        {
            return Err(validation_error(
                format!("{key} is backend-owned and must not be supplied by provider output"),
                child_path,
            ));
        }
        if let Some(nested) = value.as_object() {
            reject_backend_owned_fields(nested, &child_path)?;
        }
        if let Some(items) = value.as_array() {
            for (index, item) in items.iter().enumerate() {
                if let Some(nested) = item.as_object() {
                    reject_backend_owned_fields(nested, &format!("{child_path}[{index}]"))?;
                }
            }
        }
    }
    Ok(())
}

fn material_refs(
    object: &serde_json::Map<String, serde_json::Value>,
    path: &str,
    allowed_material_refs: &HashSet<String>,
) -> Result<Vec<String>, PromptPackValidationError> {
    let Some(value) = object.get("material_refs") else {
        return Ok(Vec::new());
    };
    let refs = value
        .as_array()
        .ok_or_else(|| validation_error("material_refs must be an array", path))?;
    refs.iter()
        .enumerate()
        .map(|(index, item)| {
            let item_path = format!("{path}[{index}]");
            let material_ref = item.as_str().ok_or_else(|| {
                validation_error("material_refs item must be a string", &item_path)
            })?;
            if !allowed_material_refs.contains(material_ref) {
                return Err(validation_error(
                    format!("unknown material_ref {material_ref}"),
                    item_path,
                ));
            }
            Ok(material_ref.to_string())
        })
        .collect()
}

fn optional_string_field(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    path: &str,
) -> Result<Option<String>, PromptPackValidationError> {
    match object.get(key) {
        Some(value) if value.is_null() => Ok(None),
        Some(value) => value
            .as_str()
            .map(|text| {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .ok_or_else(|| validation_error(format!("{key} must be a string or null"), path)),
        None => Ok(None),
    }
}

fn semantic_text(
    object: &serde_json::Map<String, serde_json::Value>,
    entity_kind: &str,
    candidate_index: usize,
    warnings: &mut Vec<serde_json::Value>,
) -> Result<Option<String>, PromptPackValidationError> {
    match object.get("text") {
        Some(value) => {
            let text = value.as_str().ok_or_else(|| {
                validation_error(
                    "text must be a string",
                    format!("$.{entity_kind}_candidates[{candidate_index}].text"),
                )
            })?;
            let trimmed = text.trim();
            if trimmed.is_empty() {
                warnings.push(skip_warning(entity_kind, candidate_index));
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        None => {
            warnings.push(skip_warning(entity_kind, candidate_index));
            Ok(None)
        }
    }
}

fn skip_warning(entity_kind: &str, candidate_index: usize) -> serde_json::Value {
    serde_json::json!({
        "code": format!("blank_{entity_kind}_candidate"),
        "candidate_index": candidate_index,
        "message": format!("{entity_kind} candidate skipped because text is blank")
    })
}

fn optional_index_ref(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    path: &str,
    index_to_ref: &HashMap<usize, String>,
    label: &str,
    target_label: &str,
) -> Result<Option<String>, PromptPackValidationError> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let Some(index) = value.as_u64() else {
        return Err(validation_error(
            format!("{label} must be a non-negative integer"),
            path,
        ));
    };
    let index = usize::try_from(index)
        .map_err(|_| validation_error(format!("{label} is too large"), path))?;
    index_to_ref.get(&index).cloned().map(Some).ok_or_else(|| {
        validation_error(
            format!("unknown {label} {index}; {label} {index} points to skipped {target_label}"),
            path,
        )
    })
}

fn allowed_refs(
    source_ref_id: &str,
    segments: &[serde_json::Value],
    key_points: &[serde_json::Value],
    quotes: &[serde_json::Value],
    claims: &[serde_json::Value],
    evidence: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({
        "source_refs": [source_ref_id],
        "segment_refs": collect_string_field(segments, "segment_ref"),
        "key_point_refs": collect_string_field(key_points, "key_point_ref"),
        "quote_refs": collect_string_field(quotes, "quote_ref"),
        "claim_refs": collect_string_field(claims, "claim_id"),
        "evidence_refs": collect_string_field(evidence, "evidence_id")
    })
}

fn collect_string_field(items: &[serde_json::Value], key: &str) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| item.get(key).and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect()
}

fn string_set(value: Option<&serde_json::Value>) -> HashSet<String> {
    value
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn empty_run_graph(run_id: i64) -> serde_json::Value {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "graph_kind": YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND,
        "run_id": run_id,
        "sources": [],
        "segments": [],
        "key_points": [],
        "quotes": [],
        "claims": [],
        "evidence": [],
        "allowed_refs": {
            "source_refs": [],
            "segment_refs": [],
            "key_point_refs": [],
            "quote_refs": [],
            "claim_refs": [],
            "evidence_refs": []
        }
    })
}

fn merge_source_graph(
    merged: &mut serde_json::Value,
    source_graph: &serde_json::Value,
) -> AppResult<()> {
    for key in [
        "sources",
        "segments",
        "key_points",
        "quotes",
        "claims",
        "evidence",
    ] {
        append_array_values(merged, source_graph, key)?;
    }

    for bucket in [
        "source_refs",
        "segment_refs",
        "key_point_refs",
        "quote_refs",
        "claim_refs",
        "evidence_refs",
    ] {
        append_allowed_ref_bucket(merged, source_graph, bucket)?;
    }

    Ok(())
}

fn append_array_values(
    merged: &mut serde_json::Value,
    source_graph: &serde_json::Value,
    key: &str,
) -> AppResult<()> {
    let source_items = source_graph
        .get(key)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| AppError::internal(format!("intermediate graph {key} must be an array")))?;
    let merged_items = merged
        .get_mut(key)
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| AppError::internal(format!("merged intermediate graph {key} missing")))?;
    merged_items.extend(source_items.iter().cloned());
    Ok(())
}

fn append_allowed_ref_bucket(
    merged: &mut serde_json::Value,
    source_graph: &serde_json::Value,
    bucket: &str,
) -> AppResult<()> {
    let source_refs = source_graph
        .get("allowed_refs")
        .and_then(|allowed_refs| allowed_refs.get(bucket))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            AppError::internal(format!(
                "intermediate graph allowed_refs.{bucket} must be an array"
            ))
        })?;
    let merged_refs = merged
        .get_mut("allowed_refs")
        .and_then(|allowed_refs| allowed_refs.get_mut(bucket))
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| {
            AppError::internal(format!(
                "merged intermediate graph allowed_refs.{bucket} missing"
            ))
        })?;

    let mut seen = merged_refs
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect::<HashSet<_>>();
    for value in source_refs {
        let Some(reference) = value.as_str() else {
            return Err(AppError::internal(format!(
                "intermediate graph allowed_refs.{bucket} item must be a string"
            )));
        };
        if !seen.insert(reference.to_string()) {
            return Err(AppError::internal(format!(
                "duplicate ref {reference} in allowed_refs.{bucket}"
            )));
        }
        merged_refs.push(serde_json::json!(reference));
    }

    Ok(())
}

fn validate_graph_kind_for_pack(pack_id: &str) -> Result<(), PromptPackValidationError> {
    if pack_id == "youtube_summary" {
        Ok(())
    } else {
        Err(validation_error(
            format!("unsupported intermediate graph pack_id {pack_id}"),
            "$.pack_id",
        ))
    }
}

fn validation_error(
    message: impl Into<String>,
    object_path: impl Into<String>,
) -> PromptPackValidationError {
    PromptPackValidationError {
        message: message.into(),
        object_path: Some(object_path.into()),
    }
}
