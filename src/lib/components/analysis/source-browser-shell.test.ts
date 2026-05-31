import { describe, expect, it } from "vitest";
import rawShellSource from "./source-browser-shell.svelte?raw";

const shellSource = rawShellSource.replace(/\r\n/g, "\n");

function sourceBetween(source: string, start: string, end: string) {
  const startIndex = source.indexOf(start);
  expect(startIndex).toBeGreaterThanOrEqual(0);
  const endIndex = source.indexOf(end, startIndex);
  expect(endIndex).toBeGreaterThan(startIndex);
  return source.slice(startIndex, endIndex);
}

function sourcePropsBlock() {
  return sourceBetween(shellSource, "type Props = {", "  };");
}

function componentCall(tag: string, marker: string = `<${tag}`) {
  const markerIndex = shellSource.indexOf(marker);
  expect(markerIndex).toBeGreaterThanOrEqual(0);
  const openIndex = shellSource.lastIndexOf(`<${tag}`, markerIndex);
  expect(openIndex).toBeGreaterThanOrEqual(0);
  return sourceBetween(shellSource.slice(openIndex), `<${tag}`, "/>");
}

function functionBlock(name: string) {
  return sourceBetween(shellSource, `function ${name}`, "\n  }\n");
}

describe("source browser shell component contract", () => {
  it("uses the subject-aware source browser model and keeps data fetching outside the shell", () => {
    expect(shellSource).toContain("sourceBrowserTabsForSubject");
    expect(shellSource).toContain("reconcileSourceBrowserTab");
    expect(shellSource).toContain("SourceBrowserSubject");
    expect(shellSource).not.toContain("$lib/api/");
    expect(shellSource).not.toContain("invoke(");
  });

  it("requires explicit browser subjects instead of source prop fallback", () => {
    const propsBlock = sourcePropsBlock();

    expect(propsBlock).toContain("subject?: SourceBrowserSubject | null");
    expect(propsBlock).not.toContain("source?: Source | null");
    expect(shellSource).toContain("subject: explicitSubject = null");
    expect(shellSource).toContain("const subject = $derived(explicitSubject);");
    expect(shellSource).not.toContain("explicitSubject ??");
    expect(shellSource).not.toContain('{ kind: "source" as const, source }');
    expect(shellSource).toContain('subject && subject.kind === "source" ? sourceBrowserData : null');
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
    expect(shellSource).toContain("helpDescription");
    expect(shellSource).toContain("sourceLabelForItem");
    expect(shellSource).toContain("Group items are limited to the source rows loaded in this browser session");
  });

  it("renders run snapshot tabs through grouped snapshot data without live activity props", () => {
    expect(shellSource).toContain("<SnapshotGroupSourcesView");
    expect(shellSource).toContain("<SnapshotItemsView");
    expect(shellSource).toContain("<RunSnapshotMetadataView");
    expect(shellSource).toContain("snapshotBrowserData");
    expect(shellSource).toContain('subject.kind === "run_snapshot"');
    expect(shellSource).toContain('activeTab === "transcript"');
    expect(shellSource).toContain("showSyncActions={false}");
    expect(shellSource).not.toContain("SourceReaderHeader");
  });

  it("groups live single-source data behind sourceBrowserData", () => {
    const propsBlock = sourcePropsBlock();

    expect(shellSource).toContain("type SourceBrowserData =");
    expect(propsBlock).toContain("sourceBrowserData?: SourceBrowserData | null");
    expect(propsBlock).toContain("groupBrowserData?: SourceGroupBrowserData | null");
    expect(propsBlock).toContain("snapshotBrowserData?: SnapshotBrowserData | null");
    expect(propsBlock).toContain("loadingItems?: boolean");
    expect(propsBlock).not.toContain("sourceItems: SourceItem[]");
    expect(propsBlock).not.toContain("sourceJobs: SourceJobRecord[]");
    expect(propsBlock).not.toContain("youtubeVideoDetail: YoutubeVideoDetail | null");
    expect(propsBlock).not.toContain("onSyncYoutubeTranscript");
    expect(shellSource).toContain('subject && subject.kind === "source" ? sourceBrowserData : null');
  });

  it("accepts and forwards evidence highlight tokens to every trace-capable child", () => {
    const propsBlock = sourcePropsBlock();

    expect(shellSource).toContain("EvidenceHighlightToken");
    expect(propsBlock).toContain("highlightToken?: EvidenceHighlightToken | null");
    expect(shellSource).toContain("highlightToken = null");

    expect(componentCall("SnapshotGroupSourcesView")).toContain("{highlightToken}");
    expect(componentCall("SnapshotItemsView")).toContain("{highlightToken}");
    expect(componentCall("TelegramTimelineReader", 'ariaLabel="Run snapshot source material timeline"')).toContain("{highlightToken}");
    expect(componentCall("YoutubeTranscriptReader", "snapshotItems={snapshotData?.readerItems ?? []}")).toContain("{highlightToken}");
    expect(componentCall("SourceGroupSourcesView")).toContain("{highlightToken}");
    expect(componentCall("TelegramTimelineReader", "items={sourceData.liveReaderItems}")).toContain("{highlightToken}");
    expect(componentCall("YoutubeTranscriptReader", "segments={sourceData.youtubeTranscriptSegments}")).toContain("{highlightToken}");
    expect(componentCall("UniversalItemsView")).toContain("{highlightToken}");
    expect(componentCall("YoutubeCommentsView")).toContain("{highlightToken}");
  });

  it("auto-selects the tab that renders highlighted source item evidence once per token", () => {
    const tabTarget = functionBlock("highlightTabForToken");

    expect(shellSource).toContain("liveSourceItemRef");
    expect(shellSource).toContain("youtubeSegmentRef");
    expect(shellSource).toContain("let lastHighlightTabTokenId");
    expect(shellSource).toContain("lastHighlightTabTokenId === highlightToken.tokenId");
    expect(shellSource).toContain("lastHighlightTabTokenId = highlightToken.tokenId");
    expect(shellSource).toContain("activeTab = targetTab");

    expect(tabTarget).toContain("sourceData.sourceItems.find((item) => liveSourceItemRef(item) === highlightToken.traceRef)");
    expect(tabTarget).toContain(
      "sourceData.youtubeTranscriptSegments.some((segment) => youtubeSegmentRef(segment) === highlightToken.traceRef)",
    );
    expect(tabTarget).toContain('return tabAvailable("transcript") ? "transcript" : null;');
    expect(tabTarget).toContain('sourceSubject.sourceType === "youtube"');
    expect(tabTarget).toContain('sourceSubject.sourceSubtype === "video"');
    expect(tabTarget).toContain('matchingItem.youtubeComment || matchingItem.itemKind === "youtube_comment"');
    expect(tabTarget).toContain('return tabAvailable("comments") ? "comments" : null;');
    expect(tabTarget).toContain('return tabAvailable("items") ? "items" : null;');
    expect(tabTarget).toContain('groupData.sourceItems.some((item) => liveSourceItemRef(item) === highlightToken.traceRef)');
    expect(tabTarget).toContain("groupData.liveReaderItems.some((item) => item.ref === highlightToken.traceRef)");
    expect(tabTarget).toContain('return tabAvailable("sources") ? "sources" : null;');
  });

  it("auto-selects the tab that renders highlighted run snapshot evidence once per token", () => {
    const tabTarget = functionBlock("highlightTabForToken");

    expect(tabTarget).toContain('subject.kind === "run_snapshot"');
    expect(tabTarget).toContain("snapshotData.readerItems.some((item) => item.ref === highlightToken.traceRef)");
    expect(tabTarget).toContain('snapshotSubject.readerKind === "source_group"');
    expect(tabTarget).toContain('return tabAvailable("sources") ? "sources" : null;');
    expect(tabTarget).toContain('snapshotSubject.readerKind === "telegram_timeline"');
    expect(tabTarget).toContain('return tabAvailable("timeline") ? "timeline" : null;');
    expect(tabTarget).toContain('snapshotSubject.readerKind === "youtube_transcript"');
    expect(tabTarget).toContain('return tabAvailable("transcript") ? "transcript" : null;');
    expect(tabTarget).toContain('return tabAvailable("items") ? "items" : null;');
  });
});
