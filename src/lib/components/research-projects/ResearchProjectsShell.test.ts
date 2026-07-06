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
  it("renders the project rail panel from the railPanel bag", () => {
    render(ResearchProjectsShell, {
      props: {
        railPanel: {
          summaries: [summary({ name: "Беларусь" })],
          selectedProjectId: null,
          now: NOW,
        },
        selectedProjectId: null,
      },
    });

    expect(screen.getByText("Беларусь")).toBeTruthy();
    expect(screen.getByPlaceholderText("Поиск проектов")).toBeTruthy();
  });

  it("renders the tabs row under the toolbar and the section placeholder instead of the grid", () => {
    expect(shellSource).toContain("<ProjectTabs");
    expect(shellSource).toContain("{...tabs}");
    const toolbarIndex = shellSource.indexOf("<ProjectToolbar");
    const tabsIndex = shellSource.indexOf("<ProjectTabs");
    const statsIndex = shellSource.indexOf('class="research-projects-shell__statsbar"');
    expect(tabsIndex).toBeGreaterThan(toolbarIndex);
    expect(tabsIndex).toBeLessThan(statsIndex);
    expect(shellSource).toContain("sectionPlaceholder");
    expect(shellSource).toContain("research-projects-shell__section-placeholder");
  });

  it("shows the placeholder text when sectionPlaceholder is provided", () => {
    render(ResearchProjectsShell, {
      props: {
        railPanel: { summaries: [], selectedProjectId: null, now: NOW },
        selectedProjectId: 1,
        sectionPlaceholder: "Раздел «Обзор» в разработке",
      },
    });
    expect(screen.getByText("Раздел «Обзор» в разработке")).toBeTruthy();
  });

  it("passes row activation through to the sources grid", () => {
    expect(shellSource).toContain("{activeSourceId}");
    expect(shellSource).toContain("{onActivateSource}");
  });

  it("renders the rail panel in the aside", () => {
    expect(shellSource).toContain("<ProjectRailPanel");
    expect(shellSource).toContain("{...railPanel}");
    expect(shellSource).not.toContain("ProjectRailSections");
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

  it("forwards project selection through the railPanel bag", async () => {
    const onSelect = vi.fn();
    render(ResearchProjectsShell, {
      props: {
        railPanel: {
          summaries: [summary({ id: 7, name: "Pick me" })],
          selectedProjectId: null,
          now: NOW,
          onSelect,
        },
        selectedProjectId: null,
      },
    });

    await fireEvent.click(screen.getByText("Pick me"));
    expect(onSelect).toHaveBeenCalledWith(7);
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
      props: {
        railPanel: { summaries: [], selectedProjectId: null, now: NOW },
        selectedProjectId: null,
        inspector: inspectorBag,
      },
    });

    expect(screen.getByText("Инспектор источника")).toBeTruthy();
    expect(screen.getByText("ФинБеларусь")).toBeTruthy();
  });

  it("uses a responsive inspector shell that can overlay the grid", () => {
    expect(shellSource).toContain("container-type: inline-size");
    expect(shellSource).toContain("container-name: app");
    expect(shellSource).toContain("research-projects-shell__inspector-backdrop");
    expect(shellSource).toContain("research-projects-shell__inspector");
    expect(shellSource).toContain("@container app (max-width: 1160px)");
    expect(shellSource).toContain("position: absolute");
    expect(shellSource).toContain("width: 324px");
    expect(shellSource).toContain("box-shadow: -10px 0 30px");
  });

  it("provides a named container for responsive source-table controls", () => {
    expect(shellSource).toContain("container-type: inline-size");
    expect(shellSource).toContain("container-name: sources");
  });

  it("closes the open inspector from the overlay backdrop", async () => {
    const onToggle = vi.fn();
    render(ResearchProjectsShell, {
      props: {
        railPanel: { summaries: [], selectedProjectId: null, now: NOW },
        selectedProjectId: null,
        inspector: {
          open: true,
          selected: null,
          periodLabel: "Весь период",
          promptLabel: "По умолчанию",
          modelLabel: "gpt-4.1",
          onToggle,
        },
      },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Закрыть инспектор" }));
    expect(onToggle).toHaveBeenCalledOnce();
  });

  it("renders the run dock for the selected project", () => {
    expect(shellSource).toContain("<RunDock");
    expect(shellSource).toContain("{...runDock}");
  });

  it("renders the bulk bar as an in-flow strip below the stats bar", () => {
    expect(shellSource).toContain("research-projects-shell__statsbar");
    expect(shellSource).toContain("<SourcesFilterBar");
    expect(shellSource).toContain("<SourcesFilterBar {...filterBar} />");
    expect(shellSource).toContain("{...filterBar}");
    expect(shellSource).toContain("<SourcesBulkBar {...bulkBar} />");
    expect(shellSource).toContain("research-projects-shell__bulkbar");
    expect(shellSource).toContain("<SourcesFilterRow");
    expect(shellSource).toContain("<SourcesFilterRow {...filterRow} />");
    expect(shellSource).toContain("{...filterRow}");
    const statsIndex = shellSource.indexOf('class="research-projects-shell__statsbar"');
    const statsCloseIndex = shellSource.indexOf("</div>", statsIndex);
    const bulkIndex = shellSource.indexOf("<SourcesBulkBar");
    const bulkWrapperIndex = shellSource.indexOf('class="research-projects-shell__bulkbar"');
    const gridIndex = shellSource.indexOf('class="research-projects-shell__grid"');
    expect(statsIndex).toBeGreaterThan(-1);
    expect(statsCloseIndex).toBeGreaterThan(statsIndex);
    expect(bulkWrapperIndex).toBeGreaterThan(statsCloseIndex);
    expect(bulkIndex).toBeGreaterThan(bulkWrapperIndex);
    expect(bulkIndex).toBeLessThan(gridIndex);
    expect(shellSource).toContain("overlay={gridOverlay}");
  });
});
