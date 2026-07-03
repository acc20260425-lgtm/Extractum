import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui/data-grid-date-format";
import type { LibraryCatalogStatus, LibrarySourceProvider } from "$lib/types/library-sources";
import type { ProjectSourceRecord } from "$lib/types/projects";

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

// svar column.sort receives raw CELL VALUES and requires a strict 0|1|-1 result.
// Comparators are exported for unit tests.
type SortResult = 0 | 1 | -1;

function sign(value: number): SortResult {
  return value < 0 ? -1 : value > 0 ? 1 : 0;
}

export function compareRuStrings(a: unknown, b: unknown): SortResult {
  return sign(String(a ?? "").localeCompare(String(b ?? ""), "ru", { sensitivity: "base" }));
}

export function compareMaterialsLabels(a: unknown, b: unknown): SortResult {
  const num = (v: unknown) => Number(String(v ?? "").replace(/\D/g, "")) || 0;
  return sign(num(a) - num(b));
}

// null = "oldest": sinks below real dates on ascending sort (and inverts with desc,
// which svar does by negating the comparator).
export function compareNullableTimestamps(a: unknown, b: unknown): SortResult {
  const num = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : -Infinity);
  const x = num(a);
  const y = num(b);
  return x < y ? -1 : x > y ? 1 : 0;
}

export function sourceGridColumns(): ExtractumDataGridColumn[] {
  return [
    { id: "title", header: "Источник", width: 260, flexgrow: 1, sort: compareRuStrings },
    { id: "typeLabel", header: "Тип", width: 116, sort: true },
    { id: "materialsLabel", header: "Материалы", width: 116, sort: compareMaterialsLabels },
    {
      id: "lastSyncedAt",
      header: "Последний сбор",
      width: 150,
      dateTimeFormat: "datetime",
      sort: compareNullableTimestamps,
    },
    { id: "statusLabel", header: "Статус", width: 104, sort: true },
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
