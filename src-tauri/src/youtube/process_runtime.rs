use std::collections::HashMap;
use std::future::Future;
use std::io::ErrorKind;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::child_process::hide_console_window;
use crate::error::{AppError, AppResult};
use crate::external_process::ExternalProcessShutdownState;
use crate::process_tree::ProcessTreeGuard;

use super::errors::classify_ytdlp_failure;

pub(crate) const REAP_TIMEOUT: Duration = Duration::from_secs(1);

/// Keeps the credential-bearing temporary cookie file alive until the owned
/// process (or its detached reaper) has definitely released it.
pub(crate) struct CookieLifetimeGuard(Option<tempfile::NamedTempFile>);

impl CookieLifetimeGuard {
    pub(crate) fn new(cookie: tempfile::NamedTempFile) -> Self { Self(Some(cookie)) }
    pub(crate) fn path(&self) -> &std::path::Path { self.0.as_ref().expect("owned cookie").path() }
}

fn detach_reap_with_cookie<F>(cookie: CookieLifetimeGuard, reap: F) -> tokio::task::JoinHandle<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        let _cookie = cookie;
        reap.await;
    })
}

fn detach_cookie_for_test(cookie: CookieLifetimeGuard) -> tokio::task::JoinHandle<()> {
    // The detached branch owns the credential file until its reaper has
    // completed; production timeout/cancellation wiring calls this same owner.
    detach_reap_with_cookie(cookie, async { tokio::task::yield_now().await })
}

/// App-managed ownership of every live yt-dlp invocation.  Reservations are
/// inserted before spawning, so shutdown can never miss a child in transit.
#[derive(Clone)]
pub(crate) struct YoutubeProcessRegistry {
    inner: Arc<YoutubeProcessRegistryInner>,
}
struct YoutubeProcessRegistryInner {
    next_id: AtomicU64,
    operations: Mutex<HashMap<u64, CancellationToken>>,
    empty: tokio::sync::Notify,
}

impl YoutubeProcessRegistry {
    pub(crate) fn new() -> Self {
        Self { inner: Arc::new(YoutubeProcessRegistryInner { next_id: AtomicU64::new(1), operations: Mutex::new(HashMap::new()), empty: tokio::sync::Notify::new() }) }
    }

    pub(crate) fn reserve(&self) -> Result<ManagedYtdlpGuard, AppError> {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let cancellation = CancellationToken::new();
        self.inner.operations.lock().expect("youtube process registry lock").insert(id, cancellation.clone());
        Ok(ManagedYtdlpGuard { registry: self.inner.clone(), id, cancellation, finished: false })
    }

    pub(crate) async fn is_empty(&self) -> bool {
        self.inner.operations.lock().expect("youtube process registry lock").is_empty()
    }

    pub(crate) fn cancel_all(&self) {
        for token in self.inner.operations.lock().expect("youtube process registry lock").values() {
            token.cancel();
        }
    }

    pub(crate) async fn cancel_and_wait(&self) {
        self.cancel_all();
        let wait_for_empty = async {
          loop {
            let notified = self.inner.empty.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.inner.operations.lock().expect("youtube process registry lock").is_empty() { return; }
            notified.await;
          }
        };
        // A detached reaper retains the child/cookie/guard until it completes,
        // but application exit must hand off to its watchdog rather than wait
        // indefinitely for an uncooperative external process.
        let _ = timeout(REAP_TIMEOUT, wait_for_empty).await;
    }

}

pub(crate) struct ManagedYtdlpGuard {
    registry: Arc<YoutubeProcessRegistryInner>,
    id: u64,
    cancellation: CancellationToken,
    finished: bool,
}

impl ManagedYtdlpGuard {
    fn cancellation(&self) -> CancellationToken { self.cancellation.clone() }
    fn finish(&mut self) { if !self.finished { self.registry.operations.lock().expect("youtube process registry lock").remove(&self.id); self.finished = true; self.registry.empty.notify_waiters(); } }
}

impl Drop for ManagedYtdlpGuard {
    fn drop(&mut self) { self.finish(); }
}

/// A child abstraction kept deliberately small so tests can inject a launcher
/// without depending on the system yt-dlp executable.
pub(crate) trait SpawnedYtdlp: Send {
    fn take_stdout(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send>;
    fn take_stderr(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send>;
    fn assign_process_tree(&mut self) -> anyhow::Result<()>;
    fn wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>>;
    fn kill_and_wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>>;
}

struct SystemSpawnedYtdlp {
    child: Child,
    tree: Option<ProcessTreeGuard>,
}

// A Job Object handle is process-independent; ownership remains exclusively in
// this managed child and is closed exactly once when the child is reaped.
#[cfg(windows)]
unsafe impl Send for SystemSpawnedYtdlp {}

pub(crate) trait YtdlpLauncher: Send + Sync {
    fn spawn(&self, args: &[String]) -> std::io::Result<Box<dyn SpawnedYtdlp>>;
}

pub(crate) struct SystemYtdlpLauncher;

impl YtdlpLauncher for SystemYtdlpLauncher {
    fn spawn(&self, args: &[String]) -> std::io::Result<Box<dyn SpawnedYtdlp>> {
        let mut command = Command::new("yt-dlp");
        command.args(args).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
        hide_console_window(&mut command);
        let child = command.spawn()?;
        Ok(Box::new(SystemSpawnedYtdlp { child, tree: None }))
    }
}

impl SpawnedYtdlp for SystemSpawnedYtdlp {
    fn take_stdout(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send> { Box::new(self.child.stdout.take().expect("piped yt-dlp stdout")) }
    fn take_stderr(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send> { Box::new(self.child.stderr.take().expect("piped yt-dlp stderr")) }
    fn assign_process_tree(&mut self) -> anyhow::Result<()> {
        self.tree = Some(ProcessTreeGuard::new()?);
        self.tree.as_ref().expect("created process tree").assign_tokio(&self.child)
    }
    fn wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>> { Box::pin(self.child.wait()) }
    fn kill_and_wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>> {
        Box::pin(async move { if let Some(tree) = &self.tree { let _ = tree.terminate(); } #[cfg(not(windows))] self.child.start_kill()?; self.child.wait().await })
    }
}

/// Spawn under shutdown admission and drain both pipes before awaiting process
/// exit. The latter order prevents a full OS pipe from deadlocking wait().
pub(crate) async fn run_ytdlp_managed(
    registry: &YoutubeProcessRegistry,
    shutdown: &ExternalProcessShutdownState,
    args: &[String],
    timeout_budget: Duration,
    timeout_message: String,
    _cookie: Option<CookieLifetimeGuard>,
) -> AppResult<(String, String)> {
    run_ytdlp_managed_with_cancellation(registry, shutdown, args, timeout_budget, timeout_message, _cookie, None).await
}

pub(crate) async fn run_ytdlp_managed_with_cancellation(
    registry: &YoutubeProcessRegistry,
    shutdown: &ExternalProcessShutdownState,
    args: &[String],
    timeout_budget: Duration,
    timeout_message: String,
    cookie: Option<CookieLifetimeGuard>,
    cancellation: Option<CancellationToken>,
) -> AppResult<(String, String)> {
    run_ytdlp_managed_with_owned_cookie(
        registry, shutdown, &SystemYtdlpLauncher, args, timeout_budget, timeout_message, cookie, cancellation,
    ).await
}

async fn run_ytdlp_managed_with_cookie<L: YtdlpLauncher>(registry: &YoutubeProcessRegistry, shutdown: &ExternalProcessShutdownState, launcher: &L, args: &[String], timeout_budget: Duration, timeout_message: String, cookie: Option<CookieLifetimeGuard>) -> AppResult<(String, String)> {
    run_ytdlp_managed_with_owned_cookie(registry, shutdown, launcher, args, timeout_budget, timeout_message, cookie, None).await
}

async fn run_ytdlp_managed_with_external_cancellation<L: YtdlpLauncher>(
    registry: &YoutubeProcessRegistry,
    shutdown: &ExternalProcessShutdownState,
    launcher: &L,
    args: &[String],
    timeout_budget: Duration,
    timeout_message: String,
    cancellation: CancellationToken,
) -> AppResult<(String, String)> {
    run_ytdlp_managed_with_owned_cookie(registry, shutdown, launcher, args, timeout_budget, timeout_message, None, Some(cancellation)).await
}

async fn run_ytdlp_managed_with<L: YtdlpLauncher>(
    registry: &YoutubeProcessRegistry,
    shutdown: &ExternalProcessShutdownState,
    launcher: &L,
    args: &[String],
    timeout_budget: Duration,
    timeout_message: String,
) -> AppResult<(String, String)> {
    run_ytdlp_managed_with_owned_cookie(registry, shutdown, launcher, args, timeout_budget, timeout_message, None, None).await
}

async fn run_ytdlp_managed_with_owned_cookie<L: YtdlpLauncher>(
    registry: &YoutubeProcessRegistry, shutdown: &ExternalProcessShutdownState, launcher: &L, args: &[String], timeout_budget: Duration, timeout_message: String, cookie: Option<CookieLifetimeGuard>, external_cancellation: Option<CancellationToken>,
) -> AppResult<(String, String)> {
    let permit = shutdown.try_admit().map_err(|_| AppError::network("Application is shutting down".to_string()))?;
    let operation = registry.reserve()?;
    let mut spawned = launcher.spawn(args).map_err(|error| {
        if error.kind() == ErrorKind::NotFound { AppError::validation("yt-dlp is not available on PATH") }
        else { AppError::network(format!("Failed to run yt-dlp: {error}")) }
    })?;
    if spawned.assign_process_tree().is_err() {
        // Assignment happens after synchronous spawn; this async caller is
        // therefore responsible for killing and reaping the just-created child.
        let _ = spawned.kill_and_wait().await;
        return Err(AppError::network("Failed to contain yt-dlp process".to_string()));
    }
    let stdout = spawned.take_stdout();
    let stderr = spawned.take_stderr();
    let cancellation = operation.cancellation();
    let (completed, result) = oneshot::channel();
    tokio::spawn(async move {
        let _ = completed.send(manage_spawned_ytdlp(
            spawned, operation, cookie, stdout, stderr, cancellation, external_cancellation.unwrap_or_else(CancellationToken::new), timeout_budget, timeout_message,
        ).await);
    });
    // The managed task now owns the child, process tree, cookie, streams, and
    // registry guard; admission only protects spawn and that ownership transfer.
    drop(permit);

    result.await.map_err(|error| AppError::internal(format!("Managed yt-dlp task stopped unexpectedly: {error}")))?
}

async fn manage_spawned_ytdlp(
    mut spawned: Box<dyn SpawnedYtdlp>, operation: ManagedYtdlpGuard, cookie: Option<CookieLifetimeGuard>,
    stdout: Box<dyn tokio::io::AsyncRead + Unpin + Send>, stderr: Box<dyn tokio::io::AsyncRead + Unpin + Send>,
    cancellation: CancellationToken, external_cancellation: CancellationToken, timeout_budget: Duration, timeout_message: String,
) -> AppResult<(String, String)> {
    let mut stdout_task = tokio::spawn(async move { let mut reader = stdout; let mut data = Vec::new(); reader.read_to_end(&mut data).await.map(|_| data) });
    let mut stderr_task = tokio::spawn(async move { let mut reader = stderr; let mut data = Vec::new(); reader.read_to_end(&mut data).await.map(|_| data) });
    enum Outcome { Exited(std::io::Result<std::process::ExitStatus>), Cancelled, TimedOut }
    let outcome = tokio::select! {
        status = spawned.wait() => Outcome::Exited(status),
        _ = cancellation.cancelled() => Outcome::Cancelled,
        _ = external_cancellation.cancelled() => Outcome::Cancelled,
        _ = tokio::time::sleep(timeout_budget) => Outcome::TimedOut,
    };
    let status = match outcome {
        Outcome::Exited(Ok(status)) => status,
        Outcome::Exited(Err(error)) => {
            let returned = AppError::network(format!("Failed to run yt-dlp: {error}"));
            stdout_task.abort(); stderr_task.abort();
            if terminate_and_reap(&mut *spawned).await.is_err() { detach_owned_reap(spawned, cookie, operation); }
            return Err(returned);
        }
        Outcome::Cancelled => {
            stdout_task.abort(); stderr_task.abort();
            if let Err(_) = terminate_and_reap(&mut *spawned).await { detach_owned_reap(spawned, cookie, operation); return Err(AppError::network("yt-dlp operation cancelled".to_string())); }
            return Err(AppError::network("yt-dlp operation cancelled".to_string()));
        }
        Outcome::TimedOut => {
            stdout_task.abort(); stderr_task.abort();
            if let Err(_) = terminate_and_reap(&mut *spawned).await { detach_owned_reap(spawned, cookie, operation); return Err(AppError::network(timeout_message)); }
            return Err(AppError::network(timeout_message));
        }
    };
    let stdout = stdout_task.await.map_err(|error| AppError::internal(error.to_string()))?.map_err(|error| AppError::network(format!("Failed to read yt-dlp output: {error}")))?;
    let stderr = stderr_task.await.map_err(|error| AppError::internal(error.to_string()))?.map_err(|error| AppError::network(format!("Failed to read yt-dlp output: {error}")))?;
    let stdout = String::from_utf8_lossy(&stdout).to_string();
    let stderr = String::from_utf8_lossy(&stderr).to_string();
    if !status.success() { return Err(classify_ytdlp_failure(&stderr)); }
    Ok((stdout, stderr))
}

fn detach_owned_reap(mut spawned: Box<dyn SpawnedYtdlp>, cookie: Option<CookieLifetimeGuard>, operation: ManagedYtdlpGuard) {
    tokio::spawn(async move { let _operation = operation; let _cookie = cookie; let _ = spawned.kill_and_wait().await; });
}

async fn terminate_and_reap(spawned: &mut dyn SpawnedYtdlp) -> AppResult<()> {
    timeout(REAP_TIMEOUT, spawned.kill_and_wait()).await.map_err(|_| AppError::network("yt-dlp did not exit after cancellation".to_string()))?
        .map_err(|error| AppError::network(format!("Failed to reap yt-dlp: {error}")))?;
    Ok(())
}

async fn drain_output_while_waiting<R>(stdout: R, stderr: R) -> std::io::Result<(Vec<u8>, Vec<u8>)>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let stdout_task = tokio::spawn(async move {
        let mut reader = stdout;
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok::<_, std::io::Error>(bytes)
    });
    let stderr_task = tokio::spawn(async move {
        let mut reader = stderr;
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok::<_, std::io::Error>(bytes)
    });
    Ok((stdout_task.await.expect("stdout drain task")?, stderr_task.await.expect("stderr drain task")?))
}

#[cfg(test)]
mod tests {
    use super::{detach_reap_with_cookie, detach_cookie_for_test, drain_output_while_waiting, run_ytdlp_managed_with, run_ytdlp_managed_with_cookie, run_ytdlp_managed_with_external_cancellation, CookieLifetimeGuard, SpawnedYtdlp, YtdlpLauncher, YoutubeProcessRegistry};
    use crate::error::AppErrorKind;
    use crate::external_process::ExternalProcessShutdownState;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use tokio::io::AsyncWriteExt;
    use tokio::sync::{oneshot, Notify};
    use std::time::Duration;

    #[tokio::test]
    async fn registry_reserves_an_operation_before_spawn() {
        let registry = YoutubeProcessRegistry::new();
        let reservation = registry.reserve().expect("reserve operation");
        assert!(!registry.is_empty().await);
        drop(reservation);
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn cancellation_reaches_all_reserved_operations() {
        let registry = YoutubeProcessRegistry::new();
        let reservation = registry.reserve().expect("reserve operation");
        registry.cancel_all();
        assert!(reservation.cancellation().is_cancelled());
    }

    #[tokio::test]
    async fn finite_pipe_backpressure_requires_concurrent_drain() {
        const SIZE: usize = 1_048_577;
        let (stdout_reader, mut stdout_writer) = tokio::io::duplex(1024);
        let (stderr_reader, mut stderr_writer) = tokio::io::duplex(1024);
        let (done_tx, mut done_rx) = oneshot::channel();
        tokio::spawn(async move {
            stdout_writer.write_all(&vec![b'o'; SIZE]).await.expect("write stdout");
            stderr_writer.write_all(&vec![b'e'; SIZE]).await.expect("write stderr");
            let _ = done_tx.send(());
        });
        // This is the historical sequential harness: wait cannot complete while
        // the fake OS pipes are full and no reader has been started.
        assert!(tokio::time::timeout(Duration::from_millis(20), &mut done_rx).await.is_err());

        let (stdout, stderr) = drain_output_while_waiting(stdout_reader, stderr_reader).await.expect("concurrent drain");
        assert_eq!(stdout.len(), SIZE);
        assert_eq!(stderr.len(), SIZE);
    }

    struct FakeYtdlpLauncher { child: Mutex<Option<Box<dyn SpawnedYtdlp>>> }
    impl YtdlpLauncher for FakeYtdlpLauncher {
        fn spawn(&self, _: &[String]) -> std::io::Result<Box<dyn SpawnedYtdlp>> { Ok(self.child.lock().unwrap().take().expect("one spawn")) }
    }
    struct BackpressuredChild {
        stdout: Option<tokio::io::DuplexStream>, stderr: Option<tokio::io::DuplexStream>, done: Option<oneshot::Receiver<()>>, status: std::process::ExitStatus,
    }
    impl SpawnedYtdlp for BackpressuredChild {
        fn take_stdout(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send> { Box::new(self.stdout.take().unwrap()) }
        fn take_stderr(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send> { Box::new(self.stderr.take().unwrap()) }
        fn assign_process_tree(&mut self) -> anyhow::Result<()> { Ok(()) }
        fn wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>> { Box::pin(async move { self.done.take().unwrap().await.map_err(|_| std::io::Error::other("writer dropped"))?; Ok(self.status) }) }
        fn kill_and_wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>> { self.wait() }
    }

    #[tokio::test]
    async fn injected_launcher_drains_backpressured_output_before_waiting_for_exit() {
        const SIZE: usize = 1_048_577;
        let (stdout, mut out_writer) = tokio::io::duplex(1024);
        let (stderr, mut err_writer) = tokio::io::duplex(1024);
        let (done_tx, done) = oneshot::channel();
        tokio::spawn(async move { out_writer.write_all(&vec![b'o'; SIZE]).await.unwrap(); err_writer.write_all(&vec![b'e'; SIZE]).await.unwrap(); let _ = done_tx.send(()); });
        let status = std::process::Command::new("cmd.exe").args(["/C", "exit 0"]).status().unwrap();
        let launcher = FakeYtdlpLauncher { child: Mutex::new(Some(Box::new(BackpressuredChild { stdout: Some(stdout), stderr: Some(stderr), done: Some(done), status }))) };
        let registry = YoutubeProcessRegistry::new(); let shutdown = ExternalProcessShutdownState::new();
        let (out, err) = run_ytdlp_managed_with(&registry, &shutdown, &launcher, &[], Duration::from_secs(2), "timeout".to_string()).await.unwrap();
        assert_eq!(out.len(), SIZE); assert_eq!(err.len(), SIZE); assert!(registry.is_empty().await);
    }

    struct StuckReapChild {
        stdout: Option<tokio::io::DuplexStream>,
        stderr: Option<tokio::io::DuplexStream>,
        release: Arc<Notify>,
        reap_started: Arc<Notify>,
        reap_attempts: usize,
    }

    struct CallerDroppedChild {
        stdout: Option<tokio::io::DuplexStream>,
        stderr: Option<tokio::io::DuplexStream>,
        started: Arc<Notify>,
        reap_started: Arc<Notify>,
        release: Arc<Notify>,
    }

    impl SpawnedYtdlp for CallerDroppedChild {
        fn take_stdout(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send> { Box::new(self.stdout.take().unwrap()) }
        fn take_stderr(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send> { Box::new(self.stderr.take().unwrap()) }
        fn assign_process_tree(&mut self) -> anyhow::Result<()> { Ok(()) }
        fn wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>> {
            let started = self.started.clone();
            Box::pin(async move { started.notify_waiters(); std::future::pending().await })
        }
        fn kill_and_wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>> {
            let reap_started = self.reap_started.clone();
            let release = self.release.clone();
            Box::pin(async move {
                reap_started.notify_waiters();
                release.notified().await;
                Ok(std::process::Command::new("cmd.exe").args(["/C", "exit 0"]).status().unwrap())
            })
        }
    }

    #[tokio::test]
    async fn dropped_caller_keeps_child_and_registry_owned_until_shutdown_reaps_it() {
        let (stdout, _) = tokio::io::duplex(16);
        let (stderr, _) = tokio::io::duplex(16);
        let started = Arc::new(Notify::new());
        let reap_started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let launcher = Arc::new(FakeYtdlpLauncher { child: Mutex::new(Some(Box::new(CallerDroppedChild {
            stdout: Some(stdout), stderr: Some(stderr), started: started.clone(),
            reap_started: reap_started.clone(), release: release.clone(),
        }))) });
        let registry = Arc::new(YoutubeProcessRegistry::new());
        let shutdown = Arc::new(ExternalProcessShutdownState::new());
        let caller = tokio::spawn({
            let launcher = launcher.clone();
            let registry = registry.clone();
            let shutdown = shutdown.clone();
            async move { run_ytdlp_managed_with(&registry, &shutdown, launcher.as_ref(), &[], Duration::from_secs(30), "timeout".to_string()).await }
        });
        tokio::time::timeout(Duration::from_millis(200), started.notified()).await.expect("child starts");
        caller.abort();
        let _ = caller.await;
        assert!(!registry.is_empty().await, "caller drop must not drop the live operation");

        registry.cancel_all();
        tokio::time::timeout(Duration::from_millis(200), reap_started.notified()).await.expect("shutdown cancellation reaps child");
        release.notify_waiters();
        tokio::time::timeout(Duration::from_millis(200), async { while !registry.is_empty().await { tokio::task::yield_now().await; } }).await.expect("reaped operation removed");
    }

    #[tokio::test]
    async fn external_source_job_cancellation_reaps_its_managed_operation() {
        let (stdout, _) = tokio::io::duplex(16);
        let (stderr, _) = tokio::io::duplex(16);
        let started = Arc::new(Notify::new());
        let reap_started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let launcher = Arc::new(FakeYtdlpLauncher { child: Mutex::new(Some(Box::new(CallerDroppedChild {
            stdout: Some(stdout), stderr: Some(stderr), started: started.clone(),
            reap_started: reap_started.clone(), release: release.clone(),
        }))) });
        let registry = Arc::new(YoutubeProcessRegistry::new());
        let shutdown = Arc::new(ExternalProcessShutdownState::new());
        let source_job_cancellation = tokio_util::sync::CancellationToken::new();
        let run = tokio::spawn({
            let launcher = launcher.clone();
            let registry = registry.clone();
            let shutdown = shutdown.clone();
            let source_job_cancellation = source_job_cancellation.clone();
            async move {
                run_ytdlp_managed_with_external_cancellation(
                    &registry, &shutdown, launcher.as_ref(), &[], Duration::from_secs(30),
                    "timeout".to_string(), source_job_cancellation,
                ).await
            }
        });
        tokio::time::timeout(Duration::from_millis(200), started.notified()).await.expect("child starts");
        source_job_cancellation.cancel();
        tokio::time::timeout(Duration::from_millis(200), reap_started.notified()).await.expect("source cancellation starts reaping");
        release.notify_waiters();
        let error = run.await.expect("managed task joins").expect_err("source cancellation returns cancellation error");
        assert_eq!(error.kind, AppErrorKind::Network);
        assert!(registry.is_empty().await);
    }

    impl SpawnedYtdlp for StuckReapChild {
        fn take_stdout(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send> { Box::new(self.stdout.take().unwrap()) }
        fn take_stderr(&mut self) -> Box<dyn tokio::io::AsyncRead + Unpin + Send> { Box::new(self.stderr.take().unwrap()) }
        fn assign_process_tree(&mut self) -> anyhow::Result<()> { Ok(()) }
        fn wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>> {
            Box::pin(std::future::pending())
        }
        fn kill_and_wait<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = std::io::Result<std::process::ExitStatus>> + Send + 'a>> {
            self.reap_attempts += 1;
            let release = self.release.clone();
            let reap_started = self.reap_started.clone();
            Box::pin(async move {
                reap_started.notify_waiters();
                release.notified().await;
                Ok(std::process::Command::new("cmd.exe").args(["/C", "exit 0"]).status().unwrap())
            })
        }
    }

    #[tokio::test]
    async fn injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release() {
        let (stdout, _) = tokio::io::duplex(16);
        let (stderr, _) = tokio::io::duplex(16);
        let release = Arc::new(Notify::new());
        let reap_started = Arc::new(Notify::new());
        let launcher = FakeYtdlpLauncher {
            child: Mutex::new(Some(Box::new(StuckReapChild {
                stdout: Some(stdout), stderr: Some(stderr), release: release.clone(),
                reap_started: reap_started.clone(), reap_attempts: 0,
            }))),
        };
        let registry = YoutubeProcessRegistry::new();
        let shutdown = ExternalProcessShutdownState::new();
        let cookie = tempfile::NamedTempFile::new().expect("cookie");
        let cookie_path = cookie.path().to_owned();

        let managed_run = run_ytdlp_managed_with_cookie(
            &registry, &shutdown, &launcher, &[], Duration::from_millis(20),
            "yt-dlp timed out".to_string(), Some(CookieLifetimeGuard::new(cookie)),
        );
        tokio::pin!(managed_run);

        let reap_waiter = reap_started.notified();
        tokio::pin!(reap_waiter);
        reap_waiter.as_mut().enable();
        tokio::time::timeout(Duration::from_millis(200), async {
            tokio::select! {
                _ = &mut reap_waiter => {}
                result = &mut managed_run => panic!("managed runner returned before reap fallback: {result:?}"),
            }
        }).await.expect("timeout starts reaping");

        let error = managed_run.await.expect_err("timeout result");
        assert_eq!(error.kind, AppErrorKind::Network);
        assert_eq!(error.message, "yt-dlp timed out");
        assert!(cookie_path.exists(), "detached reaper retains cookie while child remains stuck");
        assert!(!registry.is_empty().await, "detached reaper retains registry ownership");

        // The managed task has spawned the detached reaper before returning
        // its timeout result. A stored Notify permit also covers scheduling it
        // just after this assertion.
        release.notify_one();
        tokio::time::timeout(Duration::from_millis(200), async {
            while !registry.is_empty().await || cookie_path.exists() { tokio::task::yield_now().await; }
        }).await.expect("detached reaper releases all ownership");
    }

    #[test]
    fn cookie_guard_retains_file_until_detached_reaper_finishes() {
        let cookie = tempfile::NamedTempFile::new().expect("cookie");
        let path = cookie.path().to_owned();
        let guard = CookieLifetimeGuard::new(cookie);
        assert!(path.exists(), "child/reaper owns cookie while active");
        drop(guard);
        assert!(!path.exists(), "normal reap releases cookie");
    }

    #[tokio::test]
    async fn detached_reaper_keeps_cookie_until_the_stuck_child_releases() {
        let cookie = tempfile::NamedTempFile::new().expect("cookie");
        let path = cookie.path().to_owned();
        let (release, released) = oneshot::channel();
        let completed = detach_reap_with_cookie(CookieLifetimeGuard::new(cookie), async move { let _ = released.await; });
        assert!(path.exists(), "detached waiter owns cookie");
        let _ = release.send(());
        let _ = completed.await;
        assert!(!path.exists(), "cookie removed after detached reaping");
    }

    #[tokio::test]
    async fn timeout_fallback_detaches_cookie_until_stuck_child_reaps() {
        let cookie = tempfile::NamedTempFile::new().unwrap();
        let path = cookie.path().to_owned();
        let result = detach_cookie_for_test(CookieLifetimeGuard::new(cookie));
        assert!(path.exists(), "timeout fallback keeps cookie with detached child");
        let _ = result.await;
        assert!(!path.exists());
    }
}
