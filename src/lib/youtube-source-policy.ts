import type { YoutubeAvailabilityStatus } from "$lib/types/sources";

const retryableYoutubeAvailabilityStatuses = new Set<string>([
  "live_ended_transcript_pending",
  "no_captions",
  "unavailable_unknown",
] satisfies YoutubeAvailabilityStatus[]);

export function isRetryableYoutubeAvailabilityStatus(status: string | null | undefined): boolean {
  return status !== null && status !== undefined && retryableYoutubeAvailabilityStatuses.has(status);
}
