mod analysis_documents;
mod archive_read_model;
mod compression;
mod db;
#[cfg(test)]
mod diagnostics;
mod error;
mod forum_topics;
mod ingest_provenance;
mod job_helpers;
mod media;
mod migrations;
mod readiness;
mod topic_memberships;
use migrations::{build_migrations, prepare_database};

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

mod takeout_import;
use takeout_import::{
    cancel_takeout_source_import, list_takeout_import_recovery_states,
    list_takeout_source_import_jobs, run_takeout_export_dc_spike,
    start_takeout_migrated_history_import, start_takeout_source_import, TakeoutImportState,
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
    ask_llm_stream, cancel_llm_request, clear_llm_profile_api_key, get_llm_profiles,
    get_llm_request_snapshots, list_llm_provider_models, save_llm_profile, set_active_llm_profile,
    LlmSchedulerState,
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
        .manage(LlmSchedulerState::new())
        .manage(SourceIdentityRepairState::new())
        .manage(SecretStoreState::system())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:extractum.db", build_migrations())
                .build(),
        );

    #[cfg(debug_assertions)]
    let builder = builder.plugin(tauri_plugin_mcp_bridge::init());

    builder
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
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
            run_takeout_export_dc_spike,
            list_source_items,
            list_source_forum_topics,
            export_source_to_notebooklm,
            get_llm_profiles,
            get_llm_request_snapshots,
            save_llm_profile,
            clear_llm_profile_api_key,
            set_active_llm_profile,
            list_llm_provider_models,
            ask_llm_stream,
            cancel_llm_request,
            list_analysis_sources,
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
