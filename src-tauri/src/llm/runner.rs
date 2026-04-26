use tokio::time::{timeout, Duration};

use super::gemini::stream_gemini_response;
use super::openai_compat::stream_openai_compat_response;
use super::{LlmChatRequest, LlmCompletion, ProviderKind, ResolvedLlmProfile, OMNIROUTE_CONFIG};

const LLM_STREAM_TIMEOUT_SECS: u64 = 90;

pub(crate) fn validate_request(request: &LlmChatRequest) -> Result<(), String> {
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
            stream_openai_compat_response(request, profile, on_delta, OMNIROUTE_CONFIG).await
        }
    }
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
