import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeAll, expect, it } from "vitest";
import { dismissActiveModal, openConfirmModal } from "$lib/modals";
import ModalHost from "./modal-host.svelte";

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

afterEach(dismissActiveModal);
afterEach(cleanup);

it("modal host > requires explicit confirmation or cancellation", async () => {
  render(ModalHost);
  const cancelled = openConfirmModal({
    title: "Delete source?",
    message: "This removes Belarus election coverage from the Library.",
    confirmLabel: "Delete source",
    cancelLabel: "Keep source",
    tone: "danger",
  });

  expect(await screen.findByRole("alertdialog")).toBeTruthy();
  expect(screen.getByRole("heading", { name: "Delete source?" })).toBeTruthy();
  const backdrop = document.querySelector<HTMLElement>(".modal-backdrop");
  expect(backdrop).not.toBeNull();
  await fireEvent.pointerDown(backdrop!);
  await fireEvent.click(backdrop!);
  expect(screen.getByRole("alertdialog")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Keep source" }));
  await expect(cancelled).resolves.toBe(false);

  const confirmed = openConfirmModal({
    title: "Delete source?",
    message: "This removes Belarus election coverage from the Library.",
    confirmLabel: "Delete source",
    cancelLabel: "Keep source",
    tone: "danger",
  });
  await fireEvent.click(await screen.findByRole("button", { name: "Delete source" }));
  await expect(confirmed).resolves.toBe(true);
});
