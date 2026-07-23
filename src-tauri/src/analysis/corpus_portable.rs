use std::future::Future;
use std::pin::Pin;

use extractum_core::error::{AppError, AppResult};

use super::models::AnalysisSourceKind;

pub type AnalysisPortFuture<'a, T> = Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

pub trait AnalysisCorpusReader: Send + Sync + 'static {
    fn load_corpus(
        &self,
        request: AnalysisCorpusRequest,
    ) -> AnalysisPortFuture<'_, Vec<AnalysisCorpusMessage>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisCorpusRequest {
    source_kind: AnalysisSourceKind,
    source_ids: Vec<i64>,
    period_from: i64,
    period_to: i64,
    youtube_corpus_mode: YoutubeCorpusMode,
    include_migrated_history: bool,
}

impl AnalysisCorpusRequest {
    pub fn new(
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
        period_from: i64,
        period_to: i64,
        youtube_corpus_mode: YoutubeCorpusMode,
        include_migrated_history: bool,
    ) -> AppResult<Self> {
        if period_from > period_to {
            return Err(AppError::validation(
                "period_from must be less than or equal to period_to",
            ));
        }
        if source_ids.is_empty() || source_ids.iter().any(|source_id| *source_id <= 0) {
            return Err(AppError::validation(
                "Analysis corpus must contain positive source IDs",
            ));
        }
        let mut seen = std::collections::HashSet::new();
        let source_ids = source_ids
            .into_iter()
            .filter(|source_id| seen.insert(*source_id))
            .collect();
        Ok(Self {
            source_kind,
            source_ids,
            period_from,
            period_to,
            youtube_corpus_mode,
            include_migrated_history,
        })
    }

    pub fn source_kind(&self) -> AnalysisSourceKind {
        self.source_kind
    }
    pub fn source_ids(&self) -> &[i64] {
        &self.source_ids
    }
    pub fn period_from(&self) -> i64 {
        self.period_from
    }
    pub fn period_to(&self) -> i64 {
        self.period_to
    }
    pub fn youtube_corpus_mode(&self) -> YoutubeCorpusMode {
        self.youtube_corpus_mode
    }
    pub fn include_migrated_history(&self) -> bool {
        self.include_migrated_history
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisCorpusMessage {
    item_id: i64,
    source_id: i64,
    external_id: String,
    published_at: i64,
    author: Option<String>,
    content: String,
    r#ref: String,
    item_kind: Option<String>,
    source_type: Option<String>,
    source_subtype: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
}

impl AnalysisCorpusMessage {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        item_id: i64,
        source_id: i64,
        external_id: String,
        published_at: i64,
        author: Option<String>,
        content: String,
        r#ref: String,
        item_kind: Option<String>,
        source_type: Option<String>,
        source_subtype: Option<String>,
        metadata_zstd: Option<Vec<u8>>,
    ) -> Self {
        Self {
            item_id,
            source_id,
            external_id,
            published_at,
            author,
            content,
            r#ref,
            item_kind,
            source_type,
            source_subtype,
            metadata_zstd,
        }
    }

    pub fn item_id(&self) -> i64 {
        self.item_id
    }
    pub fn source_id(&self) -> i64 {
        self.source_id
    }
    pub fn external_id(&self) -> &str {
        &self.external_id
    }
    pub fn published_at(&self) -> i64 {
        self.published_at
    }
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn reference(&self) -> &str {
        &self.r#ref
    }
    pub fn item_kind(&self) -> Option<&str> {
        self.item_kind.as_deref()
    }
    pub fn source_type(&self) -> Option<&str> {
        self.source_type.as_deref()
    }
    pub fn source_subtype(&self) -> Option<&str> {
        self.source_subtype.as_deref()
    }
    pub fn metadata_zstd(&self) -> Option<&[u8]> {
        self.metadata_zstd.as_deref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum YoutubeCorpusMode {
    TranscriptOnly,
    TranscriptDescription,
    TranscriptDescriptionComments,
}

impl YoutubeCorpusMode {
    pub fn from_wire(value: Option<&str>) -> Result<Self, String> {
        match value.unwrap_or("transcript_description") {
            "transcript_only" => Ok(Self::TranscriptOnly),
            "transcript_description" => Ok(Self::TranscriptDescription),
            "transcript_description_comments" => Ok(Self::TranscriptDescriptionComments),
            other => Err(format!("Unsupported youtube_corpus_mode '{other}'")),
        }
    }
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::TranscriptOnly => "transcript_only",
            Self::TranscriptDescription => "transcript_description",
            Self::TranscriptDescriptionComments => "transcript_description_comments",
        }
    }
    pub fn includes_description(self) -> bool {
        matches!(
            self,
            Self::TranscriptDescription | Self::TranscriptDescriptionComments
        )
    }
    pub fn includes_comments(self) -> bool {
        matches!(self, Self::TranscriptDescriptionComments)
    }
}

impl std::str::FromStr for YoutubeCorpusMode {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_wire(Some(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisRunPreflightLimits {
    max_messages_per_run: usize,
    max_chunks_per_run: usize,
    max_estimated_input_chars_per_run: usize,
    max_background_requests_per_run: usize,
}

impl Default for AnalysisRunPreflightLimits {
    fn default() -> Self {
        Self {
            max_messages_per_run: 10_000,
            max_chunks_per_run: 80,
            max_estimated_input_chars_per_run: 1_500_000,
            max_background_requests_per_run: 80,
        }
    }
}

impl AnalysisRunPreflightLimits {
    pub fn max_messages_per_run(&self) -> usize {
        self.max_messages_per_run
    }
    pub fn max_chunks_per_run(&self) -> usize {
        self.max_chunks_per_run
    }
    pub fn max_estimated_input_chars_per_run(&self) -> usize {
        self.max_estimated_input_chars_per_run
    }
    pub fn max_background_requests_per_run(&self) -> usize {
        self.max_background_requests_per_run
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisRunPreflight {
    source_ids: Vec<i64>,
    message_count: usize,
    estimated_input_chars: usize,
    estimated_chunks: usize,
    limits: AnalysisRunPreflightLimits,
}

impl AnalysisRunPreflight {
    pub(crate) fn from_observation(
        source_ids: Vec<i64>,
        message_count: usize,
        estimated_input_chars: usize,
        estimated_chunks: usize,
        limits: AnalysisRunPreflightLimits,
    ) -> Self {
        Self {
            source_ids,
            message_count,
            estimated_input_chars,
            estimated_chunks,
            limits,
        }
    }
    pub fn source_ids(&self) -> &[i64] {
        &self.source_ids
    }
    pub fn message_count(&self) -> usize {
        self.message_count
    }
    pub fn estimated_input_chars(&self) -> usize {
        self.estimated_input_chars
    }
    pub fn estimated_chunks(&self) -> usize {
        self.estimated_chunks
    }
    pub fn limits(&self) -> &AnalysisRunPreflightLimits {
        &self.limits
    }
}

pub(crate) fn estimate_message_input_chars(
    content: &str,
    r#ref: &str,
    author: Option<&str>,
) -> usize {
    content.len() + r#ref.len() + author.unwrap_or("").len() + 64
}

pub(crate) fn estimate_preflight_chunk_count(message_sizes: &[usize], max_chars: usize) -> usize {
    let mut chunks = 0;
    let mut current_chars = 0;
    for size in message_sizes {
        if current_chars > 0 && current_chars + size > max_chars {
            chunks += 1;
            current_chars = 0;
        }
        current_chars += size;
    }
    if current_chars > 0 {
        chunks += 1;
    }
    chunks
}

pub async fn preflight_analysis_corpus(
    reader: &dyn AnalysisCorpusReader,
    request: &AnalysisCorpusRequest,
    chunk_target_chars: usize,
    limits: AnalysisRunPreflightLimits,
) -> AppResult<AnalysisRunPreflight> {
    let corpus = reader.load_corpus(request.clone()).await?;
    let message_sizes: Vec<_> = corpus
        .iter()
        .map(|message| {
            estimate_message_input_chars(message.content(), message.reference(), message.author())
        })
        .collect();
    let estimated_input_chars = message_sizes.iter().sum();
    let estimated_chunks = estimate_preflight_chunk_count(&message_sizes, chunk_target_chars);
    Ok(AnalysisRunPreflight::from_observation(
        request.source_ids().to_vec(),
        message_sizes.len(),
        estimated_input_chars,
        estimated_chunks,
        limits,
    ))
}

pub(crate) fn preflight_limit_error(preflight: &AnalysisRunPreflight) -> Option<String> {
    let limits = preflight.limits();
    if preflight.message_count() <= limits.max_messages_per_run()
        && preflight.estimated_chunks() <= limits.max_chunks_per_run()
        && preflight.estimated_input_chars() <= limits.max_estimated_input_chars_per_run()
    {
        return None;
    }
    Some(format!("Analysis scope is too large: {} documents, {} estimated chunks, {} estimated input characters. Narrow the period or choose a smaller source scope.", preflight.message_count(), preflight.estimated_chunks(), preflight.estimated_input_chars()))
}

#[allow(dead_code)]
pub(crate) fn model_limit_preflight_error(
    preflight: &AnalysisRunPreflight,
    model_input_limit: Option<usize>,
) -> Option<String> {
    let model_input_limit = model_input_limit.filter(|limit| *limit > 0)?;
    if preflight.estimated_chunks() == 0 {
        return None;
    }
    let estimated_chunk_chars = preflight
        .estimated_input_chars()
        .div_ceil(preflight.estimated_chunks());
    if estimated_chunk_chars <= model_input_limit {
        return None;
    }
    Some(format!("Analysis scope is too large for the selected model: {estimated_chunk_chars} estimated input characters per chunk exceeds model input limit {model_input_limit}. Choose a model with a larger context window, narrow the period, or choose a smaller source scope."))
}
