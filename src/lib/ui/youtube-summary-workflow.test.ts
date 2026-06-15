import { describe, expect, it } from "vitest";
import {
  canStartYoutubeSummary,
  retainSelectedRunId,
  shouldApplyRunEventToRunsPanel,
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

  it("marks run terminal from completed event", () => {
    const runs = updateRunListFromEvent(
      [{ runId: 42, runStatus: "running", latestMessage: "Running" }],
      {
        runId: 42,
        requestId: "req-42",
        kind: "completed",
        runStatus: "complete",
        phase: "terminal",
        stageRunId: null,
        stageName: null,
        sourceSnapshotId: null,
        queuePosition: null,
        progressCurrent: 2,
        progressTotal: 2,
        message: "Completed",
        error: null,
      },
    );

    expect(runs[0].runStatus).toBe("complete");
    expect(runs[0].latestMessage).toBe("Completed");
  });

  it("clears selected run when refreshed project runs no longer contain it", () => {
    expect(retainSelectedRunId(5, [{ runId: 4 }])).toBeNull();
    expect(retainSelectedRunId(5, [{ runId: 5 }])).toBe(5);
    expect(retainSelectedRunId(null, [{ runId: 5 }])).toBeNull();
  });

  it("does not insert unknown global prompt pack events into project-scoped panels", () => {
    const event: PromptPackRunEvent = {
      runId: 99,
      requestId: "req-99",
      kind: "started",
      runStatus: "running",
      phase: "stage",
      stageRunId: null,
      stageName: null,
      sourceSnapshotId: null,
      queuePosition: null,
      progressCurrent: 0,
      progressTotal: 1,
      message: "Running",
      error: null,
    };

    expect(shouldApplyRunEventToRunsPanel([], event, 7)).toBe(false);
    expect(shouldApplyRunEventToRunsPanel([], event, null)).toBe(true);
    expect(shouldApplyRunEventToRunsPanel([{ runId: 99 }], event, 7)).toBe(true);
  });
});
