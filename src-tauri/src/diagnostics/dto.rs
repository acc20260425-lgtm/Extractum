use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticSummary {
    pub app: DiagnosticAppInfo,
    pub database: DiagnosticDatabaseInfo,
    pub providers: DiagnosticProvidersInfo,
    pub runtimes: DiagnosticRuntimeInfo,
    pub telegram: DiagnosticTelegramInfo,
    pub sources: DiagnosticSourcesInfo,
    pub items: DiagnosticItemsInfo,
    pub analysis_runs: DiagnosticAnalysisRunsInfo,
    pub llm_requests: DiagnosticLlmRequestsInfo,
    pub youtube_jobs: DiagnosticYoutubeJobsInfo,
    pub ingest: DiagnosticIngestInfo,
    pub privacy: DiagnosticPrivacyInfo,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticAppInfo {
    pub app_name: String,
    pub app_version: String,
    pub build_mode: String,
    pub generated_at_unix: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticDatabaseInfo {
    pub sqlite_available: bool,
    pub migrations: DiagnosticMigrationInfo,
    pub account_count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticMigrationInfo {
    pub status: String,
    pub expected_versions: Vec<i64>,
    pub applied_versions: Vec<i64>,
    pub pending_versions: Vec<i64>,
    pub failed_versions: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticProvidersInfo {
    pub active_provider: Option<String>,
    pub profiles_by_provider: Vec<DiagnosticProviderProfileCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticProviderProfileCount {
    pub provider: String,
    pub configured_count: i64,
    pub missing_key_count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticRuntimeInfo {
    pub ytdlp: DiagnosticRuntimeCheck,
    pub secure_storage: DiagnosticRuntimeCheck,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticRuntimeCheck {
    pub status: String,
    pub available: bool,
    pub version: Option<String>,
    pub summary: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticTelegramInfo {
    pub account_count: i64,
    pub runtime_statuses: Vec<DiagnosticStatusCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticStatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticSourcesInfo {
    pub counts: Vec<DiagnosticSourceCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticSourceCount {
    pub source_type: String,
    pub source_subtype: Option<String>,
    pub active: bool,
    pub sync_state: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticItemsInfo {
    pub counts: Vec<DiagnosticItemCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticItemCount {
    pub source_type: String,
    pub source_subtype: Option<String>,
    pub item_kind: String,
    pub content_kind: String,
    pub has_content: bool,
    pub has_media: bool,
    pub media_kind: Option<String>,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticAnalysisRunsInfo {
    pub counts: Vec<DiagnosticAnalysisRunCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticAnalysisRunCount {
    pub provider: String,
    pub run_type: String,
    pub scope_type: String,
    pub status: String,
    pub snapshot_state: String,
    pub error_kind: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticLlmRequestsInfo {
    pub counts: Vec<DiagnosticLlmRequestCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticLlmRequestCount {
    pub provider: String,
    pub kind: String,
    pub state: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticYoutubeJobsInfo {
    pub counts: Vec<DiagnosticYoutubeJobCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticYoutubeJobCount {
    pub job_type: String,
    pub status: String,
    pub warning_state: String,
    pub error_kind: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticIngestInfo {
    pub batches: Vec<DiagnosticIngestBatchCount>,
    pub warnings: Vec<DiagnosticIngestWarningCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticIngestBatchCount {
    pub provider: String,
    pub ingest_kind: String,
    pub status: String,
    pub completeness: String,
    pub error_kind: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticIngestWarningCount {
    pub provider: String,
    pub ingest_kind: String,
    pub status: String,
    pub warning_code: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticPrivacyInfo {
    pub excluded_data_classes: Vec<String>,
}

pub(crate) fn excluded_data_classes() -> Vec<String> {
    [
        "source_content",
        "message_bodies",
        "transcript_text",
        "comment_text",
        "prompt_text",
        "report_text",
        "chat_text",
        "api_keys",
        "telegram_api_hashes",
        "youtube_cookies",
        "telegram_sessions",
        "raw_provider_payloads",
        "local_secret_paths",
        "local_database_path",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL_SOURCE_TITLE: &str = "private source title";
    const SENTINEL_URL: &str = "https://youtube.example/watch?v=private";
    const SENTINEL_PROFILE_ID: &str = "my-private-profile";
    const SENTINEL_BASE_URL: &str = "https://llm.internal.example/v1";
    const SENTINEL_RAW_ERROR: &str = "raw provider error with prompt text";

    #[test]
    fn diagnostic_summary_fixture_serializes_without_forbidden_sentinels() {
        let summary = fixture_summary();

        let json = serde_json::to_string(&summary).expect("serialize summary");

        for sentinel in [
            SENTINEL_SOURCE_TITLE,
            SENTINEL_URL,
            SENTINEL_PROFILE_ID,
            SENTINEL_BASE_URL,
            SENTINEL_RAW_ERROR,
        ] {
            assert!(
                !json.contains(sentinel),
                "summary leaked {sentinel}: {json}"
            );
        }
        assert!(json.contains("source_content"));
        assert!(json.contains("telegram"));
        assert!(json.contains("gemini"));
        assert!(json.contains("export_dc_fallback"));
        assert!(json.contains("sqliteAvailable"));
    }

    fn fixture_summary() -> DiagnosticSummary {
        DiagnosticSummary {
            app: DiagnosticAppInfo {
                app_name: "extractum".to_string(),
                app_version: "0.1.0".to_string(),
                build_mode: "debug".to_string(),
                generated_at_unix: 1_717_300_000,
            },
            database: DiagnosticDatabaseInfo {
                sqlite_available: true,
                migrations: DiagnosticMigrationInfo {
                    status: "current".to_string(),
                    expected_versions: vec![1, 2, 3],
                    applied_versions: vec![1, 2, 3],
                    pending_versions: Vec::new(),
                    failed_versions: Vec::new(),
                },
                account_count: 2,
            },
            providers: DiagnosticProvidersInfo {
                active_provider: Some("gemini".to_string()),
                profiles_by_provider: vec![DiagnosticProviderProfileCount {
                    provider: "gemini".to_string(),
                    configured_count: 1,
                    missing_key_count: 0,
                }],
            },
            runtimes: DiagnosticRuntimeInfo {
                ytdlp: DiagnosticRuntimeCheck {
                    status: "available".to_string(),
                    available: true,
                    version: Some("2026.01.01".to_string()),
                    summary: None,
                },
                secure_storage: DiagnosticRuntimeCheck {
                    status: "available".to_string(),
                    available: true,
                    version: None,
                    summary: None,
                },
            },
            telegram: DiagnosticTelegramInfo {
                account_count: 2,
                runtime_statuses: vec![DiagnosticStatusCount {
                    status: "ready".to_string(),
                    count: 1,
                }],
            },
            sources: DiagnosticSourcesInfo {
                counts: vec![DiagnosticSourceCount {
                    source_type: "telegram".to_string(),
                    source_subtype: Some("supergroup".to_string()),
                    active: true,
                    sync_state: "synced".to_string(),
                    count: 3,
                }],
            },
            items: DiagnosticItemsInfo {
                counts: vec![DiagnosticItemCount {
                    source_type: "youtube".to_string(),
                    source_subtype: Some("video".to_string()),
                    item_kind: "youtube_comment".to_string(),
                    content_kind: "text_only".to_string(),
                    has_content: true,
                    has_media: false,
                    media_kind: None,
                    count: 7,
                }],
            },
            analysis_runs: DiagnosticAnalysisRunsInfo {
                counts: vec![DiagnosticAnalysisRunCount {
                    provider: "gemini".to_string(),
                    run_type: "report".to_string(),
                    scope_type: "single_source".to_string(),
                    status: "failed".to_string(),
                    snapshot_state: "not_captured".to_string(),
                    error_kind: "network".to_string(),
                    count: 1,
                }],
            },
            llm_requests: DiagnosticLlmRequestsInfo {
                counts: vec![DiagnosticLlmRequestCount {
                    provider: "gemini".to_string(),
                    kind: "analysis_report_map".to_string(),
                    state: "running".to_string(),
                    count: 1,
                }],
            },
            youtube_jobs: DiagnosticYoutubeJobsInfo {
                counts: vec![DiagnosticYoutubeJobCount {
                    job_type: "youtube_video_full_sync".to_string(),
                    status: "failed".to_string(),
                    warning_state: "none".to_string(),
                    error_kind: "network".to_string(),
                    count: 1,
                }],
            },
            ingest: DiagnosticIngestInfo {
                batches: vec![DiagnosticIngestBatchCount {
                    provider: "telegram".to_string(),
                    ingest_kind: "takeout".to_string(),
                    status: "completed".to_string(),
                    completeness: "complete".to_string(),
                    error_kind: "none".to_string(),
                    count: 1,
                }],
                warnings: vec![DiagnosticIngestWarningCount {
                    provider: "telegram".to_string(),
                    ingest_kind: "takeout".to_string(),
                    status: "completed".to_string(),
                    warning_code: "export_dc_fallback".to_string(),
                    count: 2,
                }],
            },
            privacy: DiagnosticPrivacyInfo {
                excluded_data_classes: excluded_data_classes(),
            },
        }
    }
}
