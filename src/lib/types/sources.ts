export type TelegramSourceKind = "channel" | "supergroup" | "group";

export type DialogKindFilter = "all" | TelegramSourceKind;

export interface TelegramSourceInfo {
  id: number;
  title: string;
  username: string | null;
  telegram_source_kind: TelegramSourceKind;
  is_member: boolean;
  photo_data_url: string | null;
}

export interface SourceRecord {
  id: number;
  source_type: string;
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

export interface ItemRecord {
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

export type ForumTopicFilter =
  | { kind: "topic"; topic_id: number }
  | { kind: "uncategorized" };

export interface SourceForumTopicRecord {
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

export interface SyncResult {
  inserted: number;
  skipped: number;
  last_message_id: number | null;
  initial_sync_policy_applied: string | null;
  warnings: string[];
}

export interface SyncSettingsRecord {
  initial_sync_mode: "recent_messages" | "recent_days";
  initial_sync_value: number;
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
