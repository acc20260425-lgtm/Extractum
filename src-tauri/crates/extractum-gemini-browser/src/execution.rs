use std::{path::Path, path::PathBuf, time::Duration};

use super::{
    error::{GeminiBrowserError, GeminiBrowserErrorKind, GeminiBrowserResult},
    executor::{BrowserExecutor, BrowserRunContext, BrowserStopReason, StatusObserver},
    run_log,
    runtime::{GeminiBrowserJob, GeminiBrowserJobRuntime},
    state::{ActiveRunControl, GeminiBrowserDomainState},
    GeminiBrowserArtifactRefs, GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

#[derive(Clone, Debug)]
pub struct DeliveredJobInput {
    pub job: GeminiBrowserJob,
    pub runs_dir: PathBuf,
    pub browser_profile_dir: String,
    pub artifact_dir: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryOutcome {
    Completed {
        result: GeminiBrowserRunResult,
    },
    AlreadyTerminal {
        result: Option<GeminiBrowserRunResult>,
    },
    Cancelled {
        result: GeminiBrowserRunResult,
        stop_error: Option<GeminiBrowserError>,
    },
    TimedOut {
        result: GeminiBrowserRunResult,
        stop_error: Option<GeminiBrowserError>,
    },
    Failed {
        result: GeminiBrowserRunResult,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CancelRunOutcome {
    ActiveCancellationRequested {
        stop_error: Option<GeminiBrowserError>,
    },
    QueuedCancelled {
        result: GeminiBrowserRunResult,
    },
    AlreadyTerminal,
    Missing,
}

enum ExecutionSelection {
    Completed(GeminiBrowserResult<GeminiBrowserRunResult>),
    Cancelled,
    TimedOut,
}

async fn stop_executor_once(
    active: &ActiveRunControl,
    executor: &dyn BrowserExecutor,
    reason: BrowserStopReason,
) -> Option<GeminiBrowserError> {
    active
        .stop_result()
        .get_or_init(|| async { executor.stop(reason).await.err() })
        .await
        .clone()
}

pub async fn execute_delivered_job(
    runtime: &GeminiBrowserJobRuntime,
    state: &GeminiBrowserDomainState,
    executor: &dyn BrowserExecutor,
    observer: &dyn StatusObserver,
    input: DeliveredJobInput,
) -> GeminiBrowserResult<DeliveryOutcome> {
    let run = match run_log::read_run(&input.runs_dir, &input.job.run_id) {
        Ok(run) => run,
        Err(error) if error.kind() == GeminiBrowserErrorKind::NotFound => {
            return Ok(DeliveryOutcome::AlreadyTerminal { result: None });
        }
        Err(error) => return Err(error),
    };
    if run.status.is_terminal() {
        return Ok(DeliveryOutcome::AlreadyTerminal { result: run.result });
    }

    run_log::mark_running(&input.runs_dir, &input.job.run_id)?;
    state.start_run(input.job.run_id.clone()).await;
    let active = state
        .active_control(&input.job.run_id)
        .ok_or_else(|| GeminiBrowserError::invariant("Gemini Browser active run missing"))?;
    let cancellation = active.cancellation();
    let timeout = runtime.execution_timeout();
    let mut send = executor.send(BrowserRunContext {
        request: input.job.run_request(),
        browser_profile_dir: input.browser_profile_dir.clone(),
        artifact_dir: input.artifact_dir.clone(),
        browser_config: input.job.browser_config.clone(),
    });
    let selected = tokio::select! {
        biased;
        _ = cancellation.cancelled() => ExecutionSelection::Cancelled,
        _ = tokio::time::sleep(timeout) => ExecutionSelection::TimedOut,
        result = &mut send => ExecutionSelection::Completed(result),
    };
    drop(send);

    match selected {
        ExecutionSelection::Completed(Ok(result)) => {
            terminalize(runtime, state, observer, &input, result.clone()).await?;
            Ok(DeliveryOutcome::Completed { result })
        }
        ExecutionSelection::Completed(Err(error)) => {
            let result = failed_result(&input.job.run_id, error.to_string());
            terminalize(runtime, state, observer, &input, result.clone()).await?;
            Ok(DeliveryOutcome::Failed { result })
        }
        ExecutionSelection::Cancelled => {
            let stop_error = stop_executor_once(
                &active,
                executor,
                BrowserStopReason::Cancelled {
                    run_id: input.job.run_id.clone(),
                },
            )
            .await;
            let result = cancelled_result(&input.job.run_id);
            terminalize(runtime, state, observer, &input, result.clone()).await?;
            Ok(DeliveryOutcome::Cancelled { result, stop_error })
        }
        ExecutionSelection::TimedOut => {
            let stop_error = stop_executor_once(
                &active,
                executor,
                BrowserStopReason::TimedOut {
                    run_id: input.job.run_id.clone(),
                    timeout,
                },
            )
            .await;
            let result = timeout_result(&input.job.run_id, timeout);
            terminalize(runtime, state, observer, &input, result.clone()).await?;
            Ok(DeliveryOutcome::TimedOut { result, stop_error })
        }
    }
}

pub async fn cancel_run(
    runs_dir: &Path,
    runtime: &GeminiBrowserJobRuntime,
    state: &GeminiBrowserDomainState,
    executor: &dyn BrowserExecutor,
    observer: &dyn StatusObserver,
    run_id: &str,
) -> GeminiBrowserResult<CancelRunOutcome> {
    runtime.request_cancel(run_id);
    if let Some(active) = state.cancel_active(run_id) {
        let stop_error = stop_executor_once(
            &active,
            executor,
            BrowserStopReason::Cancelled {
                run_id: run_id.to_string(),
            },
        )
        .await;
        return Ok(CancelRunOutcome::ActiveCancellationRequested { stop_error });
    }
    let run = match run_log::read_run(runs_dir, run_id) {
        Ok(run) => run,
        Err(error) if error.kind() == GeminiBrowserErrorKind::NotFound => {
            runtime.clear_cancelled(run_id);
            return Ok(CancelRunOutcome::Missing);
        }
        Err(error) => return Err(error),
    };
    if run.status.is_terminal() {
        runtime.clear_cancelled(run_id);
        return Ok(CancelRunOutcome::AlreadyTerminal);
    }
    if run.status == GeminiBrowserRunStatus::Queued {
        let result = cancelled_result(run_id);
        run_log::finish_run(runs_dir, run_id, result.clone())?;
        runtime.complete_waiter(run_id, Ok(result.clone()));
        runtime.clear_cancelled(run_id);
        state.finish_run(run_id).await;
        let profile = state
            .status_snapshot_option()
            .map(|status| status.browser_profile_dir)
            .unwrap_or_default();
        publish_terminal(state, observer, profile, &result);
        return Ok(CancelRunOutcome::QueuedCancelled { result });
    }
    Ok(CancelRunOutcome::Missing)
}

async fn terminalize(
    runtime: &GeminiBrowserJobRuntime,
    state: &GeminiBrowserDomainState,
    observer: &dyn StatusObserver,
    input: &DeliveredJobInput,
    result: GeminiBrowserRunResult,
) -> GeminiBrowserResult<()> {
    run_log::finish_run(&input.runs_dir, &input.job.run_id, result.clone())?;
    runtime.complete_waiter(&input.job.run_id, Ok(result.clone()));
    runtime.clear_cancelled(&input.job.run_id);
    state.finish_run(&input.job.run_id).await;
    publish_terminal(state, observer, input.browser_profile_dir.clone(), &result);
    Ok(())
}

fn publish_terminal(
    state: &GeminiBrowserDomainState,
    observer: &dyn StatusObserver,
    browser_profile_dir: String,
    result: &GeminiBrowserRunResult,
) {
    state.update_status_snapshot(browser_profile_dir, |status| {
        status.status =
            GeminiBrowserDomainState::provider_status_kind_for_run_status(&result.status);
        status.active_run_id = None;
        status.queue_depth = 0;
        status.latest_message = result.message.clone();
        status.manual_action = result.manual_action.clone();
    });
    if let Some(status) = state.status_snapshot_option() {
        observer.publish(&status);
    }
}

fn cancelled_result(run_id: &str) -> GeminiBrowserRunResult {
    GeminiBrowserRunResult {
        run_id: run_id.to_string(),
        status: GeminiBrowserRunStatus::Cancelled,
        text: None,
        message: Some("Cancelled".to_string()),
        manual_action: None,
        artifacts: GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 0,
        debug_summary: None,
    }
}

fn timeout_result(run_id: &str, timeout: Duration) -> GeminiBrowserRunResult {
    GeminiBrowserRunResult {
        run_id: run_id.to_string(),
        status: GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some(format!(
            "Gemini Browser job timed out after {}s",
            timeout.as_secs()
        )),
        manual_action: None,
        artifacts: GeminiBrowserArtifactRefs::default(),
        elapsed_ms: timeout.as_millis().try_into().unwrap_or(u64::MAX),
        debug_summary: None,
    }
}

fn failed_result(run_id: &str, message: String) -> GeminiBrowserRunResult {
    GeminiBrowserRunResult {
        run_id: run_id.to_string(),
        status: GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some(message),
        manual_action: None,
        artifacts: GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 0,
        debug_summary: None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;
    use tokio::sync::Notify;

    use super::*;

    use super::super::{
        executor::{BrowserExecutorFuture, BrowserSessionContext},
        GeminiBrowserArtifactMode, GeminiBrowserProviderConfig, GeminiBrowserProviderStatus,
        GeminiBrowserProviderStatusKind,
    };

    struct BlockingExecutor {
        send_started: Notify,
        release_send: Notify,
        stop_started: Notify,
        release_stop: Notify,
        stop_reasons: Mutex<Vec<BrowserStopReason>>,
        send_result: GeminiBrowserResult<GeminiBrowserRunResult>,
        stop_error: Option<GeminiBrowserError>,
    }

    impl BlockingExecutor {
        fn new(
            send_result: GeminiBrowserResult<GeminiBrowserRunResult>,
            stop_error: Option<GeminiBrowserError>,
        ) -> Self {
            Self {
                send_started: Notify::new(),
                release_send: Notify::new(),
                stop_started: Notify::new(),
                release_stop: Notify::new(),
                stop_reasons: Mutex::new(Vec::new()),
                send_result,
                stop_error,
            }
        }
    }

    impl BrowserExecutor for BlockingExecutor {
        fn status(
            &self,
            _context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            panic!("status is not used by execution tests")
        }

        fn open(
            &self,
            _context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            panic!("open is not used by execution tests")
        }

        fn resume(
            &self,
            _context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            panic!("resume is not used by execution tests")
        }

        fn send(
            &self,
            _context: BrowserRunContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserRunResult> {
            Box::pin(async move {
                self.send_started.notify_one();
                self.release_send.notified().await;
                self.send_result.clone()
            })
        }

        fn stop(&self, reason: BrowserStopReason) -> BrowserExecutorFuture<'_, ()> {
            Box::pin(async move {
                self.stop_reasons.lock().push(reason);
                self.stop_started.notify_one();
                self.release_stop.notified().await;
                match &self.stop_error {
                    Some(error) => Err(error.clone()),
                    None => Ok(()),
                }
            })
        }
    }

    #[derive(Default)]
    struct RecordingObserver(Mutex<Vec<GeminiBrowserProviderStatus>>);

    impl StatusObserver for RecordingObserver {
        fn publish(&self, status: &GeminiBrowserProviderStatus) {
            self.0.lock().push(status.clone());
        }
    }

    fn job(run_id: &str) -> GeminiBrowserJob {
        GeminiBrowserJob {
            run_id: run_id.to_string(),
            prompt: "hello".to_string(),
            source: "settings_test".to_string(),
            artifact_mode: GeminiBrowserArtifactMode::Reduced,
            browser_config: None::<GeminiBrowserProviderConfig>,
        }
    }

    fn success_result(run_id: &str) -> GeminiBrowserRunResult {
        GeminiBrowserRunResult {
            run_id: run_id.to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("late success".to_string()),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 7,
            debug_summary: None,
        }
    }

    fn setup_input(temp: &tempfile::TempDir, run_id: &str) -> DeliveredJobInput {
        run_log::create_queued_run(temp.path(), run_id, "settings_test", "hello")
            .expect("create queued run");
        DeliveredJobInput {
            job: job(run_id),
            runs_dir: temp.path().to_path_buf(),
            browser_profile_dir: "profile".to_string(),
            artifact_dir: temp.path().join(run_id).display().to_string(),
        }
    }

    fn assert_cancelled_payload(result: &GeminiBrowserRunResult, run_id: &str) {
        assert_eq!(result, &cancelled_result(run_id));
        assert_eq!(
            serde_json::to_string(result).expect("serialize cancelled result"),
            format!(
                "{{\"run_id\":\"{run_id}\",\"status\":\"cancelled\",\"text\":null,\"message\":\"Cancelled\",\"manual_action\":null,\"artifacts\":{{\"run_dir\":null,\"html\":null,\"screenshot\":null,\"telemetry\":null,\"answer_extraction\":null,\"artifact_write_error\":null}},\"elapsed_ms\":0,\"debug_summary\":null}}"
            )
        );
    }

    #[tokio::test]
    async fn active_cancellation_stops_executor_once_and_ignores_late_success() {
        let temp = tempfile::tempdir().expect("temp dir");
        let input = setup_input(&temp, "run-cancel-active");
        let runtime = Arc::new(GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(
            30,
        )));
        let waiter = runtime
            .register_waiter("run-cancel-active")
            .expect("waiter");
        let state = Arc::new(GeminiBrowserDomainState::default());
        let observer = Arc::new(RecordingObserver::default());
        let executor = Arc::new(BlockingExecutor::new(
            Ok(success_result("run-cancel-active")),
            None,
        ));

        let delivery = {
            let runtime = runtime.clone();
            let state = state.clone();
            let observer = observer.clone();
            let executor = executor.clone();
            tokio::spawn(async move {
                execute_delivered_job(&runtime, &state, &*executor, &*observer, input).await
            })
        };
        executor.send_started.notified().await;
        let cancellation = {
            let runs_dir = temp.path().to_path_buf();
            let runtime = runtime.clone();
            let state = state.clone();
            let observer = observer.clone();
            let executor = executor.clone();
            tokio::spawn(async move {
                cancel_run(
                    &runs_dir,
                    &runtime,
                    &state,
                    &*executor,
                    &*observer,
                    "run-cancel-active",
                )
                .await
            })
        };
        executor.stop_started.notified().await;
        assert_eq!(
            *executor.stop_reasons.lock(),
            vec![BrowserStopReason::Cancelled {
                run_id: "run-cancel-active".to_string()
            }]
        );
        executor.release_stop.notify_waiters();
        assert_eq!(
            cancellation.await.expect("cancel task").expect("cancel"),
            CancelRunOutcome::ActiveCancellationRequested { stop_error: None }
        );
        executor.release_send.notify_waiters();
        let outcome = delivery.await.expect("delivery task").expect("delivery");
        let result = match outcome {
            DeliveryOutcome::Cancelled { result, stop_error } => {
                assert_eq!(stop_error, None);
                result
            }
            other => panic!("unexpected outcome: {other:?}"),
        };
        assert_cancelled_payload(&result, "run-cancel-active");
        assert_eq!(
            run_log::read_run(temp.path(), "run-cancel-active")
                .expect("run")
                .result,
            Some(result.clone())
        );
        assert_eq!(
            waiter
                .await
                .expect("waiter channel")
                .expect("waiter result"),
            result
        );
        assert_eq!(state.active_run_id().await, None);
        assert!(!runtime.is_cancelled("run-cancel-active"));
        assert_eq!(
            observer.0.lock().last().expect("status").status,
            GeminiBrowserProviderStatusKind::Stopped
        );

        assert_eq!(
            cancel_run(
                temp.path(),
                &runtime,
                &state,
                &*executor,
                &*observer,
                "run-cancel-active",
            )
            .await
            .expect("second cancel"),
            CancelRunOutcome::AlreadyTerminal
        );
        assert_eq!(executor.stop_reasons.lock().len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn execution_timeout_stops_executor_with_typed_timeout_reason() {
        let temp = tempfile::tempdir().expect("temp dir");
        let input = setup_input(&temp, "run-timeout");
        let timeout = Duration::from_secs(5);
        let runtime = Arc::new(GeminiBrowserJobRuntime::new_for_test(timeout));
        let state = Arc::new(GeminiBrowserDomainState::default());
        let observer = Arc::new(RecordingObserver::default());
        let stop_error = GeminiBrowserError::transport("stop diagnostic");
        let executor = Arc::new(BlockingExecutor::new(
            Ok(success_result("run-timeout")),
            Some(stop_error.clone()),
        ));
        let delivery = {
            let runtime = runtime.clone();
            let state = state.clone();
            let observer = observer.clone();
            let executor = executor.clone();
            tokio::spawn(async move {
                execute_delivered_job(&runtime, &state, &*executor, &*observer, input).await
            })
        };
        executor.send_started.notified().await;
        tokio::time::advance(timeout).await;
        executor.stop_started.notified().await;
        assert_eq!(
            *executor.stop_reasons.lock(),
            vec![BrowserStopReason::TimedOut {
                run_id: "run-timeout".to_string(),
                timeout,
            }]
        );
        executor.release_stop.notify_waiters();
        executor.release_send.notify_waiters();
        let outcome = delivery.await.expect("delivery task").expect("delivery");
        let result = match outcome {
            DeliveryOutcome::TimedOut {
                result,
                stop_error: actual,
            } => {
                assert_eq!(actual, Some(stop_error));
                result
            }
            other => panic!("unexpected outcome: {other:?}"),
        };
        assert_eq!(result, timeout_result("run-timeout", timeout));
        assert_eq!(
            result.message.as_deref(),
            Some("Gemini Browser job timed out after 5s")
        );
        assert_eq!(result.elapsed_ms, 5_000);
        assert_eq!(
            run_log::read_run(temp.path(), "run-timeout")
                .expect("run")
                .result,
            Some(result)
        );
    }

    #[tokio::test]
    async fn cancel_gemini_browser_job_cancels_queued_run_and_waiter() {
        let temp = tempfile::tempdir().expect("temp dir");
        run_log::create_queued_run(temp.path(), "run-cancel-queued", "settings_test", "hello")
            .expect("queued run");
        let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(30));
        let waiter = runtime
            .register_waiter("run-cancel-queued")
            .expect("waiter");
        let state = GeminiBrowserDomainState::default();
        let observer = RecordingObserver::default();
        let executor = BlockingExecutor::new(Ok(success_result("unused")), None);

        let outcome = cancel_run(
            temp.path(),
            &runtime,
            &state,
            &executor,
            &observer,
            "run-cancel-queued",
        )
        .await
        .expect("cancel queued");
        let result = match outcome {
            CancelRunOutcome::QueuedCancelled { result } => result,
            other => panic!("unexpected outcome: {other:?}"),
        };
        assert_cancelled_payload(&result, "run-cancel-queued");
        assert_eq!(
            waiter
                .await
                .expect("waiter channel")
                .expect("waiter result"),
            result
        );
        assert!(executor.stop_reasons.lock().is_empty());
        assert_eq!(
            run_log::read_run(temp.path(), "run-cancel-queued")
                .expect("run")
                .status,
            GeminiBrowserRunStatus::Cancelled
        );
        assert_eq!(
            observer.0.lock().last().expect("status").status,
            GeminiBrowserProviderStatusKind::Stopped
        );
    }

    #[tokio::test]
    async fn cancel_gemini_browser_job_requests_stop_for_active_run() {
        let temp = tempfile::tempdir().expect("temp dir");
        let input = setup_input(&temp, "run-cancel-request");
        let runtime = Arc::new(GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(
            30,
        )));
        let state = Arc::new(GeminiBrowserDomainState::default());
        let observer = Arc::new(RecordingObserver::default());
        let executor = Arc::new(BlockingExecutor::new(
            Ok(success_result("run-cancel-request")),
            None,
        ));
        let delivery = {
            let runtime = runtime.clone();
            let state = state.clone();
            let observer = observer.clone();
            let executor = executor.clone();
            tokio::spawn(async move {
                execute_delivered_job(&runtime, &state, &*executor, &*observer, input).await
            })
        };
        executor.send_started.notified().await;
        assert_eq!(
            run_log::read_run(temp.path(), "run-cancel-request")
                .expect("running run")
                .status,
            GeminiBrowserRunStatus::Running
        );
        let cancellation = {
            let runs = temp.path().to_path_buf();
            let runtime = runtime.clone();
            let state = state.clone();
            let observer = observer.clone();
            let executor = executor.clone();
            tokio::spawn(async move {
                cancel_run(
                    &runs,
                    &runtime,
                    &state,
                    &*executor,
                    &*observer,
                    "run-cancel-request",
                )
                .await
            })
        };
        executor.stop_started.notified().await;
        assert_eq!(
            run_log::read_run(temp.path(), "run-cancel-request")
                .expect("still running")
                .status,
            GeminiBrowserRunStatus::Running
        );
        executor.release_stop.notify_waiters();
        assert!(matches!(
            cancellation.await.expect("cancel task").expect("cancel"),
            CancelRunOutcome::ActiveCancellationRequested { stop_error: None }
        ));
        executor.release_send.notify_waiters();
        let outcome = delivery.await.expect("delivery task").expect("delivery");
        assert!(matches!(outcome, DeliveryOutcome::Cancelled { .. }));
        assert_eq!(executor.stop_reasons.lock().len(), 1);
    }

    async fn timeout_then_success_sequence() -> (
        DeliveryOutcome,
        GeminiBrowserRunResult,
        Arc<GeminiBrowserJobRuntime>,
        Arc<GeminiBrowserDomainState>,
    ) {
        let temp = tempfile::tempdir().expect("temp dir");
        let timeout = Duration::from_secs(2);
        let runtime = Arc::new(GeminiBrowserJobRuntime::new_for_test(timeout));
        let state = Arc::new(GeminiBrowserDomainState::default());
        let observer = Arc::new(RecordingObserver::default());

        let first_input = setup_input(&temp, "run-timeout-first");
        let first_executor = Arc::new(BlockingExecutor::new(
            Ok(success_result("run-timeout-first")),
            None,
        ));
        let first_delivery = {
            let runtime = runtime.clone();
            let state = state.clone();
            let observer = observer.clone();
            let executor = first_executor.clone();
            tokio::spawn(async move {
                execute_delivered_job(&runtime, &state, &*executor, &*observer, first_input).await
            })
        };
        first_executor.send_started.notified().await;
        tokio::time::advance(timeout).await;
        first_executor.stop_started.notified().await;
        first_executor.release_stop.notify_waiters();
        let first_outcome = first_delivery
            .await
            .expect("first task")
            .expect("first delivery");

        let second_input = setup_input(&temp, "run-success-next");
        let second_result = success_result("run-success-next");
        let second_executor = BlockingExecutor::new(Ok(second_result.clone()), None);
        second_executor.release_send.notify_one();
        let second_outcome =
            execute_delivered_job(&runtime, &state, &second_executor, &*observer, second_input)
                .await
                .expect("second delivery");
        assert_eq!(
            second_outcome,
            DeliveryOutcome::Completed {
                result: second_result.clone()
            }
        );
        assert_eq!(first_executor.stop_reasons.lock().len(), 1);
        (first_outcome, second_result, runtime, state)
    }

    #[tokio::test(start_paused = true)]
    async fn worker_timeout_marks_run_failed_and_processes_next_job() {
        let (first, second, _, _) = timeout_then_success_sequence().await;
        assert!(matches!(first, DeliveryOutcome::TimedOut { .. }));
        assert_eq!(second.status, GeminiBrowserRunStatus::Ok);
    }

    #[tokio::test(start_paused = true)]
    async fn worker_timeout_clears_active_and_cancelled_state() {
        let (first, _, runtime, state) = timeout_then_success_sequence().await;
        assert!(matches!(first, DeliveryOutcome::TimedOut { .. }));
        assert_eq!(state.active_run_id().await, None);
        assert!(!runtime.is_cancelled("run-timeout-first"));
        assert!(!runtime.is_cancelled("run-success-next"));
    }

    #[tokio::test]
    async fn cancel_missing_run_returns_without_run_log_side_effects() {
        let temp = tempfile::tempdir().expect("temp dir");
        let outcome = cancel_run(
            temp.path(),
            &GeminiBrowserJobRuntime::default(),
            &GeminiBrowserDomainState::default(),
            &BlockingExecutor::new(Ok(success_result("unused")), None),
            &RecordingObserver::default(),
            "missing",
        )
        .await
        .expect("missing cancellation");
        assert_eq!(outcome, CancelRunOutcome::Missing);
        assert!(run_log::list_runs(temp.path(), 10)
            .expect("runs")
            .runs
            .is_empty());
    }

    #[tokio::test]
    async fn cancel_queued_run_updates_terminal_snapshot() {
        let temp = tempfile::tempdir().expect("temp dir");
        run_log::create_queued_run(temp.path(), "queued-snapshot", "settings_test", "hello")
            .expect("queued run");
        let state = GeminiBrowserDomainState::default();
        let observer = RecordingObserver::default();
        let outcome = cancel_run(
            temp.path(),
            &GeminiBrowserJobRuntime::default(),
            &state,
            &BlockingExecutor::new(Ok(success_result("unused")), None),
            &observer,
            "queued-snapshot",
        )
        .await
        .expect("cancel queued");
        assert!(matches!(outcome, CancelRunOutcome::QueuedCancelled { .. }));
        let snapshot = state.status_snapshot_option().expect("terminal snapshot");
        assert_eq!(snapshot.status, GeminiBrowserProviderStatusKind::Stopped);
        assert_eq!(snapshot.latest_message.as_deref(), Some("Cancelled"));
        assert_eq!(observer.0.lock().last(), Some(&snapshot));
    }

    #[tokio::test]
    async fn restart_worker_entry_skips_terminal_cancelled_run_log() {
        let temp = tempfile::tempdir().expect("temp dir");
        let input = setup_input(&temp, "terminal-cancelled");
        let cancelled = cancelled_result("terminal-cancelled");
        run_log::finish_run(temp.path(), "terminal-cancelled", cancelled.clone())
            .expect("terminal run");
        let executor = BlockingExecutor::new(Ok(success_result("unused")), None);
        let outcome = execute_delivered_job(
            &GeminiBrowserJobRuntime::default(),
            &GeminiBrowserDomainState::default(),
            &executor,
            &RecordingObserver::default(),
            input,
        )
        .await
        .expect("terminal delivery");
        assert_eq!(
            outcome,
            DeliveryOutcome::AlreadyTerminal {
                result: Some(cancelled)
            }
        );
        assert!(executor.stop_reasons.lock().is_empty());
    }

    #[tokio::test]
    async fn restart_worker_entry_acknowledges_missing_run_log_without_sidecar() {
        let temp = tempfile::tempdir().expect("temp dir");
        let input = DeliveredJobInput {
            job: job("missing-delivery"),
            runs_dir: temp.path().to_path_buf(),
            browser_profile_dir: "profile".to_string(),
            artifact_dir: temp.path().join("missing-delivery").display().to_string(),
        };
        let executor = BlockingExecutor::new(Ok(success_result("unused")), None);
        let outcome = execute_delivered_job(
            &GeminiBrowserJobRuntime::default(),
            &GeminiBrowserDomainState::default(),
            &executor,
            &RecordingObserver::default(),
            input,
        )
        .await
        .expect("missing delivery");
        assert_eq!(outcome, DeliveryOutcome::AlreadyTerminal { result: None });
        assert!(executor.stop_reasons.lock().is_empty());
    }

    #[tokio::test]
    async fn worker_handler_marks_run_running_and_terminal() {
        let temp = tempfile::tempdir().expect("temp dir");
        let input = setup_input(&temp, "worker-success");
        let expected = success_result("worker-success");
        let executor = BlockingExecutor::new(Ok(expected.clone()), None);
        executor.release_send.notify_one();
        let outcome = execute_delivered_job(
            &GeminiBrowserJobRuntime::default(),
            &GeminiBrowserDomainState::default(),
            &executor,
            &RecordingObserver::default(),
            input,
        )
        .await
        .expect("delivery");
        assert_eq!(
            outcome,
            DeliveryOutcome::Completed {
                result: expected.clone()
            }
        );
        let run = run_log::read_run(temp.path(), "worker-success").expect("run");
        assert_eq!(run.status, GeminiBrowserRunStatus::Ok);
        assert_eq!(run.result, Some(expected));
    }

    #[tokio::test]
    async fn worker_handler_converts_executor_error_to_terminal_failed_result() {
        let temp = tempfile::tempdir().expect("temp dir");
        let input = setup_input(&temp, "worker-failure");
        let executor =
            BlockingExecutor::new(Err(GeminiBrowserError::browser("executor failed")), None);
        executor.release_send.notify_one();
        let outcome = execute_delivered_job(
            &GeminiBrowserJobRuntime::default(),
            &GeminiBrowserDomainState::default(),
            &executor,
            &RecordingObserver::default(),
            input,
        )
        .await
        .expect("delivery terminalizes error");
        let result = match outcome {
            DeliveryOutcome::Failed { result } => result,
            other => panic!("unexpected outcome: {other:?}"),
        };
        assert_eq!(result.message.as_deref(), Some("executor failed"));
        assert_eq!(
            run_log::read_run(temp.path(), "worker-failure")
                .expect("run")
                .result,
            Some(result)
        );
    }
}
