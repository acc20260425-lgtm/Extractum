import { beforeEach, expect, it, vi } from "vitest";
import {
  logoutTelegramAccount,
  signInTelegramAccount,
} from "$lib/api/accounts";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

beforeEach(() => {
  invokeMock.mockReset();
});

it("uses one production session temp-path helper with the frozen extension", async () => {
  invokeMock.mockResolvedValueOnce(true).mockResolvedValueOnce(true);

  await expect(signInTelegramAccount({ accountId: 7, code: "12345" })).resolves.toBe(true);
  await expect(logoutTelegramAccount(7)).resolves.toBe(true);

  expect(invokeMock.mock.calls).toEqual([
    ["tg_sign_in", { accountId: 7, code: "12345" }],
    ["tg_logout", { accountId: 7 }],
  ]);
});
