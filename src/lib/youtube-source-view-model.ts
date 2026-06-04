import type { YoutubeCorpusMode } from "$lib/types/analysis";
import type { Source } from "$lib/types/sources";
import type {
  YoutubeContentStatus,
  YoutubePlaylistDetail,
  YoutubeVideoDetail,
} from "$lib/types/youtube";

export type YoutubeDetailErrorState = {
  sourceId: number;
  sourceSubtype: string | null;
  message: string;
} | null;

export type YoutubeProviderHeaderSummary = {
  sourceKind: "video" | "playlist";
  title: string;
  channelLabel: string;
  durationLabel: string | null;
  publishedLabel: string | null;
  canonicalUrl: string | null;
  thumbnailUrl: string | null;
  availabilityLabel: string;
  captionsLabel: string;
  captionsCountLabel: string;
  commentsLabel: string;
  commentsCountLabel: string;
};

export type YoutubeContentStatusLine = {
  state: YoutubeContentStatus["state"];
  label: string;
  countLabel: string;
  lastSyncedLabel: string | null;
};

export type YoutubeCorpusOptionView = {
  value: YoutubeCorpusMode;
  label: string;
  description: string;
  countLabel: string;
  available: boolean;
  disabledReason: string | null;
  evidenceWarning: string | null;
};

type YoutubeDetail = YoutubeVideoDetail | YoutubePlaylistDetail | null;

export function formatYoutubeDuration(value: number | null | undefined) {
  if (value === null || value === undefined) return null;
  const hours = Math.floor(value / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  const seconds = value % 60;
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

export function detailErrorForYoutubeSource(error: YoutubeDetailErrorState, source: Pick<Source, "id"> | null) {
  if (!error || !source || error.sourceId !== source.id) return null;
  return error.message;
}

export function youtubeContentStatusLine(
  kind: "captions" | "comments",
  status: YoutubeContentStatus,
  formatTimestamp: (value: number | null) => string,
): YoutubeContentStatusLine {
  const unit = kind === "captions"
    ? status.segmentCount === 1 ? "segment" : "segments"
    : status.itemCount === 1 ? "comment" : "comments";
  const count = kind === "captions" ? status.segmentCount : status.itemCount;
  return {
    state: status.state,
    label: status.label,
    countLabel: `${count} ${unit}`,
    lastSyncedLabel: status.lastSyncedAt === null ? null : `Synced ${formatTimestamp(status.lastSyncedAt)}`,
  };
}

export function youtubeProviderHeaderSummary(
  source: Pick<Source, "sourceSubtype" | "title" | "externalId">,
  detail: YoutubeDetail,
  formatTimestamp: (value: number | null) => string,
): YoutubeProviderHeaderSummary {
  const summary = detail?.summary ?? null;
  const title = summary?.title ?? source.title ?? source.externalId;
  const captions = summary?.captions ?? null;
  const comments = summary?.comments ?? null;
  return {
    sourceKind: source.sourceSubtype === "playlist" ? "playlist" : "video",
    title,
    channelLabel: summary?.channelHandle ?? summary?.channelTitle ?? "YouTube",
    durationLabel: formatYoutubeDuration(summary?.durationSeconds),
    publishedLabel: summary?.publishedAt === null || summary?.publishedAt === undefined
      ? null
      : formatTimestamp(summary.publishedAt),
    canonicalUrl: summary?.canonicalUrl ?? null,
    thumbnailUrl: summary?.thumbnailUrl ?? null,
    availabilityLabel: (summary?.availabilityStatus ?? "unknown").replaceAll("_", " "),
    captionsLabel: captions?.label ?? "Captions unknown",
    captionsCountLabel: captions ? youtubeContentStatusLine("captions", captions, formatTimestamp).countLabel : "0 segments",
    commentsLabel: comments?.label ?? "Comments unknown",
    commentsCountLabel: comments ? youtubeContentStatusLine("comments", comments, formatTimestamp).countLabel : "0 comments",
  };
}

export function youtubeCorpusOptionViews(detail: YoutubeVideoDetail | null): YoutubeCorpusOptionView[] {
  const captions = detail?.summary.captions ?? null;
  const description = detail?.sourceMetadata.description?.trim() ?? "";
  const comments = detail?.summary.comments ?? null;
  const transcriptAvailable = (captions?.segmentCount ?? 0) > 0;
  const descriptionAvailable = description.length > 0;
  const commentsAvailable = (comments?.itemCount ?? 0) > 0;
  const segmentLabel = `${captions?.segmentCount ?? 0} ${(captions?.segmentCount ?? 0) === 1 ? "segment" : "segments"}`;

  return [
    {
      value: "transcript_only",
      label: "Transcript",
      description: "Use only timestamp-backed video transcript evidence.",
      countLabel: segmentLabel,
      available: transcriptAvailable,
      disabledReason: transcriptAvailable ? null : "Transcript segments are not loaded.",
      evidenceWarning: null,
    },
    {
      value: "transcript_description",
      label: "Transcript + description",
      description: "Use transcript evidence plus author-provided description context.",
      countLabel: `${segmentLabel} + description`,
      available: transcriptAvailable && descriptionAvailable,
      disabledReason: !transcriptAvailable
        ? "Transcript segments are not loaded."
        : descriptionAvailable ? null : "Video description is not loaded.",
      evidenceWarning: null,
    },
    {
      value: "transcript_description_comments",
      label: "Transcript + description + comments",
      description: "Use transcript, description, and audience reactions.",
      countLabel: `${segmentLabel} + description + ${comments?.itemCount ?? 0} ${(comments?.itemCount ?? 0) === 1 ? "comment" : "comments"}`,
      available: transcriptAvailable && descriptionAvailable && commentsAvailable,
      disabledReason: !transcriptAvailable
        ? "Transcript segments are not loaded."
        : !descriptionAvailable
          ? "Video description is not loaded."
          : commentsAvailable ? null : "Comments are not loaded.",
      evidenceWarning: "Audience comments are user-generated evidence.",
    },
  ];
}
