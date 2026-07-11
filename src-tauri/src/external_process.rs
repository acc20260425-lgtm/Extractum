use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tokio::sync::Notify;

pub(crate) const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);
pub(crate) const SHUTDOWN_WATCHDOG_TIMEOUT: Duration = Duration::from_secs(4);

pub(crate) type ExitCallback = Arc<dyn Fn(i32) + Send + Sync>;
pub(crate) type WatchdogScheduler = Arc<dyn Fn(ShutdownTiming, ExitCallback) + Send + Sync>;
pub(crate) type ShutdownCleanup =
    Pin<Box<dyn Future<Output = Result<(), ShutdownCleanupError>> + Send + 'static>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShutdownTiming {
    pub(crate) graceful: Duration,
    pub(crate) watchdog: Duration,
}

impl Default for ShutdownTiming {
    fn default() -> Self {
        Self {
            graceful: GRACEFUL_SHUTDOWN_TIMEOUT,
            watchdog: SHUTDOWN_WATCHDOG_TIMEOUT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShutdownPhase {
    Running,
    ShuttingDown,
    Completed,
}

#[derive(Debug)]
pub(crate) struct AdmissionRejected;

#[derive(Debug)]
pub(crate) enum ShutdownCleanupError {
    Failed,
}

struct AdmissionState {
    open: bool,
    active_startups: usize,
    first_exit_code: Option<i32>,
    phase: ShutdownPhase,
}

struct ExternalProcessShutdownInner {
    inner: Mutex<AdmissionState>,
    startup_idle: Notify,
}

#[derive(Clone)]
pub(crate) struct ExternalProcessShutdownState(Arc<ExternalProcessShutdownInner>);

pub(crate) struct AdmissionPermit {
    state: ExternalProcessShutdownState,
}

impl ExternalProcessShutdownState {
    pub(crate) fn new() -> Self {
        Self(Arc::new(ExternalProcessShutdownInner {
            inner: Mutex::new(AdmissionState {
                open: true,
                active_startups: 0,
                first_exit_code: None,
                phase: ShutdownPhase::Running,
            }),
            startup_idle: Notify::new(),
        }))
    }

    pub(crate) fn try_admit(&self) -> Result<AdmissionPermit, AdmissionRejected> {
        let mut admission = self.0.inner.lock();
        if !admission.open {
            return Err(AdmissionRejected);
        }
        admission.active_startups += 1;
        Ok(AdmissionPermit {
            state: self.clone(),
        })
    }

    pub(crate) fn begin_shutdown(&self, code: Option<i32>) -> bool {
        let mut admission = self.0.inner.lock();
        if admission.phase != ShutdownPhase::Running {
            return false;
        }
        admission.open = false;
        admission.first_exit_code = Some(code.unwrap_or(0));
        admission.phase = ShutdownPhase::ShuttingDown;
        true
    }

    pub(crate) fn phase(&self) -> ShutdownPhase {
        self.0.inner.lock().phase
    }

    pub(crate) fn exit_code(&self) -> i32 {
        self.0.inner.lock().first_exit_code.unwrap_or(0)
    }

    pub(crate) async fn wait_for_startups(&self) {
        loop {
            let notified = self.0.startup_idle.notified();
            if self.0.inner.lock().active_startups == 0 {
                return;
            }
            notified.await;
        }
    }

    pub(crate) fn complete(&self) {
        self.0.inner.lock().phase = ShutdownPhase::Completed;
    }

    pub(crate) async fn run_cleanup_steps(&self, cleanup_steps: Vec<ShutdownCleanup>) {
        let mut tasks = tokio::task::JoinSet::new();
        for cleanup in cleanup_steps {
            tasks.spawn(cleanup);
        }
        while tasks.join_next().await.is_some() {}
        self.complete();
    }

    pub(crate) fn run_watchdog(&self, exit: &ExitCallback) -> bool {
        if self.phase() == ShutdownPhase::Completed {
            return false;
        }
        self.complete();
        exit(self.exit_code());
        true
    }
}

impl Drop for AdmissionPermit {
    fn drop(&mut self) {
        let mut admission = self.state.0.inner.lock();
        debug_assert!(admission.active_startups > 0);
        admission.active_startups -= 1;
        if admission.active_startups == 0 {
            self.state.0.startup_idle.notify_waiters();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::sync::Barrier;

    #[tokio::test]
    async fn permits_acquired_before_shutdown_are_waited_for() {
        let state = ExternalProcessShutdownState::new();
        let permit = state.try_admit().expect("running admits");

        assert!(state.begin_shutdown(Some(23)));
        assert!(state.try_admit().is_err());
        assert_eq!(state.phase(), ShutdownPhase::ShuttingDown);

        let waiting = state.wait_for_startups();
        assert!(tokio::time::timeout(Duration::from_millis(10), waiting)
            .await
            .is_err());

        drop(permit);
        state.wait_for_startups().await;
        assert_eq!(state.exit_code(), 23);
    }

    #[test]
    fn only_the_first_shutdown_request_transitions_and_preserves_its_exit_code() {
        let state = ExternalProcessShutdownState::new();

        assert!(state.begin_shutdown(Some(23)));
        assert!(!state.begin_shutdown(Some(9)));
        assert_eq!(state.exit_code(), 23);
        state.complete();
        assert!(!state.begin_shutdown(Some(7)));
        assert_eq!(state.phase(), ShutdownPhase::Completed);
    }

    #[tokio::test]
    async fn cleanup_steps_start_concurrently_and_continue_after_a_failure() {
        let state = ExternalProcessShutdownState::new();
        let barrier = Arc::new(Barrier::new(2));
        let completed = Arc::new(Mutex::new(Vec::new()));

        let failing_barrier = barrier.clone();
        let successful_completed = completed.clone();
        state
            .run_cleanup_steps(vec![
                Box::pin(async move {
                    failing_barrier.wait().await;
                    Err(ShutdownCleanupError::Failed)
                }),
                Box::pin(async move {
                    barrier.wait().await;
                    successful_completed.lock().unwrap().push("successful");
                    Ok(())
                }),
            ])
            .await;

        assert_eq!(*completed.lock().unwrap(), vec!["successful"]);
        assert_eq!(state.phase(), ShutdownPhase::Completed);
    }

    #[test]
    fn watchdog_exits_with_the_preserved_code_unless_cleanup_completed() {
        let state = ExternalProcessShutdownState::new();
        state.begin_shutdown(Some(23));
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded_calls = calls.clone();
        let exit: ExitCallback = Arc::new(move |code| recorded_calls.lock().unwrap().push(code));

        assert!(state.run_watchdog(&exit));
        state.complete();
        assert!(!state.run_watchdog(&exit));
        assert_eq!(*calls.lock().unwrap(), vec![23]);
    }

    #[test]
    fn timing_exposes_the_graceful_and_watchdog_budgets() {
        assert_eq!(ShutdownTiming::default().graceful, GRACEFUL_SHUTDOWN_TIMEOUT);
        assert_eq!(ShutdownTiming::default().watchdog, SHUTDOWN_WATCHDOG_TIMEOUT);
    }
}
