use sqlx::SqlitePool;

use super::super::corpus::{AnalysisCorpusMessage, AnalysisCorpusReader, AnalysisCorpusRequest};
use super::super::store::{capture_run_snapshot, sanitize_snapshot_error};
use super::AnalysisExecutionError;

const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";

pub async fn capture_analysis_corpus(
    pool: &SqlitePool,
    reader: &dyn AnalysisCorpusReader,
    run_id: i64,
    scope_label: &str,
    request: &AnalysisCorpusRequest,
) -> Result<Vec<AnalysisCorpusMessage>, AnalysisExecutionError> {
    let corpus = reader.load_corpus(request.clone()).await.map_err(|error| {
        AnalysisExecutionError::CaptureFailed(sanitize_snapshot_error(
            "Corpus preload failed",
            &error.to_string(),
        ))
    })?;

    if corpus.is_empty() {
        return Err(AnalysisExecutionError::CaptureFailed(
            SNAPSHOT_CAPTURE_FAILED_MESSAGE.to_string(),
        ));
    }

    capture_run_snapshot(pool, run_id, scope_label, &corpus)
        .await
        .map_err(|error| {
            AnalysisExecutionError::CaptureFailed(sanitize_snapshot_error(
                SNAPSHOT_CAPTURE_FAILED_MESSAGE,
                &error.to_string(),
            ))
        })
}
