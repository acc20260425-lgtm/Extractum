# Analysis UI Smoke Harness Design

Status: active design

## Summary

Add an opt-in automated smoke harness for `/analysis` that runs against the real debug Tauri application, real SQLite-backed fixture data, and the real webview DOM. The harness replaces the fragile manual smoke checklist for the recent Analysis Workspace Parity and Source Browser work, while staying outside the default `npm.cmd run verify` gate.

The primary command is:

```powershell
npm.cmd run smoke:analysis
```

The command launches the debug Tauri app, connects to the debug MCP bridge, seeds deterministic analysis fixtures, exercises the selected `/analysis` surfaces, clears fixtures, saves failure artifacts, and exits nonzero on any failed assertion.

## Goals

- Provide a repeatable automated smoke check for the most regression-prone `/analysis` user flows.
- Exercise the real Tauri app, debug fixture commands, SQLite persistence path, and webview DOM.
- Keep the smoke check opt-in and separate from `npm.cmd run verify`.
- Avoid adding Playwright, WebDriver, or browser automation dependencies for the MVP.
- Clean up fixture data and spawned app processes reliably.
- Produce useful failure output for local debugging.

## Non-Goals

- This is not a default CI or `verify` gate.
- This does not test real Telegram auth, live sync, YouTube network requests, or provider-backed LLM calls.
- This does not replace unit tests, raw source contract tests, or component tests.
- This does not validate visual pixel perfection.
- This does not implement source-group NotebookLM export; it only verifies the current disabled affordance.

## Command And Process Model

Add a package script:

```json
"smoke:analysis": "node scripts/smoke-analysis.mjs"
```

The script starts `npm.cmd run tauri dev` by default. It owns the spawned process and must terminate the full process tree during teardown. On Windows, the implementation should use a process-tree-aware cleanup such as `taskkill /PID <pid> /T /F` for spawned runs.

The script may also support an attach mode:

```powershell
npm.cmd run smoke:analysis -- --attach
```

Attach mode connects to an already-running debug Tauri app and does not stop the app at the end. Attach mode is optional for MVP implementation. The default command path must launch and clean up the app itself.

## MCP Bridge Connection

The debug Tauri app already installs `tauri_plugin_mcp_bridge` under `debug_assertions`. The harness connects directly to the plugin WebSocket rather than using Playwright.

Connection behavior:

- Try `ws://127.0.0.1:{9223..9322}` until a bridge responds.
- Use the MCP bridge `invoke_tauri` command for `plugin:mcp-bridge|get_backend_state`.
- Require the expected app identifier `org.ai.extractum`.
- Fail with a clear message if no debug bridge is available or the identifier is different.
- Require a Node runtime with built-in `globalThis.WebSocket`; fail with an actionable message if it is missing.

All webview interaction uses MCP bridge `execute_js` calls. The script should not call production app APIs directly from Node. Fixture setup and cleanup happen inside the webview through:

```js
window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures")
window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures")
```

## Selector Strategy

Prefer accessible text and existing unique UI text where reliable. Add a small number of inert smoke selectors only where DOM structure or duplicated labels make text selectors brittle.

Recommended `data-smoke-id` targets:

- `analysis-workspace-tools`
- `analysis-report-setup`
- `analysis-source-surface`
- `source-browser-tabs`
- `notebooklm-export-dialog`
- `template-editor-drawer`
- `source-group-editor-drawer`

These attributes are test hooks only. They must not change layout, styling, or user-visible behavior.

## Fixture Lifecycle

The harness uses the existing debug fixture commands.

Setup:

1. Launch or attach to the debug Tauri app.
2. Resize the main window to a deterministic desktop size, such as `1280x860`.
3. Clear existing analysis redesign fixtures.
4. Seed analysis redesign fixtures.
5. Navigate to `/analysis`.
6. Verify that the expected fixture labels are available.

Expected fixture set:

- `__analysis_redesign_fixture__ Telegram Channel`
- `__analysis_redesign_fixture__ Telegram Supergroup`
- `__analysis_redesign_fixture__ YouTube Video`
- `__analysis_redesign_fixture__ YouTube Playlist`
- `__analysis_redesign_fixture__ Telegram Group`
- `__analysis_redesign_fixture__ Completed Snapshot Run`
- `__analysis_redesign_fixture__ Group Snapshot Run`

The implementation may validate the full fixture summary counts if they remain deterministic. At minimum, it must fail if any fixture required by the smoke scenarios is missing.

Teardown:

- Always attempt `clear_analysis_redesign_fixtures`, even after partial failure.
- Report fixture cleanup failure separately and return nonzero.
- Preserve failure artifacts before teardown when possible.
- Stop the spawned Tauri process tree unless running in attach mode.

## Smoke Scenarios

### Workspace Parity

Single-source setup:

- Select a single-source fixture.
- Confirm `Workspace tools` renders below the canvas header / mode tabs and above the setup body.
- Confirm NotebookLM export is visible and enabled when `currentSource` is non-null.
- Click NotebookLM export and confirm one canvas-level `NotebookLmExportDialog` opens.
- Close the dialog.
- Confirm `Edit templates` opens the template drawer below workspace tools.
- Confirm `Edit groups` opens the source-group drawer below workspace tools.
- Confirm setup still exposes `Run report` and `Sync source`.

Source-group setup:

- Select the source-group fixture.
- Confirm NotebookLM export is visible but disabled.
- Confirm the visible helper reason is present and associated with the disabled button.
- Confirm template and group drawers remain available.

Opened single-source run:

- Open the completed snapshot run.
- Confirm workspace tools remain in the same canvas-level location.
- Confirm NotebookLM export is enabled only if the canvas-level `currentSource` is restored.
- If `currentSource` is not restored for the opened run, saved-run metadata alone must not make the export dialog open.

Opened source-group run:

- Open the group snapshot run.
- Confirm NotebookLM export is visible but disabled with the source-group reason.

Source mode:

- Switch from `Report` to `Source`.
- Confirm workspace tools remain above the source body, not inside setup-only UI.

### Source Browser

Telegram live source:

- Open `__analysis_redesign_fixture__ Telegram Channel`.
- Confirm tab order `Timeline | Items | Metadata | Activity`.
- Confirm the default selected tab is `Timeline`.

YouTube video:

- Open `__analysis_redesign_fixture__ YouTube Video`.
- Confirm tab order `Transcript | Comments | Items | Metadata | Activity`.
- Confirm the default selected tab is `Transcript`.

YouTube playlist:

- Open `__analysis_redesign_fixture__ YouTube Playlist`.
- Confirm tab order `Videos | Items | Metadata | Activity`.
- Confirm the default selected tab is `Videos`.

Live source group:

- Open `__analysis_redesign_fixture__ Telegram Group`.
- Confirm tab order `Sources | Items | Metadata | Activity`.
- Confirm the default selected tab is `Sources`.

Run snapshot:

- Open `__analysis_redesign_fixture__ Completed Snapshot Run`.
- Confirm the source browser header identifies the view as a run snapshot.
- Confirm the header exposes `View live source`.
- Confirm snapshot tabs include `Sources | Items | Metadata`.
- Confirm snapshot tabs do not include `Activity`.

## Failure Artifacts

The harness prints grouped step output with explicit pass/fail status. On failure it should create a timestamped artifact directory under:

```text
tmp/analysis-smoke/
```

Artifacts should include the most useful data available:

- failed step name;
- current path or URL;
- visible text summary;
- relevant DOM summary;
- webview console logs if the bridge exposes them or the script can collect them;
- native screenshot through MCP bridge screenshot support.

`tmp/analysis-smoke/` must be ignored by git.

## Testing Strategy

The real smoke command remains opt-in. It should not be called by `npm.cmd run verify`.

Implementation should still include lightweight automated safeguards:

- A package-script contract test verifies `smoke:analysis` exists and `verify` does not invoke it.
- Pure helper tests cover bridge response parsing, port range selection, tab-order assertions, fixture summary validation, and artifact path generation if these helpers are split out from the script.
- Source contract tests verify smoke-only selectors are present only on intended analysis surfaces.
- The implementation plan should include a final manual run of `npm.cmd run smoke:analysis` in a local GUI-capable environment and archive the result under `docs/superpowers/archive/verification/`.

## Error Handling

- Fail early if the debug MCP bridge cannot be discovered.
- Fail early if backend state does not match `org.ai.extractum`.
- Fail with a clear message if Node does not provide a built-in WebSocket implementation.
- Fail if required fixtures are missing after seeding.
- Time out waits with actionable messages that name the expected surface or text.
- Always attempt fixture cleanup and process cleanup.
- Exit nonzero if scenario assertions fail, fixture cleanup fails, or process cleanup fails.

## Risks

- GUI smoke automation is environment-dependent. It needs a desktop session, WebView2, and a debug Tauri app that can open normally.
- DOM text and layout changes can break smoke assertions. The harness should use stable smoke selectors sparingly and keep user-visible text assertions focused on contract-level behavior.
- Process cleanup on Windows can leave child processes behind if implemented naively. The launch path must own and clean the process tree.
- The smoke command can be slower than unit tests, which is why it remains opt-in.

## Acceptance Criteria

- `npm.cmd run smoke:analysis` exists and is documented as opt-in.
- The default command launches the real debug Tauri app and connects through the MCP bridge.
- If attach mode is implemented, it connects to an existing app without stopping it during teardown.
- The command seeds and clears analysis redesign fixtures through existing debug commands.
- Workspace Parity surfaces are checked in setup, opened-run, and source-mode states.
- Source Browser tab contracts are checked for Telegram source, YouTube video, YouTube playlist, live source group, and run snapshot.
- Source-group NotebookLM export is verified as disabled with visible reason.
- Failure artifacts are written under a gitignored `tmp/analysis-smoke/` directory.
- `npm.cmd run verify` does not run the smoke harness.
