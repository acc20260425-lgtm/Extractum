import { describe, expect, it } from "vitest";
import {
  runSnapshotAvailabilityFromPage,
  sourceBasisDescription,
  sourceBasisLabel,
  sourceCanvasSurface,
  youtubeCorpusModeLabel,
  type RunSnapshotAvailability,
} from "./analysis-report-canvas-state";
import type { AnalysisRunDetail, AnalysisRunMessagesPage } from "./types/analysis";

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 42,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Source A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Source A",
    period_from: 1704067200,
    period_to: 1706659200,
    output_language: "Russian",
    prompt_template_id: 3,
    prompt_template_name: "Weekly",
    prompt_template_version: 5,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description_comments",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: true,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-05-18T10:00:00Z",
    snapshot_error: null,
    created_at: 1706660000,
    completed_at: 1706660400,
    result_markdown: "Report",
    ...overrides,
  };
}

function page(messageCount: number): AnalysisRunMessagesPage {
  return {
    messages: Array.from({ length: messageCount }, (_, index) => ({
      item_id: index + 1,
      source_id: 7,
      external_id: `m-${index + 1}`,
      author: null,
      published_at: 1704067200 + index,
      ref: `S7-${index + 1}`,
      content: `Message ${index + 1}`,
      item_kind: "message",
      source_type: "telegram",
      source_subtype: null,
      metadata_json: null,
    })),
    next_cursor: null,
    has_more: false,
  };
}

describe("report canvas state", () => {
  it("uses report setup and live source when no run is open", () => {
    expect(sourceCanvasSurface({
      currentRun: null,
      sourceViewBasis: "live_source",
      snapshotAvailability: "unknown",
    })).toBe("live_source");

    expect(sourceBasisLabel({
      currentRun: null,
      sourceViewBasis: "live_source",
      snapshotAvailability: "unknown",
    })).toBe("Live source");
  });

  it("marks active empty snapshot pages as capturing only after probing snapshot data", () => {
    const availability = runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "running", completed_at: null }),
      page: page(0),
      loading: false,
      errorMessage: "",
    });

    expect(availability).toBe("capturing");
  });

  it("marks failed and cancelled empty snapshot pages as unavailable after probing snapshot data", () => {
    expect(runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "failed", error: "Provider failed" }),
      page: page(0),
      loading: false,
      errorMessage: "",
    })).toBe("unavailable");

    expect(runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "cancelled" }),
      page: page(0),
      loading: false,
      errorMessage: "",
    })).toBe("unavailable");
  });

  it("treats any non-empty snapshot page as available regardless of terminal status", () => {
    expect(runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "failed", error: "Failed after capture" }),
      page: page(1),
      loading: false,
      errorMessage: "",
    })).toBe("available");
  });

  it("does not infer snapshot availability from run status alone", () => {
    expect(runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "completed" }),
      page: null,
      loading: false,
      errorMessage: "",
    })).toBe("unknown");
  });

  it("keeps explicit live source visible while an opened run remains bound", () => {
    expect(sourceCanvasSurface({
      currentRun: run(),
      sourceViewBasis: "live_source",
      snapshotAvailability: "available",
    })).toBe("live_source");

    expect(sourceBasisLabel({
      currentRun: run(),
      sourceViewBasis: "live_source",
      snapshotAvailability: "available",
    })).toBe("Live source");
  });

  it.each([
    ["unknown", "Checking snapshot"],
    ["capturing", "Snapshot pending"],
    ["available", "Snapshot available"],
    ["unavailable", "Snapshot unavailable"],
  ] satisfies Array<[RunSnapshotAvailability, string]>)("labels %s snapshot basis", (availability, label) => {
    expect(sourceBasisLabel({
      currentRun: run(),
      sourceViewBasis: "run_snapshot",
      snapshotAvailability: availability,
    })).toBe(label);
  });

  it.each([
    ["unknown", "Checking whether a frozen source snapshot is available for this run."],
    ["capturing", "Snapshot capture is still in progress for this run."],
    ["available", "Frozen source material captured for this run is available."],
    ["unavailable", "No frozen source snapshot is available for this run."],
  ] satisfies Array<[RunSnapshotAvailability, string]>)("describes %s snapshot basis consistently", (availability, description) => {
    expect(sourceBasisDescription({
      currentRun: run(),
      sourceViewBasis: "run_snapshot",
      snapshotAvailability: availability,
    })).toBe(description);
  });

  it("labels YouTube corpus modes for run headers", () => {
    expect(youtubeCorpusModeLabel("transcript_only")).toBe("Transcript");
    expect(youtubeCorpusModeLabel("transcript_description")).toBe("Transcript + description");
    expect(youtubeCorpusModeLabel("transcript_description_comments")).toBe("Transcript + description + comments");
    expect(youtubeCorpusModeLabel(null)).toBe("Not recorded");
  });
});
