use super::super::super::corpus::{
    AnalysisCorpusRequest, AnalysisRunPreflight, AnalysisRunPreflightLimits, YoutubeCorpusMode,
};
use super::super::super::models::{AnalysisScopeKind, AnalysisSourceKind, ResolvedAnalysisScope};
use super::super::{
    chunk_target_chars_for_model_input_limit, resolve_analysis_telegram_history_scope,
    ReportRunInput, StartAnalysisReportRequest,
};
use super::harness::{sample_prompt_template, sample_resolved_profile};

#[test]
fn report_run_input_carries_resolved_profile_snapshot() {
    let input = ReportRunInput {
        run_id: 9,
        scope: ResolvedAnalysisScope::for_source(
            2,
            AnalysisSourceKind::Telegram,
            vec![2],
            "Source".to_string(),
        )
        .expect("resolved scope"),
        corpus_request: AnalysisCorpusRequest::new(
            AnalysisSourceKind::Telegram,
            vec![2],
            10,
            20,
            YoutubeCorpusMode::TranscriptDescription,
            false,
        )
        .expect("corpus request"),
        period_from: 10,
        period_to: 20,
        output_language: "English".to_string(),
        prompt_template: sample_prompt_template(),
        model_override: Some("gemini-2.5-pro".to_string()),
        resolved_profile: sample_resolved_profile(),
        chunk_target_chars: 16_000,
        preflight: AnalysisRunPreflight::from_observation(
            vec![2],
            1,
            500,
            1,
            AnalysisRunPreflightLimits::default(),
        ),
    };

    assert_eq!(input.resolved_profile.profile_id(), "research");
    assert_eq!(input.resolved_profile.default_model(), "gemini-2.5-flash");
    assert_eq!(input.scope.scope_kind(), AnalysisScopeKind::SingleSource);
    assert_eq!(input.scope.scope_label_snapshot(), "Source");
}

#[test]
fn telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match() {
    let (scope, include_migrated_history) =
        resolve_analysis_telegram_history_scope(true, AnalysisSourceKind::Telegram)
            .expect("resolve Telegram opt-in");

    assert!(include_migrated_history);
    assert_eq!(
        scope,
        crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED
    );
}

#[test]
fn migrated_history_opt_in_rejects_non_telegram_analysis() {
    let error = resolve_analysis_telegram_history_scope(true, AnalysisSourceKind::Youtube)
        .expect_err("reject non-Telegram opt-in");
    assert_eq!(error.kind, extractum_core::error::AppErrorKind::Validation);
}

#[test]
fn report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape() {
    let request = StartAnalysisReportRequest::for_source(
        1,
        1,
        2,
        "Russian".to_string(),
        1,
        None,
        None,
        None,
        true,
    )
    .expect("construct source request");

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

#[test]
fn start_analysis_report_request_constructors_preserve_source_group_and_project_scopes() {
    let group = StartAnalysisReportRequest::for_source_group(
        42,
        10,
        20,
        " English ".to_string(),
        7,
        Some("model".to_string()),
        Some("profile".to_string()),
        Some("transcript_only".to_string()),
        true,
    )
    .expect("construct group request");
    let project = StartAnalysisReportRequest::for_project(
        73,
        30,
        40,
        "Russian".to_string(),
        8,
        None,
        None,
        None,
        false,
    )
    .expect("construct project request");

    assert_eq!(
        group.source_group_id,
        Some(42),
        "RED: CP2 report request constructors"
    );
    assert_eq!(group.source_id, None);
    assert_eq!(group.project_id, None);
    assert_eq!(group.output_language, "English");
    assert_eq!(group.model_override.as_deref(), Some("model"));
    assert_eq!(group.profile_id.as_deref(), Some("profile"));
    assert_eq!(
        group.youtube_corpus_mode.as_deref(),
        Some("transcript_only")
    );
    assert!(group.include_migrated_history);
    assert_eq!(project.project_id, Some(73));
    assert_eq!(project.source_id, None);
    assert_eq!(project.source_group_id, None);

    let zero = StartAnalysisReportRequest::from_command(
        Some(0),
        None,
        None,
        1,
        2,
        "English".to_string(),
        1,
        None,
        None,
        None,
        false,
    )
    .expect("contained zero survives early construction");
    assert_eq!(zero.source_id, Some(0));

    let missing = match StartAnalysisReportRequest::from_command(
        None,
        None,
        None,
        1,
        2,
        "English".to_string(),
        1,
        None,
        None,
        None,
        false,
    ) {
        Ok(_) => panic!("missing scope should be rejected"),
        Err(error) => error,
    };
    assert_eq!(
        missing.kind,
        extractum_core::error::AppErrorKind::Validation
    );

    let multiple = match StartAnalysisReportRequest::from_command(
        Some(1),
        Some(2),
        None,
        1,
        2,
        "English".to_string(),
        1,
        None,
        None,
        None,
        false,
    ) {
        Ok(_) => panic!("multiple scopes should be rejected"),
        Err(error) => error,
    };
    assert_eq!(
        multiple.kind,
        extractum_core::error::AppErrorKind::Validation
    );
}

#[test]
fn resolved_analysis_scope_rejects_zero_or_multiple_identities() {
    let zero = ResolvedAnalysisScope::for_source(
        0,
        AnalysisSourceKind::Telegram,
        vec![10],
        "Telegram source".to_string(),
    );
    assert!(zero.is_err(), "RED: CP2 scope identity cardinality");

    let source = ResolvedAnalysisScope::for_source(
        7,
        AnalysisSourceKind::Telegram,
        vec![7],
        "Source".to_string(),
    )
    .expect("source scope");
    let group = ResolvedAnalysisScope::for_source_group(
        8,
        AnalysisSourceKind::Telegram,
        vec![7],
        "Group".to_string(),
    )
    .expect("group scope");
    let project = ResolvedAnalysisScope::for_project(
        9,
        AnalysisSourceKind::Youtube,
        vec![20],
        "Project".to_string(),
    )
    .expect("project scope");

    assert_eq!(source.scope_kind(), AnalysisScopeKind::SingleSource);
    assert_eq!(
        (
            source.source_id(),
            source.source_group_id(),
            source.project_id()
        ),
        (Some(7), None, None)
    );
    assert_eq!(
        (
            group.source_id(),
            group.source_group_id(),
            group.project_id()
        ),
        (None, Some(8), None)
    );
    assert_eq!(
        (
            project.source_id(),
            project.source_group_id(),
            project.project_id()
        ),
        (None, None, Some(9))
    );
}

#[test]
fn resolved_analysis_scope_requires_nonempty_stable_sources_and_label() {
    let empty_sources = ResolvedAnalysisScope::for_source_group(
        8,
        AnalysisSourceKind::Telegram,
        Vec::new(),
        "Group".to_string(),
    );
    assert!(
        empty_sources.is_err(),
        "RED: CP2 stable source order and label"
    );

    assert!(ResolvedAnalysisScope::for_project(
        9,
        AnalysisSourceKind::Youtube,
        vec![20, 0],
        "Project".to_string(),
    )
    .is_err());
    assert!(ResolvedAnalysisScope::for_source(
        7,
        AnalysisSourceKind::Telegram,
        vec![7],
        "   ".to_string(),
    )
    .is_err());

    let scope = ResolvedAnalysisScope::for_project(
        9,
        AnalysisSourceKind::Youtube,
        vec![20, 10, 20],
        "  Research project  ".to_string(),
    )
    .expect("valid project scope");
    assert_eq!(scope.source_ids(), &[20, 10]);
    assert_eq!(scope.scope_label_snapshot(), "Research project");
    assert_eq!(scope.source_kind(), AnalysisSourceKind::Youtube);
}
