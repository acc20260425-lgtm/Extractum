use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;

use extractum_core::error::{AppError, AppResult};

const DEFAULT_CONCURRENCY_LIMIT: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmRequestPriority {
    Interactive,
    Background,
}

impl LlmRequestPriority {
    fn rank(self) -> u8 {
        match self {
            Self::Interactive => 0,
            Self::Background => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmRequestKind {
    ProviderTest,
    AnalysisChat,
    AnalysisReportMap,
    AnalysisReportReduce,
    PromptPackStage,
}

pub fn llm_request_kind_diagnostic_key(kind: LlmRequestKind) -> &'static str {
    match kind {
        LlmRequestKind::ProviderTest => "provider_test",
        LlmRequestKind::AnalysisChat => "analysis_chat",
        LlmRequestKind::AnalysisReportMap => "analysis_report_map",
        LlmRequestKind::AnalysisReportReduce => "analysis_report_reduce",
        LlmRequestKind::PromptPackStage => "prompt_pack_stage",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SchedulerKey {
    provider: String,
    profile_id: String,
}

#[derive(Clone, Debug)]
pub struct LlmRequestMetadata {
    pub request_id: String,
    pub profile_id: String,
    pub provider: String,
    pub kind: LlmRequestKind,
    pub priority: LlmRequestPriority,
    pub owner_run_id: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmRequestSnapshotState {
    Queued,
    Running,
}

pub fn llm_request_state_diagnostic_key(state: LlmRequestSnapshotState) -> &'static str {
    match state {
        LlmRequestSnapshotState::Queued => "queued",
        LlmRequestSnapshotState::Running => "running",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LlmRequestSnapshot {
    pub request_id: String,
    pub kind: LlmRequestKind,
    pub provider: String,
    pub profile_id: String,
    pub priority: LlmRequestPriority,
    pub state: LlmRequestSnapshotState,
    pub queue_position: Option<usize>,
    pub owner_run_id: Option<i64>,
}

impl LlmRequestMetadata {
    fn scheduler_key(&self) -> SchedulerKey {
        SchedulerKey {
            provider: self.provider.clone(),
            profile_id: self.profile_id.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LlmRequestControl {
    token: CancellationToken,
}

impl LlmRequestControl {
    fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    fn cancel(&self) {
        self.token.cancel();
    }

    fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    pub async fn run_cancellable<Fut, T, E>(&self, future: Fut) -> Result<T, LlmRequestError>
    where
        Fut: Future<Output = Result<T, E>>,
        E: Into<AppError>,
    {
        if self.is_cancelled() {
            return Err(LlmRequestError::Cancelled);
        }

        tokio::select! {
            result = future => result.map_err(|error| LlmRequestError::Failed(error.into())),
            _ = self.token.cancelled() => Err(LlmRequestError::Cancelled),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmRequestError {
    Cancelled,
    Failed(AppError),
}

type QueueCallback = Arc<dyn Fn(usize) + Send + Sync>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RequestState {
    Queued,
    Running,
}

impl RequestState {
    fn snapshot_state(self) -> LlmRequestSnapshotState {
        match self {
            Self::Queued => LlmRequestSnapshotState::Queued,
            Self::Running => LlmRequestSnapshotState::Running,
        }
    }
}

struct RequestEntry {
    meta: LlmRequestMetadata,
    state: RequestState,
    queue_callback: QueueCallback,
    control: LlmRequestControl,
}

#[derive(Default)]
struct KeyState {
    active_count: usize,
    queue: VecDeque<String>,
}

#[derive(Default)]
struct SchedulerInner {
    requests: HashMap<String, RequestEntry>,
    keys: HashMap<SchedulerKey, KeyState>,
}

impl SchedulerInner {
    fn queue_position(&self, key: &SchedulerKey, request_id: &str) -> Option<usize> {
        self.keys.get(key).and_then(|key_state| {
            key_state
                .queue
                .iter()
                .position(|queued_request_id| queued_request_id == request_id)
                .map(|index| index + 1)
        })
    }

    fn snapshots(&self) -> Vec<LlmRequestSnapshot> {
        let mut snapshots = self
            .requests
            .iter()
            .map(|(request_id, entry)| {
                let key = entry.meta.scheduler_key();
                LlmRequestSnapshot {
                    request_id: request_id.clone(),
                    kind: entry.meta.kind,
                    provider: entry.meta.provider.clone(),
                    profile_id: entry.meta.profile_id.clone(),
                    priority: entry.meta.priority,
                    state: entry.state.snapshot_state(),
                    queue_position: (entry.state == RequestState::Queued)
                        .then(|| self.queue_position(&key, request_id))
                        .flatten(),
                    owner_run_id: entry.meta.owner_run_id,
                }
            })
            .collect::<Vec<_>>();

        snapshots.sort_by(|left, right| {
            left.provider
                .cmp(&right.provider)
                .then_with(|| left.profile_id.cmp(&right.profile_id))
                .then_with(|| {
                    snapshot_state_rank(left.state).cmp(&snapshot_state_rank(right.state))
                })
                .then_with(|| {
                    left.queue_position
                        .unwrap_or(0)
                        .cmp(&right.queue_position.unwrap_or(0))
                })
                .then_with(|| left.request_id.cmp(&right.request_id))
        });
        snapshots
    }

    fn queue_updates_for_key(&self, key: &SchedulerKey) -> Vec<(QueueCallback, usize)> {
        let Some(key_state) = self.keys.get(key) else {
            return Vec::new();
        };

        key_state
            .queue
            .iter()
            .enumerate()
            .filter_map(|(index, request_id)| {
                self.requests
                    .get(request_id)
                    .map(|entry| (entry.queue_callback.clone(), index + 1))
            })
            .collect()
    }

    fn insert_queued_request(&mut self, key: &SchedulerKey, request_id: &str) {
        let request_priority = self
            .requests
            .get(request_id)
            .map(|entry| entry.meta.priority.rank())
            .unwrap_or(LlmRequestPriority::Background.rank());

        let insert_at = {
            let key_state = self.keys.entry(key.clone()).or_default();
            key_state
                .queue
                .iter()
                .enumerate()
                .find_map(|(index, queued_request_id)| {
                    let queued_priority = self
                        .requests
                        .get(queued_request_id)
                        .map(|entry| entry.meta.priority.rank())
                        .unwrap_or(LlmRequestPriority::Background.rank());
                    (queued_priority > request_priority).then_some(index)
                })
                .unwrap_or(key_state.queue.len())
        };

        let key_state = self.keys.entry(key.clone()).or_default();
        key_state.queue.insert(insert_at, request_id.to_string());
    }

    fn remove_request(&mut self, request_id: &str) -> Vec<(QueueCallback, usize)> {
        let Some(entry) = self.requests.remove(request_id) else {
            return Vec::new();
        };

        let key = entry.meta.scheduler_key();
        let remove_key_state = {
            let Some(key_state) = self.keys.get_mut(&key) else {
                return Vec::new();
            };

            match entry.state {
                RequestState::Queued => {
                    if let Some(index) = key_state
                        .queue
                        .iter()
                        .position(|queued| queued == request_id)
                    {
                        key_state.queue.remove(index);
                    }
                }
                RequestState::Running => {
                    key_state.active_count = key_state.active_count.saturating_sub(1);
                }
            }

            key_state.active_count == 0 && key_state.queue.is_empty()
        };

        let updates = self.queue_updates_for_key(&key);
        if remove_key_state {
            self.keys.remove(&key);
        }
        updates
    }
}

fn snapshot_state_rank(state: LlmRequestSnapshotState) -> u8 {
    match state {
        LlmRequestSnapshotState::Running => 0,
        LlmRequestSnapshotState::Queued => 1,
    }
}

pub struct LlmSchedulerState {
    inner: Mutex<SchedulerInner>,
    state_changed: Notify,
}

impl LlmSchedulerState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SchedulerInner::default()),
            state_changed: Notify::new(),
        }
    }

    fn emit_queue_updates(updates: Vec<(QueueCallback, usize)>) {
        for (callback, position) in updates {
            callback(position);
        }
    }

    async fn register_request<Q>(
        &self,
        meta: LlmRequestMetadata,
        on_queue: Q,
    ) -> AppResult<LlmRequestControl>
    where
        Q: Fn(usize) + Send + Sync + 'static,
    {
        let queue_callback: QueueCallback = Arc::new(on_queue);
        let control = LlmRequestControl::new();
        let key = meta.scheduler_key();
        let request_id = meta.request_id.clone();
        let request_kind = meta.kind;
        let updates = {
            let mut inner = self.inner.lock().await;
            if inner.requests.contains_key(&request_id) {
                return Err(AppError::conflict(format!(
                    "LLM request '{}' ({request_kind:?}) is already registered",
                    request_id,
                )));
            }

            inner.requests.insert(
                request_id.clone(),
                RequestEntry {
                    meta,
                    state: RequestState::Queued,
                    queue_callback: queue_callback.clone(),
                    control: control.clone(),
                },
            );
            inner.insert_queued_request(&key, &request_id);
            inner.queue_updates_for_key(&key)
        };

        Self::emit_queue_updates(updates);
        self.state_changed.notify_waiters();
        Ok(control)
    }

    async fn wait_for_turn(&self, request_id: &str) -> Result<LlmRequestControl, LlmRequestError> {
        enum WaitDecision {
            Start(LlmRequestControl, Vec<(QueueCallback, usize)>),
            Cancelled,
            Wait,
        }

        loop {
            let wake = self.state_changed.notified();
            let decision = {
                let mut inner = self.inner.lock().await;
                let Some(entry) = inner.requests.get(request_id) else {
                    return Err(LlmRequestError::Cancelled);
                };
                let control = entry.control.clone();
                if control.is_cancelled() {
                    WaitDecision::Cancelled
                } else {
                    let key = entry.meta.scheduler_key();
                    let can_start = inner
                        .keys
                        .get(&key)
                        .map(|key_state| {
                            key_state.active_count < DEFAULT_CONCURRENCY_LIMIT
                                && key_state
                                    .queue
                                    .front()
                                    .map(|queued_request_id| queued_request_id == request_id)
                                    .unwrap_or(false)
                        })
                        .unwrap_or(false);

                    if !can_start {
                        WaitDecision::Wait
                    } else {
                        if let Some(key_state) = inner.keys.get_mut(&key) {
                            key_state.queue.pop_front();
                            key_state.active_count += 1;
                        }
                        if let Some(entry) = inner.requests.get_mut(request_id) {
                            entry.state = RequestState::Running;
                        }

                        WaitDecision::Start(control, inner.queue_updates_for_key(&key))
                    }
                }
            };

            match decision {
                WaitDecision::Start(control, updates) => {
                    Self::emit_queue_updates(updates);
                    self.state_changed.notify_waiters();
                    return Ok(control);
                }
                WaitDecision::Cancelled => return Err(LlmRequestError::Cancelled),
                WaitDecision::Wait => wake.await,
            }
        }
    }

    async fn finish_request(&self, request_id: &str) {
        let updates = {
            let mut inner = self.inner.lock().await;
            inner.remove_request(request_id)
        };
        Self::emit_queue_updates(updates);
        self.state_changed.notify_waiters();
    }

    pub async fn cancel_request(&self, request_id: &str) -> bool {
        let control = {
            let inner = self.inner.lock().await;
            inner
                .requests
                .get(request_id)
                .map(|entry| entry.control.clone())
        };

        if let Some(control) = control {
            control.cancel();
            self.state_changed.notify_waiters();
            return true;
        }

        false
    }

    pub async fn cancel_run_requests(&self, run_id: i64) -> usize {
        let controls = {
            let inner = self.inner.lock().await;
            inner
                .requests
                .values()
                .filter(|entry| entry.meta.owner_run_id == Some(run_id))
                .map(|entry| entry.control.clone())
                .collect::<Vec<_>>()
        };

        for control in &controls {
            control.cancel();
        }
        if !controls.is_empty() {
            self.state_changed.notify_waiters();
        }

        controls.len()
    }

    pub async fn request_snapshots(&self) -> Vec<LlmRequestSnapshot> {
        self.inner.lock().await.snapshots()
    }

    pub async fn active_owner_run_ids(&self) -> HashSet<i64> {
        self.inner
            .lock()
            .await
            .requests
            .values()
            .filter_map(|entry| entry.meta.owner_run_id)
            .collect()
    }

    pub async fn run_request<T, Q, F, Fut>(
        &self,
        meta: LlmRequestMetadata,
        on_queue: Q,
        work: F,
    ) -> Result<T, LlmRequestError>
    where
        Q: Fn(usize) + Send + Sync + 'static,
        F: FnOnce(LlmRequestControl) -> Fut,
        Fut: Future<Output = Result<T, LlmRequestError>>,
    {
        let request_id = meta.request_id.clone();
        self.register_request(meta, on_queue)
            .await
            .map_err(LlmRequestError::Failed)?;

        let result = match self.wait_for_turn(&request_id).await {
            Ok(control) => work(control).await,
            Err(error) => Err(error),
        };

        self.finish_request(&request_id).await;
        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use tokio::sync::{mpsc, Mutex as TokioMutex, Notify};
    use tokio::time::{timeout, Duration};

    use super::{
        llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key, LlmRequestError,
        LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmRequestSnapshotState,
        LlmSchedulerState,
    };
    use extractum_core::error::{AppError, AppErrorKind};

    #[test]
    fn llm_request_diagnostic_keys_are_stable_snake_case() {
        assert_eq!(
            llm_request_kind_diagnostic_key(LlmRequestKind::AnalysisChat),
            "analysis_chat"
        );
        assert_eq!(
            llm_request_kind_diagnostic_key(LlmRequestKind::AnalysisReportReduce),
            "analysis_report_reduce"
        );
        assert_eq!(
            llm_request_state_diagnostic_key(LlmRequestSnapshotState::Queued),
            "queued"
        );
        assert_eq!(
            llm_request_state_diagnostic_key(LlmRequestSnapshotState::Running),
            "running"
        );
    }

    fn metadata(
        request_id: &str,
        profile_id: &str,
        priority: LlmRequestPriority,
        kind: LlmRequestKind,
    ) -> LlmRequestMetadata {
        LlmRequestMetadata {
            request_id: request_id.to_string(),
            profile_id: profile_id.to_string(),
            provider: "gemini".to_string(),
            kind,
            priority,
            owner_run_id: None,
        }
    }

    async fn assert_scheduler_empty(scheduler: &LlmSchedulerState) {
        let inner = scheduler.inner.lock().await;
        assert!(inner.requests.is_empty(), "requests should be cleaned up");
        assert!(inner.keys.is_empty(), "key registry should be cleaned up");
    }

    #[tokio::test]
    async fn requests_with_different_profiles_run_without_blocking_each_other() {
        let scheduler = Arc::new(LlmSchedulerState::new());
        let started = Arc::new(TokioMutex::new(Vec::new()));
        let first_release = Arc::new(Notify::new());
        let second_release = Arc::new(Notify::new());

        let first_scheduler = scheduler.clone();
        let first_started = started.clone();
        let first_release_wait = first_release.clone();
        let first = tokio::spawn(async move {
            first_scheduler
                .run_request(
                    metadata(
                        "a-1",
                        "default",
                        LlmRequestPriority::Background,
                        LlmRequestKind::AnalysisReportMap,
                    ),
                    |_| {},
                    move |control| {
                        let first_started = first_started.clone();
                        async move {
                            first_started.lock().await.push("a-1".to_string());
                            control
                                .run_cancellable(async move {
                                    first_release_wait.notified().await;
                                    Ok::<_, String>("done")
                                })
                                .await
                        }
                    },
                )
                .await
        });

        let second_scheduler = scheduler.clone();
        let second_started = started.clone();
        let second_release_wait = second_release.clone();
        let second = tokio::spawn(async move {
            second_scheduler
                .run_request(
                    metadata(
                        "b-1",
                        "secondary",
                        LlmRequestPriority::Background,
                        LlmRequestKind::AnalysisReportMap,
                    ),
                    |_| {},
                    move |control| {
                        let second_started = second_started.clone();
                        async move {
                            second_started.lock().await.push("b-1".to_string());
                            control
                                .run_cancellable(async move {
                                    second_release_wait.notified().await;
                                    Ok::<_, String>("done")
                                })
                                .await
                        }
                    },
                )
                .await
        });

        timeout(Duration::from_secs(1), async {
            loop {
                if started.lock().await.len() == 2 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("both requests should start");

        first_release.notify_waiters();
        second_release.notify_waiters();

        assert_eq!(first.await.expect("join first"), Ok("done"));
        assert_eq!(second.await.expect("join second"), Ok("done"));
    }

    #[tokio::test]
    async fn interactive_requests_jump_ahead_of_background_queue() {
        let scheduler = Arc::new(LlmSchedulerState::new());
        let release = Arc::new(Notify::new());
        let (started_tx, mut started_rx) = mpsc::unbounded_channel::<String>();

        let mut handles = Vec::new();
        for request_id in ["bg-1", "bg-2"] {
            let scheduler = scheduler.clone();
            let release_wait = release.clone();
            let started_tx = started_tx.clone();
            handles.push(tokio::spawn(async move {
                scheduler
                    .run_request(
                        metadata(
                            request_id,
                            "default",
                            LlmRequestPriority::Background,
                            LlmRequestKind::AnalysisReportMap,
                        ),
                        |_| {},
                        move |control| {
                            let started_tx = started_tx.clone();
                            async move {
                                let _ = started_tx.send(request_id.to_string());
                                control
                                    .run_cancellable(async move {
                                        release_wait.notified().await;
                                        Ok::<_, String>("done")
                                    })
                                    .await
                            }
                        },
                    )
                    .await
            }));
        }

        let queued_background = {
            let scheduler = scheduler.clone();
            let release_wait = release.clone();
            let started_tx = started_tx.clone();
            tokio::spawn(async move {
                scheduler
                    .run_request(
                        metadata(
                            "bg-3",
                            "default",
                            LlmRequestPriority::Background,
                            LlmRequestKind::AnalysisReportMap,
                        ),
                        |_| {},
                        move |control| {
                            let started_tx = started_tx.clone();
                            async move {
                                let _ = started_tx.send("bg-3".to_string());
                                control
                                    .run_cancellable(async move {
                                        release_wait.notified().await;
                                        Ok::<_, String>("done")
                                    })
                                    .await
                            }
                        },
                    )
                    .await
            })
        };

        let interactive = {
            let scheduler = scheduler.clone();
            let started_tx = started_tx.clone();
            tokio::spawn(async move {
                scheduler
                    .run_request(
                        metadata(
                            "interactive-1",
                            "default",
                            LlmRequestPriority::Interactive,
                            LlmRequestKind::AnalysisChat,
                        ),
                        |_| {},
                        move |control| {
                            let started_tx = started_tx.clone();
                            async move {
                                let _ = started_tx.send("interactive-1".to_string());
                                control
                                    .run_cancellable(async move { Ok::<_, String>("interactive") })
                                    .await
                            }
                        },
                    )
                    .await
            })
        };

        assert_eq!(
            started_rx.recv().await.expect("first started"),
            "bg-1".to_string()
        );
        assert_eq!(
            started_rx.recv().await.expect("second started"),
            "bg-2".to_string()
        );

        release.notify_one();

        assert_eq!(
            timeout(Duration::from_secs(1), started_rx.recv())
                .await
                .expect("third started")
                .expect("request id"),
            "interactive-1".to_string()
        );

        release.notify_waiters();

        for handle in handles {
            let _ = handle.await;
        }
        let _ = interactive.await;
        let _ = queued_background.await;
    }

    #[tokio::test]
    async fn queued_requests_can_be_cancelled_before_start() {
        let scheduler = Arc::new(LlmSchedulerState::new());
        let release = Arc::new(Notify::new());

        let mut running_handles = Vec::new();
        for request_id in ["run-1", "run-2"] {
            let scheduler = scheduler.clone();
            let release_wait = release.clone();
            running_handles.push(tokio::spawn(async move {
                scheduler
                    .run_request(
                        metadata(
                            request_id,
                            "default",
                            LlmRequestPriority::Background,
                            LlmRequestKind::AnalysisReportMap,
                        ),
                        |_| {},
                        move |control| async move {
                            control
                                .run_cancellable(async move {
                                    release_wait.notified().await;
                                    Ok::<_, String>("done")
                                })
                                .await
                        },
                    )
                    .await
            }));
        }

        let scheduler_for_cancel = scheduler.clone();
        let queued = tokio::spawn(async move {
            scheduler_for_cancel
                .run_request(
                    metadata(
                        "queued",
                        "default",
                        LlmRequestPriority::Background,
                        LlmRequestKind::AnalysisReportMap,
                    ),
                    |_| {},
                    move |control| async move {
                        control
                            .run_cancellable(async move { Ok::<_, String>("should-not-run") })
                            .await
                    },
                )
                .await
        });

        timeout(Duration::from_secs(1), async {
            loop {
                if scheduler.cancel_request("queued").await {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("queued request should register");

        assert_eq!(
            queued.await.expect("join queued"),
            Err(LlmRequestError::Cancelled)
        );

        release.notify_waiters();
        for handle in running_handles {
            let _ = handle.await;
        }

        assert_scheduler_empty(&scheduler).await;
    }

    #[tokio::test]
    async fn cancelling_owned_run_requests_aborts_running_work() {
        let scheduler = Arc::new(LlmSchedulerState::new());
        let scheduler_for_task = scheduler.clone();

        let task = tokio::spawn(async move {
            scheduler_for_task
                .run_request(
                    LlmRequestMetadata {
                        owner_run_id: Some(77),
                        ..metadata(
                            "owned",
                            "default",
                            LlmRequestPriority::Background,
                            LlmRequestKind::AnalysisReportReduce,
                        )
                    },
                    |_| {},
                    move |control| async move {
                        control
                            .run_cancellable(async move {
                                tokio::time::sleep(Duration::from_secs(30)).await;
                                Ok::<_, String>("done")
                            })
                            .await
                    },
                )
                .await
        });

        timeout(Duration::from_secs(1), async {
            loop {
                if scheduler.cancel_run_requests(77).await > 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("owned request should register");

        assert_eq!(
            task.await.expect("join task"),
            Err(LlmRequestError::Cancelled)
        );
        assert_scheduler_empty(&scheduler).await;
    }

    #[tokio::test]
    async fn request_snapshots_report_running_and_queued_requests() {
        let scheduler = Arc::new(LlmSchedulerState::new());
        let release = Arc::new(Notify::new());

        let mut handles = Vec::new();
        for request_id in ["run-1", "run-2"] {
            let scheduler = scheduler.clone();
            let release_wait = release.clone();
            handles.push(tokio::spawn(async move {
                scheduler
                    .run_request(
                        LlmRequestMetadata {
                            owner_run_id: Some(77),
                            ..metadata(
                                request_id,
                                "default",
                                LlmRequestPriority::Background,
                                LlmRequestKind::AnalysisReportMap,
                            )
                        },
                        |_| {},
                        move |control| async move {
                            control
                                .run_cancellable(async move {
                                    release_wait.notified().await;
                                    Ok::<_, String>("done")
                                })
                                .await
                        },
                    )
                    .await
            }));
        }

        let queued_scheduler = scheduler.clone();
        let queued = tokio::spawn(async move {
            queued_scheduler
                .run_request(
                    LlmRequestMetadata {
                        owner_run_id: Some(77),
                        ..metadata(
                            "queued",
                            "default",
                            LlmRequestPriority::Interactive,
                            LlmRequestKind::AnalysisChat,
                        )
                    },
                    |_| {},
                    move |control| async move {
                        control
                            .run_cancellable(async move { Ok::<_, String>("queued") })
                            .await
                    },
                )
                .await
        });

        let snapshots = timeout(Duration::from_secs(1), async {
            loop {
                let snapshots = scheduler.request_snapshots().await;
                if snapshots.len() == 3
                    && snapshots
                        .iter()
                        .filter(|snapshot| snapshot.state == LlmRequestSnapshotState::Running)
                        .count()
                        == 2
                    && snapshots
                        .iter()
                        .any(|snapshot| snapshot.state == LlmRequestSnapshotState::Queued)
                {
                    break snapshots;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("snapshots should include active requests");

        let queued_snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.request_id == "queued")
            .expect("queued snapshot");
        assert_eq!(queued_snapshot.kind, LlmRequestKind::AnalysisChat);
        assert_eq!(queued_snapshot.priority, LlmRequestPriority::Interactive);
        assert_eq!(queued_snapshot.state, LlmRequestSnapshotState::Queued);
        assert_eq!(queued_snapshot.queue_position, Some(1));
        assert_eq!(queued_snapshot.owner_run_id, Some(77));

        release.notify_waiters();
        for handle in handles {
            assert_eq!(handle.await.expect("join running"), Ok("done"));
        }
        assert_eq!(queued.await.expect("join queued"), Ok("queued"));
        assert!(scheduler.request_snapshots().await.is_empty());
    }

    #[tokio::test]
    async fn active_owner_run_ids_reports_running_and_queued_owned_requests() {
        let scheduler = Arc::new(LlmSchedulerState::new());
        let released = Arc::new(AtomicBool::new(false));
        let release = Arc::new(Notify::new());

        let mut handles = Vec::new();
        for (request_id, run_id) in [("owned-77", 77), ("owned-88", 88), ("owned-99", 99)] {
            let scheduler = scheduler.clone();
            let released = released.clone();
            let release = release.clone();
            handles.push(tokio::spawn(async move {
                scheduler
                    .run_request(
                        LlmRequestMetadata {
                            owner_run_id: Some(run_id),
                            ..metadata(
                                request_id,
                                "default",
                                LlmRequestPriority::Background,
                                LlmRequestKind::AnalysisReportMap,
                            )
                        },
                        |_| {},
                        move |control| async move {
                            control
                                .run_cancellable(async move {
                                    loop {
                                        if released.load(Ordering::SeqCst) {
                                            break;
                                        }
                                        release.notified().await;
                                    }
                                    Ok::<_, String>("done")
                                })
                                .await
                        },
                    )
                    .await
            }));
        }

        timeout(Duration::from_secs(1), async {
            loop {
                let snapshots = scheduler.request_snapshots().await;
                if snapshots.len() == 3
                    && snapshots
                        .iter()
                        .filter(|snapshot| snapshot.state == LlmRequestSnapshotState::Running)
                        .count()
                        == 2
                    && snapshots
                        .iter()
                        .any(|snapshot| snapshot.state == LlmRequestSnapshotState::Queued)
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("owned requests should register");

        let owners = scheduler.active_owner_run_ids().await;
        assert_eq!(owners, HashSet::from([77, 88, 99]));

        released.store(true, Ordering::SeqCst);
        release.notify_waiters();
        for handle in handles {
            assert_eq!(handle.await.expect("join owned"), Ok("done"));
        }
        assert!(scheduler.active_owner_run_ids().await.is_empty());
    }

    #[tokio::test]
    async fn queue_positions_are_recomputed_after_cancelling_a_queued_request() {
        let scheduler = Arc::new(LlmSchedulerState::new());
        let release = Arc::new(Notify::new());
        let (queue_tx, mut queue_rx) = mpsc::unbounded_channel::<(String, usize)>();
        let (started_tx, mut started_rx) = mpsc::unbounded_channel::<String>();

        let mut running_handles = Vec::new();
        for request_id in ["run-1", "run-2"] {
            let scheduler = scheduler.clone();
            let release_wait = release.clone();
            running_handles.push(tokio::spawn(async move {
                scheduler
                    .run_request(
                        metadata(
                            request_id,
                            "default",
                            LlmRequestPriority::Background,
                            LlmRequestKind::AnalysisReportMap,
                        ),
                        |_| {},
                        move |control| async move {
                            control
                                .run_cancellable(async move {
                                    release_wait.notified().await;
                                    Ok::<_, String>("done")
                                })
                                .await
                        },
                    )
                    .await
            }));
        }

        let mut queued_handles = Vec::new();
        for request_id in ["queued-1", "queued-2"] {
            let scheduler = scheduler.clone();
            let queue_tx = queue_tx.clone();
            let started_tx = started_tx.clone();
            queued_handles.push(tokio::spawn(async move {
                scheduler
                    .run_request(
                        metadata(
                            request_id,
                            "default",
                            LlmRequestPriority::Background,
                            LlmRequestKind::AnalysisReportMap,
                        ),
                        move |position| {
                            let _ = queue_tx.send((request_id.to_string(), position));
                        },
                        move |control| {
                            let started_tx = started_tx.clone();
                            async move {
                                let _ = started_tx.send(request_id.to_string());
                                control
                                    .run_cancellable(async move { Ok::<_, String>(request_id) })
                                    .await
                            }
                        },
                    )
                    .await
            }));
        }

        let mut seen_positions = Vec::new();
        timeout(Duration::from_secs(1), async {
            while seen_positions.len() < 3 {
                let entry = queue_rx.recv().await.expect("queue update");
                seen_positions.push(entry);
            }
        })
        .await
        .expect("queue updates should arrive");

        assert!(seen_positions.contains(&("queued-1".to_string(), 1)));
        assert!(seen_positions.contains(&("queued-2".to_string(), 2)));

        assert!(scheduler.cancel_request("queued-1").await);

        let recomputed = timeout(Duration::from_secs(1), queue_rx.recv())
            .await
            .expect("recomputed position")
            .expect("queue update");
        assert_eq!(recomputed, ("queued-2".to_string(), 1));

        release.notify_one();

        assert_eq!(
            timeout(Duration::from_secs(1), started_rx.recv())
                .await
                .expect("queued request should start")
                .expect("request id"),
            "queued-2".to_string()
        );

        release.notify_waiters();

        assert_eq!(
            queued_handles.remove(0).await.expect("join cancelled"),
            Err(LlmRequestError::Cancelled)
        );
        assert_eq!(
            queued_handles.remove(0).await.expect("join started"),
            Ok("queued-2")
        );
        for handle in running_handles {
            let _ = handle.await;
        }

        assert_scheduler_empty(&scheduler).await;
    }

    #[tokio::test]
    async fn failed_requests_release_capacity_for_next_queued_request() {
        let scheduler = Arc::new(LlmSchedulerState::new());
        let release = Arc::new(Notify::new());
        let (started_tx, mut started_rx) = mpsc::unbounded_channel::<String>();

        let first_scheduler = scheduler.clone();
        let first_release_wait = release.clone();
        let first = tokio::spawn(async move {
            first_scheduler
                .run_request(
                    metadata(
                        "run-1",
                        "default",
                        LlmRequestPriority::Background,
                        LlmRequestKind::AnalysisReportMap,
                    ),
                    |_| {},
                    move |control| async move {
                        control
                            .run_cancellable(async move {
                                first_release_wait.notified().await;
                                Ok::<_, String>("done")
                            })
                            .await
                    },
                )
                .await
        });

        let second_scheduler = scheduler.clone();
        let second_started_tx = started_tx.clone();
        let second = tokio::spawn(async move {
            second_scheduler
                .run_request(
                    metadata(
                        "run-2",
                        "default",
                        LlmRequestPriority::Background,
                        LlmRequestKind::AnalysisReportMap,
                    ),
                    |_| {},
                    move |control| {
                        let started_tx = second_started_tx.clone();
                        async move {
                            let _ = started_tx.send("run-2".to_string());
                            control
                                .run_cancellable(async move {
                                    Err::<&'static str, String>("boom".to_string())
                                })
                                .await
                        }
                    },
                )
                .await
        });

        let third_scheduler = scheduler.clone();
        let third_started_tx = started_tx.clone();
        let third = tokio::spawn(async move {
            third_scheduler
                .run_request(
                    metadata(
                        "run-3",
                        "default",
                        LlmRequestPriority::Background,
                        LlmRequestKind::AnalysisReportMap,
                    ),
                    |_| {},
                    move |control| {
                        let started_tx = third_started_tx.clone();
                        async move {
                            let _ = started_tx.send("run-3".to_string());
                            control
                                .run_cancellable(async move { Ok::<_, String>("done") })
                                .await
                        }
                    },
                )
                .await
        });

        assert_eq!(
            timeout(Duration::from_secs(1), started_rx.recv())
                .await
                .expect("failing request should start")
                .expect("request id"),
            "run-2".to_string()
        );
        assert_eq!(
            timeout(Duration::from_secs(1), started_rx.recv())
                .await
                .expect("queued request should start after failure")
                .expect("request id"),
            "run-3".to_string()
        );

        release.notify_waiters();

        assert_eq!(first.await.expect("join first"), Ok("done"));
        assert_eq!(
            second.await.expect("join second"),
            Err(LlmRequestError::Failed(AppError::from("boom")))
        );
        assert_eq!(third.await.expect("join third"), Ok("done"));

        assert_scheduler_empty(&scheduler).await;
    }

    #[tokio::test]
    async fn failed_requests_preserve_typed_error_kind() {
        let scheduler = LlmSchedulerState::new();

        let result = scheduler
            .run_request(
                metadata(
                    "typed-failure",
                    "default",
                    LlmRequestPriority::Background,
                    LlmRequestKind::ProviderTest,
                ),
                |_| {},
                |control| async move {
                    control
                        .run_cancellable(async move {
                            Err::<(), _>(AppError::validation("bad request"))
                        })
                        .await
                },
            )
            .await;

        let error = match result {
            Err(LlmRequestError::Failed(error)) => error,
            other => panic!("expected typed failed request, got {other:?}"),
        };

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "bad request");
    }
}
