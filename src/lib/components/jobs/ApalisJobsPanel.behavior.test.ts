import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({
  loadApalisJobs: vi.fn(),
  pruneOldTerminalApalisJobs: vi.fn(),
}));
vi.mock("$lib/api/apalis-jobs", () => ({
  APALIS_OLD_TERMINAL_PRUNE_HOURS: 24,
  ...api,
}));
vi.mock("@svar-ui/svelte-core", async () => ({
  Locale: (await import("$lib/testing/SvarLocaleReceiver.svelte")).default,
}));
vi.mock("@svar-ui/svelte-grid", async () => ({
  Grid: (await import("$lib/testing/SvarGridReceiver.svelte")).default,
  Willow: (await import("$lib/testing/SvarWillowReceiver.svelte")).default,
}));
vi.mock("@svar-ui/core-locales", () => ({ ru: {} }));
vi.mock("@svar-ui/grid-locales", () => ({ en: {} }));

import type { ApalisJobRow, ApalisJobsListResponse } from "$lib/types/apalis-jobs";
import ApalisJobsPanel from "./ApalisJobsPanel.svelte";

function job(overrides: Partial<ApalisJobRow> = {}): ApalisJobRow {
  return {
    id: "job-1",
    jobType: "youtube-sync",
    status: "Running",
    attempts: 1,
    maxAttempts: 3,
    runAt: "2026-08-08T09:00:00.000Z",
    lockAt: "2026-08-08T09:01:00.000Z",
    lockBy: "worker-1",
    doneAt: null,
    lastActivityAt: "2026-08-08T09:02:00.000Z",
    priority: 1,
    idempotencyKey: "sync:video-1",
    jobPreview: null,
    jobTruncated: false,
    jobJson: { video_id: "[redacted]" },
    lastResult: { state: "running" },
    lastResultTruncated: false,
    metadata: { source: "youtube" },
    metadataTruncated: true,
    ...overrides,
  };
}

function response(jobs = [job(), job({ id: "job-2", idempotencyKey: "sync:video-2", status: "Failed" })]): ApalisJobsListResponse {
  return {
    jobs,
    totalMatching: jobs.length,
    statusCounts: [{ status: "Running", count: 1 }, { status: "Failed", count: 1 }],
    jobTypeCounts: [{ jobType: "youtube-sync", count: jobs.length }],
    refreshedAt: "2026-08-08T09:03:00.000Z",
    limit: 100,
  };
}

beforeEach(() => api.loadApalisJobs.mockResolvedValue(response()));
afterEach(cleanup);
afterEach(() => vi.clearAllMocks());

describe("apalis jobs panel", () => {
  it("reloads backend results when filters change", async () => {
    render(ApalisJobsPanel);
    await waitFor(() => expect(api.loadApalisJobs).toHaveBeenCalledWith({
      limit: 100,
      status: null,
      jobType: null,
      search: null,
    }));

    await fireEvent.change(screen.getByLabelText("Status"), { target: { value: "Failed" } });
    await waitFor(() => expect(api.loadApalisJobs).toHaveBeenLastCalledWith({
      limit: 100,
      status: "Failed",
      jobType: null,
      search: null,
    }));

    await fireEvent.change(screen.getByLabelText("Job type"), { target: { value: "youtube-sync" } });
    await waitFor(() => expect(api.loadApalisJobs).toHaveBeenLastCalledWith({
      limit: 100,
      status: "Failed",
      jobType: "youtube-sync",
      search: null,
    }));
  });

  it("formats timestamps in the user's locale and time zone", async () => {
    render(ApalisJobsPanel);
    await waitFor(() => expect(api.loadApalisJobs).toHaveBeenCalled());
    const expected = new Intl.DateTimeFormat(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date("2026-08-08T09:02:00.000Z"));
    expect(screen.getByLabelText("Formatted grid date").textContent).toBe(expected);
  });

  it("presents safe payload labels and preserves the selected job", async () => {
    render(ApalisJobsPanel);
    await screen.findByRole("heading", { name: "sync:video-1" });
    await fireEvent.click(screen.getByRole("button", { name: "Select grid row job-2" }));
    expect(screen.getByRole("heading", { name: "sync:video-2" })).toBeTruthy();

    api.loadApalisJobs.mockResolvedValueOnce(response());
    await fireEvent.change(screen.getByLabelText("Limit"), { target: { value: "200" } });
    await waitFor(() => expect(api.loadApalisJobs).toHaveBeenLastCalledWith(expect.objectContaining({ limit: 200 })));
    expect(screen.getByRole("heading", { name: "sync:video-2" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Job payload" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Last result" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Metadata" })).toBeTruthy();
    expect(screen.getAllByText("redacted").length).toBeGreaterThan(0);
    expect(screen.getByText("truncated")).toBeTruthy();
  });
});
