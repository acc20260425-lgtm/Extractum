import { describe, expect, it } from "vitest";
import {
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  projectRunDisabledReason,
} from "./research-projects-model";
import type { LibrarySourceRecord } from "$lib/types/library-sources";
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
  },
];

const library: LibrarySourceRecord[] = [
  {
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
    const rows = buildLibrarySourcesView(library, projectSources, "project:1", []);
    expect(rows[0]).toMatchObject({
      sourceId: 10,
      alreadyConnected: true,
      connectable: false,
      disabledReason: "Already in project",
    });
  });

  it("builds project source table rows and disables mixed-provider runs", () => {
    const links = buildProjectSourceLinksView("project:1", projectSources);
    expect(links[0]).toMatchObject({
      projectId: "project:1",
      sourceId: "source:10",
      provider: "youtube",
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
    ).toBe("Mixed-provider project runs are not supported yet.");
  });
});
