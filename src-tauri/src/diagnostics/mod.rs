mod database;
mod dto;
mod redaction;
mod runtime;

use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::LlmSchedulerState;
use crate::secret_store::SecretStoreState;
use crate::telegram::TelegramState;
use crate::time::now_secs;
use crate::youtube::jobs::SourceJobState;

#[allow(unused_imports)]
pub(crate) use database::{load_account_ids, load_database_diagnostics};
#[allow(unused_imports)]
pub(crate) use dto::*;
#[allow(unused_imports)]
pub(crate) use redaction::{
    redact_json_value, redact_text, sanitized_error_message, MAX_SANITIZED_TEXT_CHARS,
};
#[allow(unused_imports)]
pub(crate) use runtime::{
    check_secure_storage, check_ytdlp_runtime, load_in_memory_runtime_diagnostics,
    load_provider_diagnostics, load_runtime_checks,
};

#[tauri::command]
pub(crate) async fn get_diagnostic_summary(
    handle: AppHandle,
    telegram_state: tauri::State<'_, TelegramState>,
    source_job_state: tauri::State<'_, SourceJobState>,
    llm_scheduler: tauri::State<'_, LlmSchedulerState>,
    secret_store: tauri::State<'_, SecretStoreState>,
) -> AppResult<DiagnosticSummary> {
    build_diagnostic_summary(
        &handle,
        telegram_state.inner(),
        source_job_state.inner(),
        llm_scheduler.inner(),
        secret_store.inner(),
    )
    .await
    .map_err(sanitize_diagnostic_error)
}

async fn build_diagnostic_summary(
    handle: &AppHandle,
    telegram_state: &TelegramState,
    source_job_state: &SourceJobState,
    llm_scheduler: &LlmSchedulerState,
    secret_store: &SecretStoreState,
) -> AppResult<DiagnosticSummary> {
    let pool = get_pool(handle).await?;
    let (database, sources, items, analysis_runs, ingest) =
        load_database_diagnostics(&pool).await?;
    let account_ids = load_account_ids(&pool).await?;
    let providers = load_provider_diagnostics(&pool, secret_store).await?;
    let runtimes = load_runtime_checks(secret_store).await;
    let (telegram, llm_requests, youtube_jobs) = load_in_memory_runtime_diagnostics(
        telegram_state,
        source_job_state,
        llm_scheduler,
        &account_ids,
        database.account_count,
    )
    .await;

    Ok(DiagnosticSummary {
        app: DiagnosticAppInfo {
            app_name: env!("CARGO_PKG_NAME").to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            build_mode: if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "release".to_string()
            },
            generated_at_unix: now_secs(),
        },
        database,
        providers,
        runtimes,
        telegram,
        sources,
        items,
        analysis_runs,
        llm_requests,
        youtube_jobs,
        ingest,
        privacy: DiagnosticPrivacyInfo {
            excluded_data_classes: excluded_data_classes(),
        },
    })
}

pub(crate) fn sanitize_diagnostic_error(error: AppError) -> AppError {
    let message = sanitized_error_message(&error.message);
    debug_assert!(message.chars().count() <= MAX_SANITIZED_TEXT_CHARS);
    AppError::new(
        error.kind,
        format!("Diagnostic summary failed: {message}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{AppError, AppErrorKind};

    const SENTINEL_API_KEY: &str = "sk-sentinel-command-error";
    const SENTINEL_PATH: &str =
        "C:\\Users\\Dima\\AppData\\Roaming\\org.ai.extractum\\extractum.db";
    const SENTINEL_PAYLOAD: &str = "raw provider payload with private message";
    const COMMAND_ERROR_PREFIX: &str = "Diagnostic summary failed: ";

    #[test]
    fn sanitize_diagnostic_error_bounds_and_redacts_command_errors() {
        let unicode_tail =
            "\u{043F}\u{0440}\u{0438}\u{0432}\u{0430}\u{0442}\u{043D}\u{044B}\u{0439} \u{0444}\u{0440}\u{0430}\u{0433}\u{043C}\u{0435}\u{043D}\u{0442} ".repeat(100);
        let error = AppError::internal(format!(
            "Database error at {SENTINEL_PATH}; api_key={SENTINEL_API_KEY}; payload: {SENTINEL_PAYLOAD}; unicode context: {unicode_tail}"
        ));

        let sanitized = sanitize_diagnostic_error(error);

        assert_eq!(sanitized.kind, AppErrorKind::Internal);
        assert!(sanitized.message.starts_with(COMMAND_ERROR_PREFIX));
        assert!(!sanitized.message.contains(SENTINEL_API_KEY));
        assert!(!sanitized.message.contains(SENTINEL_PATH));
        assert!(!sanitized.message.contains(SENTINEL_PAYLOAD));
        assert!(
            sanitized.message.chars().count()
                <= COMMAND_ERROR_PREFIX.chars().count() + MAX_SANITIZED_TEXT_CHARS,
            "bounded command error was too long: {}",
            sanitized.message.chars().count()
        );
    }

    #[test]
    fn serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data() {
        let summary = DiagnosticSummary {
            app: DiagnosticAppInfo {
                app_name: "extractum".to_string(),
                app_version: "0.1.0".to_string(),
                build_mode: "debug".to_string(),
                generated_at_unix: 1,
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
                account_count: 1,
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
                    status: "check_failed".to_string(),
                    available: false,
                    version: None,
                    summary: Some(sanitized_error_message(
                        "yt-dlp failed for https://youtube.example/watch?v=private",
                    )),
                },
                secure_storage: DiagnosticRuntimeCheck {
                    status: "available".to_string(),
                    available: true,
                    version: None,
                    summary: None,
                },
            },
            telegram: DiagnosticTelegramInfo {
                account_count: 1,
                runtime_statuses: vec![DiagnosticStatusCount {
                    status: "ready".to_string(),
                    count: 1,
                }],
            },
            sources: DiagnosticSourcesInfo {
                counts: vec![DiagnosticSourceCount {
                    source_type: "telegram".to_string(),
                    source_subtype: Some("channel".to_string()),
                    active: true,
                    sync_state: "synced".to_string(),
                    count: 1,
                }],
            },
            items: DiagnosticItemsInfo {
                counts: vec![DiagnosticItemCount {
                    source_type: "telegram".to_string(),
                    source_subtype: Some("channel".to_string()),
                    item_kind: "telegram_message".to_string(),
                    content_kind: "text_only".to_string(),
                    has_content: true,
                    has_media: false,
                    media_kind: None,
                    count: 12,
                }],
            },
            analysis_runs: DiagnosticAnalysisRunsInfo {
                counts: vec![DiagnosticAnalysisRunCount {
                    provider: "gemini".to_string(),
                    run_type: "report".to_string(),
                    scope_type: "single_source".to_string(),
                    status: "failed".to_string(),
                    snapshot_state: "failed".to_string(),
                    error_kind: "network".to_string(),
                    count: 1,
                }],
            },
            llm_requests: DiagnosticLlmRequestsInfo {
                counts: vec![DiagnosticLlmRequestCount {
                    provider: "gemini".to_string(),
                    kind: "analysis_chat".to_string(),
                    state: "queued".to_string(),
                    count: 1,
                }],
            },
            youtube_jobs: DiagnosticYoutubeJobsInfo {
                counts: vec![DiagnosticYoutubeJobCount {
                    job_type: "youtube_video_comments_sync".to_string(),
                    status: "failed".to_string(),
                    warning_state: "present".to_string(),
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
                    count: 1,
                }],
            },
            privacy: DiagnosticPrivacyInfo {
                excluded_data_classes: excluded_data_classes(),
            },
        };

        let json = serde_json::to_string(&summary).expect("serialize summary");

        for allowed in [
            "gemini",
            "telegram",
            "channel",
            "synced",
            "network",
            "export_dc_fallback",
            "source_content",
            "message_bodies",
            "local_database_path",
        ] {
            assert!(json.contains(allowed), "missing allowed value {allowed}: {json}");
        }

        for forbidden in [
            "youtube.example",
            "private",
            "api_key=",
            "apiHash",
            "baseUrl",
            "profileId",
            "source title",
            "raw provider payload",
            "extractum.db",
            "telegram_42.session.json",
        ] {
            assert!(
                !json.contains(forbidden),
                "summary leaked {forbidden}: {json}"
            );
        }
    }
}
