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
