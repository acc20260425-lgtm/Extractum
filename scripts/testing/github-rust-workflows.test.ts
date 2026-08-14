import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const workflow = (name: string) => readFileSync(
  path.resolve(".github", "workflows", name),
  "utf8",
);

describe("Rust GitHub workflow command contracts", () => {
  it("uses the local fast Rust gate", () => {
    expect(workflow("rust-fast.yml")).toContain("npm.cmd run check:rust:fast");
  });

  it("uses the local fast and full gates", () => {
    const full = workflow("rust-full.yml");

    expect(full).toContain("npm.cmd run check:rust:fast");
    expect(full).toContain("npm.cmd run verify");
  });

  it("uses advisory and full gates before bundling", () => {
    const release = workflow("rust-release.yml");

    expect(release).toContain("npm.cmd run check:rust:advisories");
    expect(release).toContain("npm.cmd run check:rust:fast");
    expect(release).toContain("npm.cmd run verify");
  });
});
