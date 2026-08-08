import { expect, it, vi } from "vitest";

it("passes keyboard navigation callbacks to the source grid", async () => {
  const modulePath = "./page-keyboard";
  const keyboard = await import(/* @vite-ignore */ modulePath);
  const activate = vi.fn();
  const inspect = vi.fn();
  const escape = vi.fn(() => true);
  const callbacks = keyboard.projectSourceKeyboardContract({
    hasProject: true,
    activeSection: "sources",
    dialogsClosed: true,
    keyboardHint: "↑↓ строка · Enter инспектор",
    activate,
    inspect,
    escape,
  });

  expect(callbacks.enabled).toBe(true);
  expect(callbacks.keyboardHint).toBe("↑↓ строка · Enter инспектор");
  callbacks.onKeyboardActivateSource("12");
  callbacks.onKeyboardInspectSource("13");
  expect(callbacks.onKeyboardEscape()).toBe(true);
  expect(activate).toHaveBeenCalledWith("12");
  expect(inspect).toHaveBeenCalledWith("13");
});
