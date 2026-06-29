// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ResearchProjectsShell from "./ResearchProjectsShell.svelte";
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

describe("ResearchProjectsShell", () => {
  it("renders the project rail from summaries", () => {
    render(ResearchProjectsShell, {
      props: { summaries: [summary({ name: "Беларусь" })], selectedProjectId: 1, now: NOW },
    });

    expect(screen.getByText("Беларусь")).toBeTruthy();
  });

  it("forwards project selection", async () => {
    const onSelectProject = vi.fn();
    render(ResearchProjectsShell, {
      props: {
        summaries: [summary({ id: 7, name: "Pick me" })],
        selectedProjectId: null,
        now: NOW,
        onSelectProject,
      },
    });

    await fireEvent.click(screen.getByText("Pick me"));
    expect(onSelectProject).toHaveBeenCalledWith(7);
  });
});
