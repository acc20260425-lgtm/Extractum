import { describe, expect, it, vi } from "vitest";
import {
  connectProjectSourceIds,
  connectedSourceIdsForProject,
  normalizeProjectSourceIds,
} from "./project-add-source-workflow";
import type { ProjectSourceRecord } from "$lib/types/projects";

function deps() {
  return {
    addProjectSources: vi.fn(),
    refreshAfterProjectSourceConnect: vi.fn(),
    setProjectAddSourceSaving: vi.fn(),
    setProjectAddSourceStatus: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };
}

describe("project add-source workflow", () => {
  it("normalizes source IDs before calling addProjectSources", async () => {
    const d = deps();
    d.addProjectSources.mockResolvedValue({ added_count: 1, already_present_count: 0 });

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10, 10, null, undefined, Number.NaN, 11],
      origin: "new_source",
      emptyBehavior: "missing_source_id_status",
      deps: d,
    });

    expect(d.addProjectSources).toHaveBeenCalledOnce();
    expect(d.addProjectSources).toHaveBeenCalledWith({ projectId: 7, sourceIds: [10, 11] });
    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith("Source added and connected to project.");
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
  });

  it("reports an already-present project connection from backend outcome", async () => {
    const d = deps();
    d.addProjectSources.mockResolvedValue({ added_count: 0, already_present_count: 1 });

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10],
      origin: "existing_source",
      deps: d,
    });

    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith("Already connected to this project.");
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
  });

  it("reports existing Library source connection success", async () => {
    const d = deps();
    d.addProjectSources.mockResolvedValue({ added_count: 1, already_present_count: 0 });

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10],
      origin: "existing_source",
      deps: d,
    });

    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith("Already in Library. Connected to project.");
  });

  it("does not claim a connection when the backend reports no changed or existing rows", async () => {
    const d = deps();
    d.addProjectSources.mockResolvedValue({ added_count: 0, already_present_count: 0 });

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10],
      origin: "new_source",
      deps: d,
    });

    expect(d.setProjectAddSourceStatus).not.toHaveBeenCalled();
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
  });

  it("refreshes and reports scalar missing source ID without connecting", async () => {
    const d = deps();

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [undefined],
      origin: "new_source",
      emptyBehavior: "missing_source_id_status",
      deps: d,
    });

    expect(d.addProjectSources).not.toHaveBeenCalled();
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith(
      "Source added to Library, but auto-connect could not be completed.",
    );
  });

  it("keeps empty playlist batch silent and does not connect", async () => {
    const d = deps();

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [],
      origin: "new_source",
      emptyBehavior: "silent",
      deps: d,
    });

    expect(d.addProjectSources).not.toHaveBeenCalled();
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
    expect(d.setProjectAddSourceStatus).not.toHaveBeenCalled();
  });

  it("keeps Library add success visible when project connect fails", async () => {
    const d = deps();
    d.addProjectSources.mockRejectedValue(new Error("network"));

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10],
      origin: "new_source",
      deps: d,
    });

    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith(
      "Source added to Library, but connecting it to the project failed: Error connecting source to project: Error: network",
    );
  });

  it("derives connected source IDs from project source records", () => {
    const rows: Pick<ProjectSourceRecord, "project_id" | "source_id">[] = [
      { project_id: 7, source_id: 10 },
      { project_id: 7, source_id: 11 },
      { project_id: 8, source_id: 12 },
    ];

    expect([...connectedSourceIdsForProject(rows, 7)]).toEqual([10, 11]);
    expect([...connectedSourceIdsForProject(rows, null)]).toEqual([]);
  });

  it("filters non-finite and duplicate IDs", () => {
    expect(normalizeProjectSourceIds([1, 1, null, undefined, Number.NaN, 2])).toEqual([1, 2]);
  });
});
