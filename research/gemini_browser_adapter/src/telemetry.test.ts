import { describe, expect, it } from "vitest";
import { redactUrl } from "./telemetry";

describe("telemetry redaction", () => {
  it("redacts token-like query parameters", () => {
    const url = redactUrl("https://gemini.google.com/app?token=secret&authuser=0&safe=yes");
    expect(url).toContain("token=<redacted>");
    expect(url).toContain("authuser=<redacted>");
    expect(url).toContain("safe=yes");
  });
});
