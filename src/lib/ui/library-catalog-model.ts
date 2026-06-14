import type {
  LibraryCatalogFilterCount,
  LibraryCatalogRecord,
  LibrarySourceProvider,
  LibrarySourceSubtype,
  LibraryTelegramSourceDetails,
  LibraryYoutubeSourceDetails,
} from "$lib/types/library-sources";

export type LibraryCatalogSourceStatus = "active" | "syncing" | "error" | "unavailable";

export type LibraryCatalogSourceView = {
  id: string;
  sourceId: number;
  provider: LibrarySourceProvider;
  sourceSubtype: LibrarySourceSubtype;
  title: string;
  subtitle: string | null;
  typeLabel: string;
  status: LibraryCatalogSourceStatus;
  statusDetail: string | null;
  projectCount: number;
  itemCount: number;
  itemCountLabel: string;
  addedAtLabel: string;
  lastSyncedLabel: string;
  canonicalUrl: string | null;
  externalId: string | null;
  youtube: LibraryYoutubeSourceDetails | null;
  telegram: LibraryTelegramSourceDetails | null;
};

export type LibraryCatalogFilterId =
  | "all"
  | `provider:${LibrarySourceProvider}`
  | `provider:${LibrarySourceProvider}/subtype:${Exclude<LibrarySourceSubtype, null>}`;

export type LibraryCatalogFilterTreeRow = {
  id: LibraryCatalogFilterId;
  label: string;
  provider: LibrarySourceProvider | "all";
  subtype?: Exclude<LibrarySourceSubtype, null>;
  count: number;
  disabled?: boolean;
  disabledReason?: string;
  data?: LibraryCatalogFilterTreeRow[];
};

export type LibraryCatalogFilterState = {
  filterId: LibraryCatalogFilterId;
  query: string;
};

export const LIBRARY_CATALOG_ALL_FILTER_ID: LibraryCatalogFilterId = "all";

const PROVIDER_LABELS: Record<LibrarySourceProvider, string> = {
  telegram: "Telegram",
  youtube: "YouTube",
  rss: "RSS",
  forum: "Forum",
  web: "Web",
  other: "Other",
};

const SUBTYPE_LABELS: Record<Exclude<LibrarySourceSubtype, null>, string> = {
  video: "Video",
  playlist: "Playlist",
  channel: "Channel",
  supergroup: "Supergroup",
  group: "Group",
  feed: "Feed",
  thread: "Thread",
  board: "Board",
  site: "Site",
};

function sourceRowId(sourceId: number) {
  return `source:${sourceId}`;
}

function countLabel(count: number) {
  if (count === 1) return "1 item";
  return `${count} items`;
}

function dateLabel(unixSeconds: number | null) {
  if (!unixSeconds) return null;
  return new Intl.DateTimeFormat("en-US", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(unixSeconds * 1000));
}

function typeLabel(provider: LibrarySourceProvider, subtype: LibrarySourceSubtype) {
  const providerLabel = PROVIDER_LABELS[provider] ?? provider;
  if (!subtype) return `${providerLabel} source`;
  return `${providerLabel} / ${SUBTYPE_LABELS[subtype] ?? subtype}`;
}

export function buildLibraryCatalogSourcesView(
  records: LibraryCatalogRecord[],
): LibraryCatalogSourceView[] {
  return records.map((record) => ({
    id: sourceRowId(record.source.source_id),
    sourceId: record.source.source_id,
    provider: record.source.provider,
    sourceSubtype: record.source.source_subtype,
    title: record.source.title ?? `Source #${record.source.source_id}`,
    subtitle: record.source.subtitle,
    typeLabel: typeLabel(record.source.provider, record.source.source_subtype),
    status: record.status,
    statusDetail: record.status_detail,
    projectCount: record.source.project_count,
    itemCount: record.source.item_count,
    itemCountLabel: countLabel(record.source.item_count),
    addedAtLabel: dateLabel(record.source.created_at) ?? "Unknown",
    lastSyncedLabel: dateLabel(record.source.last_synced_at) ?? "Never",
    canonicalUrl: record.source.canonical_url,
    externalId: record.source.external_id,
    youtube: record.source.youtube,
    telegram: record.source.telegram,
  }));
}

function countProviderFromBackend(
  counts: LibraryCatalogFilterCount[],
  provider: LibrarySourceProvider,
) {
  return counts
    .filter((count) => count.provider === provider)
    .reduce((total, count) => total + count.count, 0);
}

function backendSubtypeRow(
  counts: LibraryCatalogFilterCount[],
  provider: LibrarySourceProvider,
  subtype: Exclude<LibrarySourceSubtype, null>,
  label: string,
): LibraryCatalogFilterTreeRow {
  const backendCount = counts.find(
    (count) => count.provider === provider && count.source_subtype === subtype,
  );
  return {
    id: `provider:${provider}/subtype:${subtype}` as LibraryCatalogFilterId,
    label,
    provider,
    subtype,
    count: backendCount?.count ?? 0,
    disabled: backendCount?.disabled ?? false,
    disabledReason: backendCount?.disabled_reason ?? undefined,
  };
}

export function buildLibraryCatalogFilterTree(
  counts: LibraryCatalogFilterCount[],
): LibraryCatalogFilterTreeRow[] {
  const total = counts.reduce((sum, count) => sum + count.count, 0);
  return [
    {
      id: LIBRARY_CATALOG_ALL_FILTER_ID,
      label: "All sources",
      provider: "all",
      count: total,
    },
    {
      id: "provider:youtube",
      label: "YouTube",
      provider: "youtube",
      count: countProviderFromBackend(counts, "youtube"),
      data: [
        backendSubtypeRow(counts, "youtube", "video", "Videos"),
        backendSubtypeRow(counts, "youtube", "playlist", "Playlists"),
        backendSubtypeRow(counts, "youtube", "channel", "Channels"),
      ],
    },
    {
      id: "provider:telegram",
      label: "Telegram",
      provider: "telegram",
      count: countProviderFromBackend(counts, "telegram"),
      data: [
        backendSubtypeRow(counts, "telegram", "channel", "Channels"),
        backendSubtypeRow(counts, "telegram", "supergroup", "Supergroups"),
        backendSubtypeRow(counts, "telegram", "group", "Groups"),
      ],
    },
  ];
}

function filterParts(filterId: LibraryCatalogFilterId): {
  provider: LibrarySourceProvider | null;
  subtype: Exclude<LibrarySourceSubtype, null> | null;
} {
  if (filterId === LIBRARY_CATALOG_ALL_FILTER_ID) return { provider: null, subtype: null };
  const [providerPart, subtypePart] = filterId.split("/subtype:");
  const provider = providerPart.replace("provider:", "") as LibrarySourceProvider;
  return {
    provider,
    subtype: subtypePart ? (subtypePart as Exclude<LibrarySourceSubtype, null>) : null,
  };
}

export function filterLibraryCatalogSources(
  sources: LibraryCatalogSourceView[],
  filters: LibraryCatalogFilterState,
) {
  const query = filters.query.trim().toLocaleLowerCase();
  const { provider, subtype } = filterParts(filters.filterId);

  return sources.filter((source) => {
    const matchesProvider = !provider || source.provider === provider;
    const matchesSubtype = !subtype || source.sourceSubtype === subtype;
    const matchesQuery =
      !query ||
      `${source.title} ${source.subtitle ?? ""} ${source.typeLabel} ${source.externalId ?? ""}`
        .toLocaleLowerCase()
        .includes(query);
    return matchesProvider && matchesSubtype && matchesQuery;
  });
}

export function reconcileLibraryCatalogSourceSelection(
  sources: LibraryCatalogSourceView[],
  selectedSourceId: string | null,
) {
  if (selectedSourceId && sources.some((source) => source.id === selectedSourceId)) {
    return selectedSourceId;
  }
  return sources[0]?.id ?? null;
}
