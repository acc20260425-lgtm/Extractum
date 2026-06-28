import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  addProjectSources,
  createProject,
  deleteProject,
  getProjectDataRange,
  listProjectRuns,
  listProjectSources,
  listProjects,
  listResearchProjects,
  removeProjectSources,
  setProjectArchived,
  setProjectPinned,
  startProjectAnalysis,
  updateProject,
} from "./projects";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const invokeMock = vi.mocked(invoke);

describe("projects api", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps project crud commands", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await listProjects();
    expect(invokeMock).toHaveBeenLastCalledWith("list_projects");

    invokeMock.mockResolvedValueOnce({ id: 1 });
    await createProject({ name: "Alpha", description: "Desc" });
    expect(invokeMock).toHaveBeenLastCalledWith("create_project", {
      name: "Alpha",
      description: "Desc",
    });

    invokeMock.mockResolvedValueOnce({ id: 1 });
    await updateProject({ projectId: 1, name: "Beta", description: null });
    expect(invokeMock).toHaveBeenLastCalledWith("update_project", {
      projectId: 1,
      name: "Beta",
      description: null,
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await deleteProject(1);
    expect(invokeMock).toHaveBeenLastCalledWith("delete_project", { projectId: 1 });
  });

  it("maps source membership and run commands", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await listProjectSources(2);
    expect(invokeMock).toHaveBeenLastCalledWith("list_project_sources", { projectId: 2 });

    invokeMock.mockResolvedValueOnce({ added_count: 2, already_present_count: 1 });
    await addProjectSources({ projectId: 2, sourceIds: [5, 6, 5] });
    expect(invokeMock).toHaveBeenLastCalledWith("add_project_sources", {
      projectId: 2,
      sourceIds: [5, 6, 5],
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await removeProjectSources({ projectId: 2, sourceIds: [5] });
    expect(invokeMock).toHaveBeenLastCalledWith("remove_project_sources", {
      projectId: 2,
      sourceIds: [5],
    });

    invokeMock.mockResolvedValueOnce([]);
    await listProjectRuns(2);
    expect(invokeMock).toHaveBeenLastCalledWith("list_project_runs", { projectId: 2 });

    invokeMock.mockResolvedValueOnce(77);
    await startProjectAnalysis({
      projectId: 2,
      periodFrom: 1,
      periodTo: 2,
      outputLanguage: "en",
      promptTemplateId: 3,
      modelOverride: null,
      profileId: null,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("start_project_analysis", {
      projectId: 2,
      periodFrom: 1,
      periodTo: 2,
      outputLanguage: "en",
      promptTemplateId: 3,
      modelOverride: null,
      profileId: null,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });
  });

  it("maps research projects v10 commands", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await listResearchProjects();
    expect(invokeMock).toHaveBeenLastCalledWith("list_research_projects");

    invokeMock.mockResolvedValueOnce({ from: 10, to: 20 });
    await getProjectDataRange({
      projectId: 2,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("get_project_data_range", {
      projectId: 2,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await setProjectPinned({ projectId: 2, pinned: true });
    expect(invokeMock).toHaveBeenLastCalledWith("set_project_pinned", {
      projectId: 2,
      pinned: true,
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await setProjectArchived({ projectId: 2, archived: true });
    expect(invokeMock).toHaveBeenLastCalledWith("set_project_archived", {
      projectId: 2,
      archived: true,
    });
  });
});
