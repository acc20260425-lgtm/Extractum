# Diagnostics Problem-First Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `/diagnostics` show actionable issue tables immediately in `Only issues` mode while preserving the full overview-first report order in `All tables`.

**Architecture:** Keep the change inside the existing Diagnostics route. Reuse the existing issue-row helpers and `DiagnosticCountTable`; split route markup into render snippets so the same overview and table areas can be ordered by mode without duplicating table logic.

**Tech Stack:** Svelte 5 snippets, SvelteKit route component, Vitest raw-source contract tests, existing `$lib/components/ui/*` primitives, `npm.cmd run test`, `npm.cmd run check`, Tauri MCP live inspection.

---

## File Structure

- Modify: `src/lib/diagnostics-ux-contract.test.ts`
  - Owns raw-source UX contracts for Diagnostics page ordering and empty issue-mode behavior.
- Modify: `src/routes/diagnostics/+page.svelte`
  - Owns Diagnostics mode state, visible table sections, route-level section ordering, and responsive layout CSS.
- Reference only: `src/lib/diagnostics-view-model.ts`
  - Existing `diagnosticRowHasIssue` and `filterDiagnosticIssueRows` helpers remain unchanged.
- Reference only: `src/lib/components/diagnostics/DiagnosticCountTable.svelte`
  - Existing table renderer remains unchanged; empty mode for no matching issue sections is handled by the route.
- Modify: `docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md`
  - Mark executed steps as complete during implementation.

---

### Task 1: Freeze The Diagnostics Problem-First Contract

**Files:**
- Modify: `src/lib/diagnostics-ux-contract.test.ts`
- Modify: `docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md`

- [ ] **Step 1: Add failing route-order assertions**

In `src/lib/diagnostics-ux-contract.test.ts`, add this test inside the existing `describe("diagnostics UX contract", () => { ... })` block:

```ts
  it("orders diagnostics issue details before overview only in issue mode", () => {
    expect(diagnosticsPageSource).toContain('{#if diagnosticsTableMode === "issues"}');
    expect(diagnosticsPageSource).toContain("{@render diagnosticsTableArea(tableSections)}");
    expect(diagnosticsPageSource).toContain("{@render diagnosticsOverviewArea(summary)}");

    const issueBranchIndex = diagnosticsPageSource.indexOf('{#if diagnosticsTableMode === "issues"}');
    const issueTableIndex = diagnosticsPageSource.indexOf(
      "{@render diagnosticsTableArea(tableSections)}",
      issueBranchIndex,
    );
    const issueOverviewIndex = diagnosticsPageSource.indexOf(
      "{@render diagnosticsOverviewArea(summary)}",
      issueBranchIndex,
    );
    const allBranchIndex = diagnosticsPageSource.indexOf("{:else}", issueBranchIndex);
    const allOverviewIndex = diagnosticsPageSource.indexOf(
      "{@render diagnosticsOverviewArea(summary)}",
      allBranchIndex,
    );
    const allTableIndex = diagnosticsPageSource.indexOf(
      "{@render diagnosticsTableArea(tableSections)}",
      allBranchIndex,
    );

    expect(issueBranchIndex).toBeGreaterThan(0);
    expect(issueTableIndex).toBeGreaterThan(issueBranchIndex);
    expect(issueOverviewIndex).toBeGreaterThan(issueBranchIndex);
    expect(issueTableIndex).toBeLessThan(issueOverviewIndex);
    expect(allBranchIndex).toBeGreaterThan(issueBranchIndex);
    expect(allOverviewIndex).toBeGreaterThan(allBranchIndex);
    expect(allTableIndex).toBeGreaterThan(allBranchIndex);
    expect(allOverviewIndex).toBeLessThan(allTableIndex);
  });
```

- [ ] **Step 2: Add failing empty issue-mode area assertions**

In the same file, add this test after the route-order test:

```ts
  it("renders an immediate table-area empty state when issue mode has no matching rows", () => {
    expect(diagnosticsPageSource).toContain("visibleDiagnosticsTableSections");
    expect(diagnosticsPageSource).toContain('class="diagnostics-table-area diagnostics-tables"');
    expect(diagnosticsPageSource).toContain('class="diagnostics-overview-area"');
    expect(diagnosticsPageSource).toContain("No diagnostic issue rows match this view.");
  });
```

- [ ] **Step 3: Run the focused UX contract and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/diagnostics-ux-contract.test.ts
```

Expected: FAIL because `diagnosticsTableArea`, `diagnosticsOverviewArea`, `visibleDiagnosticsTableSections`, and the issue-mode empty state do not exist yet.

- [ ] **Step 4: Mark Task 1 steps complete in this plan**

In `docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md`, change Task 1 checkboxes completed so the implementation history is visible in the plan.

- [ ] **Step 5: Commit the failing contract**

Run:

```powershell
git add src/lib/diagnostics-ux-contract.test.ts docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md
git commit -m "test: capture diagnostics problem-first contract"
```

---

### Task 2: Reorder Diagnostics Content By Table Mode

**Files:**
- Modify: `src/routes/diagnostics/+page.svelte`
- Modify: `src/lib/diagnostics-ux-contract.test.ts`
- Modify: `docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md`

- [ ] **Step 1: Add a visible table-section type**

In `src/routes/diagnostics/+page.svelte`, add this type immediately after `type DiagnosticTableSection = { ... };`:

```ts
  type VisibleDiagnosticTableSection = DiagnosticTableSection & {
    visibleRows: DiagnosticTableRow[];
  };
```

- [ ] **Step 2: Add visible table-section helper**

In `src/routes/diagnostics/+page.svelte`, add this function immediately after `visibleDiagnosticRows`:

```ts
  function visibleDiagnosticsTableSections(sections: DiagnosticTableSection[]): VisibleDiagnosticTableSection[] {
    return sections
      .map((section) => ({
        ...section,
        visibleRows: visibleDiagnosticRows(section.rows),
      }))
      .filter((section) => diagnosticsTableMode === "all" || section.visibleRows.length > 0);
  }
```

- [ ] **Step 3: Replace the summary-and-table markup with mode-dependent renders**

In `src/routes/diagnostics/+page.svelte`, keep this block unchanged:

```svelte
    <div class="diagnostics-table-controls" aria-label="Diagnostics table display">
      <Button size="sm" variant="secondary" selected={diagnosticsTableMode === "issues"} onclick={() => (diagnosticsTableMode = "issues")}>Only issues</Button>
      <Button size="sm" variant="secondary" selected={diagnosticsTableMode === "all"} onclick={() => (diagnosticsTableMode = "all")}>All tables</Button>
    </div>
```

Delete the existing blocks that start with:

```svelte
    <div class="status-strip" aria-label="Diagnostics health overview">
```

and end after the current:

```svelte
    <div class="diagnostics-tables">
      {#each tableSections as section (section.title)}
        {#if diagnosticsTableMode === "all" || visibleDiagnosticRows(section.rows).length > 0}
          <DiagnosticCountTable
            title={section.title}
            description={section.description}
            columns={section.columns}
            rows={visibleDiagnosticRows(section.rows)}
            totalRows={section.rows.length}
            open={hasDiagnosticIssue(section.rows)}
          />
        {/if}
      {/each}
    </div>
```

Insert this mode-dependent render block in the same location:

```svelte
    {#if diagnosticsTableMode === "issues"}
      {@render diagnosticsTableArea(tableSections)}
      {@render diagnosticsOverviewArea(summary)}
    {:else}
      {@render diagnosticsOverviewArea(summary)}
      {@render diagnosticsTableArea(tableSections)}
    {/if}
```

- [ ] **Step 4: Add the Diagnostics overview snippet**

In `src/routes/diagnostics/+page.svelte`, add this snippet after the closing `</section>` and before `<style>`:

```svelte
{#snippet diagnosticsOverviewArea(current: DiagnosticSummaryDto)}
  <div class="diagnostics-overview-area">
    <div class="status-strip" aria-label="Diagnostics health overview">
      {#each statusStripItems(current) as item (item.label)}
        <div class="status-tile">
          <span>{item.label}</span>
          <strong>{item.value}</strong>
          <Badge variant={item.tone}>{item.meta}</Badge>
        </div>
      {/each}
    </div>

    <div class="diagnostics-grid">
      <SurfaceCard title="App and build" meta="Factual diagnostic summary metadata">
        <div class="meta-grid">
          <MetaCell label="App">{current.app.appName}</MetaCell>
          <MetaCell label="Version">{current.app.appVersion}</MetaCell>
          <MetaCell label="Build">{labelFromKey(current.app.buildMode)}</MetaCell>
          <MetaCell label="Generated">{formatSummaryGeneratedAt(current.app.generatedAtUnix).replace("Summary generated ", "")}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Database" meta="SQLite availability and migration state">
        <div class="meta-grid">
          <MetaCell label="SQLite">{availabilityLabel(current.database.sqliteAvailable)}</MetaCell>
          <MetaCell label="Migrations">{labelFromKey(current.database.migrations.status)}</MetaCell>
          <MetaCell label="Accounts">{current.database.accountCount}</MetaCell>
          <MetaCell label="Pending versions">{current.database.migrations.pendingVersions.length}</MetaCell>
          <MetaCell label="Failed versions">{current.database.migrations.failedVersions.length}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Runtimes" meta="Backend-reported runtime checks">
        <div class="meta-grid">
          <MetaCell label="Secure storage">{labelFromKey(current.runtimes.secureStorage.status)}</MetaCell>
          <MetaCell label="Secure storage available">{availabilityLabel(current.runtimes.secureStorage.available)}</MetaCell>
          <MetaCell label="yt-dlp">{labelFromKey(current.runtimes.ytdlp.status)}</MetaCell>
          <MetaCell label="yt-dlp available">{availabilityLabel(current.runtimes.ytdlp.available)}</MetaCell>
          <MetaCell label="yt-dlp version">{current.runtimes.ytdlp.version ?? "Unknown"}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Privacy boundary" meta="Data classes intentionally excluded by backend diagnostics">
        {#if privacyLabels(current).length > 0}
          <div class="privacy-chips">
            {#each privacyLabels(current) as item (item)}
              <Badge variant="neutral">{item}</Badge>
            {/each}
          </div>
        {:else}
          <StatusMessage tone="muted" surface={false}>{privacyNote(current)}</StatusMessage>
        {/if}
      </SurfaceCard>
    </div>
  </div>
{/snippet}
```

- [ ] **Step 5: Add the Diagnostics table-area snippet**

In `src/routes/diagnostics/+page.svelte`, add this snippet immediately after the overview snippet and before `<style>`:

```svelte
{#snippet diagnosticsTableArea(tableSections: DiagnosticTableSection[])}
  {@const visibleSections = visibleDiagnosticsTableSections(tableSections)}
  <div class="diagnostics-table-area diagnostics-tables">
    {#each visibleSections as section (section.title)}
      <DiagnosticCountTable
        title={section.title}
        description={section.description}
        columns={section.columns}
        rows={section.visibleRows}
        totalRows={section.rows.length}
        open={hasDiagnosticIssue(section.rows)}
      />
    {:else}
      <StatusMessage tone="muted" className="diagnostics-empty-state">
        No diagnostic issue rows match this view.
      </StatusMessage>
    {/each}
  </div>
{/snippet}
```

- [ ] **Step 6: Add layout CSS for the new areas**

In `src/routes/diagnostics/+page.svelte`, add these CSS rules before the existing `.status-strip` rule:

```css
  .diagnostics-overview-area {
    display: flex;
    flex-direction: column;
    gap: 0.95rem;
  }
```

Add this rule after the existing `.diagnostics-tables` rule:

```css
  :global(.diagnostics-empty-state.ui-status-message) {
    grid-column: 1 / -1;
  }
```

Keep the existing `.diagnostics-grid, .diagnostics-tables` selector so the table area keeps its two-column desktop layout and one-column narrow layout.

- [ ] **Step 7: Run focused Diagnostics tests**

Run:

```powershell
npm.cmd run test -- src/lib/diagnostics-ux-contract.test.ts src/lib/diagnostics-route-contract.test.ts src/lib/diagnostics-view-model.test.ts
```

Expected: PASS. The focused set should include the new route-order raw-source checks and the existing row-filter helper checks.

- [ ] **Step 8: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 9: Mark Task 2 steps complete in this plan**

In `docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md`, change Task 2 checkboxes completed after the focused tests and Svelte check pass.

- [ ] **Step 10: Commit the implementation**

Run:

```powershell
git add src/routes/diagnostics/+page.svelte src/lib/diagnostics-ux-contract.test.ts docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md
git commit -m "feat(diagnostics): surface issue tables first"
```

---

### Task 3: Verify Live Diagnostics Layout And Merge Readiness

**Files:**
- Modify: `docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md`
- Modify only files from Task 2 if verification exposes a small route or test issue.

- [ ] **Step 1: Run full Vitest suite**

Run:

```powershell
npm.cmd run test
```

Expected: all Vitest files pass. The previous baseline was `71` files and `657` tests; the exact count can increase after Task 1.

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 3: Inspect `/diagnostics` at normal desktop width**

With the Tauri app running, connect to the MCP bridge and resize the main window to about `1280x860`. If the session is not connected, run the Tauri MCP `driver_session` action `start` with host `localhost` and port `9223`.

Run this JavaScript in the main webview on `/diagnostics`:

```js
(() => {
  if (location.pathname !== "/diagnostics") {
    window.location.href = "/diagnostics";
    return { navigating: true };
  }
  const qsa = (selector) => Array.from(document.querySelectorAll(selector));
  const rect = (el) => {
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return {
      top: Math.round(r.top),
      bottom: Math.round(r.bottom),
      width: Math.round(r.width),
      height: Math.round(r.height),
    };
  };
  const controls = document.querySelector(".diagnostics-table-controls");
  const tableArea = document.querySelector(".diagnostics-table-area");
  const overviewArea = document.querySelector(".diagnostics-overview-area");
  const firstTable = document.querySelector(".diagnostics-table-area table");
  return {
    viewport: { width: innerWidth, height: innerHeight },
    overflowX: document.documentElement.scrollWidth > document.documentElement.clientWidth,
    controls: rect(controls),
    tableArea: rect(tableArea),
    overviewArea: rect(overviewArea),
    firstTable: rect(firstTable),
    tableBeforeOverview: !!tableArea && !!overviewArea && tableArea.getBoundingClientRect().top < overviewArea.getBoundingClientRect().top,
    firstIssueTableInFirstViewport: !!firstTable && firstTable.getBoundingClientRect().top < innerHeight,
    tableSummaries: qsa(".diagnostic-count-table summary").map((summary) => summary.innerText.trim()),
  };
})()
```

Expected:

- `overflowX: false`
- `tableBeforeOverview: true`
- `firstIssueTableInFirstViewport: true`
- table summaries include filtered counts such as `Analysis runs\n1/2 rows` and `Ingest batches\n3/4 rows` when the local data has those issue rows.

- [ ] **Step 4: Inspect `/diagnostics` at about `900px` width**

Resize the same window to about `900x760`, keep `/diagnostics` in `Only issues`, and run the same JavaScript from Step 3.

Expected:

- `overflowX: false`
- `tableBeforeOverview: true`
- `firstIssueTableInFirstViewport: true`
- the first issue table is no longer pushed below all healthy overview cards.

- [ ] **Step 5: Verify `All tables` keeps overview-first order**

In the running `/diagnostics` page, click `All tables` or run this JavaScript:

```js
(() => {
  Array.from(document.querySelectorAll("button")).find((button) => button.innerText.trim() === "All tables")?.click();
  const tableArea = document.querySelector(".diagnostics-table-area");
  const overviewArea = document.querySelector(".diagnostics-overview-area");
  return {
    modeClicked: true,
    overviewBeforeTables: !!tableArea && !!overviewArea && overviewArea.getBoundingClientRect().top < tableArea.getBoundingClientRect().top,
    tableSummaries: Array.from(document.querySelectorAll(".diagnostic-count-table summary")).map((summary) => summary.innerText.trim()),
  };
})()
```

Expected:

- `overviewBeforeTables: true`
- table summaries use full counts, such as `Analysis runs\n2 rows` and `Ingest batches\n4 rows` when the local data has those rows.

- [ ] **Step 6: Restore the app window**

Resize the Tauri main window back to about `1280x860` so the user's app is left in the expected desktop shape.

- [ ] **Step 7: Mark Task 3 verification steps complete in this plan**

In `docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md`, change Task 3 checkboxes completed after verification passes.

- [ ] **Step 8: Commit verification notes or fixes**

If only this plan file changed because checkboxes were marked, run:

```powershell
git add docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md
git commit -m "docs(superpowers): complete diagnostics problem-first plan"
```

If verification required a small source fix in Task 2 files, include those files in the same commit:

```powershell
git add src/routes/diagnostics/+page.svelte src/lib/diagnostics-ux-contract.test.ts docs/superpowers/plans/2026-06-04-diagnostics-problem-first.md
git commit -m "fix(diagnostics): polish problem-first layout"
```

---

## Execution Notes

- Commit after every task.
- Use TDD: commit the failing raw-source contract before changing route markup.
- Keep changes scoped to `/diagnostics` frontend layout and tests.
- Do not modify backend diagnostics commands, diagnostics DTOs, database code, or Tauri IPC.
- Do not commit temporary screenshots or files under `tmp/`.
- If the app is already running on port `1420`, do not treat a dev-server port warning after successful UI checks as a layout failure.
