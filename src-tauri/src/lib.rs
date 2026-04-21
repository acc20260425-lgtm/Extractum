// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri_plugin_sql::{Migration, MigrationKind};

mod telegram;
use telegram::{TelegramState, tg_init, tg_is_authenticated, tg_send_code, tg_sign_in, tg_logout};

mod sources;
use sources::{list_telegram_channels, add_telegram_source, list_sources, list_accounts, create_account, set_account_phone, delete_account};

#[tauri::command]
fn ping_db() -> String {
    "Rust: Database plugin is initialized and migrations should have run.".to_string()
}

/// Before the sql plugin runs, remove stale migration records whose SQL has changed.
/// This allows us to update migration files without deleting the database.
async fn patch_migrations(app: &tauri::AppHandle) {
    use sqlx::SqlitePool;
    use tauri::Manager;

    let app_dir = match app.path().app_config_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let db_path = app_dir.join("extractum.db");
    if !db_path.exists() {
        return;
    }

    let url = format!("sqlite:{}", db_path.to_string_lossy());
    if let Ok(pool) = SqlitePool::connect(&url).await {
        // Remove migration 2 so it gets re-applied with the new (no-op) SQL and checksum
        let _ = sqlx::query(
            "DELETE FROM _sqlx_migrations WHERE version = 2 AND description = 'add is_member to sources'"
        )
        .execute(&pool)
        .await;
        pool.close().await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
    ];

    tauri::Builder::default()
        .manage(TelegramState::new())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(patch_migrations(&handle));
            Ok(())
        })
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:extractum.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            ping_db,
            tg_init,
            tg_is_authenticated,
            tg_send_code,
            tg_sign_in,
            tg_logout,
            list_accounts,
            create_account,
            set_account_phone,
            delete_account,
            list_telegram_channels,
            add_telegram_source,
            list_sources
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
