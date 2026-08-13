mod external_process;
#[cfg(test)]
mod shutdown;
use std::sync::Arc;

use external_process::{
    ExternalProcessShutdownState, ShutdownCleanup, ShutdownStart, ShutdownTiming,
};
mod analysis_documents;
mod apalis_jobs;
mod archive_read_model;
mod child_process;
mod compression {
    pub(crate) use extractum_core::compression::{
        compress_json_bytes, compress_text, decompress_bytes, decompress_text,
    };
}
mod db;
mod diagnostics;
mod process_tree;
mod security_config;
#[cfg(test)]
mod security_config_tests;
use apalis_jobs::{apalis_jobs_list, apalis_jobs_prune_terminal};
use diagnostics::get_diagnostic_summary;
mod error {
    pub(crate) use extractum_core::error::{database_error, AppError, AppErrorKind, AppResult};
}
mod forum_topics;
mod ingest_provenance;
mod job_helpers;
mod library_sources;
use library_sources::{list_library_catalog, list_library_sources};
mod media;
mod migrations;
mod projects;
mod readiness;
mod topic_memberships;
use migrations::{build_migrations, prepare_database};
use projects::{
    add_project_sources, create_project, delete_project,
    delete_project_youtube_video_source_from_library, list_project_sources, list_projects,
    list_research_projects, remove_project_sources, set_project_archived, set_project_pinned,
    update_project,
};
mod prompt_packs;
use prompt_packs::{
    cancel_prompt_pack_run, cleanup_interrupted_prompt_pack_runs, delete_prompt_pack_run,
    list_active_prompt_pack_runs, list_prompt_pack_run_stages, list_prompt_pack_runs,
    preflight_youtube_summary_run, start_youtube_summary_run, update_prompt_pack_run,
    PromptPackRunState,
};
#[cfg(all(dev, feature = "prompt-pack-dev-fixtures"))]
use prompt_packs::{
    clear_prompt_pack_cancellation_smoke_fixture, seed_prompt_pack_cancellation_smoke_fixture,
};
use prompt_packs::{get_prompt_pack_library, seed_builtin_prompt_packs};
use prompt_packs::{
    get_prompt_pack_result, get_prompt_pack_stage_artifact, get_prompt_pack_validation_findings,
    list_prompt_pack_audit_events, list_prompt_pack_stage_artifacts,
};

mod secret_store;
use secret_store::SecretStoreState;

mod accounts;
use accounts::{
    clear_account_phone, create_account, delete_account, get_account, list_accounts,
    set_account_phone,
};
mod account_deletion;

mod telegram;
#[path = "telegram_impl/lib.rs"]
mod telegram_impl;
mod telegram_session_store;
use telegram::{
    restore_telegram_accounts, tg_get_account_statuses, tg_init, tg_is_authenticated, tg_logout,
    tg_send_code, tg_sign_in, TelegramState,
};

mod source_ingest;
use source_ingest::SourceIngestLocks;

mod sql_helpers;
mod time {
    pub(crate) use extractum_core::time::{now_rfc3339_utc, now_secs, ymd_to_unix_midnight};
}
mod tx;

use tauri::Manager;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupPromptPackStep {
    SeedBuiltins,
    CleanupInterruptedRuns,
}

const STARTUP_PROMPT_PACK_STEPS: [StartupPromptPackStep; 2] = [
    StartupPromptPackStep::SeedBuiltins,
    StartupPromptPackStep::CleanupInterruptedRuns,
];

async fn initialize_prompt_pack_runtime(handle: tauri::AppHandle) {
    for step in STARTUP_PROMPT_PACK_STEPS {
        match step {
            StartupPromptPackStep::SeedBuiltins => {
                if let Err(error) = seed_builtin_prompt_packs(handle.clone()).await {
                    eprintln!("Prompt Pack seed failed: {error}");
                }
            }
            StartupPromptPackStep::CleanupInterruptedRuns => {
                cleanup_interrupted_prompt_pack_runs(handle.clone()).await;
            }
        }
    }
}

mod takeout_import;
use takeout_import::{
    cancel_takeout_source_import, list_takeout_import_recovery_states,
    list_takeout_source_import_jobs, run_takeout_export_dc_spike,
    start_takeout_migrated_history_import, start_takeout_source_import, TakeoutImportState,
};
#[cfg(dev)]
use takeout_import::{
    clear_takeout_cancellation_smoke_fixture, seed_takeout_cancellation_smoke_fixture,
};

mod sources;
use sources::identity_repair::{
    get_source_identity_repair_status, preview_source_identity_repair,
    run_startup_source_identity_repair, SourceIdentityRepairState,
};
use sources::{
    add_telegram_source, audit_legacy_telegram_source_metadata,
    clear_legacy_telegram_source_metadata, delete_source, get_sync_settings,
    list_source_forum_topics, list_source_items, list_sources, list_telegram_sources,
    save_sync_settings, sync_source,
};

mod youtube;
use youtube::detail::{
    get_youtube_playlist_detail, get_youtube_video_detail, list_youtube_source_summaries,
};
use youtube::job_commands::{
    cancel_source_job, list_source_jobs, retry_failed_youtube_playlist_videos,
    sync_youtube_playlist_video, sync_youtube_source,
};
#[cfg(dev)]
use youtube::job_commands::{
    clear_source_job_cancellation_smoke_fixture, seed_source_job_cancellation_smoke_fixture,
};
use youtube::jobs::SourceJobState;
use youtube::preview::{add_youtube_source, preview_youtube_source};
use youtube::process_runtime::YoutubeProcessRegistry;
use youtube::runtime::get_youtube_runtime_status;
use youtube::settings::{
    clear_youtube_auth, get_youtube_auth_status, get_youtube_settings, save_youtube_cookies,
    save_youtube_settings,
};
use youtube::thumbnail::{resolve_youtube_thumbnail, YoutubeThumbnailState};
use youtube::transcript_reader::list_youtube_transcript_segments;

mod notebooklm_export;
use notebooklm_export::export_source_to_notebooklm;

mod llm;
use llm::{
    ask_llm_stream, cancel_llm_request, clear_llm_profile_api_key, delete_llm_profile,
    get_llm_profiles, get_llm_request_snapshots, list_llm_provider_models, save_llm_profile,
    set_active_llm_profile, LlmSchedulerState,
};

mod gemini_browser;
use gemini_browser::{
    gemini_bridge_get_run, gemini_bridge_list_runs, gemini_bridge_open_browser,
    gemini_bridge_open_run_folder, gemini_bridge_resume, gemini_bridge_send_single,
    gemini_bridge_start_cdp_chrome, gemini_bridge_status, gemini_bridge_status_snapshot,
    gemini_bridge_stop, start_gemini_browser_job_worker, GeminiBrowserJobRuntime,
    GeminiBrowserState,
};

mod analysis;
use analysis::{cleanup_interrupted_analysis_runs, AnalysisState};

#[tauri::command]
fn ping_db() -> String {
    "Rust: Database plugin is initialized and migrations should have run.".to_string()
}

macro_rules! telegram_command_registration_inventory {
    ($consumer:ident, $($arguments:tt)*) => {
        $consumer! {
            $($arguments)*;
            list_accounts,
            get_account,
            create_account,
            set_account_phone,
            clear_account_phone,
            delete_account,
            tg_init,
            tg_is_authenticated,
            tg_get_account_statuses,
            tg_send_code,
            tg_sign_in,
            tg_logout,
        }
    };
}

macro_rules! telegram_command_handler {
    (
        [$($before:tt)*],
        [
            $($(#[$after_attribute:meta])* $after:ident
                $(=> ($implementation:path; (($($parameter:ident : $parameter_type:ty),* $(,)?) -> $result:ty; [$($wire:literal),* $(,)?])))?
            ),* $(,)?
        ];
        $($command:ident),* $(,)?
    ) => {
        tauri::generate_handler![
            $($before)*
            $($command,)*
            $($(#[$after_attribute])* $after,)*
        ]
    };
}

macro_rules! application_command_inventory {
    ($consumer:ident) => {
        $crate::telegram_command_registration_inventory! {
            $consumer,
            [
                ping_db,
                get_diagnostic_summary,
                apalis_jobs_list,
                apalis_jobs_prune_terminal,
            ],
            [
                delete_source,
                list_projects,
                list_research_projects,
                create_project,
                update_project,
                delete_project,
                set_project_pinned,
                set_project_archived,
                list_project_sources,
                add_project_sources,
                remove_project_sources,
                delete_project_youtube_video_source_from_library,
                // Preserve this established IPC request shape in the Wave 0 inventory.
                start_project_analysis => (crate::projects::start_project_analysis; ((
                    handle: tauri::AppHandle,
                    state: tauri::State<'_, extractum_analysis::AnalysisState>,
                    project_id: i64,
                    period_from: i64,
                    period_to: i64,
                    output_language: String,
                    prompt_template_id: i64,
                    model_override: Option<String>,
                    profile_id: Option<String>,
                    youtube_corpus_mode: Option<String>,
                    include_migrated_history: bool,
                ) -> crate::error::AppResult<i64>; ["projectId", "periodFrom", "periodTo", "outputLanguage", "promptTemplateId", "modelOverride", "profileId", "youtubeCorpusMode", "includeMigratedHistory"])),
                get_project_data_range => (crate::projects::get_project_data_range; ((
                    handle: tauri::AppHandle,
                    project_id: i64,
                    youtube_corpus_mode: Option<String>,
                    include_migrated_history: bool,
                ) -> crate::error::AppResult<crate::projects::ProjectDataRange>; ["projectId", "youtubeCorpusMode", "includeMigratedHistory"])),
                list_project_runs => (crate::projects::list_project_runs; ((
                    handle: tauri::AppHandle,
                    project_id: i64,
                ) -> crate::error::AppResult<Vec<extractum_analysis::AnalysisRunSummary>>; ["projectId"])),
                get_prompt_pack_library,
                preflight_youtube_summary_run,
                start_youtube_summary_run,
                cancel_prompt_pack_run,
                update_prompt_pack_run,
                delete_prompt_pack_run,
                list_prompt_pack_runs,
                list_active_prompt_pack_runs,
                list_prompt_pack_run_stages,
                get_prompt_pack_result,
                list_prompt_pack_stage_artifacts,
                get_prompt_pack_stage_artifact,
                get_prompt_pack_validation_findings,
                list_prompt_pack_audit_events,
                #[cfg(all(dev, feature = "prompt-pack-dev-fixtures"))]
                seed_prompt_pack_cancellation_smoke_fixture,
                #[cfg(all(dev, feature = "prompt-pack-dev-fixtures"))]
                clear_prompt_pack_cancellation_smoke_fixture,
                get_source_identity_repair_status,
                preview_source_identity_repair,
                audit_legacy_telegram_source_metadata,
                clear_legacy_telegram_source_metadata,
                list_telegram_sources,
                add_telegram_source,
                list_sources,
                get_sync_settings,
                save_sync_settings,
                sync_source,
                start_takeout_migrated_history_import,
                start_takeout_source_import,
                cancel_takeout_source_import,
                list_takeout_source_import_jobs,
                list_takeout_import_recovery_states,
                #[cfg(dev)]
                seed_takeout_cancellation_smoke_fixture,
                #[cfg(dev)]
                clear_takeout_cancellation_smoke_fixture,
                run_takeout_export_dc_spike,
                list_source_items,
                list_source_forum_topics,
                export_source_to_notebooklm,
                get_llm_profiles,
                get_llm_request_snapshots,
                save_llm_profile,
                clear_llm_profile_api_key,
                delete_llm_profile,
                set_active_llm_profile,
                list_llm_provider_models,
                ask_llm_stream,
                cancel_llm_request,
                gemini_bridge_status,
                gemini_bridge_status_snapshot,
                gemini_bridge_open_browser,
                gemini_bridge_start_cdp_chrome,
                gemini_bridge_send_single,
                gemini_bridge_resume,
                gemini_bridge_stop,
                gemini_bridge_list_runs,
                gemini_bridge_get_run,
                gemini_bridge_open_run_folder,
                list_analysis_sources => (crate::analysis::list_analysis_sources; ((
                    handle: tauri::AppHandle,
                    repair_state: tauri::State<'_, crate::sources::SourceIdentityRepairState>,
                ) -> crate::error::AppResult<Vec<extractum_analysis::AnalysisSourceOption>>; [])),
                list_library_sources,
                list_library_catalog,
                list_analysis_prompt_templates => (crate::analysis::list_analysis_prompt_templates; ((handle: tauri::AppHandle, template_kind: Option<String>) -> crate::error::AppResult<Vec<extractum_analysis::AnalysisPromptTemplate>>; ["templateKind"])),
                create_analysis_prompt_template => (crate::analysis::create_analysis_prompt_template; ((handle: tauri::AppHandle, name: String, template_kind: String, body: String) -> crate::error::AppResult<extractum_analysis::AnalysisPromptTemplate>; ["name", "templateKind", "body"])),
                update_analysis_prompt_template => (crate::analysis::update_analysis_prompt_template; ((handle: tauri::AppHandle, template_id: i64, name: String, body: String) -> crate::error::AppResult<extractum_analysis::AnalysisPromptTemplate>; ["templateId", "name", "body"])),
                delete_analysis_prompt_template => (crate::analysis::delete_analysis_prompt_template; ((handle: tauri::AppHandle, template_id: i64) -> crate::error::AppResult<()>; ["templateId"])),
                list_analysis_source_groups => (crate::analysis::list_analysis_source_groups; ((handle: tauri::AppHandle) -> crate::error::AppResult<Vec<extractum_analysis::AnalysisSourceGroup>>; [])),
                create_analysis_source_group => (crate::analysis::create_analysis_source_group; ((handle: tauri::AppHandle, name: String, source_type: String, source_ids: Vec<i64>) -> crate::error::AppResult<extractum_analysis::AnalysisSourceGroup>; ["name", "sourceType", "sourceIds"])),
                update_analysis_source_group => (crate::analysis::update_analysis_source_group; ((handle: tauri::AppHandle, group_id: i64, name: String, source_type: String, source_ids: Vec<i64>) -> crate::error::AppResult<extractum_analysis::AnalysisSourceGroup>; ["groupId", "name", "sourceType", "sourceIds"])),
                delete_analysis_source_group => (crate::analysis::delete_analysis_source_group; ((handle: tauri::AppHandle, group_id: i64) -> crate::error::AppResult<()>; ["groupId"])),
                // Preserve this established IPC request shape in the Wave 0 inventory.
                list_analysis_runs => (crate::analysis::list_analysis_runs; ((handle: tauri::AppHandle, source_id: Option<i64>, source_group_id: Option<i64>, limit: Option<i64>, query: Option<String>, status: Option<String>, provider: Option<String>, model: Option<String>, template: Option<String>, date_from: Option<String>, date_to: Option<String>) -> crate::error::AppResult<Vec<extractum_analysis::AnalysisRunSummary>>; ["sourceId", "sourceGroupId", "limit", "query", "status", "provider", "model", "template", "dateFrom", "dateTo"])),
                list_active_analysis_runs => (crate::analysis::list_active_analysis_runs; ((handle: tauri::AppHandle, state: tauri::State<'_, extractum_analysis::AnalysisState>) -> crate::error::AppResult<Vec<extractum_analysis::AnalysisRunSummary>>; [])),
                get_analysis_run => (crate::analysis::get_analysis_run; ((handle: tauri::AppHandle, run_id: i64) -> crate::error::AppResult<Option<extractum_analysis::AnalysisRunDetail>>; ["runId"])),
                list_analysis_run_messages => (crate::analysis::list_analysis_run_messages; ((handle: tauri::AppHandle, run_id: i64, after: Option<extractum_analysis::AnalysisRunMessageCursor>, limit: Option<i64>, source_id: Option<i64>, around_ref: Option<String>) -> crate::error::AppResult<extractum_analysis::AnalysisRunMessagesPage>; ["runId", "after", "limit", "sourceId", "aroundRef"])),
                delete_analysis_run => (crate::analysis::delete_analysis_run; ((handle: tauri::AppHandle, state: tauri::State<'_, extractum_analysis::AnalysisState>, run_id: i64) -> crate::error::AppResult<()>; ["runId"])),
                get_analysis_run_trace => (crate::analysis::get_analysis_run_trace; ((handle: tauri::AppHandle, run_id: i64) -> crate::error::AppResult<extractum_analysis::AnalysisTraceData>; ["runId"])),
                resolve_analysis_trace_refs => (crate::analysis::resolve_analysis_trace_refs; ((handle: tauri::AppHandle, run_id: i64, refs: Vec<String>) -> crate::error::AppResult<Vec<extractum_analysis::AnalysisTraceRef>>; ["runId", "refs"])),
                list_analysis_chat_messages => (crate::analysis::list_analysis_chat_messages; ((handle: tauri::AppHandle, run_id: i64) -> crate::error::AppResult<Vec<extractum_analysis::AnalysisChatMessage>>; ["runId"])),
                clear_analysis_chat_messages => (crate::analysis::clear_analysis_chat_messages; ((handle: tauri::AppHandle, run_id: i64) -> crate::error::AppResult<()>; ["runId"])),
                ask_analysis_run_question => (crate::analysis::ask_analysis_run_question; ((handle: tauri::AppHandle, run_id: i64, question: String, model_override: Option<String>, profile_id: Option<String>) -> crate::error::AppResult<String>; ["runId", "question", "modelOverride", "profileId"])),
                // Preserve this established IPC request shape in the Wave 0 inventory.
                start_analysis_report => (crate::analysis::start_analysis_report; ((handle: tauri::AppHandle, state: tauri::State<'_, extractum_analysis::AnalysisState>, source_id: Option<i64>, source_group_id: Option<i64>, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool) -> crate::error::AppResult<i64>; ["sourceId", "sourceGroupId", "periodFrom", "periodTo", "outputLanguage", "promptTemplateId", "modelOverride", "profileId", "youtubeCorpusMode", "includeMigratedHistory"])),
                cancel_analysis_run => (crate::analysis::cancel_analysis_run; ((handle: tauri::AppHandle, state: tauri::State<'_, extractum_analysis::AnalysisState>, scheduler: tauri::State<'_, std::sync::Arc<extractum_llm::LlmSchedulerState>>, run_id: i64) -> crate::error::AppResult<()>; ["runId"])),
                #[cfg(dev)]
                seed_analysis_redesign_fixtures => (crate::analysis::seed_analysis_redesign_fixtures; ((handle: tauri::AppHandle, state: tauri::State<'_, extractum_analysis::AnalysisState>) -> crate::error::AppResult<crate::analysis::AnalysisRedesignFixtureSummary>; [])),
                #[cfg(dev)]
                clear_analysis_redesign_fixture_active_runs => (crate::analysis::clear_analysis_redesign_fixture_active_runs; ((handle: tauri::AppHandle, state: tauri::State<'_, extractum_analysis::AnalysisState>) -> crate::error::AppResult<()>; [])),
                #[cfg(dev)]
                clear_analysis_redesign_fixtures => (crate::analysis::clear_analysis_redesign_fixtures; ((handle: tauri::AppHandle, state: tauri::State<'_, extractum_analysis::AnalysisState>) -> crate::error::AppResult<crate::analysis::AnalysisRedesignFixtureSummary>; [])),
                preview_youtube_source,
                add_youtube_source,
                sync_youtube_source,
                sync_youtube_playlist_video,
                cancel_source_job,
                list_source_jobs,
                retry_failed_youtube_playlist_videos,
                #[cfg(dev)]
                seed_source_job_cancellation_smoke_fixture,
                #[cfg(dev)]
                clear_source_job_cancellation_smoke_fixture,
                get_youtube_runtime_status,
                list_youtube_source_summaries,
                get_youtube_video_detail,
                get_youtube_playlist_detail,
                list_youtube_transcript_segments,
                get_youtube_settings,
                save_youtube_settings,
                get_youtube_auth_status,
                save_youtube_cookies,
                clear_youtube_auth,
                resolve_youtube_thumbnail
            ]
        }
    };
}

macro_rules! define_application_ipc_wrapper_body {
    (
        $(#[$attribute:meta])* $command:ident =>
        ($implementation:path; (($($parameter:ident : $parameter_type:ty),* $(,)?) -> $result:ty; [$($wire:literal),* $(,)?]))
    ) => {
        $(#[$attribute])*
        #[tauri::command]
        pub(super) async fn $command($($parameter: $parameter_type),*) -> $result {
            $implementation($($parameter),*).await
        }
    };
    ($(#[$attribute:meta])* $command:ident) => {};
}

macro_rules! define_application_ipc_wrapper {
    (start_project_analysis => $signature:tt) => {
        define_application_ipc_wrapper_body!(
            #[allow(clippy::too_many_arguments)]
            start_project_analysis => $signature
        );
    };
    (list_analysis_runs => $signature:tt) => {
        define_application_ipc_wrapper_body!(
            #[allow(clippy::too_many_arguments)]
            list_analysis_runs => $signature
        );
    };
    (start_analysis_report => $signature:tt) => {
        define_application_ipc_wrapper_body!(
            #[allow(clippy::too_many_arguments)]
            start_analysis_report => $signature
        );
    };
    ($(#[$attribute:meta])* $command:ident => $signature:tt) => {
        define_application_ipc_wrapper_body!(
            $(#[$attribute])*
            $command => $signature
        );
    };
    ($(#[$attribute:meta])* $command:ident) => {};
}

macro_rules! define_application_ipc_wrappers {
    (
        [$($before:ident,)*],
        [
            $($(#[$after_attribute:meta])* $after:ident
                $(=> ($implementation:path; (($($parameter:ident : $parameter_type:ty),* $(,)?) -> $result:ty; [$($wire:literal),* $(,)?])))?
            ),* $(,)?
        ];
        $($telegram:ident),* $(,)?
    ) => {
        $(
            define_application_ipc_wrapper!(
                $(#[$after_attribute])* $after
                $(=> ($implementation; (($($parameter: $parameter_type),*) -> $result; [$($wire),*])))?
            );
        )*
    };
}

mod application_ipc_commands {
    application_command_inventory!(define_application_ipc_wrappers);
}

use application_ipc_commands::{
    ask_analysis_run_question, cancel_analysis_run, clear_analysis_chat_messages,
    create_analysis_prompt_template, create_analysis_source_group, delete_analysis_prompt_template,
    delete_analysis_run, delete_analysis_source_group, get_analysis_run, get_analysis_run_trace,
    get_project_data_range, list_active_analysis_runs, list_analysis_chat_messages,
    list_analysis_prompt_templates, list_analysis_run_messages, list_analysis_runs,
    list_analysis_source_groups, list_analysis_sources, list_project_runs,
    resolve_analysis_trace_refs, start_analysis_report, start_project_analysis,
    update_analysis_prompt_template, update_analysis_source_group,
};
#[cfg(dev)]
use application_ipc_commands::{
    clear_analysis_redesign_fixture_active_runs, clear_analysis_redesign_fixtures,
    seed_analysis_redesign_fixtures,
};

#[cfg(test)]
pub(crate) use application_command_inventory;
pub(crate) use telegram_command_registration_inventory;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    prepare_database().expect("database preparation failed");

    let builder = tauri::Builder::default()
        .manage(ExternalProcessShutdownState::new())
        .manage(YoutubeProcessRegistry::new())
        .manage(TelegramState::new())
        .manage(SourceIngestLocks::new())
        .manage(TakeoutImportState::new())
        .manage(SourceJobState::new())
        .manage(AnalysisState::new())
        .manage(PromptPackRunState::new())
        .manage(Arc::new(LlmSchedulerState::new()))
        .manage(GeminiBrowserState::new())
        .manage(GeminiBrowserJobRuntime::default())
        .manage(SourceIdentityRepairState::new())
        .manage(YoutubeThumbnailState::new())
        .manage(SecretStoreState::system())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(crate::db::DB_URL, build_migrations())
                .build(),
        );

    #[cfg(dev)]
    let builder = {
        security_config::require_local_dev_command(security_config::MCP_BIND_ADDRESS)
            .expect("MCP bridge requires a localhost development build");
        builder.plugin(
            tauri_plugin_mcp_bridge::Builder::new()
                .bind_address(security_config::MCP_BIND_ADDRESS)
                .build(),
        )
    };

    builder
        .setup(|app| {
            if security_config::production_devtools_allowed(true) {
                #[cfg(feature = "csp-verification")]
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            let worker_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = start_gemini_browser_job_worker(worker_handle).await {
                    eprintln!("Failed to start Gemini Browser job worker: {error}");
                }
            });
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                initialize_prompt_pack_runtime(handle.clone()).await;
                cleanup_interrupted_analysis_runs(handle.clone()).await;
                restore_telegram_accounts(handle).await;
            });
            let repair_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                run_startup_source_identity_repair(repair_handle).await;
            });
            Ok(())
        })
        .invoke_handler(application_command_inventory!(telegram_command_handler))
        .build(tauri::generate_context!())
        .expect("error while building Tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
                let shutdown = app.state::<ExternalProcessShutdownState>().inner().clone();
                let scheduler = external_process::os_thread_watchdog_scheduler();
                let clock = external_process::system_monotonic_clock();
                let exit_handle = app.clone();
                let exit: external_process::ExitCallback =
                    std::sync::Arc::new(move |code| exit_handle.exit(code));

                match shutdown.start(code, ShutdownTiming::default(), &scheduler, exit, clock) {
                    ShutdownStart::Started(run) => {
                        api.prevent_exit();
                        let registry = app.state::<YoutubeProcessRegistry>().inner().clone();
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            run.coordinate(Box::new(move || {
                                let sidecar_handle = handle.clone();
                                let chrome_handle = handle.clone();
                                let youtube: ShutdownCleanup = Box::pin(async move {
                                    registry.cancel_and_wait().await;
                                    Ok(())
                                });
                                let sidecar: ShutdownCleanup = Box::pin(async move {
                                    let state = sidecar_handle.state::<GeminiBrowserState>();
                                    gemini_browser::shutdown_sidecar(
                                        &sidecar_handle,
                                        state.inner(),
                                    )
                                    .await;
                                    Ok(())
                                });
                                let chrome: ShutdownCleanup = Box::pin(async move {
                                    let state = chrome_handle.state::<GeminiBrowserState>();
                                    gemini_browser::shutdown_cdp_chrome(state.inner()).await;
                                    Ok(())
                                });
                                vec![youtube, sidecar, chrome]
                            }))
                            .await;
                        });
                    }
                    ShutdownStart::AlreadyShuttingDown => api.prevent_exit(),
                    ShutdownStart::Completed => {}
                }
            }
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn startup_seeds_prompt_packs_before_interrupted_run_cleanup() {
        assert_eq!(
            super::STARTUP_PROMPT_PACK_STEPS,
            [
                super::StartupPromptPackStep::SeedBuiltins,
                super::StartupPromptPackStep::CleanupInterruptedRuns,
            ]
        );
    }

    macro_rules! complete_application_inventory_names {
        (
            [$($before:ident,)*],
            [
                $($(#[$after_attribute:meta])* $after:ident
                    $(=> ($implementation:path; (($($parameter:ident : $parameter_type:ty),* $(,)?) -> $result:ty; [$($wire:literal),* $(,)?])))?
                ),* $(,)?
            ];
            $($command:ident),* $(,)?
        ) => {
            [
                $(stringify!($before),)*
                $(stringify!($command),)*
                $(stringify!($after)),*
            ]
        };
    }

    #[test]
    fn telegram_command_registration_inventory_is_exact() {
        let registered = application_command_inventory!(complete_application_inventory_names);
        let frozen = [
            "list_accounts",
            "get_account",
            "create_account",
            "set_account_phone",
            "clear_account_phone",
            "delete_account",
            "tg_init",
            "tg_is_authenticated",
            "tg_get_account_statuses",
            "tg_send_code",
            "tg_sign_in",
            "tg_logout",
        ];

        for command in frozen {
            assert_eq!(
                registered
                    .iter()
                    .filter(|registered| **registered == command)
                    .count(),
                1,
                "{command} must occur exactly once in the complete application command inventory",
            );
        }
    }

    #[test]
    fn analysis_command_registration_inventory_is_exact() {
        let registered = application_command_inventory!(complete_application_inventory_names);
        let analysis = [
            "list_analysis_sources",
            "list_analysis_runs",
            "list_active_analysis_runs",
            "get_analysis_run",
            "list_analysis_run_messages",
            "get_analysis_run_trace",
            "delete_analysis_run",
            "resolve_analysis_trace_refs",
            "list_analysis_prompt_templates",
            "create_analysis_prompt_template",
            "update_analysis_prompt_template",
            "delete_analysis_prompt_template",
            "list_analysis_source_groups",
            "create_analysis_source_group",
            "update_analysis_source_group",
            "delete_analysis_source_group",
            "list_analysis_chat_messages",
            "clear_analysis_chat_messages",
            "ask_analysis_run_question",
            "start_analysis_report",
            "cancel_analysis_run",
        ];
        let project = [
            "start_project_analysis",
            "list_project_runs",
            "get_project_data_range",
        ];
        let development = [
            "seed_analysis_redesign_fixtures",
            "clear_analysis_redesign_fixtures",
            "clear_analysis_redesign_fixture_active_runs",
        ];

        assert_eq!(analysis.len(), 21);
        assert_eq!(project.len(), 3);
        assert_eq!(development.len(), 3);
        for command in analysis.into_iter().chain(project).chain(development) {
            assert_eq!(
                registered
                    .iter()
                    .filter(|registered| **registered == command)
                    .count(),
                1,
                "{command} must occur exactly once in the complete application command inventory",
            );
        }
    }
}
