import { describe, expect, it } from "vitest";
import { parseEnvelope } from "./protocol.js";
import { redactText, redactUrl } from "./redaction.js";

describe("gemini browser sidecar protocol", () => {
  it("parses a status command envelope", () => {
    const envelope = parseEnvelope(
      JSON.stringify({
        id: "1",
        command: { type: "status", browser_profile_dir: "G:/Extractum/profile" },
      }),
    );

    expect(envelope.command.type).toBe("status");
  });

  it("rejects envelopes without command type", () => {
    expect(() => parseEnvelope(JSON.stringify({ id: "1", command: {} }))).toThrow(
      "Sidecar command type is required",
    );
  });

  it("redacts sensitive URL params and prompt text", () => {
    expect(redactUrl("https://gemini.google.com/app?authuser=dima&prompt=hello")).toContain(
      "authuser=%5Bredacted%5D",
    );
    expect(redactText("hello answer", "hello")).toBe("[prompt] answer");
  });
});
