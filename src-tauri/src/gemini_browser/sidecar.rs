use std::{path::PathBuf, process::Stdio, time::Duration};

use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::runtime::Handle;
use tokio::time::timeout;

use crate::error::{AppError, AppResult};
use crate::{external_process::ExternalProcessShutdownState, process_tree::ProcessTreeGuard};

use super::sidecar_launch::{
    bundled_sidecar_path_from_current_exe, resolve_launch_mode, GeminiBrowserBuildProfile, GeminiBrowserSidecarLaunch,
};
use super::{
    GeminiBrowserProviderConfig, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRunRequest, GeminiBrowserRunResult, GeminiBrowserRunStatus,
    GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope, GeminiBrowserSidecarResponse,
    GeminiBrowserState,
};

#[derive(Deserialize)]
struct SidecarLine {
    id: String,
    response: GeminiBrowserSidecarResponse,
}

enum GeminiBrowserSidecarTransport {
    Node {
        child: Option<Child>,
        stdin: Option<ChildStdin>,
        stdout: BufReader<ChildStdout>,
        process_tree: ProcessTreeGuard,
    },
}

pub(crate) struct GeminiBrowserSidecarProcess {
    transport: GeminiBrowserSidecarTransport,
    next_id: u64,
}

enum ResumeSidecarOutcome {
    Status(GeminiBrowserProviderStatus),
    LegacyAck,
}

impl GeminiBrowserSidecarProcess {
    async fn spawn(handle: &AppHandle) -> AppResult<Self> {
        let repo_root =
            std::env::current_dir().map_err(|error| AppError::internal(error.to_string()))?;
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
                Self::spawn_node_script(handle, node, script).await
            }
            GeminiBrowserSidecarLaunch::Bundled { .. } => Self::spawn_bundled(handle).await,
        }
    }

    async fn spawn_node_script(handle: &AppHandle, node: String, script_path: PathBuf) -> AppResult<Self> {
        let child = Command::new(node)
            .arg(script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                AppError::internal(format!("Failed to start Gemini browser sidecar: {error}"))
            })?;
        Self::install_node_child(handle, child).await
    }

    async fn spawn_bundled(handle: &AppHandle) -> AppResult<Self> {
        let path = bundled_sidecar_path_from_current_exe()
            .map_err(|error| AppError::internal(format!("Gemini sidecar bundle is unavailable: {error}")))?;
        let mut command = Command::new(path);
        command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        crate::child_process::hide_console_window(&mut command);
        let child = command.spawn().map_err(|error| {
            AppError::internal(format!("Failed to start bundled Gemini sidecar: {error}"))
        })?;
        Self::install_node_child(handle, child).await
    }

    async fn install_node_child(handle: &AppHandle, mut child: Child) -> AppResult<Self> {
        let shutdown = handle
            .state::<ExternalProcessShutdownState>()
            .inner()
            .clone();
        let _admission = shutdown
            .try_admit()
            .map_err(|_| AppError::internal("Gemini browser sidecar is shutting down"))?;
        let process_tree = ProcessTreeGuard::new()
            .map_err(|_| AppError::internal("Failed to contain Gemini browser sidecar"))?;
        if process_tree.assign_tokio(&child).is_err() {
            let _ = child.kill().await;
            return Err(AppError::internal("Failed to contain Gemini browser sidecar"));
        }
        Self::from_node_child(child, process_tree)
    }

    fn from_node_child(mut child: Child, process_tree: ProcessTreeGuard) -> AppResult<Self> {
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::internal("Gemini browser sidecar stdin was unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::internal("Gemini browser sidecar stdout was unavailable"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::internal("Gemini browser sidecar stderr was unavailable"))?;
        tokio::spawn(drain_sidecar_stderr(stderr));
        Ok(Self {
            transport: GeminiBrowserSidecarTransport::Node {
                child: Some(child),
                stdin: Some(stdin),
                stdout: BufReader::new(stdout),
                process_tree,
            },
            next_id: 1,
        })
    }

    async fn request(
        &mut self,
        command: GeminiBrowserSidecarCommand,
    ) -> AppResult<GeminiBrowserSidecarResponse> {
        let id = format!("gemini-sidecar-{}", self.next_id);
        self.next_id += 1;

        match &mut self.transport {
            GeminiBrowserSidecarTransport::Node { stdin, stdout, .. } => {
                let stdin = stdin.as_mut().ok_or_else(|| {
                    AppError::internal("Gemini browser sidecar stdin was already closed")
                })?;
                request_node(stdin, stdout, &id, command).await
            }
        }
    }

    async fn graceful_shutdown(mut self) {
        let _ = self.request(GeminiBrowserSidecarCommand::Stop).await;
        let GeminiBrowserSidecarTransport::Node { child, stdin, process_tree, .. } = &mut self.transport;
        stdin.take();
        if let Some(child) = child.as_mut() {
            if timeout(Duration::from_secs(1), child.wait()).await.is_err() {
                let _ = process_tree.terminate();
                let _ = child.kill().await;
            }
        }
    }
}

impl Drop for GeminiBrowserSidecarProcess {
    fn drop(&mut self) {
        match &mut self.transport {
            GeminiBrowserSidecarTransport::Node { child, process_tree, .. } => {
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
    id: &str,
    command: GeminiBrowserSidecarCommand,
) -> AppResult<GeminiBrowserSidecarResponse> {
    request_jsonl(stdin, stdout, id, command).await
}

async fn request_jsonl<W, R>(
    stdin: &mut W,
    stdout: &mut BufReader<R>,
    id: &str,
    command: GeminiBrowserSidecarCommand,
) -> AppResult<GeminiBrowserSidecarResponse>
where
    W: AsyncWrite + Unpin + Send,
    R: AsyncRead + Unpin + Send,
{
    let envelope = GeminiBrowserSidecarEnvelope {
        id: id.to_string(),
        command,
    };
    let mut line =
        serde_json::to_string(&envelope).map_err(|error| AppError::internal(error.to_string()))?;
    line.push('\n');
    stdin.write_all(line.as_bytes()).await.map_err(|error| {
        AppError::internal(format!("Failed to write Gemini sidecar request: {error}"))
    })?;
    stdin.flush().await.map_err(|error| {
        AppError::internal(format!("Failed to flush Gemini sidecar request: {error}"))
    })?;

    loop {
        let mut response_line = String::new();
        let bytes = stdout
            .read_line(&mut response_line)
            .await
            .map_err(|error| {
                AppError::internal(format!("Failed to read Gemini sidecar response: {error}"))
            })?;
        if bytes == 0 {
            return Err(AppError::internal(
                "Gemini browser sidecar exited without a response",
            ));
        }
        if let Some(response) = decode_sidecar_line_for_request(id, &response_line)? {
            return Ok(response);
        }
    }
}

async fn drain_sidecar_stderr<R>(mut stderr: R)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut buffer = [0_u8; 8_192];
    while stderr.read(&mut buffer).await.ok().is_some_and(|read| read > 0) {}
}

fn decode_sidecar_line(id: &str, response_line: &str) -> AppResult<GeminiBrowserSidecarResponse> {
    decode_sidecar_line_for_request(id, response_line)?
        .ok_or_else(|| AppError::internal("Gemini browser sidecar response id mismatch"))
}

fn decode_sidecar_line_for_request(
    id: &str,
    response_line: &str,
) -> AppResult<Option<GeminiBrowserSidecarResponse>> {
    let response: SidecarLine = serde_json::from_str(response_line)
        .map_err(|error| AppError::internal(format!("Invalid Gemini sidecar response: {error}")))?;
    if response.id != id {
        return Ok(None);
    }
    Ok(Some(response.response))
}

fn take_complete_jsonl_line(buffer: &mut String) -> Option<String> {
    while let Some(newline_index) = buffer.find('\n') {
        let line: String = buffer.drain(..=newline_index).collect();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}

async fn request_sidecar(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    command: GeminiBrowserSidecarCommand,
) -> AppResult<GeminiBrowserSidecarResponse> {
    let mut sidecar = state.sidecar().await;
    if sidecar.is_none() {
        *sidecar = Some(GeminiBrowserSidecarProcess::spawn(handle).await?);
    }
    let process = sidecar
        .as_mut()
        .ok_or_else(|| AppError::internal("Gemini browser sidecar was not initialized"))?;
    let response = if let Some(cancellation) = state.cancellation_token().await {
        tokio::select! {
            response = process.request(command) => response,
            _ = cancellation.cancelled() => {
                state.mark_sidecar_tainted().await;
                *sidecar = None;
                return Err(AppError::internal("Gemini browser sidecar request was cancelled"));
            }
        }
    } else {
        process.request(command).await
    };
    match response {
        Ok(GeminiBrowserSidecarResponse::Error { message }) => Err(AppError::internal(message)),
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
    active_run_id: Option<String>,
    queue_depth: usize,
) -> AppResult<GeminiBrowserProviderStatus> {
    match request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::Status {
            browser_profile_dir: browser_profile_dir.clone(),
            browser_config,
        },
    )
    .await
    {
        Ok(GeminiBrowserSidecarResponse::Status { mut status }) => {
            status.active_run_id = active_run_id;
            status.queue_depth = queue_depth;
            Ok(status)
        }
        Ok(_) => Err(AppError::internal(
            "Unexpected Gemini sidecar status response",
        )),
        Err(_) => Ok(GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::NotStarted,
            manual_action: None,
            active_run_id,
            queue_depth,
            browser_profile_dir,
            latest_message: Some("Gemini browser sidecar is not running.".to_string()),
        }),
    }
}

pub(crate) async fn open_browser(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_profile_dir: String,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
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
        _ => Err(AppError::internal(
            "Unexpected Gemini sidecar open_browser response",
        )),
    }
}

pub(crate) async fn resume(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_profile_dir: String,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserProviderStatus> {
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
                ResumeSidecarOutcome::LegacyAck => Err(AppError::internal(
                    "Gemini sidecar resume protocol is outdated after restart",
                )),
            }
        }
    }
}

fn classify_resume_response(
    response: GeminiBrowserSidecarResponse,
) -> AppResult<ResumeSidecarOutcome> {
    match response {
        GeminiBrowserSidecarResponse::Status { status } => Ok(ResumeSidecarOutcome::Status(status)),
        GeminiBrowserSidecarResponse::Ack => Ok(ResumeSidecarOutcome::LegacyAck),
        _ => Err(AppError::internal(
            "Unexpected Gemini sidecar resume response",
        )),
    }
}

pub(crate) async fn send_single(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    request: GeminiBrowserRunRequest,
    browser_profile_dir: String,
    artifact_dir: String,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserRunResult> {
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
        _ => Err(AppError::internal(
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

pub(crate) fn sidecar_unavailable_result(
    request: GeminiBrowserRunRequest,
) -> GeminiBrowserRunResult {
    GeminiBrowserRunResult {
        run_id: request.run_id,
        status: GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some("Gemini browser sidecar is unavailable.".to_string()),
        manual_action: None,
        artifacts: Default::default(),
        elapsed_ms: 0,
        debug_summary: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_sidecar_line_rejects_mismatched_ids() {
        let line = r#"{"id":"other","response":{"type":"ack"}}"#;

        let error = decode_sidecar_line("expected", line).unwrap_err();

        assert!(error.to_string().contains("response id mismatch"));
    }

    #[test]
    fn decode_sidecar_line_accepts_ack_for_matching_id() {
        let line = r#"{"id":"expected","response":{"type":"ack"}}"#;

        let response = decode_sidecar_line("expected", line).expect("decode response");

        assert!(matches!(response, GeminiBrowserSidecarResponse::Ack));
    }

    #[test]
    fn decode_sidecar_line_for_request_skips_stale_response_ids() {
        let stale = r#"{"id":"previous","response":{"type":"ack"}}"#;
        let expected = r#"{"id":"expected","response":{"type":"ack"}}"#;

        assert!(decode_sidecar_line_for_request("expected", stale)
            .expect("decode stale response")
            .is_none());
        assert!(matches!(
            decode_sidecar_line_for_request("expected", expected)
                .expect("decode expected response"),
            Some(GeminiBrowserSidecarResponse::Ack)
        ));
    }

    #[test]
    fn take_complete_jsonl_lines_handles_partial_and_multiple_chunks() {
        let mut buffer = String::new();
        buffer.push_str("{\"id\":\"one\"");
        assert!(take_complete_jsonl_line(&mut buffer).is_none());

        buffer.push_str(
            ",\"response\":{\"type\":\"ack\"}}\n\n{\"id\":\"two\",\"response\":{\"type\":\"ack\"}}\n",
        );

        assert_eq!(
            take_complete_jsonl_line(&mut buffer).as_deref(),
            Some(r#"{"id":"one","response":{"type":"ack"}}"#)
        );
        assert_eq!(
            take_complete_jsonl_line(&mut buffer).as_deref(),
            Some(r#"{"id":"two","response":{"type":"ack"}}"#)
        );
        assert!(take_complete_jsonl_line(&mut buffer).is_none());
    }

    #[tokio::test]
    async fn jsonl_transport_round_trips_a_duplex_request() {
        let (client, server) = tokio::io::duplex(1024);
        let (client_read, mut client_write) = tokio::io::split(client);
        let (server_read, mut server_write) = tokio::io::split(server);
        let server = tokio::spawn(async move {
            let mut lines = BufReader::new(server_read).lines();
            let request = lines.next_line().await.expect("read request").expect("request line");
            assert!(request.contains("gemini-sidecar-1"));
            server_write
                .write_all(b"{\"id\":\"gemini-sidecar-1\",\"response\":{\"type\":\"ack\"}}\n")
                .await
                .expect("write response");
        });

        let response = request_jsonl(
            &mut client_write,
            &mut BufReader::new(client_read),
            "gemini-sidecar-1",
            GeminiBrowserSidecarCommand::Stop,
        )
        .await
        .expect("JSONL response");

        server.await.expect("sidecar task");
        assert!(matches!(response, GeminiBrowserSidecarResponse::Ack));
    }

    #[tokio::test]
    async fn stderr_drain_consumes_sidecar_output_concurrently() {
        let (mut writer, reader) = tokio::io::duplex(1024);
        let drain = tokio::spawn(drain_sidecar_stderr(reader));

        writer.write_all(b"sidecar diagnostic\n").await.expect("write stderr");
        drop(writer);

        drain.await.expect("stderr drain completes");
    }

    #[test]
    fn resume_response_classifies_legacy_ack_for_retry() {
        let outcome = classify_resume_response(GeminiBrowserSidecarResponse::Ack)
            .expect("classify resume response");

        assert!(matches!(outcome, ResumeSidecarOutcome::LegacyAck));
    }
}
