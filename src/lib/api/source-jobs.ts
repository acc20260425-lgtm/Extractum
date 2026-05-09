import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  SourceJobEvent,
  SourceJobRecord,
  SourceJobStatus,
  YoutubeSyncOptions,
} from "$lib/types/sources";

export type { SourceJobRecord };

export interface SourceJobListFilter {
  sourceId?: number;
  status?: SourceJobStatus;
  limit?: number;
}

export function listSourceJobs(filter: SourceJobListFilter = {}) {
  return invoke<SourceJobRecord[]>("list_source_jobs", { filter });
}

export function syncYoutubeSource(sourceId: number, options: YoutubeSyncOptions) {
  return invoke<SourceJobRecord>("sync_youtube_source", { sourceId, options });
}

export function syncYoutubePlaylistVideo(
  playlistSourceId: number,
  videoSourceId: number,
  options: YoutubeSyncOptions,
) {
  return invoke<SourceJobRecord>("sync_youtube_playlist_video", {
    playlistSourceId,
    videoSourceId,
    options,
  });
}

export function retryFailedYoutubePlaylistVideos(
  sourceId: number,
  options: YoutubeSyncOptions,
) {
  return invoke<SourceJobRecord>("retry_failed_youtube_playlist_videos", { sourceId, options });
}

export function cancelSourceJob(jobId: string) {
  return invoke<void>("cancel_source_job", { jobId });
}

export function listenToSourceJobEvents(callback: (event: SourceJobRecord) => void) {
  return listen<SourceJobEvent>("sources://source-job", (event) => callback(event.payload));
}
