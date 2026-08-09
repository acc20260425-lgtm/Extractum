#[cfg(test)]
mod tests {
    use crate::external_process::{
        ExitCallback, ExternalProcessShutdownState, MonotonicClock, ShutdownCleanup, ShutdownStart,
        ShutdownTiming, WatchdogScheduler,
    };
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    #[tokio::test(start_paused = true)]
    async fn exit_requests_delegate_once_and_complete_bounded_cleanup() {
        let state = ExternalProcessShutdownState::new();
        let scheduled = Arc::new(Mutex::new(Vec::new()));
        let recorded = scheduled.clone();
        let scheduler: WatchdogScheduler = Arc::new(move |timing, _| {
            recorded.lock().expect("scheduler log").push(timing);
        });
        let exits = Arc::new(Mutex::new(Vec::new()));
        let recorded_exits = exits.clone();
        let exit: ExitCallback = Arc::new(move |code| {
            recorded_exits.lock().expect("exit log").push(code);
        });
        let std_origin = Instant::now();
        let tokio_origin = tokio::time::Instant::now();
        let clock: MonotonicClock =
            Arc::new(move || std_origin + tokio::time::Instant::now().duration_since(tokio_origin));
        let timing = ShutdownTiming {
            graceful: Duration::from_secs(3),
            watchdog: Duration::from_secs(4),
        };
        let ShutdownStart::Started(run) =
            state.start(Some(23), timing, &scheduler, exit.clone(), clock.clone())
        else {
            panic!("first exit request must delegate to coordinator");
        };
        assert!(matches!(
            state.start(Some(99), timing, &scheduler, exit, clock),
            ShutdownStart::AlreadyShuttingDown
        ));
        let cleanups: Vec<ShutdownCleanup> = vec![Box::pin(async {
            tokio::time::sleep(Duration::from_secs(4)).await;
            Ok(())
        })];
        let task = tokio::spawn(run.coordinate(Box::new(move || cleanups)));
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(3)).await;
        task.await.expect("bounded cleanup task");

        assert_eq!(
            scheduled.lock().expect("scheduler log").as_slice(),
            &[timing]
        );
        assert_eq!(exits.lock().expect("exit log").as_slice(), &[23]);
    }
}
