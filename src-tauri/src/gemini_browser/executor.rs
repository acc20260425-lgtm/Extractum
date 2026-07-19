use tauri::AppHandle;

use crate::error::{AppError, AppErrorKind, AppResult};

use extractum_gemini_browser::{
    BrowserExecutor, BrowserExecutorFuture, BrowserRunContext, BrowserSessionContext,
    BrowserStopReason, GeminiBrowserError, GeminiBrowserErrorKind, StatusObserver,
};

use super::{sidecar, GeminiBrowserProviderStatus, GeminiBrowserState};

#[derive(Clone, Copy)]
pub(crate) enum DomainErrorContext {
    Persistence,
    Protocol,
    Transport,
    Browser,
    Invariant,
}

pub(crate) fn domain_error_to_app(error: GeminiBrowserError) -> AppError {
    match error.kind() {
        GeminiBrowserErrorKind::Validation => AppError::validation(error.message()),
        GeminiBrowserErrorKind::NotFound => AppError::not_found(error.message()),
        GeminiBrowserErrorKind::Conflict => AppError::conflict(error.message()),
        GeminiBrowserErrorKind::Persistence
        | GeminiBrowserErrorKind::Protocol
        | GeminiBrowserErrorKind::Transport
        | GeminiBrowserErrorKind::Browser
        | GeminiBrowserErrorKind::Timeout
        | GeminiBrowserErrorKind::Cancellation
        | GeminiBrowserErrorKind::Invariant => AppError::internal(error.message()),
    }
}

pub(crate) fn app_error_to_domain(
    error: AppError,
    context: DomainErrorContext,
) -> GeminiBrowserError {
    let message = error.message;
    match error.kind {
        AppErrorKind::Validation => GeminiBrowserError::validation(message),
        AppErrorKind::NotFound => GeminiBrowserError::not_found(message),
        AppErrorKind::Conflict => GeminiBrowserError::conflict(message),
        AppErrorKind::Auth | AppErrorKind::Network | AppErrorKind::Internal => match context {
            DomainErrorContext::Persistence => GeminiBrowserError::persistence(message),
            DomainErrorContext::Protocol => GeminiBrowserError::protocol(message),
            DomainErrorContext::Transport => GeminiBrowserError::transport(message),
            DomainErrorContext::Browser => GeminiBrowserError::browser(message),
            DomainErrorContext::Invariant => GeminiBrowserError::invariant(message),
        },
    }
}

pub(crate) struct AppBrowserExecutor<'a> {
    handle: &'a AppHandle,
    state: &'a GeminiBrowserState,
}

impl<'a> AppBrowserExecutor<'a> {
    pub(crate) fn new(handle: &'a AppHandle, state: &'a GeminiBrowserState) -> Self {
        Self { handle, state }
    }
}

impl BrowserExecutor for AppBrowserExecutor<'_> {
    fn status(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
        Box::pin(async move {
            sidecar::status(
                self.handle,
                self.state,
                context.browser_profile_dir,
                context.browser_config,
            )
            .await
        })
    }

    fn open(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
        Box::pin(async move {
            sidecar::open_browser(
                self.handle,
                self.state,
                context.browser_profile_dir,
                context.browser_config,
            )
            .await
        })
    }

    fn resume(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
        Box::pin(async move {
            sidecar::resume(
                self.handle,
                self.state,
                context.browser_profile_dir,
                context.browser_config,
            )
            .await
        })
    }

    fn send(
        &self,
        context: BrowserRunContext,
    ) -> BrowserExecutorFuture<'_, super::GeminiBrowserRunResult> {
        Box::pin(async move {
            sidecar::send_single(
                self.handle,
                self.state,
                context.request,
                context.browser_profile_dir,
                context.artifact_dir,
                context.browser_config,
            )
            .await
        })
    }

    fn stop(&self, reason: BrowserStopReason) -> BrowserExecutorFuture<'_, ()> {
        Box::pin(async move {
            let result = match reason {
                BrowserStopReason::Requested => {
                    stop_owned_browser_resources(self.handle, self.state).await
                }
                BrowserStopReason::Cancelled { .. } | BrowserStopReason::TimedOut { .. } => {
                    discard_abandoned_transport(self.state, |_| {}).await
                }
            };
            result.map_err(|error| app_error_to_domain(error, DomainErrorContext::Transport))
        })
    }
}

pub(crate) struct AppStatusObserver;

impl StatusObserver for AppStatusObserver {
    fn publish(&self, _status: &GeminiBrowserProviderStatus) {}
}

async fn discard_abandoned_transport(
    state: &GeminiBrowserState,
    before_discard: impl FnOnce(bool),
) -> AppResult<()> {
    state.mark_sidecar_tainted().await;
    before_discard(state.sidecar_tainted().await);
    let process = state.sidecar().await.take();
    drop(process);
    let cdp_result = stop_owned_cdp_chrome(state).await;
    state.clear_sidecar_taint().await;
    cdp_result
}

async fn stop_owned_cdp_chrome(state: &GeminiBrowserState) -> AppResult<()> {
    let Some(mut process) = state.cdp_chrome_process().await.take() else {
        return Ok(());
    };
    tokio::task::spawn_blocking(move || process.shutdown())
        .await
        .map_err(|_| AppError::internal("Chrome shutdown task did not complete"))?
        .map_err(|error| AppError::internal(format!("Failed to stop Chrome: {error}")))
}

async fn stop_owned_browser_resources(
    handle: &AppHandle,
    state: &GeminiBrowserState,
) -> AppResult<()> {
    let cdp_result = stop_owned_cdp_chrome(state).await;
    let sidecar_result = sidecar::stop(handle, state).await;
    match cdp_result {
        Err(error) => Err(error),
        Ok(()) => sidecar_result,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use super::{discard_abandoned_transport, domain_error_to_app};
    use extractum_gemini_browser::GeminiBrowserError;

    #[test]
    fn gemini_browser_error_maps_to_exact_legacy_app_error_json() {
        let error =
            GeminiBrowserError::timeout("Gemini Browser job timed out waiting for worker result");

        let app_error = domain_error_to_app(error);

        assert_eq!(app_error.kind, crate::error::AppErrorKind::Internal);
        assert_eq!(
            serde_json::to_string(&app_error).expect("serialize app error"),
            "{\"kind\":\"internal\",\"message\":\"Gemini Browser job timed out waiting for worker result\"}"
        );
    }

    #[tokio::test]
    async fn cancelled_run_marks_the_sidecar_transport_tainted() {
        let state = crate::gemini_browser::GeminiBrowserState::new();
        let observed_taint = Arc::new(AtomicBool::new(false));

        discard_abandoned_transport(&state, {
            let observed_taint = observed_taint.clone();
            move |tainted| observed_taint.store(tainted, Ordering::SeqCst)
        })
        .await
        .expect("discard abandoned transport");

        assert!(observed_taint.load(Ordering::SeqCst));
        assert!(!state.sidecar_tainted().await);
        discard_abandoned_transport(&state, |_| {})
            .await
            .expect("repeated discard succeeds");
    }
}
