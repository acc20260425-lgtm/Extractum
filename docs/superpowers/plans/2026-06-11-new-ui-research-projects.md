# New UI Research Projects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first new Extractum UI slice at `/projects`: an Ultra HD-oriented Research Projects workspace with a dense Sources tab and Connect from Library workflow.

**Architecture:** Keep the current `/analysis` UI intact and introduce the new UI as a separate vertical slice. New feature screens use product-owned `extractum-ui` wrappers; shadcn-svelte provides interaction primitives, SVAR provides dense data grids, and a transition adapter maps the new product language to existing analysis sources/source groups.

**Tech Stack:** Svelte 5, SvelteKit SPA/Tauri, TypeScript, Vitest, Tailwind CSS v4, shadcn-svelte, `@svar-ui/svelte-grid`, `@lucide/svelte`.

---

## Approved Design Inputs

- Design spec: `docs/superpowers/specs/2026-06-11-new-ui-research-projects-design.md`
- shadcn-svelte install and CLI docs: `https://www.shadcn-svelte.com/docs/installation/sveltekit`, `https://www.shadcn-svelte.com/docs/cli`
- shadcn-svelte Tailwind v4/Svelte 5 note: `https://www.shadcn-svelte.com/docs/migration/tailwind-v4`
- SVAR Svelte Grid local skill: `C:/Users/Dima/.codex/skills/svar-svelte/grid/index.md`
- SVAR MCP confirmation: Grid wrapper should provide stable row ids, `multiselect`, fixed-height host, `data-action="ignore-click"` checkbox cells, Willow theme, and Locale context.

## File Structure

### Foundation And Generated Primitives

- Modify: `package.json`
  - Add Tailwind/shadcn/SVAR dependencies and scripts stay unchanged.
- Modify: `package-lock.json`
  - Lock installed dependencies.
- Modify: `vite.config.js`
  - Add Tailwind Vite plugin before `sveltekit()`.
- Create: `components.json`
  - Configure shadcn-svelte aliases with `ui` at `$lib/components/ui`.
- Create: `src/lib/styles/base.css`
  - Own Extractum product tokens, Tailwind import, shadcn CSS variables, SVAR variable bridge, and compact density defaults.
- Modify: `src/routes/+layout.svelte`
  - Import `src/lib/styles/base.css`, keep old layout behavior and old UI global compatibility classes.
- Create or modify: `src/lib/utils.ts`
  - Provide shadcn `cn` helper.
- Create: `src/lib/components/ui/<lowercase>/*`
  - shadcn-svelte generated primitives. Keep existing PascalCase files in `src/lib/components/ui/*` unchanged for old UI.

### Product Wrapper Layer

- Create: `src/lib/components/extractum-ui/Button.svelte`
- Create: `src/lib/components/extractum-ui/TextInput.svelte`
- Create: `src/lib/components/extractum-ui/Select.svelte`
- Create: `src/lib/components/extractum-ui/Checkbox.svelte`
- Create: `src/lib/components/extractum-ui/Badge.svelte`
- Create: `src/lib/components/extractum-ui/Tabs.svelte`
- Create: `src/lib/components/extractum-ui/Sheet.svelte`
- Create: `src/lib/components/extractum-ui/ProviderBadge.svelte`
- Create: `src/lib/components/extractum-ui/StatusBadge.svelte`
- Create: `src/lib/components/extractum-ui/DataGrid.svelte`
- Create: `src/lib/components/extractum-ui/GridSelectCell.svelte`
- Create: `src/lib/components/extractum-ui/index.ts`

These components are the only product-facing entrypoints for new UI feature screens. `DataGrid.svelte` is the only component in this layer that imports `@svar-ui/svelte-grid`.

### New UI View Model And Workflow

- Create: `src/lib/ui/research-projects-model.ts`
  - Pure types and adapters: projects, library sources, project links, filters, source-job status projection, connectability, source-group update command.
- Create: `src/lib/ui/research-projects-workflow.ts`
  - Async loading of source groups, analysis sources, active runs, source jobs, and connect persistence through existing APIs.
- Test: `src/lib/ui/research-projects-model.test.ts`
- Test: `src/lib/ui/research-projects-workflow.test.ts`

### Feature Components

- Create: `src/lib/components/research-projects/ProjectsShell.svelte`
- Create: `src/lib/components/research-projects/IconRail.svelte`
- Create: `src/lib/components/research-projects/ProjectRail.svelte`
- Create: `src/lib/components/research-projects/TopCommandBar.svelte`
- Create: `src/lib/components/research-projects/ProjectWorkspace.svelte`
- Create: `src/lib/components/research-projects/SourcesTab.svelte`
- Create: `src/lib/components/research-projects/ConnectFromLibrary.svelte`
- Create: `src/lib/components/research-projects/LibrarySourceCell.svelte`
- Create: `src/lib/components/research-projects/ProjectSourceSummary.svelte`
- Create: `src/lib/components/research-projects/BottomQueue.svelte`

Feature components import from `$lib/components/extractum-ui`, `$lib/ui/research-projects-*`, and `@lucide/svelte`. They do not import shadcn primitives, `bits-ui`, or `@svar-ui/*` directly.

### Route And Contracts

- Create: `src/routes/projects/+page.svelte`
- Test: `src/lib/research-projects-import-boundary.test.ts`
- Test: `src/lib/research-projects-route-contract.test.ts`
- Test: `src/lib/research-projects-foundation-contract.test.ts`

---

## Task 0: Execution Branch And Baseline

**Files:**
- No source files changed.

- [x] **Step 1: Create the implementation branch**

Run:

```powershell
git switch -c feature/new-ui-research-projects
```

Expected: branch changes to `feature/new-ui-research-projects`.

- [x] **Step 2: Confirm the worktree starts clean**

Run:

```powershell
git status --short
```

Expected: no output.

- [x] **Step 3: Run the existing fast checks**

Run:

```powershell
npm.cmd run test
npm.cmd run check
```

Expected: both commands pass before any implementation changes. If they fail, stop and capture the existing failure before changing new UI files.

---

## Task 1: Tailwind, shadcn-svelte, And SVAR Foundation

**Files:**
- Create: `src/lib/research-projects-foundation-contract.test.ts`
- Create: `src/lib/styles/base.css`
- Create: `components.json`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `vite.config.js`
- Modify: `src/routes/+layout.svelte`
- Create or modify: `src/lib/utils.ts`
- Create: `src/lib/components/ui/button/*`
- Create: `src/lib/components/ui/badge/*`
- Create: `src/lib/components/ui/input/*`
- Create: `src/lib/components/ui/checkbox/*`
- Create: `src/lib/components/ui/tabs/*`
- Create: `src/lib/components/ui/sheet/*`
- Create: `src/lib/components/ui/dialog/*`
- Create: `src/lib/components/ui/dropdown-menu/*`
- Create: `src/lib/components/ui/tooltip/*`
- Create: `src/lib/components/ui/select/*`
- Create: `src/lib/components/ui/separator/*`
- Create: `src/lib/components/ui/sonner/*`
- Create: `src/lib/components/ui/command/*`
- Create: `src/lib/components/ui/label/*`
- Create: `src/lib/components/ui/scroll-area/*`

- [x] **Step 1: Write the failing foundation contract**

Create `src/lib/research-projects-foundation-contract.test.ts`:

```ts
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
```

- [x] **Step 2: Run the foundation contract and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-foundation-contract.test.ts
```

Expected: FAIL because Tailwind, shadcn generated folders, SVAR packages, `components.json`, and `base.css` are not present yet.

- [x] **Step 3: Install Tailwind v4 and SVAR packages**

Run:

```powershell
npm.cmd install @svar-ui/svelte-grid @svar-ui/svelte-core @svar-ui/grid-locales @svar-ui/core-locales clsx tailwind-merge tailwind-variants
npm.cmd install -D tailwindcss @tailwindcss/vite tw-animate-css
```

Expected: `package.json` and `package-lock.json` update. If network access is blocked, rerun with escalated permissions.

- [x] **Step 4: Initialize shadcn-svelte with Extractum paths**

Run:

```powershell
npx.cmd shadcn-svelte@latest init --preset bcivVKXQ --base-color zinc --css src/lib/styles/base.css --lib-alias '$lib' --components-alias '$lib/components' --utils-alias '$lib/utils' --hooks-alias '$lib/hooks' --ui-alias '$lib/components/ui' --no-deps --overwrite
```

Expected:
- `components.json` exists.
- `src/lib/utils.ts` exists and exports `cn`.
- `src/lib/styles/base.css` exists.
- Existing `src/lib/components/ui/Button.svelte` remains present.

- [x] **Step 5: Add the shadcn primitives for the first slice**

Run:

```powershell
npx.cmd shadcn-svelte@latest add button badge input checkbox tabs sheet dialog dropdown-menu tooltip select separator sonner command label scroll-area -y
```

Expected: lower-case directories appear under `src/lib/components/ui/*`. Existing PascalCase legacy components remain unchanged.

- [x] **Step 6: Wire Tailwind into Vite**

Modify `vite.config.js` to this structure while preserving the current Tauri server config:

```js
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [tailwindcss(), sveltekit()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

- [x] **Step 7: Import the base stylesheet from the root layout**

Add this import at the top of `src/routes/+layout.svelte`:

```svelte
<script lang="ts">
  import "$lib/styles/base.css";
```

Keep the existing layout markup, theme state, `ToastHost`, `ModalHost`, `AppSidebar`, and old workspace CSS.

- [x] **Step 8: Add Extractum product tokens and SVAR bridge**

Replace the generated `src/lib/styles/base.css` contents with product-owned tokens that keep shadcn variables available:

```css
@import "tailwindcss";
@import "tw-animate-css";

@custom-variant dark (&:is([data-theme="dark"] *));

:root {
  color-scheme: light;
  --background: hsl(210 24% 98%);
  --foreground: hsl(213 32% 12%);
  --muted: hsl(214 22% 94%);
  --muted-foreground: hsl(215 12% 43%);
  --popover: hsl(0 0% 100%);
  --popover-foreground: hsl(213 32% 12%);
  --card: hsl(0 0% 100%);
  --card-foreground: hsl(213 32% 12%);
  --border: hsl(214 20% 86%);
  --input: hsl(214 20% 86%);
  --primary: hsl(215 82% 45%);
  --primary-foreground: hsl(0 0% 100%);
  --secondary: hsl(214 22% 94%);
  --secondary-foreground: hsl(213 32% 12%);
  --accent: hsl(205 80% 94%);
  --accent-foreground: hsl(215 82% 32%);
  --destructive: hsl(0 72% 50%);
  --destructive-foreground: hsl(0 0% 100%);
  --ring: hsl(215 82% 45%);
  --radius: 6px;

  --extractum-bg: #eef1f5;
  --extractum-surface: #fbfcfd;
  --extractum-surface-raised: #ffffff;
  --extractum-surface-subtle: #f3f6f9;
  --extractum-border: #d7dde5;
  --extractum-border-strong: #c6d0dc;
  --extractum-text: #17212b;
  --extractum-muted: #6e7c8a;
  --extractum-primary: #0f66d8;
  --extractum-primary-hover: #0b56b8;
  --extractum-success: #1aa36f;
  --extractum-warning: #d48b18;
  --extractum-danger: #d94d4d;
  --extractum-radius: 6px;
  --extractum-density-row-height: 34px;
  --extractum-density-control-height: 32px;
  --extractum-font: "Segoe UI", "Inter", "IBM Plex Sans", Arial, sans-serif;
}

[data-theme="dark"] {
  color-scheme: dark;
  --background: hsl(213 26% 8%);
  --foreground: hsl(210 24% 94%);
  --muted: hsl(213 22% 15%);
  --muted-foreground: hsl(214 12% 66%);
  --popover: hsl(213 26% 10%);
  --popover-foreground: hsl(210 24% 94%);
  --card: hsl(213 26% 10%);
  --card-foreground: hsl(210 24% 94%);
  --border: hsl(213 18% 24%);
  --input: hsl(213 18% 24%);
  --primary: hsl(213 88% 64%);
  --primary-foreground: hsl(213 26% 8%);
  --secondary: hsl(213 22% 15%);
  --secondary-foreground: hsl(210 24% 94%);
  --accent: hsl(213 28% 18%);
  --accent-foreground: hsl(213 88% 74%);
  --destructive: hsl(0 72% 62%);
  --destructive-foreground: hsl(213 26% 8%);
  --ring: hsl(213 88% 64%);

  --extractum-bg: #0f1419;
  --extractum-surface: #182028;
  --extractum-surface-raised: #1d2730;
  --extractum-surface-subtle: #111820;
  --extractum-border: #2d3a46;
  --extractum-border-strong: #42515f;
  --extractum-text: #edf2f7;
  --extractum-muted: #90a1b2;
  --extractum-primary: #61a3ff;
  --extractum-primary-hover: #3e88f2;
}

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-destructive-foreground: var(--destructive-foreground);
  --color-ring: var(--ring);
  --radius-sm: 4px;
  --radius-md: var(--radius);
  --radius-lg: 8px;
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

html,
body {
  min-height: 100%;
}

body {
  margin: 0;
  font-family: var(--extractum-font);
  background: var(--extractum-bg);
  color: var(--extractum-text);
}

.extractum-svar-theme {
  --wx-color-primary: var(--extractum-primary);
  --wx-color-primary-selected: color-mix(in srgb, var(--extractum-primary) 14%, transparent);
  --wx-color-primary-font: var(--primary-foreground);
  --wx-color-font: var(--extractum-text);
  --wx-color-font-alt: var(--extractum-muted);
  --wx-background: var(--extractum-surface);
  --wx-background-alt: var(--extractum-surface-subtle);
  --wx-background-hover: color-mix(in srgb, var(--extractum-primary) 8%, var(--extractum-surface));
  --wx-border: 1px solid var(--extractum-border);
  --wx-border-radius: var(--extractum-radius);
  --wx-font-family: var(--extractum-font);
  --wx-font-size: 13px;
  --wx-line-height: 18px;
  --wx-padding: 6px;
  --wx-table-header-background: var(--extractum-surface-subtle);
  --wx-table-border: 1px solid var(--extractum-border);
  --wx-table-header-cell-border: 1px solid var(--extractum-border);
  --wx-table-cell-border: 1px solid var(--extractum-border);
  --wx-table-select-background: color-mix(in srgb, var(--extractum-primary) 13%, transparent);
  --wx-table-select-color: var(--extractum-text);
  --wx-header-font-weight: 600;
}
```

- [x] **Step 9: Run the foundation contract**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-foundation-contract.test.ts
```

Expected: PASS.

- [x] **Step 10: Run type checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [x] **Step 11: Commit foundation**

Run:

```powershell
git add package.json package-lock.json vite.config.js components.json src/lib/utils.ts src/lib/styles/base.css src/routes/+layout.svelte src/lib/components/ui src/lib/research-projects-foundation-contract.test.ts
git commit -m "feat: add new ui component foundation"
```

Expected: commit succeeds.

---

## Task 2: Research Projects Transition Adapter

**Files:**
- Create: `src/lib/ui/research-projects-model.test.ts`
- Create: `src/lib/ui/research-projects-model.ts`

- [x] **Step 1: Write adapter tests**

Create `src/lib/ui/research-projects-model.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  buildSourceGroupUpdateInput,
  connectableSelection,
  filterLibrarySources,
  type LibrarySourceView,
} from "./research-projects-model";
import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
import type { SourceJobRecord } from "$lib/types/sources";

function source(overrides: Partial<AnalysisSourceOption> = {}): AnalysisSourceOption {
  return {
    id: 1,
    account_id: 10,
    source_type: "telegram",
    title: "Radar BPLA",
    item_count: 128,
    last_synced_at: 1_717_000_000,
    ...overrides,
  };
}

function job(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 3,
    related_source_id: null,
    job_type: "youtube_video_full_sync",
    status: "running",
    message: "Syncing",
    progress_current: 1,
    progress_total: 3,
    started_at: 1_717_000_100,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 100,
    name: "Рынок БПЛА",
    source_type: "telegram",
    members: [{ source_id: 1, source_title: "Radar BPLA", item_count: 128 }],
    created_at: 1_716_000_000,
    updated_at: 1_717_000_000,
    ...overrides,
  };
}

describe("research projects model", () => {
  it("projects source groups as research projects without leaking source-group wording", () => {
    const projects = buildResearchProjectsView([group()], []);

    expect(projects).toEqual([
      expect.objectContaining({
        id: "source-group:100",
        title: "Рынок БПЛА",
        sourceCount: 1,
        materialCount: 128,
        backing: { kind: "source_group", groupId: 100, sourceType: "telegram" },
        status: "ready",
      }),
    ]);
  });

  it("marks already connected and unsupported library rows as non-connectable", () => {
    const [telegram, rss] = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "rss", title: "Новости БПЛА" }),
      ],
      [group()],
      "source-group:100",
    );

    expect(telegram.alreadyConnected).toBe(true);
    expect(telegram.connectable).toBe(false);
    expect(telegram.disabledReason).toBe("Источник уже подключен к этому проекту.");
    expect(rss.provider).toBe("rss");
    expect(rss.connectable).toBe(false);
    expect(rss.disabledReason).toBe("Подключение RSS к проектам будет доступно после миграции библиотеки.");
  });

  it("marks active or failed source jobs on library rows before generic provider decisions", () => {
    const [syncing, failed] = buildLibrarySourcesView(
      [
        source({ id: 3, source_type: "youtube", title: "Alpha Drones" }),
        source({ id: 4, source_type: "youtube", title: "Broken Channel" }),
      ],
      [group({ source_type: "youtube", members: [] })],
      "source-group:100",
      [
        job({ source_id: 3, status: "running" }),
        job({ source_id: 4, status: "failed", error: "API quota exceeded" }),
      ],
    );

    expect(syncing.status).toBe("syncing");
    expect(syncing.connectable).toBe(false);
    expect(syncing.disabledReason).toBe("Источник сейчас синхронизируется.");
    expect(failed.status).toBe("error");
    expect(failed.disabledReason).toBe("Последняя синхронизация завершилась ошибкой: API quota exceeded");
  });

  it("filters the library by search text and provider chips", () => {
    const rows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
      ],
      [],
      null,
    );

    expect(filterLibrarySources(rows, { query: "alpha", providers: [] }).map((row) => row.id))
      .toEqual(["source:2"]);
    expect(filterLibrarySources(rows, { query: "", providers: ["telegram"] }).map((row) => row.id))
      .toEqual(["source:1"]);
  });

  it("counts only connectable selected rows", () => {
    const rows: LibrarySourceView[] = [
      {
        id: "source:1",
        sourceId: 1,
        provider: "telegram",
        title: "Connectable",
        subtitle: null,
        projectCount: 0,
        lastCollectedLabel: null,
        localCopyLabel: "10 материалов",
        status: "active",
        disabledReason: null,
        alreadyConnected: false,
        connectable: true,
      },
      {
        id: "source:2",
        sourceId: 2,
        provider: "rss",
        title: "Unsupported",
        subtitle: null,
        projectCount: 0,
        lastCollectedLabel: null,
        localCopyLabel: "5 материалов",
        status: "unavailable",
        disabledReason: "RSS is not persistable.",
        alreadyConnected: false,
        connectable: false,
      },
    ];

    expect(connectableSelection(rows, new Set(["source:1", "source:2"]))).toEqual([rows[0]]);
  });

  it("builds a provider-safe source-group update command", () => {
    const project = buildResearchProjectsView([group()], [])[0];
    const libraryRows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 3, source_type: "telegram", title: "Drone News" }),
        source({ id: 4, source_type: "youtube", title: "Alpha Drones" }),
      ],
      [group()],
      "source-group:100",
    );

    expect(buildSourceGroupUpdateInput(project, group(), new Set(["source:3", "source:4"]), libraryRows))
      .toEqual({
        ok: true,
        input: {
          groupId: 100,
          name: "Рынок БПЛА",
          sourceType: "telegram",
          sourceIds: [1, 3],
        },
        connectedCount: 1,
        refusedCount: 1,
      });
  });

  it("renders project source links from already connected library rows", () => {
    const rows = buildLibrarySourcesView([source()], [group()], "source-group:100");

    expect(buildProjectSourceLinksView("source-group:100", rows)).toEqual([
      expect.objectContaining({
        projectId: "source-group:100",
        sourceId: "source:1",
        provider: "telegram",
        connectionStatus: "connected",
      }),
    ]);
  });
});
```

- [x] **Step 2: Run adapter tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-model.test.ts
```

Expected: FAIL because `research-projects-model.ts` does not exist.

- [x] **Step 3: Implement the adapter**

Create `src/lib/ui/research-projects-model.ts`:

```ts
import type {
  AnalysisGroupSourceType,
  AnalysisRunSummary,
  AnalysisSourceGroup,
  AnalysisSourceOption,
  AnalysisSourceOptionType,
  UpdateAnalysisSourceGroupInput,
} from "$lib/types/analysis";
import type { SourceJobRecord } from "$lib/types/sources";

export type LibrarySourceProvider = AnalysisSourceOptionType | "web" | "other";
export type ProjectStatus = "ready" | "running" | "needs_attention" | "empty";
export type LibrarySourceStatus = "active" | "needs_account" | "syncing" | "error" | "unavailable";

export type ResearchProjectBacking =
  | { kind: "source_group"; groupId: number; sourceType: AnalysisGroupSourceType }
  | { kind: "synthetic"; disabledReason: string };

export type ResearchProjectView = {
  id: string;
  title: string;
  description: string | null;
  periodLabel: string;
  sourceCount: number;
  evidenceCount: number;
  materialCount: number;
  lastRunLabel: string | null;
  status: ProjectStatus;
  backing: ResearchProjectBacking;
};

export type LibrarySourceView = {
  id: string;
  sourceId: number;
  provider: LibrarySourceProvider;
  title: string;
  subtitle: string | null;
  projectCount: number;
  lastCollectedLabel: string | null;
  localCopyLabel: string | null;
  status: LibrarySourceStatus;
  disabledReason: string | null;
  alreadyConnected: boolean;
  connectable: boolean;
};

export type ProjectSourceLinkView = {
  projectId: string;
  sourceId: string;
  provider: LibrarySourceProvider;
  title: string;
  connectionStatus: "connected" | "pending" | "failed" | "already_connected";
  filterSummary: string;
};

export type LibraryFilterState = {
  query: string;
  providers: LibrarySourceProvider[];
};

export type SourceGroupUpdateDecision =
  | {
      ok: true;
      input: UpdateAnalysisSourceGroupInput;
      connectedCount: number;
      refusedCount: number;
    }
  | { ok: false; reason: string; connectedCount: 0; refusedCount: number };

export const PROJECT_PERIOD_LABEL = "01.01.2024 - 31.05.2025";

const PROVIDER_LABELS: Record<LibrarySourceProvider, string> = {
  telegram: "Telegram",
  youtube: "YouTube",
  rss: "RSS",
  forum: "форумов",
  web: "Web",
  other: "источников этого типа",
};

function sourceProjectId(groupId: number) {
  return `source-group:${groupId}`;
}

function sourceRowId(sourceId: number) {
  return `source:${sourceId}`;
}

function materialLabel(count: number) {
  if (count === 1) return "1 материал";
  return `${count} материалов`;
}

function dateLabel(unixSeconds: number | null) {
  if (!unixSeconds) return null;
  return new Intl.DateTimeFormat("ru-RU", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(unixSeconds * 1000));
}

function latestRunLabel(project: AnalysisSourceGroup, runs: AnalysisRunSummary[]) {
  const run = runs
    .filter((candidate) => candidate.source_group_id === project.id)
    .sort((left, right) => right.created_at - left.created_at)[0];
  return run ? dateLabel(run.created_at) : null;
}

function projectStatus(group: AnalysisSourceGroup, runs: AnalysisRunSummary[]): ProjectStatus {
  if (runs.some((run) => run.source_group_id === group.id && (run.status === "queued" || run.status === "running"))) {
    return "running";
  }
  if (group.members.length === 0) return "empty";
  if (group.members.every((member) => member.item_count <= 0)) return "needs_attention";
  return "ready";
}

export function buildResearchProjectsView(
  groups: AnalysisSourceGroup[],
  runs: AnalysisRunSummary[] = [],
): ResearchProjectView[] {
  return groups.map((group) => {
    const materialCount = group.members.reduce((total, member) => total + member.item_count, 0);
    return {
      id: sourceProjectId(group.id),
      title: group.name,
      description: `${PROVIDER_LABELS[group.source_type]} проект, сохраненный через текущую модель источников.`,
      periodLabel: PROJECT_PERIOD_LABEL,
      sourceCount: group.members.length,
      evidenceCount: materialCount,
      materialCount,
      lastRunLabel: latestRunLabel(group, runs),
      status: projectStatus(group, runs),
      backing: { kind: "source_group", groupId: group.id, sourceType: group.source_type },
    };
  });
}

function groupMembership(groups: AnalysisSourceGroup[]) {
  const membership = new Map<number, Set<number>>();
  for (const group of groups) {
    for (const member of group.members) {
      const current = membership.get(member.source_id) ?? new Set<number>();
      current.add(group.id);
      membership.set(member.source_id, current);
    }
  }
  return membership;
}

function selectedGroup(groups: AnalysisSourceGroup[], selectedProjectId: string | null) {
  if (!selectedProjectId?.startsWith("source-group:")) return null;
  const groupId = Number(selectedProjectId.replace("source-group:", ""));
  return groups.find((group) => group.id === groupId) ?? null;
}

function unsupportedReason(provider: LibrarySourceProvider) {
  if (provider === "telegram" || provider === "youtube") return null;
  return `Подключение ${PROVIDER_LABELS[provider]} к проектам будет доступно после миграции библиотеки.`;
}

function providerMismatchReason(project: AnalysisSourceGroup | null, provider: LibrarySourceProvider) {
  if (!project) return "Выберите проект с сохраняемой группой источников.";
  if (provider === project.source_type) return null;
  return `Этот проект сейчас сохраняет только ${PROVIDER_LABELS[project.source_type]} источники.`;
}

function activeJobBySource(sourceJobs: SourceJobRecord[]) {
  const jobsBySource = new Map<number, SourceJobRecord>();
  for (const job of sourceJobs) {
    if (job.status !== "queued" && job.status !== "running" && job.status !== "failed") {
      continue;
    }
    const current = jobsBySource.get(job.source_id);
    if (!current || job.started_at > current.started_at) {
      jobsBySource.set(job.source_id, job);
    }
  }
  return jobsBySource;
}

function jobBlockedState(job: SourceJobRecord | undefined) {
  if (!job) return null;
  if (job.status === "queued" || job.status === "running") {
    return {
      status: "syncing" as const,
      disabledReason: "Источник сейчас синхронизируется.",
    };
  }
  if (job.status === "failed") {
    return {
      status: "error" as const,
      disabledReason: job.error
        ? `Последняя синхронизация завершилась ошибкой: ${job.error}`
        : "Последняя синхронизация завершилась ошибкой.",
    };
  }
  return null;
}

export function buildLibrarySourcesView(
  sources: AnalysisSourceOption[],
  groups: AnalysisSourceGroup[],
  selectedProjectId: string | null,
  sourceJobs: SourceJobRecord[] = [],
): LibrarySourceView[] {
  const membership = groupMembership(groups);
  const project = selectedGroup(groups, selectedProjectId);
  const connectedIds = new Set(project?.members.map((member) => member.source_id) ?? []);
  const jobsBySource = activeJobBySource(sourceJobs);

  return sources.map((source) => {
    const provider = source.source_type;
    const alreadyConnected = connectedIds.has(source.id);
    const jobState = jobBlockedState(jobsBySource.get(source.id));
    const disabledReason = jobState?.disabledReason ?? (alreadyConnected
      ? "Источник уже подключен к этому проекту."
      : unsupportedReason(provider) ?? providerMismatchReason(project, provider));
    const connectable = disabledReason === null;

    return {
      id: sourceRowId(source.id),
      sourceId: source.id,
      provider,
      title: source.title ?? `Source #${source.id}`,
      subtitle: source.account_id ? `Account #${source.account_id}` : null,
      projectCount: membership.get(source.id)?.size ?? 0,
      lastCollectedLabel: dateLabel(source.last_synced_at),
      localCopyLabel: materialLabel(source.item_count),
      status: jobState?.status ?? (connectable || alreadyConnected ? "active" : "unavailable"),
      disabledReason,
      alreadyConnected,
      connectable,
    };
  });
}

export function filterLibrarySources(
  sources: LibrarySourceView[],
  filters: LibraryFilterState,
) {
  const query = filters.query.trim().toLocaleLowerCase();
  const providers = new Set(filters.providers);
  return sources.filter((source) => {
    const matchesQuery = !query || `${source.title} ${source.subtitle ?? ""}`.toLocaleLowerCase().includes(query);
    const matchesProvider = providers.size === 0 || providers.has(source.provider);
    return matchesQuery && matchesProvider;
  });
}

export function connectableSelection(
  sources: LibrarySourceView[],
  selectedIds: Set<string>,
) {
  return sources.filter((source) => selectedIds.has(source.id) && source.connectable);
}

export function buildSourceGroupUpdateInput(
  project: ResearchProjectView | null,
  group: AnalysisSourceGroup | null,
  selectedIds: Set<string>,
  librarySources: LibrarySourceView[],
): SourceGroupUpdateDecision {
  const selected = librarySources.filter((source) => selectedIds.has(source.id));
  const connectable = selected.filter((source) => source.connectable);

  if (!project || project.backing.kind !== "source_group" || !group) {
    return {
      ok: false,
      reason: "Этот проект пока нельзя сохранить через текущую модель групп источников.",
      connectedCount: 0,
      refusedCount: selected.length,
    };
  }

  const allowed = connectable.filter((source) => source.provider === project.backing.sourceType);
  if (allowed.length === 0) {
    return {
      ok: false,
      reason: "В выбранных строках нет источников, которые можно подключить к этому проекту.",
      connectedCount: 0,
      refusedCount: selected.length,
    };
  }

  const sourceIds = Array.from(new Set([
    ...group.members.map((member) => member.source_id),
    ...allowed.map((source) => source.sourceId),
  ])).sort((left, right) => left - right);

  return {
    ok: true,
    input: {
      groupId: group.id,
      name: group.name,
      sourceType: project.backing.sourceType,
      sourceIds,
    },
    connectedCount: allowed.length,
    refusedCount: selected.length - allowed.length,
  };
}

export function buildProjectSourceLinksView(
  projectId: string | null,
  librarySources: LibrarySourceView[],
): ProjectSourceLinkView[] {
  if (!projectId) return [];
  return librarySources
    .filter((source) => source.alreadyConnected)
    .map((source) => ({
      projectId,
      sourceId: source.id,
      provider: source.provider,
      title: source.title,
      connectionStatus: "connected",
      filterSummary: "Фильтры проекта применяются при запуске анализа.",
    }));
}
```

- [x] **Step 4: Run adapter tests**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-model.test.ts
```

Expected: PASS.

- [x] **Step 5: Run full unit suite**

Run:

```powershell
npm.cmd run test
```

Expected: PASS.

- [x] **Step 6: Commit adapter**

Run:

```powershell
git add src/lib/ui/research-projects-model.ts src/lib/ui/research-projects-model.test.ts
git commit -m "feat: add research projects transition adapter"
```

Expected: commit succeeds.

---

## Task 3: Research Projects Workflow

**Files:**
- Create: `src/lib/ui/research-projects-workflow.test.ts`
- Create: `src/lib/ui/research-projects-workflow.ts`

- [x] **Step 1: Write workflow tests**

Create `src/lib/ui/research-projects-workflow.test.ts` with these scenarios:

```ts
import { describe, expect, it, vi } from "vitest";
import { createResearchProjectsWorkflow, type ResearchProjectsWorkflowState } from "./research-projects-workflow";
import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
import type { SourceJobRecord } from "$lib/types/sources";

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 10,
    name: "Рынок БПЛА",
    source_type: "telegram",
    members: [{ source_id: 1, source_title: "Radar BPLA", item_count: 12 }],
    created_at: 100,
    updated_at: 200,
    ...overrides,
  };
}

function source(overrides: Partial<AnalysisSourceOption> = {}): AnalysisSourceOption {
  return {
    id: 2,
    account_id: 1,
    source_type: "telegram",
    title: "Drone News",
    item_count: 20,
    last_synced_at: 300,
    ...overrides,
  };
}

function sourceJob(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 2,
    related_source_id: null,
    job_type: "youtube_video_full_sync",
    status: "running",
    message: "Syncing",
    progress_current: 1,
    progress_total: 3,
    started_at: 300,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

function createHarness(initial: Partial<ResearchProjectsWorkflowState> = {}) {
  const state: ResearchProjectsWorkflowState = {
    groups: [],
    sources: [],
    runs: [],
    sourceJobs: [],
    projects: [],
    librarySources: [],
    projectSourceLinks: [],
    selectedProjectId: null,
    selectedLibrarySourceIds: new Set<string>(),
    loading: false,
    saving: false,
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: Partial<ResearchProjectsWorkflowState>) => Object.assign(state, patch)),
    listGroups: vi.fn(),
    listSources: vi.fn(),
    listRuns: vi.fn(),
    listSourceJobs: vi.fn(),
    updateGroup: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  return { state, deps, workflow: createResearchProjectsWorkflow(deps) };
}

describe("research projects workflow", () => {
  it("loads projects, sources, and selects the first project", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listGroups.mockResolvedValueOnce([group()]);
    deps.listSources.mockResolvedValueOnce([source({ id: 1, title: "Radar BPLA" })]);
    deps.listRuns.mockResolvedValueOnce([]);
    deps.listSourceJobs.mockResolvedValueOnce([]);

    await workflow.loadWorkspace();

    expect(state.selectedProjectId).toBe("source-group:10");
    expect(state.projects[0].title).toBe("Рынок БПЛА");
    expect(state.librarySources[0].alreadyConnected).toBe(true);
    expect(state.projectSourceLinks).toHaveLength(1);
    expect(state.loading).toBe(false);
  });

  it("threads source jobs into derived library rows", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listGroups.mockResolvedValueOnce([group({ source_type: "youtube", members: [] })]);
    deps.listSources.mockResolvedValueOnce([source({ id: 2, source_type: "youtube", title: "Alpha Drones" })]);
    deps.listRuns.mockResolvedValueOnce([]);
    deps.listSourceJobs.mockResolvedValueOnce([sourceJob()]);

    await workflow.loadWorkspace();

    expect(state.sourceJobs).toHaveLength(1);
    expect(state.librarySources[0]).toEqual(expect.objectContaining({
      status: "syncing",
      connectable: false,
      disabledReason: "Источник сейчас синхронизируется.",
    }));
  });

  it("persists only safe selected rows through updateGroup", async () => {
    const currentGroup = group();
    const { state, deps, workflow } = createHarness({
      groups: [currentGroup],
      sources: [source({ id: 1, title: "Radar BPLA" }), source({ id: 2, title: "Drone News" })],
      selectedProjectId: "source-group:10",
      selectedLibrarySourceIds: new Set(["source:2"]),
    });
    deps.updateGroup.mockResolvedValueOnce({ ...currentGroup, members: [...currentGroup.members, { source_id: 2, source_title: "Drone News", item_count: 20 }] });
    deps.listGroups.mockResolvedValueOnce([{ ...currentGroup, members: [...currentGroup.members, { source_id: 2, source_title: "Drone News", item_count: 20 }] }]);
    deps.listSources.mockResolvedValueOnce(state.sources);
    deps.listRuns.mockResolvedValueOnce([]);
    deps.listSourceJobs.mockResolvedValueOnce([]);

    await workflow.refreshDerivedState();
    await workflow.connectSelectedSources();

    expect(deps.updateGroup).toHaveBeenCalledWith({
      groupId: 10,
      name: "Рынок БПЛА",
      sourceType: "telegram",
      sourceIds: [1, 2],
    });
    expect(state.status).toBe("Подключено источников: 1.");
    expect(state.selectedLibrarySourceIds.size).toBe(0);
    expect(state.saving).toBe(false);
  });

  it("refuses unsupported or already-connected selections without calling updateGroup", async () => {
    const { state, deps, workflow } = createHarness({
      groups: [group()],
      sources: [source({ id: 3, source_type: "rss", title: "Новости БПЛА" })],
      selectedProjectId: "source-group:10",
      selectedLibrarySourceIds: new Set(["source:3"]),
    });

    await workflow.refreshDerivedState();
    await workflow.connectSelectedSources();

    expect(deps.updateGroup).not.toHaveBeenCalled();
    expect(state.status).toBe("В выбранных строках нет источников, которые можно подключить к этому проекту.");
  });
});
```

- [x] **Step 2: Run workflow tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-workflow.test.ts
```

Expected: FAIL because `research-projects-workflow.ts` does not exist.

- [x] **Step 3: Implement workflow**

Create `src/lib/ui/research-projects-workflow.ts`:

```ts
import {
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  buildSourceGroupUpdateInput,
  type LibrarySourceView,
  type ProjectSourceLinkView,
  type ResearchProjectView,
} from "./research-projects-model";
import type {
  AnalysisRunSummary,
  AnalysisSourceGroup,
  AnalysisSourceOption,
  UpdateAnalysisSourceGroupInput,
} from "$lib/types/analysis";
import type { SourceJobRecord } from "$lib/types/sources";

export interface ResearchProjectsWorkflowState {
  groups: AnalysisSourceGroup[];
  sources: AnalysisSourceOption[];
  runs: AnalysisRunSummary[];
  sourceJobs: SourceJobRecord[];
  projects: ResearchProjectView[];
  librarySources: LibrarySourceView[];
  projectSourceLinks: ProjectSourceLinkView[];
  selectedProjectId: string | null;
  selectedLibrarySourceIds: Set<string>;
  loading: boolean;
  saving: boolean;
  status: string;
}

export interface ResearchProjectsWorkflowDeps {
  getState(): ResearchProjectsWorkflowState;
  patch(patch: Partial<ResearchProjectsWorkflowState>): void;
  listGroups(): Promise<AnalysisSourceGroup[]>;
  listSources(): Promise<AnalysisSourceOption[]>;
  listRuns(): Promise<AnalysisRunSummary[]>;
  listSourceJobs(): Promise<SourceJobRecord[]>;
  updateGroup(input: UpdateAnalysisSourceGroupInput): Promise<AnalysisSourceGroup>;
  formatError(action: string, error: unknown): string;
}

function selectedProject(
  projects: ResearchProjectView[],
  selectedProjectId: string | null,
) {
  return projects.find((project) => project.id === selectedProjectId) ?? projects[0] ?? null;
}

function selectedGroup(
  groups: AnalysisSourceGroup[],
  selectedProjectId: string | null,
) {
  if (!selectedProjectId?.startsWith("source-group:")) return null;
  const groupId = Number(selectedProjectId.replace("source-group:", ""));
  return groups.find((group) => group.id === groupId) ?? null;
}

export function createResearchProjectsWorkflow(deps: ResearchProjectsWorkflowDeps) {
  async function refreshDerivedState() {
    const state = deps.getState();
    const projects = buildResearchProjectsView(state.groups, state.runs);
    const currentProject = selectedProject(projects, state.selectedProjectId);
    const selectedProjectId = currentProject?.id ?? null;
    const librarySources = buildLibrarySourcesView(
      state.sources,
      state.groups,
      selectedProjectId,
      state.sourceJobs,
    );
    deps.patch({
      projects,
      selectedProjectId,
      librarySources,
      projectSourceLinks: buildProjectSourceLinksView(selectedProjectId, librarySources),
    });
  }

  async function loadWorkspace() {
    deps.patch({ loading: true });
    try {
      const [groups, sources, runs, sourceJobs] = await Promise.all([
        deps.listGroups(),
        deps.listSources(),
        deps.listRuns(),
        deps.listSourceJobs(),
      ]);
      deps.patch({ groups, sources, runs, sourceJobs });
      await refreshDerivedState();
    } catch (error) {
      deps.patch({ status: deps.formatError("loading research projects", error) });
    } finally {
      deps.patch({ loading: false });
    }
  }

  async function connectSelectedSources() {
    const state = deps.getState();
    const project = selectedProject(state.projects, state.selectedProjectId);
    const group = selectedGroup(state.groups, state.selectedProjectId);
    const decision = buildSourceGroupUpdateInput(
      project,
      group,
      state.selectedLibrarySourceIds,
      state.librarySources,
    );

    if (!decision.ok) {
      deps.patch({ status: decision.reason });
      return;
    }

    deps.patch({ saving: true });
    try {
      await deps.updateGroup(decision.input);
      deps.patch({
        status: decision.refusedCount > 0
          ? `Подключено источников: ${decision.connectedCount}. Отклонено: ${decision.refusedCount}.`
          : `Подключено источников: ${decision.connectedCount}.`,
        selectedLibrarySourceIds: new Set<string>(),
      });
      await loadWorkspace();
    } catch (error) {
      deps.patch({ status: deps.formatError("connecting library sources", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  return {
    refreshDerivedState,
    loadWorkspace,
    connectSelectedSources,
  };
}
```

- [x] **Step 4: Run workflow tests**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-workflow.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit workflow**

Run:

```powershell
git add src/lib/ui/research-projects-workflow.ts src/lib/ui/research-projects-workflow.test.ts
git commit -m "feat: add research projects workflow"
```

Expected: commit succeeds.

---

## Task 4: Extractum shadcn Wrapper Layer

**Files:**
- Create: `src/lib/research-projects-import-boundary.test.ts`
- Create: `src/lib/components/extractum-ui/Button.svelte`
- Create: `src/lib/components/extractum-ui/TextInput.svelte`
- Create: `src/lib/components/extractum-ui/Select.svelte`
- Create: `src/lib/components/extractum-ui/Checkbox.svelte`
- Create: `src/lib/components/extractum-ui/Badge.svelte`
- Create: `src/lib/components/extractum-ui/Tabs.svelte`
- Create: `src/lib/components/extractum-ui/Sheet.svelte`
- Create: `src/lib/components/extractum-ui/ProviderBadge.svelte`
- Create: `src/lib/components/extractum-ui/StatusBadge.svelte`
- Create: `src/lib/components/extractum-ui/index.ts`

- [x] **Step 1: Write import-boundary test**

Create `src/lib/research-projects-import-boundary.test.ts`:

```ts
// @ts-nocheck
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));

function collectFiles(relativeDir: string): string[] {
  const fullDir = path.join(repoRoot, relativeDir);
  if (!existsSync(fullDir)) return [];
  return readdirSync(fullDir).flatMap((entry) => {
    const fullPath = path.join(fullDir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) return collectFiles(path.relative(repoRoot, fullPath));
    return [fullPath];
  });
}

function sourceOf(file: string) {
  return readFileSync(file, "utf8");
}

describe("research projects import boundaries", () => {
  it("keeps new feature screens behind Extractum UI wrappers", () => {
    const featureFiles = [
      ...collectFiles("src/lib/components/research-projects"),
      ...collectFiles("src/routes/projects"),
    ].filter((file) => file.endsWith(".svelte") || file.endsWith(".ts"));

    const offenders = featureFiles
      .map((file) => [path.relative(repoRoot, file).replaceAll("\\", "/"), sourceOf(file)] as const)
      .filter(([, source]) =>
        source.includes("@svar-ui/") ||
        source.includes("bits-ui") ||
        source.includes("$lib/components/ui/"),
      )
      .map(([file]) => file);

    expect(offenders).toEqual([]);
  });

  it("allows lower-level library imports only in the product wrapper layer", () => {
    const wrapperFiles = collectFiles("src/lib/components/extractum-ui");
    const wrapperSources = wrapperFiles.map(sourceOf).join("\n");

    expect(wrapperSources).toContain("$lib/components/ui/button/index.js");
    expect(wrapperSources).toContain("$lib/components/ui/sheet/index.js");
    expect(wrapperSources).not.toContain("src/lib/new-ui");
  });
});
```

- [x] **Step 2: Run boundary test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
```

Expected: FAIL because `extractum-ui` wrappers do not exist.

- [x] **Step 3: Create wrapper exports**

Create `src/lib/components/extractum-ui/index.ts`:

```ts
export { default as ExtractumButton } from "./Button.svelte";
export { default as ExtractumTextInput } from "./TextInput.svelte";
export { default as ExtractumSelect } from "./Select.svelte";
export { default as ExtractumCheckbox } from "./Checkbox.svelte";
export { default as ExtractumBadge } from "./Badge.svelte";
export { default as ExtractumTabs } from "./Tabs.svelte";
export { default as ExtractumSheet } from "./Sheet.svelte";
export { default as ProviderBadge } from "./ProviderBadge.svelte";
export { default as StatusBadge } from "./StatusBadge.svelte";
```

- [x] **Step 4: Create shadcn wrapper components**

Create compact wrappers over shadcn primitives. Each wrapper accepts a `class` prop and adds stable product classes:

```svelte
<!-- src/lib/components/extractum-ui/Button.svelte -->
<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let {
    class: className,
    density = "compact",
    ...rest
  }: ComponentProps<typeof Button> & { density?: "compact" | "normal" } = $props();
</script>

<Button
  class={cn(
    "extractum-button",
    density === "compact" && "h-[var(--extractum-density-control-height)] rounded-[var(--extractum-radius)] px-2 text-[13px]",
    className,
  )}
  {...rest}
/>
```

Create `TextInput.svelte` and `Badge.svelte` exactly as follows:

```svelte
<!-- TextInput.svelte -->
<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";
  let { class: className, ...rest }: ComponentProps<typeof Input> = $props();
</script>

<Input class={cn("extractum-input h-[32px] rounded-[var(--extractum-radius)] text-[13px]", className)} {...rest} />
```

```svelte
<!-- Badge.svelte -->
<script lang="ts">
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";
  let { class: className, ...rest }: ComponentProps<typeof Badge> = $props();
</script>

<Badge class={cn("extractum-badge rounded-[4px] px-1.5 py-0 text-[11px] font-medium", className)} {...rest} />
```

For `Select.svelte`, `Checkbox.svelte`, `Tabs.svelte`, and `Sheet.svelte`, re-export or wrap the generated shadcn namespace while adding product classes at the feature usage boundary. `Sheet.svelte` must wrap `Sheet.Content` with a wide class suitable for Connect from Library: `w-[min(1180px,calc(100vw-96px))]`.

- [x] **Step 5: Create provider and status badges**

Create `ProviderBadge.svelte` with provider classes for `telegram`, `youtube`, `rss`, `forum`, `web`, and `other`. Create `StatusBadge.svelte` with statuses `active`, `needs_account`, `syncing`, `error`, and `unavailable`. Use `ExtractumBadge` internally.

- [x] **Step 6: Run boundary and type checks**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
npm.cmd run check
```

Expected: PASS.

- [x] **Step 7: Commit wrappers**

Run:

```powershell
git add src/lib/components/extractum-ui src/lib/research-projects-import-boundary.test.ts
git commit -m "feat: add Extractum UI wrapper layer"
```

Expected: commit succeeds.

---

## Task 5: SVAR Data Grid Product Wrapper

**Files:**
- Modify: `src/lib/components/extractum-ui/index.ts`
- Create: `src/lib/components/extractum-ui/DataGrid.svelte`
- Create: `src/lib/components/extractum-ui/GridSelectCell.svelte`
- Create: `src/lib/types/svar-locales.d.ts`
- Modify: `src/lib/research-projects-import-boundary.test.ts`

- [x] **Step 1: Extend boundary test for SVAR wrapper ownership**

Add this test case to `src/lib/research-projects-import-boundary.test.ts`:

```ts
it("routes SVAR Grid through ExtractumDataGrid only", () => {
  const dataGridSource = readFileSync(
    path.join(repoRoot, "src/lib/components/extractum-ui/DataGrid.svelte"),
    "utf8",
  );
  expect(dataGridSource).toContain('from "@svar-ui/svelte-grid"');
  expect(dataGridSource).toContain("selectedRows");
  expect(dataGridSource).toContain("rowStyle");
  expect(dataGridSource).toContain("Locale");
  expect(dataGridSource).toContain("Willow");
  expect(dataGridSource).toContain("fonts={false}");

  const selectCellSource = readFileSync(
    path.join(repoRoot, "src/lib/components/extractum-ui/GridSelectCell.svelte"),
    "utf8",
  );
  expect(selectCellSource).toContain('data-action="ignore-click"');
  expect(selectCellSource).toContain('api.exec("select-row"');
});
```

- [x] **Step 2: Run the boundary test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
```

Expected: FAIL because `DataGrid.svelte` and `GridSelectCell.svelte` are missing.

- [x] **Step 3: Implement SVAR checkbox cell**

Create `src/lib/components/extractum-ui/GridSelectCell.svelte`:

```svelte
<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";

  let { api, row } = $props<{
    api: { exec: (action: string, data: Record<string, unknown>) => void };
    row: { id: string; selected?: boolean; connectable?: boolean; disabledReason?: string | null };
  }>();

  function toggle(checked: boolean) {
    if (row.connectable === false) return;
    api.exec("select-row", { id: row.id, mode: checked, toggle: true });
  }
</script>

<div class="extractum-grid-select-cell" data-action="ignore-click" title={row.disabledReason ?? undefined}>
  <Checkbox disabled={row.connectable === false} checked={!!row.selected} onCheckedChange={toggle} aria-label="Выбрать источник" />
</div>
```

- [x] **Step 4: Implement SVAR DataGrid wrapper**

Create `src/lib/components/extractum-ui/DataGrid.svelte`:

```svelte
<script lang="ts">
  import { Grid, Willow, type IColumnConfig } from "@svar-ui/svelte-grid";
  import { Locale } from "@svar-ui/svelte-core";
  import { ru as gridRu } from "@svar-ui/grid-locales";
  import { ru as coreRu } from "@svar-ui/core-locales";
  import { cn } from "$lib/utils.js";

  type GridRow = { id: string; connectable?: boolean; alreadyConnected?: boolean; status?: string };

  let {
    rows,
    columns,
    selectedRowIds = [],
    height = "100%",
    class: className,
    overlay = "Нет данных",
    onSelectedRowIdsChange = () => {},
  }: {
    rows: GridRow[];
    columns: IColumnConfig[];
    selectedRowIds?: string[];
    height?: string;
    class?: string;
    overlay?: string;
    onSelectedRowIdsChange?: (ids: string[]) => void;
  } = $props();

  let api = $state<any>(null);

  function rowStyle(row: GridRow) {
    return [
      row.connectable === false ? "is-disabled" : "",
      row.alreadyConnected ? "is-connected" : "",
      row.status ? `status-${row.status}` : "",
    ].filter(Boolean).join(" ");
  }

  function emitSelection() {
    if (!api) return;
    onSelectedRowIdsChange(api.getState().selectedRows.map(String));
  }
</script>

<div class={cn("extractum-svar-theme extractum-data-grid", className)} style={`height:${height};`}>
  <Locale words={{ ...coreRu, ...gridRu }}>
    <Willow fonts={false}>
      <Grid
        data={rows}
        {columns}
        bind:this={api}
        selectedRows={selectedRowIds}
        {rowStyle}
        {overlay}
        multiselect
        select
        sizes={{ rowHeight: 34, headerHeight: 34, columnWidth: 160 }}
        onselectrow={emitSelection}
      />
    </Willow>
  </Locale>
</div>

<style>
  .extractum-data-grid {
    min-height: 0;
    width: 100%;
    overflow: hidden;
  }

  .extractum-data-grid :global(.wx-grid),
  .extractum-data-grid :global(.wx-table-box) {
    height: 100%;
  }

  .extractum-data-grid :global(.wx-cell) {
    padding: 5px 8px;
    font-size: 12.5px;
  }

  .extractum-data-grid :global(.wx-row.is-disabled:not(.wx-selected) .wx-cell) {
    color: var(--extractum-muted);
    background: color-mix(in srgb, var(--extractum-surface-subtle) 80%, transparent);
  }

  .extractum-data-grid :global(.wx-row.is-connected:not(.wx-selected) .wx-cell) {
    background: color-mix(in srgb, var(--extractum-success) 8%, var(--extractum-surface));
  }
 </style>
```

- [x] **Step 5: Export the DataGrid wrapper**

Add to `src/lib/components/extractum-ui/index.ts`:

```ts
export { default as ExtractumDataGrid } from "./DataGrid.svelte";
export { default as GridSelectCell } from "./GridSelectCell.svelte";
```

- [x] **Step 6: Run boundary and check**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
npm.cmd run check
```

Expected: PASS. If the exact generated shadcn Checkbox API differs, adapt `GridSelectCell.svelte` to the generated API and keep the `data-action="ignore-click"` and `api.exec("select-row", ...)` contract.

- [x] **Step 7: Commit SVAR wrapper**

Run:

```powershell
git add src/lib/components/extractum-ui src/lib/research-projects-import-boundary.test.ts
git commit -m "feat: wrap SVAR grid for Extractum UI"
```

Expected: commit succeeds.

---

## Task 6: Route Contract And New `/projects` Shell

**Files:**
- Create: `src/lib/research-projects-route-contract.test.ts`
- Create: `src/routes/projects/+page.svelte`
- Create: `src/lib/components/research-projects/ProjectsShell.svelte`
- Create: `src/lib/components/research-projects/IconRail.svelte`
- Create: `src/lib/components/research-projects/ProjectRail.svelte`
- Create: `src/lib/components/research-projects/TopCommandBar.svelte`
- Create: `src/lib/components/research-projects/ProjectWorkspace.svelte`

- [x] **Step 1: Write route contract**

Create `src/lib/research-projects-route-contract.test.ts`:

```ts
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import projectsRouteSource from "../routes/projects/+page.svelte?raw";
import shellSource from "./components/research-projects/ProjectsShell.svelte?raw";
import projectRailSource from "./components/research-projects/ProjectRail.svelte?raw";
import workspaceSource from "./components/research-projects/ProjectWorkspace.svelte?raw";

const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));

describe("research projects route contract", () => {
  it("adds the new route without redirecting through the old analysis workspace", () => {
    expect(projectsRouteSource).toContain('data-ui-route="research-projects"');
    expect(projectsRouteSource).toContain("createResearchProjectsWorkflow");
    expect(projectsRouteSource).toContain("listAnalysisSourceGroups");
    expect(projectsRouteSource).toContain("listAnalysisSources");
    expect(projectsRouteSource).toContain("listSourceJobs");
    expect(projectsRouteSource).not.toContain('goto("/analysis")');
  });

  it("renders the dense project control deck regions", () => {
    expect(shellSource).toContain('data-ui-region="icon-rail"');
    expect(shellSource).toContain('data-ui-region="project-rail"');
    expect(shellSource).toContain('data-ui-region="top-command-bar"');
    expect(shellSource).toContain('data-ui-region="project-workspace"');
    expect(shellSource).toContain("grid-template-columns: 56px 260px minmax(0, 1fr)");
  });

  it("keeps project rail and workspace in product language", () => {
    expect(projectRailSource).toContain("Проекты");
    expect(projectRailSource).not.toContain("source group");
    expect(workspaceSource).toContain("Overview");
    expect(workspaceSource).toContain("Sources");
    expect(workspaceSource).toContain("Evidence");
  });
});
```

- [x] **Step 2: Run route contract and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts
```

Expected: FAIL because route and shell components do not exist.

- [x] **Step 3: Implement shell components**

Create:
- `IconRail.svelte`: narrow rail with lucide icons for Projects, Library, Runs, Diagnostics, Settings. Use icon-only buttons/links with `title`.
- `ProjectRail.svelte`: project search, list of `ResearchProjectView`, counts, selected state.
- `TopCommandBar.svelte`: period, prompt preset, model select, run button, export menu button.
- `ProjectWorkspace.svelte`: tabs (`Overview`, `Sources`, `Evidence`, `Reports`, `Runs`, `Prompts`) using `ExtractumTabs`.
- `ProjectsShell.svelte`: three-column Ultra HD shell with the required `data-ui-region` attributes.

Use this layout CSS in `ProjectsShell.svelte`:

```css
.projects-shell {
  display: grid;
  grid-template-columns: 56px 260px minmax(0, 1fr);
  min-height: calc(100vh - 68px);
  border: 1px solid var(--extractum-border);
  background: var(--extractum-surface);
}
```

- [x] **Step 4: Implement `/projects` route**

Create `src/routes/projects/+page.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import ProjectsShell from "$lib/components/research-projects/ProjectsShell.svelte";
  import { listAnalysisSourceGroups, updateAnalysisSourceGroup } from "$lib/api/analysis-source-groups";
  import { listAnalysisSources } from "$lib/api/analysis-workspace";
  import { listActiveAnalysisRuns } from "$lib/api/analysis-runs";
  import { listSourceJobs } from "$lib/api/source-jobs";
  import { createResearchProjectsWorkflow, type ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";

  const state = $state<ResearchProjectsWorkflowState>({
    groups: [],
    sources: [],
    runs: [],
    sourceJobs: [],
    projects: [],
    librarySources: [],
    projectSourceLinks: [],
    selectedProjectId: null,
    selectedLibrarySourceIds: new Set<string>(),
    loading: false,
    saving: false,
    status: "",
  });

  const workflow = createResearchProjectsWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listGroups: listAnalysisSourceGroups,
    listSources: listAnalysisSources,
    listRuns: listActiveAnalysisRuns,
    listSourceJobs: () => listSourceJobs({ limit: 50 }),
    updateGroup: updateAnalysisSourceGroup,
    formatError: (action, error) => `Ошибка ${action}: ${String(error)}`,
  });

  onMount(() => {
    void workflow.loadWorkspace();
  });

  function selectProject(projectId: string) {
    state.selectedProjectId = projectId;
    void workflow.refreshDerivedState();
  }
</script>

<section data-ui-route="research-projects">
  <ProjectsShell
    {state}
    onSelectProject={selectProject}
    onConnectSelectedSources={workflow.connectSelectedSources}
    onSelectedLibrarySourceIdsChange={(ids) => (state.selectedLibrarySourceIds = new Set(ids))}
  />
</section>
```

- [x] **Step 5: Run route contract and import-boundary tests**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts src/lib/research-projects-import-boundary.test.ts
npm.cmd run check
```

Expected: PASS.

- [x] **Step 6: Commit shell**

Run:

```powershell
git add src/routes/projects src/lib/components/research-projects src/lib/research-projects-route-contract.test.ts
git commit -m "feat: add research projects route shell"
```

Expected: commit succeeds.

---

## Task 7: Sources Tab With SVAR Project Sources Grid

**Files:**
- Modify: `src/lib/research-projects-route-contract.test.ts`
- Create: `src/lib/components/research-projects/SourcesTab.svelte`
- Create: `src/lib/components/research-projects/ProjectSourceSummary.svelte`
- Create: `src/lib/components/research-projects/LibrarySourceCell.svelte`
- Modify: `src/lib/components/research-projects/ProjectWorkspace.svelte`

- [x] **Step 1: Add route contract coverage for Sources tab**

Add assertions:

```ts
import sourcesTabSource from "./components/research-projects/SourcesTab.svelte?raw";

it("uses SVAR-backed product grid for project sources", () => {
  expect(sourcesTabSource).toContain("ExtractumDataGrid");
  expect(sourcesTabSource).toContain("ProviderBadge");
  expect(sourcesTabSource).toContain("StatusBadge");
  expect(sourcesTabSource).toContain('data-ui-action="connect-library"');
  expect(sourcesTabSource).not.toContain("@svar-ui/");
});
```

- [x] **Step 2: Run route contract and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts
```

Expected: FAIL because `SourcesTab.svelte` does not exist.

- [x] **Step 3: Implement Sources tab**

`SourcesTab.svelte` responsibilities:
- show connected counts and material counts through `ProjectSourceSummary`;
- render connected project sources with `ExtractumDataGrid`;
- use `LibrarySourceCell` for the source/name column;
- show `ProviderBadge` and `StatusBadge`;
- expose `Connect from Library` button with `data-ui-action="connect-library"`.

Columns should use stable ids:

```ts
const columns = [
  { id: "title", header: "Источник", flexgrow: 1, cell: LibrarySourceCell },
  { id: "provider", header: "Тип", width: 120 },
  { id: "localCopyLabel", header: "Локальная копия", width: 140 },
  { id: "connectionStatus", header: "Статус", width: 140 },
];
```

- [x] **Step 4: Mount Sources tab in ProjectWorkspace**

`ProjectWorkspace.svelte` must pass:
- current project;
- `state.projectSourceLinks`;
- `state.librarySources`;
- `onOpenConnectLibrary`.

Keep placeholder panels for `Evidence`, `Reports`, `Runs`, and `Prompts`, but do not implement those out-of-scope workspaces.

- [x] **Step 5: Run tests and check**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts src/lib/research-projects-import-boundary.test.ts
npm.cmd run check
```

Expected: PASS.

- [x] **Step 6: Commit Sources tab**

Run:

```powershell
git add src/lib/components/research-projects src/lib/research-projects-route-contract.test.ts
git commit -m "feat: add research project sources tab"
```

Expected: commit succeeds.

---

## Task 8: Connect From Library Workflow

**Files:**
- Modify: `src/lib/research-projects-route-contract.test.ts`
- Create: `src/lib/components/research-projects/ConnectFromLibrary.svelte`
- Create: `src/lib/components/research-projects/BottomQueue.svelte`
- Modify: `src/lib/components/research-projects/ProjectsShell.svelte`
- Modify: `src/lib/components/research-projects/SourcesTab.svelte`

- [ ] **Step 1: Add route contract coverage for Connect from Library**

Add assertions:

```ts
import connectSource from "./components/research-projects/ConnectFromLibrary.svelte?raw";

it("renders the Connect from Library working sheet with searchable SVAR grid", () => {
  expect(connectSource).toContain("ExtractumSheet");
  expect(connectSource).toContain("ExtractumDataGrid");
  expect(connectSource).toContain("GridSelectCell");
  expect(connectSource).toContain('data-ui-panel="library-connect"');
  expect(connectSource).toContain('placeholder="Поиск по источникам..."');
  expect(connectSource).toContain('data-ui-panel="project-filters"');
  expect(connectSource).toContain('data-ui-panel="change-log"');
  expect(connectSource).toContain("selectedConnectableCount");
  expect(connectSource).toContain("Подключить выбранные");
  expect(connectSource).not.toContain("@svar-ui/");
  expect(connectSource).not.toContain("$lib/components/ui/");
});

it("renders the bottom queue from source jobs and active LLM runs", () => {
  const bottomQueueSource = readFileSync(
    path.join(repoRoot, "src/lib/components/research-projects/BottomQueue.svelte"),
    "utf8",
  );
  expect(bottomQueueSource).toContain("sourceJobs");
  expect(bottomQueueSource).toContain("runs");
  expect(bottomQueueSource).toContain('data-ui-region="bottom-queue"');
});
```

- [ ] **Step 2: Run route contract and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts
```

Expected: FAIL because `ConnectFromLibrary.svelte` does not exist.

- [ ] **Step 3: Implement ConnectFromLibrary**

Component props:

```ts
{
  open: boolean;
  project: ResearchProjectView | null;
  librarySources: LibrarySourceView[];
  selectedSourceIds: Set<string>;
  saving: boolean;
  status: string;
  onOpenChange: (open: boolean) => void;
  onSelectedSourceIdsChange: (ids: string[]) => void;
  onConnectSelectedSources: () => void | Promise<void>;
}
```

Behavior:
- search input filters with `filterLibrarySources`;
- provider chips filter providers;
- SVAR `ExtractumDataGrid` renders columns:
  - `selected` with `GridSelectCell`, width 44;
  - `title` with `LibrarySourceCell`, flex;
  - `provider`, width 120;
  - `projectCount`, width 90;
  - `lastCollectedLabel`, width 160;
  - `localCopyLabel`, width 130;
  - `status`, width 130;
- selected count counts only `connectableSelection`;
- primary button is disabled when `selectedConnectableCount === 0 || saving`;
- a project filter panel with `data-ui-panel="project-filters"` shows `project.periodLabel`, material type chips (`Статьи`, `Посты`, `Видео`), include-comments and include-transcripts checkboxes, and tag chips for `бпла` and `регулирование`;
- a change log panel with `data-ui-panel="change-log"` lists already-connected rows, refused selected rows with `disabledReason`, and active/failed job rows from `librarySources`;
- rows with `disabledReason` show the reason in the side status panel and in the row tooltip.

Use this derived count:

```ts
let query = $state("");
let providerFilters = $state<LibrarySourceProvider[]>([]);
let filteredSources = $derived(filterLibrarySources(librarySources, { query, providers: providerFilters }));
let selectedConnectableCount = $derived(connectableSelection(librarySources, selectedSourceIds).length);
```

- [ ] **Step 4: Mount the sheet from ProjectsShell**

`ProjectsShell.svelte` owns `let connectOpen = $state(false);`.

Wire:
- `SourcesTab` opens it through `onOpenConnectLibrary={() => (connectOpen = true)}`;
- `ConnectFromLibrary` receives `project={currentProject}`, `state.librarySources`, `state.selectedLibrarySourceIds`, `state.saving`, `state.status`, and callbacks from the route.

- [ ] **Step 5: Add BottomQueue**

Create `BottomQueue.svelte` as a compact pinned strip with `data-ui-region="bottom-queue"` and these props:

```ts
{
  loading: boolean;
  saving: boolean;
  status: string;
  sourceJobs: SourceJobRecord[];
  runs: AnalysisRunSummary[];
}
```

It renders:
- loading and saving state;
- current status;
- queued/running source jobs with progress labels from `progress_current` and `progress_total`;
- failed source jobs with `error`;
- queued/running LLM runs from `runs` with their `scope_label` and `status`.

Mount it at the bottom of `ProjectsShell` and pass:

```svelte
<BottomQueue
  loading={state.loading}
  saving={state.saving}
  status={state.status}
  sourceJobs={state.sourceJobs}
  runs={state.runs}
/>
```

- [ ] **Step 6: Run tests and check**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/research-projects-route-contract.test.ts src/lib/research-projects-import-boundary.test.ts
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 7: Commit Connect from Library**

Run:

```powershell
git add src/lib/components/research-projects src/lib/research-projects-route-contract.test.ts
git commit -m "feat: add library connect workflow"
```

Expected: commit succeeds.

---

## Task 9: Old UI Fallback And Navigation Preservation

**Files:**
- Modify: `src/lib/research-projects-route-contract.test.ts`
- Modify: `src/routes/+page.svelte` or leave unchanged after test confirms old default behavior.
- Modify: `src/routes/+layout.svelte`

- [ ] **Step 1: Add fallback contract**

Add:

```ts
import homeRouteSource from "../routes/+page.svelte?raw";
import layoutSource from "../routes/+layout.svelte?raw";

it("keeps old analysis fallback available while the new UI lives at /projects", () => {
  expect(homeRouteSource).toContain('goto("/analysis")');
  expect(layoutSource).toContain('href: "/projects"');
  expect(layoutSource).toContain('href: "/analysis"');
  expect(projectsRouteSource).toContain('data-ui-route="research-projects"');
});
```

- [ ] **Step 2: Run the route contract and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts
```

Expected: FAIL because `/projects` has not been added to the app navigation yet.

- [ ] **Step 3: Add Projects nav entry without removing Analysis**

In `src/routes/+layout.svelte`, add a new nav item before Workspace:

```ts
{
  href: "/projects",
  label: "Projects",
  caption: "Research control deck",
  icon: FolderKanban,
  active: (pathname: string) => pathname.startsWith("/projects"),
},
```

Keep `/analysis` as `"Workspace"` so the old UI remains reachable.

- [ ] **Step 4: Run tests and check**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts src/lib/analysis-ui-smoke-contract.test.ts
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 5: Commit fallback preservation**

Run:

```powershell
git add src/routes/+layout.svelte src/lib/research-projects-route-contract.test.ts
git commit -m "feat: expose projects route without removing analysis"
```

Expected: commit succeeds.

---

## Task 10: Visual QA, Ultra HD Density, And Final Verification

**Files:**
- Create: `docs/superpowers/verification/2026-06-11-new-ui-research-projects.md`

- [ ] **Step 1: Run all automated verification**

Run:

```powershell
npm.cmd run test
npm.cmd run check
npm.cmd run build
```

Expected: PASS.

- [ ] **Step 2: Start the dev server**

Run:

```powershell
npm.cmd run dev -- --host 127.0.0.1
```

Expected: Vite serves the app, usually at `http://127.0.0.1:1420/`. If port `1420` is occupied because of the Tauri strict port setting, stop the occupying server or run the Tauri app flow already used by this repo.

- [ ] **Step 3: Inspect `/projects` at Ultra HD**

Use Playwright MCP if available. If the MCP transport fails, use the cached Playwright CLI workaround from this session:

```powershell
node -e "const { chromium } = require('C:/Users/Dima/AppData/Local/npm-cache/_npx/9833c18b2d85bc59/node_modules/playwright'); (async () => { const browser = await chromium.launch({ executablePath: 'C:/Users/Dima/AppData/Local/ms-playwright/chromium-1223/chrome-win64/chrome.exe' }); const page = await browser.newPage({ viewport: { width: 2560, height: 1440 } }); await page.goto('http://127.0.0.1:1420/projects'); await page.screenshot({ path: 'artifacts/new-ui-projects-ultrahd.png', fullPage: true }); console.log(await page.title()); await browser.close(); })();"
```

Expected:
- no blank page;
- icon rail, project rail, top command bar, and workspace are visible;
- Sources tab has dense rows and stable grid height;
- Connect from Library opens as a wide working sheet;
- disabled rows and already-connected rows are visually distinct;
- no obvious text overlap at 2560x1440.

- [ ] **Step 4: Inspect smaller desktop fallback**

Run the same script with viewport `{ width: 1366, height: 768 }` and save `artifacts/new-ui-projects-1366.png`.

Expected:
- no catastrophic overlap;
- project rail/workspace remain usable;
- Connect from Library still fits inside the viewport.

- [ ] **Step 5: Record verification notes**

Create `docs/superpowers/verification/2026-06-11-new-ui-research-projects.md`:

```md
# New UI Research Projects Verification

Date: 2026-06-11

## Commands

- `npm.cmd run test`: PASS
- `npm.cmd run check`: PASS
- `npm.cmd run build`: PASS

## Visual QA

- `/projects` at 2560x1440: PASS
- `/projects` at 1366x768: PASS
- `Connect from Library` wide sheet: PASS
- Old `/analysis` reachable: PASS

## Notes

- Unsupported RSS/forum providers are visible but disabled in Connect from Library.
- Telegram/YouTube source-group-backed projects are the only persistable first-slice connect targets.
```

- [ ] **Step 6: Commit verification**

Run:

```powershell
git add docs/superpowers/verification/2026-06-11-new-ui-research-projects.md
git commit -m "docs: verify research projects UI"
```

Expected: commit succeeds.

---

## Self-Review

### Spec Coverage

- New UI starts from scratch at `/projects`: Task 6.
- Old `/analysis` remains functional as fallback: Task 9.
- Ultra HD dense layout with icon rail, project rail, top bar, workspace: Task 6 and Task 10.
- shadcn-svelte used for controls through wrappers: Task 1 and Task 4.
- SVAR used for dense source grids through product wrapper: Task 5, Task 7, Task 8.
- Extractum wrapper layer owns appearance: Task 4 and import-boundary tests.
- Transition adapter maps projects/library to existing source groups/sources: Task 2.
- Connect from Library large sheet with search, provider filters, grid, multiselect, project filters/status panels, selected count, and connect action: Task 8.
- Source sync/job states and active LLM runs are surfaced in library rows and BottomQueue: Task 2, Task 3, and Task 8.
- Persistence only through safe Telegram/YouTube source-group-backed groups: Task 2 and Task 3.
- Unsupported providers visible but disabled: Task 2 and Task 8.
- Both `/projects` and legacy `/analysis` remain reachable from navigation: Task 9.
- Testing strategy includes model, workflow, route contracts, import boundaries, SVAR wrapper contract, check/build, and visual QA: Tasks 1-10.

### Placeholder Scan

- No placeholder markers or unspecified implementation steps remain.
- Each task names exact files, commands, and expected outcomes.
- Code-facing tasks include concrete test or implementation snippets.

### Type Consistency

- `ResearchProjectView`, `LibrarySourceView`, and `ProjectSourceLinkView` are defined in Task 2 and reused consistently in Tasks 3, 7, and 8.
- `SourceJobRecord` is added to Task 2 and Task 3 state/deps so source job status is not invented in components.
- `selectedLibrarySourceIds` is consistently a `Set<string>` in workflow and route state, and component callbacks pass string arrays back to the route.
- `buildSourceGroupUpdateInput` returns existing `UpdateAnalysisSourceGroupInput`, matching `updateAnalysisSourceGroup`.
- Feature components import `extractum-ui`; raw shadcn and SVAR imports are confined to wrapper files.
