# Gemini Browser Run History Inspector Design

## Context

The Gemini Browser Provider now has:

- user-controlled Chrome CDP attach;
- a safe `Start Chrome` command;
- file-backed run records under the app-data Gemini browser runs directory;
- an inline `Run inspector` in the Browser Providers settings panel;
- copied diagnostics that omit prompt text, answer text, local paths, account hints, and sensitive URLs;
- answer extraction diagnostics for stable, missing, and `timeout_latest` outcomes.

The current UI still treats the latest or active run as the primary diagnostic
surface. `Recent browser runs` is visible, but it is a passive list. When the
provider behaves inconsistently across retries, the operator has to mentally
compare rows, open folders, copy diagnostics one run at a time, or rely on the
latest run only.

## Problem

Browser Provider debugging is usually comparative:

- one run is `needs_manual_action`, then the next is `ok`;
- one run returns `timeout_latest`, then a retry returns `stable`;
- a long prompt changes answer length across attempts;
- the user wants to inspect a run from two attempts ago, not just the newest
  one;
- a failed run has the artifact folder, while a later successful run hides the
  earlier failure context.

The existing run log already stores enough information for a better UI, but the
Settings panel does not yet expose it as a real diagnostic workflow.

## Goals

- Turn `Recent browser runs` into a compact, clickable run history.
- Keep the existing inline `Run inspector` as the detail pane for the selected
  run.
- Add filters that help find problematic runs quickly.
- Make partial-risk runs visibly different from normal successful runs.
- Preserve the existing privacy boundary: no prompt text beyond
  `prompt_preview`, no answer text in history rows, and no raw artifact paths in
  copied diagnostics.
- Keep this as a Settings-panel improvement, not a new global debug screen.
- Reuse existing `gemini_bridge_list_runs`, `gemini_bridge_open_run_folder`,
  run log DTOs, and run inspector helpers where possible.

## Non-Goals

- Do not add a new Tauri window, route, or application-level Debug tab.
- Do not read or render full artifact file contents in this slice.
- Do not add health-check probing in this slice.
- Do not change Browser Provider automation behavior or sidecar protocol.
- Do not expose full prompt or answer text in the history list.
- Do not add pagination or large-scale log management in v1.

## Proposed UX

The Browser Providers panel keeps the current overall structure:

1. Provider controls.
2. Test prompt.
3. Run inspector.
4. Run history.

The `Run history` block replaces the passive `Recent browser runs` list. It is
a master list for the existing inspector detail pane.

### History Controls

Add a compact filter row above the run list:

- `All`
- `Problems`
- `Partial risk`
- `Manual action`
- `Failed`

Only one filter is active at a time in v1.

Filter definitions:

- `All`: every loaded run.
- `Problems`: runs with status `failed`, `timeout`, `browser_crashed`,
  `blocked`, `needs_login`, `needs_manual_action`, or a result with
  `partial_risk`.
- `Partial risk`: runs whose result debug summary has
  `answer_completion_reason === "timeout_latest"`.
- `Manual action`: runs whose status or result status is
  `needs_manual_action` or `needs_login`, or whose result has a non-null
  `manual_action`.
- `Failed`: runs whose status or result status is `failed`, `timeout`,
  `browser_crashed`, or `blocked`.

The existing `Refresh` action remains in the inspector header. Refresh updates
status and run history, then preserves the selected run when possible.

### History Rows

Each history row is a button-like row that selects a run for the inspector.

Each row should show:

- status;
- a short risk/status badge, such as `partial`, `manual`, `failed`, `running`,
  or `stable`;
- `prompt_preview`;
- updated time;
- elapsed time if a result exists;
- result text length if a result exists;
- answer completion reason if a debug summary exists.

Rows must not show full answer text, full prompt text, raw artifact paths, or
copied diagnostics.

The selected row is visually highlighted. Running/queued rows should remain
recognizable, but they should not shift the layout.

### Inspector Selection

Selection rules:

- On initial load, select the active run when present; otherwise select the
  newest run.
- When the user manually selects a run, preserve that selection across refreshes
  if the run still exists.
- When a new run starts from the Settings test prompt, select that new run.
- If the selected run disappears because the loaded list changed, fall back to
  the active run or newest visible run.
- Filtering should not erase the selected run globally. If the current filter
  hides the selected run, the UI may either show a small note or select the
  first visible row. V1 should choose the first visible row to keep behavior
  simple and predictable.

The current `Run inspector` detail pane should continue to use the selected
run, not an implicit latest-only run.

## Data Model And Helpers

No new backend DTO is required for v1.

Frontend helpers should live near the existing inspector helper code, likely in
`src/lib/gemini-browser-run-inspector.ts`, unless the implementation becomes
large enough to justify a small sibling module.

Suggested derived concepts:

```ts
type GeminiBrowserRunHistoryFilter =
  | "all"
  | "problems"
  | "partial_risk"
  | "manual_action"
  | "failed";

interface GeminiBrowserRunHistoryRow {
  run: GeminiBrowserRun;
  status: GeminiBrowserRunStatus;
  badge: "ok" | "stable" | "partial" | "manual" | "failed" | "running" | "queued";
  isProblem: boolean;
  isPartialRisk: boolean;
  elapsedMs: number | null;
  resultTextLength: number | null;
  answerCompletionReason: GeminiBrowserAnswerCompletionReason | null;
}
```

The exact interface can differ, but the logic should be testable without
rendering the Svelte component.

## Error Handling

If `gemini_bridge_list_runs` fails, keep the current status/provider controls
usable and show a small history-specific message.

If opening a run folder fails, keep the selected run and show the existing
inspector message pattern.

If a run lacks `debug_summary`, show `Debug summary unavailable` in the detail
pane and omit completion fields in its history row.

If older run JSON lacks `answer_extraction`, `debug_summary.extraction`, or
newer optional fields, history helpers must tolerate missing values.

## Privacy And Safety

The history list is local operator UI, but it should still follow the reduced
diagnostic boundary:

- `prompt_preview` is allowed because it is already persisted by run log.
- Full prompt text is not shown.
- Full answer text is not shown.
- Artifact paths are not shown in row content.
- Copied diagnostics stay sanitized through the existing helper.
- Opening a run folder continues to go through the existing Tauri command and
  backend security checks.

## Testing Strategy

Add focused helper tests for:

- filter classification for `partial_risk`, `needs_manual_action`, `failed`,
  `timeout`, `browser_crashed`, `blocked`, `running`, and normal `ok stable`;
- row summary fields for elapsed time, result text length, and answer completion
  reason;
- missing `debug_summary` and older run DTOs;
- selection preservation across refreshed run lists;
- fallback selection when the selected run is no longer visible after filtering.

Add Svelte source-contract tests, consistent with the current repo style, for:

- filter labels exist;
- history rows are selectable;
- selected row class/state exists;
- current `Run inspector` still renders copy/open-folder actions.

Manual validation:

1. Run a short stable Settings prompt and confirm it appears as a normal stable
   row.
2. Run a long prompt or inspect an existing `partial_risk` run and confirm it
   is visible through the `Partial risk` filter.
3. Trigger or inspect a manual-action run and confirm it appears through the
   `Manual action` and `Problems` filters.
4. Select an older run and confirm `Run inspector`, `Copy diagnostics`, and
   `Open run folder` operate on that selected run.
5. Press `Refresh` and confirm the selected run stays selected if it still
   exists.

## Acceptance Criteria

- `Recent browser runs` becomes a usable `Run history` master list.
- The existing `Run inspector` shows the selected history run.
- Filters for `All`, `Problems`, `Partial risk`, `Manual action`, and `Failed`
  work from derived run data.
- Partial-risk runs are visibly flagged.
- History rows do not expose full prompt text, answer text, or artifact paths.
- Refresh preserves the selected run when possible.
- Older run records without new debug fields remain renderable.
- Automated helper tests cover filtering, row derivation, and selection.
- Existing Browser Provider diagnostics and open-folder behavior continue to
  work.

## Future Work

- Add a one-click Provider Health Check.
- Add an Artifact Summary Viewer for reduced local artifact facts.
- Add a larger dedicated Browser Provider debug screen if Settings becomes too
  dense.
- Add retention controls for old run folders.
