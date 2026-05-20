use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

use super::streaming::{find_event_boundary, parse_sse_data};
use super::{
    resolve_effective_model, LlmChatRequest, LlmCompletion, LlmMessage, LlmProviderModel, LlmUsage,
    ProviderKind, ResolvedLlmProfile,
};

#[derive(Clone)]
pub(super) struct OpenAiCompatProviderConfig {
    pub(super) provider: ProviderKind,
    pub(super) base_url: String,
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

fn build_openai_compat_request(
    messages: &[LlmMessage],
    model: &str,
) -> AppResult<OpenAiCompatChatRequest> {
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
                return Err(AppError::validation(format!(
                    "Unsupported message role '{other}'"
                )));
            }
        }
    }

    if mapped_messages.is_empty() {
        return Err(AppError::validation("At least one message is required"));
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

fn map_openai_compat_usage(usage: &OpenAiCompatUsage) -> LlmUsage {
    LlmUsage {
        input_tokens: usage.prompt_tokens,
        output_tokens: usage.completion_tokens,
        total_tokens: usage.total_tokens,
    }
}

fn format_openai_compat_error(
    config: &OpenAiCompatProviderConfig,
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
            config.provider.display_name(),
            status.as_u16()
        );
    }

    if body.trim().is_empty() {
        format!(
            "{} request failed with HTTP {}",
            config.provider.display_name(),
            status.as_u16()
        )
    } else {
        format!(
            "{} request failed with HTTP {}: {}",
            config.provider.display_name(),
            status.as_u16(),
            body.trim()
        )
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

pub(in crate::llm) async fn stream_openai_compat_response<F>(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
    on_delta: &mut F,
    config: &OpenAiCompatProviderConfig,
) -> AppResult<LlmCompletion>
where
    F: FnMut(&str),
{
    if profile.api_key.trim().is_empty() {
        return Err(AppError::auth(format!(
            "Profile '{}' does not have an {} API key configured",
            profile.profile_id,
            config.provider.display_name()
        )));
    }

    let model = resolve_effective_model(profile, request.model_override.as_deref())?;
    let request_body = build_openai_compat_request(&request.messages, &model)?;
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let client = HttpClient::new();
    let response = client
        .post(url)
        .bearer_auth(profile.api_key.as_str())
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(AppError::llm_network)?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::llm_network(format_openai_compat_error(
            config, status, &body,
        )));
    }

    let mut response = response;
    let mut buffer = Vec::new();
    let mut full_text = String::new();
    let mut last_usage = None;

    while let Some(chunk) = response.chunk().await.map_err(AppError::llm_network)? {
        buffer.extend_from_slice(&chunk);

        while let Some((boundary, delimiter_len)) = find_event_boundary(&buffer) {
            let event_bytes = buffer[..boundary].to_vec();
            buffer.drain(..boundary + delimiter_len);

            let Some(data) = parse_sse_data(&event_bytes)? else {
                continue;
            };

            let parsed: OpenAiCompatChatChunk =
                serde_json::from_str(&data).map_err(AppError::llm_network)?;

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
                serde_json::from_str(&data).map_err(AppError::llm_network)?;
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

pub(super) async fn list_openai_compat_models(
    api_key: &str,
    config: &OpenAiCompatProviderConfig,
) -> AppResult<Vec<LlmProviderModel>> {
    if api_key.trim().is_empty() {
        return Err(AppError::auth(format!(
            "{} API key is required to load available models",
            config.provider.display_name()
        )));
    }

    let client = HttpClient::new();
    let response = client
        .get(format!("{}/models", config.base_url.trim_end_matches('/')))
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(AppError::llm_network)?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::llm_network(format_openai_compat_error(
            config, status, &body,
        )));
    }

    let parsed: OpenAiCompatModelsResponse =
        response.json().await.map_err(AppError::llm_network)?;
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

#[cfg(test)]
mod tests {
    use super::{
        build_openai_compat_request, extract_openai_compat_delta, list_openai_compat_models,
        map_openai_compat_model, map_openai_compat_usage, OpenAiCompatChatChunk, OpenAiCompatModel,
        OpenAiCompatProviderConfig,
    };
    use crate::error::AppErrorKind;
    use crate::llm::LlmMessage;
    use crate::llm::ProviderKind;

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

    #[test]
    fn openai_compat_request_rejects_unsupported_roles_with_typed_validation_error() {
        let error = match build_openai_compat_request(
            &[LlmMessage {
                role: "tool".to_string(),
                content: "lookup".to_string(),
            }],
            "if/kimi-k2-thinking",
        ) {
            Ok(_) => panic!("unsupported role should fail"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Unsupported message role 'tool'");
    }

    #[tokio::test]
    async fn openai_compat_model_listing_requires_typed_auth_error() {
        let config = OpenAiCompatProviderConfig {
            provider: ProviderKind::OmniRoute,
            base_url: "http://localhost:20128/v1".to_string(),
        };

        let error = list_openai_compat_models("   ", &config)
            .await
            .expect_err("reject missing api key");

        assert_eq!(error.kind, AppErrorKind::Auth);
        assert_eq!(
            error.message,
            "OpenAI-compatible API key is required to load available models"
        );
    }
}
