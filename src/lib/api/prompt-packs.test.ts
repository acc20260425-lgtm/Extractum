import { beforeEach, describe, expect, it, vi } from "vitest";
import { getPromptPackLibrary } from "./prompt-packs";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("prompt pack api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads prompt pack library with the registered command name", async () => {
    invokeMock.mockResolvedValueOnce({ packs: [] });

    await expect(getPromptPackLibrary()).resolves.toEqual({ packs: [] });

    expect(invokeMock).toHaveBeenCalledWith("get_prompt_pack_library");
  });
});
