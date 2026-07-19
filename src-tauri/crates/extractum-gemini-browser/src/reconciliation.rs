use std::{collections::BTreeMap, future::Future};

use super::{
    error::GeminiBrowserResult, state::GeminiBrowserDomainState, GeminiBrowserArtifactRefs,
    GeminiBrowserRun, GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizedQueueState {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueueInspectionSnapshot {
    Unavailable,
    Available(BTreeMap<String, NormalizedQueueState>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupReconciliationSnapshot {
    pub runs: Vec<GeminiBrowserRun>,
    pub queue: QueueInspectionSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationAction {
    Finish {
        run_id: String,
        result: GeminiBrowserRunResult,
    },
}

pub fn reconcile_startup(snapshot: StartupReconciliationSnapshot) -> Vec<ReconciliationAction> {
    snapshot
        .runs
        .into_iter()
        .filter_map(|run| reconcile_run(run, &snapshot.queue))
        .collect()
}

fn reconcile_run(
    run: GeminiBrowserRun,
    queue: &QueueInspectionSnapshot,
) -> Option<ReconciliationAction> {
    if run.status.is_terminal() {
        return None;
    }
    let result = match (&run.status, queue) {
        (GeminiBrowserRunStatus::Running, QueueInspectionSnapshot::Unavailable) => {
            interrupted_result(&run.run_id)
        }
        (GeminiBrowserRunStatus::Queued, QueueInspectionSnapshot::Unavailable) => return None,
        (status, QueueInspectionSnapshot::Available(states)) => match states.get(&run.run_id) {
            Some(NormalizedQueueState::Succeeded) => failed_result(
                &run.run_id,
                "Gemini Browser Apalis job completed before run log captured a result",
            ),
            Some(NormalizedQueueState::Failed) => failed_result(
                &run.run_id,
                "Gemini Browser Apalis job failed before completion",
            ),
            Some(NormalizedQueueState::Queued) if *status == GeminiBrowserRunStatus::Queued => {
                return None
            }
            Some(NormalizedQueueState::Running) if *status == GeminiBrowserRunStatus::Queued => {
                failed_result(
                    &run.run_id,
                    "Gemini Browser queue state was running without an active sidecar",
                )
            }
            Some(NormalizedQueueState::Queued | NormalizedQueueState::Running)
                if *status == GeminiBrowserRunStatus::Running =>
            {
                interrupted_result(&run.run_id)
            }
            None if *status == GeminiBrowserRunStatus::Queued => failed_result(
                &run.run_id,
                "Gemini Browser queued job was missing from Apalis storage",
            ),
            None => interrupted_result(&run.run_id),
            _ => return None,
        },
        _ => return None,
    };
    Some(ReconciliationAction::Finish {
        run_id: run.run_id,
        result,
    })
}

pub async fn ensure_startup_reconciled<Load, LoadFuture, Apply, ApplyFuture>(
    state: &GeminiBrowserDomainState,
    load_snapshot: Load,
    apply_actions: Apply,
) -> GeminiBrowserResult<()>
where
    Load: FnOnce() -> LoadFuture + Send,
    LoadFuture: Future<Output = GeminiBrowserResult<StartupReconciliationSnapshot>> + Send,
    Apply: FnOnce(Vec<ReconciliationAction>) -> ApplyFuture + Send,
    ApplyFuture: Future<Output = GeminiBrowserResult<()>> + Send,
{
    state
        .ensure_startup_reconciled(|| async {
            let snapshot = load_snapshot().await?;
            apply_actions(reconcile_startup(snapshot)).await
        })
        .await
}

fn interrupted_result(run_id: &str) -> GeminiBrowserRunResult {
    failed_result(
        run_id,
        "Gemini Browser worker was interrupted before completion",
    )
}

fn failed_result(run_id: &str, message: &str) -> GeminiBrowserRunResult {
    GeminiBrowserRunResult {
        run_id: run_id.to_string(),
        status: GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some(message.to_string()),
        manual_action: None,
        artifacts: GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 0,
        debug_summary: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(run_id: &str, status: GeminiBrowserRunStatus) -> GeminiBrowserRun {
        GeminiBrowserRun {
            run_id: run_id.to_string(),
            source: "settings_test".to_string(),
            status,
            prompt_preview: "hello".to_string(),
            created_at: "2026-07-19T00:00:00Z".to_string(),
            updated_at: "2026-07-19T00:00:00Z".to_string(),
            result: None,
        }
    }

    fn message(action: &ReconciliationAction) -> Option<&str> {
        match action {
            ReconciliationAction::Finish { result, .. } => result.message.as_deref(),
        }
    }

    #[test]
    fn restart_reconciliation_degraded_leaves_queued_run_log_records() {
        let actions = reconcile_startup(StartupReconciliationSnapshot {
            runs: vec![run("queued", GeminiBrowserRunStatus::Queued)],
            queue: QueueInspectionSnapshot::Unavailable,
        });
        assert!(actions.is_empty());
    }

    #[test]
    fn degraded_apalis_queue_inspection_leaves_queued_run_log_records_for_worker_entry() {
        let actions = reconcile_startup(StartupReconciliationSnapshot {
            runs: vec![run("queued", GeminiBrowserRunStatus::Queued)],
            queue: QueueInspectionSnapshot::Unavailable,
        });
        assert_eq!(actions, Vec::<ReconciliationAction>::new());
    }

    #[test]
    fn restart_reconciliation_matrix_handles_supported_apalis_states() {
        let runs = vec![
            run("queued", GeminiBrowserRunStatus::Queued),
            run("missing", GeminiBrowserRunStatus::Queued),
            run("running", GeminiBrowserRunStatus::Running),
            run("done", GeminiBrowserRunStatus::Queued),
            run("failed", GeminiBrowserRunStatus::Running),
        ];
        let queue = BTreeMap::from([
            ("queued".to_string(), NormalizedQueueState::Queued),
            ("running".to_string(), NormalizedQueueState::Running),
            ("done".to_string(), NormalizedQueueState::Succeeded),
            ("failed".to_string(), NormalizedQueueState::Failed),
        ]);
        let actions = reconcile_startup(StartupReconciliationSnapshot {
            runs,
            queue: QueueInspectionSnapshot::Available(queue),
        });
        assert_eq!(actions.len(), 4);
        let messages = actions.iter().filter_map(message).collect::<Vec<_>>();
        assert!(messages.contains(&"Gemini Browser queued job was missing from Apalis storage"));
        assert!(messages.contains(&"Gemini Browser worker was interrupted before completion"));
        assert!(messages
            .contains(&"Gemini Browser Apalis job completed before run log captured a result"));
        assert!(messages.contains(&"Gemini Browser Apalis job failed before completion"));
    }
}
