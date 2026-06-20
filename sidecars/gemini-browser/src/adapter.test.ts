import { describe, expect, it } from "vitest";
import {
  GEMINI_DOM_CONTRACT_VERSION,
  answerCandidates,
  composerCandidates,
  sendCandidates,
} from "./dom-contract";

describe("production Gemini DOM contract", () => {
  it("keeps the selected resilient-scoring contract version explicit", () => {
    expect(GEMINI_DOM_CONTRACT_VERSION).toBe("2026-06-20-resilient-scoring");
  });

  it("has candidates for composer, send, and answer extraction", () => {
    expect(composerCandidates.length).toBeGreaterThan(0);
    expect(sendCandidates.length).toBeGreaterThan(0);
    expect(answerCandidates.length).toBeGreaterThan(0);
    expect(answerCandidates.some((candidate) => candidate.selector === "main section")).toBe(false);
  });
});
