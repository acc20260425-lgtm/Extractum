use super::super::AnalysisRedesignFixtureSummary;
#[tokio::test]
async fn summary_serializes_with_camel_case_keys() {
    let summary = AnalysisRedesignFixtureSummary {
        accounts: 1,
        llm_profiles: 1,
        sources: 4,
        source_groups: 1,
        prompt_templates: 1,
        runs: 7,
        snapshot_messages: 4,
        chat_messages: 2,
        youtube_transcript_segments: 3,
        youtube_playlist_items: 2,
    };

    let value = serde_json::to_value(summary).expect("serialize summary");

    assert_eq!(value["llmProfiles"], 1);
    assert_eq!(value["sourceGroups"], 1);
    assert_eq!(value["promptTemplates"], 1);
    assert_eq!(value["snapshotMessages"], 4);
    assert_eq!(value["youtubeTranscriptSegments"], 3);
    assert_eq!(value["youtubePlaylistItems"], 2);
}
