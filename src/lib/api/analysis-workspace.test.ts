import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getWorkspaceAccountStatuses,
  listAnalysisSources,
  listWorkspaceAccounts,
} from "./analysis-workspace";
import type { AccountRuntimeStatus, AccountRecord } from "$lib/types/accounts";
import type { AnalysisSourceOption } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("analysis workspace api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads workspace accounts with the registered command name", async () => {
    const accounts: AccountRecord[] = [{
      id: 1,
      label: "Main",
      api_id: 123,
      phone: "+100",
      created_at: 10,
    }];
    invokeMock.mockResolvedValueOnce(accounts);

    await expect(listWorkspaceAccounts()).resolves.toEqual(accounts);

    expect(invokeMock).toHaveBeenLastCalledWith("list_accounts");
  });

  it("loads account runtime statuses for the given account ids", async () => {
    const statuses: AccountRuntimeStatus[] = [{
      account_id: 1,
      status: "ready",
      message: null,
    }];
    invokeMock.mockResolvedValueOnce(statuses);

    await expect(getWorkspaceAccountStatuses([1, 2])).resolves.toEqual(statuses);

    expect(invokeMock).toHaveBeenLastCalledWith("tg_get_account_statuses", {
      accountIds: [1, 2],
    });
  });

  it("loads analysis source metrics with the registered command name", async () => {
    const sources: AnalysisSourceOption[] = [{
      id: 7,
      account_id: 1,
      title: "Source",
      item_count: 12,
      last_synced_at: 100,
    }];
    invokeMock.mockResolvedValueOnce(sources);

    await expect(listAnalysisSources()).resolves.toEqual(sources);

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_sources");
  });
});
