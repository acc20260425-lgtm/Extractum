import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeAll, expect, it, vi } from "vitest";
import DesktopDialog from "./desktop-dialog.svelte";

beforeAll(() => {
  Object.defineProperty(Element.prototype, "animate", {
    configurable: true,
    value: () => {
      const animation = { cancel: () => {}, onfinish: null } as unknown as Animation;
      setTimeout(() => animation.onfinish?.(new Event("finish") as AnimationPlaybackEvent), 0);
      return animation;
    },
  });
});

afterEach(cleanup);

it("desktop dialog > requires explicit dismissal", async () => {
  const onClose = vi.fn();
  render(DesktopDialog, {
    open: true,
    title: "Edit Telegram source",
    description: "Update the source configuration.",
    smokeId: "edit-source",
    onClose,
  });

  expect(screen.getByRole("dialog")).toBeTruthy();
  expect(screen.getByRole("heading", { name: "Edit Telegram source" })).toBeTruthy();
  expect(screen.getByText("Update the source configuration.")).toBeTruthy();

  const backdrop = document.querySelector<HTMLElement>(".dialog-backdrop");
  expect(backdrop).not.toBeNull();
  await fireEvent.pointerDown(backdrop!);
  await fireEvent.click(backdrop!);
  expect(onClose).not.toHaveBeenCalled();

  await fireEvent.click(screen.getByRole("button", { name: "Close dialog" }));
  expect(onClose).toHaveBeenCalledOnce();
});
