import { expect, it, vi } from "vitest";

it("passes keyboard navigation callbacks to the source grid", async () => {
  const modulePath = "./page-keyboard";
  const keyboard = await import(/* @vite-ignore */ modulePath);
  const activate = vi.fn();
  const inspect = vi.fn();
  const escape = vi.fn(() => true);
  const callbacks = keyboard.projectSourceKeyboardCallbacks({ activate, inspect, escape });

  expect(keyboard.projectSourceKeyboardEnabled({
    hasProject: true,
    activeSection: "sources",
    connectOpen: false,
    addSourceOpen: false,
    disconnectOpen: false,
  })).toBe(true);
  expect(keyboard.PROJECT_SOURCE_KEYBOARD_HINT).toBe("↑↓ строка · Enter инспектор");
  callbacks.onKeyboardActivateSource("12");
  callbacks.onKeyboardInspectSource("13");
  expect(callbacks.onKeyboardEscape()).toBe(true);
  expect(activate).toHaveBeenCalledWith("12");
  expect(inspect).toHaveBeenCalledWith("13");
});
