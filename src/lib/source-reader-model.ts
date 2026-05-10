import type { AnalysisRunMessage } from "$lib/types/analysis";
import type { SourceItem, YoutubeTranscriptSegment } from "$lib/types/sources";

export type SourceReaderBasis = "live_source" | "run_snapshot";
export type SourceReaderKind =
  | "telegram_message"
  | "youtube_transcript"
  | "youtube_comment"
  | "youtube_description"
  | "generic_item";

export interface SourceReaderMediaCard {
  kind: string;
  title: string;
  summary: string | null;
  fileName: string | null;
  mimeType: string | null;
}

export interface SourceReaderItem {
  id: string;
  sourceId: number;
  sourceTitle: string;
  externalId: string;
  ref: string | null;
  kind: SourceReaderKind;
  author: string | null;
  publishedAt: number;
  content: string;
  topicLabel: string | null;
  replyLabel: string | null;
  reactionLabel: string | null;
  mediaCards: SourceReaderMediaCard[];
  youtubeStartSeconds: number | null;
  youtubeEndSeconds: number | null;
  youtubeUrl: string | null;
  captionLabel: string | null;
  selected: boolean;
}

export interface SourceReaderDayGroup {
  key: string;
  label: string;
  items: SourceReaderItem[];
}

export interface SourceReaderSourceGroup {
  sourceId: number;
  sourceTitle: string;
  items: SourceReaderItem[];
}

export function sourceItemToReaderItem(
  item: SourceItem,
  {
    sourceTitle,
    selectedTraceRef = null,
  }: { sourceTitle: string; selectedTraceRef?: string | null },
): SourceReaderItem {
  const ref = null;
  return {
    id: `live:${item.id}`,
    sourceId: item.sourceId,
    sourceTitle,
    externalId: item.externalId,
    ref,
    kind: itemKind(item.itemKind),
    author: item.author,
    publishedAt: item.publishedAt,
    content: item.content ?? (item.hasMedia ? "Media-only message" : ""),
    topicLabel: item.forumTopicTitle,
    replyLabel: replyLabel(item),
    reactionLabel: reactionLabel(item.reactionCount),
    mediaCards: mediaCardsFromSourceItem(item),
    youtubeStartSeconds: null,
    youtubeEndSeconds: null,
    youtubeUrl: null,
    captionLabel: null,
    selected: selectedTraceRef !== null && ref === selectedTraceRef,
  };
}

export function analysisRunMessageToReaderItem(
  message: AnalysisRunMessage,
  {
    sourceTitle,
    selectedTraceRef = null,
  }: { sourceTitle: string; selectedTraceRef?: string | null },
): SourceReaderItem {
  const metadata = metadataObject(message.metadata_json);
  const startSeconds = millisecondsToSeconds(numberValue(metadata.start_ms));
  const endSeconds = millisecondsToSeconds(numberValue(metadata.end_ms));
  const canonicalUrl = stringValue(metadata.canonical_url);
  const captionLanguage = stringValue(metadata.caption_language);
  const captionTrackKind = stringValue(metadata.caption_track_kind);
  return {
    id: `snapshot:${message.ref}`,
    sourceId: message.source_id,
    sourceTitle,
    externalId: message.external_id,
    ref: message.ref,
    kind: itemKind(message.item_kind),
    author: message.author,
    publishedAt: message.published_at,
    content: message.content || "No text content captured for this snapshot row.",
    topicLabel: stringValue(metadata.forum_topic_title),
    replyLabel: null,
    reactionLabel: null,
    mediaCards: mediaCardsFromMetadata(metadata),
    youtubeStartSeconds: startSeconds,
    youtubeEndSeconds: endSeconds,
    youtubeUrl:
      canonicalUrl && startSeconds !== null
        ? youtubeTimestampUrl(canonicalUrl, startSeconds)
        : canonicalUrl,
    captionLabel: [captionLanguage, captionTrackKind].filter(Boolean).join(" ") || null,
    selected: selectedTraceRef !== null && message.ref === selectedTraceRef,
  };
}

export function youtubeSegmentToReaderItem(
  segment: YoutubeTranscriptSegment,
  {
    sourceTitle,
    canonicalUrl,
    selectedTraceRef = null,
  }: { sourceTitle: string; canonicalUrl: string | null; selectedTraceRef?: string | null },
): SourceReaderItem {
  const startSeconds = millisecondsToSeconds(segment.startMs);
  const endSeconds = millisecondsToSeconds(segment.endMs);
  return {
    id: `youtube-segment:${segment.id}`,
    sourceId: segment.sourceId,
    sourceTitle,
    externalId: `segment:${segment.segmentIndex}`,
    ref: null,
    kind: "youtube_transcript",
    author: null,
    publishedAt: startSeconds ?? 0,
    content: segment.text,
    topicLabel: null,
    replyLabel: null,
    reactionLabel: null,
    mediaCards: [],
    youtubeStartSeconds: startSeconds,
    youtubeEndSeconds: endSeconds,
    youtubeUrl: canonicalUrl && startSeconds !== null ? youtubeTimestampUrl(canonicalUrl, startSeconds) : null,
    captionLabel: [segment.captionLanguage, segment.captionTrackKind].filter(Boolean).join(" ") || null,
    selected: selectedTraceRef !== null && false,
  };
}

export function groupReaderItemsByDay(items: SourceReaderItem[]): SourceReaderDayGroup[] {
  const grouped = new Map<string, SourceReaderItem[]>();
  for (const item of [...items].sort(compareReaderItems)) {
    const key = new Date(item.publishedAt * 1000).toISOString().slice(0, 10);
    grouped.set(key, [...(grouped.get(key) ?? []), item]);
  }
  return [...grouped.entries()].map(([key, groupedItems]) => ({
    key,
    label: key,
    items: groupedItems,
  }));
}

export function groupReaderItemsBySource(items: SourceReaderItem[]): SourceReaderSourceGroup[] {
  const grouped = new Map<number, SourceReaderItem[]>();
  for (const item of [...items].sort(compareReaderItems)) {
    grouped.set(item.sourceId, [...(grouped.get(item.sourceId) ?? []), item]);
  }
  return [...grouped.entries()]
    .sort(([left], [right]) => left - right)
    .map(([sourceId, groupedItems]) => ({
      sourceId,
      sourceTitle: groupedItems[0]?.sourceTitle ?? `Source ${sourceId}`,
      items: groupedItems,
    }));
}

export function formatYoutubeTime(totalSeconds: number) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

export function youtubeTimestampUrl(canonicalUrl: string, seconds: number) {
  const url = new URL(canonicalUrl);
  url.searchParams.set("t", String(seconds));
  return url.toString();
}

function compareReaderItems(left: SourceReaderItem, right: SourceReaderItem) {
  return right.publishedAt - left.publishedAt || left.sourceId - right.sourceId || left.id.localeCompare(right.id);
}

function itemKind(value: string | null): SourceReaderKind {
  if (value === "telegram_message") return "telegram_message";
  if (value === "youtube_transcript") return "youtube_transcript";
  if (value === "youtube_comment") return "youtube_comment";
  if (value === "youtube_description") return "youtube_description";
  return "generic_item";
}

function replyLabel(item: Pick<SourceItem, "replyToMessageId" | "replyToTopMessageId">) {
  if (item.replyToMessageId !== null) return `Reply to #${item.replyToMessageId}`;
  if (item.replyToTopMessageId !== null) return `Thread #${item.replyToTopMessageId}`;
  return null;
}

function reactionLabel(value: number | null) {
  if (value === null || value <= 0) return null;
  return value === 1 ? "1 reaction" : `${value} reactions`;
}

function mediaCardsFromSourceItem(item: SourceItem): SourceReaderMediaCard[] {
  if (!item.hasMedia || !item.mediaKind) return [];
  return [
    {
      kind: item.mediaKind,
      title: mediaTitle(item.mediaKind),
      summary: item.mediaSummary,
      fileName: item.mediaFileName,
      mimeType: item.mediaMimeType,
    },
  ];
}

function mediaCardsFromMetadata(metadata: Record<string, unknown>): SourceReaderMediaCard[] {
  const mediaKind = stringValue(metadata.media_kind);
  if (!mediaKind) return [];
  return [
    {
      kind: mediaKind,
      title: mediaTitle(mediaKind),
      summary: stringValue(metadata.media_summary),
      fileName: stringValue(metadata.media_file_name),
      mimeType: stringValue(metadata.media_mime_type),
    },
  ];
}

function mediaTitle(kind: string) {
  if (kind.includes("photo") || kind.includes("image")) return "Image";
  if (kind.includes("video")) return "Video";
  if (kind.includes("document")) return "Document";
  return kind.replaceAll("_", " ");
}

function metadataObject(value: unknown): Record<string, unknown> {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return {};
}

function stringValue(value: unknown) {
  return typeof value === "string" && value.trim() ? value : null;
}

function numberValue(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function millisecondsToSeconds(value: number | null) {
  return value === null ? null : Math.floor(value / 1000);
}
