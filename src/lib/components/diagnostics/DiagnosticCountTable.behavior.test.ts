import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, it } from "vitest";
import DiagnosticCountTable from "./DiagnosticCountTable.svelte";

afterEach(cleanup);

it("DiagnosticCountTable > expands and collapses large tables", async () => {
  render(DiagnosticCountTable, {
    title: "Library source health",
    description: "Counts by provider and synchronization state",
    columns: [
      { key: "provider", label: "Provider" },
      { key: "active", label: "Active", align: "end" },
      { key: "failed", label: "Failed", align: "end" },
    ],
    rows: [
      { provider: "YouTube", active: 14, failed: 1 },
      { provider: "Telegram", active: 8, failed: 0 },
    ],
    totalRows: 22,
  });

  const details = screen.getByLabelText("Expand Library source health diagnostics section") as HTMLDetailsElement;
  expect(details.open).toBe(true);
  expect(screen.getByText("2/22 rows")).toBeTruthy();
  expect(screen.getByRole("table", { name: "Diagnostic counts for Library source health" })).toBeTruthy();

  await fireEvent.click(details.querySelector("summary")!);
  expect(details.open).toBe(false);
  await fireEvent.click(details.querySelector("summary")!);
  expect(details.open).toBe(true);
});
