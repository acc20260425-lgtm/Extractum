use super::super::validate_report_preflight;
use crate::analysis::corpus::{AnalysisRunPreflight, AnalysisRunPreflightLimits};
use crate::error::AppErrorKind;

#[test]
fn validate_report_preflight_rejects_empty_corpus() {
    let error = validate_report_preflight(&AnalysisRunPreflight::from_observation(
        vec![1],
        0,
        0,
        0,
        AnalysisRunPreflightLimits::default(),
    ))
    .expect_err("empty corpus should fail");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(
        error.message,
        "No synced source documents were found for the selected analysis scope and period"
    );
}

#[test]
fn validate_report_preflight_rejects_oversized_runs() {
    let error = validate_report_preflight(&AnalysisRunPreflight::from_observation(
        vec![1],
        10_001,
        100_000,
        10,
        AnalysisRunPreflightLimits::default(),
    ))
    .expect_err("oversized corpus should fail");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert!(error.message.contains("Analysis scope is too large"));
}

#[test]
fn validate_report_preflight_allows_runs_within_limits() {
    validate_report_preflight(&AnalysisRunPreflight::from_observation(
        vec![1],
        100,
        50_000,
        4,
        AnalysisRunPreflightLimits::default(),
    ))
    .expect("preflight should pass");
}
