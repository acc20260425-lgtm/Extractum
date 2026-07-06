import { describe, expect, it } from "vitest";
import {
  compareProjectSourceAddedAt,
  compareProjectSourceMaterialLabels,
  compareProjectSourceTitles,
  projectSourceGridColumns,
} from "./research-projects-project-source-grid";

describe("projectSourceGridColumns", () => {
  it("enables sorting for every visible project source column", () => {
    const columns = projectSourceGridColumns(undefined);
    const byId = new Map(columns.map((column) => [String(column.id), column]));

    expect(byId.get("title")?.sort).toBe(compareProjectSourceTitles);
    expect(byId.get("typeLabel")?.sort).toBe(true);
    expect(byId.get("localCopyLabel")?.sort).toBe(compareProjectSourceMaterialLabels);
    expect(byId.get("addedAt")?.sort).toBe(compareProjectSourceAddedAt);
  });

  it("keeps the existing column labels and date formatting", () => {
    const columns = projectSourceGridColumns(undefined);

    expect(columns.map((column) => column.id)).toEqual([
      "title",
      "typeLabel",
      "localCopyLabel",
      "addedAt",
    ]);
    expect(columns.map((column) => column.header)).toEqual([
      "Title",
      "Type",
      "Details",
      "Added to project at",
    ]);
    expect(columns.find((column) => column.id === "addedAt")?.dateTimeFormat).toBe("datetime");
  });
});

describe("project source sort comparators", () => {
  it("orders titles case-insensitively", () => {
    expect(compareProjectSourceTitles({ title: "alpha" }, { title: "Beta" })).toBeLessThan(0);
    expect(compareProjectSourceTitles({ title: "Gamma" }, { title: "beta" })).toBeGreaterThan(0);
    expect(compareProjectSourceTitles({ title: "AI" }, { title: "ai" })).toBe(0);
  });

  it("orders Details by numeric material count", () => {
    expect(
      compareProjectSourceMaterialLabels(
        { localCopyLabel: "10 materials" },
        { localCopyLabel: "2 materials" },
      ),
    ).toBeGreaterThan(0);
    expect(
      compareProjectSourceMaterialLabels(
        { localCopyLabel: "1 material" },
        { localCopyLabel: "10 materials" },
      ),
    ).toBeLessThan(0);
    expect(
      compareProjectSourceMaterialLabels(
        { localCopyLabel: "3 materials" },
        { localCopyLabel: "3 materials" },
      ),
    ).toBe(0);
  });

  it("orders Added to project at by raw timestamp", () => {
    expect(compareProjectSourceAddedAt({ addedAt: 100 }, { addedAt: 200 })).toBeLessThan(0);
    expect(compareProjectSourceAddedAt({ addedAt: 300 }, { addedAt: 200 })).toBeGreaterThan(0);
    expect(compareProjectSourceAddedAt({ addedAt: 200 }, { addedAt: 200 })).toBe(0);
  });
});
