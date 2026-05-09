#![allow(dead_code)]

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum YoutubeAvailabilityStatus {
    Available,
    Upcoming,
    LiveNow,
    LiveEndedTranscriptPending,
    NoCaptions,
    PrivateOrAuthRequired,
    MembersOnly,
    AgeRestricted,
    GeoBlocked,
    Deleted,
    RemovedFromPlaylist,
    UnavailableUnknown,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum YoutubePreviewKind {
    Video,
    Playlist,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubePreview {
    pub(crate) kind: YoutubePreviewKind,
    pub(crate) external_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) published_at: Option<String>,
    pub(crate) playlist_video_count: Option<i64>,
    pub(crate) captions_estimate: Option<YoutubeCaptionsEstimate>,
    pub(crate) availability_status: YoutubeAvailabilityStatus,
    pub(crate) warnings: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub(crate) struct YoutubeCaptionsEstimate {
    pub(crate) has_manual: bool,
    pub(crate) has_auto: bool,
    pub(crate) languages: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum YoutubeVideoForm {
    Regular,
    Short,
    Live,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeChapter {
    pub(crate) index: i64,
    pub(crate) title: String,
    pub(crate) start_ms: i64,
    pub(crate) end_ms: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeVideoMetadata {
    pub(crate) video_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) author_display: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) description: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) chapters: Vec<YoutubeChapter>,
    pub(crate) view_count: Option<i64>,
    pub(crate) like_count: Option<i64>,
    pub(crate) comment_count: Option<i64>,
    pub(crate) category: Option<String>,
    pub(crate) video_form: YoutubeVideoForm,
    pub(crate) availability_status: YoutubeAvailabilityStatus,
    pub(crate) raw_metadata_json: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubePlaylistMetadata {
    pub(crate) playlist_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) video_count: Option<i64>,
    pub(crate) items: Vec<YoutubePlaylistItemMetadata>,
    pub(crate) availability_status: YoutubeAvailabilityStatus,
    pub(crate) raw_metadata_json: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubePlaylistItemMetadata {
    pub(crate) video_id: String,
    pub(crate) position: Option<i64>,
    pub(crate) title_snapshot: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) availability_status: YoutubeAvailabilityStatus,
    pub(crate) raw_metadata_json: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum YoutubeCaptionTrackKind {
    Manual,
    Auto,
    Unknown,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeCaptionTrack {
    pub(crate) language: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) track_kind: YoutubeCaptionTrackKind,
    pub(crate) is_auto_generated: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeTranscriptSegment {
    pub(crate) index: i64,
    pub(crate) start_ms: i64,
    pub(crate) end_ms: Option<i64>,
    pub(crate) text: String,
    pub(crate) chapter_index: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeTranscript {
    pub(crate) video_id: String,
    pub(crate) language: Option<String>,
    pub(crate) track_kind: YoutubeCaptionTrackKind,
    pub(crate) is_auto_generated: bool,
    pub(crate) segments: Vec<YoutubeTranscriptSegment>,
    pub(crate) raw_payload: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeComment {
    pub(crate) comment_id: String,
    pub(crate) parent_comment_id: Option<String>,
    pub(crate) is_reply: bool,
    pub(crate) author: Option<String>,
    pub(crate) author_channel_id: Option<String>,
    pub(crate) author_channel_url: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) text: String,
    pub(crate) like_count: Option<i64>,
    pub(crate) is_pinned: Option<bool>,
    pub(crate) is_hearted: Option<bool>,
    pub(crate) raw_payload: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::{YoutubeAvailabilityStatus, YoutubePreviewKind, YoutubeVideoForm};

    #[test]
    fn availability_status_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&YoutubeAvailabilityStatus::LiveEndedTranscriptPending)
                .expect("serialize status"),
            "\"live_ended_transcript_pending\""
        );
    }

    #[test]
    fn preview_kind_deserializes_snake_case() {
        let kind: YoutubePreviewKind = serde_json::from_str("\"playlist\"").expect("deserialize");

        assert_eq!(kind, YoutubePreviewKind::Playlist);
    }

    #[test]
    fn video_form_serializes_short_value() {
        assert_eq!(
            serde_json::to_string(&YoutubeVideoForm::Short).expect("serialize form"),
            "\"short\""
        );
    }
}
