import { describe, expect, it } from "vitest";
import { describeError, errorKind, formatAppError } from "./app-error";

describe("app-error", () => {
  it("reads structured app error objects", () => {
    const error = { kind: "validation", message: " Invalid input " };

    expect(errorKind(error)).toBe("validation");
    expect(describeError(error)).toBe("Invalid input");
    expect(formatAppError("saving settings", error)).toBe(
      "Error saving settings (validation): Invalid input",
    );
  });

  it("reads serialized app error payloads returned as strings", () => {
    const error = JSON.stringify({ kind: "not_found", message: "Missing run" });

    expect(errorKind(error)).toBe("not_found");
    expect(describeError(error)).toBe("Missing run");
    expect(formatAppError("opening run", error)).toBe("Error opening run (not_found): Missing run");
  });

  it("falls back to plain strings and Error instances", () => {
    expect(errorKind("plain failure")).toBeNull();
    expect(describeError(" plain failure ")).toBe("plain failure");
    expect(formatAppError("loading data", "plain failure")).toBe("Error loading data: plain failure");

    expect(describeError(new Error("boom"))).toBe("boom");
  });

  it("hides internal kind labels while keeping the message", () => {
    const error = { kind: "internal", message: "Database unavailable" };

    expect(errorKind(error)).toBe("internal");
    expect(formatAppError("loading data", error)).toBe("Error loading data: Database unavailable");
  });

  it("returns Unknown error for invalid or empty values", () => {
    expect(describeError("   ")).toBe("Unknown error");
    expect(describeError({ message: "" })).toBe("Unknown error");
    expect(describeError(null)).toBe("Unknown error");
    expect(errorKind({ kind: "unexpected", message: "nope" })).toBeNull();
  });
});
