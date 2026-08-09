import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const tauri = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: tauri.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: tauri.listen }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

import AnalysisPage from "./+page.svelte";

afterEach(cleanup);

function response(command: string) {
  if (command === "list_sources") return [];
  if (command === "list_analysis_sources") return [];
  if (command === "get_llm_profiles") return { active_profile: "research", profiles: [{ profile_id: "research", provider: "openai", default_model: "gpt-default", api_key_configured: true, base_url: "" }] };
  if (command === "list_llm_provider_models") return [{ model: "gpt-fast", name: "gpt-fast", display_name: "GPT Fast", description: "Fast", input_token_limit: 1000, output_token_limit: 500, supported_generation_methods: ["generate"] }];
  if (command === "get_youtube_runtime_status") return { ytdlpAvailable: false, ytdlpVersion: null, message: "Unavailable" };
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
  localStorage.clear();
  window.history.replaceState({}, "", "/analysis");
});

describe("analysis LLM run controls", () => {
  it("loads LLM profiles and provider models for the analysis controls", async () => {
    render(AnalysisPage);
    await waitFor(() => expect(tauri.invoke).toHaveBeenCalledWith("get_llm_profiles"));

    expect(tauri.invoke).toHaveBeenCalledWith("list_llm_provider_models", expect.objectContaining({ profileId: "research", provider: "openai" }));
    await fireEvent.click(screen.getByRole("tab", { name: "Report" }));
    expect(await screen.findByRole("combobox", { name: "LLM profile" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "research - openai/gpt-default" })).toBeTruthy();
    expect(screen.getByRole("combobox", { name: "Model" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "GPT Fast - gpt-fast" })).toBeTruthy();
  });
});
