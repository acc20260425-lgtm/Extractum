import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";

vi.mock("@svar-ui/svelte-core", async () => ({ Locale: (await import("$lib/testing/SvarLocaleReceiver.svelte")).default }));
vi.mock("@svar-ui/svelte-grid", async () => ({ Grid: (await import("$lib/testing/SvarGridReceiver.svelte")).default, Willow: (await import("$lib/testing/SvarWillowReceiver.svelte")).default }));
vi.mock("@svar-ui/core-locales", () => ({ ru: {} }));
vi.mock("@svar-ui/grid-locales", () => ({ en: {} }));

import ResearchProjectsShell from "./ResearchProjectsShell.svelte";

let resizeCallback: ResizeObserverCallback | null = null;
beforeEach(() => {
  class ResizeObserverStub {
    constructor(callback: ResizeObserverCallback) { resizeCallback = callback; }
    observe() {}
    unobserve() {}
    disconnect() {}
  }
  vi.stubGlobal("ResizeObserver", ResizeObserverStub);
  vi.stubGlobal("CSS", { escape: (value: string) => value });
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); resizeCallback = null; });

const summary = (overrides: Record<string, unknown> = {}) => ({ id: 1, name: "Alpha", description: null, source_count: 1, material_count: 10, status: "ready", last_run_at: null, pinned: false, archived: false, updated_at: 1, ...overrides });
const source = (overrides: Record<string, unknown> = {}) => ({ project_id: 1, source_id: 7, provider: "telegram", source_subtype: "supergroup", title: "Research channel", subtitle: null, item_count: 10, added_at: 1, last_synced_at: 2, sync_status: "active", handle: "@research", ...overrides });
const railPanel = (overrides: Record<string, unknown> = {}) => ({ summaries: [summary()], selectedProjectId: 1, now: 10, ...overrides });
const toolbar = { title: "Alpha", periodPresets: [], selectedPeriodLabel: "All time", promptOptions: [], modelOptions: [] };
const inspector = (overrides: Record<string, unknown> = {}) => ({ open: true, selected: null, periodLabel: "All time", promptLabel: "Default", modelLabel: "Model", ...overrides });

describe("ResearchProjectsShell", () => {
  it("displays tabs and the selected section placeholder", () => {
    render(ResearchProjectsShell, { railPanel: railPanel() as never, selectedProjectId: 1, toolbar: toolbar as never, tabs: { active: "overview" }, sectionPlaceholder: "Overview is being prepared" });
    const toolbarTitle = screen.getByRole("heading", { name: "Alpha" });
    const tabs = screen.getByRole("tablist");
    const placeholder = screen.getByText("Overview is being prepared");
    expect(tabs.getAttribute("aria-label")).toBe("Разделы проекта");
    expect(screen.getByRole("tab", { name: "Обзор" }).getAttribute("aria-selected")).toBe("true");
    expect(placeholder.textContent).toBe("Overview is being prepared");
    expect(screen.queryByRole("grid")).toBeNull();
    expect(toolbarTitle.compareDocumentPosition(tabs) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(tabs.compareDocumentPosition(placeholder) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("forwards source row activation to the grid", async () => {
    const onActivateSource = vi.fn();
    render(ResearchProjectsShell, { railPanel: railPanel() as never, selectedProjectId: 1, sources: [source()] as never, onActivateSource });
    await fireEvent.click(screen.getByText("Research channel"));
    expect(onActivateSource).toHaveBeenCalledWith("7");
    expect(onActivateSource).toHaveBeenCalledOnce();
  });

  it("presents the project rail", async () => {
    const onSelect = vi.fn();
    render(ResearchProjectsShell, { railPanel: railPanel({ selectedProjectId: null, onSelect }) as never, selectedProjectId: null });
    expect(screen.getByText("Alpha")).toBeTruthy();
    expect(screen.getByPlaceholderText("Поиск проектов")).toBeTruthy();
    await fireEvent.click(screen.getByText("Alpha"));
    expect(onSelect).toHaveBeenCalledWith(1);
  });

  it("renders the selected project's source grid", () => {
    render(ResearchProjectsShell, { railPanel: railPanel() as never, selectedProjectId: 1, sources: [source()] as never });
    expect(screen.getByRole("grid", { name: "Источники проекта" })).toBeTruthy();
    expect(screen.getByText("Research channel")).toBeTruthy();
    expect(JSON.parse(screen.getByRole("grid").dataset.rowIds ?? "[]")).toEqual(["7"]);
    expect(screen.queryByText("Выберите проект")).toBeNull();
  });

  it("presents the toolbar before the source grid", () => {
    render(ResearchProjectsShell, { railPanel: railPanel() as never, selectedProjectId: 1, sources: [source()] as never, toolbar: toolbar as never });
    const title = screen.getByRole("heading", { name: "Alpha" });
    const grid = screen.getByRole("grid");
    expect(title.tagName).toBe("H1");
    expect(title.compareDocumentPosition(grid) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("overlays the inspector on constrained layouts", async () => {
    const onToggle = vi.fn();
    const view = render(ResearchProjectsShell, { railPanel: railPanel() as never, selectedProjectId: 1, sources: [source()] as never, inspector: inspector({ onToggle }) as never });
    const shell = screen.getByRole("application", { name: "Research projects" });
    expect(resizeCallback).not.toBeNull();
    resizeCallback?.([{ target: shell, contentRect: { width: 1000 } } as unknown as ResizeObserverEntry], {} as ResizeObserver);
    const backdrop = screen.getByRole("button", { name: "Закрыть инспектор" });
    expect(shell.dataset.constrained).toBe("true");
    const dialog = screen.getByRole("dialog", { name: "Source inspector" });
    expect(dialog.getAttribute("aria-modal")).toBe("true");
    expect(dialog.contains(screen.getByText("Инспектор источника"))).toBe(true);
    await fireEvent.click(backdrop);
    expect(onToggle).toHaveBeenCalledOnce();
    resizeCallback?.([{ target: shell, contentRect: { width: 1300 } } as unknown as ResizeObserverEntry], {} as ResizeObserver);
    expect(shell.dataset.constrained).toBe("false");
    expect(screen.queryByRole("button", { name: "Закрыть инспектор" })).toBeNull();
    expect(screen.queryByRole("dialog", { name: "Source inspector" })).toBeNull();
  });

  it("presents the selected project's run dock", () => {
    render(ResearchProjectsShell, { railPanel: railPanel() as never, selectedProjectId: 1, runDock: { activeRunLabel: "Report is running", queueCount: 2 } });
    expect(screen.getByText("Report is running")).toBeTruthy();
    expect(screen.getByText("Очередь: 2")).toBeTruthy();
  });

  it("presents bulk actions between statistics and the source grid", async () => {
    render(ResearchProjectsShell, { railPanel: railPanel() as never, selectedProjectId: 1, sources: [source()] as never, filterBar: { filtersOpen: false, shownCount: 1, totalCount: 1 }, bulkBar: { count: 1 }, filterRow: { filters: { query: "", types: [], statuses: [], materialsMin: null, materialsMax: null, lastSyncFrom: null, lastSyncTo: null } } as never });
    const statistics = screen.getByRole("status", { name: "Source statistics" });
    const bulk = screen.getByRole("region", { name: "Массовые действия" });
    const filter = screen.getByRole("search", { name: "Source filters" });
    const grid = screen.getByRole("grid");
    expect(statistics.textContent).toContain("1 из 1");
    expect(bulk.textContent).toContain("Выбрано: 1");
    expect(filter.contains(screen.getByPlaceholderText("Поиск"))).toBe(true);
    expect(JSON.parse(grid.dataset.rowIds ?? "[]")).toEqual(["7"]);
    expect(statistics.compareDocumentPosition(bulk) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(bulk.compareDocumentPosition(filter) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(filter.compareDocumentPosition(grid) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(screen.getByText("Выбрано: 1")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Add source" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Connect from Library" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Синхронизировать" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Delete from Library" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Удалить" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Фильтр по типу" }));
    expect(screen.getByRole("checkbox", { name: "telegram" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Фильтр по статусу" }));
    expect(screen.getByRole("checkbox", { name: "active" })).toBeTruthy();
  });
});
