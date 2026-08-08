import { getByRole } from "@testing-library/dom";
import { createRequire } from "node:module";
import { render } from "svelte/server";
import { describe, expect, it, vi } from "vitest";

import {
  apalisJobsNavigationItem,
  createApalisJobsRouteOrchestration,
} from "$lib/apalis-jobs-route";
import JobsPage from "../routes/jobs/+page.svelte";

const JSDOM = (createRequire(import.meta.url)("jsdom") as {
  JSDOM: new (html: string) => { window: { document: Document } };
}).JSDOM;

describe("apalis jobs inspector frontend source contracts", () => {
  it("adds Jobs as a separate top-level navigation item in both modes", () => {
    for (const mode of ["legacy", "projects"] as const) {
      const item = apalisJobsNavigationItem(mode);
      expect(item).toMatchObject({ href: "/jobs", label: "Jobs", caption: "Apalis queue" });
      expect(item.active("/jobs/queued")).toBe(true);
      expect(item.active("/settings")).toBe(false);
    }
  });

  it("implements manual refresh and guarded pruning without auto polling", async () => {
    const body = new JSDOM(render(JobsPage).body).window.document.body;
    expect(getByRole(body, "button", { name: /Refresh/ })).toBeTruthy();
    expect(getByRole(body, "button", { name: /Delete old finished jobs/ })).toBeTruthy();

    const prune = vi.fn().mockResolvedValue({ deletedCount: 2, cutoffAt: "2026-08-07", olderThanHours: 24 });
    const refresh = vi.fn().mockResolvedValue(undefined);
    const schedule = vi.fn();
    const rejected = createApalisJobsRouteOrchestration({ confirmPrune: () => false, schedule });
    await rejected.manualRefresh(refresh);
    expect(refresh).toHaveBeenCalledOnce();
    expect(schedule).not.toHaveBeenCalled();
    await expect(rejected.guardedPrune(prune, refresh)).resolves.toBeNull();
    expect(prune).not.toHaveBeenCalled();
    expect(refresh).toHaveBeenCalledOnce();
    expect(schedule).not.toHaveBeenCalled();

    const accepted = createApalisJobsRouteOrchestration({ confirmPrune: () => true, schedule });
    await expect(accepted.guardedPrune(prune, refresh)).resolves.toMatchObject({ deletedCount: 2 });
    expect(prune).toHaveBeenCalledOnce();
    expect(refresh).toHaveBeenCalledTimes(2);
    expect(schedule).not.toHaveBeenCalled();
  });
});
