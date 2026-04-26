use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Emitter};
use tokio::time::{timeout, Duration};

use crate::db::get_pool;
use crate::error::AppResult;

const LLM_RESPONSE_EVENT: &str = "llm://response";
const DEFAULT_PROFILE_ID: &str = "default";
const DEFAULT_PROVIDER: &str = "gemini";
const DEFAULT_MODEL: &str = "gemini-2.5-flash";
const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";
const OMNIROUTE_API_BASE: &str = "http://localhost:20128/v1";
const LLM_STREAM_TIMEOUT_SECS: u64 = 90;
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

    fn display_name(self) -> &'static str {
        match self {
            Self::Gemini => "Gemini",
            Self::OmniRoute => "OmniRoute",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmChatRequest {
    pub request_id: String,
    pub profile_id: Option<String>,
    pub messages: Vec<LlmMessage>,
    pub model_override: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmUsage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmStreamEvent {
    pub request_id: String,
    pub kind: String,
    pub delta: Option<String>,
    pub text: Option<String>,
    pub provider: String,
    pub model: String,
    pub usage: Option<LlmUsage>,
    pub error: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmProfile {
    pub profile_id: String,
    pub provider: String,
    pub default_model: String,
    pub api_key: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmProfilesState {
    pub active_profile: String,
    pub default_profile: LlmProfile,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmProviderModel {
    pub model: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub input_token_limit: Option<i64>,
    pub output_token_limit: Option<i64>,
    pub supported_generation_methods: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct ResolvedLlmProfile {
    pub(crate) profile_id: String,
    pub(crate) provider: ProviderKind,
    pub(crate) default_model: String,
    pub(crate) api_key: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct LlmCompletion {
    pub provider: String,
    pub model: String,
    pub text: String,
    pub usage: Option<LlmUsage>,
}

#[derive(Serialize)]
struct GeminiGenerateContentRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemInstruction")]
    system_instruction: Option<GeminiContent>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerateContentResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    usage_metadata: Option<GeminiUsageMetadata>,
    prompt_feedback: Option<GeminiPromptFeedback>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GeminiPromptFeedback {
    block_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: Option<i64>,
    candidates_token_count: Option<i64>,
    total_token_count: Option<i64>,
}

#[derive(Deserialize)]
struct GoogleApiErrorEnvelope {
    error: GoogleApiErrorBody,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleApiErrorBody {
    message: String,
    status: Option<String>,
    code: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiListModelsResponse {
    models: Option<Vec<GeminiModel>>,
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiModel {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    input_token_limit: Option<i64>,
    output_token_limit: Option<i64>,
    supported_generation_methods: Option<Vec<String>>,
}

#[derive(Serialize)]
struct OpenAiCompatChatRequest {
    model: String,
    messages: Vec<OpenAiCompatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<OpenAiCompatStreamOptions>,
}

#[derive(Serialize)]
struct OpenAiCompatStreamOptions {
    include_usage: bool,
}

#[derive(Serialize)]
struct OpenAiCompatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct OpenAiCompatChatChunk {
    choices: Option<Vec<OpenAiCompatChoice>>,
    usage: Option<OpenAiCompatUsage>,
}

#[derive(Deserialize, Debug)]
struct OpenAiCompatChoice {
    delta: Option<OpenAiCompatDelta>,
    message: Option<OpenAiCompatMessageResponse>,
}

#[derive(Deserialize, Debug)]
struct OpenAiCompatDelta {
    content: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OpenAiCompatMessageResponse {
    content: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
struct OpenAiCompatUsage {
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    total_tokens: Option<i64>,
}

#[derive(Deserialize)]
struct OpenAiCompatModelsResponse {
    data: Vec<OpenAiCompatModel>,
}

#[derive(Deserialize)]
struct OpenAiCompatModel {
    id: String,
    object: Option<String>,
    owned_by: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiCompatErrorEnvelope {
    error: OpenAiCompatErrorBody,
}

#[derive(Deserialize)]
struct OpenAiCompatErrorBody {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<serde_json::Value>,
}

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

async fn save_profile_to_pool(
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

async fn load_profiles_state_from_pool(pool: &Pool<Sqlite>) -> Result<LlmProfilesState, String> {
    let active_profile = read_setting(pool, active_profile_key())
        .await?
        .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string());
    let default_profile = load_profile_from_pool(pool, DEFAULT_PROFILE_ID).await?;

    Ok(LlmProfilesState {
        active_profile,
        default_profile,
    })
}

async fn resolve_profile_from_pool(
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

fn validate_profile_input(
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

fn validate_request(request: &LlmChatRequest) -> Result<(), String> {
    if request.request_id.trim().is_empty() {
        return Err("request_id cannot be empty".to_string());
    }
    if request.messages.is_empty() {
        return Err("At least one message is required".to_string());
    }
    if request
        .messages
        .iter()
        .all(|message| message.content.trim().is_empty())
    {
        return Err("Messages cannot all be empty".to_string());
    }

    Ok(())
}

fn build_gemini_request(messages: &[LlmMessage]) -> Result<GeminiGenerateContentRequest, String> {
    let mut system_chunks = Vec::new();
    let mut contents = Vec::new();

    for message in messages {
        let content = message.content.trim();
        if content.is_empty() {
            continue;
        }

        match message.role.as_str() {
            "system" => {
                system_chunks.push(content.to_string());
            }
            "user" => contents.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: content.to_string(),
                }],
            }),
            "assistant" => contents.push(GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiPart {
                    text: content.to_string(),
                }],
            }),
            other => {
                return Err(format!("Unsupported message role '{other}'"));
            }
        }
    }

    if contents.is_empty() {
        return Err("At least one user or assistant message is required".to_string());
    }

    let system_instruction = if system_chunks.is_empty() {
        None
    } else {
        Some(GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart {
                text: system_chunks.join("\n\n"),
            }],
        })
    };

    Ok(GeminiGenerateContentRequest {
        contents,
        system_instruction,
    })
}

fn build_openai_compat_request(
    messages: &[LlmMessage],
    model: &str,
) -> Result<OpenAiCompatChatRequest, String> {
    let mut mapped_messages = Vec::new();

    for message in messages {
        let content = message.content.trim();
        if content.is_empty() {
            continue;
        }

        match message.role.as_str() {
            "system" | "user" | "assistant" => mapped_messages.push(OpenAiCompatMessage {
                role: message.role.clone(),
                content: content.to_string(),
            }),
            other => {
                return Err(format!("Unsupported message role '{other}'"));
            }
        }
    }

    if mapped_messages.is_empty() {
        return Err("At least one message is required".to_string());
    }

    Ok(OpenAiCompatChatRequest {
        model: model.to_string(),
        messages: mapped_messages,
        stream: true,
        stream_options: Some(OpenAiCompatStreamOptions {
            include_usage: true,
        }),
    })
}

fn extract_text(response: &GeminiGenerateContentResponse) -> String {
    response
        .candidates
        .as_ref()
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate.content.as_ref())
        .map(|content| {
            content
                .parts
                .iter()
                .map(|part| part.text.as_str())
                .collect::<String>()
        })
        .unwrap_or_default()
}

fn extract_openai_compat_delta(chunk: &OpenAiCompatChatChunk) -> String {
    chunk
        .choices
        .as_ref()
        .and_then(|choices| choices.first())
        .map(|choice| {
            choice
                .delta
                .as_ref()
                .and_then(|delta| delta.content.as_ref())
                .or_else(|| {
                    choice
                        .message
                        .as_ref()
                        .and_then(|message| message.content.as_ref())
                })
                .cloned()
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

fn map_usage(usage: &GeminiUsageMetadata) -> LlmUsage {
    LlmUsage {
        input_tokens: usage.prompt_token_count,
        output_tokens: usage.candidates_token_count,
        total_tokens: usage.total_token_count,
    }
}

fn map_openai_compat_usage(usage: &OpenAiCompatUsage) -> LlmUsage {
    LlmUsage {
        input_tokens: usage.prompt_tokens,
        output_tokens: usage.completion_tokens,
        total_tokens: usage.total_tokens,
    }
}

fn find_event_boundary(buffer: &[u8]) -> Option<(usize, usize)> {
    if buffer.len() < 2 {
        return None;
    }

    for index in 0..buffer.len() - 1 {
        if buffer[index] == b'\n' && buffer[index + 1] == b'\n' {
            return Some((index, 2));
        }
        if index + 3 < buffer.len()
            && buffer[index] == b'\r'
            && buffer[index + 1] == b'\n'
            && buffer[index + 2] == b'\r'
            && buffer[index + 3] == b'\n'
        {
            return Some((index, 4));
        }
    }

    None
}

fn parse_sse_data(event_bytes: &[u8]) -> Result<Option<String>, String> {
    let text = String::from_utf8(event_bytes.to_vec()).map_err(|e| e.to_string())?;
    let mut data_lines = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_string());
        }
    }

    if data_lines.is_empty() {
        return Ok(None);
    }

    let data = data_lines.join("\n");
    if data.trim() == "[DONE]" {
        return Ok(None);
    }

    Ok(Some(data))
}

fn format_google_error(status: reqwest::StatusCode, body: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<GoogleApiErrorEnvelope>(body) {
        let code = parsed.error.code.unwrap_or(i64::from(status.as_u16()));
        let status_label = parsed.error.status.unwrap_or_else(|| status.to_string());
        return format!("{status_label} ({code}): {}", parsed.error.message);
    }

    if body.trim().is_empty() {
        format!("Gemini request failed with HTTP {}", status.as_u16())
    } else {
        format!(
            "Gemini request failed with HTTP {}: {}",
            status.as_u16(),
            body.trim()
        )
    }
}

fn format_openai_compat_error(
    provider: ProviderKind,
    status: reqwest::StatusCode,
    body: &str,
) -> String {
    if let Ok(parsed) = serde_json::from_str::<OpenAiCompatErrorEnvelope>(body) {
        let mut details = parsed.error.message;
        if let Some(error_type) = parsed.error.error_type {
            details = format!("{error_type}: {details}");
        }
        if let Some(code) = parsed.error.code {
            details = format!("{details} ({code})");
        }
        return format!(
            "{} request failed with HTTP {}: {details}",
            provider.display_name(),
            status.as_u16()
        );
    }

    if body.trim().is_empty() {
        format!(
            "{} request failed with HTTP {}",
            provider.display_name(),
            status.as_u16()
        )
    } else {
        format!(
            "{} request failed with HTTP {}: {}",
            provider.display_name(),
            status.as_u16(),
            body.trim()
        )
    }
}

fn strip_gemini_model_prefix(name: &str) -> String {
    name.strip_prefix("models/").unwrap_or(name).to_string()
}

fn map_gemini_model(model: GeminiModel) -> LlmProviderModel {
    let model_id = strip_gemini_model_prefix(&model.name);
    LlmProviderModel {
        display_name: model
            .display_name
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| model_id.clone()),
        description: model.description.unwrap_or_default(),
        input_token_limit: model.input_token_limit,
        output_token_limit: model.output_token_limit,
        supported_generation_methods: model.supported_generation_methods.unwrap_or_default(),
        model: model_id,
        name: model.name,
    }
}

fn map_openai_compat_model(model: OpenAiCompatModel) -> LlmProviderModel {
    let description = model.owned_by.unwrap_or_default();
    let generation_method = model.object.unwrap_or_else(|| "model".to_string());

    LlmProviderModel {
        display_name: model.id.clone(),
        description,
        input_token_limit: None,
        output_token_limit: None,
        supported_generation_methods: vec![generation_method],
        model: model.id.clone(),
        name: model.id,
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

pub(crate) fn resolve_effective_model(
    profile: &ResolvedLlmProfile,
    model_override: Option<&str>,
) -> Result<String, String> {
    let model = model_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(profile.default_model.as_str())
        .trim()
        .to_string();

    if model.is_empty() {
        return Err("Model override cannot be empty".to_string());
    }

    Ok(model)
}

async fn stream_with_provider<F>(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
    on_delta: &mut F,
) -> Result<LlmCompletion, String>
where
    F: FnMut(&str),
{
    match profile.provider {
        ProviderKind::Gemini => stream_gemini_response(request, profile, on_delta).await,
        ProviderKind::OmniRoute => {
            stream_openai_compat_response(
                request,
                profile,
                on_delta,
                OMNIROUTE_API_BASE,
                ProviderKind::OmniRoute,
            )
            .await
        }
    }
}

async fn stream_gemini_response<F>(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
    on_delta: &mut F,
) -> Result<LlmCompletion, String>
where
    F: FnMut(&str),
{
    if profile.api_key.trim().is_empty() {
        return Err(format!(
            "Profile '{}' does not have a Gemini API key configured",
            profile.profile_id
        ));
    }

    let model = resolve_effective_model(profile, request.model_override.as_deref())?;

    let request_body = build_gemini_request(&request.messages)?;

    let url = format!("{GEMINI_API_BASE}/models/{model}:streamGenerateContent?alt=sse");
    let client = HttpClient::new();
    let response = client
        .post(url)
        .header("x-goog-api-key", profile.api_key.as_str())
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format_google_error(status, &body));
    }

    let mut response = response;
    let mut buffer = Vec::new();
    let mut full_text = String::new();
    let mut last_usage = None;

    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        buffer.extend_from_slice(&chunk);

        while let Some((boundary, delimiter_len)) = find_event_boundary(&buffer) {
            let event_bytes = buffer[..boundary].to_vec();
            buffer.drain(..boundary + delimiter_len);

            let Some(data) = parse_sse_data(&event_bytes)? else {
                continue;
            };

            let parsed: GeminiGenerateContentResponse =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;

            if let Some(block_reason) = parsed
                .prompt_feedback
                .as_ref()
                .and_then(|feedback| feedback.block_reason.clone())
            {
                return Err(format!("Prompt blocked by Gemini: {block_reason}"));
            }

            if let Some(usage) = parsed.usage_metadata.as_ref() {
                last_usage = Some(map_usage(usage));
            }

            let delta = extract_text(&parsed);
            if !delta.is_empty() {
                full_text.push_str(&delta);
                on_delta(&delta);
            }
        }
    }

    if !buffer.is_empty() {
        if let Some(data) = parse_sse_data(&buffer)? {
            let parsed: GeminiGenerateContentResponse =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            if let Some(block_reason) = parsed
                .prompt_feedback
                .as_ref()
                .and_then(|feedback| feedback.block_reason.clone())
            {
                return Err(format!("Prompt blocked by Gemini: {block_reason}"));
            }
            if let Some(usage) = parsed.usage_metadata.as_ref() {
                last_usage = Some(map_usage(usage));
            }
            let delta = extract_text(&parsed);
            if !delta.is_empty() {
                full_text.push_str(&delta);
                on_delta(&delta);
            }
        }
    }

    Ok(LlmCompletion {
        provider: profile.provider.as_str().to_string(),
        model,
        text: full_text,
        usage: last_usage,
    })
}

async fn stream_openai_compat_response<F>(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
    on_delta: &mut F,
    api_base: &str,
    provider: ProviderKind,
) -> Result<LlmCompletion, String>
where
    F: FnMut(&str),
{
    if profile.api_key.trim().is_empty() {
        return Err(format!(
            "Profile '{}' does not have an {} API key configured",
            profile.profile_id,
            provider.display_name()
        ));
    }

    let model = resolve_effective_model(profile, request.model_override.as_deref())?;
    let request_body = build_openai_compat_request(&request.messages, &model)?;
    let url = format!("{}/chat/completions", api_base.trim_end_matches('/'));
    let client = HttpClient::new();
    let response = client
        .post(url)
        .bearer_auth(profile.api_key.as_str())
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format_openai_compat_error(provider, status, &body));
    }

    let mut response = response;
    let mut buffer = Vec::new();
    let mut full_text = String::new();
    let mut last_usage = None;

    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        buffer.extend_from_slice(&chunk);

        while let Some((boundary, delimiter_len)) = find_event_boundary(&buffer) {
            let event_bytes = buffer[..boundary].to_vec();
            buffer.drain(..boundary + delimiter_len);

            let Some(data) = parse_sse_data(&event_bytes)? else {
                continue;
            };

            let parsed: OpenAiCompatChatChunk =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;

            if let Some(usage) = parsed.usage.as_ref() {
                last_usage = Some(map_openai_compat_usage(usage));
            }

            let delta = extract_openai_compat_delta(&parsed);
            if !delta.is_empty() {
                full_text.push_str(&delta);
                on_delta(&delta);
            }
        }
    }

    if !buffer.is_empty() {
        if let Some(data) = parse_sse_data(&buffer)? {
            let parsed: OpenAiCompatChatChunk =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            if let Some(usage) = parsed.usage.as_ref() {
                last_usage = Some(map_openai_compat_usage(usage));
            }
            let delta = extract_openai_compat_delta(&parsed);
            if !delta.is_empty() {
                full_text.push_str(&delta);
                on_delta(&delta);
            }
        }
    }

    Ok(LlmCompletion {
        provider: profile.provider.as_str().to_string(),
        model,
        text: full_text,
        usage: last_usage,
    })
}

async fn list_gemini_models(api_key: &str) -> Result<Vec<LlmProviderModel>, String> {
    if api_key.trim().is_empty() {
        return Err("Gemini API key is required to load available models".to_string());
    }

    let client = HttpClient::new();
    let mut page_token: Option<String> = None;
    let mut models = Vec::new();

    loop {
        let mut request = client
            .get(format!("{GEMINI_API_BASE}/models"))
            .header("x-goog-api-key", api_key)
            .query(&[("pageSize", "1000")]);

        if let Some(token) = page_token.as_deref() {
            request = request.query(&[("pageToken", token)]);
        }

        let response = request.send().await.map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format_google_error(status, &body));
        }

        let parsed: GeminiListModelsResponse = response.json().await.map_err(|e| e.to_string())?;
        models.extend(
            parsed
                .models
                .unwrap_or_default()
                .into_iter()
                .map(map_gemini_model)
                .filter(|model| {
                    model
                        .supported_generation_methods
                        .iter()
                        .any(|method| method == "generateContent")
                }),
        );

        page_token = parsed
            .next_page_token
            .filter(|token| !token.trim().is_empty());
        if page_token.is_none() {
            break;
        }
    }

    models.sort_by(|left, right| {
        left.display_name
            .to_ascii_lowercase()
            .cmp(&right.display_name.to_ascii_lowercase())
            .then_with(|| left.model.cmp(&right.model))
    });

    Ok(models)
}

async fn list_openai_compat_models(
    api_key: &str,
    api_base: &str,
    provider: ProviderKind,
) -> Result<Vec<LlmProviderModel>, String> {
    if api_key.trim().is_empty() {
        return Err(format!(
            "{} API key is required to load available models",
            provider.display_name()
        ));
    }

    let client = HttpClient::new();
    let response = client
        .get(format!("{}/models", api_base.trim_end_matches('/')))
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format_openai_compat_error(provider, status, &body));
    }

    let parsed: OpenAiCompatModelsResponse = response.json().await.map_err(|e| e.to_string())?;
    let mut models: Vec<_> = parsed
        .data
        .into_iter()
        .map(map_openai_compat_model)
        .collect();

    models.sort_by(|left, right| {
        left.display_name
            .to_ascii_lowercase()
            .cmp(&right.display_name.to_ascii_lowercase())
            .then_with(|| left.model.cmp(&right.model))
    });

    Ok(models)
}

pub(crate) async fn run_llm_collect_with_profile(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
) -> Result<LlmCompletion, String> {
    validate_request(request)?;

    let result = timeout(
        Duration::from_secs(LLM_STREAM_TIMEOUT_SECS),
        stream_with_provider(request, profile, &mut |_| {}),
    )
    .await;

    match result {
        Ok(result) => result,
        Err(_) => Err(format!(
            "LLM request timed out after {LLM_STREAM_TIMEOUT_SECS} seconds"
        )),
    }
}

pub(crate) async fn run_llm_stream_with_profile<F>(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
    mut on_delta: F,
) -> Result<LlmCompletion, String>
where
    F: FnMut(&str),
{
    validate_request(request)?;

    let result = timeout(
        Duration::from_secs(LLM_STREAM_TIMEOUT_SECS),
        stream_with_provider(request, profile, &mut on_delta),
    )
    .await;

    match result {
        Ok(result) => result,
        Err(_) => Err(format!(
            "LLM request timed out after {LLM_STREAM_TIMEOUT_SECS} seconds"
        )),
    }
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
    set_active: Option<bool>,
) -> AppResult<LlmProfilesState> {
    let pool = get_pool(&handle).await?;
    let (profile_id, provider_kind, default_model) =
        validate_profile_input(profile_id, provider, default_model)?;
    let set_active = set_active.unwrap_or(true);

    save_profile_to_pool(
        &pool,
        &profile_id,
        provider_kind.as_str(),
        &default_model,
        api_key.trim(),
        set_active,
    )
    .await?;

    Ok(load_profiles_state_from_pool(&pool).await?)
}

#[tauri::command]
pub async fn list_llm_provider_models(
    handle: AppHandle,
    provider: String,
    profile_id: Option<String>,
    api_key: Option<String>,
) -> AppResult<Vec<LlmProviderModel>> {
    let provider_kind = ProviderKind::parse(&provider)?;
    let configured_key = api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    let api_key = if let Some(key) = configured_key {
        key
    } else {
        let pool = get_pool(&handle).await?;
        let profile = resolve_profile_from_pool(&pool, profile_id.as_deref()).await?;
        profile.api_key
    };

    let timeout_secs = match provider_kind {
        ProviderKind::Gemini => GEMINI_MODELS_TIMEOUT_SECS,
        ProviderKind::OmniRoute => OPENAI_COMPAT_MODELS_TIMEOUT_SECS,
    };

    let result = timeout(Duration::from_secs(timeout_secs), async move {
        match provider_kind {
            ProviderKind::Gemini => list_gemini_models(&api_key).await,
            ProviderKind::OmniRoute => {
                list_openai_compat_models(&api_key, OMNIROUTE_API_BASE, ProviderKind::OmniRoute)
                    .await
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

#[cfg(test)]
mod tests {
    use super::{
        build_gemini_request, build_openai_compat_request, extract_openai_compat_delta,
        extract_text, find_event_boundary, load_profiles_state_from_pool, map_gemini_model,
        map_openai_compat_model, map_openai_compat_usage, map_usage, parse_sse_data,
        resolve_profile_from_pool, save_profile_to_pool, GeminiContent,
        GeminiGenerateContentResponse, GeminiModel, GeminiPart, LlmMessage, OpenAiCompatChatChunk,
        OpenAiCompatModel,
    };

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

    #[test]
    fn gemini_request_mapping_keeps_system_history_and_roles() {
        let request = build_gemini_request(&[
            LlmMessage {
                role: "system".to_string(),
                content: "You are concise.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            LlmMessage {
                role: "assistant".to_string(),
                content: "Hi there".to_string(),
            },
        ])
        .expect("build request");

        assert_eq!(
            request.system_instruction,
            Some(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: "You are concise.".to_string()
                }]
            })
        );
        assert_eq!(request.contents[0].role, "user");
        assert_eq!(request.contents[1].role, "model");
    }

    #[test]
    fn sse_data_and_usage_are_parsed_from_stream_chunks() {
        let frame = b"data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"text\":\"Hello\"}]}}],\"usageMetadata\":{\"promptTokenCount\":3,\"candidatesTokenCount\":4,\"totalTokenCount\":7}}\n\n";
        let (boundary, delimiter) = find_event_boundary(frame).expect("find boundary");
        assert_eq!(delimiter, 2);
        let payload = parse_sse_data(&frame[..boundary])
            .expect("parse sse")
            .expect("payload");
        let parsed: GeminiGenerateContentResponse =
            serde_json::from_str(&payload).expect("parse response");

        assert_eq!(extract_text(&parsed), "Hello");
        let usage = map_usage(&parsed.usage_metadata.expect("usage"));
        assert_eq!(usage.input_tokens, Some(3));
        assert_eq!(usage.output_tokens, Some(4));
        assert_eq!(usage.total_tokens, Some(7));
    }

    #[test]
    fn gemini_model_mapping_uses_short_model_id() {
        let model = map_gemini_model(GeminiModel {
            name: "models/gemini-2.5-flash".to_string(),
            display_name: Some("Gemini 2.5 Flash".to_string()),
            description: Some("Fast model".to_string()),
            input_token_limit: Some(1_048_576),
            output_token_limit: Some(65_536),
            supported_generation_methods: Some(vec![
                "generateContent".to_string(),
                "countTokens".to_string(),
            ]),
        });

        assert_eq!(model.model, "gemini-2.5-flash");
        assert_eq!(model.name, "models/gemini-2.5-flash");
        assert_eq!(model.display_name, "Gemini 2.5 Flash");
        assert!(model
            .supported_generation_methods
            .contains(&"generateContent".to_string()));
    }

    #[test]
    fn openai_compat_request_keeps_standard_roles() {
        let request = build_openai_compat_request(
            &[
                LlmMessage {
                    role: "system".to_string(),
                    content: "You are concise.".to_string(),
                },
                LlmMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            ],
            "if/kimi-k2-thinking",
        )
        .expect("build request");

        assert_eq!(request.model, "if/kimi-k2-thinking");
        assert!(request.stream);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
    }

    #[test]
    fn openai_compat_stream_chunk_mapping_reads_delta_and_usage() {
        let payload = r#"{"choices":[{"delta":{"content":"Hello"}}],"usage":{"prompt_tokens":3,"completion_tokens":4,"total_tokens":7}}"#;
        let parsed: OpenAiCompatChatChunk = serde_json::from_str(payload).expect("parse chunk");

        assert_eq!(extract_openai_compat_delta(&parsed), "Hello");
        let usage = map_openai_compat_usage(&parsed.usage.expect("usage"));
        assert_eq!(usage.input_tokens, Some(3));
        assert_eq!(usage.output_tokens, Some(4));
        assert_eq!(usage.total_tokens, Some(7));
    }

    #[test]
    fn openai_compat_model_mapping_uses_model_id() {
        let model = map_openai_compat_model(OpenAiCompatModel {
            id: "gg/gemini-2.5-pro".to_string(),
            object: Some("model".to_string()),
            owned_by: Some("omniroute".to_string()),
        });

        assert_eq!(model.model, "gg/gemini-2.5-pro");
        assert_eq!(model.name, "gg/gemini-2.5-pro");
        assert_eq!(model.display_name, "gg/gemini-2.5-pro");
        assert_eq!(model.description, "omniroute");
    }
}
