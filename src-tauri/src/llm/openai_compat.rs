use reqwest::Client as HttpClient;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use extractum_core::error::{AppError, AppResult};

use super::streaming::{find_event_boundary, parse_sse_data};
use super::{
    resolve_effective_model, LlmChatRequest, LlmCompletion, LlmProviderModel, LlmUsage,
    ProviderKind, ResolvedLlmProfile,
};

const OPENAI_COMPAT_STREAM_MAX_ATTEMPTS: usize = 3;
const OPENAI_COMPAT_RETRY_DELAY_MS: u64 = 600;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i64>,
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
    context_length: Option<i64>,
    max_output_tokens: Option<i64>,
    capabilities: Option<std::collections::BTreeMap<String, bool>>,
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
    request: &LlmChatRequest,
    model: &str,
) -> AppResult<OpenAiCompatChatRequest> {
    let mut mapped_messages = Vec::new();

    for message in &request.messages {
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
        max_tokens: request.max_output_tokens,
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

fn is_retryable_openai_compat_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 429 | 500 | 502 | 503 | 504)
}

fn map_openai_compat_model(model: OpenAiCompatModel) -> LlmProviderModel {
    let description = model.owned_by.unwrap_or_default();
    let generation_method = model.object.unwrap_or_else(|| "model".to_string());
    let mut supported_generation_methods = model
        .capabilities
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(capability, enabled)| enabled.then_some(capability))
        .collect::<Vec<_>>();

    if supported_generation_methods.is_empty() {
        supported_generation_methods.push(generation_method);
    }

    LlmProviderModel {
        display_name: model.id.clone(),
        description,
        input_token_limit: model.context_length,
        output_token_limit: model.max_output_tokens,
        supported_generation_methods,
        model: model.id.clone(),
        name: model.id,
    }
}

pub(super) async fn stream_openai_compat_response<F>(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
    on_delta: &mut F,
    config: &OpenAiCompatProviderConfig,
) -> AppResult<LlmCompletion>
where
    F: FnMut(&str),
{
    let access = profile.provider_access();
    if access.api_key().expose_secret().trim().is_empty() {
        return Err(AppError::auth(format!(
            "Profile '{}' does not have an {} API key configured",
            profile.profile_id(),
            config.provider.display_name()
        )));
    }

    let model = resolve_effective_model(profile, request.model_override.as_deref())?;
    let request_body = build_openai_compat_request(request, &model)?;
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let client = HttpClient::new();
    let mut response = None;

    for attempt in 1..=OPENAI_COMPAT_STREAM_MAX_ATTEMPTS {
        let candidate = match client
            .post(url.clone())
            .bearer_auth(access.api_key().expose_secret())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            Ok(candidate) => candidate,
            Err(_) if attempt < OPENAI_COMPAT_STREAM_MAX_ATTEMPTS => {
                sleep(Duration::from_millis(
                    OPENAI_COMPAT_RETRY_DELAY_MS * attempt as u64,
                ))
                .await;
                continue;
            }
            Err(error) => return Err(AppError::llm_network(error)),
        };

        if candidate.status().is_success() {
            response = Some(candidate);
            break;
        }

        let status = candidate.status();
        let body = candidate.text().await.unwrap_or_default();
        let error = format_openai_compat_error(config, status, &body);

        if is_retryable_openai_compat_status(status) && attempt < OPENAI_COMPAT_STREAM_MAX_ATTEMPTS
        {
            sleep(Duration::from_millis(
                OPENAI_COMPAT_RETRY_DELAY_MS * attempt as u64,
            ))
            .await;
            continue;
        }

        return Err(AppError::llm_network(error));
    }

    let mut response = response.ok_or_else(|| {
        AppError::llm_network("OpenAI-compatible request failed before streaming")
    })?;
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
        provider: profile.provider().as_str().to_string(),
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
        map_openai_compat_model, map_openai_compat_usage, stream_openai_compat_response,
        OpenAiCompatChatChunk, OpenAiCompatModel, OpenAiCompatModelsResponse,
        OpenAiCompatProviderConfig,
    };
    use extractum_core::error::AppErrorKind;

    use super::super::{
        LlmChatRequest, LlmMessage, LlmProviderAccess, ProviderKind, ResolvedLlmProfile,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    };

    async fn read_http_request(socket: &mut TcpStream) {
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let read = socket.read(&mut chunk).await.expect("read request");
            if read == 0 {
                return;
            }
            buffer.extend_from_slice(&chunk[..read]);
            if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                break position + 4;
            }
        };

        let headers = String::from_utf8_lossy(&buffer[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);

        let body_read = buffer.len().saturating_sub(header_end);
        if content_length > body_read {
            let mut body_tail = vec![0_u8; content_length - body_read];
            socket
                .read_exact(&mut body_tail)
                .await
                .expect("read request body");
        }
    }

    async fn start_transient_openai_compat_server() -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let base_url = format!("http://{}", listener.local_addr().expect("server address"));
        let attempts = Arc::new(AtomicUsize::new(0));
        let server_attempts = Arc::clone(&attempts);

        tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.expect("accept request");
                read_http_request(&mut socket).await;
                let attempt = server_attempts.fetch_add(1, Ordering::SeqCst) + 1;

                if attempt == 1 {
                    let body = r#"{"error":{"message":"temporary outage","type":"server_error"}}"#;
                    let response = format!(
                        "HTTP/1.1 500 Internal Server Error\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket
                        .write_all(response.as_bytes())
                        .await
                        .expect("write transient response");
                    continue;
                }

                let body = "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n";
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write success response");
                break;
            }
        });

        (base_url, attempts)
    }

    #[test]
    fn openai_compat_request_keeps_standard_roles() {
        let request = build_openai_compat_request(
            &LlmChatRequest {
                request_id: "openai-compat-test".to_string(),
                profile_id: None,
                model_override: None,
                max_output_tokens: Some(4096),
                messages: vec![
                    LlmMessage {
                        role: "system".to_string(),
                        content: "You are concise.".to_string(),
                    },
                    LlmMessage {
                        role: "user".to_string(),
                        content: "Hello".to_string(),
                    },
                ],
            },
            "if/kimi-k2-thinking",
        )
        .expect("build request");

        assert_eq!(request.model, "if/kimi-k2-thinking");
        assert!(request.stream);
        let serialized = serde_json::to_value(&request).expect("serialize request");
        assert_eq!(serialized["max_tokens"], 4096);
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
            context_length: None,
            max_output_tokens: None,
            capabilities: None,
        });

        assert_eq!(model.model, "gg/gemini-2.5-pro");
        assert_eq!(model.name, "gg/gemini-2.5-pro");
        assert_eq!(model.display_name, "gg/gemini-2.5-pro");
        assert_eq!(model.description, "omniroute");
    }

    #[test]
    fn openai_compat_model_mapping_reads_omniroute_limits_and_capabilities() {
        let response: OpenAiCompatModelsResponse = serde_json::from_str(
            r#"{
                "data": [{
                    "id": "gemini/gemini-2.5-flash",
                    "object": "model",
                    "owned_by": "gemini",
                    "context_length": 1048576,
                    "max_output_tokens": 8192,
                    "capabilities": {
                        "tool_calling": true,
                        "reasoning": true
                    }
                }]
            }"#,
        )
        .expect("parse OmniRoute model metadata");

        let model = map_openai_compat_model(
            response
                .data
                .into_iter()
                .next()
                .expect("model metadata exists"),
        );

        assert_eq!(model.input_token_limit, Some(1_048_576));
        assert_eq!(model.output_token_limit, Some(8_192));
        assert_eq!(
            model.supported_generation_methods,
            vec!["reasoning".to_string(), "tool_calling".to_string()]
        );
    }

    #[test]
    fn openai_compat_request_rejects_unsupported_roles_with_typed_validation_error() {
        let error = match build_openai_compat_request(
            &LlmChatRequest {
                request_id: "unsupported-role".to_string(),
                profile_id: None,
                model_override: None,
                max_output_tokens: None,
                messages: vec![LlmMessage {
                    role: "tool".to_string(),
                    content: "lookup".to_string(),
                }],
            },
            "if/kimi-k2-thinking",
        ) {
            Ok(_) => panic!("unsupported role should fail"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Unsupported message role 'tool'");
    }

    #[test]
    fn openai_compat_retry_status_policy_is_bounded_to_transient_failures() {
        for status in [429, 500, 502, 503, 504] {
            assert!(
                super::is_retryable_openai_compat_status(
                    reqwest::StatusCode::from_u16(status).expect("valid status")
                ),
                "{status} should be retryable"
            );
        }

        for status in [400, 401, 403, 404] {
            assert!(
                !super::is_retryable_openai_compat_status(
                    reqwest::StatusCode::from_u16(status).expect("valid status")
                ),
                "{status} should not be retryable"
            );
        }
    }

    #[tokio::test]
    async fn openai_compat_stream_retries_transient_http_before_streaming() {
        let (base_url, attempts) = start_transient_openai_compat_server().await;
        let config = OpenAiCompatProviderConfig {
            provider: ProviderKind::OpenAiCompatible,
            base_url,
        };
        let profile = ResolvedLlmProfile::new(
            "default".to_string(),
            "if/kimi-k2-thinking".to_string(),
            LlmProviderAccess::new(
                ProviderKind::OpenAiCompatible,
                "test-key".to_string().into(),
                config.base_url.clone(),
            ),
        );
        let request = LlmChatRequest {
            request_id: "retry-test".to_string(),
            profile_id: None,
            messages: vec![LlmMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            model_override: None,
            max_output_tokens: None,
        };
        let mut deltas = Vec::new();

        let completion = stream_openai_compat_response(
            &request,
            &profile,
            &mut |delta| {
                deltas.push(delta.to_string());
            },
            &config,
        )
        .await
        .expect("retry transient response");

        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(completion.provider, "openai_compatible");
        assert_eq!(completion.text, "ok");
        assert_eq!(deltas, vec!["ok"]);
    }

    #[tokio::test]
    async fn openai_compat_model_listing_requires_typed_auth_error() {
        let config = OpenAiCompatProviderConfig {
            provider: ProviderKind::OpenAiCompatible,
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
