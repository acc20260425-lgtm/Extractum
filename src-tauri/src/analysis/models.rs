use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use extractum_core::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisScopeKind {
    SingleSource,
    SourceGroup,
    Project,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisSourceKind {
    Telegram,
    Youtube,
}

pub struct ResolvedAnalysisScope {
    scope_kind: AnalysisScopeKind,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
    source_kind: AnalysisSourceKind,
    source_ids: Vec<i64>,
    scope_label_snapshot: String,
}

impl ResolvedAnalysisScope {
    fn new(
        scope_kind: AnalysisScopeKind,
        identity: i64,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
        scope_label_snapshot: String,
    ) -> AppResult<Self> {
        if identity <= 0 {
            return Err(AppError::validation(
                "Analysis scope identity must be positive",
            ));
        }
        if source_ids.is_empty() || source_ids.iter().any(|source_id| *source_id <= 0) {
            return Err(AppError::validation(
                "Analysis scope must contain positive source IDs",
            ));
        }
        let mut seen = std::collections::HashSet::new();
        let source_ids = source_ids
            .into_iter()
            .filter(|source_id| seen.insert(*source_id))
            .collect();
        let scope_label_snapshot = scope_label_snapshot.trim().to_string();
        if scope_label_snapshot.is_empty() {
            return Err(AppError::validation("Analysis scope label cannot be empty"));
        }
        let (source_id, source_group_id, project_id) = match scope_kind {
            AnalysisScopeKind::SingleSource => (Some(identity), None, None),
            AnalysisScopeKind::SourceGroup => (None, Some(identity), None),
            AnalysisScopeKind::Project => (None, None, Some(identity)),
        };
        Ok(Self {
            scope_kind,
            source_id,
            source_group_id,
            project_id,
            source_kind,
            source_ids,
            scope_label_snapshot,
        })
    }

    pub fn for_source(
        source_id: i64,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
        scope_label_snapshot: String,
    ) -> AppResult<Self> {
        Self::new(
            AnalysisScopeKind::SingleSource,
            source_id,
            source_kind,
            source_ids,
            scope_label_snapshot,
        )
    }

    pub fn for_source_group(
        source_group_id: i64,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
        scope_label_snapshot: String,
    ) -> AppResult<Self> {
        Self::new(
            AnalysisScopeKind::SourceGroup,
            source_group_id,
            source_kind,
            source_ids,
            scope_label_snapshot,
        )
    }

    pub fn for_project(
        project_id: i64,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
        scope_label_snapshot: String,
    ) -> AppResult<Self> {
        Self::new(
            AnalysisScopeKind::Project,
            project_id,
            source_kind,
            source_ids,
            scope_label_snapshot,
        )
    }

    pub fn scope_kind(&self) -> AnalysisScopeKind {
        self.scope_kind
    }

    pub fn source_id(&self) -> Option<i64> {
        self.source_id
    }

    pub fn source_group_id(&self) -> Option<i64> {
        self.source_group_id
    }

    pub fn project_id(&self) -> Option<i64> {
        self.project_id
    }

    pub fn source_kind(&self) -> AnalysisSourceKind {
        self.source_kind
    }

    pub fn source_ids(&self) -> &[i64] {
        &self.source_ids
    }

    pub fn scope_label_snapshot(&self) -> &str {
        &self.scope_label_snapshot
    }
}

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
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
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
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
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

pub struct AnalysisForeignLabelMatch {
    term: String,
    source_ids: Vec<i64>,
    project_ids: Vec<i64>,
}

impl AnalysisForeignLabelMatch {
    pub fn new(term: String, source_ids: Vec<i64>, project_ids: Vec<i64>) -> AppResult<Self> {
        let term = term.trim().to_string();
        if term.is_empty() {
            return Err(AppError::validation(
                "Foreign label search term cannot be empty",
            ));
        }
        if source_ids.iter().any(|id| *id <= 0) || project_ids.iter().any(|id| *id <= 0) {
            return Err(AppError::validation("Foreign label IDs must be positive"));
        }
        Ok(Self {
            term,
            source_ids: stable_ids(source_ids),
            project_ids: stable_ids(project_ids),
        })
    }

    pub(crate) fn term(&self) -> &str {
        &self.term
    }
    pub(crate) fn source_ids(&self) -> &[i64] {
        &self.source_ids
    }
    pub(crate) fn project_ids(&self) -> &[i64] {
        &self.project_ids
    }
}

fn stable_ids(ids: Vec<i64>) -> Vec<i64> {
    let mut seen = std::collections::HashSet::new();
    ids.into_iter().filter(|id| seen.insert(*id)).collect()
}

pub struct AnalysisSourceLabel {
    source_id: i64,
    title: Option<String>,
}

impl AnalysisSourceLabel {
    pub fn new(source_id: i64, title: Option<String>) -> AppResult<Self> {
        if source_id <= 0 {
            return Err(AppError::validation(
                "Analysis source label ID must be positive",
            ));
        }
        Ok(Self { source_id, title })
    }
    pub fn source_id(&self) -> i64 {
        self.source_id
    }
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

pub struct AnalysisProjectLabel {
    project_id: i64,
    name: Option<String>,
}

impl AnalysisProjectLabel {
    pub fn new(project_id: i64, name: Option<String>) -> AppResult<Self> {
        if project_id <= 0 {
            return Err(AppError::validation(
                "Analysis project label ID must be positive",
            ));
        }
        Ok(Self { project_id, name })
    }
    pub fn project_id(&self) -> i64 {
        self.project_id
    }
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

pub struct AnalysisForeignLabels {
    sources: Vec<AnalysisSourceLabel>,
    projects: Vec<AnalysisProjectLabel>,
}

impl AnalysisForeignLabels {
    pub fn new(
        sources: Vec<AnalysisSourceLabel>,
        projects: Vec<AnalysisProjectLabel>,
    ) -> AppResult<Self> {
        let mut source_ids = std::collections::HashSet::new();
        let mut project_ids = std::collections::HashSet::new();
        if sources
            .iter()
            .any(|label| !source_ids.insert(label.source_id()))
            || projects
                .iter()
                .any(|label| !project_ids.insert(label.project_id()))
        {
            return Err(AppError::validation("Foreign labels must have unique IDs"));
        }
        Ok(Self { sources, projects })
    }
    pub(crate) fn source_title(&self, id: i64) -> Option<String> {
        self.sources
            .iter()
            .find(|label| label.source_id == id)
            .and_then(|label| label.title.clone())
    }
    pub(crate) fn project_name(&self, id: i64) -> Option<String> {
        self.projects
            .iter()
            .find(|label| label.project_id == id)
            .and_then(|label| label.name.clone())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnalysisForeignLabelRef {
    Source(i64),
    Project(i64),
}

pub struct AnalysisChatRun {
    run: AnalysisRunDetail,
}

impl AnalysisChatRun {
    pub fn needs_legacy_foreign_label(&self) -> bool {
        self.run
            .scope_label_snapshot
            .as_deref()
            .is_none_or(|label| label.trim().is_empty())
            && (self.run.source_id.is_some() || self.run.project_id.is_some())
    }
    pub(crate) fn new(run: AnalysisRunDetail) -> Self {
        Self { run }
    }
    pub(crate) fn into_detail(self) -> AnalysisRunDetail {
        self.run
    }
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
