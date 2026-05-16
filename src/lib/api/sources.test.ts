import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  addYoutubeSource,
  addTelegramSource,
  listYoutubeTranscriptSegments,
  listSourceForumTopics,
  listSourceItems,
  listSources,
  listTelegramSources,
  previewYoutubeSource,
  saveSyncSettings,
} from "./sources";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("sources api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("lists sources with typed arguments and maps source fields", async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 7,
        source_type: "telegram",
        source_subtype: "channel",
        account_id: 2,
        external_id: "123",
        title: "News",
        last_sync_state: 99,
        last_synced_at: 1_700_000,
        is_member: true,
        is_active: true,
        created_at: 1_600_000,
        telegram_username: "newsroom",
        avatar_data_url: "data:image/jpeg;base64,abc",
      },
    ]);

    await expect(listSources(null)).resolves.toEqual([
      {
        id: 7,
        sourceType: "telegram",
        sourceSubtype: "channel",
        accountId: 2,
        externalId: "123",
        title: "News",
        lastSyncState: 99,
        lastSyncedAt: 1_700_000,
        isMember: true,
        isActive: true,
        createdAt: 1_600_000,
        telegramUsername: "newsroom",
        avatarDataUrl: "data:image/jpeg;base64,abc",
      },
    ]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_sources", { accountId: null });
  });

  it("adds telegram sources with expectedSubtype", async () => {
    invokeMock.mockResolvedValueOnce({
      id: 8,
      source_type: "telegram",
      source_subtype: "supergroup",
      account_id: 3,
      external_id: "456",
      title: "Forum",
      last_sync_state: null,
      last_synced_at: null,
      is_member: true,
      is_active: true,
      created_at: 1_600_001,
      avatar_data_url: null,
    });

    await expect(
      addTelegramSource({
        accountId: 3,
        sourceRef: "456",
        expectedSubtype: "supergroup",
      }),
    ).resolves.toMatchObject({
      id: 8,
      sourceSubtype: "supergroup",
      accountId: 3,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("add_telegram_source", {
      request: { accountId: 3, sourceRef: "456", expectedSubtype: "supergroup" },
    });
  });

  it("maps live telegram dialogs with sourceSubtype", async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 123,
        title: "Forum",
        username: "forum",
        source_subtype: "supergroup",
        is_member: true,
        photo_data_url: null,
      },
    ]);

    await expect(listTelegramSources(3)).resolves.toEqual([
      {
        id: 123,
        title: "Forum",
        username: "forum",
        sourceSubtype: "supergroup",
        isMember: true,
        photoDataUrl: null,
      },
    ]);
  });

  it("previews youtube sources with a url argument", async () => {
    invokeMock.mockResolvedValueOnce({
      kind: "video",
      external_id: "abc123",
      canonical_url: "https://www.youtube.com/watch?v=abc123",
      title: "Demo",
      channel_title: "Channel",
      channel_id: "UC1",
      channel_handle: "@channel",
      channel_url: "https://www.youtube.com/@channel",
      thumbnail_url: null,
      duration_seconds: 120,
      published_at: "2026-05-01",
      playlist_video_count: null,
      captions_estimate: null,
      availability_status: "available",
      warnings: [],
    });

    await expect(previewYoutubeSource("https://youtu.be/abc123")).resolves.toMatchObject({
      kind: "video",
      externalId: "abc123",
      canonicalUrl: "https://www.youtube.com/watch?v=abc123",
    });
    expect(invokeMock).toHaveBeenLastCalledWith("preview_youtube_source", {
      url: "https://youtu.be/abc123",
    });
  });

  it("adds youtube sources with a url argument", async () => {
    invokeMock.mockResolvedValueOnce({
      id: 10,
      source_type: "youtube",
      source_subtype: "video",
      account_id: null,
      external_id: "abc123",
      title: "Demo",
      last_sync_state: null,
      last_synced_at: null,
      is_member: false,
      is_active: true,
      created_at: 1,
      avatar_data_url: null,
    });

    await expect(addYoutubeSource("https://youtu.be/abc123")).resolves.toMatchObject({
      id: 10,
      sourceType: "youtube",
      externalId: "abc123",
    });
    expect(invokeMock).toHaveBeenLastCalledWith("add_youtube_source", {
      url: "https://youtu.be/abc123",
    });
  });

  it("maps non-Telegram source fields without legacy Telegram fields", async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 10,
        source_type: "youtube",
        source_subtype: "video",
        account_id: null,
        external_id: "dQw4w9WgXcQ",
        title: "Demo video",
        last_sync_state: null,
        last_synced_at: null,
        is_member: false,
        is_active: true,
        created_at: 1_700_500,
        avatar_data_url: null,
      },
    ]);

    await expect(listSources(null)).resolves.toEqual([
      {
        id: 10,
        sourceType: "youtube",
        sourceSubtype: "video",
        accountId: null,
        externalId: "dQw4w9WgXcQ",
        title: "Demo video",
        lastSyncState: null,
        lastSyncedAt: null,
        isMember: false,
        isActive: true,
        createdAt: 1_700_500,
        telegramUsername: null,
        avatarDataUrl: null,
      },
    ]);
  });

  it("saves sync settings with camel case request fields", async () => {
    invokeMock.mockResolvedValueOnce({
      initial_sync_mode: "recent_days",
      initial_sync_value: 14,
    });

    await expect(
      saveSyncSettings({
        initialSyncMode: "recent_days",
        initialSyncValue: 14,
      }),
    ).resolves.toEqual({
      initialSyncMode: "recent_days",
      initialSyncValue: 14,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("save_sync_settings", {
      settings: { initialSyncMode: "recent_days", initialSyncValue: 14 },
    });
  });

  it("lists source items with camel case topic filters", async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 1,
        source_id: 7,
        external_id: "100",
        item_kind: "telegram_message",
        author: "alice",
        published_at: 1_700_100,
        content: "hello",
        content_kind: "text_only",
        has_media: false,
        media_kind: null,
        media_summary: null,
        media_file_name: null,
        media_mime_type: null,
        has_raw_data: true,
        forum_topic_id: 200,
        forum_topic_title: "Announcements",
        forum_topic_top_message_id: 700,
        reply_to_msg_id: 44,
        reply_to_peer_kind: "channel",
        reply_to_peer_id: "99",
        reply_to_top_id: 12,
        reaction_count: 5,
      },
    ]);

    await expect(
      listSourceItems({
        sourceId: 7,
        limit: 120,
        beforePublishedAt: null,
        topicFilter: { kind: "topic", topicId: 200 },
      }),
    ).resolves.toEqual([
      {
        id: 1,
        sourceId: 7,
        externalId: "100",
        itemKind: "telegram_message",
        author: "alice",
        publishedAt: 1_700_100,
        content: "hello",
        contentKind: "text_only",
        hasMedia: false,
        mediaKind: null,
        mediaSummary: null,
        mediaFileName: null,
        mediaMimeType: null,
        hasRawData: true,
        forumTopicId: 200,
        forumTopicTitle: "Announcements",
        forumTopicTopMessageId: 700,
        replyToMessageId: 44,
        replyToPeerKind: "channel",
        replyToPeerId: "99",
        replyToTopMessageId: 12,
        reactionCount: 5,
      },
    ]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_source_items", {
      request: {
        sourceId: 7,
        limit: 120,
        beforePublishedAt: null,
        topicFilter: { kind: "topic", topicId: 200 },
      },
    });
  });

  it("passes a selected source item id for focused source item paging", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await expect(
      listSourceItems({
        sourceId: 7,
        limit: 50,
        beforePublishedAt: null,
        topicFilter: null,
        aroundItemId: 99,
      }),
    ).resolves.toEqual([]);

    expect(invokeMock).toHaveBeenLastCalledWith("list_source_items", {
      request: {
        sourceId: 7,
        limit: 50,
        beforePublishedAt: null,
        topicFilter: null,
        aroundItemId: 99,
      },
    });
  });

  it("wraps paged YouTube transcript segment loading", async () => {
    invokeMock.mockResolvedValueOnce({
      segments: [
        {
          id: 9,
          source_id: 20,
          item_id: 4,
          segment_index: 2,
          start_ms: 754000,
          end_ms: 756500,
          text: "Transcript text",
          caption_language: "en",
          caption_track_kind: "manual",
          is_auto_generated: false,
        },
      ],
      next_cursor: {
        startMs: 754000,
        segmentId: 9,
      },
      has_more: true,
    });

    await expect(listYoutubeTranscriptSegments({
      sourceId: 20,
      after: { startMs: 700000, segmentId: 8 },
      limit: 50,
      searchQuery: "text",
    })).resolves.toEqual({
      segments: [
        {
          id: 9,
          sourceId: 20,
          itemId: 4,
          segmentIndex: 2,
          startMs: 754000,
          endMs: 756500,
          text: "Transcript text",
          captionLanguage: "en",
          captionTrackKind: "manual",
          isAutoGenerated: false,
        },
      ],
      nextCursor: { startMs: 754000, segmentId: 9 },
      hasMore: true,
    });

    expect(invokeMock).toHaveBeenLastCalledWith("list_youtube_transcript_segments", {
      request: {
        sourceId: 20,
        after: { startMs: 700000, segmentId: 8 },
        limit: 50,
        searchQuery: "text",
      },
    });
  });

  it("passes a selected timestamp for focused YouTube transcript paging", async () => {
    invokeMock.mockResolvedValueOnce({
      segments: [],
      next_cursor: null,
      has_more: false,
    });

    await expect(
      listYoutubeTranscriptSegments({
        sourceId: 20,
        after: null,
        limit: 50,
        searchQuery: null,
        aroundStartMs: 754000,
      }),
    ).resolves.toEqual({
      segments: [],
      nextCursor: null,
      hasMore: false,
    });

    expect(invokeMock).toHaveBeenLastCalledWith("list_youtube_transcript_segments", {
      request: {
        sourceId: 20,
        after: null,
        limit: 50,
        searchQuery: null,
        aroundStartMs: 754000,
      },
    });
  });

  it("maps camel-case YouTube transcript cursors returned by Tauri", async () => {
    invokeMock.mockResolvedValueOnce({
      segments: [],
      next_cursor: {
        startMs: 754000,
        segmentId: 9,
      },
      has_more: true,
    });

    await expect(
      listYoutubeTranscriptSegments({
        sourceId: 20,
        after: null,
        limit: 50,
        searchQuery: null,
      }),
    ).resolves.toMatchObject({
      nextCursor: { startMs: 754000, segmentId: 9 },
      hasMore: true,
    });
  });

  it("maps source forum topic fields", async () => {
    invokeMock.mockResolvedValueOnce([
      {
        kind: "topic",
        key: "topic:200",
        title: "Announcements",
        message_count: 3,
        topic_id: 200,
        top_message_id: 700,
        icon_color: 1,
        icon_emoji_id: 2,
        is_closed: false,
        is_pinned: true,
        is_hidden: false,
        is_deleted: false,
        sort_order: 4,
      },
    ]);

    await expect(listSourceForumTopics(7)).resolves.toEqual([
      {
        kind: "topic",
        key: "topic:200",
        title: "Announcements",
        messageCount: 3,
        topicId: 200,
        topMessageId: 700,
        iconColor: 1,
        iconEmojiId: 2,
        isClosed: false,
        isPinned: true,
        isHidden: false,
        isDeleted: false,
        sortOrder: 4,
      },
    ]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_source_forum_topics", { sourceId: 7 });
  });
});
