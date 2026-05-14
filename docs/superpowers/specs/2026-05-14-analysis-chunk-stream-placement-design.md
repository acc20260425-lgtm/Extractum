# Analysis Chunk Stream Placement Design

Date: 2026-05-14

## Goal

Restore the analysis chunk summary stream that was lost during the result-first redesign, without letting background chunk events steal the user's current companion tab.

## Background

The legacy analysis workspace exposed chunk summaries through `WorkspaceInspector` as a `Chunks` tab. The redesigned workspace replaced the inspector with `RunCompanionTabs` and currently exposes only:

```text
Evidence | Chat | Runs
```

The data path still exists:

- `AnalysisRunEvent.chunk_summary` still arrives from the backend.
- `applyAnalysisRunEvent` still stores chunk summaries in `LiveRunState.chunkSummaries`.
- `focusedRunChunkSummaries` still returns summaries for the focused live run.
- `analysis-run-workflow.ts` still patches legacy `inspectorMode: "chunks"` when chunk summaries arrive, but redesigned route state no longer renders `WorkspaceInspector`.

The product problem is not missing data; it is missing placement.

## Decision

Add a fourth run companion tab:

```text
Evidence | Chat | Chunks | Runs
```

`Chunks` is a run-bound companion tool, like `Evidence` and `Chat`. It belongs in `RunCompanionTabs`, not in `ReportCanvas` and not inside `Runs`.

## Behavior

### Tab Availability

`Chunks` remains visible in the companion tab list to keep layout stable.

When no run is open, the tab is disabled or explanatory. If selected through restored state or an invalid state transition, workspace normalization should move back to `Runs`.

When a run is open, `Chunks` is available.

### No Automatic Focus Stealing

Receiving a `chunk_summary` must not switch the active companion tab.

If the user is reading `Evidence`, using `Chat`, or browsing `Runs`, that tab stays active. Chunk events only update the `Chunks` tab label or badge.

This is a deliberate change from the legacy `inspectorMode: "chunks"` behavior. The old automatic inspector switch was useful when the inspector was mostly a live-run monitor; it is too disruptive in the result-first companion.

### Count And Progress Indicator

The `Chunks` tab shows a compact indicator when summaries exist for the focused/opened run.

Acceptable labels:

```text
Chunks 3/8
Chunks 3
```

The preferred label is `Chunks 3/8` when `total` is known from the latest summary, otherwise `Chunks 3`.

The indicator is informational only. It does not pulse, animate aggressively, or pull focus.

### Running Runs

For a running run, the tab shows the live chunk stream.

Empty state before the first chunk:

```text
Waiting for the first chunk summary.
```

Once summaries exist, they are shown newest or natural chunk order depending on the existing component behavior. The current `ChunkSummaries` order by chunk index is acceptable because it matches analysis progression and keeps chunk numbering stable.

### Terminal Runs

For a completed, failed, or cancelled opened run:

- if chunk summaries are still present in live-state memory, show them;
- if no summaries are available, show an honest empty state:

```text
Chunk summaries are only available while the run is streaming.
```

This feature does not introduce persisted chunk summaries. Saved run snapshots remain report/evidence/chat artifacts; chunk summaries stay a live-run stream unless a later product decision persists them.

## UI Design

Reuse `ChunkSummaries` where practical, but adapt its styling for `RunCompanionTabs`.

The companion panel already provides the outer framed surface. The chunk stream body should not render as a large nested card. It should behave like the other companion tab bodies:

- compact header or status line;
- scrollable list of chunk summaries;
- details expansion for long summaries, topics, notable points, and candidate refs;
- no page-section hero treatment;
- no nested card inside card.

Candidate refs in chunk summaries are informational in this pass. They do not need `Show in source` behavior unless implementation can add it cheaply without expanding scope. Evidence refs remain the primary source navigation path.

## State Model

Extend `CompanionTab`:

```ts
export type CompanionTab = "evidence" | "chat" | "chunks" | "runs";
```

Update workspace normalization:

- without an opened run, restored `evidence`, `chat`, or `chunks` should normalize to `runs`;
- with an opened run, `chunks` is valid.

Update run event handling:

- remove the redesigned dependency on legacy `inspectorMode: "chunks"` for UI navigation;
- do not patch `companionTab: "chunks"` when chunk summaries arrive;
- keep storing `chunk_summary` in `LiveRunState.chunkSummaries`.

The legacy `AnalysisRunInspectorMode` may remain temporarily if still required by older workflow tests or non-rendered state patches, but new tests should make clear that redesigned companion navigation is driven by `workspaceUiState.companionTab`.

## Data Flow

1. Backend emits `AnalysisRunEvent` with optional `chunk_summary`.
2. `applyAnalysisRunEvent` updates `liveRuns[runId].chunkSummaries`.
3. The route derives `focusedRunChunkSummaries(focusedLiveRun)`.
4. The route passes summaries and active/terminal run state into `RunCompanionTabs`.
5. `RunCompanionTabs` computes the `Chunks` label/count and renders `ChunkSummaries` when selected.

No new Tauri command is needed.

## Tests

Add focused frontend tests before implementation:

- `analysis-workspace-state.test.ts`
  - `CompanionTab` accepts `chunks`;
  - restored `chunks` without open run normalizes to `runs`.
- `analysis-run-workflow.test.ts`
  - chunk summary events update live run state without requiring companion tab navigation;
  - legacy `inspectorMode` expectations should be revised so the redesigned UI does not depend on auto-opening chunks.
- `analysis-run-companion-tabs.test.ts`
  - tab list includes `Chunks`;
  - `Chunks` label includes a count/total when summaries exist;
  - `RunCompanionTabs` renders `ChunkSummaries` for the selected chunks tab;
  - empty states differ between running/no chunks and terminal/no chunks.
- `analysis-run-companion-route.test.ts`
  - route passes focused chunk summaries and run active state into `RunCompanionTabs`;
  - no route-level `inspectorMode` is reintroduced.

Run Svelte autofixer for changed Svelte components before committing implementation.

## Out Of Scope

- Persisting chunk summaries into saved run records.
- Adding source navigation from chunk candidate refs.
- Moving source ingest jobs into `Runs`.
- Redesigning the full `Runs` tab.
- Runtime animation or notification toasts for chunk events.

## Acceptance Criteria

- A user can open `Chunks` for an active run and watch chunk summaries arrive.
- Incoming chunk summaries do not change the currently selected companion tab.
- The `Chunks` tab shows a compact count when summaries exist.
- Completed or terminal runs show in-memory summaries when available and a clear empty state when not.
- The implementation keeps source-basis trust rules unchanged.
- The old hidden `inspectorMode: "chunks"` path no longer represents the redesigned visible UI behavior.
