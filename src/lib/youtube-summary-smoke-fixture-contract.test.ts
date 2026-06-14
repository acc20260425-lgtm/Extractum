// @ts-nocheck
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { isYoutubeSummarySmokeFixtureEnabled } from "$lib/ui/youtube-summary-smoke-fixture";

const files = [
  "src/lib/api/prompt-packs.ts",
  "src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte",
  "src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte",
  "src/lib/components/research-projects/YoutubeSummaryResultView.svelte",
];

function readUiSources() {
  return files.map((file) => readFileSync(file, "utf8")).join("\n");
}

describe("youtube summary smoke fixture guard", () => {
  it("enables smoke fixture only in dev with explicit opt-in flag", () => {
    expect(
      isYoutubeSummarySmokeFixtureEnabled({
        DEV: true,
        VITE_YOUTUBE_SUMMARY_SMOKE_FIXTURE: "1",
      }),
    ).toBe(true);
    expect(
      isYoutubeSummarySmokeFixtureEnabled({
        DEV: true,
        VITE_YOUTUBE_SUMMARY_SMOKE_FIXTURE: "0",
      }),
    ).toBe(false);
    expect(
      isYoutubeSummarySmokeFixtureEnabled({
        DEV: false,
        VITE_YOUTUBE_SUMMARY_SMOKE_FIXTURE: "1",
      }),
    ).toBe(false);
    expect(isYoutubeSummarySmokeFixtureEnabled({ DEV: true })).toBe(false);
  });

  it("wires fixture code through the guard helper and not legacy analysis APIs", () => {
    const source = readUiSources();

    expect(source).toContain("isYoutubeSummarySmokeFixtureEnabled(import.meta.env)");
    expect(source).toContain("VITE_YOUTUBE_SUMMARY_SMOKE_FIXTURE");
    expect(source).toContain("YouTube Summary Smoke Fixture");
    expect(source).not.toContain("analysis_runs");
  });
});
