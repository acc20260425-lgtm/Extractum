// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SourcesTab from "./SourcesTab.svelte";
import {
  type LibrarySourceView,
  PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM,
  type ProjectSourceLinkView,
  type ResearchProjectView,
} from "$lib/ui/research-projects-model";

beforeAll(() => {
  class ResizeObserverStub {
    observe() {}
    unobserve() {}
    disconnect() {}
  }

  vi.stubGlobal("ResizeObserver", ResizeObserverStub);
});

afterEach(cleanup);

const project: ResearchProjectView = {
  id: "project:1",
  projectId: 1,
  title: "Project",
  description: null,
  periodLabel: "All time",
  sourceCount: 1,
  evidenceCount: 1,
  materialCount: 1,
  lastRunLabel: null,
  status: "ready",
  backing: { kind: "project", projectId: 1 },
};

const youtubeVideoLink: ProjectSourceLinkView = {
  projectId: "project:1",
  sourceId: "source:10",
  sourceNumericId: 10,
  provider: "youtube",
  subtype: "video",
  typeLabel: "YouTube / Video",
  title: "Video",
  subtitle: "Channel",
  itemCount: 1,
  localCopyLabel: "1 material",
  addedAt: 1,
  addedAtLabel: null,
  connectionStatus: "connected",
  filterSummary: "Channel",
};

type SourcesTabProps = {
  project: ResearchProjectView | null;
  projectSourceLinks: ProjectSourceLinkView[];
  librarySources: LibrarySourceView[];
  selectedSourceIds: string[];
  saving?: boolean;
  onSelectedSourceIdsChange: (sourceIds: string[]) => void;
  onOpenAddSource: () => void;
  onOpenConnectLibrary: () => void;
  onRemoveSource: (sourceId: number | number[]) => void | Promise<void>;
  onDeleteProjectSourceFromLibrary?: (sourceId: number) => void | Promise<void>;
  onSyncSelectedSources: (sourceIds: number[]) => void | Promise<void>;
};

function renderSourcesTab(overrides: Partial<SourcesTabProps> = {}) {
  const props: SourcesTabProps = {
    project,
    projectSourceLinks: [youtubeVideoLink],
    librarySources: [],
    selectedSourceIds: ["source:10"],
    onSelectedSourceIdsChange: vi.fn(),
    onOpenAddSource: vi.fn(),
    onOpenConnectLibrary: vi.fn(),
    onRemoveSource: vi.fn(),
    onSyncSelectedSources: vi.fn(),
    ...overrides,
  };

  return render(SourcesTab, {
    props,
  });
}

describe("SourcesTab", () => {
  it("confirms before deleting a selected YouTube video source from Library", async () => {
    const onDeleteProjectSourceFromLibrary = vi.fn();
    const onSelectedSourceIdsChange = vi.fn();
    renderSourcesTab({ onDeleteProjectSourceFromLibrary, onSelectedSourceIdsChange });

    await fireEvent.click(
      screen.getByRole("button", { name: "Delete selected YouTube video from Library" }),
    );

    expect(onDeleteProjectSourceFromLibrary).not.toHaveBeenCalled();
    expect(screen.getByText(PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM)).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Delete from Library permanently" }));

    expect(onDeleteProjectSourceFromLibrary).toHaveBeenCalledWith(10);
    expect(onSelectedSourceIdsChange).toHaveBeenCalledWith([]);
  });

  it("keeps the source when Library deletion confirmation is cancelled", async () => {
    const onDeleteProjectSourceFromLibrary = vi.fn();
    renderSourcesTab({ onDeleteProjectSourceFromLibrary });

    await fireEvent.click(
      screen.getByRole("button", { name: "Delete selected YouTube video from Library" }),
    );
    await fireEvent.click(screen.getByRole("button", { name: "Cancel Library deletion" }));

    expect(onDeleteProjectSourceFromLibrary).not.toHaveBeenCalled();
  });
});
