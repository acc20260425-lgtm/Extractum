import { describe, expect, it } from "vitest";
import {
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM,
  projectSourceLibraryDeleteStatus,
  projectRunDisabledReason,
  reconcileProjectSourceSelection,
  selectedProjectSourceLibraryDeleteDisabledReason,
  selectedProjectSourcesSyncDisabledReason,
} from "./research-projects-model";
import type { LibraryCatalogRecord } from "$lib/types/library-sources";
import type { ProjectRecord, ProjectSourceRecord } from "$lib/types/projects";

const projects: ProjectRecord[] = [
  { id: 1, name: "Alpha", description: "Desc", created_at: 100, updated_at: 200 },
];

const projectSources: ProjectSourceRecord[] = [
  {
    project_id: 1,
    source_id: 10,
    provider: "youtube",
    source_subtype: "video",
    title: "Video",
    subtitle: "Channel",
    item_count: 3,
    added_at: 300,
    last_synced_at: 110,
    sync_status: "error",
    handle: "v1",
  },
];

const library: LibraryCatalogRecord[] = [
  {
    source: {
      source_id: 10,
      provider: "youtube",
      source_subtype: "video",
      account_id: null,
      external_id: "v1",
      title: "Video",
      subtitle: "Channel",
      canonical_url: "https://youtu.be/v1",
      created_at: 100,
      last_synced_at: 110,
      item_count: 3,
      project_count: 1,
      youtube: {
        video_form: "video",
        duration_seconds: 120,
        playlist_video_count: null,
        channel_title: "Channel",
        availability_status: "available",
      },
      telegram: null,
    },
    latest_job: null,
    status: "error",
    status_detail: "Last sync failed",
    capabilities: {
      can_refresh_source: true,
      can_delete: false,
      can_edit: false,
      can_connect_to_project: true,
    },
    disabled_reasons: {
      refresh_source: null,
      delete: "Source 10 is used by 1 project(s). Remove it from projects first.",
      edit: "Source editing is not available yet.",
      connect_to_project: null,
    },
  },
];

describe("research projects model", () => {
  it("builds real project cards from project records", () => {
    const rows = buildResearchProjectsView(projects, projectSources, []);
    expect(rows).toMatchObject([
      {
        id: "project:1",
        projectId: 1,
        title: "Alpha",
        description: "Desc",
        sourceCount: 1,
        materialCount: 3,
        status: "ready",
      },
    ]);
  });

  it("marks already connected library sources without hiding them", () => {
    const rows = buildLibrarySourcesView(library, projectSources, "project:1");
    expect(rows[0]).toMatchObject({
      sourceId: 10,
      typeLabel: "YouTube / Video",
      alreadyConnected: true,
      connectable: false,
      disabledReason: "Already in project",
    });
  });

  it("keeps catalog, project records, project rows, and connect API IDs in one numeric id-space", () => {
    const libraryRows = buildLibrarySourcesView(library, projectSources, "project:1");
    const projectRows = buildProjectSourceLinksView("project:1", projectSources);
    const addProjectSourcesPayload = { projectId: 1, sourceIds: [libraryRows[0].sourceId] };

    expect(libraryRows[0].sourceId).toBe(projectSources[0].source_id);
    expect(projectRows[0].sourceNumericId).toBe(projectSources[0].source_id);
    expect(addProjectSourcesPayload.sourceIds).toEqual([projectSources[0].source_id]);
  });

  it("uses catalog disabled reasons as project Library source base state", () => {
    const rows = buildLibrarySourcesView(
      [
        {
          ...library[0],
          capabilities: {
            ...library[0].capabilities,
            can_connect_to_project: false,
          },
          disabled_reasons: {
            ...library[0].disabled_reasons,
            connect_to_project: "Source type cannot be connected.",
          },
        },
      ],
      [],
      "project:1",
    );

    expect(rows[0]).toMatchObject({
      status: "error",
      disabledReason: "Source type cannot be connected.",
      connectable: false,
    });
  });

  it("builds project source table rows and disables mixed-provider runs", () => {
    const links = buildProjectSourceLinksView("project:1", projectSources);
    expect(links[0]).toMatchObject({
      projectId: "project:1",
      sourceId: "source:10",
      provider: "youtube",
      subtype: "video",
      typeLabel: "YouTube / Video",
      title: "Video",
      addedAt: 300,
    });

    expect(projectRunDisabledReason(null, [])).toBe("Select a project");
    expect(projectRunDisabledReason(projects[0], [])).toBe("Add sources to run analysis");
    expect(projectRunDisabledReason(projects[0], projectSources)).toBeNull();
    expect(
      projectRunDisabledReason(projects[0], [
        ...projectSources,
        { ...projectSources[0], source_id: 11, provider: "telegram" },
      ]),
    ).toBe("Mixed-provider project analysis runs are not supported yet.");
  });

  it("keeps source selection scoped to the current project rows", () => {
    const links = buildProjectSourceLinksView("project:1", [
      ...projectSources,
      { ...projectSources[0], project_id: 1, source_id: 11, title: "Second" },
      { ...projectSources[0], project_id: 2, source_id: 12, title: "Other project" },
    ]);

    expect(reconcileProjectSourceSelection(["source:10", "source:12", "source:99"], links)).toEqual([
      "source:10",
    ]);
  });

  it("allows syncing selected YouTube videos only", () => {
    const youtubeRows = buildProjectSourceLinksView("project:1", [
      projectSources[0],
    ]);
    const playlistRows = buildProjectSourceLinksView("project:1", [
      { ...projectSources[0], source_id: 11, source_subtype: "playlist", title: "Playlist" },
    ]);
    const mixedRows = buildProjectSourceLinksView("project:1", [
      projectSources[0],
      { ...projectSources[0], source_id: 12, provider: "telegram", source_subtype: "supergroup" },
    ]);

    expect(selectedProjectSourcesSyncDisabledReason([])).toBe("Select sources to sync");
    expect(selectedProjectSourcesSyncDisabledReason(youtubeRows)).toBeNull();
    expect(selectedProjectSourcesSyncDisabledReason(playlistRows)).toBe(
      "Selected sources include unsupported sync types",
    );
    expect(selectedProjectSourcesSyncDisabledReason(mixedRows)).toBe(
      "Selected sources include unsupported sync types",
    );
  });

  it("requires exactly one selected YouTube video for Library deletion", () => {
    const youtubeRows = buildProjectSourceLinksView("project:1", [
      projectSources[0],
    ]);
    const playlistRows = buildProjectSourceLinksView("project:1", [
      { ...projectSources[0], source_id: 11, source_subtype: "playlist", title: "Playlist" },
    ]);
    const telegramRows = buildProjectSourceLinksView("project:1", [
      { ...projectSources[0], source_id: 12, provider: "telegram", source_subtype: "supergroup" },
    ]);

    expect(selectedProjectSourceLibraryDeleteDisabledReason([])).toBe(
      "Select one YouTube video source",
    );
    expect(selectedProjectSourceLibraryDeleteDisabledReason([...youtubeRows, ...playlistRows])).toBe(
      "Select one YouTube video source",
    );
    expect(selectedProjectSourceLibraryDeleteDisabledReason(youtubeRows)).toBeNull();
    expect(selectedProjectSourceLibraryDeleteDisabledReason(playlistRows)).toBe(
      "Only YouTube videos can be deleted from Library here",
    );
    expect(selectedProjectSourceLibraryDeleteDisabledReason(telegramRows)).toBe(
      "Only YouTube videos can be deleted from Library here",
    );
  });

  it("formats project source Library delete outcomes", () => {
    expect(PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM).toContain(
      "Delete this YouTube video from the project and Library?",
    );
    expect(
      projectSourceLibraryDeleteStatus({
        status: "deleted",
        blocking_projects: [],
        remaining_blocking_project_count: 0,
      }),
    ).toBe("Source deleted from project and Library.");
    expect(
      projectSourceLibraryDeleteStatus({
        status: "blocked_by_other_projects",
        blocking_projects: [
          { project_id: 1, title: "Alpha", archived: false },
          { project_id: 2, title: "Beta", archived: true },
          { project_id: 3, title: "Gamma", archived: false },
        ],
        remaining_blocking_project_count: 2,
      }),
    ).toBe(
      "Cannot delete from Library: source is used by other projects: Alpha, Beta, Gamma, and 2 more.",
    );
    expect(
      projectSourceLibraryDeleteStatus({
        status: "blocked_by_other_projects",
        blocking_projects: [{ project_id: 1, title: "Alpha", archived: false }],
        remaining_blocking_project_count: 0,
      }),
    ).toBe("Cannot delete from Library: source is used by other projects: Alpha.");
  });
});
