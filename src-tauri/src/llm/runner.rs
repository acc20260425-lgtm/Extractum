use tokio::time::{timeout, Duration};

use crate::error::{AppError, AppResult};

use super::gemini::stream_gemini_response;
use super::openai_compat::{stream_openai_compat_response, OpenAiCompatProviderConfig};
use super::{LlmChatRequest, LlmCompletion, ProviderKind, ResolvedLlmProfile};

const LLM_STREAM_TIMEOUT_SECS: u64 = 90;

pub(crate) fn validate_request(request: &LlmChatRequest) -> AppResult<()> {
    if request.request_id.trim().is_empty() {
        return Err(AppError::validation("request_id cannot be empty"));
    }
    if request.messages.is_empty() {
        return Err(AppError::validation("At least one message is required"));
    }
    if request
        .messages
        .iter()
        .all(|message| message.content.trim().is_empty())
    {
        return Err(AppError::validation("Messages cannot all be empty"));
    }

    Ok(())
}

pub(crate) fn resolve_effective_model(
    profile: &ResolvedLlmProfile,
    model_override: Option<&str>,
) -> AppResult<String> {
    let model = model_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(profile.default_model.as_str())
        .trim()
        .to_string();

    if model.is_empty() {
        return Err(AppError::validation("Model override cannot be empty"));
    }

    Ok(model)
}

async fn stream_with_provider<F>(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
    on_delta: &mut F,
) -> AppResult<LlmCompletion>
where
    F: FnMut(&str),
{
    match profile.provider {
        ProviderKind::Gemini => stream_gemini_response(request, profile, on_delta).await,
        ProviderKind::OpenAiCompatible => {
            let config = OpenAiCompatProviderConfig {
                provider: ProviderKind::OpenAiCompatible,
                base_url: profile.base_url.clone(),
            };
            stream_openai_compat_response(request, profile, on_delta, &config).await
        }
    }
}

pub(crate) async fn run_llm_collect_with_profile(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
) -> AppResult<LlmCompletion> {
    validate_request(request)?;

    let result = timeout(
        Duration::from_secs(LLM_STREAM_TIMEOUT_SECS),
        stream_with_provider(request, profile, &mut |_| {}),
    )
    .await;

    match result {
        Ok(result) => result,
        Err(_) => Err(AppError::network(format!(
            "LLM request timed out after {LLM_STREAM_TIMEOUT_SECS} seconds"
        ))),
    }
}

pub(crate) async fn run_llm_stream_with_profile<F>(
    request: &LlmChatRequest,
    profile: &ResolvedLlmProfile,
    mut on_delta: F,
) -> AppResult<LlmCompletion>
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
        Err(_) => Err(AppError::network(format!(
            "LLM request timed out after {LLM_STREAM_TIMEOUT_SECS} seconds"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_effective_model, run_llm_collect_with_profile, validate_request};
    use crate::error::AppErrorKind;
    use crate::llm::{LlmChatRequest, ProviderKind, ResolvedLlmProfile};

    #[test]
    fn validate_request_returns_typed_validation_error() {
        let request = LlmChatRequest {
            request_id: "   ".to_string(),
            profile_id: None,
            messages: vec![],
            model_override: None,
        };

        let error = validate_request(&request).expect_err("reject empty request id");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "request_id cannot be empty");
    }

    #[test]
    fn resolve_effective_model_returns_typed_validation_error() {
        let profile = ResolvedLlmProfile {
            profile_id: "default".to_string(),
            provider: ProviderKind::Gemini,
            default_model: "   ".to_string(),
            api_key: String::new(),
            base_url: String::new(),
        };

        let error = resolve_effective_model(&profile, None).expect_err("reject empty model");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Model override cannot be empty");
    }

    #[tokio::test]
    async fn run_llm_collect_returns_typed_validation_error() {
        let request = LlmChatRequest {
            request_id: "   ".to_string(),
            profile_id: None,
            messages: vec![],
            model_override: None,
        };
        let profile = ResolvedLlmProfile {
            profile_id: "default".to_string(),
            provider: ProviderKind::Gemini,
            default_model: "gemini-2.5-flash".to_string(),
            api_key: String::new(),
            base_url: String::new(),
        };

        let error = match run_llm_collect_with_profile(&request, &profile).await {
            Ok(_) => panic!("invalid request should fail before provider call"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "request_id cannot be empty");
    }
}
