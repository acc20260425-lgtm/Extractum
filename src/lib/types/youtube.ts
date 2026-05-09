export interface YoutubeSettings {
  authEnabled: boolean;
  preferredCaptionsLanguage: string;
  delayBetweenRequestsMs: number;
  maxParallelVideoSyncs: number;
  maxParallelCommentSyncs: number;
  pauseOnAuthChallenge: boolean;
  dailySoftLimit: number;
  retryBackoffMs: number;
  stopAfterConsecutiveFailures: number;
}

export interface YoutubeAuthStatus {
  enabled: boolean;
  hasCookies: boolean;
  message: "Auth disabled" | "Cookies stored" | "No cookies configured" | string;
}

export type YoutubeContentSyncState =
  | "not_synced"
  | "synced"
  | "unavailable"
  | "failed"
  | "unknown";

export interface YoutubeRuntimeStatus {
  ytdlpAvailable: boolean;
  ytdlpVersion: string | null;
  message: string;
}

export interface YoutubeContentStatus {
  state: YoutubeContentSyncState;
  itemCount: number;
  segmentCount: number;
  lastSyncedAt: number | null;
  label: string;
}

export interface YoutubeSourceSummary {
  sourceId: number;
  sourceSubtype: string;
  title: string | null;
  channelTitle: string | null;
  channelHandle: string | null;
  canonicalUrl: string | null;
  thumbnailUrl: string | null;
  durationSeconds: number | null;
  publishedAt: number | null;
  availabilityStatus: string | null;
  videoCount: number | null;
  linkedVideoCount: number | null;
  unavailableCount: number | null;
  captions: YoutubeContentStatus;
  comments: YoutubeContentStatus;
}

export interface YoutubePlaylistMembership {
  playlistSourceId: number;
  playlistTitle: string | null;
  position: number | null;
  availabilityStatus: string;
}

export interface YoutubeVideoDetail {
  summary: YoutubeSourceSummary;
  playlistMemberships: YoutubePlaylistMembership[];
}

export interface YoutubePlaylistItemDetail {
  position: number | null;
  videoId: string;
  videoSourceId: number | null;
  title: string | null;
  canonicalUrl: string | null;
  thumbnailUrl: string | null;
  durationSeconds: number | null;
  publishedAt: number | null;
  availabilityStatus: string;
  isRemovedFromPlaylist: boolean;
  captions: YoutubeContentStatus;
  comments: YoutubeContentStatus;
}

export interface YoutubePlaylistDetail {
  summary: YoutubeSourceSummary;
  items: YoutubePlaylistItemDetail[];
}
