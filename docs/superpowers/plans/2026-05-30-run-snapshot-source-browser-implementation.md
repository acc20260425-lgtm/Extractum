# Run Snapshot Source Browser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Route available saved run snapshots through the shared source browser model while preserving frozen-only snapshot semantics.

**Architecture:** Extend `SourceBrowserSubject` with a canonical `run_snapshot` subject and deterministic snapshot reader-kind derivation. Add snapshot-only leaves for group sources, flat items, and metadata; keep `SourceReaderHeader` route-owned and keep pending/unavailable/checking snapshots outside the shell. Snapshot data enters `SourceBrowserShell` through one grouped `snapshotBrowserData` prop.

**Tech Stack:** Svelte 5 runes, SvelteKit, TypeScript, Vitest raw component contract tests, Tauri fixture smoke.

---

## Execution Protocol

- Start from clean `main`.
- Create the branch before Task 0:

```bash
git switch -c run-snapshot-source-browser
```

- Execute tasks in order.
- After each task:
  - mark every completed step in this plan with `[x]`;
  - run the task verification;
  - commit exactly the files listed by that task.
- Do not move `SourceReaderHeader` into `SourceBrowserShell` in this slice.
- Do not add snapshot data as many standalone shell props. Use grouped `snapshotBrowserData`.
- Do not adapt `UniversalItemsView` to `SourceReaderItem[]`. Create `SnapshotItemsView`.

## File Map

- `src/lib/source-browser-model.ts`: add snapshot subject types, snapshot tabs/defaults/reconciliation, and `deriveRunSnapshotBrowserKind`.
- `src/lib/source-browser-model.test.ts`: add snapshot subject and reader-kind derivation tests.
- `src/lib/components/analysis/snapshot-items-view.svelte`: new flat frozen-row item browser over `SourceReaderItem[]`.
- `src/lib/components/analysis/snapshot-group-sources-view.svelte`: new group snapshot sources leaf with global run-snapshot paging only.
- `src/lib/components/analysis/run-snapshot-metadata-view.svelte`: new snapshot metadata leaf using route-owned run/snapshot fields.
- `src/lib/components/analysis/source-browser-shell.svelte`: add snapshot data prop and snapshot tab bodies below the already external header.
- `src/lib/components/analysis/source-browser-shell.test.ts`: shell contract tests for snapshot leaves and grouped props.
- `src/lib/components/analysis/report-source-surface.svelte`: derive snapshot subject/kind and route available snapshots through `SourceBrowserShell`; keep pending/unavailable/checking branches unchanged.
- `src/lib/analysis-source-readers.test.ts`: reader and route raw-contract tests for snapshot browser migration.
- `src/lib/analysis-report-canvas.test.ts`: update snapshot source canvas contract tests.
- `docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md`: final implementation status update.
- `docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md`: checkbox tracking.

## Task 0: Preflight Current Snapshot Contracts

**Files:**
- Modify: `docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md`

- [x] **Step 1: Confirm the branch and clean tree**

Run:

```bash
git status --short --branch
```

Expected: branch is `run-snapshot-source-browser`; no modified files except this plan after checkboxes are edited.

- [x] **Step 2: Inspect current source and snapshot model types**

Run:

```bash
rg -n "export type SourceType|export type SourceSubtype|export interface SourceReaderItem|export interface AnalysisRunDetail|export interface AnalysisRunMessage|export interface SourceFilterOption" src/lib/types/sources.ts src/lib/types/analysis.ts src/lib/source-reader-model.ts
```

Expected findings:

```text
src/lib/types/sources.ts: SourceType and SourceSubtype unions exist.
src/lib/types/analysis.ts: AnalysisRunDetail and AnalysisRunMessage exist.
src/lib/source-reader-model.ts: SourceReaderItem and SourceFilterOption exist.
```

- [x] **Step 3: Inspect current snapshot branch and live shell props**

Run:

```bash
rg -n "sourceViewBasis === \"run_snapshot\"|runSnapshotMessages|allSnapshotReaderItems|snapshotReaderItems|<SourceGroupReader|<YoutubeTranscriptReader|<TelegramTimelineReader|type Props|groupBrowserData|SourceActivityView" src/lib/components/analysis/report-source-surface.svelte src/lib/components/analysis/source-browser-shell.svelte
```

Expected findings:

```text
report-source-surface.svelte contains the run_snapshot branch and direct snapshot readers.
report-source-surface.svelte contains runSnapshotMessages, allSnapshotReaderItems, and filtered snapshotReaderItems.
source-browser-shell.svelte contains live source/group props and SourceActivityView only for live source subjects.
```

- [x] **Step 4: Inspect design status and UI component APIs**

Run:

```bash
rg -n "^> Status:" docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md
rg -n "selected =|ariaPressed|ariaLabel|oninput|onchange|value =" src/lib/components/ui/Button.svelte src/lib/components/ui/Input.svelte src/lib/components/ui/Select.svelte
```

Expected findings:

```text
2026-05-30-run-snapshot-source-browser-design.md has status: approved design, pending implementation plan.
Button.svelte supports selected, ariaPressed, and ariaLabel props.
Input.svelte supports ariaLabel and oninput props.
Select.svelte supports value and onchange props.
```

- [x] **Step 5: Record preflight decisions**

Add this under the Task 0 step list after running the commands:

```markdown
Preflight decisions:

- `RunSnapshotBrowserSubject.sourceType` will use `SourceType | null`.
- `RunSnapshotBrowserSubject.sourceSubtype` will use `SourceSubtype | null`.
- `deriveRunSnapshotBrowserKind` will accept string-compatible route inputs so raw `AnalysisRunMessage.source_type` can be used without backend DTO changes.
- `runSnapshotMessages` is the unfiltered loaded snapshot message window; `allSnapshotReaderItems` is the unfiltered reader-row projection; `snapshotReaderItems` is source-focus filtered.
- Snapshot available branches will use `allSnapshotReaderItems` for reader-kind derivation, not the source-filtered `snapshotReaderItems`.
- `SourceReaderHeader` remains outside `SourceBrowserShell`.
- The design spec status is `approved design, pending implementation plan`; Task 5 Step 4 will replace that exact line.
- Snapshot leaf controls use the existing UI component APIs: `Button selected` plus `ariaPressed`, `Input ariaLabel/oninput`, and `Select value/onchange`.
```

Preflight decisions:

- `RunSnapshotBrowserSubject.sourceType` will use `SourceType | null`.
- `RunSnapshotBrowserSubject.sourceSubtype` will use `SourceSubtype | null`.
- `deriveRunSnapshotBrowserKind` will accept string-compatible route inputs so raw `AnalysisRunMessage.source_type` can be used without backend DTO changes.
- `runSnapshotMessages` is the unfiltered loaded snapshot message window; `allSnapshotReaderItems` is the unfiltered reader-row projection; `snapshotReaderItems` is source-focus filtered.
- Snapshot available branches will use `allSnapshotReaderItems` for reader-kind derivation, not the source-filtered `snapshotReaderItems`.
- `SourceReaderHeader` remains outside `SourceBrowserShell`.
- The design spec status is `approved design, pending implementation plan`; Task 5 Step 4 will replace that exact line.
- Snapshot leaf controls use the existing UI component APIs: `Button selected` plus `ariaPressed`, `Input ariaLabel/oninput`, and `Select value/onchange`.

- [x] **Step 6: Run whitespace and status checks**

Run:

```bash
git diff --check
git status --short --branch
```

Expected: no whitespace errors; only this plan file is modified.

- [x] **Step 7: Commit preflight**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md
git commit -m "docs: record run snapshot browser preflight"
```

## Task 1: Snapshot Browser Model

**Files:**
- Modify: `src/lib/source-browser-model.ts`
- Modify: `src/lib/source-browser-model.test.ts`
- Modify: `docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md`

- [x] **Step 1: Add snapshot model test imports**

In `src/lib/source-browser-model.test.ts`, add `deriveRunSnapshotBrowserKind` to the existing named import from `./source-browser-model`:

```ts
  deriveRunSnapshotBrowserKind,
```

Add this type import near the existing type imports:

```ts
import type { SourceReaderItem } from "./source-reader-model";
```

- [x] **Step 2: Add snapshot fixtures to model tests**

Below the existing `sourceGroup` fixture, add:

```ts
function snapshotSubject(
  readerKind: "source_group" | "telegram_timeline" | "youtube_transcript" | "generic_items",
  overrides: Partial<{
    runId: number;
    scopeType: "source" | "source_group";
    scopeLabel: string;
    sourceType: Source["sourceType"] | null;
    sourceSubtype: Source["sourceSubtype"] | null;
  }> = {},
) {
  return {
    kind: "run_snapshot" as const,
    snapshot: {
      runId: overrides.runId ?? 500,
      scopeType: overrides.scopeType ?? (readerKind === "source_group" ? "source_group" : "source"),
      scopeLabel: overrides.scopeLabel ?? "Snapshot run",
      readerKind,
      sourceType: overrides.sourceType ?? null,
      sourceSubtype: overrides.sourceSubtype ?? null,
    },
  };
}

function snapshotReaderItem(overrides: Partial<SourceReaderItem> = {}): SourceReaderItem {
  return {
    id: "snapshot:s1-i1",
    sourceId: 1,
    sourceTitle: "Snapshot source",
    externalId: "external-1",
    ref: "s1-i1",
    kind: "telegram_message",
    author: "Alice",
    publishedAt: 1710000000,
    content: "Snapshot row",
    topicLabel: null,
    replyLabel: null,
    reactionLabel: null,
    mediaCards: [],
    youtubeStartSeconds: null,
    youtubeEndSeconds: null,
    youtubeUrl: null,
    captionLabel: null,
    historyScope: "current",
    historyScopeLabel: null,
    isMigratedHistory: false,
    selected: false,
    ...overrides,
  };
}
```

- [x] **Step 3: Add failing snapshot tab/default/applicability tests**

Inside `describe("source browser model", () => {`, after the live source group tabs test, add:

```ts
  it("derives canonical tabs for run snapshot subjects", () => {
    expect(sourceBrowserTabsForSubject(snapshotSubject("source_group")).map((tab) => tab.id))
      .toEqual(["sources", "items", "metadata"]);
    expect(sourceBrowserTabsForSubject(snapshotSubject("telegram_timeline")).map((tab) => tab.id))
      .toEqual(["timeline", "items", "metadata"]);
    expect(sourceBrowserTabsForSubject(snapshotSubject("youtube_transcript")).map((tab) => tab.id))
      .toEqual(["transcript", "items", "metadata"]);
    expect(sourceBrowserTabsForSubject(snapshotSubject("generic_items")).map((tab) => tab.id))
      .toEqual(["items", "metadata"]);

    expect(smartDefaultSourceBrowserTab(snapshotSubject("source_group"))).toBe("sources");
    expect(smartDefaultSourceBrowserTab(snapshotSubject("telegram_timeline"))).toBe("timeline");
    expect(smartDefaultSourceBrowserTab(snapshotSubject("youtube_transcript"))).toBe("transcript");
    expect(smartDefaultSourceBrowserTab(snapshotSubject("generic_items"))).toBe("items");
    expect(sourceBrowserShellAppliesToSubject(snapshotSubject("generic_items"))).toBe(true);
  });
```

- [x] **Step 4: Add failing reader-kind derivation tests**

After the snapshot tab/default test, add:

```ts
  it("derives run snapshot reader kinds deterministically", () => {
    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source_group",
      sourceType: "telegram",
      sourceSubtype: null,
      snapshotReaderItems: [],
    })).toBe("source_group");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "youtube",
      sourceSubtype: "video",
      snapshotReaderItems: [snapshotReaderItem({ kind: "youtube_transcript" })],
    })).toBe("youtube_transcript");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "telegram",
      sourceSubtype: "supergroup",
      snapshotReaderItems: [snapshotReaderItem({ kind: "telegram_message" })],
    })).toBe("telegram_timeline");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "youtube",
      sourceSubtype: "video",
      snapshotReaderItems: [
        snapshotReaderItem({ kind: "youtube_transcript" }),
        snapshotReaderItem({ id: "snapshot:s1-c1", kind: "youtube_comment" }),
      ],
    })).toBe("generic_items");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "youtube",
      sourceSubtype: "video",
      snapshotReaderItems: [snapshotReaderItem({ kind: "telegram_message" })],
    })).toBe("generic_items");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "telegram",
      sourceSubtype: "supergroup",
      snapshotReaderItems: [],
    })).toBe("generic_items");
  });
```

- [x] **Step 5: Add failing snapshot reconciliation tests**

After the existing source group reconciliation test, add:

```ts
  it("reconciles run snapshot tab transitions without leaking live-only tabs", () => {
    const groupSnapshot = snapshotSubject("source_group");
    const telegramSnapshot = snapshotSubject("telegram_timeline");
    const youtubeSnapshot = snapshotSubject("youtube_transcript");
    const genericSnapshot = snapshotSubject("generic_items");
    const telegramSubject = {
      kind: "source" as const,
      source: source({ id: 3, sourceType: "telegram", sourceSubtype: "supergroup" }),
    };

    expect(reconcileSourceBrowserTab("items", groupSnapshot)).toBe("items");
    expect(reconcileSourceBrowserTab("metadata", groupSnapshot)).toBe("metadata");
    expect(reconcileSourceBrowserTab("activity", groupSnapshot)).toBe("sources");
    expect(reconcileSourceBrowserTab("comments", youtubeSnapshot)).toBe("transcript");
    expect(reconcileSourceBrowserTab("videos", telegramSnapshot)).toBe("timeline");
    expect(reconcileSourceBrowserTab("transcript", genericSnapshot)).toBe("items");
    expect(reconcileSourceBrowserTab("sources", telegramSnapshot)).toBe("timeline");
    expect(reconcileSourceBrowserTab("timeline", telegramSubject)).toBe("timeline");
    expect(reconcileSourceBrowserTab("metadata", telegramSubject)).toBe("metadata");
  });
```

- [x] **Step 6: Run model tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/source-browser-model.test.ts
```

Expected: FAIL because `run_snapshot` subject types and `deriveRunSnapshotBrowserKind` are not implemented.

- [x] **Step 7: Implement snapshot model types and helpers**

In `src/lib/source-browser-model.ts`, change the imports to include `SourceReaderItem`, `SourceType`, and `SourceSubtype`:

```ts
import type { SourceReaderItem } from "$lib/source-reader-model";
import type { AnalysisSourceGroup } from "$lib/types/analysis";
import type { Source, SourceItem, SourceJobRecord, SourceSubtype, SourceType } from "$lib/types/sources";
import type { YoutubeVideoDetail } from "$lib/types/youtube";
```

Add these types above `export type SourceBrowserSubject`:

```ts
export type RunSnapshotBrowserKind =
  | "source_group"
  | "telegram_timeline"
  | "youtube_transcript"
  | "generic_items";

export interface RunSnapshotBrowserSubject {
  runId: number;
  scopeType: "source" | "source_group";
  scopeLabel: string;
  readerKind: RunSnapshotBrowserKind;
  sourceType: SourceType | null;
  sourceSubtype: SourceSubtype | null;
}

export interface RunSnapshotBrowserKindInput {
  scopeType: string | null;
  sourceType: string | null;
  sourceSubtype: string | null;
  snapshotReaderItems: Pick<SourceReaderItem, "kind">[];
}
```

Replace `SourceBrowserSubject` with:

```ts
export type SourceBrowserSubject =
  | { kind: "source"; source: Source }
  | { kind: "source_group"; group: AnalysisSourceGroup }
  | { kind: "run_snapshot"; snapshot: RunSnapshotBrowserSubject };
```

Add this helper below `sourceTabIds`:

```ts
function snapshotTabIds(readerKind: RunSnapshotBrowserKind): SourceBrowserTabId[] {
  if (readerKind === "source_group") return ["sources", "items", "metadata"];
  if (readerKind === "telegram_timeline") return ["timeline", "items", "metadata"];
  if (readerKind === "youtube_transcript") return ["transcript", "items", "metadata"];
  return ["items", "metadata"];
}
```

Replace `sourceBrowserTabsForSubject` with:

```ts
export function sourceBrowserTabsForSubject(subject: SourceBrowserSubject): SourceBrowserTab[] {
  const ids: SourceBrowserTabId[] = subject.kind === "source_group"
    ? ["sources", "items", "metadata", "activity"]
    : subject.kind === "run_snapshot"
      ? snapshotTabIds(subject.snapshot.readerKind)
      : sourceTabIds(subject.source);

  return tabRecords(ids);
}
```

Replace `sourceBrowserShellAppliesToSubject` with:

```ts
export function sourceBrowserShellAppliesToSubject(subject: SourceBrowserSubject): boolean {
  if (subject.kind === "source_group" || subject.kind === "run_snapshot") return true;
  return sourceBrowserShellAppliesToSource(subject.source);
}
```

Replace `smartDefaultSourceBrowserTab` with:

```ts
export function smartDefaultSourceBrowserTab(input: SourceBrowserModelInput): SourceBrowserTabId {
  if (isSourceBrowserSubject(input) && input.kind === "source_group") return "sources";
  if (isSourceBrowserSubject(input) && input.kind === "run_snapshot") {
    return snapshotTabIds(input.snapshot.readerKind)[0] ?? "items";
  }
  const source = isSourceBrowserSubject(input) ? input.source : input;
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") return "transcript";
  if (source.sourceType === "youtube" && source.sourceSubtype === "playlist") return "videos";
  if (source.sourceType === "telegram") return "timeline";
  return "items";
}
```

Add this exported helper below `reconcileSourceBrowserTab`:

```ts
export function deriveRunSnapshotBrowserKind(input: RunSnapshotBrowserKindInput): RunSnapshotBrowserKind {
  if (input.scopeType === "source_group") return "source_group";
  if (input.snapshotReaderItems.length === 0) return "generic_items";

  const kinds = new Set(input.snapshotReaderItems.map((item) => item.kind));
  if (
    input.sourceType === "youtube" &&
    input.sourceSubtype === "video" &&
    kinds.size === 1 &&
    kinds.has("youtube_transcript")
  ) {
    return "youtube_transcript";
  }
  if (
    input.sourceType === "telegram" &&
    kinds.size === 1 &&
    kinds.has("telegram_message")
  ) {
    return "telegram_timeline";
  }
  return "generic_items";
}
```

- [x] **Step 8: Run model tests and verify they pass**

Run:

```bash
npm.cmd run test -- src/lib/source-browser-model.test.ts
```

Expected: PASS.

- [x] **Step 9: Commit model task**

Run:

```bash
git add src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md
git commit -m "feat: add run snapshot browser model"
```

## Task 2: Snapshot Leaves

**Files:**
- Create: `src/lib/components/analysis/snapshot-items-view.svelte`
- Create: `src/lib/components/analysis/snapshot-group-sources-view.svelte`
- Create: `src/lib/components/analysis/run-snapshot-metadata-view.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md`

- [x] **Step 1: Add raw imports for snapshot leaves**

In `src/lib/analysis-source-readers.test.ts`, add these imports near the other raw component imports:

```ts
import runSnapshotMetadataViewSource from "./components/analysis/run-snapshot-metadata-view.svelte?raw";
import snapshotGroupSourcesViewSource from "./components/analysis/snapshot-group-sources-view.svelte?raw";
import snapshotItemsViewSource from "./components/analysis/snapshot-items-view.svelte?raw";
```

- [x] **Step 2: Add failing snapshot leaf contract tests**

After the `"renders universal Items as a loaded-window browser"` test, add:

```ts
  it("renders snapshot Items as a frozen SourceReaderItem browser", () => {
    expect(snapshotItemsViewSource).toContain("SourceReaderItem");
    expect(snapshotItemsViewSource).not.toContain("SourceItem");
    expect(snapshotItemsViewSource).toContain("Search snapshot items");
    expect(snapshotItemsViewSource).toContain("Snapshot items are limited to frozen rows loaded for this run");
    expect(snapshotItemsViewSource).toContain("Load older snapshot messages");
    expect(snapshotItemsViewSource).toContain("selectedTraceRef");
    expect(snapshotItemsViewSource).toContain("ariaPressed");
  });

  it("renders snapshot group Sources with global snapshot paging only", () => {
    expect(snapshotGroupSourcesViewSource).toContain("groupReaderItemsBySource");
    expect(snapshotGroupSourcesViewSource).toContain("Load older snapshot messages");
    expect(snapshotGroupSourcesViewSource).toContain("showSyncActions={false}");
    expect(snapshotGroupSourcesViewSource).toContain("otherItems");
    expect(snapshotGroupSourcesViewSource).toContain("other-item-list");
    expect(snapshotGroupSourcesViewSource).not.toContain("hasMoreBySource");
    expect(snapshotGroupSourcesViewSource).not.toContain("onLoadMoreSource");
  });

  it("renders run snapshot metadata from route-owned fields", () => {
    expect(runSnapshotMetadataViewSource).toContain("AnalysisRunDetail");
    expect(runSnapshotMetadataViewSource).toContain("Run snapshot");
    expect(runSnapshotMetadataViewSource).toContain("snapshot.readerKind");
    expect(runSnapshotMetadataViewSource).toContain("sourceOptions");
    expect(runSnapshotMetadataViewSource).not.toContain("RawJsonPanel");
  });
```

- [x] **Step 3: Run reader tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because the snapshot leaf files do not exist yet.

- [x] **Step 4: Create `SnapshotItemsView`**

Create `src/lib/components/analysis/snapshot-items-view.svelte` with:

```svelte
<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { SourceReaderItem } from "$lib/source-reader-model";

  const ALL_KINDS = "__all_snapshot_item_kinds__";

  let {
    items,
    loading,
    hasMore,
    selectedTraceRef = null,
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceReaderItem[];
    loading: boolean;
    hasMore: boolean;
    selectedTraceRef?: string | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();

  let search = $state("");
  let selectedKind = $state(ALL_KINDS);
  let sortMode = $state<"newest" | "oldest">("newest");

  const kindChips = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const item of items) counts.set(item.kind, (counts.get(item.kind) ?? 0) + 1);
    return Array.from(counts, ([kind, count]) => ({ kind, label: itemKindLabel(kind), count }));
  });
  const visibleItems = $derived.by(() => {
    const query = search.trim().toLowerCase();
    const filtered = items.filter((item) => {
      if (selectedKind !== ALL_KINDS && item.kind !== selectedKind) return false;
      if (!query) return true;
      return [item.content, item.author, item.sourceTitle]
        .some((value) => value?.toLowerCase().includes(query));
    });
    const direction = sortMode === "newest" ? -1 : 1;
    return [...filtered].sort((left, right) => {
      const leftTime = left.publishedAt ?? 0;
      const rightTime = right.publishedAt ?? 0;
      return (leftTime - rightTime) * direction || left.id.localeCompare(right.id);
    });
  });

  function inputValue(event: Event) {
    const target = event.currentTarget;
    return target instanceof HTMLInputElement ? target.value : "";
  }

  function changeSort(event: Event) {
    sortMode = (event.currentTarget as HTMLSelectElement).value as "newest" | "oldest";
  }

  function itemKindLabel(kind: string) {
    const [first = "", ...rest] = kind.split("_");
    return [first === "youtube" ? "YouTube" : capitalize(first), ...rest].join(" ");
  }

  function capitalize(value: string) {
    if (!value) return value;
    return value.charAt(0).toUpperCase() + value.slice(1);
  }

  function itemSelected(item: SourceReaderItem) {
    return item.selected || (selectedTraceRef !== null && item.ref === selectedTraceRef);
  }
</script>

<section class="snapshot-items-view" aria-label="Run snapshot items">
  <div class="items-toolbar">
    <label class="search-field">
      <span>Search snapshot items</span>
      <Input
        type="search"
        value={search}
        placeholder="Search snapshot items"
        ariaLabel="Search snapshot items"
        oninput={(event) => (search = inputValue(event))}
      />
    </label>

    <label class="sort-field">
      <span>Sort snapshot items</span>
      <Select value={sortMode} onchange={changeSort}>
        <option value="newest">Newest first</option>
        <option value="oldest">Oldest first</option>
      </Select>
    </label>
  </div>

  <div class="kind-chips" aria-label="Snapshot item kinds">
    <Button
      type="button"
      size="sm"
      variant={selectedKind === ALL_KINDS ? "secondary" : "ghost"}
      selected={selectedKind === ALL_KINDS}
      ariaPressed={selectedKind === ALL_KINDS}
      onclick={() => (selectedKind = ALL_KINDS)}
    >
      All
    </Button>
    {#each kindChips as chip (chip.kind)}
      <Button
        type="button"
        size="sm"
        variant={selectedKind === chip.kind ? "secondary" : "ghost"}
        selected={selectedKind === chip.kind}
        ariaPressed={selectedKind === chip.kind}
        onclick={() => (selectedKind = chip.kind)}
      >
        {chip.label} ({chip.count})
      </Button>
    {/each}
  </div>

  <p class="items-help">
    Snapshot items are limited to frozen rows loaded for this run. Load older snapshot messages to fetch more captured rows.
  </p>

  {#if !loading && items.length === 0}
    <EmptyState description="No frozen source rows are loaded for this run snapshot." />
  {:else if !loading && visibleItems.length === 0}
    <EmptyState description="No snapshot items match the current filters." />
  {:else}
    <ul class="item-list">
      {#each visibleItems as item (item.id)}
        <li>
          <article class:selected={itemSelected(item)} data-trace-ref={item.ref}>
            <div class="item-heading">
              <strong>{itemKindLabel(item.kind)}</strong>
              <span>{formatTimestamp(item.publishedAt)}</span>
            </div>
            <div class="item-meta">
              <Badge variant="neutral">{item.sourceTitle}</Badge>
              {#if item.author}<Badge variant="neutral">{item.author}</Badge>{/if}
              {#if item.ref}<Badge variant="info">{item.ref}</Badge>{/if}
              <Badge variant="neutral">{item.externalId}</Badge>
            </div>
            <p>{item.content || "No text content captured for this snapshot row."}</p>
          </article>
        </li>
      {/each}
    </ul>
  {/if}

  {#if hasMore}
    <div class="reader-footer">
      <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
        {loading ? "Loading..." : "Load older snapshot messages"}
      </Button>
    </div>
  {/if}
</section>

<style>
  .snapshot-items-view {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
  }

  .items-toolbar,
  .kind-chips,
  .item-meta,
  .item-heading {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .items-toolbar {
    align-items: flex-end;
  }

  .search-field,
  .sort-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--muted);
    font-size: 0.78rem;
  }

  .search-field {
    flex: 1 1 16rem;
  }

  .sort-field {
    flex: 0 1 12rem;
  }

  .items-help,
  .item-heading span,
  p {
    color: var(--muted);
  }

  .items-help {
    margin: 0;
    font-size: 0.82rem;
  }

  .item-list {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  article {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    padding: 0.7rem 0.8rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  article.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 24%, transparent);
  }

  .item-heading {
    justify-content: space-between;
  }

  p {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    line-height: 1.45;
  }

  .reader-footer {
    display: flex;
    justify-content: center;
  }
</style>
```

- [x] **Step 5: Create `SnapshotGroupSourcesView`**

Create `src/lib/components/analysis/snapshot-group-sources-view.svelte` with:

```svelte
<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import { groupReaderItemsBySource, type SourceReaderItem } from "$lib/source-reader-model";

  let {
    items,
    selectedGroupSourceId,
    loading,
    hasMore,
    selectedTraceRef = null,
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceReaderItem[];
    selectedGroupSourceId: number | null;
    loading: boolean;
    hasMore: boolean;
    selectedTraceRef?: string | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();

  const sourceGroups = $derived(
    groupReaderItemsBySource(
      selectedGroupSourceId === null
        ? items
        : items.filter((item) => item.sourceId === selectedGroupSourceId),
    ),
  );
</script>

<section class="snapshot-group-sources-view" aria-label="Run snapshot group sources">
  {#if !loading && sourceGroups.length === 0}
    <EmptyState description="No frozen source rows are loaded for this group snapshot." />
  {:else}
    {#each sourceGroups as group (group.sourceId)}
      {@const youtubeTranscriptItems = group.items.filter((item) => item.kind === "youtube_transcript")}
      {@const telegramItems = group.items.filter((item) => item.kind === "telegram_message")}
      {@const otherItems = group.items.filter((item) => item.kind !== "youtube_transcript" && item.kind !== "telegram_message")}
      <section class="source-bucket" aria-label={group.sourceTitle}>
        <div class="source-heading">
          <h3>{group.sourceTitle}</h3>
          <span>{group.items.length} frozen rows</span>
        </div>

        {#if youtubeTranscriptItems.length > 0}
          <YoutubeTranscriptReader
            detail={null}
            segments={[]}
            snapshotItems={youtubeTranscriptItems}
            {loading}
            hasMore={false}
            transcriptSearch=""
            showSyncActions={false}
            sourceTitle={group.sourceTitle}
            {selectedTraceRef}
            {formatTimestamp}
            onChangeTranscriptSearch={() => {}}
            onLoadMore={() => {}}
            onSyncTranscript={() => {}}
            onSyncMetadata={() => {}}
          />
        {/if}

        {#if telegramItems.length > 0}
          <TelegramTimelineReader
            items={telegramItems}
            {loading}
            hasMore={false}
            ariaLabel="Run snapshot source material timeline"
            {formatTimestamp}
            onLoadMore={() => {}}
          />
        {/if}

        {#if otherItems.length > 0}
          <ul class="other-item-list" aria-label={group.sourceTitle + " other snapshot rows"}>
            {#each otherItems as item (item.id)}
              <li class:selected={item.selected} data-trace-ref={item.ref}>
                <div>
                  <strong>{item.kind.replaceAll("_", " ")}</strong>
                  <span>{formatTimestamp(item.publishedAt)}</span>
                </div>
                <p>{item.content || "No text content captured for this snapshot row."}</p>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {/each}

    {#if hasMore}
      <div class="source-group-footer">
        <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
          {loading ? "Loading..." : "Load older snapshot messages"}
        </Button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .snapshot-group-sources-view,
  .source-bucket {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .source-bucket {
    padding-top: 0.8rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
  }

  .source-heading {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .source-heading h3,
  .source-heading span {
    margin: 0;
  }

  .source-heading span {
    color: var(--muted);
    font-size: 0.82rem;
  }

  .other-item-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .other-item-list li {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.65rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  .other-item-list li.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 24%, transparent);
  }

  .other-item-list div {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .other-item-list span,
  .other-item-list p {
    color: var(--muted);
  }

  .other-item-list p {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .source-group-footer {
    display: flex;
    justify-content: center;
  }

  @media (max-width: 760px) {
    .source-heading {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
```

- [x] **Step 6: Create `RunSnapshotMetadataView`**

Create `src/lib/components/analysis/run-snapshot-metadata-view.svelte` with:

```svelte
<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { RunSnapshotBrowserSubject } from "$lib/source-browser-model";
  import type { SourceFilterOption, SourceReaderItem } from "$lib/source-reader-model";
  import type { AnalysisRunDetail } from "$lib/types/analysis";

  let {
    run,
    snapshot,
    readerItems,
    sourceOptions,
    snapshotAvailability,
    snapshotError = "",
    formatTimestamp,
  }: {
    run: AnalysisRunDetail;
    snapshot: RunSnapshotBrowserSubject;
    readerItems: SourceReaderItem[];
    sourceOptions: SourceFilterOption[];
    snapshotAvailability: RunSnapshotAvailability;
    snapshotError?: string;
    formatTimestamp: (value: number | null) => string;
  } = $props();
</script>

<section class="run-snapshot-metadata-view" aria-label="Run snapshot metadata">
  <div class="metadata-heading">
    <div>
      <span>Run snapshot</span>
      <h3>{run.scope_label}</h3>
    </div>
    <Badge variant="success">{snapshotAvailability}</Badge>
  </div>

  <section class="metadata-section" aria-label="Summary">
    <h4>Summary</h4>
    <dl>
      <dt>Run id</dt>
      <dd>{run.id}</dd>
      <dt>Run title</dt>
      <dd>{run.scope_label}</dd>
      <dt>Scope type</dt>
      <dd>{snapshot.scopeType}</dd>
      <dt>Reader kind</dt>
      <dd>{snapshot.readerKind}</dd>
      <dt>Loaded rows</dt>
      <dd>{readerItems.length}</dd>
      <dt>Source type</dt>
      <dd>{snapshot.sourceType ?? "n/a"}</dd>
      <dt>Source subtype</dt>
      <dd>{snapshot.sourceSubtype ?? "n/a"}</dd>
      <dt>Created</dt>
      <dd>{formatTimestamp(run.created_at)}</dd>
      <dt>Completed</dt>
      <dd>{formatTimestamp(run.completed_at)}</dd>
    </dl>
  </section>

  {#if sourceOptions.length > 0}
    <section class="metadata-section" aria-label="Snapshot sources">
      <h4>Snapshot sources</h4>
      <ul>
        {#each sourceOptions as option (option.id)}
          <li>
            <span>{option.label}</span>
            <Badge variant="neutral">{option.count} rows</Badge>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if snapshotError}
    <section class="metadata-section" aria-label="Snapshot error">
      <h4>Snapshot error</h4>
      <p>{snapshotError}</p>
    </section>
  {/if}
</section>

<style>
  .run-snapshot-metadata-view,
  .metadata-section,
  .metadata-heading {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-width: 0;
  }

  .metadata-heading {
    flex-direction: row;
    justify-content: space-between;
    align-items: flex-start;
  }

  .metadata-heading span {
    color: var(--muted);
    font-size: 0.75rem;
    text-transform: uppercase;
  }

  h3,
  h4,
  p {
    margin: 0;
  }

  dl {
    display: grid;
    grid-template-columns: minmax(8rem, 0.35fr) 1fr;
    gap: 0.5rem 0.8rem;
    margin: 0;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  p {
    color: var(--muted);
    overflow-wrap: anywhere;
  }

  @media (max-width: 760px) {
    .metadata-heading,
    li {
      flex-direction: column;
      align-items: flex-start;
    }

    dl {
      grid-template-columns: 1fr;
    }
  }
</style>
```

- [x] **Step 7: Run reader tests and Svelte check**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
npm.cmd run check
```

Expected: both PASS.

- [x] **Step 8: Commit snapshot leaves**

Run:

```bash
git add src/lib/components/analysis/snapshot-items-view.svelte src/lib/components/analysis/snapshot-group-sources-view.svelte src/lib/components/analysis/run-snapshot-metadata-view.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/analysis-source-readers.test.ts docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md
git commit -m "feat: add run snapshot browser leaves"
```

## Task 3: Source Browser Shell Snapshot Branches

**Files:**
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.test.ts`
- Modify: `docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md`

- [x] **Step 1: Add shell raw-contract tests**

In `src/lib/components/analysis/source-browser-shell.test.ts`, add this test:

```ts
  it("renders run snapshot tabs through grouped snapshot data without live activity props", () => {
    expect(shellSource).toContain("<SnapshotGroupSourcesView");
    expect(shellSource).toContain("<SnapshotItemsView");
    expect(shellSource).toContain("<RunSnapshotMetadataView");
    expect(shellSource).toContain("snapshotBrowserData");
    expect(shellSource).toContain('subject.kind === "run_snapshot"');
    expect(shellSource).toContain('activeTab === "transcript"');
    expect(shellSource).toContain("showSyncActions={false}");
    expect(shellSource).not.toContain("SourceReaderHeader");
  });
```

- [x] **Step 2: Run shell tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts
```

Expected: FAIL because shell has no snapshot branches or snapshot data prop.

- [x] **Step 3: Add snapshot imports and grouped data type**

In `src/lib/components/analysis/source-browser-shell.svelte`, add imports:

```ts
  import RunSnapshotMetadataView from "$lib/components/analysis/run-snapshot-metadata-view.svelte";
  import SnapshotGroupSourcesView from "$lib/components/analysis/snapshot-group-sources-view.svelte";
  import SnapshotItemsView from "$lib/components/analysis/snapshot-items-view.svelte";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { SourceFilterOption } from "$lib/source-reader-model";
  import type { AnalysisRunDetail } from "$lib/types/analysis";
```

Below `type SourceGroupBrowserData = { ... };`, add:

```ts
  type SnapshotBrowserData = {
    run: AnalysisRunDetail;
    readerItems: SourceReaderItem[];
    selectedSourceId: number | null;
    sourceOptions: SourceFilterOption[];
    loading: boolean;
    hasMore: boolean;
    availability: RunSnapshotAvailability;
    error: string;
    selectedTraceRef: string | null;
    onLoadMore: () => void | Promise<void>;
  };
```

In `type Props`, add:

```ts
    snapshotBrowserData?: SnapshotBrowserData | null;
```

In the `$props()` destructuring, add:

```ts
    snapshotBrowserData = null,
```

- [x] **Step 4: Derive snapshot subject state**

Below `groupSubject`, add:

```ts
  const snapshotSubject = $derived(subject && subject.kind === "run_snapshot" ? subject.snapshot : null);
  const snapshotData = $derived(subject && subject.kind === "run_snapshot" ? snapshotBrowserData : null);
```

Replace `subjectKey` with:

```ts
  const subjectKey = $derived(
    subject
      ? subject.kind === "source"
        ? `source:${subject.source.id}`
        : subject.kind === "source_group"
          ? `source_group:${subject.group.id}`
          : `run_snapshot:${subject.snapshot.runId}:${subject.snapshot.readerKind}`
      : null,
  );
```

Replace the disabled fallback loaded row expression:

```svelte
Loaded rows: {itemsForActiveSubject.length}.
```

with:

```svelte
Loaded rows: {snapshotData?.readerItems.length ?? itemsForActiveSubject.length}.
```

- [x] **Step 5: Add snapshot tab bodies before live branches**

Inside the shell markup, place these branches before the existing live `sources` branch:

```svelte
  {#if activeTab === "sources" && snapshotSubject?.readerKind === "source_group"}
    <SnapshotGroupSourcesView
      items={snapshotData?.readerItems ?? []}
      selectedGroupSourceId={snapshotData?.selectedSourceId ?? null}
      loading={snapshotData?.loading ?? false}
      hasMore={snapshotData?.hasMore ?? false}
      selectedTraceRef={snapshotData?.selectedTraceRef ?? selectedTraceRef}
      {formatTimestamp}
      onLoadMore={snapshotData?.onLoadMore ?? (() => {})}
    />
  {:else if activeTab === "items" && subject?.kind === "run_snapshot"}
    <SnapshotItemsView
      items={snapshotData?.readerItems ?? []}
      loading={snapshotData?.loading ?? false}
      hasMore={snapshotData?.hasMore ?? false}
      selectedTraceRef={snapshotData?.selectedTraceRef ?? selectedTraceRef}
      {formatTimestamp}
      onLoadMore={snapshotData?.onLoadMore ?? (() => {})}
    />
  {:else if activeTab === "metadata" && snapshotSubject && snapshotData}
    <RunSnapshotMetadataView
      run={snapshotData.run}
      snapshot={snapshotSubject}
      readerItems={snapshotData.readerItems}
      sourceOptions={snapshotData.sourceOptions}
      snapshotAvailability={snapshotData.availability}
      snapshotError={snapshotData.error}
      {formatTimestamp}
    />
  {:else if activeTab === "timeline" && snapshotSubject?.readerKind === "telegram_timeline"}
    <TelegramTimelineReader
      items={snapshotData?.readerItems ?? []}
      loading={snapshotData?.loading ?? false}
      hasMore={snapshotData?.hasMore ?? false}
      ariaLabel="Run snapshot source material timeline"
      {formatTimestamp}
      onLoadMore={snapshotData?.onLoadMore ?? (() => {})}
    />
  {:else if activeTab === "transcript" && snapshotSubject?.readerKind === "youtube_transcript"}
    <YoutubeTranscriptReader
      detail={null}
      segments={[]}
      snapshotItems={snapshotData?.readerItems ?? []}
      loading={snapshotData?.loading ?? false}
      hasMore={snapshotData?.hasMore ?? false}
      transcriptSearch=""
      showSyncActions={false}
      sourceTitle={snapshotSubject.scopeLabel}
      selectedTraceRef={snapshotData?.selectedTraceRef ?? selectedTraceRef}
      {formatTimestamp}
      onChangeTranscriptSearch={() => {}}
      onLoadMore={snapshotData?.onLoadMore ?? (() => {})}
      onSyncTranscript={() => {}}
      onSyncMetadata={() => {}}
    />
```

Change the old first live branch from:

```svelte
  {#if activeTab === "sources" && groupSubject}
```

to:

```svelte
  {:else if activeTab === "sources" && groupSubject}
```

- [x] **Step 6: Ensure live-only activity remains live-only**

Confirm the only `SourceActivityView` branch still begins with:

```svelte
  {:else if activeTab === "activity" && sourceSubject}
```

Do not add an Activity branch for `snapshotSubject`.

- [x] **Step 7: Run shell tests and Svelte check**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts
npm.cmd run check
```

Expected: both PASS.

- [x] **Step 8: Commit shell task**

Run:

```bash
git add src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/source-browser-shell.test.ts docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md
git commit -m "feat: add run snapshot branches to source browser shell"
```

## Task 4: Route Available Snapshots Through The Shell

**Files:**
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md`

- [x] **Step 1: Update route/source reader raw tests**

In `src/lib/analysis-source-readers.test.ts`, replace the `"keeps saved snapshots outside SourceBrowserShell"` test with:

```ts
  it("routes available run snapshots through SourceBrowserShell while keeping the header route-owned", () => {
    expect(reportSourceSurfaceSource).toContain('sourceViewBasis === "run_snapshot"');
    expect(reportSourceSurfaceSource).toContain("<SourceReaderHeader");
    expect(reportSourceSurfaceSource).toContain("runSnapshotSubject");
    expect(reportSourceSurfaceSource).toContain("snapshotBrowserData");
    expect(reportSourceSurfaceSource).toContain('subject={runSnapshotSubject}');
    expect(reportSourceSurfaceSource).toContain("{onViewLiveSource}");
    expect(reportSourceSurfaceSource).not.toContain("<SourceGroupReader");
  });
```

In the `"replaces transitional source panels in ReportSourceSurface"` test, replace these expectations:

```ts
    expect(reportSourceSurfaceSource).toContain("<TelegramTimelineReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubeTranscriptReader");
    expect(reportSourceSurfaceSource).toContain("<SourceGroupReader");
```

with:

```ts
    expect(reportSourceSurfaceSource).not.toContain("<TelegramTimelineReader");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubeTranscriptReader");
    expect(reportSourceSurfaceSource).not.toContain("<SourceGroupReader");
```

Add this test near the other snapshot/source browser tests:

```ts
  it("keeps snapshot shell data frozen-only and live props empty", () => {
    expect(reportSourceSurfaceSource).toContain("deriveRunSnapshotBrowserKind");
    expect(reportSourceSurfaceSource).toContain("allSnapshotReaderItems");
    expect(reportSourceSurfaceSource).toContain("sourceJobs={[]}");
    expect(reportSourceSurfaceSource).toContain("takeoutRecovery={null}");
    expect(reportSourceSurfaceSource).toContain("sourceSyncDisabledReason={() => null}");
    expect(reportSourceSurfaceSource).toContain("snapshotBrowserData={{");
  });
```

In the `"keeps YouTube live sync actions out of readonly snapshot transcript readers"` test, replace:

```ts
    expect(reportSourceSurfaceSource).toContain("showSyncActions={false}");
    expect(sourceGroupSourcesViewSource).toContain("showSyncActions={false}");
```

with:

```ts
    expect(sourceBrowserShellSource).toContain("showSyncActions={false}");
    expect(sourceGroupSourcesViewSource).toContain("showSyncActions={false}");
    expect(snapshotGroupSourcesViewSource).toContain("showSyncActions={false}");
```

In the `"passes live YouTube video comments and jobs only into live transcript readers"` test, replace:

```ts
    expect(reportSourceSurfaceSource).toContain("showSyncActions={false}");
```

with:

```ts
    expect(sourceBrowserShellSource).toContain("showSyncActions={false}");
```

- [x] **Step 2: Update report canvas snapshot contract tests**

In `src/lib/analysis-report-canvas.test.ts`, replace the `"keeps source-group run snapshots pageable through the grouped reader"` test with:

```ts
  it("keeps source-group run snapshots pageable through the snapshot browser", () => {
    expect(reportSourceSurfaceSource).toContain("snapshotBrowserData");
    expect(reportSourceSurfaceSource).toContain("hasMoreRunSnapshotMessages");
    expect(reportSourceSurfaceSource).toContain("onLoadMoreRunSnapshotMessages");
    expect(reportSourceSurfaceSource).toContain("Load older snapshot messages");
    expect(reportSourceSurfaceSource).not.toContain("hasMoreBySource={{}}");
  });
```

In `"keeps snapshot and live source basis explicit"`, replace the direct reader expectations:

```ts
    expect(reportSourceSurfaceSource).toContain("<TelegramTimelineReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubeTranscriptReader");
```

with:

```ts
    expect(reportSourceSurfaceSource).toContain("runSnapshotSubject");
    expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
```

Remove the unused raw import:

```ts
import sourceGroupReaderSource from "./components/analysis/source-group-reader.svelte?raw";
```

- [x] **Step 3: Run route contract tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts
```

Expected: FAIL because `ReportSourceSurface` still renders direct snapshot readers.

- [x] **Step 4: Update `ReportSourceSurface` imports**

In `src/lib/components/analysis/report-source-surface.svelte`, remove these imports:

```ts
  import SourceGroupReader from "$lib/components/analysis/source-group-reader.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
```

Change the source browser model import to:

```ts
  import {
    deriveRunSnapshotBrowserKind,
    sourceBrowserShellAppliesToSource,
    sourceBrowserShellAppliesToSubject,
  } from "$lib/source-browser-model";
```

- [x] **Step 5: Add snapshot source metadata derivations**

Below `snapshotSourceOptions`, add:

```ts
  const snapshotSourceType = $derived.by(() => {
    if (currentSource) return currentSource.sourceType;
    const values = new Set(runSnapshotMessages.map((message) => message.source_type).filter(Boolean));
    return values.size === 1 ? Array.from(values)[0] ?? null : null;
  });
  const snapshotSourceSubtype = $derived.by(() => {
    if (currentSource) return currentSource.sourceSubtype;
    const values = new Set(runSnapshotMessages.map((message) => message.source_subtype).filter(Boolean));
    return values.size === 1 ? Array.from(values)[0] ?? null : null;
  });
  const runSnapshotBrowserKind = $derived(
    deriveRunSnapshotBrowserKind({
      scopeType: currentRun?.scope_type ?? null,
      sourceType: snapshotSourceType,
      sourceSubtype: snapshotSourceSubtype,
      snapshotReaderItems: allSnapshotReaderItems,
    }),
  );
  const runSnapshotSubject = $derived(
    currentRun && snapshotAvailability === "available"
      ? {
          kind: "run_snapshot" as const,
          snapshot: {
            runId: currentRun.id,
            scopeType: currentRun.scope_type === "source_group" ? "source_group" as const : "source" as const,
            scopeLabel: currentRun.scope_label,
            readerKind: runSnapshotBrowserKind,
            sourceType: snapshotSourceType === "telegram" || snapshotSourceType === "youtube" || snapshotSourceType === "rss" || snapshotSourceType === "forum"
              ? snapshotSourceType
              : null,
            sourceSubtype: snapshotSourceSubtype === "channel" ||
              snapshotSourceSubtype === "supergroup" ||
              snapshotSourceSubtype === "group" ||
              snapshotSourceSubtype === "video" ||
              snapshotSourceSubtype === "playlist" ||
              snapshotSourceSubtype === "feed" ||
              snapshotSourceSubtype === "thread" ||
              snapshotSourceSubtype === "board" ||
              snapshotSourceSubtype === "site"
              ? snapshotSourceSubtype
              : null,
          },
        }
      : null,
  );
```

- [x] **Step 6: Replace available snapshot direct readers with shell**

Inside the `snapshotAvailability === "available"` branch, keep the existing `SourceReaderHeader` unchanged.

Replace the direct reader block:

```svelte
      {#if currentRun?.scope_type === "source_group"}
        ...
      {/if}
```

with:

```svelte
      {#if runSnapshotSubject && sourceBrowserShellAppliesToSubject(runSnapshotSubject)}
        <SourceBrowserShell
          subject={runSnapshotSubject}
          source={null}
          groupBrowserData={null}
          snapshotBrowserData={{
            run: currentRun,
            readerItems: snapshotReaderItems,
            selectedSourceId: selectedSnapshotSourceId,
            sourceOptions: snapshotSourceOptions,
            loading: loadingRunSnapshotMessages,
            hasMore: hasMoreRunSnapshotMessages,
            availability: snapshotAvailability,
            error: runSnapshotError,
            selectedTraceRef,
            onLoadMore: onLoadMoreRunSnapshotMessages,
          }}
          liveReaderItems={[]}
          takeoutRecovery={null}
          sourceItems={[]}
          sourceRouteError={null}
          sourceItemsHasMore={false}
          loadingItems={loadingRunSnapshotMessages}
          sourceTopics={[]}
          loadingSourceTopics={false}
          selectedTopicKey="__all_topics__"
          showTopicSelector={false}
          youtubeVideoDetail={null}
          youtubePlaylistDetail={null}
          youtubeTranscriptSegments={[]}
          youtubeTranscriptSearch=""
          youtubeTranscriptHasMore={false}
          loadingYoutubeTranscriptSegments={false}
          loadingYoutubeDetail={false}
          sourceJobs={[]}
          {selectedTraceRef}
          {telegramHistoryScope}
          currentSourceContentLabel="Run snapshot material"
          sourceSyncDisabledReason={() => null}
          {formatTimestamp}
          {onSyncSource}
          onLoadMoreSourceItems={onLoadMoreRunSnapshotMessages}
          {onChangeSelectedTopicKey}
          {onChangeTelegramHistoryScope}
          {onChangeTranscriptSearch}
          {onLoadMoreYoutubeTranscriptSegments}
          {onOpenSource}
          {onSyncYoutubeMetadata}
          {onSyncYoutubeTranscript}
          {onSyncYoutubeComments}
          {onSyncYoutubePlaylist}
          onRetryFailedYoutubePlaylistVideos={onRetryFailedYoutubePlaylistVideos}
          {onSyncYoutubePlaylistVideo}
          {onRetryYoutubePlaylistVideo}
          {onStartTakeoutImport}
          {onStartMigratedHistoryImport}
          onCancelSourceJob={onCancelSourceJob}
        />
      {:else}
        <StatusMessage tone="muted">This run snapshot is not browsable yet.</StatusMessage>
      {/if}
```

Do not change the pending/unavailable/checking snapshot status branches.

- [x] **Step 7: Run route tests and Svelte check**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/source-browser-model.test.ts
npm.cmd run check
```

Expected: both PASS.

- [x] **Step 8: Commit route task**

Run:

```bash
git add src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md
git commit -m "feat: route run snapshots through source browser"
```

## Task 5: Verification, Smoke, And Documentation Status

**Files:**
- Modify: `docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md`
- Modify: `docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md`

- [x] **Step 1: Run focused frontend tests**

Run:

```bash
npm.cmd run test -- src/lib/source-browser-model.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts
```

Expected: PASS.

- [x] **Step 2: Run full verification**

Run:

```bash
npm.cmd run verify
```

Expected: PASS, including Vitest, `svelte-check`, Rust checks/tests, and `git diff HEAD --check`.

- [x] **Step 3: Run Tauri acceptance smoke**

Start the app:

```bash
npm.cmd run tauri dev
```

Use the MCP bridge console to seed fixtures:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

In `/analysis`, verify:

- Open `__analysis_redesign_fixture__ Group Snapshot Run`.
- Switch to Source mode.
- The header still shows `Run snapshot`.
- `View live source` remains in the header area, not inside the tabs.
- Tabs are `Sources`, `Items`, `Metadata`.
- `Sources` is selected by default for the group snapshot.
- `Sources` shows member source sections and frozen rows.
- `Sources` uses `Load older snapshot messages` as the global paging action.
- `Items` shows frozen rows with source labels and the snapshot help copy.
- `Metadata` shows run snapshot fields.
- No `Activity` tab appears.
- No source sync CTA, Takeout CTA, retry CTA, cancel job action, or source job card appears.
- `View live source` transitions out of snapshot browsing.

Stop the Tauri dev process after the smoke.

- [x] **Step 4: Update spec status**

In `docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md`, change:

```markdown
> Status: approved design, pending implementation plan
```

to:

```markdown
> Status: implemented on 2026-05-30; pending merge
```

- [x] **Step 5: Run final clean checks**

Run:

```bash
git diff --check
git status --short --branch
```

Expected: no whitespace errors; only the spec and this plan file have unstaged checkbox/status changes.

- [x] **Step 6: Commit verification task**

Run:

```bash
git add docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md
git commit -m "test: verify run snapshot source browser"
```

## Acceptance Checklist

- Available saved run snapshots enter `SourceBrowserShell`.
- Pending, unavailable, and checking snapshots remain status-only outside the shell.
- `SourceReaderHeader` remains route/surface-owned and outside `SourceBrowserShell`.
- `View live source` remains route-owned and transitions out of snapshot browsing.
- Snapshot tabs exclude `Activity`, `Comments`, and `Videos`.
- `deriveRunSnapshotBrowserKind` is the only reader-kind decision helper.
- Empty available snapshots use `Items | Metadata`.
- Snapshot group `Sources` uses global run-snapshot paging copy only.
- `SnapshotItemsView` imports `SourceReaderItem` and does not import `SourceItem`.
- `SourceBrowserShell` keeps snapshot data grouped under `snapshotBrowserData`.
- Snapshot leaves do not receive `sourceJobs`, `takeoutRecovery`, or `sourceSyncDisabledReason`.
- Existing live source and live source group tests remain passing.
