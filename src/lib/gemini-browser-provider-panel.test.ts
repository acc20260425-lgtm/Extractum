import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { statusLabel } from "./gemini-browser-provider-panel-contract";

const componentSource = readFileSync(
  path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "components/settings/gemini-browser-provider-panel.svelte",
  ),
  "utf8",
);

describe("gemini browser provider panel copy contract", () => {
  it("maps provider statuses to compact operator labels", () => {
    expect(statusLabel("ready", null)).toBe("Ready");
    expect(statusLabel("needs_login", "login")).toBe("Login required");
    expect(statusLabel("needs_manual_action", "account_picker")).toBe("Choose account");
    expect(statusLabel("running", null)).toBe("Running");
    expect(statusLabel("failed", null)).toBe("Failed");
    expect(statusLabel("not_started", null)).toBe("Not started");
  });

  it("treats Resume as an open-or-reattach command that returns provider status", () => {
    expect(componentSource).toContain("status = await geminiBridgeResume();");
  });
});
