use sqlx::{Pool, Sqlite};

use crate::error::{AppError, AppResult};

use super::{
    default_base_url_for_provider, normalize_base_url, ProviderKind, DEFAULT_MODEL,
    DEFAULT_PROFILE_ID, DEFAULT_PROVIDER,
};
use super::{LlmProfile, LlmProfilesState, ResolvedLlmProfile};

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

fn profile_base_url_key(profile_id: &str) -> String {
    format!("llm.profile.{profile_id}.base_url")
}

fn profile_provider_key_prefix() -> &'static str {
    "llm.profile."
}

fn profile_provider_key_suffix() -> &'static str {
    ".provider"
}

fn normalize_profile_id(raw_profile_id: &str) -> AppResult<String> {
    let profile_id = raw_profile_id.trim().to_ascii_lowercase();
    if profile_id.is_empty() {
        return Err(AppError::validation("Profile ID cannot be empty"));
    }

    if !profile_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(AppError::validation(
            "Profile ID can only contain ASCII letters, numbers, dashes, and underscores",
        ));
    }

    Ok(profile_id)
}

fn profile_id_from_provider_key(key: &str) -> Option<String> {
    key.strip_prefix(profile_provider_key_prefix())
        .and_then(|value| value.strip_suffix(profile_provider_key_suffix()))
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

async fn read_setting(pool: &Pool<Sqlite>, key: &str) -> AppResult<Option<String>> {
    sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)
}

async fn write_setting(pool: &Pool<Sqlite>, key: &str, value: &str) -> AppResult<()> {
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

async fn list_profile_ids_from_pool(pool: &Pool<Sqlite>) -> AppResult<Vec<String>> {
    let like_pattern = format!(
        "{}%{}",
        profile_provider_key_prefix(),
        profile_provider_key_suffix()
    );
    let keys = sqlx::query_scalar::<_, String>(
        "SELECT key FROM app_settings WHERE key LIKE ? ORDER BY key",
    )
    .bind(like_pattern)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut profile_ids = keys
        .into_iter()
        .filter_map(|key| profile_id_from_provider_key(&key))
        .collect::<Vec<_>>();

    if !profile_ids
        .iter()
        .any(|profile_id| profile_id == DEFAULT_PROFILE_ID)
    {
        profile_ids.push(DEFAULT_PROFILE_ID.to_string());
    }

    profile_ids.sort();
    profile_ids.dedup();
    Ok(profile_ids)
}

async fn load_profile_from_pool(
    pool: &Pool<Sqlite>,
    profile_id: &str,
) -> AppResult<LlmProfile> {
    let profile_id = normalize_profile_id(profile_id)?;
    let provider = read_setting(pool, &profile_provider_key(&profile_id))
        .await?
        .unwrap_or_else(|| DEFAULT_PROVIDER.to_string());
    let default_model = read_setting(pool, &profile_model_key(&profile_id))
        .await?
        .unwrap_or_else(|| DEFAULT_MODEL.to_string());
    let api_key = read_setting(pool, &profile_api_key(&profile_id))
        .await?
        .unwrap_or_default();
    let base_url = read_setting(pool, &profile_base_url_key(&profile_id))
        .await?
        .unwrap_or_else(|| default_base_url_for_provider(&provider).to_string());

    Ok(LlmProfile {
        profile_id,
        provider,
        default_model,
        api_key,
        base_url,
    })
}

pub(super) async fn save_profile_to_pool(
    pool: &Pool<Sqlite>,
    profile_id: &str,
    provider: &str,
    default_model: &str,
    api_key: &str,
    base_url: &str,
    set_active: bool,
) -> AppResult<()> {
    let profile_id = normalize_profile_id(profile_id)?;

    write_setting(pool, &profile_provider_key(&profile_id), provider).await?;
    write_setting(pool, &profile_model_key(&profile_id), default_model).await?;
    write_setting(pool, &profile_api_key(&profile_id), api_key).await?;
    write_setting(pool, &profile_base_url_key(&profile_id), base_url).await?;

    if set_active {
        write_setting(pool, active_profile_key(), &profile_id).await?;
    }

    Ok(())
}

pub(super) async fn load_profiles_state_from_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<LlmProfilesState> {
    let active_profile = read_setting(pool, active_profile_key())
        .await?
        .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string());
    let active_profile = normalize_profile_id(&active_profile)?;

    let mut profile_ids = list_profile_ids_from_pool(pool).await?;
    if !profile_ids
        .iter()
        .any(|profile_id| profile_id == &active_profile)
    {
        profile_ids.push(active_profile.clone());
    }

    profile_ids.sort();
    profile_ids.dedup();

    let mut profiles = Vec::with_capacity(profile_ids.len());
    for profile_id in profile_ids {
        profiles.push(load_profile_from_pool(pool, &profile_id).await?);
    }

    Ok(LlmProfilesState {
        active_profile,
        profiles,
    })
}

pub(super) async fn resolve_profile_from_pool(
    pool: &Pool<Sqlite>,
    requested_profile_id: Option<&str>,
) -> AppResult<ResolvedLlmProfile> {
    let profiles_state = load_profiles_state_from_pool(pool).await?;
    let profile_id = requested_profile_id
        .map(normalize_profile_id)
        .transpose()?
        .unwrap_or_else(|| profiles_state.active_profile.clone());

    if !profiles_state
        .profiles
        .iter()
        .any(|profile| profile.profile_id == profile_id)
    {
        return Err(AppError::not_found(format!(
            "Profile '{profile_id}' was not found"
        )));
    }

    let profile = load_profile_from_pool(pool, &profile_id).await?;
    let provider = ProviderKind::parse(&profile.provider)?;

    Ok(ResolvedLlmProfile {
        profile_id,
        provider,
        default_model: profile.default_model,
        api_key: profile.api_key,
        base_url: profile.base_url,
    })
}

pub(super) async fn set_active_profile_in_pool(
    pool: &Pool<Sqlite>,
    profile_id: &str,
) -> AppResult<()> {
    let profile_id = normalize_profile_id(profile_id)?;
    let profile_ids = list_profile_ids_from_pool(pool).await?;
    if !profile_ids
        .iter()
        .any(|existing_id| existing_id == &profile_id)
    {
        return Err(AppError::not_found(format!(
            "Profile '{profile_id}' was not found"
        )));
    }

    write_setting(pool, active_profile_key(), &profile_id).await
}

pub(super) fn validate_profile_id(profile_id: &str) -> AppResult<String> {
    normalize_profile_id(profile_id)
}

pub(super) fn validate_profile_input(
    profile_id: Option<String>,
    provider: String,
    default_model: String,
    base_url: Option<String>,
) -> AppResult<(String, ProviderKind, String, String)> {
    let profile_id =
        normalize_profile_id(&profile_id.unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string()))?;
    let provider_kind = ProviderKind::parse(&provider)?;
    let default_model = default_model.trim().to_string();
    if default_model.is_empty() {
        return Err(AppError::validation("Default model cannot be empty"));
    }

    let base_url = normalize_base_url(provider_kind, base_url.as_deref())?;

    Ok((profile_id, provider_kind, default_model, base_url))
}

#[cfg(test)]
mod tests {
    use super::{
        load_profiles_state_from_pool, resolve_profile_from_pool, save_profile_to_pool,
        set_active_profile_in_pool, validate_profile_id,
    };
    use crate::error::AppErrorKind;

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
            "",
            true,
        )
        .await
        .expect("save profile");

        let state = load_profiles_state_from_pool(&pool)
            .await
            .expect("load state");
        assert_eq!(state.active_profile, "default");
        assert_eq!(state.profiles.len(), 1);
        assert_eq!(state.profiles[0].provider, "gemini");
        assert_eq!(state.profiles[0].default_model, "gemini-2.5-flash");
        assert_eq!(state.profiles[0].api_key, "test-key");
        assert_eq!(state.profiles[0].base_url, "");
    }

    #[tokio::test]
    async fn active_profile_resolution_uses_saved_selection() {
        let pool = memory_pool().await;

        save_profile_to_pool(
            &pool,
            "alt",
            "gemini",
            "gemini-2.0-flash",
            "alt-key",
            "",
            true,
        )
        .await
        .expect("save alt profile");

        let resolved = resolve_profile_from_pool(&pool, None)
            .await
            .expect("resolve active");
        assert_eq!(resolved.profile_id, "alt");
        assert_eq!(resolved.default_model, "gemini-2.0-flash");
        assert_eq!(resolved.api_key, "alt-key");
        assert_eq!(resolved.base_url, "");
    }

    #[tokio::test]
    async fn profile_state_lists_multiple_saved_profiles() {
        let pool = memory_pool().await;

        save_profile_to_pool(
            &pool,
            "default",
            "gemini",
            "gemini-2.5-flash",
            "default-key",
            "",
            false,
        )
        .await
        .expect("save default profile");
        save_profile_to_pool(
            &pool,
            "omni_local",
            "omniroute",
            "if/kimi-k2-thinking",
            "omni-key",
            "http://localhost:3010/v1",
            false,
        )
        .await
        .expect("save second profile");
        set_active_profile_in_pool(&pool, "omni_local")
            .await
            .expect("set active profile");

        let state = load_profiles_state_from_pool(&pool)
            .await
            .expect("load profiles state");

        assert_eq!(state.active_profile, "omni_local");
        assert_eq!(state.profiles.len(), 2);
        assert_eq!(state.profiles[1].profile_id, "omni_local");
        assert_eq!(state.profiles[1].base_url, "http://localhost:3010/v1");
    }

    #[test]
    fn validate_profile_id_rejects_invalid_characters() {
        let error = validate_profile_id("prod west").expect_err("invalid profile id");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(
            error.message,
            "Profile ID can only contain ASCII letters, numbers, dashes, and underscores"
        );
    }

    #[tokio::test]
    async fn set_active_profile_returns_typed_not_found_error() {
        let pool = memory_pool().await;

        let error = set_active_profile_in_pool(&pool, "missing")
            .await
            .expect_err("missing profile");

        assert_eq!(error.kind, AppErrorKind::NotFound);
        assert_eq!(error.message, "Profile 'missing' was not found");
    }
}
