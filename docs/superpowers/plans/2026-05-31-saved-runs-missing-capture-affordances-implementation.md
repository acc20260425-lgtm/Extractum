# Saved Runs Missing Capture Affordances Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make unavailable saved-run source snapshots explicit in Runs, opened-run metadata, Source, Evidence, and Chat without changing backend report execution.

**Architecture:** Add one pure frontend snapshot affordance helper that owns state classification, status predicates, probe-state mapping, and display copy. Route-owned snapshot probing feeds a richer `SnapshotProbeState` into existing analysis surfaces, while components render helper decisions instead of open-coded missing-snapshot wording.

**Tech Stack:** Svelte 5, SvelteKit 2, TypeScript, Vitest pure helper tests, Vitest raw component contract tests, full project verification through `npm.cmd run verify`.

---

## Execution Protocol

- Start on branch `saved-runs-missing-capture-affordances`; do not create a worktree for this slice.
- Before implementation code, use `superpowers:test-driven-development`.
- Execute tasks in order. Each task should leave focused tests passing before committing.
- Mark completed checkboxes in this plan as work progresses.
- Do not add Runs cleanup filters, migration/repair actions, backend DTO fields, report execution changes, snapshot capture changes, or GUI smoke coverage in this slice.
- For Svelte edits, run the Svelte autofixer on touched components before final verification.

## Files

- Create: `src/lib/analysis-run-snapshot-affordance.ts`
  - Pure helper for snapshot affordance state, copy, status predicates, probe-state mapping, and sanitized error text.
- Create: `src/lib/analysis-run-snapshot-affordance.test.ts`
  - Focused Vitest matrix for the helper.
- Modify: `src/lib/analysis-report-canvas-state.ts`
  - Reuse `isActiveRunStatus` from the helper and keep the existing `RunSnapshotAvailability` API stable.
- Modify: `src/routes/analysis/+page.svelte`
  - Derive `runSnapshotProbeState` from `runSnapshotAvailability`, `loadingRunSnapshotMessages`, and `runSnapshotError`.
  - Pass `snapshotProbeState` to `ReportCanvas` and `RunCompanionTabs`.
  - Pass `snapshotProbeState` into route-owned Chat and Evidence decisions.
- Modify: `src/lib/components/analysis/report-canvas.svelte`
  - Accept `snapshotProbeState`.
  - Pass it to `ReportRunHeader` and `ReportSourceSurface`.
- Modify: `src/lib/components/analysis/run-companion-tabs.svelte`
  - Accept `snapshotProbeState`.
  - Pass it to `RunEvidenceTab`.
- Modify: `src/lib/analysis-run-companion-state.ts`
  - Route Chat and Evidence disabled copy through the helper.
- Modify: `src/lib/analysis-run-companion-state.test.ts`
  - Assert distinct Chat/Evidence reasons for legacy, capture-failed, not-captured, inconsistent, and verification-failed states.
- Modify: `src/lib/components/analysis/run-companion-runs-tab.svelte`
  - Render compact degraded snapshot badges for saved rows.
- Modify: `src/lib/components/analysis/report-run-header.svelte`
  - Use helper warning/details for opened-run snapshot status.
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
  - Use helper copy for unavailable run snapshot states and keep live-source copy explicit.
- Modify: `src/lib/components/analysis/run-evidence-tab.svelte`
  - Pass `snapshotProbeState` into Evidence action decisions.
- Modify: `src/lib/components/analysis/run-chat-tab.svelte`
  - No logic change expected; raw contract test should prove it continues to render `chatAvailability`.
- Modify: `src/lib/analysis-report-canvas.test.ts`
  - Raw component contracts for opened-run details and Source copy.
- Modify: `src/lib/analysis-run-companion-tabs.test.ts`
  - Raw component contracts for Runs degraded badges, Evidence helper usage, and Chat gating.
- Modify: `src/lib/analysis-report-canvas-route.test.ts`
  - Route contract for deriving and passing `snapshotProbeState`.
- Modify: `src/lib/analysis-run-companion-route.test.ts`
  - Route contract for passing `snapshotProbeState` into Chat/Evidence decisions.
- Modify: `docs/backlog.md`
  - Update Saved Runs Discoverability acceptance text so it describes missing/capture-failed affordances instead of already-shipped narrowing.
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md`
  - Track completed checkboxes during execution.

---

### Task 1: Snapshot Affordance Helper

**Files:**
- Create: `src/lib/analysis-run-snapshot-affordance.test.ts`
- Create: `src/lib/analysis-run-snapshot-affordance.ts`
- Modify: `src/lib/analysis-report-canvas-state.ts`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md`

- [x] **Step 1: Write the failing helper tests**

Create `src/lib/analysis-run-snapshot-affordance.test.ts` with:

```ts
import { describe, expect, it } from "vitest";
import {
  isActiveRunStatus,
  isTerminalRunStatus,
  snapshotAffordanceForRun,
  snapshotProbeStateFromAvailability,
  type SnapshotAffordanceInput,
} from "./analysis-run-snapshot-affordance";

function input(overrides: Partial<SnapshotAffordanceInput> = {}): SnapshotAffordanceInput {
  return {
    snapshotState: "captured",
    snapshotCapturedAt: "2026-05-18T10:00:00Z",
    snapshotError: null,
    probeState: "available",
    runStatus: "completed",
    surface: "run-details",
    ...overrides,
  };
}

describe("analysis run snapshot affordance", () => {
  it("marks captured available snapshots as available without degraded copy", () => {
    expect(snapshotAffordanceForRun(input())).toMatchObject({
      state: "available",
      severity: "none",
      compactLabel: null,
      badgeVariant: null,
      headerWarning: null,
      detailTitle: "Snapshot available",
      disabledReason: null,
      sanitizedError: null,
    });
  });

  it("distinguishes legacy missing snapshots", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: "missing_legacy",
      snapshotCapturedAt: null,
      probeState: "unavailable",
      surface: "runs-row",
    }))).toMatchObject({
      state: "legacy_missing",
      severity: "warning",
      compactLabel: "Legacy snapshot missing",
      badgeVariant: "warning",
      headerWarning: "Saved report is readable, but this legacy run has no saved source snapshot.",
      disabledReason: "Exact source resolution is unavailable because this legacy run has no saved source snapshot.",
    });
  });

  it("distinguishes capture failures with sanitized backend errors", () => {
    const affordance = snapshotAffordanceForRun(input({
      snapshotState: "capture_failed",
      snapshotCapturedAt: null,
      snapshotError: "  provider stack trace\nline 2  ",
      probeState: "unavailable",
      surface: "run-details",
    }));

    expect(affordance).toMatchObject({
      state: "capture_failed_with_error",
      severity: "error",
      compactLabel: "Snapshot capture failed",
      badgeVariant: "danger",
      detailTitle: "Snapshot capture failed",
      sanitizedError: "provider stack trace line 2",
    });
  });

  it("uses softer copy when capture_failed has no error and the run failed or was cancelled", () => {
    for (const status of ["failed", "cancelled"]) {
      expect(snapshotAffordanceForRun(input({
        snapshotState: "capture_failed",
        snapshotCapturedAt: null,
        snapshotError: null,
        probeState: "unavailable",
        runStatus: status,
      }))).toMatchObject({
        state: "not_captured_before_terminal",
        compactLabel: "Snapshot not captured",
        detailTitle: "Snapshot was not captured before the run ended",
      });
    }
  });

  it("keeps capture_failed without error distinct when the terminal cause is unknown", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: "capture_failed",
      snapshotCapturedAt: null,
      snapshotError: null,
      probeState: "unavailable",
      runStatus: "completed",
    }))).toMatchObject({
      state: "capture_failed_without_error_unknown",
      compactLabel: "Snapshot not captured",
      detailTitle: "Saved snapshot is unavailable",
    });
  });

  it("maps failed and cancelled null snapshot states with unavailable rows to not captured before terminal", () => {
    for (const status of ["failed", "cancelled"]) {
      expect(snapshotAffordanceForRun(input({
        snapshotState: null,
        snapshotCapturedAt: null,
        probeState: "unavailable",
        runStatus: status,
      }))).toMatchObject({
        state: "not_captured_before_terminal",
        detailTitle: "Snapshot was not captured before the run ended",
      });
    }
  });

  it("marks captured snapshots with unavailable rows as inconsistent", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: "captured",
      probeState: "unavailable",
    }))).toMatchObject({
      state: "inconsistent",
      compactLabel: "Snapshot rows unavailable",
      detailTitle: "Snapshot rows are unavailable",
      disabledReason: "Exact source resolution is unavailable because the run is marked captured but saved snapshot rows are unavailable.",
    });
  });

  it("marks captured snapshots with probe errors as verification failures", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: "captured",
      probeState: "error",
    }))).toMatchObject({
      state: "verification_failed",
      compactLabel: "Snapshot check failed",
      detailTitle: "Saved snapshot could not be verified",
      disabledReason: "Exact source resolution is unavailable because Extractum could not verify the saved snapshot rows.",
    });
  });

  it("handles the null snapshot state matrix", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      snapshotCapturedAt: null,
      probeState: "unknown",
      runStatus: "running",
    })).state).toBe("pending");

    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      snapshotCapturedAt: null,
      probeState: "available",
      runStatus: "completed",
    })).state).toBe("available");

    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      snapshotCapturedAt: null,
      probeState: "unavailable",
      runStatus: "completed",
    })).toMatchObject({
      state: "unknown",
      detailTitle: "Saved snapshot is unavailable",
    });

    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      snapshotCapturedAt: null,
      probeState: "error",
      runStatus: "completed",
    })).state).toBe("verification_failed");
  });

  it("classifies active, terminal, checking, and loading states", () => {
    expect(isActiveRunStatus("queued")).toBe(true);
    expect(isActiveRunStatus("running")).toBe(true);
    expect(isActiveRunStatus("completed")).toBe(false);

    expect(isTerminalRunStatus("completed")).toBe(true);
    expect(isTerminalRunStatus("failed")).toBe(true);
    expect(isTerminalRunStatus("cancelled")).toBe(true);
    expect(isTerminalRunStatus("running")).toBe(false);

    expect(snapshotAffordanceForRun(input({
      probeState: "loading",
      runStatus: "running",
    })).state).toBe("checking");
    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      probeState: "unknown",
      runStatus: "queued",
    })).state).toBe("pending");
  });

  it("derives probe state from route snapshot availability, loading, and error signals", () => {
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "available",
      loadingRunSnapshotMessages: false,
      runSnapshotError: "",
    })).toBe("available");
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "unavailable",
      loadingRunSnapshotMessages: false,
      runSnapshotError: "load failed",
    })).toBe("error");
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "unknown",
      loadingRunSnapshotMessages: true,
      runSnapshotError: "",
    })).toBe("loading");
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "capturing",
      loadingRunSnapshotMessages: false,
      runSnapshotError: "",
    })).toBe("unknown");
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "unknown",
      loadingRunSnapshotMessages: false,
      runSnapshotError: "",
    })).toBe("unknown");
  });
});
```

- [x] **Step 2: Run the helper test and verify it fails for missing module**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-snapshot-affordance.test.ts
```

Expected: FAIL because `src/lib/analysis-run-snapshot-affordance.ts` does not exist.

- [x] **Step 3: Implement the helper**

Create `src/lib/analysis-run-snapshot-affordance.ts` with:

```ts
import type { AnalysisSnapshotState } from "$lib/types/analysis";

export type SnapshotAffordanceSurface =
  | "runs-row"
  | "opened-header"
  | "run-details"
  | "source-tab"
  | "evidence-tab"
  | "chat-tab";

export type SnapshotAffordanceState =
  | "available"
  | "legacy_missing"
  | "capture_failed_with_error"
  | "not_captured_before_terminal"
  | "capture_failed_without_error_unknown"
  | "inconsistent"
  | "verification_failed"
  | "checking"
  | "pending"
  | "unknown";

export type SnapshotAffordanceSeverity =
  | "none"
  | "info"
  | "warning"
  | "error";

export type SnapshotProbeState =
  | "available"
  | "unavailable"
  | "error"
  | "loading"
  | "unknown";

export type SnapshotBadgeVariant = "neutral" | "info" | "warning" | "danger";
export type SnapshotAvailabilitySignal = "unknown" | "capturing" | "available" | "unavailable";

export interface SnapshotAffordanceInput {
  snapshotState: AnalysisSnapshotState | null;
  snapshotCapturedAt: string | null;
  snapshotError: string | null;
  probeState: SnapshotProbeState;
  runStatus: "queued" | "running" | "completed" | "failed" | "cancelled" | string;
  surface: SnapshotAffordanceSurface;
}

export interface SnapshotAffordance {
  state: SnapshotAffordanceState;
  severity: SnapshotAffordanceSeverity;
  compactLabel: string | null;
  badgeVariant: SnapshotBadgeVariant | null;
  headerWarning: string | null;
  detailTitle: string | null;
  detailDescription: string | null;
  disabledReason: string | null;
  sanitizedError: string | null;
}

export interface SnapshotProbeStateInput {
  snapshotAvailability: SnapshotAvailabilitySignal;
  loadingRunSnapshotMessages: boolean;
  runSnapshotError: string;
}

const AVAILABLE: SnapshotAffordance = {
  state: "available",
  severity: "none",
  compactLabel: null,
  badgeVariant: null,
  headerWarning: null,
  detailTitle: "Snapshot available",
  detailDescription: "Frozen source material captured for this run is available.",
  disabledReason: null,
  sanitizedError: null,
};

export function isActiveRunStatus(status: string): boolean {
  return status === "queued" || status === "running";
}

export function isTerminalRunStatus(status: string): boolean {
  return status === "completed" || status === "failed" || status === "cancelled";
}

export function snapshotProbeStateFromAvailability({
  snapshotAvailability,
  loadingRunSnapshotMessages,
  runSnapshotError,
}: SnapshotProbeStateInput): SnapshotProbeState {
  if (snapshotAvailability === "available") return "available";
  if (runSnapshotError.trim()) return "error";
  if (loadingRunSnapshotMessages) return "loading";
  // Existing RunSnapshotAvailability "capturing" means an active run has no
  // snapshot rows yet; keep that pending instead of treating it as degraded.
  if (snapshotAvailability === "capturing") return "unknown";
  if (snapshotAvailability === "unavailable") return "unavailable";
  return "unknown";
}

export function snapshotAffordanceForRun(input: SnapshotAffordanceInput): SnapshotAffordance {
  const sanitizedError = sanitizeSnapshotError(input.snapshotError);

  if (input.probeState === "available") return AVAILABLE;
  if (input.snapshotState === "missing_legacy") return legacyMissingAffordance();
  if (input.snapshotState === "capture_failed" && sanitizedError) return captureFailedAffordance(sanitizedError);
  if (input.snapshotState === "capture_failed") {
    return input.runStatus === "failed" || input.runStatus === "cancelled"
      ? notCapturedBeforeTerminalAffordance()
      : captureFailedWithoutErrorAffordance();
  }
  if (input.snapshotState === "captured" && input.probeState === "unavailable") {
    return inconsistentAffordance();
  }
  if (input.probeState === "error") return verificationFailedAffordance(sanitizedError);
  if (input.probeState === "loading") return checkingAffordance();
  if (isActiveRunStatus(input.runStatus)) return pendingAffordance();
  if (
    input.snapshotState === null
    && input.probeState === "unavailable"
    && (input.runStatus === "failed" || input.runStatus === "cancelled")
  ) {
    return notCapturedBeforeTerminalAffordance();
  }
  if (input.snapshotState === null && input.probeState === "unavailable") {
    return unknownUnavailableAffordance();
  }
  return checkingAffordance();
}

function sanitizeSnapshotError(value: string | null): string | null {
  const sanitized = (value ?? "").replace(/\s+/g, " ").trim();
  return sanitized ? sanitized : null;
}

function baseAffordance({
  state,
  severity,
  compactLabel,
  badgeVariant,
  headerWarning,
  detailTitle,
  detailDescription,
  disabledReason,
  sanitizedError = null,
}: SnapshotAffordance): SnapshotAffordance {
  return {
    state,
    severity,
    compactLabel,
    badgeVariant,
    headerWarning,
    detailTitle,
    detailDescription,
    disabledReason,
    sanitizedError,
  };
}

function legacyMissingAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "legacy_missing",
    severity: "warning",
    compactLabel: "Legacy snapshot missing",
    badgeVariant: "warning",
    headerWarning: "Saved report is readable, but this legacy run has no saved source snapshot.",
    detailTitle: "Legacy run has no saved snapshot",
    detailDescription: "Older saved runs may not include frozen source rows, so exact source browsing, evidence source resolution, and follow-up chat stay unavailable.",
    disabledReason: "Exact source resolution is unavailable because this legacy run has no saved source snapshot.",
    sanitizedError: null,
  });
}

function captureFailedAffordance(sanitizedError: string): SnapshotAffordance {
  return baseAffordance({
    state: "capture_failed_with_error",
    severity: "error",
    compactLabel: "Snapshot capture failed",
    badgeVariant: "danger",
    headerWarning: "Saved report is readable, but Extractum could not save the frozen source context for this run.",
    detailTitle: "Snapshot capture failed",
    detailDescription: "Extractum could not save the frozen source context for this run. Exact source browsing, evidence source resolution, and follow-up chat stay unavailable.",
    disabledReason: "Exact source resolution is unavailable because snapshot capture failed for this run.",
    sanitizedError,
  });
}

function notCapturedBeforeTerminalAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "not_captured_before_terminal",
    severity: "warning",
    compactLabel: "Snapshot not captured",
    badgeVariant: "warning",
    headerWarning: "Saved report is readable, but the run ended before a frozen source snapshot was saved.",
    detailTitle: "Snapshot was not captured before the run ended",
    detailDescription: "The saved report remains readable, but there is no frozen source corpus for exact source browsing, evidence source resolution, or follow-up chat.",
    disabledReason: "Exact source resolution is unavailable because the run ended before a frozen source snapshot was saved.",
    sanitizedError: null,
  });
}

function captureFailedWithoutErrorAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "capture_failed_without_error_unknown",
    severity: "warning",
    compactLabel: "Snapshot not captured",
    badgeVariant: "warning",
    headerWarning: "Saved report is readable, but saved source context is unavailable for this run.",
    detailTitle: "Saved snapshot is unavailable",
    detailDescription: "Extractum did not record a snapshot capture error, but saved snapshot rows are unavailable for this run.",
    disabledReason: "Exact source resolution is unavailable because saved snapshot rows are unavailable for this run.",
    sanitizedError: null,
  });
}

function inconsistentAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "inconsistent",
    severity: "error",
    compactLabel: "Snapshot rows unavailable",
    badgeVariant: "danger",
    headerWarning: "Saved report is readable, but the stored snapshot marker is inconsistent with saved rows.",
    detailTitle: "Snapshot rows are unavailable",
    detailDescription: "This run is marked as captured, but Extractum could not load saved snapshot rows for it.",
    disabledReason: "Exact source resolution is unavailable because the run is marked captured but saved snapshot rows are unavailable.",
    sanitizedError: null,
  });
}

function verificationFailedAffordance(sanitizedError: string | null): SnapshotAffordance {
  return baseAffordance({
    state: "verification_failed",
    severity: "error",
    compactLabel: "Snapshot check failed",
    badgeVariant: "danger",
    headerWarning: "Saved report is readable, but Extractum could not verify the saved source snapshot.",
    detailTitle: "Saved snapshot could not be verified",
    detailDescription: "Extractum could not verify saved snapshot rows for this run. Exact source browsing, evidence source resolution, and follow-up chat stay unavailable until verification succeeds.",
    disabledReason: "Exact source resolution is unavailable because Extractum could not verify the saved snapshot rows.",
    sanitizedError,
  });
}

function unknownUnavailableAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "unknown",
    severity: "warning",
    compactLabel: "Snapshot unavailable",
    badgeVariant: "warning",
    headerWarning: "Saved report is readable, but saved source context is unavailable for this run.",
    detailTitle: "Saved snapshot is unavailable",
    detailDescription: "Saved snapshot rows are unavailable for this run, and the run does not identify the missing context as a legacy snapshot.",
    disabledReason: "Exact source resolution is unavailable because saved snapshot rows are unavailable for this run.",
    sanitizedError: null,
  });
}

function checkingAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "checking",
    severity: "info",
    compactLabel: null,
    badgeVariant: null,
    headerWarning: null,
    detailTitle: "Checking saved snapshot",
    detailDescription: "Extractum is checking whether frozen source material is available for this run.",
    disabledReason: "Exact source resolution is unavailable until the saved snapshot check finishes.",
    sanitizedError: null,
  });
}

function pendingAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "pending",
    severity: "info",
    compactLabel: null,
    badgeVariant: null,
    headerWarning: null,
    detailTitle: "Snapshot pending",
    detailDescription: "Snapshot capture is still pending for this active run.",
    disabledReason: "Exact source resolution is unavailable until the run snapshot is captured.",
    sanitizedError: null,
  });
}
```

- [x] **Step 4: Reuse the helper status predicate in canvas state**

In `src/lib/analysis-report-canvas-state.ts`, add the import and re-export near the top:

```ts
import { isActiveRunStatus } from "$lib/analysis-run-snapshot-affordance";
export { isActiveRunStatus } from "$lib/analysis-run-snapshot-affordance";
```

Then remove the local function:

```ts
export function isActiveRunStatus(status: string) {
  return status === "queued" || status === "running";
}
```

- [x] **Step 5: Run focused state tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-snapshot-affordance.test.ts src/lib/analysis-report-canvas-state.test.ts
```

Expected: PASS for both files.

- [x] **Step 6: Commit helper**

Run:

```powershell
git add src/lib/analysis-run-snapshot-affordance.ts src/lib/analysis-run-snapshot-affordance.test.ts src/lib/analysis-report-canvas-state.ts docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md
git commit -m "feat: add saved run snapshot affordance helper"
```

Expected: commit succeeds on `saved-runs-missing-capture-affordances`.

---

### Task 2: Route Probe State Wiring

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/run-companion-tabs.svelte`
- Modify: `src/lib/analysis-report-canvas-route.test.ts`
- Modify: `src/lib/analysis-run-companion-route.test.ts`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md`

- [x] **Step 1: Write failing route contract tests**

In `src/lib/analysis-report-canvas-route.test.ts`, add this test:

```ts
  it("derives snapshot probe state and passes it into the report canvas", () => {
    expect(analysisPageSource).toContain("snapshotProbeStateFromAvailability");
    expect(analysisPageSource).toContain("const runSnapshotProbeState = $derived(");
    expect(analysisPageSource).toContain("snapshotAvailability: runSnapshotAvailability");
    expect(analysisPageSource).toContain("{loadingRunSnapshotMessages}");
    expect(analysisPageSource).toContain("{runSnapshotError}");
    expect(analysisPageSource).toContain("snapshotProbeState={runSnapshotProbeState}");
  });
```

In `src/lib/analysis-run-companion-route.test.ts`, add assertions to the existing Chat and Evidence tests:

```ts
    expect(analysisPageSource).toContain("snapshotProbeState: runSnapshotProbeState");
    expect(analysisPageSource).toContain("snapshotProbeState={runSnapshotProbeState}");
```

- [x] **Step 2: Run route contract tests and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas-route.test.ts src/lib/analysis-run-companion-route.test.ts
```

Expected: FAIL because the route has not imported `snapshotProbeStateFromAvailability` or passed `snapshotProbeState`.

- [x] **Step 3: Import and derive route probe state**

In `src/routes/analysis/+page.svelte`, add:

```ts
  import { snapshotProbeStateFromAvailability } from "$lib/analysis-run-snapshot-affordance";
```

Near the existing `runSnapshotAvailability` derived usage, add:

```ts
  const runSnapshotProbeState = $derived(snapshotProbeStateFromAvailability({
    snapshotAvailability: runSnapshotAvailability,
    loadingRunSnapshotMessages,
    runSnapshotError,
  }));
```

Where the route calls `chatAvailabilityForRun`, change the argument to include:

```ts
    snapshotProbeState: runSnapshotProbeState,
```

Where the route calls `evidenceSourceActionDecision`, change the argument to include:

```ts
    snapshotProbeState: runSnapshotProbeState,
```

In the `<ReportCanvas>` call, add:

```svelte
    snapshotProbeState={runSnapshotProbeState}
```

In the `<RunCompanionTabs>` call, add:

```svelte
      snapshotProbeState={runSnapshotProbeState}
```

- [x] **Step 4: Accept and pass the prop through `ReportCanvas`**

In `src/lib/components/analysis/report-canvas.svelte`, import the type:

```ts
  import type { SnapshotProbeState } from "$lib/analysis-run-snapshot-affordance";
```

Add `snapshotProbeState` to destructuring after `runSnapshotAvailability`, and add this prop type:

```ts
    snapshotProbeState: SnapshotProbeState;
```

Pass it to `ReportRunHeader`:

```svelte
        {snapshotProbeState}
```

Pass it to `ReportSourceSurface`:

```svelte
      {snapshotProbeState}
```

- [x] **Step 5: Accept and pass the prop through `RunCompanionTabs`**

In `src/lib/components/analysis/run-companion-tabs.svelte`, import the type:

```ts
  import type { SnapshotProbeState } from "$lib/analysis-run-snapshot-affordance";
```

Add `snapshotProbeState` to destructuring after `snapshotAvailability`, and add this prop type:

```ts
    snapshotProbeState: SnapshotProbeState;
```

Pass it to `RunEvidenceTab`:

```svelte
        {snapshotProbeState}
```

- [x] **Step 6: Run route contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas-route.test.ts src/lib/analysis-run-companion-route.test.ts
```

Expected: PASS.

- [x] **Step 7: Commit route probe wiring**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/run-companion-tabs.svelte src/lib/analysis-report-canvas-route.test.ts src/lib/analysis-run-companion-route.test.ts docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md
git commit -m "feat: pass saved run snapshot probe state"
```

Expected: commit succeeds.

---

### Task 3: Companion Chat And Evidence Decisions

**Files:**
- Modify: `src/lib/analysis-run-companion-state.test.ts`
- Modify: `src/lib/analysis-run-companion-state.ts`
- Modify: `src/lib/components/analysis/run-evidence-tab.svelte`
- Modify: `src/lib/analysis-run-companion-tabs.test.ts`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md`

- [x] **Step 1: Write failing companion state tests**

In `src/lib/analysis-run-companion-state.test.ts`, update the imports:

```ts
import type { SnapshotProbeState } from "./analysis-run-snapshot-affordance";
```

Add `snapshotProbeState` to existing `chatAvailabilityForRun` and `evidenceSourceActionDecision` calls. For current tests, use `snapshotProbeState: snapshotAvailability === "available" ? "available" : "unknown"` except unavailable cases, where use `"unavailable"`. In the existing `"maps completed run chat availability"` table, change the unavailable expected reason from `"missing_snapshot"` to `"inconsistent"` because the default test run is marked `snapshot_state: "captured"` while the probe reports unavailable rows.

Add these tests:

```ts
  it.each([
    [
      "missing_legacy",
      "unavailable",
      "legacy_missing",
      "Saved context unavailable",
      "legacy run has no saved source snapshot",
    ],
    [
      "capture_failed",
      "unavailable",
      "capture_failed_with_error",
      "Saved context unavailable",
      "snapshot capture failed",
    ],
    [
      null,
      "unavailable",
      "not_captured_before_terminal",
      "Saved context unavailable",
      "run ended before a frozen source snapshot was saved",
    ],
    [
      "captured",
      "unavailable",
      "inconsistent",
      "Saved context unavailable",
      "marked captured but saved snapshot rows are unavailable",
    ],
    [
      "captured",
      "error",
      "verification_failed",
      "Saved context unavailable",
      "could not verify the saved snapshot rows",
    ],
  ] satisfies Array<[AnalysisRunDetail["snapshot_state"], SnapshotProbeState, string, string, string]>)(
    "uses snapshot affordance copy for completed-run chat when state is %s and probe is %s",
    (snapshotState, snapshotProbeState, reason, title, descriptionFragment) => {
      const snapshotError = snapshotState === "capture_failed" ? "sqlite write failed" : null;
      const availability = snapshotProbeState === "available" ? "available" : "unavailable";

      expect(chatAvailabilityForRun({
        currentRun: run({
          status: snapshotState === null ? "failed" : "completed",
          snapshot_state: snapshotState,
          snapshot_error: snapshotError,
        }),
        snapshotAvailability: availability,
        snapshotProbeState,
      })).toMatchObject({
        enabled: false,
        reason,
        title,
      });

      expect(chatAvailabilityForRun({
        currentRun: run({
          status: snapshotState === null ? "failed" : "completed",
          snapshot_state: snapshotState,
          snapshot_error: snapshotError,
        }),
        snapshotAvailability: availability,
        snapshotProbeState,
      }).description).toContain(descriptionFragment);
    },
  );

  it.each([
    [
      "missing_legacy",
      "unavailable",
      "legacy run has no saved source snapshot",
    ],
    [
      "capture_failed",
      "unavailable",
      "snapshot capture failed",
    ],
    [
      null,
      "unavailable",
      "run ended before a frozen source snapshot was saved",
    ],
    [
      "captured",
      "unavailable",
      "marked captured but saved snapshot rows are unavailable",
    ],
    [
      "captured",
      "error",
      "could not verify the saved snapshot rows",
    ],
  ] satisfies Array<[AnalysisRunDetail["snapshot_state"], SnapshotProbeState, string]>)(
    "uses snapshot affordance copy for completed-run evidence when state is %s and probe is %s",
    (snapshotState, snapshotProbeState, reasonFragment) => {
      const snapshotError = snapshotState === "capture_failed" ? "capture failed" : null;

      expect(evidenceSourceActionDecision({
        currentRun: run({
          status: snapshotState === null ? "failed" : "completed",
          snapshot_state: snapshotState,
          snapshot_error: snapshotError,
        }),
        selectedTrace: traceRef(),
        snapshotAvailability: "unavailable",
        snapshotProbeState,
      })).toMatchObject({ kind: "unavailable" });
      expect(evidenceSourceActionDecision({
        currentRun: run({
          status: snapshotState === null ? "failed" : "completed",
          snapshot_state: snapshotState,
          snapshot_error: snapshotError,
        }),
        selectedTrace: traceRef(),
        snapshotAvailability: "unavailable",
        snapshotProbeState,
      })).toMatchObject({
        reason: expect.stringContaining(reasonFragment),
      });
    },
  );

  it.each(["completed", "failed", "cancelled"])(
    "lets available saved snapshots win for %s evidence actions",
    (status) => {
      expect(evidenceSourceActionDecision({
        currentRun: run({ status, snapshot_state: "captured" }),
        selectedTrace: traceRef(),
        snapshotAvailability: "available",
        snapshotProbeState: "available",
      })).toEqual({
        kind: "run_snapshot",
        canvasMode: "source",
        sourceViewBasis: "run_snapshot",
        highlightedRef: "s7-i11",
      });
    },
  );
```

- [x] **Step 2: Run companion tests and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-companion-state.test.ts
```

Expected: FAIL because `snapshotProbeState` is not in the function contracts and distinct reasons are not implemented.

- [x] **Step 3: Extend companion decision contracts**

In `src/lib/analysis-run-companion-state.ts`, update imports:

```ts
  import {
    isActiveRunStatus,
    isTerminalRunStatus,
    snapshotAffordanceForRun,
    type SnapshotAffordanceState,
    type SnapshotProbeState,
  } from "$lib/analysis-run-snapshot-affordance";
```

Extend `ChatAvailabilityReason`:

```ts
  | "legacy_missing"
  | "capture_failed_with_error"
  | "not_captured_before_terminal"
  | "capture_failed_without_error_unknown"
  | "inconsistent"
  | "verification_failed"
  | "unknown_snapshot";
```

Add this mapper near the other small state helpers:

```ts
function chatReasonForSnapshotAffordance(state: SnapshotAffordanceState): ChatAvailabilityReason {
  if (
    state === "legacy_missing"
    || state === "capture_failed_with_error"
    || state === "not_captured_before_terminal"
    || state === "capture_failed_without_error_unknown"
    || state === "inconsistent"
    || state === "verification_failed"
  ) {
    return state;
  }
  if (state === "unknown") return "unknown_snapshot";
  return "missing_snapshot";
}
```

Change `chatAvailabilityForRun` signature to:

```ts
export function chatAvailabilityForRun({
  currentRun,
  snapshotAvailability,
  snapshotProbeState,
}: {
  currentRun: AnalysisRunDetail | null;
  snapshotAvailability: RunSnapshotAvailability;
  snapshotProbeState: SnapshotProbeState;
}): ChatAvailability {
```

Replace the active run check with:

```ts
  if (isActiveRunStatus(currentRun.status)) {
```

Move the existing failed/cancelled terminal-run branch below the saved-context unavailable branch. Then replace the `snapshotAvailability === "unavailable"` branch with:

```ts
  if (snapshotAvailability === "unavailable" || snapshotProbeState === "unavailable" || snapshotProbeState === "error") {
    const affordance = snapshotAffordanceForRun({
      snapshotState: currentRun.snapshot_state,
      snapshotCapturedAt: currentRun.snapshot_captured_at,
      snapshotError: currentRun.snapshot_error,
      probeState: snapshotProbeState,
      runStatus: currentRun.status,
      surface: "chat-tab",
    });

    return {
      enabled: false,
      reason: chatReasonForSnapshotAffordance(affordance.state),
      title: "Saved context unavailable",
      description: affordance.detailDescription ?? "Saved snapshot context is unavailable for this run.",
    };
  }
```

After that unavailable branch, keep the failed/cancelled terminal-run guard:

```ts
  if (currentRun.status === "failed" || currentRun.status === "cancelled") {
    return {
      enabled: false,
      reason: "terminal_run",
      title: "Chat is disabled for this run",
      description: "For this MVP, follow-up chat is available only for completed reports.",
    };
  }
```

This preserves the existing terminal-run behavior when saved context is not the reason Chat is unavailable, while allowing failed/cancelled runs without snapshots to use `not_captured_before_terminal` copy.

Change `evidenceSourceActionDecision` signature to:

```ts
export function evidenceSourceActionDecision({
  currentRun,
  selectedTrace,
  snapshotAvailability,
  snapshotProbeState,
}: {
  currentRun: AnalysisRunDetail | null;
  selectedTrace: AnalysisTraceRef | null;
  snapshotAvailability: RunSnapshotAvailability;
  snapshotProbeState: SnapshotProbeState;
}): EvidenceSourceActionDecision {
```

Replace the completed-run unavailable branch with:

```ts
  const hasUsableSnapshot = snapshotAvailability === "available" && snapshotProbeState === "available";

  if (!hasUsableSnapshot && (isTerminalRunStatus(currentRun.status) || snapshotProbeState === "error")) {
    const affordance = snapshotAffordanceForRun({
      snapshotState: currentRun.snapshot_state,
      snapshotCapturedAt: currentRun.snapshot_captured_at,
      snapshotError: currentRun.snapshot_error,
      probeState: snapshotProbeState,
      runStatus: currentRun.status,
      surface: "evidence-tab",
    });

    return {
      kind: "unavailable",
      reason: affordance.disabledReason ?? "Exact source resolution is unavailable because saved snapshot rows are unavailable for this run.",
    };
  }
```

Keep the existing available-snapshot branch above this guard. Keep the live-source branch only for active or non-terminal runs without available saved snapshots. Failed and cancelled runs are terminal, so they must not fall through to the live-source bridge.

- [x] **Step 4: Pass `snapshotProbeState` from `RunEvidenceTab`**

In `src/lib/components/analysis/run-evidence-tab.svelte`, import the type:

```ts
  import type { SnapshotProbeState } from "$lib/analysis-run-snapshot-affordance";
```

Add `snapshotProbeState` to props after `snapshotAvailability`, add this prop type:

```ts
    snapshotProbeState: SnapshotProbeState;
```

Pass it into the derived decision:

```ts
    snapshotProbeState,
```

- [x] **Step 5: Update raw companion component contract**

In `src/lib/analysis-run-companion-tabs.test.ts`, update the Evidence test assertions:

```ts
    expect(evidenceTabSource).toContain("snapshotProbeState");
    expect(evidenceTabSource).toContain("evidenceSourceActionDecision");
    expect(evidenceTabSource).toContain("sourceDecision.reason");
```

Update the Chat test assertions:

```ts
    expect(chatTabSource).toContain("chatAvailability");
    expect(chatTabSource).toContain("{chatAvailability.title}");
    expect(chatTabSource).toContain("{chatAvailability.description}");
```

- [x] **Step 6: Run focused companion tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-companion-state.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-run-companion-route.test.ts
```

Expected: PASS.

- [x] **Step 7: Commit companion decisions**

Run:

```powershell
git add src/lib/analysis-run-companion-state.ts src/lib/analysis-run-companion-state.test.ts src/lib/components/analysis/run-evidence-tab.svelte src/lib/analysis-run-companion-tabs.test.ts docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md
git commit -m "feat: explain degraded saved run companion actions"
```

Expected: commit succeeds.

---

### Task 4: Runs Tab Degraded Snapshot Badges

**Files:**
- Modify: `src/lib/components/analysis/run-companion-runs-tab.svelte`
- Modify: `src/lib/analysis-run-companion-tabs.test.ts`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md`

- [x] **Step 1: Write failing raw component contract**

In the `"contains only analysis report runs in the Runs tab"` test in `src/lib/analysis-run-companion-tabs.test.ts`, add:

```ts
    expect(runsTabSource).toContain("snapshotAffordanceForRun");
    expect(runsTabSource).toContain("snapshotAffordanceForRow");
    expect(runsTabSource).toContain("snapshotAffordance?.compactLabel");
    expect(runsTabSource).toContain("snapshotAffordance.badgeVariant");
```

- [x] **Step 2: Run raw contract test and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-companion-tabs.test.ts
```

Expected: FAIL because the Runs tab does not import or render snapshot affordances.

- [x] **Step 3: Render saved-row snapshot affordance badges**

In `src/lib/components/analysis/run-companion-runs-tab.svelte`, add:

```ts
  import { snapshotAffordanceForRun } from "$lib/analysis-run-snapshot-affordance";
```

The existing `BadgeVariant` type in `src/lib/components/ui/types.ts` supports `"danger"`, so the helper's `badgeVariant` can be passed directly to `<Badge variant={snapshotAffordance.badgeVariant}>`.

Add this function after `inputValue`:

```ts
  function snapshotAffordanceForRow(entry: { kind: "active" | "saved"; run: AnalysisRunSummary }) {
    if (entry.kind !== "saved") return null;

    return snapshotAffordanceForRun({
      snapshotState: entry.run.snapshot_state,
      snapshotCapturedAt: entry.run.snapshot_captured_at,
      snapshotError: entry.run.snapshot_error,
      probeState: "unknown",
      runStatus: entry.run.status,
      surface: "runs-row",
    });
  }
```

Inside the `#each` block after `{@const run = entry.run}`, add:

```svelte
        {@const snapshotAffordance = snapshotAffordanceForRow(entry)}
```

Inside `.run-title`, after the existing `active/saved` badge, add:

```svelte
              {#if snapshotAffordance?.compactLabel && snapshotAffordance.badgeVariant}
                <Badge variant={snapshotAffordance.badgeVariant}>
                  {snapshotAffordance.compactLabel}
                </Badge>
              {/if}
```

Do not render `snapshotAffordance.sanitizedError` in the row.

- [x] **Step 4: Run raw contract test**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-companion-tabs.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit Runs tab badges**

Run:

```powershell
git add src/lib/components/analysis/run-companion-runs-tab.svelte src/lib/analysis-run-companion-tabs.test.ts docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md
git commit -m "feat: badge degraded saved run snapshots"
```

Expected: commit succeeds.

---

### Task 5: Opened Run Header Details

**Files:**
- Modify: `src/lib/components/analysis/report-run-header.svelte`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md`

- [x] **Step 1: Write failing header contract test**

In `"renders required opened-run header metadata"` in `src/lib/analysis-report-canvas.test.ts`, add:

```ts
    expect(reportRunHeaderSource).toContain("snapshotAffordanceForRun");
    expect(reportRunHeaderSource).toContain("snapshotProbeState");
    expect(reportRunHeaderSource).toContain("snapshotAffordance.headerWarning");
    expect(reportRunHeaderSource).toContain("Snapshot status");
    expect(reportRunHeaderSource).toContain("Snapshot captured");
    expect(reportRunHeaderSource).toContain("Snapshot note");
    expect(reportRunHeaderSource).toContain("Snapshot error");
    expect(reportRunHeaderSource).toContain("snapshotAffordance.sanitizedError");
```

- [x] **Step 2: Run canvas contract test and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts
```

Expected: FAIL because `ReportRunHeader` still uses `hasSnapshotWarning` and lacks snapshot detail metadata.

- [x] **Step 3: Use helper in `ReportRunHeader`**

In `src/lib/components/analysis/report-run-header.svelte`, add:

```ts
  import {
    snapshotAffordanceForRun,
    type SnapshotProbeState,
  } from "$lib/analysis-run-snapshot-affordance";
```

Add `snapshotProbeState` to props after `snapshotAvailability`, and add this prop type:

```ts
    snapshotProbeState: SnapshotProbeState;
```

Replace `hasSnapshotWarning` with:

```ts
  const snapshotAffordance = $derived(snapshotAffordanceForRun({
    snapshotState: currentRun.snapshot_state,
    snapshotCapturedAt: currentRun.snapshot_captured_at,
    snapshotError: currentRun.snapshot_error,
    probeState: snapshotProbeState,
    runStatus: currentRun.status,
    surface: "opened-header",
  }));
```

Replace the warning block with:

```svelte
  {#if snapshotAffordance.headerWarning}
    <p class="snapshot-warning">
      {snapshotAffordance.headerWarning}
    </p>
  {/if}
```

Inside `.run-meta-grid`, after `Source basis`, add:

```svelte
      <MetaCell label="Snapshot status">{snapshotAffordance.detailTitle ?? basisLabel}</MetaCell>
      <MetaCell label="Snapshot captured">{currentRun.snapshot_captured_at ?? "Not recorded"}</MetaCell>
      {#if snapshotAffordance.detailDescription}
        <MetaCell label="Snapshot note">{snapshotAffordance.detailDescription}</MetaCell>
      {/if}
      {#if snapshotAffordance.sanitizedError}
        <MetaCell label="Snapshot error">{snapshotAffordance.sanitizedError}</MetaCell>
      {/if}
```

Keep the existing `Source basis` MetaCell; it still describes the current view basis.

- [x] **Step 4: Run canvas contract test**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit header details**

Run:

```powershell
git add src/lib/components/analysis/report-run-header.svelte src/lib/analysis-report-canvas.test.ts docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md
git commit -m "feat: show saved run snapshot details"
```

Expected: commit succeeds.

---

### Task 6: Source Tab Degraded Snapshot Copy

**Files:**
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md`

- [ ] **Step 1: Write failing Source contracts**

In `"keeps snapshot and live source basis explicit"` in `src/lib/analysis-report-canvas.test.ts`, add:

```ts
    expect(reportSourceSurfaceSource).toContain("snapshotAffordanceForRun");
    expect(reportSourceSurfaceSource).toContain("snapshotProbeState");
    expect(reportSourceSurfaceSource).toContain("snapshotAffordance.detailDescription");
    expect(reportSourceSurfaceSource).toContain("snapshotAffordance.sanitizedError");
    expect(reportSourceSurfaceSource).toContain("This is live data, not the saved run snapshot.");
```

In `"keeps live source and run snapshot basis visible"` in `src/lib/analysis-source-readers.test.ts`, add:

```ts
    expect(reportSourceSurfaceSource).toContain("snapshotAffordance.detailTitle");
    expect(reportSourceSurfaceSource).toContain("snapshotAffordance.detailDescription");
    expect(reportSourceSurfaceSource).not.toContain("<StatusMessage tone=\"error\">{runSnapshotError}</StatusMessage>");
    expect(reportSourceSurfaceSource).toContain("canViewLiveSourceForSnapshot");
```

- [ ] **Step 2: Run Source contract tests and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because `ReportSourceSurface` still uses generic unavailable copy.

- [ ] **Step 3: Use helper in `ReportSourceSurface`**

In `src/lib/components/analysis/report-source-surface.svelte`, add:

```ts
  import {
    snapshotAffordanceForRun,
    type SnapshotProbeState,
  } from "$lib/analysis-run-snapshot-affordance";
```

Add `snapshotProbeState` to props after `snapshotAvailability`, and add this prop type:

```ts
    snapshotProbeState: SnapshotProbeState;
```

Add the derived helper after `sourceBasis`:

```ts
  const canViewLiveSourceForSnapshot = $derived(currentSource !== null || currentGroup !== null);
  const snapshotAffordance = $derived(currentRun
    ? snapshotAffordanceForRun({
        snapshotState: currentRun.snapshot_state,
        snapshotCapturedAt: currentRun.snapshot_captured_at,
        snapshotError: currentRun.snapshot_error,
        probeState: snapshotProbeState,
        runStatus: currentRun.status,
        surface: "source-tab",
      })
    : null);
```

In the unavailable run snapshot branch, change the `SourceReaderHeader` props to:

```svelte
        title={snapshotAffordance?.detailTitle ?? sourceBasisLabel(sourceBasis)}
        surfaceLabel={readerSurfaceLabel}
        subtitle={snapshotAffordance?.detailDescription ?? sourceBasisDescription(sourceBasis)}
```

Replace the unavailable/checking status messages with:

```svelte
      {#if snapshotAvailability === "capturing" || snapshotAffordance?.state === "pending"}
        <StatusMessage tone="muted">{snapshotAffordance?.detailDescription ?? "Snapshot pending. The frozen source corpus is not browsable yet."}</StatusMessage>
      {:else if snapshotAffordance?.state === "checking"}
        <StatusMessage tone="muted">{snapshotAffordance.detailDescription}</StatusMessage>
      {:else if snapshotAffordance}
        <StatusMessage>
          {snapshotAffordance.detailDescription}
        </StatusMessage>
        {#if snapshotAffordance.sanitizedError}
          <StatusMessage tone="error">{snapshotAffordance.sanitizedError}</StatusMessage>
        {/if}
        {#if canViewLiveSourceForSnapshot}
          <StatusMessage tone="muted">
            View live source opens current source data. This is live data, not the saved run snapshot.
          </StatusMessage>
        {/if}
      {:else}
        <StatusMessage tone="muted">Checking run snapshot availability...</StatusMessage>
      {/if}
```

Change the unavailable run snapshot branch header prop from `canViewLiveSource={true}` to:

```svelte
        canViewLiveSource={canViewLiveSourceForSnapshot}
```

Do not render `runSnapshotError` directly in this branch. Probe errors should show the generic `verification_failed` copy from `snapshotAffordanceForRun`; only `snapshotAffordance.sanitizedError`, derived from backend `snapshot_error`, is displayed as error detail.

- [ ] **Step 4: Run Source contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit Source copy**

Run:

```powershell
git add src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers.test.ts docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md
git commit -m "feat: explain unavailable saved run source snapshots"
```

Expected: commit succeeds.

---

### Task 7: Backlog Acceptance And Verification

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md`

- [ ] **Step 1: Update Saved Runs backlog acceptance**

In `docs/backlog.md`, replace the Saved Runs Discoverability acceptance item:

```markdown
- Large saved-run histories can be narrowed quickly without reconstructing the original run context.
```

with:

```markdown
- Missing legacy and capture-failed saved runs show explicit affordances in the
  Runs list, opened-run details, Source, Evidence, and Chat surfaces.
- Live source browsing remains an explicit action and is not presented as the
  saved run corpus when a run snapshot is unavailable.
```

- [ ] **Step 2: Run focused frontend tests for the whole slice**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-snapshot-affordance.test.ts src/lib/analysis-run-companion-state.test.ts src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas-route.test.ts src/lib/analysis-run-companion-route.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS for all listed files.

- [ ] **Step 3: Run Svelte and TypeScript check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check` exits with 0 errors and 0 warnings.

- [ ] **Step 4: Run full verification**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS. This should include Vitest, Svelte check, Rust checks/tests, and diff checks as defined by `scripts/verify.mjs`.

- [ ] **Step 5: Commit verification bookkeeping**

If only `docs/backlog.md` and this plan's checkboxes changed after verification, run:

```powershell
git add docs/backlog.md docs/superpowers/plans/2026-05-31-saved-runs-missing-capture-affordances-implementation.md
git commit -m "docs: track saved run affordance implementation"
```

Expected: commit succeeds. If the backlog and plan checkbox updates were already included in task commits and no files changed, skip this commit.

---

## Acceptance Checklist

- [ ] Pure helper covers captured/available, legacy missing, capture failed with error, capture failed without error, terminal-before-capture, inconsistent captured marker, verification failure, null snapshot state matrix, active/checking/pending, and status predicates.
- [ ] Runs tab visibly badges degraded saved rows and does not show raw or sanitized snapshot errors in rows.
- [ ] Opened-run header shows short degraded warning only and puts details/error text in `Run details`.
- [ ] Source tab distinguishes legacy missing, capture failed, not captured before end, inconsistent rows, and verification failure.
- [ ] Evidence `Show in source` remains disabled for completed runs without usable saved snapshots and uses helper disabled reasons.
- [ ] Chat remains disabled without usable saved snapshot context and uses helper-derived disabled copy.
- [ ] Live source browsing remains explicit and is described as live data rather than saved run corpus.
- [ ] `docs/backlog.md` Saved Runs Discoverability acceptance text describes missing/capture-failed affordances rather than already-shipped narrowing.
- [ ] Backend DTOs, report execution, snapshot capture, Runs filters, cleanup flows, and GUI smoke coverage remain unchanged.
