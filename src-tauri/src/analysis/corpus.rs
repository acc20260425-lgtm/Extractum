mod live;
mod snapshot;
mod source_resolution;

#[allow(unused_imports)]
pub(crate) use self::live::live_corpus_ref;
pub(crate) use self::live::{load_corpus_messages, push_analysis_document_kind_filter};
#[allow(unused_imports)]
pub(crate) use self::snapshot::load_run_corpus_messages;
pub(crate) use self::snapshot::{
    list_run_snapshot_messages_page, load_run_snapshot_messages, load_trace_resolution_messages,
    ListRunSnapshotMessagesRequest,
};
#[cfg(test)]
pub(crate) use self::source_resolution::resolve_run_source_ids;
#[allow(unused_imports)]
pub(crate) use self::source_resolution::ResolvedAnalysisSources;
pub(crate) use self::source_resolution::{
    resolve_analysis_sources, AnalysisSourceResolutionError, AnalysisSourceResolutionErrorCode,
};
use sqlx::{Pool, Sqlite};

use crate::error::AppResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisRunPreflightLimits {
    pub max_messages_per_run: usize,
    pub max_chunks_per_run: usize,
    pub max_estimated_input_chars_per_run: usize,
    /// Reserved for future retry-aware budgeting. Currently equals
    /// `max_chunks_per_run` because each chunk creates exactly one
    /// background request.
    pub max_background_requests_per_run: usize,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisRunPreflight {
    pub source_ids: Vec<i64>,
    pub message_count: usize,
    pub estimated_input_chars: usize,
    pub estimated_chunks: usize,
    pub limits: AnalysisRunPreflightLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum YoutubeCorpusMode {
    TranscriptOnly,
    TranscriptDescription,
    TranscriptDescriptionComments,
}

impl YoutubeCorpusMode {
    pub(crate) fn from_wire(value: Option<&str>) -> Result<Self, String> {
        match value.unwrap_or("transcript_description") {
            "transcript_only" => Ok(Self::TranscriptOnly),
            "transcript_description" => Ok(Self::TranscriptDescription),
            "transcript_description_comments" => Ok(Self::TranscriptDescriptionComments),
            other => Err(format!("Unsupported youtube_corpus_mode '{other}'")),
        }
    }

    pub(crate) fn as_wire(self) -> &'static str {
        match self {
            Self::TranscriptOnly => "transcript_only",
            Self::TranscriptDescription => "transcript_description",
            Self::TranscriptDescriptionComments => "transcript_description_comments",
        }
    }

    pub(crate) fn includes_description(self) -> bool {
        matches!(
            self,
            Self::TranscriptDescription | Self::TranscriptDescriptionComments
        )
    }

    pub(crate) fn includes_comments(self) -> bool {
        matches!(self, Self::TranscriptDescriptionComments)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CorpusLoadRequest {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) include_migrated_history: bool,
}

pub(crate) fn estimate_message_input_chars(
    content: &str,
    r#ref: &str,
    author: Option<&str>,
) -> usize {
    content.len() + r#ref.len() + author.unwrap_or("").len() + 64
}

pub(crate) fn estimate_preflight_chunk_count(message_sizes: &[usize], max_chars: usize) -> usize {
    let mut chunks = 0usize;
    let mut current_chars = 0usize;

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

pub(crate) async fn preflight_analysis_run(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
    chunk_target_chars: usize,
    limits: AnalysisRunPreflightLimits,
) -> AppResult<AnalysisRunPreflight> {
    if request.source_ids.is_empty() {
        return Ok(AnalysisRunPreflight {
            source_ids: Vec::new(),
            message_count: 0,
            estimated_input_chars: 0,
            estimated_chunks: 0,
            limits,
        });
    }

    let corpus = load_corpus_messages(pool, request).await?;

    let mut message_sizes = Vec::with_capacity(corpus.len());
    let mut estimated_input_chars = 0usize;
    for message in &corpus {
        let size = estimate_message_input_chars(
            &message.content,
            &message.r#ref,
            message.author.as_deref(),
        );
        estimated_input_chars += size;
        message_sizes.push(size);
    }

    let estimated_chunks = estimate_preflight_chunk_count(&message_sizes, chunk_target_chars);

    Ok(AnalysisRunPreflight {
        source_ids: request.source_ids.clone(),
        message_count: message_sizes.len(),
        estimated_input_chars,
        estimated_chunks,
        limits,
    })
}

pub(crate) fn preflight_limit_error(preflight: &AnalysisRunPreflight) -> Option<String> {
    let exceeds_messages = preflight.message_count > preflight.limits.max_messages_per_run;
    let exceeds_chunks = preflight.estimated_chunks > preflight.limits.max_chunks_per_run;
    let exceeds_chars =
        preflight.estimated_input_chars > preflight.limits.max_estimated_input_chars_per_run;

    if !(exceeds_messages || exceeds_chunks || exceeds_chars) {
        return None;
    }

    Some(format!(
        "Analysis scope is too large: {} documents, {} estimated chunks, \
         {} estimated input characters. \
         Narrow the period or choose a smaller source scope.",
        preflight.message_count, preflight.estimated_chunks, preflight.estimated_input_chars
    ))
}

// Kept separate until report preflight can load selected-model metadata.
#[allow(dead_code)]
pub(crate) fn model_limit_preflight_error(
    preflight: &AnalysisRunPreflight,
    model_input_limit: Option<usize>,
) -> Option<String> {
    let model_input_limit = model_input_limit.filter(|limit| *limit > 0)?;
    if preflight.estimated_chunks == 0 {
        return None;
    }

    let estimated_chunk_chars = preflight
        .estimated_input_chars
        .div_ceil(preflight.estimated_chunks);
    if estimated_chunk_chars <= model_input_limit {
        return None;
    }

    Some(format!(
        "Analysis scope is too large for the selected model: \
         {estimated_chunk_chars} estimated input characters per chunk exceeds \
         model input limit {model_input_limit}. \
         Choose a model with a larger context window, narrow the period, \
         or choose a smaller source scope."
    ))
}

#[cfg(test)]
mod tests;
