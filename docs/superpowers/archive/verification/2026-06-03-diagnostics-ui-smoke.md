# Diagnostics UI Smoke Verification

> Date: 2026-06-03
> Scope: historical live smoke for the shipped read-only `/diagnostics` route.

## Environment

- Running Tauri app with frontend at `http://localhost:1420`.
- Tauri MCP bridge connected at `localhost:9223`.
- Smoke screenshots saved as ignored local artifacts:
  - `artifacts/diagnostics-smoke.png`
  - `artifacts/diagnostics-smoke-after-refresh.png`

## Checked

- Sidebar navigation opened `/diagnostics` and the app topbar label showed
  `Diagnostics`.
- Page rendered the operator diagnostics hero, status strip, App/build,
  Database, Runtimes, Privacy boundary, and all aggregate table sections.
- Empty count sections rendered quiet empty rows instead of disappearing.
- Manual `Refresh` kept the summary visible and updated the generated timestamp.
- Privacy boundary stayed visible.
- Page body did not show `Raw JSON`, copy controls, stack traces, or raw
  command payloads.
- Browser console showed only bridge/Vite messages during the smoke.
- Document had no horizontal overflow in the checked viewport.

## Caveat

The Tauri IPC monitor did not capture the frontend `get_diagnostic_summary`
invoke directly in this run. Evidence for refresh behavior came from the UI
timestamp changing after manual refresh, the loaded diagnostics content, and the
source-contract tests that require the route to call the API wrapper rather than
`invoke` directly.

## Verification Gate

Fresh full-project verification was run after the diagnostics slice:

- frontend Vitest: 67 files, 637 tests passed;
- `svelte-check`: 0 errors, 0 warnings;
- Rust tests: 663 passed;
- `git diff HEAD --check`: clean.
