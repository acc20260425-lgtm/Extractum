use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::secret_store::SecretStoreState;

use super::cookies;

const AUTH_ENABLED_KEY: &str = "youtube.auth.enabled";
const PREFERRED_CAPTIONS_LANGUAGE_KEY: &str = "youtube.captions.preferred_language";
const DELAY_BETWEEN_REQUESTS_MS_KEY: &str = "youtube.sync.delay_between_requests_ms";
const MAX_PARALLEL_VIDEO_SYNCS_KEY: &str = "youtube.sync.max_parallel_video_syncs";
const MAX_PARALLEL_COMMENT_SYNCS_KEY: &str = "youtube.sync.max_parallel_comment_syncs";
const PAUSE_ON_AUTH_CHALLENGE_KEY: &str = "youtube.sync.pause_on_auth_challenge";
const DAILY_SOFT_LIMIT_KEY: &str = "youtube.sync.daily_soft_limit";
const RETRY_BACKOFF_MS_KEY: &str = "youtube.sync.retry_backoff_ms";
const STOP_AFTER_CONSECUTIVE_FAILURES_KEY: &str = "youtube.sync.stop_after_consecutive_failures";

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSettingsDto {
    pub auth_enabled: bool,
    pub preferred_captions_language: String,
    pub delay_between_requests_ms: i64,
    pub max_parallel_video_syncs: i64,
    pub max_parallel_comment_syncs: i64,
    pub pause_on_auth_challenge: bool,
    pub daily_soft_limit: i64,
    pub retry_backoff_ms: i64,
    pub stop_after_consecutive_failures: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeAuthStatusDto {
    pub enabled: bool,
    pub has_cookies: bool,
    pub message: String,
}

pub(crate) fn default_youtube_settings() -> YoutubeSettingsDto {
    YoutubeSettingsDto {
        auth_enabled: false,
        preferred_captions_language: "original".to_string(),
        delay_between_requests_ms: 1_000,
        max_parallel_video_syncs: 1,
        max_parallel_comment_syncs: 1,
        pause_on_auth_challenge: true,
        daily_soft_limit: 0,
        retry_backoff_ms: 3_000,
        stop_after_consecutive_failures: 3,
    }
}

pub(crate) fn validate_youtube_settings(
    settings: YoutubeSettingsDto,
) -> AppResult<YoutubeSettingsDto> {
    Ok(YoutubeSettingsDto {
        auth_enabled: settings.auth_enabled,
        preferred_captions_language: validate_preferred_captions_language(
            &settings.preferred_captions_language,
        )?,
        delay_between_requests_ms: validate_range(
            settings.delay_between_requests_ms,
            0,
            60_000,
            "Delay between requests",
        )?,
        max_parallel_video_syncs: validate_range(
            settings.max_parallel_video_syncs,
            1,
            4,
            "Max parallel video syncs",
        )?,
        max_parallel_comment_syncs: validate_range(
            settings.max_parallel_comment_syncs,
            1,
            2,
            "Max parallel comment syncs",
        )?,
        pause_on_auth_challenge: settings.pause_on_auth_challenge,
        daily_soft_limit: validate_range(settings.daily_soft_limit, 0, 10_000, "Daily soft limit")?,
        retry_backoff_ms: validate_range(settings.retry_backoff_ms, 0, 300_000, "Retry backoff")?,
        stop_after_consecutive_failures: validate_range(
            settings.stop_after_consecutive_failures,
            1,
            50,
            "Stop after consecutive failures",
        )?,
    })
}

fn validate_range(value: i64, min: i64, max: i64, label: &str) -> AppResult<i64> {
    if value < min || value > max {
        return Err(AppError::validation(format!(
            "{label} must be between {min} and {max}"
        )));
    }
    Ok(value)
}

fn validate_preferred_captions_language(value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("original") {
        return Ok("original".to_string());
    }
    let valid = (2..=32).contains(&trimmed.len())
        && trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !valid {
        return Err(AppError::validation(
            "Preferred captions language must be 'original' or a 2-32 character language code",
        ));
    }
    Ok(trimmed.to_ascii_lowercase())
}

async fn read_setting(pool: &sqlx::Pool<sqlx::Sqlite>, key: &str) -> AppResult<Option<String>> {
    sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)
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
    .map_err(AppError::database)?;

    Ok(())
}

fn parse_bool_setting(value: Option<String>, default: bool, key: &str) -> AppResult<bool> {
    let Some(value) = value else {
        return Ok(default);
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(AppError::validation(format!(
            "Invalid YouTube setting {key}: expected true or false"
        ))),
    }
}

fn parse_i64_setting(value: Option<String>, default: i64, key: &str) -> AppResult<i64> {
    value
        .map(|stored| {
            stored.trim().parse::<i64>().map_err(|_| {
                AppError::validation(format!("Invalid YouTube setting {key}: expected integer"))
            })
        })
        .transpose()
        .map(|parsed| parsed.unwrap_or(default))
}

pub(crate) async fn load_youtube_settings_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> AppResult<YoutubeSettingsDto> {
    let defaults = default_youtube_settings();
    let settings = YoutubeSettingsDto {
        auth_enabled: parse_bool_setting(
            read_setting(pool, AUTH_ENABLED_KEY).await?,
            defaults.auth_enabled,
            AUTH_ENABLED_KEY,
        )?,
        preferred_captions_language: read_setting(pool, PREFERRED_CAPTIONS_LANGUAGE_KEY)
            .await?
            .unwrap_or(defaults.preferred_captions_language),
        delay_between_requests_ms: parse_i64_setting(
            read_setting(pool, DELAY_BETWEEN_REQUESTS_MS_KEY).await?,
            defaults.delay_between_requests_ms,
            DELAY_BETWEEN_REQUESTS_MS_KEY,
        )?,
        max_parallel_video_syncs: parse_i64_setting(
            read_setting(pool, MAX_PARALLEL_VIDEO_SYNCS_KEY).await?,
            defaults.max_parallel_video_syncs,
            MAX_PARALLEL_VIDEO_SYNCS_KEY,
        )?,
        max_parallel_comment_syncs: parse_i64_setting(
            read_setting(pool, MAX_PARALLEL_COMMENT_SYNCS_KEY).await?,
            defaults.max_parallel_comment_syncs,
            MAX_PARALLEL_COMMENT_SYNCS_KEY,
        )?,
        pause_on_auth_challenge: parse_bool_setting(
            read_setting(pool, PAUSE_ON_AUTH_CHALLENGE_KEY).await?,
            defaults.pause_on_auth_challenge,
            PAUSE_ON_AUTH_CHALLENGE_KEY,
        )?,
        daily_soft_limit: parse_i64_setting(
            read_setting(pool, DAILY_SOFT_LIMIT_KEY).await?,
            defaults.daily_soft_limit,
            DAILY_SOFT_LIMIT_KEY,
        )?,
        retry_backoff_ms: parse_i64_setting(
            read_setting(pool, RETRY_BACKOFF_MS_KEY).await?,
            defaults.retry_backoff_ms,
            RETRY_BACKOFF_MS_KEY,
        )?,
        stop_after_consecutive_failures: parse_i64_setting(
            read_setting(pool, STOP_AFTER_CONSECUTIVE_FAILURES_KEY).await?,
            defaults.stop_after_consecutive_failures,
            STOP_AFTER_CONSECUTIVE_FAILURES_KEY,
        )?,
    };
    validate_youtube_settings(settings)
}

async fn save_youtube_settings_to_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    settings: &YoutubeSettingsDto,
) -> AppResult<YoutubeSettingsDto> {
    let validated = validate_youtube_settings(settings.clone())?;

    write_setting(pool, AUTH_ENABLED_KEY, bool_wire(validated.auth_enabled)).await?;
    write_setting(
        pool,
        PREFERRED_CAPTIONS_LANGUAGE_KEY,
        &validated.preferred_captions_language,
    )
    .await?;
    write_setting(
        pool,
        DELAY_BETWEEN_REQUESTS_MS_KEY,
        &validated.delay_between_requests_ms.to_string(),
    )
    .await?;
    write_setting(
        pool,
        MAX_PARALLEL_VIDEO_SYNCS_KEY,
        &validated.max_parallel_video_syncs.to_string(),
    )
    .await?;
    write_setting(
        pool,
        MAX_PARALLEL_COMMENT_SYNCS_KEY,
        &validated.max_parallel_comment_syncs.to_string(),
    )
    .await?;
    write_setting(
        pool,
        PAUSE_ON_AUTH_CHALLENGE_KEY,
        bool_wire(validated.pause_on_auth_challenge),
    )
    .await?;
    write_setting(
        pool,
        DAILY_SOFT_LIMIT_KEY,
        &validated.daily_soft_limit.to_string(),
    )
    .await?;
    write_setting(
        pool,
        RETRY_BACKOFF_MS_KEY,
        &validated.retry_backoff_ms.to_string(),
    )
    .await?;
    write_setting(
        pool,
        STOP_AFTER_CONSECUTIVE_FAILURES_KEY,
        &validated.stop_after_consecutive_failures.to_string(),
    )
    .await?;

    Ok(validated)
}

pub(crate) async fn youtube_auth_status_from_state(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secrets: &SecretStoreState,
) -> AppResult<YoutubeAuthStatusDto> {
    let settings = load_youtube_settings_from_pool(pool).await?;
    let has_cookies = cookies::read_youtube_cookies(secrets).await?.is_some();
    Ok(YoutubeAuthStatusDto {
        enabled: settings.auth_enabled,
        has_cookies,
        message: auth_status_message(settings.auth_enabled, has_cookies).to_string(),
    })
}

pub(crate) async fn load_youtube_auth_cookies_from_state(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secrets: &SecretStoreState,
) -> AppResult<Option<String>> {
    let settings = load_youtube_settings_from_pool(pool).await?;
    if !settings.auth_enabled {
        return Ok(None);
    }
    cookies::read_youtube_cookies(secrets).await
}

pub(crate) async fn save_youtube_cookies_to_state(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secrets: &SecretStoreState,
    cookie_text: String,
) -> AppResult<YoutubeAuthStatusDto> {
    cookies::save_youtube_cookies(secrets, cookie_text).await?;
    write_setting(pool, AUTH_ENABLED_KEY, "true").await?;
    youtube_auth_status_from_state(pool, secrets).await
}

pub(crate) async fn clear_youtube_auth_in_state(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secrets: &SecretStoreState,
) -> AppResult<YoutubeAuthStatusDto> {
    cookies::clear_youtube_cookies(secrets).await?;
    write_setting(pool, AUTH_ENABLED_KEY, "false").await?;
    youtube_auth_status_from_state(pool, secrets).await
}

fn auth_status_message(enabled: bool, has_cookies: bool) -> &'static str {
    match (enabled, has_cookies) {
        (false, _) => "Auth disabled",
        (true, true) => "Cookies stored",
        (true, false) => "No cookies configured",
    }
}

fn bool_wire(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

#[tauri::command]
pub async fn get_youtube_settings(handle: AppHandle) -> AppResult<YoutubeSettingsDto> {
    let pool = get_pool(&handle).await?;
    load_youtube_settings_from_pool(&pool).await
}

#[tauri::command]
pub async fn save_youtube_settings(
    handle: AppHandle,
    settings: YoutubeSettingsDto,
) -> AppResult<YoutubeSettingsDto> {
    let pool = get_pool(&handle).await?;
    save_youtube_settings_to_pool(&pool, &settings).await
}

#[tauri::command]
pub async fn get_youtube_auth_status(
    handle: AppHandle,
    secrets: tauri::State<'_, SecretStoreState>,
) -> AppResult<YoutubeAuthStatusDto> {
    let pool = get_pool(&handle).await?;
    youtube_auth_status_from_state(&pool, &secrets).await
}

#[tauri::command]
pub async fn save_youtube_cookies(
    handle: AppHandle,
    secrets: tauri::State<'_, SecretStoreState>,
    cookies: String,
) -> AppResult<YoutubeAuthStatusDto> {
    let pool = get_pool(&handle).await?;
    save_youtube_cookies_to_state(&pool, &secrets, cookies).await
}

#[tauri::command]
pub async fn clear_youtube_auth(
    handle: AppHandle,
    secrets: tauri::State<'_, SecretStoreState>,
) -> AppResult<YoutubeAuthStatusDto> {
    let pool = get_pool(&handle).await?;
    clear_youtube_auth_in_state(&pool, &secrets).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::secret_store::{tests::InMemorySecretStore, SecretStoreState};
    use crate::sources::test_support::memory_pool;

    use super::{
        clear_youtube_auth_in_state, default_youtube_settings,
        load_youtube_auth_cookies_from_state, load_youtube_settings_from_pool,
        save_youtube_cookies_to_state, save_youtube_settings_to_pool, validate_youtube_settings,
    };

    #[tokio::test]
    async fn youtube_settings_default_when_app_settings_are_missing() {
        let pool = memory_pool().await;
        let settings = load_youtube_settings_from_pool(&pool)
            .await
            .expect("load defaults");

        assert_eq!(settings.delay_between_requests_ms, 1_000);
        assert_eq!(settings.daily_soft_limit, 0);
        assert!(!settings.auth_enabled);
    }

    #[test]
    fn validate_youtube_settings_rejects_out_of_range_values() {
        let mut settings = default_youtube_settings();
        settings.delay_between_requests_ms = -1;
        assert!(validate_youtube_settings(settings).is_err());

        let mut settings = default_youtube_settings();
        settings.max_parallel_video_syncs = 100;
        assert!(validate_youtube_settings(settings).is_err());
    }

    #[test]
    fn validate_youtube_settings_normalizes_preferred_captions_language() {
        let mut settings = default_youtube_settings();
        settings.preferred_captions_language = " EN-us ".to_string();

        let validated = validate_youtube_settings(settings).expect("validate settings");

        assert_eq!(validated.preferred_captions_language, "en-us");
    }

    #[test]
    fn youtube_settings_serializes_with_camel_case_keys() {
        let value =
            serde_json::to_value(default_youtube_settings()).expect("serialize youtube settings");

        assert!(value.get("authEnabled").is_some());
        assert!(value.get("preferredCaptionsLanguage").is_some());
        assert!(value.get("delayBetweenRequestsMs").is_some());
        assert!(value.get("maxParallelVideoSyncs").is_some());
        assert!(value.get("maxParallelCommentSyncs").is_some());
        assert!(value.get("pauseOnAuthChallenge").is_some());
        assert!(value.get("dailySoftLimit").is_some());
        assert!(value.get("retryBackoffMs").is_some());
        assert!(value.get("stopAfterConsecutiveFailures").is_some());
        assert!(value.get("auth_enabled").is_none());
    }

    #[tokio::test]
    async fn youtube_settings_roundtrip_through_app_settings() {
        let pool = memory_pool().await;
        let mut settings = default_youtube_settings();
        settings.auth_enabled = true;
        settings.preferred_captions_language = "EN".to_string();
        settings.delay_between_requests_ms = 500;
        settings.max_parallel_video_syncs = 2;
        settings.daily_soft_limit = 200;

        let saved = save_youtube_settings_to_pool(&pool, &settings)
            .await
            .expect("save settings");
        let loaded = load_youtube_settings_from_pool(&pool)
            .await
            .expect("load settings");

        assert_eq!(saved.preferred_captions_language, "en");
        assert_eq!(loaded, saved);
    }

    #[tokio::test]
    async fn invalid_youtube_settings_do_not_write_partial_values() {
        let pool = memory_pool().await;
        let mut valid = default_youtube_settings();
        valid.delay_between_requests_ms = 2_000;
        save_youtube_settings_to_pool(&pool, &valid)
            .await
            .expect("save valid settings");

        let mut invalid = valid.clone();
        invalid.delay_between_requests_ms = 3_000;
        invalid.max_parallel_video_syncs = 100;

        save_youtube_settings_to_pool(&pool, &invalid)
            .await
            .expect_err("invalid settings should fail");
        let loaded = load_youtube_settings_from_pool(&pool)
            .await
            .expect("load settings");

        assert_eq!(loaded.delay_between_requests_ms, 2_000);
        assert_eq!(loaded.max_parallel_video_syncs, 1);
    }

    #[tokio::test]
    async fn invalid_stored_settings_return_validation_error_with_key() {
        let pool = memory_pool().await;
        sqlx::query("INSERT INTO app_settings (key, value) VALUES (?, ?)")
            .bind("youtube.auth.enabled")
            .bind("sometimes")
            .execute(&pool)
            .await
            .expect("insert invalid setting");

        let err = load_youtube_settings_from_pool(&pool)
            .await
            .expect_err("invalid stored setting should fail");

        assert!(err.message.contains("youtube.auth.enabled"));
    }

    #[tokio::test]
    async fn saving_cookies_enables_auth_and_clear_disables_it() {
        let pool = memory_pool().await;
        let store = Arc::new(InMemorySecretStore::new());
        let secrets = SecretStoreState::new(store);
        let cookies = ".youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n".to_string();

        let saved = save_youtube_cookies_to_state(&pool, &secrets, cookies)
            .await
            .expect("save cookies");
        assert!(saved.enabled);
        assert!(saved.has_cookies);
        assert_eq!(saved.message, "Cookies stored");

        let cleared = clear_youtube_auth_in_state(&pool, &secrets)
            .await
            .expect("clear auth");
        assert!(!cleared.enabled);
        assert!(!cleared.has_cookies);
        assert_eq!(cleared.message, "Auth disabled");
    }

    #[tokio::test]
    async fn auth_cookies_load_only_when_auth_is_enabled() {
        let pool = memory_pool().await;
        let store = Arc::new(InMemorySecretStore::new());
        let secrets = SecretStoreState::new(store);
        let cookies = ".youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n".to_string();

        save_youtube_cookies_to_state(&pool, &secrets, cookies.clone())
            .await
            .expect("save cookies");
        assert_eq!(
            load_youtube_auth_cookies_from_state(&pool, &secrets)
                .await
                .expect("load enabled cookies"),
            Some(cookies)
        );

        let mut settings = load_youtube_settings_from_pool(&pool)
            .await
            .expect("load settings");
        settings.auth_enabled = false;
        save_youtube_settings_to_pool(&pool, &settings)
            .await
            .expect("disable auth");

        assert_eq!(
            load_youtube_auth_cookies_from_state(&pool, &secrets)
                .await
                .expect("load disabled cookies"),
            None
        );
    }
}
