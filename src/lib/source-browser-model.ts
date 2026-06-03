import type { SourceReaderItem } from "$lib/source-reader-model";
import type { AnalysisSourceGroup } from "$lib/types/analysis";
import type { Source, SourceItem, SourceJobRecord, SourceSubtype, SourceType } from "$lib/types/sources";
import type { YoutubeVideoDetail } from "$lib/types/youtube";

export type SourceBrowserTabId =
  | "timeline"
  | "transcript"
  | "comments"
  | "videos"
  | "sources"
  | "items"
  | "metadata"
  | "activity";

export interface SourceBrowserTab {
  id: SourceBrowserTabId;
  label: string;
}

export type RunSnapshotBrowserKind =
  | "source_group"
  | "telegram_timeline"
  | "youtube_transcript"
  | "generic_items";

export interface RunSnapshotBrowserSubject {
  runId: number;
  scopeType: "source" | "source_group";
  scopeLabel: string;
  readerKind: RunSnapshotBrowserKind;
  sourceType: SourceType | null;
  sourceSubtype: SourceSubtype | null;
}

export interface RunSnapshotBrowserKindInput {
  scopeType: string | null;
  sourceType: string | null;
  sourceSubtype: string | null;
  snapshotReaderItems: Pick<SourceReaderItem, "kind">[];
}

export type SourceBrowserSubject =
  | { kind: "source"; source: Source }
  | { kind: "source_group"; group: AnalysisSourceGroup }
  | { kind: "run_snapshot"; snapshot: RunSnapshotBrowserSubject };

type SourceBrowserSourceLike = Pick<Source, "sourceType" | "sourceSubtype">;
type SourceBrowserModelInput = SourceBrowserSubject | SourceBrowserSourceLike;

export interface SourceItemKindChip {
  kind: string;
  label: string;
  count: number;
}

export interface LoadedSourceItemFilter {
  kind: string | null;
  search: string;
}

export type LoadedSourceItemSort = "newest" | "oldest";

export type CommentsCoverageState =
  | "unknown"
  | "not_synced"
  | "syncing"
  | "failed"
  | "synced_empty"
  | "synced_with_rows";

export interface CommentsCoverageInput {
  items: SourceItem[];
  detail: YoutubeVideoDetail | null;
  jobs: SourceJobRecord[];
  routeError: string | null;
  loadingItems: boolean;
}

export interface LoadedYoutubeCommentReply {
  item: SourceItem;
  parentLoaded: boolean;
}

export interface LoadedYoutubeCommentThread {
  item: SourceItem;
  replies: LoadedYoutubeCommentReply[];
  parentLoaded: boolean;
}

export type LoadedYoutubeCommentSort = LoadedSourceItemSort | "most_liked";

export interface RawJsonPreview {
  preview: string;
  full: string;
  truncated: boolean;
}

const TAB_LABELS: Record<SourceBrowserTabId, string> = {
  timeline: "Timeline",
  transcript: "Transcript",
  comments: "Comments",
  videos: "Videos",
  sources: "Sources",
  items: "Items",
  metadata: "Metadata",
  activity: "Activity",
};

function isSourceBrowserSubject(input: SourceBrowserModelInput): input is SourceBrowserSubject {
  return "kind" in input
    && (input.kind === "source" || input.kind === "source_group" || input.kind === "run_snapshot");
}

function tabRecords(ids: SourceBrowserTabId[]): SourceBrowserTab[] {
  return ids.map((id) => ({ id, label: TAB_LABELS[id] }));
}

function sourceTabIds(source: SourceBrowserSourceLike): SourceBrowserTabId[] {
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") {
    return ["transcript", "comments", "items", "metadata", "activity"];
  }
  if (source.sourceType === "youtube" && source.sourceSubtype === "playlist") {
    return ["videos", "items", "metadata", "activity"];
  }
  if (source.sourceType === "telegram") {
    return ["timeline", "items", "metadata", "activity"];
  }
  return ["items", "metadata", "activity"];
}

function snapshotTabIds(readerKind: RunSnapshotBrowserKind): SourceBrowserTabId[] {
  if (readerKind === "source_group") return ["sources", "items", "metadata"];
  if (readerKind === "telegram_timeline") return ["timeline", "items", "metadata"];
  if (readerKind === "youtube_transcript") return ["transcript", "items", "metadata"];
  return ["items", "metadata"];
}

export function sourceBrowserTabsForSubject(subject: SourceBrowserSubject): SourceBrowserTab[] {
  const ids: SourceBrowserTabId[] = subject.kind === "source_group"
    ? ["sources", "items", "metadata", "activity"]
    : subject.kind === "run_snapshot"
      ? snapshotTabIds(subject.snapshot.readerKind)
    : sourceTabIds(subject.source);

  return tabRecords(ids);
}

export function sourceBrowserTabsForSource(source: SourceBrowserSourceLike): SourceBrowserTab[] {
  return tabRecords(sourceTabIds(source));
}

export function sourceBrowserShellAppliesToSubject(subject: SourceBrowserSubject): boolean {
  if (subject.kind === "source_group" || subject.kind === "run_snapshot") return true;
  return sourceBrowserShellAppliesToSource(subject.source);
}

export function sourceBrowserShellAppliesToSource(source: SourceBrowserSourceLike): boolean {
  return source.sourceType === "telegram"
    || (source.sourceType === "youtube" && (source.sourceSubtype === "video" || source.sourceSubtype === "playlist"));
}

export function smartDefaultSourceBrowserTab(input: SourceBrowserModelInput): SourceBrowserTabId {
  if (isSourceBrowserSubject(input) && input.kind === "source_group") return "sources";
  if (isSourceBrowserSubject(input) && input.kind === "run_snapshot") {
    return snapshotTabIds(input.snapshot.readerKind)[0] ?? "items";
  }
  const source = isSourceBrowserSubject(input) ? input.source : input;
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") return "transcript";
  if (source.sourceType === "youtube" && source.sourceSubtype === "playlist") return "videos";
  if (source.sourceType === "telegram") return "timeline";
  return "items";
}

export function reconcileSourceBrowserTab(
  activeTab: SourceBrowserTabId | null,
  input: SourceBrowserModelInput,
): SourceBrowserTabId {
  const tabs = isSourceBrowserSubject(input)
    ? sourceBrowserTabsForSubject(input)
    : sourceBrowserTabsForSource(input);
  return activeTab && tabs.some((tab) => tab.id === activeTab)
    ? activeTab
    : smartDefaultSourceBrowserTab(input);
}

export function deriveRunSnapshotBrowserKind(input: RunSnapshotBrowserKindInput): RunSnapshotBrowserKind {
  if (input.scopeType === "source_group") return "source_group";
  if (input.snapshotReaderItems.length === 0) return "generic_items";

  const kinds = new Set(input.snapshotReaderItems.map((item) => item.kind));
  if (
    input.sourceType === "youtube"
    && input.sourceSubtype === "video"
    && kinds.size === 1
    && kinds.has("youtube_transcript")
  ) {
    return "youtube_transcript";
  }
  if (
    input.sourceType === "telegram"
    && kinds.size === 1
    && kinds.has("telegram_message")
  ) {
    return "telegram_timeline";
  }
  return "generic_items";
}

export function sourceItemKindChips(items: SourceItem[]): SourceItemKindChip[] {
  const counts = new Map<string, number>();
  for (const item of items) {
    counts.set(item.itemKind, (counts.get(item.itemKind) ?? 0) + 1);
  }
  return Array.from(counts, ([kind, count]) => ({
    kind,
    label: sourceItemKindLabel(kind),
    count,
  }));
}

export function sourceItemPreviewText(item: Pick<SourceItem, "content" | "hasMedia" | "mediaKind">): string {
  if (item.content && item.content.trim().length > 0) return item.content;
  if (item.hasMedia) return `Media-only item${item.mediaKind ? ` (${item.mediaKind})` : ""}. Text was not loaded.`;
  return "No text content loaded.";
}

export function sourceItemContextLine(
  item: Pick<SourceItem, "author" | "externalId" | "hasMedia" | "mediaKind">,
  sourceLabel: string,
): string {
  return [
    item.author,
    sourceLabel,
    item.externalId,
    item.hasMedia ? item.mediaKind ?? "media" : null,
  ].filter(Boolean).join(" - ");
}

export function filterLoadedSourceItems(items: SourceItem[], filter: LoadedSourceItemFilter): SourceItem[] {
  const search = filter.search.trim().toLocaleLowerCase();
  return items.filter((item) => {
    if (filter.kind !== null && item.itemKind !== filter.kind) return false;
    if (!search) return true;
    return [item.content, item.author]
      .some((value) => value?.toLocaleLowerCase().includes(search));
  });
}

export function sortLoadedSourceItems(items: SourceItem[], sort: LoadedSourceItemSort): SourceItem[] {
  const direction = sort === "newest" ? -1 : 1;
  return [...items].sort((left, right) => {
    const timestampOrder = (left.publishedAt - right.publishedAt) * direction;
    return timestampOrder || left.id - right.id;
  });
}

export function commentsCoverageState(input: CommentsCoverageInput): CommentsCoverageState {
  if (input.routeError) return "failed";
  if (input.jobs.some(isActiveYoutubeCommentJob)) return "syncing";
  if (loadedYoutubeCommentItems(input.items).length > 0) return "synced_with_rows";
  if (input.loadingItems) return "unknown";

  const commentStatus = input.detail?.summary.comments.state ?? "unknown";
  if (commentStatus === "failed") return "failed";
  if (commentStatus === "not_synced") return "not_synced";
  if (commentStatus === "synced") {
    return input.detail?.summary.comments.itemCount === 0 ? "synced_empty" : "unknown";
  }
  return "unknown";
}

export function groupLoadedYoutubeComments(items: SourceItem[]): LoadedYoutubeCommentThread[] {
  const comments = loadedYoutubeCommentItems(items);
  const byCommentId = new Map<string, SourceItem>();
  for (const item of comments) {
    const commentId = item.youtubeComment?.commentId;
    if (commentId) byCommentId.set(commentId, item);
  }

  const repliesByParentId = new Map<string, LoadedYoutubeCommentReply[]>();
  const orphans: LoadedYoutubeCommentThread[] = [];

  for (const item of comments) {
    const comment = item.youtubeComment;
    if (!comment?.isReply) continue;
    const parentId = comment.parentCommentId;
    if (parentId && byCommentId.has(parentId)) {
      const replies = repliesByParentId.get(parentId) ?? [];
      replies.push({ item, parentLoaded: true });
      repliesByParentId.set(parentId, replies);
    } else {
      orphans.push({ item, replies: [], parentLoaded: false });
    }
  }

  const roots = comments
    .filter((item) => !item.youtubeComment?.isReply)
    .map((item) => ({
      item,
      replies: repliesByParentId.get(item.youtubeComment?.commentId ?? "") ?? [],
      parentLoaded: true,
    }));

  return [...roots, ...orphans];
}

export function filterLoadedYoutubeComments(items: SourceItem[], search: string): SourceItem[] {
  const query = search.trim().toLocaleLowerCase();
  return loadedYoutubeCommentItems(items).filter((item) => {
    if (!query) return true;
    return [item.content, item.author]
      .some((value) => value?.toLocaleLowerCase().includes(query));
  });
}

export function sortLoadedYoutubeComments(
  items: SourceItem[],
  sort: LoadedYoutubeCommentSort,
): SourceItem[] {
  const comments = loadedYoutubeCommentItems(items);
  if (sort !== "most_liked") return sortLoadedSourceItems(comments, sort);
  return [...comments].sort((left, right) => {
    const leftLikes = left.youtubeComment?.likeCount ?? -1;
    const rightLikes = right.youtubeComment?.likeCount ?? -1;
    return rightLikes - leftLikes || right.publishedAt - left.publishedAt || left.id - right.id;
  });
}

export function formatRawJsonPreview(value: unknown, maxChars: number): RawJsonPreview | null {
  if (value === null || value === undefined) return null;
  try {
    const full = JSON.stringify(value, null, 2);
    if (!full) return null;
    const limit = Math.max(0, maxChars);
    if (full.length <= limit) {
      return { preview: full, full, truncated: false };
    }
    return {
      preview: `${full.slice(0, limit)}\n...`,
      full,
      truncated: true,
    };
  } catch {
    return null;
  }
}

function loadedYoutubeCommentItems(items: SourceItem[]): SourceItem[] {
  return items.filter((item) => item.youtubeComment);
}

function isActiveYoutubeCommentJob(job: SourceJobRecord) {
  const isCommentJob = job.job_type === "youtube_video_comments_sync"
    || job.job_type === "youtube_video_full_sync";
  const isActive = job.status === "queued"
    || job.status === "running"
    || job.status === "cancel_requested";
  return isCommentJob && isActive;
}

function sourceItemKindLabel(kind: string) {
  const [first = "", ...rest] = kind.split("_");
  return [
    first === "youtube" ? "YouTube" : capitalize(first),
    ...rest,
  ].join(" ");
}

function capitalize(value: string) {
  if (!value) return value;
  return value.charAt(0).toLocaleUpperCase() + value.slice(1);
}
