import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  addTelegramSource,
  listSourceForumTopics,
  listSourceItems,
  listSources,
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
        telegram_source_kind: "channel",
        account_id: 2,
        external_id: "123",
        title: "News",
        last_sync_state: 99,
        last_synced_at: 1_700_000,
        is_member: true,
        is_active: true,
        created_at: 1_600_000,
        avatar_data_url: "data:image/jpeg;base64,abc",
      },
    ]);

    await expect(listSources(null)).resolves.toEqual([
      {
        id: 7,
        sourceType: "telegram",
        telegramSourceKind: "channel",
        accountId: 2,
        externalId: "123",
        title: "News",
        lastSyncState: 99,
        lastSyncedAt: 1_700_000,
        isMember: true,
        isActive: true,
        createdAt: 1_600_000,
        avatarDataUrl: "data:image/jpeg;base64,abc",
      },
    ]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_sources", { accountId: null });
  });

  it("adds telegram sources with a request wrapper", async () => {
    invokeMock.mockResolvedValueOnce({
      id: 8,
      source_type: "telegram",
      telegram_source_kind: "supergroup",
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
        expectedKind: "supergroup",
      }),
    ).resolves.toMatchObject({
      id: 8,
      telegramSourceKind: "supergroup",
      accountId: 3,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("add_telegram_source", {
      request: { accountId: 3, sourceRef: "456", expectedKind: "supergroup" },
    });
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
