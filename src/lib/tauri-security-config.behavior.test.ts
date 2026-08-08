import { expect, it } from "vitest";

import baseConfig from "../../src-tauri/tauri.conf.json";
import mcpConfig from "../../src-tauri/tauri.mcp.conf.json";
import { buildTauriArgs } from "../../scripts/tauri.mjs";

it("MCP capabilities are development only", () => {
  expect(baseConfig.app.withGlobalTauri).toBe(false);
  expect((baseConfig.build as { features?: string[] }).features ?? [])
    .not.toContain("prompt-pack-dev-fixtures");
  expect(mcpConfig).toEqual({
    build: { features: ["prompt-pack-dev-fixtures"] },
    app: { withGlobalTauri: true },
  });
  expect(buildTauriArgs(["dev"])).toEqual([
    "dev", "--config", "src-tauri/tauri.mcp.conf.json",
  ]);
  expect(buildTauriArgs(["build"])).toEqual(["build"]);
});
