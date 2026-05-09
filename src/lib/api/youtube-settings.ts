import { invoke } from "@tauri-apps/api/core";
import type { YoutubeAuthStatus, YoutubeSettings } from "$lib/types/youtube";

interface RawYoutubeSettings {
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

// This intentionally mirrors YoutubeSettings because the Rust DTO serializes
// with #[serde(rename_all = "camelCase")]. Keep the Rust serialization test
// and the Vitest wrapper test in place so a future rename mismatch fails at
// the API boundary.

export function getYoutubeSettings() {
  return invoke<RawYoutubeSettings>("get_youtube_settings").then(mapYoutubeSettings);
}

export function saveYoutubeSettings(settings: YoutubeSettings) {
  return invoke<RawYoutubeSettings>("save_youtube_settings", { settings }).then(
    mapYoutubeSettings,
  );
}

export function getYoutubeAuthStatus() {
  return invoke<YoutubeAuthStatus>("get_youtube_auth_status");
}

export function saveYoutubeCookies(cookies: string) {
  return invoke<YoutubeAuthStatus>("save_youtube_cookies", { cookies });
}

export function clearYoutubeAuth() {
  return invoke<YoutubeAuthStatus>("clear_youtube_auth");
}

function mapYoutubeSettings(settings: RawYoutubeSettings): YoutubeSettings {
  return {
    authEnabled: settings.authEnabled,
    preferredCaptionsLanguage: settings.preferredCaptionsLanguage,
    delayBetweenRequestsMs: settings.delayBetweenRequestsMs,
    maxParallelVideoSyncs: settings.maxParallelVideoSyncs,
    maxParallelCommentSyncs: settings.maxParallelCommentSyncs,
    pauseOnAuthChallenge: settings.pauseOnAuthChallenge,
    dailySoftLimit: settings.dailySoftLimit,
    retryBackoffMs: settings.retryBackoffMs,
    stopAfterConsecutiveFailures: settings.stopAfterConsecutiveFailures,
  };
}
