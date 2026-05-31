# Saved Runs Missing/Capture-Failed Affordances Design

Date: 2026-05-31
Branch: `saved-runs-missing-capture-affordances`
Status: active design

## Context

Saved analysis runs already expose enough snapshot metadata for the frontend to
distinguish normal captured snapshots from degraded saved-run states:

- `snapshot_state`: `captured`, `missing_legacy`, `capture_failed`, or `null`
- `snapshot_captured_at`
- `snapshot_error`
- route-owned snapshot row probing through `snapshotAvailability`

The current UI has honest fallback behavior in the important places: completed
runs with missing snapshot rows do not resolve evidence or chat against live
source data. The remaining gap is affordance quality. Users can open these runs,
but the Runs list, opened-run header, Source, Evidence, and Chat surfaces do not
consistently explain whether the saved source context is unavailable because the
run is legacy, capture failed, capture never happened before the run ended, or a
captured marker is inconsistent with missing rows.

## Goal

Make degraded saved-run snapshot states explicit across the existing opened-run
experience without redesigning the Runs tab or changing backend report
execution.

The target product behavior is:

- saved reports remain readable when source snapshots are unavailable;
- exact source browsing, evidence source resolution, and follow-up chat explain
  why saved context is unavailable;
- live source browsing remains an explicit action and is never presented as the
  saved run corpus;
- wording distinguishes `missing_legacy` from `capture_failed` where that helps
  the user.

## Non-Goals

- Do not add cleanup, repair, retry-capture, or migration workflows.
- Do not add new Runs tab filters or grouping in this slice.
- Do not change backend DTO shape unless implementation proves a current field
  is unusable.
- Do not change report execution, snapshot capture, or source browser
  semantics.
- Do not expand GUI smoke coverage in this slice.

## Approach

Use a small shared frontend snapshot affordance model.

Add a pure helper at `src/lib/analysis-run-snapshot-affordance.ts` that
accepts:

- the opened or listed run snapshot fields;
- `snapshotAvailability` when available;
- optional context such as whether the UI is rendering a Runs list row, opened
  header, Source tab, Evidence tab, or Chat tab.

The helper returns display-ready UI decisions such as:

- compact label;
- badge variant;
- short header warning;
- detailed note/error text;
- Source tab unavailable message;
- Evidence disabled reason;
- Chat disabled title/description override.

Components should render these decisions instead of duplicating snapshot-state
wording inline.

The helper should return most user-facing copy, not only a state enum. Component
code may choose where to place returned strings, but it should not locally
invent alternate explanations for the same degraded state.

## Helper Contract

The implementation should define a concrete helper contract before wiring UI
components. The expected shape is:

```ts
export type SnapshotAffordanceSurface =
  | "runs-row"
  | "opened-header"
  | "run-details"
  | "source-tab"
  | "evidence-tab"
  | "chat-tab";

export type SnapshotAffordanceState =
  | "available"
  | "legacy_missing"
  | "capture_failed_with_error"
  | "not_captured_before_terminal"
  | "capture_failed_without_error_unknown"
  | "inconsistent"
  | "verification_failed"
  | "checking"
  | "pending"
  | "unknown";

export type SnapshotAffordanceSeverity =
  | "none"
  | "info"
  | "warning"
  | "error";

export type SnapshotProbeState =
  | "available"
  | "unavailable"
  | "error"
  | "loading"
  | "unknown";

export interface SnapshotAffordanceInput {
  snapshotState: "captured" | "missing_legacy" | "capture_failed" | null;
  snapshotCapturedAt: string | null;
  snapshotError: string | null;
  probeState: SnapshotProbeState;
  runStatus: "queued" | "running" | "completed" | "failed" | "cancelled" | string;
  surface: SnapshotAffordanceSurface;
}

export interface SnapshotAffordance {
  state: SnapshotAffordanceState;
  severity: SnapshotAffordanceSeverity;
  compactLabel: string | null;
  badgeVariant: "neutral" | "info" | "warning" | "danger" | null;
  headerWarning: string | null;
  detailTitle: string | null;
  detailDescription: string | null;
  disabledReason: string | null;
  sanitizedError: string | null;
}
```

The helper module should also expose or use explicit status predicates rather
than open-coded string checks in components:

```ts
export function isActiveRunStatus(status: string): boolean;
export function isTerminalRunStatus(status: string): boolean;
```

`isActiveRunStatus` covers `queued` and `running`.
`isTerminalRunStatus` covers `completed`, `failed`, and `cancelled`. When a
copy branch needs to distinguish successful completion from a run that ended
before capture, it should still check the concrete status after using the
terminal predicate.

`SnapshotProbeState` is intentionally richer than the existing
`snapshotAvailability` value. The route already tracks enough information to
distinguish:

- rows are available;
- probing completed and found no rows;
- probing failed with an error;
- probing is loading;
- probing has not run yet.

The helper input can derive this from `snapshotAvailability`,
`loadingRunSnapshotMessages`, and `runSnapshotError`, or from an equivalent
route-owned value. The key contract is that `unavailable` and `error` remain
distinct when producing copy.

## State Semantics

The helper should use these priority rules:

1. If `snapshotAvailability === "available"`, the saved snapshot is usable.
   This remains true for failed or cancelled runs that captured rows before the
   terminal status.
2. If `snapshot_state === "missing_legacy"`, show legacy-specific copy:
   `Legacy run has no saved snapshot`.
3. If `snapshot_state === "capture_failed"` and `snapshot_error` is present,
   derive `capture_failed_with_error`, show `Snapshot capture failed`, and
   expose the sanitized error in details.
4. If `snapshot_state === "capture_failed"` without `snapshot_error` on a
   failed or cancelled terminal run, derive `not_captured_before_terminal` and
   show softer copy: `Snapshot was not captured before the run ended`.
5. If `snapshot_state === "capture_failed"` without `snapshot_error` on a run
   whose terminal-before-capture cause cannot be derived, derive
   `capture_failed_without_error_unknown`. UI copy may stay soft, but tests
   should preserve this internal distinction.
6. If `snapshot_state === "captured"` but the probe reports unavailable rows,
   show an integrity-style unavailable message:
   `Snapshot is marked captured, but saved rows are unavailable`.
7. If `snapshot_state === "captured"` but the probe failed with an error, derive
   `verification_failed`. Do not claim rows are missing. Say that the saved
   snapshot could not be verified.
8. If the run is active or the probe is still unknown/loading, preserve the
   existing checking/pending semantics.

These labels are user-facing wording constraints, not necessarily exact final
strings. Implementation may tune grammar to fit each surface, but it must keep
the distinctions above.

### Null Snapshot State Matrix

`snapshot_state === null` should be handled explicitly instead of left to each
component:

| Case | Behavior |
| --- | --- |
| active run + `snapshot_state: null` | pending/checking; no degraded Runs badge |
| completed run + `snapshot_state: null` + available probe | available wins |
| completed run + `snapshot_state: null` + unavailable probe | unknown unavailable/not captured copy; do not call it legacy |
| completed run + `snapshot_state: null` + probe error | `verification_failed`; cannot verify saved snapshot copy |
| failed/cancelled + `snapshot_state: null` + unavailable probe | not captured before run ended |
| failed/cancelled + `snapshot_state: null` + available probe | available wins |
| unknown/loading probe | checking/pending copy |

## UI Behavior

### Runs Tab

Saved-run rows should visibly identify degraded snapshot states next to the
existing status and kind badges.

Expected states:

- normal captured/available: `Snapshot available` only if the final row layout
  still has enough room and the badge does not create visual noise;
- `missing_legacy`: `Legacy snapshot missing`;
- `capture_failed` with error: `Snapshot capture failed`;
- `capture_failed` without error: `Snapshot not captured`;
- active or unknown snapshot state: no noisy degraded badge unless an existing
  pending/checking badge is already useful.

The required behavior is degraded-state visibility. A badge for normal
available snapshots is optional and should be omitted if the Runs row becomes
too dense.

No new filters or cleanup controls are added.

### Opened-Run Header

The header keeps a short warning for degraded saved context. The warning should
say that the saved report remains readable and source context is degraded, but
it should not list every tab-level consequence.

`Run details` gains explicit snapshot metadata, for example:

- `Snapshot status`
- `Snapshot captured`
- `Snapshot note` or `Snapshot error`

Only sanitized `snapshot_error` text from the backend may be displayed.

### Source Tab

When the user views `sourceViewBasis === "run_snapshot"` and the snapshot is not
available, the Source tab message should use the shared affordance model:

- legacy missing: explain that older saved runs may not have saved snapshot
  rows;
- capture failed: explain that Extractum could not save the frozen source
  context for this run;
- not captured before ended: explain that the run ended before a frozen source
  snapshot was saved;
- captured marker but missing rows: explain the stored snapshot looks
  inconsistent.

`View live source` can remain available when a live scope exists, but copy must
keep it clearly separate from the saved run corpus.

### Evidence Tab

For completed runs without usable snapshot rows, `Show in source` remains
disabled. The disabled reason comes from the shared affordance model and should
match the degraded cause. Evidence rows can still be shown as report refs; only
exact source resolution is unavailable.

### Chat Tab

Follow-up chat remains disabled when saved snapshot context is unavailable. The
disabled title/description should distinguish:

- legacy saved context missing;
- snapshot capture failed;
- snapshot was not captured before run end;
- captured marker but rows unavailable.

Do not add live-source fallback chat.

## Data Flow

No backend changes are planned.

Frontend flow:

1. `list_analysis_runs` and `get_analysis_run` provide `snapshot_state`,
   `snapshot_captured_at`, and `snapshot_error`.
2. The route continues probing snapshot rows to derive `snapshotAvailability`.
3. Components ask the shared helper for display decisions.
4. Existing action guards for Evidence and Chat continue to prevent silent live
   source replacement context.

## Error Handling

This slice explains degraded states; it does not repair them.

If `snapshot_error` is present, display it only where details are expected,
primarily opened-run details and the Source tab. Do not repeat long error text
in every tab.

Surface-specific `snapshot_error` rules:

- Runs row: never show the sanitized error text; show only the compact degraded
  badge.
- Header warning: never show long error text.
- Run details: show sanitized error text.
- Source tab unavailable state: may show sanitized error text, preferably in a
  details/collapsible area or otherwise visually secondary copy.
- Evidence disabled reason: do not show long error text.
- Chat disabled copy: do not show long error text.

If row probing succeeds and finds no rows, use unavailable-row copy. If row
probing fails, use verification-failed copy. For `captured + probe error`, say
the saved snapshot could not be verified. For `captured + unavailable`, use the
integrity-style message that the run is marked captured but saved rows are
unavailable.

## Testing

Add focused Vitest coverage for the pure helper:

- captured/available snapshot;
- `missing_legacy`;
- `capture_failed` with `snapshot_error`;
- `capture_failed` without `snapshot_error` as
  `not_captured_before_terminal`;
- `capture_failed` without `snapshot_error` as
  `capture_failed_without_error_unknown`;
- failed/cancelled run that ended before capture;
- `captured` marker with unavailable rows;
- `captured` marker with probe error as `verification_failed`;
- `snapshot_state === null` matrix cases;
- `isTerminalRunStatus` covers completed/failed/cancelled and excludes queued/running;
- active/checking/pending cases.

Update existing analysis state/component source-contract tests as needed:

- Runs tab includes a snapshot badge for saved rows;
- opened-run header exposes snapshot detail fields;
- Source/Evidence/Chat use shared affordance decisions instead of ad hoc
  missing-snapshot text;
- Chat and Evidence disabled reasons distinguish degraded causes.

Verification before completion should include focused Vitest for touched
analysis tests and `npm.cmd run verify`.

GUI smoke is not required unless the implementation changes route-level smoke
contracts or smoke fixture interactions.

## Acceptance Criteria

- Runs tab saved rows visibly identify degraded snapshot states without adding
  new filters or cleanup controls.
- Opening a legacy missing-snapshot run shows a short header warning, details
  metadata, and tab-level explanations specific to legacy missing context.
- Opening a capture-failed or not-captured run shows distinct copy from
  `missing_legacy`.
- Completed runs without usable saved snapshot rows still do not resolve
  Evidence or Chat against live source context.
- Live source browsing remains explicit and clearly labeled as live data.
- Backend report execution and snapshot capture behavior are unchanged.
- The stale Saved Runs backlog acceptance text is corrected to describe
  affordances rather than already-shipped narrowing.
