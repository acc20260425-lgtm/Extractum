use std::{path::PathBuf, process::Stdio, time::Duration};

use tauri::{AppHandle, Manager};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::runtime::Handle;
use tokio::time::timeout;

use crate::error::AppResult;
use crate::{external_process::ExternalProcessShutdownState, process_tree::ProcessTreeGuard};

use super::domain_error::{GeminiBrowserError, GeminiBrowserResult};
use super::protocol::{classify_resume_response, GeminiBrowserJsonlCodec, ResumeSidecarOutcome};
use super::sidecar_launch::{
    bundled_sidecar_path, resolve_launch_mode, GeminiBrowserBuildProfile,
    GeminiBrowserSidecarLaunch,
};
use super::{
    GeminiBrowserProviderConfig, GeminiBrowserProviderStatus, GeminiBrowserRunRequest,
    GeminiBrowserRunResult, GeminiBrowserSidecarCommand, GeminiBrowserSidecarResponse,
    GeminiBrowserState,
};

enum GeminiBrowserSidecarTransport {
    Node {
        child: Option<Child>,
        stdin: Option<ChildStdin>,
        stdout: BufReader<ChildStdout>,
        process_tree: ProcessTreeGuard,
    },
}

fn bundled_sidecar_path_from_current_exe() -> std::io::Result<PathBuf> {
    std::env::current_exe().map(|executable| bundled_sidecar_path(&executable))
}

pub(crate) struct GeminiBrowserSidecarProcess {
    transport: GeminiBrowserSidecarTransport,
    codec: GeminiBrowserJsonlCodec,
    next_id: u64,
}

impl GeminiBrowserSidecarProcess {
    async fn spawn(handle: &AppHandle) -> GeminiBrowserResult<Self> {
        let shutdown = handle
            .state::<ExternalProcessShutdownState>()
            .inner()
            .clone();
        let _admission = shutdown.try_admit().map_err(|_| {
            GeminiBrowserError::transport("Gemini browser sidecar is shutting down")
        })?;
        let repo_root = std::env::current_dir()
            .map_err(|error| GeminiBrowserError::transport(error.to_string()))?;
        let dev_script = super::sidecar_launch::dev_sidecar_script(&repo_root);
        let build_profile = if cfg!(debug_assertions) {
            GeminiBrowserBuildProfile::Debug
        } else {
            GeminiBrowserBuildProfile::Release
        };
        let force_dev = std::env::var("EXTRACTUM_GEMINI_BROWSER_DEV_SIDECAR")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let force_bundled = std::env::var("EXTRACTUM_GEMINI_BROWSER_BUNDLED_SIDECAR")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        match resolve_launch_mode(
            build_profile,
            force_dev,
            force_bundled,
            &repo_root,
            dev_script.exists(),
        ) {
            GeminiBrowserSidecarLaunch::DevNodeScript { node, script } => {
                Self::spawn_node_script(node, script).await
            }
            GeminiBrowserSidecarLaunch::Bundled { .. } => Self::spawn_bundled().await,
        }
    }

    async fn spawn_node_script(node: String, script_path: PathBuf) -> GeminiBrowserResult<Self> {
        let process_tree = ProcessTreeGuard::new().map_err(|_| {
            GeminiBrowserError::transport("Failed to contain Gemini browser sidecar")
        })?;
        let mut command = Command::new(node);
        command.arg(script_path);
        configure_sidecar_command(&mut command);
        let child = command.spawn().map_err(|error| {
            GeminiBrowserError::transport(format!(
                "Failed to start Gemini browser sidecar: {error}"
            ))
        })?;
        Self::install_node_child(child, process_tree).await
    }

    async fn spawn_bundled() -> GeminiBrowserResult<Self> {
        let path = bundled_sidecar_path_from_current_exe().map_err(|error| {
            GeminiBrowserError::transport(format!("Gemini sidecar bundle is unavailable: {error}"))
        })?;
        let process_tree = ProcessTreeGuard::new().map_err(|_| {
            GeminiBrowserError::transport("Failed to contain Gemini browser sidecar")
        })?;
        let mut command = Command::new(path);
        configure_sidecar_command(&mut command);
        let child = command.spawn().map_err(|error| {
            GeminiBrowserError::transport(format!(
                "Failed to start bundled Gemini sidecar: {error}"
            ))
        })?;
        Self::install_node_child(child, process_tree).await
    }

    async fn install_node_child(
        mut child: Child,
        process_tree: ProcessTreeGuard,
    ) -> GeminiBrowserResult<Self> {
        if process_tree.assign_tokio(&child).is_err() {
            let _ = child.kill().await;
            return Err(GeminiBrowserError::transport(
                "Failed to contain Gemini browser sidecar",
            ));
        }
        Self::from_node_child(child, process_tree)
    }

    fn from_node_child(
        mut child: Child,
        process_tree: ProcessTreeGuard,
    ) -> GeminiBrowserResult<Self> {
        let stdin = child.stdin.take().ok_or_else(|| {
            GeminiBrowserError::transport("Gemini browser sidecar stdin was unavailable")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            GeminiBrowserError::transport("Gemini browser sidecar stdout was unavailable")
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            GeminiBrowserError::transport("Gemini browser sidecar stderr was unavailable")
        })?;
        tokio::spawn(drain_sidecar_stderr(stderr));
        Ok(Self {
            transport: GeminiBrowserSidecarTransport::Node {
                child: Some(child),
                stdin: Some(stdin),
                stdout: BufReader::new(stdout),
                process_tree,
            },
            codec: GeminiBrowserJsonlCodec::new(),
            next_id: 1,
        })
    }

    async fn request(
        &mut self,
        command: GeminiBrowserSidecarCommand,
    ) -> GeminiBrowserResult<GeminiBrowserSidecarResponse> {
        let id = format!("gemini-sidecar-{}", self.next_id);
        self.next_id += 1;

        let codec = &mut self.codec;
        match &mut self.transport {
            GeminiBrowserSidecarTransport::Node { stdin, stdout, .. } => {
                let stdin = stdin.as_mut().ok_or_else(|| {
                    GeminiBrowserError::transport("Gemini browser sidecar stdin was already closed")
                })?;
                request_node(stdin, stdout, codec, &id, command).await
            }
        }
    }

    async fn graceful_shutdown(mut self) {
        let _ = self.request(GeminiBrowserSidecarCommand::Stop).await;
        let GeminiBrowserSidecarTransport::Node {
            child,
            stdin,
            process_tree,
            ..
        } = &mut self.transport;
        stdin.take();
        if let Some(child) = child.as_mut() {
            if timeout(Duration::from_secs(1), child.wait()).await.is_err() {
                let _ = process_tree.terminate();
                let _ = child.kill().await;
            }
        }
    }
}

fn configure_sidecar_command(command: &mut Command) {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    crate::child_process::hide_console_window(command);
}

impl Drop for GeminiBrowserSidecarProcess {
    fn drop(&mut self) {
        match &mut self.transport {
            GeminiBrowserSidecarTransport::Node {
                child,
                process_tree,
                ..
            } => {
                let _ = process_tree.terminate();
                if let Some(mut child) = child.take() {
                    let _ = child.start_kill();
                    if let Some(handle) = Handle::try_current().ok() {
                        handle.spawn(async move {
                            let _ = child.kill().await;
                        });
                    } else if let Ok(runtime) = tokio::runtime::Runtime::new() {
                        runtime.block_on(async move {
                            let _ = child.kill().await;
                        });
                    }
                }
            }
        }
    }
}

async fn request_node(
    stdin: &mut ChildStdin,
    stdout: &mut BufReader<ChildStdout>,
    codec: &mut GeminiBrowserJsonlCodec,
    id: &str,
    command: GeminiBrowserSidecarCommand,
) -> GeminiBrowserResult<GeminiBrowserSidecarResponse> {
    request_jsonl(stdin, stdout, codec, id, command).await
}

async fn request_jsonl<W, R>(
    stdin: &mut W,
    stdout: &mut BufReader<R>,
    codec: &mut GeminiBrowserJsonlCodec,
    id: &str,
    command: GeminiBrowserSidecarCommand,
) -> GeminiBrowserResult<GeminiBrowserSidecarResponse>
where
    W: AsyncWrite + Unpin + Send,
    R: AsyncRead + Unpin + Send,
{
    let line = codec.encode_request(id, &command)?;
    stdin.write_all(&line).await.map_err(|error| {
        GeminiBrowserError::transport(format!("Failed to write Gemini sidecar request: {error}"))
    })?;
    stdin.flush().await.map_err(|error| {
        GeminiBrowserError::transport(format!("Failed to flush Gemini sidecar request: {error}"))
    })?;

    loop {
        let mut response_chunk = [0_u8; 8_192];
        let bytes = stdout.read(&mut response_chunk).await.map_err(|error| {
            GeminiBrowserError::transport(format!(
                "Failed to read Gemini sidecar response: {error}"
            ))
        })?;
        if bytes == 0 {
            return Err(GeminiBrowserError::transport(
                "Gemini browser sidecar exited without a response",
            ));
        }
        if let Some(response) = codec.push_response_bytes(id, &response_chunk[..bytes])? {
            return Ok(response);
        }
    }
}

async fn drain_sidecar_stderr<R>(mut stderr: R)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut buffer = [0_u8; 8_192];
    while stderr
        .read(&mut buffer)
        .await
        .ok()
        .is_some_and(|read| read > 0)
    {}
}

async fn request_sidecar(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    command: GeminiBrowserSidecarCommand,
) -> GeminiBrowserResult<GeminiBrowserSidecarResponse> {
    let mut sidecar = state.sidecar().await;
    if sidecar.is_none() {
        *sidecar = Some(GeminiBrowserSidecarProcess::spawn(handle).await?);
    }
    let process = sidecar.as_mut().ok_or_else(|| {
        GeminiBrowserError::invariant("Gemini browser sidecar was not initialized")
    })?;
    let response = if let Some(cancellation) = state.cancellation_token().await {
        tokio::select! {
            response = process.request(command) => response,
            _ = cancellation.cancelled() => {
                state.mark_sidecar_tainted().await;
                *sidecar = None;
                return Err(GeminiBrowserError::cancellation("Gemini browser sidecar request was cancelled"));
            }
        }
    } else {
        process.request(command).await
    };
    match response {
        Ok(GeminiBrowserSidecarResponse::Error { message }) => {
            Err(GeminiBrowserError::browser(message))
        }
        Ok(response) => Ok(response),
        Err(error) => {
            *sidecar = None;
            Err(error)
        }
    }
}

pub(crate) async fn status(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_profile_dir: String,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    match request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::Status {
            browser_profile_dir,
            browser_config,
        },
    )
    .await
    {
        Ok(GeminiBrowserSidecarResponse::Status { status }) => Ok(status),
        Ok(_) => Err(GeminiBrowserError::invariant(
            "Unexpected Gemini sidecar status response",
        )),
        Err(error) => Err(error),
    }
}

pub(crate) async fn open_browser(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_profile_dir: String,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    match request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::OpenBrowser {
            browser_profile_dir,
            browser_config,
        },
    )
    .await?
    {
        GeminiBrowserSidecarResponse::Status { status } => Ok(status),
        _ => Err(GeminiBrowserError::invariant(
            "Unexpected Gemini sidecar open_browser response",
        )),
    }
}

pub(crate) async fn resume(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_profile_dir: String,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus> {
    let first_response = request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::Resume {
            run_id: None,
            browser_profile_dir: browser_profile_dir.clone(),
            browser_config: browser_config.clone(),
        },
    )
    .await?;

    match classify_resume_response(first_response)? {
        ResumeSidecarOutcome::Status(status) => Ok(status),
        ResumeSidecarOutcome::LegacyAck => {
            *state.sidecar().await = None;
            let retry_response = request_sidecar(
                handle,
                state,
                GeminiBrowserSidecarCommand::Resume {
                    run_id: None,
                    browser_profile_dir,
                    browser_config,
                },
            )
            .await?;

            match classify_resume_response(retry_response)? {
                ResumeSidecarOutcome::Status(status) => Ok(status),
                ResumeSidecarOutcome::LegacyAck => Err(GeminiBrowserError::protocol(
                    "Gemini sidecar resume protocol is outdated after restart",
                )),
            }
        }
    }
}

pub(crate) async fn send_single(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    request: GeminiBrowserRunRequest,
    browser_profile_dir: String,
    artifact_dir: String,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> GeminiBrowserResult<GeminiBrowserRunResult> {
    match request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::SendSingle {
            request,
            browser_profile_dir,
            artifact_dir,
            browser_config,
        },
    )
    .await?
    {
        GeminiBrowserSidecarResponse::RunResult { result } => Ok(result),
        _ => Err(GeminiBrowserError::invariant(
            "Unexpected Gemini sidecar send_single response",
        )),
    }
}

pub(crate) async fn stop(_handle: &AppHandle, state: &GeminiBrowserState) -> AppResult<()> {
    let tainted = state.sidecar_tainted().await;
    let process = state.sidecar().await.take();
    if !tainted {
        if let Some(process) = process {
            process.graceful_shutdown().await;
        }
    }
    state.clear_sidecar_taint().await;
    Ok(())
}

pub(crate) async fn shutdown_sidecar(handle: &AppHandle, state: &GeminiBrowserState) {
    let _ = stop(handle, state).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stderr_drain_consumes_sidecar_output_concurrently() {
        let (mut writer, reader) = tokio::io::duplex(1024);
        let drain = tokio::spawn(drain_sidecar_stderr(reader));

        writer
            .write_all(b"sidecar diagnostic\n")
            .await
            .expect("write stderr");
        drop(writer);

        drain.await.expect("stderr drain completes");
    }
}
