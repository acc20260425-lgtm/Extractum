import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const tauri = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: tauri.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: tauri.listen }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

import AnalysisPage from "./+page.svelte";

afterEach(cleanup);

const rawSource = {
  id: 7, source_type: "telegram", source_subtype: "supergroup", account_id: 2,
  external_id: "research", title: "Research channel", last_sync_state: 4,
  last_synced_at: 1_700_000_000, is_member: true, is_active: true, created_at: 1,
  telegram_username: "research", avatar_data_url: null, migrated_history_status: "imported",
  migrated_history_detected_at: 1, migrated_history_refreshed_at: 2,
  migrated_history_row_count: 14, migrated_history_import_completed: true,
};

function response(command: string) {
  if (command === "list_sources") return [rawSource];
  if (command === "list_analysis_sources") return [{ id: 7, account_id: 2, source_type: "telegram", title: "Research channel", item_count: 38, last_synced_at: 1_700_000_000 }];
  if (command === "get_llm_profiles") return { active_profile: "research", profiles: [] };
  if (command === "get_youtube_runtime_status") return { ytdlpAvailable: false, ytdlpVersion: null, message: "Unavailable" };
  if (command === "list_source_forum_topics") return { topics: [], topic_resolution_state: { status: "never_run", resolver_version: 0, unresolved_count: 0, pending_item_count: 0, memberships_refreshed_at: null } };
  if (command === "list_analysis_run_messages") return { messages: [], next_cursor: null, has_more: false };
  if (command === "list_youtube_transcript_segments") return { segments: [], next_cursor: null, has_more: false };
  if (command === "get_analysis_run_trace") return { refs: [] };
  if (command === "get_analysis_run") return null;
  if (command.startsWith("get_")) return null;
  return [];
}

beforeEach(() => {
  tauri.invoke.mockReset();
  tauri.invoke.mockImplementation(async (command: string) => response(command));
  tauri.listen.mockReset();
  tauri.listen.mockResolvedValue(() => {});
  window.history.replaceState({}, "", "/analysis");
  localStorage.setItem("extractum.analysis.workspace.v1", JSON.stringify({
    version: 1,
    workspaceSelection: { kind: "source", sourceId: 7 },
    canvasMode: "report",
    sourceViewBasis: "live_source",
    companionTab: "runs",
    runs: { historyScope: "all", runFilter: "all", runsFilter: {
      query: "", status: "all", scope: "all", dateFrom: "", dateTo: "", provider: "", model: "", template: "",
    } },
  }));
});

describe("analysis redesign final route contract", () => {
  it("renders the approved three-zone analysis workspace", async () => {
    render(AnalysisPage);
    await screen.findByRole("heading", { name: "Research channel" });

    expect(screen.getAllByRole("complementary")).toHaveLength(2);
    expect(screen.getAllByRole("button", { name: "Open source switcher" })).toHaveLength(2);
    expect(screen.getByLabelText("Analysis context")).toBeTruthy();
    expect(screen.getByRole("tablist", { name: "Report canvas mode" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("tab", { name: "Report" }));
    expect(screen.getByRole("region", { name: "Report setup" })).toBeTruthy();
    expect(screen.getByRole("tablist", { name: "Run companion tabs" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "Evidence" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "Chat" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "Runs" })).toBeTruthy();
  });

  it("does not render the legacy wide analysis workspace surfaces", async () => {
    render(AnalysisPage);
    await fireEvent.click(await screen.findByRole("tab", { name: "Report" }));

    expect(screen.queryByText("Workspace rail")).toBeNull();
    expect(screen.queryByText("Workspace main")).toBeNull();
    expect(screen.queryByText("Workspace inspector")).toBeNull();
    expect(screen.queryByRole("button", { name: "Settings" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Accounts" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Source activity" })).toBeNull();
    expect(screen.queryByText("Inspector mode")).toBeNull();
  });

  it("passes migrated historical scope opt-in through report setup", async () => {
    render(AnalysisPage);
    const setup = await screen.findByRole("region", { name: "Report setup" });
    const checkbox = within(setup).getByRole("checkbox", { name: /Include migrated historical scope/ });

    expect(checkbox).toBeTruthy();
    expect((checkbox as HTMLInputElement).checked).toBe(false);
    await fireEvent.click(checkbox);
    expect((checkbox as HTMLInputElement).checked).toBe(true);
  });
});
