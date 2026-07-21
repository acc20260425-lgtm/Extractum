use std::{future::Future, pin::Pin};

use extractum_core::error::AppResult;
use extractum_gemini_browser::{
    GeminiBrowserProviderConfig, GeminiBrowserProviderStatus, GeminiBrowserRunResult,
};

pub type PromptPackBrowserFuture<'a, T> = Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

pub trait PromptPackBrowserExecutor: Send + Sync + 'static {
    fn read_status(
        &self,
        request: PromptPackBrowserStatusRequest,
    ) -> PromptPackBrowserFuture<'_, GeminiBrowserProviderStatus>;

    fn submit(
        &self,
        request: PromptPackBrowserRunRequest,
    ) -> PromptPackBrowserFuture<'_, GeminiBrowserRunResult>;

    fn cancel(&self, request: PromptPackBrowserCancelRequest) -> PromptPackBrowserFuture<'_, ()>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackBrowserStatusRequest {
    provider_config: Option<GeminiBrowserProviderConfig>,
}

impl PromptPackBrowserStatusRequest {
    pub fn new(provider_config: Option<GeminiBrowserProviderConfig>) -> Self {
        Self { provider_config }
    }

    pub fn provider_config(&self) -> Option<&GeminiBrowserProviderConfig> {
        self.provider_config.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackBrowserRunRequest {
    run_id: String,
    prompt: String,
    source: String,
    artifact_mode: String,
    provider_config: Option<GeminiBrowserProviderConfig>,
}

impl PromptPackBrowserRunRequest {
    pub fn new(
        run_id: String,
        prompt: String,
        source: String,
        artifact_mode: String,
        provider_config: Option<GeminiBrowserProviderConfig>,
    ) -> Self {
        Self {
            run_id,
            prompt,
            source,
            artifact_mode,
            provider_config,
        }
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn artifact_mode(&self) -> &str {
        &self.artifact_mode
    }

    pub fn provider_config(&self) -> Option<&GeminiBrowserProviderConfig> {
        self.provider_config.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackBrowserCancelRequest {
    run_id: String,
}

impl PromptPackBrowserCancelRequest {
    pub fn new(run_id: String) -> Self {
        Self { run_id }
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }
}
