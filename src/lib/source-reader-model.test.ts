import { describe, expect, it } from "vitest";
import {
  analysisRunMessageToReaderItem,
  formatYoutubeTime,
  groupReaderItemsByDay,
  groupReaderItemsBySource,
  sourceItemToReaderItem,
  youtubeTimestampUrl,
} from "./source-reader-model";
import type { AnalysisRunMessage } from "./types/analysis";
import type { SourceItem } from "./types/sources";

function sourceItem(overrides: Partial<SourceItem> = {}): SourceItem {
  return {
    id: 1,
    sourceId: 10,
    externalId: "100",
    itemKind: "telegram_message",
    author: "Alice",
    publishedAt: 1710000000,
    content: "Hello",
    contentKind: "text_only",
    hasMedia: false,
    mediaKind: null,
    mediaSummary: null,
    mediaFileName: null,
    mediaMimeType: null,
    hasRawData: true,
    forumTopicId: null,
    forumTopicTitle: null,
    forumTopicTopMessageId: null,
    replyToMessageId: null,
    replyToPeerKind: null,
    replyToPeerId: null,
    replyToTopMessageId: null,
    reactionCount: null,
    ...overrides,
  };
}

function runMessage(overrides: Partial<AnalysisRunMessage> = {}): AnalysisRunMessage {
  return {
    item_id: 4,
    source_id: 20,
    external_id: "transcript:v1:en:manual",
    author: "Demo Channel",
    published_at: 1710000200,
    ref: "s20-i4@754000ms",
    content: "Transcript text",
    item_kind: "youtube_transcript",
    source_type: "youtube",
    source_subtype: "video",
    metadata_json: {
      canonical_url: "https://www.youtube.com/watch?v=v1",
      start_ms: 754000,
      end_ms: 756500,
      caption_language: "en",
      caption_track_kind: "manual",
      item_kind: "youtube_transcript",
    },
    ...overrides,
  };
}

describe("source reader model", () => {
  it("normalizes live Telegram items with reply, reaction, topic, and media metadata", () => {
    const item = sourceItem({
      hasMedia: true,
      mediaKind: "photo",
      mediaSummary: "Image 1200x800",
      mediaFileName: "image.jpg",
      mediaMimeType: "image/jpeg",
      forumTopicId: 7,
      forumTopicTitle: "Announcements",
      replyToMessageId: 99,
      replyToTopMessageId: 7,
      reactionCount: 3,
    });

    const readerItem = sourceItemToReaderItem(item, { sourceTitle: "Telegram A" });

    expect(readerItem.kind).toBe("telegram_message");
    expect(readerItem.sourceId).toBe(10);
    expect(readerItem.sourceTitle).toBe("Telegram A");
    expect(readerItem.topicLabel).toBe("Announcements");
    expect(readerItem.replyLabel).toBe("Reply to #99");
    expect(readerItem.reactionLabel).toBe("3 reactions");
    expect(readerItem.mediaCards).toEqual([
      {
        kind: "photo",
        title: "Image",
        summary: "Image 1200x800",
        fileName: "image.jpg",
        mimeType: "image/jpeg",
      },
    ]);
  });

  it("normalizes run snapshot YouTube transcript metadata", () => {
    const readerItem = analysisRunMessageToReaderItem(runMessage(), { sourceTitle: "Video One" });

    expect(readerItem.kind).toBe("youtube_transcript");
    expect(readerItem.ref).toBe("s20-i4@754000ms");
    expect(readerItem.youtubeStartSeconds).toBe(754);
    expect(readerItem.youtubeUrl).toBe("https://www.youtube.com/watch?v=v1&t=754");
    expect(readerItem.captionLabel).toBe("en manual");
  });

  it("groups reader items by source without merging unrelated source material", () => {
    const groups = groupReaderItemsBySource([
      analysisRunMessageToReaderItem(runMessage({ source_id: 2, ref: "s2-i1" }), {
        sourceTitle: "Source 2",
      }),
      analysisRunMessageToReaderItem(runMessage({ source_id: 1, ref: "s1-i1" }), {
        sourceTitle: "Source 1",
      }),
    ]);

    expect(groups.map((group) => group.sourceId)).toEqual([1, 2]);
    expect(groups[0].sourceTitle).toBe("Source 1");
    expect(groups[1].sourceTitle).toBe("Source 2");
  });

  it("groups timeline items by UTC day", () => {
    const groups = groupReaderItemsByDay([
      sourceItemToReaderItem(sourceItem({ id: 1, publishedAt: 1710020000 }), {
        sourceTitle: "A",
      }),
      sourceItemToReaderItem(sourceItem({ id: 2, publishedAt: 1709900000 }), {
        sourceTitle: "A",
      }),
    ]);

    expect(groups).toHaveLength(2);
    expect(groups[0].items[0].id).toBe("live:1");
  });

  it("formats YouTube timestamps and appends canonical time links", () => {
    expect(formatYoutubeTime(0)).toBe("0:00");
    expect(formatYoutubeTime(754)).toBe("12:34");
    expect(formatYoutubeTime(3723)).toBe("1:02:03");
    expect(youtubeTimestampUrl("https://www.youtube.com/watch?v=v1", 754)).toBe(
      "https://www.youtube.com/watch?v=v1&t=754",
    );
    expect(youtubeTimestampUrl("https://www.youtube.com/watch?v=v1&t=1", 754)).toBe(
      "https://www.youtube.com/watch?v=v1&t=754",
    );
  });
});
