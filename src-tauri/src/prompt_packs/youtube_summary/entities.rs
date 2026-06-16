use std::collections::{HashMap, HashSet};

use crate::prompt_packs::stage_io::TranscriptAnalysisStageInput;
use crate::prompt_packs::validation::PromptPackValidationError;

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
            &segments.items,
            &key_points.items,
            &quotes.items,
            &claims,
            &evidence
        )
    }))
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
        )?;
        let key_point_ref = format!("{source_ref_id}_key_point_{}", items.len() + 1);

        index_to_ref.insert(index, key_point_ref.clone());
        items.push(serde_json::json!({
            "key_point_ref": key_point_ref,
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
        )?;
        let quote_ref = format!("{source_ref_id}_quote_{}", items.len() + 1);

        index_to_ref.insert(index, quote_ref.clone());
        items.push(serde_json::json!({
            "quote_ref": quote_ref,
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
        )?;
        items.push(serde_json::json!({
            "evidence_id": format!("{source_ref_id}_evidence_{}", items.len() + 1),
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
            format!("unknown {label} {index}; {label} {index} points to skipped quote candidate"),
            path,
        )
    })
}

fn allowed_refs(
    segments: &[serde_json::Value],
    key_points: &[serde_json::Value],
    quotes: &[serde_json::Value],
    claims: &[serde_json::Value],
    evidence: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({
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
