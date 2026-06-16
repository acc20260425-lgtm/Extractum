use super::entities::load_merged_intermediate_entities_for_run;
use super::synthesis_input::build_synthesis_stage_input;
use super::test_support::*;

#[tokio::test]
async fn build_synthesis_stage_input_collects_successful_transcript_outputs() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("persist transcript fixtures");

    let input = build_synthesis_stage_input(&pool, 1)
        .await
        .expect("synthesis input");

    assert_eq!(input["stage"], "youtube_summary/synthesis");
    assert_eq!(input["videos"].as_array().expect("videos").len(), 2);
    assert_eq!(
        input["claim_candidates"].as_array().expect("claims").len(),
        2
    );
    assert_eq!(
        input["evidence_fragment_candidates"]
            .as_array()
            .expect("evidence")
            .len(),
        2
    );
}

#[tokio::test]
async fn build_synthesis_stage_input_uses_latest_parsed_output_wrappers() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "Old first summary",
                claim: "Old first claim",
                evidence: "Old first evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("persist transcript fixtures");

    let first_stage_run_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/transcript_analysis'
         ORDER BY id ASC
         LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("first stage row");
    crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
        &pool,
        1,
        first_stage_run_id,
        "parsed_output",
        2,
        3,
        &transcript_analysis_json("New first summary", "New first claim", "New first evidence"),
    )
    .await
    .expect("insert retry parsed output");

    let input = build_synthesis_stage_input(&pool, 1)
        .await
        .expect("synthesis input");
    let claims = input["claim_candidates"].as_array().expect("claims");

    assert_eq!(claims.len(), 2);
    assert_eq!(claims[0]["source_ref_id"], "source_ref_1");
    assert_eq!(claims[0]["candidate"]["text"], "New first claim");
    assert!(claims[0]["candidate"].get("source_ref_id").is_none());
}

#[tokio::test]
async fn build_synthesis_stage_input_merges_intermediate_graphs_and_allowed_refs() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("fixtures");

    let input = build_synthesis_stage_input(&pool, 1)
        .await
        .expect("synthesis input");

    assert_eq!(
        input["canonical_graph"]["sources"]
            .as_array()
            .expect("sources")
            .len(),
        2
    );
    assert_eq!(
        input["allowed_refs"]["claim_refs"]
            .as_array()
            .expect("claim refs")
            .len(),
        2
    );
    assert_eq!(
        input["allowed_refs"]["evidence_refs"]
            .as_array()
            .expect("evidence refs")
            .len(),
        2
    );
    assert_ne!(
        input["allowed_refs"]["claim_refs"][0], input["allowed_refs"]["claim_refs"][1],
        "source-qualified graph refs must not collide across source artifacts"
    );
    assert!(input["allowed_refs"]["claim_refs"]
        .as_array()
        .expect("claim refs")
        .iter()
        .all(|value| value.as_str().unwrap_or("").contains("_claim_")));
    assert!(input["allowed_refs"].get("segment_refs").is_some());
    assert!(input["allowed_refs"].get("key_point_refs").is_some());
    assert!(input["allowed_refs"].get("quote_refs").is_some());
    assert!(input["canonical_graph"].get("warnings").is_none());
}

#[tokio::test]
async fn build_synthesis_stage_input_orders_graph_by_source_snapshot_id() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("fixtures");

    let input = build_synthesis_stage_input(&pool, 1).await.expect("input");
    let sources = input["canonical_graph"]["sources"]
        .as_array()
        .expect("sources");

    let first_id = sources[0]["source_snapshot_id"].as_i64().expect("first id");
    let second_id = sources[1]["source_snapshot_id"]
        .as_i64()
        .expect("second id");
    assert!(first_id < second_id);
}

#[tokio::test]
async fn load_merged_intermediate_entities_rejects_duplicate_refs_across_sources() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("fixtures");

    overwrite_intermediate_entities_artifact_with_local_refs(&pool, 1, "source_ref_1", "claim_1")
        .await;
    overwrite_intermediate_entities_artifact_with_local_refs(&pool, 1, "source_ref_2", "claim_1")
        .await;

    let error = load_merged_intermediate_entities_for_run(&pool, 1)
        .await
        .expect_err("duplicate refs rejected");

    assert!(error.message.contains("duplicate ref claim_1"));
    assert!(error.message.contains("allowed_refs.claim_refs"));
}

async fn overwrite_intermediate_entities_artifact_with_local_refs(
    pool: &sqlx::SqlitePool,
    run_id: i64,
    source_ref_id: &str,
    claim_ref: &str,
) {
    let (stage_run_id, source_snapshot_id): (i64, i64) = sqlx::query_as(
        "SELECT stages.id, snapshots.id
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
    .expect("stage row");

    let graph = serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "graph_kind": "youtube_summary_intermediate_entities",
        "run_id": run_id,
        "attempt_number": 2,
        "sources": [{
            "source_ref_id": source_ref_id,
            "source_snapshot_id": source_snapshot_id,
            "title": null
        }],
        "segments": [],
        "key_points": [],
        "quotes": [],
        "claims": [{
            "claim_id": claim_ref,
            "text": "Claim",
            "material_refs": []
        }],
        "evidence": [{
            "evidence_id": "evidence_1",
            "text": "Evidence",
            "quote_ref": null,
            "material_refs": []
        }],
        "warnings": [],
        "allowed_refs": {
            "source_refs": [source_ref_id],
            "segment_refs": [],
            "key_point_refs": [],
            "quote_refs": [],
            "claim_refs": [claim_ref],
            "evidence_refs": ["evidence_1"]
        }
    });

    crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "intermediate_entities",
        2,
        5,
        &graph.to_string(),
    )
    .await
    .expect("insert local-ref graph");
}
