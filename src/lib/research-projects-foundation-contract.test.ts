// @ts-nocheck
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import packageJson from "../../package.json";
import layoutSource from "../routes/+layout.svelte?raw";
import viteConfigSource from "../../vite.config.js?raw";

const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));

function read(relativePath: string) {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

describe("new UI foundation", () => {
  it("installs Tailwind, shadcn-svelte support, and SVAR Grid dependencies", () => {
    expect(packageJson.dependencies["@svar-ui/svelte-grid"]).toBeDefined();
    expect(packageJson.dependencies["@svar-ui/svelte-core"]).toBeDefined();
    expect(packageJson.dependencies["@svar-ui/grid-locales"]).toBeDefined();
    expect(packageJson.dependencies["@svar-ui/core-locales"]).toBeDefined();
    expect(packageJson.dependencies["tailwind-variants"]).toBeDefined();
    expect(packageJson.dependencies["clsx"]).toBeDefined();
    expect(packageJson.dependencies["tailwind-merge"]).toBeDefined();
    expect(packageJson.devDependencies.tailwindcss).toBeDefined();
    expect(packageJson.devDependencies["@tailwindcss/vite"]).toBeDefined();
    expect(packageJson.devDependencies["tw-animate-css"]).toBeDefined();
  });

  it("wires Tailwind through Vite without changing the Tauri server settings", () => {
    expect(viteConfigSource).toContain('import tailwindcss from "@tailwindcss/vite";');
    expect(viteConfigSource).toContain("plugins: [tailwindcss(), sveltekit()]");
    expect(viteConfigSource).toContain("port: 1420");
    expect(viteConfigSource).toContain("strictPort: true");
  });

  it("keeps shadcn generated primitives in lower-case ui folders beside legacy PascalCase ui files", () => {
    expect(existsSync(path.join(repoRoot, "src/lib/components/ui/Button.svelte"))).toBe(true);
    expect(existsSync(path.join(repoRoot, "src/lib/components/ui/button/index.ts"))).toBe(true);
    expect(existsSync(path.join(repoRoot, "src/lib/components/ui/sheet/index.ts"))).toBe(true);
    expect(existsSync(path.join(repoRoot, "src/lib/components/ui/tabs/index.ts"))).toBe(true);
    expect(read("components.json")).toContain('"ui": "$lib/components/ui"');
  });

  it("moves shared tokens into the product base stylesheet", () => {
    expect(layoutSource).toContain('import "$lib/styles/base.css";');
    const baseCss = read("src/lib/styles/base.css");
    expect(baseCss).toContain('@import "tailwindcss";');
    expect(baseCss).toContain('@import "tw-animate-css";');
    expect(baseCss).toContain("--extractum-density-row-height: 34px");
    expect(baseCss).toContain("--wx-table-header-background");
    expect(baseCss).toContain("[data-theme=\"dark\"]");
  });
});
