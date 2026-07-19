use std::{collections::BTreeMap, future::Future};

use super::{
    domain_error::GeminiBrowserResult, portable_state::GeminiBrowserDomainState,
    GeminiBrowserArtifactRefs, GeminiBrowserRun, GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedQueueState {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum QueueInspectionSnapshot {
    Unavailable,
    Available(BTreeMap<String, NormalizedQueueState>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StartupReconciliationSnapshot {
    pub(crate) runs: Vec<GeminiBrowserRun>,
    pub(crate) queue: QueueInspectionSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ReconciliationAction {
    Finish {
        run_id: String,
        result: GeminiBrowserRunResult,
    },
}

pub(crate) fn reconcile_startup(
    snapshot: StartupReconciliationSnapshot,
) -> Vec<ReconciliationAction> {
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

pub(crate) async fn ensure_startup_reconciled<Load, LoadFuture, Apply, ApplyFuture>(
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
