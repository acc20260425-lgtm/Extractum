use serde::{Deserialize, Serialize};

use crate::media::ItemMediaMetadata;

pub(crate) const DEFAULT_MAX_WORDS_PER_FILE: usize = 300_000;
pub(crate) const DEFAULT_MAX_BYTES_PER_FILE: usize = 50_000_000;
pub(crate) const DEFAULT_MIN_MESSAGE_LENGTH: usize = 3;

#[derive(Deserialize)]
pub struct NotebookLmExportRequest {
    pub source_id: i64,
    pub output_dir: String,
    pub period_from: Option<i64>,
    pub period_to: Option<i64>,
    pub include_media_placeholders: bool,
    pub min_message_length: i64,
    pub max_words_per_file: i64,
    pub max_bytes_per_file: i64,
    pub overwrite_existing: bool,
}

#[derive(Clone)]
pub(crate) struct NotebookLmExportConfig {
    pub(crate) source_id: i64,
    pub(crate) output_dir: String,
    pub(crate) period_from: Option<i64>,
    pub(crate) period_to: Option<i64>,
    pub(crate) include_media_placeholders: bool,
    pub(crate) min_message_length: usize,
    pub(crate) max_words_per_file: usize,
    pub(crate) max_bytes_per_file: usize,
    pub(crate) overwrite_existing: bool,
}

#[derive(Serialize)]
pub struct NotebookLmExportResult {
    pub output_dir: String,
    pub files: Vec<NotebookLmExportFile>,
    pub glossary_file: Option<String>,
    pub exported_message_count: usize,
    pub skipped_message_count: usize,
    pub warning_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct NotebookLmExportFile {
    pub path: String,
    pub message_count: usize,
    pub byte_size: usize,
    pub approximate_word_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotebookLmExportSource {
    pub(crate) id: i64,
    pub(crate) source_type: String,
    pub(crate) telegram_source_kind: String,
    pub(crate) external_id: String,
    pub(crate) title: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NotebookLmExportMessage {
    pub(crate) item_id: i64,
    pub(crate) source_id: i64,
    pub(crate) external_id: String,
    pub(crate) author: Option<String>,
    pub(crate) published_at: i64,
    pub(crate) text: Option<String>,
    pub(crate) content_kind: String,
    pub(crate) has_media: bool,
    pub(crate) media_kind: Option<String>,
    pub(crate) media_metadata: ItemMediaMetadata,
    pub(crate) media_placeholders: Vec<String>,
    pub(crate) urls: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParticipantSummary {
    pub(crate) author: String,
    pub(crate) message_count: usize,
    pub(crate) first_seen: i64,
    pub(crate) last_seen: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderedMessageBlock {
    pub(crate) message: NotebookLmExportMessage,
    pub(crate) markdown: String,
    pub(crate) approximate_word_count: usize,
    pub(crate) byte_size: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ChunkFile {
    pub(crate) filename: String,
    pub(crate) title_period: String,
    pub(crate) period_start: i64,
    pub(crate) period_end: i64,
    pub(crate) part_number: usize,
    pub(crate) blocks: Vec<RenderedMessageBlock>,
}
