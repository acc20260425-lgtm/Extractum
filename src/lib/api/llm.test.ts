import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  askLlmStream,
  cancelLlmRequest,
  clearLlmProfileApiKey,
  getLlmProfiles,
  LLM_RESPONSE_EVENT,
  listLlmProviderModels,
  listenToLlmResponses,
  saveLlmProfile,
} from "./llm";
import type { LlmStreamEvent } from "$lib/types/llm";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("llm api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("loads and saves provider profiles through typed command wrappers", async () => {
    invokeMock.mockResolvedValueOnce({ active_profile: "default", profiles: [] });
    await expect(getLlmProfiles()).resolves.toEqual({ active_profile: "default", profiles: [] });
    expect(invokeMock).toHaveBeenLastCalledWith("get_llm_profiles");

    invokeMock.mockResolvedValueOnce({ active_profile: "work", profiles: [] });
    await saveLlmProfile({
      profileId: "work",
      provider: "gemini",
      defaultModel: "gemini-2.5-flash",
      apiKey: null,
      baseUrl: "",
      setActive: true,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("save_llm_profile", {
      profileId: "work",
      provider: "gemini",
      defaultModel: "gemini-2.5-flash",
      apiKey: null,
      baseUrl: "",
      setActive: true,
    });

    invokeMock.mockResolvedValueOnce({ active_profile: "work", profiles: [] });
    await clearLlmProfileApiKey("work");
    expect(invokeMock).toHaveBeenLastCalledWith("clear_llm_profile_api_key", {
      profileId: "work",
    });
  });

  it("wraps model listing with provider, key, and base url arguments", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await listLlmProviderModels({
      provider: "omniroute",
      apiKey: "secret",
      baseUrl: "http://localhost:20128/v1",
    });

    expect(invokeMock).toHaveBeenCalledWith("list_llm_provider_models", {
      provider: "omniroute",
      apiKey: "secret",
      baseUrl: "http://localhost:20128/v1",
    });
  });

  it("wraps stream start and cancellation commands", async () => {
    await askLlmStream({
      requestId: "settings-test-1",
      profileId: "default",
      messages: [{ role: "user", content: "hello" }],
      modelOverride: null,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("ask_llm_stream", {
      requestId: "settings-test-1",
      profileId: "default",
      messages: [{ role: "user", content: "hello" }],
      modelOverride: null,
    });

    await cancelLlmRequest("settings-test-1");
    expect(invokeMock).toHaveBeenLastCalledWith("cancel_llm_request", {
      requestId: "settings-test-1",
    });
  });

  it("listens on the shared LLM response event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToLlmResponses(handler)).resolves.toBe(unlisten);
    expect(LLM_RESPONSE_EVENT).toBe("llm://response");
    expect(listenMock).toHaveBeenCalledWith(LLM_RESPONSE_EVENT, expect.any(Function));

    const event = { payload: { request_id: "r1", kind: "completed" } as LlmStreamEvent };
    listenMock.mock.calls[0][1](event);
    expect(handler).toHaveBeenCalledWith(event);
  });
});
