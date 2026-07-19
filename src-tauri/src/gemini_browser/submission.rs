use crate::error::{AppError, AppResult};

use super::{
    jobs::{
        GeminiBrowserArtifactMode, GeminiBrowserJob, GeminiBrowserJobRuntime,
        GeminiBrowserWaiterReceiver, QueuedGeminiBrowserJob,
    },
    GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig, GeminiBrowserRunRequest,
    GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

#[derive(Debug)]
pub(crate) struct SendSinglePromptEnqueueHandoff {
    pub(crate) waiter: GeminiBrowserWaiterReceiver,
}

#[derive(Debug)]
pub(crate) enum SendSinglePromptEnqueueError {
    App(AppError),
    EnqueueFailed {
        run_id: String,
        source: AppError,
        failed_result: GeminiBrowserRunResult,
    },
}

impl From<AppError> for SendSinglePromptEnqueueError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}

pub(crate) async fn send_single_prompt_enqueue_core<Enqueue, EnqueueFut>(
    runs_root: &std::path::Path,
    runtime: &GeminiBrowserJobRuntime,
    request: GeminiBrowserRunRequest,
    browser_config: Option<GeminiBrowserProviderConfig>,
    enqueue: Enqueue,
) -> Result<SendSinglePromptEnqueueHandoff, SendSinglePromptEnqueueError>
where
    Enqueue: FnOnce(GeminiBrowserJob) -> EnqueueFut,
    EnqueueFut: std::future::Future<Output = AppResult<QueuedGeminiBrowserJob>>,
{
    let artifact_mode = GeminiBrowserArtifactMode::from_wire(Some(&request.artifact_mode))?;
    runtime.ensure_worker_ready_for_enqueue().await?;
    reject_duplicate_existing_run_or_waiter(runtime, runs_root, &request.run_id).await?;
    let queued_run =
        super::create_queued_run(runs_root, &request.run_id, &request.source, &request.prompt)?;
    let waiter = runtime.register_waiter(&request.run_id)?;
    match enqueue(GeminiBrowserJob {
        run_id: request.run_id.clone(),
        prompt: request.prompt.clone(),
        source: request.source.clone(),
        artifact_mode,
        browser_config,
    })
    .await
    {
        Ok(_queued) => {}
        Err(error) => {
            runtime.remove_waiter(&request.run_id);
            let failed = GeminiBrowserRunResult {
                run_id: request.run_id.clone(),
                status: GeminiBrowserRunStatus::Failed,
                text: None,
                message: Some(format!("Gemini Browser job enqueue failed: {error}")),
                manual_action: None,
                artifacts: GeminiBrowserArtifactRefs::default(),
                elapsed_ms: 0,
                debug_summary: None,
            };
            let _failed_run = super::finish_run(runs_root, &request.run_id, failed.clone())?;
            return Err(SendSinglePromptEnqueueError::EnqueueFailed {
                run_id: request.run_id.clone(),
                source: error,
                failed_result: failed,
            });
        }
    };
    let _queued_run = queued_run;
    Ok(SendSinglePromptEnqueueHandoff { waiter })
}

async fn reject_duplicate_existing_run_or_waiter(
    runtime: &GeminiBrowserJobRuntime,
    runs_root: &std::path::Path,
    run_id: &str,
) -> AppResult<()> {
    if runtime.has_waiter(run_id) {
        return Err(AppError::conflict(format!(
            "Gemini Browser run_id '{run_id}' already has an active waiter"
        )));
    }
    if super::list_runs(runs_root, usize::MAX)?
        .runs
        .into_iter()
        .any(|run| run.run_id == run_id)
    {
        return Err(AppError::conflict(format!(
            "Gemini Browser run_id '{run_id}' already exists"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_runtime() -> GeminiBrowserJobRuntime {
        let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
        runtime.mark_worker_ready("2026-07-19T00:00:00Z".to_string());
        runtime
    }

    fn test_request(run_id: &str) -> GeminiBrowserRunRequest {
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
        let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let request = test_request("run-order");
        let handoff = send_single_prompt_enqueue_core(temp.path(), &runtime, request, None, {
            let observed = observed.clone();
            let runs = temp.path().to_path_buf();
            move |job| async move {
                observed.store(
                    super::super::list_runs(&runs, 10)
                        .expect("list runs")
                        .runs
                        .iter()
                        .any(|run| run.run_id == job.run_id),
                    std::sync::atomic::Ordering::SeqCst,
                );
                Ok(QueuedGeminiBrowserJob {
                    run_id: job.run_id,
                    queue_position: Some(1),
                })
            }
        })
        .await
        .expect("enqueue handoff");
        assert!(observed.load(std::sync::atomic::Ordering::SeqCst));
        runtime.remove_waiter("run-order");
        drop(handoff);
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        super::super::create_queued_run(temp.path(), "duplicate", "settings_test", "hello")
            .expect("create existing run");
        let runtime = ready_runtime();
        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            test_request("duplicate"),
            None,
            |_| async { panic!("enqueue must not run") },
        )
        .await
        .expect_err("duplicate rejected");
        assert!(matches!(error, SendSinglePromptEnqueueError::App(_)));
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_terminal_run_id_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        super::super::create_queued_run(temp.path(), "terminal", "settings_test", "hello")
            .expect("create existing run");
        super::super::finish_run(
            temp.path(),
            "terminal",
            GeminiBrowserRunResult {
                run_id: "terminal".to_string(),
                status: GeminiBrowserRunStatus::Ok,
                text: Some("done".to_string()),
                message: None,
                manual_action: None,
                artifacts: GeminiBrowserArtifactRefs::default(),
                elapsed_ms: 1,
                debug_summary: None,
            },
        )
        .expect("finish existing run");
        let runtime = ready_runtime();
        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            test_request("terminal"),
            None,
            |_| async { panic!("enqueue must not run") },
        )
        .await
        .expect_err("terminal duplicate rejected");
        assert!(matches!(error, SendSinglePromptEnqueueError::App(_)));
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_duplicate_waiter_before_enqueue() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let _receiver = runtime.register_waiter("waiter").expect("register waiter");
        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            test_request("waiter"),
            None,
            |_| async { panic!("enqueue must not run") },
        )
        .await
        .expect_err("duplicate waiter rejected");
        assert!(matches!(error, SendSinglePromptEnqueueError::App(_)));
        runtime.remove_waiter("waiter");
    }

    #[tokio::test]
    async fn send_single_prompt_marks_run_failed_when_enqueue_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let error = send_single_prompt_enqueue_core(
            temp.path(),
            &runtime,
            test_request("enqueue-fails"),
            None,
            |_| async { Err(AppError::internal("push failed")) },
        )
        .await
        .expect_err("enqueue failure returned");
        assert!(matches!(
            error,
            SendSinglePromptEnqueueError::EnqueueFailed { .. }
        ));
        let run = super::super::read_run(temp.path(), "enqueue-fails").expect("read failed run");
        assert_eq!(run.status, GeminiBrowserRunStatus::Failed);
        assert!(!runtime.has_waiter("enqueue-fails"));
    }

    #[tokio::test]
    async fn send_single_prompt_rejects_invalid_artifact_mode_before_side_effects() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime = ready_runtime();
        let mut request = test_request("invalid-artifacts");
        request.artifact_mode = "unknown".to_string();
        let error =
            send_single_prompt_enqueue_core(temp.path(), &runtime, request, None, |_| async {
                panic!("enqueue must not run")
            })
            .await
            .expect_err("invalid artifact mode rejected");
        assert!(matches!(error, SendSinglePromptEnqueueError::App(_)));
        assert!(super::super::list_runs(temp.path(), 10)
            .expect("list runs")
            .runs
            .is_empty());
    }

    #[tokio::test]
    async fn failed_run_log_transition_returns_app_error_without_side_effects() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runs_file = temp.path().join("not-a-directory");
        std::fs::write(&runs_file, "fixture").expect("write fixture");
        let runtime = ready_runtime();
        let error = send_single_prompt_enqueue_core(
            &runs_file,
            &runtime,
            test_request("run-log-fails"),
            None,
            |_| async { panic!("enqueue must not run") },
        )
        .await
        .expect_err("run log failure returned");
        assert!(matches!(error, SendSinglePromptEnqueueError::App(_)));
        assert!(!runtime.has_waiter("run-log-fails"));
    }
}
