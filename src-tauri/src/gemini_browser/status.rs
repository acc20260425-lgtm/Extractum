use std::{path::Path, time::Duration};

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::{
    browser_executor::{BrowserExecutor, BrowserSessionContext, StatusObserver},
    domain_error::{GeminiBrowserErrorKind, GeminiBrowserResult},
    portable_state::GeminiBrowserDomainState,
    run_log, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind, GeminiBrowserRun,
    GeminiBrowserRunStatus,
};

const STATUS_SNAPSHOT_RUN_SCAN_LIMIT: usize = 200;
const STATUS_SNAPSHOT_ACTIVITY_GRACE_MINUTES: i64 = 30;

pub(crate) async fn read_provider_status(
    state: &GeminiBrowserDomainState,
    executor: &dyn BrowserExecutor,
    session: BrowserSessionContext,
    queue_depth: usize,
    live_timeout: Duration,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    let browser_profile_dir = session.browser_profile_dir.clone();
    let cached = state.status_snapshot(browser_profile_dir.clone())?;
    let active_run_id = state.active_run_id().await;
    match tokio::time::timeout(live_timeout, executor.status(session)).await {
        Ok(Ok(mut status)) => {
            status.active_run_id = active_run_id;
            status.queue_depth = queue_depth;
            Ok(status)
        }
        Ok(Err(error))
            if matches!(
                error.kind(),
                GeminiBrowserErrorKind::Transport
                    | GeminiBrowserErrorKind::Protocol
                    | GeminiBrowserErrorKind::Browser
            ) =>
        {
            let mut status = GeminiBrowserDomainState::not_started_status(browser_profile_dir);
            status.active_run_id = active_run_id;
            status.queue_depth = queue_depth;
            Ok(status)
        }
        Ok(Err(_)) | Err(_) => Ok(cached),
    }
}

pub(crate) fn read_reconciled_status_snapshot(
    state: &GeminiBrowserDomainState,
    runs_dir: &Path,
    browser_profile_dir: String,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    let snapshot = state.status_snapshot(browser_profile_dir.clone())?;
    let reconciled = status_snapshot_from_reconciled_run_log(
        runs_dir,
        snapshot.clone(),
        state.active_run_id_snapshot(),
    )?;
    if state.set_status_snapshot_if_current(&snapshot, reconciled.clone()) {
        Ok(reconciled)
    } else {
        state.status_snapshot(browser_profile_dir)
    }
}

pub(crate) async fn open_provider(
    executor: &dyn BrowserExecutor,
    observer: &dyn StatusObserver,
    session: BrowserSessionContext,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    let status = executor.open(session).await?;
    observer.publish(&status);
    Ok(status)
}

pub(crate) async fn resume_provider(
    executor: &dyn BrowserExecutor,
    observer: &dyn StatusObserver,
    session: BrowserSessionContext,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    let status = executor.resume(session).await?;
    observer.publish(&status);
    Ok(status)
}

fn status_snapshot_from_reconciled_run_log(
    runs_root: &Path,
    snapshot: GeminiBrowserProviderStatus,
    active_run_id: Option<String>,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    status_snapshot_from_reconciled_run_log_at(
        runs_root,
        snapshot,
        active_run_id,
        OffsetDateTime::now_utc(),
    )
}

fn status_snapshot_from_reconciled_run_log_at(
    runs_root: &Path,
    mut snapshot: GeminiBrowserProviderStatus,
    active_run_id: Option<String>,
    now: OffsetDateTime,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    let runs = run_log::list_runs(runs_root, STATUS_SNAPSHOT_RUN_SCAN_LIMIT)?.runs;
    let fresh_queued_count = runs
        .iter()
        .filter(|run| {
            run.status == GeminiBrowserRunStatus::Queued && run_log_activity_is_fresh(run, now)
        })
        .count();
    let stale_queued_count = runs
        .iter()
        .filter(|run| {
            run.status == GeminiBrowserRunStatus::Queued && !run_log_activity_is_fresh(run, now)
        })
        .count();
    if let Some(active_run_id) = active_run_id {
        snapshot.status = GeminiBrowserProviderStatusKind::Running;
        snapshot.active_run_id = Some(active_run_id);
        snapshot.queue_depth = fresh_queued_count;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some("Running".to_string());
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }
    if fresh_queued_count > 0 {
        snapshot.status = GeminiBrowserProviderStatusKind::Running;
        snapshot.active_run_id = None;
        snapshot.queue_depth = fresh_queued_count;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some("Queued".to_string());
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }
    snapshot.active_run_id = None;
    snapshot.queue_depth = 0;
    if stale_queued_count > 0 && snapshot.status == GeminiBrowserProviderStatusKind::Running {
        snapshot.status = GeminiBrowserProviderStatusKind::NotStarted;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some(
                "Gemini browser has stale queued run-log entries; waiting for cleanup.".to_string(),
            );
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }
    if snapshot.status == GeminiBrowserProviderStatusKind::Running {
        if let Some(latest) = runs.first().and_then(|run| run.result.as_ref()) {
            snapshot.status =
                GeminiBrowserDomainState::provider_status_kind_for_run_status(&latest.status);
            snapshot.latest_message = latest.message.clone();
            snapshot.manual_action = latest.manual_action.clone();
        } else {
            snapshot.status = GeminiBrowserProviderStatusKind::NotStarted;
            snapshot.latest_message = Some("Gemini browser sidecar is not running.".to_string());
            snapshot.manual_action = None;
        }
    }
    Ok(snapshot)
}

fn run_log_activity_is_fresh(run: &GeminiBrowserRun, now: OffsetDateTime) -> bool {
    let Ok(updated_at) = OffsetDateTime::parse(&run.updated_at, &Rfc3339) else {
        return false;
    };
    now - updated_at <= time::Duration::minutes(STATUS_SNAPSHOT_ACTIVITY_GRACE_MINUTES)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Instant,
    };

    use super::*;
    use crate::gemini_browser::{
        browser_executor::{BrowserExecutorFuture, BrowserRunContext, BrowserStopReason},
        domain_error::GeminiBrowserError,
        GeminiBrowserRunResult,
    };

    struct SlowExecutor;

    struct ReadyExecutor {
        calls: AtomicUsize,
    }

    impl BrowserExecutor for SlowExecutor {
        fn status(
            &self,
            _context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            Box::pin(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(GeminiBrowserProviderStatus {
                    status: GeminiBrowserProviderStatusKind::Ready,
                    manual_action: None,
                    active_run_id: None,
                    queue_depth: 0,
                    browser_profile_dir: "profile-dir".to_string(),
                    latest_message: Some("Ready".to_string()),
                })
            })
        }

        fn open(
            &self,
            _context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            Box::pin(async { Err(GeminiBrowserError::invariant("unused")) })
        }

        fn resume(
            &self,
            _context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            Box::pin(async { Err(GeminiBrowserError::invariant("unused")) })
        }

        fn send(
            &self,
            _context: BrowserRunContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserRunResult> {
            Box::pin(async { Err(GeminiBrowserError::invariant("unused")) })
        }

        fn stop(&self, _reason: BrowserStopReason) -> BrowserExecutorFuture<'_, ()> {
            Box::pin(async { Err(GeminiBrowserError::invariant("unused")) })
        }
    }

    impl BrowserExecutor for ReadyExecutor {
        fn status(
            &self,
            context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                Ok(GeminiBrowserProviderStatus {
                    status: GeminiBrowserProviderStatusKind::Ready,
                    manual_action: None,
                    active_run_id: None,
                    queue_depth: 0,
                    browser_profile_dir: context.browser_profile_dir,
                    latest_message: Some("Live ready".to_string()),
                })
            })
        }

        fn open(
            &self,
            _context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            Box::pin(async { Err(GeminiBrowserError::invariant("unused")) })
        }

        fn resume(
            &self,
            _context: BrowserSessionContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus> {
            Box::pin(async { Err(GeminiBrowserError::invariant("unused")) })
        }

        fn send(
            &self,
            _context: BrowserRunContext,
        ) -> BrowserExecutorFuture<'_, GeminiBrowserRunResult> {
            Box::pin(async { Err(GeminiBrowserError::invariant("unused")) })
        }

        fn stop(&self, _reason: BrowserStopReason) -> BrowserExecutorFuture<'_, ()> {
            Box::pin(async { Err(GeminiBrowserError::invariant("unused")) })
        }
    }

    #[tokio::test]
    async fn provider_status_uses_cached_snapshot_when_sidecar_is_busy() {
        let state = GeminiBrowserDomainState::default();
        state.set_status_snapshot(GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-busy".to_string()),
            queue_depth: 1,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("Running".to_string()),
        });
        let started = Instant::now();
        let status = read_provider_status(
            &state,
            &SlowExecutor,
            BrowserSessionContext {
                browser_profile_dir: "profile-dir".to_string(),
                browser_config: None,
            },
            0,
            Duration::from_millis(25),
        )
        .await
        .expect("cached status");
        assert!(started.elapsed() < Duration::from_millis(200));
        assert_eq!(status.status, GeminiBrowserProviderStatusKind::Running);
        assert_eq!(status.active_run_id.as_deref(), Some("run-busy"));
    }

    #[tokio::test]
    async fn provider_status_live_probe_does_not_mutate_cached_snapshot() {
        let state = GeminiBrowserDomainState::default();
        let cached = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: Some("run-cached".to_string()),
            queue_depth: 1,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: Some("Cached running".to_string()),
        };
        state.set_status_snapshot(cached.clone());
        let executor = ReadyExecutor {
            calls: AtomicUsize::new(0),
        };
        let returned = read_provider_status(
            &state,
            &executor,
            BrowserSessionContext {
                browser_profile_dir: "profile-dir".to_string(),
                browser_config: None,
            },
            0,
            Duration::from_millis(25),
        )
        .await
        .expect("live status");
        assert_eq!(returned.status, GeminiBrowserProviderStatusKind::Ready);
        assert_eq!(state.status_snapshot_option(), Some(cached));
    }

    #[test]
    fn status_snapshot_core_returns_cached_status_without_polling_live_sidecar() {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = GeminiBrowserDomainState::default();
        let cached = GeminiBrowserDomainState::not_started_status("profile-dir".to_string());
        state.set_status_snapshot(cached.clone());
        let returned =
            read_reconciled_status_snapshot(&state, temp.path(), "profile-dir".to_string())
                .expect("cached snapshot");
        assert_eq!(returned, cached);
    }

    #[test]
    fn provider_status_snapshot_from_reconciled_runs_does_not_keep_stale_running_snapshot() {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = GeminiBrowserDomainState::default();
        state.set_status_snapshot(GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: None,
            queue_depth: 0,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: None,
        });
        let status =
            read_reconciled_status_snapshot(&state, temp.path(), "profile-dir".to_string())
                .expect("reconciled snapshot");
        assert_eq!(status.status, GeminiBrowserProviderStatusKind::NotStarted);
    }

    #[tokio::test]
    async fn provider_status_snapshot_from_reconciled_runs_preserves_live_active_run() {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = GeminiBrowserDomainState::default();
        state.set_status_snapshot(GeminiBrowserDomainState::not_started_status(
            "profile-dir".to_string(),
        ));
        let _token = state.start_run("active-run".to_string()).await;
        let status =
            read_reconciled_status_snapshot(&state, temp.path(), "profile-dir".to_string())
                .expect("active snapshot");
        assert_eq!(status.status, GeminiBrowserProviderStatusKind::Running);
        assert_eq!(status.active_run_id.as_deref(), Some("active-run"));
    }

    #[test]
    fn provider_status_snapshot_from_reconciled_runs_ignores_stale_queued_rows() {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = GeminiBrowserDomainState::default();
        state.set_status_snapshot(GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: None,
            queue_depth: 4,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: None,
        });
        let status =
            read_reconciled_status_snapshot(&state, temp.path(), "profile-dir".to_string())
                .expect("stale rows ignored");
        assert_eq!(status.queue_depth, 0);
        assert_eq!(status.status, GeminiBrowserProviderStatusKind::NotStarted);
    }

    #[test]
    fn provider_status_snapshot_read_core_writes_reconciled_snapshot_back() {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = GeminiBrowserDomainState::default();
        state.set_status_snapshot(GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Running,
            manual_action: None,
            active_run_id: None,
            queue_depth: 0,
            browser_profile_dir: "profile-dir".to_string(),
            latest_message: None,
        });
        let returned =
            read_reconciled_status_snapshot(&state, temp.path(), "profile-dir".to_string())
                .expect("reconciled snapshot");
        assert_eq!(state.status_snapshot_option(), Some(returned));
    }

    #[test]
    fn provider_status_snapshot_read_core_skips_stale_write_back_when_snapshot_changed() {
        let state = GeminiBrowserDomainState::default();
        let expected = GeminiBrowserDomainState::not_started_status("profile-dir".to_string());
        let newer = GeminiBrowserProviderStatus {
            latest_message: Some("newer".to_string()),
            ..expected.clone()
        };
        state.set_status_snapshot(newer.clone());
        assert!(!state.set_status_snapshot_if_current(&expected, expected.clone()));
        assert_eq!(state.status_snapshot_option(), Some(newer));
    }

    #[tokio::test]
    async fn provider_status_read_core_waits_for_startup_reconciliation_before_live_status() {
        let state = GeminiBrowserDomainState::default();
        let order = std::sync::Arc::new(parking_lot::Mutex::new(Vec::new()));
        state
            .ensure_startup_reconciled({
                let order = order.clone();
                move || async move {
                    order.lock().push("reconcile");
                    Ok(())
                }
            })
            .await
            .expect("reconciled");
        let executor = ReadyExecutor {
            calls: AtomicUsize::new(0),
        };
        order.lock().push("status");
        read_provider_status(
            &state,
            &executor,
            BrowserSessionContext {
                browser_profile_dir: "profile-dir".to_string(),
                browser_config: None,
            },
            0,
            Duration::from_millis(25),
        )
        .await
        .expect("status");
        assert_eq!(order.lock().as_slice(), ["reconcile", "status"]);
    }
}
