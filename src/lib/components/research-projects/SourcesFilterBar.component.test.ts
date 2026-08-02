// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SourcesFilterBar from "./SourcesFilterBar.svelte";

afterEach(cleanup);

const base = { filtersOpen: false, shownCount: 8, totalCount: 10 };

function visibleActionCopy(button: HTMLElement) {
  return button.textContent?.replace(/^\s*\+\s*/, "").replace(/\s+/g, " ").trim();
}

describe("SourcesFilterBar", () => {
  it("shows the counter and toggles the filter row", async () => {
    const onToggleFilters = vi.fn();
    render(SourcesFilterBar, { props: { ...base, onToggleFilters } });
    expect(screen.getByText("8 из 10")).toBeTruthy();
    await fireEvent.click(screen.getByText("Фильтры"));
    expect(onToggleFilters).toHaveBeenCalledOnce();
  });

  it("shows the badge with the active filter count only when filters are active", () => {
    const { unmount } = render(SourcesFilterBar, { props: { ...base } });
    expect(document.querySelector(".sources-filter-bar__badge")).toBeNull();
    unmount();

    render(SourcesFilterBar, {
      props: {
        ...base,
        filtersActive: true,
        chips: [
          { key: "type:telegram", label: "Тип: telegram", dot: "var(--extractum-provider-telegram)" },
          { key: "query", label: "Источник: фин", dot: null },
        ],
      },
    });
    expect(document.querySelector(".sources-filter-bar__badge")?.textContent).toBe("2");
  });

  it("renders chips and removes one by its close button", async () => {
    const onRemoveChip = vi.fn();
    render(SourcesFilterBar, {
      props: {
        ...base,
        filtersActive: true,
        chips: [{ key: "query", label: "Источник: фин", dot: null }],
        onRemoveChip,
      },
    });
    expect(screen.getByText("Источник: фин")).toBeTruthy();
    await fireEvent.click(screen.getByLabelText("Убрать фильтр Источник: фин"));
    expect(onRemoveChip).toHaveBeenCalledWith("query");
  });

  it("shows «Сбросить» only when filters are active and forwards the click", async () => {
    const onClearAll = vi.fn();
    const { unmount } = render(SourcesFilterBar, { props: { ...base } });
    expect(screen.queryByText("Сбросить")).toBeNull();
    unmount();

    render(SourcesFilterBar, { props: { ...base, filtersActive: true, onClearAll } });
    await fireEvent.click(screen.getByText("Сбросить"));
    expect(onClearAll).toHaveBeenCalledOnce();
  });

  it("exposes separate Add source and Connect from Library actions", async () => {
    const onAddSource = vi.fn();
    const onConnectFromLibrary = vi.fn();
    render(SourcesFilterBar, { props: { ...base, onAddSource, onConnectFromLibrary } });

    await fireEvent.click(screen.getByRole("button", { name: "Add source" }));
    await fireEvent.click(screen.getByRole("button", { name: "Connect from Library" }));

    expect(onAddSource).toHaveBeenCalledOnce();
    expect(onConnectFromLibrary).toHaveBeenCalledOnce();
  });

  it("keeps visible copy, aria-label and title synchronized for source actions", () => {
    render(SourcesFilterBar, { props: { ...base } });

    const add = screen.getByRole("button", { name: "Add source" });
    const connect = screen.getByRole("button", { name: "Connect from Library" });

    expect(visibleActionCopy(add)).toBe("Add source");
    expect(add.getAttribute("aria-label")).toBe("Add source");
    expect(add.getAttribute("title")).toBe("Add source");
    expect(add.getAttribute("data-ui-action")).toBe("add-source");

    expect(connect.textContent?.replace(/\s+/g, " ").trim()).toBe("Connect from Library");
    expect(connect.getAttribute("aria-label")).toBe("Connect from Library");
    expect(connect.getAttribute("title")).toBe("Connect from Library");
    expect(connect.getAttribute("data-ui-action")).toBe("connect-library");
  });

  it("shows the optional keyboard hint beside the row count", () => {
    render(SourcesFilterBar, { props: { ...base, keyboardHint: "↑↓ строка · Enter инспектор" } });

    expect(screen.getByText("↑↓ строка · Enter инспектор")).toBeTruthy();
  });
});
