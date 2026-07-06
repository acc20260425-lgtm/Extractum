import { describe, expect, it } from "vitest";
import {
  compareLibraryCatalogCreatedAt,
  compareLibraryCatalogItemCounts,
  compareLibraryCatalogLastSyncedAt,
  compareLibraryCatalogProjectCounts,
  compareLibraryCatalogTitles,
  libraryCatalogGridColumns,
} from "./library-catalog-grid";

describe("library catalog grid columns", () => {
  it("enables sorting for every visible Library source column", () => {
    const columns = libraryCatalogGridColumns(undefined);
    const byId = new Map(columns.map((column) => [String(column.id), column]));

    expect(byId.get("title")?.sort).toBe(compareLibraryCatalogTitles);
    expect(byId.get("typeLabel")?.sort).toBe(true);
    expect(byId.get("status")?.sort).toBe(true);
    expect(byId.get("projectCount")?.sort).toBe(compareLibraryCatalogProjectCounts);
    expect(byId.get("itemCountLabel")?.sort).toBe(compareLibraryCatalogItemCounts);
    expect(byId.get("createdAt")?.sort).toBe(compareLibraryCatalogCreatedAt);
    expect(byId.get("lastSyncedAt")?.sort).toBe(compareLibraryCatalogLastSyncedAt);
  });

  it("keeps the Library source table column contract", () => {
    const columns = libraryCatalogGridColumns(undefined);

    expect(columns.map((column) => column.id)).toEqual([
      "title",
      "typeLabel",
      "status",
      "projectCount",
      "itemCountLabel",
      "createdAt",
      "lastSyncedAt",
    ]);
    expect(columns.find((column) => column.id === "createdAt")?.dateTimeFormat).toBe("datetime");
    expect(columns.find((column) => column.id === "lastSyncedAt")?.dateTimeFormat).toBe(
      "datetime",
    );
  });

  it("orders title with Russian-aware case-insensitive comparison", () => {
    const rows = [{ title: "бета" }, { title: "Альфа" }];

    expect([...rows].sort(compareLibraryCatalogTitles).map((row) => row.title)).toEqual([
      "Альфа",
      "бета",
    ]);
  });

  it("orders Projects and Items by raw numeric values", () => {
    const rows = [
      { projectCount: 10, itemCount: 2, itemCountLabel: "2 items" },
      { projectCount: 2, itemCount: 100, itemCountLabel: "100 items" },
    ];

    expect([...rows].sort(compareLibraryCatalogProjectCounts).map((row) => row.projectCount)).toEqual([
      2,
      10,
    ]);
    expect([...rows].sort(compareLibraryCatalogItemCounts).map((row) => row.itemCount)).toEqual([
      2,
      100,
    ]);
  });

  it("orders dates by raw timestamps and treats missing dates as oldest", () => {
    const rows = [{ createdAt: 20, lastSyncedAt: null }, { createdAt: null, lastSyncedAt: 10 }];

    expect([...rows].sort(compareLibraryCatalogCreatedAt).map((row) => row.createdAt)).toEqual([
      null,
      20,
    ]);
    expect([...rows].sort(compareLibraryCatalogLastSyncedAt).map((row) => row.lastSyncedAt)).toEqual([
      null,
      10,
    ]);
  });
});
