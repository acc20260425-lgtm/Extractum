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
