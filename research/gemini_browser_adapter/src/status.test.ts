import { describe, expect, it } from "vitest";
import { isManualActionStatus, isSuccessStatus, isTerminalStatus } from "./types";

describe("Gemini adapter status helpers", () => {
  it("detects terminal statuses", () => {
    expect(isTerminalStatus("ok")).toBe(true);
    expect(isTerminalStatus("generation_timeout")).toBe(true);
    expect(isTerminalStatus("running")).toBe(false);
  });

  it("detects manual-action statuses", () => {
    expect(isManualActionStatus("login_required")).toBe(true);
    expect(isManualActionStatus("manual_action_required")).toBe(true);
    expect(isManualActionStatus("rate_limited")).toBe(false);
  });

  it("detects success statuses", () => {
    expect(isSuccessStatus("ok")).toBe(true);
    expect(isSuccessStatus("ready")).toBe(true);
    expect(isSuccessStatus("response_parse_failed")).toBe(false);
  });
});
