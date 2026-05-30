import { describe, expect, it } from "vitest";
import shellSource from "./source-browser-shell.svelte?raw";

describe("source browser shell component contract", () => {
  it("uses the source browser model and keeps data fetching outside the shell", () => {
    expect(shellSource).toContain("sourceBrowserTabsForSource");
    expect(shellSource).toContain("reconcileSourceBrowserTab");
    expect(shellSource).not.toContain("$lib/api/");
    expect(shellSource).not.toContain("invoke(");
  });

  it("renders provider readers and playlist videos through route-owned props", () => {
    expect(shellSource).toContain("<TelegramTimelineReader");
    expect(shellSource).toContain("<YoutubeTranscriptReader");
    expect(shellSource).toContain("<YoutubePlaylistVideosView");
    expect(shellSource).toContain("timeline");
    expect(shellSource).toContain("transcript");
    expect(shellSource).toContain("videos");
    expect(shellSource).toContain("youtubePlaylistDetail");
  });
});
