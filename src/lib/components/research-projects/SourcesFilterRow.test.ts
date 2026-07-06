// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SourcesFilterRow from "./SourcesFilterRow.svelte";
import { SOURCE_FILTER_ROW_GRID_TEMPLATE } from "$lib/ui/research-projects-source-row";
import { emptySourceFilters } from "$lib/ui/research-projects-source-filters";

afterEach(cleanup);

describe("SourcesFilterRow", () => {
  it("emits a new filters object when the search query changes", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, { props: { filters: emptySourceFilters(), onChange } });
    await fireEvent.input(screen.getByPlaceholderText("Поиск"), { target: { value: "фин" } });
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), query: "фин" });
  });

  it("clears the query with the × button", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, {
      props: { filters: { ...emptySourceFilters(), query: "фин" }, onChange },
    });
    await fireEvent.click(screen.getByTitle("Очистить поиск"));
    expect(onChange).toHaveBeenCalledWith(emptySourceFilters());
  });

  it("toggles a provider type through the type popover", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, { props: { filters: emptySourceFilters(), onChange } });
    await fireEvent.click(screen.getByLabelText("Фильтр по типу"));
    await fireEvent.click(await screen.findByLabelText("telegram"));
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), types: ["telegram"] });
  });

  it("shows the selected type in the trigger label", () => {
    render(SourcesFilterRow, {
      props: { filters: { ...emptySourceFilters(), types: ["youtube"] } },
    });
    expect(screen.getByLabelText("Фильтр по типу").textContent).toContain("youtube");
  });

  it("toggles a status through the status popover", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, { props: { filters: emptySourceFilters(), onChange } });
    await fireEvent.click(screen.getByLabelText("Фильтр по статусу"));
    await fireEvent.click(await screen.findByLabelText("error"));
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), statuses: ["error"] });
  });

  it("emits materials and date range updates", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, { props: { filters: emptySourceFilters(), onChange } });

    await fireEvent.input(screen.getByLabelText("Материалы от"), { target: { value: "10" } });
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), materialsMin: 10 });

    await fireEvent.input(screen.getByLabelText("Синхронизирован с"), {
      target: { value: "2026-05-01" },
    });
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), syncedFrom: "2026-05-01" });
  });

  it("uses the shared source table grid template", () => {
    render(SourcesFilterRow, { props: { filters: emptySourceFilters() } });
    const row = document.querySelector(".sources-filter-row") as HTMLElement | null;

    expect(row?.getAttribute("style")).toContain(
      `grid-template-columns: ${SOURCE_FILTER_ROW_GRID_TEMPLATE}`,
    );
  });
});
