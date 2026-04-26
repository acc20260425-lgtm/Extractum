use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use super::streaming::{find_event_boundary, parse_sse_data};
use super::{resolve_effective_model, LlmChatRequest, LlmCompletion, LlmMessage, LlmProviderModel};
use super::{LlmUsage, ResolvedLlmProfile};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

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

fn map_usage(usage: &GeminiUsageMetadata) -> LlmUsage {
    LlmUsage {
        input_tokens: usage.prompt_token_count,
        output_tokens: usage.candidates_token_count,
        total_tokens: usage.total_token_count,
    }
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

pub(super) async fn stream_gemini_response<F>(
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

pub(super) async fn list_gemini_models(api_key: &str) -> Result<Vec<LlmProviderModel>, String> {
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

#[cfg(test)]
mod tests {
    use super::{
        build_gemini_request, extract_text, map_gemini_model, map_usage, GeminiContent,
        GeminiGenerateContentResponse, GeminiModel, GeminiPart,
    };
    use crate::llm::LlmMessage;

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
    fn gemini_stream_chunk_text_and_usage_are_parsed() {
        let payload = r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"Hello"}]}}],"usageMetadata":{"promptTokenCount":3,"candidatesTokenCount":4,"totalTokenCount":7}}"#;
        let parsed: GeminiGenerateContentResponse =
            serde_json::from_str(payload).expect("parse response");

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
}
