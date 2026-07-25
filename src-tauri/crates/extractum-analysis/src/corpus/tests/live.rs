use super::super::super::corpus::YoutubeCorpusMode;

#[test]
fn youtube_corpus_mode_parses_wire_values_and_defaults() {
    assert_eq!(
        YoutubeCorpusMode::from_wire(None).expect("default mode"),
        YoutubeCorpusMode::TranscriptDescription
    );
    assert_eq!(
        YoutubeCorpusMode::from_wire(Some("transcript_only")).expect("transcript only"),
        YoutubeCorpusMode::TranscriptOnly
    );
    assert_eq!(
        YoutubeCorpusMode::from_wire(Some("transcript_description_comments"))
            .expect("comments mode"),
        YoutubeCorpusMode::TranscriptDescriptionComments
    );
    assert!(YoutubeCorpusMode::from_wire(Some("all_text")).is_err());
    assert_eq!(
        "transcript_only"
            .parse::<YoutubeCorpusMode>()
            .expect("parse transcript-only mode"),
        YoutubeCorpusMode::TranscriptOnly,
        "RED: CP2 YouTube corpus FromStr"
    );
    assert_eq!(
        YoutubeCorpusMode::TranscriptOnly.as_wire(),
        "transcript_only"
    );
    assert_eq!(
        YoutubeCorpusMode::TranscriptDescription.as_wire(),
        "transcript_description"
    );
    assert_eq!(
        YoutubeCorpusMode::TranscriptDescriptionComments.as_wire(),
        "transcript_description_comments"
    );
}
