use parking_lot::RwLock;
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;

use super::{
    path_string, profile_dir, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRunStatus,
};

#[derive(Default)]
pub struct GeminiBrowserState {
    active_run_id: Mutex<Option<String>>,
    cancellation: Mutex<Option<CancellationToken>>,
    sidecar: Mutex<Option<super::sidecar::GeminiBrowserSidecarProcess>>,
    status_snapshot: RwLock<Option<GeminiBrowserProviderStatus>>,
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

    pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
        *self.status_snapshot.write() = Some(status);
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

    fn not_started_status(browser_profile_dir: String) -> GeminiBrowserProviderStatus {
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
            true
        } else {
            false
        }
    }

    pub(crate) async fn sidecar(
        &self,
    ) -> MutexGuard<'_, Option<super::sidecar::GeminiBrowserSidecarProcess>> {
        self.sidecar.lock().await
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
}
