import { invoke } from "@tauri-apps/api/core";
import type {
  AddTelegramSourceInput,
  ForumTopicFilter,
  InitialSyncMode,
  ListSourceItemsInput,
  ListYoutubeTranscriptSegmentsInput,
  Source,
  SourceForumTopic,
  SourceForumTopicsResult,
  SourceItemsCursor,
  SourceItem,
  SourceSubtype,
  SyncSettings,
  SyncSourceResult,
  TelegramItemHistoryScope,
  TelegramDialogSource,
  TelegramMigratedHistoryStatus,
  TelegramSourceKind,
  TopicResolutionStatus,
  YoutubeAvailabilityStatus,
  YoutubePreview,
  YoutubePreviewKind,
  YoutubeTranscriptSegmentsPage,
} from "$lib/types/sources";

const SOURCE_COMMANDS = {
  listSources: "list_sources",
  listTelegramSources: "list_telegram_sources",
  addTelegramSource: "add_telegram_source",
  previewYoutubeSource: "preview_youtube_source",
  addYoutubeSource: "add_youtube_source",
  deleteSource: "delete_source",
  getSyncSettings: "get_sync_settings",
  saveSyncSettings: "save_sync_settings",
  syncSource: "sync_source",
  listSourceItems: "list_source_items",
  listSourceForumTopics: "list_source_forum_topics",
  listYoutubeTranscriptSegments: "list_youtube_transcript_segments",
} as const;

interface RawTelegramDialogSource {
  id: number;
  title: string;
  username: string | null;
  source_subtype: TelegramSourceKind;
  is_member: boolean;
  photo_data_url: string | null;
}

interface RawSource {
  id: number;
  source_type: Source["sourceType"];
  source_subtype?: SourceSubtype | null;
  account_id: number | null;
  external_id: string;
  title: string | null;
  last_sync_state: number | null;
  last_synced_at: number | null;
  is_member: boolean;
  is_active: boolean;
  created_at: number;
  telegram_username?: string | null;
  avatar_data_url: string | null;
  migrated_history_status?: TelegramMigratedHistoryStatus | null;
  migrated_history_detected_at?: number | null;
  migrated_history_refreshed_at?: number | null;
  migrated_history_row_count?: number | null;
  migrated_history_import_completed?: boolean | null;
}

interface RawSourceItem {
  id: number;
  source_id: number;
  external_id: string;
  item_kind: string;
  author: string | null;
  published_at: number;
  content: string | null;
  content_kind: string;
  has_media: boolean;
  media_kind: string | null;
  media_summary: string | null;
  media_file_name: string | null;
  media_mime_type: string | null;
  has_raw_data: boolean;
  forum_topic_id: number | null;
  forum_topic_title: string | null;
  forum_topic_top_message_id: number | null;
  reply_to_msg_id: number | null;
  reply_to_peer_kind: string | null;
  reply_to_peer_id: string | null;
  reply_to_top_id: number | null;
  reaction_count: number | null;
  history_scope: TelegramItemHistoryScope;
  is_migrated_history: boolean;
  migration_domain: "migrated_from_chat" | null;
  history_scope_label: "Current supergroup history" | "Migrated small-group history";
  page_cursor: SourceItemsCursor;
  youtube_comment?: RawYoutubeCommentItem | null;
}

interface RawYoutubeCommentItem {
  comment_id: string | null;
  parent_comment_id: string | null;
  is_reply: boolean;
  like_count: number | null;
  is_pinned: boolean;
  is_hearted: boolean;
  author_channel_url: string | null;
}

interface RawYoutubeTranscriptSegmentCursor {
  startMs: number;
  segmentId: number;
}

interface RawYoutubeTranscriptSegment {
  id: number;
  source_id: number;
  item_id: number;
  segment_index: number;
  start_ms: number;
  end_ms: number | null;
  text: string;
  caption_language: string | null;
  caption_track_kind: string | null;
  is_auto_generated: boolean;
}

interface RawYoutubeTranscriptSegmentsPage {
  segments: RawYoutubeTranscriptSegment[];
  next_cursor: RawYoutubeTranscriptSegmentCursor | null;
  has_more: boolean;
}

interface RawYoutubeCaptionsEstimate {
  has_manual: boolean;
  has_auto: boolean;
  languages: string[];
}

interface RawYoutubePreview {
  kind: YoutubePreviewKind;
  external_id: string;
  canonical_url: string;
  title: string | null;
  channel_title: string | null;
  channel_id: string | null;
  channel_handle: string | null;
  channel_url: string | null;
  thumbnail_url: string | null;
  duration_seconds: number | null;
  published_at: string | null;
  playlist_video_count: number | null;
  captions_estimate: RawYoutubeCaptionsEstimate | null;
  availability_status: YoutubeAvailabilityStatus;
  warnings: string[];
}

interface RawSourceForumTopic {
  kind: "topic" | "uncategorized";
  key: string;
  title: string;
  message_count: number;
  topic_id: number | null;
  top_message_id: number | null;
  icon_color: number | null;
  icon_emoji_id: number | null;
  is_closed: boolean;
  is_pinned: boolean;
  is_hidden: boolean;
  is_deleted: boolean;
  sort_order: number | null;
}

interface RawTopicResolutionStateSummary {
  status: TopicResolutionStatus;
  resolver_version: number;
  unresolved_count: number;
  pending_item_count: number;
  memberships_refreshed_at: number | null;
}

interface RawSourceForumTopicsResponse {
  topics: RawSourceForumTopic[];
  topic_resolution_state: RawTopicResolutionStateSummary;
}

interface RawSyncResult {
  inserted: number;
  skipped: number;
  last_message_id: number | null;
  initial_sync_policy_applied: string | null;
  warnings: string[];
}

interface RawSyncSettings {
  initial_sync_mode: InitialSyncMode;
  initial_sync_value: number;
}

export function listSources(accountId: number | null) {
  return invoke<RawSource[]>(SOURCE_COMMANDS.listSources, { accountId }).then((sources) =>
    sources.map(mapSource),
  );
}

export function listTelegramSources(accountId: number) {
  return invoke<RawTelegramDialogSource[]>(SOURCE_COMMANDS.listTelegramSources, {
    accountId,
  }).then((sources) => sources.map(mapTelegramDialogSource));
}

export function addTelegramSource(input: AddTelegramSourceInput) {
  return invoke<RawSource>(SOURCE_COMMANDS.addTelegramSource, {
    request: {
      accountId: input.accountId,
      sourceRef: input.sourceRef,
      expectedSubtype: input.expectedSubtype,
    },
  }).then(mapSource);
}

export function previewYoutubeSource(url: string) {
  return invoke<RawYoutubePreview>(SOURCE_COMMANDS.previewYoutubeSource, { url }).then(
    mapYoutubePreview,
  );
}

export function addYoutubeSource(url: string) {
  return invoke<RawSource>(SOURCE_COMMANDS.addYoutubeSource, { url }).then(mapSource);
}

export function deleteSource(sourceId: number) {
  return invoke<void>(SOURCE_COMMANDS.deleteSource, { sourceId });
}

export function getSyncSettings() {
  return invoke<RawSyncSettings>(SOURCE_COMMANDS.getSyncSettings).then(mapSyncSettings);
}

export function saveSyncSettings(settings: SyncSettings) {
  return invoke<RawSyncSettings>(SOURCE_COMMANDS.saveSyncSettings, {
    settings: {
      initialSyncMode: settings.initialSyncMode,
      initialSyncValue: settings.initialSyncValue,
    },
  }).then(mapSyncSettings);
}

export function syncSource(sourceId: number) {
  return invoke<RawSyncResult>(SOURCE_COMMANDS.syncSource, { sourceId }).then(mapSyncResult);
}

export function listSourceItems(input: ListSourceItemsInput) {
  return invoke<RawSourceItem[]>(SOURCE_COMMANDS.listSourceItems, {
    request: {
      sourceId: input.sourceId,
      limit: input.limit,
      beforePublishedAt: input.beforePublishedAt,
      beforeCursor: input.beforeCursor ? input.beforeCursor : null,
      topicFilter: mapForumTopicFilter(input.topicFilter),
      historyScope: input.historyScope ?? "current",
      ...(input.aroundItemId !== undefined ? { aroundItemId: input.aroundItemId } : {}),
    },
  }).then((items) => items.map(mapSourceItem));
}

export function listSourceForumTopics(sourceId: number) {
  return invoke<RawSourceForumTopicsResponse>(SOURCE_COMMANDS.listSourceForumTopics, {
    sourceId,
  }).then(mapSourceForumTopicsResponse);
}

export function listYoutubeTranscriptSegments(input: ListYoutubeTranscriptSegmentsInput) {
  return invoke<RawYoutubeTranscriptSegmentsPage>(
    SOURCE_COMMANDS.listYoutubeTranscriptSegments,
    {
      request: {
        sourceId: input.sourceId,
        after: input.after
          ? {
              startMs: input.after.startMs,
              segmentId: input.after.segmentId,
            }
          : null,
        limit: input.limit,
        searchQuery: input.searchQuery,
        ...(input.aroundStartMs !== undefined ? { aroundStartMs: input.aroundStartMs } : {}),
      },
    },
  ).then(mapYoutubeTranscriptSegmentsPage);
}

function mapTelegramDialogSource(source: RawTelegramDialogSource): TelegramDialogSource {
  return {
    id: source.id,
    title: source.title,
    username: source.username,
    sourceSubtype: source.source_subtype,
    isMember: source.is_member,
    photoDataUrl: source.photo_data_url,
  };
}

function mapSource(source: RawSource): Source {
  return {
    id: source.id,
    sourceType: source.source_type,
    sourceSubtype: source.source_subtype ?? null,
    accountId: source.account_id,
    externalId: source.external_id,
    title: source.title,
    lastSyncState: source.last_sync_state,
    lastSyncedAt: source.last_synced_at,
    isMember: source.is_member,
    isActive: source.is_active,
    createdAt: source.created_at,
    telegramUsername: source.telegram_username ?? null,
    avatarDataUrl: source.avatar_data_url,
    migratedHistoryStatus: source.migrated_history_status ?? "none",
    migratedHistoryDetectedAt: source.migrated_history_detected_at ?? null,
    migratedHistoryRefreshedAt: source.migrated_history_refreshed_at ?? null,
    migratedHistoryRowCount: source.migrated_history_row_count ?? 0,
    migratedHistoryImportCompleted: source.migrated_history_import_completed ?? false,
  };
}

function mapSourceItem(item: RawSourceItem): SourceItem {
  return {
    id: item.id,
    sourceId: item.source_id,
    externalId: item.external_id,
    itemKind: item.item_kind,
    author: item.author,
    publishedAt: item.published_at,
    content: item.content,
    contentKind: item.content_kind,
    hasMedia: item.has_media,
    mediaKind: item.media_kind,
    mediaSummary: item.media_summary,
    mediaFileName: item.media_file_name,
    mediaMimeType: item.media_mime_type,
    hasRawData: item.has_raw_data,
    forumTopicId: item.forum_topic_id,
    forumTopicTitle: item.forum_topic_title,
    forumTopicTopMessageId: item.forum_topic_top_message_id,
    replyToMessageId: item.reply_to_msg_id,
    replyToPeerKind: item.reply_to_peer_kind,
    replyToPeerId: item.reply_to_peer_id,
    replyToTopMessageId: item.reply_to_top_id,
    reactionCount: item.reaction_count,
    historyScope: item.history_scope,
    isMigratedHistory: item.is_migrated_history,
    migrationDomain: item.migration_domain,
    historyScopeLabel: item.history_scope_label,
    pageCursor: item.page_cursor,
    ...(item.youtube_comment
      ? {
          youtubeComment: {
            commentId: item.youtube_comment.comment_id,
            parentCommentId: item.youtube_comment.parent_comment_id,
            isReply: item.youtube_comment.is_reply,
            likeCount: item.youtube_comment.like_count,
            isPinned: item.youtube_comment.is_pinned,
            isHearted: item.youtube_comment.is_hearted,
            authorChannelUrl: item.youtube_comment.author_channel_url,
          },
        }
      : {}),
  };
}

function mapYoutubeTranscriptSegmentsPage(
  page: RawYoutubeTranscriptSegmentsPage,
): YoutubeTranscriptSegmentsPage {
  return {
    segments: page.segments.map((segment) => ({
      id: segment.id,
      sourceId: segment.source_id,
      itemId: segment.item_id,
      segmentIndex: segment.segment_index,
      startMs: segment.start_ms,
      endMs: segment.end_ms,
      text: segment.text,
      captionLanguage: segment.caption_language,
      captionTrackKind: segment.caption_track_kind,
      isAutoGenerated: segment.is_auto_generated,
    })),
    nextCursor: page.next_cursor
      ? {
          startMs: page.next_cursor.startMs,
          segmentId: page.next_cursor.segmentId,
        }
      : null,
    hasMore: page.has_more,
  };
}

function mapYoutubePreview(preview: RawYoutubePreview): YoutubePreview {
  return {
    kind: preview.kind,
    externalId: preview.external_id,
    canonicalUrl: preview.canonical_url,
    title: preview.title,
    channelTitle: preview.channel_title,
    channelId: preview.channel_id,
    channelHandle: preview.channel_handle,
    channelUrl: preview.channel_url,
    thumbnailUrl: preview.thumbnail_url,
    durationSeconds: preview.duration_seconds,
    publishedAt: preview.published_at,
    playlistVideoCount: preview.playlist_video_count,
    captionsEstimate: preview.captions_estimate
      ? {
          hasManual: preview.captions_estimate.has_manual,
          hasAuto: preview.captions_estimate.has_auto,
          languages: preview.captions_estimate.languages,
        }
      : null,
    availabilityStatus: preview.availability_status,
    warnings: preview.warnings,
  };
}

function mapSourceForumTopic(topic: RawSourceForumTopic): SourceForumTopic {
  return {
    kind: topic.kind,
    key: topic.key,
    title: topic.title,
    messageCount: topic.message_count,
    topicId: topic.topic_id,
    topMessageId: topic.top_message_id,
    iconColor: topic.icon_color,
    iconEmojiId: topic.icon_emoji_id,
    isClosed: topic.is_closed,
    isPinned: topic.is_pinned,
    isHidden: topic.is_hidden,
    isDeleted: topic.is_deleted,
    sortOrder: topic.sort_order,
  };
}

function mapSourceForumTopicsResponse(
  response: RawSourceForumTopicsResponse,
): SourceForumTopicsResult {
  return {
    topics: response.topics.map(mapSourceForumTopic),
    topicResolutionState: {
      status: response.topic_resolution_state.status,
      resolverVersion: response.topic_resolution_state.resolver_version,
      unresolvedCount: response.topic_resolution_state.unresolved_count,
      pendingItemCount: response.topic_resolution_state.pending_item_count,
      membershipsRefreshedAt: response.topic_resolution_state.memberships_refreshed_at,
    },
  };
}

function mapSyncResult(result: RawSyncResult): SyncSourceResult {
  return {
    inserted: result.inserted,
    skipped: result.skipped,
    lastMessageId: result.last_message_id,
    initialSyncPolicyApplied: result.initial_sync_policy_applied,
    warnings: result.warnings,
  };
}

function mapSyncSettings(settings: RawSyncSettings): SyncSettings {
  return {
    initialSyncMode: settings.initial_sync_mode,
    initialSyncValue: settings.initial_sync_value,
  };
}

function mapForumTopicFilter(filter: ForumTopicFilter | null) {
  if (!filter) {
    return null;
  }
  if (filter.kind === "uncategorized") {
    return filter;
  }
  return {
    kind: "topic" as const,
    topicId: filter.topicId,
  };
}
