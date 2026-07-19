use tokio::sync::{Mutex, MutexGuard};

use extractum_gemini_browser::GeminiBrowserDomainState;

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

    pub async fn active_run_id(&self) -> Option<String> {
        self.domain.active_run_id_snapshot()
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
