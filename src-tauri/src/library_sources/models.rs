use serde::Serialize;
use sqlx::FromRow;

use crate::youtube::jobs::SourceJobRecord;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryYoutubeSourceDetails {
    pub video_form: Option<String>,
    pub duration_seconds: Option<i64>,
    pub playlist_video_count: Option<i64>,
    pub channel_title: Option<String>,
    pub availability_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryTelegramSourceDetails {
    pub account_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibrarySourceRecord {
    pub source_id: i64,
    pub provider: String,
    pub source_subtype: Option<String>,
    pub account_id: Option<i64>,
    pub external_id: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub canonical_url: Option<String>,
    pub created_at: i64,
    pub last_synced_at: Option<i64>,
    pub item_count: i64,
    pub project_count: i64,
    pub youtube: Option<LibraryYoutubeSourceDetails>,
    pub telegram: Option<LibraryTelegramSourceDetails>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct LibraryCatalogResponse {
    pub sources: Vec<LibraryCatalogRecord>,
    pub filter_counts: Vec<LibraryCatalogFilterCount>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct LibraryCatalogRecord {
    pub source: LibrarySourceRecord,
    pub latest_job: Option<SourceJobRecord>,
    pub status: LibraryCatalogStatus,
    pub status_detail: Option<String>,
    pub capabilities: LibraryCatalogCapabilities,
    pub disabled_reasons: LibraryCatalogDisabledReasons,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryCatalogStatus {
    Active,
    Syncing,
    Error,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LibraryCatalogCapabilities {
    pub can_refresh_source: bool,
    pub can_delete: bool,
    pub can_edit: bool,
    pub can_connect_to_project: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LibraryCatalogDisabledReasons {
    pub refresh_source: Option<String>,
    pub delete: Option<String>,
    pub edit: Option<String>,
    pub connect_to_project: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LibraryCatalogFilterCount {
    pub provider: String,
    pub source_subtype: Option<String>,
    pub count: i64,
    pub disabled: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, FromRow)]
pub(crate) struct LibrarySourceRow {
    pub(crate) source_id: i64,
    pub(crate) provider: String,
    pub(crate) source_subtype: Option<String>,
    pub(crate) account_id: Option<i64>,
    pub(crate) external_id: Option<String>,
    pub(crate) source_title: Option<String>,
    pub(crate) created_at: i64,
    pub(crate) last_synced_at: Option<i64>,
    pub(crate) item_count: i64,
    pub(crate) project_count: i64,
    pub(crate) video_title: Option<String>,
    pub(crate) video_canonical_url: Option<String>,
    pub(crate) video_channel_title: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) video_form: Option<String>,
    pub(crate) video_availability_status: Option<String>,
    pub(crate) playlist_title: Option<String>,
    pub(crate) playlist_canonical_url: Option<String>,
    pub(crate) playlist_channel_title: Option<String>,
    pub(crate) playlist_video_count: Option<i64>,
    pub(crate) playlist_availability_status: Option<String>,
}
