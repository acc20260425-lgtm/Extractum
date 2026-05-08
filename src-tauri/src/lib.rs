mod compression;
mod db;
mod error;
mod forum_topics;
mod media;
mod migrations;
use migrations::{build_migrations, prepare_database};

mod secret_store;
use secret_store::SecretStoreState;

mod accounts;
use accounts::{
    clear_account_phone, create_account, delete_account, get_account, list_accounts,
    set_account_phone,
};

mod telegram;
mod telegram_session_store;
use telegram::{
    restore_telegram_accounts, tg_get_account_statuses, tg_init, tg_is_authenticated, tg_logout,
    tg_send_code, tg_sign_in, TelegramState,
};

mod source_ingest;
use source_ingest::SourceIngestLocks;

mod takeout_import;
use takeout_import::{
    cancel_takeout_source_import, list_takeout_source_import_jobs, run_takeout_export_dc_spike,
    start_takeout_source_import, TakeoutImportState,
};

mod sources;
use sources::{
    add_telegram_source, delete_source, get_sync_settings, list_source_forum_topics,
    list_source_items, list_sources, list_telegram_sources, save_sync_settings, sync_source,
};

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
    list_analysis_chat_messages, list_analysis_prompt_templates, list_analysis_runs,
    list_analysis_source_groups, list_analysis_sources, resolve_analysis_trace_refs,
    start_analysis_report, update_analysis_prompt_template, update_analysis_source_group,
    AnalysisState,
};

#[tauri::command]
fn ping_db() -> String {
    "Rust: Database plugin is initialized and migrations should have run.".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    prepare_database();

    let mut builder = tauri::Builder::default()
        .manage(TelegramState::new())
        .manage(SourceIngestLocks::new())
        .manage(TakeoutImportState::new())
        .manage(AnalysisState::new())
        .manage(LlmSchedulerState::new())
        .manage(SecretStoreState::system())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:extractum.db", build_migrations())
                .build(),
        );

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                cleanup_interrupted_analysis_runs(handle.clone()).await;
                restore_telegram_accounts(handle).await;
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
            list_telegram_sources,
            add_telegram_source,
            list_sources,
            get_sync_settings,
            save_sync_settings,
            sync_source,
            start_takeout_source_import,
            cancel_takeout_source_import,
            list_takeout_source_import_jobs,
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
            delete_analysis_run,
            get_analysis_run_trace,
            resolve_analysis_trace_refs,
            list_analysis_chat_messages,
            clear_analysis_chat_messages,
            ask_analysis_run_question,
            start_analysis_report,
            cancel_analysis_run
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
