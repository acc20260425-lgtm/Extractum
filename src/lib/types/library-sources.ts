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
