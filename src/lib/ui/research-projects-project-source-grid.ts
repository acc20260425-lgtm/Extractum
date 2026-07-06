import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui";

type SortResult = -1 | 0 | 1;
type RowLike = Record<string, unknown>;

function sign(value: number): SortResult {
  return value < 0 ? -1 : value > 0 ? 1 : 0;
}

function numericValue(value: unknown): number {
  return typeof value === "number" && Number.isFinite(value) ? value : -Infinity;
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
    {
      id: "addedAt",
      header: "Added to project at",
      width: 180,
      dateTimeFormat: "datetime",
      sort: compareProjectSourceAddedAt,
    },
  ];
}
