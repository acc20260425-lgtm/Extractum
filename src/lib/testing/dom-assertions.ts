import { expect } from "vitest";

expect.extend({
  toBeEnabled(received: HTMLButtonElement) {
    return { pass: !received.disabled, message: () => "expected element to be enabled" };
  },
  toBeDisabled(received: HTMLButtonElement) {
    return { pass: received.disabled, message: () => "expected element to be disabled" };
  },
  toBeChecked(received: HTMLElement) {
    const pass = received instanceof HTMLInputElement
      ? received.checked
      : received.getAttribute("aria-checked") === "true";
    return { pass, message: () => "expected checkbox to be checked" };
  },
  toHaveValue(received: HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement, expected: unknown) {
    return {
      pass: this.equals(received.value, expected),
      message: () => `expected element value ${JSON.stringify(received.value)} to equal ${String(expected)}`,
    };
  },
});

declare module "vitest" {
  interface Assertion<T = any> {
    toBeEnabled(): T;
    toBeDisabled(): T;
    toBeChecked(): T;
    toHaveValue(expected: unknown): T;
  }
  interface AsymmetricMatchersContaining {
    toBeEnabled(): unknown;
    toBeDisabled(): unknown;
    toBeChecked(): unknown;
    toHaveValue(expected: unknown): unknown;
  }
}
