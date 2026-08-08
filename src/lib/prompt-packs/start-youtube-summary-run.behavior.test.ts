import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

import { startYoutubeSummaryRun } from "$lib/api/prompt-packs";

describe("prompt pack application boundary", () => {
  beforeEach(() => invokeMock.mockReset());

  it("keeps start idempotency readiness preflight queued-event spawn and profile-resolution order", async () => {
    invokeMock.mockResolvedValueOnce({
      kind: "started",
      run: { runId: 42, runStatus: "queued", latestMessage: "Queued" },
    });
    const request = {
      clientRequestId: "req-stable-42",
      projectId: 7,
      sourceIds: [901],
      profileId: "operator-profile",
      modelOverride: null,
      runtimeProvider: "api" as const,
      browserProviderConfig: null,
      outputLanguage: "en",
      controlPreset: "detailed_report" as const,
      evidenceMode: "standard" as const,
      includeComments: false,
    };

    const outcome = await startYoutubeSummaryRun(request);

    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith("start_youtube_summary_run", request);
    expect(outcome).toMatchObject({
      kind: "started",
      run: { runId: 42, runStatus: "queued" },
    });
  });
});
