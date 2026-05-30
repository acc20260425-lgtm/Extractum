import { describe, expect, it } from "vitest";
import {
  SmokeAssertionError,
  SmokeBridgeError,
  assertEmptyFixtureSummary,
  assertTabOrderLabels,
  bridgePortCandidates,
  classifyBridgeFailure,
  executeJs,
  expectedFixtureLabels,
  sanitizeArtifactName,
  validateFixtureLabels,
  validateFixtureSummary,
} from "../../scripts/analysis-smoke-helpers.mjs";

describe("analysis smoke helper contracts", () => {
  it("builds the MCP bridge port range deterministically", () => {
    expect(bridgePortCandidates(9223, 9226)).toEqual([9223, 9224, 9225, 9226]);
  });

  it("sanitizes deterministic step names for artifact paths", () => {
    expect(sanitizeArtifactName("source-browser.youtube-video-tabs")).toBe("source-browser.youtube-video-tabs");
    expect(sanitizeArtifactName("Workspace Parity: Source Group")).toBe("workspace-parity-source-group");
  });

  it("asserts exact tab order", () => {
    expect(() => assertTabOrderLabels(["Sources", "Items", "Metadata"], ["Sources", "Items", "Metadata"]))
      .not.toThrow();
    expect(() => assertTabOrderLabels(["Sources", "Metadata", "Items"], ["Sources", "Items", "Metadata"]))
      .toThrow(SmokeAssertionError);
  });

  it("validates required fixture labels", () => {
    expect(validateFixtureLabels(expectedFixtureLabels)).toEqual(expectedFixtureLabels);
    expect(() => validateFixtureLabels(expectedFixtureLabels.filter((label) => !label.includes("YouTube Video"))))
      .toThrow(SmokeAssertionError);
  });

  it("validates deterministic fixture summary minimums", () => {
    expect(validateFixtureSummary({
      accounts: 1,
      chatMessages: 2,
      llmProfiles: 1,
      promptTemplates: 1,
      runs: 6,
      snapshotMessages: 4,
      sourceGroups: 1,
      sources: 4,
      youtubePlaylistItems: 2,
      youtubeTranscriptSegments: 3,
    })).toBe(true);

    expect(() => validateFixtureSummary({
      accounts: 1,
      chatMessages: 2,
      llmProfiles: 1,
      promptTemplates: 1,
      runs: 6,
      snapshotMessages: 4,
      sourceGroups: 0,
      sources: 4,
      youtubePlaylistItems: 2,
      youtubeTranscriptSegments: 3,
    })).toThrow(SmokeAssertionError);
  });

  it("validates empty fixture cleanup summaries", () => {
    const emptySummary = {
      accounts: 0,
      chatMessages: 0,
      llmProfiles: 0,
      promptTemplates: 0,
      runs: 0,
      snapshotMessages: 0,
      sourceGroups: 0,
      sources: 0,
      youtubePlaylistItems: 0,
      youtubeTranscriptSegments: 0,
    };

    expect(assertEmptyFixtureSummary(emptySummary)).toBe(true);

    expect(() => assertEmptyFixtureSummary({
      ...emptySummary,
      snapshotMessages: 1,
    })).toThrow(SmokeAssertionError);

    const { accounts, ...missingAccountSummary } = emptySummary;
    expect(() => assertEmptyFixtureSummary(missingAccountSummary)).toThrow(SmokeAssertionError);
  });

  it("classifies bridge failures distinctly", () => {
    expect(classifyBridgeFailure(new SmokeBridgeError("bridge unavailable", "bridge-unavailable")).kind)
      .toBe("bridge-unavailable");
    expect(classifyBridgeFailure(new Error("ASSERT: missing tab")).kind).toBe("assertion");
    expect(classifyBridgeFailure(new Error("Script execution timeout")).kind).toBe("script-timeout");
  });

  it("keeps executeJs assertion failures typed as smoke assertions", async () => {
    await expect(executeJs(fakeSocketResponse({
      id: "execute_js-1",
      success: false,
      error: "ASSERT: missing source-browser-tabs",
    }), "return true;")).rejects.toThrow(SmokeAssertionError);
  });

  it("classifies app identifier mismatch separately from unavailable bridge", () => {
    expect(classifyBridgeFailure(new SmokeBridgeError("unexpected app identifier", "app-identifier-mismatch")).kind)
      .toBe("app-identifier-mismatch");
  });
});

function fakeSocketResponse(response: Record<string, unknown>) {
  const listeners = new Map<string, Set<(event: { data?: string }) => void>>();
  const socket = {
    addEventListener(type: string, listener: (event: { data?: string }) => void) {
      const set = listeners.get(type) ?? new Set();
      set.add(listener);
      listeners.set(type, set);
    },
    removeEventListener(type: string, listener: (event: { data?: string }) => void) {
      listeners.get(type)?.delete(listener);
    },
    send(message: string) {
      const request = JSON.parse(message);
      queueMicrotask(() => {
        const next = { ...response, id: request.id };
        listeners.get("message")?.forEach((listener) => listener({ data: JSON.stringify(next) }));
      });
    },
  };
  return socket;
}
