mod analysis_documents;
mod apalis_jobs;
mod archive_read_model;
mod compression;
mod db;
mod diagnostics;
use apalis_jobs::{apalis_jobs_list, apalis_jobs_prune_terminal};
use diagnostics::get_diagnostic_summary;
mod error;
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
    delete_project_youtube_video_source_from_library, get_project_data_range, list_project_runs,
    list_project_sources, list_projects, list_research_projects, remove_project_sources,
    set_project_archived, set_project_pinned, start_project_analysis, update_project,
};
mod prompt_packs;
use prompt_packs::{
    cancel_prompt_pack_run, cleanup_interrupted_prompt_pack_runs, delete_prompt_pack_run,
    list_active_prompt_pack_runs, list_prompt_pack_run_stages, list_prompt_pack_runs,
    preflight_youtube_summary_run, start_youtube_summary_run, update_prompt_pack_run,
    PromptPackRunState,
};
#[cfg(debug_assertions)]
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
mod telegram_session_store;
use telegram::{
    restore_telegram_accounts, tg_get_account_statuses, tg_init, tg_is_authenticated, tg_logout,
    tg_send_code, tg_sign_in, TelegramState,
};

mod source_ingest;
use source_ingest::SourceIngestLocks;

mod sql_helpers;
mod time;
mod tx;

use tauri::Manager;

mod takeout_import;
use takeout_import::{
    cancel_takeout_source_import, list_takeout_import_recovery_states,
    list_takeout_source_import_jobs, run_takeout_export_dc_spike,
    start_takeout_migrated_history_import, start_takeout_source_import, TakeoutImportState,
};
#[cfg(debug_assertions)]
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
#[cfg(debug_assertions)]
use youtube::job_commands::{
    clear_source_job_cancellation_smoke_fixture, seed_source_job_cancellation_smoke_fixture,
};
use youtube::jobs::SourceJobState;
use youtube::preview::{add_youtube_source, preview_youtube_source};
use youtube::runtime::get_youtube_runtime_status;
use youtube::settings::{
    clear_youtube_auth, get_youtube_auth_status, get_youtube_settings, save_youtube_cookies,
    save_youtube_settings,
};
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
use analysis::{
    ask_analysis_run_question, cancel_analysis_run, cleanup_interrupted_analysis_runs,
    clear_analysis_chat_messages, create_analysis_prompt_template, create_analysis_source_group,
    delete_analysis_prompt_template, delete_analysis_run, delete_analysis_source_group,
    get_analysis_run, get_analysis_run_trace, list_active_analysis_runs,
    list_analysis_chat_messages, list_analysis_prompt_templates, list_analysis_run_messages,
    list_analysis_runs, list_analysis_source_groups, list_analysis_sources,
    resolve_analysis_trace_refs, start_analysis_report, update_analysis_prompt_template,
    update_analysis_source_group, AnalysisState,
};
#[cfg(debug_assertions)]
use analysis::{
    clear_analysis_redesign_fixture_active_runs, clear_analysis_redesign_fixtures,
    seed_analysis_redesign_fixtures,
};

#[tauri::command]
fn ping_db() -> String {
    "Rust: Database plugin is initialized and migrations should have run.".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    prepare_database().expect("database preparation failed");

    let builder = tauri::Builder::default()
        .manage(TelegramState::new())
        .manage(SourceIngestLocks::new())
        .manage(TakeoutImportState::new())
        .manage(SourceJobState::new())
        .manage(AnalysisState::new())
        .manage(PromptPackRunState::new())
        .manage(LlmSchedulerState::new())
        .manage(GeminiBrowserState::new())
        .manage(GeminiBrowserJobRuntime::default())
        .manage(SourceIdentityRepairState::new())
        .manage(SecretStoreState::system())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(crate::db::DB_URL, build_migrations())
                .build(),
        );

    #[cfg(debug_assertions)]
    let builder = builder.plugin(tauri_plugin_mcp_bridge::init());

    builder
        .setup(|app| {
            app.state::<GeminiBrowserState>()
                .init_status_snapshot(app.handle())?;
            let worker_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = start_gemini_browser_job_worker(worker_handle).await {
                    eprintln!("Failed to start Gemini Browser job worker: {error}");
                }
            });
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = seed_builtin_prompt_packs(handle.clone()).await {
                    eprintln!("Prompt Pack seed failed: {error}");
                }
                cleanup_interrupted_prompt_pack_runs(handle.clone()).await;
                cleanup_interrupted_analysis_runs(handle.clone()).await;
                restore_telegram_accounts(handle).await;
            });
            let repair_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                run_startup_source_identity_repair(repair_handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping_db,
            get_diagnostic_summary,
            apalis_jobs_list,
            apalis_jobs_prune_terminal,
            tg_init,
            tg_is_authenticated,
            tg_get_account_statuses,
            tg_send_code,
            tg_sign_in,
            tg_logout,
            list_accounts,
            get_account,
            create_account,
            set_account_phone,
            clear_account_phone,
            delete_account,
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
            start_project_analysis,
            get_project_data_range,
            list_project_runs,
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
            #[cfg(debug_assertions)]
            seed_prompt_pack_cancellation_smoke_fixture,
            #[cfg(debug_assertions)]
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
            #[cfg(debug_assertions)]
            seed_takeout_cancellation_smoke_fixture,
            #[cfg(debug_assertions)]
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
            list_analysis_sources,
            list_library_sources,
            list_library_catalog,
            list_analysis_prompt_templates,
            create_analysis_prompt_template,
            update_analysis_prompt_template,
            delete_analysis_prompt_template,
            list_analysis_source_groups,
            create_analysis_source_group,
            update_analysis_source_group,
            delete_analysis_source_group,
            list_analysis_runs,
            list_active_analysis_runs,
            get_analysis_run,
            list_analysis_run_messages,
            delete_analysis_run,
            get_analysis_run_trace,
            resolve_analysis_trace_refs,
            list_analysis_chat_messages,
            clear_analysis_chat_messages,
            ask_analysis_run_question,
            start_analysis_report,
            cancel_analysis_run,
            #[cfg(debug_assertions)]
            seed_analysis_redesign_fixtures,
            #[cfg(debug_assertions)]
            clear_analysis_redesign_fixture_active_runs,
            #[cfg(debug_assertions)]
            clear_analysis_redesign_fixtures,
            preview_youtube_source,
            add_youtube_source,
            sync_youtube_source,
            sync_youtube_playlist_video,
            cancel_source_job,
            list_source_jobs,
            retry_failed_youtube_playlist_videos,
            #[cfg(debug_assertions)]
            seed_source_job_cancellation_smoke_fixture,
            #[cfg(debug_assertions)]
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
            clear_youtube_auth
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
