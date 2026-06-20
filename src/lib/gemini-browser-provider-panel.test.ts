import { describe, expect, it } from "vitest";
import { statusLabel } from "./gemini-browser-provider-panel-contract";

describe("gemini browser provider panel copy contract", () => {
  it("maps provider statuses to compact operator labels", () => {
    expect(statusLabel("ready", null)).toBe("Ready");
    expect(statusLabel("needs_login", "login")).toBe("Login required");
    expect(statusLabel("needs_manual_action", "account_picker")).toBe("Choose account");
    expect(statusLabel("running", null)).toBe("Running");
    expect(statusLabel("failed", null)).toBe("Failed");
    expect(statusLabel("not_started", null)).toBe("Not started");
  });
});
