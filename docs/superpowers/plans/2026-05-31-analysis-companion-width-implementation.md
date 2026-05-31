# Analysis Companion Width Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the analysis run companion, especially Evidence, readable on desktop by widening the companion column and making Evidence list/detail layout depend on panel width.

**Architecture:** Keep this as a CSS/layout-only slice. Add one focused raw-source contract test file that locks the workspace grid, Evidence container query, and no-special-layout scope for Chat/Chunks/Runs. Then change only the analysis workspace grid and Evidence/TracePanel CSS.

**Tech Stack:** Svelte 5, SvelteKit, Vitest raw component contract tests, Tauri MCP/manual viewport verification.

---

## Source Spec

- `docs/superpowers/specs/2026-05-31-analysis-companion-width-design.md`

## Files

Create:

- `src/lib/analysis-companion-layout.test.ts`
  - Focused raw-source contract tests for the analysis workspace grid and Evidence container-query layout.

Modify:

- `src/routes/analysis/+page.svelte`
  - Update `.analysis-workspace` desktop grid companion column from the old 430px cap to a wider bounded `clamp(...)` track.
  - Preserve existing `@media (max-width: 1500px)` and `@media (max-width: 1180px)` behavior.
- `src/lib/components/analysis/run-evidence-tab.svelte`
  - Add `container-type: inline-size` to `.run-evidence-tab`.
- `src/lib/components/analysis/trace-panel.svelte`
  - Replace viewport-only `@media (min-width: 1280px)` two-column Evidence layout with `@container (min-width: 34rem)`.

Do not modify:

- `src/lib/components/analysis/run-chat-tab.svelte`
- `src/lib/components/analysis/chunk-summaries.svelte`
- `src/lib/components/analysis/run-companion-runs-tab.svelte`

Those tabs inherit the wider companion column but do not receive special inner layout changes in this slice.

---

## Task 1: Add Layout Contract Tests

**Files:**

- Create: `src/lib/analysis-companion-layout.test.ts`

- [ ] **Step 1: Write the failing raw-source contract tests**

Create `src/lib/analysis-companion-layout.test.ts` with this content:

```ts
import { describe, expect, it } from "vitest";
import rawAnalysisPageSource from "../routes/analysis/+page.svelte?raw";
import rawChunkSummariesSource from "./components/analysis/chunk-summaries.svelte?raw";
import rawRunChatTabSource from "./components/analysis/run-chat-tab.svelte?raw";
import rawRunCompanionRunsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";
import rawRunEvidenceTabSource from "./components/analysis/run-evidence-tab.svelte?raw";
import rawTracePanelSource from "./components/analysis/trace-panel.svelte?raw";

const analysisPageSource = normalizeLineEndings(rawAnalysisPageSource);
const chunkSummariesSource = normalizeLineEndings(rawChunkSummariesSource);
const runChatTabSource = normalizeLineEndings(rawRunChatTabSource);
const runCompanionRunsTabSource = normalizeLineEndings(rawRunCompanionRunsTabSource);
const runEvidenceTabSource = normalizeLineEndings(rawRunEvidenceTabSource);
const tracePanelSource = normalizeLineEndings(rawTracePanelSource);

function normalizeLineEndings(source: string) {
  return source.replace(/\r\n/g, "\n");
}

function sourceBetween(source: string, start: string, end: string) {
  const startIndex = source.indexOf(start);
  expect(startIndex, `missing start marker: ${start}`).toBeGreaterThanOrEqual(0);
  const endIndex = source.indexOf(end, startIndex + start.length);
  expect(endIndex, `missing end marker after ${start}: ${end}`).toBeGreaterThan(startIndex);
  return source.slice(startIndex, endIndex + end.length);
}

describe("analysis companion layout", () => {
  it("widens the desktop companion column while preserving existing stacking breakpoints", () => {
    const workspaceRule = sourceBetween(analysisPageSource, ".analysis-workspace {", "\n  }\n");
    const mediumBreakpoint = sourceBetween(
      analysisPageSource,
      "@media (max-width: 1500px)",
      "@media (max-width: 1180px)",
    );
    const narrowBreakpoint = sourceBetween(
      analysisPageSource,
      "@media (max-width: 1180px)",
      "</style>",
    );

    expect(workspaceRule).toContain("minmax(4.25rem, 4.75rem)");
    expect(workspaceRule).toContain("minmax(0, 1.45fr)");
    expect(workspaceRule).toContain("minmax(420px, clamp(480px, 30vw, 560px))");
    expect(workspaceRule).not.toContain("minmax(320px, 430px)");

    expect(mediumBreakpoint).toContain("@media (max-width: 1500px)");
    expect(mediumBreakpoint).toContain("grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1fr);");
    expect(mediumBreakpoint).toContain("grid-column: 2;");

    expect(narrowBreakpoint).toContain("@media (max-width: 1180px)");
    expect(narrowBreakpoint).toContain("grid-template-columns: 1fr;");
    expect(narrowBreakpoint).toContain("grid-column: 1;");
  });

  it("uses Evidence panel width, not viewport width, for trace list/detail columns", () => {
    const evidenceRootRule = sourceBetween(runEvidenceTabSource, ".run-evidence-tab {", "\n  }\n");
    const traceBaseRule = sourceBetween(tracePanelSource, ".trace-layout {", "\n  }\n");
    const containerRule = sourceBetween(tracePanelSource, "@container (min-width: 34rem)", "</style>");

    expect(evidenceRootRule).toContain("container-type: inline-size;");

    expect(traceBaseRule).toContain("grid-template-columns: minmax(0, 1fr);");
    expect(tracePanelSource).not.toContain("@media (min-width: 1280px)");

    expect(containerRule).toContain(".trace-layout {");
    expect(containerRule).toContain("grid-template-columns: minmax(12rem, 0.9fr) minmax(16rem, 1.1fr);");
    expect(containerRule).toContain("align-items: start;");
    expect(containerRule).toContain(".trace-detail {");
    expect(containerRule).toContain("padding-left: 0.9rem;");
    expect(containerRule).toContain("border-left: 1px solid var(--border);");
    expect(containerRule).not.toContain("minmax(0, 0.95fr) minmax(0, 1.05fr)");
  });

  it("does not add companion-width-specific inner layouts to Chat, Chunks, or Runs", () => {
    for (const source of [runChatTabSource, chunkSummariesSource, runCompanionRunsTabSource]) {
      expect(source).not.toContain("container-type:");
      expect(source).not.toContain("@container");
      expect(source).not.toContain("analysis companion width");
    }
  });
});
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-companion-layout.test.ts
```

Expected: FAIL. The failures should include missing `minmax(420px, clamp(480px, 30vw, 560px))`, missing `container-type: inline-size`, and the old `@media (min-width: 1280px)` still being present.

---

## Task 2: Implement Workspace And Evidence CSS

**Files:**

- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/run-evidence-tab.svelte`
- Modify: `src/lib/components/analysis/trace-panel.svelte`
- Test: `src/lib/analysis-companion-layout.test.ts`

- [ ] **Step 1: Widen the desktop companion column**

In `src/routes/analysis/+page.svelte`, find the `.analysis-workspace` CSS rule and replace the current desktop grid:

```css
grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.6fr) minmax(320px, 430px);
```

with:

```css
grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.45fr) minmax(420px, clamp(480px, 30vw, 560px));
```

Do not change these existing breakpoint blocks:

```css
@media (max-width: 1500px) {
  .analysis-workspace {
    grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1fr);
  }

  .companion-slot {
    grid-column: 2;
  }
}

@media (max-width: 1180px) {
  .analysis-workspace {
    grid-template-columns: 1fr;
  }

  .companion-slot {
    grid-column: 1;
  }
}
```

- [ ] **Step 2: Make the Evidence tab root the container**

In `src/lib/components/analysis/run-evidence-tab.svelte`, update the `.run-evidence-tab` rule from:

```css
.run-evidence-tab {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
```

to:

```css
.run-evidence-tab {
  container-type: inline-size;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
```

- [ ] **Step 3: Replace TracePanel viewport media query with container query**

In `src/lib/components/analysis/trace-panel.svelte`, find:

```css
@media (min-width: 1280px) {
  .trace-layout {
    grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.05fr);
    align-items: start;
  }

  .trace-detail {
    padding-top: 0;
    padding-left: 0.9rem;
    border-top: 0;
    border-left: 1px solid var(--border);
    min-height: 100%;
  }
}
```

Replace it with:

```css
@container (min-width: 34rem) {
  .trace-layout {
    grid-template-columns: minmax(12rem, 0.9fr) minmax(16rem, 1.1fr);
    align-items: start;
  }

  .trace-detail {
    padding-top: 0;
    padding-left: 0.9rem;
    border-top: 0;
    border-left: 1px solid var(--border);
    min-height: 100%;
  }
}
```

- [ ] **Step 4: Run the focused layout test and verify it passes**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-companion-layout.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run existing related tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-route-effects.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-companion-layout.test.ts
```

Expected: PASS. This checks the new focused test plus existing route/source-reader raw contracts.

- [ ] **Step 6: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 7: Commit the implementation**

Run:

```powershell
git add src/lib/analysis-companion-layout.test.ts src/routes/analysis/+page.svelte src/lib/components/analysis/run-evidence-tab.svelte src/lib/components/analysis/trace-panel.svelte
git commit -m "fix: widen analysis companion evidence layout"
```

Expected: commit succeeds with the new test and implementation together.

---

## Task 3: Verify Desktop Width Behavior In The Running App

**Files:**

- No file changes expected.

- [ ] **Step 1: Connect to the running Tauri app if available**

Use the Tauri MCP bridge if the app is running:

```json
{"action":"start","port":9223}
```

Then verify backend state:

```json
{}
```

Expected: backend state reports `identifier: "org.ai.extractum"` and a visible `main` window. If the app is not running or MCP is unavailable, record that viewport verification was skipped and continue with automated tests.

- [ ] **Step 2: Measure the 1920px desktop layout**

Resize the Tauri window to a wide desktop size:

```json
{"action":"resize","windowId":"main","width":1920,"height":1010}
```

Run this webview script:

```js
(() => {
  const workspace = document.querySelector(".analysis-workspace");
  const companion = document.querySelector(".companion-slot");
  const panel = document.querySelector("#run-companion-panel");
  const traceLayout = document.querySelector(".trace-layout");
  const canvas = document.querySelector(".report-canvas, [data-smoke-id='analysis-report-canvas'], .canvas-slot");
  const rect = (el) => el
    ? Object.fromEntries(["width", "height", "left", "top"].map((key) => [key, Math.round(el.getBoundingClientRect()[key])]))
    : null;
  return {
    viewport: { width: window.innerWidth, height: window.innerHeight },
    workspaceGrid: workspace ? getComputedStyle(workspace).gridTemplateColumns : null,
    companionSlot: rect(companion),
    companionPanel: rect(panel),
    traceLayout: rect(traceLayout),
    traceLayoutGrid: traceLayout ? getComputedStyle(traceLayout).gridTemplateColumns : null,
    canvas: rect(canvas),
  };
})()
```

Expected:

- `companionSlot.width` is greater than `430`.
- `workspaceGrid` contains three columns.
- `traceLayoutGrid` contains two columns when the Evidence tab is visible and panel content width is at least `34rem`.
- `canvas.width` remains larger than the companion width.

- [ ] **Step 3: Measure an intermediate desktop width**

Resize to `1440x900`:

```json
{"action":"resize","windowId":"main","width":1440,"height":900}
```

Run the same webview script from Step 2.

Expected:

- Because the existing `@media (max-width: 1500px)` breakpoint still applies, `workspaceGrid` has the rail plus one main column.
- `.companion-slot` is below the canvas in grid column `2`.
- The canvas is not squeezed by a third companion column.

If the host window chrome or side navigation makes the effective webview width slightly different, use the script output rather than the nominal window size and record the observed width.

- [ ] **Step 4: Measure a near-breakpoint desktop width**

Resize to `1600x900`:

```json
{"action":"resize","windowId":"main","width":1600,"height":900}
```

Run the same webview script from Step 2.

Expected:

- `workspaceGrid` has three columns only if the effective webview width is above `1500px`.
- The companion column resolves around the lower clamp range instead of immediately taking `560px`.
- The canvas remains usable for the current Source/Report surface.

- [ ] **Step 5: Capture a screenshot for visual sanity if MCP is available**

Run:

```json
{"windowId":"main","format":"png","maxWidth":1200}
```

Expected: the Evidence companion no longer reads as a cramped split panel on wide desktop. If Evidence is not the selected companion tab, select it before capturing or record that the screenshot covered only the workspace grid.

---

## Task 4: Final Verification And Plan Completion

**Files:**

- Modify: `docs/superpowers/plans/2026-05-31-analysis-companion-width-implementation.md` if checking off tasks during execution.

- [ ] **Step 1: Run the full frontend test suite**

Run:

```powershell
npm.cmd run test
```

Expected: all Vitest files pass.

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 3: Run whitespace/diff hygiene**

Run:

```powershell
git diff --check
```

Expected: no output and exit code `0`.

- [ ] **Step 4: Inspect the diff for scope**

Run:

```powershell
git diff --stat HEAD~1..HEAD
```

Expected: implementation changes are limited to:

- `src/lib/analysis-companion-layout.test.ts`;
- `src/routes/analysis/+page.svelte`;
- `src/lib/components/analysis/run-evidence-tab.svelte`;
- `src/lib/components/analysis/trace-panel.svelte`;
- this plan file only if execution checkboxes were committed.

No changes should appear in Chat, Chunks, or Runs inner component files.

- [ ] **Step 5: Commit plan checkbox updates only if they changed**

If the implementation process updated checkboxes in this plan, run:

```powershell
git add docs/superpowers/plans/2026-05-31-analysis-companion-width-implementation.md
git commit -m "docs: mark companion width plan complete"
```

Expected: commit succeeds. If no plan checkbox changes were made, skip this commit.

---

## Acceptance Checklist

- [ ] On a 1920px desktop viewport, the companion panel is wider than the old 430px cap.
- [ ] Evidence two-column list/detail layout depends on the Evidence panel container width, not viewport width.
- [ ] `.trace-layout` remains single-column below the `34rem` container threshold.
- [ ] Existing `@media (max-width: 1500px)` workspace stacking behavior is preserved.
- [ ] Intermediate desktop widths around 1440px do not squeeze the canvas because the companion stacks below the canvas.
- [ ] Near-breakpoint width around 1600px keeps the canvas usable and companion near the lower clamp range.
- [ ] Chat, Chunks, and Runs receive no special inner layout changes.
- [ ] Evidence data flow, trace selection, snapshot logic, and Source navigation behavior are unchanged.
- [ ] Focused layout tests, related raw contract tests, `npm.cmd run test`, `npm.cmd run check`, and `git diff --check` pass.
