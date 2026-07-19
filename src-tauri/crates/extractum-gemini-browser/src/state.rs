use std::future::Future;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;

use super::{
    error::{GeminiBrowserError, GeminiBrowserResult},
    GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind, GeminiBrowserRunStatus,
};

#[derive(Default)]
pub struct GeminiBrowserDomainState {
    active: Mutex<Option<ActiveRunControl>>,
    status_snapshot: RwLock<Option<GeminiBrowserProviderStatus>>,
    startup_reconciliation: OnceCell<()>,
}

#[derive(Clone)]
pub(crate) struct ActiveRunControl {
    run_id: String,
    cancellation: CancellationToken,
    stop_result: Arc<OnceCell<Option<super::error::GeminiBrowserError>>>,
}

impl ActiveRunControl {
    pub(crate) fn run_id(&self) -> &str {
        &self.run_id
    }

    pub(crate) fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    pub(crate) fn stop_result(&self) -> &OnceCell<Option<super::error::GeminiBrowserError>> {
        &self.stop_result
    }
}

impl GeminiBrowserDomainState {
    pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
        *self.status_snapshot.write() = Some(Self::not_started_status(browser_profile_dir));
    }

    pub(crate) fn ensure_status_snapshot(&self, browser_profile_dir: String) {
        if self.status_snapshot.read().is_some() {
            return;
        }
        let mut guard = self.status_snapshot.write();
        if guard.is_none() {
            *guard = Some(Self::not_started_status(browser_profile_dir));
        }
    }

    pub(crate) fn update_status_snapshot(
        &self,
        browser_profile_dir: String,
        update: impl FnOnce(&mut GeminiBrowserProviderStatus),
    ) {
        self.ensure_status_snapshot(browser_profile_dir);
        if let Some(snapshot) = self.status_snapshot.write().as_mut() {
            update(snapshot);
        }
    }

    pub(crate) fn status_snapshot(
        &self,
        browser_profile_dir: String,
    ) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
        self.ensure_status_snapshot(browser_profile_dir);
        self.status_snapshot
            .read()
            .clone()
            .ok_or_else(|| GeminiBrowserError::invariant("Gemini Browser status snapshot missing"))
    }

    pub(crate) async fn ensure_startup_reconciled<F, Fut>(
        &self,
        reconcile: F,
    ) -> GeminiBrowserResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = GeminiBrowserResult<()>>,
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

    pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
        *self.status_snapshot.write() = Some(status);
    }

    pub(crate) fn status_snapshot_option(&self) -> Option<GeminiBrowserProviderStatus> {
        self.status_snapshot.read().clone()
    }

    pub(crate) async fn active_run_id(&self) -> Option<String> {
        self.active
            .lock()
            .as_ref()
            .map(|active| active.run_id.clone())
    }

    pub fn active_run_id_snapshot(&self) -> Option<String> {
        self.active
            .lock()
            .as_ref()
            .map(|active| active.run_id.clone())
    }

    pub(crate) async fn start_run(&self, run_id: String) {
        *self.active.lock() = Some(ActiveRunControl {
            run_id,
            cancellation: CancellationToken::new(),
            stop_result: Arc::new(OnceCell::new()),
        });
    }

    pub(crate) fn active_control(&self, run_id: &str) -> Option<ActiveRunControl> {
        self.active
            .lock()
            .as_ref()
            .filter(|active| active.run_id == run_id)
            .cloned()
    }

    pub(crate) fn cancel_active(&self, run_id: &str) -> Option<ActiveRunControl> {
        let active = self.active_control(run_id)?;
        active.cancellation.cancel();
        Some(active)
    }

    pub(crate) async fn finish_run(&self, run_id: &str) {
        let mut active = self.active.lock();
        if active.as_ref().map(|active| active.run_id.as_str()) == Some(run_id) {
            *active = None;
        }
    }

    pub(crate) async fn request_stop(&self) -> bool {
        if let Some(active) = self.active.lock().as_ref() {
            active.cancellation.cancel();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn state_tracks_active_run_and_cancellation() {
        let state = GeminiBrowserDomainState::default();
        state.start_run("run-1".to_string()).await;
        let active = state.active_control("run-1").expect("active control");
        assert!(!active.cancellation().is_cancelled());
        assert_eq!(state.active_run_id().await, Some("run-1".to_string()));
        assert!(state.request_stop().await);
        assert!(active.cancellation().is_cancelled());
        state.finish_run("run-1").await;
        assert_eq!(state.active_run_id().await, None);
    }

    #[test]
    fn status_snapshot_initializes_to_not_started_from_profile_dir() {
        let state = GeminiBrowserDomainState::default();
        state.init_status_snapshot("profile-dir".to_string());
        let snapshot = state
            .status_snapshot_option()
            .expect("snapshot initialized");
        assert_eq!(snapshot.status, GeminiBrowserProviderStatusKind::NotStarted);
        assert_eq!(snapshot.browser_profile_dir, "profile-dir");
        assert_eq!(
            snapshot.latest_message.as_deref(),
            Some("Gemini browser sidecar is not running.")
        );
    }

    #[test]
    fn update_status_snapshot_mutates_cached_status() {
        let state = GeminiBrowserDomainState::default();
        state.update_status_snapshot("profile-dir".to_string(), |status| {
            status.status = GeminiBrowserProviderStatusKind::Running;
            status.active_run_id = Some("run-1".to_string());
            status.queue_depth = 2;
            status.latest_message = Some("Running".to_string());
        });
        let snapshot = state
            .status_snapshot_option()
            .expect("snapshot initialized");
        assert_eq!(snapshot.status, GeminiBrowserProviderStatusKind::Running);
        assert_eq!(snapshot.active_run_id.as_deref(), Some("run-1"));
        assert_eq!(snapshot.queue_depth, 2);
    }

    #[tokio::test]
    async fn startup_reconciliation_gate_runs_once_after_success() {
        let state = GeminiBrowserDomainState::default();
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
        let state = GeminiBrowserDomainState::default();
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls_first = calls.clone();
        let error = state
            .ensure_startup_reconciled(move || async move {
                calls_first.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(GeminiBrowserError::persistence("fixture failure"))
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
        let state = GeminiBrowserDomainState::default();
        let expected = GeminiBrowserDomainState::not_started_status("profile-dir".to_string());
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
        assert_eq!(state.status_snapshot_option(), Some(newer));
    }
}
