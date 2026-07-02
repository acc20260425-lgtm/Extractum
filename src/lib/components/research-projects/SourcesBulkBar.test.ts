// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SourcesBulkBar from "./SourcesBulkBar.svelte";

afterEach(cleanup);

describe("SourcesBulkBar", () => {
  it("shows the selected count", () => {
    render(SourcesBulkBar, { props: { count: 3 } });
    expect(screen.getByText("Выбрано: 3")).toBeTruthy();
  });

  it("disables the sync button and exposes the title when syncDisabled", () => {
    render(SourcesBulkBar, {
      props: { count: 2, syncDisabled: true, syncTitle: "Нет источников для синхронизации" },
    });
    const sync = screen.getByRole("button", { name: "Синхронизировать" }) as HTMLButtonElement;
    expect(sync.disabled).toBe(true);
    expect(sync.getAttribute("title")).toBe("Нет источников для синхронизации");
  });

  it("calls onClear when clicking «Снять выделение»", async () => {
    const onClear = vi.fn();
    render(SourcesBulkBar, { props: { count: 1, onClear } });
    await fireEvent.click(screen.getByText("Снять выделение"));
    expect(onClear).toHaveBeenCalledOnce();
  });

  it("calls onSync when the enabled sync button is clicked", async () => {
    const onSync = vi.fn();
    render(SourcesBulkBar, { props: { count: 1, onSync } });
    await fireEvent.click(screen.getByRole("button", { name: "Синхронизировать" }));
    expect(onSync).toHaveBeenCalledOnce();
  });

  it("confirms before deleting: opens a dialog, deletes only on confirm", async () => {
    const onDelete = vi.fn();
    render(SourcesBulkBar, { props: { count: 2, onDelete } });

    // The bar's delete button opens the dialog; it must NOT delete immediately.
    await fireEvent.click(screen.getByRole("button", { name: "Удалить" }));
    expect(onDelete).not.toHaveBeenCalled();

    // Confirm inside the dialog.
    await fireEvent.click(screen.getByRole("button", { name: "Да, удалить" }));
    expect(onDelete).toHaveBeenCalledOnce();
  });
});
