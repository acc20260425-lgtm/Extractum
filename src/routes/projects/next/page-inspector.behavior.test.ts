import { expect, it, vi } from "vitest";

it("passes v11 source details and open-source action to the inspector", async () => {
  const modulePath = "./page-inspector";
  const inspector = await import(/* @vite-ignore */ modulePath);
  const row = {
    title: "A video",
    handle: "video-id",
    typeLabel: "Video",
    typeDot: "red",
  };
  const openUrl = vi.fn(async () => undefined);
  const selection = inspector.projectInspectorSelection(row);
  const url = inspector.youtubeProjectSourceUrl({
    provider: "youtube",
    source_subtype: "video",
    handle: "video-id",
  });
  const actions = inspector.projectInspectorActions({ url, openUrl });

  expect(selection).toMatchObject({ typeLabel: "Video", typeDot: "red" });
  expect(url).toBe("https://www.youtube.com/watch?v=video-id");
  expect(actions.openDisabled).toBe(false);
  await actions.onOpen();
  expect(openUrl).toHaveBeenCalledWith(url);
});
