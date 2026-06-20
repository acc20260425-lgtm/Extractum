# Gemini Browser Sidecar Packaging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Gemini Browser Provider sidecar launchable from a packaged Tauri app instead of relying only on `node sidecars/gemini-browser/dist/index.js` from the repository working directory.

**Architecture:** Keep the existing TypeScript sidecar protocol and resilient adapter. Add a release-aware sidecar launcher that prefers a Tauri bundled sidecar binary and keeps an explicit development fallback for local runs. Package only the sidecar executable and sidecar code; browser profile data, cookies, credentials, run artifacts, and live Gemini transcripts remain app-data/runtime files and are never bundled.

**Tech Stack:** Tauri 2, `tauri-plugin-shell`, Rust async command handling, Node/TypeScript sidecar, Playwright, `esbuild` CJS sidecar bundling, `pkg`-style Node sidecar binary packaging, Vitest, Cargo tests.

---

## Context

The research decision in `research/gemini_browser_adapter/DECISION.md` is accepted: production uses the resilient-scoring adapter with safe telemetry/artifacts only. The MVP production plan in `docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md` is complete.

The remaining release gap is called out in that plan:

```text
Release follow-up: Packaging the sidecar as a release artifact is tracked outside this MVP plan.
```

Current launcher behavior in `src-tauri/src/gemini_browser/sidecar.rs` is development-only:

```rust
let script_path = std::env::current_dir()?
    .join("sidecars")
    .join("gemini-browser")
    .join("dist")
    .join("index.js");
let mut child = Command::new("node").arg(script_path)...
```

This works from the repository after `npm.cmd run test:gemini-browser-sidecar:build`, but a bundled app should not depend on repo cwd.

Tauri 2 sidecar packaging uses `bundle.externalBin` with platform target-triple binary names, and the official sidecar API expects the sidecar filename rather than a raw path. Task 9 verifies this assumption with a packaged app smoke that forces bundled sidecar mode and checks for a sidecar-origin `status` response.

This first packaging slice is host-target only. It supports the current developer machine target returned by `rustc --print host-tuple`; cross-target Tauri builds are intentionally out of scope until a CI/release matrix exists.

---

## File Structure

- Modify `src-tauri/Cargo.toml`: add `tauri-plugin-shell`.
- Modify `src-tauri/tauri.conf.json`: add `bundle.externalBin` for `binaries/gemini-browser-sidecar`.
- Modify `src-tauri/src/lib.rs`: register `tauri_plugin_shell::init()`.
- Modify `src-tauri/src/gemini_browser/sidecar.rs`: replace repo-cwd process spawning with a release-aware launcher.
- Create `src-tauri/src/gemini_browser/sidecar_launch.rs`: pure launch-mode resolution helpers and tests.
- Modify `src-tauri/src/gemini_browser/mod.rs`: include and expose the launch helper module internally.
- Create `scripts/build-gemini-browser-sidecar.mjs`: build the TypeScript sidecar and create `src-tauri/binaries/gemini-browser-sidecar-<target-triple>[.exe]`.
- Create `scripts/check-gemini-browser-sidecar-binary.mjs`: verify the expected platform-named binary exists before bundle builds.
- Modify `package.json`: add packaging/check scripts.
- Modify `.gitignore`: ignore generated `src-tauri/binaries/gemini-browser-sidecar-*` binaries.
- Modify `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`: add release packaging notes.
- Modify `research/gemini_browser_adapter/DECISION.md`: add a short production packaging handoff note.

---

## Task 1: Branch And Baseline Guard

**Files:**
- Modify: `docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md`

- [x] **Step 1: Create the feature branch**

Run:

```powershell
git switch -c gemini-browser-sidecar-packaging
```

Expected: `Switched to a new branch 'gemini-browser-sidecar-packaging'`.

- [x] **Step 2: Confirm the worktree is clean**

Run:

```powershell
git status --short --branch
```

Expected: only the new branch header, no modified files.

- [x] **Step 3: Run current sidecar verification**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
```

Expected: sidecar typecheck, unit tests, and build pass.

- [x] **Step 4: Run current Rust Gemini browser tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser
```

Expected: existing `gemini_browser` tests pass.

- [x] **Step 5: Mark Task 1 complete and commit**

Update this task's checkboxes to `[x]`.

Run:

```powershell
git add docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "docs: start Gemini sidecar packaging plan"
```

Expected: commit contains only the plan checkbox update.

---

## Task 2: Packaging Feasibility Smoke

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md`

- [x] **Step 1: Add the sidecar packager dependencies**

Run:

```powershell
$npm = if ($IsWindows -or $env:OS -eq 'Windows_NT') { 'npm.cmd' } else { 'npm' }
& $npm install --save-dev pkg esbuild
```

Expected: `package.json` and `package-lock.json` include `pkg` and `esbuild`.

- [x] **Step 2: Build the current TypeScript sidecar**

Run:

```powershell
$npm = if ($IsWindows -or $env:OS -eq 'Windows_NT') { 'npm.cmd' } else { 'npm' }
& $npm run test:gemini-browser-sidecar:build
```

Expected: `sidecars/gemini-browser/dist/index.js` exists.

- [x] **Step 3: Package the current sidecar before Rust/Tauri changes**

Run:

```powershell
$ext = if ($IsWindows -or $env:OS -eq 'Windows_NT') { '.exe' } else { '' }
$npx = if ($IsWindows -or $env:OS -eq 'Windows_NT') { 'npx.cmd' } else { 'npx' }
$pkgPlatform = if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
  'win'
} elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)) {
  'macos'
} elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)) {
  'linux'
} else {
  throw 'Unsupported pkg platform'
}
$pkgArch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
  'X64' { 'x64'; break }
  'Arm64' { 'arm64'; break }
  default { throw "Unsupported pkg architecture: $([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)" }
}
$pkgTarget = "node18-$pkgPlatform-$pkgArch"
New-Item -ItemType Directory -Force artifacts | Out-Null
@'
{
  "pkg": {
    "assets": [
      "../node_modules/playwright-core/browsers.json"
    ]
  }
}
'@ | Set-Content -Encoding UTF8 artifacts/gemini-browser-pkg.config.json
& $npx esbuild sidecars/gemini-browser/src/index.ts --bundle --platform=node --format=cjs --packages=external --outfile=artifacts/gemini-browser-sidecar-bundle.cjs
& $npx pkg artifacts/gemini-browser-sidecar-bundle.cjs --config artifacts/gemini-browser-pkg.config.json --targets $pkgTarget --no-bytecode --public --public-packages "*" --output "artifacts/gemini-browser-sidecar-feasibility$ext"
```

Expected: the command exits `0` and writes a local feasibility binary under `artifacts/`.

This intentionally uses `esbuild` to convert the TypeScript/ESM sidecar entrypoint into a CommonJS bundle before `pkg` runs. Direct `pkg sidecars/gemini-browser/dist/index.js` is not valid for this project because the current developer Node can be newer than `pkg`'s supported target list, and the ESM/Playwright dependency graph needs an explicit CommonJS entrypoint plus the `playwright-core/browsers.json` asset.

If this `esbuild -> pkg` pipeline fails because `pkg` cannot package the current Playwright dependency graph, stop this plan before changing Rust/Tauri. Replace the packaging tool in this task and in Task 6 with the smallest working sidecar binary packaging approach, then re-run this feasibility smoke.

- [x] **Step 4: Smoke the packaged binary protocol**

Run:

```powershell
$ext = if ($IsWindows -or $env:OS -eq 'Windows_NT') { '.exe' } else { '' }
$response = @'
{"id":"feasibility-1","command":{"type":"status","browser_profile_dir":"artifacts/gemini-browser-feasibility-profile"}}
'@ | & "artifacts/gemini-browser-sidecar-feasibility$ext"
$response
```

Expected: stdout contains one JSON response with `id: "feasibility-1"` and `response.type: "status"`.

If the packaged process hangs or does not exit after stdin closes, stop and add a tiny script smoke before proceeding. Do not proceed to Rust/Tauri integration until the packaged binary can answer `status`.

This is a protocol/import feasibility gate only. It proves the packaged Node entrypoint starts, parses JSONL, and answers `status`; it does not launch a Playwright browser context. Browser launch from the packaged binary is verified later by Task 7.

- [x] **Step 5: Confirm feasibility artifacts are ignored**

Run:

```powershell
git status --short --untracked-files=all artifacts
```

Expected: no feasibility binary, stdout, stderr, stdin, or profile data appears in git status.

- [x] **Step 6: Commit**

Update this task's checkboxes to `[x]`.

Run:

```powershell
git add package.json package-lock.json docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "build: prove Gemini sidecar packaging feasibility"
```

Expected: commit includes the packager dependencies and plan checkbox update only. Feasibility binaries and profile artifacts remain ignored.

---

## Task 3: Add Launch Mode Resolution

**Files:**
- Create: `src-tauri/src/gemini_browser/sidecar_launch.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Test: `src-tauri/src/gemini_browser/sidecar_launch.rs`

- [x] **Step 1: Add failing launch resolution tests**

Create `src-tauri/src/gemini_browser/sidecar_launch.rs`:

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeminiBrowserSidecarLaunch {
    Bundled { name: String },
    DevNodeScript { node: String, script: PathBuf },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeminiBrowserBuildProfile {
    Debug,
    Release,
}

pub(crate) const GEMINI_BROWSER_SIDECAR_NAME: &str = "gemini-browser-sidecar";

pub(crate) fn dev_sidecar_script(repo_root: &Path) -> PathBuf {
    repo_root
        .join("sidecars")
        .join("gemini-browser")
        .join("dist")
        .join("index.js")
}

pub(crate) fn resolve_launch_mode(
    build_profile: GeminiBrowserBuildProfile,
    force_dev: bool,
    force_bundled: bool,
    repo_root: &Path,
    dev_script_exists: bool,
) -> GeminiBrowserSidecarLaunch {
    if force_bundled {
        return GeminiBrowserSidecarLaunch::Bundled {
            name: GEMINI_BROWSER_SIDECAR_NAME.to_string(),
        };
    }

    if force_dev && dev_script_exists {
        return GeminiBrowserSidecarLaunch::DevNodeScript {
            node: "node".to_string(),
            script: dev_sidecar_script(repo_root),
        };
    }

    if build_profile == GeminiBrowserBuildProfile::Debug && dev_script_exists {
        return GeminiBrowserSidecarLaunch::DevNodeScript {
            node: "node".to_string(),
            script: dev_sidecar_script(repo_root),
        };
    }

    GeminiBrowserSidecarLaunch::Bundled {
        name: GEMINI_BROWSER_SIDECAR_NAME.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_launch_mode_prefers_bundled_when_forced() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Debug,
            false,
            true,
            Path::new("G:/Develop/Extractum"),
            true,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::Bundled {
                name: "gemini-browser-sidecar".to_string()
            }
        );
    }

    #[test]
    fn resolve_launch_mode_keeps_dev_node_fallback_for_debug_repo_runs() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Debug,
            false,
            false,
            Path::new("G:/Develop/Extractum"),
            true,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::DevNodeScript {
                node: "node".to_string(),
                script: PathBuf::from(
                    "G:/Develop/Extractum/sidecars/gemini-browser/dist/index.js"
                )
            }
        );
    }

    #[test]
    fn resolve_launch_mode_uses_bundled_by_default_for_release_even_when_repo_dist_exists() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Release,
            false,
            false,
            Path::new("G:/Develop/Extractum"),
            true,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::Bundled {
                name: "gemini-browser-sidecar".to_string()
            }
        );
    }

    #[test]
    fn resolve_launch_mode_allows_explicit_dev_sidecar_override_in_release() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Release,
            true,
            false,
            Path::new("G:/Develop/Extractum"),
            true,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::DevNodeScript {
                node: "node".to_string(),
                script: PathBuf::from(
                    "G:/Develop/Extractum/sidecars/gemini-browser/dist/index.js"
                )
            }
        );
    }

    #[test]
    fn resolve_launch_mode_falls_back_to_bundled_when_debug_dev_script_is_absent() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Debug,
            false,
            false,
            Path::new("G:/Develop/Extractum"),
            false,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::Bundled {
                name: "gemini-browser-sidecar".to_string()
            }
        );
    }
}
```

- [x] **Step 2: Wire the module**

Modify `src-tauri/src/gemini_browser/mod.rs`:

```rust
mod commands;
mod paths;
mod run_log;
mod sidecar;
mod sidecar_launch;
mod state;
mod types;
```

Keep existing exports unchanged.

- [x] **Step 3: Run the focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser::sidecar_launch
```

Expected: all `sidecar_launch` tests pass.

- [x] **Step 4: Commit**

Update this task's checkboxes to `[x]`.

Run:

```powershell
git add src-tauri/src/gemini_browser/sidecar_launch.rs src-tauri/src/gemini_browser/mod.rs docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "feat: add Gemini sidecar launch resolution"
```

Expected: commit includes the new launch resolver and plan checkbox update.

---

## Task 4: Add Tauri Shell Plugin

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Add the shell plugin dependency**

Modify `src-tauri/Cargo.toml` dependencies:

```toml
tauri-plugin-shell = "2"
```

Expected: the dependency sits with the other Tauri plugins.

- [x] **Step 2: Register the shell plugin**

Modify `src-tauri/src/lib.rs` in the Tauri builder chain:

```rust
.plugin(tauri_plugin_shell::init())
```

Expected: it is registered next to the existing Tauri plugins.

- [x] **Step 3: Run dev-path checks**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser
```

Expected: both commands pass. No runtime launch behavior changes in this task.

- [x] **Step 4: Commit**

Update this task's checkboxes to `[x]`.

Run:

```powershell
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "feat: add Tauri shell support for Gemini sidecar"
```

Expected: commit includes only Tauri shell plugin setup and plan checkbox update.

---

## Task 5: Implement Shell Sidecar Transport

**Files:**
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Test: `src-tauri/src/gemini_browser/sidecar.rs`

- [x] **Step 1: Replace direct repo-cwd spawning with release-safe launch dispatch**

Modify the imports in `src-tauri/src/gemini_browser/sidecar.rs`:

```rust
use std::process::Stdio;

use serde::Deserialize;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
```

Add the launch helper import:

```rust
use super::sidecar_launch::{
    resolve_launch_mode, GeminiBrowserBuildProfile, GeminiBrowserSidecarLaunch,
};
```

Replace `GeminiBrowserSidecarProcess::spawn()` with:

```rust
async fn spawn(handle: &AppHandle) -> AppResult<Self> {
    let repo_root = std::env::current_dir()
        .map_err(|error| AppError::internal(error.to_string()))?;
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
```

The bundled path is release default. The Node script path is allowed by default only in debug builds, or in release only when `EXTRACTUM_GEMINI_BROWSER_DEV_SIDECAR=1` is explicitly set.

- [x] **Step 2: Replace the `Child`-specific process wrapper with a transport enum**

Modify `src-tauri/src/gemini_browser/sidecar.rs` so `GeminiBrowserSidecarProcess` stores one of two transports:

```rust
enum GeminiBrowserSidecarTransport {
    Node {
        child: Child,
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
    },
    Shell {
        child: Option<tauri_plugin_shell::process::CommandChild>,
        rx: tauri::async_runtime::Receiver<tauri_plugin_shell::process::CommandEvent>,
        stdout_buffer: String,
    },
}

pub(crate) struct GeminiBrowserSidecarProcess {
    transport: GeminiBrowserSidecarTransport,
    next_id: u64,
}
```

If the exact receiver type differs in the installed `tauri-plugin-shell` version, use the compiler error to import the concrete channel receiver returned by `sidecar(...).spawn()`. Keep the enum shape, the `stdout_buffer`, and request behavior identical.

- [x] **Step 3: Keep Node request behavior unchanged**

Move the existing stdin/stdout write-read logic into:

```rust
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

    let mut response_line = String::new();
    let bytes = stdout.read_line(&mut response_line).await.map_err(|error| {
        AppError::internal(format!("Failed to read Gemini sidecar response: {error}"))
    })?;
    if bytes == 0 {
        return Err(AppError::internal(
            "Gemini browser sidecar exited without a response",
        ));
    }
    decode_sidecar_line(id, &response_line)
}
```

Add a shared decoder:

```rust
fn decode_sidecar_line(id: &str, response_line: &str) -> AppResult<GeminiBrowserSidecarResponse> {
    let response: SidecarLine = serde_json::from_str(response_line)
        .map_err(|error| AppError::internal(format!("Invalid Gemini sidecar response: {error}")))?;
    if response.id != id {
        return Err(AppError::internal(
            "Gemini browser sidecar response id mismatch",
        ));
    }
    Ok(response.response)
}
```

- [x] **Step 4: Add spawn helpers for both launch modes**

Add:

```rust
async fn spawn_node_script(node: String, script_path: std::path::PathBuf) -> AppResult<Self> {
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
    let command = handle
        .shell()
        .sidecar(name)
        .map_err(|error| AppError::internal(format!("Gemini sidecar bundle is unavailable: {error}")))?;
    let (rx, child) = command
        .spawn()
        .map_err(|error| AppError::internal(format!("Failed to start bundled Gemini sidecar: {error}")))?;

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
            child,
            stdin,
            stdout: BufReader::new(stdout),
        },
        next_id: 1,
    })
}
```

- [x] **Step 5: Add shell request behavior**

Add:

```rust
async fn request_shell(
    child: &mut tauri_plugin_shell::process::CommandChild,
    rx: &mut tauri::async_runtime::Receiver<tauri_plugin_shell::process::CommandEvent>,
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
        AppError::internal(format!("Failed to write bundled Gemini sidecar request: {error}"))
    })?;

    while let Some(event) = rx.recv().await {
        if let tauri_plugin_shell::process::CommandEvent::Stdout(bytes) = event {
            stdout_buffer.push_str(&String::from_utf8_lossy(&bytes));
            while let Some(line) = take_complete_jsonl_line(stdout_buffer) {
                return decode_sidecar_line(id, &line);
            }
        }
        if let tauri_plugin_shell::process::CommandEvent::Stderr(bytes) = event {
            let line = String::from_utf8_lossy(&bytes);
            if line.contains("Error") || line.contains("error") {
                continue;
            }
        }
    }

    Err(AppError::internal(
        "Bundled Gemini browser sidecar exited without a response",
    ))
}
```

- [x] **Step 6: Update `request`, caller, and `Drop`**

Update `request`:

```rust
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
```

Modify `request_sidecar`:

```rust
*sidecar = Some(GeminiBrowserSidecarProcess::spawn(handle).await?);
```

Update `Drop`:

```rust
match &mut self.transport {
    GeminiBrowserSidecarTransport::Node { child, .. } => {
        let _ = child.start_kill();
    }
    GeminiBrowserSidecarTransport::Shell { child, .. } => {
        if let Some(child) = child.take() {
            let _ = child.kill();
        }
    }
}
```

- [x] **Step 7: Add decoder tests**

Add tests in `src-tauri/src/gemini_browser/sidecar.rs`:

```rust
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
    fn take_complete_jsonl_lines_handles_partial_and_multiple_chunks() {
        let mut buffer = String::new();
        buffer.push_str("{\"id\":\"one\"");
        assert!(take_complete_jsonl_line(&mut buffer).is_none());

        buffer.push_str(",\"response\":{\"type\":\"ack\"}}\n\n{\"id\":\"two\",\"response\":{\"type\":\"ack\"}}\n");

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
}
```

Add the helper used by the test and `request_shell`:

```rust
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
```

Then update `request_shell` to call `take_complete_jsonl_line(stdout_buffer)` after appending stdout bytes.

If the installed `tauri-plugin-shell` receiver type can be constructed in tests, add one more test that feeds stdout events in two chunks:

```rust
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
```

If the receiver type is not practical to instantiate, keep the helper-level partial/multiple-line coverage above and note that `request_shell` uses the helper directly.

- [x] **Step 8: Run Rust checks**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser
```

Expected: all Gemini browser Rust tests pass.

- [x] **Step 9: Commit**

Update this task's checkboxes to `[x]`.

Run:

```powershell
git add src-tauri/src/gemini_browser/sidecar.rs docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "feat: add bundled Gemini sidecar transport"
```

Expected: commit includes shell transport support and plan checkbox update.

---

## Task 6: Add Sidecar Binary Build Scripts

**Files:**
- Create: `scripts/build-gemini-browser-sidecar.mjs`
- Create: `scripts/check-gemini-browser-sidecar-binary.mjs`
- Modify: `package.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `.gitignore`

- [x] **Step 1: Add the build script**

Create `scripts/build-gemini-browser-sidecar.mjs`:

```js
import { existsSync, mkdirSync, renameSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = process.cwd();
const sidecarEntry = path.join(repoRoot, "sidecars", "gemini-browser", "src", "index.ts");
const sidecarDist = path.join(repoRoot, "sidecars", "gemini-browser", "dist", "index.js");
const binariesDir = path.join(repoRoot, "src-tauri", "binaries");
const packageWorkDir = path.join(repoRoot, "artifacts", "gemini-browser-sidecar-package");
const bundleOutput = path.join(packageWorkDir, "index.cjs");
const pkgConfigPath = path.join(packageWorkDir, "pkg.config.json");
const extension = process.platform === "win32" ? ".exe" : "";
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
const npxCommand = process.platform === "win32" ? "npx.cmd" : "npx";

function run(label, command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    throw new Error(`${label} failed with exit code ${result.status}`);
  }
}

function output(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed: ${result.stderr}`);
  }
  return result.stdout.trim();
}

function pkgTarget() {
  const platform =
    process.platform === "win32"
      ? "win"
      : process.platform === "darwin"
        ? "macos"
        : process.platform === "linux"
          ? "linux"
          : null;
  const arch =
    process.arch === "x64"
      ? "x64"
      : process.arch === "arm64"
        ? "arm64"
        : null;

  if (!platform || !arch) {
    throw new Error(`Unsupported pkg target platform: ${process.platform}/${process.arch}`);
  }

  return `node18-${platform}-${arch}`;
}

run("sidecar TypeScript build", npmCommand, ["run", "test:gemini-browser-sidecar:build"]);

if (!existsSync(sidecarDist)) {
  throw new Error(`Missing sidecar dist entry: ${sidecarDist}`);
}

const targetTriple = output("rustc", ["--print", "host-tuple"]);
if (!targetTriple) {
  throw new Error("rustc did not return a host tuple");
}
const requestedTarget =
  process.env.GEMINI_BROWSER_SIDECAR_TARGET ?? process.env.CARGO_BUILD_TARGET ?? "";
if (requestedTarget && requestedTarget !== targetTriple) {
  throw new Error(
    `Gemini browser sidecar packaging is host-target only in v1. ` +
      `Requested ${requestedTarget}, host is ${targetTriple}.`,
  );
}

mkdirSync(binariesDir, { recursive: true });
mkdirSync(packageWorkDir, { recursive: true });

const rawOutput = path.join(binariesDir, `gemini-browser-sidecar${extension}`);
const tauriOutput = path.join(
  binariesDir,
  `gemini-browser-sidecar-${targetTriple}${extension}`,
);
const browsersJsonAsset = path.relative(
  packageWorkDir,
  path.join(repoRoot, "node_modules", "playwright-core", "browsers.json"),
);

rmSync(rawOutput, { force: true });
rmSync(tauriOutput, { force: true });
rmSync(bundleOutput, { force: true });
writeFileSync(
  pkgConfigPath,
  JSON.stringify(
    {
      pkg: {
        assets: [browsersJsonAsset.replace(/\\/g, "/")],
      },
    },
    null,
    2,
  ),
);

run("sidecar CommonJS bundle", npxCommand, [
  "esbuild",
  sidecarEntry,
  "--bundle",
  "--platform=node",
  "--format=cjs",
  "--packages=external",
  `--outfile=${bundleOutput}`,
]);

run("Node sidecar binary packaging", npxCommand, [
  "pkg",
  bundleOutput,
  "--config",
  pkgConfigPath,
  "--targets",
  pkgTarget(),
  "--no-bytecode",
  "--public",
  "--public-packages",
  "*",
  "--output",
  rawOutput,
]);

if (!existsSync(rawOutput)) {
  throw new Error(`Sidecar packager did not create ${rawOutput}`);
}

renameSync(rawOutput, tauriOutput);
console.log(`Wrote ${path.relative(repoRoot, tauriOutput)}`);
```

This script uses the Tauri target-triple naming pattern for external binaries. It intentionally supports host-target builds only in v1 and fails fast if `GEMINI_BROWSER_SIDECAR_TARGET` or `CARGO_BUILD_TARGET` requests a different target.

- [x] **Step 2: Add the binary check script**

Create `scripts/check-gemini-browser-sidecar-binary.mjs`:

```js
import { existsSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = process.cwd();
const extension = process.platform === "win32" ? ".exe" : "";
const result = spawnSync("rustc", ["--print", "host-tuple"], {
  cwd: repoRoot,
  encoding: "utf8",
  shell: process.platform === "win32",
});

if (result.status !== 0) {
  console.error(result.stderr);
  process.exit(result.status ?? 1);
}

const targetTriple = result.stdout.trim();
const requestedTarget =
  process.env.GEMINI_BROWSER_SIDECAR_TARGET ?? process.env.CARGO_BUILD_TARGET ?? "";
if (requestedTarget && requestedTarget !== targetTriple) {
  console.error(
    `Gemini browser sidecar packaging is host-target only in v1. ` +
      `Requested ${requestedTarget}, host is ${targetTriple}.`,
  );
  process.exit(1);
}
const expectedPath = path.join(
  repoRoot,
  "src-tauri",
  "binaries",
  `gemini-browser-sidecar-${targetTriple}${extension}`,
);

if (!existsSync(expectedPath)) {
  console.error(`Missing Gemini browser sidecar binary: ${expectedPath}`);
  console.error("Run: npm.cmd run build:gemini-browser-sidecar");
  process.exit(1);
}

console.log(`Found ${path.relative(repoRoot, expectedPath)}`);
```

- [x] **Step 3: Add package scripts**

Modify root `package.json` scripts:

```json
"build:gemini-browser-sidecar": "node scripts/build-gemini-browser-sidecar.mjs",
"check:gemini-browser-sidecar-binary": "node scripts/check-gemini-browser-sidecar-binary.mjs",
"build:tauri-prereqs": "npm run build && npm run build:gemini-browser-sidecar && npm run check:gemini-browser-sidecar-binary"
```

Keep existing sidecar test scripts unchanged.

- [x] **Step 4: Enforce sidecar packaging in Tauri build**

Modify `src-tauri/tauri.conf.json`:

```json
"build": {
  "beforeDevCommand": "npm run dev",
  "devUrl": "http://localhost:1420",
  "beforeBuildCommand": "npm run build:tauri-prereqs",
  "frontendDist": "../build"
},
"bundle": {
  "active": true,
  "targets": "all",
  "externalBin": [
    "binaries/gemini-browser-sidecar"
  ],
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ]
}
```

Expected:

- normal `npm.cmd run build` still builds only the Svelte frontend;
- `npm.cmd run tauri build` runs `npm run build:tauri-prereqs` through Tauri's `beforeBuildCommand`;
- missing sidecar binaries fail with the explicit `check:gemini-browser-sidecar-binary` message before bundling;
- stale sidecar binaries are not detected in place; `build:tauri-prereqs` eliminates staleness by rebuilding the sidecar binary before checking it exists.
- `GEMINI_BROWSER_SIDECAR_TARGET` or `CARGO_BUILD_TARGET` values different from the host tuple fail with the explicit host-target-only message.

- [x] **Step 5: Ignore generated sidecar binaries**

Modify `.gitignore`:

```gitignore
src-tauri/binaries/gemini-browser-sidecar-*
```

Do not ignore `src-tauri/binaries/.gitkeep` if a later task adds one.

- [x] **Step 6: Confirm the sidecar packager dependencies**

Run:

```powershell
npm.cmd ls pkg esbuild
```

Expected: `pkg` and `esbuild` are installed from Task 2 and listed as dev dependencies.

- [x] **Step 7: Run script checks**

Run:

```powershell
npm.cmd run build:gemini-browser-sidecar
npm.cmd run check:gemini-browser-sidecar-binary
npm.cmd run build:tauri-prereqs
git status --short --untracked-files=all src-tauri\\binaries
```

Expected:

- build script writes `src-tauri/binaries/gemini-browser-sidecar-<target-triple>.exe` on Windows or no extension on Unix;
- check script finds it;
- `build:tauri-prereqs` runs the frontend build plus sidecar build/check without invoking the full Tauri bundle;
- `git status` does not show the generated binary as tracked or untracked noise.

- [x] **Step 8: Commit scripts, build enforcement, and metadata**

Update this task's checkboxes to `[x]`.

Run:

```powershell
git add scripts/build-gemini-browser-sidecar.mjs scripts/check-gemini-browser-sidecar-binary.mjs package.json package-lock.json src-tauri/tauri.conf.json .gitignore docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "build: add Gemini sidecar binary packaging scripts"
```

Expected: commit includes scripts, package metadata, Tauri build enforcement, ignore rule, and plan checkbox update. The generated sidecar binary remains untracked.

---

## Task 7: Add Packaged Launch Smoke

**Files:**
- Modify: `sidecars/gemini-browser/src/index.ts`
- Create: `scripts/gemini-browser-sidecar-smoke.mjs`
- Modify: `package.json`

- [ ] **Step 1: Add a Playwright launch smoke mode to the sidecar**

Modify the top of `sidecars/gemini-browser/src/index.ts` before creating the JSON-line `readline` interface:

```ts
if (process.argv.includes("--playwright-smoke")) {
  const { chromium } = await import("@playwright/test");
  const profileDirArg = process.argv.find((arg) => arg.startsWith("--profile-dir="));
  const profileDir =
    profileDirArg?.slice("--profile-dir=".length) ?? "artifacts/gemini-browser-playwright-smoke-profile";
  const context = await chromium.launchPersistentContext(profileDir, {
    headless: true,
    viewport: { width: 800, height: 600 },
  });
  const page = context.pages()[0] ?? (await context.newPage());
  await page.goto("data:text/html,<title>Gemini Sidecar Smoke</title><main>ok</main>");
  const title = await page.title();
  await context.close();
  process.stdout.write(`${JSON.stringify({ ok: true, title })}\n`);
  process.exit(0);
}
```

Expected: this mode exercises the packaged Playwright import and Chromium launch path without navigating to Gemini or automating any Google account surface.

This is an internal diagnostic sidecar mode for packaging verification only. It is not a user-facing Gemini Browser Provider command and should not be exposed through Tauri commands or UI.

- [ ] **Step 2: Add a JSON-line and Playwright sidecar smoke script**

Create `scripts/gemini-browser-sidecar-smoke.mjs`:

```js
import { spawn } from "node:child_process";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = process.cwd();
const mode = process.argv.includes("--binary") ? "binary" : "node";
const playwrightSmoke = process.argv.includes("--playwright");

function hostTuple() {
  const result = spawnSync("rustc", ["--print", "host-tuple"], {
    cwd: repoRoot,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    throw new Error(result.stderr);
  }
  return result.stdout.trim();
}

const extension = process.platform === "win32" ? ".exe" : "";
const command =
  mode === "binary"
    ? path.join(
        repoRoot,
        "src-tauri",
        "binaries",
        `gemini-browser-sidecar-${hostTuple()}${extension}`,
      )
    : process.execPath;
const args =
  mode === "binary"
    ? []
    : [path.join(repoRoot, "sidecars", "gemini-browser", "dist", "index.js")];

if (playwrightSmoke) {
  const profileDir = path.join(repoRoot, "artifacts", `gemini-browser-playwright-smoke-${mode}`);
  args.push("--playwright-smoke", `--profile-dir=${profileDir}`);
}

const child = spawn(command, args, {
  cwd: repoRoot,
  stdio: ["pipe", "pipe", "pipe"],
});

if (playwrightSmoke) {
  let stdout = "";
  let stderr = "";
  const timeout = setTimeout(() => {
    child.kill();
    console.error("Timed out waiting for Playwright smoke response");
    process.exit(1);
  }, 15000);

  child.stdout.on("data", (chunk) => {
    stdout += chunk.toString();
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });
  child.on("exit", (code) => {
    clearTimeout(timeout);
    if (code !== 0) {
      console.error(stderr);
      process.exit(code ?? 1);
    }
    const line = stdout.split(/\r?\n/).find((entry) => entry.trim().length > 0);
    const parsed = line ? JSON.parse(line) : null;
    if (!parsed?.ok || parsed.title !== "Gemini Sidecar Smoke") {
      console.error(`Unexpected Playwright smoke output: ${stdout}`);
      process.exit(1);
    }
    console.log(line);
  });
} else {
  const request = {
    id: "smoke-1",
    command: {
      type: "status",
      browser_profile_dir: path.join(repoRoot, "artifacts", "gemini-browser-smoke-profile"),
    },
  };

  let stdout = "";
  let stderr = "";
  const timeout = setTimeout(() => {
    child.kill();
    console.error("Timed out waiting for sidecar status response");
    process.exit(1);
  }, 5000);

  child.stdout.on("data", (chunk) => {
    stdout += chunk.toString();
    const line = stdout.split(/\r?\n/).find((entry) => entry.trim().length > 0);
    if (!line) return;
    clearTimeout(timeout);
    child.kill();
    const parsed = JSON.parse(line);
    if (parsed.id !== "smoke-1") {
      console.error(`Unexpected response id: ${line}`);
      process.exit(1);
    }
    if (parsed.response?.type !== "status") {
      console.error(`Unexpected response type: ${line}`);
      process.exit(1);
    }
    console.log(line);
  });

  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });

  child.on("exit", (code) => {
    if (!stdout.trim()) {
      clearTimeout(timeout);
      console.error(stderr);
      process.exit(code ?? 1);
    }
  });

  child.stdin.write(`${JSON.stringify(request)}\n`);
}
```

- [ ] **Step 3: Add smoke scripts**

Modify `package.json` scripts:

```json
"smoke:gemini-browser-sidecar:node": "node scripts/gemini-browser-sidecar-smoke.mjs",
"smoke:gemini-browser-sidecar:binary": "node scripts/gemini-browser-sidecar-smoke.mjs --binary",
"smoke:gemini-browser-sidecar:playwright:node": "node scripts/gemini-browser-sidecar-smoke.mjs --playwright",
"smoke:gemini-browser-sidecar:playwright:binary": "node scripts/gemini-browser-sidecar-smoke.mjs --binary --playwright"
```

- [ ] **Step 4: Run sidecar status smoke in Node mode**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:build
npm.cmd run smoke:gemini-browser-sidecar:node
```

Expected: stdout contains one JSON response with `id: "smoke-1"` and `response.type: "status"`.

- [ ] **Step 5: Run sidecar status smoke in binary mode**

Run:

```powershell
npm.cmd run build:gemini-browser-sidecar
npm.cmd run smoke:gemini-browser-sidecar:binary
```

Expected: stdout contains one JSON response with `id: "smoke-1"` and `response.type: "status"`.

- [ ] **Step 6: Run Playwright launch smoke in Node and binary modes**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:build
npm.cmd run smoke:gemini-browser-sidecar:playwright:node
npm.cmd run build:gemini-browser-sidecar
npm.cmd run smoke:gemini-browser-sidecar:playwright:binary
```

Expected: both Playwright smoke commands return one JSON response with `ok: true` and `title: "Gemini Sidecar Smoke"`.

If this fails because Playwright browser binaries are not installed on the test machine, run:

```powershell
npx.cmd playwright install chromium
```

Then re-run the smoke commands. Do not treat missing browser binaries as a reason to skip packaged sidecar launch verification.

- [ ] **Step 7: Commit**

Update this task's checkboxes to `[x]`.

Run:

```powershell
git add sidecars/gemini-browser/src/index.ts scripts/gemini-browser-sidecar-smoke.mjs package.json docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "test: add Gemini sidecar launch smoke"
```

Expected: commit includes the sidecar Playwright smoke mode, smoke script, package scripts, and plan checkbox update.

---

## Task 8: Wire Bundle Verification Into Build Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`
- Modify: `research/gemini_browser_adapter/DECISION.md`

- [ ] **Step 1: Document release sidecar build order**

Append to `README.md`:

```markdown
## Gemini Browser Sidecar Packaging

The Gemini Browser Provider uses a TypeScript/Playwright sidecar. Development
runs can launch `sidecars/gemini-browser/dist/index.js` through local Node.
Release builds must first create the Tauri external sidecar binary:

```powershell
npm.cmd run test:gemini-browser-sidecar
npm.cmd run tauri build
```

`tauri build` runs `npm run build:tauri-prereqs` through
`src-tauri/tauri.conf.json > build.beforeBuildCommand`, which builds the
frontend, packages `gemini-browser-sidecar-<target-triple>[.exe]`, and checks
that the expected binary exists before Tauri starts bundling.

This v1 packaging flow is host-target only. Do not use it for
`tauri build --target ...` until cross-target sidecar binary generation is added.

Generated binaries under `src-tauri/binaries/gemini-browser-sidecar-*` are
local build artifacts and are not committed. Browser profile data and Gemini
run artifacts are app-data runtime files, not bundle resources. Use
`npm.cmd run smoke:gemini-browser-sidecar:playwright:binary` to verify the
packaged binary can import Playwright and launch Chromium without navigating to
Gemini.
```
```

- [ ] **Step 2: Add packaging note to the product spec**

Append under `## MVP Implementation Notes - 2026-06-20` in `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`:

```markdown
- Release packaging uses a Tauri external sidecar binary named
  `gemini-browser-sidecar`. Development runs may still use the local Node
  script fallback, but packaged app builds run `build:tauri-prereqs` through
  Tauri's `beforeBuildCommand`, including the sidecar binary check before
  bundling.
```

- [ ] **Step 3: Add packaging note to the research decision**

Append under `## Production Handoff - 2026-06-20` in `research/gemini_browser_adapter/DECISION.md`:

```markdown
- Sidecar packaging follow-up:
  `docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md`.
```

- [ ] **Step 4: Commit docs**

Update this task's checkboxes to `[x]`.

Run:

```powershell
git add README.md docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md research/gemini_browser_adapter/DECISION.md docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "docs: document Gemini sidecar packaging flow"
```

Expected: commit includes documentation updates and plan checkbox update.

---

## Task 9: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md`

- [ ] **Step 1: Run sidecar verification**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
```

Expected: sidecar typecheck, unit tests, and build pass.

- [ ] **Step 2: Run sidecar launch smokes**

Run:

```powershell
npm.cmd run smoke:gemini-browser-sidecar:node
npm.cmd run build:gemini-browser-sidecar
npm.cmd run check:gemini-browser-sidecar-binary
npm.cmd run smoke:gemini-browser-sidecar:binary
npm.cmd run smoke:gemini-browser-sidecar:playwright:node
npm.cmd run smoke:gemini-browser-sidecar:playwright:binary
npm.cmd run build:tauri-prereqs
```

Expected: status smoke commands return one valid `status` response, Playwright smoke commands return `ok: true`, and `build:tauri-prereqs` completes before any full Tauri bundle attempt.

- [ ] **Step 3: Run Rust checks**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser
```

Expected: all Gemini browser Rust tests pass.

- [ ] **Step 4: Run frontend checks touched by provider UI**

Run:

```powershell
npm.cmd run test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
```

Expected: targeted Vitest tests pass and Svelte check exits `0`.

- [ ] **Step 5: Run full Tauri bundle verification**

Run:

```powershell
npm.cmd run tauri build
```

Expected: the full Tauri build exits `0`. This is the expensive final check that verifies `bundle.externalBin`, platform-suffixed binary naming, and Tauri's `beforeBuildCommand` integration together.

If the build fails because a local platform installer/signing tool is unavailable, record the exact error in this plan and do not mark the packaging slice complete. A successful `build:tauri-prereqs` is not a substitute for this full bundle verification.

- [ ] **Step 6: Smoke packaged app sidecar resolution**

Run the packaged app with bundled sidecar mode forced:

```powershell
$env:EXTRACTUM_GEMINI_BROWSER_BUNDLED_SIDECAR = '1'
Start-Process -FilePath 'src-tauri\target\release\extractum.exe'
```

In the app:

1. Open `Settings -> Browser Providers`.
2. Click `Check Status`.
3. Confirm the panel reports a sidecar-origin status message such as `Browser has not been opened.` or `Browser page is available.`

Expected: the panel must not show the Rust fallback message `Gemini browser sidecar is not running.` This verifies that `handle.shell().sidecar("gemini-browser-sidecar")` resolves the external binary configured as `binaries/gemini-browser-sidecar` and that the packaged sidecar answers the JSON-line `status` request.

Do not click `Open Gemini` in this smoke unless separately performing a manual live-browser check. This sidecar resolution smoke must not navigate to Gemini or touch Google account state.

- [ ] **Step 7: Confirm generated binaries are ignored**

Run:

```powershell
git status --short --untracked-files=all src-tauri\\binaries
```

Expected: no generated `gemini-browser-sidecar-*` binary appears in git status.

- [ ] **Step 8: Commit final verification note**

Update this task's checkboxes to `[x]`.

Append this section to the end of this plan:

```markdown
## Verification Notes

- Sidecar tests: passed
- Node sidecar smoke: passed
- Binary sidecar smoke: passed
- Node Playwright sidecar smoke: passed
- Binary Playwright sidecar smoke: passed
- Tauri build prerequisite enforcement: passed
- Full Tauri bundle build: passed
- Packaged app sidecar resolution smoke: passed
- Rust Gemini browser tests: passed
- Frontend provider tests/check: passed
- Generated sidecar binaries: ignored
```

Run:

```powershell
git add docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md
git commit -m "docs: record Gemini sidecar packaging verification"
```

Expected: final commit contains only plan checkbox and verification-note updates.

---

## Self-Review

**Spec coverage:** This plan covers the release follow-up explicitly left outside the MVP: bundled sidecar configuration, launch path, generated binary naming, smoke tests, docs, and verification. It does not change the selected resilient adapter or research matrix.

**Security boundary:** Browser profiles, cookies, Google auth state, prompts, live DOM, screenshots, telemetry, and run logs remain runtime/app-data artifacts. The only packaged artifact is the sidecar executable.

**Known risk:** Packaging a Playwright-powered Node sidecar into a single binary can expose tool-specific limitations. Task 2 makes the packaging tool an explicit feasibility gate before any Rust/Tauri integration changes, and Task 6 turns the proven command into reusable build/check scripts. The v1 build scripts are host-target only; cross-target packaging needs a later release-matrix slice.

**Out of scope:** This plan does not add real Gemini automated smoke tests, does not bundle a Chromium browser, does not automate Google account flows, and does not move run logs into SQLite.
