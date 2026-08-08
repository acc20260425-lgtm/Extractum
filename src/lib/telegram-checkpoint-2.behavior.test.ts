import { beforeEach, expect, it, vi } from "vitest";
import {
  clearAccountPhone,
  createAccount,
  deleteAccount,
  getAccount,
  getAccountRuntimeStatuses,
  initializeTelegramAccount,
  listAccounts,
  logoutTelegramAccount,
  sendTelegramCode,
  setAccountPhone,
  signInTelegramAccount,
} from "$lib/api/accounts";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

beforeEach(() => {
  invokeMock.mockReset();
});

it("pins the eleven public account and Telegram wrapper IPC names and default camelCase keys", async () => {
  invokeMock.mockResolvedValue(undefined);

  await listAccounts();
  await getAccount(7);
  await createAccount({ label: "Personal", apiId: 123, apiHash: "hash" });
  await setAccountPhone({ accountId: 7, phone: "+100" });
  await clearAccountPhone(7);
  await deleteAccount(7);
  await initializeTelegramAccount(7);
  await getAccountRuntimeStatuses([7, 8]);
  await sendTelegramCode({ accountId: 7, phone: "+100" });
  await signInTelegramAccount({ accountId: 7, code: "12345" });
  await logoutTelegramAccount(7);

  expect(invokeMock.mock.calls).toEqual([
    ["list_accounts"],
    ["get_account", { accountId: 7 }],
    ["create_account", { label: "Personal", apiId: 123, apiHash: "hash" }],
    ["set_account_phone", { accountId: 7, phone: "+100" }],
    ["clear_account_phone", { accountId: 7 }],
    ["delete_account", { accountId: 7 }],
    ["tg_init", { accountId: 7 }],
    ["tg_get_account_statuses", { accountIds: [7, 8] }],
    ["tg_send_code", { accountId: 7, phone: "+100" }],
    ["tg_sign_in", { accountId: 7, code: "12345" }],
    ["tg_logout", { accountId: 7 }],
  ]);
});
