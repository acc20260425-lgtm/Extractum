import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

function functionSlice(name: string, nextName: string) {
  const start = analysisPageSource.indexOf(`  ${name}`);
  const end = analysisPageSource.indexOf(`\n  ${nextName}`, start + 1);

  expect(start, `${name} should exist`).toBeGreaterThan(-1);
  expect(end, `${nextName} should follow ${name}`).toBeGreaterThan(start);

  return analysisPageSource.slice(start, end);
}

function expectOrder(source: string, first: string, second: string, message: string) {
  const firstIndex = source.indexOf(first);
  const secondIndex = source.indexOf(second);

  expect(firstIndex, `${message}: missing first marker`).toBeGreaterThan(-1);
  expect(secondIndex, `${message}: missing second marker`).toBeGreaterThan(-1);
  expect(firstIndex, message).toBeLessThan(secondIndex);
}

describe("analysis source reader route wiring", () => {
  it("loads live source group pages per member without closing the opened run", () => {
    expect(analysisPageSource).toContain("groupLiveItemsBySource");
    expect(analysisPageSource).toContain("loadLiveGroupSourcePage");
    expect(analysisPageSource).toContain("selectedGroupSourceId");
    expect(analysisPageSource).toContain("function changeSelectedGroupSourceId");
    expect(analysisPageSource).toContain("void loadLiveGroupSourcePage(sourceId)");
    expect(analysisPageSource).toContain("onChangeSelectedGroupSourceId={changeSelectedGroupSourceId}");
    expect(analysisPageSource).not.toContain("clearCurrentRunForWorkspaceSwitch(sourceId");
  });

  it("keeps source browser tab state out of the analysis route", () => {
    expect(analysisPageSource).not.toContain("activeSourceBrowserTab");
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
    expect(analysisPageSource).toContain("{sourceItemsHasMore}");
    expect(analysisPageSource).toContain("onLoadMoreSourceItems");
    expect(analysisPageSource).toContain("onLoadLiveGroupSourcePage");
    expect(analysisPageSource).toContain("onChangeSelectedGroupSourceId");
    expect(analysisPageSource).toContain("onChangeSelectedTopicKey={(value) => void changeSelectedTopicKey(value)}");
  });

  it("pages live single-source material without changing the selected topic filter", () => {
    expect(analysisPageSource).toContain("sourceItemsCursor");
    expect(analysisPageSource).toContain("sourceItemsBeforePublishedAt");
    expect(analysisPageSource).toContain("sourceItemsHasMore");
    expect(analysisPageSource).toContain("async function loadMoreSourceItems");
    expect(analysisPageSource).toContain("beforeCursor: isTelegramSource ? sourceItemsCursor : null");
    expect(analysisPageSource).toContain("beforePublishedAt: isTelegramSource ? null : sourceItemsBeforePublishedAt");
    expect(analysisPageSource).toContain("topicFilter: source && sourceCapabilities(source).hasTopics ? currentTopicFilter() : null");
  });

  it("wires Telegram history scope changes through opaque backend cursors", () => {
    expect(analysisPageSource).toContain("telegramHistoryScope");
    expect(analysisPageSource).toContain("function changeTelegramHistoryScope");
    expect(analysisPageSource).toContain("historyScope: isTelegramSource ? telegramHistoryScope : \"current\"");
    expect(analysisPageSource).toContain("pageCursor");
    expect(analysisPageSource).toContain("onChangeTelegramHistoryScope={changeTelegramHistoryScope}");
    expect(analysisPageSource).not.toContain("JSON.parse(sourceItemsCursor");
    expect(analysisPageSource).not.toContain("atob(sourceItemsCursor");
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

  it("loads live source pages around the selected trace before source readers scroll", () => {
    const focusedLoad = functionSlice(
      "async function loadSourcePageAroundTrace",
      "async function showSelectedTraceInSource",
    );

    expect(analysisPageSource).not.toContain("function sourceReaderFocusInput");
    expect(focusedLoad).toContain("decision,");
    expect(focusedLoad).toContain("trace,");
    expect(focusedLoad).toContain("requestId,");
    expect(focusedLoad).toContain("canonicalRef,");
    expect(focusedLoad).toContain("sourceScope,");
    expect(focusedLoad).toContain("const liveTarget = focusedLiveSourceTargetForTrace(trace);");
    expect(focusedLoad).toContain(
      'const aroundItemId = liveTarget.kind === "source_item" ? liveTarget.aroundItemId : trace.item_id;',
    );
    expect(focusedLoad).toContain(
      'const aroundStartMs = liveTarget.kind === "youtube_transcript" ? liveTarget.aroundStartMs : null;',
    );
    expect(focusedLoad).toContain("aroundItemId,");
    expect(focusedLoad).toContain("aroundStartMs,");
    expect(focusedLoad).toContain("aroundRef: trace.ref");
  });

  it("uses loaded source data matching to gate focused highlight creation", () => {
    const focusedLoad = functionSlice(
      "async function loadSourcePageAroundTrace",
      "async function showSelectedTraceInSource",
    );

    expect(analysisPageSource).toContain("pendingFocusMatchesCurrent,");
    expect(analysisPageSource).toContain("loadedSourceDataContainsTraceRef,");
    expect(analysisPageSource).toContain("analysisRunMessageToReaderItem");
    expect(focusedLoad).not.toContain("void requestId;");
    expect(focusedLoad).not.toContain("void canonicalRef;");
    expect(focusedLoad).not.toContain("void sourceScope;");
    expect(focusedLoad).toContain("const tokenId = `${requestId}:highlight`;");
    expect(focusedLoad).toContain("transientSourceHighlight = {");
    expect(focusedLoad).toContain("scheduleSourceHighlightClear(tokenId);");
    expect(focusedLoad).toContain("createdAt: Date.now(),");
    expect(focusedLoad).toContain('status = "Selected evidence was not found in the loaded source window.";');
    expectOrder(
      focusedLoad,
      "loadedSourceDataContainsTraceRef(",
      "transientSourceHighlight = {",
      "loaded-data match must happen before highlight creation",
    );
    expectOrder(
      focusedLoad,
      "transientSourceHighlight = {",
      "scheduleSourceHighlightClear(tokenId);",
      "highlight must be assigned before scheduling its clear timer",
    );
  });

  it("matches snapshot, live item, and transcript focused load data without fabricating rows", () => {
    const focusedLoad = functionSlice(
      "async function loadSourcePageAroundTrace",
      "async function showSelectedTraceInSource",
    );

    expect(focusedLoad).toContain("analysisRunMessageToReaderItem(message, {");
    expect(focusedLoad).toContain("sourceTitle: snapshotSourceTitle(message.source_id),");
    expect(focusedLoad).toContain('{ kind: "snapshot", items: snapshotItems }');
    expect(focusedLoad).toContain('{ kind: "source_items", items }');
    expect(focusedLoad).toContain('{ kind: "youtube_transcript", segments: page.segments }');
    expect(focusedLoad).toContain("canonicalRef,");
    expect(focusedLoad).toContain("sourceScope,");
    expect(focusedLoad).not.toContain("canonicalRef, content:");
    expect(focusedLoad).not.toContain("ref: canonicalRef");
  });

  it("uses the generic single-source item path for source-item targets before transcript loading", () => {
    const focusedLoad = functionSlice(
      "async function loadSourcePageAroundTrace",
      "async function showSelectedTraceInSource",
    );
    const sourceItemBranchStart = focusedLoad.indexOf('if (liveTarget.kind === "source_item")');
    const transcriptBranchStart = focusedLoad.indexOf(
      'if (source.sourceType === "youtube" && source.sourceSubtype === "video")',
    );

    expect(sourceItemBranchStart, "focused live source-item branch should exist").toBeGreaterThan(-1);
    expect(transcriptBranchStart, "focused transcript branch should exist").toBeGreaterThan(-1);
    expect(sourceItemBranchStart, "source-item targets must not fall through to transcript loading").toBeLessThan(
      transcriptBranchStart,
    );

    const sourceItemBranch = focusedLoad.slice(sourceItemBranchStart, transcriptBranchStart);
    expect(sourceItemBranch).toContain("const items = await listSourceItems({");
    expect(sourceItemBranch).toContain("aroundItemId");
    expect(sourceItemBranch).toContain('historyScope: source.sourceType === "telegram" ? telegramHistoryScope : "current",');
    expect(sourceItemBranch).toContain('{ kind: "source_items", items }');
    expect(sourceItemBranch).toContain("applySourceItemsPage(items, false);");
    expect(sourceItemBranch).not.toContain("listYoutubeTranscriptSegments(");
  });

  it("clears pending focus on focused load success, absence, and request-owned failure through route helpers", () => {
    const focusedLoad = functionSlice(
      "async function loadSourcePageAroundTrace",
      "async function showSelectedTraceInSource",
    );
    const catchBranch = focusedLoad.slice(
      focusedLoad.indexOf("} catch (error)"),
      focusedLoad.indexOf("} finally"),
    );

    expect(focusedLoad).toContain("pendingEvidenceSourceFocus = null;");
    expect(focusedLoad).toContain("clearSourceHighlight();");
    expect(focusedLoad).toContain("return handleFocusedSourceLoadResult(");
    expect(catchBranch).toContain("failFocusedSourceLoad(");
    expect(catchBranch).not.toContain("pendingEvidenceSourceFocus = null;");
    expect(catchBranch).not.toContain("clearSourceHighlight();");
  });
});
