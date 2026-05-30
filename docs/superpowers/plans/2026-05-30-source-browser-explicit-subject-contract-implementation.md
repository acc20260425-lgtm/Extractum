# Source Browser Explicit Subject Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the legacy `source` compatibility prop from `SourceBrowserShell` so every shell caller identifies its browser subject through explicit `subject`.

**Architecture:** Keep the existing subject model and grouped browser data objects. `SourceBrowserShell` stops inferring a source subject from a separate `source` prop; `ReportSourceSurface` already passes explicit source, source-group, and run-snapshot subjects, so the route change is limited to removing the redundant live-source prop.

**Tech Stack:** Svelte 5, SvelteKit 2, TypeScript, Vitest raw component contract tests, `svelte-check`, project verification through `npm.cmd run verify`.

---

## Execution Protocol

- Start from `main`.
- Create a branch before Task 0.
- After each task, mark completed checkboxes in this plan, then commit the task.
- This is a shell contract cleanup. Do not change source browser subjects, tabs, defaults, reconciliation, route state ownership, backend APIs, UI layout, or copy.
- Do not rename or reshape `sourceBrowserData`.
- Do not rename or reshape `groupBrowserData`.
- Do not rename or reshape `snapshotBrowserData`.
- Do not remove unrelated `source={currentSource}` props outside `SourceBrowserShell` invocations.

## Files

- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
  - Remove `source?: Source | null` from `Props`.
  - Remove `source = null` from `$props()` destructuring.
  - Replace the `explicitSubject ?? source` fallback with `const subject = $derived(explicitSubject);`.
  - Keep the `Source` type import if it is still used by `SourceBrowserData`.
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
  - Remove redundant `source={currentSource}` from the live single-source `SourceBrowserShell` call.
  - Keep explicit `subject={{ kind: "source", source: currentSource }}`.
  - Keep `sourceBrowserData`, `groupBrowserData`, and `snapshotBrowserData` unchanged.
- Modify: `src/lib/components/analysis/source-browser-shell.test.ts`
  - Assert the shell no longer exposes the legacy `source` prop or fallback.
- Modify: `src/lib/analysis-source-readers.test.ts`
  - Add call-block scoped assertions for all `SourceBrowserShell` invocations.
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md`
  - Mark implemented after final verification.
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md`
  - Track task checkboxes during execution.

---

### Task 0: Preflight Audit

**Files:**
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md`

- [x] **Step 1: Create the feature branch**

Run:

```bash
git switch -c source-browser-explicit-subject-contract
```

Expected: branch changes from `main` to `source-browser-explicit-subject-contract`.

- [x] **Step 2: Audit `SourceBrowserShell` call sites and unrelated `source` props**

Run:

```bash
rg -n "<SourceBrowserShell|source=\{currentSource\}|source=\{null\}" src
```

Expected current classification:

```text
SourceBrowserShell call sites:
- src/lib/components/analysis/report-source-surface.svelte run snapshot call: explicit subject={runSnapshotSubject}; no source prop.
- src/lib/components/analysis/report-source-surface.svelte live single-source call: explicit subject={{ kind: "source", source: currentSource }} plus redundant source={currentSource}.
- src/lib/components/analysis/report-source-surface.svelte live source-group call: explicit subject={{ kind: "source_group", group: currentGroup }}; no source prop.

Unrelated source props:
- src/lib/components/analysis/report-canvas.svelte source={currentSource}; not a SourceBrowserShell prop and not part of this cleanup.
```

Stop condition: if any production `SourceBrowserShell` call lacks `subject=`, stop and update the plan before deleting the fallback.

- [x] **Step 3: Audit current shell fallback and `Source` type usage**

Run:

```bash
rg -n "source\?: Source|source = null|explicitSubject \?\?|const subject =|sourceSyncDisabledReason|SourceBrowserData" src/lib/components/analysis/source-browser-shell.svelte
```

Expected current facts:

```text
- Props includes source?: Source | null.
- Props destructuring includes source = null.
- subject is derived from explicitSubject ?? (source ? { kind: "source" as const, source } : null).
- SourceBrowserData still uses Source in sourceSyncDisabledReason: (source: Source) => string | null.
```

Do not delete the `Source` import just because the top-level `source` prop is removed. `npm.cmd run check` is the authority for whether the import is still needed.

- [x] **Step 4: Record the actual audit result in this plan**

Recorded after running Steps 2 and 3:

```text
Actual audit result:
- SourceBrowserShell production calls without subject=: none found.
- SourceBrowserShell production calls with source={currentSource}: live single-source ReportSourceSurface call only.
- SourceBrowserShell production calls with source={null}: none found.
- Unrelated non-shell source={currentSource}: report-canvas.svelte only.
- Source import after cleanup: expected to remain because SourceBrowserData uses Source.
```

- [x] **Step 5: Commit preflight**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md
git commit -m "docs: record source browser explicit subject preflight"
```

Expected: commit succeeds with only this plan file staged.

---

### Task 1: Remove Shell Source Fallback

**Files:**
- Modify: `src/lib/components/analysis/source-browser-shell.test.ts`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md`

- [ ] **Step 1: Add shell contract assertions**

In `src/lib/components/analysis/source-browser-shell.test.ts`, add this test inside the `describe("source browser shell component contract", () => {` block after the existing `"uses the subject-aware source browser model and keeps data fetching outside the shell"` test:

```ts
  it("requires explicit browser subjects instead of source prop fallback", () => {
    const propsBlock = sourcePropsBlock();

    expect(propsBlock).toContain("subject?: SourceBrowserSubject | null");
    expect(propsBlock).not.toContain("source?: Source | null");
    expect(shellSource).toContain("subject: explicitSubject = null");
    expect(shellSource).toContain("const subject = $derived(explicitSubject);");
    expect(shellSource).not.toContain("explicitSubject ??");
    expect(shellSource).not.toContain('{ kind: "source" as const, source }');
    expect(shellSource).toContain('subject && subject.kind === "source" ? sourceBrowserData : null');
  });
```

- [ ] **Step 2: Add call-block scoped route assertions**

In `src/lib/analysis-source-readers.test.ts`, add this helper after the existing `sourceBrowserShellCall(marker: string)` helper:

```ts
function sourceBrowserShellCalls() {
  const calls: string[] = [];
  let searchIndex = 0;

  while (true) {
    const openIndex = reportSourceSurfaceSource.indexOf("<SourceBrowserShell", searchIndex);
    if (openIndex === -1) {
      break;
    }
    const closeIndex = reportSourceSurfaceSource.indexOf("/>", openIndex);
    expect(closeIndex).toBeGreaterThan(openIndex);
    calls.push(reportSourceSurfaceSource.slice(openIndex, closeIndex + 2));
    searchIndex = closeIndex + 2;
  }

  return calls;
}
```

Then add this test inside the `describe("analysis source readers", () => {` block after the existing `"routes live browsable sources and source groups through SourceBrowserShell"` test:

```ts
  it("passes explicit subjects to every SourceBrowserShell call without legacy source props", () => {
    const shellCalls = sourceBrowserShellCalls();
    const liveSourceShellCall = sourceBrowserShellCall('subject={{ kind: "source", source: currentSource }}');
    const liveGroupShellCall = sourceBrowserShellCall('subject={{ kind: "source_group", group: currentGroup }}');
    const snapshotShellCall = sourceBrowserShellCall("subject={runSnapshotSubject}");

    expect(shellCalls).toHaveLength(3);
    for (const shellCall of shellCalls) {
      expect(shellCall).toContain("subject=");
      expect(shellCall).not.toContain("source={");
    }

    expect(liveSourceShellCall).toContain('subject={{ kind: "source", source: currentSource }}');
    expect(liveSourceShellCall).toContain("sourceBrowserData={{");
    expect(liveSourceShellCall).not.toContain("source={currentSource}");

    expect(liveGroupShellCall).toContain('subject={{ kind: "source_group", group: currentGroup }}');
    expect(liveGroupShellCall).toContain("groupBrowserData={{");
    expect(liveGroupShellCall).not.toContain("source={null}");
    expect(liveGroupShellCall).not.toContain("sourceBrowserData={{");

    expect(snapshotShellCall).toContain("subject={runSnapshotSubject}");
    expect(snapshotShellCall).toContain("snapshotBrowserData={{");
    expect(snapshotShellCall).not.toContain("source={null}");
    expect(snapshotShellCall).not.toContain("sourceBrowserData={{");
  });
```

This test intentionally checks only `SourceBrowserShell` call blocks. It must not assert that unrelated files lack `source={currentSource}`.

- [ ] **Step 3: Run focused tests to verify the new assertions fail**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because `source-browser-shell.svelte` still contains `source?: Source | null`, still derives from `explicitSubject ?? source`, and `ReportSourceSurface` still passes `source={currentSource}` to the live single-source shell call.

- [ ] **Step 4: Remove the legacy source prop from `SourceBrowserShell`**

In `src/lib/components/analysis/source-browser-shell.svelte`, change the props block from:

```ts
  type Props = {
    subject?: SourceBrowserSubject | null;
    source?: Source | null;
    sourceBrowserData?: SourceBrowserData | null;
    groupBrowserData?: SourceGroupBrowserData | null;
    snapshotBrowserData?: SnapshotBrowserData | null;
    selectedTraceRef?: string | null;
    loadingItems?: boolean;
    formatTimestamp: (value: number | null) => string;
  };
```

to:

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

Change the `$props()` destructuring from:

```ts
  let {
    subject: explicitSubject = null,
    source = null,
    sourceBrowserData = null,
    groupBrowserData = null,
    snapshotBrowserData = null,
    selectedTraceRef = null,
    loadingItems = false,
    formatTimestamp,
  }: Props = $props();
```

to:

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
```

Change the subject derivation from:

```ts
  const subject = $derived(explicitSubject ?? (source ? { kind: "source" as const, source } : null));
```

to:

```ts
  const subject = $derived(explicitSubject);
```

Keep the `Source` type import if `SourceBrowserData` still references it.

- [ ] **Step 5: Remove the redundant live-source shell prop from `ReportSourceSurface`**

In `src/lib/components/analysis/report-source-surface.svelte`, change the live single-source shell invocation from:

```svelte
      <SourceBrowserShell
        subject={{ kind: "source", source: currentSource }}
        source={currentSource}
        sourceBrowserData={{
```

to:

```svelte
      <SourceBrowserShell
        subject={{ kind: "source", source: currentSource }}
        sourceBrowserData={{
```

Do not change `sourceBrowserData`, `groupBrowserData`, or `snapshotBrowserData`.

- [ ] **Step 6: Run focused tests to verify the contract passes**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [ ] **Step 7: Run Svelte type checking**

Run:

```bash
npm.cmd run check
```

Expected: PASS with `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 8: Run focused source browser regression tests**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/source-browser-model.test.ts
```

Expected: PASS.

- [ ] **Step 9: Commit implementation**

Run:

```bash
git add src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/report-source-surface.svelte docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md
git commit -m "refactor: require explicit source browser subjects"
```

Expected: commit succeeds.

---

### Task 2: Final Verification And Documentation

**Files:**
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md`

- [ ] **Step 1: Run a final scoped source-browser grep**

Run:

```bash
rg -n "<SourceBrowserShell|source=\{currentSource\}|source=\{null\}" src
```

Expected classification:

```text
- Every SourceBrowserShell call in src/lib/components/analysis/report-source-surface.svelte includes subject=.
- No SourceBrowserShell call includes source={currentSource}.
- No SourceBrowserShell call includes source={null}.
- Any remaining source={currentSource} match belongs to a non-SourceBrowserShell component such as report-canvas.svelte.
```

- [ ] **Step 2: Run full verification**

Run:

```bash
npm.cmd run verify
```

Expected: PASS for frontend tests, `svelte-check`, Rust tests, and frontend diff check.

- [ ] **Step 3: Mark the design spec implemented**

In `docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md`, change:

```md
> Status: approved design, pending implementation plan
```

to:

```md
> Status: implemented on 2026-05-30; pending merge
```

- [ ] **Step 4: Run docs diff check**

Run:

```bash
git diff -- docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md
```

Expected: diff shows the spec status update and checked-off Task 2 steps only.

- [ ] **Step 5: Commit final verification docs**

Run:

```bash
git add docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md
git commit -m "docs: mark explicit subject cleanup verified"
```

Expected: commit succeeds.

---

## Acceptance Checklist

- [ ] `SourceBrowserShell` has no top-level `source?: Source | null` prop.
- [ ] `SourceBrowserShell` does not derive `subject` from a fallback `source` prop.
- [ ] Every production `SourceBrowserShell` call passes explicit `subject=`.
- [ ] No production `SourceBrowserShell` call passes `source={currentSource}`.
- [ ] No production `SourceBrowserShell` call passes `source={null}`.
- [ ] `report-canvas.svelte` unrelated `source={currentSource}` remains untouched.
- [ ] `sourceBrowserData`, `groupBrowserData`, and `snapshotBrowserData` names and shapes are unchanged.
- [ ] Source browser tabs/defaults/reconciliation are unchanged.
- [ ] `SourceBrowserShell` still imports no `$lib/api/*` modules and calls no `invoke`.
- [ ] Focused frontend tests pass.
- [ ] `npm.cmd run check` passes.
- [ ] `npm.cmd run verify` passes.
