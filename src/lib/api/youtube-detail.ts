import { invoke } from "@tauri-apps/api/core";
import type {
  YoutubePlaylistDetail,
  YoutubeRuntimeStatus,
  YoutubeSourceSummary,
  YoutubeVideoDetail,
} from "$lib/types/youtube";

export function getYoutubeRuntimeStatus() {
  return invoke<YoutubeRuntimeStatus>("get_youtube_runtime_status");
}

export function listYoutubeSourceSummaries(sourceIds: number[]) {
  return invoke<YoutubeSourceSummary[]>("list_youtube_source_summaries", { sourceIds });
}

export function getYoutubeVideoDetail(sourceId: number) {
  return invoke<YoutubeVideoDetail>("get_youtube_video_detail", { sourceId });
}

export function getYoutubePlaylistDetail(sourceId: number) {
  return invoke<YoutubePlaylistDetail>("get_youtube_playlist_detail", { sourceId });
}
