import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui";

type SortResult = -1 | 0 | 1;
type RowLike = Record<string, unknown>;

function sign(value: number): SortResult {
  return value < 0 ? -1 : value > 0 ? 1 : 0;
}

function numericValue(value: unknown): number {
  return typeof value === "number" && Number.isFinite(value) ? value : -Infinity;
}

export function compareLibraryCatalogTitles(left: RowLike, right: RowLike): SortResult {
  return sign(
    String(left.title ?? "").localeCompare(String(right.title ?? ""), "ru", {
      sensitivity: "base",
    }),
  );
}

export function compareLibraryCatalogProjectCounts(left: RowLike, right: RowLike): SortResult {
  return sign(numericValue(left.projectCount) - numericValue(right.projectCount));
}

export function compareLibraryCatalogItemCounts(left: RowLike, right: RowLike): SortResult {
  return sign(numericValue(left.itemCount) - numericValue(right.itemCount));
}

export function compareLibraryCatalogCreatedAt(left: RowLike, right: RowLike): SortResult {
  return sign(numericValue(left.createdAt) - numericValue(right.createdAt));
}

export function compareLibraryCatalogLastSyncedAt(left: RowLike, right: RowLike): SortResult {
  return sign(numericValue(left.lastSyncedAt) - numericValue(right.lastSyncedAt));
}

export function libraryCatalogGridColumns(
  titleCell: ExtractumDataGridColumn["cell"],
): ExtractumDataGridColumn[] {
  return [
    {
      id: "title",
      header: "Source",
      width: 320,
      cell: titleCell,
      sort: compareLibraryCatalogTitles,
    },
    { id: "typeLabel", header: "Type", width: 150, sort: true },
    { id: "status", header: "Status", width: 110, sort: true },
    {
      id: "projectCount",
      header: "Projects",
      width: 92,
      sort: compareLibraryCatalogProjectCounts,
    },
    {
      id: "itemCountLabel",
      header: "Items",
      width: 100,
      sort: compareLibraryCatalogItemCounts,
    },
    {
      id: "createdAt",
      header: "Added",
      width: 136,
      dateTimeFormat: "datetime",
      sort: compareLibraryCatalogCreatedAt,
    },
    {
      id: "lastSyncedAt",
      header: "Last synced",
      width: 136,
      dateTimeFormat: "datetime",
      sort: compareLibraryCatalogLastSyncedAt,
    },
  ];
}
