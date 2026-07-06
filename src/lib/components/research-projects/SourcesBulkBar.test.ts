// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SourcesBulkBar from "./SourcesBulkBar.svelte";
import rawSource from "./SourcesBulkBar.svelte?raw";

afterEach(cleanup);

const source = rawSource.replace(/\r\n/g, "\n");

describe("SourcesBulkBar", () => {
  it("shows the selected count", () => {
    render(SourcesBulkBar, { props: { count: 3 } });
    expect(screen.getByText("Выбрано: 3")).toBeTruthy();
  });

  it("renders as an in-flow strip instead of an overlay", () => {
    expect(source).toContain(".sources-bulk-bar {");
    expect(source).not.toContain("position: absolute");
    expect(source).not.toContain("inset: 0");
    expect(source).not.toContain("z-index: 5");
    expect(source).toContain("flex-shrink: 0");
  });

  it("disables the sync button and exposes the title when syncDisabled", () => {
    render(SourcesBulkBar, {
      props: {
        count: 2,
        syncDisabled: true,
        syncTitle: "Нет источников для синхронизации",
      },
    });
    const sync = screen.getByRole("button", {
      name: "Синхронизировать",
    }) as HTMLButtonElement;
    expect(sync.disabled).toBe(true);
    expect(sync.getAttribute("title")).toBe("Нет источников для синхронизации");
  });

  it("calls onClear when clicking Clear selection", async () => {
    const onClear = vi.fn();
    render(SourcesBulkBar, { props: { count: 1, onClear } });
    await fireEvent.click(screen.getByText("Снять выделение"));
    expect(onClear).toHaveBeenCalledOnce();
  });

  it("calls onSync when the enabled sync button is clicked", async () => {
    const onSync = vi.fn();
    render(SourcesBulkBar, { props: { count: 1, onSync } });
    await fireEvent.click(
      screen.getByRole("button", { name: "Синхронизировать" }),
    );
    expect(onSync).toHaveBeenCalledOnce();
  });

  it("confirms before deleting: opens a dialog, deletes only on confirm", async () => {
    const onDelete = vi.fn();
    render(SourcesBulkBar, { props: { count: 2, onDelete } });

    await fireEvent.click(screen.getByRole("button", { name: "Удалить" }));
    expect(onDelete).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("button", { name: "Да, удалить" }));
    expect(onDelete).toHaveBeenCalledOnce();
  });

  it("shows Delete from Library as a separate action and respects disabled reason", () => {
    render(SourcesBulkBar, {
      props: {
        count: 2,
        libraryDeleteDisabled: true,
        libraryDeleteTitle: "Select one YouTube video source",
      },
    });

    const button = screen.getByRole("button", {
      name: "Delete from Library",
    }) as HTMLButtonElement;
    expect(button.disabled).toBe(true);
    expect(button.getAttribute("title")).toBe("Select one YouTube video source");
  });

  it("keeps Delete from Library visible copy and accessible names stable", () => {
    render(SourcesBulkBar, {
      props: {
        count: 1,
        libraryDeleteDisabled: false,
      },
    });

    const button = screen.getByRole("button", { name: "Delete from Library" });
    expect(button.textContent?.replace(/\s+/g, " ").trim()).toBe("Delete from Library");
  });

  it("confirms before deleting from Library and deletes only on confirm", async () => {
    const onDeleteFromLibrary = vi.fn();
    render(SourcesBulkBar, {
      props: {
        count: 1,
        libraryDeleteDisabled: false,
        onDeleteFromLibrary,
      },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Delete from Library" }));
    expect(onDeleteFromLibrary).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("button", { name: "Delete from Library permanently" }));
    expect(onDeleteFromLibrary).toHaveBeenCalledOnce();
  });

  it("does not delete from Library when the confirmation is cancelled", async () => {
    const onDeleteFromLibrary = vi.fn();
    render(SourcesBulkBar, {
      props: {
        count: 1,
        libraryDeleteDisabled: false,
        onDeleteFromLibrary,
      },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Delete from Library" }));
    await fireEvent.click(screen.getByRole("button", { name: "Cancel Library deletion" }));
    expect(onDeleteFromLibrary).not.toHaveBeenCalled();
  });
});
