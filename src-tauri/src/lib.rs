// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use sha2::{Digest, Sha384};
use std::path::PathBuf;
use tauri_plugin_sql::{Migration, MigrationKind};

mod db;

mod telegram;
use telegram::{restore_telegram_accounts, tg_get_account_statuses, tg_init, tg_is_authenticated, tg_logout, tg_send_code, tg_sign_in, TelegramState};

mod sources;
use sources::{list_telegram_channels, add_telegram_source, list_sources, sync_channel, get_items, list_accounts, get_account, create_account, set_account_phone, clear_account_phone, delete_account, delete_source};

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

const APP_IDENTIFIER: &str = "org.ai.extractum";
const DB_FILENAME: &str = "extractum.db";

/// Before the sql plugin runs, remove stale migration records whose SQL has changed.
/// This allows us to update migration files without deleting the database.
async fn patch_migrations(db_path: &PathBuf) {
    use sqlx::SqlitePool;

    if !db_path.exists() {
        return;
    }

    let url = format!("sqlite:{}", db_path.to_string_lossy());
    if let Ok(pool) = SqlitePool::connect(&url).await {
        let expected_checksum = Sha384::digest(include_str!("../migrations/2.sql").as_bytes()).to_vec();
        let has_v3 = sqlx::query_scalar::<_, i64>(
            "SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version = 3)"
        )
        .fetch_one(&pool)
        .await
        .map(|exists| exists != 0)
        .unwrap_or(false);

        let v2_checksum = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT checksum FROM _sqlx_migrations WHERE version = 2"
        )
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();

        match v2_checksum {
            Some(checksum) if checksum != expected_checksum => {
                if has_v3 {
                    // Once later migrations are applied, deleting v2 leaves a gap that sqlx will not backfill.
                    // Update the metadata in place so startup validation passes without replaying schema changes.
                    let _ = sqlx::query(
                        "UPDATE _sqlx_migrations
                         SET description = ?, success = 1, checksum = ?
                         WHERE version = 2"
                    )
                    .bind("add is_member to sources")
                    .bind(&expected_checksum)
                    .execute(&pool)
                    .await;
                } else {
                    // Safe only before later migrations exist: let sqlx replay the no-op v2 with the new checksum.
                    let _ = sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 2")
                        .execute(&pool)
                        .await;
                }
            }
            None if has_v3 => {
                // Repair older upgraded databases that lost v2 metadata after the previous patch strategy.
                let _ = sqlx::query(
                    "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
                     VALUES (?, ?, 1, ?, 0)"
                )
                .bind(2_i64)
                .bind("add is_member to sources")
                .bind(&expected_checksum)
                .execute(&pool)
                .await;
            }
            _ => {}
        }

        pool.close().await;
    }
}

fn app_config_db_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(APP_IDENTIFIER).join(DB_FILENAME))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Some(db_path) = app_config_db_path() {
        tauri::async_runtime::block_on(patch_migrations(&db_path));
    }

    let migrations = vec![
        Migration {
            version: 1,
            description: "initialize storage",
            sql: include_str!("../migrations/1.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add is_member to sources",
            sql: include_str!("../migrations/2.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "add accounts table",
            sql: include_str!("../migrations/3.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "add last synced at to sources",
            sql: include_str!("../migrations/4.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "add analysis storage",
            sql: include_str!("../migrations/5.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "add analysis source groups",
            sql: include_str!("../migrations/6.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "add source group id to analysis runs",
            sql: include_str!("../migrations/7.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 8,
            description: "add analysis chat history",
            sql: include_str!("../migrations/8.sql"),
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .manage(TelegramState::new())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:extractum.db", migrations)
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
