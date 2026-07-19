use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    future::Future,
    time::Duration,
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, watch};

use super::{
    domain_error::{GeminiBrowserError, GeminiBrowserResult},
    GeminiBrowserProviderConfig, GeminiBrowserRunRequest, GeminiBrowserRunResult,
};

const DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS: u64 = 20 * 60;

pub(crate) type GeminiBrowserWaiterResult = GeminiBrowserResult<GeminiBrowserRunResult>;
type GeminiBrowserWaiterSender = oneshot::Sender<GeminiBrowserWaiterResult>;
pub(crate) type GeminiBrowserWaiterReceiver = oneshot::Receiver<GeminiBrowserWaiterResult>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueuedGeminiBrowserJob {
    pub(crate) run_id: String,
    pub(crate) queue_position: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GeminiBrowserArtifactMode {
    Reduced,
    Full,
}

impl GeminiBrowserArtifactMode {
    pub(crate) fn from_wire(value: Option<&str>) -> GeminiBrowserResult<Self> {
        match value.unwrap_or("reduced") {
            "reduced" => Ok(Self::Reduced),
            "full" => Ok(Self::Full),
            other => Err(GeminiBrowserError::validation(format!(
                "unsupported Gemini Browser artifact_mode '{other}'"
            ))),
        }
    }

    pub(crate) fn as_wire(&self) -> &'static str {
        match self {
            Self::Reduced => "reduced",
            Self::Full => "full",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct GeminiBrowserJob {
    pub(crate) run_id: String,
    pub(crate) prompt: String,
    pub(crate) source: String,
    pub(crate) artifact_mode: GeminiBrowserArtifactMode,
    pub(crate) browser_config: Option<GeminiBrowserProviderConfig>,
}

impl GeminiBrowserJob {
    pub(crate) fn run_request(&self) -> GeminiBrowserRunRequest {
        GeminiBrowserRunRequest {
            run_id: self.run_id.clone(),
            prompt: self.prompt.clone(),
            source: self.source.clone(),
            artifact_mode: self.artifact_mode.as_wire().to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GeminiBrowserWorkerStatus {
    Starting,
    Ready {
        started_at: String,
    },
    Failed {
        started_at: Option<String>,
        error: String,
    },
}

pub(crate) struct GeminiBrowserJobRuntime {
    waiters: Mutex<HashMap<String, GeminiBrowserWaiterSender>>,
    cancelled_runs: Mutex<HashSet<String>>,
    worker_status: watch::Sender<GeminiBrowserWorkerStatus>,
    waiter_timeout: Duration,
    execution_timeout: Duration,
    worker_hard_guard_timeout: Duration,
}

impl Default for GeminiBrowserJobRuntime {
    fn default() -> Self {
        Self::new_with_timeouts(
            Duration::from_secs(DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS + 5),
            Duration::from_secs(DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS),
            Duration::from_secs(DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS + 15),
        )
    }
}

impl GeminiBrowserJobRuntime {
    #[cfg(test)]
    pub(crate) fn new_for_test(execution_timeout: Duration) -> Self {
        Self::new_for_test_with_timeouts(
            execution_timeout + Duration::from_millis(50),
            execution_timeout,
            execution_timeout + Duration::from_millis(100),
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_timeouts(
        waiter_timeout: Duration,
        execution_timeout: Duration,
        worker_hard_guard_timeout: Duration,
    ) -> Self {
        assert!(execution_timeout < waiter_timeout);
        assert!(waiter_timeout < worker_hard_guard_timeout);
        Self::new_with_timeouts(waiter_timeout, execution_timeout, worker_hard_guard_timeout)
    }

    #[cfg(test)]
    pub(crate) fn new_for_waiter_timeout_test(waiter_timeout: Duration) -> Self {
        Self::new_with_timeouts(
            waiter_timeout,
            waiter_timeout + Duration::from_millis(50),
            waiter_timeout + Duration::from_millis(100),
        )
    }

    fn new_with_timeouts(
        waiter_timeout: Duration,
        execution_timeout: Duration,
        worker_hard_guard_timeout: Duration,
    ) -> Self {
        let (worker_status, _) = watch::channel(GeminiBrowserWorkerStatus::Starting);
        Self {
            waiters: Mutex::new(HashMap::new()),
            cancelled_runs: Mutex::new(HashSet::new()),
            worker_status,
            waiter_timeout,
            execution_timeout,
            worker_hard_guard_timeout,
        }
    }

    pub(crate) fn register_waiter(
        &self,
        run_id: &str,
    ) -> GeminiBrowserResult<GeminiBrowserWaiterReceiver> {
        let mut waiters = self.waiters.lock();
        if waiters.contains_key(run_id) {
            return Err(GeminiBrowserError::conflict(format!(
                "Run '{run_id}' already has an active Gemini Browser waiter"
            )));
        }
        let (sender, receiver) = oneshot::channel();
        waiters.insert(run_id.to_string(), sender);
        Ok(receiver)
    }

    pub(crate) fn complete_waiter(&self, run_id: &str, result: GeminiBrowserWaiterResult) {
        if let Some(sender) = self.waiters.lock().remove(run_id) {
            let _ = sender.send(result);
        }
    }

    pub(crate) fn remove_waiter(&self, run_id: &str) {
        self.waiters.lock().remove(run_id);
    }

    pub(crate) fn has_waiter(&self, run_id: &str) -> bool {
        self.waiters.lock().contains_key(run_id)
    }

    #[cfg(test)]
    pub(crate) fn has_waiter_for_test(&self, run_id: &str) -> bool {
        self.has_waiter(run_id)
    }

    #[cfg(test)]
    pub(crate) fn worker_status_for_test(&self) -> GeminiBrowserWorkerStatus {
        self.worker_status.borrow().clone()
    }

    pub(crate) async fn wait_for_registered_result(
        &self,
        run_id: &str,
        receiver: GeminiBrowserWaiterReceiver,
    ) -> GeminiBrowserResult<GeminiBrowserRunResult> {
        match tokio::time::timeout(self.waiter_timeout, receiver).await {
            Ok(Ok(result)) => {
                self.remove_waiter(run_id);
                result
            }
            Ok(Err(_)) => {
                self.remove_waiter(run_id);
                Err(GeminiBrowserError::invariant(
                    "Gemini Browser worker channel closed unexpectedly",
                ))
            }
            Err(_) => {
                self.remove_waiter(run_id);
                Err(GeminiBrowserError::timeout(
                    "Gemini Browser job timed out waiting for worker result",
                ))
            }
        }
    }

    pub(crate) fn execution_timeout(&self) -> Duration {
        self.execution_timeout
    }

    pub(crate) fn worker_execution_timeout(&self) -> Duration {
        self.execution_timeout
    }

    pub(crate) fn worker_hard_guard_timeout(&self) -> Duration {
        self.worker_hard_guard_timeout
    }

    pub(crate) fn mark_worker_ready(&self, started_at: String) {
        self.worker_status
            .send_replace(GeminiBrowserWorkerStatus::Ready { started_at });
    }

    pub(crate) fn mark_worker_failed(&self, error: impl Into<String>) {
        self.worker_status
            .send_replace(GeminiBrowserWorkerStatus::Failed {
                started_at: None,
                error: error.into(),
            });
    }

    pub(crate) async fn ensure_worker_ready_for_enqueue(&self) -> GeminiBrowserResult<()> {
        self.ensure_worker_ready_for_enqueue_with_timeout(Duration::from_secs(5))
            .await
    }

    pub(crate) async fn ensure_worker_ready_for_enqueue_with_timeout(
        &self,
        timeout: Duration,
    ) -> GeminiBrowserResult<()> {
        match worker_status_enqueue_result(self.worker_status.borrow().clone()) {
            WorkerReadinessDecision::Ready => return Ok(()),
            WorkerReadinessDecision::Failed(error) => return Err(error),
            WorkerReadinessDecision::Starting => {}
        }
        let mut receiver = self.worker_status.subscribe();
        let wait_for_ready = async move {
            loop {
                receiver.changed().await.map_err(|_| {
                    GeminiBrowserError::invariant(
                        "Gemini Browser worker status channel closed unexpectedly",
                    )
                })?;
                match worker_status_enqueue_result(receiver.borrow().clone()) {
                    WorkerReadinessDecision::Ready => return Ok(()),
                    WorkerReadinessDecision::Failed(error) => return Err(error),
                    WorkerReadinessDecision::Starting => {}
                }
            }
        };
        tokio::time::timeout(timeout, wait_for_ready)
            .await
            .unwrap_or_else(|_| {
                Err(GeminiBrowserError::timeout(
                    "Gemini Browser worker is still starting",
                ))
            })
    }

    pub(crate) fn request_cancel(&self, run_id: &str) {
        self.cancelled_runs.lock().insert(run_id.to_string());
    }

    pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
        self.cancelled_runs.lock().contains(run_id)
    }

    pub(crate) fn clear_cancelled(&self, run_id: &str) {
        self.cancelled_runs.lock().remove(run_id);
    }
}

enum WorkerReadinessDecision {
    Starting,
    Ready,
    Failed(GeminiBrowserError),
}

fn worker_status_enqueue_result(status: GeminiBrowserWorkerStatus) -> WorkerReadinessDecision {
    match status {
        GeminiBrowserWorkerStatus::Starting => WorkerReadinessDecision::Starting,
        GeminiBrowserWorkerStatus::Ready { .. } => WorkerReadinessDecision::Ready,
        GeminiBrowserWorkerStatus::Failed { error, .. } => {
            WorkerReadinessDecision::Failed(GeminiBrowserError::invariant(format!(
                "Gemini Browser worker failed to start: {error}"
            )))
        }
    }
}

pub(crate) async fn run_registered_worker<Setup, SetupFuture, WorkerFuture, WorkerError>(
    runtime: &GeminiBrowserJobRuntime,
    setup: Setup,
) -> GeminiBrowserResult<()>
where
    Setup: FnOnce() -> SetupFuture + Send,
    SetupFuture: Future<Output = GeminiBrowserResult<WorkerFuture>> + Send,
    WorkerFuture: Future<Output = Result<(), WorkerError>> + Send,
    WorkerError: Display + Send,
{
    let worker = match setup().await {
        Ok(worker) => worker,
        Err(error) => {
            runtime.mark_worker_failed(error.to_string());
            return Err(error);
        }
    };
    runtime.mark_worker_ready(
        time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
    );
    worker.await.map_err(|error| {
        let error = GeminiBrowserError::invariant(error.to_string());
        runtime.mark_worker_failed(error.to_string());
        error
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wait_for_result_removes_waiter_on_timeout() {
        let runtime =
            GeminiBrowserJobRuntime::new_for_waiter_timeout_test(Duration::from_millis(1));
        let receiver = runtime
            .register_waiter("run-timeout")
            .expect("register waiter");
        let error = runtime
            .wait_for_registered_result("run-timeout", receiver)
            .await
            .expect_err("timeout error");
        assert_eq!(
            error.kind(),
            super::super::domain_error::GeminiBrowserErrorKind::Timeout
        );
        assert_eq!(
            error.message(),
            "Gemini Browser job timed out waiting for worker result"
        );
        assert!(!runtime.has_waiter("run-timeout"));
    }
}
