use super::super::{
    chunk_target_chars_for_model_input_limit, resolve_analysis_telegram_history_scope,
    ReportRunInput, StartAnalysisReportRequest,
};
use super::harness::{sample_prompt_template, sample_resolved_profile};
use crate::analysis::corpus::{
    AnalysisRunPreflight, AnalysisRunPreflightLimits, CorpusLoadRequest, YoutubeCorpusMode,
};

#[test]
fn report_run_input_carries_resolved_profile_snapshot() {
    let input = ReportRunInput {
        run_id: 9,
        scope_label: "Source".to_string(),
        corpus_request: CorpusLoadRequest {
            source_type: crate::sources::TELEGRAM_SOURCE_TYPE.to_string(),
            source_ids: vec![2],
            period_from: 10,
            period_to: 20,
            youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
            include_migrated_history: false,
        },
        period_from: 10,
        period_to: 20,
        output_language: "English".to_string(),
        prompt_template: sample_prompt_template(),
        model_override: Some("gemini-2.5-pro".to_string()),
        resolved_profile: sample_resolved_profile(),
        chunk_target_chars: 16_000,
        preflight: AnalysisRunPreflight {
            source_ids: vec![2],
            message_count: 1,
            estimated_input_chars: 500,
            estimated_chunks: 1,
            limits: AnalysisRunPreflightLimits::default(),
        },
    };

    assert_eq!(input.resolved_profile.profile_id(), "research");
    assert_eq!(input.resolved_profile.default_model(), "gemini-2.5-flash");
}

#[test]
fn telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match() {
    let (scope, include_migrated_history) =
        resolve_analysis_telegram_history_scope(true, "telegram").expect("resolve Telegram opt-in");

    assert!(include_migrated_history);
    assert_eq!(
        scope,
        crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED
    );
}

#[test]
fn migrated_history_opt_in_rejects_non_telegram_analysis() {
    let error = resolve_analysis_telegram_history_scope(true, "youtube")
        .expect_err("reject non-Telegram opt-in");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
}

#[test]
fn report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape() {
    let request = StartAnalysisReportRequest {
        source_id: Some(1),
        source_group_id: None,
        project_id: None,
        period_from: 1,
        period_to: 2,
        output_language: "Russian".to_string(),
        prompt_template_id: 1,
        model_override: None,
        profile_id: None,
        youtube_corpus_mode: None,
        include_migrated_history: true,
    };

    assert!(request.include_migrated_history);
}

#[test]
fn chunk_target_chars_are_derived_from_model_input_limit_with_fallback() {
    assert_eq!(chunk_target_chars_for_model_input_limit(None), 16_000);
    assert_eq!(
        chunk_target_chars_for_model_input_limit(Some(8_192)),
        11_259
    );
    assert!(chunk_target_chars_for_model_input_limit(Some(32_768)) > 16_000);
}
