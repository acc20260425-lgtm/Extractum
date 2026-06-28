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

export interface SourceRowView {
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
