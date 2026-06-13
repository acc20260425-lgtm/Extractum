import { describe, expect, it, vi } from "vitest";
import type { Source } from "$lib/types/sources";
import type { PlaylistImportRow } from "./library-add-source-model";
import { addSelectedYoutubePlaylistVideos } from "./library-add-source-workflow";

function source(id: number): Source {
  return {
    id,
    sourceType: "youtube",
    sourceSubtype: "video",
    accountId: null,
    externalId: `video-${id}`,
    title: `Video ${id}`,
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: false,
    isActive: true,
    createdAt: 1_700_000_000,
    telegramUsername: null,
    avatarDataUrl: null,
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
  };
}

function row(overrides: Partial<PlaylistImportRow> = {}): PlaylistImportRow {
  const videoId = overrides.id ?? "video-1";
  return {
    id: videoId,
    addable: true,
    disabledReason: null,
    item: {
      position: 1,
      videoId,
      videoSourceId: null,
      title: `Video ${videoId}`,
      canonicalUrl: `https://www.youtube.com/watch?v=${videoId}`,
      thumbnailUrl: null,
      durationSeconds: null,
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
    },
    ...overrides,
  };
}

describe("library add source workflow", () => {
  it("adds selected playlist videos sequentially", async () => {
    const addYoutubeSource = vi.fn()
      .mockResolvedValueOnce(source(101))
      .mockResolvedValueOnce(source(102));

    const summary = await addSelectedYoutubePlaylistVideos({
      rows: [row({ id: "a" }), row({ id: "b" })],
      addYoutubeSource,
      formatError: (_action, error) => String(error),
    });

    expect(addYoutubeSource).toHaveBeenNthCalledWith(1, "https://www.youtube.com/watch?v=a");
    expect(addYoutubeSource).toHaveBeenNthCalledWith(2, "https://www.youtube.com/watch?v=b");
    expect(summary.added).toBe(2);
    expect(summary.failed).toBe(0);
    expect(summary.results.map((result) => result.sourceId)).toEqual([101, 102]);
  });

  it("skips rows that become non-addable before execution", async () => {
    const addYoutubeSource = vi.fn();

    const summary = await addSelectedYoutubePlaylistVideos({
      rows: [
        row({
          id: "linked",
          addable: false,
          disabledReason: "Already in Library",
        }),
      ],
      addYoutubeSource,
      formatError: (_action, error) => String(error),
    });

    expect(addYoutubeSource).not.toHaveBeenCalled();
    expect(summary.skipped).toBe(1);
    expect(summary.results[0]).toMatchObject({
      id: "linked",
      status: "skipped",
      message: "Already in Library",
    });
  });

  it("reports partial failure without stopping later rows", async () => {
    const addYoutubeSource = vi.fn()
      .mockResolvedValueOnce(source(101))
      .mockRejectedValueOnce(new Error("network down"))
      .mockResolvedValueOnce(source(103));

    const summary = await addSelectedYoutubePlaylistVideos({
      rows: [row({ id: "a" }), row({ id: "b" }), row({ id: "c" })],
      addYoutubeSource,
      formatError: (_action, error) => error instanceof Error ? error.message : String(error),
    });

    expect(addYoutubeSource).toHaveBeenCalledTimes(3);
    expect(summary.added).toBe(2);
    expect(summary.failed).toBe(1);
    expect(summary.results[1]).toMatchObject({
      id: "b",
      status: "failed",
      sourceId: null,
      message: "network down",
    });
  });
});
