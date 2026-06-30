use std::{path::PathBuf, process::Stdio};

use serde::Deserialize;
use tauri::AppHandle;
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::runtime::Handle;

use crate::error::{AppError, AppResult};

use super::sidecar_launch::{
    resolve_launch_mode, GeminiBrowserBuildProfile, GeminiBrowserSidecarLaunch,
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
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
    },
    Shell {
        child: Option<CommandChild>,
        rx: tauri::async_runtime::Receiver<CommandEvent>,
        stdout_buffer: String,
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
                Self::spawn_node_script(node, script).await
            }
            GeminiBrowserSidecarLaunch::Bundled { name } => Self::spawn_bundled(handle, name).await,
        }
    }

    async fn spawn_node_script(node: String, script_path: PathBuf) -> AppResult<Self> {
        let child = Command::new(node)
            .arg(script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                AppError::internal(format!("Failed to start Gemini browser sidecar: {error}"))
            })?;
        Self::from_node_child(child)
    }

    async fn spawn_bundled(handle: &AppHandle, name: String) -> AppResult<Self> {
        let command = handle.shell().sidecar(name).map_err(|error| {
            AppError::internal(format!("Gemini sidecar bundle is unavailable: {error}"))
        })?;
        let (rx, child) = command.spawn().map_err(|error| {
            AppError::internal(format!("Failed to start bundled Gemini sidecar: {error}"))
        })?;

        Ok(Self {
            transport: GeminiBrowserSidecarTransport::Shell {
                child: Some(child),
                rx,
                stdout_buffer: String::new(),
            },
            next_id: 1,
        })
    }

    fn from_node_child(mut child: Child) -> AppResult<Self> {
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::internal("Gemini browser sidecar stdin was unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::internal("Gemini browser sidecar stdout was unavailable"))?;
        Ok(Self {
            transport: GeminiBrowserSidecarTransport::Node {
                child: Some(child),
                stdin,
                stdout: BufReader::new(stdout),
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
                request_node(stdin, stdout, &id, command).await
            }
            GeminiBrowserSidecarTransport::Shell {
                child,
                rx,
                stdout_buffer,
            } => {
                let child = child.as_mut().ok_or_else(|| {
                    AppError::internal("Bundled Gemini browser sidecar was already stopped")
                })?;
                request_shell(child, rx, stdout_buffer, &id, command).await
            }
        }
    }
}

impl Drop for GeminiBrowserSidecarProcess {
    fn drop(&mut self) {
        match &mut self.transport {
            GeminiBrowserSidecarTransport::Node { child, .. } => {
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
            GeminiBrowserSidecarTransport::Shell { child, .. } => {
                if let Some(child) = child.take() {
                    let _ = child.kill();
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

async fn request_shell(
    child: &mut CommandChild,
    rx: &mut tauri::async_runtime::Receiver<CommandEvent>,
    stdout_buffer: &mut String,
    id: &str,
    command: GeminiBrowserSidecarCommand,
) -> AppResult<GeminiBrowserSidecarResponse> {
    let envelope = GeminiBrowserSidecarEnvelope {
        id: id.to_string(),
        command,
    };
    let mut line =
        serde_json::to_string(&envelope).map_err(|error| AppError::internal(error.to_string()))?;
    line.push('\n');
    child.write(line.as_bytes()).map_err(|error| {
        AppError::internal(format!(
            "Failed to write bundled Gemini sidecar request: {error}"
        ))
    })?;

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                stdout_buffer.push_str(&String::from_utf8_lossy(&bytes));
                while let Some(line) = take_complete_jsonl_line(stdout_buffer) {
                    if let Some(response) = decode_sidecar_line_for_request(id, &line)? {
                        return Ok(response);
                    }
                }
            }
            CommandEvent::Stderr(_) => {}
            CommandEvent::Error(message) => {
                return Err(AppError::internal(format!(
                    "Bundled Gemini browser sidecar errored: {message}"
                )));
            }
            CommandEvent::Terminated(payload) => {
                return Err(AppError::internal(format!(
                    "Bundled Gemini browser sidecar exited without a response: code={:?}",
                    payload.code
                )));
            }
            _ => {}
        }
    }

    Err(AppError::internal(
        "Bundled Gemini browser sidecar exited without a response",
    ))
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
    match process.request(command).await {
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

pub(crate) async fn stop(handle: &AppHandle, state: &GeminiBrowserState) -> AppResult<()> {
    let _ = request_sidecar(handle, state, GeminiBrowserSidecarCommand::Stop).await;
    *state.sidecar().await = None;
    Ok(())
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
    async fn shell_transport_waits_for_complete_jsonl_line_across_stdout_events() {
        let mut buffer = String::new();
        buffer.push_str("{\"id\":\"expected\"");
        assert!(take_complete_jsonl_line(&mut buffer).is_none());

        buffer.push_str(",\"response\":{\"type\":\"ack\"}}\n");
        let line = take_complete_jsonl_line(&mut buffer).expect("complete line");

        let response = decode_sidecar_line("expected", &line).expect("decode response");
        assert!(matches!(response, GeminiBrowserSidecarResponse::Ack));
    }

    #[test]
    fn resume_response_classifies_legacy_ack_for_retry() {
        let outcome = classify_resume_response(GeminiBrowserSidecarResponse::Ack)
            .expect("classify resume response");

        assert!(matches!(outcome, ResumeSidecarOutcome::LegacyAck));
    }
}
