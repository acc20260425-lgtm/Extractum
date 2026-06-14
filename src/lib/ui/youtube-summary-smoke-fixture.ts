import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";

export interface YoutubeSummarySmokeFixtureEnv {
  DEV?: boolean;
  VITE_YOUTUBE_SUMMARY_SMOKE_FIXTURE?: string;
}

export const YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID = 990_001;
export const YOUTUBE_SUMMARY_SMOKE_FIXTURE_RUN_ID = 990_101;
export const YOUTUBE_SUMMARY_SMOKE_FIXTURE_ACTIVE_RUN_ID = 990_102;
export const YOUTUBE_SUMMARY_SMOKE_FIXTURE_LABEL = "YouTube Summary Smoke Fixture";

export function isYoutubeSummarySmokeFixtureEnabled(
  env: YoutubeSummarySmokeFixtureEnv,
): boolean {
  return env.DEV === true && env.VITE_YOUTUBE_SUMMARY_SMOKE_FIXTURE === "1";
}

export function withYoutubeSummarySmokeFixtureSources(
  sources: LibraryCatalogSourceView[],
  env: YoutubeSummarySmokeFixtureEnv,
): LibraryCatalogSourceView[] {
  if (!isYoutubeSummarySmokeFixtureEnabled(env)) return sources;
  if (sources.some((source) => source.sourceId === YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID)) {
    return sources;
  }
  return [youtubeSummarySmokeFixtureSource(), ...sources];
}

function youtubeSummarySmokeFixtureSource(): LibraryCatalogSourceView {
  return {
    id: `source:${YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID}`,
    sourceId: YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID,
    provider: "youtube",
    sourceSubtype: "video",
    title: YOUTUBE_SUMMARY_SMOKE_FIXTURE_LABEL,
    subtitle: "Dev-only prompt pack smoke fixture",
    typeLabel: "YouTube / Video",
    status: "active",
    statusDetail: "Synced fixture",
    projectCount: 0,
    itemCount: 2,
    itemCountLabel: "2 items",
    addedAtLabel: "06/14/2026, 12:00 AM",
    lastSyncedLabel: "06/14/2026, 12:05 AM",
    canonicalUrl: "https://www.youtube.com/watch?v=extractum-fixture",
    externalId: "extractum-fixture",
    youtube: {
      channel_title: "Extractum Fixture Channel",
      video_form: "video",
      duration_seconds: 734,
      playlist_video_count: null,
      availability_status: "available",
    },
    telegram: null,
  };
}
