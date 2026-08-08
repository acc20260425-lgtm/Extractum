import { describe, expect, it, vi } from "vitest";

describe("library prototype contract", () => {
  it("renders Library as a separate route backed by the current workflow", async () => {
    const modulePath = "./library-page";
    const page = await import(/* @vite-ignore */ modulePath);
    const state = page.createLibraryPageState();
    const listCatalog = vi.fn(async () => ({ sources: [], filter_counts: [] }));
    const workflow = page.createLibraryPageWorkflow({
      state,
      listCatalog,
      formatError: vi.fn((action, error) => `${action}: ${String(error)}`),
    });

    expect(page.LIBRARY_PAGE_ROUTE).toEqual({ id: "library-prototype", href: "/projects/library" });
    await workflow.loadLibrary();
    expect(listCatalog).toHaveBeenCalledTimes(1);
    expect(state).toMatchObject({ catalogRecords: [], sources: [], loading: false });
  });
});
