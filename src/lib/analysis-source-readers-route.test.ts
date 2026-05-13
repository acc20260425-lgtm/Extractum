import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis source reader route wiring", () => {
  it("loads live source group pages per member without closing the opened run", () => {
    expect(analysisPageSource).toContain("groupLiveItemsBySource");
    expect(analysisPageSource).toContain("loadLiveGroupSourcePage");
    expect(analysisPageSource).toContain("selectedGroupSourceId");
    expect(analysisPageSource).toContain("onChangeSelectedGroupSourceId={(sourceId) =>");
    expect(analysisPageSource).not.toContain("clearCurrentRunForWorkspaceSwitch(sourceId");
  });

  it("loads YouTube transcript segments through a paged API", () => {
    expect(analysisPageSource).toContain("listYoutubeTranscriptSegments");
    expect(analysisPageSource).toContain("youtubeTranscriptSegments");
    expect(analysisPageSource).toContain("youtubeTranscriptCursor");
    expect(analysisPageSource).toContain("loadYoutubeTranscriptFirstPage");
    expect(analysisPageSource).toContain("loadMoreYoutubeTranscriptSegments");
  });

  it("passes source reader props into ReportSourceSurface", () => {
    expect(analysisPageSource).toContain("{youtubeTranscriptSegments}");
    expect(analysisPageSource).toContain("{groupLiveItemsBySource}");
    expect(analysisPageSource).toContain("{selectedGroupSourceId}");
    expect(analysisPageSource).toContain("{sourceTopics}");
    expect(analysisPageSource).toContain("{loadingSourceTopics}");
    expect(analysisPageSource).toContain("{selectedTopicKey}");
    expect(analysisPageSource).toContain("showTopicSelector={shouldShowTopicSelector()}");
    expect(analysisPageSource).toContain("onLoadMoreYoutubeTranscriptSegments");
    expect(analysisPageSource).toContain("onLoadLiveGroupSourcePage");
    expect(analysisPageSource).toContain("onChangeSelectedGroupSourceId");
    expect(analysisPageSource).toContain("onChangeSelectedTopicKey={(value) => void changeSelectedTopicKey(value)}");
  });

  it("supports run snapshot source filtering through the snapshot-only API", () => {
    expect(analysisPageSource).toContain("selectedSnapshotSourceId");
    expect(analysisPageSource).toContain("sourceId: selectedSnapshotSourceId");
    expect(analysisPageSource).not.toContain("listSourceItems({ runId");
  });

  it("includes related playlist-video jobs in selected video source activity", () => {
    expect(analysisPageSource).toContain("related_source_id === source.id");
    expect(analysisPageSource).toContain("seenSourceJobIds");
    expect(analysisPageSource).toContain("right.started_at - left.started_at");
  });
});
