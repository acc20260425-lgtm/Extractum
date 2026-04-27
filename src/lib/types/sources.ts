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
}

export interface SyncResult {
  inserted: number;
  skipped: number;
  last_message_id: number | null;
  initial_sync_policy_applied: string | null;
}

export interface SyncSettingsRecord {
  initial_sync_mode: "recent_messages" | "recent_days";
  initial_sync_value: number;
}
