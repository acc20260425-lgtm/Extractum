import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("$app/navigation", () => ({ goto: vi.fn() }));
vi.mock("$app/state", () => ({ page: { url: new URL("http://localhost/analysis") } }));

import AppLayout from "../../routes/+layout.svelte";

const children = createRawSnippet(() => ({ render: () => "<p>Workspace content</p>" }));

function renderShell() {
  return render(AppLayout, { children });
}

beforeEach(() => localStorage.clear());
afterEach(cleanup);

describe("app sidebar", () => {
  it("exposes primary navigation", () => {
    renderShell();

    const navigation = screen.getByRole("navigation", { name: "Primary navigation" });
    expect(navigation).toBeTruthy();
    expect(screen.getByRole("link", { name: "Workspace" }).getAttribute("href")).toBe("/analysis");
    expect(screen.getByRole("link", { name: "Accounts" }).getAttribute("href")).toBe("/accounts");
    expect(screen.getByRole("link", { name: "Settings" }).getAttribute("href")).toBe("/settings");
  });

  it("persists desktop collapse independently from the mobile drawer", async () => {
    localStorage.setItem("extractum.sidebar.collapsed", "true");
    renderShell();

    await fireEvent.click(screen.getByRole("button", { name: "Expand navigation" }));
    expect(localStorage.getItem("extractum.sidebar.collapsed")).toBe("false");

    await fireEvent.click(screen.getByRole("button", { name: "Open navigation" }));
    expect(screen.getByRole("button", { name: "Open navigation" }).getAttribute("aria-expanded")).toBe("true");
    await fireEvent.click(screen.getByRole("button", { name: "Close navigation" }));
    expect(localStorage.getItem("extractum.sidebar.collapsed")).toBe("false");
  });

  it("leaves the theme control in the workspace top bar", () => {
    renderShell();

    const themeControl = screen.getByRole("button", { name: "Switch to dark theme" });
    expect(themeControl.closest(".workspace-topbar")).not.toBeNull();
    expect(screen.getByRole("complementary", { name: "App sidebar" }).contains(themeControl)).toBe(false);
  });

  it("controls and closes the mobile navigation drawer accessibly", async () => {
    renderShell();
    const trigger = screen.getByRole("button", { name: "Open navigation" });

    await fireEvent.click(trigger);
    const sidebar = screen.getByRole("complementary", { name: "App sidebar" });
    expect(sidebar.getAttribute("tabindex")).toBe("-1");
    expect(document.activeElement).toBe(sidebar);

    await fireEvent.keyDown(window, { key: "Escape" });
    expect(sidebar.getAttribute("tabindex")).toBeNull();
    await waitFor(() => expect(document.activeElement).toBe(trigger));

    await fireEvent.click(trigger);
    await fireEvent.click(screen.getByRole("link", { name: "Accounts" }));
    expect(sidebar.getAttribute("tabindex")).toBeNull();
    await waitFor(() => expect(document.activeElement).toBe(trigger));
  });

  it("keeps collapsed navigation accessible", () => {
    localStorage.setItem("extractum.sidebar.collapsed", "true");
    renderShell();

    const workspace = screen.getByRole("link", { name: "Workspace" });
    expect(workspace.getAttribute("aria-current")).toBe("page");
    expect(workspace.getAttribute("title")).toBe("Workspace");
    expect(screen.getByRole("button", { name: "Expand navigation" })).toBeTruthy();
  });

  it("exposes mobile menu expansion semantics", async () => {
    renderShell();
    const trigger = screen.getByRole("button", { name: "Open navigation" });

    expect(trigger.getAttribute("aria-controls")).toBe("app-sidebar");
    expect(trigger.getAttribute("aria-expanded")).toBe("false");
    await fireEvent.click(trigger);
    expect(trigger.getAttribute("aria-expanded")).toBe("true");
  });
});
