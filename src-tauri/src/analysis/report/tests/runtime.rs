use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use extractum_core::error::{AppError, AppResult};
use extractum_llm::{LlmProviderAccess, LlmSchedulerState, ProviderKind, ResolvedLlmProfile};

use super::super::super::corpus::{
    preflight_analysis_corpus, AnalysisCorpusMessage, AnalysisCorpusReader, AnalysisCorpusRequest,
    AnalysisPortFuture, AnalysisRunPreflightLimits, YoutubeCorpusMode,
};
use super::super::super::models::{
    AnalysisChatEvent, AnalysisEventSink, AnalysisRunEvent, AnalysisSourceKind,
    ResolvedAnalysisScope,
};
use super::super::super::AnalysisState;
use super::super::{
    execute_analysis_report, finalize_analysis_report_execution, AnalysisReportExecutionTicket,
    ReportRunInput,
};
use super::harness::{
    report_capture_pool, sample_prompt_template, start_openai_compat_completion_server, SAMPLE_JSON,
};

enum CorpusReply {
    Messages(Vec<AnalysisCorpusMessage>),
}

struct RuntimeCorpusReader {
    replies: Mutex<VecDeque<CorpusReply>>,
}

impl RuntimeCorpusReader {
    fn new(replies: Vec<CorpusReply>) -> Self {
        Self {
            replies: Mutex::new(replies.into()),
        }
    }
}

impl AnalysisCorpusReader for RuntimeCorpusReader {
    fn load_corpus(
        &self,
        _request: AnalysisCorpusRequest,
    ) -> AnalysisPortFuture<'_, Vec<AnalysisCorpusMessage>> {
        Box::pin(async move {
            match self.replies.lock().expect("replies lock").pop_front() {
                Some(CorpusReply::Messages(messages)) => Ok(messages),
                None => Err(AppError::internal("unexpected corpus read")),
            }
        })
    }
}

#[derive(Default)]
struct RecordingAnalysisEventSink {
    run_events: Mutex<Vec<AnalysisRunEvent>>,
}

impl RecordingAnalysisEventSink {
    fn take_run_events(&self) -> Vec<AnalysisRunEvent> {
        std::mem::take(&mut *self.run_events.lock().expect("run events lock"))
    }
}

impl AnalysisEventSink for RecordingAnalysisEventSink {
    fn publish_run(&self, event: AnalysisRunEvent) {
        self.run_events.lock().expect("run events lock").push(event);
    }

    fn publish_chat(&self, _event: AnalysisChatEvent) {}
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
    .expect("runtime corpus request")
}

fn message() -> AnalysisCorpusMessage {
    AnalysisCorpusMessage::new(
        1,
        2,
        "42".to_string(),
        1_700_000_000,
        Some("analyst".to_string()),
        "Preflight source document".to_string(),
        "s2-i1".to_string(),
        Some("telegram_message".to_string()),
        Some("telegram".to_string()),
        Some("channel".to_string()),
        None,
    )
}

#[tokio::test]
async fn report_execution_publishes_typed_events_in_existing_order() -> AppResult<()> {
    let run_id = 601;
    let pool = report_capture_pool(run_id).await;
    let state = AnalysisState::new();
    state.insert_active_report_run(run_id).await;
    let reader = RuntimeCorpusReader::new(vec![
        CorpusReply::Messages(vec![message()]),
        CorpusReply::Messages(vec![message()]),
    ]);
    let corpus_request = request();
    let preflight = preflight_analysis_corpus(
        &reader,
        &corpus_request,
        16_000,
        AnalysisRunPreflightLimits::default(),
    )
    .await?;
    let expected_started_message = format!(
        "Preflight passed: {} documents, {} estimated chunks, {} estimated input characters.",
        preflight.message_count(),
        preflight.estimated_chunks(),
        preflight.estimated_input_chars()
    );
    let (base_url, server) = start_openai_compat_completion_server(vec![
        SAMPLE_JSON.to_string(),
        "Final report [s2-i1].".to_string(),
    ]);
    let resolved_profile = ResolvedLlmProfile::new(
        "research".to_string(),
        "test-model".to_string(),
        LlmProviderAccess::new(
            ProviderKind::OpenAiCompatible,
            "secret-key".to_string().into(),
            base_url,
        ),
    );
    let ticket = AnalysisReportExecutionTicket {
        input: ReportRunInput {
            run_id,
            scope: ResolvedAnalysisScope::for_source(
                2,
                AnalysisSourceKind::Telegram,
                vec![2],
                "Runtime scope".to_string(),
            )?,
            corpus_request,
            period_from: 10,
            period_to: 20,
            output_language: "English".to_string(),
            prompt_template: sample_prompt_template(),
            model_override: None,
            resolved_profile,
            chunk_target_chars: 16_000,
            preflight,
        },
    };
    let sink = Arc::new(RecordingAnalysisEventSink::default());

    let outcome = execute_analysis_report(
        &pool,
        &state,
        Arc::new(LlmSchedulerState::new()),
        &reader,
        sink.clone(),
        ticket,
    )
    .await;
    finalize_analysis_report_execution(Some(&pool), &state, sink.as_ref(), run_id, outcome).await;

    let events = sink.take_run_events();
    let order = events
        .iter()
        .map(|event| (event.kind.as_str(), event.phase.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        order,
        vec![
            ("started", "load_items"),
            ("progress", "chunking"),
            ("progress", "map"),
            ("queued", "map"),
            ("started", "map"),
            ("progress", "map"),
            ("progress", "reduce"),
            ("queued", "reduce"),
            ("started", "reduce"),
            ("delta", "reduce"),
            ("progress", "persist"),
            ("completed", "persist"),
        ],
        "RED: CP4 report event order"
    );
    let normalized_events = events
        .iter()
        .map(|event| {
            let normalized_request_id = match event.request_id.as_deref() {
                None => serde_json::Value::Null,
                Some(request_id) if request_id.starts_with("analysis-map-601-1-") => {
                    serde_json::json!("<map>")
                }
                Some(request_id) if request_id.starts_with("analysis-reduce-601-") => {
                    serde_json::json!("<reduce>")
                }
                Some(request_id) => serde_json::json!(format!("<unexpected:{request_id}>")),
            };
            let mut value = serde_json::to_value(event).expect("serialize run event");
            value["request_id"] = normalized_request_id;
            value
        })
        .collect::<Vec<_>>();
    assert_eq!(
        normalized_events,
        vec![
            serde_json::json!({
                "run_id": 601,
                "request_id": null,
                "kind": "started",
                "phase": "load_items",
                "queue_position": null,
                "message": expected_started_message,
                "progress_current": null,
                "progress_total": null,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": null,
                "kind": "progress",
                "phase": "chunking",
                "queue_position": null,
                "message": "Loaded 1 source documents. Preparing chunks...",
                "progress_current": null,
                "progress_total": null,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": null,
                "kind": "progress",
                "phase": "map",
                "queue_position": null,
                "message": "Dispatching 1 chunk analysis request...",
                "progress_current": 0,
                "progress_total": 1,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": "<map>",
                "kind": "queued",
                "phase": "map",
                "queue_position": 1,
                "message": "Chunk 1 of 1 queued at position 1...",
                "progress_current": 0,
                "progress_total": 1,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": "<map>",
                "kind": "started",
                "phase": "map",
                "queue_position": null,
                "message": "Analyzing chunk 1 of 1...",
                "progress_current": 0,
                "progress_total": 1,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": "<map>",
                "kind": "progress",
                "phase": "map",
                "queue_position": null,
                "message": "Chunk 1 of 1 summarized.",
                "progress_current": 1,
                "progress_total": 1,
                "delta": null,
                "chunk_summary": {
                    "index": 1,
                    "total": 1,
                    "message_count": 1,
                    "summary": "Brief",
                    "topics": ["sync"],
                    "notable_points": ["Point"],
                    "candidate_refs": ["s1-i2"],
                },
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": null,
                "kind": "progress",
                "phase": "reduce",
                "queue_position": null,
                "message": "Writing final report...",
                "progress_current": null,
                "progress_total": null,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": "<reduce>",
                "kind": "queued",
                "phase": "reduce",
                "queue_position": 1,
                "message": "Final report queued at position 1...",
                "progress_current": null,
                "progress_total": null,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": "<reduce>",
                "kind": "started",
                "phase": "reduce",
                "queue_position": null,
                "message": "Writing final report...",
                "progress_current": null,
                "progress_total": null,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": "<reduce>",
                "kind": "delta",
                "phase": "reduce",
                "queue_position": null,
                "message": null,
                "progress_current": null,
                "progress_total": null,
                "delta": "Final report [s2-i1].",
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": "<reduce>",
                "kind": "progress",
                "phase": "persist",
                "queue_position": null,
                "message": "Saving report...",
                "progress_current": null,
                "progress_total": null,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
            serde_json::json!({
                "run_id": 601,
                "request_id": "<reduce>",
                "kind": "completed",
                "phase": "persist",
                "queue_position": null,
                "message": "Report completed with 1 cited references.",
                "progress_current": null,
                "progress_total": null,
                "delta": null,
                "chunk_summary": null,
                "error": null,
            }),
        ]
    );

    let map_request_id = events[3].request_id.as_deref().expect("map request ID");
    assert!(map_request_id.starts_with("analysis-map-601-1-"));
    for event in &events[3..=5] {
        assert_eq!(event.request_id.as_deref(), Some(map_request_id));
    }
    assert_eq!(events[7].queue_position, Some(1));
    let reduce_request_id = events[7].request_id.as_deref().expect("reduce request ID");
    assert!(reduce_request_id.starts_with("analysis-reduce-601-"));
    for event in &events[7..=11] {
        assert_eq!(event.request_id.as_deref(), Some(reduce_request_id));
    }
    server.join().expect("completion server");
    Ok(())
}

#[tokio::test]
async fn terminal_cleanup_always_removes_active_report_state() {
    let persistence_failure_run_id = 602;
    let pool = report_capture_pool(persistence_failure_run_id).await;
    sqlx::query("DROP TABLE analysis_runs")
        .execute(&pool)
        .await
        .expect("remove terminal persistence target");
    let state = AnalysisState::new();
    state
        .insert_active_report_run(persistence_failure_run_id)
        .await;
    let persistence_failure_sink = RecordingAnalysisEventSink::default();

    finalize_analysis_report_execution(
        Some(&pool),
        &state,
        &persistence_failure_sink,
        persistence_failure_run_id,
        Err(super::super::AnalysisExecutionError::Failed(
            "provider failed".to_string(),
        )),
    )
    .await;

    assert!(
        !state
            .active_report_run_ids()
            .await
            .contains(&persistence_failure_run_id),
        "RED: CP4 terminal cleanup"
    );
    assert_eq!(
        persistence_failure_sink
            .take_run_events()
            .iter()
            .map(|event| serde_json::to_value(event).expect("serialize failed event"))
            .collect::<Vec<_>>(),
        vec![serde_json::json!({
            "run_id": 602,
            "request_id": null,
            "kind": "failed",
            "phase": "persist",
            "queue_position": null,
            "message": "Report run failed.",
            "progress_current": null,
            "progress_total": null,
            "delta": null,
            "chunk_summary": null,
            "error": "provider failed",
        })]
    );

    let missing_pool_run_id = 603;
    state.insert_active_report_run(missing_pool_run_id).await;
    let missing_pool_sink = RecordingAnalysisEventSink::default();
    finalize_analysis_report_execution(
        None,
        &state,
        &missing_pool_sink,
        missing_pool_run_id,
        Err(super::super::AnalysisExecutionError::Failed(
            "terminal pool unavailable".to_string(),
        )),
    )
    .await;

    assert!(
        !state
            .active_report_run_ids()
            .await
            .contains(&missing_pool_run_id),
        "RED: CP4 terminal cleanup"
    );
    assert_eq!(
        missing_pool_sink
            .take_run_events()
            .iter()
            .map(|event| serde_json::to_value(event).expect("serialize failed event"))
            .collect::<Vec<_>>(),
        vec![serde_json::json!({
            "run_id": 603,
            "request_id": null,
            "kind": "failed",
            "phase": "persist",
            "queue_position": null,
            "message": "Report run failed.",
            "progress_current": null,
            "progress_total": null,
            "delta": null,
            "chunk_summary": null,
            "error": "terminal pool unavailable",
        })]
    );
}
