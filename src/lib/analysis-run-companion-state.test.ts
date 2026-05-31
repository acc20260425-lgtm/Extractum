import { describe, expect, it } from "vitest";
import {
  chatAvailabilityForRun,
  defaultCompanionTabForOpenedRun,
  evidenceSourceActionDecision,
  filterCompanionRuns,
  hasActiveCompanionRunsFilter,
  hasSavedRunsForWorkspace,
  runsFilterDefaults,
  type CompanionRunsFilterState,
} from "./analysis-run-companion-state";
import type { RunSnapshotAvailability } from "./analysis-report-canvas-state";
import type { SnapshotProbeState } from "./analysis-run-snapshot-affordance";
import type { WorkspaceSelection } from "./analysis-workspace-state";
import type { AnalysisRunDetail, AnalysisRunSummary, AnalysisTraceRef } from "./types/analysis";

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 42,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Telegram A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Telegram A",
    period_from: 1710000000,
    period_to: 1710100000,
    output_language: "Russian",
    prompt_template_id: 1,
    prompt_template_name: "Weekly",
    prompt_template_version: 3,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description",
    telegram_history_scope: "current",
    status: "completed",
    result_markdown: "Saved report",
    error: null,
    has_trace_data: true,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-05-18T10:00:00Z",
    snapshot_error: null,
    created_at: 1710100010,
    completed_at: 1710100100,
    ...overrides,
  };
}

function summary(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 1,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Telegram A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Telegram A",
    period_from: 1710000000,
    period_to: 1710100000,
    output_language: "Russian",
    prompt_template_id: 1,
    prompt_template_name: "Weekly",
    prompt_template_version: 3,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: true,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-05-18T10:00:00Z",
    snapshot_error: null,
    created_at: 1710100010,
    completed_at: 1710100100,
    ...overrides,
  };
}

function traceRef(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
  return {
    ref: "s7-i11",
    item_id: 11,
    source_id: 7,
    external_id: "11",
    published_at: 1710000020,
    excerpt: "Saved excerpt",
    youtube_url: null,
    youtube_timestamp_seconds: null,
    youtube_display_label: null,
    is_synthetic: false,
    ...overrides,
  };
}

describe("analysis run companion state", () => {
  it("defaults completed runs to Evidence and active runs to Runs", () => {
    expect(defaultCompanionTabForOpenedRun(run({ status: "completed" }))).toBe("evidence");
    expect(defaultCompanionTabForOpenedRun(run({ status: "queued", completed_at: null }))).toBe("runs");
    expect(defaultCompanionTabForOpenedRun(run({ status: "running", completed_at: null }))).toBe("runs");
    expect(defaultCompanionTabForOpenedRun(run({ status: "failed", error: "failed" }))).toBe("runs");
    expect(defaultCompanionTabForOpenedRun(null)).toBe("runs");
  });

  it.each([
    ["unknown", false, "checking_snapshot"],
    ["capturing", false, "checking_snapshot"],
    ["available", true, "enabled"],
    ["unavailable", false, "inconsistent"],
  ] satisfies Array<[RunSnapshotAvailability, boolean, string]>)(
    "maps completed run chat availability for %s snapshot",
    (snapshotAvailability, enabled, reason) => {
      expect(chatAvailabilityForRun({
        currentRun: run({ status: "completed" }),
        snapshotAvailability,
        snapshotProbeState: snapshotAvailability === "available"
          ? "available"
          : snapshotAvailability === "unavailable"
            ? "unavailable"
            : "unknown",
      })).toMatchObject({ enabled, reason });
    },
  );

  it("disables chat for no run, active runs, failed runs, and cancelled runs", () => {
    expect(chatAvailabilityForRun({
      currentRun: null,
      snapshotAvailability: "unknown",
      snapshotProbeState: "unknown",
    })).toMatchObject({ enabled: false, reason: "no_run" });
    expect(chatAvailabilityForRun({
      currentRun: run({ status: "running", completed_at: null }),
      snapshotAvailability: "capturing",
      snapshotProbeState: "unknown",
    })).toMatchObject({ enabled: false, reason: "pending_completion" });
    expect(chatAvailabilityForRun({
      currentRun: run({ status: "failed", error: "provider failed" }),
      snapshotAvailability: "available",
      snapshotProbeState: "available",
    })).toMatchObject({ enabled: false, reason: "terminal_run" });
    expect(chatAvailabilityForRun({
      currentRun: run({ status: "cancelled" }),
      snapshotAvailability: "available",
      snapshotProbeState: "available",
    })).toMatchObject({ enabled: false, reason: "terminal_run" });
  });

  it.each([
    [
      "missing_legacy",
      "unavailable",
      "legacy_missing",
      "Saved context unavailable",
      "Older saved runs may not include frozen source rows",
    ],
    [
      "capture_failed",
      "unavailable",
      "capture_failed_with_error",
      "Saved context unavailable",
      "could not save the frozen source context",
    ],
    [
      null,
      "unavailable",
      "not_captured_before_terminal",
      "Saved context unavailable",
      "there is no frozen source corpus",
    ],
    [
      "captured",
      "unavailable",
      "inconsistent",
      "Saved context unavailable",
      "marked as captured, but Extractum could not load saved snapshot rows",
    ],
    [
      "captured",
      "error",
      "verification_failed",
      "Saved context unavailable",
      "could not verify saved snapshot rows",
    ],
  ] satisfies Array<[AnalysisRunDetail["snapshot_state"], SnapshotProbeState, string, string, string]>)(
    "uses snapshot affordance copy for completed-run chat when state is %s and probe is %s",
    (snapshotState, snapshotProbeState, reason, title, descriptionFragment) => {
      const snapshotError = snapshotState === "capture_failed" ? "sqlite write failed" : null;
      const availability = snapshotProbeState === "available" ? "available" : "unavailable";

      const result = chatAvailabilityForRun({
        currentRun: run({
          status: snapshotState === null ? "failed" : "completed",
          snapshot_state: snapshotState,
          snapshot_error: snapshotError,
        }),
        snapshotAvailability: availability,
        snapshotProbeState,
      });

      expect(result).toMatchObject({
        enabled: false,
        reason,
        title,
      });
      expect(result.description).toContain(descriptionFragment);
    },
  );

  it("prefers run snapshot for Show in source when snapshot is available", () => {
    expect(evidenceSourceActionDecision({
      currentRun: run({ status: "completed" }),
      selectedTrace: traceRef(),
      snapshotAvailability: "available",
      snapshotProbeState: "available",
    })).toEqual({
      kind: "run_snapshot",
      canvasMode: "source",
      sourceViewBasis: "run_snapshot",
      highlightedRef: "s7-i11",
    });
  });

  it("does not resolve completed-run evidence against live source when snapshot is missing", () => {
    expect(evidenceSourceActionDecision({
      currentRun: run({ status: "completed" }),
      selectedTrace: traceRef(),
      snapshotAvailability: "unavailable",
      snapshotProbeState: "unavailable",
    })).toEqual({
      kind: "unavailable",
      reason: "Exact source resolution is unavailable because the run is marked captured but saved snapshot rows are unavailable.",
    });
  });

  it("allows explicit live source bridge for active runs without labeling it as snapshot", () => {
    expect(evidenceSourceActionDecision({
      currentRun: run({ status: "running", completed_at: null }),
      selectedTrace: traceRef(),
      snapshotAvailability: "unavailable",
      snapshotProbeState: "unavailable",
    })).toMatchObject({
      kind: "live_source",
      canvasMode: "source",
      sourceViewBasis: "live_source",
      highlightedRef: "s7-i11",
    });
  });

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

      const result = evidenceSourceActionDecision({
        currentRun: run({
          status: snapshotState === null ? "failed" : "completed",
          snapshot_state: snapshotState,
          snapshot_error: snapshotError,
        }),
        selectedTrace: traceRef(),
        snapshotAvailability: "unavailable",
        snapshotProbeState,
      });

      expect(result).toMatchObject({ kind: "unavailable" });
      expect(result).toMatchObject({
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

  it("filters companion runs across active and saved report runs", () => {
    const filter: CompanionRunsFilterState = {
      ...runsFilterDefaults(),
      query: "weekly openai",
      status: "queued_running",
      scope: "current",
      dateFrom: "",
      dateTo: "",
      provider: "",
      model: "",
      template: "",
    };
    const workspaceSelection: WorkspaceSelection = { kind: "source", sourceId: 7 };

    const result = filterCompanionRuns({
      activeRuns: [
        summary({ id: 10, status: "running", provider: "openai", model: "gpt-5.4" }),
        summary({ id: 11, status: "queued", source_id: 8, source_title: "Other" }),
      ],
      savedRuns: [
        summary({ id: 12, status: "completed" }),
      ],
      filter,
      workspaceSelection,
    });

    expect(result.map((entry) => [entry.kind, entry.run.id])).toEqual([["active", 10]]);
  });

  it("supports cancelled status and optional provider/model/template filters", () => {
    const result = filterCompanionRuns({
      activeRuns: [],
      savedRuns: [
        summary({ id: 20, status: "cancelled", provider: "openai", model: "gpt-5.4", prompt_template_name: "Weekly" }),
        summary({ id: 21, status: "failed", provider: "gemini", model: "flash", prompt_template_name: "Daily" }),
      ],
      filter: {
        query: "",
        status: "cancelled",
        scope: "all",
        dateFrom: "2024-03-01",
        dateTo: "2024-03-31",
        provider: "openai",
        model: "gpt-5.4",
        template: "weekly",
      },
      workspaceSelection: { kind: "none" },
    });

    expect(result.map((entry) => entry.run.id)).toEqual([20]);
  });

  it("keeps local saved-run filtering as a final consistency guard", () => {
    const filter: CompanionRunsFilterState = {
      ...runsFilterDefaults(),
      query: "needle",
      status: "completed",
      provider: "gemini",
      model: "flash",
      template: "digest",
    };
    const entries = filterCompanionRuns({
      activeRuns: [],
      savedRuns: [
        summary({
          id: 30,
          scope_label: "Needle report",
          status: "completed",
          provider: "gemini",
          model: "gemini-2.5-flash",
          prompt_template_name: "Daily digest",
        }),
        summary({
          id: 31,
          scope_label: "Needle report",
          status: "failed",
          provider: "gemini",
          model: "gemini-2.5-flash",
          prompt_template_name: "Daily digest",
        }),
      ],
      filter,
      workspaceSelection: { kind: "none" },
    });

    expect(entries.map((entry) => entry.run.id)).toEqual([30]);
  });

  it("detects active companion run filters beyond the defaults", () => {
    expect(hasActiveCompanionRunsFilter(runsFilterDefaults())).toBe(false);
    expect(hasActiveCompanionRunsFilter({
      ...runsFilterDefaults(),
      provider: "openai",
    })).toBe(true);
    expect(hasActiveCompanionRunsFilter({
      ...runsFilterDefaults(),
      scope: "current",
    })).toBe(true);
  });

  it("detects saved runs for the selected workspace scope", () => {
    expect(hasSavedRunsForWorkspace({
      savedRuns: [
        summary({ id: 20, source_id: 7, source_group_id: null }),
        summary({ id: 21, source_id: 8, source_title: "Other" }),
      ],
      workspaceSelection: { kind: "source", sourceId: 7 },
    })).toBe(true);

    expect(hasSavedRunsForWorkspace({
      savedRuns: [
        summary({ id: 22, scope_type: "source_group", source_id: null, source_group_id: 4, source_group_name: "Group A" }),
      ],
      workspaceSelection: { kind: "source_group", sourceGroupId: 4 },
    })).toBe(true);

    expect(hasSavedRunsForWorkspace({
      savedRuns: [summary({ id: 23, source_id: 9, source_title: "Other" })],
      workspaceSelection: { kind: "source", sourceId: 7 },
    })).toBe(false);
  });
});
