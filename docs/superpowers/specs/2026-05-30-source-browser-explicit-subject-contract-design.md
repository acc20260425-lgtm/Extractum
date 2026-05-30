# Source Browser Explicit Subject Contract Design

> Date: 2026-05-30
> Status: implemented on 2026-05-30; pending merge
> Scope: remove the legacy `source` compatibility prop from `SourceBrowserShell`.

## Summary

Remove the remaining compatibility `source` prop from `SourceBrowserShell` and
make explicit `subject` passing the only supported shell contract. This is a
small cleanup after the source browser subject model, grouped browser data
objects, live group migration, run snapshot migration, and legacy wrapper
cleanup.

This is a shell contract cleanup only. It must not change browser subjects, tab
sets, smart defaults, reconciliation, route state ownership, backend behavior,
loading semantics, sync/job callbacks, UI layout, or user-visible copy.

## Current Context

`SourceBrowserShell` is the canonical production render path for:

- live Telegram sources;
- live YouTube video sources;
- live YouTube playlist sources;
- live source groups;
- available run snapshots.

All current `SourceBrowserShell` production calls already pass an explicit
`subject`:

- live single-source browsing passes `{ kind: "source", source: currentSource }`;
- live source groups pass `{ kind: "source_group", group: currentGroup }`;
- available run snapshots pass `runSnapshotSubject`.

The shell still keeps an incremental compatibility path:

```ts
source?: Source | null;
const subject = $derived(
  explicitSubject ?? (source ? { kind: "source" as const, source } : null),
);
```

The only current shell caller using that compatibility prop is the live
single-source branch in `ReportSourceSurface`, where `source={currentSource}` is
now redundant because the same call already passes an explicit source subject.

`report-canvas.svelte` still has its own `source={currentSource}` prop for a
different component. That prop is outside this cleanup.

## Primary Invariants

- `SourceBrowserShell` requires explicit subject ownership from its caller.
- `SourceBrowserShell` does not infer a browser subject from a separate `source`
  prop.
- `sourceBrowserData` remains the live single-source data object.
- `groupBrowserData` remains the live source-group data object.
- `snapshotBrowserData` remains the run-snapshot data object.
- No group or snapshot caller passes source-only dummy props to satisfy shell
  compatibility.
- The shell remains API-free: no `$lib/api/*` imports and no `invoke` calls.

## Goals

- Remove `source?: Source | null` from `SourceBrowserShell` props.
- Remove `source = null` from `SourceBrowserShell` `$props()` destructuring.
- Replace the fallback subject derivation with an explicit-subject-only
  derivation.
- Remove redundant `source={currentSource}` from the live single-source
  `SourceBrowserShell` invocation in `ReportSourceSurface`.
- Keep `sourceBrowserData`, `groupBrowserData`, and `snapshotBrowserData` names
  and shapes unchanged.
- Keep all browser behavior unchanged.
- Update raw contract tests so the explicit subject contract is protected.

## Non-Goals

- Do not change source browser subject kinds or source browser model helpers.
- Do not change tab availability, labels, smart defaults, or reconciliation.
- Do not rename or reshape `sourceBrowserData`, `groupBrowserData`, or
  `snapshotBrowserData`.
- Do not move route state ownership into `SourceBrowserShell`.
- Do not change `ReportCanvas` props or unrelated `source={...}` props outside
  `SourceBrowserShell` invocations.
- Do not change backend APIs, Tauri commands, data loading, or persistence.
- Do not change UI layout, copy, or visual styling.

## Proposed Shell Contract

`SourceBrowserShell` should keep explicit `subject` as the only way to identify
the browser subject:

```ts
type Props = {
  subject?: SourceBrowserSubject | null;
  sourceBrowserData?: SourceBrowserData | null;
  groupBrowserData?: SourceGroupBrowserData | null;
  snapshotBrowserData?: SnapshotBrowserData | null;
  selectedTraceRef?: string | null;
  loadingItems?: boolean;
  formatTimestamp: (value: number | null) => string;
};
```

The local derivation can stay intentionally small to minimize mechanical diff:

```ts
let {
  subject: explicitSubject = null,
  sourceBrowserData = null,
  groupBrowserData = null,
  snapshotBrowserData = null,
  selectedTraceRef = null,
  loadingItems = false,
  formatTimestamp,
}: Props = $props();

const subject = $derived(explicitSubject);
```

Keeping the internal `explicitSubject` alias is acceptable for this cleanup. A
future purely mechanical cleanup can rename the local variable if that improves
readability.

## Route Wiring

`ReportSourceSurface` should continue to pass explicit subjects for all shell
invocations:

```svelte
<SourceBrowserShell
  subject={{ kind: "source", source: currentSource }}
  sourceBrowserData={{ ... }}
  {selectedTraceRef}
  {formatTimestamp}
/>
```

The live source-group and run-snapshot shell calls stay structurally unchanged
and continue to pass only their subject-specific data objects:

- `groupBrowserData={{ ... }}` for live source groups;
- `snapshotBrowserData={{ ... }}` for available run snapshots.

No shell invocation should pass `source={currentSource}` or `source={null}`.

## Testing Strategy

Focused tests should assert:

- `SourceBrowserShell` props no longer include `source?: Source | null`.
- `SourceBrowserShell` no longer derives `subject` from `source`.
- `SourceBrowserShell` still derives `sourceData`, `groupData`, and
  `snapshotData` from explicit `subject.kind`.
- `ReportSourceSurface` passes explicit `subject` in every `SourceBrowserShell`
  invocation.
- The live single-source shell invocation no longer includes
  `source={currentSource}`.
- Live group and run snapshot shell invocations do not include `source={null}` or
  `sourceBrowserData={{ ... }}`.
- `report-canvas.svelte` may still contain its unrelated `source={currentSource}`
  prop.
- `SourceBrowserShell` still imports no `$lib/api/*` modules and calls no
  `invoke`.
- Existing Telegram, YouTube video, YouTube playlist, live group, and run
  snapshot component contracts still pass.
- Existing source browser model tests remain unchanged.

Verification should include:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/source-browser-model.test.ts
npm.cmd run check
npm.cmd run verify
```

## Rollout

Land this as a narrow follow-up cleanup after `sourceBrowserData` consolidation.
If implementation finds a production `SourceBrowserShell` caller without an
explicit subject, stop and either add the explicit subject in that caller or
downgrade the slice to docs/tests only. Do not restore the legacy `source` prop
as a hidden fallback once all production callers are explicit.
