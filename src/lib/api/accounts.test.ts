import { beforeEach, describe, expect, expectTypeOf, it, vi } from "vitest";
import {
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
  TELEGRAM_ACCOUNT_STATUS_EVENT,
} from "./accounts";
import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("account api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("loads accounts with the registered command name", async () => {
    const accounts: AccountRecord[] = [{
      id: 1,
      label: "Main",
      api_id: 123,
      phone: "+100",
      created_at: 10,
    }];
    invokeMock.mockResolvedValueOnce(accounts);

    await expect(listAccounts()).resolves.toEqual(accounts);

    expect(invokeMock).toHaveBeenLastCalledWith("list_accounts");
  });

  it("loads an account with the expected payload", async () => {
    const account: AccountRecord = {
      id: 7,
      label: "Personal",
      api_id: 456,
      phone: null,
      created_at: 20,
    };
    invokeMock.mockResolvedValueOnce(account);

    await expect(getAccount(7)).resolves.toEqual(account);

    expect(invokeMock).toHaveBeenLastCalledWith("get_account", { accountId: 7 });
  });

  it("creates an account with the expected payload", async () => {
    const account: AccountRecord = {
      id: 8,
      label: "Work",
      api_id: 789,
      phone: null,
      created_at: 30,
    };
    invokeMock.mockResolvedValueOnce(account);

    await expect(createAccount({
      label: "Work",
      apiId: 789,
      apiHash: "hash",
    })).resolves.toEqual(account);

    expect(invokeMock).toHaveBeenLastCalledWith("create_account", {
      label: "Work",
      apiId: 789,
      apiHash: "hash",
    });
  });

  it("deletes an account with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteAccount(8)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("delete_account", { accountId: 8 });
  });

  it("sets and clears an account phone with the expected payloads", async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await expect(setAccountPhone({ accountId: 8, phone: "+123" })).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("set_account_phone", {
      accountId: 8,
      phone: "+123",
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await expect(clearAccountPhone(8)).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("clear_account_phone", { accountId: 8 });
  });

  it("loads account runtime statuses for the given account ids", async () => {
    const statuses: AccountRuntimeStatus[] = [{
      account_id: 1,
      status: "ready",
      message: null,
    }];
    invokeMock.mockResolvedValueOnce(statuses);

    await expect(getAccountRuntimeStatuses([1, 2])).resolves.toEqual(statuses);

    expect(invokeMock).toHaveBeenLastCalledWith("tg_get_account_statuses", {
      accountIds: [1, 2],
    });
  });

  it("initializes a Telegram account with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(true);

    await expect(initializeTelegramAccount(8)).resolves.toBe(true);

    expect(invokeMock).toHaveBeenLastCalledWith("tg_init", { accountId: 8 });
  });

  it("sends a Telegram code and signs in with the expected payloads", async () => {
    invokeMock.mockResolvedValueOnce("Code sent");
    const sendCodeResult = sendTelegramCode({ accountId: 8, phone: "+123" });
    expectTypeOf(sendCodeResult).toEqualTypeOf<Promise<string>>();
    await expect(sendCodeResult).resolves.toBe("Code sent");
    expect(invokeMock).toHaveBeenLastCalledWith("tg_send_code", {
      accountId: 8,
      phone: "+123",
    });

    invokeMock.mockResolvedValueOnce(true);
    const signInResult = signInTelegramAccount({ accountId: 8, code: "12345" });
    expectTypeOf(signInResult).toEqualTypeOf<Promise<boolean>>();
    await expect(signInResult).resolves.toBe(true);
    expect(invokeMock).toHaveBeenLastCalledWith("tg_sign_in", {
      accountId: 8,
      code: "12345",
    });
  });

  it("logs out a Telegram account with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(true);

    const logoutResult = logoutTelegramAccount(8);
    expectTypeOf(logoutResult).toEqualTypeOf<Promise<boolean>>();
    await expect(logoutResult).resolves.toBe(true);

    expect(invokeMock).toHaveBeenLastCalledWith("tg_logout", { accountId: 8 });
  });

  it("listens on the shared Telegram account status event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToAccountRuntimeStatus(handler)).resolves.toBe(unlisten);
    expect(TELEGRAM_ACCOUNT_STATUS_EVENT).toBe("telegram://account-status");
    expect(listenMock).toHaveBeenCalledWith(TELEGRAM_ACCOUNT_STATUS_EVENT, expect.any(Function));

    const event = {
      payload: {
        account_id: 1,
        status: "ready",
        message: null,
      } satisfies AccountRuntimeStatus,
    };
    listenMock.mock.calls[0][1](event);
    expect(handler).toHaveBeenCalledWith(event);
  });
});
