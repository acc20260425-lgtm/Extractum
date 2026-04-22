mod db;
mod migrations;
use migrations::{build_migrations, prepare_database};

mod accounts;
use accounts::{clear_account_phone, create_account, delete_account, get_account, list_accounts, set_account_phone};

mod telegram;
use telegram::{restore_telegram_accounts, tg_get_account_statuses, tg_init, tg_is_authenticated, tg_logout, tg_send_code, tg_sign_in, TelegramState};

mod sources;
use sources::{list_telegram_channels, add_telegram_source, list_sources, sync_channel, get_items, delete_source};

mod llm;
use llm::{get_llm_profiles, save_llm_profile, ask_llm_stream};

mod analysis;
use analysis::{
    ask_analysis_run_question, clear_analysis_chat_messages, create_analysis_prompt_template, create_analysis_source_group,
    delete_analysis_prompt_template, delete_analysis_source_group, get_analysis_run,
    get_analysis_run_trace, list_analysis_chat_messages, list_analysis_prompt_templates, list_analysis_runs,
    list_analysis_source_groups, list_analysis_sources, start_analysis_report,
    update_analysis_prompt_template, update_analysis_source_group,
    resolve_analysis_trace_refs,
};

#[tauri::command]
fn ping_db() -> String {
    "Rust: Database plugin is initialized and migrations should have run.".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    prepare_database();

    tauri::Builder::default()
        .manage(TelegramState::new())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:extractum.db", build_migrations())
                .build(),
        )
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
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
            list_telegram_channels,
            add_telegram_source,
            list_sources,
            sync_channel,
            get_items,
            get_llm_profiles,
            save_llm_profile,
            ask_llm_stream,
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
            get_analysis_run,
            get_analysis_run_trace,
            resolve_analysis_trace_refs,
            list_analysis_chat_messages,
            clear_analysis_chat_messages,
            ask_analysis_run_question,
            start_analysis_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
