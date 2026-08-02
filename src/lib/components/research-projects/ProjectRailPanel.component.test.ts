// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectRailPanel from "./ProjectRailPanel.svelte";
import rawSource from "./ProjectRailPanel.svelte?raw";
import type { ProjectSummary } from "$lib/types/projects";

afterEach(cleanup);

const source = rawSource.replace(/\r\n/g, "\n");
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
  it("renders the project list as a v11 listbox with the active project selected", () => {
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [summary({ id: 1, name: "First" }), summary({ id: 2, name: "Chosen" })],
        selectedProjectId: 2,
      },
    });

    expect(screen.getByRole("listbox", { name: "Проекты" })).toBeTruthy();
    expect(screen.getByRole("option", { name: /Chosen/, selected: true })).toBeTruthy();
  });

  it("keeps v11 panel affordance hooks in source", () => {
    expect(source).toContain('aria-label="Проекты"');
    expect(source).toContain("aria-expanded={archiveOpen}");
    expect(source).toContain('aria-label={compact ? "Комфортный вид" : "Компактный вид"}');
    expect(source).toContain('aria-label="Создать проект"');
  });

  it("renders header actions: compact toggle, create, disabled sync", () => {
    render(ProjectRailPanel, { props: { ...baseProps, summaries: [summary()] } });
    expect(screen.getByTitle("Компактный вид")).toBeTruthy();
    expect(screen.getByTitle("Создать проект")).toBeTruthy();
    const sync = screen.getByTitle("Скоро") as HTMLButtonElement;
    expect(sync.disabled).toBe(true);
  });

  it("exposes the project list header controls with stable v11 action hooks", async () => {
    render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary({ id: 5 })], selectedProjectId: 5 },
    });

    const compactButton = screen.getByRole("button", { name: "Компактный вид" });
    expect(compactButton.getAttribute("data-ui-action")).toBe("toggle-project-compact");
    expect(compactButton.getAttribute("aria-pressed")).toBe("false");

    await fireEvent.click(compactButton);
    const comfortButton = screen.getByRole("button", { name: "Комфортный вид" });
    expect(comfortButton.getAttribute("aria-pressed")).toBe("true");

    expect(screen.getByRole("button", { name: "Создать проект" }).getAttribute("data-ui-action")).toBe(
      "create-project",
    );
    const sync = screen.getByRole("button", { name: "Синхронизация скоро" }) as HTMLButtonElement;
    expect(sync.getAttribute("data-ui-action")).toBe("sync-projects");
    expect(sync.getAttribute("aria-disabled")).toBe("true");
    expect(sync.disabled).toBe(true);
    const menuButton = screen.getByRole("button", { name: "Действия выбранного проекта" });
    expect(menuButton.getAttribute("data-ui-action")).toBe("selected-project-actions");
    expect(menuButton.textContent?.trim()).toBe("⋯");
  });

  it("exposes search clear as a named icon action", async () => {
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [summary({ id: 1, name: "Alpha" }), summary({ id: 2, name: "Beta" })],
      },
    });
    const input = screen.getByPlaceholderText("Поиск проектов") as HTMLInputElement;
    await fireEvent.input(input, { target: { value: "alp" } });

    const clear = screen.getByRole("button", { name: "Очистить поиск проектов" });
    expect(clear.getAttribute("data-ui-action")).toBe("clear-project-search");

    await fireEvent.click(clear);
    expect(input.value).toBe("");
  });

  it("keeps the header project menu available with guidance without a selected project", async () => {
    const { unmount } = render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary()] },
    });
    const emptyMenu = screen.getByRole("button", { name: "Действия выбранного проекта" }) as HTMLButtonElement;
    expect(emptyMenu.textContent?.trim()).toBe("⋯");
    expect(emptyMenu.getAttribute("data-ui-action")).toBe("selected-project-actions");
    expect(emptyMenu.disabled).toBe(false);

    await fireEvent.click(emptyMenu);
    expect(await screen.findByText("Выберите проект")).toBeTruthy();
    unmount();

    render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary({ id: 5 })], selectedProjectId: 5 },
    });
    const activeMenu = screen.getByRole("button", { name: "Действия выбранного проекта" }) as HTMLButtonElement;
    expect(activeMenu.disabled).toBe(false);
    expect(activeMenu.getAttribute("aria-disabled")).toBeNull();
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
    const archiveToggle = screen.getByRole("button", { name: /Архив/ });
    expect(archiveToggle.getAttribute("data-ui-action")).toBe("toggle-project-archive");
    expect(archiveToggle.getAttribute("aria-expanded")).toBe("false");
    expect(screen.getByText("2")).toBeTruthy();

    await fireEvent.click(archiveToggle);
    expect(archiveToggle.getAttribute("aria-expanded")).toBe("true");
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
