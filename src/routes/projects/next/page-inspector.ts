type InspectorRow = {
  title: string;
  handle: string;
  typeLabel: string;
  typeDot: string;
};

type ProjectSource = {
  provider: string;
  source_subtype: string | null;
  handle?: string | null;
};

export function projectInspectorSelection(row: InspectorRow | null) {
  if (!row) return null;
  return {
    title: row.title,
    handle: row.handle,
    typeLabel: row.typeLabel,
    typeDot: row.typeDot,
  };
}

export function youtubeProjectSourceUrl(source: ProjectSource): string | null {
  const externalId = source.handle?.trim();
  if (source.provider !== "youtube" || !externalId) return null;
  if (source.source_subtype === "video") {
    return `https://www.youtube.com/watch?v=${encodeURIComponent(externalId)}`;
  }
  if (source.source_subtype === "playlist") {
    return `https://www.youtube.com/playlist?list=${encodeURIComponent(externalId)}`;
  }
  if (source.source_subtype === "channel") {
    return externalId.startsWith("@")
      ? `https://www.youtube.com/${encodeURIComponent(externalId)}`
      : `https://www.youtube.com/channel/${encodeURIComponent(externalId)}`;
  }
  return null;
}

export function projectInspectorActions({
  url,
  openUrl,
  onError,
}: {
  url: string | null;
  openUrl: (url: string) => void | Promise<void>;
  onError?: (error: unknown) => void;
}) {
  return {
    openDisabled: url === null,
    async onOpen() {
      if (!url) return;
      try {
        await openUrl(url);
      } catch (error) {
        onError?.(error);
      }
    },
  };
}
