// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri_plugin_sql::{Migration, MigrationKind};


#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod telegram;
use telegram::{TelegramState, tg_init, tg_is_authenticated, tg_send_code, tg_sign_in, tg_logout};

mod sources;
use sources::{list_telegram_channels, add_telegram_source, list_sources};

#[tauri::command]
fn ping_db() -> String {
    "Rust: Database plugin is initialized and migrations should have run.".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "initialize storage",
            sql: include_str!("../migrations/1.sql"),
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .manage(TelegramState::new()) // Register global state
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:extractum.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            greet,
            ping_db,
            tg_init,
            tg_is_authenticated,
            tg_send_code,
            tg_sign_in,
            tg_logout,
            list_telegram_channels,
            add_telegram_source,
            list_sources
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
