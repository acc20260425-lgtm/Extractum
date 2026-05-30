import { describe, expect, it } from "vitest";
import shellSource from "./source-browser-shell.svelte?raw";

describe("source browser shell component contract", () => {
  it("uses the subject-aware source browser model and keeps data fetching outside the shell", () => {
    expect(shellSource).toContain("sourceBrowserTabsForSubject");
    expect(shellSource).toContain("reconcileSourceBrowserTab");
    expect(shellSource).toContain("SourceBrowserSubject");
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

  it("renders source group tabs through route-owned props", () => {
    expect(shellSource).toContain("<SourceGroupSourcesView");
    expect(shellSource).toContain("<SourceGroupMetadataView");
    expect(shellSource).toContain("<SourceGroupActivityView");
    expect(shellSource).toContain('activeTab === "sources"');
    expect(shellSource).toContain("groupBrowserData");
    expect(shellSource).toContain("liveReaderItems");
    expect(shellSource).toContain("sourceItems");
    expect(shellSource).toContain("sourceLabelForItem");
    expect(shellSource).toContain("Group items are limited to the source rows loaded in this browser session");
  });
});
