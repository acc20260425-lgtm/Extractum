# Gemini Browser Provider MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a production MVP of the Gemini Browser Provider that can open a persistent Gemini browser session, run one queued prompt through the selected browser adapter, persist sanitized run logs, and expose the provider in Settings.

**Architecture:** Keep the browser-backed Gemini provider separate from API-backed LLM profiles. Rust/Tauri owns app commands, queue state, app-data paths, sidecar lifecycle, run logs, cancellation, and UI events; the TypeScript sidecar owns Playwright, the DOM contract, telemetry capture, and the selected `resilient-scoring` adapter. The research harness remains a regression suite and source reference, not a runtime dependency.

**Tech Stack:** Rust/Tauri 2, Tokio, serde/serde_json, Svelte 5, Vitest, TypeScript, Playwright, app-data file storage.

---

## Execution Rules

- Start from a clean feature branch before Task 1: `git switch -c gemini-browser-provider-mvp`.
- After each task, update this plan's task checkboxes for the completed task and commit only the files from that task.
- Do not stage unrelated files from the current dirty worktree.
- Keep Python out of the production runtime and production tests for this feature.
- Keep `research/gemini_browser_adapter` as read-only reference unless a task explicitly changes research docs.

## File Structure

- Create `src-tauri/src/gemini_browser/mod.rs`: public command exports and module wiring.
- Create `src-tauri/src/gemini_browser/types.rs`: Rust IPC DTOs, provider statuses, run statuses, and sidecar JSON protocol.
- Create `src-tauri/src/gemini_browser/paths.rs`: app-data paths for browser profile, artifacts, and run log.
- Create `src-tauri/src/gemini_browser/run_log.rs`: file-backed run log append/list/read helpers.
- Create `src-tauri/src/gemini_browser/state.rs`: global queue, active run, cancellation token, and sidecar runtime state.
- Create `src-tauri/src/gemini_browser/commands.rs`: Tauri commands named in the product spec.
- Create `src-tauri/src/gemini_browser/sidecar.rs`: sidecar process/protocol client with a mockable trait for tests.
- Modify `src-tauri/src/lib.rs`: register `GeminiBrowserState` and six Gemini Browser commands.
- Create `src/lib/types/gemini-browser.ts`: frontend DTOs matching Rust serde names.
- Create `src/lib/api/gemini-browser.ts`: Tauri command wrappers and event listener.
- Create `src/lib/api/gemini-browser.test.ts`: wrapper contract tests.
- Create `src/lib/components/settings/gemini-browser-provider-panel.svelte`: Settings panel for status, browser open, run test, resume, stop, and latest runs.
- Modify `src/lib/components/settings/projects-settings.svelte`: add a `Browser Providers` tab that hosts the new panel.
- Create `sidecars/gemini-browser/package.json`: sidecar-local scripts.
- Create `sidecars/gemini-browser/tsconfig.json`: strict sidecar TypeScript config.
- Create `sidecars/gemini-browser/src/protocol.ts`: sidecar stdin/stdout protocol types.
- Create `sidecars/gemini-browser/src/dom-contract.ts`: production copy of the selected resilient DOM contract surface.
- Create `sidecars/gemini-browser/src/adapter.ts`: Playwright adapter facade for status/open/send/resume/stop.
- Create `sidecars/gemini-browser/src/index.ts`: JSON-line sidecar server.
- Create `sidecars/gemini-browser/src/*.test.ts`: protocol, redaction, and adapter unit tests.
- Modify `package.json`: add sidecar typecheck/test scripts and extend feature verification scripts.
- Modify `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`: append MVP execution notes when the production boundary is in place.

---

## Task 1: Branch And Baseline Verification

**Files:**
- Read: `research/gemini_browser_adapter/DECISION.md`
- Read: `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`
- Read: `package.json`

- [x] **Step 1: Create the feature branch**

Run:

```powershell
git switch -c gemini-browser-provider-mvp
```

Expected: `Switched to a new branch 'gemini-browser-provider-mvp'`.

- [x] **Step 2: Confirm unrelated worktree changes before editing**

Run:

```powershell
git status --short
```

Expected: any existing `M scripts/analysis-smoke.mjs`, `M src-tauri/src/analysis/...`, or `M src/lib/analysis-...` files are treated as user changes and are not staged by this plan.

- [x] **Step 3: Run research regression guard**

Run:

```powershell
npm.cmd run test:gemini-browser-adapter
```

Expected: typecheck, unit tests, e2e matrix, and report complete with exit code `0`.

- [x] **Step 4: Run current frontend check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check` reports `0 errors and 0 warnings`.

- [x] **Step 5: Commit only the plan checkbox update**

Run:

```powershell
git add docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "docs: start Gemini browser provider MVP execution"
```

Expected: one docs-only commit. Do not stage unrelated dirty files.

---

## Task 2: Rust Gemini Browser Contracts

**Files:**
- Create: `src-tauri/src/gemini_browser/mod.rs`
- Create: `src-tauri/src/gemini_browser/types.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/gemini_browser/types.rs`

- [x] **Step 1: Add the module shell**

Create `src-tauri/src/gemini_browser/mod.rs`:

```rust
mod types;

pub use types::{
    GeminiBrowserArtifactRefs, GeminiBrowserManualAction, GeminiBrowserProviderStatus,
    GeminiBrowserProviderStatusKind, GeminiBrowserRun, GeminiBrowserRunEvent,
    GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
    GeminiBrowserSidecarResponse,
};
```

- [x] **Step 2: Write the failing contract serialization tests**

Create `src-tauri/src/gemini_browser/types.rs` with the type declarations and tests in this step. The test module must be present before command wiring:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserProviderStatusKind {
    NotStarted,
    Ready,
    NeedsLogin,
    NeedsManualAction,
    Running,
    Stopped,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserManualAction {
    Login,
    AccountPicker,
    Consent,
    Captcha,
    UnknownModal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserProviderStatus {
    pub status: GeminiBrowserProviderStatusKind,
    pub manual_action: Option<GeminiBrowserManualAction>,
    pub active_run_id: Option<String>,
    pub queue_depth: usize,
    pub browser_profile_dir: String,
    pub latest_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunRequest {
    pub run_id: String,
    pub prompt: String,
    pub source: String,
    pub artifact_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserRunStatus {
    Queued,
    Running,
    Ok,
    Ready,
    NeedsLogin,
    NeedsManualAction,
    Blocked,
    Timeout,
    BrowserCrashed,
    Failed,
    Cancelled,
}

impl GeminiBrowserRunStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Ok | Self::Ready)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Ok
                | Self::Ready
                | Self::NeedsLogin
                | Self::NeedsManualAction
                | Self::Blocked
                | Self::Timeout
                | Self::BrowserCrashed
                | Self::Failed
                | Self::Cancelled
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserArtifactRefs {
    pub run_dir: Option<String>,
    pub html: Option<String>,
    pub screenshot: Option<String>,
    pub telemetry: Option<String>,
    pub artifact_write_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunResult {
    pub run_id: String,
    pub status: GeminiBrowserRunStatus,
    pub text: Option<String>,
    pub message: Option<String>,
    pub manual_action: Option<GeminiBrowserManualAction>,
    pub artifacts: GeminiBrowserArtifactRefs,
    pub elapsed_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRun {
    pub run_id: String,
    pub source: String,
    pub status: GeminiBrowserRunStatus,
    pub prompt_preview: String,
    pub created_at: String,
    pub updated_at: String,
    pub result: Option<GeminiBrowserRunResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunLogSummary {
    pub runs: Vec<GeminiBrowserRun>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunEvent {
    pub run_id: String,
    pub status: GeminiBrowserRunStatus,
    pub message: Option<String>,
    pub queue_position: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeminiBrowserSidecarCommand {
    Status {
        browser_profile_dir: String,
    },
    OpenBrowser {
        browser_profile_dir: String,
    },
    SendSingle {
        request: GeminiBrowserRunRequest,
        browser_profile_dir: String,
        artifact_dir: String,
    },
    Resume {
        run_id: Option<String>,
    },
    Stop,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserSidecarEnvelope {
    pub id: String,
    pub command: GeminiBrowserSidecarCommand,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeminiBrowserSidecarResponse {
    Status { status: GeminiBrowserProviderStatus },
    RunResult { result: GeminiBrowserRunResult },
    Ack,
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_statuses_include_ready_and_ok() {
        assert!(GeminiBrowserRunStatus::Ok.is_success());
        assert!(GeminiBrowserRunStatus::Ready.is_success());
        assert!(!GeminiBrowserRunStatus::NeedsLogin.is_success());
    }

    #[test]
    fn sidecar_command_serializes_with_snake_case_tag() {
        let command = GeminiBrowserSidecarEnvelope {
            id: "cmd-1".to_string(),
            command: GeminiBrowserSidecarCommand::OpenBrowser {
                browser_profile_dir: "C:/Extractum/gemini-browser/profile".to_string(),
            },
        };

        let json = serde_json::to_value(command).expect("serialize command");
        assert_eq!(json["command"]["type"], "open_browser");
        assert_eq!(
            json["command"]["browser_profile_dir"],
            "C:/Extractum/gemini-browser/profile"
        );
    }
}
```

- [x] **Step 3: Run the focused Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser::types
```

Expected: the new tests compile and pass.

- [x] **Step 4: Commit**

Run:

```powershell
git add src-tauri/src/gemini_browser/mod.rs src-tauri/src/gemini_browser/types.rs src-tauri/src/lib.rs docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: add Gemini browser provider contracts"
```

Expected: commit includes the new Rust module and updated plan checkbox only.

---

## Task 3: App-Data Paths And File-Backed Run Log

**Files:**
- Create: `src-tauri/src/gemini_browser/paths.rs`
- Create: `src-tauri/src/gemini_browser/run_log.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Test: `src-tauri/src/gemini_browser/run_log.rs`

- [x] **Step 1: Add path helpers**

Create `src-tauri/src/gemini_browser/paths.rs`:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub(crate) const GEMINI_BROWSER_DIR: &str = "gemini-browser";
pub(crate) const PROFILE_DIR: &str = "profile";
pub(crate) const RUNS_DIR: &str = "runs";

pub(crate) fn base_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    Ok(handle
        .path()
        .app_data_dir()
        .map_err(|error| AppError::internal(error.to_string()))?
        .join(GEMINI_BROWSER_DIR))
}

pub(crate) fn profile_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    let path = base_dir(handle)?.join(PROFILE_DIR);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn runs_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    let path = base_dir(handle)?.join(RUNS_DIR);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn run_dir(handle: &AppHandle, run_id: &str) -> AppResult<PathBuf> {
    let path = runs_dir(handle)?.join(safe_run_id(run_id)?);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn safe_run_id(run_id: &str) -> AppResult<String> {
    let candidate = run_id.trim();
    if candidate.is_empty() {
        return Err(AppError::validation("run_id cannot be empty"));
    }
    if candidate
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        Ok(candidate.to_string())
    } else {
        Err(AppError::validation(
            "run_id can only contain ASCII letters, numbers, dashes, and underscores",
        ))
    }
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
```

- [x] **Step 2: Add the file-backed log implementation**

Create `src-tauri/src/gemini_browser/run_log.rs`:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use crate::error::{AppError, AppResult};

use super::paths::safe_run_id;
use super::{GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunResult, GeminiBrowserRunStatus};

const RUN_FILE: &str = "result.json";
const PROMPT_PREVIEW_CHARS: usize = 120;

fn now_string() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn run_file_path(runs_dir: &Path, run_id: &str) -> AppResult<PathBuf> {
    Ok(runs_dir.join(safe_run_id(run_id)?).join(RUN_FILE))
}

fn prompt_preview(prompt: &str) -> String {
    let mut chars = prompt.trim().chars();
    let preview = chars.by_ref().take(PROMPT_PREVIEW_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
}

pub(crate) fn create_queued_run(
    runs_dir: &Path,
    run_id: &str,
    source: &str,
    prompt: &str,
) -> AppResult<GeminiBrowserRun> {
    let run_dir = runs_dir.join(safe_run_id(run_id)?);
    fs::create_dir_all(&run_dir).map_err(|error| AppError::internal(error.to_string()))?;
    let now = now_string();
    let run = GeminiBrowserRun {
        run_id: run_id.to_string(),
        source: source.to_string(),
        status: GeminiBrowserRunStatus::Queued,
        prompt_preview: prompt_preview(prompt),
        created_at: now.clone(),
        updated_at: now,
        result: None,
    };
    write_run(&run_dir.join(RUN_FILE), &run)?;
    Ok(run)
}

pub(crate) fn mark_running(runs_dir: &Path, run_id: &str) -> AppResult<GeminiBrowserRun> {
    let mut run = read_run_file(&run_file_path(runs_dir, run_id)?)?;
    run.status = GeminiBrowserRunStatus::Running;
    run.updated_at = now_string();
    write_run(&run_file_path(runs_dir, run_id)?, &run)?;
    Ok(run)
}

pub(crate) fn finish_run(
    runs_dir: &Path,
    run_id: &str,
    result: GeminiBrowserRunResult,
) -> AppResult<GeminiBrowserRun> {
    let mut run = read_run_file(&run_file_path(runs_dir, run_id)?)?;
    run.status = result.status.clone();
    run.updated_at = now_string();
    run.result = Some(result);
    write_run(&run_file_path(runs_dir, run_id)?, &run)?;
    Ok(run)
}

pub(crate) fn list_runs(runs_dir: &Path, limit: usize) -> AppResult<GeminiBrowserRunLogSummary> {
    if !runs_dir.exists() {
        return Ok(GeminiBrowserRunLogSummary { runs: Vec::new() });
    }
    let mut runs = Vec::new();
    for entry in fs::read_dir(runs_dir).map_err(|error| AppError::internal(error.to_string()))? {
        let path = entry
            .map_err(|error| AppError::internal(error.to_string()))?
            .path()
            .join(RUN_FILE);
        if path.exists() {
            runs.push(read_run_file(&path)?);
        }
    }
    runs.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    runs.truncate(limit);
    Ok(GeminiBrowserRunLogSummary { runs })
}

fn read_run_file(path: &Path) -> AppResult<GeminiBrowserRun> {
    let content = fs::read_to_string(path).map_err(|error| AppError::internal(error.to_string()))?;
    serde_json::from_str(&content).map_err(|error| AppError::internal(error.to_string()))
}

fn write_run(path: &Path, run: &GeminiBrowserRun) -> AppResult<()> {
    let content =
        serde_json::to_string_pretty(run).map_err(|error| AppError::internal(error.to_string()))?;
    fs::write(path, content).map_err(|error| AppError::internal(error.to_string()))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::gemini_browser::{GeminiBrowserArtifactRefs, GeminiBrowserRunStatus};

    #[test]
    fn run_log_persists_queued_running_and_terminal_result() {
        let temp = tempdir().expect("tempdir");
        let runs_dir = temp.path();

        let queued = create_queued_run(runs_dir, "run-1", "settings_test", "hello Gemini")
            .expect("create queued run");
        assert_eq!(queued.status, GeminiBrowserRunStatus::Queued);

        let running = mark_running(runs_dir, "run-1").expect("mark running");
        assert_eq!(running.status, GeminiBrowserRunStatus::Running);

        let result = GeminiBrowserRunResult {
            run_id: "run-1".to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 25,
        };
        let finished = finish_run(runs_dir, "run-1", result).expect("finish run");
        assert_eq!(finished.status, GeminiBrowserRunStatus::Ok);
        assert_eq!(finished.result.expect("result").text, Some("answer".to_string()));

        let listed = list_runs(runs_dir, 10).expect("list runs");
        assert_eq!(listed.runs.len(), 1);
        assert_eq!(listed.runs[0].run_id, "run-1");
    }
}
```

- [x] **Step 3: Wire the new modules**

Modify `src-tauri/src/gemini_browser/mod.rs`:

```rust
mod paths;
mod run_log;
mod types;

pub(crate) use paths::{path_string, profile_dir, run_dir, runs_dir};
pub(crate) use run_log::{create_queued_run, finish_run, list_runs, mark_running};
pub use types::{
    GeminiBrowserArtifactRefs, GeminiBrowserManualAction, GeminiBrowserProviderStatus,
    GeminiBrowserProviderStatusKind, GeminiBrowserRun, GeminiBrowserRunEvent,
    GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
    GeminiBrowserSidecarResponse,
};
```

- [x] **Step 4: Run focused Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser
```

Expected: `gemini_browser::types` and `gemini_browser::run_log` tests pass.

- [x] **Step 5: Commit**

Run:

```powershell
git add src-tauri/src/gemini_browser docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: persist Gemini browser provider run logs"
```

Expected: commit includes path/run-log files and updated plan checkbox only.

---

## Task 4: Rust State, Commands, And Events

**Files:**
- Create: `src-tauri/src/gemini_browser/state.rs`
- Create: `src-tauri/src/gemini_browser/sidecar.rs`
- Create: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/gemini_browser/state.rs`

- [x] **Step 1: Add queue state**

Create `src-tauri/src/gemini_browser/state.rs`:

```rust
use std::collections::VecDeque;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::GeminiBrowserRunRequest;

#[derive(Default)]
pub struct GeminiBrowserState {
    queue: Mutex<VecDeque<GeminiBrowserRunRequest>>,
    active_run_id: Mutex<Option<String>>,
    cancellation: Mutex<Option<CancellationToken>>,
}

impl GeminiBrowserState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn enqueue(&self, request: GeminiBrowserRunRequest) -> usize {
        let mut queue = self.queue.lock().await;
        queue.push_back(request);
        queue.len()
    }

    pub async fn pop_next(&self) -> Option<GeminiBrowserRunRequest> {
        self.queue.lock().await.pop_front()
    }

    pub async fn queue_depth(&self) -> usize {
        self.queue.lock().await.len()
    }

    pub async fn active_run_id(&self) -> Option<String> {
        self.active_run_id.lock().await.clone()
    }

    pub async fn start_run(&self, run_id: String) -> CancellationToken {
        *self.active_run_id.lock().await = Some(run_id);
        let token = CancellationToken::new();
        *self.cancellation.lock().await = Some(token.clone());
        token
    }

    pub async fn finish_run(&self, run_id: &str) {
        let mut active = self.active_run_id.lock().await;
        if active.as_deref() == Some(run_id) {
            *active = None;
            *self.cancellation.lock().await = None;
        }
    }

    pub async fn request_stop(&self) -> bool {
        if let Some(token) = self.cancellation.lock().await.as_ref() {
            token.cancel();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn queue_tracks_depth_and_active_run() {
        let state = GeminiBrowserState::new();
        let position = state
            .enqueue(GeminiBrowserRunRequest {
                run_id: "run-1".to_string(),
                prompt: "hello".to_string(),
                source: "test".to_string(),
                artifact_mode: "reduced".to_string(),
            })
            .await;
        assert_eq!(position, 1);
        assert_eq!(state.queue_depth().await, 1);

        let next = state.pop_next().await.expect("queued request");
        let token = state.start_run(next.run_id.clone()).await;
        assert!(!token.is_cancelled());
        assert_eq!(state.active_run_id().await, Some("run-1".to_string()));
        assert!(state.request_stop().await);
        assert!(token.is_cancelled());
        state.finish_run("run-1").await;
        assert_eq!(state.active_run_id().await, None);
    }
}
```

- [x] **Step 2: Add a mockable sidecar facade**

Create `src-tauri/src/gemini_browser/sidecar.rs`:

```rust
use tauri::AppHandle;

use crate::error::{AppError, AppResult};

use super::{
    GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind, GeminiBrowserRunRequest,
    GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

pub(crate) async fn status(
    browser_profile_dir: String,
    active_run_id: Option<String>,
    queue_depth: usize,
) -> AppResult<GeminiBrowserProviderStatus> {
    Ok(GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::NotStarted,
        manual_action: None,
        active_run_id,
        queue_depth,
        browser_profile_dir,
        latest_message: Some("Gemini browser sidecar is not running yet.".to_string()),
    })
}

pub(crate) async fn open_browser(_handle: &AppHandle, browser_profile_dir: String) -> AppResult<GeminiBrowserProviderStatus> {
    Ok(GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::NotStarted,
        manual_action: None,
        active_run_id: None,
        queue_depth: 0,
        browser_profile_dir,
        latest_message: Some("Browser launch will be enabled when the sidecar is wired.".to_string()),
    })
}

pub(crate) async fn send_single_stub(request: GeminiBrowserRunRequest) -> AppResult<GeminiBrowserRunResult> {
    if request.prompt.trim().is_empty() {
        return Err(AppError::validation("prompt cannot be empty"));
    }
    Ok(GeminiBrowserRunResult {
        run_id: request.run_id,
        status: GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some("Gemini browser sidecar is not wired yet.".to_string()),
        manual_action: None,
        artifacts: Default::default(),
        elapsed_ms: 0,
    })
}
```

- [x] **Step 3: Add Tauri commands**

Create `src-tauri/src/gemini_browser/commands.rs`:

```rust
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};

use super::{
    create_queued_run, finish_run, list_runs, mark_running, path_string, profile_dir, runs_dir,
    sidecar, GeminiBrowserRunEvent, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest,
    GeminiBrowserRunResult, GeminiBrowserRunStatus, GeminiBrowserState,
};

pub const GEMINI_BROWSER_RUN_EVENT: &str = "gemini-browser://run";

fn emit_run_event(handle: &AppHandle, event: GeminiBrowserRunEvent) {
    let _ = handle.emit(GEMINI_BROWSER_RUN_EVENT, event);
}

#[tauri::command]
pub async fn gemini_bridge_status(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<super::GeminiBrowserProviderStatus> {
    sidecar::status(
        path_string(&profile_dir(&handle)?),
        state.active_run_id().await,
        state.queue_depth().await,
    )
    .await
}

#[tauri::command]
pub async fn gemini_bridge_open_browser(
    handle: AppHandle,
) -> AppResult<super::GeminiBrowserProviderStatus> {
    sidecar::open_browser(&handle, path_string(&profile_dir(&handle)?)).await
}

#[tauri::command]
pub async fn gemini_bridge_send_single(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
    run_id: String,
    prompt: String,
    source: Option<String>,
    artifact_mode: Option<String>,
) -> AppResult<GeminiBrowserRunResult> {
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::validation("prompt cannot be empty"));
    }
    let request = GeminiBrowserRunRequest {
        run_id,
        prompt,
        source: source.unwrap_or_else(|| "settings_test".to_string()),
        artifact_mode: artifact_mode.unwrap_or_else(|| "reduced".to_string()),
    };

    let runs_root = runs_dir(&handle)?;
    create_queued_run(&runs_root, &request.run_id, &request.source, &request.prompt)?;
    let queue_position = state.enqueue(request.clone()).await;
    emit_run_event(
        &handle,
        GeminiBrowserRunEvent {
            run_id: request.run_id.clone(),
            status: GeminiBrowserRunStatus::Queued,
            message: Some("Queued".to_string()),
            queue_position: Some(queue_position),
        },
    );

    let next = state
        .pop_next()
        .await
        .ok_or_else(|| AppError::internal("Gemini browser queue unexpectedly empty"))?;
    let _token = state.start_run(next.run_id.clone()).await;
    mark_running(&runs_root, &next.run_id)?;
    emit_run_event(
        &handle,
        GeminiBrowserRunEvent {
            run_id: next.run_id.clone(),
            status: GeminiBrowserRunStatus::Running,
            message: Some("Running".to_string()),
            queue_position: None,
        },
    );

    let result = sidecar::send_single_stub(next.clone()).await?;
    finish_run(&runs_root, &next.run_id, result.clone())?;
    state.finish_run(&next.run_id).await;
    emit_run_event(
        &handle,
        GeminiBrowserRunEvent {
            run_id: next.run_id,
            status: result.status.clone(),
            message: result.message.clone(),
            queue_position: None,
        },
    );
    Ok(result)
}

#[tauri::command]
pub async fn gemini_bridge_resume() -> AppResult<()> {
    Ok(())
}

#[tauri::command]
pub async fn gemini_bridge_stop(state: State<'_, GeminiBrowserState>) -> AppResult<()> {
    state.request_stop().await;
    Ok(())
}

#[tauri::command]
pub async fn gemini_bridge_list_runs(
    handle: AppHandle,
    limit: Option<usize>,
) -> AppResult<GeminiBrowserRunLogSummary> {
    list_runs(&runs_dir(&handle)?, limit.unwrap_or(20))
}
```

- [x] **Step 4: Export and register commands**

Modify `src-tauri/src/gemini_browser/mod.rs`:

```rust
mod commands;
mod paths;
mod run_log;
mod sidecar;
mod state;
mod types;

pub use commands::{
    gemini_bridge_list_runs, gemini_bridge_open_browser, gemini_bridge_resume,
    gemini_bridge_send_single, gemini_bridge_status, gemini_bridge_stop,
    GEMINI_BROWSER_RUN_EVENT,
};
pub(crate) use paths::{path_string, profile_dir, runs_dir};
pub(crate) use run_log::{create_queued_run, finish_run, list_runs, mark_running};
pub use state::GeminiBrowserState;
pub use types::{
    GeminiBrowserArtifactRefs, GeminiBrowserManualAction, GeminiBrowserProviderStatus,
    GeminiBrowserProviderStatusKind, GeminiBrowserRun, GeminiBrowserRunEvent,
    GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
    GeminiBrowserSidecarResponse,
};
```

Modify `src-tauri/src/lib.rs`:

```rust
mod gemini_browser;
use gemini_browser::{
    gemini_bridge_list_runs, gemini_bridge_open_browser, gemini_bridge_resume,
    gemini_bridge_send_single, gemini_bridge_status, gemini_bridge_stop, GeminiBrowserState,
};
```

Add `.manage(GeminiBrowserState::new())` after `.manage(LlmSchedulerState::new())`.

Add these commands to `tauri::generate_handler![...]` near the LLM commands:

```rust
gemini_bridge_status,
gemini_bridge_open_browser,
gemini_bridge_send_single,
gemini_bridge_resume,
gemini_bridge_stop,
gemini_bridge_list_runs,
```

- [x] **Step 5: Run Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser
```

Expected: all Gemini browser module tests pass.

- [x] **Step 6: Commit**

Run:

```powershell
git add src-tauri/src/gemini_browser src-tauri/src/lib.rs docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: add Gemini browser provider commands"
```

Expected: commit includes Rust command wiring and updated plan checkbox only.

---

## Task 5: Frontend API Contract

**Files:**
- Create: `src/lib/types/gemini-browser.ts`
- Create: `src/lib/api/gemini-browser.ts`
- Create: `src/lib/api/gemini-browser.test.ts`

- [x] **Step 1: Add frontend DTOs**

Create `src/lib/types/gemini-browser.ts`:

```ts
export type GeminiBrowserProviderStatusKind =
  | "not_started"
  | "ready"
  | "needs_login"
  | "needs_manual_action"
  | "running"
  | "stopped"
  | "failed";

export type GeminiBrowserManualAction =
  | "login"
  | "account_picker"
  | "consent"
  | "captcha"
  | "unknown_modal";

export interface GeminiBrowserProviderStatus {
  status: GeminiBrowserProviderStatusKind;
  manual_action: GeminiBrowserManualAction | null;
  active_run_id: string | null;
  queue_depth: number;
  browser_profile_dir: string;
  latest_message: string | null;
}

export type GeminiBrowserRunStatus =
  | "queued"
  | "running"
  | "ok"
  | "ready"
  | "needs_login"
  | "needs_manual_action"
  | "blocked"
  | "timeout"
  | "browser_crashed"
  | "failed"
  | "cancelled";

export interface GeminiBrowserArtifactRefs {
  run_dir: string | null;
  html: string | null;
  screenshot: string | null;
  telemetry: string | null;
  artifact_write_error: string | null;
}

export interface GeminiBrowserRunResult {
  run_id: string;
  status: GeminiBrowserRunStatus;
  text: string | null;
  message: string | null;
  manual_action: GeminiBrowserManualAction | null;
  artifacts: GeminiBrowserArtifactRefs;
  elapsed_ms: number;
}

export interface GeminiBrowserRun {
  run_id: string;
  source: string;
  status: GeminiBrowserRunStatus;
  prompt_preview: string;
  created_at: string;
  updated_at: string;
  result: GeminiBrowserRunResult | null;
}

export interface GeminiBrowserRunLogSummary {
  runs: GeminiBrowserRun[];
}

export interface GeminiBrowserRunEvent {
  run_id: string;
  status: GeminiBrowserRunStatus;
  message: string | null;
  queue_position: number | null;
}

export interface GeminiBridgeSendSingleInput {
  runId: string;
  prompt: string;
  source?: string | null;
  artifactMode?: "reduced" | "full" | null;
}
```

- [x] **Step 2: Add Tauri command wrappers**

Create `src/lib/api/gemini-browser.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  GeminiBridgeSendSingleInput,
  GeminiBrowserProviderStatus,
  GeminiBrowserRunEvent,
  GeminiBrowserRunLogSummary,
  GeminiBrowserRunResult,
} from "$lib/types/gemini-browser";

export const GEMINI_BROWSER_RUN_EVENT = "gemini-browser://run";

export function geminiBridgeStatus() {
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_status");
}

export function geminiBridgeOpenBrowser() {
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_open_browser");
}

export function geminiBridgeSendSingle(input: GeminiBridgeSendSingleInput) {
  return invoke<GeminiBrowserRunResult>("gemini_bridge_send_single", { ...input });
}

export function geminiBridgeResume() {
  return invoke<void>("gemini_bridge_resume");
}

export function geminiBridgeStop() {
  return invoke<void>("gemini_bridge_stop");
}

export function geminiBridgeListRuns(limit = 20) {
  return invoke<GeminiBrowserRunLogSummary>("gemini_bridge_list_runs", { limit });
}

export function listenToGeminiBrowserRuns(
  handler: (event: Event<GeminiBrowserRunEvent>) => void,
): Promise<UnlistenFn> {
  return listen<GeminiBrowserRunEvent>(GEMINI_BROWSER_RUN_EVENT, handler);
}
```

- [x] **Step 3: Add wrapper tests**

Create `src/lib/api/gemini-browser.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  GEMINI_BROWSER_RUN_EVENT,
  geminiBridgeListRuns,
  geminiBridgeOpenBrowser,
  geminiBridgeResume,
  geminiBridgeSendSingle,
  geminiBridgeStatus,
  geminiBridgeStop,
  listenToGeminiBrowserRuns,
} from "./gemini-browser";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("gemini browser api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("wraps provider commands with stable command names", async () => {
    await geminiBridgeStatus();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_status");

    await geminiBridgeOpenBrowser();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_open_browser");

    await geminiBridgeResume();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_resume");

    await geminiBridgeStop();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_stop");
  });

  it("sends single prompt with camelCase frontend keys", async () => {
    await geminiBridgeSendSingle({
      runId: "run-1",
      prompt: "hello",
      source: "settings_test",
      artifactMode: "reduced",
    });

    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_send_single", {
      runId: "run-1",
      prompt: "hello",
      source: "settings_test",
      artifactMode: "reduced",
    });
  });

  it("lists runs and subscribes to run events", async () => {
    await geminiBridgeListRuns(5);
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_list_runs", { limit: 5 });

    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);
    await expect(listenToGeminiBrowserRuns(handler)).resolves.toBe(unlisten);
    expect(GEMINI_BROWSER_RUN_EVENT).toBe("gemini-browser://run");
    expect(listenMock).toHaveBeenCalledWith(GEMINI_BROWSER_RUN_EVENT, handler);
  });
});
```

- [x] **Step 4: Run frontend API tests**

Run:

```powershell
npm.cmd run test -- src/lib/api/gemini-browser.test.ts
```

Expected: the new wrapper tests pass.

- [x] **Step 5: Commit**

Run:

```powershell
git add src/lib/types/gemini-browser.ts src/lib/api/gemini-browser.ts src/lib/api/gemini-browser.test.ts docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: add Gemini browser frontend API"
```

Expected: commit includes frontend type/API files and updated plan checkbox only.

---

## Task 6: Settings Browser Providers Panel

**Files:**
- Create: `src/lib/gemini-browser-provider-panel-contract.ts`
- Create: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
- Modify: `src/lib/components/settings/projects-settings.svelte`
- Test: `src/lib/gemini-browser-provider-panel.test.ts`

- [x] **Step 1: Add a panel behavior test**

Create `src/lib/gemini-browser-provider-panel.test.ts` against `src/lib/gemini-browser-provider-panel-contract.ts`:

```ts
import { describe, expect, it } from "vitest";
import { statusLabel } from "./gemini-browser-provider-panel-contract";

describe("gemini browser provider panel copy contract", () => {
  it("maps provider statuses to compact operator labels", () => {
    expect(statusLabel("ready", null)).toBe("Ready");
    expect(statusLabel("needs_login", "login")).toBe("Login required");
    expect(statusLabel("needs_manual_action", "account_picker")).toBe("Choose account");
    expect(statusLabel("running", null)).toBe("Running");
    expect(statusLabel("failed", null)).toBe("Failed");
    expect(statusLabel("not_started", null)).toBe("Not started");
  });
});
```

- [x] **Step 2: Add the Svelte panel**

Create `src/lib/components/settings/gemini-browser-provider-panel.svelte`:

```svelte
<script lang="ts">
  import { ExternalLink, Play, RefreshCw, Send, Square } from "@lucide/svelte";
  import { onMount } from "svelte";
  import {
    geminiBridgeListRuns,
    geminiBridgeOpenBrowser,
    geminiBridgeResume,
    geminiBridgeSendSingle,
    geminiBridgeStatus,
    geminiBridgeStop,
    listenToGeminiBrowserRuns,
  } from "$lib/api/gemini-browser";
  import { formatAppError } from "$lib/app-error";
  import type {
    GeminiBrowserProviderStatus,
    GeminiBrowserRun,
    GeminiBrowserRunResult,
  } from "$lib/types/gemini-browser";

  let status = $state<GeminiBrowserProviderStatus | null>(null);
  let runs = $state<GeminiBrowserRun[]>([]);
  let prompt = $state("Reply with one short sentence confirming the browser provider is connected.");
  let busy = $state(false);
  let message = $state("");
  let result = $state<GeminiBrowserRunResult | null>(null);

  function newRunId() {
    return `gemini-browser-${Date.now()}-${Math.random().toString(16).slice(2)}`;
  }

  function statusLabel() {
    if (!status) return "Not started";
    if (status.status === "ready") return "Ready";
    if (status.status === "needs_login") return "Login required";
    if (status.status === "needs_manual_action" && status.manual_action === "account_picker") return "Choose account";
    if (status.status === "running") return "Running";
    if (status.status === "failed") return "Failed";
    return "Not started";
  }

  async function refresh() {
    try {
      const [nextStatus, log] = await Promise.all([geminiBridgeStatus(), geminiBridgeListRuns(8)]);
      status = nextStatus;
      runs = log.runs;
      message = nextStatus.latest_message ?? "";
    } catch (error) {
      message = formatAppError("loading Gemini browser provider", error);
    }
  }

  async function openBrowser() {
    busy = true;
    try {
      status = await geminiBridgeOpenBrowser();
      message = status.latest_message ?? "Browser opened.";
    } catch (error) {
      message = formatAppError("opening Gemini browser", error);
    } finally {
      busy = false;
    }
  }

  async function sendTestPrompt() {
    if (!prompt.trim()) {
      message = "Enter a prompt first.";
      return;
    }
    busy = true;
    result = null;
    try {
      result = await geminiBridgeSendSingle({
        runId: newRunId(),
        prompt: prompt.trim(),
        source: "settings_test",
        artifactMode: "reduced",
      });
      message = result.message ?? result.status;
      await refresh();
    } catch (error) {
      message = formatAppError("running Gemini browser prompt", error);
    } finally {
      busy = false;
    }
  }

  async function resumeProvider() {
    busy = true;
    try {
      await geminiBridgeResume();
      await refresh();
    } catch (error) {
      message = formatAppError("resuming Gemini browser provider", error);
    } finally {
      busy = false;
    }
  }

  async function stopProvider() {
    busy = true;
    try {
      await geminiBridgeStop();
      await refresh();
    } catch (error) {
      message = formatAppError("stopping Gemini browser provider", error);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;
    void refresh();
    void listenToGeminiBrowserRuns(({ payload }) => {
      if (disposed) return;
      message = payload.message ?? payload.status;
      void refresh();
    }).then((detach) => {
      if (disposed) {
        detach();
        return;
      }
      unlisten = detach;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

<div class="gemini-browser-panel">
  <div class="panel-head">
    <div>
      <h2>Browser Providers</h2>
      <p>Gemini via a persistent local browser profile.</p>
    </div>
    <span class="status-pill">{statusLabel()}</span>
  </div>

  <div class="provider-grid">
    <div class="provider-card">
      <div class="row">
        <strong>Gemini Browser</strong>
        <button type="button" onclick={refresh} disabled={busy} title="Refresh status">
          <RefreshCw size={14} />
        </button>
      </div>
      <p class="mono">{status?.browser_profile_dir ?? "Profile path will appear after status load."}</p>
      {#if message}
        <p class="message">{message}</p>
      {/if}
      <div class="actions">
        <button type="button" onclick={openBrowser} disabled={busy}>
          <ExternalLink size={14} />
          <span>Open</span>
        </button>
        <button type="button" onclick={resumeProvider} disabled={busy}>
          <Play size={14} />
          <span>Resume</span>
        </button>
        <button type="button" onclick={stopProvider} disabled={busy}>
          <Square size={14} />
          <span>Stop</span>
        </button>
      </div>
    </div>

    <div class="provider-card">
      <label for="gemini-browser-prompt">Test prompt</label>
      <textarea id="gemini-browser-prompt" bind:value={prompt} rows="5"></textarea>
      <button type="button" onclick={sendTestPrompt} disabled={busy || !prompt.trim()}>
        <Send size={14} />
        <span>{busy ? "Running..." : "Send"}</span>
      </button>
      {#if result?.text}
        <pre>{result.text}</pre>
      {/if}
    </div>
  </div>

  <div class="runs-list">
    <h3>Recent browser runs</h3>
    {#each runs as run (run.run_id)}
      <div class="run-row">
        <span>{run.status}</span>
        <code>{run.run_id}</code>
        <p>{run.prompt_preview}</p>
      </div>
    {:else}
      <p class="empty">No browser runs yet.</p>
    {/each}
  </div>
</div>

<style>
  .gemini-browser-panel {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .panel-head,
  .row,
  .actions,
  .run-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .panel-head {
    justify-content: space-between;
  }

  .panel-head h2,
  .runs-list h3 {
    margin: 0;
    font-size: 18px;
  }

  .panel-head p,
  .message,
  .empty {
    margin: 4px 0 0;
    color: var(--muted-foreground);
    font-size: 13px;
  }

  .status-pill {
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 700;
  }

  .provider-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 14px;
  }

  .provider-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    background: var(--card);
  }

  .provider-card button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 7px 10px;
    background: var(--background);
    color: var(--foreground);
    font-weight: 650;
  }

  .provider-card textarea {
    width: 100%;
    box-sizing: border-box;
    resize: vertical;
    margin: 6px 0 10px;
  }

  .mono,
  .run-row code {
    font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
    font-size: 12px;
    overflow-wrap: anywhere;
  }

  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px;
    max-height: 180px;
    overflow: auto;
  }

  .runs-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .run-row {
    align-items: flex-start;
    border-bottom: 1px solid var(--border);
    padding: 8px 0;
  }

  .run-row span {
    min-width: 110px;
    font-weight: 700;
  }

  .run-row p {
    margin: 0;
    flex: 1;
  }

  @media (max-width: 820px) {
    .provider-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
```

- [x] **Step 3: Add the Settings tab**

Modify `src/lib/components/settings/projects-settings.svelte` imports:

```svelte
import { Bot } from "@lucide/svelte";
import GeminiBrowserProviderPanel from "$lib/components/settings/gemini-browser-provider-panel.svelte";
```

Change active tab type by usage so it accepts `"browser"` and add this tab button after LLM Profiles:

```svelte
<button
  class="tab-btn"
  class:active={activeTab === "browser"}
  onclick={() => activeTab = "browser"}
>
  <Bot size={14} />
  <span>Browser Providers</span>
</button>
```

Add this branch before Telegram:

```svelte
{:else if activeTab === "browser"}
  <div class="settings-card">
    <GeminiBrowserProviderPanel />
  </div>
```

- [x] **Step 4: Run component-related checks**

Run:

```powershell
npm.cmd run test -- src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
```

Expected: Vitest passes and `svelte-check` reports `0 errors and 0 warnings`.

- [x] **Step 5: Commit**

Run:

```powershell
git add src/lib/gemini-browser-provider-panel-contract.ts src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/components/settings/projects-settings.svelte src/lib/gemini-browser-provider-panel.test.ts docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: add Gemini browser settings panel"
```

Expected: commit includes Settings UI files and updated plan checkbox only.

---

## Task 7: TypeScript Sidecar Protocol Skeleton

**Files:**
- Create: `sidecars/gemini-browser/package.json`
- Create: `sidecars/gemini-browser/tsconfig.json`
- Create: `sidecars/gemini-browser/tsconfig.build.json`
- Create: `sidecars/gemini-browser/src/protocol.ts`
- Create: `sidecars/gemini-browser/src/redaction.ts`
- Create: `sidecars/gemini-browser/src/protocol.test.ts`
- Modify: `.gitignore`
- Modify: `package.json`

- [x] **Step 1: Add sidecar package scripts**

Create `sidecars/gemini-browser/package.json`:

```json
{
  "name": "@extractum/gemini-browser-sidecar",
  "private": true,
  "type": "module",
  "scripts": {
    "build": "tsc -p tsconfig.build.json",
    "check": "tsc -p tsconfig.json --noEmit",
    "test": "vitest run src"
  }
}
```

Create `sidecars/gemini-browser/tsconfig.json`:

```json
{
  "extends": "../../tsconfig.json",
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "bundler",
    "types": ["node", "vitest"],
    "noEmit": true,
    "strict": true
  },
  "include": ["src/**/*.ts"]
}
```

Create `sidecars/gemini-browser/tsconfig.build.json`:

```json
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "noEmit": false,
    "outDir": "dist",
    "rootDir": "src",
    "sourceMap": true,
    "declaration": false
  },
  "exclude": ["src/**/*.test.ts"]
}
```

- [x] **Step 2: Add protocol and redaction utilities**

Create `sidecars/gemini-browser/src/protocol.ts`:

```ts
export type GeminiBrowserRunStatus =
  | "queued"
  | "running"
  | "ok"
  | "ready"
  | "needs_login"
  | "needs_manual_action"
  | "blocked"
  | "timeout"
  | "browser_crashed"
  | "failed"
  | "cancelled";

export interface GeminiBrowserRunRequest {
  run_id: string;
  prompt: string;
  source: string;
  artifact_mode: "reduced" | "full";
}

export interface GeminiBrowserRunResult {
  run_id: string;
  status: GeminiBrowserRunStatus;
  text: string | null;
  message: string | null;
  manual_action: string | null;
  artifacts: {
    run_dir: string | null;
    html: string | null;
    screenshot: string | null;
    telemetry: string | null;
    artifact_write_error: string | null;
  };
  elapsed_ms: number;
}

export interface GeminiBrowserProviderStatus {
  status: "not_started" | "ready" | "needs_login" | "needs_manual_action" | "running" | "stopped" | "failed";
  manual_action: string | null;
  active_run_id: string | null;
  queue_depth: number;
  browser_profile_dir: string;
  latest_message: string | null;
}

export type SidecarCommand =
  | { type: "status"; browser_profile_dir: string }
  | { type: "open_browser"; browser_profile_dir: string }
  | {
      type: "send_single";
      request: GeminiBrowserRunRequest;
      browser_profile_dir: string;
      artifact_dir: string;
    }
  | { type: "resume"; run_id: string | null }
  | { type: "stop" };

export interface SidecarEnvelope {
  id: string;
  command: SidecarCommand;
}

export type SidecarResponse =
  | { type: "status"; status: GeminiBrowserProviderStatus }
  | { type: "run_result"; result: GeminiBrowserRunResult }
  | { type: "ack" }
  | { type: "error"; message: string };

export function parseEnvelope(line: string): SidecarEnvelope {
  const value = JSON.parse(line) as SidecarEnvelope;
  if (!value.id || typeof value.id !== "string") {
    throw new Error("Sidecar envelope id is required");
  }
  if (!value.command || typeof value.command.type !== "string") {
    throw new Error("Sidecar command type is required");
  }
  return value;
}
```

Create `sidecars/gemini-browser/src/redaction.ts`:

```ts
const SECRET_QUERY_KEYS = new Set(["authuser", "token", "key", "password", "prompt"]);

export function redactUrl(rawUrl: string): string {
  try {
    const url = new URL(rawUrl);
    for (const key of [...url.searchParams.keys()]) {
      if (SECRET_QUERY_KEYS.has(key.toLowerCase())) {
        url.searchParams.set(key, "[redacted]");
      }
    }
    return url.toString();
  } catch {
    return "[invalid-url]";
  }
}

export function redactText(value: string, prompt: string): string {
  const trimmedPrompt = prompt.trim();
  if (!trimmedPrompt) return value;
  return value.split(trimmedPrompt).join("[prompt]");
}
```

- [x] **Step 3: Add sidecar unit tests**

Create `sidecars/gemini-browser/src/protocol.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { parseEnvelope } from "./protocol";
import { redactText, redactUrl } from "./redaction";

describe("gemini browser sidecar protocol", () => {
  it("parses a status command envelope", () => {
    const envelope = parseEnvelope(
      JSON.stringify({
        id: "1",
        command: { type: "status", browser_profile_dir: "G:/Extractum/profile" },
      }),
    );

    expect(envelope.command.type).toBe("status");
  });

  it("rejects envelopes without command type", () => {
    expect(() => parseEnvelope(JSON.stringify({ id: "1", command: {} }))).toThrow(
      "Sidecar command type is required",
    );
  });

  it("redacts sensitive URL params and prompt text", () => {
    expect(redactUrl("https://gemini.google.com/app?authuser=dima&prompt=hello")).toContain(
      "authuser=%5Bredacted%5D",
    );
    expect(redactText("hello answer", "hello")).toBe("[prompt] answer");
  });
});
```

- [x] **Step 4: Add root scripts**

Modify `package.json` scripts:

```json
"test:gemini-browser-sidecar:typecheck": "tsc -p sidecars/gemini-browser/tsconfig.json --noEmit",
"test:gemini-browser-sidecar:unit": "node scripts/run-vitest.mjs run sidecars/gemini-browser/src",
"test:gemini-browser-sidecar:build": "tsc -p sidecars/gemini-browser/tsconfig.build.json",
"test:gemini-browser-sidecar": "npm run test:gemini-browser-sidecar:typecheck && npm run test:gemini-browser-sidecar:unit && npm run test:gemini-browser-sidecar:build"
```

- [x] **Step 5: Run sidecar tests**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
```

Expected: sidecar typecheck and unit tests pass.

- [x] **Step 6: Commit**

Run:

```powershell
git add sidecars/gemini-browser .gitignore package.json docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: add Gemini browser sidecar protocol"
```

Expected: commit includes sidecar skeleton, package scripts, and updated plan checkbox only.

---

## Task 8: Port Resilient Adapter Into Sidecar

**Files:**
- Create: `sidecars/gemini-browser/src/dom-contract.ts`
- Create: `sidecars/gemini-browser/src/adapter.ts`
- Create: `sidecars/gemini-browser/src/artifacts.ts`
- Create: `sidecars/gemini-browser/src/adapter.test.ts`
- Read: `research/gemini_browser_adapter/src/resilient-scoring.ts`
- Read: `research/gemini_browser_adapter/src/dom-contract.ts`
- Read: `research/gemini_browser_adapter/src/artifacts.ts`

- [x] **Step 1: Copy the production DOM contract names**

Create `sidecars/gemini-browser/src/dom-contract.ts`:

```ts
export interface GeminiSelectorCandidate {
  selector: string;
  score: number;
  purpose: "composer" | "send" | "answer" | "manual_action";
}

export const GEMINI_DOM_CONTRACT_VERSION = "2026-06-20-resilient-scoring";

export const composerCandidates: GeminiSelectorCandidate[] = [
  { selector: "rich-textarea textarea", score: 100, purpose: "composer" },
  { selector: "textarea[aria-label*='prompt' i]", score: 80, purpose: "composer" },
  { selector: "[contenteditable='true']", score: 50, purpose: "composer" },
];

export const sendCandidates: GeminiSelectorCandidate[] = [
  { selector: "button[aria-label*='send' i]", score: 100, purpose: "send" },
  { selector: "button[type='submit']", score: 70, purpose: "send" },
];

export const answerCandidates: GeminiSelectorCandidate[] = [
  { selector: "[data-response-index]", score: 100, purpose: "answer" },
  { selector: "message-content", score: 90, purpose: "answer" },
  { selector: "article [dir='ltr']", score: 65, purpose: "answer" },
];
```

- [x] **Step 2: Add safe artifact writer**

Create `sidecars/gemini-browser/src/artifacts.ts`:

```ts
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "@playwright/test";
import type { GeminiBrowserRunRequest, GeminiBrowserRunResult } from "./protocol";
import { redactText, redactUrl } from "./redaction";

export async function captureFailureArtifacts(input: {
  page: Page;
  artifactDir: string;
  request: GeminiBrowserRunRequest;
  status: GeminiBrowserRunResult["status"];
  message: string;
}): Promise<GeminiBrowserRunResult["artifacts"]> {
  await mkdir(input.artifactDir, { recursive: true });
  const telemetryPath = join(input.artifactDir, "telemetry.json");
  const htmlPath = input.request.artifact_mode === "full" ? join(input.artifactDir, "page.html") : null;
  const screenshotPath = input.request.artifact_mode === "full" ? join(input.artifactDir, "page.png") : null;
  let artifactWriteError: string | null = null;

  const pageUrl = await input.page.url().catch(() => "about:blank");
  const telemetry = {
    status: input.status,
    message: input.message,
    url: redactUrl(pageUrl),
    artifact_mode: input.request.artifact_mode,
  };

  await writeFile(telemetryPath, JSON.stringify(telemetry, null, 2)).catch((error) => {
    artifactWriteError = String(error);
  });

  if (htmlPath) {
    const html = await input.page.content().catch(() => "<html><body>[page unavailable]</body></html>");
    await writeFile(htmlPath, redactText(html, input.request.prompt)).catch((error) => {
      artifactWriteError = String(error);
    });
  }

  if (screenshotPath) {
    await input.page.screenshot({ path: screenshotPath, fullPage: true }).catch((error) => {
      artifactWriteError = String(error);
    });
  }

  return {
    run_dir: input.artifactDir,
    html: htmlPath,
    screenshot: screenshotPath,
    telemetry: telemetryPath,
    artifact_write_error: artifactWriteError,
  };
}
```

- [x] **Step 3: Add adapter facade**

Create `sidecars/gemini-browser/src/adapter.ts`:

```ts
import { chromium, type BrowserContext, type Page } from "@playwright/test";
import { mkdir } from "node:fs/promises";
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRunRequest,
  GeminiBrowserRunResult,
} from "./protocol";
import { answerCandidates, composerCandidates, sendCandidates } from "./dom-contract";
import { captureFailureArtifacts } from "./artifacts";

export class GeminiBrowserAdapter {
  private context: BrowserContext | null = null;
  private page: Page | null = null;

  async status(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    return {
      status: this.page ? "ready" : "not_started",
      manual_action: null,
      active_run_id: null,
      queue_depth: 0,
      browser_profile_dir: browserProfileDir,
      latest_message: this.page ? "Browser page is available." : "Browser has not been opened.",
    };
  }

  async openBrowser(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    await mkdir(browserProfileDir, { recursive: true });
    this.context = await chromium.launchPersistentContext(browserProfileDir, {
      headless: false,
      viewport: { width: 1280, height: 900 },
    });
    this.page = this.context.pages()[0] ?? (await this.context.newPage());
    await this.page.goto("https://gemini.google.com/app", { waitUntil: "domcontentloaded" });
    return this.status(browserProfileDir);
  }

  async sendSingle(input: {
    request: GeminiBrowserRunRequest;
    browserProfileDir: string;
    artifactDir: string;
  }): Promise<GeminiBrowserRunResult> {
    const start = Date.now();
    if (!this.page) {
      await this.openBrowser(input.browserProfileDir);
    }
    const page = this.page;
    if (!page) {
      throw new Error("Gemini browser page was not created");
    }

    try {
      const composer = await firstVisible(page, composerCandidates.map((candidate) => candidate.selector));
      if (!composer) {
        return this.failure(page, input.request, input.artifactDir, "needs_login", "Composer was not found.", start);
      }
      await composer.fill(input.request.prompt).catch(async () => {
        await composer.click();
        await page.keyboard.insertText(input.request.prompt);
      });

      const send = await firstVisible(page, sendCandidates.map((candidate) => candidate.selector));
      if (!send) {
        return this.failure(page, input.request, input.artifactDir, "needs_manual_action", "Send button was not found.", start);
      }
      await send.click();

      const answer = await waitForAnswerText(page, input.request.prompt);
      if (!answer) {
        return this.failure(page, input.request, input.artifactDir, "timeout", "Answer did not appear before timeout.", start);
      }

      return {
        run_id: input.request.run_id,
        status: "ok",
        text: answer,
        message: null,
        manual_action: null,
        artifacts: {
          run_dir: input.artifactDir,
          html: null,
          screenshot: null,
          telemetry: null,
          artifact_write_error: null,
        },
        elapsed_ms: Date.now() - start,
      };
    } catch (error) {
      return this.failure(page, input.request, input.artifactDir, "failed", String(error), start);
    }
  }

  async stop(): Promise<void> {
    await this.context?.close().catch(() => undefined);
    this.context = null;
    this.page = null;
  }

  private async failure(
    page: Page,
    request: GeminiBrowserRunRequest,
    artifactDir: string,
    status: GeminiBrowserRunResult["status"],
    message: string,
    start: number,
  ): Promise<GeminiBrowserRunResult> {
    return {
      run_id: request.run_id,
      status,
      text: null,
      message,
      manual_action: status === "needs_login" ? "login" : null,
      artifacts: await captureFailureArtifacts({ page, artifactDir, request, status, message }),
      elapsed_ms: Date.now() - start,
    };
  }
}

async function firstVisible(page: Page, selectors: string[]) {
  for (const selector of selectors) {
    const locator = page.locator(selector).last();
    if ((await locator.count()) > 0 && (await locator.isVisible().catch(() => false))) {
      return locator;
    }
  }
  return null;
}

async function waitForAnswerText(page: Page, prompt: string): Promise<string | null> {
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    for (const selector of answerCandidates.map((candidate) => candidate.selector)) {
      const texts = await page.locator(selector).allTextContents().catch(() => []);
      const answer = texts
        .map((text) => text.trim())
        .filter((text) => text.length > 0 && text !== prompt)
        .at(-1);
      if (answer) return answer;
    }
    await page.waitForTimeout(500);
  }
  return null;
}
```

- [x] **Step 4: Add adapter tests for non-browser helpers**

Create `sidecars/gemini-browser/src/adapter.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  GEMINI_DOM_CONTRACT_VERSION,
  answerCandidates,
  composerCandidates,
  sendCandidates,
} from "./dom-contract";

describe("production Gemini DOM contract", () => {
  it("keeps the selected resilient-scoring contract version explicit", () => {
    expect(GEMINI_DOM_CONTRACT_VERSION).toBe("2026-06-20-resilient-scoring");
  });

  it("has candidates for composer, send, and answer extraction", () => {
    expect(composerCandidates.length).toBeGreaterThan(0);
    expect(sendCandidates.length).toBeGreaterThan(0);
    expect(answerCandidates.length).toBeGreaterThan(0);
    expect(answerCandidates.some((candidate) => candidate.selector === "main section")).toBe(false);
  });
});
```

- [x] **Step 5: Run sidecar verification and research guard**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
npm.cmd run test:gemini-browser-adapter
```

Expected: sidecar tests pass; research matrix still passes.

- [x] **Step 6: Commit**

Run:

```powershell
git add sidecars/gemini-browser docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: port resilient Gemini browser adapter"
```

Expected: commit includes sidecar adapter files and updated plan checkbox only.

---

## Task 9: Sidecar JSON-Line Server And Rust Process Client

**Files:**
- Create: `sidecars/gemini-browser/src/index.ts`
- Create: `sidecars/gemini-browser/tsconfig.build.json`
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `package.json`
- Test: `sidecars/gemini-browser/src/protocol.test.ts`

- [ ] **Step 1: Add the sidecar JSON-line server**

Create `sidecars/gemini-browser/src/index.ts`:

```ts
import readline from "node:readline";
import { GeminiBrowserAdapter } from "./adapter";
import { parseEnvelope, type SidecarResponse } from "./protocol";

const adapter = new GeminiBrowserAdapter();

function writeResponse(id: string, response: SidecarResponse) {
  process.stdout.write(`${JSON.stringify({ id, response })}\n`);
}

const rl = readline.createInterface({
  input: process.stdin,
  crlfDelay: Infinity,
});

rl.on("line", async (line) => {
  let id = "unknown";
  try {
    const envelope = parseEnvelope(line);
    id = envelope.id;
    const command = envelope.command;
    if (command.type === "status") {
      writeResponse(id, { type: "status", status: await adapter.status(command.browser_profile_dir) });
      return;
    }
    if (command.type === "open_browser") {
      writeResponse(id, { type: "status", status: await adapter.openBrowser(command.browser_profile_dir) });
      return;
    }
    if (command.type === "send_single") {
      writeResponse(id, {
        type: "run_result",
        result: await adapter.sendSingle({
          request: command.request,
          browserProfileDir: command.browser_profile_dir,
          artifactDir: command.artifact_dir,
        }),
      });
      return;
    }
    if (command.type === "resume") {
      writeResponse(id, { type: "ack" });
      return;
    }
    if (command.type === "stop") {
      await adapter.stop();
      writeResponse(id, { type: "ack" });
    }
  } catch (error) {
    writeResponse(id, { type: "error", message: String(error) });
  }
});
```

- [ ] **Step 2: Add root script for local sidecar smoke**

Modify `package.json` scripts:

```json
"gemini-browser-sidecar": "node sidecars/gemini-browser/dist/index.js"
```

- [ ] **Step 3: Build the sidecar server**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:build
```

Expected: `sidecars/gemini-browser/dist/index.js` is emitted without TypeScript errors.

- [ ] **Step 4: Add sidecar process storage to Rust state**

Modify `src-tauri/src/gemini_browser/state.rs` imports and state:

```rust
use tokio::sync::{Mutex, MutexGuard};

#[derive(Default)]
pub struct GeminiBrowserState {
    queue: Mutex<VecDeque<GeminiBrowserRunRequest>>,
    active_run_id: Mutex<Option<String>>,
    cancellation: Mutex<Option<CancellationToken>>,
    sidecar: Mutex<Option<super::sidecar::GeminiBrowserSidecarProcess>>,
}

impl GeminiBrowserState {
    pub(crate) async fn sidecar(
        &self,
    ) -> MutexGuard<'_, Option<super::sidecar::GeminiBrowserSidecarProcess>> {
        self.sidecar.lock().await
    }
}
```

- [ ] **Step 5: Replace Rust sidecar stub with a JSON-line process client**

Replace `src-tauri/src/gemini_browser/sidecar.rs` with:

```rust
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
            .map_err(|error| AppError::internal(format!("Failed to start Gemini browser sidecar: {error}")))?;
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
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|error| AppError::internal(format!("Failed to write Gemini sidecar request: {error}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|error| AppError::internal(format!("Failed to flush Gemini sidecar request: {error}")))?;

        let mut response_line = String::new();
        let bytes = self
            .stdout
            .read_line(&mut response_line)
            .await
            .map_err(|error| AppError::internal(format!("Failed to read Gemini sidecar response: {error}")))?;
        if bytes == 0 {
            return Err(AppError::internal("Gemini browser sidecar exited without a response"));
        }
        let response: SidecarLine = serde_json::from_str(&response_line)
            .map_err(|error| AppError::internal(format!("Invalid Gemini sidecar response: {error}")))?;
        if response.id != id {
            return Err(AppError::internal("Gemini browser sidecar response id mismatch"));
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
        Ok(_) => Err(AppError::internal("Unexpected Gemini sidecar status response")),
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
        GeminiBrowserSidecarCommand::OpenBrowser { browser_profile_dir },
    )
    .await?
    {
        GeminiBrowserSidecarResponse::Status { status } => Ok(status),
        _ => Err(AppError::internal("Unexpected Gemini sidecar open_browser response")),
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
        _ => Err(AppError::internal("Unexpected Gemini sidecar send_single response")),
    }
}

pub(crate) async fn stop(handle: &AppHandle, state: &GeminiBrowserState) -> AppResult<()> {
    let _ = request_sidecar(handle, state, GeminiBrowserSidecarCommand::Stop).await;
    *state.sidecar().await = None;
    Ok(())
}

pub(crate) fn sidecar_unavailable_result(request: GeminiBrowserRunRequest) -> GeminiBrowserRunResult {
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
```

- [ ] **Step 6: Route commands through the process client**

Modify `src-tauri/src/gemini_browser/commands.rs`:

```rust
sidecar::status(
    &handle,
    &state,
    path_string(&profile_dir(&handle)?),
    state.active_run_id().await,
    state.queue_depth().await,
)
.await
```

Use this for `gemini_bridge_open_browser`:

```rust
pub async fn gemini_bridge_open_browser(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<super::GeminiBrowserProviderStatus> {
    sidecar::open_browser(&handle, &state, path_string(&profile_dir(&handle)?)).await
}
```

Replace the send call:

```rust
let artifact_dir = super::paths::path_string(&super::paths::run_dir(&handle, &next.run_id)?);
let result = match sidecar::send_single(
    &handle,
    &state,
    next.clone(),
    path_string(&profile_dir(&handle)?),
    artifact_dir,
)
.await
{
    Ok(result) => result,
    Err(_) => sidecar::sidecar_unavailable_result(next.clone()),
};
```

Use this for stop:

```rust
pub async fn gemini_bridge_stop(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<()> {
    state.request_stop().await;
    sidecar::stop(&handle, &state).await
}
```

- [ ] **Step 7: Run sidecar and Rust checks**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser
```

Expected: sidecar and Rust tests pass.

- [ ] **Step 8: Commit**

Run:

```powershell
git add sidecars/gemini-browser/src/index.ts sidecars/gemini-browser/tsconfig.build.json src-tauri/src/gemini_browser/sidecar.rs src-tauri/src/gemini_browser/state.rs src-tauri/src/gemini_browser/commands.rs package.json docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: add Gemini browser sidecar server"
```

Expected: commit includes sidecar server/client process wiring and updated plan checkbox only.

---

## Task 10: Prompt Pack Handoff Spike

**Files:**
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Create: `src-tauri/src/prompt_packs/gemini_browser_stage.rs`
- Test: `src-tauri/src/prompt_packs/gemini_browser_stage.rs`

- [ ] **Step 1: Add a browser completion adapter function**

Create `src-tauri/src/prompt_packs/gemini_browser_stage.rs`:

```rust
use crate::error::{AppError, AppResult};
use crate::gemini_browser::{GeminiBrowserRunResult, GeminiBrowserRunStatus};

pub(crate) fn browser_result_to_completion_text(result: GeminiBrowserRunResult) -> AppResult<String> {
    match result.status {
        GeminiBrowserRunStatus::Ok => result
            .text
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| AppError::internal("Gemini browser result did not include text")),
        GeminiBrowserRunStatus::Ready => Err(AppError::internal(
            "Gemini browser readiness result cannot be used as a prompt completion",
        )),
        status => Err(AppError::internal(format!(
            "Gemini browser prompt failed with status {status:?}: {}",
            result.message.unwrap_or_else(|| "No message".to_string())
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gemini_browser::{GeminiBrowserArtifactRefs, GeminiBrowserRunResult};

    fn result(status: GeminiBrowserRunStatus, text: Option<&str>) -> GeminiBrowserRunResult {
        GeminiBrowserRunResult {
            run_id: "run-1".to_string(),
            status,
            text: text.map(ToString::to_string),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 1,
        }
    }

    #[test]
    fn ok_browser_result_maps_to_completion_text() {
        assert_eq!(
            browser_result_to_completion_text(result(GeminiBrowserRunStatus::Ok, Some("answer")))
                .expect("completion"),
            "answer"
        );
    }

    #[test]
    fn ready_result_is_not_prompt_completion() {
        let error = browser_result_to_completion_text(result(GeminiBrowserRunStatus::Ready, None))
            .expect_err("ready is not completion");
        assert!(error.message.contains("readiness"));
    }
}
```

- [ ] **Step 2: Wire module export**

Modify `src-tauri/src/prompt_packs/mod.rs`:

```rust
pub(crate) mod gemini_browser_stage;
```

- [ ] **Step 3: Leave runtime selection behind a narrow feature flag**

Modify `src-tauri/src/prompt_packs/runtime.rs` only by adding a comment near `run_transcript_analysis_stage_request`:

```rust
// Gemini Browser Provider completion routing will call gemini_browser_stage::browser_result_to_completion_text
// after the provider command returns a successful single-prompt result. The default Prompt Pack path remains
// API-backed until a run request explicitly selects the browser provider.
```

- [ ] **Step 4: Run focused Prompt Pack test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser_stage
```

Expected: completion mapping tests pass.

- [ ] **Step 5: Commit**

Run:

```powershell
git add src-tauri/src/prompt_packs/gemini_browser_stage.rs src-tauri/src/prompt_packs/mod.rs src-tauri/src/prompt_packs/runtime.rs docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "feat: add Gemini browser prompt pack handoff"
```

Expected: commit includes the narrow handoff module and updated plan checkbox only.

---

## Task 11: Final Verification And Documentation

**Files:**
- Modify: `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`
- Modify: `research/gemini_browser_adapter/DECISION.md`

- [ ] **Step 1: Document MVP implementation status**

Append to `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`:

```markdown
## MVP Implementation Notes - 2026-06-20

- The production provider boundary is `src-tauri/src/gemini_browser`.
- Browser Provider UI is exposed through Settings -> Browser Providers.
- Browser profile and run logs live under the Tauri app data directory, not in the repository.
- Runtime automation uses the TypeScript sidecar boundary under `sidecars/gemini-browser`.
- The selected first adapter remains `resilient-scoring`; research verification remains available through `npm.cmd run test:gemini-browser-adapter` in PowerShell.
```

Append to `research/gemini_browser_adapter/DECISION.md`:

```markdown
## Production Handoff - 2026-06-20

- MVP implementation plan: `docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md`.
- Production code must not import from `research/gemini_browser_adapter`; research stays as a regression harness and evidence source.
```

- [ ] **Step 2: Run full feature verification**

Run:

```powershell
npm.cmd run test:gemini-browser-adapter
npm.cmd run test:gemini-browser-sidecar
npm.cmd run test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser_stage
```

Expected:

```text
research matrix exits 0
sidecar typecheck and unit tests pass
frontend unit tests pass
svelte-check reports 0 errors and 0 warnings
Rust Gemini browser tests pass
Rust prompt-pack handoff tests pass
```

- [ ] **Step 3: Confirm no Python runtime additions**

Run:

```powershell
git diff --name-only main...HEAD
rg -n "python|\\.py|child_process.*python|Command::new\\(\"python" src-tauri/src src sidecars/gemini-browser package.json
```

Expected: no production Python runtime invocation for Gemini Browser Provider.

- [ ] **Step 4: Commit documentation**

Run:

```powershell
git add docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md research/gemini_browser_adapter/DECISION.md docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md
git commit -m "docs: document Gemini browser MVP handoff"
```

Expected: docs-only commit.

---

## Self-Review

**Spec coverage:** The plan covers separate Browser Provider UI, app-data browser profile, run queue, six Tauri commands, sidecar-owned Playwright, file-backed run logs, manual-action statuses, selected `resilient-scoring` adapter, Prompt Pack handoff, and no Python runtime.

**Release follow-up:** Packaging the sidecar as a release artifact is tracked outside this MVP plan. Task 9 provides a working development process client that runs `node sidecars/gemini-browser/dist/index.js`.

**Forbidden marker scan:** The plan avoids marker strings, vague validation steps, and undefined function names. Each task contains concrete paths, commands, and expected results.

**Type consistency:** Rust serde names use snake_case DTO fields; frontend command wrappers use Tauri camelCase argument keys; sidecar JSON-line protocol uses snake_case payload fields to match Rust DTO serialization.
