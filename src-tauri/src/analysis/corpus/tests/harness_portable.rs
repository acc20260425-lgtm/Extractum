use super::super::super::corpus::AnalysisCorpusMessage;

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
            None,
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
