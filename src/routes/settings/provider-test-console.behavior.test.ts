import { getAllByRole, getByRole, getByText, queryByRole, queryByText, within } from "@testing-library/dom";
import { createRequire } from "node:module";
import { render } from "svelte/server";
import type { ComponentProps } from "svelte";
import { expect, it, vi } from "vitest";

import ProviderTestConsole from "$lib/components/settings/ProviderTestConsole.svelte";
import { providerTestConsoleActions } from "$lib/provider-test-console";

const JSDOM = (createRequire(import.meta.url)("jsdom") as {
  JSDOM: new (html: string) => { window: { document: Document } };
}).JSDOM;

type ProviderTestConsoleProps = ComponentProps<typeof ProviderTestConsole>;

function providerConsoleDocument(overrides: Partial<ProviderTestConsoleProps> = {}) {
  const props: ProviderTestConsoleProps = {
    open: true,
    prompt: "Explain local-first research",
    providerLabel: "OpenAI-compatible",
    providerModelLine: "OpenAI-compatible / gpt-5-mini",
    testing: false,
    canRun: true,
    status: "Provider test failed: endpoint unavailable",
    output: "",
    usage: "",
    onOpen: vi.fn(),
    onClose: vi.fn(),
    onPromptChange: vi.fn(),
    onRun: vi.fn(),
    onCancel: vi.fn(),
    ...overrides,
  };
  return new JSDOM(render(ProviderTestConsole, { props }).body).window.document;
}

it("opens the provider console from the LLM profile actions", () => {
  const document = providerConsoleDocument();
  const trigger = getByRole(document.body, "button", { name: "Open provider test console" });
  expect(trigger.textContent).toContain("Open test");
  expect(trigger.getAttribute("type")).toBe("button");
  const setOpen = vi.fn();
  providerTestConsoleActions(setOpen).open();
  expect(setOpen).toHaveBeenLastCalledWith(true);
  providerTestConsoleActions(setOpen).close();
  expect(setOpen).toHaveBeenLastCalledWith(false);
  expect(setOpen).toHaveBeenCalledTimes(2);
  expect(getByRole(document.body, "dialog", { name: "Provider Test Console" })).toBeTruthy();
});

it("does not keep a duplicate smoke test panel outside the console dialog", () => {
  const document = providerConsoleDocument();
  expect(queryByRole(document.body, "heading", { name: "Smoke test" })).toBeNull();
  expect(queryByText(document.body, "Provider test")).toBeNull();
  expect(queryByText(document.body, "Latest response")).toBeNull();
  expect(document.querySelector(".summary-strip")).toBeNull();
  expect(getAllByRole(document.body, "dialog")).toHaveLength(1);
});

it("shows provider test status inside the console dialog", () => {
  const document = providerConsoleDocument();
  const dialog = getByRole(document.body, "dialog", { name: "Provider Test Console" });
  const status = within(dialog).getByRole("alert");
  expect(status).toBeTruthy();
  expect(getByText(status, "Provider test failed: endpoint unavailable")).toBeTruthy();
  expect(dialog.contains(status)).toBe(true);
  expect(document.body.contains(status)).toBe(true);
  expect(getByText(dialog, "Streaming output")).toBeTruthy();
});
