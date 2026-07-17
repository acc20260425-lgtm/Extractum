use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::sync::Notify;

const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);
const SHUTDOWN_WATCHDOG_TIMEOUT: Duration = Duration::from_secs(4);

pub type ExitCallback = Arc<dyn Fn(i32) + Send + Sync>;
pub type MonotonicClock = Arc<dyn Fn() -> Instant + Send + Sync>;
pub type WatchdogTask = Box<dyn FnOnce() + Send>;
pub type WatchdogScheduler = Arc<dyn Fn(ShutdownTiming, WatchdogTask) + Send + Sync>;
pub type ShutdownCleanup =
    Pin<Box<dyn Future<Output = Result<(), ShutdownCleanupError>> + Send + 'static>>;
pub type CleanupFactory = Box<dyn FnOnce() -> Vec<ShutdownCleanup> + Send + 'static>;

pub fn warn_shutdown_stage(operation_id: u64, stage: &'static str) {
    eprintln!("external process cleanup warning: operation_id={operation_id} stage={stage}");
}

fn warn_shutdown_coordinator_stage(stage: &'static str) {
    eprintln!("external process shutdown warning: stage={stage}");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShutdownTiming {
    pub graceful: Duration,
    pub watchdog: Duration,
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
enum ShutdownPhase {
    Running,
    ShuttingDown,
    Completed,
}

#[derive(Debug)]
pub struct AdmissionRejected;

#[derive(Debug)]
pub enum ShutdownCleanupError {
    #[allow(dead_code)]
    // Current subsystem shutdown adapters are infallible; tests cover isolation.
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
pub struct ExternalProcessShutdownState(Arc<ExternalProcessShutdownInner>);

pub struct AdmissionPermit {
    state: ExternalProcessShutdownState,
}

pub enum ShutdownStart {
    Started(ShutdownRun),
    AlreadyShuttingDown,
    Completed,
}

pub struct ShutdownRun {
    state: ExternalProcessShutdownState,
    deadline: Instant,
    clock: MonotonicClock,
    exit: ExitCallback,
}

pub fn system_monotonic_clock() -> MonotonicClock {
    Arc::new(Instant::now)
}

pub fn os_thread_watchdog_scheduler() -> WatchdogScheduler {
    Arc::new(|timing, watchdog| {
        std::thread::spawn(move || {
            std::thread::sleep(timing.watchdog);
            watchdog();
        });
    })
}

impl ExternalProcessShutdownState {
    pub fn new() -> Self {
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

    pub fn try_admit(&self) -> Result<AdmissionPermit, AdmissionRejected> {
        let mut admission = self.0.inner.lock();
        if !admission.open {
            return Err(AdmissionRejected);
        }
        admission.active_startups += 1;
        Ok(AdmissionPermit {
            state: self.clone(),
        })
    }

    pub fn start(
        &self,
        code: Option<i32>,
        timing: ShutdownTiming,
        scheduler: &WatchdogScheduler,
        exit: ExitCallback,
        clock: MonotonicClock,
    ) -> ShutdownStart {
        {
            let mut admission = self.0.inner.lock();
            match admission.phase {
                ShutdownPhase::Running => {
                    admission.open = false;
                    admission.first_exit_code = Some(code.unwrap_or(0));
                    admission.phase = ShutdownPhase::ShuttingDown;
                }
                ShutdownPhase::ShuttingDown => return ShutdownStart::AlreadyShuttingDown,
                ShutdownPhase::Completed => return ShutdownStart::Completed,
            }
        }

        let deadline = clock() + timing.graceful;
        self.schedule_watchdog(timing, scheduler, exit.clone());
        ShutdownStart::Started(ShutdownRun {
            state: self.clone(),
            deadline,
            clock,
            exit,
        })
    }

    async fn wait_for_startups(&self) {
        self.wait_for_startups_with_after_check(|| {}).await;
    }

    async fn wait_for_startups_with_after_check<F>(&self, after_check: F)
    where
        F: FnOnce(),
    {
        let mut after_check = Some(after_check);
        loop {
            let notified = self.0.startup_idle.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.0.inner.lock().active_startups == 0 {
                return;
            }
            if let Some(after_check) = after_check.take() {
                after_check();
            }
            notified.await;
        }
    }

    fn complete_and_exit(&self, exit: &ExitCallback) -> bool {
        let exit_code = {
            let mut admission = self.0.inner.lock();
            if admission.phase != ShutdownPhase::ShuttingDown {
                return false;
            }
            admission.phase = ShutdownPhase::Completed;
            admission.first_exit_code.unwrap_or(0)
        };
        exit(exit_code);
        true
    }

    fn run_watchdog(&self, exit: &ExitCallback) -> bool {
        self.complete_and_exit(exit)
    }

    fn schedule_watchdog(
        &self,
        timing: ShutdownTiming,
        scheduler: &WatchdogScheduler,
        exit: ExitCallback,
    ) {
        let state = self.clone();
        scheduler(
            timing,
            Box::new(move || {
                state.run_watchdog(&exit);
            }),
        );
    }
}

impl ShutdownRun {
    fn remaining(&self) -> Option<Duration> {
        self.deadline
            .checked_duration_since((self.clock)())
            .filter(|remaining| !remaining.is_zero())
    }

    pub async fn coordinate(self, cleanup_factory: CleanupFactory) {
        let Some(remaining) = self.remaining() else {
            warn_shutdown_coordinator_stage("admission_deadline_elapsed");
            self.state.complete_and_exit(&self.exit);
            return;
        };

        if tokio::time::timeout(remaining, self.state.wait_for_startups())
            .await
            .is_err()
        {
            warn_shutdown_coordinator_stage("admission_deadline_elapsed");
            self.state.complete_and_exit(&self.exit);
            return;
        }

        let Some(remaining) = self.remaining() else {
            warn_shutdown_coordinator_stage("cleanup_deadline_elapsed");
            self.state.complete_and_exit(&self.exit);
            return;
        };

        let cleanup = async move {
            let mut tasks = tokio::task::JoinSet::new();
            for step in cleanup_factory() {
                tasks.spawn(step);
            }
            while let Some(result) = tasks.join_next().await {
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(ShutdownCleanupError::Failed)) => {
                        warn_shutdown_coordinator_stage("cleanup_failed");
                    }
                    Err(_) => warn_shutdown_coordinator_stage("cleanup_panicked"),
                }
            }
        };

        if tokio::time::timeout(remaining, cleanup).await.is_err() {
            warn_shutdown_coordinator_stage("cleanup_deadline_elapsed");
        }
        self.state.complete_and_exit(&self.exit);
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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use tokio::sync::Barrier;

    fn recording_scheduler() -> (
        WatchdogScheduler,
        Arc<Mutex<Vec<ShutdownTiming>>>,
        Arc<Mutex<Option<WatchdogTask>>>,
    ) {
        let timings = Arc::new(Mutex::new(Vec::new()));
        let watchdog = Arc::new(Mutex::new(None));
        let recorded_timings = timings.clone();
        let recorded_watchdog = watchdog.clone();
        let scheduler: WatchdogScheduler = Arc::new(move |timing, task| {
            recorded_timings.lock().unwrap().push(timing);
            *recorded_watchdog.lock().unwrap() = Some(task);
        });
        (scheduler, timings, watchdog)
    }

    fn tokio_aligned_clock() -> MonotonicClock {
        let std_origin = Instant::now();
        let tokio_origin = tokio::time::Instant::now();
        Arc::new(move || std_origin + tokio::time::Instant::now().duration_since(tokio_origin))
    }

    fn phase(state: &ExternalProcessShutdownState) -> ShutdownPhase {
        state.0.inner.lock().phase
    }

    fn saved_exit_code(state: &ExternalProcessShutdownState) -> i32 {
        state.0.inner.lock().first_exit_code.unwrap_or(0)
    }

    #[tokio::test(start_paused = true)]
    async fn admission_wait_consumes_the_shared_graceful_budget() {
        let state = ExternalProcessShutdownState::new();
        let permit = state.try_admit().expect("running admits");
        let (scheduler, _, watchdog) = recording_scheduler();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded_calls = calls.clone();
        let exit: ExitCallback = Arc::new(move |code| recorded_calls.lock().unwrap().push(code));
        let timing = ShutdownTiming {
            graceful: Duration::from_secs(3),
            watchdog: Duration::from_secs(4),
        };
        let ShutdownStart::Started(run) =
            state.start(None, timing, &scheduler, exit, tokio_aligned_clock())
        else {
            panic!("first request must start");
        };
        let factory_called = Arc::new(AtomicBool::new(false));
        let recorded_factory = factory_called.clone();
        let task = tokio::spawn(run.coordinate(Box::new(move || {
            recorded_factory.store(true, Ordering::SeqCst);
            vec![Box::pin(async {
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(())
            })]
        })));

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(2)).await;
        drop(permit);
        tokio::task::yield_now().await;
        assert!(factory_called.load(Ordering::SeqCst));
        tokio::time::advance(Duration::from_secs(1)).await;
        task.await.unwrap();

        assert_eq!(*calls.lock().unwrap(), vec![0]);
        watchdog.lock().unwrap().take().unwrap()();
        assert_eq!(*calls.lock().unwrap(), vec![0]);
    }

    #[tokio::test(start_paused = true)]
    async fn exhausted_admission_budget_skips_the_cleanup_factory() {
        let state = ExternalProcessShutdownState::new();
        let permit = state.try_admit().expect("running admits");
        let (scheduler, _, _) = recording_scheduler();
        let timing = ShutdownTiming {
            graceful: Duration::from_secs(3),
            watchdog: Duration::from_secs(4),
        };
        let ShutdownStart::Started(run) = state.start(
            None,
            timing,
            &scheduler,
            Arc::new(|_| {}),
            tokio_aligned_clock(),
        ) else {
            panic!("first request must start");
        };
        let factory_called = Arc::new(AtomicBool::new(false));
        let recorded_factory = factory_called.clone();
        let task = tokio::spawn(run.coordinate(Box::new(move || {
            recorded_factory.store(true, Ordering::SeqCst);
            Vec::new()
        })));

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(3)).await;
        task.await.unwrap();

        assert!(!factory_called.load(Ordering::SeqCst));
        drop(permit);
    }

    #[tokio::test]
    async fn cleanup_tasks_start_concurrently_and_isolate_error_and_panic() {
        let state = ExternalProcessShutdownState::new();
        let (scheduler, _, _) = recording_scheduler();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded_calls = calls.clone();
        let exit: ExitCallback = Arc::new(move |code| recorded_calls.lock().unwrap().push(code));
        let ShutdownStart::Started(run) = state.start(
            Some(23),
            ShutdownTiming::default(),
            &scheduler,
            exit,
            Arc::new(Instant::now),
        ) else {
            panic!("first request must start");
        };
        let started = Arc::new(AtomicUsize::new(0));
        let settled = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(4));
        let mut cleanups: Vec<ShutdownCleanup> = Vec::new();

        for outcome in [0_u8, 1, 2] {
            let started = started.clone();
            let settled = settled.clone();
            let barrier = barrier.clone();
            cleanups.push(Box::pin(async move {
                started.fetch_add(1, Ordering::SeqCst);
                barrier.wait().await;
                match outcome {
                    0 => {
                        settled.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    }
                    1 => {
                        settled.fetch_add(1, Ordering::SeqCst);
                        Err(ShutdownCleanupError::Failed)
                    }
                    _ => panic!("cleanup fixture panic"),
                }
            }));
        }

        let task = tokio::spawn(run.coordinate(Box::new(move || cleanups)));
        barrier.wait().await;
        assert_eq!(started.load(Ordering::SeqCst), 3);
        task.await.unwrap();

        assert_eq!(settled.load(Ordering::SeqCst), 2);
        assert_eq!(*calls.lock().unwrap(), vec![23]);
    }

    #[test]
    fn start_returns_started_and_schedules_one_watchdog() {
        let state = ExternalProcessShutdownState::new();
        let (scheduler, timings, watchdog) = recording_scheduler();
        let result = state.start(
            Some(23),
            ShutdownTiming::default(),
            &scheduler,
            Arc::new(|_| {}),
            Arc::new(Instant::now),
        );

        assert!(matches!(result, ShutdownStart::Started(_)));
        assert_eq!(
            timings.lock().unwrap().as_slice(),
            &[ShutdownTiming::default()]
        );
        assert!(watchdog.lock().unwrap().is_some());
        assert_eq!(saved_exit_code(&state), 23);
        assert!(state.try_admit().is_err());
    }

    #[test]
    fn repeated_start_does_not_replace_code_or_schedule_again() {
        let state = ExternalProcessShutdownState::new();
        let (scheduler, timings, _) = recording_scheduler();
        let exit: ExitCallback = Arc::new(|_| {});
        let clock: MonotonicClock = Arc::new(Instant::now);

        assert!(matches!(
            state.start(
                Some(23),
                ShutdownTiming::default(),
                &scheduler,
                exit.clone(),
                clock.clone()
            ),
            ShutdownStart::Started(_)
        ));
        assert!(matches!(
            state.start(Some(99), ShutdownTiming::default(), &scheduler, exit, clock),
            ShutdownStart::AlreadyShuttingDown
        ));
        assert_eq!(timings.lock().unwrap().len(), 1);
        assert_eq!(saved_exit_code(&state), 23);
    }

    #[test]
    fn start_reports_completed_after_watchdog_claims_exit() {
        let state = ExternalProcessShutdownState::new();
        let (scheduler, _, watchdog) = recording_scheduler();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded_calls = calls.clone();
        let exit: ExitCallback = Arc::new(move |code| recorded_calls.lock().unwrap().push(code));
        let clock: MonotonicClock = Arc::new(Instant::now);

        let _ = state.start(
            Some(23),
            ShutdownTiming::default(),
            &scheduler,
            exit.clone(),
            clock.clone(),
        );
        watchdog.lock().unwrap().take().unwrap()();

        assert!(matches!(
            state.start(Some(99), ShutdownTiming::default(), &scheduler, exit, clock),
            ShutdownStart::Completed
        ));
        assert_eq!(*calls.lock().unwrap(), vec![23]);
    }

    #[tokio::test]
    async fn permits_acquired_before_shutdown_are_waited_for() {
        let state = ExternalProcessShutdownState::new();
        let permit = state.try_admit().expect("running admits");
        let (scheduler, _, _) = recording_scheduler();

        assert!(matches!(
            state.start(
                Some(23),
                ShutdownTiming::default(),
                &scheduler,
                Arc::new(|_| {}),
                Arc::new(Instant::now)
            ),
            ShutdownStart::Started(_)
        ));
        assert!(state.try_admit().is_err());
        assert_eq!(phase(&state), ShutdownPhase::ShuttingDown);

        let waiting = state.wait_for_startups();
        assert!(tokio::time::timeout(Duration::from_millis(10), waiting)
            .await
            .is_err());

        drop(permit);
        state.wait_for_startups().await;
        assert_eq!(saved_exit_code(&state), 23);
    }

    #[tokio::test]
    async fn permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown() {
        let state = ExternalProcessShutdownState::new();
        let permit = state.try_admit().expect("running admits");
        let (scheduler, _, _) = recording_scheduler();
        let _ = state.start(
            None,
            ShutdownTiming::default(),
            &scheduler,
            Arc::new(|_| {}),
            Arc::new(Instant::now),
        );

        tokio::time::timeout(
            Duration::from_millis(10),
            state.wait_for_startups_with_after_check(|| drop(permit)),
        )
        .await
        .expect("a drop after the count check wakes the registered waiter");
    }

    #[test]
    fn watchdog_exits_with_the_preserved_code_unless_cleanup_completed() {
        let state = ExternalProcessShutdownState::new();
        let (scheduler, _, _) = recording_scheduler();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded_calls = calls.clone();
        let exit: ExitCallback = Arc::new(move |code| recorded_calls.lock().unwrap().push(code));
        let _ = state.start(
            Some(23),
            ShutdownTiming::default(),
            &scheduler,
            exit.clone(),
            Arc::new(Instant::now),
        );

        assert!(state.run_watchdog(&exit));
        assert!(!state.run_watchdog(&exit));
        assert_eq!(*calls.lock().unwrap(), vec![23]);
    }

    #[tokio::test]
    async fn concurrent_watchdogs_invoke_exit_once() {
        let state = ExternalProcessShutdownState::new();
        let (scheduler, _, _) = recording_scheduler();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded_calls = calls.clone();
        let exit: ExitCallback = Arc::new(move |code| recorded_calls.lock().unwrap().push(code));
        let _ = state.start(
            Some(23),
            ShutdownTiming::default(),
            &scheduler,
            exit.clone(),
            Arc::new(Instant::now),
        );
        let barrier = Arc::new(Barrier::new(3));

        let first = {
            let state = state.clone();
            let exit = exit.clone();
            let barrier = barrier.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                state.run_watchdog(&exit)
            })
        };
        let second = {
            let state = state.clone();
            let exit = exit.clone();
            let barrier = barrier.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                state.run_watchdog(&exit)
            })
        };

        barrier.wait().await;
        let results = [first.await.unwrap(), second.await.unwrap()];
        assert_eq!(results.into_iter().filter(|result| *result).count(), 1);
        assert_eq!(*calls.lock().unwrap(), vec![23]);
    }

    #[test]
    fn injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback() {
        let state = ExternalProcessShutdownState::new();
        let timings = Arc::new(Mutex::new(Vec::new()));
        let recorded_timings = timings.clone();
        let scheduler: WatchdogScheduler = Arc::new(move |timing, watchdog| {
            recorded_timings.lock().unwrap().push(timing);
            watchdog();
        });
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded_calls = calls.clone();
        let exit: ExitCallback = Arc::new(move |code| recorded_calls.lock().unwrap().push(code));

        assert!(matches!(
            state.start(
                Some(23),
                ShutdownTiming::default(),
                &scheduler,
                exit,
                Arc::new(Instant::now)
            ),
            ShutdownStart::Started(_)
        ));

        assert_eq!(*timings.lock().unwrap(), vec![ShutdownTiming::default()]);
        assert_eq!(*calls.lock().unwrap(), vec![23]);
        assert_eq!(phase(&state), ShutdownPhase::Completed);
    }

    #[test]
    fn timing_exposes_the_graceful_and_watchdog_budgets() {
        assert_eq!(
            ShutdownTiming::default().graceful,
            GRACEFUL_SHUTDOWN_TIMEOUT
        );
        assert_eq!(
            ShutdownTiming::default().watchdog,
            SHUTDOWN_WATCHDOG_TIMEOUT
        );
    }
}
