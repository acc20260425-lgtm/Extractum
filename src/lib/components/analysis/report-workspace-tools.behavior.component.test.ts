import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import ReportWorkspaceTools from "./report-workspace-tools.svelte";

afterEach(cleanup);

describe("analysis priority UX contract", () => {
  it("keeps the report canvas top chrome compact and action-oriented", async () => {
    const onOpenNotebookLmExport = vi.fn();
    const onToggleTemplateEditor = vi.fn();
    const onToggleGroupEditor = vi.fn();
    const view = render(ReportWorkspaceTools, {
      props: {
        compact: true,
        showNotebookLmExport: true,
        canExportNotebookLm: true,
        exportDisabledReason: null,
        exportingNotebookLm: false,
        templateEditorOpen: false,
        groupEditorOpen: false,
        onOpenNotebookLmExport,
        onToggleTemplateEditor,
        onToggleGroupEditor,
      },
    });

    expect(screen.getByRole("region", { name: "Workspace actions" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Export for NotebookLM" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Edit templates" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Edit groups" })).toBeTruthy();
    expect(screen.queryByText("Workspace tools")).toBeNull();
    expect(screen.queryByText("Export for NotebookLM")).toBeNull();
    expect(screen.getByRole("button", { name: "Edit templates" }).getAttribute("aria-expanded")).toBe("false");
    expect(screen.getByRole("button", { name: "Edit groups" }).getAttribute("aria-expanded")).toBe("false");
    await fireEvent.click(screen.getByRole("button", { name: "Export for NotebookLM" }));
    expect(onOpenNotebookLmExport).toHaveBeenCalledOnce();
    await fireEvent.click(screen.getByRole("button", { name: "Edit templates" }));
    expect(onToggleTemplateEditor).toHaveBeenCalledOnce();
    await fireEvent.click(screen.getByRole("button", { name: "Edit groups" }));
    expect(onToggleGroupEditor).toHaveBeenCalledOnce();

    await view.rerender({
      compact: false,
      showNotebookLmExport: true,
      canExportNotebookLm: false,
      exportDisabledReason: "YouTube group export is unavailable.",
      exportingNotebookLm: false,
      templateEditorOpen: true,
      groupEditorOpen: true,
      onOpenNotebookLmExport,
      onToggleTemplateEditor,
      onToggleGroupEditor,
    });
    expect(screen.getByText("Workspace tools")).toBeTruthy();
    expect(screen.getByText("Export for NotebookLM")).toBeTruthy();
    expect((screen.getByRole("button", { name: "Export for NotebookLM" }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText("YouTube group export is unavailable.")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Export for NotebookLM" }).getAttribute("aria-describedby")).toBe("notebooklm-export-disabled-reason");
    expect(screen.getByRole("button", { name: "Hide templates" }).getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByRole("button", { name: "Hide groups" }).getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByRole("button", { name: "Hide groups" }).textContent).toContain("Hide groups");
  });
});
