import { formatAppError } from "$lib/app-error";
import type { Source } from "$lib/types/sources";
import type { YoutubePlaylistDetail, YoutubeVideoDetail } from "$lib/types/youtube";
import type { YoutubeDetailErrorState } from "$lib/youtube-source-view-model";

export async function resolveYoutubeDetailForSource(
  source: Pick<Source, "id" | "sourceSubtype">,
  deps: {
    getYoutubeVideoDetail: (sourceId: number) => Promise<YoutubeVideoDetail>;
    getYoutubePlaylistDetail: (sourceId: number) => Promise<YoutubePlaylistDetail>;
  },
) {
  try {
    if (source.sourceSubtype === "playlist") {
      const playlistDetail = await deps.getYoutubePlaylistDetail(source.id);
      return { kind: "playlist" as const, videoDetail: null, playlistDetail, error: null };
    }
    const videoDetail = await deps.getYoutubeVideoDetail(source.id);
    return { kind: "video" as const, videoDetail, playlistDetail: null, error: null };
  } catch (error) {
    return {
      kind: "error" as const,
      videoDetail: null,
      playlistDetail: null,
      error: {
        sourceId: source.id,
        sourceSubtype: source.sourceSubtype,
        message: formatAppError("loading YouTube detail", error),
      } satisfies Exclude<YoutubeDetailErrorState, null>,
    };
  }
}
