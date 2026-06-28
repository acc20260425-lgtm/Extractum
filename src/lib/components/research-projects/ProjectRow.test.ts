// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/svelte";
import ProjectRow from "./ProjectRow.svelte";
import { buildProjectRailRow } from "$lib/ui/research-projects-rail";
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

describe("ProjectRow", () => {
  it("renders the project name and meta", () => {
    const row = buildProjectRailRow(
      summary({ name: "Беларусь", source_count: 10, status: "ready" }),
      NOW,
    );

    render(ProjectRow, { props: { row } });

    expect(screen.getByText("Беларусь")).toBeTruthy();
    expect(screen.getByText("10 источников · готов")).toBeTruthy();
  });
});
