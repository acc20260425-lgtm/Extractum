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
pub struct LlmStreamEvent {
    pub request_id: String,
    pub kind: String,
    pub queue_position: Option<usize>,
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
    pub api_key_configured: bool,
    pub base_url: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LlmProfilesState {
    pub active_profile: String,
    pub profiles: Vec<LlmProfile>,
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
    pub(crate) api_key: SecretString,
    pub(crate) base_url: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct LlmCompletion {
    pub provider: String,
    pub model: String,
    pub text: String,
    pub usage: Option<LlmUsage>,
}
