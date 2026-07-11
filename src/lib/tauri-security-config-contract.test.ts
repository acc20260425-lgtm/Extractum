import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const readJson = (relativePath: string) =>
  JSON.parse(readFileSync(path.join(repoRoot, relativePath), "utf8")) as Record<string, unknown>;
const readSource = (relativePath: string) => readFileSync(path.join(repoRoot, relativePath), "utf8");

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

  it("limits MCP and fixture commands to dev builds on localhost", () => {
    const source = readSource("src-tauri/src/lib.rs");
    const fixtureCommands = [
      "seed_prompt_pack_cancellation_smoke_fixture",
      "clear_prompt_pack_cancellation_smoke_fixture",
      "seed_takeout_cancellation_smoke_fixture",
      "clear_takeout_cancellation_smoke_fixture",
      "seed_source_job_cancellation_smoke_fixture",
      "clear_source_job_cancellation_smoke_fixture",
      "seed_analysis_redesign_fixtures",
      "clear_analysis_redesign_fixture_active_runs",
      "clear_analysis_redesign_fixtures",
    ];

    expect(source).toMatch(
      /#\[cfg\(dev\)\]\s*let builder = builder\.plugin\(\s*tauri_plugin_mcp_bridge::Builder::new\(\)\s*\.bind_address\("127\.0\.0\.1"\)\s*\.build\(\),\s*\);/s,
    );
    expect(source).not.toMatch(/#\[cfg\(debug_assertions\)\][\s\S]{0,300}tauri_plugin_mcp_bridge/);

    for (const command of fixtureCommands) {
      expect(
        source.match(new RegExp(`#\\[cfg\\(dev\\)\\][\\s\\S]{0,400}\\b${command}\\b`, "g")),
      ).toHaveLength(2);
      expect(source).not.toMatch(
        new RegExp(`#\\[cfg\\(debug_assertions\\)\\]\\s*${command}`),
      );
    }
  });
});
