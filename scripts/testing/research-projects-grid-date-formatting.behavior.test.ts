import { expect, it } from "vitest";

import * as projectSourceGrid from "../../src/lib/ui/research-projects-project-source-grid";

it("project source date columns use the shared formatter", () => {
  const columns = projectSourceGrid.projectSourceGridColumns(undefined);
  expect(columns.find((column) => column.id === "addedAt")).toMatchObject({
    header: "Added to project at",
    dateTimeFormat: "datetime",
  });

  const connectFromLibraryGridColumns = (projectSourceGrid as {
    connectFromLibraryGridColumns?: () => Array<{ id: string; dateTimeFormat?: string }>;
  }).connectFromLibraryGridColumns;
  expect(connectFromLibraryGridColumns).toBeTypeOf("function");
  expect(connectFromLibraryGridColumns?.().find((column) => column.id === "lastCollectedAt"))
    .toMatchObject({ dateTimeFormat: "datetime" });
});
