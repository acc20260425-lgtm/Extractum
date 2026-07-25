use super::super::super::corpus::AnalysisCorpusMessage;
use super::super::super::models::{AnalysisRunDetail, AnalysisSnapshotState};
use extractum_core::compression::compress_json_bytes;

pub(super) fn sample_corpus() -> Vec<AnalysisCorpusMessage> {
    vec![
        AnalysisCorpusMessage::new(
            11,
            2,
            "100".to_string(),
            1_710_000_000,
            Some("Alice".to_string()),
            "First frozen message".to_string(),
            "s2-i11".to_string(),
            Some("youtube_transcript".to_string()),
            Some("youtube".to_string()),
            Some("video".to_string()),
            Some(
                compress_json_bytes(br#"{"video_id":"video2","item_kind":"youtube_transcript"}"#)
                    .expect("compress metadata"),
            ),
        ),
        AnalysisCorpusMessage::new(
            12,
            4,
            "101".to_string(),
            1_710_000_100,
            None,
            "Second frozen message".to_string(),
            "s4-i12".to_string(),
            Some("telegram_message".to_string()),
            Some("telegram".to_string()),
            Some("channel".to_string()),
            None,
        ),
    ]
}

pub(super) fn sample_run() -> AnalysisRunDetail {
    AnalysisRunDetail {
        id: 1,
        run_type: "report".to_string(),
        scope_type: "source_group".to_string(),
        source_id: None,
        source_title: None,
        source_group_id: Some(9),
        source_group_name: Some("Live group".to_string()),
        project_id: None,
        project_name: None,
        scope_label: "Frozen group".to_string(),
        period_from: 1_700_000_000,
        period_to: 1_800_000_000,
        output_language: "English".to_string(),
        prompt_template_id: Some(1),
        prompt_template_name: Some("Default".to_string()),
        prompt_template_version: 1,
        provider_profile: "default".to_string(),
        provider: "gemini".to_string(),
        model: "gemini-2.5-flash".to_string(),
        youtube_corpus_mode: "transcript_description".to_string(),
        telegram_history_scope: "current".to_string(),
        status: "completed".to_string(),
        result_markdown: Some("Saved report".to_string()),
        error: None,
        has_trace_data: true,
        snapshot_state: Some(AnalysisSnapshotState::Captured),
        snapshot_captured_at: Some("2026-05-18T10:00:00Z".to_string()),
        snapshot_error: None,
        created_at: 1_710_000_500,
        completed_at: Some(1_710_000_600),
        scope_label_snapshot: Some("Frozen group".to_string()),
        snapshot_message_count: 1,
    }
}
