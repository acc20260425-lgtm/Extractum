import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";

vi.mock("@svar-ui/svelte-core", async () => ({ Locale: (await import("$lib/testing/SvarLocaleReceiver.svelte")).default }));
vi.mock("@svar-ui/svelte-grid", async () => ({ Grid: (await import("$lib/testing/SvarGridReceiver.svelte")).default, Willow: (await import("$lib/testing/SvarWillowReceiver.svelte")).default }));
vi.mock("@svar-ui/core-locales", () => ({ ru: {} }));
vi.mock("@svar-ui/grid-locales", () => ({ en: {} }));

import SourcesGrid from "./SourcesGrid.svelte";

const originalCss = Object.getOwnPropertyDescriptor(globalThis, "CSS");
beforeAll(() => vi.stubGlobal("CSS", { escape: (value: string) => value }));
afterAll(() => { vi.unstubAllGlobals(); if (originalCss) Object.defineProperty(globalThis, "CSS", originalCss); });
afterEach(cleanup);

const source = (overrides: Record<string, unknown> = {}) => ({
  project_id: 1, source_id: 7, provider: "telegram", source_subtype: "supergroup",
  title: "Research channel", subtitle: null, item_count: 1234, added_at: 1,
  last_synced_at: 100, sync_status: "active", handle: "@research", ...overrides,
});

describe("SourcesGrid", () => {
  it("presents configured source rows and responsive columns", () => {
    render(SourcesGrid, { sources: [source(), source({ source_id: 8, title: "Video", provider: "youtube" })] as never });
    const grid = screen.getByRole("grid", { name: "Источники проекта" });
    expect(JSON.parse(grid.dataset.rowIds ?? "[]")).toEqual(["7", "8"]);
    expect(JSON.parse(grid.dataset.columnIds ?? "[]")).toEqual(["selected", "title", "typeLabel", "materialsLabel", "lastSyncedAt", "statusLabel"]);
    expect(JSON.parse(grid.dataset.responsive ?? "{}")["760"].columns.find((column: { id: string }) => column.id === "lastSyncedAt").hidden).toBe(true);
    expect(JSON.parse(grid.dataset.responsive ?? "{}")["600"].columns.find((column: { id: string }) => column.id === "typeLabel").hidden).toBe(true);
    expect(JSON.parse(grid.dataset.sizes ?? "{}")).toEqual({ rowHeight: 44, headerHeight: 34, columnWidth: 160 });
    expect(grid.dataset.multiselect).toBe("true");
  });

  it("labels and explains an empty source grid", () => {
    render(SourcesGrid, { sources: [] });
    expect(screen.getByRole("region", { name: "Источники проекта" })).toBeTruthy();
    expect(screen.getByRole("grid", { name: "Источники проекта" })).toBeTruthy();
    expect(screen.getByText("Нет источников")).toBeTruthy();
  });

  it("renders source title and status cells", () => {
    render(SourcesGrid, { sources: [source()] as never });
    expect(screen.getByText("Research channel")).toBeTruthy();
    expect(screen.getByText("@research")).toBeTruthy();
    expect(screen.getByText("active")).toBeTruthy();
    expect(screen.getByRole("checkbox", { name: "Выбрать источник" })).toBeTruthy();
  });

  it("separates row activation from checkbox selection", async () => {
    const onActivateSource = vi.fn();
    const onSelectedSourceIdsChange = vi.fn();
    render(SourcesGrid, { sources: [source()] as never, onActivateSource, onSelectedSourceIdsChange });
    await fireEvent.click(screen.getByText("Research channel"));
    expect(onActivateSource).toHaveBeenCalledWith("7");
    await fireEvent.click(screen.getByRole("checkbox", { name: "Выбрать источник" }));
    expect(onSelectedSourceIdsChange).toHaveBeenCalledWith(["7"]);
    expect(onActivateSource).toHaveBeenCalledOnce();
  });

  it("navigates rendered source rows by keyboard", async () => {
    const onKeyboardActivateSource = vi.fn();
    const onKeyboardInspectSource = vi.fn();
    const onKeyboardEscape = vi.fn(() => true);
    const onSelectedSourceIdsChange = vi.fn();
    render(SourcesGrid, { sources: [source(), source({ source_id: 8, title: "Video" })] as never, activeSourceId: "7", selectedSourceIds: [], keyboardNavigationEnabled: true, onKeyboardActivateSource, onKeyboardInspectSource, onKeyboardEscape, onSelectedSourceIdsChange });
    expect(JSON.parse(screen.getByRole("grid").dataset.rowIds ?? "[]")).toEqual(["7", "8"]);
    const down = new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true });
    document.dispatchEvent(down);
    expect(onKeyboardActivateSource).toHaveBeenCalledWith("8");
    expect(down.defaultPrevented).toBe(true);
    const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    document.dispatchEvent(enter);
    expect(onKeyboardInspectSource).toHaveBeenCalledWith("7");
    expect(enter.defaultPrevented).toBe(true);
    await fireEvent.keyDown(screen.getAllByRole("checkbox", { name: "Выбрать источник" })[0]!, { key: "ArrowDown" });
    expect(onKeyboardActivateSource).toHaveBeenCalledOnce();
    const space = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
    document.dispatchEvent(space);
    expect(onSelectedSourceIdsChange).toHaveBeenCalledWith(["7"]);
    expect(space.defaultPrevented).toBe(true);
    const escape = new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true });
    document.dispatchEvent(escape);
    expect(onKeyboardEscape).toHaveBeenCalledOnce();
    expect(escape.defaultPrevented).toBe(true);
  });

  it("supports row selection and tri-state select-all", async () => {
    const onSelectedSourceIdsChange = vi.fn();
    render(SourcesGrid, { sources: [source(), source({ source_id: 8 })] as never, selectedSourceIds: ["7"], onSelectedSourceIdsChange });
    const rows = screen.getAllByRole("checkbox", { name: "Выбрать источник" });
    const all = screen.getByRole("checkbox", { name: "Выбрать все источники" }) as HTMLInputElement;
    expect((rows[0] as HTMLInputElement).checked).toBe(true);
    expect((rows[1] as HTMLInputElement).checked).toBe(false);
    expect(all.indeterminate).toBe(true);
    await fireEvent.click(all);
    expect(onSelectedSourceIdsChange).toHaveBeenLastCalledWith(["7", "8"]);
  });
});
