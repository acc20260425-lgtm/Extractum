use serde::{Deserialize, Serialize};

use super::LlmUsage;

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
