import { describe, expect, it } from "vitest";
import {
  canStartYoutubeSummary,
  summarizePreflightPartitions,
  updateRunListFromEvent,
} from "./youtube-summary-workflow";
import type { PromptPackRunEvent, YoutubeSummaryPreflightResponse } from "$lib/types/prompt-packs";

describe("youtube summary workflow", () => {
  it("allows start only with included videos and no blocking failures", () => {
    const preflight: YoutubeSummaryPreflightResponse = {
      packId: "youtube_summary",
      packVersion: "1.0.0",
      includedVideos: [{ sourceId: 1, videoId: "v1", title: "Ready", estimatedInputTokens: 1200 }],
      skippedVideos: [],
      blockingFailures: [],
      estimatedInputTokens: 1200,
      selectedModelInputLimit: 32000,
    };

    expect(canStartYoutubeSummary(preflight)).toBe(true);
  });

  it("summarizes partial playlist partitions", () => {
    const summary = summarizePreflightPartitions({
      includedVideos: [{ sourceId: 1, videoId: "v1", title: "Ready", estimatedInputTokens: 1200 }],
      skippedVideos: [{ sourceId: 2, videoId: "v2", title: "Missing", reason: "no_usable_transcript" }],
      blockingFailures: [],
    });

    expect(summary).toEqual({
      includedCount: 1,
      skippedCount: 1,
      blockingCount: 0,
      hasPartialCoverage: true,
    });
  });

  it("updates run list from prompt pack run event", () => {
    const event: PromptPackRunEvent = {
      runId: 42,
      requestId: "req-42",
      kind: "progress",
      runStatus: "running",
      phase: "stage",
      stageRunId: 1001,
      stageName: "youtube_summary/transcript_analysis",
      sourceSnapshotId: 501,
      queuePosition: null,
      progressCurrent: 1,
      progressTotal: 2,
      message: "Analyzing transcript",
      error: null,
    };

    const runs = updateRunListFromEvent([], event);

    expect(runs[0].runId).toBe(42);
    expect(runs[0].runStatus).toBe("running");
    expect(runs[0].latestMessage).toBe("Analyzing transcript");
  });
});
