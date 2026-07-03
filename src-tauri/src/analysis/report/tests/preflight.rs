use super::super::validate_report_preflight;
use crate::analysis::corpus::{AnalysisRunPreflight, AnalysisRunPreflightLimits};
use crate::error::AppErrorKind;

#[test]
fn validate_report_preflight_rejects_empty_corpus() {
    let error = validate_report_preflight(&AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 0,
        estimated_input_chars: 0,
        estimated_chunks: 0,
        limits: AnalysisRunPreflightLimits::default(),
    })
    .expect_err("empty corpus should fail");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(
        error.message,
        "No synced source documents were found for the selected analysis scope and period"
    );
}

#[test]
fn validate_report_preflight_rejects_oversized_runs() {
    let error = validate_report_preflight(&AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 10_001,
        estimated_input_chars: 100_000,
        estimated_chunks: 10,
        limits: AnalysisRunPreflightLimits::default(),
    })
    .expect_err("oversized corpus should fail");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert!(error.message.contains("Analysis scope is too large"));
}

#[test]
fn validate_report_preflight_allows_runs_within_limits() {
    validate_report_preflight(&AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 100,
        estimated_input_chars: 50_000,
        estimated_chunks: 4,
        limits: AnalysisRunPreflightLimits::default(),
    })
    .expect("preflight should pass");
}
