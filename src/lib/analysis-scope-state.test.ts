import { describe, expect, it } from "vitest";
import {
  analysisHistoryScopeParams,
  currentAnalysisGroup,
  currentAnalysisScopeSummary,
  currentAnalysisScopeTitle,
  currentAnalysisSource,
  currentAnalysisSourceMetric,
} from "./analysis-scope-state";
import type {
  AnalysisSourceGroup,
  AnalysisSourceOption,
} from "./types/analysis";
import type { Source } from "./types/sources";

function source(overrides: Partial<Source> = {}): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "channel",
    accountId: 1,
    externalId: "@extractum",
    title: "Extractum",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
    telegramUsername: null,
    avatarDataUrl: null,
    ...overrides,
  };
}

function metric(overrides: Partial<AnalysisSourceOption> = {}): AnalysisSourceOption {
  return {
    id: 1,
    account_id: 1,
    source_type: "telegram",
    title: "Extractum",
    item_count: 42,
    last_synced_at: 200,
    ...overrides,
  };
}

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 7,
    name: "Research group",
    source_type: "telegram",
    members: [
      { source_id: 1, source_title: "Extractum", item_count: 42 },
      { source_id: 2, source_title: "Signals", item_count: 13 },
    ],
    created_at: 100,
    updated_at: 200,
    ...overrides,
  };
}

describe("analysis-scope-state", () => {
  it("finds the selected source and its metric only when ids are present", () => {
    const sources = [source({ id: 1 }), source({ id: 2, title: "Other" })];
    const metrics = { 1: metric({ id: 1 }) };

    expect(currentAnalysisSource("2", sources)).toEqual(sources[1]);
    expect(currentAnalysisSource("", sources)).toBeNull();
    expect(currentAnalysisSource("9", sources)).toBeNull();

    expect(currentAnalysisSourceMetric(sources[0], metrics)).toEqual(metrics[1]);
    expect(currentAnalysisSourceMetric(sources[1], metrics)).toBeNull();
    expect(currentAnalysisSourceMetric(null, metrics)).toBeNull();
  });

  it("finds the selected source group only when an id is present", () => {
    const groups = [group({ id: 7 }), group({ id: 8, name: "Other" })];

    expect(currentAnalysisGroup("8", groups)).toEqual(groups[1]);
    expect(currentAnalysisGroup("", groups)).toBeNull();
    expect(currentAnalysisGroup("9", groups)).toBeNull();
  });

  it("formats current scope title for source and group workspaces", () => {
    expect(currentAnalysisScopeTitle("source_group", null, group({ name: "Signals" })))
      .toBe("Signals");
    expect(currentAnalysisScopeTitle("source_group", null, null)).toBe("Source group");
    expect(currentAnalysisScopeTitle("single_source", source({ title: "Named" }), null))
      .toBe("Named");
    expect(currentAnalysisScopeTitle("single_source", source({ title: null, externalId: "@raw" }), null))
      .toBe("@raw");
    expect(currentAnalysisScopeTitle("single_source", null, null)).toBe("Source");
  });

  it("formats current scope summary from group membership or source metrics", () => {
    expect(currentAnalysisScopeSummary("source_group", null, group(), null))
      .toBe("2 sources in this group workspace.");
    expect(currentAnalysisScopeSummary("source_group", null, null, null))
      .toBe("Select a saved source group to run a cross-source report.");
    expect(currentAnalysisScopeSummary("single_source", source(), null, metric({ item_count: 12 })))
      .toBe("12 synced items available locally for analysis.");
    expect(currentAnalysisScopeSummary("single_source", source(), null, null))
      .toBe("This source is available in the workspace but has no synced item count yet.");
    expect(currentAnalysisScopeSummary("single_source", null, null, null))
      .toBe("Select a synced source to inspect messages and launch a report.");
  });

  it("builds run history scope params from the current filter and selected ids", () => {
    expect(analysisHistoryScopeParams("all", "single_source", "", "")).toEqual({
      sourceId: null,
      sourceGroupId: null,
    });
    expect(analysisHistoryScopeParams("current", "single_source", "12", ""))
      .toEqual({ sourceId: 12, sourceGroupId: null });
    expect(analysisHistoryScopeParams("current", "source_group", "", "7"))
      .toEqual({ sourceId: null, sourceGroupId: 7 });
    expect(analysisHistoryScopeParams("current", "single_source", "", ""))
      .toBeNull();
  });
});
