import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
import type { AddTelegramSourceInput, TelegramDialogSource } from "$lib/types/sources";
import type { YoutubePlaylistDetail, YoutubePlaylistItemDetail } from "$lib/types/youtube";

export const YOUTUBE_PLAYLIST_IMPORT_LIMIT = 10;

export type YoutubeSmartImportProvider = "youtube" | "telegram" | "unknown";
export type YoutubeSmartImportKind = "video" | "playlist" | "channel" | "unsupported";

export interface YoutubeSmartImportClassification {
  provider: YoutubeSmartImportProvider;
  kind: YoutubeSmartImportKind;
  supported: boolean;
  reason: string | null;
  normalizedUrl?: string;
}

export interface PlaylistImportRow {
  id: string;
  item: YoutubePlaylistItemDetail;
  addable: boolean;
  disabledReason: string | null;
}

export interface PlaylistImportItemResult {
  id: string;
  title: string;
  canonicalUrl: string | null;
  status: "added" | "skipped" | "failed";
  sourceId: number | null;
  message: string | null;
}

export interface PlaylistImportSummary {
  added: number;
  skipped: number;
  failed: number;
  results: PlaylistImportItemResult[];
}

function youtubeHost(host: string) {
  const normalized = host.toLocaleLowerCase();
  return normalized === "youtu.be" || normalized === "youtube.com" || normalized.endsWith(".youtube.com");
}

function telegramHost(host: string) {
  const normalized = host.toLocaleLowerCase();
  return normalized === "t.me" || normalized === "telegram.me" || normalized.endsWith(".telegram.org");
}

function firstNonEmptySegment(url: URL) {
  return url.pathname.split("/").find((segment) => segment.trim().length > 0) ?? "";
}

function parseImportUrl(input: string) {
  const hasScheme = /^[a-zA-Z][a-zA-Z\d+.-]*:\/\//.test(input);
  return new URL(hasScheme ? input : `https://${input}`);
}

export function classifyYoutubeImportInput(input: string): YoutubeSmartImportClassification {
  const trimmed = input.trim();
  if (!trimmed) {
    return {
      provider: "unknown",
      kind: "unsupported",
      supported: false,
      reason: "Enter a YouTube video or playlist URL.",
    };
  }

  let parsed: URL;
  try {
    parsed = parseImportUrl(trimmed);
  } catch {
    return {
      provider: "unknown",
      kind: "unsupported",
      supported: false,
      reason: "Enter a valid URL.",
    };
  }

  const host = parsed.host.toLocaleLowerCase();
  if (telegramHost(host)) {
    return {
      provider: "telegram",
      kind: "unsupported",
      supported: false,
      reason: "Telegram sources are added from the Telegram tab.",
    };
  }

  if (!youtubeHost(host)) {
    return {
      provider: "unknown",
      kind: "unsupported",
      supported: false,
      reason: "Enter a YouTube video or playlist URL.",
    };
  }

  const firstSegment = firstNonEmptySegment(parsed);
  if (firstSegment.startsWith("@") || firstSegment === "channel" || firstSegment === "c" || firstSegment === "user") {
    return {
      provider: "youtube",
      kind: "channel",
      supported: false,
      reason: "YouTube channel import is not supported yet.",
    };
  }

  if (
    parsed.searchParams.get("v") ||
    (host === "youtu.be" && firstSegment) ||
    firstSegment === "shorts" ||
    firstSegment === "live"
  ) {
    return { provider: "youtube", kind: "video", supported: true, reason: null, normalizedUrl: parsed.toString() };
  }

  if (parsed.searchParams.get("list")) {
    return { provider: "youtube", kind: "playlist", supported: true, reason: null, normalizedUrl: parsed.toString() };
  }

  return {
    provider: "youtube",
    kind: "unsupported",
    supported: false,
    reason: "Enter a YouTube video or playlist URL.",
  };
}

export function libraryYoutubePlaylistSources(sources: LibraryCatalogSourceView[]) {
  return sources.filter((source) => source.provider === "youtube" && source.sourceSubtype === "playlist");
}

function playlistRowDisabledReason(item: YoutubePlaylistItemDetail) {
  if (item.videoSourceId !== null) return "Already in Library";
  if (!item.canonicalUrl) return "Missing video URL";
  return null;
}

export function buildPlaylistImportRows(detail: YoutubePlaylistDetail | null): PlaylistImportRow[] {
  return (detail?.items ?? []).map((item) => {
    const disabledReason = playlistRowDisabledReason(item);
    return {
      id: item.videoId,
      item,
      addable: disabledReason === null,
      disabledReason,
    };
  });
}

export function selectedAddablePlaylistRows(rows: PlaylistImportRow[], selectedIds: Set<string>) {
  return rows.filter((row) => selectedIds.has(row.id) && row.addable);
}

export function playlistSelectionLimitMessage(selectedAddableCount: number) {
  if (selectedAddableCount <= YOUTUBE_PLAYLIST_IMPORT_LIMIT) return null;
  return `Select ${YOUTUBE_PLAYLIST_IMPORT_LIMIT} or fewer videos for one import run.`;
}

export function emptyPlaylistImportSummary(): PlaylistImportSummary {
  return { added: 0, skipped: 0, failed: 0, results: [] };
}

export function summarizePlaylistImportResults(results: PlaylistImportItemResult[]): PlaylistImportSummary {
  return {
    added: results.filter((result) => result.status === "added").length,
    skipped: results.filter((result) => result.status === "skipped").length,
    failed: results.filter((result) => result.status === "failed").length,
    results,
  };
}

export function telegramDialogAddInput(
  accountId: number,
  dialog: TelegramDialogSource,
): AddTelegramSourceInput {
  return {
    accountId,
    sourceRef: String(dialog.id),
    expectedSubtype: dialog.sourceSubtype,
  };
}
