import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const readJson = (relativePath: string) =>
  JSON.parse(readFileSync(path.join(repoRoot, relativePath), "utf8")) as Record<string, unknown>;

describe("Tauri security configuration", () => {
  it("keeps the base configuration production-safe", () => {
    const config = readJson("src-tauri/tauri.conf.json");
    const app = config.app as Record<string, unknown>;
    const security = app.security as Record<string, unknown>;
    const csp = security.csp as string;

    expect(app.withGlobalTauri).toBe(false);
    expect(csp).toContain("default-src 'self'");
    expect(csp).toContain("connect-src 'self' ipc: http://ipc.localhost");
    expect(csp).toContain("img-src 'self' asset: http://asset.localhost data: blob:");
    expect(csp).toContain("style-src 'self' 'unsafe-inline'");
    expect(csp).toContain("font-src 'self' data:");
    expect(csp).toContain("script-src 'self'");
    expect(security.dangerousDisableAssetCspModification).toBe(false);
  });

  it("limits the MCP overlay to exposing the global Tauri object", () => {
    expect(readJson("src-tauri/tauri.mcp.conf.json")).toEqual({
      app: { withGlobalTauri: true },
    });
  });

  it("does not grant frontend SQL permissions", () => {
    const capability = readJson("src-tauri/capabilities/default.json");
    const permissions = capability.permissions as string[];

    expect(permissions.filter((permission) => permission.startsWith("sql:"))).toEqual([]);
  });
});
