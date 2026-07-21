use std::{future::Future, pin::Pin};

use extractum_core::error::AppResult;

pub type PromptPackPortFuture<'a, T> = Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

pub trait PromptPackSourceReader: Send + Sync + 'static {
    fn load_source(
        &self,
        source_id: i64,
    ) -> PromptPackPortFuture<'_, Option<PromptPackSourceRecord>>;

    fn load_video(
        &self,
        request: YoutubeVideoReadRequest,
    ) -> PromptPackPortFuture<'_, Option<PromptPackYoutubeVideoRecord>>;

    fn load_playlist_items(
        &self,
        playlist_source_id: i64,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackPlaylistItemRecord>>;

    fn load_transcript_segments(
        &self,
        source_id: i64,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackTranscriptSegment>>;

    fn select_comment_candidates(
        &self,
        request: CommentCandidateReadRequest,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackCommentCandidate>>;

    fn load_comment_body(
        &self,
        request: CommentBodyReadRequest,
    ) -> PromptPackPortFuture<'_, String>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackSourceRecord {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
    title: Option<String>,
}

impl PromptPackSourceRecord {
    pub fn new(
        id: i64,
        source_type: String,
        source_subtype: Option<String>,
        title: Option<String>,
    ) -> Self {
        Self {
            id,
            source_type,
            source_subtype,
            title,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    pub(crate) fn source_type(&self) -> &str {
        &self.source_type
    }

    pub(crate) fn source_subtype(&self) -> Option<&str> {
        self.source_subtype.as_deref()
    }

    pub(crate) fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YoutubeVideoReadRequest {
    source_id: i64,
}

impl YoutubeVideoReadRequest {
    pub fn new(source_id: i64) -> Self {
        Self { source_id }
    }

    pub fn source_id(&self) -> i64 {
        self.source_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackYoutubeVideoRecord {
    source_id: i64,
    video_id: String,
    canonical_url: String,
    title: Option<String>,
    channel_title: Option<String>,
    published_at: Option<String>,
    description: Option<String>,
}

impl PromptPackYoutubeVideoRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_id: i64,
        video_id: String,
        canonical_url: String,
        title: Option<String>,
        channel_title: Option<String>,
        published_at: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            source_id,
            video_id,
            canonical_url,
            title,
            channel_title,
            published_at,
            description,
        }
    }

    pub(crate) fn source_id(&self) -> i64 {
        self.source_id
    }

    pub(crate) fn video_id(&self) -> &str {
        &self.video_id
    }

    pub(crate) fn canonical_url(&self) -> &str {
        &self.canonical_url
    }

    pub(crate) fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub(crate) fn channel_title(&self) -> Option<&str> {
        self.channel_title.as_deref()
    }

    pub(crate) fn published_at(&self) -> Option<&str> {
        self.published_at.as_deref()
    }

    pub(crate) fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackPlaylistItemRecord {
    video_source_id: Option<i64>,
    video_id: String,
    title: Option<String>,
}

impl PromptPackPlaylistItemRecord {
    pub fn new(video_source_id: Option<i64>, video_id: String, title: Option<String>) -> Self {
        Self {
            video_source_id,
            video_id,
            title,
        }
    }

    pub(crate) fn video_source_id(&self) -> Option<i64> {
        self.video_source_id
    }

    pub(crate) fn video_id(&self) -> &str {
        &self.video_id
    }

    pub(crate) fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackTranscriptSegment {
    start_ms: i64,
    end_ms: i64,
    text: String,
}

impl PromptPackTranscriptSegment {
    pub fn new(start_ms: i64, end_ms: i64, text: String) -> Self {
        Self {
            start_ms,
            end_ms,
            text,
        }
    }

    pub(crate) fn start_ms(&self) -> i64 {
        self.start_ms
    }

    pub(crate) fn end_ms(&self) -> i64 {
        self.end_ms
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommentCandidateReadRequest {
    source_id: i64,
    limit: i64,
}

impl CommentCandidateReadRequest {
    pub fn new(source_id: i64, limit: i64) -> Self {
        Self { source_id, limit }
    }

    pub fn source_id(&self) -> i64 {
        self.source_id
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackCommentCandidate {
    external_id: Option<String>,
    body: String,
}

impl PromptPackCommentCandidate {
    pub fn new(external_id: Option<String>, body: String) -> Self {
        Self { external_id, body }
    }

    pub(crate) fn external_id(&self) -> Option<&str> {
        self.external_id.as_deref()
    }

    pub(crate) fn body(&self) -> &str {
        &self.body
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommentBodyReadRequest {
    source_id: i64,
    external_id: Option<String>,
}

impl CommentBodyReadRequest {
    pub fn new(source_id: i64, external_id: Option<String>) -> Self {
        Self {
            source_id,
            external_id,
        }
    }

    pub fn source_id(&self) -> i64 {
        self.source_id
    }

    pub fn external_id(&self) -> Option<&str> {
        self.external_id.as_deref()
    }
}
