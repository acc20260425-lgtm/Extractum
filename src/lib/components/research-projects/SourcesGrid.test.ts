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
    expect(source).toContain("<ExtractumDataGrid");
  });

  it("labels the grid and provides an empty overlay in Russian", () => {
    expect(source).toContain('ariaLabel="Источники проекта"');
    expect(source).toContain('overlay="Нет источников"');
  });
});
