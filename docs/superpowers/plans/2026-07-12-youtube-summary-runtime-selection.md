# YouTube Summary Runtime Selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve the user's last YouTube Summary runtime choice locally and make the actual launch provider explicit at submission time.

**Architecture:** A small framework-independent TypeScript helper owns storage keys, value normalization, and failure isolation. `YoutubeSummaryRunDialog` restores those preferences before preflight, persists selector changes, and derives its submit label from the same runtime state sent to the backend.

**Tech Stack:** TypeScript, Svelte 5 runes, browser `localStorage`, Vitest source contracts and unit tests.

## Global Constraints

- Reuse only existing values: runtime `api | gemini_browser`; browser mode `managed | cdp_attach`.
- Use exactly `extractum.youtubeSummary.runtimeProvider` and `extractum.youtubeSummary.browserProviderMode` as storage keys.
- Storage failures are non-fatal and never block dialog open, preflight, or launch.
- Do not add backend settings, migrations, Tauri commands, DTO fields, or value-registry entries.
- Do not persist profile, model override, CDP endpoint, status, run history, or preflight state.
- Do not automatically switch providers based on readiness.
- Follow TDD and keep commits scoped to each task.

---

### Task 1: Runtime preference storage helper

**Files:**
- Create: `src/lib/youtube-summary-runtime-preferences.ts`
- Create: `src/lib/youtube-summary-runtime-preferences.test.ts`

**Interfaces:**
- Produces: `YoutubeSummaryRuntimePreferences`, `loadYoutubeSummaryRuntimePreferences`, `saveYoutubeSummaryRuntimeProvider`, and `saveYoutubeSummaryBrowserProviderMode`.
- Consumes: existing `PromptPackRuntimeProvider` and `GeminiBrowserProviderMode` frontend types plus a minimal `RuntimePreferenceStorage` interface.

- [ ] **Step 1: Write failing unit tests for defaults, valid restoration, malformed values, and storage exceptions.**

```ts
import { describe, expect, it } from "vitest";
import {
  loadYoutubeSummaryRuntimePreferences,
  saveYoutubeSummaryBrowserProviderMode,
  saveYoutubeSummaryRuntimeProvider,
} from "./youtube-summary-runtime-preferences";

function memoryStorage(initial: Record<string, string> = {}) {
  const values = new Map(Object.entries(initial));
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    value: (key: string) => values.get(key),
  };
}

describe("youtube summary runtime preferences", () => {
  it("uses safe defaults without storage", () => {
    expect(loadYoutubeSummaryRuntimePreferences(null)).toEqual({
      runtimeProvider: "api",
      browserProviderMode: "managed",
    });
  });

  it("restores supported values and rejects malformed values", () => {
    expect(loadYoutubeSummaryRuntimePreferences(memoryStorage({
      "extractum.youtubeSummary.runtimeProvider": "gemini_browser",
      "extractum.youtubeSummary.browserProviderMode": "cdp_attach",
    }))).toEqual({ runtimeProvider: "gemini_browser", browserProviderMode: "cdp_attach" });

    expect(loadYoutubeSummaryRuntimePreferences(memoryStorage({
      "extractum.youtubeSummary.runtimeProvider": "automatic",
      "extractum.youtubeSummary.browserProviderMode": "remote",
    }))).toEqual({ runtimeProvider: "api", browserProviderMode: "managed" });
  });

  it("isolates read and write failures", () => {
    const throwing = {
      getItem: () => { throw new Error("blocked"); },
      setItem: () => { throw new Error("blocked"); },
    };
    expect(loadYoutubeSummaryRuntimePreferences(throwing)).toEqual({
      runtimeProvider: "api",
      browserProviderMode: "managed",
    });
    expect(() => saveYoutubeSummaryRuntimeProvider(throwing, "gemini_browser")).not.toThrow();
    expect(() => saveYoutubeSummaryBrowserProviderMode(throwing, "cdp_attach")).not.toThrow();
  });

  it("writes each normalized preference to its scoped key", () => {
    const storage = memoryStorage();
    saveYoutubeSummaryRuntimeProvider(storage, "gemini_browser");
    saveYoutubeSummaryBrowserProviderMode(storage, "cdp_attach");
    expect(storage.value("extractum.youtubeSummary.runtimeProvider")).toBe("gemini_browser");
    expect(storage.value("extractum.youtubeSummary.browserProviderMode")).toBe("cdp_attach");
  });
});
```

- [ ] **Step 2: Run RED.**

Run: `npm.cmd run test -- src/lib/youtube-summary-runtime-preferences.test.ts`
Expected: FAIL with module-resolution error because the helper does not exist.

- [ ] **Step 3: Implement the minimal helper.**

```ts
import type { GeminiBrowserProviderMode } from "$lib/types/gemini-browser";
import type { PromptPackRuntimeProvider } from "$lib/types/prompt-packs";

const RUNTIME_PROVIDER_KEY = "extractum.youtubeSummary.runtimeProvider";
const BROWSER_PROVIDER_MODE_KEY = "extractum.youtubeSummary.browserProviderMode";

export interface RuntimePreferenceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export interface YoutubeSummaryRuntimePreferences {
  runtimeProvider: PromptPackRuntimeProvider;
  browserProviderMode: GeminiBrowserProviderMode;
}

export function loadYoutubeSummaryRuntimePreferences(
  storage: Pick<RuntimePreferenceStorage, "getItem"> | null,
): YoutubeSummaryRuntimePreferences {
  try {
    const runtime = storage?.getItem(RUNTIME_PROVIDER_KEY);
    const mode = storage?.getItem(BROWSER_PROVIDER_MODE_KEY);
    return {
      runtimeProvider: runtime === "gemini_browser" ? "gemini_browser" : "api",
      browserProviderMode: mode === "cdp_attach" ? "cdp_attach" : "managed",
    };
  } catch {
    return { runtimeProvider: "api", browserProviderMode: "managed" };
  }
}

export function saveYoutubeSummaryRuntimeProvider(
  storage: Pick<RuntimePreferenceStorage, "setItem"> | null,
  value: PromptPackRuntimeProvider,
) {
  try { storage?.setItem(RUNTIME_PROVIDER_KEY, value === "gemini_browser" ? value : "api"); } catch {}
}

export function saveYoutubeSummaryBrowserProviderMode(
  storage: Pick<RuntimePreferenceStorage, "setItem"> | null,
  value: GeminiBrowserProviderMode,
) {
  try { storage?.setItem(BROWSER_PROVIDER_MODE_KEY, value === "cdp_attach" ? value : "managed"); } catch {}
}
```

- [ ] **Step 4: Run GREEN and TypeScript check.**

Run: `npm.cmd run test -- src/lib/youtube-summary-runtime-preferences.test.ts`
Run: `npm.cmd run check`
Expected: PASS with no Svelte/TypeScript diagnostics.

- [ ] **Step 5: Commit.**

```powershell
git add src/lib/youtube-summary-runtime-preferences.ts src/lib/youtube-summary-runtime-preferences.test.ts
git commit -m "feat: persist youtube summary runtime preference"
```

### Task 2: Dialog restoration, explicit CTA, and verification docs

**Files:**
- Modify: `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte`
- Modify: `src/lib/youtube-summary-launch-contract.test.ts`
- Modify: `docs/project.md`
- Inspect: `docs/value-registry.md`

**Interfaces:**
- Consumes: all four exports from `src/lib/youtube-summary-runtime-preferences.ts` and existing dialog runtime/browser types.
- Produces: restored runtime state before preflight, persisted selector changes, and provider-specific launch labels.

- [ ] **Step 1: Extend the source contract with failing assertions.**

Add a focused test to `src/lib/youtube-summary-launch-contract.test.ts`:

```ts
it("restores runtime preferences before preflight and names the submitted provider", () => {
  const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");

  expect(dialog).toContain("loadYoutubeSummaryRuntimePreferences");
  expect(dialog).toContain("saveYoutubeSummaryRuntimeProvider");
  expect(dialog).toContain("saveYoutubeSummaryBrowserProviderMode");
  expect(dialog).toContain('runtimeProvider === "gemini_browser" ? "Run via Gemini Browser" : "Run via API"');
  expect(dialog).not.toContain('runtimeProvider = "api";');

  const openEffect = dialog.slice(dialog.indexOf("$effect(() =>"), dialog.indexOf("async function loadProfiles"));
  expect(openEffect.indexOf("loadYoutubeSummaryRuntimePreferences"))
    .toBeLessThan(openEffect.indexOf("runPreflight"));
});
```

- [ ] **Step 2: Run RED.**

Run: `npm.cmd run test -- src/lib/youtube-summary-launch-contract.test.ts`
Expected: FAIL because the dialog still resets to API and has a generic `Start` label.

- [ ] **Step 3: Import the helper, add safe storage access, and derive the CTA label.**

```ts
import {
  loadYoutubeSummaryRuntimePreferences,
  saveYoutubeSummaryBrowserProviderMode,
  saveYoutubeSummaryRuntimeProvider,
} from "$lib/youtube-summary-runtime-preferences";

const runButtonLabel = $derived(
  runtimeProvider === "gemini_browser" ? "Run via Gemini Browser" : "Run via API",
);

function runtimePreferenceStorage() {
  return typeof localStorage === "undefined" ? null : localStorage;
}
```

- [ ] **Step 4: Restore preferences before status refresh and preflight.**

Replace the runtime/mode resets in the open effect with:

```ts
const preferences = loadYoutubeSummaryRuntimePreferences(runtimePreferenceStorage());
runtimeProvider = preferences.runtimeProvider;
browserProviderMode = preferences.browserProviderMode;
browserStatus = null;
browserRuns = [];
browserStatusLoadError = null;
preflight = null;
error = null;
includeComments = false;
void loadProfiles();
if (runtimeProvider === "gemini_browser") void refreshBrowserStatus();
if (source) queueMicrotask(() => void runPreflight());
```

Keep existing output language, summary mode, and CDP endpoint initialization unchanged.

- [ ] **Step 5: Persist runtime and browser-mode changes before async refreshes.**

```ts
function handleRuntimeChange(event: Event) {
  runtimeProvider = (event.currentTarget as HTMLSelectElement).value as PromptPackRuntimeProvider;
  saveYoutubeSummaryRuntimeProvider(runtimePreferenceStorage(), runtimeProvider);
  if (runtimeProvider === "gemini_browser") void refreshBrowserStatus();
  void runPreflight();
}

function handleBrowserModeChange(event: Event) {
  browserProviderMode = (event.currentTarget as HTMLSelectElement).value as GeminiBrowserProviderMode;
  saveYoutubeSummaryBrowserProviderMode(runtimePreferenceStorage(), browserProviderMode);
  void refreshBrowserStatus();
  void runPreflight();
}
```

Wire the browser-mode select to `onchange={handleBrowserModeChange}`.

- [ ] **Step 6: Replace the generic submit copy.**

```svelte
<ExtractumButton
  type="submit"
  disabled={!source || loading || !canStartYoutubeSummary(preflight) || Boolean(browserRuntimeBlockingCheck)}
>
  {runButtonLabel}
</ExtractumButton>
```

- [ ] **Step 7: Run focused GREEN checks.**

Run: `npm.cmd run test -- src/lib/youtube-summary-runtime-preferences.test.ts src/lib/youtube-summary-launch-contract.test.ts src/lib/api/prompt-packs.test.ts`
Run: `npm.cmd run check`
Expected: PASS; API-wrapper tests still prove both runtime variants are forwarded unchanged.

- [ ] **Step 8: Update current-state documentation and inspect controlled values.**

Add to the Prompt Pack/YouTube Summary description in `docs/project.md`:

```markdown
- YouTube Summary remembers the last local API/Gemini Browser runtime and browser mode, and its launch button names the provider that will receive the run.
```

Inspect `docs/value-registry.md`; do not modify it because no new status/state/kind/mode/provider value was introduced.

- [ ] **Step 9: Run full verification.**

Run: `npm.cmd run test`
Run: `npm.cmd run check`
Run: `git diff --check`
Expected: 149 existing test files plus the new helper test file pass; Svelte check reports zero errors/warnings; diff check passes.

- [ ] **Step 10: Commit.**

```powershell
git add src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte src/lib/youtube-summary-launch-contract.test.ts docs/project.md
git commit -m "fix: preserve youtube summary runtime selection"
```

## Completion Evidence

The implementation is complete only when both commits exist, the full frontend suite is green, the dialog reopens with the last valid runtime/mode, malformed storage falls back to API/managed, and the launch button label matches the `runtimeProvider` submitted to preflight and start commands.
