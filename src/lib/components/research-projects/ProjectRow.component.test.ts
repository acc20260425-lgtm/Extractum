// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectRow from "./ProjectRow.svelte";
import rawSource from "./ProjectRow.svelte?raw";
import { buildProjectRailRow } from "$lib/ui/research-projects-rail";
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

describe("ProjectRow", () => {
  it("exposes the row as a v11 listbox option with selected and state attributes", () => {
    const row = buildProjectRailRow(summary({ id: 5, name: "Act", pinned: true }), NOW);
    const { container } = render(ProjectRow, { props: { row, variant: "active" } });

    const option = screen.getByRole("option", { name: /Act/, selected: true });
    expect(option).toBeTruthy();
    const root = container.querySelector(".project-row");
    expect(root?.getAttribute("data-variant")).toBe("active");
    expect(root?.getAttribute("data-pinned")).toBe("true");
    expect(root?.getAttribute("data-archived")).toBe("false");
    expect(root?.getAttribute("data-compact")).toBe("false");
  });

  it("keeps v11 focus and hover affordance hooks in source", () => {
    expect(source).toContain(":focus-visible");
    expect(source).toContain('aria-selected={variant === "active"}');
    expect(source).toContain("Действия проекта");
    expect(source).not.toContain("progress");
  });

  it("renders the project name and meta", () => {
    const row = buildProjectRailRow(
      summary({ name: "Беларусь", source_count: 10, status: "ready" }),
      NOW,
    );

    render(ProjectRow, { props: { row } });

    expect(screen.getByText("Беларусь")).toBeTruthy();
    expect(screen.getByText("10 источников · готов")).toBeTruthy();
  });

  it("calls onSelect with the project id when clicked", async () => {
    const onSelect = vi.fn();
    const row = buildProjectRailRow(summary({ id: 42, name: "Pick" }), NOW);

    render(ProjectRow, { props: { row, onSelect } });
    await fireEvent.click(screen.getByText("Pick"));

    expect(onSelect).toHaveBeenCalledWith(42);
  });

  it("shows a pinned marker only when the project is pinned", () => {
    const { unmount } = render(ProjectRow, {
      props: { row: buildProjectRailRow(summary({ pinned: true }), NOW) },
    });
    expect(screen.getByTitle("Закреплён")).toBeTruthy();
    unmount();

    render(ProjectRow, {
      props: { row: buildProjectRailRow(summary({ pinned: false }), NOW) },
    });
    expect(screen.queryByTitle("Закреплён")).toBeNull();
  });

  it("renders a status dot reflecting the project status", () => {
    render(ProjectRow, {
      props: { row: buildProjectRailRow(summary({ status: "running" }), NOW) },
    });

    expect(screen.getByTestId("project-row-status-dot").dataset.status).toBe("running");
  });

  it("marks the active variant with a data attribute and shows the accent bar", () => {
    const row = buildProjectRailRow(summary({ name: "Act" }), NOW);
    const { container } = render(ProjectRow, { props: { row, variant: "active" } });
    const root = container.querySelector(".project-row");
    expect(root?.getAttribute("data-variant")).toBe("active");
    expect(container.querySelector(".project-row__active-bar")).toBeTruthy();
  });

  it("hides the meta line and sets a title in compact mode", () => {
    const row = buildProjectRailRow(summary({ name: "Cmp", source_count: 3 }), NOW);
    const { container } = render(ProjectRow, { props: { row, compact: true } });
    expect(screen.queryByText("3 источника · готов")).toBeNull();
    expect(container.querySelector(".project-row")?.getAttribute("title")).toBe(
      "Cmp — 3 источника · готов",
    );
  });

  it("renders the actions trigger for the context menu", () => {
    const row = buildProjectRailRow(summary(), NOW);
    render(ProjectRow, { props: { row } });
    expect(screen.getByTitle("Действия")).toBeTruthy();
  });

  it("names the row menu trigger with the project for screen readers", () => {
    const row = buildProjectRailRow(summary({ name: "Menu" }), NOW);
    render(ProjectRow, { props: { row } });
    expect(screen.getByRole("button", { name: "Действия проекта Menu" })).toBeTruthy();
  });

  it("opens the menu and forwards edit / pin / delete-request actions", async () => {
    const onEdit = vi.fn();
    const onTogglePin = vi.fn();
    const onRequestDelete = vi.fn();
    const row = buildProjectRailRow(summary({ id: 7, name: "Menu", pinned: false }), NOW);
    render(ProjectRow, { props: { row, onEdit, onTogglePin, onRequestDelete } });

    await fireEvent.click(screen.getByTitle("Действия"));
    await fireEvent.click(await screen.findByText("Редактировать"));
    expect(onEdit).toHaveBeenCalledWith(7);

    await fireEvent.click(screen.getByTitle("Действия"));
    await fireEvent.click(await screen.findByText("Закрепить"));
    expect(onTogglePin).toHaveBeenCalledWith(7, true);

    await fireEvent.click(screen.getByTitle("Действия"));
    await fireEvent.click(await screen.findByText("Удалить"));
    expect(onRequestDelete).toHaveBeenCalledWith(7, "Menu");
  });

  it("shows only unarchive + delete for the archived variant", async () => {
    const row = buildProjectRailRow(summary({ archived: true }), NOW);
    render(ProjectRow, { props: { row, variant: "archived" } });
    await fireEvent.click(screen.getByTitle("Действия"));
    expect(await screen.findByText("Из архива")).toBeTruthy();
    expect(screen.queryByText("Редактировать")).toBeNull();
    expect(screen.queryByText("Закрепить")).toBeNull();
  });
});
