# Analysis UX Polish Pass 2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the remaining `/analysis` UX polish without cutting scope, with one focused task per commit and an explicit user approval gate between tasks.

**Architecture:** Keep the work in the existing Svelte component boundaries: Runs companion density, source switcher behavior, source reader hierarchy, transcript search control, and mobile quick-source rail. Each task must be independently testable and committed before moving to the next task.

**Tech Stack:** Svelte 5, TypeScript, Vitest raw component contract tests, Svelte check, Tauri runtime fixture verification.

---

## Execution Rules

- Work from `main` on a new branch named `polish/analysis-ux-pass-2`.
- Do exactly one task at a time.
- After each task:
  - run the task-specific verification;
  - run `npm.cmd run check`;
  - run `git diff --check`;
  - commit the task;
  - stop and wait for the user's explicit instruction to continue.
- Do not merge the branch until the user asks.
- Do not add a remote.
- Do not revert unrelated user changes.

## Branch Setup

- [ ] **Step 1: Confirm clean state**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## main
```

- [ ] **Step 2: Create the pass 2 branch**

Run:

```powershell
git switch -c polish/analysis-ux-pass-2
```

Expected:

```text
Switched to a new branch 'polish/analysis-ux-pass-2'
```

---

### Task 1: Compress Runs Companion Filters

**Goal:** Make the Runs tab show the list sooner by keeping search, scope, status, and refresh visible while moving date/provider/model/template filters behind an `Advanced filters` disclosure.

**Files:**
- Modify: `src/lib/components/analysis/run-companion-runs-tab.svelte`
- Test: `src/lib/analysis-run-companion-tabs.test.ts`
- Optional Test: `src/lib/analysis-redesign-safety-contract.test.ts`

- [ ] **Step 1: Add a failing raw component test**

Add expectations that the Runs tab includes an `Advanced filters` disclosure and keeps the advanced fields inside it.

```ts
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";

it("keeps dense run filters behind an advanced filters disclosure", () => {
  expect(runsTabSource).toContain("<summary>Advanced filters</summary>");
  expect(runsTabSource).toContain('class="advanced-filters"');
  expect(runsTabSource).toContain('ariaLabel="Runs from date"');
  expect(runsTabSource).toContain('ariaLabel="Provider filter"');
});
```

- [ ] **Step 2: Run the targeted test and confirm it fails**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-companion-tabs.test.ts
```

Expected: FAIL because the component does not yet contain `Advanced filters`.

- [ ] **Step 3: Implement the disclosure**

In `src/lib/components/analysis/run-companion-runs-tab.svelte`, wrap `.date-row` and `.meta-row` in:

```svelte
<details class="advanced-filters">
  <summary>Advanced filters</summary>
  <div class="advanced-filter-grid">
    <div class="date-row" aria-label="Date range">
      ...
    </div>

    <div class="meta-row">
      ...
    </div>
  </div>
</details>
```

Keep search, scope, status, and refresh controls outside the disclosure.

Add CSS:

```css
.advanced-filters {
  border: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--panel-strong) 62%, transparent);
}

.advanced-filters summary {
  cursor: pointer;
  padding: 0.55rem 0.65rem;
  color: var(--muted);
  font-size: 0.82rem;
  font-weight: 600;
}

.advanced-filter-grid {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  padding: 0 0.65rem 0.65rem;
}
```

- [ ] **Step 4: Verify**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-companion-tabs.test.ts
npm.cmd run check
git diff --check
```

Expected: all pass, no diff whitespace output.

- [ ] **Step 5: Commit**

Run:

```powershell
git add src/lib/components/analysis/run-companion-runs-tab.svelte src/lib/analysis-run-companion-tabs.test.ts
git commit -m "polish: condense analysis run filters"
```

- [ ] **Step 6: Stop**

Report the commit hash and wait for explicit user instruction to continue.

---

### Task 2: Make Source Switcher A Fast Switcher

**Goal:** Separate quick switching from destructive management, close the switcher after source/group selection, and reduce noisy empty search buckets.

**Files:**
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
- Modify: `src/lib/components/analysis/source-switcher-panel.svelte`
- Test: `src/lib/analysis-compact-source-rail.test.ts`
- Test: `src/lib/analysis-redesign-route-contract.test.ts`

- [ ] **Step 1: Add failing contract tests**

Add raw source expectations:

```ts
import compactRailSource from "./components/analysis/compact-source-rail.svelte?raw";
import sourceSwitcherSource from "./components/analysis/source-switcher-panel.svelte?raw";

it("closes the source switcher after quick source or group selection", () => {
  expect(compactRailSource).toContain("selectSourceAndClose");
  expect(compactRailSource).toContain("selectGroupAndClose");
});

it("keeps destructive source deletion out of quick switch rows", () => {
  expect(sourceSwitcherSource).not.toContain("onDeleteSource(source)");
  expect(sourceSwitcherSource).toContain("Manage sources");
});
```

- [ ] **Step 2: Run the targeted tests and confirm failure**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-redesign-route-contract.test.ts
```

Expected: FAIL because wrapper functions and deletion demotion do not exist yet.

- [ ] **Step 3: Add close-after-select wrappers**

In `compact-source-rail.svelte`, add:

```ts
function selectSourceAndClose(sourceId: number) {
  onSelectSource(sourceId);
  sourceSwitcherOpen = false;
}

function selectGroupAndClose(groupId: number) {
  onSelectGroup(groupId);
  sourceSwitcherOpen = false;
}
```

Pass them to `SourceSwitcherPanel`:

```svelte
onSelectSource={selectSourceAndClose}
onSelectGroup={selectGroupAndClose}
```

Keep quick rail icon buttons using the direct `onSelectSource` and `onSelectGroup` callbacks, because those buttons are outside the panel and do not need to close it.

- [ ] **Step 4: Remove destructive delete from quick switch cards**

In `source-switcher-panel.svelte`:

- remove `Trash2` from the lucide import;
- remove `onDeleteSource` from props if it becomes unused;
- remove the `Delete` button from `.row-actions`;
- keep `Manage sources` as the path to source management and destructive actions.

If the prop must remain because parent wiring still passes it, leave the prop but do not render a delete action in the quick switch rows.

- [ ] **Step 5: Quiet empty buckets during search**

Change empty bucket copy so a nonmatching bucket does not dominate the panel while another bucket has matches:

```svelte
{:else if filteredSourceCatalog.length === 0 && filteredGroups.length === 0}
  <div class="panel-empty">No sources or groups match the current search.</div>
{:else if filteredSourceCatalog.length === 0}
  <div class="panel-empty subtle">No source matches.</div>
```

Mirror the group section with `No group matches.`.

Add CSS:

```css
.panel-empty.subtle {
  padding: 0.35rem 0;
  border: 0;
  background: transparent;
  color: var(--muted);
}
```

- [ ] **Step 6: Verify**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-redesign-route-contract.test.ts
npm.cmd run check
git diff --check
```

Expected: all pass, no diff whitespace output.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src/lib/components/analysis/compact-source-rail.svelte src/lib/components/analysis/source-switcher-panel.svelte src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-redesign-route-contract.test.ts
git commit -m "polish: streamline analysis source switching"
```

- [ ] **Step 8: Stop**

Report the commit hash and wait for explicit user instruction to continue.

---

### Task 3: De-Duplicate Source Reader Context

**Goal:** Keep one workspace-level context title, make the reader header compact, and remove the duplicate group `Source focus` control.

**Files:**
- Modify: `src/lib/components/analysis/source-reader-header.svelte`
- Modify: `src/lib/components/analysis/source-group-reader.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Test: `src/lib/analysis-source-readers.test.ts`
- Test: `src/lib/analysis-report-canvas.test.ts`

- [ ] **Step 1: Add failing raw component tests**

```ts
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import sourceGroupReaderSource from "./components/analysis/source-group-reader.svelte?raw";

it("uses a compact source reader heading instead of repeating the selected title", () => {
  expect(sourceReaderHeaderSource).toContain("surfaceLabel");
  expect(sourceReaderHeaderSource).not.toContain("<h2>{title}</h2>");
});

it("keeps source focus controls in one reader header location", () => {
  expect(sourceGroupReaderSource).not.toContain("<span>Source focus</span>");
  expect(sourceGroupReaderSource).not.toContain("group-filter");
});
```

- [ ] **Step 2: Run targeted tests and confirm failure**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts
```

Expected: FAIL because the duplicated title and group filter still exist.

- [ ] **Step 3: Compact `SourceReaderHeader`**

In `source-reader-header.svelte`, replace repeated title rendering with a semantic surface label:

```ts
let {
  title,
  subtitle,
  surfaceLabel = "Source material",
  ...
}: {
  title: string;
  subtitle: string;
  surfaceLabel?: string;
  ...
} = $props();
```

Render:

```svelte
<div class="reader-title">
  <span class="eyebrow">{surfaceLabel}</span>
  <p>{subtitle}</p>
</div>
```

Do not render `<h2>{title}</h2>` in this component. Keep `title` in the type if callers still pass it and if it is useful as future context, but the visible repeated title should go away.

- [ ] **Step 4: Remove duplicate group focus in `SourceGroupReader`**

In `source-group-reader.svelte`, remove:

```svelte
{#if allSourceGroups.length > 1}
  <label class="group-filter">
    ...
  </label>
{/if}
```

Keep `selectedGroupSourceId` filtering logic because `SourceReaderHeader` remains the single control location.

Remove `.group-filter` CSS.

- [ ] **Step 5: Pass a compact label from the source surface**

In `report-source-surface.svelte`, pass a label such as:

```svelte
surfaceLabel={workspaceSelection.kind === "source_group" ? "Group sources" : "Source material"}
```

Use the local selection variable names that already exist in the file.

- [ ] **Step 6: Verify**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts
npm.cmd run check
git diff --check
```

Expected: all pass, no diff whitespace output.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src/lib/components/analysis/source-reader-header.svelte src/lib/components/analysis/source-group-reader.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts
git commit -m "polish: reduce analysis source reader repetition"
```

- [ ] **Step 8: Stop**

Report the commit hash and wait for explicit user instruction to continue.

---

### Task 4: Polish YouTube Transcript Search

**Goal:** Make transcript search read as one compact input with the search icon inside and a visible placeholder.

**Files:**
- Modify: `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Test: `src/lib/analysis-source-readers.test.ts`
- Test: `src/lib/analysis-redesign-safety-contract.test.ts`

- [ ] **Step 1: Add failing raw component tests**

```ts
import youtubeTranscriptSource from "./components/analysis/youtube-transcript-reader.svelte?raw";

it("renders transcript search as one compact input shell", () => {
  expect(youtubeTranscriptSource).toContain('placeholder="Search transcript"');
  expect(youtubeTranscriptSource).toContain('class="search-icon"');
  expect(youtubeTranscriptSource).toContain('class="search-input-wrap"');
});
```

- [ ] **Step 2: Run targeted tests and confirm failure**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-redesign-safety-contract.test.ts
```

Expected: FAIL because placeholder and shell classes are missing.

- [ ] **Step 3: Implement compact search shell**

In `youtube-transcript-reader.svelte`, change the search markup to:

```svelte
<label class="search-field">
  <span class="sr-only">Search transcript</span>
  <div class="search-input-wrap">
    <Search class="search-icon" size={15} aria-hidden="true" />
    <Input
      type="search"
      value={transcriptSearch}
      placeholder="Search transcript"
      ariaLabel="Search transcript"
      oninput={(event) => onChangeTranscriptSearch(inputValue(event))}
    />
  </div>
</label>
```

If there is no existing `.sr-only` helper in global CSS, add a local one:

```css
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
```

Add CSS:

```css
.search-input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 0.75rem;
  z-index: 1;
  color: var(--muted);
  pointer-events: none;
}

.search-input-wrap :global(input) {
  padding-left: 2.15rem;
}
```

- [ ] **Step 4: Verify**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-redesign-safety-contract.test.ts
npm.cmd run check
git diff --check
```

Expected: all pass, no diff whitespace output.

- [ ] **Step 5: Commit**

Run:

```powershell
git add src/lib/components/analysis/youtube-transcript-reader.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-redesign-safety-contract.test.ts
git commit -m "polish: refine transcript search control"
```

- [ ] **Step 6: Stop**

Report the commit hash and wait for explicit user instruction to continue.

---

### Task 5: Tighten Mobile Quick Source Rail

**Goal:** Reduce first-screen height on narrow/mobile layouts by making the quick-source rail a compact horizontal context bar.

**Files:**
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
- Test: `src/lib/analysis-compact-source-rail.test.ts`
- Test: `src/lib/analysis-redesign-route-contract.test.ts`

- [ ] **Step 1: Add failing raw component tests**

```ts
import compactRailSource from "./components/analysis/compact-source-rail.svelte?raw";

it("uses a compact mobile source context bar", () => {
  expect(compactRailSource).toContain("mobile-current-label");
  expect(compactRailSource).toContain("quick-list-scroll");
  expect(compactRailSource).toContain("@media (max-width: 720px)");
});
```

- [ ] **Step 2: Run targeted tests and confirm failure**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-redesign-route-contract.test.ts
```

Expected: FAIL because the mobile context bar names are missing.

- [ ] **Step 3: Add a visible compact current context label for narrow layouts**

In `compact-source-rail.svelte`, derive:

```ts
const currentContextLabel = $derived(
  currentSource
    ? (youtubeSummaries[currentSource.id]?.title ?? currentSource.title ?? currentSource.externalId)
    : currentGroup
      ? currentGroup.name
      : "Choose source",
);
```

Inside `.rail-top`, after `.current-context-button`, add:

```svelte
<button
  class="mobile-current-label"
  type="button"
  onclick={() => (sourceSwitcherOpen = true)}
>
  <span>{currentContextLabel}</span>
</button>
```

- [ ] **Step 4: Wrap quick choices for horizontal scrolling**

Keep the existing `.quick-list` markup, but add a stable class hook:

```svelte
<div class="quick-list quick-list-scroll" aria-label="Quick source choices">
```

- [ ] **Step 5: Add mobile rail CSS**

Add:

```css
.mobile-current-label {
  display: none;
  min-width: 0;
  border: 0;
  background: transparent;
  color: var(--text);
  text-align: left;
  cursor: pointer;
}

.mobile-current-label span {
  display: block;
  max-width: 14rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 720px) {
  .compact-source-rail {
    padding: 0.45rem;
    gap: 0.4rem;
  }

  .rail-top {
    min-width: 0;
    flex: 0 0 auto;
  }

  .current-context-button {
    width: 2.35rem;
    min-height: 2.35rem;
  }

  .current-context-button :global(svg) {
    display: none;
  }

  .mobile-current-label {
    display: inline-flex;
  }

  .quick-list-scroll {
    flex: 1 1 auto;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: thin;
  }
}
```

- [ ] **Step 6: Verify**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-redesign-route-contract.test.ts
npm.cmd run check
git diff --check
```

Expected: all pass, no diff whitespace output.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src/lib/components/analysis/compact-source-rail.svelte src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-redesign-route-contract.test.ts
git commit -m "polish: tighten mobile analysis source rail"
```

- [ ] **Step 8: Stop**

Report the commit hash and wait for explicit user instruction to continue.

---

### Task 6: Fixture-Backed Verification And Documentation

**Goal:** Verify pass 2 as a complete UX pass in the real Tauri app and document the result.

**Files:**
- Modify: `docs/superpowers/verification/2026-05-11-analysis-redesign-ux-polish.md`
- Create artifacts under: `artifacts/`

- [ ] **Step 1: Run full automated verification**

Run:

```powershell
npm.cmd test
npm.cmd run check
git diff --check
```

Expected:

```text
tests pass
0 errors, 0 warnings
git diff --check has no output
```

- [ ] **Step 2: Start Tauri dev**

Run:

```powershell
npm.cmd run tauri dev
```

If sandbox blocks the command or the app needs unsandboxed GUI/process access, request escalation for this command.

- [ ] **Step 3: Connect Tauri MCP and seed fixtures**

Use the Tauri MCP bridge, then run in the webview:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

Expected seed shape:

```json
{
  "accounts": 1,
  "chatMessages": 2,
  "llmProfiles": 1,
  "promptTemplates": 1,
  "runs": 6,
  "snapshotMessages": 4,
  "sourceGroups": 1,
  "sources": 4,
  "youtubePlaylistItems": 2,
  "youtubeTranscriptSegments": 3
}
```

- [ ] **Step 4: Inspect required states**

Capture screenshots for:

```text
artifacts/analysis-ux-pass-2-runs-filters.png
artifacts/analysis-ux-pass-2-source-switcher.png
artifacts/analysis-ux-pass-2-source-reader.png
artifacts/analysis-ux-pass-2-youtube-search.png
artifacts/analysis-ux-pass-2-mobile-rail.png
```

Verify:

- Runs tab shows at least the first run card sooner on desktop.
- Advanced filters expand and preserve date/provider/model/template filtering.
- Source switcher closes after selecting a source or group.
- Destructive source deletion is not shown in quick switch rows.
- Source reader has one source focus control in group mode.
- YouTube transcript search looks like one compact input.
- Mobile rail avoids repeated tall context chrome and has no horizontal overflow.

- [ ] **Step 5: Update verification document**

Append a `Pass 2 Implementation Verification` section to `docs/superpowers/verification/2026-05-11-analysis-redesign-ux-polish.md` with:

```markdown
## Pass 2 Implementation Verification

Date: 2026-05-12
Branch: `polish/analysis-ux-pass-2`

Batch 2 fixes implemented:

- Runs date/provider/model/template filters moved behind `Advanced filters`.
- Source switcher now behaves as a quick switcher and closes after source/group selection.
- Source deletion is managed outside the quick switcher path.
- Source reader context is less repetitive, with one source focus control.
- YouTube transcript search is a compact input with inline icon and placeholder.
- Mobile quick-source rail is shorter and more context-aware.

Automated verification:

```text
npm.cmd test
npm.cmd run check
git diff --check
```

Runtime verification used Tauri dev mode with seeded analysis redesign fixtures.

New artifacts:

```text
artifacts/analysis-ux-pass-2-runs-filters.png
artifacts/analysis-ux-pass-2-source-switcher.png
artifacts/analysis-ux-pass-2-source-reader.png
artifacts/analysis-ux-pass-2-youtube-search.png
artifacts/analysis-ux-pass-2-mobile-rail.png
```

Observed pass notes:

- [replace with concrete observed notes from runtime verification]
```

Replace the bracketed observed note before committing.

- [ ] **Step 6: Commit**

Run:

```powershell
git add docs/superpowers/verification/2026-05-11-analysis-redesign-ux-polish.md
git commit -m "docs: record analysis ux polish pass 2"
```

- [ ] **Step 7: Stop**

Report the commit hash, summarize verification evidence, and wait for explicit user instruction about merge or further review.
