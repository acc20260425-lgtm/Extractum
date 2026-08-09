import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import UniversalItemsView from "./universal-items-view.svelte";
import type { SourceItem } from "$lib/types/sources";

afterEach(cleanup);

function sourceItem(overrides: Partial<SourceItem> = {}): SourceItem {
  return {
    id: 1,
    sourceId: 1,
    externalId: "message-1",
    itemKind: "telegram_message",
    author: "Ada",
    publishedAt: 1_700_000_000,
    content: "Readable evidence paragraph",
    contentKind: "text",
    hasMedia: false,
    mediaKind: null,
    mediaSummary: null,
    mediaFileName: null,
    mediaMimeType: null,
    hasRawData: false,
    forumTopicId: null,
    forumTopicTitle: null,
    forumTopicTopMessageId: null,
    replyToMessageId: null,
    replyToPeerKind: null,
    replyToPeerId: null,
    replyToTopMessageId: null,
    reactionCount: null,
    historyScope: "current",
    isMigratedHistory: false,
    migrationDomain: null,
    historyScopeLabel: "Current supergroup history",
    pageCursor: "cursor-1",
    ...overrides,
  };
}

describe("analysis priority UX contract", () => {
  it("turns loaded items into a reader instead of a raw dump", async () => {
    render(UniversalItemsView, {
      props: {
        items: [
          sourceItem(),
          sourceItem({ id: 2, externalId: "media-2", itemKind: "forum_post", author: "Grace", content: null, contentKind: "media", hasMedia: true, mediaKind: "document" }),
        ],
        loading: false,
        hasMore: true,
        formatTimestamp: (value) => value === null ? "Never" : `time:${value}`,
        onLoadMore: vi.fn(),
      },
    });

    expect(screen.getByRole("region", { name: "Universal source items" })).toBeTruthy();
    expect(screen.getByText("Readable evidence paragraph")).toBeTruthy();
    expect(screen.getByText("Unknown item kind")).toBeTruthy();
    expect(screen.getByText("Media-only item (document). Text was not loaded.")).toBeTruthy();
    await fireEvent.input(screen.getByRole("searchbox", { name: "Search loaded items" }), { target: { value: "Ada" } });
    expect(screen.queryByText("Media-only item (document). Text was not loaded.")).toBeNull();
  });
});
