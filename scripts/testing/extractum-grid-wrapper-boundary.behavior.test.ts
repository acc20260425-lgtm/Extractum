import { expect, it } from "vitest";

import * as gridContract from "../../src/lib/components/extractum-ui/data-grid-date-format";

it("SVAR grid APIs stay inside Extractum wrappers", () => {
  const contract = (gridContract as {
    EXTRACTUM_GRID_WRAPPER_CONTRACT?: Record<string, unknown>;
  }).EXTRACTUM_GRID_WRAPPER_CONTRACT;

  expect(contract).toEqual({
    package: "@svar-ui/svelte-grid",
    publicWrappers: ["ExtractumDataGrid", "ExtractumTreeDataGrid"],
    dataGrid: {
      selection: "selectedRows",
      rowStyle: true,
      locale: true,
      theme: "Willow",
      fonts: false,
      emptyOverlay: "visibleOverlay",
      dateTimeColumns: true,
      responsiveDateTimeColumns: true,
    },
    treeDataGrid: {
      tree: true,
      toggleEvent: "treetoggle",
      selection: "selectedRows",
      selectionEvent: "onselectrow",
      locale: true,
      theme: "Willow",
      fonts: false,
    },
    selectCell: {
      ignoredClickAttribute: "data-action",
      ignoredClickValue: "ignore-click",
      selectionCommand: "select-row",
    },
  });
});
