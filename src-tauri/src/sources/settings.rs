use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};

const DEFAULT_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 500;
const MIN_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 50;
const MAX_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 5_000;
const DEFAULT_INITIAL_SYNC_DAY_LIMIT: i64 = 30;
const MIN_INITIAL_SYNC_DAY_LIMIT: i64 = 1;
const MAX_INITIAL_SYNC_DAY_LIMIT: i64 = 365;
const INITIAL_SYNC_MODE_SETTING_KEY: &str = "sync.initial.mode";
const INITIAL_SYNC_VALUE_SETTING_KEY: &str = "sync.initial.value";
pub(super) const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InitialSyncMode {
    RecentMessages,
    RecentDays,
}

impl InitialSyncMode {
    fn parse(value: &str) -> AppResult<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "recent_messages" => Ok(Self::RecentMessages),
            "recent_days" => Ok(Self::RecentDays),
            other => Err(AppError::validation(format!(
                "Unsupported initial sync mode '{other}'"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::RecentMessages => "recent_messages",
            Self::RecentDays => "recent_days",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SyncSettingsRecord {
    pub initial_sync_mode: InitialSyncMode,
    pub initial_sync_value: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSyncSettingsRequest {
    pub initial_sync_mode: InitialSyncMode,
    pub initial_sync_value: i64,
}

fn default_sync_settings() -> SyncSettingsRecord {
    SyncSettingsRecord {
        initial_sync_mode: InitialSyncMode::RecentMessages,
        initial_sync_value: DEFAULT_INITIAL_SYNC_MESSAGE_LIMIT,
    }
}

fn validate_sync_settings(
    initial_sync_mode: InitialSyncMode,
    initial_sync_value: i64,
) -> AppResult<SyncSettingsRecord> {
    let allowed_range = match initial_sync_mode {
        InitialSyncMode::RecentMessages => {
            MIN_INITIAL_SYNC_MESSAGE_LIMIT..=MAX_INITIAL_SYNC_MESSAGE_LIMIT
        }
        InitialSyncMode::RecentDays => MIN_INITIAL_SYNC_DAY_LIMIT..=MAX_INITIAL_SYNC_DAY_LIMIT,
    };

    if !allowed_range.contains(&initial_sync_value) {
        let (unit_label, min_value, max_value) = match initial_sync_mode {
            InitialSyncMode::RecentMessages => (
                "messages",
                MIN_INITIAL_SYNC_MESSAGE_LIMIT,
                MAX_INITIAL_SYNC_MESSAGE_LIMIT,
            ),
            InitialSyncMode::RecentDays => (
                "days",
                MIN_INITIAL_SYNC_DAY_LIMIT,
                MAX_INITIAL_SYNC_DAY_LIMIT,
            ),
        };
        return Err(AppError::validation(format!(
            "Initial sync value for {} must be between {} and {} {}",
            initial_sync_mode.as_str(),
            min_value,
            max_value,
            unit_label
        )));
    }

    Ok(SyncSettingsRecord {
        initial_sync_mode,
        initial_sync_value,
    })
}

pub(super) fn initial_sync_policy_label(settings: &SyncSettingsRecord) -> String {
    match settings.initial_sync_mode {
        InitialSyncMode::RecentMessages => {
            let unit = if settings.initial_sync_value == 1 {
                "message"
            } else {
                "messages"
            };
            format!("last {} {}", settings.initial_sync_value, unit)
        }
        InitialSyncMode::RecentDays => {
            let unit = if settings.initial_sync_value == 1 {
                "day"
            } else {
                "days"
            };
            format!("last {} {}", settings.initial_sync_value, unit)
        }
    }
}

async fn read_setting(pool: &sqlx::Pool<sqlx::Sqlite>, key: &str) -> AppResult<Option<String>> {
    sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))
}

async fn write_setting(pool: &sqlx::Pool<sqlx::Sqlite>, key: &str, value: &str) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO app_settings (key, value)
        VALUES (?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value
        "#,
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(())
}

pub(super) async fn load_sync_settings_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> AppResult<SyncSettingsRecord> {
    let default_settings = default_sync_settings();
    let mode = read_setting(pool, INITIAL_SYNC_MODE_SETTING_KEY)
        .await?
        .as_deref()
        .map(InitialSyncMode::parse)
        .transpose()?
        .unwrap_or(default_settings.initial_sync_mode);
    let value = read_setting(pool, INITIAL_SYNC_VALUE_SETTING_KEY)
        .await?
        .as_deref()
        .and_then(|stored| stored.trim().parse::<i64>().ok())
        .unwrap_or(match mode {
            InitialSyncMode::RecentMessages => DEFAULT_INITIAL_SYNC_MESSAGE_LIMIT,
            InitialSyncMode::RecentDays => DEFAULT_INITIAL_SYNC_DAY_LIMIT,
        });

    validate_sync_settings(mode, value)
}

async fn save_sync_settings_to_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    settings: &SyncSettingsRecord,
) -> AppResult<()> {
    write_setting(
        pool,
        INITIAL_SYNC_MODE_SETTING_KEY,
        settings.initial_sync_mode.as_str(),
    )
    .await?;
    write_setting(
        pool,
        INITIAL_SYNC_VALUE_SETTING_KEY,
        &settings.initial_sync_value.to_string(),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_sync_settings(handle: AppHandle) -> AppResult<SyncSettingsRecord> {
    let pool = get_pool(&handle).await?;
    load_sync_settings_from_pool(&pool).await
}

#[tauri::command]
pub async fn save_sync_settings(
    handle: AppHandle,
    settings: SaveSyncSettingsRequest,
) -> AppResult<SyncSettingsRecord> {
    let pool = get_pool(&handle).await?;
    let validated =
        validate_sync_settings(settings.initial_sync_mode, settings.initial_sync_value)?;
    save_sync_settings_to_pool(&pool, &validated).await?;
    Ok(validated)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .expect("create app_settings");
        pool
    }

    #[test]
    fn initial_sync_policy_label_formats_messages_and_days() {
        assert_eq!(
            initial_sync_policy_label(&SyncSettingsRecord {
                initial_sync_mode: InitialSyncMode::RecentMessages,
                initial_sync_value: 500,
            }),
            "last 500 messages"
        );
        assert_eq!(
            initial_sync_policy_label(&SyncSettingsRecord {
                initial_sync_mode: InitialSyncMode::RecentDays,
                initial_sync_value: 1,
            }),
            "last 1 day"
        );
    }

    #[test]
    fn validate_sync_settings_rejects_out_of_range_values() {
        let result = validate_sync_settings(InitialSyncMode::RecentDays, 0);
        assert!(result.is_err());

        let result = validate_sync_settings(InitialSyncMode::RecentMessages, 10_000);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_settings_default_when_app_settings_are_missing() {
        let pool = memory_pool().await;
        let loaded = load_sync_settings_from_pool(&pool)
            .await
            .expect("load default sync settings");

        assert_eq!(loaded, default_sync_settings());
    }

    #[tokio::test]
    async fn sync_settings_roundtrip_through_app_settings() {
        let pool = memory_pool().await;
        let expected = SyncSettingsRecord {
            initial_sync_mode: InitialSyncMode::RecentDays,
            initial_sync_value: 14,
        };

        save_sync_settings_to_pool(&pool, &expected)
            .await
            .expect("save sync settings");
        let loaded = load_sync_settings_from_pool(&pool)
            .await
            .expect("load sync settings");

        assert_eq!(loaded, expected);
    }
}
