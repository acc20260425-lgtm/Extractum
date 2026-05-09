import { beforeEach, describe, expect, it, vi } from "vitest";
import { listAnalysisSources } from "./analysis-workspace";
import type { AnalysisSourceOption } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("analysis workspace api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads analysis source metrics with the registered command name", async () => {
    const sources: AnalysisSourceOption[] = [{
      id: 7,
      account_id: 1,
      source_type: "telegram",
      title: "Source",
      item_count: 12,
      last_synced_at: 100,
    }];
    invokeMock.mockResolvedValueOnce(sources);

    await expect(listAnalysisSources()).resolves.toEqual(sources);

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_sources");
  });
});
