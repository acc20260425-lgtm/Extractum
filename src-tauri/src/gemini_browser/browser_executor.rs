use std::{future::Future, pin::Pin, time::Duration};

use super::{
    domain_error::GeminiBrowserResult,
    types::{
        GeminiBrowserProviderConfig, GeminiBrowserProviderStatus, GeminiBrowserRunRequest,
        GeminiBrowserRunResult,
    },
};

pub(crate) type BrowserExecutorFuture<'a, T> =
    Pin<Box<dyn Future<Output = GeminiBrowserResult<T>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BrowserSessionContext {
    pub(crate) browser_profile_dir: String,
    pub(crate) browser_config: Option<GeminiBrowserProviderConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BrowserRunContext {
    pub(crate) request: GeminiBrowserRunRequest,
    pub(crate) browser_profile_dir: String,
    pub(crate) artifact_dir: String,
    pub(crate) browser_config: Option<GeminiBrowserProviderConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BrowserStopReason {
    Requested,
    Cancelled { run_id: String },
    TimedOut { run_id: String, timeout: Duration },
}

pub(crate) trait BrowserExecutor: Send + Sync {
    fn status(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus>;
    fn open(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus>;
    fn resume(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus>;
    fn send(&self, context: BrowserRunContext)
        -> BrowserExecutorFuture<'_, GeminiBrowserRunResult>;
    fn stop(&self, reason: BrowserStopReason) -> BrowserExecutorFuture<'_, ()>;
}

pub(crate) trait StatusObserver: Send + Sync {
    fn publish(&self, status: &GeminiBrowserProviderStatus);
}
