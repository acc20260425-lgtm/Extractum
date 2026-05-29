use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct AnalysisSourceOption {
    pub id: i64,
    pub account_id: Option<i64>,
    pub source_type: String,
    pub title: Option<String>,
    pub item_count: i64,
    pub last_synced_at: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisPromptTemplate {
    pub id: i64,
    pub name: String,
    pub template_kind: String,
    pub body: String,
    pub version: i64,
    pub is_builtin: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisSourceGroupMember {
    pub source_id: i64,
    pub source_title: Option<String>,
    pub item_count: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AnalysisSourceGroup {
    pub id: i64,
    pub name: String,
    pub source_type: String,
    pub members: Vec<AnalysisSourceGroupMember>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisTraceRef {
    pub r#ref: String,
    pub item_id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub published_at: i64,
    pub excerpt: String,
    pub youtube_url: Option<String>,
    pub youtube_timestamp_seconds: Option<i64>,
    pub youtube_display_label: Option<String>,
    pub is_synthetic: bool,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisTraceData {
    pub refs: Vec<AnalysisTraceRef>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisSnapshotState {
    Captured,
    MissingLegacy,
    CaptureFailed,
}

#[derive(Serialize)]
pub struct AnalysisRunSummary {
    pub id: i64,
    pub run_type: String,
    pub scope_type: String,
    pub source_id: Option<i64>,
    pub source_title: Option<String>,
    pub source_group_id: Option<i64>,
    pub source_group_name: Option<String>,
    pub scope_label: String,
    pub period_from: i64,
    pub period_to: i64,
    pub output_language: String,
    pub prompt_template_id: Option<i64>,
    pub prompt_template_name: Option<String>,
    pub prompt_template_version: i64,
    pub provider_profile: String,
    pub provider: String,
    pub model: String,
    pub youtube_corpus_mode: String,
    pub telegram_history_scope: String,
    pub status: String,
    pub error: Option<String>,
    pub has_trace_data: bool,
    pub snapshot_state: Option<AnalysisSnapshotState>,
    pub snapshot_captured_at: Option<String>,
    pub snapshot_error: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Serialize)]
pub struct AnalysisRunDetail {
    pub id: i64,
    pub run_type: String,
    pub scope_type: String,
    pub source_id: Option<i64>,
    pub source_title: Option<String>,
    pub source_group_id: Option<i64>,
    pub source_group_name: Option<String>,
    pub scope_label: String,
    pub period_from: i64,
    pub period_to: i64,
    pub output_language: String,
    pub prompt_template_id: Option<i64>,
    pub prompt_template_name: Option<String>,
    pub prompt_template_version: i64,
    pub provider_profile: String,
    pub provider: String,
    pub model: String,
    pub youtube_corpus_mode: String,
    pub telegram_history_scope: String,
    pub status: String,
    pub result_markdown: Option<String>,
    pub error: Option<String>,
    pub has_trace_data: bool,
    pub snapshot_state: Option<AnalysisSnapshotState>,
    pub snapshot_captured_at: Option<String>,
    pub snapshot_error: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    #[serde(skip_serializing)]
    pub(crate) scope_label_snapshot: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub(crate) snapshot_message_count: i64,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct AnalysisRunMessageCursor {
    pub published_at: i64,
    pub r#ref: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisRunMessage {
    pub item_id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub author: Option<String>,
    pub published_at: i64,
    pub r#ref: String,
    pub content: String,
    pub item_kind: Option<String>,
    pub source_type: Option<String>,
    pub source_subtype: Option<String>,
    pub metadata_json: Option<serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisRunMessagesPage {
    pub messages: Vec<AnalysisRunMessage>,
    pub next_cursor: Option<AnalysisRunMessageCursor>,
    pub has_more: bool,
}

#[derive(FromRow)]
pub(crate) struct AnalysisRunRow {
    pub(crate) id: i64,
    pub(crate) run_type: String,
    pub(crate) scope_type: String,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_title: Option<String>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) source_group_name: Option<String>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: String,
    pub(crate) prompt_template_id: Option<i64>,
    pub(crate) prompt_template_name: Option<String>,
    pub(crate) prompt_template_version: i64,
    pub(crate) provider_profile: String,
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) youtube_corpus_mode: String,
    pub(crate) telegram_history_scope: String,
    pub(crate) status: String,
    pub(crate) result_markdown: Option<String>,
    pub(crate) trace_data_zstd: Option<Vec<u8>>,
    pub(crate) scope_label_snapshot: Option<String>,
    pub(crate) snapshot_captured_at: Option<String>,
    pub(crate) snapshot_error: Option<String>,
    pub(crate) snapshot_message_count: i64,
    pub(crate) error: Option<String>,
    pub(crate) created_at: i64,
    pub(crate) completed_at: Option<i64>,
}

#[derive(FromRow)]
pub(crate) struct AnalysisSourceGroupRow {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) source_type: String,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

#[derive(Serialize)]
pub struct AnalysisRunEvent {
    pub run_id: i64,
    pub request_id: Option<String>,
    pub kind: String,
    pub phase: String,
    pub queue_position: Option<usize>,
    pub message: Option<String>,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub delta: Option<String>,
    pub chunk_summary: Option<AnalysisChunkSummaryEvent>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct AnalysisChunkSummaryEvent {
    pub index: i64,
    pub total: i64,
    pub message_count: i64,
    pub summary: String,
    pub topics: Vec<String>,
    pub notable_points: Vec<String>,
    pub candidate_refs: Vec<String>,
}

#[derive(Serialize)]
pub struct AnalysisChatEvent {
    pub request_id: String,
    pub run_id: i64,
    pub kind: String,
    pub queue_position: Option<usize>,
    pub delta: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(FromRow)]
pub(crate) struct StoredRunSnapshotRow {
    pub(crate) item_id: i64,
    pub(crate) source_id: i64,
    pub(crate) external_id: String,
    pub(crate) author: Option<String>,
    pub(crate) published_at: i64,
    pub(crate) r#ref: String,
    pub(crate) content_zstd: Vec<u8>,
    pub(crate) item_kind: Option<String>,
    pub(crate) source_type: Option<String>,
    pub(crate) source_subtype: Option<String>,
    pub(crate) metadata_zstd: Option<Vec<u8>>,
}

#[derive(Clone)]
pub(crate) struct CorpusMessage {
    pub(crate) item_id: i64,
    pub(crate) source_id: i64,
    pub(crate) external_id: String,
    pub(crate) published_at: i64,
    pub(crate) author: Option<String>,
    pub(crate) content: String,
    pub(crate) r#ref: String,
    pub(crate) item_kind: Option<String>,
    pub(crate) source_type: Option<String>,
    pub(crate) source_subtype: Option<String>,
    pub(crate) metadata_zstd: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ChunkSummary {
    pub(crate) summary: String,
    pub(crate) topics: Vec<String>,
    pub(crate) notable_points: Vec<String>,
    pub(crate) candidate_refs: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisChatTurn {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisChatMessage {
    pub id: i64,
    pub run_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}
