import type { SourceJobRecord } from "$lib/types/sources";

export type LibrarySourceProvider = "telegram" | "youtube" | "rss" | "forum" | "web" | "other";

export type LibrarySourceSubtype =
  | "video"
  | "playlist"
  | "channel"
  | "supergroup"
  | "group"
  | "feed"
  | "thread"
  | "board"
  | "site"
  | null;

export interface LibraryYoutubeSourceDetails {
  video_form: string | null;
  duration_seconds: number | null;
  playlist_video_count: number | null;
  channel_title: string | null;
  availability_status: string | null;
}

export interface LibraryTelegramSourceDetails {
  account_id: number | null;
}

export interface LibrarySourceRecord {
  source_id: number;
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  account_id: number | null;
  external_id: string | null;
  title: string | null;
  subtitle: string | null;
  canonical_url: string | null;
  created_at: number;
  last_synced_at: number | null;
  item_count: number;
  project_count: number;
  youtube: LibraryYoutubeSourceDetails | null;
  telegram: LibraryTelegramSourceDetails | null;
}

export type LibraryCatalogStatus = "active" | "syncing" | "error" | "unavailable";

export interface LibraryCatalogCapabilities {
  can_refresh_source: boolean;
  can_delete: boolean;
  can_edit: boolean;
  can_connect_to_project: boolean;
}

export interface LibraryCatalogDisabledReasons {
  refresh_source: string | null;
  delete: string | null;
  edit: string | null;
  connect_to_project: string | null;
}

export interface LibraryCatalogRecord {
  source: LibrarySourceRecord;
  latest_job: SourceJobRecord | null;
  status: LibraryCatalogStatus;
  status_detail: string | null;
  capabilities: LibraryCatalogCapabilities;
  disabled_reasons: LibraryCatalogDisabledReasons;
}

export interface LibraryCatalogFilterCount {
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  count: number;
  disabled: boolean;
  disabled_reason: string | null;
}

export interface LibraryCatalogResponse {
  sources: LibraryCatalogRecord[];
  filter_counts: LibraryCatalogFilterCount[];
}
