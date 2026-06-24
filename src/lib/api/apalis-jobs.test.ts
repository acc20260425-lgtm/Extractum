import { beforeEach, describe, expect, it, vi } from "vitest";
import apalisJobsApiSource from "./apalis-jobs.ts?raw";
import { loadApalisJobs, pruneOldTerminalApalisJobs } from "./apalis-jobs";
import type {
  ApalisJobsListResponse,
  ApalisJobsPruneTerminalResponse,
} from "$lib/types/apalis-jobs";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

function responseFixture(): ApalisJobsListResponse {
  return {
    jobs: [
      {
        id: "job-1",
        jobType: "gemini-browser",
        status: "Pending",
        attempts: 0,
        maxAttempts: 1,
        runAt: "2026-06-23T10:00:00Z",
        lockAt: null,
        lockBy: null,
        doneAt: null,
        lastActivityAt: "2026-06-23T10:00:00Z",
        priority: 0,
        idempotencyKey: "run-1",
        jobPreview: null,
        jobTruncated: false,
        jobJson: { run_id: "run-1" },
        lastResult: null,
        lastResultTruncated: false,
        metadata: null,
        metadataTruncated: false,
      },
    ],
    totalMatching: 1,
    statusCounts: [{ status: "Pending", count: 1 }],
    jobTypeCounts: [{ jobType: "gemini-browser", count: 1 }],
    refreshedAt: "2026-06-23T10:00:01Z",
    limit: 100,
  };
}

describe("apalis jobs api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads Apalis jobs through the dedicated Tauri command", async () => {
    const fixture = responseFixture();
    invokeMock.mockResolvedValueOnce(fixture);

    await expect(
      loadApalisJobs({
        limit: 50,
        status: "Pending",
        jobType: "gemini-browser",
        search: "run",
      }),
    ).resolves.toBe(fixture);

    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith("apalis_jobs_list", {
      request: {
        limit: 50,
        status: "Pending",
        jobType: "gemini-browser",
        search: "run",
      },
    });
  });

  it("prunes old terminal Apalis jobs through the dedicated Tauri command", async () => {
    const fixture: ApalisJobsPruneTerminalResponse = {
      deletedCount: 3,
      cutoffAt: "2026-06-23T12:00:00Z",
      olderThanHours: 24,
    };
    invokeMock.mockResolvedValueOnce(fixture);

    await expect(pruneOldTerminalApalisJobs()).resolves.toBe(fixture);

    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith("apalis_jobs_prune_terminal", {
      request: {
        olderThanHours: 24,
      },
    });
  });

  it("keeps the wrapper narrow and free of logging or client-side mapping", () => {
    expect(apalisJobsApiSource).not.toContain("console.error");
    expect(apalisJobsApiSource).not.toContain("JSON.stringify");
    expect(apalisJobsApiSource).not.toContain(".then(");
    expect(apalisJobsApiSource).not.toContain("filter(");
  });
});
