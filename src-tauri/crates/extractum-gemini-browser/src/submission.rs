use std::{future::Future, path::Path};

use super::{
    error::{GeminiBrowserError, GeminiBrowserResult},
    executor::StatusObserver,
    run_log,
    runtime::{GeminiBrowserArtifactMode, GeminiBrowserJob, GeminiBrowserJobRuntime},
    state::GeminiBrowserDomainState,
    GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig, GeminiBrowserProviderStatusKind,
    GeminiBrowserRunRequest, GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

pub use super::runtime::QueuedGeminiBrowserJob;

pub async fn submit_and_wait<Enqueue, EnqueueFuture>(
    runs_dir: &Path,
    runtime: &GeminiBrowserJobRuntime,
    state: &GeminiBrowserDomainState,
    observer: &dyn StatusObserver,
    request: GeminiBrowserRunRequest,
    browser_config: Option<GeminiBrowserProviderConfig>,
    enqueue: Enqueue,
) -> GeminiBrowserResult<GeminiBrowserRunResult>
where
    Enqueue: FnOnce(GeminiBrowserJob) -> EnqueueFuture + Send,
    EnqueueFuture: Future<Output = GeminiBrowserResult<QueuedGeminiBrowserJob>> + Send,
{
    let artifact_mode = GeminiBrowserArtifactMode::from_wire(Some(&request.artifact_mode))?;
    runtime.ensure_worker_ready_for_enqueue().await?;
    reject_duplicate_existing_run_or_waiter(runtime, runs_dir, &request.run_id)?;
    run_log::create_queued_run(runs_dir, &request.run_id, &request.source, &request.prompt)?;
    let waiter = runtime.register_waiter(&request.run_id)?;
    let queued = enqueue(GeminiBrowserJob {
        run_id: request.run_id.clone(),
        prompt: request.prompt.clone(),
        source: request.source.clone(),
        artifact_mode,
        browser_config,
    })
    .await;
    let queued = match queued {
        Ok(queued) => queued,
        Err(error) => {
            runtime.remove_waiter(&request.run_id);
            let result = failed_enqueue_result(&request.run_id, &error);
            run_log::finish_run(runs_dir, &request.run_id, result.clone())?;
            publish_result(state, observer, &result);
            return Err(error);
        }
    };
    state.update_status_snapshot(String::new(), |status| {
        status.status = GeminiBrowserProviderStatusKind::Running;
        status.active_run_id = None;
        status.queue_depth = queued.queue_position.unwrap_or(1);
        status.latest_message = Some("Queued".to_string());
        status.manual_action = None;
    });
    if let Some(status) = state.status_snapshot_option() {
        observer.publish(&status);
    }
    runtime
        .wait_for_registered_result(&request.run_id, waiter)
        .await
}

fn reject_duplicate_existing_run_or_waiter(
    runtime: &GeminiBrowserJobRuntime,
    runs_dir: &Path,
    run_id: &str,
) -> GeminiBrowserResult<()> {
    if runtime.has_waiter(run_id) {
        return Err(GeminiBrowserError::conflict(format!(
            "Gemini Browser run_id '{run_id}' already has an active waiter"
        )));
    }
    if run_log::list_runs(runs_dir, usize::MAX)?
        .runs
        .into_iter()
        .any(|run| run.run_id == run_id)
    {
        return Err(GeminiBrowserError::conflict(format!(
            "Gemini Browser run_id '{run_id}' already exists"
        )));
    }
    Ok(())
}

fn failed_enqueue_result(run_id: &str, error: &GeminiBrowserError) -> GeminiBrowserRunResult {
    GeminiBrowserRunResult {
        run_id: run_id.to_string(),
        status: GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some(format!("Gemini Browser job enqueue failed: {error}")),
        manual_action: None,
        artifacts: GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 0,
        debug_summary: None,
    }
}

fn publish_result(
    state: &GeminiBrowserDomainState,
    observer: &dyn StatusObserver,
    result: &GeminiBrowserRunResult,
) {
    state.update_status_snapshot(String::new(), |status| {
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

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use parking_lot::Mutex;

    use super::*;

    #[derive(Default)]
    struct Observer(Mutex<Vec<super::super::GeminiBrowserProviderStatus>>);
    impl StatusObserver for Observer {
        fn publish(&self, status: &super::super::GeminiBrowserProviderStatus) {
            self.0.lock().push(status.clone());
        }
    }

    fn ready_runtime() -> GeminiBrowserJobRuntime {
        let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
        runtime.mark_worker_ready("2026-07-19T00:00:00Z".to_string());
        runtime
    }

    fn request(run_id: &str) -> GeminiBrowserRunRequest {
        GeminiBrowserRunRequest {
            run_id: run_id.to_string(),
            prompt: "hello".to_string(),
            source: "settings_test".to_string(),
            artifact_mode: "reduced".to_string(),
        }
    }

    #[tokio::test]
    async fn send_single_prompt_handoff_writes_run_log_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let state = GeminiBrowserDomainState::default();
        let observed = Arc::new(AtomicBool::new(false));
        let result = submit_and_wait(
            temp.path(),
            &runtime,
            &state,
            &Observer::default(),
            request("order"),
            None,
            {
                let observed = observed.clone();
                let runs = temp.path().to_path_buf();
                move |job| async move {
                    observed.store(
                        run_log::read_run(&runs, &job.run_id).is_ok(),
                        Ordering::SeqCst,
                    );
                    Err(GeminiBrowserError::persistence(
                        "stop after enqueue observation",
                    ))
                }
            },
        )
        .await;
        assert!(result.is_err());
        assert!(observed.load(Ordering::SeqCst));
    }

    async fn duplicate_case(run_id: &str, terminal: bool) {
        let temp = tempfile::tempdir().expect("temp dir");
        run_log::create_queued_run(temp.path(), run_id, "settings_test", "hello").expect("run");
        if terminal {
            run_log::finish_run(
                temp.path(),
                run_id,
                failed_enqueue_result(run_id, &GeminiBrowserError::persistence("fixture")),
            )
            .expect("finish");
        }
        let runtime = ready_runtime();
        let error = submit_and_wait(
            temp.path(),
            &runtime,
            &GeminiBrowserDomainState::default(),
            &Observer::default(),
            request(run_id),
            None,
            |_| async { panic!("enqueue") },
        )
        .await
        .expect_err("duplicate");
        assert_eq!(
            error.kind(),
            super::super::error::GeminiBrowserErrorKind::Conflict
        );
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue() {
        duplicate_case("duplicate", false).await;
    }
    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_terminal_run_id_before_enqueue() {
        duplicate_case("terminal", true).await;
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_waiter_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let _receiver = runtime.register_waiter("waiter").expect("waiter");
        let error = submit_and_wait(
            temp.path(),
            &runtime,
            &GeminiBrowserDomainState::default(),
            &Observer::default(),
            request("waiter"),
            None,
            |_| async { panic!("enqueue") },
        )
        .await
        .expect_err("duplicate waiter");
        assert_eq!(
            error.kind(),
            super::super::error::GeminiBrowserErrorKind::Conflict
        );
        runtime.remove_waiter("waiter");
    }

    #[tokio::test]
    async fn send_single_prompt_marks_run_failed_when_enqueue_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let error = submit_and_wait(
            temp.path(),
            &runtime,
            &GeminiBrowserDomainState::default(),
            &Observer::default(),
            request("failed"),
            None,
            |_| async { Err(GeminiBrowserError::persistence("push failed")) },
        )
        .await
        .expect_err("enqueue fails");
        assert_eq!(error.message(), "push failed");
        assert_eq!(
            run_log::read_run(temp.path(), "failed")
                .expect("run")
                .status,
            GeminiBrowserRunStatus::Failed
        );
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_invalid_artifact_mode_before_side_effects() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let mut value = request("invalid");
        value.artifact_mode = "bad".to_string();
        let error = submit_and_wait(
            temp.path(),
            &runtime,
            &GeminiBrowserDomainState::default(),
            &Observer::default(),
            value,
            None,
            |_| async { panic!("enqueue") },
        )
        .await
        .expect_err("invalid");
        assert_eq!(
            error.kind(),
            super::super::error::GeminiBrowserErrorKind::Validation
        );
        assert!(run_log::list_runs(temp.path(), 10)
            .expect("runs")
            .runs
            .is_empty());
    }

    #[tokio::test]
    async fn failed_run_log_transition_returns_app_error_without_side_effects() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runs_file = temp.path().join("file");
        std::fs::write(&runs_file, "x").expect("file");
        let runtime = ready_runtime();
        let error = submit_and_wait(
            &runs_file,
            &runtime,
            &GeminiBrowserDomainState::default(),
            &Observer::default(),
            request("log-fail"),
            None,
            |_| async { panic!("enqueue") },
        )
        .await
        .expect_err("log failure");
        assert_eq!(
            error.kind(),
            super::super::error::GeminiBrowserErrorKind::Persistence
        );
        assert!(!runtime.has_waiter("log-fail"));
    }
}
