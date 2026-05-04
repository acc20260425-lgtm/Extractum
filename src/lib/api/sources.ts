import { invoke } from "@tauri-apps/api/core";
import type {
  ForumTopicFilter,
  InitialSyncMode,
  Source,
  SourceForumTopic,
  SourceItem,
  SyncSettings,
  SyncSourceResult,
  TelegramDialogSource,
  TelegramSourceKind,
} from "$lib/types/sources";

const SOURCE_COMMANDS = {
  listSources: "list_sources",
  listTelegramSources: "list_telegram_sources",
  addTelegramSource: "add_telegram_source",
  deleteSource: "delete_source",
  getSyncSettings: "get_sync_settings",
  saveSyncSettings: "save_sync_settings",
  syncSource: "sync_source",
  listSourceItems: "list_source_items",
  listSourceForumTopics: "list_source_forum_topics",
} as const;

interface RawTelegramDialogSource {
  id: number;
  title: string;
  username: string | null;
  telegram_source_kind: TelegramSourceKind;
  is_member: boolean;
  photo_data_url: string | null;
}

interface RawSource {
  id: number;
  source_type: Source["sourceType"];
  telegram_source_kind: TelegramSourceKind;
  account_id: number | null;
  external_id: string;
  title: string | null;
  last_sync_state: number | null;
  last_synced_at: number | null;
  is_member: boolean;
  is_active: boolean;
  created_at: number;
  avatar_data_url: string | null;
}

interface RawSourceItem {
  id: number;
  source_id: number;
  external_id: string;
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

export interface AddTelegramSourceInput {
  accountId: number;
  sourceRef: string;
  expectedKind: TelegramSourceKind | null;
}

export interface ListSourceItemsInput {
  sourceId: number;
  limit: number;
  beforePublishedAt: number | null;
  topicFilter: ForumTopicFilter | null;
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
      expectedKind: input.expectedKind,
    },
  }).then(mapSource);
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
      topicFilter: mapForumTopicFilter(input.topicFilter),
    },
  }).then((items) => items.map(mapSourceItem));
}

export function listSourceForumTopics(sourceId: number) {
  return invoke<RawSourceForumTopic[]>(SOURCE_COMMANDS.listSourceForumTopics, {
    sourceId,
  }).then((topics) => topics.map(mapSourceForumTopic));
}

function mapTelegramDialogSource(source: RawTelegramDialogSource): TelegramDialogSource {
  return {
    id: source.id,
    title: source.title,
    username: source.username,
    telegramSourceKind: source.telegram_source_kind,
    isMember: source.is_member,
    photoDataUrl: source.photo_data_url,
  };
}

function mapSource(source: RawSource): Source {
  return {
    id: source.id,
    sourceType: source.source_type,
    telegramSourceKind: source.telegram_source_kind,
    accountId: source.account_id,
    externalId: source.external_id,
    title: source.title,
    lastSyncState: source.last_sync_state,
    lastSyncedAt: source.last_synced_at,
    isMember: source.is_member,
    isActive: source.is_active,
    createdAt: source.created_at,
    avatarDataUrl: source.avatar_data_url,
  };
}

function mapSourceItem(item: RawSourceItem): SourceItem {
  return {
    id: item.id,
    sourceId: item.source_id,
    externalId: item.external_id,
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
    topicId: "topicId" in filter ? filter.topicId : filter.topic_id,
  };
}
