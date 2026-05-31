# Evidence To Source Navigation Design

Date: 2026-05-31
Status: ready for review

## Context

The analysis workspace already has the core ingredients for evidence-centered
source review:

- the Evidence tab can select a trace ref and call `Show in source`;
- route state tracks `selectedTraceRef`;
- saved run snapshots can load rows through `list_analysis_run_messages`;
- live Telegram and YouTube readers can load pages around item/timestamp focus;
- reader components scroll selected trace refs into view when the row is loaded.

The missing product behavior is the bounded navigation contract around that
path: after the user follows evidence into Source, Extractum should make the
targeted source row feel intentional, temporarily highlighted, and reversible.

## Goal

Make Evidence -> Source navigation clear and reliable for opened analysis runs.
When a user selects evidence and chooses `Show in source`, Source mode should
open the best available source basis, load rows around the selected evidence,
temporarily highlight the target row, and offer a concise return affordance back
to evidence review.

## Scope

This slice covers only Evidence -> Source navigation.

In scope:

- selected trace ref from the Evidence tab;
- `Show in source` action state and disabled reasons;
- run snapshot Source basis when a saved snapshot is available;
- live Source basis only when existing decision logic allows it;
- load-around behavior through existing route/API calls:
  - run snapshots use `aroundRef`;
  - live Telegram/source items use `aroundItemId`;
  - YouTube transcripts use `aroundStartMs`;
- temporary highlight styling for the target row/group after the jump;
- a return affordance from Source mode back to Evidence review for the opened
  run.

Out of scope:

- adding new Report -> Source or Chat -> Source navigation entry points;
- changing trace ref parsing or trace resolution semantics;
- changing saved-run degraded snapshot affordances;
- adding a general navigation framework or command palette;
- changing NotebookLM export, source-group exports, or media policy.

## User Experience

1. The user opens a run and selects a trace ref in the Evidence tab.
2. The Evidence detail shows `Show in source` when source navigation is
   available. If unavailable, the existing degraded saved-run reason remains
   visible through the disabled button title/status path.
3. Clicking `Show in source` switches the canvas to Source mode and keeps the
   Evidence tab selected in the companion.
4. Source mode loads a focused page around the selected evidence before the
   reader scroll/highlight effect runs.
5. The target row receives a temporary visual highlight that is stronger than
   ordinary selected state, then settles back to the normal selected style.
6. Source mode shows a compact `Back to evidence` affordance for the opened run.
   It returns the canvas to report review with the same Evidence tab and trace
   ref selected.

`Back to evidence` is tied to the navigation entry point, not merely to the
existence of a selected trace ref. If the user reaches Source mode by switching
tabs, selecting a source, restoring workspace state, or using `View live source`,
the return affordance should not appear unless that path was explicitly created
by Evidence `Show in source`.

The existing `Back to run snapshot` control remains separate. It changes source
basis from live source back to the saved run snapshot. It does not replace
`Back to evidence`.

## Source Basis Rules

The existing `evidenceSourceActionDecision` remains the source of truth:

- If no opened run or no selected trace exists, `Show in source` is unavailable.
- If `snapshotAvailability === "available"` and the snapshot probe is usable,
  navigation uses `sourceViewBasis = "run_snapshot"`.
- Terminal runs with missing, capture-failed, or probe-failed snapshot rows keep
  source navigation disabled with degraded saved-run copy.
- In-progress runs may use `sourceViewBasis = "live_source"` when existing live
  scope data exists and the current decision logic permits it.
- There is no silent fallback from unavailable run snapshots to live source for
  completed, failed, or cancelled saved runs.

## Data Flow

`TracePanel` keeps selecting refs through `onSelectTraceRef(ref)` and invoking
`onShowInSource()` for the selected ref.

The route should track a source return context separately from durable trace
selection:

```ts
type SourceReturnContext =
  | {
      kind: "evidence";
      runId: number;
      sourceId: number;
      sourceViewBasis: "run_snapshot" | "live_source";
      traceRef: string;
    }
  | null;
```

This context is created only by Evidence `Show in source`. It is cleared when
the opened run changes, the selected source/source group changes, the selected
trace ref changes away from the context ref, or the user enters Source mode
through a non-evidence path.

Workspace events for this slice should use explicit names:

- `show_evidence_in_source`: enter Source mode from Evidence review;
- `clear_source_highlight`: expire the transient highlight token;
- `return_to_evidence_review`: return the canvas to report review while keeping
  Evidence selected;
- `switch_source_basis_to_run_snapshot`: switch live source basis back to the
  saved run snapshot.

Do not use a generic `back` event name for both evidence-return and source-basis
switching. They are visually and semantically different actions.

The route handles the jump:

1. compute `evidenceSourceActionDecision`;
2. compute the canonical navigation ref;
3. set `selectedTraceRef` to that canonical ref;
4. set `sourceReturnContext`;
5. dispatch `show_evidence_in_source`;
6. load the focused source page for the decision:
   - snapshot: `listAnalysisRunMessages({ runId, after: null, limit, sourceId,
     aroundRef: canonicalTraceRef })`;
   - live Telegram/source items: `listSourceItems({ sourceId, aroundItemId:
     trace.item_id, ... })`;
   - live YouTube transcript: `listYoutubeTranscriptSegments({ sourceId,
     aroundStartMs: trace.youtube_timestamp_seconds * 1000, ... })`;
7. after the successful focused load has been applied to route state, set
   `transientSourceHighlight` and pass it to `ReportSourceSurface` and reader
   components.

Canonical ref rule:

```ts
const canonicalTraceRef = decision.highlightedRef ?? trace.ref;
selectedTraceRef = canonicalTraceRef;
aroundRef = canonicalTraceRef;
highlightToken.traceRef = canonicalTraceRef;
sourceReturnContext.traceRef = canonicalTraceRef;
```

In the current implementation `decision.highlightedRef` is expected to equal
`trace.ref`. The rule above makes that relationship explicit and gives future
trace-resolution changes one canonical value for durable selection, focused
loading, reader highlight, and return navigation.

The highlight token must be route-owned, scoped, and short-lived:

```ts
type EvidenceHighlightToken = {
  tokenId: string;
  kind: "evidence";
  runId: number;
  sourceId: number;
  sourceViewBasis: "run_snapshot" | "live_source";
  traceRef: string;
  createdAt: number;
};
```

Readers should receive the token only when it matches the current opened run,
source id, source basis, and selected trace ref. The route clears the token
after the reader has had a chance to render and animate, and it also clears the
token when the selected trace changes, the opened run changes, the user changes
source basis/source selection, or a stale focused load completes.

## Highlight Timing

Highlight is an effect of a completed focused load, not of entering Source mode
alone.

The route may track a pending focus request while the focused page is loading,
but readers should receive an active `transientSourceHighlight` only after the
route has applied the loaded page/items for that request. This avoids the common
race where Source mode renders before the target row exists.

Reader behavior:

- when the token arrives and the target row/group exists in rendered data, the
  reader scrolls to it and applies the temporary highlight once for that
  `tokenId`;
- when the target exists in virtualized data but is not mounted yet, the reader
  or virtualizer should first scroll to the target, then apply the highlight
  after it is rendered;
- when the focused load succeeds but the target ref is not present in the loaded
  data, the reader does not fabricate a row and the route expires the highlight
  request;
- re-renders, filter re-computation, or unchanged props must not replay the same
  highlight animation for an already consumed `tokenId`.

If the current non-virtualized readers cannot prove target absence themselves,
the route-level loaded page check should still clear stale highlight state after
a successful load whose items do not contain the canonical trace ref.

## Component Contract

Reader components that can display trace refs should accept the same navigation
shape:

- `selectedTraceRef`: the durable selected evidence ref;
- `highlightToken` or equivalent scoped transient highlight input;
- existing loading and `onLoadMore` props unchanged.

Components should only apply the strong transient highlight when the token
matches the item/group ref and source id currently being rendered. A matching
highlighted row/group should expose a stable class or data attribute so tests
can assert the contract without depending on CSS internals.

At minimum, this applies to:

- Telegram timeline rows;
- YouTube transcript groups;
- run snapshot item rows;
- source-group nested Telegram/YouTube readers;
- generic snapshot "other item" rows where trace refs are rendered.

## Return Affordance

Source mode should show `Back to evidence` only when:

- there is an opened run;
- `sourceReturnContext.kind === "evidence"`;
- `sourceReturnContext.runId === currentRun.id`;
- `sourceReturnContext.traceRef === selectedTraceRef`;
- the current source or focused group member still matches
  `sourceReturnContext.sourceId`;
- the current source basis still matches `sourceReturnContext.sourceViewBasis`.

Activating it should:

- switch the canvas back to report mode;
- keep the companion tab on Evidence;
- preserve the selected trace ref;
- avoid reloading the run or trace data unnecessarily.

State invariants:

```ts
show_evidence_in_source -> {
  canvasMode: "source";
  companionTab: "evidence";
  selectedTraceRef: canonicalTraceRef;
}

return_to_evidence_review -> {
  canvasMode: "report";
  companionTab: "evidence";
  selectedTraceRef: sourceReturnContext.traceRef;
}
```

This is intentionally local UI navigation. It does not change persisted
workspace state beyond the existing canvas/companion selection rules.
Persisted workspace restores should not recreate `sourceReturnContext`; return
navigation is only for the current interactive Evidence -> Source jump.

`Back to evidence` and `Back to run snapshot` should not be presented as
interchangeable neighboring "back" buttons. `Back to evidence` returns to
report/evidence review. `Back to run snapshot` changes Source mode from live
source basis to saved snapshot basis. The implementation should keep their event
names, labels, and placement distinct enough that tests can assert which action
is wired.

## Error Handling

- If loading the focused source page fails, the route surfaces the formatted
  loading error and leaves the selected evidence intact.
- If the source page loads but does not include the selected ref, the reader
  shows no highlighted row; the route should not fabricate evidence rows and
  should clear the pending/transient highlight state.
- Disabled Evidence actions continue to use sanitized/degraded saved-run copy.
- Raw backend errors must not be rendered as snapshot failure explanations
  unless they already pass through the existing formatting/sanitization boundary.

## Testing

Implementation should use targeted tests:

- pure state tests for the new return event/state transition;
- pure state or route tests proving `Back to evidence` depends on
  `sourceReturnContext`, not just `selectedTraceRef`;
- pure state tests for the `show_evidence_in_source` and
  `return_to_evidence_review` invariants:
  `canvasMode`, `companionTab`, and `selectedTraceRef`;
- route contract tests proving Evidence `Show in source` still uses
  `aroundRef`, `aroundItemId`, and `aroundStartMs`;
- route/component tests proving the canonical ref is shared by durable
  selection, focused snapshot loading, highlight token, and return context;
- component tests or raw contracts proving reader components accept and render a
  scoped transient highlight token separately from durable selection;
- tests for successful focused loads that omit the target ref: no fake row, no
  crash, and no stale pending highlight;
- tests proving an already consumed `tokenId` does not replay highlight on every
  re-render;
- workflow scenario coverage for `Show in source` -> `Back to evidence` while
  preserving `selectedTraceRef`;
- existing degraded saved-run affordance tests must keep passing.

Full smoke coverage is not required for this slice unless implementation touches
the existing deterministic smoke harness.

## Acceptance Criteria

- Evidence `Show in source` opens Source mode for the selected trace ref.
- Available run snapshots are preferred over live source for saved runs.
- Missing/capture-failed snapshot states do not silently fall back to live
  source.
- Source readers load around the selected evidence using the existing focused
  paging APIs.
- The selected evidence row/group receives a temporary highlight that is
  visually distinct from normal selected state and scoped to the current run,
  source id, source basis, and trace ref.
- The highlight effect runs after focused page/items data is applied, consumes a
  token once, and does not replay on unrelated re-renders.
- A successful focused load that does not contain the selected trace does not
  fabricate rows, does not throw, and does not leave stale highlight state.
- Source mode offers `Back to evidence` only for the active Evidence -> Source
  entry context and preserves the selected trace ref when returning.
- `Back to evidence` appears only for Source sessions entered through Evidence
  `Show in source`, not for arbitrary Source mode with a selected trace.
- Transient highlight clears when selected trace, opened run, source basis, or
  source selection changes.
- Highlight behavior is testable through a stable `data-*` attribute, not CSS
  class names only.
- `selectedTraceRef` and transient highlight state are tested as separate
  concepts.
- `Back to evidence` and `Back to run snapshot` remain separate actions with
  separate meanings.
- Tests cover state, route wiring, and component highlight/return contracts.
