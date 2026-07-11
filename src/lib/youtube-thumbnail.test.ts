// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/svelte";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import YoutubeThumbnail from "./components/youtube-thumbnail.svelte";
import {
  resetYoutubeThumbnailCache,
  resolveYoutubeThumbnail,
} from "./youtube-thumbnail";

let observers: TestIntersectionObserver[] = [];

class TestIntersectionObserver {
  constructor(readonly callback: IntersectionObserverCallback) {
    observers.push(this);
  }
  observe() {}
  unobserve() {}
  disconnect() {}
  intersect() {
    this.callback([{ isIntersecting: true } as IntersectionObserverEntry], this as unknown as IntersectionObserver);
  }
}

afterEach(() => {
  cleanup();
  observers = [];
  invoke.mockReset();
  resetYoutubeThumbnailCache();
  vi.unstubAllGlobals();
});

describe("YoutubeThumbnail", () => {
  it("waits to resolve until its observer reports visibility", async () => {
    vi.stubGlobal("IntersectionObserver", TestIntersectionObserver);
    invoke.mockResolvedValue({ kind: "success", dataUrl: "data:image/jpeg;base64,thumb" });

    const { container } = render(YoutubeThumbnail, { props: { url: "https://i.ytimg.com/vi/a/hqdefault.jpg" } });
    expect(invoke).not.toHaveBeenCalled();

    observers[0].intersect();
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("resolve_youtube_thumbnail", { url: "https://i.ytimg.com/vi/a/hqdefault.jpg" }));
    await vi.waitFor(() => expect((container.querySelector("img") as HTMLImageElement).src).toBe("data:image/jpeg;base64,thumb"));
  });

  it("keeps a local fallback when resolution fails", async () => {
    vi.stubGlobal("IntersectionObserver", TestIntersectionObserver);
    invoke.mockResolvedValue({ kind: "terminal_error", message: "not allowed" });

    const { container } = render(YoutubeThumbnail, { props: { url: "https://i.ytimg.com/vi/a/hqdefault.jpg", fallbackSrc: "data:image/png;base64,fallback" } });
    observers[0].intersect();

    await vi.waitFor(() => expect(invoke).toHaveBeenCalled());
    expect((container.querySelector("img") as HTMLImageElement).src).toBe("data:image/png;base64,fallback");
  });

  it("memoizes terminal failures and successful data URLs", async () => {
    invoke
      .mockResolvedValueOnce({ kind: "terminal_error", message: "not allowed" })
      .mockResolvedValueOnce({ kind: "success", dataUrl: "data:image/png;base64,ok" });

    await expect(resolveYoutubeThumbnail("https://i.ytimg.com/invalid")).resolves.toBeNull();
    await expect(resolveYoutubeThumbnail("https://i.ytimg.com/invalid")).resolves.toBeNull();
    await expect(resolveYoutubeThumbnail("https://i.ytimg.com/vi/a/hqdefault.jpg")).resolves.toBe("data:image/png;base64,ok");
    await expect(resolveYoutubeThumbnail("https://i.ytimg.com/vi/a/hqdefault.jpg")).resolves.toBe("data:image/png;base64,ok");
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("evicts the least recently used entry after 128 cached results", async () => {
    invoke.mockImplementation(async (_command: string, { url }: { url: string }) => ({
      kind: "success",
      dataUrl: `data:image/png;base64,${url}`,
    }));

    const firstUrl = "https://i.ytimg.com/vi/first/hqdefault.jpg";
    await resolveYoutubeThumbnail(firstUrl);
    for (let index = 1; index <= 128; index += 1) {
      await resolveYoutubeThumbnail(`https://i.ytimg.com/vi/${index}/hqdefault.jpg`);
    }
    await resolveYoutubeThumbnail(firstUrl);

    expect(invoke).toHaveBeenCalledTimes(130);
  });

  it("retries transient errors instead of caching them", async () => {
    invoke.mockResolvedValue({ kind: "transient_error", message: "offline" });

    await expect(resolveYoutubeThumbnail("https://i.ytimg.com/vi/a/hqdefault.jpg")).resolves.toBeNull();
    await expect(resolveYoutubeThumbnail("https://i.ytimg.com/vi/a/hqdefault.jpg")).resolves.toBeNull();

    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
