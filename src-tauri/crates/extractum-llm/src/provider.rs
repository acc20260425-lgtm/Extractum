use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use tokio::time::{timeout, Duration};

use extractum_core::error::{AppError, AppResult};

use super::gemini::list_gemini_models;
use super::openai_compat::{list_openai_compat_models, OpenAiCompatProviderConfig};
use super::{LlmProviderAccess, LlmProviderModel, ResolvedLlmProfile};

const DEFAULT_OPENAI_COMPAT_BASE_URL: &str = "http://localhost:20128/v1";
const GEMINI_MODELS_TIMEOUT_SECS: u64 = 30;
const OPENAI_COMPAT_MODELS_TIMEOUT_SECS: u64 = 30;
const MODEL_LIMIT_LOOKUP_TIMEOUT_SECS: u64 = 5;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Gemini,
    OpenAiCompatible,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenAiCompatible => "openai_compatible",
        }
    }

    pub fn parse(value: &str) -> AppResult<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "gemini" => Ok(Self::Gemini),
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

pub fn normalize_base_url(provider: ProviderKind, base_url: Option<&str>) -> AppResult<String> {
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

async fn list_provider_models_without_timeout(
    access: &LlmProviderAccess,
) -> AppResult<Vec<LlmProviderModel>> {
    match access.provider() {
        ProviderKind::Gemini => list_gemini_models(access.api_key().expose_secret()).await,
        ProviderKind::OpenAiCompatible => {
            let config = OpenAiCompatProviderConfig {
                provider: access.provider(),
                base_url: access.base_url().to_string(),
            };
            list_openai_compat_models(access.api_key().expose_secret(), &config).await
        }
    }
}

pub async fn list_provider_models(access: &LlmProviderAccess) -> AppResult<Vec<LlmProviderModel>> {
    let timeout_secs = match access.provider() {
        ProviderKind::Gemini => GEMINI_MODELS_TIMEOUT_SECS,
        ProviderKind::OpenAiCompatible => OPENAI_COMPAT_MODELS_TIMEOUT_SECS,
    };

    timeout(
        Duration::from_secs(timeout_secs),
        list_provider_models_without_timeout(access),
    )
    .await
    .map_err(|_| {
        AppError::llm_network(format!(
            "Loading {} models timed out after {timeout_secs} seconds",
            access.provider().display_name(),
        ))
    })?
}

pub async fn resolve_model_input_token_limit(
    profile: &ResolvedLlmProfile,
    model: &str,
) -> Option<usize> {
    let result = timeout(
        Duration::from_secs(MODEL_LIMIT_LOOKUP_TIMEOUT_SECS),
        list_provider_models_without_timeout(profile.provider_access()),
    )
    .await;

    match result {
        Ok(Ok(models)) => model_input_token_limit_from_models(&models, model),
        Ok(Err(_)) | Err(_) => None,
    }
}

pub async fn resolve_model_output_token_limit(
    profile: &ResolvedLlmProfile,
    model: &str,
) -> Option<i64> {
    let result = timeout(
        Duration::from_secs(MODEL_LIMIT_LOOKUP_TIMEOUT_SECS),
        list_provider_models_without_timeout(profile.provider_access()),
    )
    .await;

    match result {
        Ok(Ok(models)) => model_output_token_limit_from_models(&models, model),
        Ok(Err(_)) | Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        model_input_token_limit_from_models, model_output_token_limit_from_models,
        normalize_base_url, ProviderKind,
    };
    use extractum_core::error::AppErrorKind;

    use super::super::LlmProviderModel;

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
                model: "gemini-2.5-pro".into(),
                name: "models/gemini-2.5-pro".into(),
                display_name: "Gemini 2.5 Pro".into(),
                description: String::new(),
                input_token_limit: Some(1_048_576),
                output_token_limit: None,
                supported_generation_methods: Vec::new(),
            },
            LlmProviderModel {
                model: "broken".into(),
                name: "broken".into(),
                display_name: "Broken".into(),
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
                model: "gemini-2.5-pro".into(),
                name: "models/gemini-2.5-pro".into(),
                display_name: "Gemini 2.5 Pro".into(),
                description: String::new(),
                input_token_limit: None,
                output_token_limit: Some(65_536),
                supported_generation_methods: Vec::new(),
            },
            LlmProviderModel {
                model: "broken".into(),
                name: "broken".into(),
                display_name: "Broken".into(),
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
}
