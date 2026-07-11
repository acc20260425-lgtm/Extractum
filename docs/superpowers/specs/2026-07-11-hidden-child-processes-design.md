# Hidden Child Processes on Windows Design

**Date:** 2026-07-11  
**Status:** approved for implementation planning

## Problem

Opening Analysis and Diagnostics launches `yt-dlp --version`. In the Windows
GUI build, both `tokio::process::Command` instances inherit the default console
creation behavior, so a terminal window flashes while the short-lived child
process runs.

## Design

Add one backend helper that applies Windows `CREATE_NO_WINDOW` (`0x08000000`) to
a `tokio::process::Command`. On non-Windows targets it returns the command
unchanged. Use the helper for the two confirmed runtime probes in
`youtube/runtime.rs` and `diagnostics/runtime.rs`.

The helper changes only window creation. Arguments, stdout/stderr capture,
timeouts, exit-status handling, and user-visible diagnostic results remain
unchanged. Broader child-process migration is out of scope until each launcher
is reproduced and verified separately.

## Verification

- A Windows source contract fails before implementation and requires the shared
  helper plus both call sites.
- Rust tests for YouTube and diagnostics runtime behavior continue to pass.
- `cargo check` passes.
- In a release or CSP-verification build, navigating to Analysis and
  Diagnostics no longer flashes console windows.
