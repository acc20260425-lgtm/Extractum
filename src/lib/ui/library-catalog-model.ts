import type {
  LibrarySourceProvider,
  LibrarySourceRecord,
  LibrarySourceSubtype,
  LibraryTelegramSourceDetails,
  LibraryYoutubeSourceDetails,
} from "$lib/types/library-sources";
import type { SourceJobRecord } from "$lib/types/sources";

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
export const YOUTUBE_CHANNEL_DISABLED_REASON =
  "YouTube channel sources are not supported by the current backend.";

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

function latestJobBySource(sourceJobs: SourceJobRecord[]) {
  const jobsBySource = new Map<number, SourceJobRecord>();
  for (const job of sourceJobs) {
    if (job.status !== "queued" && job.status !== "running" && job.status !== "failed") continue;
    const current = jobsBySource.get(job.source_id);
    if (!current || job.started_at > current.started_at) jobsBySource.set(job.source_id, job);
  }
  return jobsBySource;
}

function statusFromJob(job: SourceJobRecord | undefined): {
  status: LibraryCatalogSourceStatus;
  statusDetail: string | null;
} {
  if (!job) return { status: "active", statusDetail: null };
  if (job.status === "queued" || job.status === "running") {
    return { status: "syncing", statusDetail: job.message ?? "Syncing" };
  }
  if (job.status === "failed") {
    return { status: "error", statusDetail: job.error ?? "Last sync failed" };
  }
  return { status: "active", statusDetail: null };
}

export function buildLibraryCatalogSourcesView(
  records: LibrarySourceRecord[],
  sourceJobs: SourceJobRecord[] = [],
): LibraryCatalogSourceView[] {
  const jobsBySource = latestJobBySource(sourceJobs);

  return records.map((record) => {
    const jobStatus = statusFromJob(jobsBySource.get(record.source_id));
    return {
      id: sourceRowId(record.source_id),
      sourceId: record.source_id,
      provider: record.provider,
      sourceSubtype: record.source_subtype,
      title: record.title ?? `Source #${record.source_id}`,
      subtitle: record.subtitle,
      typeLabel: typeLabel(record.provider, record.source_subtype),
      status: jobStatus.status,
      statusDetail: jobStatus.statusDetail,
      projectCount: record.project_count,
      itemCount: record.item_count,
      itemCountLabel: countLabel(record.item_count),
      addedAtLabel: dateLabel(record.created_at) ?? "Unknown",
      lastSyncedLabel: dateLabel(record.last_synced_at) ?? "Never",
      canonicalUrl: record.canonical_url,
      externalId: record.external_id,
      youtube: record.youtube,
      telegram: record.telegram,
    };
  });
}

function countProvider(sources: LibraryCatalogSourceView[], provider: LibrarySourceProvider) {
  return sources.filter((source) => source.provider === provider).length;
}

function countSubtype(
  sources: LibraryCatalogSourceView[],
  provider: LibrarySourceProvider,
  subtype: Exclude<LibrarySourceSubtype, null>,
) {
  return sources.filter((source) => source.provider === provider && source.sourceSubtype === subtype)
    .length;
}

function subtypeRow(
  sources: LibraryCatalogSourceView[],
  provider: LibrarySourceProvider,
  subtype: Exclude<LibrarySourceSubtype, null>,
  label: string,
  disabled = false,
  disabledReason: string | null = null,
): LibraryCatalogFilterTreeRow {
  return {
    id: `provider:${provider}/subtype:${subtype}` as LibraryCatalogFilterId,
    label,
    provider,
    subtype,
    count: countSubtype(sources, provider, subtype),
    disabled,
    disabledReason: disabledReason ?? undefined,
  };
}

export function buildLibraryCatalogFilterTree(
  sources: LibraryCatalogSourceView[],
): LibraryCatalogFilterTreeRow[] {
  return [
    {
      id: LIBRARY_CATALOG_ALL_FILTER_ID,
      label: "All sources",
      provider: "all",
      count: sources.length,
    },
    {
      id: "provider:youtube",
      label: "YouTube",
      provider: "youtube",
      count: countProvider(sources, "youtube"),
      data: [
        subtypeRow(sources, "youtube", "video", "Videos"),
        subtypeRow(sources, "youtube", "playlist", "Playlists"),
        subtypeRow(
          sources,
          "youtube",
          "channel",
          "Channels",
          true,
          YOUTUBE_CHANNEL_DISABLED_REASON,
        ),
      ],
    },
    {
      id: "provider:telegram",
      label: "Telegram",
      provider: "telegram",
      count: countProvider(sources, "telegram"),
      data: [
        subtypeRow(sources, "telegram", "channel", "Channels"),
        subtypeRow(sources, "telegram", "supergroup", "Supergroups"),
        subtypeRow(sources, "telegram", "group", "Groups"),
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
