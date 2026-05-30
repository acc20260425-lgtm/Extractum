# Source Browser Legacy Wrapper Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove unused source browser compatibility wrappers after proving production code no longer imports or renders them.

**Architecture:** `SourceBrowserShell` remains the canonical production render path for live source groups and available run snapshots. The cleanup deletes only wrapper components that pass a per-component production usage audit, migrates raw contract tests to canonical shell/leaf files, and updates current-state documentation without changing browser behavior.

**Tech Stack:** Svelte 5, SvelteKit 2, TypeScript, Vitest raw component contract tests, Tauri/Rust verification through `npm.cmd run verify`.

---

## Execution Protocol

- Start from `main`.
- Create a branch before Task 0.
- After each task, mark the completed checkboxes in this plan and commit the task.
- Deletion is per component. A production usage blocker for one candidate must not block cleanup of the other candidate.
- Do not repack `SourceBrowserShell` props, restructure route state, add tabs, change backend code, or redesign UI in this slice.

## Files

- Delete if preflight passes: `src/lib/components/analysis/source-group-reader.svelte`
  - Legacy wrapper around `SourceGroupSourcesView`.
- Delete if preflight passes: `src/lib/components/analysis/run-snapshot-messages-panel.svelte`
  - Legacy snapshot message panel superseded by snapshot browser leaves.
- Modify: `src/lib/analysis-source-readers.test.ts`
  - Remove raw import of `source-group-reader.svelte`.
  - Keep group browser contracts on `SourceBrowserShell` and `SourceGroupSourcesView`.
- Modify: `src/lib/analysis-report-canvas.test.ts`
  - Remove raw import of `run-snapshot-messages-panel.svelte`.
  - Keep snapshot contracts on `ReportSourceSurface`, `SnapshotItemsView`, and `SnapshotGroupSourcesView`.
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`
  - Remove raw import of `source-group-reader.svelte`.
  - Keep source-group grouping contract on `SourceGroupSourcesView` and shell routing.
- Modify: `docs/design-document.md`
  - Update the frontend architecture paragraph to reflect live groups and available run snapshots in `SourceBrowserShell`.
- Modify: `docs/project.md`
  - Update the current product slice bullet for Source Browser coverage.
- Modify: `docs/frontend-architecture-evolution-analysis.md`
  - Replace stale "legacy group readers" and "saved run snapshot readers" wording with canonical shell terminology.
- Modify: `docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md`
  - Add a post-implementation note that saved run snapshots have since moved to `SourceBrowserShell`.
- Modify: `docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md`
  - Update status after the local merge and add a post-implementation note that legacy wrappers are cleanup candidates.
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md`
  - Mark the cleanup implemented after verification.
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md`
  - Track task checkboxes during execution.

---

### Task 0: Preflight Audit

**Files:**
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md`

- [x] **Step 1: Create the cleanup branch**

Run:

```bash
git switch -c source-browser-legacy-wrapper-cleanup
```

Expected: branch changes from `main` to `source-browser-legacy-wrapper-cleanup`.

- [x] **Step 2: Verify the canonical snapshot browser leaves are already in the shell**

Run:

```bash
rg -n "SnapshotGroupSourcesView|SnapshotItemsView|RunSnapshotMetadataView" src/lib/components/analysis/source-browser-shell.svelte
```

Expected: matches for imports and render branches in `source-browser-shell.svelte`. If this command has no matches, stop before deletion because the run snapshot migration has not landed.

Actual result on 2026-05-30:

```text
src/lib/components/analysis/source-browser-shell.svelte imports and renders:
- RunSnapshotMetadataView
- SnapshotGroupSourcesView
- SnapshotItemsView
```

- [x] **Step 3: Audit both wrapper candidates**

Run:

```bash
rg -n "SourceGroupReader|source-group-reader|RunSnapshotMessagesPanel|run-snapshot-messages-panel" src docs
```

Expected current classification:

```text
SourceGroupReader / source-group-reader:
- production source usage: none
- raw-test usage:
  - src/lib/analysis-source-readers.test.ts
  - src/lib/analysis-redesign-safety-contract.test.ts
- docs usage:
  - docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md
  - docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md
  - historical implementation plans under docs/superpowers/plans

RunSnapshotMessagesPanel / run-snapshot-messages-panel:
- production source usage: none
- raw-test usage:
  - src/lib/analysis-report-canvas.test.ts
- docs usage:
  - docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md
```

Production source usage means a match in a non-test file under `src`. Matches in `*.test.ts` and `docs/*` are migration targets, not deletion blockers.

Actual audit result on 2026-05-30:

```text
SourceGroupReader / source-group-reader:
- production source usage: none
- raw-test usage:
  - src/lib/analysis-source-readers.test.ts
  - src/lib/analysis-redesign-safety-contract.test.ts
- docs usage:
  - docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md
  - docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md
  - historical implementation plans under docs/superpowers/plans

RunSnapshotMessagesPanel / run-snapshot-messages-panel:
- production source usage: none
- raw-test usage:
  - src/lib/analysis-report-canvas.test.ts
- docs usage:
  - docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md
```

- [x] **Step 4: Apply the per-component hard gate**

Use this decision table:

```text
If source-group-reader.svelte has production source usage:
  Do not delete source-group-reader.svelte in Task 1.
  Keep only test/docs cleanup for SourceGroupReader references that are stale.

If source-group-reader.svelte has no production source usage:
  Delete source-group-reader.svelte in Task 1.

If run-snapshot-messages-panel.svelte has production source usage:
  Do not delete run-snapshot-messages-panel.svelte in Task 1.
  Keep only test/docs cleanup for RunSnapshotMessagesPanel references that are stale.

If run-snapshot-messages-panel.svelte has no production source usage:
  Delete run-snapshot-messages-panel.svelte in Task 1.
```

For the current repository state, both candidates are expected to pass the deletion gate.

Actual deletion decision on 2026-05-30:

```text
SourceGroupReader: delete in Task 1.
RunSnapshotMessagesPanel: delete in Task 1.
```

- [x] **Step 5: Commit the audited plan state**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md
git commit -m "docs: record source browser cleanup preflight"
```

Expected: commit succeeds with only this plan file changed.

---

### Task 1: Delete Wrappers And Migrate Tests

**Files:**
- Delete: `src/lib/components/analysis/source-group-reader.svelte`
- Delete: `src/lib/components/analysis/run-snapshot-messages-panel.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md`

- [x] **Step 1: Migrate `analysis-source-readers.test.ts` away from `SourceGroupReader`**

Remove this import:

```ts
import sourceGroupReaderSource from "./components/analysis/source-group-reader.svelte?raw";
```

In the test named `"renders Telegram topic filtering only in live single-source mode"`, replace the wrapper assertion with the canonical leaf assertion:

```ts
expect(sourceGroupSourcesViewSource).not.toContain("topic-filter");
```

Replace the test named `"keeps SourceGroupReader as a compatibility wrapper"` with this canonical contract:

```ts
it("keeps migrated source browser contracts on canonical shell and leaves", () => {
  expect(reportSourceSurfaceSource).not.toContain("<SourceGroupReader");
  expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
  expect(sourceBrowserShellSource).toContain("<SourceGroupSourcesView");
  expect(sourceBrowserShellSource).toContain("<SnapshotGroupSourcesView");
  expect(sourceBrowserShellSource).toContain("<SnapshotItemsView");
  expect(sourceBrowserShellSource).toContain("<RunSnapshotMetadataView");
  expect(sourceGroupSourcesViewSource).not.toContain("$lib/api/");
  expect(sourceGroupSourcesViewSource).not.toContain("invoke(");
  expect(snapshotGroupSourcesViewSource).not.toContain("$lib/api/");
  expect(snapshotGroupSourcesViewSource).not.toContain("invoke(");
  expect(snapshotItemsViewSource).not.toContain("$lib/api/");
  expect(snapshotItemsViewSource).not.toContain("invoke(");
  expect(runSnapshotMetadataViewSource).not.toContain("$lib/api/");
  expect(runSnapshotMetadataViewSource).not.toContain("invoke(");
});
```

In the test named `"keeps source focus controls in one reader header location"`, replace the wrapper assertions with canonical leaf assertions:

```ts
expect(sourceReaderHeaderSource).toContain("<span>Source focus</span>");
expect(sourceGroupSourcesViewSource).not.toContain("<span>Source focus</span>");
expect(sourceGroupSourcesViewSource).not.toContain("group-filter");
```

- [x] **Step 2: Migrate `analysis-report-canvas.test.ts` away from `RunSnapshotMessagesPanel`**

Remove this import:

```ts
import runSnapshotMessagesPanelSource from "./components/analysis/run-snapshot-messages-panel.svelte?raw";
```

Add this import near the existing snapshot group import:

```ts
import snapshotItemsViewSource from "./components/analysis/snapshot-items-view.svelte?raw";
```

Replace the test named `"keeps run snapshot reading bounded and snapshot-only"` with this snapshot browser contract:

```ts
it("keeps run snapshot reading bounded and snapshot-only", () => {
  expect(reportSourceSurfaceSource).toContain("snapshotBrowserData");
  expect(reportSourceSurfaceSource).toContain("hasMoreRunSnapshotMessages");
  expect(reportSourceSurfaceSource).toContain("onLoadMoreRunSnapshotMessages");
  expect(snapshotItemsViewSource).toContain("SourceReaderItem");
  expect(snapshotItemsViewSource).toContain("Load older snapshot messages");
  expect(snapshotItemsViewSource).toContain("Snapshot items are limited to frozen rows loaded for this run");
  expect(snapshotItemsViewSource).not.toContain("SourceItem");
  expect(snapshotItemsViewSource).not.toContain("listSourceItems");
  expect(snapshotGroupSourcesViewSource).toContain("Load older snapshot messages");
  expect(snapshotGroupSourcesViewSource).not.toContain("hasMoreBySource");
});
```

- [x] **Step 3: Migrate `analysis-redesign-safety-contract.test.ts` away from `SourceGroupReader`**

Remove this import:

```ts
import sourceGroupReaderSource from "./components/analysis/source-group-reader.svelte?raw";
```

In the test named `"keeps source groups grouped by source instead of merged into one pseudo-chat"`, replace the first wrapper assertion with shell and leaf assertions:

```ts
expect(reportSourceSurfaceSource).toContain('subject={{ kind: "source_group", group: currentGroup }}');
expect(reportSourceSurfaceSource).toContain("groupBrowserData");
expect(sourceGroupSourcesViewSource).toContain('class="source-group-sources-view"');
```

Keep the existing assertions in that test that read `sourceGroupSourcesViewSource`:

```ts
expect(sourceGroupSourcesViewSource).toContain("groupReaderItemsBySource");
expect(sourceGroupSourcesViewSource).toContain("youtubeItems");
expect(sourceGroupSourcesViewSource).toContain("telegramItems");
expect(sourceGroupSourcesViewSource).toContain("source-heading");
expect(sourceGroupSourcesViewSource).toContain("selectedGroupSourceId");
expect(sourceGroupSourcesViewSource).not.toContain("mergedTimeline");
expect(sourceGroupSourcesViewSource).not.toContain("pseudoChat");
```

- [x] **Step 4: Run focused tests before deletion**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/components/analysis/source-browser-shell.test.ts
```

Expected: PASS. These tests should no longer import the wrapper raw files.

- [x] **Step 5: Delete wrapper files that passed the Task 0 gate**

If both candidates passed the Task 0 production usage gate, run:

```bash
git rm --ignore-unmatch src/lib/components/analysis/source-group-reader.svelte src/lib/components/analysis/run-snapshot-messages-panel.svelte
```

If only one candidate passed the gate, delete only that candidate with `git rm` and leave the blocked file untouched.

Expected for the current repository state: both files are removed.

- [x] **Step 6: Verify `src` has no wrapper references**

Run:

```bash
rg -n "SourceGroupReader|source-group-reader|RunSnapshotMessagesPanel|run-snapshot-messages-panel" src
```

Expected: no output. `rg` exits with code `1` when no matches are found.

- [x] **Step 7: Run focused tests after deletion**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/components/analysis/source-browser-shell.test.ts
```

Expected: PASS.

- [x] **Step 8: Run Svelte/type check after deleting components**

Run:

```bash
npm.cmd run check
```

Expected: PASS with 0 errors.

- [x] **Step 9: Commit wrapper deletion and test migration**

Run:

```bash
git add -A src/lib docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md
git commit -m "test: remove legacy source browser wrappers"
```

Expected: commit includes deleted wrapper files and migrated tests.

---

### Task 2: Cleanup Current Documentation

**Files:**
- Modify: `docs/design-document.md`
- Modify: `docs/project.md`
- Modify: `docs/frontend-architecture-evolution-analysis.md`
- Modify: `docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md`
- Modify: `docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md`

- [ ] **Step 1: Update `docs/design-document.md`**

Replace the paragraph that starts with `Live single-source browsing now uses` with:

```md
Live source browsing now uses `SourceBrowserShell` for Telegram sources,
YouTube videos, YouTube playlists, live source groups, and available run
snapshots. The shell owns only local tab state and receives route-owned
data/callbacks through props. Telegram defaults to `Timeline`; YouTube videos
default to `Transcript`; YouTube playlists default to `Videos`; live source
groups default to `Sources`; available run snapshots default to their
provider-aware snapshot tab. All live sources expose loaded-window `Items`,
structured `Metadata`, and consolidated `Activity`; available run snapshots
preserve frozen snapshot semantics and do not expose live source actions.
```

- [ ] **Step 2: Update `docs/project.md`**

Replace the implemented bullet that starts with `live single-source Source Browser` with:

```md
- Source Browser for live Telegram sources, YouTube videos, YouTube playlists,
  live source groups, and available saved run snapshots, with provider-aware
  default tabs, playlist `Videos`, group `Sources`, frozen snapshot browsing,
  universal loaded item browsing, YouTube comments, structured metadata, and
  consolidated live source Activity
```

- [ ] **Step 3: Update `docs/frontend-architecture-evolution-analysis.md`**

Replace this paragraph:

```md
The live single-source Source Browser slice has shipped for Telegram sources,
YouTube videos, and YouTube playlists. It confirms the preferred frontend
direction: keep route data ownership in `/analysis`, add small focused
components for provider-aware surfaces, and keep browser tab state local to the
shell.
```

with:

```md
The Source Browser slices have shipped for live Telegram sources, YouTube
videos, YouTube playlists, live source groups, and available run snapshots.
They confirm the preferred frontend direction: keep route data ownership in
`/analysis`, add small focused components for provider-aware surfaces, and keep
browser tab state local to the shell.
```

Replace the current-shape bullet:

```md
- live source browser tabs, legacy group readers, and saved run snapshot
  readers;
```

with:

```md
- live and snapshot Source Browser tabs for single sources, source groups, and
  available run snapshots;
```

- [ ] **Step 4: Add post-implementation notes to active browser specs**

In `docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md`, add this note after the header block:

```md
> Post-implementation note: live source groups now route through
> `SourceBrowserShell`; available run snapshots were migrated into the same
> shell in a later slice. Historical context below describes the pre-slice
> state.
```

In `docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md`, replace the status line with:

```md
> Status: merged into main on 2026-05-30
```

Then add this note after the header block:

```md
> Post-implementation note: available run snapshots now route through
> `SourceBrowserShell`; legacy wrapper components are cleanup candidates in
> `2026-05-30-source-browser-legacy-wrapper-cleanup-design.md`. Historical
> context below describes the pre-slice state.
```

- [ ] **Step 5: Verify current docs no longer describe wrappers as active paths**

Run:

```bash
rg -n 'specialized readers|legacy group readers|saved run snapshot readers|keep their specialized readers|still render specialized readers|render .*SourceGroupReader' docs/design-document.md docs/project.md docs/frontend-architecture-evolution-analysis.md docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md
```

Expected: no matches in `docs/design-document.md`, `docs/project.md`, or `docs/frontend-architecture-evolution-analysis.md`. Matches inside the two active specs are acceptable only if the post-implementation note clearly marks the following section as historical pre-slice context. If `rg` exits `0` only because of those acceptable historical spec matches, continue and do not treat that command as a failure.

- [ ] **Step 6: Commit documentation cleanup**

Run:

```bash
git add docs/design-document.md docs/project.md docs/frontend-architecture-evolution-analysis.md docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md
git commit -m "docs: clarify source browser canonical paths"
```

Expected: commit includes only documentation and plan checkbox updates.

---

### Task 3: Final Verification And Status Update

**Files:**
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md`

- [ ] **Step 1: Verify no wrapper references remain in `src`**

Run:

```bash
rg -n "SourceGroupReader|source-group-reader|RunSnapshotMessagesPanel|run-snapshot-messages-panel" src
```

Expected: no output. `rg` exits with code `1` when no matches are found.

- [ ] **Step 2: Run focused frontend tests**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/source-browser-model.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run Svelte/type checks**

Run:

```bash
npm.cmd run check
```

Expected: PASS with 0 errors.

- [ ] **Step 4: Run full project verification**

Run:

```bash
npm.cmd run verify
```

Expected: PASS, including frontend tests, Svelte checks, Rust checks/tests, and `git diff HEAD --check`.

- [ ] **Step 5: Mark cleanup spec implemented**

In `docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md`, replace:

```md
> Status: approved design, pending implementation plan
```

with:

```md
> Status: implemented on 2026-05-30; pending merge
```

- [ ] **Step 6: Check whitespace**

Run:

```bash
git diff --check
```

Expected: no output.

- [ ] **Step 7: Commit final verification status**

Run:

```bash
git add docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md
git commit -m "docs: mark source browser cleanup verified"
```

Expected: commit includes the cleanup spec status and final plan checkbox updates.

---

## Acceptance Checklist

- [ ] `src/lib/components/analysis/source-group-reader.svelte` is deleted if and only if it had no production source usage.
- [ ] `src/lib/components/analysis/run-snapshot-messages-panel.svelte` is deleted if and only if it had no production source usage.
- [ ] `rg -n "SourceGroupReader|source-group-reader|RunSnapshotMessagesPanel|run-snapshot-messages-panel" src` returns no matches after test migration.
- [ ] `ReportSourceSurface` still routes live source groups and available run snapshots through `SourceBrowserShell`.
- [ ] `SourceBrowserShell` still renders canonical group and snapshot leaves.
- [ ] Snapshot leaves remain frozen-only and do not import live APIs.
- [ ] Live source groups still expose `Sources | Items | Metadata | Activity`.
- [ ] Available run snapshots still exclude `Activity`.
- [ ] Current-state docs describe `SourceBrowserShell` as the canonical production render path.
- [ ] `npm.cmd run verify` passes.
