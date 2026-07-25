mod live;
mod source_resolution;

pub(crate) use extractum_analysis::{
    AnalysisCorpusMessage, AnalysisCorpusReader, AnalysisCorpusRequest, AnalysisPortFuture,
    YoutubeCorpusMode,
};

#[allow(unused_imports)]
pub(crate) use self::live::live_corpus_ref;
pub(crate) use self::live::load_app_corpus_messages;
#[allow(unused_imports)]
pub(crate) use self::source_resolution::AppAnalysisScopeResolution;
pub(crate) use self::source_resolution::{
    resolve_analysis_sources, AnalysisSourceResolutionError, AnalysisSourceResolutionErrorCode,
};
pub(crate) use extractum_analysis::resolve_analysis_telegram_history_scope;

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
