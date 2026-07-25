use super::super::super::corpus::{
    estimate_message_input_chars, estimate_preflight_chunk_count, model_limit_preflight_error,
    preflight_limit_error, AnalysisCorpusMessage, AnalysisRunPreflight, AnalysisRunPreflightLimits,
};

#[test]
fn estimated_message_chars_match_report_chunk_accounting() {
    let message = AnalysisCorpusMessage::new(
        11,
        2,
        "100".to_string(),
        1_710_000_000,
        Some("Alice".to_string()),
        "First live document".to_string(),
        "s2-i11".to_string(),
        Some("telegram_message".to_string()),
        Some("telegram".to_string()),
        Some("channel".to_string()),
        None,
    );

    assert_eq!(
        estimate_message_input_chars(message.content(), message.reference(), message.author()),
        message.content().len() + message.reference().len() + "Alice".len() + 64
    );
}

#[test]
fn estimated_chunk_count_matches_chunk_boundary_behavior() {
    assert_eq!(estimate_preflight_chunk_count(&[], 16_000), 0);
    assert_eq!(estimate_preflight_chunk_count(&[8_000, 7_000], 16_000), 1);
    assert_eq!(estimate_preflight_chunk_count(&[8_000, 9_000], 16_000), 2);
    assert_eq!(estimate_preflight_chunk_count(&[20_000], 16_000), 1);
}

#[test]
fn default_preflight_limits_are_conservative() {
    let limits = AnalysisRunPreflightLimits::default();

    assert_eq!(limits.max_messages_per_run(), 10_000);
    assert_eq!(limits.max_chunks_per_run(), 80);
    assert_eq!(limits.max_estimated_input_chars_per_run(), 1_500_000);
    assert_eq!(limits.max_background_requests_per_run(), 80);
}

#[test]
fn preflight_limit_error_reports_all_scale_dimensions() {
    let preflight = AnalysisRunPreflight::from_observation(
        vec![1],
        73_102,
        6_200_000,
        381,
        AnalysisRunPreflightLimits::default(),
    );

    let error = preflight_limit_error(&preflight).expect("limit error");

    assert!(error.contains("73102 documents"));
    assert!(error.contains("381 estimated chunks"));
    assert!(error.contains("6200000 estimated input characters"));
    assert!(error.contains("Narrow the period or choose a smaller source scope"));
}

#[test]
fn preflight_limit_error_allows_runs_within_limits() {
    let preflight = AnalysisRunPreflight::from_observation(
        vec![1],
        1_000,
        100_000,
        10,
        AnalysisRunPreflightLimits::default(),
    );

    assert_eq!(preflight_limit_error(&preflight), None);
}

#[test]
fn model_limit_preflight_allows_unknown_or_fitting_limits() {
    let preflight = AnalysisRunPreflight::from_observation(
        vec![1],
        1_000,
        120_000,
        3,
        AnalysisRunPreflightLimits::default(),
    );

    assert_eq!(model_limit_preflight_error(&preflight, None), None);
    assert_eq!(model_limit_preflight_error(&preflight, Some(40_000)), None);
}

#[test]
fn model_limit_preflight_reports_oversized_chunks() {
    let preflight = AnalysisRunPreflight::from_observation(
        vec![1],
        1_000,
        120_001,
        3,
        AnalysisRunPreflightLimits::default(),
    );

    let error = model_limit_preflight_error(&preflight, Some(40_000)).expect("model limit error");

    assert!(error.contains("40001 estimated input characters per chunk"));
    assert!(error.contains("model input limit 40000"));
    assert!(error.contains("Choose a model with a larger context window"));
}
