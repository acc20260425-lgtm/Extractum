import { describe, expect, it } from "vitest";
import {
  detailErrorForYoutubeSource,
  formatYoutubeDuration,
  youtubeContentStatusLine,
  youtubeCorpusOptionViews,
  youtubeProviderHeaderSummary,
  type YoutubeDetailErrorState,
} from "./youtube-source-view-model";
import type { Source } from "$lib/types/sources";
import type { YoutubeVideoDetail } from "$lib/types/youtube";

function youtubeSource(overrides: Partial<Source> = {}): Source {
  return {
    id: 66,
    sourceType: "youtube",
    sourceSubtype: "video",
    accountId: null,
    externalId: "2ZMbW3Qiv6U",
    title: "Gemma video",
    lastSyncState: null,
    lastSyncedAt: 1_800_000_000,
    isMember: false,
    isActive: true,
    createdAt: 1_779_916_800,
    telegramUsername: null,
    avatarDataUrl: null,
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
    ...overrides,
  };
}

function videoDetail(overrides: Partial<YoutubeVideoDetail["summary"]> = {}): YoutubeVideoDetail {
  return {
    summary: {
      sourceId: 66,
      sourceSubtype: "video",
      title: "Gemma 4 Desktop Coder by Google",
      channelTitle: "AI Stack Engineer",
      channelHandle: "@AIStackEngineer",
      canonicalUrl: "https://www.youtube.com/watch?v=2ZMbW3Qiv6U",
      thumbnailUrl: null,
      durationSeconds: 581,
      publishedAt: 1_779_916_800,
      availabilityStatus: "available",
      videoCount: null,
      linkedVideoCount: null,
      unavailableCount: null,
      captions: {
        state: "synced",
        itemCount: 1,
        segmentCount: 239,
        lastSyncedAt: 1_800_000_000,
        label: "Captions synced",
      },
      comments: {
        state: "synced",
        itemCount: 43,
        segmentCount: 0,
        lastSyncedAt: 1_800_000_100,
        label: "Comments synced",
      },
      ...overrides,
    },
    sourceMetadata: {
      sourceId: 66,
      videoId: "2ZMbW3Qiv6U",
      canonicalUrl: "https://www.youtube.com/watch?v=2ZMbW3Qiv6U",
      title: "Gemma 4 Desktop Coder by Google",
      channelTitle: "AI Stack Engineer",
      channelId: "UCdemo",
      channelHandle: "@AIStackEngineer",
      channelUrl: "https://www.youtube.com/@AIStackEngineer",
      authorDisplay: "AI Stack Engineer",
      publishedAt: 1_779_916_800,
      durationSeconds: 581,
      description: "Demo description",
      thumbnailUrl: null,
      viewCount: 24_355,
      likeCount: 527,
      commentCount: 43,
      category: "Science & Technology",
      videoForm: "regular",
      availabilityStatus: "available",
      captionLanguageOverride: null,
      rawMetadataVersion: 1,
      rawMetadataJson: null,
    },
    playlistMemberships: [],
  };
}

describe("youtube source view model", () => {
  it("formats youtube durations for videos and playlists", () => {
    expect(formatYoutubeDuration(null)).toBeNull();
    expect(formatYoutubeDuration(581)).toBe("9:41");
    expect(formatYoutubeDuration(8064)).toBe("2:14:24");
  });

  it("keeps detail errors scoped to the selected source", () => {
    const error: YoutubeDetailErrorState = {
      sourceId: 32,
      sourceSubtype: "playlist",
      message: "Source 32 has missing or invalid typed YouTube playlist metadata",
    };

    expect(detailErrorForYoutubeSource(error, youtubeSource({ id: 32, sourceSubtype: "playlist" })))
      .toBe(error.message);
    expect(detailErrorForYoutubeSource(error, youtubeSource({ id: 66 }))).toBeNull();
    expect(detailErrorForYoutubeSource(null, youtubeSource({ id: 32 }))).toBeNull();
  });

  it("builds compact content status lines without repeated prefixes", () => {
    const detail = videoDetail();

    expect(youtubeContentStatusLine("comments", detail.summary.comments, () => "2026-05-16")).toEqual({
      state: "synced",
      label: "Comments synced",
      countLabel: "43 comments",
      lastSyncedLabel: "Synced 2026-05-16",
    });
  });

  it("builds a provider header summary from typed YouTube detail", () => {
    const header = youtubeProviderHeaderSummary(youtubeSource(), videoDetail(), (value) => String(value));

    expect(header).toMatchObject({
      sourceKind: "video",
      title: "Gemma 4 Desktop Coder by Google",
      channelLabel: "@AIStackEngineer",
      durationLabel: "9:41",
      availabilityLabel: "available",
      captionsCountLabel: "239 segments",
      commentsCountLabel: "43 comments",
    });
  });

  it("marks corpus options with counts availability and audience warnings", () => {
    const options = youtubeCorpusOptionViews(videoDetail());

    expect(options.map((option) => option.value)).toEqual([
      "transcript_only",
      "transcript_description",
      "transcript_description_comments",
    ]);
    expect(options[0]).toMatchObject({ available: true, countLabel: "239 segments" });
    expect(options[1]).toMatchObject({ available: true, countLabel: "239 segments + description" });
    expect(options[2]).toMatchObject({
      available: true,
      countLabel: "239 segments + description + 43 comments",
      evidenceWarning: "Audience comments are user-generated evidence.",
    });
  });
});
