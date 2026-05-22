use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceIngestKind {
    Sync,
    #[allow(dead_code)]
    TakeoutImport,
    Delete,
}

impl SourceIngestKind {
    fn label(self) -> &'static str {
        match self {
            Self::Sync => "sync",
            Self::TakeoutImport => "takeout import",
            Self::Delete => "delete",
        }
    }
}

#[derive(Default, Debug)]
struct SourceIngestLockState {
    active: HashMap<i64, SourceIngestKind>,
}

pub(crate) struct SourceIngestLocks {
    state: Arc<Mutex<SourceIngestLockState>>,
}

impl SourceIngestLocks {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SourceIngestLockState::default())),
        }
    }

    pub(crate) async fn try_acquire(
        &self,
        source_id: i64,
        kind: SourceIngestKind,
    ) -> AppResult<SourceIngestGuard> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AppError::internal("Source ingest lock state is poisoned"))?;
        if let Some(active_kind) = state.active.get(&source_id).copied() {
            return Err(AppError::conflict(format!(
                "Source {source_id} already has an active {} operation",
                active_kind.label()
            )));
        }

        state.active.insert(source_id, kind);

        Ok(SourceIngestGuard {
            source_id,
            state: Arc::clone(&self.state),
        })
    }

    pub(crate) async fn active_kinds_for_sources(
        &self,
        source_ids: &[i64],
    ) -> AppResult<HashMap<i64, SourceIngestKind>> {
        let source_ids = source_ids.iter().copied().collect::<HashSet<_>>();
        let state = self
            .state
            .lock()
            .map_err(|_| AppError::internal("Source ingest lock state is poisoned"))?;
        Ok(state
            .active
            .iter()
            .filter_map(|(source_id, kind)| {
                source_ids
                    .contains(source_id)
                    .then_some((*source_id, *kind))
            })
            .collect())
    }
}

#[derive(Debug)]
pub(crate) struct SourceIngestGuard {
    source_id: i64,
    state: Arc<Mutex<SourceIngestLockState>>,
}

impl Drop for SourceIngestGuard {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.active.remove(&self.source_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceIngestKind, SourceIngestLocks};
    use crate::error::AppErrorKind;

    #[tokio::test]
    async fn lock_rejects_concurrent_same_source_operations() {
        let locks = SourceIngestLocks::new();
        let _guard = locks
            .try_acquire(7, SourceIngestKind::Sync)
            .await
            .expect("first lock");

        let error = locks
            .try_acquire(7, SourceIngestKind::Delete)
            .await
            .expect_err("second lock should fail");

        assert_eq!(error.kind, AppErrorKind::Conflict);
        assert!(error.message.contains("Source 7"));
        assert!(error.message.contains("sync"));
    }

    #[tokio::test]
    async fn lock_allows_different_sources() {
        let locks = SourceIngestLocks::new();
        let _first = locks
            .try_acquire(7, SourceIngestKind::Sync)
            .await
            .expect("first lock");
        let _second = locks
            .try_acquire(8, SourceIngestKind::Delete)
            .await
            .expect("different source lock");
    }

    #[tokio::test]
    async fn lock_releases_when_guard_drops() {
        let locks = SourceIngestLocks::new();
        {
            let _guard = locks
                .try_acquire(7, SourceIngestKind::Sync)
                .await
                .expect("first lock");
        }

        let _next = locks
            .try_acquire(7, SourceIngestKind::TakeoutImport)
            .await
            .expect("lock should be released");
    }

    #[tokio::test]
    async fn active_kinds_for_sources_reports_matching_locks_only() {
        let locks = SourceIngestLocks::new();
        let _sync = locks
            .try_acquire(7, SourceIngestKind::Sync)
            .await
            .expect("sync lock");
        let _delete = locks
            .try_acquire(8, SourceIngestKind::Delete)
            .await
            .expect("delete lock");

        let active = locks
            .active_kinds_for_sources(&[7, 9, 8])
            .await
            .expect("active locks");

        assert_eq!(active.len(), 2);
        assert_eq!(active.get(&7), Some(&SourceIngestKind::Sync));
        assert_eq!(active.get(&8), Some(&SourceIngestKind::Delete));
        assert_eq!(active.get(&9), None);
    }
}
