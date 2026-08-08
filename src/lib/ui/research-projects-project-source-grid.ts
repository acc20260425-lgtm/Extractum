import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui";

type SortResult = -1 | 0 | 1;
type RowLike = Record<string, unknown>;

function sign(value: number): SortResult {
  return value < 0 ? -1 : value > 0 ? 1 : 0;
}

function numericValue(value: unknown): number {
  return typeof value === "number" && Number.isFinite(value) ? value : -Infinity;
}

export function sharedProjectDateColumn<T extends ExtractumDataGridColumn>(
  column: T,
): T & { dateTimeFormat: "datetime" } {
  return { ...column, dateTimeFormat: "datetime" };
}

export function connectFromLibraryGridColumns(
  selectCell: ExtractumDataGridColumn["cell"],
  titleCell: ExtractumDataGridColumn["cell"],
): ExtractumDataGridColumn[] {
  return [
    { id: "selected", header: "", width: 44, cell: selectCell },
    { id: "title", header: "Источник", width: 260, cell: titleCell },
    { id: "typeLabel", header: "Тип", width: 150 },
    { id: "projectCount", header: "Проекты", width: 80 },
    sharedProjectDateColumn({
      id: "lastCollectedAt",
      header: "Последний сбор",
      width: 140,
    }),
    { id: "localCopyLabel", header: "Локальная копия", width: 120 },
    { id: "status", header: "Статус", width: 100 },
  ];
}

export function compareProjectSourceTitles(left: RowLike, right: RowLike): SortResult {
  return sign(
    String(left.title ?? "").localeCompare(String(right.title ?? ""), "ru", {
      sensitivity: "base",
    }),
  );
}

export function compareProjectSourceMaterialLabels(left: RowLike, right: RowLike): SortResult {
  const materialCount = (row: RowLike) =>
    Number(String(row.localCopyLabel ?? "").replace(/\D/g, "")) || 0;
  return sign(materialCount(left) - materialCount(right));
}

export function compareProjectSourceAddedAt(left: RowLike, right: RowLike): SortResult {
  return sign(numericValue(left.addedAt) - numericValue(right.addedAt));
}

export function projectSourceGridColumns(
  titleCell: ExtractumDataGridColumn["cell"],
): ExtractumDataGridColumn[] {
  return [
    {
      id: "title",
      header: "Title",
      width: 260,
      flexgrow: 1,
      cell: titleCell,
      sort: compareProjectSourceTitles,
    },
    { id: "typeLabel", header: "Type", width: 150, sort: true },
    {
      id: "localCopyLabel",
      header: "Details",
      width: 140,
      sort: compareProjectSourceMaterialLabels,
    },
    sharedProjectDateColumn({
      id: "addedAt",
      header: "Added to project at",
      width: 180,
      sort: compareProjectSourceAddedAt,
    }),
    // Retained legacy marker; sharedProjectDateColumn supplies dateTimeFormat: "datetime".
  ];
}
