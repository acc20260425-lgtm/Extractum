// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectRailPanel from "./ProjectRailPanel.svelte";
import type { ProjectSummary } from "$lib/types/projects";

afterEach(cleanup);

const NOW = 1_000_000_000;

function summary(overrides: Partial<ProjectSummary> = {}): ProjectSummary {
  return {
    id: 1,
    name: "Alpha",
    description: null,
    source_count: 3,
    material_count: 100,
    status: "ready",
    last_run_at: null,
    pinned: false,
    archived: false,
    updated_at: 1,
    ...overrides,
  };
}

const baseProps = {
  selectedProjectId: null as number | null,
  now: NOW,
};

describe("ProjectRailPanel", () => {
  it("renders header actions: compact toggle, create, disabled sync", () => {
    render(ProjectRailPanel, { props: { ...baseProps, summaries: [summary()] } });
    expect(screen.getByTitle("Компактный вид")).toBeTruthy();
    expect(screen.getByTitle("Создать проект")).toBeTruthy();
    const sync = screen.getByTitle("Скоро") as HTMLButtonElement;
    expect(sync.disabled).toBe(true);
  });

  it("hides the header project menu without a selected project and shows it with one", () => {
    const { unmount } = render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary()] },
    });
    expect(screen.queryByTitle("Действия с проектом")).toBeNull();
    unmount();

    render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary({ id: 5 })], selectedProjectId: 5 },
    });
    expect(screen.getByTitle("Действия с проектом")).toBeTruthy();
  });

  it("filters projects by search and shows an empty state", async () => {
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [summary({ id: 1, name: "Беларусь" }), summary({ id: 2, name: "Финтех" })],
      },
    });
    const input = screen.getByPlaceholderText("Поиск проектов");
    await fireEvent.input(input, { target: { value: "фин" } });
    expect(screen.queryByText("Беларусь")).toBeNull();
    expect(screen.getByText("Финтех")).toBeTruthy();

    await fireEvent.input(input, { target: { value: "нет-такого" } });
    expect(screen.getByText("Проекты не найдены")).toBeTruthy();
  });

  it("keeps the archive collapsed by default with a full count and expands on click", async () => {
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [
          summary({ id: 1, name: "Живой" }),
          summary({ id: 2, name: "Старый аудит", archived: true }),
          summary({ id: 3, name: "Q3 ресёрч", archived: true }),
        ],
      },
    });
    expect(screen.queryByText("Старый аудит")).toBeNull();
    expect(screen.getByText("2")).toBeTruthy();

    await fireEvent.click(screen.getByText("Архив"));
    expect(screen.getByText("Старый аудит")).toBeTruthy();
    expect(screen.getByText("Q3 ресёрч")).toBeTruthy();
  });

  it("compact mode hides meta lines", async () => {
    render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary({ name: "Cmp", source_count: 3 })] },
    });
    expect(screen.getByText("3 источника · готов")).toBeTruthy();
    await fireEvent.click(screen.getByTitle("Компактный вид"));
    expect(screen.queryByText("3 источника · готов")).toBeNull();
  });

  it("renders the selected project first as the active row", () => {
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [summary({ id: 1, name: "First" }), summary({ id: 2, name: "Chosen" })],
        selectedProjectId: 2,
      },
    });
    const rows = document.querySelectorAll(".project-row");
    expect(rows[0]?.getAttribute("data-variant")).toBe("active");
    expect(rows[0]?.textContent).toContain("Chosen");
  });

  it("confirms deletion through a dialog before calling onDelete", async () => {
    const onDelete = vi.fn();
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [summary({ id: 9, name: "Del" })],
        selectedProjectId: 9,
        onDelete,
      },
    });
    await fireEvent.click(screen.getByTitle("Действия с проектом"));
    await fireEvent.click(await screen.findByText("Удалить"));
    expect(onDelete).not.toHaveBeenCalled();

    await fireEvent.click(await screen.findByText("Да, удалить"));
    expect(onDelete).toHaveBeenCalledWith(9);
  });

  it("forwards create clicks", async () => {
    const onCreate = vi.fn();
    render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [], onCreate },
    });
    await fireEvent.click(screen.getByTitle("Создать проект"));
    expect(onCreate).toHaveBeenCalledOnce();
  });
});
