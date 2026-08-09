import { describe, expect, it, vi } from "vitest";
import { reportLaunchDisabledReason } from "./analysis-state";
import { resolveYoutubeDetailForSource } from "./analysis-youtube-source-runtime";
import {
  detailErrorForYoutubeSource,
  youtubeContentStatusLine,
  youtubeDetailBoundaryState,
  youtubeCorpusOptionViews,
  youtubeEvidenceSectionLabels,
  youtubePlaylistProblemView,
  youtubeProviderHeaderSummary,
  youtubeReaderSearchLabel,
} from "./youtube-source-view-model";

const youtubeSource = (overrides: Record<string, unknown> = {}) => ({
  id: 7,
  sourceType: "youtube",
  sourceSubtype: "video",
  title: "Research video",
  externalId: "video-7",
  ...overrides,
});

const synced = { state: "synced" as const, itemCount: 2, segmentCount: 3, lastSyncedAt: 100, label: "Synced" };
const detail = () => ({
  summary: {
    sourceId: 7, sourceSubtype: "video", title: "Rendered video", channelTitle: "Channel",
    channelHandle: "@channel", canonicalUrl: "https://youtu.be/video-7", thumbnailUrl: null,
    durationSeconds: 65, publishedAt: 90, availabilityStatus: "available", videoCount: null,
    linkedVideoCount: null, unavailableCount: null, captions: synced, comments: synced,
  },
  sourceMetadata: { description: "Description" },
  playlistMemberships: [],
});

describe("analysis youtube source specialization", () => {
  it("keeps youtube detail errors scoped to the selected source", async () => {
    const getYoutubeVideoDetail = vi.fn().mockRejectedValue(new Error("runtime missing"));
    const getYoutubePlaylistDetail = vi.fn();
    const outcome = await resolveYoutubeDetailForSource(youtubeSource() as never, { getYoutubeVideoDetail, getYoutubePlaylistDetail });
    expect(getYoutubeVideoDetail).toHaveBeenCalledWith(7);
    expect(getYoutubePlaylistDetail).not.toHaveBeenCalled();
    expect(outcome.kind).toBe("error");
    expect(outcome.videoDetail).toBeNull();
    expect(outcome.playlistDetail).toBeNull();
    expect(outcome.error).toMatchObject({ sourceId: 7, sourceSubtype: "video" });
    expect(detailErrorForYoutubeSource(outcome.error, youtubeSource() as never)).toContain("runtime missing");
  });

  it("uses the scoped youtube detail problem in report preflight copy", () => {
    const reason = reportLaunchDisabledReason({
      analysisScope: "single_source", selectedSourceId: "7", selectedGroupId: "", selectedTemplateId: "1",
      periodFrom: "1", periodTo: "2", outputLanguage: "en", profileId: "profile", modelOverride: "",
      youtubeCorpusMode: "transcript_only", includeMigratedHistory: false, llmProfiles: [{ profile_id: "profile", api_key_configured: true }],
      activeLlmProfile: "profile", currentSourceMetric: { item_count: 1 }, currentSource: youtubeSource(),
      currentGroup: null, sourceCatalog: [youtubeSource()], sourceSyncDisabledReason: () => null,
      youtubeDetailProblemReason: "Video metadata failed to load.",
    } as never);
    expect(reason).toBe("Video metadata failed to load.");
    expect(detailErrorForYoutubeSource({ sourceId: 7, sourceSubtype: "video", message: "Video metadata failed to load." }, youtubeSource() as never)).toBe(reason);
    expect(detailErrorForYoutubeSource({ sourceId: 8, sourceSubtype: "video", message: "Other" }, youtubeSource() as never)).toBeNull();
  });

  it("threads youtube detail error into report setup and source browser", () => {
    const error = { sourceId: 7, sourceSubtype: "video", message: "Metadata unavailable" };
    const state = youtubeDetailBoundaryState(youtubeSource() as never, null, error);
    const unrelated = youtubeDetailBoundaryState(youtubeSource() as never, null, { ...error, sourceId: 8 });
    expect(state.reportSetupProblem).toBe("Metadata unavailable");
    expect(state.sourceBrowserError?.message).toBe("Metadata unavailable");
    expect(state.activity.badgeLabel).toBe("attention");
    expect(state.activity.metadataStatus).toBe("Detail not loaded");
    expect(unrelated.reportSetupProblem).toBeNull();
    expect(unrelated.sourceBrowserError).toBeNull();
  });

  it("promotes youtube corpus into a provider-specific report decision block", () => {
    const options = youtubeCorpusOptionViews(detail() as never);
    expect(options.map((option) => option.value)).toEqual(["transcript_only", "transcript_description", "transcript_description_comments"]);
    expect(options[0]?.available).toBe(true);
    expect(options[1]?.countLabel).toContain("description");
    expect(options[2]?.evidenceWarning).toBe("Audience comments are user-generated evidence.");
  });

  it("renders invalid playlists as problem states instead of empty playlists", () => {
    const view = youtubePlaylistProblemView("playlist id is invalid");
    expect(youtubePlaylistProblemView(null)).toBeNull();
    expect(view?.heading).toBe("Playlist metadata needs attention");
    expect(view?.message).toBe("This is not an empty playlist. playlist id is invalid");
    expect(view?.retryLabel).toBe("Retry playlist sync");
  });

  it("uses compact youtube status copy in transcript and comments readers", () => {
    const header = youtubeProviderHeaderSummary(youtubeSource() as never, detail() as never, (value) => `t:${value}`);
    const status = youtubeContentStatusLine("captions", synced, (value) => `t:${value}`);
    expect(header.title).toBe("Rendered video");
    expect(header.durationLabel).toBe("1:05");
    expect(status.countLabel).toBe("3 segments");
    expect(status.lastSyncedLabel).toBe("Synced t:100");
    expect(youtubeReaderSearchLabel("comments")).toBe("Search comments");
  });

  it("renders youtube items as evidence inventory and activity as provider steps", () => {
    const labels = youtubeEvidenceSectionLabels();
    expect(labels.inventory).toBe("Evidence inventory");
    expect(labels.activity).toBe("YouTube provider steps");
    expect(labels.inventoryRole).toBe("YouTube evidence inventory");
    expect(labels.activityRole).toBe("YouTube source activity");
    expect(youtubeReaderSearchLabel("transcript")).toBe("Search transcript");
    expect(youtubeReaderSearchLabel("comments")).toBe("Search comments");
  });
});
