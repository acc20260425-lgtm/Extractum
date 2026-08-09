import type {
  ForumTopicFilter,
  ListSourceItemsInput,
  ListYoutubeTranscriptSegmentsInput,
  SourceForumTopic,
  SourceItem,
  TelegramHistoryScope,
  YoutubeTranscriptSegment,
  YoutubeTranscriptSegmentCursor,
  YoutubeTranscriptSegmentsPage,
} from "$lib/types/sources";
import { sourceDataContainsTraceRef } from "$lib/analysis-evidence-source-navigation";

type SourceIdentity = { id: number; sourceType: string };
type SourceItemsPageRow = {
  id?: number;
  sourceId?: number;
  publishedAt?: number | null;
  pageCursor?: unknown;
};

export type SourceReaderSurfaceBinding = {
  youtubeTranscriptSegments: YoutubeTranscriptSegment[];
  groupLiveItemsBySource: Record<number, SourceItem[]>;
  groupLiveTranscriptSegmentsBySource: Record<number, YoutubeTranscriptSegment[]>;
  selectedGroupSourceId: number | null;
  sourceTopics: SourceForumTopic[];
  loadingSourceTopics: boolean;
  selectedTopicKey: string;
  showTopicSelector: boolean;
  sourceItemsHasMore: boolean;
  onLoadMoreSourceItems: () => void;
  onLoadMoreYoutubeTranscriptSegments: () => void;
  onLoadLiveGroupSourcePage: (sourceId: number) => void;
  onChangeSelectedGroupSourceId: (sourceId: number | null) => void;
  onChangeSelectedTopicKey: (value: string) => void;
};

export function sourceReaderSurfaceState({
  youtubeTranscriptSegments,
  groupLiveItemsBySource,
  groupLiveTranscriptSegmentsBySource,
  selectedGroupSourceId,
  sourceTopics,
  loadingSourceTopics,
  selectedTopicKey,
  showTopicSelector,
  sourceItemsHasMore,
  onLoadMoreSourceItems,
  onLoadMoreYoutubeTranscriptSegments,
  onLoadLiveGroupSourcePage,
  onChangeSelectedGroupSourceId,
  onChangeSelectedTopicKey,
}: SourceReaderSurfaceBinding): SourceReaderSurfaceBinding {
  return {
    youtubeTranscriptSegments,
    groupLiveItemsBySource,
    groupLiveTranscriptSegmentsBySource,
    selectedGroupSourceId,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    sourceItemsHasMore,
    onLoadMoreSourceItems: () => onLoadMoreSourceItems(),
    onLoadMoreYoutubeTranscriptSegments: () => onLoadMoreYoutubeTranscriptSegments(),
    onLoadLiveGroupSourcePage: (sourceId) => onLoadLiveGroupSourcePage(sourceId),
    onChangeSelectedGroupSourceId: (sourceId) => onChangeSelectedGroupSourceId(sourceId),
    onChangeSelectedTopicKey: (value) => onChangeSelectedTopicKey(value),
  };
}

export async function fetchSourceGroupPage<TItem>({
  sourceId,
  beforePublishedAt,
  limit,
  listSourceItems,
}: {
  sourceId: number;
  beforePublishedAt: number | null;
  limit: number;
  listSourceItems: (input: {
    sourceId: number;
    limit: number;
    beforePublishedAt: number | null;
    topicFilter: null;
  }) => Promise<TItem[]>;
}) {
  const items = await listSourceItems({ sourceId, limit, beforePublishedAt, topicFilter: null });
  const last = items.at(-1) as { publishedAt?: number | null } | undefined;
  return {
    sourceId,
    items,
    nextBeforePublishedAt: last?.publishedAt ?? beforePublishedAt,
    hasMore: items.length === limit,
    closeOpenedRun: false as const,
  };
}

export async function loadYoutubeTranscriptPage({
  sourceId,
  after,
  searchQuery,
  aroundStartMs,
  limit,
  listYoutubeTranscriptSegments,
}: {
  sourceId: number;
  after: YoutubeTranscriptSegmentCursor | null;
  searchQuery: string | null;
  aroundStartMs?: number | null;
  limit: number;
  listYoutubeTranscriptSegments: (input: ListYoutubeTranscriptSegmentsInput) => Promise<YoutubeTranscriptSegmentsPage>;
}): Promise<YoutubeTranscriptSegmentsPage> {
  return listYoutubeTranscriptSegments({
    sourceId,
    after,
    limit,
    searchQuery: searchQuery?.trim() || null,
    aroundStartMs: aroundStartMs ?? null,
  });
}

export async function fetchLiveGroupReaderPage<TItem>({
  source,
  transcriptActive,
  transcriptCursor,
  beforePublishedAt = null,
  listSourceItems,
  listYoutubeTranscriptSegments,
}: {
  source: SourceIdentity & { sourceSubtype?: string | null };
  transcriptActive: boolean;
  transcriptCursor: YoutubeTranscriptSegmentCursor | null;
  beforePublishedAt?: number | null;
  listSourceItems: (input: { sourceId: number; limit: number; beforePublishedAt: number | null; topicFilter: null }) => Promise<TItem[]>;
  listYoutubeTranscriptSegments: (input: ListYoutubeTranscriptSegmentsInput) => Promise<YoutubeTranscriptSegmentsPage>;
}) {
  if (transcriptActive && source.sourceType === "youtube" && source.sourceSubtype === "video") {
    return {
      kind: "youtube_transcript" as const,
      page: await loadYoutubeTranscriptPage({
        sourceId: source.id,
        after: transcriptCursor,
        searchQuery: null,
        aroundStartMs: null,
        limit: 80,
        listYoutubeTranscriptSegments,
      }),
    };
  }
  return {
    kind: "source_items" as const,
    page: await fetchSourceGroupPage({ sourceId: source.id, beforePublishedAt, limit: 40, listSourceItems }),
  };
}

export async function fetchBeforeReaderReveal<T>(
  fetchPage: () => Promise<T>,
  revealToReader: (page: T) => void | Promise<void>,
): Promise<T> {
  const page = await fetchPage();
  await revealToReader(page);
  return page;
}

type FocusedLiveTarget =
  | { kind: "source_item"; aroundItemId: number }
  | { kind: "youtube_transcript"; aroundStartMs: number }
  | { kind: "unsupported" };

export async function fetchFocusedLiveSourceData<TItem extends object>({
  source,
  target,
  canonicalRef,
  scope,
  itemLimit,
  historyScope,
  listSourceItems,
  listYoutubeTranscriptSegments,
}: {
  source: SourceIdentity & { sourceSubtype?: string | null };
  target: FocusedLiveTarget;
  canonicalRef: string;
  scope: { kind: "source"; sourceId: number } | { kind: "group_member"; groupId: number; sourceId: number };
  itemLimit: number;
  historyScope: TelegramHistoryScope;
  listSourceItems: (input: ListSourceItemsInput) => Promise<TItem[]>;
  listYoutubeTranscriptSegments: (input: ListYoutubeTranscriptSegmentsInput) => Promise<YoutubeTranscriptSegmentsPage>;
}) {
  if (target.kind === "source_item") {
    const page = await fetchSourceItemsPage({
      source,
      limit: itemLimit,
      beforePublishedAt: null,
      beforeCursor: null,
      historyScope,
      topicFilter: null,
      aroundItemId: target.aroundItemId,
      listSourceItems,
    });
    return {
      kind: "source_items" as const,
      page,
      containsTarget: sourceDataContainsTraceRef({ kind: "source_items", items: page.items } as never, canonicalRef, scope),
    };
  }
  if (target.kind === "youtube_transcript" && source.sourceType === "youtube" && source.sourceSubtype === "video") {
    const page = await loadYoutubeTranscriptPage({
      sourceId: source.id,
      after: null,
      searchQuery: null,
      aroundStartMs: target.aroundStartMs,
      limit: 80,
      listYoutubeTranscriptSegments,
    });
    return {
      kind: "youtube_transcript" as const,
      page,
      containsTarget: sourceDataContainsTraceRef({ kind: "youtube_transcript", segments: page.segments } as never, canonicalRef, scope),
    };
  }
  return { kind: "unsupported" as const, containsTarget: false };
}

export async function fetchSourceItemsPage<TItem extends object>({
  source,
  limit,
  beforePublishedAt,
  beforeCursor,
  historyScope,
  topicFilter,
  aroundItemId,
  listSourceItems,
}: {
  source: SourceIdentity;
  limit: number;
  beforePublishedAt: number | null;
  beforeCursor: string | null;
  historyScope: TelegramHistoryScope;
  topicFilter: ForumTopicFilter | null;
  aroundItemId?: number | null;
  listSourceItems: (input: ListSourceItemsInput) => Promise<TItem[]>;
}) {
  const telegram = source.sourceType === "telegram";
  const request: ListSourceItemsInput = {
    sourceId: source.id,
    limit,
    beforePublishedAt: telegram ? null : beforePublishedAt,
    beforeCursor: telegram ? beforeCursor : null,
    historyScope: telegram ? historyScope : "current",
    topicFilter,
  };
  if (aroundItemId !== undefined && aroundItemId !== null) request.aroundItemId = aroundItemId;
  const items = await listSourceItems(request) as Array<TItem & SourceItemsPageRow>;
  return {
    items,
    beforePublishedAt: request.beforePublishedAt,
    beforeCursor: request.beforeCursor ?? null,
    historyScope: request.historyScope ?? "current",
    topicFilter,
    aroundItemId: request.aroundItemId ?? null,
  };
}

export function sourceActivityJobs<TJob extends {
  job_id: string | number;
  source_id: number;
  related_source_id: number | null;
  started_at: number;
}>(sourceId: number, jobsBySource: Record<number, TJob[]>) {
  const direct = jobsBySource[sourceId] ?? [];
  const seen = new Set(direct.map((job) => job.job_id));
  const related = Object.values(jobsBySource).flat().filter((job) => {
    if (job.related_source_id !== sourceId || seen.has(job.job_id)) return false;
    seen.add(job.job_id);
    return true;
  });
  return [...direct, ...related].sort((left, right) => right.started_at - left.started_at);
}
