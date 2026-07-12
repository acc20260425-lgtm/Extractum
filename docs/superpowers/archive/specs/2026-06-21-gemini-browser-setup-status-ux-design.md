# Gemini Browser Setup Status UX Design

## Context

The Gemini Browser Provider can already run one-off Settings prompts through a
managed Playwright browser or a user-controlled Chrome CDP session. It also has
answer-extraction hardening, an inline Run Inspector, copied diagnostics, and a
filterable Run History.

The remaining operator friction is setup and recovery. A user can see a broad
status such as `not_started`, `ready`, `needs_manual_action`, or `needs_login`,
but the UI does not yet show a clear checklist for the pieces that make the
provider usable:

- sidecar process/protocol availability;
- provider mode and CDP endpoint validity;
- Chrome/CDP reachability;
- whether Extractum is attached to Chrome;
- whether a Gemini tab is available;
- whether Gemini appears ready for a prompt;
- whether the last test run was stable, partial-risk, failed, or blocked by a
  manual action.

This design adds an actionable setup/status surface without changing prompt
execution, retry/cancel semantics, Run History persistence, or prompt-pack
runtime routing.

## Problem

The current Browser Providers panel answers "what is the current coarse
provider status?" but not "what should I do next?"

Typical failure paths require local knowledge:

- In CDP mode, the user may need to start Chrome, open Gemini, then resume.
- A reachable endpoint can still be unattached.
- An attached Chrome can still have no Gemini tab.
- A Gemini page can still need login, account selection, consent, CAPTCHA, or a
  DOM-selector repair.
- A successful-looking run can still be partial-risk through
  `answer_completion_reason === "timeout_latest"`.

The operator should not need to infer those steps from sidecar messages or open
artifact folders before trying the safe next action.

## Goals

- Add a first-class `Setup checklist` section to the Browser Providers panel.
- Turn existing provider status, selected mode, endpoint, run history, and run
  inspector facts into compact, actionable setup rows.
- Keep the first slice safe and low-blast-radius: mostly frontend derivation,
  with a small protocol/status extension only if existing signals are
  insufficient.
- Preserve current Browser Provider behavior for `Open`, `Resume`,
  `Start Chrome`, `Stop`, `Send`, Run Inspector, and Run History.
- Make the checklist useful in both managed and CDP attach modes.
- Make the status language operational: each row should say what is ready,
  what is unknown, what needs action, and what button to use next.
- Keep privacy boundaries unchanged: no full prompt, answer text, artifact
  paths, raw DOM, cookies, or account identifiers in checklist rows.

## Non-Goals

- Do not implement retry, re-run, cancel, queue controls, or active-run
  cancellation.
- Do not route prompt-pack runtime stages through Browser Provider.
- Do not create a separate Browser Provider debug screen.
- Do not read or render artifact file contents.
- Do not loosen CDP endpoint validation.
- Do not automate Google login, account selection, phone verification,
  CAPTCHA, consent, or other security/manual flows.
- Do not change answer extraction behavior.

## Proposed UX

The Browser Providers panel keeps its existing structure but inserts a setup
section between provider controls and the test prompt:

1. Provider controls.
2. Setup checklist.
3. Test prompt.
4. Run inspector.
5. Run history.

### Provider Controls

The existing provider card remains the source of mode and connection actions:

- `Managed` / `Attach Chrome` mode selector.
- CDP endpoint input in attach mode.
- `Start Chrome`, `Open`, `Resume`, and `Stop`.
- Current profile path and latest provider message.

The setup checklist references these actions instead of duplicating hidden
commands.

### Setup Checklist Rows

Each row has:

- a short label;
- a state badge;
- one sentence of diagnostic text;
- an optional action button or "View run" affordance;
- no raw paths or sensitive text.

Suggested states:

- `ready`
- `action_needed`
- `running`
- `warning`
- `failed`
- `unknown`
- `not_applicable`

Rows:

1. `Sidecar`
   - `ready` when `gemini_bridge_status` returns a provider status.
   - `failed` when loading status fails.
   - `unknown` before first refresh.
   - Action: `Refresh`.

2. `Mode`
   - `ready` for managed mode.
   - `ready` for attach mode with a syntactically local-looking endpoint.
   - `action_needed` for an empty or obviously non-local endpoint in the UI.
   - Action: focus endpoint input or switch mode.
   - This is frontend guidance only; Rust/sidecar remain the authority for
     endpoint validation.

3. `Chrome CDP`
   - `not_applicable` in managed mode.
   - `ready` when provider status is `ready` in attach mode.
   - `action_needed` when status/manual action is `start_chrome_cdp`.
   - `unknown` when attach mode status has not loaded.
   - Action: `Start Chrome` or `Resume`, chosen from the current provider
     status message.

4. `Gemini tab`
   - `ready` when provider status is `ready`.
   - `action_needed` when latest message says no Gemini tab is available or asks
     the user to open Gemini in attached Chrome.
   - `unknown` when no browser session exists.
   - Action: `Open` when creating a Gemini tab is safe; otherwise `Resume`.

5. `Gemini readiness`
   - `ready` when the selected/latest run has `composer_found: true` and
     `send_button_found: true`, or when a successful stable run exists.
   - `action_needed` when provider status is `needs_login` or result manual
     action is `login`, `account_picker`, `consent`, `captcha`, or
     `unknown_modal`.
   - `warning` when no readiness run has been performed yet.
   - Action: `Send test prompt` when browser status is ready; otherwise the
     relevant provider action.

6. `Last test run`
   - `ready` for `ok + stable`.
   - `warning` for `ok + timeout_latest`.
   - `action_needed` for manual-action results.
   - `failed` for failed, timeout, blocked, or browser-crashed results.
   - `unknown` when there is no run.
   - Action: `View run`, selecting the run in Run Inspector/Run History.

### Empty And First-Run State

Before any run exists, the checklist should still guide setup:

- In managed mode, it can show `Open` as the primary next step.
- In attach mode, it can show `Start Chrome` or `Resume` depending on current
  status.
- `Gemini readiness` can explain that a test prompt is the readiness check once
  the browser is ready.

### Interaction Rules

- Checklist action buttons call existing component functions where possible:
  `refresh`, `startCdpChrome`, `openBrowser`, `resumeProvider`, and
  `sendTestPrompt`.
- `View run` selects the relevant run and scroll behavior is optional for this
  slice. If scroll is added, it should not be required for tests.
- Checklist row derivation should be pure and testable outside Svelte.
- The checklist must update after `refresh`, provider events, and test prompt
  completion.

## Data Model

Prefer a frontend-first model for this slice:

```ts
type GeminiBrowserSetupCheckState =
  | "ready"
  | "action_needed"
  | "running"
  | "warning"
  | "failed"
  | "unknown"
  | "not_applicable";

type GeminiBrowserSetupCheckAction =
  | "refresh"
  | "start_chrome"
  | "open"
  | "resume"
  | "send_test"
  | "view_run"
  | "focus_endpoint";

interface GeminiBrowserSetupCheck {
  id:
    | "sidecar"
    | "mode"
    | "chrome_cdp"
    | "gemini_tab"
    | "gemini_readiness"
    | "last_test_run";
  label: string;
  state: GeminiBrowserSetupCheckState;
  message: string;
  action: GeminiBrowserSetupCheckAction | null;
  runId?: string | null;
}
```

Inputs should come from existing state:

- `GeminiBrowserProviderStatus | null`
- selected `GeminiBrowserProviderMode`
- current CDP endpoint text
- latest loaded runs
- selected/active run
- current `busy` flag
- current status/load error message if status failed

Implementation can live in a new helper module if the existing
`gemini-browser-run-inspector.ts` starts to mix concerns too much. Preferred
home:

```text
src/lib/gemini-browser-setup-status.ts
```

## Backend And Protocol Impact

The first implementation should avoid a new Tauri command unless tests show the
derived model cannot be made reliable from current state.

Allowed small additions if needed:

- Add optional fields to `GeminiBrowserProviderStatus` such as
  `setup_checks` only after a concrete missing signal is identified.
- Add sidecar status facts only if they can be computed without opening live
  Gemini artifacts or changing browser behavior.

Do not change:

- run log shape unless optional backward-compatible fields are required;
- sidecar command names;
- answer extraction contracts;
- prompt-pack runtime.

## Error Handling

- If `gemini_bridge_status` fails, show the checklist with `Sidecar: failed`
  and preserve the rest as `unknown` or locally derived.
- If `gemini_bridge_list_runs` fails, setup rows that depend on history should
  become `unknown`, while provider connection rows remain usable.
- If a checklist action fails, reuse the existing `message` display pattern and
  let the next refresh re-derive checklist state.
- Older run records without `debug_summary` must remain valid; they should
  produce `unknown` or coarse run status rows, not crashes.

## Privacy And Security

- Checklist rows can show sanitized provider messages, status names, and run
  IDs.
- Checklist rows must not show:
  - full prompt text;
  - answer text;
  - raw artifact paths;
  - raw URLs with query/hash;
  - raw DOM;
  - screenshots;
  - cookies;
  - account identifiers.
- CDP guidance must keep the loopback-only boundary and dedicated-profile
  recommendation.
- Manual Google security flows remain user-controlled in Chrome.

## Testing Strategy

Add pure helper tests for:

- managed mode initial state;
- attach mode with empty, default, and invalid-looking endpoints;
- `start_chrome_cdp` manual action mapping to `Chrome CDP: action_needed`;
- ready provider status mapping to `Chrome CDP` and `Gemini tab` ready;
- no run / stable run / `timeout_latest` run / failed run / manual-action run;
- old run records without debug summary;
- checklist action selection.

Add Svelte source-contract tests for:

- `Setup checklist` section label;
- all six row labels;
- state badge rendering;
- action handler wiring for `Start Chrome`, `Open`, `Resume`, `Send test`, and
  `View run`;
- no raw artifact path rendering.

Run the relevant checks:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-setup-status.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
git diff --check
```

If backend/protocol changes become necessary, also run:

```powershell
npm.cmd run test:gemini-browser-sidecar
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
```

Manual validation:

1. Managed mode, no browser opened: checklist should point to `Open`.
2. Attach mode, no Chrome: checklist should point to `Start Chrome`.
3. Attach mode after Chrome start but before attach: checklist should point to
   `Resume`.
4. Attached Chrome with no Gemini tab: checklist should point to `Open` or
   opening Gemini.
5. Ready browser with no test run: checklist should point to `Send test`.
6. Stable test run: last test row should be `ready`.
7. Existing manual-action or failed run: last test row should be warning/action
   oriented and `View run` should select the correct run.

## Acceptance Criteria

- Browser Providers settings includes a `Setup checklist` between provider
  controls and test prompt.
- The checklist contains `Sidecar`, `Mode`, `Chrome CDP`, `Gemini tab`,
  `Gemini readiness`, and `Last test run`.
- Each row has a state, short message, and optional safe action.
- Checklist actions reuse existing provider commands and do not add retry/cancel
  semantics.
- The checklist works in managed and attach modes.
- The checklist tolerates missing status, missing run history, and older run
  DTOs.
- The checklist does not expose prompt text, answer text, artifact paths, raw
  DOM, screenshots, cookies, or account identifiers.
- Automated helper tests cover row derivation and action selection.
- Existing Run Inspector and Run History behavior remains intact.

## Future Work

- Dedicated backend `health_check` command if frontend derivation proves too
  indirect.
- Retry/re-run/cancel controls after setup recovery is understandable.
- Search/export/compare tools for Run History.
- Prompt-pack runtime routing through Browser Provider after setup and run
  lifecycle controls are stable.
