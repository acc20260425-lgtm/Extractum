import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui/data-grid-date-format";
import type { LibraryCatalogStatus, LibrarySourceProvider } from "$lib/types/library-sources";
import type { ProjectSourceRecord } from "$lib/types/projects";

export const SOURCE_TABLE_LAYOUT = {
  select: 34,
  titleMin: 160,
  titleFlexGrow: 1,
  type: 116,
  materials: 116,
  lastSync: 150,
  status: 104,
} as const;

export const SOURCE_FILTER_ROW_GRID_TEMPLATE = [
  `${SOURCE_TABLE_LAYOUT.select}px`,
  `minmax(${SOURCE_TABLE_LAYOUT.titleMin}px, 1fr)`,
  `${SOURCE_TABLE_LAYOUT.type}px`,
  `${SOURCE_TABLE_LAYOUT.materials}px`,
  `${SOURCE_TABLE_LAYOUT.lastSync}px`,
  `${SOURCE_TABLE_LAYOUT.status}px`,
].join(" ");

const SYNC_STATUS_LABELS: Record<LibraryCatalogStatus, string> = {
  active: "active",
  syncing: "sync",
  error: "error",
  unavailable: "unavailable",
};

export function sourceSyncStatusLabel(status: LibraryCatalogStatus): string {
  return SYNC_STATUS_LABELS[status];
}

export type SourceRowView = {
  sourceId: number;
  title: string;
  handle: string | null;
  provider: LibrarySourceProvider;
  typeLabel: string;
  materialsLabel: string;
  lastSyncedAt: number | null;
  syncStatus: LibraryCatalogStatus;
  statusLabel: string;
}

function formatThousands(count: number): string {
  return String(count).replace(/\B(?=(\d{3})+(?!\d))/g, " ");
}

// svar column.sort comparators receive ROW OBJECTS (verified live against the
// installed @svar-ui/svelte-grid; newer docs describe cell values) and must
// return a strict 0|1|-1. Comparators are exported for unit tests.
type SortResult = 0 | 1 | -1;

function sign(value: number): SortResult {
  return value < 0 ? -1 : value > 0 ? 1 : 0;
}

type RowLike = Record<string, unknown>;

export function compareSourceTitles(a: RowLike, b: RowLike): SortResult {
  return sign(
    String(a.title ?? "").localeCompare(String(b.title ?? ""), "ru", { sensitivity: "base" }),
  );
}

export function compareSourceMaterials(a: RowLike, b: RowLike): SortResult {
  const num = (row: RowLike) => Number(String(row.materialsLabel ?? "").replace(/\D/g, "")) || 0;
  return sign(num(a) - num(b));
}

// null = "oldest": sinks below real dates on ascending sort (and inverts with desc,
// which svar does by negating the comparator).
export function compareSourceLastSynced(a: RowLike, b: RowLike): SortResult {
  const num = (row: RowLike) => {
    const value = row.lastSyncedAt;
    return typeof value === "number" && Number.isFinite(value) ? value : -Infinity;
  };
  const x = num(a);
  const y = num(b);
  return x < y ? -1 : x > y ? 1 : 0;
}

export function sourceGridColumns(): ExtractumDataGridColumn[] {
  return [
    {
      id: "title",
      header: "Источник",
      flexgrow: SOURCE_TABLE_LAYOUT.titleFlexGrow,
      sort: compareSourceTitles,
    },
    { id: "typeLabel", header: "Тип", width: SOURCE_TABLE_LAYOUT.type, sort: true },
    {
      id: "materialsLabel",
      header: "Материалы",
      width: SOURCE_TABLE_LAYOUT.materials,
      sort: compareSourceMaterials,
    },
    {
      id: "lastSyncedAt",
      header: "Последний сбор",
      width: SOURCE_TABLE_LAYOUT.lastSync,
      dateTimeFormat: "datetime",
      sort: compareSourceLastSynced,
    },
    { id: "statusLabel", header: "Статус", width: SOURCE_TABLE_LAYOUT.status, sort: true },
  ];
}

export type SourceGridRow = SourceRowView & {
  id: string;
};

export function buildSourceGridRows(records: ProjectSourceRecord[]): SourceGridRow[] {
  return records.map((record) => {
    const view = buildSourceRow(record);
    return { ...view, id: String(view.sourceId) };
  });
}

export function buildSourceRow(record: ProjectSourceRecord): SourceRowView {
  return {
    sourceId: record.source_id,
    title: record.title ?? `Источник #${record.source_id}`,
    handle: record.handle,
    provider: record.provider,
    typeLabel: record.provider,
    materialsLabel: formatThousands(record.item_count),
    lastSyncedAt: record.last_synced_at,
    syncStatus: record.sync_status,
    statusLabel: sourceSyncStatusLabel(record.sync_status),
  };
}
