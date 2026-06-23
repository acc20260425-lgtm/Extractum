import { describe, expect, it } from "vitest";
import type { LibraryCatalogSourceView } from "./library-catalog-model";
import type { TelegramDialogSource, YoutubePreview } from "$lib/types/sources";
import type { YoutubePlaylistDetail, YoutubePlaylistItemDetail } from "$lib/types/youtube";
import {
  YOUTUBE_PLAYLIST_IMPORT_LIMIT,
  buildPlaylistImportRows,
  classifyYoutubeImportInput,
  existingYoutubeSmartImportSource,
  libraryYoutubePlaylistSources,
  playlistSelectionLimitMessage,
  selectedAddablePlaylistRows,
  telegramDialogAddInput,
} from "./library-add-source-model";

function source(overrides: Partial<LibraryCatalogSourceView> = {}): LibraryCatalogSourceView {
  return {
    id: "source:1",
    sourceId: 1,
    provider: "youtube",
    sourceSubtype: "playlist",
    title: "Playlist",
    subtitle: null,
    typeLabel: "YouTube / Playlist",
    status: "active",
    statusDetail: null,
    projectCount: 0,
    itemCount: 0,
    itemCountLabel: "0 items",
    addedAtLabel: "01/01/2026, 10:00 AM",
    lastSyncedLabel: "Never",
    canonicalUrl: "https://www.youtube.com/playlist?list=PL1",
    externalId: "PL1",
    createdAt: null,
    lastSyncedAt: null,
    youtube: {
      video_form: null,
      duration_seconds: null,
      playlist_video_count: 2,
      channel_title: "Channel",
      availability_status: "available",
    },
    telegram: null,
    ...overrides,
  };
}

function playlistItem(overrides: Partial<YoutubePlaylistItemDetail> = {}): YoutubePlaylistItemDetail {
  return {
    position: 1,
    videoId: "video-1",
    videoSourceId: null,
    title: "Video 1",
    canonicalUrl: "https://www.youtube.com/watch?v=video-1",
    thumbnailUrl: null,
    durationSeconds: 120,
    publishedAt: null,
    availabilityStatus: "available",
    isRemovedFromPlaylist: false,
    captions: {
      state: "not_synced",
      itemCount: 0,
      segmentCount: 0,
      lastSyncedAt: null,
      label: "Not synced",
    },
    comments: {
      state: "not_synced",
      itemCount: 0,
      segmentCount: 0,
      lastSyncedAt: null,
      label: "Not synced",
    },
    ...overrides,
  };
}

function playlistDetail(items: YoutubePlaylistItemDetail[]): YoutubePlaylistDetail {
  return {
    summary: {
      sourceId: 10,
      sourceSubtype: "playlist",
      title: "Playlist",
      channelTitle: "Channel",
      channelHandle: "@channel",
      canonicalUrl: "https://www.youtube.com/playlist?list=PL1",
      thumbnailUrl: null,
      durationSeconds: null,
      publishedAt: null,
      availabilityStatus: "available",
      videoCount: items.length,
      linkedVideoCount: items.filter((item) => item.videoSourceId !== null).length,
      unavailableCount: 0,
      captions: {
        state: "not_synced",
        itemCount: 0,
        segmentCount: 0,
        lastSyncedAt: null,
        label: "Not synced",
      },
      comments: {
        state: "not_synced",
        itemCount: 0,
        segmentCount: 0,
        lastSyncedAt: null,
        label: "Not synced",
      },
    },
    items,
  };
}

function youtubePreview(overrides: Partial<YoutubePreview> = {}): YoutubePreview {
  return {
    kind: "video",
    externalId: "video-1",
    canonicalUrl: "https://www.youtube.com/watch?v=video-1",
    title: "Video 1",
    channelTitle: "Channel",
    channelId: null,
    channelHandle: null,
    channelUrl: null,
    thumbnailUrl: null,
    durationSeconds: 120,
    publishedAt: null,
    playlistVideoCount: null,
    captionsEstimate: null,
    availabilityStatus: "available",
    warnings: [],
    ...overrides,
  };
}

describe("library add source model", () => {
  it("classifies YouTube video playlist and channel inputs", () => {
    expect(classifyYoutubeImportInput("https://www.youtube.com/watch?v=abc123")).toMatchObject({
      kind: "video",
      supported: true,
    });
    expect(classifyYoutubeImportInput("youtube.com/watch?v=abc123")).toMatchObject({
      kind: "video",
      supported: true,
      normalizedUrl: "https://youtube.com/watch?v=abc123",
    });
    expect(classifyYoutubeImportInput("https://youtu.be/abc123")).toMatchObject({
      kind: "video",
      supported: true,
    });
    expect(classifyYoutubeImportInput("youtu.be/abc123")).toMatchObject({
      kind: "video",
      supported: true,
      normalizedUrl: "https://youtu.be/abc123",
    });
    expect(classifyYoutubeImportInput("https://www.youtube.com/playlist?list=PLabc")).toMatchObject({
      kind: "playlist",
      supported: true,
    });
    expect(classifyYoutubeImportInput("https://www.youtube.com/@tech_trends")).toEqual({
      provider: "youtube",
      kind: "channel",
      supported: false,
      reason: "YouTube channel import is not supported yet.",
    });
    expect(classifyYoutubeImportInput("https://www.youtube.com/channel/UCabc")).toMatchObject({
      provider: "youtube",
      kind: "channel",
      supported: false,
    });
  });

  it("classifies unsupported and Telegram input without switching provider", () => {
    expect(classifyYoutubeImportInput("https://t.me/ai_news")).toEqual({
      provider: "telegram",
      kind: "unsupported",
      supported: false,
      reason: "Telegram sources are added from the Telegram tab.",
    });
    expect(classifyYoutubeImportInput("https://example.com/post")).toEqual({
      provider: "unknown",
      kind: "unsupported",
      supported: false,
      reason: "Enter a YouTube video or playlist URL.",
    });
  });

  it("filters full Library catalog to YouTube playlists only", () => {
    expect(
      libraryYoutubePlaylistSources([
        source({ sourceId: 1, sourceSubtype: "playlist" }),
        source({ sourceId: 2, sourceSubtype: "video" }),
        source({ sourceId: 3, provider: "telegram", sourceSubtype: "channel" }),
      ]).map((row) => row.sourceId),
    ).toEqual([1]);
  });

  it("finds existing YouTube smart import sources by preview kind and external id", () => {
    const existingVideo = source({
      sourceId: 7,
      sourceSubtype: "video",
      externalId: "video-1",
      title: "Existing Video",
    });
    const existingPlaylist = source({
      sourceId: 8,
      sourceSubtype: "playlist",
      externalId: "PL1",
      title: "Existing Playlist",
    });

    expect(
      existingYoutubeSmartImportSource(
        [existingPlaylist, existingVideo],
        youtubePreview({ kind: "video", externalId: "video-1" }),
      )?.sourceId,
    ).toBe(7);
    expect(
      existingYoutubeSmartImportSource(
        [existingVideo, existingPlaylist],
        youtubePreview({ kind: "playlist", externalId: "PL1" }),
      )?.sourceId,
    ).toBe(8);
    expect(
      existingYoutubeSmartImportSource(
        [existingVideo],
        youtubePreview({ kind: "playlist", externalId: "video-1" }),
      ),
    ).toBeNull();
  });

  it("marks playlist rows that cannot be added", () => {
    const rows = buildPlaylistImportRows(
      playlistDetail([
        playlistItem({ videoId: "ready" }),
        playlistItem({ videoId: "linked", videoSourceId: 22 }),
        playlistItem({ videoId: "missing-url", canonicalUrl: null }),
      ]),
    );

    expect(rows).toMatchObject([
      { id: "ready", addable: true, disabledReason: null },
      { id: "linked", addable: false, disabledReason: "Already in Library" },
      { id: "missing-url", addable: false, disabledReason: "Missing video URL" },
    ]);
  });

  it("returns only selected addable playlist rows and enforces the MVP selection limit", () => {
    const rows = buildPlaylistImportRows(
      playlistDetail([
        playlistItem({ videoId: "a" }),
        playlistItem({ videoId: "b", videoSourceId: 10 }),
        playlistItem({ videoId: "c" }),
      ]),
    );

    expect(selectedAddablePlaylistRows(rows, new Set(["a", "b", "c"])).map((row) => row.id)).toEqual([
      "a",
      "c",
    ]);
    expect(playlistSelectionLimitMessage(YOUTUBE_PLAYLIST_IMPORT_LIMIT + 1)).toBe(
      `Select ${YOUTUBE_PLAYLIST_IMPORT_LIMIT} or fewer videos for one import run.`,
    );
    expect(playlistSelectionLimitMessage(YOUTUBE_PLAYLIST_IMPORT_LIMIT)).toBeNull();
  });

  it("builds Telegram dialog add input from the selected dialog", () => {
    const dialog: TelegramDialogSource = {
      id: 456,
      title: "Forum",
      username: "forum",
      sourceSubtype: "supergroup",
      isMember: true,
      photoDataUrl: null,
    };

    expect(telegramDialogAddInput(3, dialog)).toEqual({
      accountId: 3,
      sourceRef: "456",
      expectedSubtype: "supergroup",
    });
  });
});
