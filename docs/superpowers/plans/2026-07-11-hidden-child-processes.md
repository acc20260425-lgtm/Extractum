# Hidden Child Processes on Windows Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop Windows console windows from appearing when Extractum launches `yt-dlp` from its GUI build.

**Architecture:** A focused `child_process` backend module owns the Windows-only `CREATE_NO_WINDOW` flag and applies it to Tokio commands. The two version probes and the real `yt-dlp` execution path use that helper; Node sidecar and Chrome launch behavior remain unchanged.

**Tech Stack:** Rust, Tokio `process::Command`, Tauri 2, Vitest source contracts, Windows Win32 process flags.

## Global Constraints

- `CREATE_NO_WINDOW` is exactly `0x0800_0000`, sourced from Microsoft Win32 `PROCESS_CREATION_FLAGS`.
- `hide_console_window` is the only code allowed to set creation flags on commands passed to it.
- `creation_flags` is called only under `#[cfg(windows)]`; non-Windows behavior is unchanged.
- Do not modify Gemini Node sidecar or Chrome launch paths.
- Preserve existing arguments, output capture, timeouts, errors, and DTO behavior.

---

### Task 1: Add the hidden-console command primitive

**Files:**
- Create: `src-tauri/src/child_process.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/lib/hidden-child-process-contract.test.ts`

**Interfaces:**
- Produces: `pub(crate) const CREATE_NO_WINDOW: u32` and `pub(crate) fn hide_console_window(&mut tokio::process::Command) -> &mut tokio::process::Command`.

- [ ] **Step 1: Write the failing source contract.** Require `mod child_process;`, the named constant, `#[cfg(windows)]`, `creation_flags(CREATE_NO_WINDOW)`, a non-Windows unchanged return path, and the unit assertion. Also assert that `gemini_browser/sidecar.rs` and `gemini_browser/cdp_chrome.rs` do not import this helper.

```ts
expect(libSource).toContain("mod child_process;");
expect(childProcessSource).toContain("pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;");
expect(childProcessSource).toMatch(/#\[cfg\(windows\)\][\s\S]*creation_flags\(CREATE_NO_WINDOW\)/);
expect(childProcessSource).toContain("assert_eq!(CREATE_NO_WINDOW, 0x0800_0000)");
```

- [ ] **Step 2: Run RED.**

Run: `npm.cmd run test -- src/lib/hidden-child-process-contract.test.ts`
Expected: FAIL with a Vite module-resolution error because the raw-imported
`src-tauri/src/child_process.rs` file does not exist yet. This is the intended
RED, not a test-infrastructure failure.

- [ ] **Step 3: Implement the minimal helper.**

```rust
use tokio::process::Command;

#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) fn hide_console_window(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(test)]
mod tests {
    use super::CREATE_NO_WINDOW;

    #[test]
    fn create_no_window_matches_win32_process_creation_flags() {
        assert_eq!(CREATE_NO_WINDOW, 0x0800_0000);
    }
}
```

Register `mod child_process;` in `src-tauri/src/lib.rs`.

- [ ] **Step 4: Run GREEN and cross-platform compilation checks.**

Run: `npm.cmd run test -- src/lib/hidden-child-process-contract.test.ts`
Run: `cargo test --manifest-path src-tauri/Cargo.toml child_process -- --nocapture`
Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS with no new warning.

- [ ] **Step 5: Commit.**

```powershell
git add src-tauri/src/child_process.rs src-tauri/src/lib.rs src/lib/hidden-child-process-contract.test.ts
git commit -m "feat: add hidden Windows child process helper"
```

### Task 2: Apply the helper to every yt-dlp launcher

**Files:**
- Modify: `src-tauri/src/youtube/runtime.rs`
- Modify: `src-tauri/src/diagnostics/runtime.rs`
- Modify: `src-tauri/src/youtube/ytdlp.rs`
- Modify: `src/lib/hidden-child-process-contract.test.ts`

**Interfaces:**
- Consumes: `crate::child_process::hide_console_window` from Task 1.

- [ ] **Step 1: Extend the failing contract** with `test.each` so each of the three files imports and calls `hide_console_window(&mut command)` before `.output()`, and none calls `creation_flags` directly. The string contract checks the import/call shape; the focused Rust tests and `cargo check` remain the authoritative compiler check that the import resolves and the borrow/type usage is valid.

```ts
test.each([
  ["youtube runtime", youtubeRuntimeSource],
  ["diagnostics runtime", diagnosticsRuntimeSource],
  ["yt-dlp execution", ytdlpSource],
])("%s hides the child console", (_name, source) => {
  expect(source).toContain("use crate::child_process::hide_console_window;");
  expect(source).toContain("hide_console_window(&mut command)");
  expect(source).not.toContain("creation_flags(");
});
```

- [ ] **Step 2: Run RED.**

Run: `npm.cmd run test -- src/lib/hidden-child-process-contract.test.ts`
Expected: three separately reported failing cases, one for each unmigrated
launcher. Partial migration leaves only its remaining cases red.

- [ ] **Step 3: Migrate the three launchers.** Build a mutable command, add the existing arguments, call `hide_console_window`, then preserve the existing `.output()` and timeout/error flow.

```rust
let mut command = Command::new("yt-dlp");
command.arg("--version");
hide_console_window(&mut command);
let output = tokio::time::timeout(YTDLP_RUNTIME_CHECK_TIMEOUT, command.output()).await;
```

- [ ] **Step 4: Run focused GREEN checks.**

Run: `npm.cmd run test -- src/lib/hidden-child-process-contract.test.ts`
Run: `cargo test --manifest-path src-tauri/Cargo.toml youtube::runtime -- --nocapture`
Run: `cargo test --manifest-path src-tauri/Cargo.toml diagnostics::runtime -- --nocapture`
Run: `cargo test --manifest-path src-tauri/Cargo.toml youtube::ytdlp -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Run broad Rust and Vitest verification and commit.**

Run: `npm.cmd run test`
Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Run: `git diff --check`
Expected: PASS with no new warnings or failures.

```powershell
git add src-tauri/src/youtube/runtime.rs src-tauri/src/diagnostics/runtime.rs src-tauri/src/youtube/ytdlp.rs src/lib/hidden-child-process-contract.test.ts
git commit -m "fix: hide yt-dlp console windows"
```

### Task 3: Verify the Windows GUI behavior

**Files:**
- Create: `docs/superpowers/verification/2026-07-11-hidden-child-processes.md`

**Interfaces:**
- Consumes: Tasks 1–2.
- Produces: manual release-build evidence.

- [ ] **Step 1: Build a GUI executable.**

Run: `npm.cmd run tauri build -- --no-bundle --features csp-verification`
Expected: PASS and produce the release executable with DevTools available.

- [ ] **Step 2: Launch the release executable and reproduce all three paths.** Navigate to Analysis, navigate to Diagnostics, and start a YouTube metadata/preview operation.

Expected: no console window flashes during either version probe and no console window appears or remains open during the real `yt-dlp` operation. Existing result/error behavior remains unchanged.

- [ ] **Step 3: Record commands, observations, and automated results.** State explicitly that the check used a release GUI build because dev has a parent terminal and can give a false pass.

- [ ] **Step 4: Commit verification evidence.**

```powershell
git add docs/superpowers/verification/2026-07-11-hidden-child-processes.md
git commit -m "docs: verify hidden Windows child processes"
```
