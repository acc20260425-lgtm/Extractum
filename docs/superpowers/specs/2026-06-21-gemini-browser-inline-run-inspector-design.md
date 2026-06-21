# Gemini Browser Inline Run Inspector Design

## Context

The Gemini Browser Provider now supports managed Playwright mode, user-controlled
Chrome CDP attach mode, a local Chrome launcher, file-backed run logs, and
reduced failure artifacts. The happy path works, but live debugging is still too
slow: an operator or LLM agent often has to inspect app-data run files manually
to understand whether a run failed in Chrome/CDP setup, Gemini DOM selection,
prompt submission, answer extraction, run-log refresh, or sidecar packaging.

This design adds an inline run inspector to the existing Browser Providers
settings panel. It keeps diagnostics next to the controls that create provider
runs, so a failed or suspicious run can be inspected without leaving the
provider workflow.

## Goals

- Show the most useful recent run diagnostics directly in
  `src/lib/components/settings/gemini-browser-provider-panel.svelte`.
- Make the latest active/test run explain itself without opening app-data files.
- Preserve the existing file-backed run model and app-data artifact layout.
- Add a compact `debug_summary` to Gemini Browser run results so the UI can
  report what the sidecar observed.
- Keep live Gemini privacy boundaries: no prompt body, full HTML, screenshot, or
  account data should be copied into the UI diagnostics summary.

## Non-Goals

- Do not create a new top-level route or separate Settings page.
- Do not implement a full artifact viewer.
- Do not load or render `page.html`, `page.png`, or telemetry contents in the
  frontend.
- Do not broaden CDP endpoint support or weaken the loopback-only security
  model.
- Do not replace the existing Browser Provider troubleshooting guide.

## User Experience

The Browser Providers panel gains a compact inline inspector below the test
prompt/result area.

The inspector shows:

- selected run id;
- source;
- run status and result status;
- created/updated timestamps;
- elapsed milliseconds when a result exists;
- result text length and debug final text length when a result exists;
- latest run message, sanitized before display;
- manual action when present;
- artifact path availability: run directory, telemetry, reduced/full artifact
  refs, and artifact write error;
- debug facts from `debug_summary`.

The first version does not need a new navigation surface. It should default to
the active test run when available. If no active test run exists, it should show
the newest run from the log.

Controls:

- `Refresh` reloads provider status and run log.
- `Copy diagnostics` copies a sanitized text/JSON summary for the selected run.
- `Open run folder` opens the run directory when `artifacts.run_dir` is present.

The UI should not hide the existing test prompt flow. The inspector is an
operability panel, not a replacement for the provider controls.

The panel may display artifact availability because the operator is already
using the local app, but run messages should still be sanitized before display.
The copied diagnostics summary is conservative: it includes the run id and
artifact availability flags by default, not full local paths. Sanitization must
redact local paths, `file://` paths, URL query/hash data, email-like account
hints, and overlong messages.

## Debug Summary Contract

`GeminiBrowserRunResult` gains an optional `debug_summary` object. It is produced
by the TypeScript sidecar and passed through Rust to the frontend.

Initial fields:

- `mode`: `managed` or `cdp_attach`.
- `composer_found`: whether the adapter found a visible composer.
- `send_button_found`: whether the adapter found a visible send button.
- `generation_busy_observed`: whether a stop-generation/busy control was seen
  before sending.
- `answer_found`: whether non-empty answer text was extracted.
- `answer_selector`: the selector that produced the final answer, or `null`.
- `waited_for_send_ms`: approximate wait time before send button resolution.
- `waited_for_answer_ms`: approximate wait time before answer text resolution.
- `answer_stable_ms`: stability window used before accepting the final answer.
- `answer_completion_reason`: `stable`, `timeout_latest`, or `missing`.
- `final_text_length`: final extracted answer length, or `0`.
- `error_stage`: one of `setup`, `composer`, `send`, `answer`, `artifacts`,
  `transport`, or `null`.

The summary must stay sanitized. It can include selector names, booleans,
durations, counters, and high-level stage names. It must not include prompt text,
answer text, cookies, account identifiers, URLs with sensitive query data, raw
DOM HTML, or screenshot paths beyond the already existing artifact references.

## Data Flow

1. `sidecars/gemini-browser/src/adapter.ts` records debug facts while it runs
   browser setup, composer lookup, send lookup, answer extraction, and artifact
   capture.
2. `sidecars/gemini-browser/src/protocol.ts` extends `GeminiBrowserRunResult`
   with optional `debug_summary`.
3. `src-tauri/src/gemini_browser/types.rs` mirrors the same optional structure.
4. `src-tauri/src/gemini_browser/run_log.rs` stores the result as part of the
   existing run record.
5. `src/lib/types/gemini-browser.ts` mirrors the frontend DTO.
6. `src-tauri/src/gemini_browser/commands.rs` exposes a command to open a
   recorded run directory when the persisted `result.artifacts.run_dir` is
   present. The persisted path is treated as an availability flag only; the
   command opens the canonical app-data run directory computed from the safe run
   id.
7. `gemini-browser-provider-panel.svelte` renders the inline inspector using the
   existing `geminiBridgeListRuns()` refresh path.

The change intentionally follows the existing run-log path. The frontend should
not call the sidecar directly and should not inspect filesystem artifacts. Folder
opening stays a Rust/Tauri command so path validation is centralized.

## Error Handling

Missing `debug_summary` is valid. Older run records and unexpected sidecar
responses should still render with the existing status, result, message, and
artifact refs.

`Copy diagnostics` should still work when only partial run data is available.
It should include a clear `debug_summary: unavailable` marker instead of
throwing.

If `Open run folder` fails, the UI should show a short error message and keep the
selected run visible.

The backend command must not trust the frontend button state. It should reject
queued/running runs without a terminal result, older records without
`result.artifacts.run_dir`, unsafe run ids, missing run directories, and records
whose `result.artifacts.run_dir` points outside the canonical
`gemini-browser/runs/<safe_run_id>` directory.

## Testing

Sidecar tests:

- a successful send includes a sanitized debug summary with answer selector,
  wait durations, final text length, and `answer_completion_reason: "stable"`;
- a slow answer that never stabilizes before timeout reports
  `answer_completion_reason: "timeout_latest"` when a latest visible answer is
  returned;
- a send-button failure reports `error_stage: "send"` without prompt or answer
  text in debug summary;
- a previous-generation wait records `generation_busy_observed: true`;
- TypeScript and frontend DTOs tolerate missing `debug_summary` for older run
  records while new sidecar results populate it.

Rust tests:

- `GeminiBrowserRunResult` serializes and deserializes optional
  `debug_summary`;
- run-log storage preserves `debug_summary`;
- open-run-folder command requires a persisted `result.artifacts.run_dir`, opens
  only the computed canonical app-data run directory for the safe run id, and
  returns typed errors.

Frontend tests:

- the inspector selects the active run when present;
- it falls back to the newest run when no active run exists;
- it renders missing debug summary gracefully;
- copy-diagnostics output is sanitized and includes status, run id, elapsed,
  separate `result_text_length` and `debug_final_text_length`, artifact
  availability, and debug facts;
- copy-diagnostics output does not include artifact paths, answer text, or
  path-like substrings from run messages;
- copy-diagnostics and inline messages redact URL query/hash data, `file://`
  paths, `/home/<user>` paths, `%LOCALAPPDATA%`, email-like account hints, and
  overlong messages;
- open-folder failure renders a non-destructive UI error.
- copy/selection behavior is covered in pure view-model tests; Svelte
  source-contract tests only verify wiring and rendering affordances.

Manual validation:

1. Start the app in dev mode.
2. Settings -> Browser Providers.
3. Start Chrome or attach to an existing CDP session.
4. Send the short browser-provider smoke prompt.
5. Confirm the response appears and the inline inspector shows status, elapsed,
   result text length, debug final text length, answer completion reason,
   artifact refs, and debug facts.
6. Trigger one recoverable failure, such as closing the Gemini tab before send,
   and confirm the inspector explains the stage without exposing private page
   content.

Opening the run folder is a manual UX check, not a CI gate. Automated acceptance
relies on Rust command validation plus frontend view-model and source-contract
tests.

## Acceptance Criteria

- Browser Providers panel includes a useful inline run inspector without a new
  route.
- Latest/active run diagnostics are visible after refresh and after run events.
- `debug_summary` is available for new sidecar results and optional for old
  records.
- Copied diagnostics are sanitized and useful for an LLM/debugging session.
- Existing Browser Provider controls and test prompt behavior continue to work.
- Automated coverage exists across sidecar, Rust DTO/run-log, and frontend view
  model/UI behavior.
