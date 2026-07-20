use secrecy::SecretString;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::secret_store::SecretStoreState;

mod app_types;
mod profiles;

pub use app_types::{LlmProfile, LlmProfilesState, LlmStreamEvent};
use extractum_llm::list_provider_models;
pub(crate) use extractum_llm::{
    llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key, normalize_base_url,
    resolve_effective_model,
    resolve_model_input_token_limit as resolve_model_input_token_limit_for_backend,
    resolve_model_output_token_limit as resolve_model_output_token_limit_for_backend,
    run_llm_collect_with_profile, run_llm_stream_with_profile, validate_request, LlmCompletion,
    LlmProviderAccess, LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority,
    LlmRequestSnapshot, LlmRequestSnapshotState, LlmSchedulerState, ProviderKind,
    ResolvedLlmProfile,
};
pub use extractum_llm::{LlmChatRequest, LlmMessage, LlmProviderModel, LlmUsage};
use profiles::{
    clear_profile_api_key, delete_profile_from_pool, load_profiles_state_from_pool,
    resolve_profile_from_pool, resolve_provider_access_from_pool, save_profile_to_pool,
    set_active_profile_in_pool, validate_profile_id, validate_profile_input,
};

const LLM_RESPONSE_EVENT: &str = "llm://response";
const DEFAULT_PROFILE_ID: &str = "default";
const DEFAULT_MODEL: &str = "gemini-2.5-flash";

fn emit_response_event(handle: &AppHandle, event: &LlmStreamEvent) {
    let _ = handle.emit(LLM_RESPONSE_EVENT, event);
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

fn failed_stream_event(
    request_id: String,
    provider: String,
    model: String,
    error: &AppError,
) -> LlmStreamEvent {
    StreamEvent::new(request_id, "failed", provider, model)
        .error(error.to_string())
        .build()
}

fn cancelled_stream_event(request_id: String, provider: String, model: String) -> LlmStreamEvent {
    StreamEvent::new(request_id, "cancelled", provider, model)
        .error("Request cancelled.".to_string())
        .build()
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

    let access = if configured_key.is_some() && configured_base_url.is_some() {
        LlmProviderAccess::new(
            provider_kind,
            configured_key.expect("configured key checked"),
            normalize_base_url(provider_kind, configured_base_url.as_deref())?,
        )
    } else {
        let pool = get_pool(&handle).await?;
        resolve_provider_access_from_pool(
            &pool,
            &secret_store,
            provider_kind,
            profile_id.as_deref(),
            configured_key,
            configured_base_url,
        )
        .await?
    };

    list_provider_models(&access).await
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
    let provider_name = resolved_profile.provider().as_str().to_string();
    let effective_model =
        resolve_effective_model(&resolved_profile, request.model_override.as_deref())?;

    let app_handle = handle.clone();
    let request_meta = LlmRequestMetadata {
        request_id: request.request_id.clone(),
        profile_id: resolved_profile.profile_id().to_string(),
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
                    &failed_stream_event(failed_request_id, failed_provider, failed_model, &error),
                );
            }
            Err(LlmRequestError::Cancelled) => {
                emit_response_event(
                    &cancelled_handle,
                    &cancelled_stream_event(
                        cancelled_request_id,
                        cancelled_provider,
                        cancelled_model,
                    ),
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
        cancelled_stream_event, failed_stream_event, load_provider_diagnostics_from_pool,
        save_profile_to_pool, LlmUsage, StreamEvent,
    };
    use crate::error::AppError;

    #[test]
    fn llm_stream_events_serialize_exact_lifecycle_contract() {
        let base = || {
            (
                "request-1".to_string(),
                "gemini".to_string(),
                "gemini-2.5-flash".to_string(),
            )
        };
        let (request_id, provider, model) = base();
        let queued = StreamEvent::new(request_id, "queued", provider, model)
            .queue_position(2)
            .build();
        let (request_id, provider, model) = base();
        let started = StreamEvent::new(request_id, "started", provider, model).build();
        let (request_id, provider, model) = base();
        let delta = StreamEvent::new(request_id, "delta", provider, model)
            .delta("hello".to_string())
            .build();
        let (request_id, provider, model) = base();
        let completed = StreamEvent::new(request_id, "completed", provider, model)
            .text("hello".to_string())
            .usage(Some(LlmUsage {
                input_tokens: Some(3),
                output_tokens: Some(2),
                total_tokens: Some(5),
            }))
            .build();
        let failure = AppError::network("LLM request failed: transport");
        let (request_id, provider, model) = base();
        let failed = failed_stream_event(request_id, provider, model, &failure);
        let (request_id, provider, model) = base();
        let cancelled = cancelled_stream_event(request_id, provider, model);

        assert_eq!(
            serde_json::to_string(&queued).unwrap(),
            r#"{"request_id":"request-1","kind":"queued","queue_position":2,"delta":null,"text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":null}"#
        );
        assert_eq!(
            serde_json::to_string(&started).unwrap(),
            r#"{"request_id":"request-1","kind":"started","queue_position":null,"delta":null,"text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":null}"#
        );
        assert_eq!(
            serde_json::to_string(&delta).unwrap(),
            r#"{"request_id":"request-1","kind":"delta","queue_position":null,"delta":"hello","text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":null}"#
        );
        assert_eq!(
            serde_json::to_string(&completed).unwrap(),
            r#"{"request_id":"request-1","kind":"completed","queue_position":null,"delta":null,"text":"hello","provider":"gemini","model":"gemini-2.5-flash","usage":{"input_tokens":3,"output_tokens":2,"total_tokens":5},"error":null}"#
        );
        assert_eq!(
            serde_json::to_string(&failed).unwrap(),
            r#"{"request_id":"request-1","kind":"failed","queue_position":null,"delta":null,"text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":"LLM request failed: transport"}"#
        );
        assert_eq!(
            serde_json::to_string(&cancelled).unwrap(),
            r#"{"request_id":"request-1","kind":"cancelled","queue_position":null,"delta":null,"text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":"Request cancelled."}"#
        );
    }

    #[test]
    fn llm_command_errors_and_failed_events_keep_distinct_json_shapes() {
        let error = AppError::network("LLM request failed: transport");
        assert_eq!(
            serde_json::to_string(&error).unwrap(),
            r#"{"kind":"network","message":"LLM request failed: transport"}"#,
        );

        let failed = failed_stream_event(
            "request-1".to_string(),
            "gemini".to_string(),
            "gemini-2.5-flash".to_string(),
            &error,
        );
        assert_eq!(
            serde_json::to_value(failed).unwrap()["error"],
            serde_json::json!("LLM request failed: transport"),
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
