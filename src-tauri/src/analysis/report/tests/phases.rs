use super::super::{finish_map_phase, run_analysis_step_with_cancel, ReportRunError};
use super::harness::sample_chunk_summary;
use crate::llm::LlmRequestError;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn analysis_step_cancel_wrapper_allows_completed_future() {
    let result = run_analysis_step_with_cancel(None, async { Ok::<_, LlmRequestError>("done") })
        .await
        .expect("step result");

    assert_eq!(result, "done");
}

#[tokio::test]
async fn analysis_step_cancel_wrapper_interrupts_pending_future() {
    let token = CancellationToken::new();
    token.cancel();

    let result: Result<(), LlmRequestError> =
        run_analysis_step_with_cancel(Some(token), std::future::pending()).await;

    assert!(matches!(result, Err(LlmRequestError::Cancelled)));
}

#[test]
fn finish_map_phase_preserves_chunk_order_by_original_index() {
    let ordered = vec![
        Some(sample_chunk_summary("first")),
        Some(sample_chunk_summary("second")),
        Some(sample_chunk_summary("third")),
    ];

    let collected = finish_map_phase(ordered, None).expect("collect summaries");

    assert_eq!(collected[0].summary, "first");
    assert_eq!(collected[1].summary, "second");
    assert_eq!(collected[2].summary, "third");
}

#[test]
fn finish_map_phase_rejects_missing_chunk_before_reduce() {
    let ordered = vec![Some(sample_chunk_summary("first")), None];

    let error = finish_map_phase(ordered, None).expect_err("missing chunk should fail");

    assert_eq!(
        error,
        ReportRunError::Failed("Some chunk summaries were not collected".to_string())
    );
}

#[test]
fn finish_map_phase_propagates_map_error_without_starting_reduce() {
    let ordered = vec![Some(sample_chunk_summary("first"))];

    let error = finish_map_phase(
        ordered,
        Some(ReportRunError::Cancelled(
            "Analysis run cancelled.".to_string(),
        )),
    )
    .expect_err("map cancellation should stop reduce");

    assert_eq!(
        error,
        ReportRunError::Cancelled("Analysis run cancelled.".to_string())
    );
}
