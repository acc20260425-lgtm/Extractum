import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import SourceGroupSourcesView from "./source-group-sources-view.svelte";
import type { SourceReaderItem } from "$lib/source-reader-model";

afterEach(cleanup);

function readerItem(overrides: Partial<SourceReaderItem> = {}): SourceReaderItem {
  return {
    id: "reader-1",
    sourceId: 1,
    sourceTitle: "Research channel",
    externalId: "message-1",
    ref: "source:1:item:1",
    kind: "telegram_message",
    author: "Ada",
    publishedAt: 1_700_000_000,
    content: "Grouped Telegram evidence",
    topicLabel: null,
    replyLabel: null,
    reactionLabel: null,
    mediaCards: [],
    youtubeStartSeconds: null,
    youtubeEndSeconds: null,
    youtubeUrl: null,
    captionLabel: null,
    historyScope: "current",
    historyScopeLabel: null,
    isMigratedHistory: false,
    selected: false,
    ...overrides,
  };
}

describe("analysis redesign final safety contract", () => {
  it("keeps source groups grouped by source instead of merged into one pseudo-chat", async () => {
    const onLoadMoreSource = vi.fn();
    render(SourceGroupSourcesView, {
      props: {
        items: [
          readerItem(),
          readerItem({
            id: "reader-2",
            sourceId: 2,
            sourceTitle: "Research video",
            externalId: "segment-2",
            ref: "source:2:item:2@12000ms",
            kind: "youtube_transcript",
            content: "Grouped YouTube evidence",
            youtubeStartSeconds: 12,
            youtubeEndSeconds: 18,
          }),
        ],
        selectedGroupSourceId: null,
        loading: false,
        hasMoreBySource: { 1: true, 2: true },
        youtubeDetailsBySource: {},
        formatTimestamp: (value) => value === null ? "Never" : `time:${value}`,
        onLoadMoreSource,
      },
    });

    expect(screen.getByRole("region", { name: "Source group sources" })).toBeTruthy();
    const telegram = screen.getByRole("region", { name: "Research channel" });
    expect(telegram).toBeTruthy();
    const youtube = screen.getByRole("region", { name: "Research video" });
    expect(youtube).toBeTruthy();
    expect(within(telegram).getByText("Grouped Telegram evidence")).toBeTruthy();
    expect(within(youtube).getByText("Grouped YouTube evidence")).toBeTruthy();
    expect(within(telegram).getByRole("region", { name: "Source material timeline" })).toBeTruthy();
    expect(within(youtube).getByRole("region", { name: "YouTube transcript reader" })).toBeTruthy();
    expect(screen.getAllByText("1 loaded items")).toHaveLength(2);
    const loadMore = screen.getAllByRole("button", { name: /Load (older messages|more transcript)/ });
    expect(loadMore).toHaveLength(2);
    for (const button of loadMore) await fireEvent.click(button);
    expect(onLoadMoreSource.mock.calls).toEqual([[1], [2]]);
  });
});
