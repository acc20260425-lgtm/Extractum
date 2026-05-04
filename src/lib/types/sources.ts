export type TelegramSourceKind = "channel" | "supergroup" | "group";
export type SourceType = "telegram";
export type InitialSyncMode = "recent_messages" | "recent_days";

export type DialogKindFilter = "all" | TelegramSourceKind;

export interface TelegramDialogSource {
  id: number;
  title: string;
  username: string | null;
  telegramSourceKind: TelegramSourceKind;
  isMember: boolean;
  photoDataUrl: string | null;
}

export interface Source {
  id: number;
  sourceType: SourceType;
  telegramSourceKind: TelegramSourceKind;
  accountId: number | null;
  externalId: string;
  title: string | null;
  lastSyncState: number | null;
  lastSyncedAt: number | null;
  isMember: boolean;
  isActive: boolean;
  createdAt: number;
  avatarDataUrl: string | null;
}

export interface SourceItem {
  id: number;
  sourceId: number;
  externalId: string;
  author: string | null;
  publishedAt: number;
  content: string | null;
  contentKind: string;
  hasMedia: boolean;
  mediaKind: string | null;
  mediaSummary: string | null;
  mediaFileName: string | null;
  mediaMimeType: string | null;
  hasRawData: boolean;
  forumTopicId: number | null;
  forumTopicTitle: string | null;
  forumTopicTopMessageId: number | null;
}

export type ForumTopicFilter =
  | { kind: "topic"; topicId: number }
  | { kind: "uncategorized" };

export interface SourceForumTopic {
  kind: "topic" | "uncategorized";
  key: string;
  title: string;
  messageCount: number;
  topicId: number | null;
  topMessageId: number | null;
  iconColor: number | null;
  iconEmojiId: number | null;
  isClosed: boolean;
  isPinned: boolean;
  isHidden: boolean;
  isDeleted: boolean;
  sortOrder: number | null;
}

export interface SyncSourceResult {
  inserted: number;
  skipped: number;
  lastMessageId: number | null;
  initialSyncPolicyApplied: string | null;
  warnings: string[];
}

export interface SyncSettings {
  initialSyncMode: InitialSyncMode;
  initialSyncValue: number;
}

export type TakeoutImportStatus =
  | "queued"
  | "running"
  | "cancel_requested"
  | "completed"
  | "failed"
  | "cancelled";

export type TakeoutImportPhase =
  | "queued"
  | "resolving_source"
  | "starting_takeout"
  | "validating_peer"
  | "loading_splits"
  | "counting"
  | "importing_history"
  | "finishing_takeout"
  | "refreshing_aux"
  | "completed"
  | "failed"
  | "cancelled";

export interface TakeoutImportJobRecord {
  job_id: string;
  source_id: number;
  account_id: number;
  status: TakeoutImportStatus;
  phase: TakeoutImportPhase;
  message: string | null;
  inserted: number;
  skipped: number;
  progress_current: number | null;
  progress_total: number | null;
  started_at: number;
  finished_at: number | null;
  warnings: string[];
  error: string | null;
}

export type TakeoutImportEvent = TakeoutImportJobRecord;

export interface StartTakeoutImportResponse {
  job_id: string;
}

export interface CancelTakeoutImportResponse {
  cancelled: boolean;
}

export interface NotebookLmExportRequest {
  export_id: string | null;
  source_id: number;
  output_dir: string;
  period_from: number | null;
  period_to: number | null;
  include_media_placeholders: boolean;
  min_message_length: number;
  max_words_per_file: number;
  max_bytes_per_file: number;
  overwrite_existing: boolean;
}

export interface NotebookLmExportFile {
  path: string;
  message_count: number;
  byte_size: number;
  approximate_word_count: number;
}

export interface NotebookLmExportResult {
  output_dir: string;
  files: NotebookLmExportFile[];
  glossary_file: string | null;
  exported_message_count: number;
  skipped_message_count: number;
  warning_count: number;
  warnings: string[];
}

export type NotebookLmExportEventKind = "started" | "progress" | "completed" | "failed";

export type NotebookLmExportPhase =
  | "loading"
  | "filtering"
  | "chunking"
  | "preparing_output"
  | "writing"
  | "manifest"
  | "completed"
  | "failed";

export interface NotebookLmExportEvent {
  export_id: string;
  source_id: number;
  kind: NotebookLmExportEventKind;
  phase: NotebookLmExportPhase;
  message: string | null;
  progress_current: number | null;
  progress_total: number | null;
  file_path: string | null;
  error: string | null;
}
