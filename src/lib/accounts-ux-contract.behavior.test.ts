import { getByRole } from "@testing-library/dom";
import { createRequire } from "node:module";
import { render } from "svelte/server";
import { describe, expect, it, vi } from "vitest";

vi.mock("$app/navigation", () => ({ goto: vi.fn() }));

import AccountsPage from "../routes/accounts/+page.svelte";

const JSDOM = (createRequire(import.meta.url)("jsdom") as {
  JSDOM: new (html: string) => { window: { document: Document } };
}).JSDOM;

function renderedPage() {
  const document = new JSDOM(render(AccountsPage).body).window.document;
  return document;
}

describe("accounts UX contract", () => {
  it("separates Telegram identity from YouTube access", () => {
    const document = renderedPage();
    const telegram = getByRole(document.body, "heading", { name: "Telegram accounts", level: 2 }).closest("section");
    const youtube = getByRole(document.body, "heading", { name: "YouTube access", level: 2 }).closest("section");
    expect(telegram).not.toBeNull();
    expect(youtube).not.toBeNull();
    expect(telegram).not.toBe(youtube);
    expect(youtube?.textContent).toContain("without mixing them into Telegram account identity");
  });

  it("keeps YouTube auth and sync settings in separate visual groups", () => {
    const document = renderedPage();
    const authentication = getByRole(document.body, "region", { name: "YouTube authentication" });
    const syncPolicy = getByRole(document.body, "region", { name: "YouTube sync policy" });
    expect(authentication).not.toBe(syncPolicy);
    expect(authentication.querySelector("h3")?.textContent).toBe("Authentication");
    expect(syncPolicy.querySelector("h3")?.textContent).toBe("Sync policy");
    expect(authentication.closest(".youtube-settings-panel")).toBe(syncPolicy.closest(".youtube-settings-panel"));
  });

  it("does not render embedded YouTube settings as a nested desk panel", () => {
    const embedded = renderedPage().querySelector(".youtube-settings-panel.embedded");
    expect(embedded).not.toBeNull();
    expect([
      embedded?.classList.contains("desk-panel"),
      embedded?.classList.contains("desk-panel-subtle"),
    ]).toEqual([false, false]);
  });
});
