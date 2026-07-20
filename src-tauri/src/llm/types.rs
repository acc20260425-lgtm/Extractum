use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use super::ProviderKind;

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
    pub max_output_tokens: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmUsage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
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
pub struct LlmProviderAccess {
    provider: ProviderKind,
    api_key: SecretString,
    base_url: String,
}

impl LlmProviderAccess {
    pub fn new(provider: ProviderKind, api_key: SecretString, base_url: String) -> Self {
        Self {
            provider,
            api_key,
            base_url,
        }
    }

    pub(super) fn provider(&self) -> ProviderKind {
        self.provider
    }

    pub(super) fn api_key(&self) -> &SecretString {
        &self.api_key
    }

    pub(super) fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[derive(Clone)]
pub struct ResolvedLlmProfile {
    profile_id: String,
    default_model: String,
    provider_access: LlmProviderAccess,
}

impl ResolvedLlmProfile {
    pub fn new(
        profile_id: String,
        default_model: String,
        provider_access: LlmProviderAccess,
    ) -> Self {
        Self {
            profile_id,
            default_model,
            provider_access,
        }
    }

    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    pub fn provider(&self) -> ProviderKind {
        self.provider_access.provider()
    }

    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    pub fn base_url(&self) -> &str {
        self.provider_access.base_url()
    }

    pub(super) fn provider_access(&self) -> &LlmProviderAccess {
        &self.provider_access
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmCompletion {
    pub provider: String,
    pub model: String,
    pub text: String,
    pub usage: Option<LlmUsage>,
}

#[cfg(test)]
mod tests {
    use secrecy::{ExposeSecret, SecretString};

    use super::super::ProviderKind;
    use super::{LlmProviderAccess, ResolvedLlmProfile};

    #[test]
    fn resolved_profile_construction_preserves_execution_access_and_public_metadata() {
        let profile = ResolvedLlmProfile::new(
            "research".to_string(),
            "gemini-2.5-flash".to_string(),
            LlmProviderAccess::new(
                ProviderKind::Gemini,
                SecretString::new("secret-key".to_string()),
                String::new(),
            ),
        );

        assert_eq!(profile.profile_id(), "research");
        assert_eq!(profile.provider(), ProviderKind::Gemini);
        assert_eq!(profile.default_model(), "gemini-2.5-flash");
        assert_eq!(profile.base_url(), "");
        assert_eq!(
            profile.provider_access().api_key().expose_secret(),
            "secret-key",
        );
    }
}
