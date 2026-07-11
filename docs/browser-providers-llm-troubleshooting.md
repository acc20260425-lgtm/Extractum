# Browser Providers LLM Troubleshooting Guide

Last verified against the repository on 2026-06-22.

This document is written for LLM agents that need to debug or extend Extractum Browser Providers quickly. The main target is the Gemini Browser Provider, especially failures caused by Gemini UI or DOM changes.

## Mental Model

Browser Providers let Extractum use a real browser session as an LLM transport.

The Gemini path is:

1. Settings UI calls `$lib/api/gemini-browser.ts`.
2. Tauri commands in `src-tauri/src/gemini_browser/commands.rs` receive the request.
3. Rust starts or reuses the Gemini browser sidecar from `src-tauri/src/gemini_browser/sidecar.rs`.
4. Rust and the sidecar exchange one JSONL envelope per request.
5. The sidecar adapter in `sidecars/gemini-browser/src/adapter.ts` controls Gemini through Playwright.
6. DOM selectors come from `sidecars/gemini-browser/src/dom-contract.ts`.
7. Rust records runs under the app data Browser Provider run directory.

Most user-visible provider failures are not frontend bugs. They usually come from one of four layers:

- Chrome/CDP setup.
- Sidecar launch or JSONL protocol mismatch.
- Login/manual-action state in Gemini.
- Gemini DOM drift, where composer, send, or answer selectors no longer match.

## Source Map

Frontend:

- `src/lib/components/settings/gemini-browser-provider-panel.svelte` is the operator UI.
- `src/lib/api/gemini-browser.ts` wraps Tauri invokes.
- `src/lib/types/gemini-browser.ts` defines frontend status, result, config, and manual-action types.
- `src/lib/gemini-browser-provider-panel-contract.ts` maps statuses/actions to UI labels.
- `src/lib/gemini-browser-run-inspector.ts` derives run history rows, filters,
  selected-run behavior, artifact availability, partial-risk detection, and
  copyable diagnostics.

Tauri/Rust:

- `src-tauri/src/gemini_browser/commands.rs` exposes Tauri commands.
- `src-tauri/src/gemini_browser/types.rs` defines Rust protocol/status structs.
- `src-tauri/src/gemini_browser/jobs.rs` owns the Apalis-backed SQLite queue,
  worker, completion waiters, cancellation state, and queue polling config.
- `src-tauri/src/gemini_browser/sidecar.rs` starts the sidecar and sends JSONL requests.
- `src-tauri/src/gemini_browser/sidecar_launch.rs` chooses dev Node script vs bundled sidecar binary.
- `src-tauri/src/gemini_browser/cdp_chrome.rs` starts user-controlled Chrome with a CDP port.
- `src-tauri/src/gemini_browser/paths.rs` owns app-data paths.
- `src-tauri/src/gemini_browser/run_log.rs` creates and updates run records.

Sidecar:

- `sidecars/gemini-browser/src/index.ts` is the JSONL process entrypoint.
- `sidecars/gemini-browser/src/protocol.ts` is the sidecar protocol contract.
- `sidecars/gemini-browser/src/adapter.ts` owns browser automation and run behavior.
- `sidecars/gemini-browser/src/answer-extractor.ts` owns answer candidate
  grouping, structural baseline filtering, stable/timeout completion semantics,
  and reduced extraction diagnostics.
- `sidecars/gemini-browser/src/dom-contract.ts` owns Gemini selector candidates.
- `sidecars/gemini-browser/src/cdp-endpoint.ts` validates and probes CDP endpoints.
- `sidecars/gemini-browser/src/cdp-pages.ts` selects existing Gemini pages and maps closed-target errors.
- `sidecars/gemini-browser/src/artifacts.ts` writes failure artifacts.
- `sidecars/gemini-browser/src/redaction.ts` contains artifact redaction helpers.

Research/design context:

- `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`
- `docs/superpowers/specs/2026-06-20-gemini-browser-cdp-attach-design.md`
- `docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md`
- `docs/superpowers/plans/2026-06-20-gemini-browser-cdp-attach-plan.md`
- `docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md`
- `research/gemini_browser_adapter/README.md`
- `research/gemini_browser_adapter/DOM_CONTRACT_NOTES.md`
- `research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md`
- `research/gemini_browser_adapter/TOOLS_AND_METHODS.md`

## Current Implementation Status

As of the 2026-06-21 repository audit, the Gemini Browser Provider is
implemented as a Settings-panel provider for operator-driven tests and
diagnostics, and as an optional prompt-pack runtime provider for YouTube Summary
runs.

Implemented:

- managed Playwright persistent-browser mode;
- user-controlled Chrome CDP attach mode;
- Settings UI for mode selection, CDP endpoint, `Start Chrome`, `Open`,
  `Resume`, `Stop`, and one-off test prompts;
- Rust/Tauri commands for status, open, start CDP Chrome, send single prompt,
  resume, stop, list runs, and open a run folder;
- Apalis-backed SQLite queue with one Gemini Browser worker, no automatic
  retries, and synchronous completion waiters for Settings and Prompt Pack
  callers;
- JSONL sidecar protocol and bundled sidecar binary path;
- file-backed run log with queued/running/final result records;
- hardened answer extraction with `stable`, `timeout_latest`, and `missing`
  completion reasons;
- actionable Setup checklist for sidecar, mode, Chrome CDP, Gemini tab, Gemini
  readiness, and latest/selected test-run state;
- inline Run Inspector, sanitized `Copy diagnostics`, and filterable Run
  History;
- YouTube Summary prompt-pack routing through `runtimeProvider:
  "gemini_browser"` with managed or CDP attach browser config snapshots.

Not implemented yet:

- Retry, re-run, cancel, or queued multi-run controls in the operator UI.
- Search, export, compare, retention, or artifact-content viewing for Run
  History.

## Provider Modes

The provider has two modes.

### Managed

Managed mode launches a Playwright persistent Chromium context using the Extractum profile directory:

- `src-tauri/src/gemini_browser/paths.rs`
- app-data subdir: `gemini-browser/profile`

The adapter owns the context and can close it on `stop()`.

Use this when Google login allows the managed browser. Google may reject it with a message like "this browser or app may not be secure".

### Attach Chrome

CDP attach mode connects to a user-controlled Chrome instance:

- default endpoint: `http://127.0.0.1:9222`
- app-data subdir for the dedicated Chrome profile: `gemini-browser/chrome-cdp-profile`
- UI button: `Start Chrome`
- Tauri command: `gemini_bridge_start_cdp_chrome`
- Rust launcher: `src-tauri/src/gemini_browser/cdp_chrome.rs`

Security invariant: CDP must remain loopback-only. Valid endpoints are local HTTP base URLs such as `http://127.0.0.1:9222` or `http://localhost:9222`. Reject remote hosts, credentials, non-HTTP schemes, port `0`, paths, query strings, and hashes.

## LLM transport and credential boundary

OpenAI-compatible LLM URLs must use HTTPS unless the host is `localhost` or an IP loopback address. A saved key is scoped to the selected provider and URL origin (scheme, host, effective port); changing scope requires a replacement key or clearing the old one. Path-only URL changes keep the same scope.

When a keyed legacy profile has a blank URL, backend state loading materializes the effective URL and displays it in Settings. If that SQLite write fails, loading fails closed. Run `npm.cmd run tauri dev` for MCP-enabled inspection; direct `npx tauri dev` does not load the MCP overlay. For a production CSP check, build with `--features csp-verification` and inspect the automatically opened DevTools.

### YouTube thumbnail previews and CSP

YouTube thumbnails are resolved by the backend and rendered as local `data:` URLs. Do not add YouTube or other remote hosts to the `img-src` CSP directive. When a preview is missing, inspect the thumbnail resolver result and confirm the rendered image source starts with `data:image/`; DevTools should show no CSP image refusal. The `src/lib/tauri-security-config-contract.test.ts` contract keeps remote HTTP(S) image origins out of `img-src` while allowing Tauri's local `http://asset.localhost` source.

Ownership invariant: in CDP mode Extractum owns only the Playwright CDP connection and selected page reference. It must not close the user's Chrome, profile, context, or unrelated tabs.

Operational rule:

- `Open` may create a Gemini tab in the attached Chrome profile.
- `Resume` attaches only; it should not create a tab.
- `SendPrompt` should not create a new tab in CDP mode. If no Gemini tab exists, return `needs_manual_action`.

## Status And Run Lifecycle

Provider statuses from `src/lib/types/gemini-browser.ts`:

- `not_started`: no browser/page/session is active.
- `ready`: the provider has a usable page.
- `needs_login`: Gemini needs account login.
- `needs_manual_action`: user/operator must do something in the browser.
- `running`: a run is active.
- `stopped`: provider was stopped.
- `failed`: provider setup failed.

Manual actions:

- `login`: sign in to Google/Gemini.
- `account_picker`: choose a Google account.
- `consent`: accept required consent UI.
- `captcha`: solve CAPTCHA or verification.
- `unknown_modal`: unblock an unknown modal.
- `start_chrome_cdp`: start or expose local Chrome CDP, or open a Gemini tab in attached Chrome.

Run statuses:

- `queued`
- `running`
- `ok`
- `ready`
- `needs_login`
- `needs_manual_action`
- `blocked`
- `timeout`
- `browser_crashed`
- `failed`
- `cancelled`

Run IDs look like:

```text
gemini-browser-1782012206087-ac88d81ee8eef8
```

Run records and artifacts are stored under app data:

```text
<app-data>/gemini-browser/runs/<run_id>/result.json
<app-data>/gemini-browser/runs/<run_id>/telemetry.json
<app-data>/gemini-browser/runs/<run_id>/page.html
<app-data>/gemini-browser/runs/<run_id>/page.png
```

The exact app-data root is platform-dependent and is resolved by Tauri. On Windows it is typically under `%APPDATA%`.

The default artifact mode from Rust is `reduced`. Full HTML/screenshot artifacts should be used only in controlled local/mock situations because live Gemini pages can contain account and prompt data.

Current queue behavior is Apalis-backed. `gemini_bridge_send_single` and
Prompt Pack browser stages create a queued run log record, register a
per-run waiter, push an Apalis SQLite job into the main `extractum.db`, and then
wait for the worker result. The
Gemini Browser worker has concurrency `1`: it marks the run log `running`,
calls the sidecar, stores the terminal run result, and completes the waiter.

The Settings panel freshness path is pull-based. Mounted panel polling uses
`gemini_bridge_status_snapshot`, `gemini_bridge_list_runs`, and
`gemini_bridge_get_run(run_id)` for selected details. Live
`gemini_bridge_status` probing is reserved for manual/full refreshes and
explicit provider commands, not active polling.

The file-backed run log remains the product-facing source for Settings,
history, Prompt Pack provenance, and diagnostics. Apalis rows are queue
implementation details. `queue_depth` is currently not a product authority and
may be `0` even when run history contains queued/running browser runs.

Important Apalis polling rule: do not construct Gemini Browser queue storage
with bare `SqliteStorage::new_in_queue(...)`. The Apalis SQL default poll
strategy starts at `100ms` but applies exponential backoff up to roughly
`60s` after idle periods. For interactive browser prompts this can create a
large gap between queued run creation and prompt submission to Gemini.
Gemini Browser must use `gemini_browser_queue_config()` in
`src-tauri/src/gemini_browser/jobs.rs`, which applies a fixed `100ms`
`IntervalStrategy` without backoff for both enqueue and worker storage.

## Setup Checklist

The Browser Providers panel shows `Setup checklist` between provider controls
and the test prompt. Use it before opening run folders or reading artifacts.

Rows:

- `Sidecar`: confirms that provider status can be loaded from the sidecar.
- `Mode`: confirms managed mode or a local-looking CDP endpoint in attach mode.
- `Chrome CDP`: shows whether attach mode needs `Start Chrome`, `Resume`, or no
  action.
- `Gemini tab`: shows whether a usable Gemini page is available or whether the
  operator should open/resume the browser.
- `Gemini readiness`: uses provider status and latest run debug facts to decide
  whether login/manual action is likely, or whether a test prompt should be
  sent.
- `Last test run`: classifies the selected/latest run as stable, partial-risk,
  manual-action, failed, running, or unknown.

Checklist actions reuse existing safe controls. They do not implement retry,
cancel, prompt-pack routing, login automation, artifact reading, or remote CDP.
If a row points to `View run`, inspect the selected run through Run Inspector
and Run History.

## Inline Run Inspector

The Browser Providers panel includes an inline run inspector for the selected
active run or the newest recent run. Use it before opening app-data manually.

The inspector shows status, elapsed time, result text length, debug final text
length, answer completion reason, artifact availability, manual action, and
sanitized message text plus sidecar `debug_summary` facts such as
composer/send/answer selection and wait durations.

The `Run history` list below the inspector is the first place to compare
multiple attempts. Use the filters before opening artifact folders:

- `Problems` shows failed, blocked, timeout, browser-crashed, manual-action,
  login, and partial-risk runs.
- `Partial risk` isolates `ok + timeout_latest` results that should not be fed
  into prompt-pack automation as normal completions.
- `Manual action` isolates runs that need login, account selection, Chrome CDP
  setup, consent, CAPTCHA, or another operator step.
- `Failed` isolates failed, timeout, blocked, and browser-crashed runs.

Clicking a history row drives the inline inspector. `Copy diagnostics` and
`Open run folder` always operate on the selected history run, not necessarily
the newest run.

`Copy diagnostics` intentionally omits full local artifact paths, URL query/hash
data, email-like account hints, prompt text, answer text, raw DOM, screenshots,
cookies, and account identifiers. It also truncates overlong messages. It is
the preferred first payload to paste into an LLM debugging session.

### Answer Extraction Diagnostics

Use this when Gemini visibly produced more text than Extractum received.

Check these fields first:

- `answer_completion_reason`: `stable` means the grouped candidate satisfied the quiet window; `timeout_latest` means the sidecar returned visible text without proving completion.
- `partial_risk`: `true` means the Settings UI may show text, but prompt-pack automation must not consume it as a normal completion.
- `result_text_length` vs `debug_final_text_length`: mismatch means UI/run propagation differs from sidecar extraction.
- `extraction_raw_candidate_count` and `extraction_grouped_candidate_count`: raw > grouped is normal when Gemini splits one answer into blocks.
- `extraction_selected_grouping`: `assistant_turn` is preferred; `single_node` is a fallback and should be treated with more suspicion after DOM changes.
- `extraction_largest_candidate_length` and `extraction_larger_valid_candidate_available`: a larger valid candidate means selection/scoring needs review.
- `answer_extraction_artifact_available`: local-only artifact with selector/count/length facts, not safe for external sharing without review.

For `timeout_latest`, inspect the run locally and decide whether to retry,
extend the prompt, or treat it as a failed browser-provider completion. Do not
feed `timeout_latest` text into long prompt-pack analysis as final output.

## Fast Troubleshooting Playbook

Start with the UI status, latest message, and recent `run_id`.

### Status: `Chrome CDP endpoint is configured but not attached`

Meaning:

- The endpoint is valid and reachable enough to be configured, but the sidecar has not attached a Playwright CDP session yet.

Next checks:

1. Click `Resume`.
2. If it stays unchanged, inspect `sidecars/gemini-browser/src/adapter.ts` in `resumeBrowser()` and `attachCdpBrowser()`.
3. If Chrome was launched manually, ensure the endpoint matches the UI field.

### Status/manual action: `start_chrome_cdp`

Meaning:

- Chrome CDP is not reachable, invalid, not Chrome, or no Gemini page is attached.

Next checks:

1. Use the UI `Start Chrome` button.
2. Confirm endpoint is `http://127.0.0.1:9222` unless intentionally changed.
3. Confirm the browser has a Gemini tab at `https://gemini.google.com/...`.
4. Check `sidecars/gemini-browser/src/cdp-endpoint.ts`.
5. Check `sidecars/gemini-browser/src/cdp-pages.ts`.

Do not loosen endpoint validation to support remote CDP. That would expose a full browser-control channel.

### Error: `Unexpected Gemini sidecar resume response`

Meaning:

- Rust expected a provider status response but got another sidecar response type, or the protocol shape changed.

Next checks:

1. Compare `src-tauri/src/gemini_browser/types.rs` with `sidecars/gemini-browser/src/protocol.ts`.
2. Ensure `resume` includes `browser_profile_dir` and optional `browser_config` on both sides.
3. Run sidecar typecheck and unit tests.
4. Run Rust `gemini_browser` tests.

### Error: `Gemini sidecar resume protocol is outdated after restart`

Meaning:

- Rust restarted the sidecar after seeing an old `ack`-style resume response, but the replacement still did not return provider status.

Next checks:

1. Rebuild the sidecar: `npm.cmd run test:gemini-browser-sidecar:build`.
2. In packaged scenarios, rebuild the sidecar binary: `npm.cmd run build:gemini-browser-sidecar`.
3. Confirm the Tauri app is launching the expected sidecar path in `sidecar_launch.rs`.

### Run stuck at `queued`

Meaning:

- Rust created the run record but the Apalis worker did not mark it running yet,
  the sidecar flow did not reach a final result, or the Settings panel has not
  pulled the terminal run-log state yet.

Next checks:

1. Inspect `<app-data>/gemini-browser/runs/<run_id>/result.json`.
2. Inspect the Apalis `Jobs` row in the app database (`extractum.db`) for the
   matching `idempotency_key = run_id`:
   - `run_at -> lock_at` is queue pickup latency.
   - `lock_at -> done_at` is worker/sidecar/Gemini execution time.
   - A large `run_at -> lock_at` gap means the worker did not pick up the job
     promptly. Verify that both enqueue and worker paths use
     `gemini_browser_queue_config()` rather than Apalis default polling.
3. Check whether `mark_running()` was reached in `jobs.rs`.
4. Check whether `sidecar::send_single()` returned or threw.
5. Look for a hung wait in `adapter.ts`:
   - composer wait: 30 seconds
   - send wait: up to 75 seconds while generation-busy UI is visible, with a
     10 second idle grace
   - answer wait: 120 seconds by default
6. If the browser visibly answered but run stayed queued/running, suspect answer selector drift, a run-log update failure, or a selected-detail refresh failure.

Known Apalis gotcha:

- In `apalis-sqlite = "=1.0.0-rc.8"`, the default SQL polling strategy uses
  exponential backoff after empty polls. After the worker has been idle, a new
  interactive job can wait tens of seconds before `lock_at` is set. Gemini
  Browser intentionally overrides this with a fixed `100ms` poll interval.
  Keep the regression test `worker_picks_up_job_quickly_after_idle`.

### Run status: `needs_login`

Current MVP meaning:

- The composer was not found. This often means login is required, but it can also mean consent, account picker, region block, workspace policy, or DOM drift.

Next checks:

1. Look at the attached browser page.
2. If the page clearly has the Gemini composer, this is selector drift.
3. Update `composerCandidates` in `dom-contract.ts`.
4. Add a regression in `adapter.test.ts`.

### Run status: `needs_manual_action`, message `Send button was not found`

Meaning:

- The composer was filled, but no visible send button matched the contract.

Next checks:

1. Inspect Gemini's current send button attributes.
2. Update `sendCandidates` in `dom-contract.ts`.
3. Include localized labels. Current contract includes English `send` and Russian `Отправ`.
4. Add a regression where the new button shape is visible and clickable.

### Run status: `timeout`, message `Answer did not appear before timeout`

Meaning:

- The prompt was submitted, but answer extraction did not find non-empty answer text within 60 seconds.

Next checks:

1. If Gemini visibly answered, update `answerCandidates`.
2. Avoid broad fallback selectors that can read the composer, prompt input, buttons, or page chrome as an answer.
3. The answer text filter must exclude the prompt itself.
4. Add a regression proving the prompt/composer is not misread as an answer.

### Run status: `browser_crashed`

Meaning:

- In CDP mode, the page/context/browser/connection closed before or during the run.

Next checks:

1. Confirm the Chrome window is still open.
2. Confirm the Gemini tab was not closed.
3. Check `isClosedTargetError()` in `cdp-pages.ts`.
4. Do not map mid-run closed-target errors to generic `failed`.

## Gemini DOM Drift Playbook

Use this when Gemini changes its page design.

### Step 1: Identify Which Contract Broke

The three core contracts are:

- Composer: `composerCandidates`
- Send button: `sendCandidates`
- Answer text: `answerCandidates`

They live in:

```text
sidecars/gemini-browser/src/dom-contract.ts
```

Current selector candidates:

```text
composer:
  rich-textarea textarea
  textarea[aria-label*='prompt' i]
  [contenteditable='true']

send:
  button[aria-label*='send' i]
  button[aria-label*='Отправ' i]
  button[type='submit']

answer:
  [data-response-index]
  message-content
  article [dir='ltr']
```

Failure mapping:

- Composer not found -> `needs_login` with message `Composer was not found.`
- Send not found -> `needs_manual_action` with message `Send button was not found.`
- Answer not found -> `timeout` with message `Answer did not appear before timeout.`

### Step 2: Inspect Without Leaking Data

Prefer local/manual inspection in the dedicated Chrome profile:

1. Open Settings -> Browser Providers.
2. Select `Attach Chrome`.
3. Click `Start Chrome`.
4. Sign in or open Gemini in that dedicated profile.
5. Use Chrome DevTools to inspect the relevant element.

Avoid committing live Gemini HTML. Avoid full artifacts from personal sessions. The default `reduced` artifact mode is intentional.

If a local mock can reproduce the DOM, use that instead and save fixtures/tests in the sidecar test suite.

### Step 3: Update The Smallest Selector Contract

Rules:

- Prefer stable semantic attributes over generated classes.
- Prefer role/label/text only when labels are stable enough or localized variants are included.
- Keep selectors narrow enough that they cannot match the composer or page chrome as an answer.
- Keep existing selectors unless they are now dangerous.
- Use ordered candidates: strongest/highest-confidence selectors first.

Composer gotcha:

- Gemini may keep hidden old contenteditable nodes. `waitForFirstVisible()` intentionally scans matches from the end and requires visibility. Preserve that behavior.

Send gotcha:

- The send button may be disabled until text is inserted. Ensure the test exercises the post-fill state.

Answer gotcha:

- Do not add `main`, `section`, or broad `article` fallbacks unless there is a post-submit discriminator. Broad fallbacks can produce false `ok` by reading prompt text, composer text, or UI labels.

### Step 4: Add Regression Tests First

Add or update tests in:

```text
sidecars/gemini-browser/src/adapter.test.ts
sidecars/gemini-browser/src/cdp-pages.test.ts
sidecars/gemini-browser/src/cdp-endpoint.test.ts
```

For DOM drift, prefer an adapter test that proves:

- the new composer shape is found;
- the new send shape is clicked;
- the new answer shape is extracted;
- broken answer DOM does not return `ok`;
- hidden composer candidates are ignored;
- localized send labels still work.

### Step 5: Run Verification

Fast sidecar loop:

```powershell
npm.cmd run test:gemini-browser-sidecar:typecheck
npm.cmd run test:gemini-browser-sidecar:unit
npm.cmd run test:gemini-browser-sidecar:build
```

Frontend/Rust loop when UI, commands, or protocol changed:

```powershell
npm.cmd run test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-provider-panel.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
npm.cmd run check
```

Manual smoke:

1. `npm.cmd run tauri dev`
2. Settings -> Browser Providers.
3. Select `Attach Chrome`.
4. Click `Start Chrome`.
5. Log in to Gemini if needed.
6. Click `Resume`.
7. Send: `Reply with one short sentence confirming the browser provider is connected.`
8. Confirm run status `ok` and non-empty response text.

## Protocol Debugging

The Rust and TypeScript sidecar protocol must stay aligned.

Rust protocol:

```text
src-tauri/src/gemini_browser/types.rs
```

TypeScript protocol:

```text
sidecars/gemini-browser/src/protocol.ts
```

Commands:

- `status`
- `open_browser`
- `send_single`
- `resume`
- `stop`

Important fields:

- `browser_profile_dir` is required for `status`, `open_browser`, `send_single`, and `resume`.
- `browser_config` carries provider mode and CDP endpoint.
- `send_single` also carries `request` and `artifact_dir`.

If a protocol change is made, update both sides and run:

```powershell
npm.cmd run test:gemini-browser-sidecar:typecheck
npm.cmd run test:gemini-browser-sidecar:unit
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
```

## Prompt-Pack Runtime Provider

YouTube Summary runs can use `runtimeProvider: "api"` or
`runtimeProvider: "gemini_browser"`.

- `api` keeps using the LLM profile scheduler and stores
  `provider_profile_id` plus the selected model on `prompt_pack_runs`.
- `gemini_browser` stores `runtime_provider = 'gemini_browser'` and an optional
  `browser_provider_config_json` snapshot on `prompt_pack_runs`.
- Browser-backed stages show Prompt Pack progress events, but their
  `queuePosition` stays `null` because they do not enter the API scheduler.
  They still pass through the Gemini Browser Apalis queue described above.
- Browser answers are accepted only after the existing Gemini Browser result
  converter approves them. `ready`, empty, non-ok, manual-action, and
  `timeout_latest` partial-risk answers fail closed.

The Browser Provider is not modeled as a new `llm::ProviderKind`. Prompt-pack
routing formats the same staged `LlmChatRequest` messages into one browser
prompt, sends it through `gemini_browser::send_single_prompt`, then returns to
the shared prompt-pack validation, repair, projection, and result persistence
path.

## Sidecar Launch And Packaging

Development usually launches:

```text
sidecars/gemini-browser/dist/index.js
```

The packaged app uses the bundled sidecar binary:

```text
src-tauri/binaries/gemini-browser-sidecar-<target>
```

Scripts:

```powershell
npm.cmd run test:gemini-browser-sidecar
npm.cmd run build:gemini-browser-sidecar
npm.cmd run check:gemini-browser-sidecar-binary
npm.cmd run build:tauri-prereqs
npm.cmd run smoke:gemini-browser-sidecar:node
npm.cmd run smoke:gemini-browser-sidecar:binary
npm.cmd run smoke:gemini-browser-sidecar:playwright:node
npm.cmd run smoke:gemini-browser-sidecar:playwright:binary
```

If the sidecar works in Node but not packaged:

1. Check `src-tauri/src/gemini_browser/sidecar_launch.rs`.
2. Check `src-tauri/tauri.conf.json` `bundle.externalBin`.
3. Rebuild the sidecar binary.
4. Run binary smoke.
5. If Playwright import/browser launch fails only in binary mode, focus on packaging rather than DOM selectors.

## Current Verification Baseline

The 2026-06-21 implementation audit used these automated checks:

```powershell
npm.cmd run test:gemini-browser-sidecar
npm.cmd run test -- --run src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel-state.test.ts src/lib/gemini-browser-provider-panel.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib worker_picks_up_job_quickly_after_idle
npm.cmd run check:gemini-browser-sidecar-binary
npm.cmd run check
npm.cmd run smoke:gemini-browser-sidecar:node
npm.cmd run smoke:gemini-browser-sidecar:binary
git diff --check
```

These checks prove the sidecar protocol, extractor tests, frontend inspector
helpers, Svelte type checking, Rust Gemini-browser module tests, packaged binary
presence, and basic JSONL sidecar startup. They do not prove current live Gemini
DOM behavior or Google account/login state. Run the manual smoke below when a
change depends on the live Gemini web UI.

## Artifact Rules

`sidecars/gemini-browser/src/artifacts.ts` writes failure artifacts.

Reduced mode:

- writes `telemetry.json`;
- does not write full page HTML;
- does not write screenshot.

Full mode:

- may write `page.html`;
- may write `page.png`;
- should be reserved for mock/local debugging, not live personal Google sessions.

Artifact failures should not hide the original provider failure. If artifact write behavior changes, preserve `artifact_write_error` instead of throwing over the run result.

## Security Rules

Do not weaken these unless there is a reviewed design update:

- CDP attach accepts loopback-only HTTP endpoints.
- No remote CDP hosts.
- No credentials in CDP URLs.
- No CDP path/query/hash.
- Use a dedicated Chrome profile for Browser Provider work.
- Do not ask users to attach their everyday personal Chrome profile.
- Do not save live full Gemini HTML or screenshots by default.
- Do not let `stop()` close user-controlled Chrome in CDP mode.

## Common Repair Recipes

### Add a New Gemini Composer Selector

1. Add a failing test in `adapter.test.ts`.
2. Add the selector to `composerCandidates`.
3. Keep broad `[contenteditable='true']` below stronger candidates.
4. Run sidecar tests.

### Add a New Send Button Selector

1. Add a failing test for the new button shape.
2. Add the selector to `sendCandidates`.
3. Include localization if the label is user-language dependent.
4. Ensure the test fills composer text before expecting send.

### Add a New Answer Selector

1. Add a failing test with the new answer DOM.
2. Add the selector to `answerCandidates`.
3. Add a negative test proving composer/prompt text does not become the answer.
4. Avoid broad page-level fallbacks.

### Add a New Manual Action

1. Update `src/lib/types/gemini-browser.ts`.
2. Update `src-tauri/src/gemini_browser/types.rs`.
3. Update `sidecars/gemini-browser/src/protocol.ts` if stronger TS typing is needed.
4. Update `src/lib/gemini-browser-provider-panel-contract.ts`.
5. Add frontend contract tests.
6. Add Rust serialization tests.

### Fix CDP Endpoint Behavior

1. Start in `sidecars/gemini-browser/src/cdp-endpoint.ts`.
2. Add negative validation tests first.
3. Keep loopback-only behavior.
4. Mirror any Rust-side launch validation in `src-tauri/src/gemini_browser/cdp_chrome.rs`.

## Known Limitations

- `needs_login` can mean login, consent, account picker, policy block, region block, or composer selector drift.
- CDP `status()` can report configured-but-not-attached until `Resume` attaches.
- CDP mode requires an existing browser context; it should not create a new context because that may lose the user's login state.
- `Resume` in CDP mode does not create a Gemini tab.
- Answer extraction is DOM-based, not network-based.
- Successful stable runs normally store the result JSON and inline debug
  summary, but not full HTML, screenshot, telemetry, or answer-extraction
  artifact files. Failure and non-stable paths capture more local evidence.
- Browser Provider prompt-pack execution routing currently covers YouTube
  Summary, not every prompt pack.
- Retry, re-run, search/export/compare, retention, and graceful cancel controls
  are not implemented yet.
- The current provider does not implement the full research resilience matrix
  inside the production sidecar.

## Before Declaring A Browser Provider Fix Done

Run the smallest relevant automated checks and one manual smoke when the change affects real browser behavior.

Minimum for DOM selector changes:

```powershell
npm.cmd run test:gemini-browser-sidecar:typecheck
npm.cmd run test:gemini-browser-sidecar:unit
npm.cmd run test:gemini-browser-sidecar:build
```

Minimum for frontend or protocol changes:

```powershell
npm.cmd run test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
```

Manual smoke is required when:

- selectors changed;
- CDP attach behavior changed;
- sidecar launch behavior changed;
- run lifecycle or artifact behavior changed;
- the user reported a live Gemini behavior that automated mocks cannot reproduce.
