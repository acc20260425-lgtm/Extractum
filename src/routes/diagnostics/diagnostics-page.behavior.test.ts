import { getAllByText, getByRole, queryByRole, queryByText } from "@testing-library/dom";
import { createRequire } from "node:module";
import { createRawSnippet } from "svelte";
import { render } from "svelte/server";
import { describe, expect, it, vi } from "vitest";

vi.mock("$app/navigation", () => ({ goto: vi.fn() }));
vi.mock("$app/state", () => ({ page: { url: new URL("http://localhost/diagnostics") } }));

import {
  EMPTY_DIAGNOSTIC_ISSUES_MESSAGE,
  diagnosticsTablesBeforeOverview,
  runDiagnosticsRefresh,
} from "$lib/diagnostics-page";
import {
  filterDiagnosticIssueRows,
  privacyFallbackNote,
} from "$lib/diagnostics-view-model";
import AppLayout from "../+layout.svelte";
import DiagnosticsPage from "./+page.svelte";

const JSDOM = (createRequire(import.meta.url)("jsdom") as {
  JSDOM: new (html: string) => { window: { document: Document } };
}).JSDOM;

function diagnosticsDocument() {
  return new JSDOM(render(DiagnosticsPage).body).window.document;
}

describe("diagnostics page", () => {
  it("omits raw payload and log controls", () => {
    const body = diagnosticsDocument().body;
    expect({
      copyControl: queryByRole(body, "button", { name: /copy/i }),
      rawOrLogControl: queryByText(body, /raw json|raw payload|copy logs/i),
    }).toEqual({ copyControl: null, rawOrLogControl: null });
  });

  it("loads on mount and refreshes on request", async () => {
    const load = vi.fn().mockResolvedValue({ app: "Extractum" });
    const onSuccess = vi.fn();
    const callbacks = { onStart: vi.fn(), onSuccess, onError: vi.fn(), onFinish: vi.fn() };
    await runDiagnosticsRefresh({ initial: true, load, ...callbacks });
    await runDiagnosticsRefresh({ initial: false, load, ...callbacks });
    expect(load).toHaveBeenCalledTimes(2);
    expect(callbacks.onStart.mock.calls).toEqual([[true], [false]]);
    expect(onSuccess).toHaveBeenCalledTimes(2);
  });

  it("preserves the last summary after refresh failure", async () => {
    let current = { version: "1.2.3" };
    await runDiagnosticsRefresh({
      initial: false,
      load: vi.fn().mockRejectedValue(new Error("offline")),
      onStart: vi.fn(),
      onSuccess: (next) => { current = next as typeof current; },
      onError: vi.fn(),
      onFinish: vi.fn(),
    });
    expect(current).toEqual({ version: "1.2.3" });
  });

  it("presents one initial loading status", () => {
    const body = diagnosticsDocument().body;
    expect(getAllByText(body, "Loading diagnostics...")).toHaveLength(1);
    expect(getByRole(body, "button", { name: "Refresh diagnostics" }).hasAttribute("disabled")).toBe(true);
  });

  it("handles partial privacy data", () => {
    expect(privacyFallbackNote(undefined)).toMatch(/sanitized fields only/i);
    expect(privacyFallbackNote({})).toMatch(/did not report excluded data classes/i);
  });

  it("is reachable from diagnostics navigation", () => {
    const children = createRawSnippet(() => ({ render: () => "<p>Diagnostics content</p>" }));
    const body = new JSDOM(render(AppLayout, { props: { children } }).body).window.document.body;
    const diagnostics = getByRole(body, "link", { name: "Diagnostics" });
    const settings = getByRole(body, "link", { name: "Settings" });
    expect(diagnostics.getAttribute("href")).toBe("/diagnostics");
    expect(diagnostics.getAttribute("aria-current")).toBe("page");
    expect(settings.getAttribute("href")).toBe("/settings");
    expect(settings.getAttribute("aria-current")).toBeNull();
    expect(diagnostics).not.toBe(settings);
    expect(diagnostics.textContent).toContain("Local health");
    expect(settings.textContent).toContain("Models and app");
    expect(getByRole(body, "navigation", { name: "Primary navigation" }).contains(diagnostics)).toBe(true);
  });

  it("switches between issue-focused and all-table views", () => {
    expect(diagnosticsTablesBeforeOverview("issues")).toBe(true);
    expect(diagnosticsTablesBeforeOverview("all")).toBe(false);
  });

  it("filters issue rows before presenting the summary", () => {
    expect(filterDiagnosticIssueRows([
      { status: "ready", count: 3 },
      { status: "failed", count: 1 },
    ])).toEqual([{ status: "failed", count: 1 }]);
    expect(diagnosticsTablesBeforeOverview("issues")).toBe(true);
  });

  it("orders issue details and overview by view mode", () => {
    expect(diagnosticsTablesBeforeOverview("issues")).toBe(true);
    expect(diagnosticsTablesBeforeOverview("all")).toBe(false);
  });

  it("explains an empty issue view", () => {
    expect(filterDiagnosticIssueRows([{ status: "ready", count: 3 }])).toEqual([]);
    expect(EMPTY_DIAGNOSTIC_ISSUES_MESSAGE).toBe("No diagnostic issue rows match this view.");
  });
});
