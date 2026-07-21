use super::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    build_source_intermediate_entities, INTERMEDIATE_ENTITIES_ARTIFACT_KIND,
    YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND,
};
use super::test_support::{
    test_pool_with_frozen_youtube_summary_run, transcript_analysis_stage_id,
};
use crate::stage_io::insert_stage_artifact_in_pool;
use crate::stage_io::{TranscriptAnalysisStageInput, TranscriptSegmentRegistryEntry};

fn input() -> TranscriptAnalysisStageInput {
    TranscriptAnalysisStageInput {
        stage_io_version: "1.0".to_string(),
        schema_version: "1.0".to_string(),
        stage: "youtube_summary/transcript_analysis".to_string(),
        pack_id: "youtube_summary".to_string(),
        pack_version: "1.0.0".to_string(),
        run_id: 42,
        source_ref_id: "source_ref_1".to_string(),
        allowed_source_ref_ids: vec!["source_ref_1".to_string()],
        allowed_material_refs: vec!["material_ref_1".to_string(), "material_ref_2".to_string()],
        transcript_segment_registry: vec![TranscriptSegmentRegistryEntry {
            material_ref_id: "material_ref_1".to_string(),
            text: "Transcript text".to_string(),
        }],
        comment_selection_policy: serde_json::json!({}),
        control_preset: "standard".to_string(),
        evidence_mode: "standard".to_string(),
        output_language: "en".to_string(),
    }
}

fn parsed_output() -> serde_json::Value {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/transcript_analysis",
        "video_candidate": {
            "summary_text": "Video summary",
            "segment_candidates": [
                { "title": "Intro", "summary_text": "Opening", "material_refs": ["material_ref_1"] }
            ],
            "key_point_candidates": [
                { "text": "Main point", "segment_candidate_index": 0, "material_refs": ["material_ref_1"] }
            ],
            "quote_candidates": [
                { "text": "quoted text", "segment_candidate_index": 0, "material_refs": ["material_ref_1"] }
            ],
            "action_item_candidates": [],
            "open_question_candidates": []
        },
        "claim_candidates": [
            { "text": "Claim text", "material_refs": ["material_ref_1"] }
        ],
        "evidence_fragment_candidates": [
            { "text": "Evidence text", "material_refs": ["material_ref_1"], "quote_candidate_index": 0 }
        ],
        "warning_candidates": []
    })
}

#[test]
fn graph_constants_match_contract() {
    assert_eq!(INTERMEDIATE_ENTITIES_ARTIFACT_KIND, "intermediate_entities");
    assert_eq!(
        YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND,
        "youtube_summary_intermediate_entities"
    );
}

#[test]
fn build_source_graph_assigns_backend_refs_and_allowed_refs() {
    let graph = build_source_intermediate_entities(
        &input(),
        1001,
        Some("Video title"),
        &parsed_output(),
        1,
    )
    .expect("graph");

    assert_eq!(graph["graph_kind"], YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND);
    assert_eq!(graph["sources"][0]["source_ref_id"], "source_ref_1");
    assert_eq!(
        graph["segments"][0]["segment_ref"],
        "source_ref_1_segment_1"
    );
    assert_eq!(graph["segments"][0]["source_ref_id"], "source_ref_1");
    assert_eq!(
        graph["key_points"][0]["key_point_ref"],
        "source_ref_1_key_point_1"
    );
    assert_eq!(graph["key_points"][0]["source_ref_id"], "source_ref_1");
    assert_eq!(graph["quotes"][0]["quote_ref"], "source_ref_1_quote_1");
    assert_eq!(graph["quotes"][0]["source_ref_id"], "source_ref_1");
    assert_eq!(graph["claims"][0]["claim_id"], "source_ref_1_claim_1");
    assert_eq!(graph["claims"][0]["source_ref_id"], "source_ref_1");
    assert_eq!(
        graph["evidence"][0]["evidence_id"],
        "source_ref_1_evidence_1"
    );
    assert_eq!(graph["evidence"][0]["source_ref_id"], "source_ref_1");
    assert_eq!(graph["evidence"][0]["quote_ref"], "source_ref_1_quote_1");
    assert_eq!(
        graph["allowed_refs"]["segment_refs"][0],
        "source_ref_1_segment_1"
    );
    assert_eq!(
        graph["allowed_refs"]["key_point_refs"][0],
        "source_ref_1_key_point_1"
    );
    assert_eq!(
        graph["allowed_refs"]["quote_refs"][0],
        "source_ref_1_quote_1"
    );
    assert_eq!(
        graph["allowed_refs"]["claim_refs"][0],
        "source_ref_1_claim_1"
    );
    assert_eq!(
        graph["allowed_refs"]["evidence_refs"][0],
        "source_ref_1_evidence_1"
    );
}

#[test]
fn textless_segment_is_kept_as_structural_navigation() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["segment_candidates"] = serde_json::json!([{ "title": null, "summary_text": null, "material_refs": ["material_ref_1"] }]);

    let graph = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect("graph");

    assert_eq!(graph["segments"].as_array().expect("segments").len(), 1);
    assert!(graph["segments"][0]["title"].is_null());
    assert!(graph["segments"][0]["summary_text"].is_null());
}

#[test]
fn blank_key_point_is_skipped_with_graph_warning() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["key_point_candidates"] =
        serde_json::json!([{ "text": "   ", "material_refs": ["material_ref_1"] }]);

    let graph = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect("graph");

    assert!(graph["key_points"]
        .as_array()
        .expect("key_points")
        .is_empty());
    assert_eq!(graph["warnings"][0]["code"], "blank_key_point_candidate");
}

#[test]
fn malformed_candidate_container_is_rejected() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["quote_candidates"] = serde_json::json!({ "not": "an array" });

    let error = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect_err("malformed container rejected");

    assert!(error.message.contains("quote_candidates must be an array"));
    assert_eq!(
        error.object_path.as_deref(),
        Some("$.video_candidate.quote_candidates")
    );
}

#[test]
fn invalid_material_ref_is_rejected() {
    let mut parsed = parsed_output();
    parsed["claim_candidates"][0]["material_refs"] = serde_json::json!(["live_library_ref"]);

    let error = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect_err("unknown material ref rejected");

    assert!(error
        .message
        .contains("unknown material_ref live_library_ref"));
}

#[test]
fn evidence_quote_candidate_index_to_missing_quote_is_dropped_with_warning() {
    let mut parsed = parsed_output();
    parsed["evidence_fragment_candidates"][0]["quote_candidate_index"] = serde_json::json!(999);

    let graph = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect("graph keeps evidence and drops bad optional quote link");

    assert!(graph["evidence"][0]["quote_ref"].is_null());
    assert_eq!(
        graph["warnings"][0]["code"],
        "dropped_invalid_quote_candidate_index"
    );
}

#[test]
fn provider_output_must_not_supply_backend_refs_or_ids() {
    for backend_key in [
        "segment_ref",
        "key_point_ref",
        "quote_ref",
        "claim_id",
        "evidence_id",
        "source_ref_id",
        "provider_id",
    ] {
        let mut parsed = parsed_output();
        parsed["claim_candidates"][0][backend_key] = serde_json::json!("provider-owned");

        let error =
            build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
                .expect_err("provider-owned backend ref rejected");

        let expected_path = format!("$.claim_candidates[0].{backend_key}");
        assert!(error
            .message
            .contains(&format!("{backend_key} is backend-owned")));
        assert_eq!(error.object_path.as_deref(), Some(expected_path.as_str()));
    }
}

#[test]
fn evidence_index_pointing_to_skipped_quote_candidate_is_dropped_with_warning() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["quote_candidates"][0]["text"] = serde_json::json!(" ");
    parsed["evidence_fragment_candidates"][0]["quote_candidate_index"] = serde_json::json!(0);

    let graph = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect("graph keeps evidence and drops skipped quote link");

    assert!(graph["evidence"][0]["quote_ref"].is_null());
    assert!(graph["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .any(
            |warning| warning["code"] == "dropped_invalid_quote_candidate_index"
                && warning["object_path"]
                    == "$.evidence_fragment_candidates[0].quote_candidate_index"
        ));
}

#[test]
fn key_point_index_pointing_to_skipped_segment_candidate_is_dropped_with_warning() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["segment_candidates"] = serde_json::json!([]);
    parsed["video_candidate"]["key_point_candidates"][0]["segment_candidate_index"] =
        serde_json::json!(0);

    let graph = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect("graph keeps key point and drops skipped segment link");

    assert!(graph["key_points"][0]["segment_ref"].is_null());
    assert!(graph["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .any(
            |warning| warning["code"] == "dropped_invalid_segment_candidate_index"
                && warning["object_path"]
                    == "$.video_candidate.key_point_candidates[0].segment_candidate_index"
        ));
}

#[tokio::test]
async fn graph_builder_uses_persisted_prompt_input_material_registry() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;
    let mut prompt_input = input();
    prompt_input.run_id = 1;
    prompt_input.allowed_material_refs = vec!["prompt_only_ref".to_string()];
    prompt_input.transcript_segment_registry = vec![TranscriptSegmentRegistryEntry {
        material_ref_id: "prompt_only_ref".to_string(),
        text: "Prompt-only transcript text".to_string(),
    }];
    let prompt_input_json =
        serde_json::to_string(&prompt_input).expect("serialize prompt input fixture");
    insert_stage_artifact_in_pool(&pool, 1, stage_id, "prompt_input", 1, 1, &prompt_input_json)
        .await
        .expect("insert prompt input");

    let mut parsed = parsed_output();
    parsed["video_candidate"]["segment_candidates"][0]["material_refs"] =
        serde_json::json!(["prompt_only_ref"]);
    parsed["video_candidate"]["key_point_candidates"][0]["material_refs"] =
        serde_json::json!(["prompt_only_ref"]);
    parsed["video_candidate"]["quote_candidates"][0]["material_refs"] =
        serde_json::json!(["prompt_only_ref"]);
    parsed["claim_candidates"][0]["material_refs"] = serde_json::json!(["prompt_only_ref"]);
    parsed["evidence_fragment_candidates"][0]["material_refs"] =
        serde_json::json!(["prompt_only_ref"]);

    let graph = build_or_quarantine_intermediate_entities_for_transcript_stage(
        &pool, 1, stage_id, &parsed, 1,
    )
    .await
    .expect("graph should use persisted prompt_input registry");

    assert_eq!(graph["claims"][0]["material_refs"][0], "prompt_only_ref");
}
