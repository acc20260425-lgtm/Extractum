import type { Source } from "$lib/types/sources";
import {
  summarizePlaylistImportResults,
  type PlaylistImportItemResult,
  type PlaylistImportRow,
} from "./library-add-source-model";

export interface AddSelectedYoutubePlaylistVideosInput {
  rows: PlaylistImportRow[];
  addYoutubeSource(url: string): Promise<Source>;
  formatError(action: string, error: unknown): string;
}

function resultTitle(row: PlaylistImportRow) {
  return row.item.title ?? row.item.videoId;
}

export async function addSelectedYoutubePlaylistVideos({
  rows,
  addYoutubeSource,
  formatError,
}: AddSelectedYoutubePlaylistVideosInput) {
  const results: PlaylistImportItemResult[] = [];

  for (const row of rows) {
    if (!row.addable || !row.item.canonicalUrl) {
      results.push({
        id: row.id,
        title: resultTitle(row),
        canonicalUrl: row.item.canonicalUrl,
        status: "skipped",
        sourceId: null,
        message: row.disabledReason ?? "Video cannot be added.",
      });
      continue;
    }

    try {
      const source = await addYoutubeSource(row.item.canonicalUrl);
      results.push({
        id: row.id,
        title: resultTitle(row),
        canonicalUrl: row.item.canonicalUrl,
        status: "added",
        sourceId: source.id,
        message: source.title ?? source.externalId,
      });
    } catch (error) {
      results.push({
        id: row.id,
        title: resultTitle(row),
        canonicalUrl: row.item.canonicalUrl,
        status: "failed",
        sourceId: null,
        message: formatError(`adding ${resultTitle(row)}`, error),
      });
    }
  }

  return summarizePlaylistImportResults(results);
}
