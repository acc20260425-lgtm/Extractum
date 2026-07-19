use std::future::Future;

use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;

use super::{
    executor::{app_error_to_domain, domain_error_to_app, DomainErrorContext},
    path_string,
    portable_state::GeminiBrowserDomainState,
    profile_dir, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRunStatus,
};

#[derive(Default)]
pub struct GeminiBrowserState {
    domain: GeminiBrowserDomainState,
    sidecar_tainted: Mutex<bool>,
    sidecar: Mutex<Option<super::sidecar::GeminiBrowserSidecarProcess>>,
    cdp_chrome: Mutex<Option<super::cdp_chrome::ChromeCdpProcess>>,
}

impl GeminiBrowserState {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn domain(&self) -> &GeminiBrowserDomainState {
        &self.domain
    }

    fn resolved_profile_dir(&self, handle: &tauri::AppHandle) -> crate::error::AppResult<String> {
        Ok(path_string(&profile_dir(handle)?))
    }

    pub(crate) fn init_status_snapshot(
        &self,
        handle: &tauri::AppHandle,
    ) -> crate::error::AppResult<()> {
        self.domain
            .init_status_snapshot(self.resolved_profile_dir(handle)?);
        Ok(())
    }

    pub(crate) fn update_status_snapshot(
        &self,
        handle: &tauri::AppHandle,
        update: impl FnOnce(&mut GeminiBrowserProviderStatus),
    ) -> crate::error::AppResult<()> {
        self.domain
            .update_status_snapshot(self.resolved_profile_dir(handle)?, update);
        Ok(())
    }

    pub(crate) async fn ensure_startup_reconciled<F, Fut>(
        &self,
        reconcile: F,
    ) -> crate::error::AppResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = crate::error::AppResult<()>>,
    {
        self.domain
            .ensure_startup_reconciled(|| async {
                reconcile()
                    .await
                    .map_err(|error| app_error_to_domain(error, DomainErrorContext::Persistence))
            })
            .await
            .map_err(domain_error_to_app)
    }

    pub(crate) fn provider_status_kind_for_run_status(
        status: &GeminiBrowserRunStatus,
    ) -> GeminiBrowserProviderStatusKind {
        GeminiBrowserDomainState::provider_status_kind_for_run_status(status)
    }

    pub async fn active_run_id(&self) -> Option<String> {
        self.domain.active_run_id().await
    }

    pub async fn start_run(&self, run_id: String) -> CancellationToken {
        *self.sidecar_tainted.lock().await = false;
        self.domain.start_run(run_id).await
    }

    pub async fn finish_run(&self, run_id: &str) {
        self.domain.finish_run(run_id).await;
    }

    pub async fn request_stop(&self) -> bool {
        let requested = self.domain.request_stop().await;
        if requested {
            *self.sidecar_tainted.lock().await = true;
        }
        requested
    }

    pub(crate) async fn cancellation_token(&self) -> Option<CancellationToken> {
        self.domain.cancellation_token().await
    }

    pub(crate) async fn mark_sidecar_tainted(&self) {
        *self.sidecar_tainted.lock().await = true;
    }

    pub(crate) async fn sidecar_tainted(&self) -> bool {
        *self.sidecar_tainted.lock().await
    }

    pub(crate) async fn clear_sidecar_taint(&self) {
        *self.sidecar_tainted.lock().await = false;
    }

    pub(crate) async fn sidecar(
        &self,
    ) -> MutexGuard<'_, Option<super::sidecar::GeminiBrowserSidecarProcess>> {
        self.sidecar.lock().await
    }

    pub(crate) async fn cdp_chrome_process(
        &self,
    ) -> MutexGuard<'_, Option<super::cdp_chrome::ChromeCdpProcess>> {
        self.cdp_chrome.lock().await
    }
}
