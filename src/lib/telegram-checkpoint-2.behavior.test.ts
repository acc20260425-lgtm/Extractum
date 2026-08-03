import { beforeEach, expect, it, vi } from "vitest";
import {
  TELEGRAM_ACCOUNT_STATUS_EVENT,
  clearAccountPhone,
  createAccount,
  deleteAccount,
  getAccount,
  getAccountRuntimeStatuses,
  initializeTelegramAccount,
  listAccounts,
  listenToAccountRuntimeStatus,
  logoutTelegramAccount,
  sendTelegramCode,
  setAccountPhone,
  signInTelegramAccount,
} from "$lib/api/accounts";
import {
  TAKEOUT_IMPORT_EVENT,
  cancelTakeoutSourceImport,
  listenToTakeoutImportEvents,
} from "$lib/api/takeout-import";
import type { AccountRuntimeStatus } from "$lib/types/accounts";
import type { TakeoutImportEvent } from "$lib/types/sources";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: listenMock }));

beforeEach(() => {
  invokeMock.mockReset();
  listenMock.mockReset();
});

it("pins frontend Telegram command IPC names and default camelCase keys", async () => {
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

it("pins Telegram event emission, status mutation, login result, and session ordering", async () => {
  const unlisten = vi.fn();
  const statusHandler = vi.fn();
  listenMock.mockResolvedValueOnce(unlisten);
  invokeMock.mockResolvedValueOnce(true).mockResolvedValueOnce(true);

  await expect(listenToAccountRuntimeStatus(statusHandler)).resolves.toBe(unlisten);
  await expect(signInTelegramAccount({ accountId: 7, code: "12345" })).resolves.toBe(true);
  await expect(logoutTelegramAccount(7)).resolves.toBe(true);

  const status: AccountRuntimeStatus = {
    account_id: 7,
    status: "ready",
    message: null,
  };
  listenMock.mock.calls[0][1]({ payload: status });

  expect(listenMock).toHaveBeenCalledWith(TELEGRAM_ACCOUNT_STATUS_EVENT, expect.any(Function));
  expect(TELEGRAM_ACCOUNT_STATUS_EVENT).toBe("telegram://account-status");
  expect(statusHandler).toHaveBeenCalledWith({ payload: status });
  expect(invokeMock.mock.calls).toEqual([
    ["tg_sign_in", { accountId: 7, code: "12345" }],
    ["tg_logout", { accountId: 7 }],
  ]);
});

it("pins Takeout mutation-before-event and terminal cancellation selection", async () => {
  const unlisten = vi.fn();
  const eventHandler = vi.fn();
  const terminalEvent: TakeoutImportEvent = {
    job_id: "takeout-7",
    source_id: 3,
    account_id: 7,
    batch_id: 12,
    history_scope: "current_history",
    status: "cancelled",
    phase: "cancelled",
    message: "Cancelled",
    inserted: 4,
    skipped: 1,
    progress_current: 4,
    progress_total: 4,
    started_at: 100,
    finished_at: 101,
    warnings: [],
    error: null,
  };
  listenMock.mockResolvedValueOnce(unlisten);
  invokeMock.mockResolvedValueOnce({ cancelled: true });

  await expect(listenToTakeoutImportEvents(eventHandler)).resolves.toBe(unlisten);
  await expect(cancelTakeoutSourceImport("takeout-7")).resolves.toEqual({ cancelled: true });
  listenMock.mock.calls[0][1]({ payload: terminalEvent });

  expect(listenMock).toHaveBeenCalledWith(TAKEOUT_IMPORT_EVENT, expect.any(Function));
  expect(eventHandler).toHaveBeenCalledWith({ payload: terminalEvent });
  expect(invokeMock).toHaveBeenCalledWith("cancel_takeout_source_import", { jobId: "takeout-7" });
});
