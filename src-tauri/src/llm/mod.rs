use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{timeout, Duration};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::secret_store::SecretStoreState;

mod gemini;
mod openai_compat;
mod profiles;
mod runner;
mod scheduler;
mod streaming;
mod types;

use gemini::list_gemini_models;
use openai_compat::{list_openai_compat_models, OpenAiCompatProviderConfig};
use profiles::{
    clear_profile_api_key, delete_profile_from_pool, load_profiles_state_from_pool,
    resolve_profile_from_pool, save_profile_to_pool, set_active_profile_in_pool,
    validate_profile_id, validate_profile_input,
};
pub(crate) use runner::{
    resolve_effective_model, run_llm_collect_with_profile, run_llm_stream_with_profile,
    validate_request,
};
pub(crate) use scheduler::{
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmRequestSnapshot,
    LlmRequestSnapshotState, LlmSchedulerState,
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
const MODEL_LIMIT_LOOKUP_TIMEOUT_SECS: u64 = 5;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProviderKind {
    Gemini,
    OpenAiCompatible,
}

impl ProviderKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Gemini => DEFAULT_PROVIDER,
            Self::OpenAiCompatible => "openai_compatible",
        }
    }

    pub(crate) fn parse(value: &str) -> AppResult<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            DEFAULT_PROVIDER => Ok(Self::Gemini),
            "openai_compatible" | "omniroute" => Ok(Self::OpenAiCompatible),
            other => Err(AppError::validation(format!(
                "Unsupported provider '{other}'"
            ))),
        }
    }

    pub(super) fn display_name(self) -> &'static str {
        match self {
            Self::Gemini => "Gemini",
            Self::OpenAiCompatible => "OpenAI-compatible",
        }
    }
}

fn default_base_url_for_provider_kind(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Gemini => "",
        ProviderKind::OpenAiCompatible => DEFAULT_OPENAI_COMPAT_BASE_URL,
    }
}

fn default_base_url_for_provider(provider: &str) -> &'static str {
    ProviderKind::parse(provider)
        .map(default_base_url_for_provider_kind)
        .unwrap_or("")
}

fn normalize_base_url(provider: ProviderKind, base_url: Option<&str>) -> AppResult<String> {
    match provider {
        ProviderKind::Gemini => Ok(String::new()),
        ProviderKind::OpenAiCompatible => {
            let candidate = base_url
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(DEFAULT_OPENAI_COMPAT_BASE_URL);
            let parsed = reqwest::Url::parse(candidate)
                .map_err(|_| AppError::validation(format!("Invalid base URL '{candidate}'")))?;
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err(AppError::validation("Base URL must use http or https"));
            }
            if parsed.scheme() == "http" {
                let is_loopback_ip = parsed
                    .host_str()
                    .map(|host| host.trim_start_matches('[').trim_end_matches(']'))
                    .and_then(|host| host.parse::<IpAddr>().ok())
                    .is_some_and(|ip| ip.is_loopback());
                let is_localhost = parsed
                    .host_str()
                    .is_some_and(|host| host.eq_ignore_ascii_case("localhost"));
                if !is_localhost && !is_loopback_ip {
                    return Err(AppError::validation(
                        "HTTP base URL must use localhost or a loopback IP address",
                    ));
                }
            }

            Ok(parsed.as_str().trim_end_matches('/').to_string())
        }
    }
}

fn emit_response_event(handle: &AppHandle, event: &LlmStreamEvent) {
    let _ = handle.emit(LLM_RESPONSE_EVENT, event);
}

fn model_input_token_limit_from_models(
    models: &[LlmProviderModel],
    requested_model: &str,
) -> Option<usize> {
    let requested_model = requested_model.trim();
    if requested_model.is_empty() {
        return None;
    }

    models
        .iter()
        .find(|model| {
            model.model == requested_model
                || model.name == requested_model
                || model.display_name == requested_model
        })
        .and_then(|model| usize::try_from(model.input_token_limit?).ok())
        .filter(|limit| *limit > 0)
}

fn model_output_token_limit_from_models(
    models: &[LlmProviderModel],
    requested_model: &str,
) -> Option<i64> {
    let requested_model = requested_model.trim();
    if requested_model.is_empty() {
        return None;
    }

    models
        .iter()
        .find(|model| {
            model.model == requested_model
                || model.name == requested_model
                || model.display_name == requested_model
        })
        .and_then(|model| model.output_token_limit)
        .filter(|limit| *limit > 0)
}

pub(crate) async fn resolve_model_input_token_limit_for_backend(
    profile: &ResolvedLlmProfile,
    model: &str,
) -> Option<usize> {
    let provider = profile.provider;
    let api_key = profile.api_key.clone();
    let openai_compat_config = OpenAiCompatProviderConfig {
        provider,
        base_url: profile.base_url.clone(),
    };
    let result = timeout(
        Duration::from_secs(MODEL_LIMIT_LOOKUP_TIMEOUT_SECS),
        async move {
            match provider {
                ProviderKind::Gemini => list_gemini_models(api_key.expose_secret()).await,
                ProviderKind::OpenAiCompatible => {
                    list_openai_compat_models(api_key.expose_secret(), &openai_compat_config).await
                }
            }
        },
    )
    .await;

    match result {
        Ok(Ok(models)) => model_input_token_limit_from_models(&models, model),
        Ok(Err(_)) | Err(_) => None,
    }
}

pub(crate) async fn resolve_model_output_token_limit_for_backend(
    profile: &ResolvedLlmProfile,
    model: &str,
) -> Option<i64> {
    let provider = profile.provider;
    let api_key = profile.api_key.clone();
    let openai_compat_config = OpenAiCompatProviderConfig {
        provider,
        base_url: profile.base_url.clone(),
    };
    let result = timeout(
        Duration::from_secs(MODEL_LIMIT_LOOKUP_TIMEOUT_SECS),
        async move {
            match provider {
                ProviderKind::Gemini => list_gemini_models(api_key.expose_secret()).await,
                ProviderKind::OpenAiCompatible => {
                    list_openai_compat_models(api_key.expose_secret(), &openai_compat_config).await
                }
            }
        },
    )
    .await;

    match result {
        Ok(Ok(models)) => model_output_token_limit_from_models(&models, model),
        Ok(Err(_)) | Err(_) => None,
    }
}

struct StreamEvent {
    event: LlmStreamEvent,
}

impl StreamEvent {
    fn new(request_id: String, kind: &str, provider: String, model: String) -> Self {
        Self {
            event: LlmStreamEvent {
                request_id,
                kind: kind.to_string(),
                queue_position: None,
                delta: None,
                text: None,
                provider,
                model,
                usage: None,
                error: None,
            },
        }
    }

    fn queue_position(mut self, queue_position: usize) -> Self {
        self.event.queue_position = Some(queue_position);
        self
    }

    fn delta(mut self, delta: String) -> Self {
        self.event.delta = Some(delta);
        self
    }

    fn text(mut self, text: String) -> Self {
        self.event.text = Some(text);
        self
    }

    fn usage(mut self, usage: Option<LlmUsage>) -> Self {
        self.event.usage = usage;
        self
    }

    fn error(mut self, error: String) -> Self {
        self.event.error = Some(error);
        self
    }

    fn build(self) -> LlmStreamEvent {
        self.event
    }
}

pub(crate) async fn resolve_profile_for_backend(
    handle: &AppHandle,
    requested_profile_id: Option<&str>,
) -> AppResult<ResolvedLlmProfile> {
    let pool = get_pool(handle).await?;
    let secret_store = handle.state::<SecretStoreState>();
    resolve_profile_from_pool(&pool, &secret_store, requested_profile_id).await
}

#[tauri::command]
pub async fn get_llm_request_snapshots(
    state: tauri::State<'_, LlmSchedulerState>,
) -> AppResult<Vec<LlmRequestSnapshot>> {
    Ok(state.request_snapshots().await)
}

#[tauri::command]
pub async fn get_llm_profiles(
    handle: AppHandle,
    secret_store: tauri::State<'_, SecretStoreState>,
) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    load_profiles_state_from_pool(&pool, &secret_store).await
}

pub(crate) fn llm_request_kind_diagnostic_key(kind: LlmRequestKind) -> &'static str {
    match kind {
        LlmRequestKind::ProviderTest => "provider_test",
        LlmRequestKind::AnalysisChat => "analysis_chat",
        LlmRequestKind::AnalysisReportMap => "analysis_report_map",
        LlmRequestKind::AnalysisReportReduce => "analysis_report_reduce",
        LlmRequestKind::PromptPackStage => "prompt_pack_stage",
    }
}

pub(crate) fn llm_request_state_diagnostic_key(state: LlmRequestSnapshotState) -> &'static str {
    match state {
        LlmRequestSnapshotState::Queued => "queued",
        LlmRequestSnapshotState::Running => "running",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LlmProviderDiagnosticCount {
    pub(crate) provider: String,
    pub(crate) configured_count: i64,
    pub(crate) missing_key_count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LlmProviderDiagnosticState {
    pub(crate) active_provider: Option<String>,
    pub(crate) profiles_by_provider: Vec<LlmProviderDiagnosticCount>,
}

pub(crate) async fn load_provider_diagnostics_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secret_store: &SecretStoreState,
) -> AppResult<LlmProviderDiagnosticState> {
    let state = load_profiles_state_from_pool(pool, secret_store).await?;
    let active_provider = state
        .profiles
        .iter()
        .find(|profile| profile.profile_id == state.active_profile)
        .map(|profile| profile.provider.clone());
    let mut counts = std::collections::BTreeMap::<String, (i64, i64)>::new();
    for profile in state.profiles {
        let entry = counts.entry(profile.provider).or_insert((0, 0));
        if profile.api_key_configured {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }
    Ok(LlmProviderDiagnosticState {
        active_provider,
        profiles_by_provider: counts
            .into_iter()
            .map(
                |(provider, (configured_count, missing_key_count))| LlmProviderDiagnosticCount {
                    provider,
                    configured_count,
                    missing_key_count,
                },
            )
            .collect(),
    })
}

#[tauri::command]
#[expect(
    clippy::too_many_arguments,
    reason = "Tauri command signature is the frontend IPC contract."
)]
pub async fn save_llm_profile(
    handle: AppHandle,
    secret_store: tauri::State<'_, SecretStoreState>,
    profile_id: Option<String>,
    provider: String,
    default_model: String,
    api_key: Option<String>,
    base_url: Option<String>,
    set_active: Option<bool>,
) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    let (profile_id, provider_kind, default_model, base_url) =
        validate_profile_input(profile_id, provider, default_model, base_url)?;
    let set_active = set_active.unwrap_or(false);

    save_profile_to_pool(
        &pool,
        &secret_store,
        &profile_id,
        provider_kind.as_str(),
        &default_model,
        api_key.as_deref(),
        &base_url,
        set_active,
    )
    .await?;

    load_profiles_state_from_pool(&pool, &secret_store).await
}

#[tauri::command]
pub async fn clear_llm_profile_api_key(
    handle: AppHandle,
    secret_store: tauri::State<'_, SecretStoreState>,
    profile_id: String,
) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    let profile_id = validate_profile_id(&profile_id)?;
    clear_profile_api_key(&secret_store, &profile_id).await?;
    load_profiles_state_from_pool(&pool, &secret_store).await
}

#[tauri::command]
pub async fn set_active_llm_profile(
    handle: AppHandle,
    secret_store: tauri::State<'_, SecretStoreState>,
    profile_id: String,
) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    let profile_id = validate_profile_id(&profile_id)?;
    set_active_profile_in_pool(&pool, &profile_id).await?;
    load_profiles_state_from_pool(&pool, &secret_store).await
}

#[tauri::command]
pub async fn delete_llm_profile(
    handle: AppHandle,
    secret_store: tauri::State<'_, SecretStoreState>,
    profile_id: String,
) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    let profile_id = validate_profile_id(&profile_id)?;
    delete_profile_from_pool(&pool, &secret_store, &profile_id).await?;
    load_profiles_state_from_pool(&pool, &secret_store).await
}

#[tauri::command]
pub async fn list_llm_provider_models(
    handle: AppHandle,
    secret_store: tauri::State<'_, SecretStoreState>,
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
        .map(|value| SecretString::new(value.to_string()));
    let configured_base_url = base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    let saved_profile = if configured_key.is_none() || configured_base_url.is_none() {
        let pool = get_pool(&handle).await?;
        Some(resolve_profile_from_pool(&pool, &secret_store, profile_id.as_deref()).await?)
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
            .unwrap_or_else(|| SecretString::new(String::new()))
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
        ProviderKind::OpenAiCompatible => OPENAI_COMPAT_MODELS_TIMEOUT_SECS,
    };
    let openai_compat_config = OpenAiCompatProviderConfig {
        provider: provider_kind,
        base_url,
    };

    let result = timeout(Duration::from_secs(timeout_secs), async move {
        match provider_kind {
            ProviderKind::Gemini => list_gemini_models(api_key.expose_secret()).await,
            ProviderKind::OpenAiCompatible => {
                list_openai_compat_models(api_key.expose_secret(), &openai_compat_config).await
            }
        }
    })
    .await;

    match result {
        Ok(models) => models,
        Err(_) => Err(AppError::llm_network(format!(
            "Loading {} models timed out after {timeout_secs} seconds",
            provider_kind.display_name()
        ))),
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
        max_output_tokens: None,
    };
    validate_request(&request)?;

    let resolved_profile =
        resolve_profile_for_backend(&handle, request.profile_id.as_deref()).await?;
    let provider_name = resolved_profile.provider.as_str().to_string();
    let effective_model =
        resolve_effective_model(&resolved_profile, request.model_override.as_deref())?;

    let app_handle = handle.clone();
    let request_meta = LlmRequestMetadata {
        request_id: request.request_id.clone(),
        profile_id: resolved_profile.profile_id.clone(),
        provider: provider_name.clone(),
        kind: LlmRequestKind::ProviderTest,
        priority: LlmRequestPriority::Interactive,
        owner_run_id: None,
    };
    tokio::spawn(async move {
        let scheduler = app_handle.state::<LlmSchedulerState>();
        let queued_handle = app_handle.clone();
        let started_handle = app_handle.clone();
        let delta_handle = app_handle.clone();
        let completed_handle = app_handle.clone();
        let failed_handle = app_handle.clone();
        let cancelled_handle = app_handle.clone();
        let queued_request_id = request.request_id.clone();
        let started_request_id = request.request_id.clone();
        let delta_request_id = request.request_id.clone();
        let completed_request_id = request.request_id.clone();
        let failed_request_id = request.request_id.clone();
        let cancelled_request_id = request.request_id.clone();
        let queued_provider = provider_name.clone();
        let started_provider = provider_name.clone();
        let delta_provider = provider_name.clone();
        let failed_provider = provider_name.clone();
        let cancelled_provider = provider_name.clone();
        let queued_model = effective_model.clone();
        let started_model = effective_model.clone();
        let delta_model = effective_model.clone();
        let failed_model = effective_model.clone();
        let cancelled_model = effective_model.clone();
        let scheduled_request = request.clone();
        let scheduled_profile = resolved_profile.clone();

        match scheduler
            .run_request(
                request_meta,
                move |position| {
                    emit_response_event(
                        &queued_handle,
                        &StreamEvent::new(
                            queued_request_id.clone(),
                            "queued",
                            queued_provider.clone(),
                            queued_model.clone(),
                        )
                        .queue_position(position)
                        .build(),
                    );
                },
                move |control| async move {
                    emit_response_event(
                        &started_handle,
                        &StreamEvent::new(
                            started_request_id,
                            "started",
                            started_provider,
                            started_model,
                        )
                        .build(),
                    );

                    control
                        .run_cancellable(run_llm_stream_with_profile(
                            &scheduled_request,
                            &scheduled_profile,
                            |delta| {
                                emit_response_event(
                                    &delta_handle,
                                    &StreamEvent::new(
                                        delta_request_id.clone(),
                                        "delta",
                                        delta_provider.clone(),
                                        delta_model.clone(),
                                    )
                                    .delta(delta.to_string())
                                    .build(),
                                );
                            },
                        ))
                        .await
                },
            )
            .await
        {
            Ok(completion) => {
                emit_response_event(
                    &completed_handle,
                    &StreamEvent::new(
                        completed_request_id,
                        "completed",
                        completion.provider,
                        completion.model,
                    )
                    .text(completion.text)
                    .usage(completion.usage)
                    .build(),
                );
            }
            Err(LlmRequestError::Failed(error)) => {
                emit_response_event(
                    &failed_handle,
                    &StreamEvent::new(failed_request_id, "failed", failed_provider, failed_model)
                        .error(error.to_string())
                        .build(),
                );
            }
            Err(LlmRequestError::Cancelled) => {
                emit_response_event(
                    &cancelled_handle,
                    &StreamEvent::new(
                        cancelled_request_id,
                        "cancelled",
                        cancelled_provider,
                        cancelled_model,
                    )
                    .error("Request cancelled.".to_string())
                    .build(),
                );
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn cancel_llm_request(
    state: tauri::State<'_, LlmSchedulerState>,
    request_id: String,
) -> AppResult<()> {
    let request_id = request_id.trim().to_string();
    if request_id.is_empty() {
        return Err(AppError::validation("request_id cannot be empty"));
    }

    if state.cancel_request(&request_id).await {
        Ok(())
    } else {
        Err(AppError::not_found(format!(
            "LLM request '{request_id}' is no longer active"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key,
        load_provider_diagnostics_from_pool, model_input_token_limit_from_models,
        model_output_token_limit_from_models, normalize_base_url, save_profile_to_pool,
        LlmRequestKind, LlmRequestSnapshotState, ProviderKind,
    };
    use crate::error::AppErrorKind;
    use crate::llm::LlmProviderModel;

    #[test]
    fn provider_parse_returns_typed_validation_error() {
        let error = ProviderKind::parse("unknown").expect_err("reject unsupported provider");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Unsupported provider 'unknown'");
    }

    #[test]
    fn provider_parse_accepts_openai_compatible_aliases() {
        let provider = ProviderKind::parse("openai_compatible").expect("parse canonical provider");
        assert_eq!(provider.as_str(), "openai_compatible");
        assert_eq!(provider.display_name(), "OpenAI-compatible");

        let legacy_provider = ProviderKind::parse("omniroute").expect("parse legacy provider");
        assert_eq!(legacy_provider.as_str(), "openai_compatible");
        assert_eq!(legacy_provider.display_name(), "OpenAI-compatible");
    }

    #[test]
    fn model_input_token_limit_lookup_matches_provider_model_ids_and_names() {
        let models = vec![
            LlmProviderModel {
                model: "gemini-2.5-pro".to_string(),
                name: "models/gemini-2.5-pro".to_string(),
                display_name: "Gemini 2.5 Pro".to_string(),
                description: String::new(),
                input_token_limit: Some(1_048_576),
                output_token_limit: None,
                supported_generation_methods: Vec::new(),
            },
            LlmProviderModel {
                model: "broken".to_string(),
                name: "broken".to_string(),
                display_name: "Broken".to_string(),
                description: String::new(),
                input_token_limit: Some(-1),
                output_token_limit: None,
                supported_generation_methods: Vec::new(),
            },
        ];

        assert_eq!(
            model_input_token_limit_from_models(&models, "gemini-2.5-pro"),
            Some(1_048_576)
        );
        assert_eq!(
            model_input_token_limit_from_models(&models, "models/gemini-2.5-pro"),
            Some(1_048_576)
        );
        assert_eq!(model_input_token_limit_from_models(&models, "broken"), None);
        assert_eq!(
            model_input_token_limit_from_models(&models, "missing"),
            None
        );
    }

    #[test]
    fn model_output_token_limit_lookup_matches_provider_model_ids_and_names() {
        let models = vec![
            LlmProviderModel {
                model: "gemini-2.5-pro".to_string(),
                name: "models/gemini-2.5-pro".to_string(),
                display_name: "Gemini 2.5 Pro".to_string(),
                description: String::new(),
                input_token_limit: None,
                output_token_limit: Some(65_536),
                supported_generation_methods: Vec::new(),
            },
            LlmProviderModel {
                model: "broken".to_string(),
                name: "broken".to_string(),
                display_name: "Broken".to_string(),
                description: String::new(),
                input_token_limit: None,
                output_token_limit: Some(-1),
                supported_generation_methods: Vec::new(),
            },
        ];

        assert_eq!(
            model_output_token_limit_from_models(&models, "gemini-2.5-pro"),
            Some(65_536)
        );
        assert_eq!(
            model_output_token_limit_from_models(&models, "models/gemini-2.5-pro"),
            Some(65_536)
        );
        assert_eq!(
            model_output_token_limit_from_models(&models, "Gemini 2.5 Pro"),
            Some(65_536)
        );
        assert_eq!(
            model_output_token_limit_from_models(&models, "broken"),
            None
        );
        assert_eq!(
            model_output_token_limit_from_models(&models, "missing"),
            None
        );
    }

    #[test]
    fn normalize_base_url_returns_typed_validation_error() {
        let error = normalize_base_url(ProviderKind::OpenAiCompatible, Some("ftp://localhost"))
            .expect_err("reject non-http base url");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Base URL must use http or https");
    }

    #[test]
    fn normalize_base_url_allows_https_and_loopback_http_only() {
        let cases = [
            ("https endpoint", "https://example.com/v1", true),
            ("localhost http", "http://LOCALHOST:8080/v1", true),
            ("ipv4 loopback http", "http://127.0.0.1:8080/v1", true),
            ("ipv4 loopback range http", "http://127.1.2.3:8080/v1", true),
            ("ipv6 loopback http", "http://[::1]:8080/v1", true),
            ("remote ipv4 http", "http://192.0.2.1/v1", false),
            ("remote ipv6 http", "http://[2001:db8::1]/v1", false),
            ("hostname http", "http://example.com/v1", false),
            ("unsupported scheme", "ftp://localhost/v1", false),
        ];

        for (name, url, expected_ok) in cases {
            assert_eq!(
                normalize_base_url(ProviderKind::OpenAiCompatible, Some(url)).is_ok(),
                expected_ok,
                "{name}: {url}"
            );
        }
    }

    #[test]
    fn llm_request_diagnostic_keys_are_stable_snake_case() {
        assert_eq!(
            llm_request_kind_diagnostic_key(LlmRequestKind::AnalysisChat),
            "analysis_chat"
        );
        assert_eq!(
            llm_request_kind_diagnostic_key(LlmRequestKind::AnalysisReportReduce),
            "analysis_report_reduce"
        );
        assert_eq!(
            llm_request_state_diagnostic_key(LlmRequestSnapshotState::Queued),
            "queued"
        );
        assert_eq!(
            llm_request_state_diagnostic_key(LlmRequestSnapshotState::Running),
            "running"
        );
    }

    #[tokio::test]
    async fn provider_diagnostics_exclude_profile_ids_and_base_urls() {
        use crate::migrations::apply_all_migrations_for_test_pool;
        use crate::secret_store::tests::InMemorySecretStore;
        use crate::secret_store::SecretStoreState;
        use std::sync::Arc;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        let store = Arc::new(InMemorySecretStore::new());
        let secret_store = SecretStoreState::new(store);

        save_profile_to_pool(
            &pool,
            &secret_store,
            "private-profile",
            "gemini",
            "private-model",
            Some("private-api-key"),
            "",
            true,
        )
        .await
        .expect("save profile");

        let diagnostics = load_provider_diagnostics_from_pool(&pool, &secret_store)
            .await
            .expect("load provider diagnostics");
        let json = serde_json::to_string(&diagnostics.profiles_by_provider)
            .expect("serialize provider diagnostics");

        assert_eq!(diagnostics.active_provider.as_deref(), Some("gemini"));
        assert!(json.contains("gemini"));
        assert!(!json.contains("private-profile"));
        assert!(!json.contains("private-model"));
        assert!(!json.contains("private-api-key"));
        assert!(!json.contains("base_url"));
    }
}
