use std::collections::{HashMap, HashSet};

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct AnalysisState {
    active_report_runs: Mutex<HashSet<i64>>,
    report_run_tokens: Mutex<HashMap<i64, CancellationToken>>,
}

impl AnalysisState {
    pub fn new() -> Self {
        Self {
            active_report_runs: Mutex::new(HashSet::new()),
            report_run_tokens: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn insert_active_report_run(&self, run_id: i64) {
        self.active_report_runs.lock().await.insert(run_id);
        self.report_run_tokens
            .lock()
            .await
            .insert(run_id, CancellationToken::new());
    }

    pub(crate) async fn remove_active_report_run(&self, run_id: i64) {
        self.active_report_runs.lock().await.remove(&run_id);
        self.report_run_tokens.lock().await.remove(&run_id);
    }

    pub(crate) async fn active_report_run_ids(&self) -> HashSet<i64> {
        self.active_report_runs.lock().await.clone()
    }

    pub(super) async fn request_report_run_cancel(&self, run_id: i64) -> bool {
        let active_runs = self.active_report_runs.lock().await;
        if !active_runs.contains(&run_id) {
            return false;
        }
        drop(active_runs);
        self.ensure_report_run_token(run_id).await.cancel();
        true
    }

    pub(super) async fn is_report_run_cancelled(&self, run_id: i64) -> bool {
        self.report_run_tokens
            .lock()
            .await
            .get(&run_id)
            .is_some_and(CancellationToken::is_cancelled)
    }

    pub(crate) async fn report_run_child_token(&self, run_id: i64) -> Option<CancellationToken> {
        self.report_run_tokens
            .lock()
            .await
            .get(&run_id)
            .map(CancellationToken::child_token)
    }

    async fn ensure_report_run_token(&self, run_id: i64) -> CancellationToken {
        self.report_run_tokens
            .lock()
            .await
            .entry(run_id)
            .or_insert_with(CancellationToken::new)
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::AnalysisState;

    #[tokio::test]
    async fn analysis_state_cancels_report_run_child_tokens() {
        let state = AnalysisState::new();

        state.insert_active_report_run(42).await;
        let child = state.report_run_child_token(42).await.expect("child token");
        assert!(!child.is_cancelled());

        assert!(state.request_report_run_cancel(42).await);
        tokio::time::timeout(std::time::Duration::from_secs(1), child.cancelled())
            .await
            .expect("child token cancelled");
        assert!(state.is_report_run_cancelled(42).await);

        state.remove_active_report_run(42).await;
        assert!(state.report_run_child_token(42).await.is_none());
        assert!(!state.is_report_run_cancelled(42).await);
    }
}
