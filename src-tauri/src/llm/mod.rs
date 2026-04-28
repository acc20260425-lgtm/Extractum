use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::time::{timeout, Duration};

use crate::db::get_pool;
use crate::error::AppResult;

mod gemini;
mod openai_compat;
mod profiles;
mod runner;
mod streaming;
mod types;

use gemini::list_gemini_models;
use openai_compat::{list_openai_compat_models, OpenAiCompatProviderConfig};
use profiles::{
    load_profiles_state_from_pool, resolve_profile_from_pool, save_profile_to_pool,
    set_active_profile_in_pool, validate_profile_id, validate_profile_input,
};
pub(crate) use runner::{
    resolve_effective_model, run_llm_collect_with_profile, run_llm_stream_with_profile,
    validate_request,
};
pub use types::{
    LlmChatRequest, LlmMessage, LlmProfile, LlmProfilesState, LlmProviderModel, LlmStreamEvent,
    LlmUsage,
};
pub(crate) use types::{LlmCompletion, ResolvedLlmProfile};

const LLM_RESPONSE_EVENT: &str = "llm://response";
const DEFAULT_PROFILE_ID: &str = "default";
const DEFAULT_PROVIDER: &str = "gemini";
const DEFAULT_MODEL: &str = "gemini-2.5-flash";
const DEFAULT_OPENAI_COMPAT_BASE_URL: &str = "http://localhost:20128/v1";
const GEMINI_MODELS_TIMEOUT_SECS: u64 = 30;
const OPENAI_COMPAT_MODELS_TIMEOUT_SECS: u64 = 30;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProviderKind {
    Gemini,
    OmniRoute,
}

impl ProviderKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Gemini => DEFAULT_PROVIDER,
            Self::OmniRoute => "omniroute",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            DEFAULT_PROVIDER => Ok(Self::Gemini),
            "omniroute" => Ok(Self::OmniRoute),
            other => Err(format!("Unsupported provider '{other}'")),
        }
    }

    pub(super) fn display_name(self) -> &'static str {
        match self {
            Self::Gemini => "Gemini",
            Self::OmniRoute => "OpenAI-compatible",
        }
    }
}

fn default_base_url_for_provider_kind(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Gemini => "",
        ProviderKind::OmniRoute => DEFAULT_OPENAI_COMPAT_BASE_URL,
    }
}

fn default_base_url_for_provider(provider: &str) -> &'static str {
    ProviderKind::parse(provider)
        .map(default_base_url_for_provider_kind)
        .unwrap_or("")
}

fn normalize_base_url(provider: ProviderKind, base_url: Option<&str>) -> Result<String, String> {
    match provider {
        ProviderKind::Gemini => Ok(String::new()),
        ProviderKind::OmniRoute => {
            let candidate = base_url
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(DEFAULT_OPENAI_COMPAT_BASE_URL);
            let parsed = reqwest::Url::parse(candidate)
                .map_err(|_| format!("Invalid base URL '{candidate}'"))?;
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err("Base URL must use http or https".to_string());
            }

            Ok(parsed.as_str().trim_end_matches('/').to_string())
        }
    }
}

fn emit_response_event(handle: &AppHandle, event: &LlmStreamEvent) {
    let _ = handle.emit(LLM_RESPONSE_EVENT, event);
}

pub(crate) async fn resolve_profile_for_backend(
    handle: &AppHandle,
    requested_profile_id: Option<&str>,
) -> Result<ResolvedLlmProfile, String> {
    let pool = get_pool(handle).await?;
    resolve_profile_from_pool(&pool, requested_profile_id).await
}

#[tauri::command]
pub async fn get_llm_profiles(handle: AppHandle) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    Ok(load_profiles_state_from_pool(&pool).await?)
}

#[tauri::command]
pub async fn save_llm_profile(
    handle: AppHandle,
    profile_id: Option<String>,
    provider: String,
    default_model: String,
    api_key: String,
    base_url: Option<String>,
    set_active: Option<bool>,
) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    let (profile_id, provider_kind, default_model, base_url) =
        validate_profile_input(profile_id, provider, default_model, base_url)?;
    let set_active = set_active.unwrap_or(false);

    save_profile_to_pool(
        &pool,
        &profile_id,
        provider_kind.as_str(),
        &default_model,
        api_key.trim(),
        &base_url,
        set_active,
    )
    .await?;

    Ok(load_profiles_state_from_pool(&pool).await?)
}

#[tauri::command]
pub async fn set_active_llm_profile(
    handle: AppHandle,
    profile_id: String,
) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    let profile_id = validate_profile_id(&profile_id)?;
    set_active_profile_in_pool(&pool, &profile_id).await?;
    Ok(load_profiles_state_from_pool(&pool).await?)
}

#[tauri::command]
pub async fn list_llm_provider_models(
    handle: AppHandle,
    provider: String,
    profile_id: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
) -> AppResult<Vec<LlmProviderModel>> {
    let provider_kind = ProviderKind::parse(&provider)?;
    let configured_key = api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let configured_base_url = base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    let saved_profile = if configured_key.is_none() || configured_base_url.is_none() {
        let pool = get_pool(&handle).await?;
        Some(resolve_profile_from_pool(&pool, profile_id.as_deref()).await?)
    } else {
        None
    };
    let matching_saved_profile = saved_profile
        .as_ref()
        .filter(|profile| profile.provider == provider_kind);

    let api_key = if let Some(key) = configured_key {
        key
    } else {
        matching_saved_profile
            .map(|profile| profile.api_key.clone())
            .unwrap_or_default()
    };
    let base_url = if let Some(url) = configured_base_url {
        normalize_base_url(provider_kind, Some(url.as_str()))?
    } else if let Some(profile) = matching_saved_profile {
        profile.base_url.clone()
    } else {
        normalize_base_url(provider_kind, None)?
    };

    let timeout_secs = match provider_kind {
        ProviderKind::Gemini => GEMINI_MODELS_TIMEOUT_SECS,
        ProviderKind::OmniRoute => OPENAI_COMPAT_MODELS_TIMEOUT_SECS,
    };
    let openai_compat_config = OpenAiCompatProviderConfig {
        provider: provider_kind,
        base_url,
    };

    let result = timeout(Duration::from_secs(timeout_secs), async move {
        match provider_kind {
            ProviderKind::Gemini => list_gemini_models(&api_key).await,
            ProviderKind::OmniRoute => {
                list_openai_compat_models(&api_key, &openai_compat_config).await
            }
        }
    })
    .await;

    match result {
        Ok(models) => Ok(models?),
        Err(_) => Err(format!(
            "Loading {} models timed out after {timeout_secs} seconds",
            provider_kind.display_name()
        )
        .into()),
    }
}

#[tauri::command]
pub async fn ask_llm_stream(
    handle: AppHandle,
    request_id: String,
    messages: Vec<LlmMessage>,
    model_override: Option<String>,
    profile_id: Option<String>,
) -> AppResult<()> {
    let request = LlmChatRequest {
        request_id,
        profile_id,
        messages,
        model_override,
    };
    validate_request(&request)?;

    let resolved_profile =
        resolve_profile_for_backend(&handle, request.profile_id.as_deref()).await?;
    let provider_name = resolved_profile.provider.as_str().to_string();
    let effective_model =
        resolve_effective_model(&resolved_profile, request.model_override.as_deref())?;
    let started_request_id = request.request_id.clone();
    let started_provider = provider_name.clone();
    let started_model = effective_model.clone();

    emit_response_event(
        &handle,
        &LlmStreamEvent {
            request_id: started_request_id,
            kind: "started".to_string(),
            delta: None,
            text: None,
            provider: started_provider,
            model: started_model,
            usage: None,
            error: None,
        },
    );

    let app_handle = handle.clone();
    tokio::spawn(async move {
        match run_llm_stream_with_profile(&request, &resolved_profile, |delta| {
            emit_response_event(
                &app_handle,
                &LlmStreamEvent {
                    request_id: request.request_id.clone(),
                    kind: "delta".to_string(),
                    delta: Some(delta.to_string()),
                    text: None,
                    provider: provider_name.clone(),
                    model: effective_model.clone(),
                    usage: None,
                    error: None,
                },
            );
        })
        .await
        {
            Ok(completion) => {
                emit_response_event(
                    &app_handle,
                    &LlmStreamEvent {
                        request_id: request.request_id,
                        kind: "completed".to_string(),
                        delta: None,
                        text: Some(completion.text),
                        provider: completion.provider,
                        model: completion.model,
                        usage: completion.usage,
                        error: None,
                    },
                );
            }
            Err(error) => {
                emit_response_event(
                    &app_handle,
                    &LlmStreamEvent {
                        request_id: request.request_id,
                        kind: "failed".to_string(),
                        delta: None,
                        text: None,
                        provider: provider_name,
                        model: effective_model,
                        usage: None,
                        error: Some(error),
                    },
                );
            }
        }
    });

    Ok(())
}
