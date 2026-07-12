use std::future::Future;

use parking_lot::RwLock;
use tokio::sync::{Mutex, MutexGuard, OnceCell};
use tokio_util::sync::CancellationToken;

use super::{
    path_string, profile_dir, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRunStatus,
};

#[derive(Default)]
pub struct GeminiBrowserState {
    active_run_id: Mutex<Option<String>>,
    cancellation: Mutex<Option<CancellationToken>>,
    sidecar_tainted: Mutex<bool>,
    sidecar: Mutex<Option<super::sidecar::GeminiBrowserSidecarProcess>>,
    cdp_chrome: Mutex<Option<super::cdp_chrome::ChromeCdpProcess>>,
    status_snapshot: RwLock<Option<GeminiBrowserProviderStatus>>,
    startup_reconciliation: OnceCell<()>,
}

impl GeminiBrowserState {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn init_status_snapshot(
        &self,
        handle: &tauri::AppHandle,
    ) -> crate::error::AppResult<()> {
        let snapshot = Self::not_started_status(path_string(&profile_dir(handle)?));
        *self.status_snapshot.write() = Some(snapshot);
        Ok(())
    }

    pub(crate) fn update_status_snapshot(
        &self,
        handle: &tauri::AppHandle,
        update: impl FnOnce(&mut GeminiBrowserProviderStatus),
    ) -> crate::error::AppResult<()> {
        self.ensure_status_snapshot(handle)?;
        if let Some(snapshot) = self.status_snapshot.write().as_mut() {
            update(snapshot);
        }
        Ok(())
    }

    pub(crate) fn status_snapshot(
        &self,
        handle: &tauri::AppHandle,
    ) -> crate::error::AppResult<GeminiBrowserProviderStatus> {
        self.ensure_status_snapshot(handle)?;
        self.status_snapshot.read().clone().ok_or_else(|| {
            crate::error::AppError::internal("Gemini Browser status snapshot missing")
        })
    }

    #[cfg(test)]
    pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
        *self.status_snapshot.write() = Some(status);
    }

    pub(crate) async fn ensure_startup_reconciled<F, Fut>(
        &self,
        reconcile: F,
    ) -> crate::error::AppResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = crate::error::AppResult<()>>,
    {
        self.startup_reconciliation
            .get_or_try_init(|| async { reconcile().await })
            .await
            .map(|_| ())
    }

    pub(crate) fn set_status_snapshot_if_current(
        &self,
        expected: &GeminiBrowserProviderStatus,
        next: GeminiBrowserProviderStatus,
    ) -> bool {
        let mut guard = self.status_snapshot.write();
        if guard.as_ref() == Some(expected) {
            *guard = Some(next);
            true
        } else {
            false
        }
    }

    fn ensure_status_snapshot(&self, handle: &tauri::AppHandle) -> crate::error::AppResult<()> {
        if self.status_snapshot.read().is_some() {
            return Ok(());
        }

        let snapshot = Self::not_started_status(path_string(&profile_dir(handle)?));
        let mut guard = self.status_snapshot.write();
        if guard.is_none() {
            *guard = Some(snapshot);
        }
        Ok(())
    }

    pub(crate) fn not_started_status(browser_profile_dir: String) -> GeminiBrowserProviderStatus {
        GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::NotStarted,
            manual_action: None,
            active_run_id: None,
            queue_depth: 0,
            browser_profile_dir,
            latest_message: Some("Gemini browser sidecar is not running.".to_string()),
        }
    }

    pub(crate) fn provider_status_kind_for_run_status(
        status: &GeminiBrowserRunStatus,
    ) -> GeminiBrowserProviderStatusKind {
        match status {
            GeminiBrowserRunStatus::Queued | GeminiBrowserRunStatus::Running => {
                GeminiBrowserProviderStatusKind::Running
            }
            GeminiBrowserRunStatus::Ok | GeminiBrowserRunStatus::Ready => {
                GeminiBrowserProviderStatusKind::Ready
            }
            GeminiBrowserRunStatus::NeedsLogin => GeminiBrowserProviderStatusKind::NeedsLogin,
            GeminiBrowserRunStatus::NeedsManualAction => {
                GeminiBrowserProviderStatusKind::NeedsManualAction
            }
            GeminiBrowserRunStatus::Cancelled => GeminiBrowserProviderStatusKind::Stopped,
            GeminiBrowserRunStatus::Blocked
            | GeminiBrowserRunStatus::Timeout
            | GeminiBrowserRunStatus::BrowserCrashed
            | GeminiBrowserRunStatus::Failed => GeminiBrowserProviderStatusKind::Failed,
        }
    }

    #[cfg(test)]
    pub(crate) fn init_status_snapshot_from_profile_dir_for_test(
        &self,
        browser_profile_dir: String,
    ) {
        *self.status_snapshot.write() = Some(Self::not_started_status(browser_profile_dir));
    }

    #[cfg(test)]
    pub(crate) fn update_status_snapshot_from_profile_dir_for_test(
        &self,
        browser_profile_dir: String,
        update: impl FnOnce(&mut GeminiBrowserProviderStatus),
    ) {
        if self.status_snapshot.read().is_none() {
            self.init_status_snapshot_from_profile_dir_for_test(browser_profile_dir);
        }
        if let Some(snapshot) = self.status_snapshot.write().as_mut() {
            update(snapshot);
        }
    }

    #[cfg(test)]
    pub(crate) fn status_snapshot_for_test(&self) -> Option<GeminiBrowserProviderStatus> {
        self.status_snapshot.read().clone()
    }

    pub async fn active_run_id(&self) -> Option<String> {
        self.active_run_id.lock().await.clone()
    }

    pub async fn start_run(&self, run_id: String) -> CancellationToken {
        *self.active_run_id.lock().await = Some(run_id);
        let token = CancellationToken::new();
        *self.cancellation.lock().await = Some(token.clone());
        *self.sidecar_tainted.lock().await = false;
        token
    }

    pub async fn finish_run(&self, run_id: &str) {
        let mut active = self.active_run_id.lock().await;
        if active.as_deref() == Some(run_id) {
            *active = None;
            *self.cancellation.lock().await = None;
        }
    }

    pub async fn request_stop(&self) -> bool {
        if let Some(token) = self.cancellation.lock().await.as_ref() {
            token.cancel();
            *self.sidecar_tainted.lock().await = true;
            true
        } else {
            false
        }
    }

    pub(crate) async fn cancellation_token(&self) -> Option<CancellationToken> {
        self.cancellation.lock().await.clone()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn state_tracks_active_run_and_cancellation() {
        let state = GeminiBrowserState::new();
        let token = state.start_run("run-1".to_string()).await;
        assert!(!token.is_cancelled());
        assert_eq!(state.active_run_id().await, Some("run-1".to_string()));
        assert!(state.request_stop().await);
        assert!(token.is_cancelled());
        state.finish_run("run-1").await;
        assert_eq!(state.active_run_id().await, None);
    }

    #[tokio::test]
    async fn cancelled_run_marks_the_sidecar_transport_tainted() {
        let state = GeminiBrowserState::new();
        let token = state.start_run("run-1".to_string()).await;
        assert!(state.request_stop().await);
        assert!(token.is_cancelled());
        assert!(state
            .cancellation_token()
            .await
            .expect("active token")
            .is_cancelled());
        assert!(state.sidecar_tainted().await);
    }

    #[test]
    fn status_snapshot_initializes_to_not_started_from_profile_dir() {
        let state = GeminiBrowserState::new();

        state.init_status_snapshot_from_profile_dir_for_test("profile-dir".to_string());

        let snapshot = state
            .status_snapshot_for_test()
            .expect("snapshot initialized");
        assert_eq!(
            snapshot.status,
            crate::gemini_browser::GeminiBrowserProviderStatusKind::NotStarted
        );
        assert_eq!(snapshot.browser_profile_dir, "profile-dir");
        assert_eq!(
            snapshot.latest_message.as_deref(),
            Some("Gemini browser sidecar is not running.")
        );
    }

    #[test]
    fn update_status_snapshot_mutates_cached_status() {
        let state = GeminiBrowserState::new();
        state.init_status_snapshot_from_profile_dir_for_test("profile-dir".to_string());

        state.update_status_snapshot_from_profile_dir_for_test(
            "profile-dir".to_string(),
            |status| {
                status.status = crate::gemini_browser::GeminiBrowserProviderStatusKind::Running;
                status.active_run_id = Some("run-1".to_string());
                status.queue_depth = 2;
                status.latest_message = Some("Running".to_string());
            },
        );

        let snapshot = state
            .status_snapshot_for_test()
            .expect("snapshot initialized");
        assert_eq!(
            snapshot.status,
            crate::gemini_browser::GeminiBrowserProviderStatusKind::Running
        );
        assert_eq!(snapshot.active_run_id.as_deref(), Some("run-1"));
        assert_eq!(snapshot.queue_depth, 2);
    }

    #[tokio::test]
    async fn startup_reconciliation_gate_runs_once_after_success() {
        let state = GeminiBrowserState::new();
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for _ in 0..2 {
            let calls = calls.clone();
            state
                .ensure_startup_reconciled(move || async move {
                    calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                })
                .await
                .expect("reconcile succeeds");
        }

        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn startup_reconciliation_gate_retries_after_failure() {
        let state = GeminiBrowserState::new();
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let calls_first = calls.clone();
        let error = state
            .ensure_startup_reconciled(move || async move {
                calls_first.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(crate::error::AppError::internal("fixture failure"))
            })
            .await
            .expect_err("first attempt fails");
        assert!(error.to_string().contains("fixture failure"));

        let calls_second = calls.clone();
        state
            .ensure_startup_reconciled(move || async move {
                calls_second.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            })
            .await
            .expect("second attempt succeeds");

        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn set_status_snapshot_if_current_does_not_overwrite_newer_snapshot() {
        let state = GeminiBrowserState::new();
        let expected = GeminiBrowserState::not_started_status("profile-dir".to_string());
        let newer = GeminiBrowserProviderStatus {
            latest_message: Some("worker update".to_string()),
            ..expected.clone()
        };
        let stale_reconciled = GeminiBrowserProviderStatus {
            latest_message: Some("stale pull read".to_string()),
            ..expected.clone()
        };

        state.set_status_snapshot(expected.clone());
        state.set_status_snapshot(newer.clone());

        assert!(!state.set_status_snapshot_if_current(&expected, stale_reconciled));
        assert_eq!(state.status_snapshot_for_test(), Some(newer));
    }
}
