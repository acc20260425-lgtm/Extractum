import { describe, expect, it } from "vitest";
import { scoreEditableCandidate, scoreButtonCandidate } from "./scoring";

describe("locator scoring", () => {
  it("scores visible lower-page editable prompt candidates highly", () => {
    const score = scoreEditableCandidate({
      aria: "ask gemini textbox",
      topRatio: 0.8,
      width: 480,
      height: 48,
      visible: true,
      editable: true,
    });
    expect(score).toBeGreaterThanOrEqual(8);
  });

  it("rejects hidden editable candidates", () => {
    const score = scoreEditableCandidate({
      aria: "ask gemini",
      topRatio: 0.8,
      width: 480,
      height: 48,
      visible: false,
      editable: true,
    });
    expect(score).toBe(0);
  });

  it("scores send-like buttons by label and position", () => {
    const score = scoreButtonCandidate({
      label: "send message",
      topRatio: 0.75,
      rightRatio: 0.85,
      width: 44,
      height: 36,
      visible: true,
      enabled: true,
    });
    expect(score).toBeGreaterThanOrEqual(8);
  });
});
