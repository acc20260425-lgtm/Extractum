import { expect, it } from "vitest";

import mcpConfig from "../../src-tauri/tauri.mcp.conf.json";

it("MCP capabilities are development only", () => {
  expect(mcpConfig).toEqual({
    build: { features: ["prompt-pack-dev-fixtures"] },
    app: { withGlobalTauri: true },
  });
});
