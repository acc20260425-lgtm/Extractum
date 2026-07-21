use std::collections::{HashMap, HashSet};
use std::future::Future;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::events::PromptPackEvent;
use crate::error::AppResult;
use crate::llm::LlmRequestError;

#[derive(Default)]
pub struct PromptPackRunState {
    active: Mutex<HashSet<i64>>,
    cancellation_tokens: Mutex<HashMap<i64, CancellationToken>>,
}

impl PromptPackRunState {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
        self.active.lock().await.insert(run_id);
        self.ensure_cancellation_token(run_id).await;
        Ok(())
    }

    pub(crate) async fn track_if_absent(&self, run_id: i64) -> AppResult<bool> {
        let inserted = self.active.lock().await.insert(run_id);
        self.ensure_cancellation_token(run_id).await;
        Ok(inserted)
    }

    pub(crate) async fn request_cancel(&self, run_id: i64) -> AppResult<()> {
        self.ensure_cancellation_token(run_id).await.cancel();
        Ok(())
    }

    pub(crate) async fn child_token(&self, run_id: i64) -> Option<CancellationToken> {
        self.cancellation_tokens
            .lock()
            .await
            .get(&run_id)
            .map(CancellationToken::child_token)
    }

    pub(crate) async fn finish(&self, run_id: i64) {
        self.active.lock().await.remove(&run_id);
        self.cancellation_tokens.lock().await.remove(&run_id);
    }

    pub(crate) async fn active_run_ids(&self) -> Vec<i64> {
        let mut ids = self.active.lock().await.iter().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    pub(crate) async fn apply_event(&self, event: &PromptPackEvent) {
        if matches!(
            event.kind.as_str(),
            "completed" | "partial" | "failed" | "cancelled" | "interrupted"
        ) {
            self.finish(event.run_id).await;
        }
    }

    async fn ensure_cancellation_token(&self, run_id: i64) -> CancellationToken {
        self.cancellation_tokens
            .lock()
            .await
            .entry(run_id)
            .or_insert_with(CancellationToken::new)
            .clone()
    }
}

pub(super) async fn run_with_prompt_pack_run_cancellation<Fut, T>(
    run_cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>,
{
    let Some(run_cancellation_token) = run_cancellation_token else {
        return future.await;
    };

    if run_cancellation_token.is_cancelled() {
        return Err(LlmRequestError::Cancelled);
    }

    tokio::select! {
        result = future => result,
        _ = run_cancellation_token.cancelled() => Err(LlmRequestError::Cancelled),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use super::PromptPackRunState;
    use crate::prompt_packs::events::{PromptPackEvent, PromptPackEventSink};

    struct StateObservingSink {
        state: Arc<PromptPackRunState>,
        observed_active: AtomicBool,
    }

    impl PromptPackEventSink for StateObservingSink {
        fn emit(&self, event: PromptPackEvent) {
            let active = self
                .state
                .active
                .try_lock()
                .expect("state lock must be released before synchronous emit")
                .contains(&event.run_id);
            self.observed_active.store(active, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn apply_event_updates_state_before_synchronous_sink_observes_it() {
        let state = Arc::new(PromptPackRunState::new());
        state.track(77).await.expect("track run");
        let sink = StateObservingSink {
            state: state.clone(),
            observed_active: AtomicBool::new(true),
        };
        let event = PromptPackEvent {
            run_id: 77,
            request_id: "run-77-terminal".to_string(),
            kind: "completed".to_string(),
            run_status: "complete".to_string(),
            phase: "terminal".to_string(),
            stage_run_id: None,
            stage_name: None,
            source_snapshot_id: None,
            queue_position: None,
            progress_current: Some(1),
            progress_total: Some(1),
            message: Some("Completed".to_string()),
            error: None,
        };

        state.apply_event(&event).await;
        sink.emit(event);

        assert!(!sink.observed_active.load(Ordering::SeqCst));
    }
}
