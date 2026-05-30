import type { Source, SourceItem, SourceJobRecord } from "$lib/types/sources";
import type { YoutubeVideoDetail } from "$lib/types/youtube";

export type SourceBrowserTabId =
  | "timeline"
  | "transcript"
  | "comments"
  | "items"
  | "metadata"
  | "activity";

export interface SourceBrowserTab {
  id: SourceBrowserTabId;
  label: string;
}

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

const TAB_LABELS: Record<SourceBrowserTabId, string> = {
  timeline: "Timeline",
  transcript: "Transcript",
  comments: "Comments",
  items: "Items",
  metadata: "Metadata",
  activity: "Activity",
};

export function sourceBrowserTabsForSource(source: Pick<Source, "sourceType" | "sourceSubtype">): SourceBrowserTab[] {
  const ids: SourceBrowserTabId[] =
    source.sourceType === "youtube" && source.sourceSubtype === "video"
      ? ["transcript", "comments", "items", "metadata", "activity"]
      : source.sourceType === "telegram"
        ? ["timeline", "items", "metadata", "activity"]
        : ["items", "metadata", "activity"];

  return ids.map((id) => ({ id, label: TAB_LABELS[id] }));
}

export function sourceBrowserShellAppliesToSource(source: Pick<Source, "sourceType" | "sourceSubtype">): boolean {
  return source.sourceType === "telegram"
    || (source.sourceType === "youtube" && source.sourceSubtype === "video");
}

export function smartDefaultSourceBrowserTab(source: Pick<Source, "sourceType" | "sourceSubtype">): SourceBrowserTabId {
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") return "transcript";
  if (source.sourceType === "telegram") return "timeline";
  return "items";
}

export function reconcileSourceBrowserTab(
  activeTab: SourceBrowserTabId | null,
  source: Pick<Source, "sourceType" | "sourceSubtype">,
): SourceBrowserTabId {
  const tabs = sourceBrowserTabsForSource(source);
  return activeTab && tabs.some((tab) => tab.id === activeTab)
    ? activeTab
    : smartDefaultSourceBrowserTab(source);
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
