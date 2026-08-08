import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({
  askLlmStream: vi.fn(),
  cancelLlmRequest: vi.fn(),
  clearLlmProfileApiKey: vi.fn(),
  getLlmProfiles: vi.fn(),
  listLlmProviderModels: vi.fn(),
  listenToLlmResponses: vi.fn(),
  saveLlmProfile: vi.fn(),
}));

vi.mock("$lib/api/llm", () => api);

import SettingsPage from "./+page.svelte";

beforeEach(() => {
  localStorage.setItem("extractum.uiMode", "legacy");
  api.getLlmProfiles.mockResolvedValue({ profiles: [], active_profile: "default" });
  api.listLlmProviderModels.mockResolvedValue([]);
  api.listenToLlmResponses.mockResolvedValue(() => {});
});
afterEach(cleanup);
afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

it("keeps Settings focused on LLM configuration", async () => {
  const view = render(SettingsPage);
  await waitFor(() => expect(api.getLlmProfiles).toHaveBeenCalledOnce());

  expect(screen.getByRole("heading", { name: "LLM profiles" })).not.toBeNull();
  expect(screen.getByText("Settings stay focused on LLM provider profiles and test runs.")).not.toBeNull();
  expect(view.container.querySelector('[data-settings-focus="llm-provider-profiles provider-test-runs"]')).not.toBeNull();
  expect(screen.queryByText("YouTube access")).toBeNull();
  expect(screen.queryByText("Telegram accounts")).toBeNull();
});
