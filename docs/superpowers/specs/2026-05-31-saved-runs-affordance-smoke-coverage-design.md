# Saved Runs Affordance Smoke Coverage Design

> Date: 2026-05-31
> Branch: `saved-runs-affordance-smoke-coverage`
> Status: draft for review

## Goal

Add GUI smoke coverage for saved-run snapshot affordances that were implemented
in the previous slice. The smoke must exercise real Tauri UI flows for legacy
missing snapshots and capture-failed snapshots, then record the result as fresh
verification evidence.

## Context

The current `npm.cmd run smoke:analysis` harness already launches the Tauri dev
app, discovers the MCP bridge, seeds analysis redesign fixtures, runs Source
Browser and workspace parity checks, cleans fixtures, and stops the app.

The harness currently covers captured single-source and source-group run
snapshots. It does not directly assert the new degraded saved-run affordances:

- `missing_legacy` copy and badges;
- `capture_failed` copy and sanitized `snapshot_error` display;
- disabled Evidence and Chat affordances for unavailable saved snapshots;
- live-source wording that stays explicit when a saved snapshot is unavailable.

The backend fixture seed already creates `Missing Snapshot Run`, but that label
is not part of the smoke labels or steps. A capture-failed run with a
non-empty `snapshot_error` is not currently seeded as a distinct smoke fixture.

## Scope

In scope:

- extend the analysis redesign fixture seed with one capture-failed saved run
  that has a sanitized-looking `snapshot_error`;
- expose smoke labels for `Missing Snapshot Run`, `Failed Run`, `Cancelled Run`,
  and the new `Capture Failed Snapshot Run`;
- add smoke steps that open degraded saved runs from the Runs tab and assert the
  visible affordances across Runs, opened-run header details, Source, Evidence,
  and Chat surfaces;
- record a fresh smoke verification note under `docs/superpowers/verification/`
  after the smoke passes.

Out of scope:

- changing production snapshot classification or report execution behavior;
- changing the saved-run affordance helper copy unless smoke exposes a genuine
  mismatch with the approved UX;
- adding cleanup filters, repair actions, migrations, or new backend DTO fields;
- replacing pure helper/component tests with GUI smoke. The smoke is additive.

## Fixture Design

Use the existing analysis fixture infrastructure in
`src-tauri/src/analysis/fixtures.rs`.

Keep the existing `Missing Snapshot Run` as the legacy-missing fixture:

- status: `completed`;
- no `snapshot_captured_at`;
- no `snapshot_error`;
- zero `analysis_run_messages`;
- has trace data pointing at missing saved evidence.

Add a new `Capture Failed Snapshot Run` fixture:

- status: `failed`;
- result markdown exists so the opened report remains readable;
- trace data exists so Evidence can select a trace ref;
- no `snapshot_captured_at`;
- `snapshot_error` is non-empty and already sanitized for UI display;
- zero `analysis_run_messages`.

This should make `list_analysis_runs` and `get_analysis_run` expose
`snapshot_state: "capture_failed"` and a non-empty `snapshot_error` without
requiring any product code change.

Keep the existing `Failed Run` and `Cancelled Run` available to the smoke labels.
They are useful for future not-captured-before-terminal checks, but the first
smoke addition should prioritize `missing_legacy` and capture-failed-with-error
because those are the highest-risk visual affordances.

## Smoke Step Design

Extend `scripts/analysis-smoke.mjs` with a small saved-run affordance step group.

### Runs List Checks

After seeding fixtures:

1. Open the Runs tab.
2. Search `Missing Snapshot Run`.
3. Assert the row shows `Legacy snapshot missing`.
4. Search `Capture Failed Snapshot Run`.
5. Assert the row shows `Snapshot capture failed`.
6. Assert neither row exposes raw or sanitized error details in the row body.

### Missing Legacy Run Checks

Open `Missing Snapshot Run` from the Runs tab and assert:

- the opened-run header shows the legacy warning;
- `Run details` contains `Legacy run has no saved snapshot` or equivalent
  helper-derived snapshot status;
- Source mode shows legacy-missing copy and does not render a saved snapshot
  browser;
- Source mode includes the live-source clarification text when `View live source`
  is available;
- Evidence `Show in source` is disabled with a legacy snapshot reason;
- Chat is disabled with helper-derived saved-context-unavailable copy.

### Capture Failed Run Checks

Open `Capture Failed Snapshot Run` from the Runs tab and assert:

- the opened-run header shows capture-failed warning copy;
- `Run details` contains `Snapshot capture failed`;
- `Run details` contains the sanitized snapshot error;
- Source mode shows capture-failed copy and the same sanitized snapshot error;
- Source mode includes the live-source clarification text when `View live source`
  is available;
- Evidence `Show in source` is disabled with a capture-failed reason;
- Chat is disabled with helper-derived saved-context-unavailable copy.

### Live Source Explicitness

For one degraded run with a live source, click `View live source` and assert:

- the header changes to live source basis;
- the Source surface no longer presents the unavailable saved snapshot as if it
  were browsable;
- the transition does not enable saved-run Evidence or Chat follow-up for the
  degraded snapshot context.

## Helper Boundaries

Prefer small helpers in `scripts/analysis-smoke.mjs` or
`scripts/analysis-smoke-helpers.mjs` for repeated UI assertions:

- open a run and return visible text for header/source/companion areas;
- search a run row and assert row text contains or omits expected fragments;
- select the first trace ref in an opened report if Evidence requires a selected
  trace before the disabled action is visible;
- switch companion tabs by smoke id and assert disabled button reasons.

These helpers should use existing smoke primitives such as `clickBySmokeId`,
`clickRowActionByText`, `fillByLabel`, `waitForText`, and `executeJs`.

Do not add broad DOM snapshots or brittle full-page text assertions. Assert the
smallest visible fragments that prove the approved affordance is wired.

## Verification Note

After the smoke passes, create a fresh note:

`docs/superpowers/verification/2026-05-31-saved-runs-affordance-smoke.md`

The note should include:

- date and branch;
- command run: `npm.cmd run smoke:analysis`;
- first-run caveat if the cold Rust/Tauri build times out before the MCP bridge
  appears, plus the successful warmed run evidence;
- fixture summary including the new capture-failed run;
- PASS table for missing legacy, capture failed, live-source explicitness, and
  the existing Source Browser smoke steps;
- confirmation that fixtures were cleaned and the dev process was stopped.

## Testing

Use TDD for the harness and fixture changes where possible:

- Rust fixture tests should fail first for the missing capture-failed fixture
  summary or run detail assertion, then pass after seeding is updated.
- Node smoke helper tests are not required unless existing smoke helper tests
  already cover the touched helper. The main behavioral verification is the
  Tauri smoke itself.

Required verification commands:

```powershell
npm.cmd run test -- src/lib/analysis-run-snapshot-affordance.test.ts src/lib/analysis-run-companion-state.test.ts
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures
npm.cmd run smoke:analysis
npm.cmd run verify
```

If the first smoke run fails while Rust is still compiling and no MCP bridge is
found, rerun `npm.cmd run smoke:analysis` once after the build is warm. Treat a
second bridge-discovery failure as a real smoke harness issue to investigate.

## Acceptance Criteria

- `npm.cmd run smoke:analysis` includes saved-run affordance checks for missing
  legacy and capture-failed saved runs.
- Smoke fixtures include a capture-failed saved run with sanitized snapshot
  error detail.
- Runs tab shows degraded badges without exposing error detail in rows.
- Opened-run details and Source mode show helper-derived degraded copy.
- Source mode distinguishes saved snapshot unavailability from live source
  browsing.
- Evidence and Chat disabled states remain visible and reasoned for degraded
  saved snapshots.
- Fresh verification note records the smoke result.
- `npm.cmd run verify` passes after the smoke coverage changes.
