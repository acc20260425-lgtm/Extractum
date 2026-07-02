// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ResearchProjectsShell from "./ResearchProjectsShell.svelte";
import rawShellSource from "./ResearchProjectsShell.svelte?raw";
import type { InspectorSource } from "./Inspector.svelte";
import type { ProjectSummary } from "$lib/types/projects";

// SourcesGrid (svar) does not render under jsdom, so the rail/selection are
// render-tested with no project selected, while the main grid wiring is
// verified by source assertions.
const shellSource = rawShellSource.replace(/\r\n/g, "\n");

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
      props: { summaries: [summary({ name: "Беларусь" })], selectedProjectId: null, now: NOW },
    });

    expect(screen.getByText("Беларусь")).toBeTruthy();
  });

  it("wires the sources grid in the main area for the selected project", () => {
    expect(shellSource).toContain("<SourcesGrid");
    expect(shellSource).toContain("selectedProjectId !== null");
    expect(shellSource).toContain("{sources}");
    expect(shellSource).toContain("{onSelectedSourceIdsChange}");
  });

  it("renders the toolbar above the grid for the selected project", () => {
    expect(shellSource).toContain("<ProjectToolbar");
    expect(shellSource).toContain("{...toolbar}");
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

  it("renders the inspector as the right column", () => {
    const inspectorBag = {
      open: true,
      selected: {
        title: "ФинБеларусь",
        handle: "@fb",
        statusLabel: "active",
        syncStatus: "active",
        materialsLabel: "339",
        lastSyncLabel: "02.06",
      } satisfies InspectorSource,
      periodLabel: "Весь период",
      promptLabel: "По умолчанию",
      modelLabel: "gpt-4.1",
    };

    render(ResearchProjectsShell, {
      props: { summaries: [], selectedProjectId: null, now: NOW, inspector: inspectorBag },
    });

    expect(screen.getByText("Инспектор источника")).toBeTruthy();
    expect(screen.getByText("ФинБеларусь")).toBeTruthy();
  });

  it("renders the run dock for the selected project", () => {
    expect(shellSource).toContain("<RunDock");
    expect(shellSource).toContain("{...runDock}");
  });

  it("renders the bulk-action bar above the grid when a bulkBar bag is provided", () => {
    expect(shellSource).toContain("<SourcesBulkBar");
    expect(shellSource).toContain("{...bulkBar}");
    // The bar must sit above the grid container in the main column.
    const barIndex = shellSource.indexOf("<SourcesBulkBar");
    const gridIndex = shellSource.indexOf('class="research-projects-shell__grid"');
    expect(barIndex).toBeGreaterThan(-1);
    expect(barIndex).toBeLessThan(gridIndex);
  });
});
