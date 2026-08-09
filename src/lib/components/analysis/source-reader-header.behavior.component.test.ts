import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import SourceReaderHeader from "./source-reader-header.svelte";

afterEach(cleanup);

describe("analysis priority UX contract", () => {
  it("keeps the report canvas top chrome compact and action-oriented", async () => {
    const onViewLiveSource = vi.fn();
    const onBackToRunSnapshot = vi.fn();
    const onChangeSelectedSourceId = vi.fn();
    const view = render(SourceReaderHeader, {
      props: {
        compact: true,
        title: "Project sources",
        subtitle: "3 loaded rows",
        surfaceLabel: "Source material",
        sourceViewBasis: "run_snapshot",
        sourceBasisState: "run_snapshot_available",
        canViewLiveSource: true,
        canBackToRunSnapshot: false,
        selectedSourceId: null,
        sourceOptions: [
          { id: 1, label: "Research channel", count: 1 },
          { id: 2, label: "Research video", count: 2 },
        ],
        onViewLiveSource,
        onBackToRunSnapshot,
        onChangeSelectedSourceId,
      },
    });

    expect(screen.getByRole("banner", { name: "Project sources" })).toBeTruthy();
    expect(screen.getByText("Source material")).toBeTruthy();
    expect(screen.getByText("3 loaded rows")).toBeTruthy();
    expect(screen.queryByText("Project sources")).toBeNull();
    expect(screen.getByText("Run snapshot")).toBeTruthy();
    expect(screen.getByText("Source focus")).toBeTruthy();
    const focus = screen.getByRole("combobox");
    expect((focus as HTMLSelectElement).value).toBe("__all_sources__");
    expect(screen.getByRole("option", { name: "Research channel (1)" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Research video (2)" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "View live source" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Back to run snapshot" })).toBeNull();
    await fireEvent.change(focus, { target: { value: "2" } });
    expect(onChangeSelectedSourceId).toHaveBeenLastCalledWith(2);
    await fireEvent.change(focus, { target: { value: "__all_sources__" } });
    expect(onChangeSelectedSourceId).toHaveBeenLastCalledWith(null);
    await fireEvent.click(screen.getByRole("button", { name: "View live source" }));
    expect(onViewLiveSource).toHaveBeenCalledOnce();

    await view.rerender({
      compact: false,
      title: "Research channel",
      subtitle: "Current source material",
      surfaceLabel: "Source material",
      sourceViewBasis: "live_source",
      sourceBasisState: "live_source",
      canViewLiveSource: false,
      canBackToRunSnapshot: true,
      selectedSourceId: null,
      sourceOptions: [],
      onViewLiveSource,
      onBackToRunSnapshot,
      onChangeSelectedSourceId,
    });
    expect(screen.getByText("Live source")).toBeTruthy();
    expect(screen.getByText("Current source material")).toBeTruthy();
    expect(screen.queryByRole("combobox")).toBeNull();
    expect(screen.getByRole("button", { name: "Back to run snapshot" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Back to run snapshot" }));
    expect(onBackToRunSnapshot).toHaveBeenCalledOnce();
  });
});
