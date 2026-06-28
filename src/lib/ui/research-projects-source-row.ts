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

export function sourceGridColumns(): ExtractumDataGridColumn[] {
  return [
    { id: "title", header: "Источник", width: 260, flexgrow: 1 },
    { id: "typeLabel", header: "Тип", width: 116 },
    { id: "materialsLabel", header: "Материалы", width: 116 },
    { id: "lastSyncedAt", header: "Последний сбор", width: 150, dateTimeFormat: "datetime" },
    { id: "statusLabel", header: "Статус", width: 104 },
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
