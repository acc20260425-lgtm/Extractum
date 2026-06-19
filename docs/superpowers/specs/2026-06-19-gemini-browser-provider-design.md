# Gemini Browser Provider Design

Date: 2026-06-19

Status: draft approved direction from design discussion.

## Goal

Extractum should support a browser-backed Gemini provider that can run Prompt
Pack work through the user's normal Gemini web chat session, not through the
Gemini API.

The provider is intended for any Prompt Pack, not only YouTube workflows. The
MVP proves the integration with a small `ask_gemini_raw` test pack and a single
prompt / single answer browser run.

The integration must keep automation visible, interruptible, auditable, and
recoverable. Extractum uses the user's browser session and subscription, but it
does not automate Google account access controls.

## Product Decisions

- Browser provider is separate from normal API-backed LLM profiles.
- First UI entry point is `Settings -> Browser Providers`.
- MVP uses one visible persistent Gemini browser profile.
- MVP uses one global Gemini queue for the whole application.
- MVP supports `single` mode first.
- MVP stores raw Gemini text in a standalone Gemini run log.
- Local parse / repair runs automatically after the raw answer is captured.
- Browser profile and authentication state live in app data and are never copied
  into run artifacts.
- Prompt text, raw answer, timestamps, status, and failure screenshots/traces
  belong to the Gemini run log.
- Extractum never attempts to pass Google login, 2FA, captcha, account picker,
  consent, subscription, or other manual account flows. It opens or focuses the
  visible browser and waits for the user to click Resume.

## Non-Goals

- No hidden headless automation for user-account actions.
- No Google credential storage.
- No automation of Google login, 2FA, captcha, consent, account picker, or
  account recovery.
- No parallel Gemini web tasks in MVP.
- No dependency on Gemini web output as trusted structured data before local
  validation.
- No replacement of existing API-backed LLM profiles.
- No session mode for long videos in the first slice.
- No full Prompt Pack editor changes in the first slice.

## Current Extractum Context

Extractum already has:

- Tauri 2 command layer and Rust backend.
- SvelteKit settings and project run screens.
- Prompt Pack run infrastructure.
- API-backed LLM profile management.
- Project run UI at `/projects/runs`.
- Settings route at `/settings`.

The Gemini browser provider should not be modeled as a normal `LlmProfile`,
because its runtime states are browser-specific:

- `login_required`
- `manual_action_required`
- `rate_limited`
- `generation_timeout`
- `response_parse_failed`
- `browser_not_running`

Those states need user-facing controls and audit artifacts that API-backed
providers do not need.

## Architecture

```text
Svelte Settings UI
  -> Tauri command layer
    -> Gemini Browser Provider runtime
      -> Node Playwright sidecar
        -> visible persistent Chromium profile
          -> https://gemini.google.com/
```

Svelte owns UI state and user intent. It does not import Playwright and does not
know Gemini DOM selectors.

Tauri owns:

- command API;
- sidecar lifecycle;
- app-data paths;
- global queue state;
- run log persistence;
- browser focus/open actions;
- event emission to the UI.

The Node sidecar owns:

- Playwright browser control;
- Gemini DOM detection and interaction;
- completion detection;
- screenshot/trace capture;
- raw answer extraction.

## Provider Interface

The first TypeScript-facing shape should stay small:

```ts
type GeminiBrowserProvider = {
  status(): Promise<GeminiBridgeStatus>;
  openBrowser(): Promise<GeminiBridgeStatus>;
  sendSingle(task: GeminiSingleTask): Promise<GeminiBridgeResult>;
  resume(runId?: string): Promise<GeminiBridgeStatus>;
  stop(runId?: string): Promise<GeminiBridgeStatus>;
  listRuns(): Promise<GeminiRunLogSummary[]>;
};
```

The backend command names can follow the same shape:

```text
gemini_bridge_status
gemini_bridge_open_browser
gemini_bridge_send_single
gemini_bridge_resume
gemini_bridge_stop
gemini_bridge_list_runs
```

## Core Types

```ts
type GeminiBridgeStatusKind =
  | "not_started"
  | "starting"
  | "ready"
  | "login_required"
  | "manual_action_required"
  | "busy"
  | "rate_limited"
  | "stopped"
  | "failed";

type GeminiBridgeStatus = {
  provider: "gemini-web";
  status: GeminiBridgeStatusKind;
  activeRunId: string | null;
  queueLength: number;
  browserVisible: boolean;
  manualAction:
    | null
    | "login"
    | "captcha"
    | "two_factor"
    | "account_picker"
    | "consent"
    | "subscription"
    | "unknown";
  message: string | null;
  lastError: string | null;
};

type GeminiSingleTask = {
  taskId: string;
  promptPackId: string;
  promptPackVersion: string;
  stageName: string;
  sourceId: string | null;
  promptText: string;
  expectedOutputSchema: string | null;
  auditContext: Record<string, unknown>;
};

type GeminiBridgeResultStatus =
  | "ok"
  | "login_required"
  | "manual_action_required"
  | "rate_limited"
  | "generation_timeout"
  | "response_parse_failed"
  | "failed";

type GeminiBridgeResult = {
  provider: "gemini-web";
  mode: "single";
  status: GeminiBridgeResultStatus;
  runId: string;
  rawText: string | null;
  parsedJson: unknown | null;
  warnings: string[];
  auditRefs: string[];
  screenshotPath: string | null;
  tracePath: string | null;
};
```

## Browser Session Model

Playwright launches a visible persistent Chromium context. The profile directory
is stored under application data, not inside the repository.

The user logs in manually on first use and whenever Google requires renewed
authentication. If Gemini is not usable, the provider reports
`login_required` or `manual_action_required`, keeps the browser visible, and the
UI shows Resume after the user completes the manual step.

Extractum must not copy cookies, browser profile files, auth state, or Google
credential material into artifacts, diagnostics, reports, or logs.

## MVP Single Mode Flow

```text
user opens Settings -> Browser Providers
  -> user clicks Open Gemini Browser
  -> provider reports ready or manual action
  -> user runs ask_gemini_raw
  -> Tauri enqueues one Gemini task
  -> sidecar opens a clean Gemini chat
  -> sidecar submits prompt
  -> sidecar waits for stable final answer
  -> sidecar extracts raw answer
  -> Tauri stores Gemini run log
  -> local parse/repair runs automatically
  -> UI shows raw answer, parse status, warnings, and artifacts
```

Single mode should always start from a clean Gemini chat to avoid context
leakage between tasks.

## Global Queue

MVP supports one active browser queue.

Reasons:

- one visible user chat session is not a safe parallel execution surface;
- Gemini web limits and UI state are not designed for high-throughput parallel
  jobs;
- a single queue makes user interruption and manual control clear.

The queue may contain pending tasks, but only one task may submit to Gemini at a
time.

## Gemini DOM Adapter

All Gemini selectors and surface assumptions live in one Node module, for
example:

```text
sidecars/gemini-browser/gemini-dom-contract.mjs
```

The adapter is responsible for:

- detecting loaded/ready state;
- detecting login and manual-action states;
- finding the prompt input;
- submitting text;
- detecting generation state;
- detecting rate-limit or error banners;
- extracting the latest assistant answer;
- capturing screenshots and traces on failure.

Locator strategy should prefer user-facing and accessibility-based locators.
Fallback selectors are allowed only inside the adapter.

## Completion Detection

Gemini web chat does not expose a public completion event. The sidecar uses a
combined detector:

- prompt submission succeeded;
- answer region started changing;
- stop/interruption control is no longer visible, or input is usable again;
- latest assistant text is stable for a configured quiet period;
- no known error, rate-limit, login, or manual-action banner is visible.

If no stable answer is detected before timeout, the result is
`generation_timeout` and includes screenshot/trace artifact references.

## Run Log Storage

MVP stores a standalone Gemini run log. It is intentionally separate from
`prompt_pack_runs` in the first slice.

The log should include:

- run id;
- task id;
- provider: `gemini-web`;
- mode: `single`;
- prompt pack id/version;
- stage name;
- source id if any;
- prompt artifact reference or prompt text;
- Gemini chat URL when available;
- submitted/started/finished timestamps;
- status;
- raw answer;
- parse/repair status;
- parsed JSON if validation succeeds;
- warnings;
- screenshot/trace artifact paths;
- sanitized error if any.

Browser profile and authentication state are not part of the run log.

An initial file-backed log is acceptable for the prototype sidecar. A later
production slice can move the same contract into SQLite if that fits the
existing run history UI better.

## Local Parse And Repair

Raw Gemini output is always accepted as an audit artifact.

Structured output is accepted only after local parse and validation against the
selected prompt-pack schema. If parsing fails, the run keeps `rawText` and marks
the parse stage as failed or needing repair.

For MVP, the parse/repair step runs automatically after `rawText` is captured.
The UI should expose both:

- raw answer;
- parsed/repair status.

## `ask_gemini_raw` Test Pack

The first test pack should be deliberately small.

Purpose:

- exercise the browser bridge without touching complex YouTube flows;
- verify prompt submission and raw answer capture;
- verify run log persistence;
- verify automatic parse/repair handoff;
- verify Settings -> Browser Providers UI states.

Suggested behavior:

```text
Input: free text prompt
Stage: ask_gemini_raw/single
Expected output: raw text required, parsed JSON optional
Result UI: show raw answer and artifact refs
```

The test pack should not require Gemini to return strict JSON in the first
slice.

## Settings UI

Add a `Browser Providers` area under `/settings`.

Initial panel:

- provider name: `Gemini Web`;
- status badge;
- Open Gemini Browser button;
- Check Status button;
- Resume button;
- Stop button;
- queue length;
- active run id;
- manual action message;
- last error;
- latest run log link or preview;
- latest screenshot/trace links when available.

The UI should not describe implementation details such as selectors or
Playwright. It should speak in user-facing terms:

- "Open Gemini";
- "Manual action required";
- "Complete login in the browser, then resume";
- "Waiting for Gemini response";
- "Raw answer saved";
- "Parsing failed; raw answer is still available".

When manual action is required, Extractum should open or focus the visible
browser window.

## Tauri And Sidecar Lifecycle

The first implementation can use a Node sidecar because Playwright support is
strongest in Node and Extractum already has Node/Vite tooling.

Design as a production component, implement as a prototype sidecar first:

- stable command contract in Tauri;
- sidecar process launched by Tauri;
- local IPC over stdin/stdout or localhost;
- lifecycle owned by Tauri;
- logs and artifacts written to app-controlled paths;
- no direct Playwright imports in Svelte.

Packaging the sidecar for release can be a later implementation slice, provided
the command and run log contracts remain stable.

## Recovery

Recovery is explicit and user-visible.

- `login_required`: focus browser; user logs in; user clicks Resume.
- `manual_action_required`: focus browser; user handles prompt; user clicks
  Resume.
- `rate_limited`: pause queue and show rate-limit state.
- `generation_timeout`: save screenshot/trace; user can retry, continue
  manually, or stop.
- `failed`: save sanitized error and artifacts.

The first MVP does not need long-session checkpoint recovery.

## Testing Strategy

Layered tests:

- TypeScript unit tests for mode/status view-model behavior.
- TypeScript tests for `ask_gemini_raw` pack selection and UI states.
- Rust tests for command input/output serialization and queue state transitions.
- Node tests for the Gemini DOM adapter against a local mock Gemini-like page.
- Optional manual Playwright smoke test against real Gemini web with a real user
  session.

The mock page should simulate:

- ready;
- login required;
- manual action required;
- generating;
- final answer;
- rate limit;
- timeout.

Real Gemini smoke tests are optional/manual because they depend on a live user
session and external UI behavior.

## Implementation Slices

### Slice 1: Contracts And Settings Shell

- Add browser-provider TypeScript types.
- Add Settings -> Browser Providers panel shell.
- Add disabled/static Gemini Web provider card.
- Add view-model tests for statuses and actions.

### Slice 2: Tauri Command Skeleton

- Add Gemini bridge command names.
- Return static/not-started status.
- Add queue status shape.
- Add frontend command wrappers.

### Slice 3: Prototype Node Sidecar

- Launch visible persistent Chromium profile.
- Navigate to Gemini.
- Detect ready/login/manual states.
- Focus/open browser from Settings.

### Slice 4: Single Prompt Run Log

- Implement `sendSingle`.
- Submit one prompt.
- Wait for stable answer.
- Save raw answer and run log.
- Show latest run log in Settings.

### Slice 5: Local Parse/Repair Handoff

- Add automatic post-capture parse/repair step.
- Keep raw answer as the authoritative audit artifact.
- Surface parse status and warnings.

### Slice 6: `ask_gemini_raw` Test Pack

- Add a small prompt pack for manual bridge testing.
- Route it through the Gemini browser provider.
- Keep output raw-first.

## Open Questions

- Should the standalone Gemini run log remain file-backed long term, or move
  into SQLite after the MVP?
- Should Browser Providers later support additional web providers with the same
  queue and run log model?
- Should `prompt_pack_runs` be able to reference a Gemini run log id after the
  MVP?
- Which app-data subdirectory should own browser profile, sidecar logs, and
  Gemini run logs?
