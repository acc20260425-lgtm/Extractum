# Analysis UI Smoke Harness Design

Status: implemented

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
"smoke:analysis": "node scripts/analysis-smoke.mjs"
```

The script starts `npm.cmd run tauri dev` by default. It owns the spawned process and must terminate the full process tree during teardown. On Windows, the implementation should use a process-tree-aware cleanup such as `taskkill /PID <pid> /T /F` for spawned runs.

The script may also support an attach mode:

```powershell
npm.cmd run smoke:analysis -- --attach
```

Attach mode connects to an already-running debug Tauri app and does not stop the app at the end. Attach mode is optional for MVP implementation. The default command path must launch and clean up the app itself.

MVP treats attach mode as deferred unless the implementation plan explicitly includes it. Deferred attach mode must not weaken the default launch-and-cleanup command path.

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

## MCP Bridge Protocol Assumptions

The implementation must treat the bridge protocol as an explicit contract instead of reverse-engineering it inside scenario code.

Base WebSocket request:

```json
{ "id": "request-id", "command": "command_name", "args": {} }
```

Base success response:

```json
{ "id": "request-id", "success": true, "data": {} }
```

Base error response:

```json
{ "id": "request-id", "success": false, "error": "message" }
```

`execute_js` request:

```json
{
  "id": "request-id",
  "command": "execute_js",
  "args": {
    "script": "return document.title;",
    "windowLabel": "main"
  }
}
```

`execute_js` response:

```json
{
  "id": "request-id",
  "success": true,
  "data": "Extractum",
  "error": null,
  "windowContext": { "windowLabel": "main" }
}
```

JavaScript exceptions are bridge-level success responses with `success: false` and an `error` string. On Windows and other non-macOS platforms, the current bridge implementation also reports script IPC timeout as `success: false` with `error: "Script execution timeout"`.

The harness should classify failures distinctly:

- bridge unavailable: no WebSocket connection opens on the expected port range;
- bridge disconnect: socket closes before a matching response arrives;
- bridge timeout: no matching response arrives before the harness command timeout;
- script failure: bridge response has `success: false` for `execute_js`;
- assertion failure: script failure whose error starts with a smoke assertion prefix such as `ASSERT:`;
- app contract failure: helper returns data that does not match the expected UI contract.

Window resize uses the bridge `resize_window` command:

```json
{
  "id": "request-id",
  "command": "resize_window",
  "args": { "width": 1280, "height": 860, "windowId": "main", "logical": true }
}
```

Failure screenshots use the bridge `capture_native_screenshot` command:

```json
{
  "id": "request-id",
  "command": "capture_native_screenshot",
  "args": { "format": "png", "maxWidth": 1280, "windowLabel": "main" }
}
```

The screenshot result is a data URL for the visible viewport. The harness should scroll relevant content into view before capture when a step knows the target is below the fold.

The harness should use `invoke_tauri` only for MCP bridge plugin commands such as `plugin:mcp-bridge|get_backend_state`. It should not use `invoke_tauri` to call Extractum production commands from Node; app fixture commands must run inside `execute_js` through `window.__TAURI__.core.invoke(...)`.

## Smoke Helper Layer

Because the MVP intentionally avoids Playwright and WebDriver, the script needs a small helper layer instead of ad hoc JavaScript snippets scattered through scenarios.

Required helpers:

- `bridgeRequest(command, args, timeoutMs)` sends one WebSocket request and returns the matching response.
- `executeJs(script, timeoutMs)` wraps `execute_js`, maps bridge failures to typed errors, and returns response data.
- `waitForText(text, timeoutMs)` polls visible DOM text until the text appears.
- `clickByText(text)` clicks the first visible interactive element with the requested accessible or visible text.
- `clickBySmokeId(id)` clicks an element selected by `data-smoke-id`.
- `getVisibleTextSummary()` returns a compact visible text summary for artifacts.
- `assertTabOrder(smokeId, expectedTabs)` reads the tab labels from a stable tab container and compares exact order.
- `assertSelectedTab(smokeId, expectedTab)` checks selected tab state through ARIA state or the component's stable selected marker.
- `assertDisabledWithReason(buttonText, reasonText)` verifies the button is disabled and the reason text is visible and associated.
- `captureArtifacts(stepName)` writes DOM summary, visible text, and screenshot data when available.

The helpers can live in one smoke-specific module, such as `scripts/analysis-smoke-helpers.mjs`, or in the smoke script if kept small. Scenario steps should read like user workflows and call helpers rather than embedding raw DOM traversal repeatedly.

## Selector Strategy

Prefer accessible text and existing unique UI text where reliable. Add a small number of inert smoke selectors only where DOM structure or duplicated labels make text selectors brittle.

Every smoke selector must correspond to a stable user-facing contract. Selectors should identify surfaces, commands, visible states, or accessible helper text; they should not encode incidental nesting or private implementation structure.

Recommended `data-smoke-id` targets for MVP:

- `analysis-workspace-tools`
- `notebooklm-export-button`
- `notebooklm-export-disabled-reason`
- `analysis-report-setup`
- `analysis-source-surface`
- `source-browser-header`
- `source-browser-tabs`
- `notebooklm-export-dialog`
- `template-editor-drawer`
- `template-editor-drawer-title`
- `source-group-editor-drawer`
- `source-group-editor-drawer-title`
- `run-snapshot-header`

These attributes are test hooks only. They must not change layout, styling, or user-visible behavior.

## Fixture Lifecycle

The harness uses the existing debug fixture commands.

Setup:

1. Launch the debug Tauri app. If attach mode is included in the implementation plan, attach to an existing debug app instead.
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
- `__analysis_redesign_fixture__ Telegram Source Group`
- `__analysis_redesign_fixture__ Completed Snapshot Run`
- `__analysis_redesign_fixture__ Group Snapshot Run`

The source-group fixture label is intentionally `Telegram Source Group`. The current fixture code may still call this `Telegram Group`; the implementation should normalize the fixture label or keep a single constant alias so smoke steps cannot confuse an analysis source group with a Telegram small-group source.

The implementation may validate the full fixture summary counts if they remain deterministic. At minimum, it must fail if any fixture required by the smoke scenarios is missing.

Teardown:

1. Preserve failure artifacts first, while the failed DOM state is still available.
2. Attempt `clear_analysis_redesign_fixtures`, even after partial scenario failure.
3. If the app is still reachable, verify fixture rows are gone.
4. Stop the spawned Tauri process tree. If attach mode is included and active, leave the attached app running.
5. Exit nonzero if fixture cleanup, cleanup verification, or process cleanup fails, even if all scenario assertions passed.

Fixture cleanup must be marker-scoped. The debug commands must delete only rows owned by the analysis redesign fixture marker or prefix, never arbitrary user data in the real SQLite database. The smoke harness should treat any broader cleanup behavior as a blocker.

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
- Close any open dialog or drawer before leaving the scenario.

Source-group setup:

- Select the source-group fixture.
- Confirm NotebookLM export is visible but disabled.
- Confirm the visible helper reason is present and associated with the disabled button.
- Confirm template and group drawers remain available.
- Close any open drawer before leaving the scenario.

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

- Open `__analysis_redesign_fixture__ Telegram Source Group`.
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

Required MVP artifacts:

- failed step name;
- current path or URL;
- visible text summary;
- relevant DOM summary;
- JavaScript error or exception text.

Best-effort artifacts:

- native screenshot through MCP bridge screenshot support;
- webview console logs if the bridge exposes them or the script can collect them;
- backend state snapshot.

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
- A failed assertion must not skip teardown.
- Exit nonzero if scenario assertions fail, fixture cleanup fails, or process cleanup fails.

## Risks

- GUI smoke automation is environment-dependent. It needs a desktop session, WebView2, and a debug Tauri app that can open normally.
- DOM text and layout changes can break smoke assertions. The harness should use stable smoke selectors sparingly and keep user-visible text assertions focused on contract-level behavior.
- Process cleanup on Windows can leave child processes behind if implemented naively. The launch path must own and clean the process tree.
- The smoke command can be slower than unit tests, which is why it remains opt-in.

## Acceptance Criteria

- `npm.cmd run smoke:analysis` exists and is documented as opt-in.
- The default command launches the real debug Tauri app and connects through the MCP bridge.
- Attach mode is either explicitly deferred for MVP or, if implemented, connects to an existing app without stopping it during teardown.
- Smoke steps have deterministic names used in console output and artifact folder/file names.
- Every scenario starts from a known `/analysis` route and UI state; dialogs, drawers, and overlays are closed before the next scenario.
- The command seeds and clears analysis redesign fixtures through existing debug commands.
- Fixture cleanup is marker-scoped and verifies fixture rows are gone when the app remains reachable.
- A failed assertion does not skip fixture cleanup or spawned process cleanup.
- The smoke helper layer centralizes WebSocket requests, `execute_js`, waits, clicks, tab assertions, disabled-state assertions, and artifact capture.
- Smoke-only selector contract tests fail if `data-smoke-id` hooks appear outside the intended `/analysis` surfaces.
- Workspace Parity surfaces are checked in setup, opened-run, and source-mode states.
- Source Browser tab contracts are checked for Telegram source, YouTube video, YouTube playlist, live source group, and run snapshot.
- Source-group NotebookLM export is verified as disabled with visible reason.
- Required failure artifacts are written under a gitignored `tmp/analysis-smoke/` directory; screenshot, console logs, and backend state are best-effort.
- `npm.cmd run verify` does not run the smoke harness.
