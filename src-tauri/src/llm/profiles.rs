use sqlx::{Pool, Sqlite};

use super::{LlmProfile, LlmProfilesState, ResolvedLlmProfile};
use super::{ProviderKind, DEFAULT_MODEL, DEFAULT_PROFILE_ID, DEFAULT_PROVIDER};

fn active_profile_key() -> &'static str {
    "llm.active_provider_profile"
}

fn profile_provider_key(profile_id: &str) -> String {
    format!("llm.profile.{profile_id}.provider")
}

fn profile_model_key(profile_id: &str) -> String {
    format!("llm.profile.{profile_id}.default_model")
}

fn profile_api_key(profile_id: &str) -> String {
    format!("llm.profile.{profile_id}.api_key")
}

async fn read_setting(pool: &Pool<Sqlite>, key: &str) -> Result<Option<String>, String> {
    sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
}

async fn write_setting(pool: &Pool<Sqlite>, key: &str, value: &str) -> Result<(), String> {
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
    .map_err(|e| e.to_string())?;

    Ok(())
}

async fn load_profile_from_pool(
    pool: &Pool<Sqlite>,
    profile_id: &str,
) -> Result<LlmProfile, String> {
    let provider = read_setting(pool, &profile_provider_key(profile_id))
        .await?
        .unwrap_or_else(|| DEFAULT_PROVIDER.to_string());
    let default_model = read_setting(pool, &profile_model_key(profile_id))
        .await?
        .unwrap_or_else(|| DEFAULT_MODEL.to_string());
    let api_key = read_setting(pool, &profile_api_key(profile_id))
        .await?
        .unwrap_or_default();

    Ok(LlmProfile {
        profile_id: profile_id.to_string(),
        provider,
        default_model,
        api_key,
    })
}

pub(super) async fn save_profile_to_pool(
    pool: &Pool<Sqlite>,
    profile_id: &str,
    provider: &str,
    default_model: &str,
    api_key: &str,
    set_active: bool,
) -> Result<(), String> {
    write_setting(pool, &profile_provider_key(profile_id), provider).await?;
    write_setting(pool, &profile_model_key(profile_id), default_model).await?;
    write_setting(pool, &profile_api_key(profile_id), api_key).await?;

    if set_active {
        write_setting(pool, active_profile_key(), profile_id).await?;
    }

    Ok(())
}

pub(super) async fn load_profiles_state_from_pool(
    pool: &Pool<Sqlite>,
) -> Result<LlmProfilesState, String> {
    let active_profile = read_setting(pool, active_profile_key())
        .await?
        .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string());
    let default_profile = load_profile_from_pool(pool, DEFAULT_PROFILE_ID).await?;

    Ok(LlmProfilesState {
        active_profile,
        default_profile,
    })
}

pub(super) async fn resolve_profile_from_pool(
    pool: &Pool<Sqlite>,
    requested_profile_id: Option<&str>,
) -> Result<ResolvedLlmProfile, String> {
    let profiles_state = load_profiles_state_from_pool(pool).await?;
    let profile_id = requested_profile_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&profiles_state.active_profile)
        .to_string();

    let profile = load_profile_from_pool(pool, &profile_id).await?;
    let provider = ProviderKind::parse(&profile.provider)?;

    Ok(ResolvedLlmProfile {
        profile_id,
        provider,
        default_model: profile.default_model,
        api_key: profile.api_key,
    })
}

pub(super) fn validate_profile_input(
    profile_id: Option<String>,
    provider: String,
    default_model: String,
) -> Result<(String, ProviderKind, String), String> {
    let profile_id = profile_id
        .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string())
        .trim()
        .to_string();
    if profile_id.is_empty() {
        return Err("Profile ID cannot be empty".to_string());
    }

    let provider_kind = ProviderKind::parse(&provider)?;
    let default_model = default_model.trim().to_string();
    if default_model.is_empty() {
        return Err("Default model cannot be empty".to_string());
    }

    Ok((profile_id, provider_kind, default_model))
}

#[cfg(test)]
mod tests {
    use super::{load_profiles_state_from_pool, resolve_profile_from_pool, save_profile_to_pool};

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

    #[tokio::test]
    async fn profile_settings_roundtrip_through_app_settings() {
        let pool = memory_pool().await;

        save_profile_to_pool(
            &pool,
            "default",
            "gemini",
            "gemini-2.5-flash",
            "test-key",
            true,
        )
        .await
        .expect("save profile");

        let state = load_profiles_state_from_pool(&pool)
            .await
            .expect("load state");
        assert_eq!(state.active_profile, "default");
        assert_eq!(state.default_profile.provider, "gemini");
        assert_eq!(state.default_profile.default_model, "gemini-2.5-flash");
        assert_eq!(state.default_profile.api_key, "test-key");
    }

    #[tokio::test]
    async fn active_profile_resolution_uses_saved_selection() {
        let pool = memory_pool().await;

        save_profile_to_pool(&pool, "alt", "gemini", "gemini-2.0-flash", "alt-key", true)
            .await
            .expect("save alt profile");

        let resolved = resolve_profile_from_pool(&pool, None)
            .await
            .expect("resolve active");
        assert_eq!(resolved.profile_id, "alt");
        assert_eq!(resolved.default_model, "gemini-2.0-flash");
        assert_eq!(resolved.api_key, "alt-key");
    }
}
