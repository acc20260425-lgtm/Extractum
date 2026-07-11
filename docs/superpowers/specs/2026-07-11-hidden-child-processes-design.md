# Hidden Child Processes on Windows Design

**Date:** 2026-07-11  
**Status:** revised after design review; awaiting renewed approval

## Problem

Opening Analysis and Diagnostics launches `yt-dlp --version`. In the Windows
GUI build, both `tokio::process::Command` instances inherit the default console
creation behavior, so a terminal window flashes while the short-lived child
process runs.

## Design

Create `src-tauri/src/child_process.rs` with:

```rust
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) fn hide_console_window(
    command: &mut tokio::process::Command,
) -> &mut tokio::process::Command;
```

The numeric value is independently defined by Microsoft's Win32
[`PROCESS_CREATION_FLAGS`](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags)
reference. The unit test guards against accidental later edits to the project
constant; it is not the authority for the Win32 value.

The constant is declared unconditionally so its unit test runs on every host;
on non-Windows production builds it is annotated with `#[allow(dead_code)]`.
The `creation_flags` call itself is inside `#[cfg(windows)]`, because that Tokio
API does not exist on other targets. On Windows the helper calls
`Command::creation_flags(CREATE_NO_WINDOW)`. Tokio delegates process creation
to `std`, which combines the supplied flags with `CREATE_UNICODE_ENVIRONMENT`.
On non-Windows targets the helper returns the command unchanged.

`creation_flags` replaces the caller-supplied flag set rather than merging with
an earlier call. Therefore `hide_console_window` must be the only code that
sets creation flags for commands passed to it, and call sites must not invoke
`creation_flags` before or after the helper.

Use the helper at all three confirmed `yt-dlp` launch paths:

- `youtube/runtime.rs`: Analysis runtime probe;
- `diagnostics/runtime.rs`: Diagnostics runtime probe;
- `youtube/ytdlp.rs`: actual metadata, captions, comments, and download work.

The third call site is included because it launches the same console binary
through the same Tokio API; otherwise a console could remain visible for the
whole download rather than only flash during a version probe.

The helper changes only window creation. Arguments, stdout/stderr capture,
timeouts, exit-status handling, and user-visible diagnostic results remain
unchanged. Broader child-process migration is out of scope until each launcher
is reproduced and verified separately. Specifically:

- `gemini_browser/sidecar.rs` Node launch is development-only in its normal
  mode, where the parent terminal already exists; bundled release launch uses
  the Tauri shell sidecar path and needs its own verification before changes;
- `gemini_browser/cdp_chrome.rs` launches Chrome, a GUI application that does
  not create a console window, and must not receive this flag.

The duplicated `yt-dlp --version` parsing in the two probe modules is a future
candidate for a shared `probe_ytdlp_version()` function. Consolidating result
DTOs and error semantics is not required for this focused window-creation fix.

## Verification

- A source contract fails before implementation and requires the shared helper
  plus all three `yt-dlp` call sites.
- A unit test asserts `CREATE_NO_WINDOW == 0x0800_0000`; this guards against an
  accidental later edit. The Microsoft Win32 reference remains the independent
  source of truth for the value.
- Rust tests for YouTube and diagnostics runtime behavior continue to pass.
- `cargo check` passes.
- In a release or CSP-verification build, navigating to Analysis and
  Diagnostics and starting a YouTube operation no longer flashes or holds open
  console windows. This must be tested in a release GUI build: the development
  workflow already has a parent terminal and can produce a false pass.
