// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectTabs, { PROJECT_SECTIONS } from "./ProjectTabs.svelte";
import rawSource from "./ProjectTabs.svelte?raw";

afterEach(cleanup);

const source = rawSource.replace(/\r\n/g, "\n");

describe("ProjectTabs", () => {
  it("renders all six sections with Russian labels", () => {
    render(ProjectTabs, { props: { active: "sources" } });
    for (const label of ["Обзор", "Источники", "Факты", "Отчёты", "Запуски", "Промпты"]) {
      expect(screen.getByRole("tab", { name: label })).toBeTruthy();
    }
    expect(PROJECT_SECTIONS).toHaveLength(6);
  });

  it("marks only the active tab as selected", () => {
    render(ProjectTabs, { props: { active: "sources" } });
    expect(screen.getByRole("tab", { name: "Источники" }).getAttribute("aria-selected")).toBe(
      "true",
    );
    expect(screen.getByRole("tab", { name: "Обзор" }).getAttribute("aria-selected")).toBe("false");
    expect(screen.getAllByRole("tab", { selected: true })).toHaveLength(1);
  });

  it("forwards tab selection", async () => {
    const onSelect = vi.fn();
    render(ProjectTabs, { props: { active: "sources", onSelect } });
    await fireEvent.click(screen.getByRole("tab", { name: "Отчёты" }));
    expect(onSelect).toHaveBeenCalledWith("reports");
  });

  it("keeps the v11 compact tab row contract", () => {
    expect(source).toContain("height: 40px");
    expect(source).toContain("box-shadow: inset 0 -2px 0 var(--extractum-primary)");
  });
});
