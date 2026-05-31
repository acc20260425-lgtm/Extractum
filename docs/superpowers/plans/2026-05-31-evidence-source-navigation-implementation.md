# Evidence Source Navigation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Evidence `Show in source` a scoped, returnable navigation flow: the selected evidence opens the correct saved snapshot or live source page, loads around the selected row, highlights it once, and exposes `Back to evidence` only for sessions entered from Evidence.

**Architecture:** Add a small pure helper module for Evidence-to-Source state, focus target mapping, request identity, and loaded-data matching. Keep `/analysis/+page.svelte` as the orchestrator that creates source return context, rejects unsupported targets before navigation, applies focused load responses only while the request still matches current route state, and passes transient highlight tokens to existing readers. Readers consume highlight tokens locally so rerenders do not replay animations.

**Tech Stack:** Svelte 5, TypeScript, Vitest, existing Tauri API wrappers, existing analysis workspace and source reader components.

---

## Scope

Implement only Evidence -> Source navigation and return. Do not add URL deep links, cross-run persistence of highlight tokens, new virtualizer APIs, or broad source-browser redesign.

The slice must preserve the existing Saved Runs affordance work:

- Saved snapshots remain preferred when `evidenceSourceActionDecision(...)` returns `run_snapshot`.
- Live source remains a fallback only when a live scope exists and the decision allows it.
- `Back to run snapshot` remains a source-basis action and must stay visually and semantically separate from `Back to evidence`.

## Files

Modify:

- `src/lib/analysis-evidence-source-navigation.ts`
- `src/lib/analysis-evidence-source-navigation.test.ts`
- `src/lib/analysis-workspace-state.ts`
- `src/lib/analysis-workspace-state.test.ts`
- `src/lib/source-reader-model.ts`
- `src/lib/source-reader-model.test.ts`
- `src/routes/analysis/+page.svelte`
- `src/lib/analysis-source-readers.test.ts`
- `src/lib/analysis-source-readers-route.test.ts`
- `src/lib/analysis-report-canvas.test.ts`
- `src/lib/analysis-redesign-workflow-scenarios.test.ts`
- `src/lib/components/analysis/report-source-surface.svelte`
- `src/lib/components/analysis/source-browser-shell.svelte`
- `src/lib/components/analysis/telegram-timeline-reader.svelte`
- `src/lib/components/analysis/youtube-transcript-reader.svelte`
- `src/lib/components/analysis/snapshot-items-view.svelte`
- `src/lib/components/analysis/snapshot-group-sources-view.svelte`
- `src/lib/components/analysis/source-group-sources-view.svelte`
- `src/lib/components/analysis/source-browser-shell.test.ts`
- `src/lib/analysis-run-companion-route.test.ts`
- `src/lib/analysis-route-effects.test.ts`

Do not modify:

- Backend APIs, unless a focused-load contract is found to be impossible with current DTOs.
- Smoke fixtures for Saved Runs; that slice is complete.

---

## Task 1: Add Pure Navigation Helpers With TDD

### Red

- [ ] Create `src/lib/analysis-evidence-source-navigation.test.ts`.
- [ ] Add failing tests for the state and mapping logic before creating the helper implementation.

Test cases:

- [ ] `canonicalEvidenceTraceRef(highlightedRef, trace.ref)` returns `highlightedRef` when present and falls back to `trace.ref`.
- [ ] `EvidenceSourceScope` distinguishes single source from source-group member:

```ts
expect(sourceScopesEqual(
  { kind: "group_member", groupId: 10, sourceId: 7 },
  { kind: "group_member", groupId: 11, sourceId: 7 },
)).toBe(false);
```

- [ ] `sourceReturnContextIsActive(...)` is false when any of run id, source scope, source basis, or selected trace ref changes.
- [ ] `pendingFocusMatchesCurrent(...)` is false before page/items assignment when request id or current route state changes.
- [ ] `evidenceHighlightMatchesCurrent(...)` is false when source scope or basis changes.
- [ ] `focusedLiveSourceTargetForTrace(...)` maps:
  - YouTube transcript traces with `youtube_timestamp_seconds` to `{ kind: "youtube_transcript"; aroundStartMs: Math.round(seconds * 1000) }`.
  - Telegram and YouTube comment item traces with a positive `item_id` to `{ kind: "source_item"; aroundItemId }`.
  - synthetic or metadata-only traces without a positive `item_id` and without timestamp to `{ kind: "unsupported"; reason }`.
- [ ] `loadedSourceDataContainsTraceRef(...)` returns true for:
  - snapshot rows whose DTO-derived reader item has `ref === canonicalTraceRef`;
  - live source item rows using the same source item ref builder as `source-reader-model.ts`;
  - YouTube transcript segments using the same segment ref builder as `source-reader-model.ts`;
  - source-group member rows only when `groupId` and `sourceId` match.
- [ ] `loadedSourceDataContainsTraceRef(...)` returns false for successful focused loads where the returned DTO window does not contain the selected trace.

Run:

```powershell
npm.cmd run test -- src/lib/analysis-evidence-source-navigation.test.ts
```

Expected red output:

```text
FAIL src/lib/analysis-evidence-source-navigation.test.ts
Cannot find module '$lib/analysis-evidence-source-navigation'
```

### Green

- [ ] Add `src/lib/analysis-evidence-source-navigation.ts`.
- [ ] Export these types:

```ts
export type EvidenceSourceViewBasis = "run_snapshot" | "live_source";

export type EvidenceSourceScope =
  | { kind: "source"; sourceId: number }
  | { kind: "group_member"; groupId: number; sourceId: number };

export type SourceReturnContext =
  | {
      kind: "evidence";
      runId: number;
      sourceScope: EvidenceSourceScope;
      sourceViewBasis: EvidenceSourceViewBasis;
      traceRef: string;
    }
  | null;

export type PendingEvidenceSourceFocus = {
  requestId: string;
  runId: number;
  sourceScope: EvidenceSourceScope;
  sourceViewBasis: EvidenceSourceViewBasis;
  traceRef: string;
};

export type EvidenceHighlightToken = {
  tokenId: string;
  runId: number;
  sourceScope: EvidenceSourceScope;
  sourceViewBasis: EvidenceSourceViewBasis;
  traceRef: string;
  createdAt: number;
};
```

- [ ] Export pure helpers:

```ts
export function canonicalEvidenceTraceRef(
  highlightedRef: string | null | undefined,
  traceRef: string,
): string;

export function sourceScopeForEvidence(input: {
  runSourceGroupId: number | null;
  traceSourceId: number;
}): EvidenceSourceScope;

export function sourceScopesEqual(
  left: EvidenceSourceScope | null,
  right: EvidenceSourceScope | null,
): boolean;

export function sourceReturnContextIsActive(
  context: SourceReturnContext,
  current: {
    runId: number | null;
    sourceScope: EvidenceSourceScope | null;
    sourceViewBasis: EvidenceSourceViewBasis;
    selectedTraceRef: string | null;
  },
): boolean;

export function pendingFocusMatchesCurrent(
  pending: PendingEvidenceSourceFocus | null,
  current: {
    requestId: string;
    runId: number | null;
    sourceScope: EvidenceSourceScope | null;
    sourceViewBasis: EvidenceSourceViewBasis;
    selectedTraceRef: string | null;
  },
): boolean;

export function evidenceHighlightMatchesCurrent(
  token: EvidenceHighlightToken | null,
  current: {
    runId: number | null;
    sourceScope: EvidenceSourceScope | null;
    sourceViewBasis: EvidenceSourceViewBasis;
    selectedTraceRef: string | null;
  },
): boolean;
```

- [ ] Export focused-load mapping:

```ts
export type FocusedLiveSourceTarget =
  | { kind: "source_item"; aroundItemId: number }
  | { kind: "youtube_transcript"; aroundStartMs: number }
  | { kind: "unsupported"; reason: string };

export function focusedLiveSourceTargetForTrace(trace: Pick<
  AnalysisTraceRef,
  "item_id" | "youtube_timestamp_seconds" | "is_synthetic"
>): FocusedLiveSourceTarget;
```

Use `Math.round(trace.youtube_timestamp_seconds * 1000)` for transcript milliseconds. The backend transcript reader treats `around_start_ms` as a lower search boundary; integer milliseconds avoid unstable float request values.

- [ ] Export loaded-data matching:

```ts
export type LoadedEvidenceSourceData =
  | { kind: "snapshot"; items: SourceReaderItem[] }
  | { kind: "source_items"; items: SourceItem[] }
  | { kind: "youtube_transcript"; segments: YoutubeTranscriptSegment[] };

export function loadedSourceDataContainsTraceRef(
  data: LoadedEvidenceSourceData,
  canonicalTraceRef: string,
  sourceScope: EvidenceSourceScope,
): boolean;
```

Implementation rule:

- Use existing DTO fields first.
- If DTOs do not carry a ready `ref`, use shared exported builders from `source-reader-model.ts`.
- Do not duplicate raw ref string construction in route code.

- [ ] Export ref builders from `src/lib/source-reader-model.ts`:

```ts
export function liveSourceItemRef(item: Pick<SourceItem, "sourceId" | "id">) {
  return `s${item.sourceId}-i${item.id}`;
}

export function youtubeSegmentRef(segment: Pick<YoutubeTranscriptSegment, "sourceId" | "itemId" | "startMs">) {
  return `s${segment.sourceId}-i${segment.itemId}@${segment.startMs}ms`;
}
```

### Verify

Run:

```powershell
npm.cmd run test -- src/lib/analysis-evidence-source-navigation.test.ts
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected green output:

```text
Test Files  1 passed
```

and existing source reader tests remain green.

### Commit

- [ ] Commit:

```powershell
git add src/lib/analysis-evidence-source-navigation.ts src/lib/analysis-evidence-source-navigation.test.ts src/lib/source-reader-model.ts src/lib/analysis-source-readers.test.ts
git commit -m "test: cover evidence source navigation helpers"
```

---

## Task 2: Model Workspace Events Separately From Highlight Tokens

### Red

- [ ] Extend `src/lib/analysis-workspace-state.test.ts` first.

Add state transition tests:

- [ ] `show_evidence_in_source` sets:

```ts
{
  canvasMode: "source",
  companionTab: "evidence",
  sourceViewBasis: "run_snapshot" | "live_source",
  selectedTraceRef: canonicalRef,
}
```

- [ ] `return_to_evidence_review` sets:

```ts
{
  canvasMode: "report",
  companionTab: "evidence",
  selectedTraceRef: sameRef,
}
```

- [ ] `switch_source_basis_to_run_snapshot` keeps source mode but changes only `sourceViewBasis` to `"run_snapshot"`.
- [ ] `Back to evidence` is not modeled as a generic `back` action.
- [ ] Selecting a different source, selecting a source group, or opening a different run clears `selectedTraceRef`.

Run:

```powershell
npm.cmd run test -- src/lib/analysis-workspace-state.test.ts
```

Expected red output mentions missing event types.

### Green

- [ ] Modify `src/lib/analysis-workspace-state.ts`.
- [ ] Rename event type `back_to_run_snapshot` to `switch_source_basis_to_run_snapshot`.
- [ ] Add event type:

```ts
| {
    type: "return_to_evidence_review";
    traceRef: string;
  }
```

- [ ] Keep `show_evidence_in_source` as the transition that pins the companion tab to Evidence:

```ts
case "show_evidence_in_source":
  return {
    ...current,
    canvasMode: "source",
    sourceViewBasis: event.sourceViewBasis,
    companionTab: "evidence",
    selectedTraceRef: event.highlightedRef,
  };
```

- [ ] Implement `return_to_evidence_review`:

```ts
case "return_to_evidence_review":
  if (current.openRunState.kind === "none") return normalizeWorkspaceState(current);
  return {
    ...current,
    canvasMode: "report",
    companionTab: "evidence",
    selectedTraceRef: event.traceRef,
  };
```

- [ ] Update route callers from `back_to_run_snapshot` to `switch_source_basis_to_run_snapshot`.
- [ ] Update raw route/source tests that search for the old event string.

### Verify

Run:

```powershell
npm.cmd run test -- src/lib/analysis-workspace-state.test.ts
npm.cmd run test -- src/lib/analysis-redesign-workflow-scenarios.test.ts
```

### Commit

- [ ] Commit:

```powershell
git add src/lib/analysis-workspace-state.ts src/lib/analysis-workspace-state.test.ts src/routes/analysis/+page.svelte src/lib/analysis-redesign-workflow-scenarios.test.ts
git commit -m "feat: model evidence source workspace navigation"
```

---

## Task 3: Wire Route State, Request Identity, and Focused Loads

### Red

- [ ] Add focused route contract tests before changing the route.

Use existing raw route tests where behavior is currently tested by source-string contracts. Add or extend tests in:

- `src/lib/analysis-source-readers-route.test.ts`
- `src/lib/analysis-run-companion-route.test.ts`
- `src/lib/analysis-route-effects.test.ts` if it already owns async route-effect checks.

Contract checks:

- [ ] Route declares:

```ts
let sourceReturnContext = $state<SourceReturnContext>(null);
let pendingEvidenceSourceFocus = $state<PendingEvidenceSourceFocus | null>(null);
let transientSourceHighlight = $state<EvidenceHighlightToken | null>(null);
```

- [ ] `showSelectedTraceInSource()` computes `canonicalEvidenceTraceRef(decision.highlightedRef, trace.ref)`.
- [ ] Unsupported live targets are rejected before `show_evidence_in_source` is dispatched.
- [ ] `sourceReturnContext` is created before the focused load starts so failed loads can still return to Evidence.
- [ ] Focused live-source loading uses `focusedLiveSourceTargetForTrace(trace)`.
- [ ] Transcript loading passes an integer `aroundStartMs`.
- [ ] Every focused-load response checks `pendingFocusMatchesCurrent(...)` before assigning:
  - `applySnapshotPage(...)`
  - `groupLiveItemsBySource = ...`
  - `applySourceItemsPage(...)`
  - `youtubeTranscriptSegments = ...`
- [ ] Successful focused loads call `loadedSourceDataContainsTraceRef(...)` before creating a highlight token.
- [ ] Successful focused loads that do not contain the selected trace clear pending highlight state and do not fabricate rows.
- [ ] Failed focused loads leave Source mode and `sourceReturnContext` intact, clear pending/highlight state, and surface `formatAppError("loading selected source evidence", error)`.

Run:

```powershell
npm.cmd run test -- src/lib/analysis-source-readers-route.test.ts
```

Expected red output should mention missing helper imports/state names.

### Green

- [ ] Modify `src/routes/analysis/+page.svelte` imports:

```ts
import {
  canonicalEvidenceTraceRef,
  evidenceHighlightMatchesCurrent,
  focusedLiveSourceTargetForTrace,
  loadedSourceDataContainsTraceRef,
  pendingFocusMatchesCurrent,
  sourceReturnContextIsActive,
  sourceScopeForEvidence,
  type EvidenceHighlightToken,
  type PendingEvidenceSourceFocus,
  type SourceReturnContext,
} from "$lib/analysis-evidence-source-navigation";
```

- [ ] Add route state near `selectedTraceRef`:

```ts
let sourceReturnContext = $state<SourceReturnContext>(null);
let pendingEvidenceSourceFocus = $state<PendingEvidenceSourceFocus | null>(null);
let transientSourceHighlight = $state<EvidenceHighlightToken | null>(null);
let evidenceSourceFocusSequence = 0;
let sourceHighlightClearTimer: ReturnType<typeof setTimeout> | null = null;
```

- [ ] Add small local helpers:

```ts
function nextEvidenceSourceRequestId() {
  evidenceSourceFocusSequence += 1;
  return `evidence-source-${evidenceSourceFocusSequence}`;
}

function currentEvidenceSourceScope(traceSourceId: number) {
  return sourceScopeForEvidence({
    runSourceGroupId: currentRun?.source_group_id ?? null,
    traceSourceId,
  });
}

function clearSourceHighlight(tokenId?: string) {
  if (!tokenId || transientSourceHighlight?.tokenId === tokenId) {
    transientSourceHighlight = null;
  }
}
```

- [ ] Use timeout-based route cleanup with reader-local one-shot guards:

```ts
function scheduleSourceHighlightClear(tokenId: string) {
  if (sourceHighlightClearTimer) clearTimeout(sourceHighlightClearTimer);
  sourceHighlightClearTimer = setTimeout(() => {
    clearSourceHighlight(tokenId);
    sourceHighlightClearTimer = null;
  }, 1800);
}
```

Do not add `onHighlightConsumed` prop plumbing in this slice.

- [ ] Clear navigation state when route context changes:
  - `clearCurrentRunForWorkspaceSwitch()`
  - `selectSource(...)`
  - `selectGroup(...)`
  - `changeSelectedGroupSourceId(...)`
  - `changeSelectedSnapshotSourceId(...)`
  - `viewLiveSourceForOpenedRun()`
  - `backToRunSnapshot()` / renamed source-basis action
  - `focusTraceRef(...)` when the ref differs from `selectedTraceRef`

Clear function:

```ts
function clearEvidenceSourceNavigation() {
  sourceReturnContext = null;
  pendingEvidenceSourceFocus = null;
  clearSourceHighlight();
}
```

- [ ] Keep source-return context only while scoped state is active:

```ts
const activeSourceReturnContext = $derived.by(() => {
  const runId = currentRun?.id ?? null;
  const trace = selectedTrace;
  const sourceScope = trace ? currentEvidenceSourceScope(trace.source_id) : null;
  return sourceReturnContextIsActive(sourceReturnContext, {
    runId,
    sourceScope,
    sourceViewBasis: workspaceUiState.sourceViewBasis,
    selectedTraceRef,
  })
    ? sourceReturnContext
    : null;
});
```

- [ ] Implement unsupported live target UX:
  - If decision is `live_source` and `focusedLiveSourceTargetForTrace(trace)` returns `unsupported`, do not enter Source mode.
  - Set status to a concise reason, for example: `"This evidence does not map to a browsable live source row yet."`
  - Keep Evidence selected and preserve the report canvas.
  - Snapshot decisions remain allowed because snapshot loading can focus by `aroundRef`.

- [ ] Update `showSelectedTraceInSource()` flow:

```ts
const canonicalRef = canonicalEvidenceTraceRef(decision.highlightedRef, trace.ref);
const sourceScope = currentEvidenceSourceScope(trace.source_id);
const requestId = nextEvidenceSourceRequestId();

sourceReturnContext = {
  kind: "evidence",
  runId: currentRun.id,
  sourceScope,
  sourceViewBasis: decision.sourceViewBasis,
  traceRef: canonicalRef,
};
pendingEvidenceSourceFocus = {
  requestId,
  runId: currentRun.id,
  sourceScope,
  sourceViewBasis: decision.sourceViewBasis,
  traceRef: canonicalRef,
};

selectedTraceRef = canonicalRef;
dispatchWorkspaceEvent({
  type: "show_evidence_in_source",
  sourceViewBasis: decision.sourceViewBasis,
  highlightedRef: canonicalRef,
});

await loadSourcePageAroundTrace({ decision, trace, requestId, canonicalRef, sourceScope });
```

- [ ] Refactor `loadSourcePageAroundTrace(...)` to accept the request identity and canonical ref.
- [ ] Check `pendingFocusMatchesCurrent(...)` before state assignment in every awaited branch. This is required before assignment, not only before setting a highlight token.

Snapshot branch:

```ts
const page = await listAnalysisRunMessages(...);
if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, current)) return;
applySnapshotPage(run, page, false);
const focusedSnapshotItems = page.messages.map((message) =>
  analysisRunMessageToReaderItem(message, {
    sourceTitle: sourceTitleForSnapshotMessage(message.source_id),
    selectedTraceRef: canonicalRef,
  }),
);
const containsTarget = loadedSourceDataContainsTraceRef(
  { kind: "snapshot", items: focusedSnapshotItems },
  canonicalRef,
  sourceScope,
);
```

Import `analysisRunMessageToReaderItem` in the route and add a local `sourceTitleForSnapshotMessage(sourceId)` helper mirroring the existing title fallback in `ReportSourceSurface`. Do not reconstruct snapshot refs in the route.

Live source item branch:

```ts
const items = await listSourceItems(...);
if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, current)) return;
applySourceItemsPage(items, false);
const containsTarget = loadedSourceDataContainsTraceRef(
  { kind: "source_items", items },
  canonicalRef,
  sourceScope,
);
```

YouTube transcript branch:

```ts
const page = await listYoutubeTranscriptSegments({
  sourceId: trace.source_id,
  after: null,
  limit: 80,
  searchQuery: null,
  aroundStartMs: target.kind === "youtube_transcript" ? target.aroundStartMs : null,
});
if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, current)) return;
youtubeTranscriptSegments = page.segments;
const containsTarget = loadedSourceDataContainsTraceRef(
  { kind: "youtube_transcript", segments: page.segments },
  canonicalRef,
  sourceScope,
);
```

Source-group live branch:

```ts
const items = await listSourceItems(...);
if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, current)) return;
selectedGroupSourceId = trace.source_id;
groupLiveItemsBySource = { ...groupLiveItemsBySource, [trace.source_id]: items };
const containsTarget = loadedSourceDataContainsTraceRef(
  { kind: "source_items", items },
  canonicalRef,
  sourceScope,
);
```

- [ ] After a successful in-scope focused load:
  - If `containsTarget` is true, create `transientSourceHighlight` and schedule timeout cleanup.
  - If `containsTarget` is false, clear `transientSourceHighlight`, clear `pendingEvidenceSourceFocus`, and set a muted status such as `"Selected evidence was not found in the loaded source window."`
  - Never create fake rows.

- [ ] On focused load failure:
  - Keep Source mode and `sourceReturnContext`.
  - Clear `pendingEvidenceSourceFocus`.
  - Clear `transientSourceHighlight`.
  - Set `status = formatAppError("loading selected source evidence", error)`.

### Verify

Run:

```powershell
npm.cmd run test -- src/lib/analysis-source-readers-route.test.ts
npm.cmd run test -- src/lib/analysis-route-effects.test.ts
npm.cmd run test -- src/lib/analysis-redesign-workflow-scenarios.test.ts
```

### Commit

- [ ] Commit:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-source-readers-route.test.ts src/lib/analysis-route-effects.test.ts src/lib/analysis-redesign-workflow-scenarios.test.ts
git commit -m "feat: wire evidence source focused loading"
```

---

## Task 4: Add One-Shot Highlight Support to Readers

### Red

- [ ] Extend `src/lib/analysis-source-readers.test.ts` and component contract tests.

Required reader contracts:

- [ ] Every trace-capable reader accepts `highlightToken?: EvidenceHighlightToken | null`.
- [ ] Reader rows expose stable test attributes, not CSS-only state:

```svelte
data-evidence-highlighted={isEvidenceHighlighted(rowRef) ? "true" : undefined}
```

- [ ] Readers retain existing `selected` class/selection behavior.
- [ ] A local consumed-token guard prevents replay on rerender:

```ts
const consumedHighlightTokenIds = new Set<string>();
```

- [ ] Highlight effect only runs after rendered data exists.
- [ ] If target row is absent after a successful load, no row receives `data-evidence-highlighted`.

Run:

```powershell
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected red output should point at missing `highlightToken` plumbing.

### Green

- [ ] Modify `telegram-timeline-reader.svelte`:
  - Add `highlightToken` prop.
  - Add local consumed token guard.
  - Match token by `item.ref`.
  - Scroll only when a matching rendered item exists.
  - Apply `data-evidence-highlighted="true"` to the matching row while the token is current and unconsumed.

Implementation shape:

```ts
let {
  items,
  highlightToken = null,
  ...
}: {
  items: SourceReaderItem[];
  highlightToken?: EvidenceHighlightToken | null;
  ...
} = $props();

const consumedHighlightTokenIds = new Set<string>();
const highlightedRef = $derived(highlightToken?.traceRef ?? null);

function isEvidenceHighlighted(ref: string | null) {
  return !!highlightToken
    && ref === highlightToken.traceRef
    && !consumedHighlightTokenIds.has(highlightToken.tokenId);
}
```

The effect should:

```ts
$effect(() => {
  const token = highlightToken;
  if (!token || consumedHighlightTokenIds.has(token.tokenId)) return;
  if (!items.some((item) => item.ref === token.traceRef)) return;
  consumedHighlightTokenIds.add(token.tokenId);
  void scrollSelectedMessageIntoView(token.traceRef);
});
```

- [ ] Modify `youtube-transcript-reader.svelte`:
  - Add `highlightToken`.
  - Match when `group.refs.includes(highlightToken.traceRef)`.
  - Use a row-level attribute:

```svelte
data-evidence-highlighted={groupEvidenceHighlighted(group) ? "true" : undefined}
```

  - Scroll to the group using the actual visible ref. If `refBadge(group)` returns `"3 refs"`, add a separate stable attribute for every group:

```svelte
data-trace-refs={group.refs.join(" ")}
```

Do not rely on `"3 refs"` as the lookup key for evidence highlight.

- [ ] Modify `snapshot-items-view.svelte`:
  - Add `highlightToken`.
  - Match by `item.ref`.
  - Preserve filtering behavior. If the target is filtered out by local search/kind filters, do not mutate filters in this slice; the highlight silently expires.

- [ ] Modify `snapshot-group-sources-view.svelte` and `source-group-sources-view.svelte`:
  - Add `highlightToken`.
  - Match by row/item ref and current member source id.
  - Use stable `data-evidence-highlighted`.

- [ ] Modify `source-browser-shell.svelte`:
  - Add `highlightToken?: EvidenceHighlightToken | null` prop.
  - Thread it to:
    - `SnapshotGroupSourcesView`
    - `SnapshotItemsView`
    - `TelegramTimelineReader`
    - `YoutubeTranscriptReader`
    - `SourceGroupSourcesView`

- [ ] Modify `report-source-surface.svelte`:
  - Accept `highlightToken`.
  - Pass it to every `SourceBrowserShell` and direct reader surface.

### Styling

- [ ] Add a restrained highlight style in each reader or a shared class if one already exists locally:

```css
[data-evidence-highlighted="true"] {
  outline: 2px solid var(--color-accent, #4f46e5);
  outline-offset: 2px;
}
```

Use existing design tokens when available. Do not introduce a new one-note palette.

### Verify

Run:

```powershell
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts
```

### Commit

- [ ] Commit:

```powershell
git add src/lib/components/analysis src/lib/analysis-source-readers.test.ts
git commit -m "feat: highlight evidence source rows"
```

---

## Task 5: Add the Scoped Back to Evidence Affordance

### Red

- [ ] Extend `src/lib/analysis-report-canvas.test.ts` and route/source contract tests.

Contracts:

- [ ] `Back to evidence` appears only when `activeSourceReturnContext?.kind === "evidence"`.
- [ ] A selected trace ref alone does not render `Back to evidence`.
- [ ] The affordance is placed in `ReportSourceSurface`, above `SourceReaderHeader`, in a separate compact evidence-return bar.
- [ ] It is not adjacent to or visually merged with `Back to run snapshot`.
- [ ] Clicking it dispatches `return_to_evidence_review`, sets `canvasMode: "report"`, sets `companionTab: "evidence"`, and preserves the same selected trace ref.

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts
```

Expected red output should mention missing prop/event strings.

### Green

- [ ] Modify `report-source-surface.svelte` props:

```ts
import type { SourceReturnContext } from "$lib/analysis-evidence-source-navigation";

sourceReturnContext?: SourceReturnContext;
onReturnToEvidenceReview?: () => void;
```

- [ ] Render the evidence-return bar above `SourceReaderHeader`:

```svelte
{#if sourceReturnContext?.kind === "evidence"}
  <div class="evidence-return-bar" data-smoke-id="evidence-source-return">
    <Button type="button" variant="secondary" size="sm" onclick={onReturnToEvidenceReview}>
      Back to evidence
    </Button>
  </div>
{/if}
```

Use an existing icon if the local Button convention already uses icons in this surface. Keep text compact and do not add instructional copy.

- [ ] Modify route markup to pass:

```svelte
sourceReturnContext={activeSourceReturnContext}
highlightToken={transientSourceHighlight}
onReturnToEvidenceReview={returnToEvidenceReview}
```

- [ ] Add route handler:

```ts
function returnToEvidenceReview() {
  const context = activeSourceReturnContext;
  if (!context || context.kind !== "evidence") return;
  pendingEvidenceSourceFocus = null;
  clearSourceHighlight();
  selectedTraceRef = context.traceRef;
  dispatchWorkspaceEvent({
    type: "return_to_evidence_review",
    traceRef: context.traceRef,
  });
}
```

- [ ] Keep `sourceReturnContext` after returning to Evidence only if the active-context predicate still passes. Since canvas mode becomes report, the route should not pass it to Source UI until another Evidence -> Source navigation occurs.

### Verify

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts
npm.cmd run test -- src/lib/analysis-workspace-state.test.ts
```

### Commit

- [ ] Commit:

```powershell
git add src/lib/components/analysis/report-source-surface.svelte src/routes/analysis/+page.svelte src/lib/analysis-report-canvas.test.ts src/lib/analysis-workspace-state.test.ts
git commit -m "feat: add evidence source return affordance"
```

---

## Task 6: Cover End-to-End State Invariants

### Red

- [ ] Add integration-level or raw route tests for these invariants:

1. Evidence `Show in source` for a saved snapshot:
   - enters Source mode;
   - keeps companion on Evidence;
   - sets source basis to `run_snapshot`;
   - sends focused snapshot request with `aroundRef: canonicalRef`;
   - creates `sourceReturnContext`;
   - creates highlight only after the loaded page contains the ref.

2. Evidence `Show in source` for live Telegram/source items:
   - verifies `focusedLiveSourceTargetForTrace(...)` supports the trace;
   - sends `aroundItemId`;
   - checks request id before assigning `sourceItems`;
   - creates highlight only after returned DTO items contain the ref.

3. Evidence `Show in source` for YouTube transcript:
   - sends `aroundStartMs: Math.round(timestampSeconds * 1000)`;
   - checks request id before assigning `youtubeTranscriptSegments`;
   - creates highlight only after returned DTO segments contain the ref.

4. Source-group member:
   - stores `{ kind: "group_member"; groupId; sourceId }`;
   - checks request id before assigning `groupLiveItemsBySource`;
   - does not consider the same source id in a different group valid.

5. Stale focused load:
   - if route changes run/source/basis/ref before response, no page/items assignment happens.
   - This assertion must cover assignment, not only highlight token creation.

6. Focused load failure:
   - Source mode may remain visible with the formatted error;
   - `Back to evidence` remains available because `sourceReturnContext` was created before the load;
   - no highlight token is created.

7. Target absent:
   - no fake row is added;
   - no stale pending highlight remains;
   - no reader row has `data-evidence-highlighted="true"`.

8. Arbitrary Source mode:
   - if the user reaches Source through source navigation with `selectedTraceRef` already set, `Back to evidence` is not shown without `sourceReturnContext`.

### Green

- [ ] Implement missing route helpers or test adapters only as needed.
- [ ] Prefer pure helper tests for actual behavior and route/source-string contracts only for wiring boundaries.
- [ ] Keep user-facing copy short:
  - unsupported live target: `"This evidence does not map to a browsable live source row yet."`
  - absent focused row: `"Selected evidence was not found in the loaded source window."`
  - focused load failure uses existing `formatAppError(...)`.

### Verify

Run:

```powershell
npm.cmd run test -- src/lib/analysis-source-readers-route.test.ts
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts
npm.cmd run test -- src/lib/analysis-redesign-workflow-scenarios.test.ts
npm.cmd run test -- src/lib/analysis-run-companion-route.test.ts
```

### Commit

- [ ] Commit:

```powershell
git add src/lib/analysis-source-readers-route.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-workflow-scenarios.test.ts src/lib/analysis-run-companion-route.test.ts src/routes/analysis/+page.svelte
git commit -m "test: cover evidence source navigation invariants"
```

---

## Task 7: Typecheck, Full Verification, and Cleanup

- [ ] Run targeted tests again:

```powershell
npm.cmd run test -- src/lib/analysis-evidence-source-navigation.test.ts
npm.cmd run test -- src/lib/analysis-workspace-state.test.ts
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
npm.cmd run test -- src/lib/analysis-source-readers-route.test.ts
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts
npm.cmd run test -- src/lib/analysis-redesign-workflow-scenarios.test.ts
```

- [ ] Run type and repo verification:

```powershell
npm.cmd run check
npm.cmd run verify
git diff --check
```

Expected verification:

```text
check exits 0
verify exits 0
git diff --check exits 0
```

- [ ] Inspect changed files:

```powershell
git status --short
git diff --stat
```

- [ ] Confirm no unrelated user changes were reverted.
- [ ] Confirm no stale event name `back_to_run_snapshot` remains unless deliberately preserved in compatibility tests:

```powershell
rg -n "back_to_run_snapshot|return_to_evidence_review|switch_source_basis_to_run_snapshot|sourceReturnContext|pendingEvidenceSourceFocus|transientSourceHighlight" src
```

- [ ] Final commit if Task 7 required fixes:

```powershell
git add src
git commit -m "chore: verify evidence source navigation"
```

---

## Acceptance Checklist

- [ ] Evidence `Show in source` opens Source mode and keeps the companion on Evidence.
- [ ] `selectedTraceRef` and transient highlight state are separate concepts in code and tests.
- [ ] `Back to evidence` appears only for Source sessions entered through Evidence `Show in source`.
- [ ] Arbitrary Source mode with a selected trace does not show `Back to evidence`.
- [ ] `Back to evidence` returns the canvas to report mode, keeps companion on Evidence, and preserves the selected trace ref.
- [ ] `Back to evidence` is visually and semantically separate from `Back to run snapshot`.
- [ ] `switch_source_basis_to_run_snapshot` names the source-basis action explicitly.
- [ ] Saved snapshot focused loading uses `aroundRef`.
- [ ] Live Telegram and generic source item focused loading uses `aroundItemId`.
- [ ] YouTube transcript focused loading uses integer `aroundStartMs`.
- [ ] Unsupported live trace targets do not enter Source mode and show concise unavailable copy.
- [ ] Request id and current route state are checked before any focused-load page/items assignment.
- [ ] Stale focused loads do not replace the current page/items and do not apply highlight.
- [ ] Successful focused loads that omit the selected trace do not fabricate rows and do not leave stale highlight state.
- [ ] Focused load failures keep Evidence return available and do not create highlight.
- [ ] Readers consume highlight tokens once and do not replay highlight on rerender.
- [ ] Highlight behavior is testable with stable `data-evidence-highlighted` attributes.
- [ ] Current non-virtualized readers perform scroll/highlight after items are rendered; no new virtualizer API is introduced.
- [ ] Full verification passes.
