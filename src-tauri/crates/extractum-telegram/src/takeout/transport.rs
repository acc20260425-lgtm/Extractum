use std::sync::Arc;

use grammers_client::Client;
use grammers_session::storages::MemorySession;

use super::types::{TakeoutAttempt, TakeoutFallback};

struct ExportDcAttemptTracker {
    attempt: TakeoutAttempt,
    fallbacks: Vec<TakeoutFallback>,
    fallback_used: bool,
}

impl ExportDcAttemptTracker {
    fn new(home_dc_id: i32, export_dc_id: i32) -> Self {
        Self {
            attempt: TakeoutAttempt::new(home_dc_id, export_dc_id),
            fallbacks: Vec::new(),
            fallback_used: false,
        }
    }

    fn attempt(&self) -> TakeoutAttempt {
        self.attempt
    }

    fn queue_fallback(&mut self, fallback: TakeoutFallback) {
        if fallback.kind() == super::types::TakeoutFallbackKind::ExportDc {
            self.fallback_used = true;
        }
        self.fallbacks.push(fallback);
    }

    fn drain_fallbacks(&mut self) -> Vec<TakeoutFallback> {
        std::mem::take(&mut self.fallbacks)
    }
}

pub struct TakeoutTransport {
    client: Client,
    session: Arc<MemorySession>,
    attempts: ExportDcAttemptTracker,
}

impl TakeoutTransport {
    pub(super) fn new(
        client: Client,
        session: Arc<MemorySession>,
        home_dc_id: i32,
        export_dc_id: i32,
    ) -> Self {
        Self {
            client,
            session,
            attempts: ExportDcAttemptTracker::new(home_dc_id, export_dc_id),
        }
    }

    pub fn export_dc_attempt(&self) -> TakeoutAttempt {
        self.attempts.attempt()
    }

    pub fn drain_fallbacks(&mut self) -> Vec<TakeoutFallback> {
        self.attempts.drain_fallbacks()
    }

    pub(super) fn queue_fallback(&mut self, fallback: TakeoutFallback) {
        self.attempts.queue_fallback(fallback);
    }

    pub(super) fn client(&self) -> &Client {
        &self.client
    }

    pub(super) fn session(&self) -> &Arc<MemorySession> {
        &self.session
    }

    pub(super) fn home_dc_id(&self) -> i32 {
        self.attempts.attempt().home_dc_id()
    }

    pub(super) fn export_dc_id(&self) -> Option<i32> {
        (!self.attempts.fallback_used).then(|| self.attempts.attempt().export_dc_id())
    }
}

#[cfg(test)]
mod tests {
    use std::{future::Future, sync::Arc};

    use super::super::types::{TakeoutAttempt, TakeoutFallback, TakeoutFallbackKind};
    use super::ExportDcAttemptTracker;

    async fn invoke_fake_remote<T, E, F>(
        tracker: &mut ExportDcAttemptTracker,
        observed_attempt: &std::sync::Mutex<Option<TakeoutAttempt>>,
        fallback: TakeoutFallback,
        remote: F,
    ) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        *observed_attempt.lock().expect("lock observed attempt") = Some(tracker.attempt());
        tracker.queue_fallback(fallback);
        remote.await
    }

    #[tokio::test]
    async fn transport_reports_attempt_and_fallback_after_success_or_error() {
        let mut tracker = ExportDcAttemptTracker::new(2, 40_002);
        let observed_attempt = Arc::new(std::sync::Mutex::new(None));

        for expected_result in [Ok(7_i32), Err("remote failure")] {
            let observed_before_poll = Arc::clone(&observed_attempt);
            let fallback = TakeoutFallback::new(
                TakeoutFallbackKind::ExportDc,
                "export DC fallback warning".to_string(),
                Some("export DC fallback provenance".to_string()),
            );

            let result = invoke_fake_remote(
                &mut tracker,
                observed_attempt.as_ref(),
                fallback,
                async move {
                    let attempt = observed_before_poll
                        .lock()
                        .expect("lock attempt before fake remote poll")
                        .expect("attempt snapshot must exist before fake remote poll");
                    assert_eq!(attempt.home_dc_id(), 2);
                    assert_eq!(attempt.export_dc_id(), 40_002);
                    expected_result
                },
            )
            .await;

            assert_eq!(result, expected_result);
        }

        let fallbacks = tracker.drain_fallbacks();
        assert_eq!(fallbacks.len(), 2);
        assert!(fallbacks
            .iter()
            .all(|fallback| fallback.kind() == TakeoutFallbackKind::ExportDc));
        assert!(fallbacks
            .iter()
            .all(|fallback| fallback.warning() == "export DC fallback warning"));
        assert!(fallbacks.iter().all(|fallback| {
            fallback.provenance_message() == Some("export DC fallback provenance")
        }));
        assert!(tracker.drain_fallbacks().is_empty());
    }
}
