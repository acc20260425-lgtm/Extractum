use std::process::Stdio;

use serde::Deserialize;
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::error::{AppError, AppResult};

use super::{
    GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind, GeminiBrowserRunRequest,
    GeminiBrowserRunResult, GeminiBrowserRunStatus, GeminiBrowserSidecarCommand,
    GeminiBrowserSidecarEnvelope, GeminiBrowserSidecarResponse, GeminiBrowserState,
};

#[derive(Deserialize)]
struct SidecarLine {
    id: String,
    response: GeminiBrowserSidecarResponse,
}

pub(crate) struct GeminiBrowserSidecarProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl GeminiBrowserSidecarProcess {
    async fn spawn() -> AppResult<Self> {
        let script_path = std::env::current_dir()
            .map_err(|error| AppError::internal(error.to_string()))?
            .join("sidecars")
            .join("gemini-browser")
            .join("dist")
            .join("index.js");
        let mut child = Command::new("node")
            .arg(script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                AppError::internal(format!("Failed to start Gemini browser sidecar: {error}"))
            })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::internal("Gemini browser sidecar stdin was unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::internal("Gemini browser sidecar stdout was unavailable"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    async fn request(
        &mut self,
        command: GeminiBrowserSidecarCommand,
    ) -> AppResult<GeminiBrowserSidecarResponse> {
        let id = format!("gemini-sidecar-{}", self.next_id);
        self.next_id += 1;
        let envelope = GeminiBrowserSidecarEnvelope {
            id: id.clone(),
            command,
        };
        let mut line =
            serde_json::to_string(&envelope).map_err(|error| AppError::internal(error.to_string()))?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await.map_err(|error| {
            AppError::internal(format!("Failed to write Gemini sidecar request: {error}"))
        })?;
        self.stdin.flush().await.map_err(|error| {
            AppError::internal(format!("Failed to flush Gemini sidecar request: {error}"))
        })?;

        let mut response_line = String::new();
        let bytes = self
            .stdout
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
        let response: SidecarLine = serde_json::from_str(&response_line)
            .map_err(|error| AppError::internal(format!("Invalid Gemini sidecar response: {error}")))?;
        if response.id != id {
            return Err(AppError::internal(
                "Gemini browser sidecar response id mismatch",
            ));
        }
        Ok(response.response)
    }
}

impl Drop for GeminiBrowserSidecarProcess {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

async fn request_sidecar(
    _handle: &AppHandle,
    state: &GeminiBrowserState,
    command: GeminiBrowserSidecarCommand,
) -> AppResult<GeminiBrowserSidecarResponse> {
    let mut sidecar = state.sidecar().await;
    if sidecar.is_none() {
        *sidecar = Some(GeminiBrowserSidecarProcess::spawn().await?);
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
    active_run_id: Option<String>,
    queue_depth: usize,
) -> AppResult<GeminiBrowserProviderStatus> {
    match request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::Status {
            browser_profile_dir: browser_profile_dir.clone(),
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
) -> AppResult<GeminiBrowserProviderStatus> {
    match request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::OpenBrowser {
            browser_profile_dir,
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

pub(crate) async fn send_single(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    request: GeminiBrowserRunRequest,
    browser_profile_dir: String,
    artifact_dir: String,
) -> AppResult<GeminiBrowserRunResult> {
    match request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::SendSingle {
            request,
            browser_profile_dir,
            artifact_dir,
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
    }
}
