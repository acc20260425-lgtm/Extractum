// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectRailSections from "./ProjectRailSections.svelte";
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

describe("ProjectRailSections", () => {
  it("renders projects grouped into pinned, normal, and archived sections", () => {
    const summaries = [
      summary({ id: 1, name: "Pinned one", pinned: true }),
      summary({ id: 2, name: "Normal one" }),
      summary({ id: 3, name: "Archived one", archived: true }),
    ];

    render(ProjectRailSections, { props: { summaries, now: NOW } });

    expect(screen.getByText("Закреплённые")).toBeTruthy();
    expect(screen.getByText("Проекты")).toBeTruthy();
    expect(screen.getByText("Архив")).toBeTruthy();
    expect(screen.getByText("Pinned one")).toBeTruthy();
    expect(screen.getByText("Normal one")).toBeTruthy();
    expect(screen.getByText("Archived one")).toBeTruthy();
  });

  it("hides empty section headers", () => {
    render(ProjectRailSections, {
      props: { summaries: [summary({ id: 2, name: "Only normal" })], now: NOW },
    });

    expect(screen.queryByText("Закреплённые")).toBeNull();
    expect(screen.queryByText("Архив")).toBeNull();
    expect(screen.getByText("Only normal")).toBeTruthy();
  });

  it("forwards row selection to onSelect", async () => {
    const onSelect = vi.fn();
    render(ProjectRailSections, {
      props: { summaries: [summary({ id: 9, name: "Pick me" })], now: NOW, onSelect },
    });

    await fireEvent.click(screen.getByText("Pick me"));

    expect(onSelect).toHaveBeenCalledWith(9);
  });
});
