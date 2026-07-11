import { describe, expect, it } from "vitest";

import { buildTauriArgs } from "./tauri.mjs";

describe("Tauri CLI wrapper", () => {
  it("adds the development MCP overlay to dev", () => {
    expect(buildTauriArgs(["dev"])).toEqual(["dev", "--config", "src-tauri/tauri.mcp.conf.json"]);
  });

  it("preserves an explicit long config option", () => {
    expect(buildTauriArgs(["dev", "--config", "custom.json"])).toEqual(["dev", "--config", "custom.json"]);
    expect(buildTauriArgs(["dev", "--config=x.json"])).toEqual(["dev", "--config=x.json"]);
  });

  it("preserves an explicit short config option", () => {
    expect(buildTauriArgs(["dev", "-c", "custom.json"])).toEqual(["dev", "-c", "custom.json"]);
    expect(buildTauriArgs(["dev", "-c=x.json"])).toEqual(["dev", "-c=x.json"]);
  });

  it("does not inspect flags after the argument delimiter", () => {
    expect(buildTauriArgs(["dev", "--", "--config=x.json"])).toEqual([
      "dev",
      "--config",
      "src-tauri/tauri.mcp.conf.json",
      "--",
      "--config=x.json",
    ]);
  });

  it("does not add the MCP overlay to other commands", () => {
    expect(buildTauriArgs(["build", "--debug"])).toEqual(["build", "--debug"]);
  });
});
