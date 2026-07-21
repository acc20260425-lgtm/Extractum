#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackEvent {
    pub run_id: i64,
    pub request_id: String,
    pub kind: String,
    pub run_status: String,
    pub phase: String,
    pub stage_run_id: Option<i64>,
    pub stage_name: Option<String>,
    pub source_snapshot_id: Option<i64>,
    pub queue_position: Option<i64>,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

pub trait PromptPackEventSink: Send + Sync + 'static {
    fn emit(&self, event: PromptPackEvent);
}
