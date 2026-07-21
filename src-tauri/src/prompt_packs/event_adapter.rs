use tauri::{AppHandle, Emitter};

use extractum_prompt_packs::{PromptPackEvent, PromptPackEventSink};

pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromptPackRunEvent {
    pub(crate) run_id: i64,
    pub(crate) request_id: String,
    pub(crate) kind: String,
    pub(crate) run_status: String,
    pub(crate) phase: String,
    pub(crate) stage_run_id: Option<i64>,
    pub(crate) stage_name: Option<String>,
    pub(crate) source_snapshot_id: Option<i64>,
    pub(crate) queue_position: Option<i64>,
    pub(crate) progress_current: Option<i64>,
    pub(crate) progress_total: Option<i64>,
    pub(crate) message: Option<String>,
    pub(crate) error: Option<String>,
}

impl From<PromptPackEvent> for PromptPackRunEvent {
    fn from(event: PromptPackEvent) -> Self {
        Self {
            run_id: event.run_id,
            request_id: event.request_id,
            kind: event.kind,
            run_status: event.run_status,
            phase: event.phase,
            stage_run_id: event.stage_run_id,
            stage_name: event.stage_name,
            source_snapshot_id: event.source_snapshot_id,
            queue_position: event.queue_position,
            progress_current: event.progress_current,
            progress_total: event.progress_total,
            message: event.message,
            error: event.error,
        }
    }
}

#[derive(Clone)]
pub(crate) struct TauriPromptPackEventSink {
    handle: AppHandle,
}

impl TauriPromptPackEventSink {
    pub(crate) fn new(handle: AppHandle) -> Self {
        Self { handle }
    }
}

impl PromptPackEventSink for TauriPromptPackEventSink {
    fn emit(&self, event: PromptPackEvent) {
        let _ = self
            .handle
            .emit(PROMPT_PACK_RUN_EVENT, PromptPackRunEvent::from(event));
    }
}

#[cfg(test)]
mod tests {
    use super::{PromptPackRunEvent, PROMPT_PACK_RUN_EVENT};
    use extractum_prompt_packs::PromptPackEvent;

    #[test]
    fn typed_events_map_to_exact_legacy_ipc_payloads() {
        let events = vec![
            PromptPackEvent {
                run_id: 42,
                request_id: "run-42".to_string(),
                kind: "queued".to_string(),
                run_status: "queued".to_string(),
                phase: "snapshot".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: Some(2),
                progress_current: Some(0),
                progress_total: Some(4),
                message: Some("Queued".to_string()),
                error: None,
            },
            PromptPackEvent {
                run_id: 42,
                request_id: "run-42-started".to_string(),
                kind: "started".to_string(),
                run_status: "running".to_string(),
                phase: "execution".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: None,
                progress_current: Some(0),
                progress_total: Some(4),
                message: Some("Running".to_string()),
                error: None,
            },
            PromptPackEvent {
                run_id: 42,
                request_id: "run-42-stage-44-repair".to_string(),
                kind: "queued".to_string(),
                run_status: "running".to_string(),
                phase: "repair".to_string(),
                stage_run_id: Some(44),
                stage_name: Some("youtube_summary/transcript_analysis".to_string()),
                source_snapshot_id: Some(901),
                queue_position: Some(3),
                progress_current: None,
                progress_total: None,
                message: Some("JSON repair queued at position 3".to_string()),
                error: None,
            },
            PromptPackEvent {
                run_id: 42,
                request_id: "run-42-gem-comments".to_string(),
                kind: "started".to_string(),
                run_status: "running".to_string(),
                phase: "gem_analysis".to_string(),
                stage_run_id: Some(45),
                stage_name: Some("youtube_summary/gem_analysis/comments".to_string()),
                source_snapshot_id: Some(901),
                queue_position: None,
                progress_current: None,
                progress_total: None,
                message: Some("Gem analysis: analyzing comments".to_string()),
                error: None,
            },
            PromptPackEvent {
                run_id: 42,
                request_id: "run-42-terminal".to_string(),
                kind: "completed".to_string(),
                run_status: "complete".to_string(),
                phase: "terminal".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: None,
                progress_current: Some(4),
                progress_total: Some(4),
                message: Some("Completed".to_string()),
                error: None,
            },
            PromptPackEvent {
                run_id: 43,
                request_id: "run-43-terminal".to_string(),
                kind: "failed".to_string(),
                run_status: "failed".to_string(),
                phase: "terminal".to_string(),
                stage_run_id: Some(46),
                stage_name: Some("youtube_summary/synthesis".to_string()),
                source_snapshot_id: None,
                queue_position: None,
                progress_current: Some(3),
                progress_total: Some(4),
                message: Some("Provider request failed".to_string()),
                error: Some("Provider request failed".to_string()),
            },
            PromptPackEvent {
                run_id: 44,
                request_id: "cancel-44".to_string(),
                kind: "cancelled".to_string(),
                run_status: "cancelled".to_string(),
                phase: "terminal".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: None,
                progress_current: None,
                progress_total: None,
                message: Some("Cancelled".to_string()),
                error: None,
            },
        ];
        let mapped = events
            .into_iter()
            .map(PromptPackRunEvent::from)
            .collect::<Vec<_>>();

        assert_eq!(PROMPT_PACK_RUN_EVENT, "prompt-pack-run-event");
        assert_eq!(
            serde_json::to_value(mapped).expect("serialize mapped events"),
            serde_json::json!([
                {"runId":42,"requestId":"run-42","kind":"queued","runStatus":"queued","phase":"snapshot","stageRunId":null,"stageName":null,"sourceSnapshotId":null,"queuePosition":2,"progressCurrent":0,"progressTotal":4,"message":"Queued","error":null},
                {"runId":42,"requestId":"run-42-started","kind":"started","runStatus":"running","phase":"execution","stageRunId":null,"stageName":null,"sourceSnapshotId":null,"queuePosition":null,"progressCurrent":0,"progressTotal":4,"message":"Running","error":null},
                {"runId":42,"requestId":"run-42-stage-44-repair","kind":"queued","runStatus":"running","phase":"repair","stageRunId":44,"stageName":"youtube_summary/transcript_analysis","sourceSnapshotId":901,"queuePosition":3,"progressCurrent":null,"progressTotal":null,"message":"JSON repair queued at position 3","error":null},
                {"runId":42,"requestId":"run-42-gem-comments","kind":"started","runStatus":"running","phase":"gem_analysis","stageRunId":45,"stageName":"youtube_summary/gem_analysis/comments","sourceSnapshotId":901,"queuePosition":null,"progressCurrent":null,"progressTotal":null,"message":"Gem analysis: analyzing comments","error":null},
                {"runId":42,"requestId":"run-42-terminal","kind":"completed","runStatus":"complete","phase":"terminal","stageRunId":null,"stageName":null,"sourceSnapshotId":null,"queuePosition":null,"progressCurrent":4,"progressTotal":4,"message":"Completed","error":null},
                {"runId":43,"requestId":"run-43-terminal","kind":"failed","runStatus":"failed","phase":"terminal","stageRunId":46,"stageName":"youtube_summary/synthesis","sourceSnapshotId":null,"queuePosition":null,"progressCurrent":3,"progressTotal":4,"message":"Provider request failed","error":"Provider request failed"},
                {"runId":44,"requestId":"cancel-44","kind":"cancelled","runStatus":"cancelled","phase":"terminal","stageRunId":null,"stageName":null,"sourceSnapshotId":null,"queuePosition":null,"progressCurrent":null,"progressTotal":null,"message":"Cancelled","error":null}
            ])
        );
    }
}
