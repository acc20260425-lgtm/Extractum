use std::{future::Future, pin::Pin, time::Duration};

use super::{
    error::GeminiBrowserResult,
    types::{
        GeminiBrowserProviderConfig, GeminiBrowserProviderStatus, GeminiBrowserRunRequest,
        GeminiBrowserRunResult,
    },
};

pub type BrowserExecutorFuture<'a, T> =
    Pin<Box<dyn Future<Output = GeminiBrowserResult<T>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserSessionContext {
    pub browser_profile_dir: String,
    pub browser_config: Option<GeminiBrowserProviderConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserRunContext {
    pub request: GeminiBrowserRunRequest,
    pub browser_profile_dir: String,
    pub artifact_dir: String,
    pub browser_config: Option<GeminiBrowserProviderConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrowserStopReason {
    Requested,
    Cancelled { run_id: String },
    TimedOut { run_id: String, timeout: Duration },
}

pub trait BrowserExecutor: Send + Sync {
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

pub trait StatusObserver: Send + Sync {
    fn publish(&self, status: &GeminiBrowserProviderStatus);
}
