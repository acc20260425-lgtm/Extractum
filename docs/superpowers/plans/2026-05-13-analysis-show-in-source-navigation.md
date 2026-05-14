# Analysis Show In Source Navigation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Evidence `Show in source` open a source page that contains the selected trace ref and scroll the highlighted source row into view.

**Architecture:** Add focused "around selected trace" paging entry points to the existing snapshot, Telegram, and YouTube source loaders. Keep the source-basis decision in `analysis-run-companion-state.ts`; the route owns loading the right page before the readers scroll to the already-highlighted item. Readers remain simple and only scroll selected rendered rows/groups into view.

**Tech Stack:** Svelte 5 runes, TypeScript, Vitest raw-source and route contract tests, Rust/Tauri commands backed by SQLite.

---

## Execution Rules

- Work from clean `main`.
- Use TDD for each task: add the failing test, run it and confirm the expected failure, then implement.
- Commit after each implementation task.
- Do not weaken the completed-run snapshot contract: completed runs with missing snapshots remain unavailable instead of falling back to live source.
- Run `mcp__svelte_server__.list_sections` before editing Svelte components, then use `mcp__svelte_server__.svelte_autofixer` on every changed Svelte component before committing.
- Use `apply_patch` for manual edits.

## File Structure

- Modify `src/lib/types/analysis.ts`: add optional `aroundRef` to `ListAnalysisRunMessagesInput`.
- Modify `src/lib/api/analysis-runs.ts` and `src/lib/api/analysis-runs.test.ts`: pass `aroundRef` through to Tauri.
- Modify `src-tauri/src/analysis/mod.rs` and `src-tauri/src/analysis/corpus.rs`: load snapshot pages from a selected ref without live fallback.
- Modify `src/lib/types/sources.ts`: add optional `aroundItemId` and `aroundStartMs` to source reader inputs.
- Modify `src/lib/api/sources.ts` and `src/lib/api/sources.test.ts`: pass focus inputs through to Tauri.
- Modify `src-tauri/src/sources/items.rs`, `src-tauri/src/sources/items/query.rs`, and `src-tauri/src/youtube/transcript_reader.rs`: load Telegram/source-item and YouTube transcript pages around selected evidence.
- Modify `src/routes/analysis/+page.svelte`: make `showSelectedTraceInSource()` call a focused loader before relying on the readers.
- Modify `src/lib/components/analysis/telegram-timeline-reader.svelte` and `src/lib/components/analysis/youtube-transcript-reader.svelte`: scroll selected rendered rows into view.
- Modify route/source reader contract tests:
  - `src/lib/analysis-run-companion-route.test.ts`
  - `src/lib/analysis-source-readers-route.test.ts`
  - `src/lib/analysis-source-readers.test.ts`

---

### Task 1: Add Snapshot Around-Ref Paging

**Files:**
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-runs.ts`
- Modify: `src/lib/api/analysis-runs.test.ts`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`

- [ ] **Step 1: Add failing API and Rust tests**

In `src/lib/api/analysis-runs.test.ts`, add a test that calls:

```ts
await listAnalysisRunMessages({
  runId: 7,
  after: null,
  limit: 50,
  sourceId: 12,
  aroundRef: "s12-i99",
});
expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_run_messages", {
  runId: 7,
  after: null,
  limit: 50,
  sourceId: 12,
  aroundRef: "s12-i99",
});
```

In `src-tauri/src/analysis/corpus.rs`, add a unit test near `list_run_snapshot_messages_page_reads_saved_snapshot_only` that inserts three snapshot rows and requests:

```rust
let page = list_run_snapshot_messages_page(
    &pool,
    ListRunSnapshotMessagesRequest {
        run_id: 1,
        after: None,
        limit: 2,
        source_id: Some(2),
        around_ref: Some("s2-i200".to_string()),
    },
)
.await
.expect("load around ref");

assert_eq!(
    page.messages.iter().map(|message| message.r#ref.as_str()).collect::<Vec<_>>(),
    vec!["s2-i200", "s2-i300"]
);
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```powershell
npm.cmd test -- --run src/lib/api/analysis-runs.test.ts
cargo test analysis::corpus::tests::list_run_snapshot_messages_page_starts_at_around_ref --manifest-path src-tauri/Cargo.toml
```

Expected: TypeScript test fails because `aroundRef` is not part of the input/pass-through; Rust test fails because `around_ref` and around-ref query support do not exist.

- [ ] **Step 3: Implement snapshot around-ref pass-through**

In `src/lib/types/analysis.ts`, add:

```ts
aroundRef?: string | null;
```

to `ListAnalysisRunMessagesInput`.

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
pub(crate) around_ref: Option<String>,
```

to `ListRunSnapshotMessagesRequest`.

In `src-tauri/src/analysis/mod.rs`, pass `around_ref` from the command into `ListRunSnapshotMessagesRequest`.

In `list_run_snapshot_messages_page`, when `request.around_ref` is set and `request.after` is `None`, first load the target row for the same run/source filter:

```sql
SELECT published_at, ref
FROM analysis_run_messages
WHERE run_id = ?
  AND (? IS NULL OR source_id = ?)
  AND ref = ?
LIMIT 1
```

Then page with:

```sql
AND (
  published_at > ?
  OR (published_at = ? AND ref >= ?)
)
ORDER BY published_at ASC, ref ASC
LIMIT ?
```

If the ref is not found, fall back to the normal first page without using live source.

- [ ] **Step 4: Verify and commit**

Run:

```powershell
npm.cmd test -- --run src/lib/api/analysis-runs.test.ts
cargo test analysis::corpus::tests::list_run_snapshot_messages_page_starts_at_around_ref --manifest-path src-tauri/Cargo.toml
npm.cmd run check
git diff --check
```

Commit:

```powershell
git add src/lib/types/analysis.ts src/lib/api/analysis-runs.ts src/lib/api/analysis-runs.test.ts src-tauri/src/analysis/mod.rs src-tauri/src/analysis/corpus.rs
git commit -m "feat: page run snapshots around evidence refs"
```

---

### Task 2: Add Live Source Around-Trace Paging

**Files:**
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`
- Modify: `src/lib/api/sources.test.ts`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/youtube/transcript_reader.rs`

- [ ] **Step 1: Add failing API and Rust tests**

In `src/lib/api/sources.test.ts`, add tests showing:

```ts
await listSourceItems({
  sourceId: 7,
  limit: 50,
  beforePublishedAt: null,
  topicFilter: null,
  aroundItemId: 99,
});
expect(invokeMock).toHaveBeenLastCalledWith("list_source_items", {
  request: {
    sourceId: 7,
    limit: 50,
    beforePublishedAt: null,
    topicFilter: null,
    aroundItemId: 99,
  },
});
```

and:

```ts
await listYoutubeTranscriptSegments({
  sourceId: 7,
  after: null,
  limit: 50,
  searchQuery: null,
  aroundStartMs: 754000,
});
expect(invokeMock).toHaveBeenLastCalledWith("list_youtube_transcript_segments", {
  request: {
    sourceId: 7,
    after: null,
    limit: 50,
    searchQuery: null,
    aroundStartMs: 754000,
  },
});
```

In Rust, add:

- a `src-tauri/src/sources/items/query.rs` test that `around_item_id` loads a descending page beginning with the target item;
- a `src-tauri/src/youtube/transcript_reader.rs` test that `around_start_ms` loads the target segment and following segments.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```powershell
npm.cmd test -- --run src/lib/api/sources.test.ts
cargo test sources::items::query::tests::load_item_rows_can_start_at_selected_item --manifest-path src-tauri/Cargo.toml
cargo test youtube::transcript_reader::tests::list_youtube_transcript_segments_can_start_at_selected_time --manifest-path src-tauri/Cargo.toml
```

Expected: tests fail because focus request fields do not exist.

- [ ] **Step 3: Implement focused live paging**

In `ListSourceItemsInput`, add:

```ts
aroundItemId?: number | null;
```

In `ListYoutubeTranscriptSegmentsInput`, add:

```ts
aroundStartMs?: number | null;
```

Pass both fields through `src/lib/api/sources.ts`.

In Rust `ListSourceItemsRequest`, add:

```rust
pub around_item_id: Option<i64>,
```

In `load_item_rows_from_pool`, accept `around_item_id`. If it is set, query the selected item's `published_at` for the same source and apply:

```sql
AND items.published_at <= ?
```

before the existing `ORDER BY items.published_at DESC LIMIT ?`. If the item is not found, use the normal first page.

In `ListYoutubeTranscriptSegmentsRequest`, add:

```rust
pub around_start_ms: Option<i64>,
```

When `after` is `None` and `around_start_ms` is set, add:

```sql
AND start_ms >= ?
```

before `ORDER BY start_ms ASC, id ASC LIMIT ?`.

- [ ] **Step 4: Verify and commit**

Run:

```powershell
npm.cmd test -- --run src/lib/api/sources.test.ts
cargo test sources::items::query::tests::load_item_rows_can_start_at_selected_item --manifest-path src-tauri/Cargo.toml
cargo test youtube::transcript_reader::tests::list_youtube_transcript_segments_can_start_at_selected_time --manifest-path src-tauri/Cargo.toml
npm.cmd run check
git diff --check
```

Commit:

```powershell
git add src/lib/types/sources.ts src/lib/api/sources.ts src/lib/api/sources.test.ts src-tauri/src/sources/items.rs src-tauri/src/sources/items/query.rs src-tauri/src/youtube/transcript_reader.rs
git commit -m "feat: page live sources around evidence refs"
```

---

### Task 3: Route Show In Source Through Focused Loaders

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/analysis-run-companion-route.test.ts`
- Modify: `src/lib/analysis-source-readers-route.test.ts`

- [ ] **Step 1: Add failing route contract tests**

In `src/lib/analysis-run-companion-route.test.ts`, extend the `activates Evidence for trace clicks and Show in source prefers snapshot` test with:

```ts
expect(analysisPageSource).toContain("async function showSelectedTraceInSource");
expect(analysisPageSource).toContain("await loadSourcePageAroundTrace(decision, selectedTrace)");
expect(analysisPageSource).toContain("aroundRef: trace.ref");
```

In `src/lib/analysis-source-readers-route.test.ts`, add:

```ts
it("loads live source pages around the selected trace before source readers scroll", () => {
  expect(analysisPageSource).toContain("function sourceReaderFocusInput");
  expect(analysisPageSource).toContain("aroundItemId: trace.item_id");
  expect(analysisPageSource).toContain("aroundStartMs: trace.youtube_timestamp_seconds * 1000");
  expect(analysisPageSource).toContain("loadSourcePageAroundTrace");
});
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-run-companion-route.test.ts src/lib/analysis-source-readers-route.test.ts
```

Expected: tests fail because `showSelectedTraceInSource` is synchronous and focused loader helpers do not exist.

- [ ] **Step 3: Implement route focused loading**

Change `showSelectedTraceInSource` to `async`.

After setting `selectedTraceRef` and `workspaceUiState`, call:

```ts
await loadSourcePageAroundTrace(decision, selectedTrace);
```

Update the `RunCompanionTabs` prop to:

```svelte
onShowSelectedTraceInSource={() => void showSelectedTraceInSource()}
```

Add:

```ts
function sourceReaderFocusInput(trace: AnalysisTraceRef) {
  return {
    aroundItemId: trace.item_id > 0 ? trace.item_id : null,
    aroundStartMs:
      trace.youtube_timestamp_seconds !== null ? trace.youtube_timestamp_seconds * 1000 : null,
  };
}
```

Add `loadSourcePageAroundTrace(decision, trace)`:

- for `run_snapshot`, call `listAnalysisRunMessages({ runId, after: null, limit: 50, sourceId: selectedSnapshotSourceId, aroundRef: trace.ref })`, then `applySnapshotPage(currentRun, page, false)`;
- for `live_source` + selected live source + Telegram, call `listSourceItems({ sourceId: trace.source_id, limit: 50, beforePublishedAt: null, topicFilter: null, aroundItemId })`, then assign `sourceItems`;
- for `live_source` + YouTube video, call `listYoutubeTranscriptSegments({ sourceId: trace.source_id, after: null, limit: 80, searchQuery: null, aroundStartMs })`, then assign `youtubeTranscriptSegments`, cursor, hasMore, and clear `youtubeTranscriptSearch`.

If focused loading fails, keep the source mode switch and set `status = formatAppError("loading selected source evidence", error)`.

- [ ] **Step 4: Verify and commit**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-run-companion-route.test.ts src/lib/analysis-source-readers-route.test.ts
npm.cmd run check
git diff --check
```

Commit:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-run-companion-route.test.ts src/lib/analysis-source-readers-route.test.ts
git commit -m "feat: load source pages around selected evidence"
```

---

### Task 4: Scroll Selected Reader Rows Into View

**Files:**
- Modify: `src/lib/components/analysis/telegram-timeline-reader.svelte`
- Modify: `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Add failing source reader contract tests**

In `src/lib/analysis-source-readers.test.ts`, add:

```ts
it("scrolls selected Telegram and YouTube source rows into view", () => {
  expect(telegramTimelineSource).toContain("scrollSelectedMessageIntoView");
  expect(telegramTimelineSource).toContain("scrollIntoView");
  expect(telegramTimelineSource).toContain("data-trace-ref={item.ref}");
  expect(youtubeTranscriptSource).toContain("scrollSelectedTranscriptGroupIntoView");
  expect(youtubeTranscriptSource).toContain("scrollIntoView");
  expect(youtubeTranscriptSource).toContain("data-trace-ref={visibleRef}");
});
```

- [ ] **Step 2: Run test and confirm failure**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts
```

Expected: test fails because readers only style selected rows and do not scroll.

- [ ] **Step 3: Implement Svelte scrolling**

Run `mcp__svelte_server__.list_sections` before this edit.

In each reader, import `tick`:

```svelte
import { tick } from "svelte";
```

In `telegram-timeline-reader.svelte`, add:

```ts
let timelineElement: HTMLElement | null = $state(null);

$effect(() => {
  const selectedRef = items.find((item) => item.selected)?.ref ?? null;
  if (selectedRef) {
    void scrollSelectedMessageIntoView(selectedRef);
  }
});

async function scrollSelectedMessageIntoView(selectedRef: string) {
  await tick();
  const selected = timelineElement?.querySelector<HTMLElement>(
    `[data-trace-ref="${CSS.escape(selectedRef)}"]`,
  );
  selected?.scrollIntoView({ block: "center", behavior: "smooth" });
}
```

Bind the reader section and annotate rows:

```svelte
<section class="telegram-timeline-reader" aria-label="Telegram source timeline" bind:this={timelineElement}>
...
<li class:selected={item.selected} data-trace-ref={item.ref}>
```

In `youtube-transcript-reader.svelte`, add the analogous `transcriptElement`, `scrollSelectedTranscriptGroupIntoView`, and annotate transcript group `<li>` rows with `data-trace-ref={visibleRef}`.

- [ ] **Step 4: Autofix changed Svelte components**

Run Svelte autofixer for:

```text
TelegramTimelineReader.svelte
YoutubeTranscriptReader.svelte
```

- [ ] **Step 5: Verify and commit**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts
npm.cmd run check
git diff --check
```

Commit:

```powershell
git add src/lib/components/analysis/telegram-timeline-reader.svelte src/lib/components/analysis/youtube-transcript-reader.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: scroll selected source evidence into view"
```

---

### Task 5: Full Verification And Runtime Smoke

**Files:**
- Modify: `docs/superpowers/plans/2026-05-13-analysis-show-in-source-navigation.md`

- [x] **Step 1: Run targeted tests**

Run:

```powershell
npm.cmd test -- --run src/lib/api/analysis-runs.test.ts src/lib/api/sources.test.ts src/lib/analysis-run-companion-route.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/analysis-source-readers.test.ts
```

- [x] **Step 2: Run backend targeted tests**

Run:

```powershell
cargo test list_run_snapshot_messages_page_starts_at_around_ref --manifest-path src-tauri/Cargo.toml
cargo test load_item_rows_can_start_at_selected_item --manifest-path src-tauri/Cargo.toml
cargo test list_youtube_transcript_segments_can_start_at_selected_time --manifest-path src-tauri/Cargo.toml
```

- [x] **Step 3: Run Svelte check and full tests**

Run:

```powershell
npm.cmd run check
npm.cmd test -- --run
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

- [x] **Step 4: Runtime smoke if Tauri bridge is available**

If a Tauri app is running:

```text
mcp__tauri__.driver_session action=start port=9223
```

Verify:

- Evidence `Show in source` for a saved run with snapshot opens Source mode, shows `Run snapshot`, and scrolls to the selected ref.
- Evidence `Show in source` for an active/non-completed run with unavailable snapshot opens live Source mode with warning and scrolls to the selected ref.
- Completed run with unavailable snapshot still refuses exact source resolution instead of loading live source.

If no fixture state is available, record the exact skipped part.

- [x] **Step 5: Record verification evidence and commit**

Append a `## Verification Evidence` section to this plan with exact commands and pass/fail counts.

Commit:

```powershell
git add docs/superpowers/plans/2026-05-13-analysis-show-in-source-navigation.md
git commit -m "docs: record show in source navigation verification"
```

## Verification Evidence

Recorded on 2026-05-14 from `G:\Develop\Extractum` on `main`.

- `npm.cmd test -- --run src/lib/api/analysis-runs.test.ts src/lib/api/sources.test.ts src/lib/analysis-run-companion-route.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/analysis-source-readers.test.ts`
  - result: 5 files passed, 54 tests passed.
- `cargo test list_run_snapshot_messages_page_starts_at_around_ref --manifest-path src-tauri/Cargo.toml`
  - result: 1 test passed, 0 failed.
- `cargo test load_item_rows_can_start_at_selected_item --manifest-path src-tauri/Cargo.toml`
  - result: 1 test passed, 0 failed.
- `cargo test list_youtube_transcript_segments_can_start_at_selected_time --manifest-path src-tauri/Cargo.toml`
  - result: 1 test passed, 0 failed.
- `npm.cmd run check`
  - result: `svelte-check found 0 errors and 0 warnings`.
- `npm.cmd test -- --run`
  - result: 50 files passed, 408 tests passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: 314 tests passed, 0 failed.
- `git diff --check`
  - result: exit 0, no whitespace errors.
- `mcp__tauri__.driver_session action=start port=9223`
  - result: skipped runtime smoke because no Tauri app was found at `localhost:9223`.
  - not runtime-verified in this pass: saved-run snapshot `Show in source`, active/live-source fallback scroll, and completed missing-snapshot refusal.
