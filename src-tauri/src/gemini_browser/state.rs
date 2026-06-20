use std::collections::VecDeque;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::GeminiBrowserRunRequest;

#[derive(Default)]
pub struct GeminiBrowserState {
    queue: Mutex<VecDeque<GeminiBrowserRunRequest>>,
    active_run_id: Mutex<Option<String>>,
    cancellation: Mutex<Option<CancellationToken>>,
}

impl GeminiBrowserState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn enqueue(&self, request: GeminiBrowserRunRequest) -> usize {
        let mut queue = self.queue.lock().await;
        queue.push_back(request);
        queue.len()
    }

    pub async fn pop_next(&self) -> Option<GeminiBrowserRunRequest> {
        self.queue.lock().await.pop_front()
    }

    pub async fn queue_depth(&self) -> usize {
        self.queue.lock().await.len()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gemini_browser::GeminiBrowserRunRequest;

    #[tokio::test]
    async fn queue_tracks_depth_and_active_run() {
        let state = GeminiBrowserState::new();
        let position = state
            .enqueue(GeminiBrowserRunRequest {
                run_id: "run-1".to_string(),
                prompt: "hello".to_string(),
                source: "test".to_string(),
                artifact_mode: "reduced".to_string(),
            })
            .await;
        assert_eq!(position, 1);
        assert_eq!(state.queue_depth().await, 1);

        let next = state.pop_next().await.expect("queued request");
        let token = state.start_run(next.run_id.clone()).await;
        assert!(!token.is_cancelled());
        assert_eq!(state.active_run_id().await, Some("run-1".to_string()));
        assert!(state.request_stop().await);
        assert!(token.is_cancelled());
        state.finish_run("run-1").await;
        assert_eq!(state.active_run_id().await, None);
    }
}
