import type { ProjectSourceRecord } from "$lib/types/projects";

export interface SourceFilters {
  query: string;
  types: string[];
  statuses: string[];
  materialsMin: number | null;
  materialsMax: number | null;
  /** "YYYY-MM-DD" (значение input type=date) */
  syncedFrom: string | null;
  syncedTo: string | null;
}

export function emptySourceFilters(): SourceFilters {
  return {
    query: "",
    types: [],
    statuses: [],
    materialsMin: null,
    materialsMax: null,
    syncedFrom: null,
    syncedTo: null,
  };
}

const PROVIDER_DOTS: Record<string, string> = {
  telegram: "var(--extractum-provider-telegram)",
  youtube: "var(--extractum-provider-youtube)",
};

const STATUS_DOTS: Record<string, string> = {
  active: "var(--extractum-success)",
  syncing: "var(--extractum-primary)",
  error: "var(--extractum-danger)",
  unavailable: "var(--extractum-warning)",
};

/** Начало локального дня в unix-секундах. */
function dayStart(iso: string): number {
  return new Date(`${iso}T00:00:00`).getTime() / 1000;
}

/** Конец локального дня (включительно) в unix-секундах. */
function dayEnd(iso: string): number {
  return dayStart(iso) + 86_399;
}

export function countActiveSourceFilters(filters: SourceFilters): number {
  return buildSourceFilterChips(filters).length;
}

export function filterProjectSources(
  records: ProjectSourceRecord[],
  filters: SourceFilters,
): ProjectSourceRecord[] {
  const q = filters.query.trim().toLowerCase();
  return records.filter((record) => {
    if (q) {
      const title = (record.title ?? "").toLowerCase();
      const handle = (record.handle ?? "").toLowerCase();
      if (!title.includes(q) && !handle.includes(q)) return false;
    }
    if (filters.types.length > 0 && !filters.types.includes(record.provider)) return false;
    if (filters.statuses.length > 0 && !filters.statuses.includes(record.sync_status)) return false;
    if (filters.materialsMin !== null && record.item_count < filters.materialsMin) return false;
    if (filters.materialsMax !== null && record.item_count > filters.materialsMax) return false;
    if (filters.syncedFrom !== null || filters.syncedTo !== null) {
      if (record.last_synced_at === null) return false;
      if (filters.syncedFrom !== null && record.last_synced_at < dayStart(filters.syncedFrom)) {
        return false;
      }
      if (filters.syncedTo !== null && record.last_synced_at > dayEnd(filters.syncedTo)) {
        return false;
      }
    }
    return true;
  });
}

export interface SourceFilterChip {
  key: string;
  label: string;
  dot: string | null;
}

function dateLabel(iso: string): string {
  const [y, m, d] = iso.split("-");
  return `${d}.${m}.${y}`;
}

export function buildSourceFilterChips(filters: SourceFilters): SourceFilterChip[] {
  const chips: SourceFilterChip[] = [];
  if (filters.query.trim()) {
    chips.push({ key: "query", label: `Источник: ${filters.query.trim()}`, dot: null });
  }
  for (const type of filters.types) {
    chips.push({ key: `type:${type}`, label: `Тип: ${type}`, dot: PROVIDER_DOTS[type] ?? null });
  }
  for (const status of filters.statuses) {
    chips.push({
      key: `status:${status}`,
      label: `Статус: ${status}`,
      dot: STATUS_DOTS[status] ?? null,
    });
  }
  if (filters.materialsMin !== null || filters.materialsMax !== null) {
    const min = filters.materialsMin ?? 0;
    const max = filters.materialsMax ?? "∞";
    chips.push({ key: "materials", label: `Материалы: ${min}–${max}`, dot: null });
  }
  if (filters.syncedFrom !== null || filters.syncedTo !== null) {
    const from = filters.syncedFrom ? dateLabel(filters.syncedFrom) : "…";
    const to = filters.syncedTo ? dateLabel(filters.syncedTo) : "…";
    chips.push({ key: "period", label: `Период: ${from}–${to}`, dot: null });
  }
  return chips;
}

export function removeSourceFilterChip(filters: SourceFilters, key: string): SourceFilters {
  if (key === "query") return { ...filters, query: "" };
  if (key === "materials") return { ...filters, materialsMin: null, materialsMax: null };
  if (key === "period") return { ...filters, syncedFrom: null, syncedTo: null };
  if (key.startsWith("type:")) {
    const value = key.slice("type:".length);
    return { ...filters, types: filters.types.filter((t) => t !== value) };
  }
  if (key.startsWith("status:")) {
    const value = key.slice("status:".length);
    return { ...filters, statuses: filters.statuses.filter((s) => s !== value) };
  }
  return filters;
}
