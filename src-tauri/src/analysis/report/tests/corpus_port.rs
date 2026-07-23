use std::collections::VecDeque;
use std::sync::Mutex;

use extractum_core::error::{AppError, AppResult};

use super::super::super::corpus::{
    load_run_snapshot_messages, preflight_analysis_corpus, AnalysisCorpusMessage,
    AnalysisCorpusReader, AnalysisCorpusRequest, AnalysisPortFuture, AnalysisRunPreflight,
    AnalysisRunPreflightLimits, YoutubeCorpusMode,
};
use super::super::super::models::{AnalysisRunEvent, AnalysisSourceKind};
use super::super::super::trace::build_trace_data;
use super::super::requests::{
    build_map_request, build_reduce_request, chunk_messages, parse_chunk_summary,
    ReduceRequestParams,
};
use super::super::ReportRunError;
use super::harness::{report_capture_pool, sample_prompt_template};

enum CorpusReply {
    Messages(Vec<AnalysisCorpusMessage>),
    Error(String),
}

struct StatefulCorpusReader {
    replies: Mutex<VecDeque<CorpusReply>>,
    requests: Mutex<Vec<AnalysisCorpusRequest>>,
}

impl StatefulCorpusReader {
    fn new(replies: Vec<CorpusReply>) -> Self {
        Self {
            replies: Mutex::new(replies.into()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn call_count(&self) -> usize {
        self.requests.lock().expect("requests lock").len()
    }

    fn request_log(&self) -> Vec<AnalysisCorpusRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

impl AnalysisCorpusReader for StatefulCorpusReader {
    fn load_corpus(
        &self,
        request: AnalysisCorpusRequest,
    ) -> AnalysisPortFuture<'_, Vec<AnalysisCorpusMessage>> {
        self.requests.lock().expect("requests lock").push(request);
        Box::pin(async move {
            match self.replies.lock().expect("replies lock").pop_front() {
                Some(CorpusReply::Messages(messages)) => Ok(messages),
                Some(CorpusReply::Error(message)) => Err(AppError::internal(message)),
                None => panic!("unexpected corpus read"),
            }
        })
    }
}

fn request() -> AnalysisCorpusRequest {
    AnalysisCorpusRequest::new(
        AnalysisSourceKind::Telegram,
        vec![2],
        10,
        20,
        YoutubeCorpusMode::TranscriptDescription,
        false,
    )
    .expect("request")
}

fn message(label: &str, item_id: i64) -> AnalysisCorpusMessage {
    AnalysisCorpusMessage::new(
        item_id,
        2,
        item_id.to_string(),
        10 + item_id,
        None,
        label.to_string(),
        format!("s2-i{item_id}"),
        Some("telegram_message".to_string()),
        Some("telegram".to_string()),
        Some("channel".to_string()),
        None,
    )
}

fn expected_started_message(preflight: &AnalysisRunPreflight) -> String {
    format!(
        "Preflight passed: {} documents, {} estimated chunks, {} estimated input characters.",
        preflight.message_count(),
        preflight.estimated_chunks(),
        preflight.estimated_input_chars()
    )
}

fn assert_started_event(event: &AnalysisRunEvent, run_id: i64, expected_message: &str) {
    assert_eq!(event.run_id, run_id);
    assert_eq!(event.kind, "started");
    assert_eq!(event.phase, "load_items");
    assert_eq!(event.message.as_deref(), Some(expected_message));
    assert_eq!(event.error, None);
}

fn assert_terminal_capture_event(event: &AnalysisRunEvent, run_id: i64, expected_error: &str) {
    assert_eq!(event.run_id, run_id);
    assert_eq!(event.kind, "failed");
    assert_eq!(event.phase, "persist");
    assert_eq!(
        event.message.as_deref(),
        Some("Report run failed before snapshot capture completed.")
    );
    assert_eq!(event.error.as_deref(), Some(expected_error));
}

#[tokio::test]
async fn report_execution_uses_distinct_preflight_and_capture_corpus_reads() -> AppResult<()> {
    let run_id = 501;
    let pool = report_capture_pool(run_id).await;
    let reader = StatefulCorpusReader::new(vec![
        CorpusReply::Messages(vec![message("preflight A", 1)]),
        CorpusReply::Messages(vec![message("capture B", 2)]),
    ]);
    let request = request();
    let preflight = preflight_analysis_corpus(
        &reader,
        &request,
        16_000,
        AnalysisRunPreflightLimits::default(),
    )
    .await?;
    let expected_started = expected_started_message(&preflight);
    let mut events = Vec::new();
    let capture = super::super::execution_capture::capture_report_corpus(
        &pool,
        super::super::execution_capture::CaptureReportCorpusInput {
            run_id,
            scope_label: "Frozen B scope",
            reader: &reader,
            request: &request,
            preflight: &preflight,
            on_started: |message| {
                events.push(super::super::started_load_items_event(run_id, message).event)
            },
        },
    )
    .await
    .expect("capture corpus through production seam");

    assert_eq!(reader.request_log(), vec![request.clone(), request.clone()]);
    assert_eq!(reader.call_count(), 2);
    assert_eq!(preflight.message_count(), 1);
    assert_eq!(events.len(), 1);
    assert_started_event(&events[0], run_id, &expected_started);
    assert_eq!(capture.len(), 1);
    assert_eq!(capture[0].content(), "capture B");
    assert_eq!(capture[0].reference(), "s2-i2");

    let persisted = load_run_snapshot_messages(&pool, run_id).await?;
    assert_eq!(persisted, capture);
    let stored_label: Option<String> =
        sqlx::query_scalar("SELECT scope_label_snapshot FROM analysis_runs WHERE id = ?")
            .bind(run_id)
            .fetch_one(&pool)
            .await
            .map_err(AppError::database)?;
    assert_eq!(stored_label.as_deref(), Some("Frozen B scope"));

    let chunks = chunk_messages(&persisted, 16_000);
    assert_eq!(chunks.len(), 1);
    let map_request = build_map_request(run_id, "research".to_string(), 1, 1, &chunks[0]);
    let map_prompt = &map_request.messages[1].content;
    assert!(map_prompt.contains("capture B"));
    assert!(map_prompt.contains("s2-i2"));
    assert!(!map_prompt.contains("preflight A"));
    assert!(!map_prompt.contains("s2-i1"));

    let summary = parse_chunk_summary(
        r#"{"summary":"capture B summary","topics":["B"],"notable_points":["B point"],"candidate_refs":["s2-i2"]}"#,
    )
    .expect("parse fake map JSON");
    let summaries = vec![summary];
    let prompt_template = sample_prompt_template();
    let reduce_request = build_reduce_request(ReduceRequestParams {
        run_id,
        profile_id: "research".to_string(),
        scope_label: "Frozen B scope",
        output_language: "English",
        prompt_template: &prompt_template,
        period_from: 10,
        period_to: 20,
        chunk_summaries: &summaries,
        model_override: None,
    });
    let reduce_prompt = &reduce_request.messages[1].content;
    assert!(reduce_prompt.contains("capture B summary"));
    assert!(reduce_prompt.contains("s2-i2"));
    assert!(!reduce_prompt.contains("preflight A"));
    assert!(!reduce_prompt.contains("s2-i1"));

    let trace = build_trace_data("Final report grounded in [s2-i2].", &persisted);
    assert_eq!(trace.refs.len(), 1);
    assert_eq!(trace.refs[0].r#ref, "s2-i2");
    assert_eq!(trace.refs[0].excerpt, "capture B");
    assert!(!trace.refs[0].excerpt.contains("preflight A"));
    Ok(())
}

#[tokio::test]
async fn started_load_items_uses_preflight_summary_before_empty_capture_failure() -> AppResult<()> {
    let run_id = 502;
    let completed_at = 1_750_000_502;
    let pool = report_capture_pool(run_id).await;
    let reader = StatefulCorpusReader::new(vec![
        CorpusReply::Messages(vec![message("preflight A", 1), message("preflight A2", 2)]),
        CorpusReply::Messages(Vec::new()),
    ]);
    let request = request();
    let preflight = preflight_analysis_corpus(
        &reader,
        &request,
        16_000,
        AnalysisRunPreflightLimits::default(),
    )
    .await?;
    let expected_started = expected_started_message(&preflight);
    let mut events = Vec::new();
    let result = super::super::execution_capture::capture_report_corpus(
        &pool,
        super::super::execution_capture::CaptureReportCorpusInput {
            run_id,
            scope_label: "Frozen B scope",
            reader: &reader,
            request: &request,
            preflight: &preflight,
            on_started: |message| {
                events.push(super::super::started_load_items_event(run_id, message).event)
            },
        },
    )
    .await;
    let error = match result {
        Err(ReportRunError::CaptureFailed(error)) => error,
        other => panic!("expected production capture failure, got {other:?}"),
    };
    assert_eq!(error, "Snapshot capture failed");
    let terminal = super::super::lifecycle::persist_capture_failure_event(
        Some(&pool),
        run_id,
        &error,
        completed_at,
    )
    .await;
    events.push(terminal.event);

    assert_eq!(reader.request_log(), vec![request.clone(), request.clone()]);
    assert_eq!(reader.call_count(), 2);
    assert_eq!(events.len(), 2);
    assert_started_event(&events[0], run_id, &expected_started);
    assert_terminal_capture_event(&events[1], run_id, &error);
    let row: (String, Option<String>, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT status, error, snapshot_error, completed_at FROM analysis_runs WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .map_err(AppError::database)?;
    assert_eq!(
        row,
        (
            super::super::super::ANALYSIS_STATUS_FAILED.to_string(),
            Some(error.clone()),
            Some(error),
            Some(completed_at),
        )
    );
    Ok(())
}

#[tokio::test]
async fn started_load_items_uses_preflight_summary_before_error_capture_failure() -> AppResult<()> {
    let run_id = 503;
    let completed_at = 1_750_000_503;
    let pool = report_capture_pool(run_id).await;
    let reader = StatefulCorpusReader::new(vec![
        CorpusReply::Messages(vec![message("preflight A", 1)]),
        CorpusReply::Error("capture source unavailable".to_string()),
    ]);
    let request = request();
    let preflight = preflight_analysis_corpus(
        &reader,
        &request,
        16_000,
        AnalysisRunPreflightLimits::default(),
    )
    .await?;
    let expected_started = expected_started_message(&preflight);
    let mut events = Vec::new();
    let result = super::super::execution_capture::capture_report_corpus(
        &pool,
        super::super::execution_capture::CaptureReportCorpusInput {
            run_id,
            scope_label: "Frozen B scope",
            reader: &reader,
            request: &request,
            preflight: &preflight,
            on_started: |message| {
                events.push(super::super::started_load_items_event(run_id, message).event)
            },
        },
    )
    .await;
    let error = match result {
        Err(ReportRunError::CaptureFailed(error)) => error,
        other => panic!("expected production capture failure, got {other:?}"),
    };
    assert_eq!(error, "capture source unavailable");
    let terminal = super::super::lifecycle::persist_capture_failure_event(
        Some(&pool),
        run_id,
        &error,
        completed_at,
    )
    .await;
    events.push(terminal.event);

    assert_eq!(reader.request_log(), vec![request.clone(), request.clone()]);
    assert_eq!(reader.call_count(), 2);
    assert_eq!(events.len(), 2);
    assert_started_event(&events[0], run_id, &expected_started);
    assert_terminal_capture_event(&events[1], run_id, &error);
    let row: (String, Option<String>, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT status, error, snapshot_error, completed_at FROM analysis_runs WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .map_err(AppError::database)?;
    assert_eq!(
        row,
        (
            super::super::super::ANALYSIS_STATUS_FAILED.to_string(),
            Some(error.clone()),
            Some(error),
            Some(completed_at),
        )
    );
    Ok(())
}
