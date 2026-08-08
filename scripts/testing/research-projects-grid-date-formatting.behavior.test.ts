import { expect, it } from "vitest";

import * as projectSourceGrid from "../../src/lib/ui/research-projects-project-source-grid";

it("project source date columns use the shared formatter", () => {
  const columns = projectSourceGrid.projectSourceGridColumns(undefined);
  expect(columns.find((column) => column.id === "addedAt")).toMatchObject({
    header: "Added to project at",
    dateTimeFormat: "datetime",
  });

  const sharedProjectDateColumn = (projectSourceGrid as {
    sharedProjectDateColumn?: <T extends { id: string }>(column: T) => T & { dateTimeFormat: string };
  }).sharedProjectDateColumn;
  expect(sharedProjectDateColumn).toBeTypeOf("function");
  expect(sharedProjectDateColumn?.({ id: "lastCollectedAt" }))
    .toEqual({ id: "lastCollectedAt", dateTimeFormat: "datetime" });

  const selectCell = { component: "select" };
  const titleCell = { component: "title" };
  const libraryColumns = projectSourceGrid.connectFromLibraryGridColumns(selectCell, titleCell);
  expect(libraryColumns.map((column) => column.id)).toEqual([
    "selected",
    "title",
    "typeLabel",
    "projectCount",
    "lastCollectedAt",
    "localCopyLabel",
    "status",
  ]);
  expect(libraryColumns[0]?.cell).toBe(selectCell);
  expect(libraryColumns[1]?.cell).toBe(titleCell);
  expect(libraryColumns[4]).toMatchObject({
    id: "lastCollectedAt",
    dateTimeFormat: "datetime",
  });
});
