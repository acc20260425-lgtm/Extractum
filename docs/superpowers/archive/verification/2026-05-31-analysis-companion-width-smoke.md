# Analysis Companion Width Verification

> Date: 2026-05-31
> Branch: `main`

## Commands

```powershell
npm.cmd run verify
npm.cmd run smoke:analysis
```

## Result

The merged `main` verification passed after the Analysis Companion Width slice.

The accepted `npm.cmd run verify` run included:

- Vitest: `64 passed (64)`, `612 passed (612)`;
- `svelte-check found 0 errors and 0 warnings`;
- Rust tests: `621 passed; 0 failed`;
- `git diff HEAD --check`;
- `All verification checks passed.`

The accepted `npm.cmd run smoke:analysis` run passed outside the filesystem
sandbox and included:

- `PASS source-browser.telegram-live-tabs`;
- `PASS source-browser.youtube-video-tabs`;
- `PASS source-browser.youtube-playlist-tabs`;
- `PASS source-browser.live-source-group-tabs`;
- `PASS source-browser.run-snapshot-tabs`;
- `PASS saved-runs-affordance.rows`;
- `PASS saved-runs-affordance.missing-legacy`;
- `PASS saved-runs-affordance.capture-failed`;
- `PASS workspace-parity.single-source-setup-tools`;
- `PASS workspace-parity.source-group-disabled-export`;
- `PASS workspace-parity.opened-single-run-tools`;
- `PASS workspace-parity.opened-source-group-run-disabled-export`;
- `PASS workspace-parity.source-mode-tools-placement`;
- `Analysis UI smoke passed.`

## Layout Evidence

Tauri MCP viewport measurement on the widened desktop layout showed:

- `workspaceGrid`: `76px 975.195px 560px`;
- `companionSlot.width`: `560`;
- `companionPanel.width`: `558`;
- Evidence `.run-evidence-tab.width`: `529`;
- `.trace-layout.width`: `495`;
- `.traceLayoutGrid`: `216.719px 264.891px`.

Chrome DevTools viewport measurements showed:

- at `1440x900`, the existing `@media (max-width: 1500px)` breakpoint stacks
  the companion below the canvas with grid `76px 1103.6px`;
- at `1600x900`, the desktop grid uses `76px 769.203px 480px`, keeping the
  canvas wider than the companion.

## Sandbox Caveat

The first smoke attempt inside the restricted sandbox failed before the MCP
bridge became available:

- Tauri app startup panicked with `unable to open database file`;
- cleanup hit `Get-CimInstance ... Access denied`;
- the smoke runner then reported no `org.ai.extractum` MCP bridge.

The same command passed when rerun with elevated GUI/filesystem permissions.
Treat this as an environment restriction, not an application smoke failure.
