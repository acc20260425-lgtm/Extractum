mod live;
mod snapshot;
mod source_resolution;
mod source_resolution_policy;

pub(crate) use super::corpus_portable::{
    estimate_message_input_chars, estimate_preflight_chunk_count, model_limit_preflight_error,
    preflight_analysis_corpus, preflight_limit_error, AnalysisCorpusMessage, AnalysisCorpusReader,
    AnalysisCorpusRequest, AnalysisPortFuture, AnalysisRunPreflight, AnalysisRunPreflightLimits,
    YoutubeCorpusMode,
};

#[allow(unused_imports)]
pub(crate) use self::live::live_corpus_ref;
pub(crate) use self::live::load_app_corpus_messages;
#[allow(unused_imports)]
pub(crate) use self::snapshot::load_run_corpus_messages;
pub(crate) use self::snapshot::{
    list_run_snapshot_messages_page, load_run_snapshot_messages, load_trace_resolution_messages,
    ListRunSnapshotMessagesRequest,
};
#[allow(unused_imports)]
pub(crate) use self::source_resolution::AppAnalysisScopeResolution;
pub(crate) use self::source_resolution::{
    resolve_analysis_sources, AnalysisSourceResolutionError, AnalysisSourceResolutionErrorCode,
};
pub(crate) use self::source_resolution_policy::resolve_analysis_telegram_history_scope;

use sqlx::SqlitePool;

pub(crate) struct AppAnalysisCorpusReader {
    pool: SqlitePool,
}

impl AppAnalysisCorpusReader {
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl AnalysisCorpusReader for AppAnalysisCorpusReader {
    fn load_corpus(
        &self,
        request: AnalysisCorpusRequest,
    ) -> AnalysisPortFuture<'_, Vec<AnalysisCorpusMessage>> {
        Box::pin(async move { load_app_corpus_messages(&self.pool, &request).await })
    }
}

#[cfg(test)]
mod tests;
