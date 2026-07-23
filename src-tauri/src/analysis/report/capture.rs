use sqlx::{Pool, Sqlite};

use super::super::corpus::{
    AnalysisCorpusMessage, AnalysisCorpusReader, AnalysisCorpusRequest, AppAnalysisCorpusReader,
};
use super::super::store::{capture_run_snapshot, sanitize_snapshot_error};
use super::ReportRunError;

const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";

pub(super) async fn capture_report_corpus(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    request: &AnalysisCorpusRequest,
) -> Result<Vec<AnalysisCorpusMessage>, ReportRunError> {
    let reader = AppAnalysisCorpusReader::new(pool.clone());
    let corpus = reader.load_corpus(request.clone()).await.map_err(|error| {
        ReportRunError::CaptureFailed(sanitize_snapshot_error(
            "Corpus preload failed",
            &error.to_string(),
        ))
    })?;

    if corpus.is_empty() {
        return Err(ReportRunError::CaptureFailed(
            SNAPSHOT_CAPTURE_FAILED_MESSAGE.to_string(),
        ));
    }

    capture_run_snapshot(pool, run_id, scope_label, &corpus)
        .await
        .map_err(|error| {
            ReportRunError::CaptureFailed(sanitize_snapshot_error(
                SNAPSHOT_CAPTURE_FAILED_MESSAGE,
                &error.to_string(),
            ))
        })
}
