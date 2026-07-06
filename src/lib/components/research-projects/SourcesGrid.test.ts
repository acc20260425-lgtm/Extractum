import { describe, expect, it } from "vitest";
import rawSource from "./SourcesGrid.svelte?raw";

// svar's DataGrid needs ResizeObserver/measurement APIs that jsdom lacks, so —
// matching the project convention for grid shells — the wrapper is verified by
// source assertions. The column/row logic itself is covered by pure unit tests
// in research-projects-source-row.test.ts.
const source = rawSource.replace(/\r\n/g, "\n");

describe("SourcesGrid", () => {
  it("wires the v10 source columns and rows into the data grid", () => {
    expect(source).toContain("buildSourceGridRows");
    expect(source).toContain("sourceGridColumns");
    expect(source).toContain("SOURCE_TABLE_LAYOUT");
    expect(source).toContain("width: SOURCE_TABLE_LAYOUT.select");
    expect(source).toContain("<ExtractumDataGrid");
  });

  it("keeps SVAR as the source table implementation", () => {
    expect(source).toContain("<ExtractumDataGrid");
    expect(source).not.toContain("<table");
  });

  it("labels the grid and provides an empty overlay in Russian", () => {
    expect(source).toContain('ariaLabel="Источники проекта"');
    expect(source).toContain('overlay = "Нет источников"');
    expect(source).toContain("{overlay}");
  });

  it("attaches the custom cells for the source title and status columns", () => {
    expect(source).toContain("SourceTitleCell");
    expect(source).toContain("SourceStatusCell");
    expect(source).toContain("title: SourceTitleCell");
    expect(source).toContain("statusLabel: SourceStatusCell");
  });

  it("right-aligns the materials column via columnStyle (svar has no column align)", () => {
    expect(source).toContain("{columnStyle}");
    expect(source).toContain('"materialsLabel"');
    expect(source).toContain("extractum-grid-cell-right");
  });

  it("separates row activation from checkbox selection (v10)", () => {
    expect(source).toContain("selectOnClick={false}");
    expect(source).toContain("activeRowId={activeSourceId}");
    expect(source).toContain("onRowClick={onActivateSource}");
  });

  it("adds a leading checkbox select column with a tri-state select-all header", () => {
    expect(source).toContain("SELECT_COLUMN");
    expect(source).toContain("cell: GridSelectCell");
    expect(source).toContain("cell: GridSelectAllCell");
    // per-row checkbox reads selection reactively from the grid api,
    // so rows no longer depend on selectedSourceIds (sort survives selection)
    expect(source).not.toContain("selected: selectedSourceIds.includes(row.id)");
  });
});
