# Daily Development Loop Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Halve the measured full Vitest feedback time, add reliable focused frontend/Rust commands, and reduce ordinary Rust dev/test artifact cost without weakening the full verification gate or deleting Cargo artifacts without explicit approval.

**Architecture:** Export one pure Vitest settings object from `vite.config.js`, make the Vitest wrapper safely importable, and expose changed/related/focused scripts through `package.json`. Land and verify that frontend checkpoint before changing Cargo; then ask separately about one-time cache cleanup, apply the dev-profile policy, warm only the canonical target, and record machine-specific evidence in a verification document.

**Tech Stack:** SvelteKit 2, Vite 6, Vitest 4.1.5, Node.js ESM, TypeScript, Cargo/Rust 2021, PowerShell on Windows.

## Global Constraints

- Implement the final approved revision of `docs/superpowers/specs/2026-07-14-daily-development-loop-performance-design.md`, committed as `92239f1e`, or a descendant of that commit.
- Use `npm.cmd`, not plain `npm`, for every npm-script command on Windows.
- Keep `npm.cmd run verify` authoritative; do not remove, skip, weaken, or replace any step in `scripts/verify.mjs`.
- Keep Vitest isolation enabled, retain per-file environments, and do not set `maxWorkers`, file-level concurrency, or a global DOM environment.
- Use `pool: "threads"` with automatic worker selection.
- Keep ordinary Rust commands on the canonical `src-tauri/target`; do not add a `--target-dir` or create a new `codex-*` target.
- Do not rewrite archived plans or historical verification evidence that contains old target-directory commands.
- Do not add dependencies, upgrade packages, split Vitest projects, install/configure `sccache`, change the linker, introduce `cargo-nextest`, or split the Rust crate.
- Do not change release profiles, debug assertions, Tauri MCP behavior, application behavior, runtime values, UI values, or `docs/value-registry.md`.
- Do not run any Cargo command after editing the dev profile until the user has explicitly approved or declined the one-time cleanup and the selected cleanup branch is complete.
- Never run `cargo clean` without a separate explicit user approval during Task 3.
- Preserve unrelated user changes; each task begins by inspecting the worktree and stages only its listed files.

---

### Task 1: Frontend Feedback-Loop Checkpoint

**Files:**
- Modify: `scripts/run-vitest.test.ts:1-22`
- Modify: `scripts/run-vitest.mjs:1-22`
- Modify: `vite.config.js:1-35`
- Modify: `package.json:6-40`
- Create: `src/lib/development-loop-performance-contract.test.ts`

**Interfaces:**
- Produces: `normalizeRelatedFileArgs(args: string[], cwd?: string): string[]`, exported from `scripts/run-vitest.mjs` without executing Vitest when imported.
- Produces: `VITEST_TEST_CONFIG`, a pure named export whose `pool` is exactly `"threads"` and which has no own `maxWorkers` property.
- Produces: `test:changed`, `test:changed:last`, `test:related`, and `test:rust` package scripts; canonicalizes the existing `test:rust:prompt-pack-runs` script.
- Preserves: the existing research-adapter exclusion, watch wrapper, conditional `svelteTesting()` plugin, Vite/Tauri server configuration, and full test inventory.

- [ ] **Step 1: Verify the clean approved baseline and preflight thread compatibility**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git merge-base --is-ancestor 92239f1e HEAD
$approvedSpecPresent = $LASTEXITCODE -eq 0
$newContractExists = Test-Path 'src/lib/development-loop-performance-contract.test.ts'
$separateVitestConfigs = @(
    'vitest.config.js', 'vitest.config.ts', 'vitest.config.mjs',
    'vitest.config.mts', 'vitest.config.cjs', 'vitest.config.cts'
) | Where-Object { Test-Path $_ }
$testFiles = @(rg --files src scripts sidecars research | Where-Object {
    $_ -match '\.(test|spec)\.(js|jsx|ts|tsx|mjs|mts|cjs|cts)$'
})
$chdirViolations = if ($testFiles.Count -gt 0) {
    @(Select-String -Path $testFiles -Pattern 'process\.chdir\s*\(')
} else { @() }
$envWriteViolations = if ($testFiles.Count -gt 0) {
    @(Select-String -Path $testFiles -Pattern '(delete\s+process\.env(?:\.|\[)|process\.env(?:\.[A-Za-z_$][\w$]*|\[[^\]]+\])\s*(?:=|\+=|-=|\*=|/=|\?\?=|\|\|=|&&=))')
} else { @() }
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_PRESENT=$approvedSpecPresent"
"CONTRACT_EXISTS=$newContractExists"
"SEPARATE_VITEST_CONFIG_COUNT=$($separateVitestConfigs.Count)"
"TEST_FILE_COUNT=$($testFiles.Count)"
"CHDIR_VIOLATION_COUNT=$($chdirViolations.Count)"
"ENV_WRITE_VIOLATION_COUNT=$($envWriteViolations.Count)"
if (
    $status.Count -ne 0 -or
    -not $approvedSpecPresent -or
    $newContractExists -or
    $separateVitestConfigs.Count -ne 0 -or
    $chdirViolations.Count -ne 0 -or
    $envWriteViolations.Count -ne 0
) { exit 1 }
```

Expected: clean tree; approved spec is an ancestor; no new contract or separate Vitest config exists; test inventory is nonzero; no executable `process.chdir()` or direct `process.env` mutation is found in tests.

- [ ] **Step 2: Add a failing raw-source test for an import-safe wrapper**

In `scripts/run-vitest.test.ts`, add this test inside the existing `describe` block:

```ts
  it("can be imported without starting Vitest", () => {
    expect(runnerSource).toContain('import { pathToFileURL } from "node:url"');
    expect(runnerSource).toMatch(
      /if \(process\.argv\[1\] && import\.meta\.url === pathToFileURL\(process\.argv\[1\]\)\.href\) \{\s*runVitest\(\);\s*\}/s,
    );
  });
```

- [ ] **Step 3: Run the wrapper test to verify RED**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/run-vitest.test.ts
```

Expected: FAIL in `can be imported without starting Vitest` because the wrapper has no guarded entrypoint.

- [ ] **Step 4: Introduce the guarded entrypoint without changing arguments**

Replace `scripts/run-vitest.mjs` with:

```js
import { spawnSync } from "node:child_process";
import { realpathSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

const DEFAULT_EXCLUDES = ["research/gemini_browser_adapter/tests/**"];

function runVitest() {
  const realCwd = realpathSync.native(process.cwd());
  process.chdir(realCwd);

  const defaultExcludeArgs = DEFAULT_EXCLUDES.flatMap((glob) => ["--exclude", glob]);
  const vitestCli = path.join(realCwd, "node_modules", "vitest", "vitest.mjs");
  const result = spawnSync(
    process.execPath,
    [vitestCli, ...process.argv.slice(2), ...defaultExcludeArgs],
    {
      cwd: realCwd,
      env: process.env,
      stdio: "inherit",
    },
  );

  if (result.error) {
    throw result.error;
  }

  process.exit(result.status ?? 1);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runVitest();
}
```

- [ ] **Step 5: Run the wrapper test to verify the guard is GREEN**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/run-vitest.test.ts
```

Expected: PASS; the adapter-exclusion and watch-script tests also remain green.

- [ ] **Step 6: Add the failing frontend configuration and wrapper contract**

Create `src/lib/development-loop-performance-contract.test.ts`:

```ts
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { normalizeRelatedFileArgs } from "../../scripts/run-vitest.mjs";
import { VITEST_TEST_CONFIG } from "../../vite.config.js";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const readSource = (relativePath: string) =>
  readFileSync(path.join(repoRoot, relativePath), "utf8").replace(/\r\n/g, "\n");
const packageJson = JSON.parse(readSource("package.json")) as {
  scripts: Record<string, string>;
};

describe("daily development loop configuration", () => {
  it("uses adaptive Vitest threads through one owned config object", () => {
    expect(VITEST_TEST_CONFIG.pool).toBe("threads");
    expect(Object.prototype.hasOwnProperty.call(VITEST_TEST_CONFIG, "maxWorkers")).toBe(false);
    expect(readSource("vite.config.js")).toMatch(/\btest:\s*VITEST_TEST_CONFIG\b/);
  });

  it("has no separate root Vitest config", () => {
    for (const extension of ["js", "ts", "mjs", "mts", "cjs", "cts"]) {
      expect(existsSync(path.join(repoRoot, `vitest.config.${extension}`))).toBe(false);
    }
  });

  it("owns the focused package scripts and canonical Rust target", () => {
    expect(packageJson.scripts["test:changed"]).toBe(
      "node scripts/run-vitest.mjs run --changed",
    );
    expect(packageJson.scripts["test:changed:last"]).toBe(
      "node scripts/run-vitest.mjs run --changed=HEAD~1",
    );
    expect(packageJson.scripts["test:related"]).toBe(
      "node scripts/run-vitest.mjs related --run",
    );
    expect(packageJson.scripts["test:rust"]).toBe(
      "cargo test --manifest-path src-tauri/Cargo.toml --lib",
    );
    expect(packageJson.scripts["test:rust:prompt-pack-runs"]).toBe(
      "cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_pack_run",
    );
    expect(packageJson.scripts["test:rust"]).not.toContain("--target-dir");
    expect(packageJson.scripts["test:rust:prompt-pack-runs"]).not.toContain("--target-dir");
  });
});

describe("related-test path normalization", () => {
  const windowsPath = "src\\lib\\api\\llm.ts";
  const portablePath = "src/lib/api/llm.ts";

  it("normalizes an existing related operand", () => {
    expect(normalizeRelatedFileArgs(["related", windowsPath], repoRoot)).toEqual([
      "related",
      portablePath,
    ]);
  });

  it("leaves options and non-file patterns unchanged", () => {
    expect(normalizeRelatedFileArgs(["related", "-t", "foo\\bar"], repoRoot)).toEqual([
      "related",
      "-t",
      "foo\\bar",
    ]);
  });

  it("leaves a missing operand unchanged", () => {
    expect(
      normalizeRelatedFileArgs(["related", "src\\lib\\missing-file.ts"], repoRoot),
    ).toEqual(["related", "src\\lib\\missing-file.ts"]);
  });

  it("also normalizes an existing path-valued flag argument", () => {
    expect(
      normalizeRelatedFileArgs(["related", "--config", windowsPath], repoRoot),
    ).toEqual(["related", "--config", portablePath]);
  });

  it("does not normalize operands for other Vitest commands", () => {
    expect(normalizeRelatedFileArgs(["run", windowsPath], repoRoot)).toEqual([
      "run",
      windowsPath,
    ]);
  });
});
```

- [ ] **Step 7: Run the new contract to verify RED**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/development-loop-performance-contract.test.ts
```

Expected: FAIL during module loading because `normalizeRelatedFileArgs` and `VITEST_TEST_CONFIG` are not exported yet. The wrapper must not recursively start Vitest.

- [ ] **Step 8: Export the pure Vitest config and assign it to the Vite factory**

In `vite.config.js`, add this block after the imports:

```js
/** @type {{ pool: "threads" }} */
export const VITEST_TEST_CONFIG = {
  pool: "threads",
};
```

Add this property to the object passed through `defineConfig`, immediately after `plugins`:

```js
  test: VITEST_TEST_CONFIG,
```

Do not call the default async config factory from the contract and do not move or duplicate any plugin factory.

- [ ] **Step 9: Add argument normalization to the import-safe wrapper**

Replace `scripts/run-vitest.mjs` with:

```js
import { spawnSync } from "node:child_process";
import { existsSync, realpathSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

const DEFAULT_EXCLUDES = ["research/gemini_browser_adapter/tests/**"];

/**
 * @param {string[]} args
 * @param {string} [cwd]
 * @returns {string[]}
 */
export function normalizeRelatedFileArgs(args, cwd = process.cwd()) {
  if (args[0] !== "related") {
    return [...args];
  }

  return args.map((arg, index) => {
    if (index === 0 || arg.startsWith("-")) {
      return arg;
    }

    const normalized = arg.replaceAll("\\", "/");
    return existsSync(path.resolve(cwd, normalized)) ? normalized : arg;
  });
}

function runVitest() {
  const realCwd = realpathSync.native(process.cwd());
  process.chdir(realCwd);

  const defaultExcludeArgs = DEFAULT_EXCLUDES.flatMap((glob) => ["--exclude", glob]);
  const vitestCli = path.join(realCwd, "node_modules", "vitest", "vitest.mjs");
  const args = normalizeRelatedFileArgs(process.argv.slice(2), realCwd);
  const result = spawnSync(process.execPath, [vitestCli, ...args, ...defaultExcludeArgs], {
    cwd: realCwd,
    env: process.env,
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }

  process.exit(result.status ?? 1);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runVitest();
}
```

The classifier intentionally normalizes any non-option argument that resolves to a file, including a file-valued option argument; it never rewrites the option token itself.
Because importing the wrapper from a TypeScript test brings this `.mjs` file
under the repository's strict `checkJs` analysis, resolve any wrapper type
diagnostic with narrow JSDoc parameter/return annotations. Do not disable
`checkJs`, add `@ts-ignore`, or weaken the project TypeScript settings.

- [ ] **Step 10: Add the focused package scripts and remove the slice-specific target**

In `package.json`, keep the existing `test` script and add these adjacent scripts:

```json
"test:changed": "node scripts/run-vitest.mjs run --changed",
"test:changed:last": "node scripts/run-vitest.mjs run --changed=HEAD~1",
"test:related": "node scripts/run-vitest.mjs related --run",
"test:rust": "cargo test --manifest-path src-tauri/Cargo.toml --lib",
```

Replace only the existing `test:rust:prompt-pack-runs` value with:

```json
"test:rust:prompt-pack-runs": "cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_pack_run"
```

- [ ] **Step 11: Run focused GREEN checks**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/run-vitest.test.ts src/lib/development-loop-performance-contract.test.ts
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
npm.cmd run check
```

Expected: both test files pass; Svelte/TypeScript check exits 0; importing the wrapper does not spawn a nested Vitest process.

- [ ] **Step 12: Run one complete frontend suite before the checkpoint**

Run:

```powershell
npm.cmd run test
```

Expected: PASS with at least the pre-change inventory of 156 files and 1,253 tests, plus the new contract tests. Runtime should be materially below the 130.49-second same-machine forks baseline; the observed 60-70-second range is informational, not a hard gate.

- [ ] **Step 13: Review and commit the frontend checkpoint**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$changed = @(git status --short)
$changed
$allowed = @(
    'package.json',
    'scripts/run-vitest.mjs',
    'scripts/run-vitest.test.ts',
    'src/lib/development-loop-performance-contract.test.ts',
    'vite.config.js'
)
$paths = @(
    git diff --name-only
    git diff --cached --name-only
    git ls-files --others --exclude-standard
) | Sort-Object -Unique
$paths = @($paths)
if (@($paths | Where-Object { $_ -notin $allowed }).Count -ne 0) { exit 1 }
```

Expected: only the five listed files changed and `git diff --check` exits 0.

Commit:

```powershell
git add -- package.json scripts/run-vitest.mjs scripts/run-vitest.test.ts src/lib/development-loop-performance-contract.test.ts vite.config.js
git diff --cached --check
git commit -m "perf: speed up frontend feedback loop"
```

---

### Task 2: Verify the Clean Frontend Checkpoint

**Files:**
- Create: `docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md`
- Temporary probe only, restore before writing evidence: `scripts/tauri.mjs`

**Interfaces:**
- Consumes: the clean Task 1 checkpoint and all four new package scripts.
- Produces: measured frontend evidence, including three complete suite durations, the clean-tree result, the expected force-rerun result, and focused changed/related behavior.
- Preserves: `scripts/tauri.mjs` byte-for-byte after the reversible probe.

- [ ] **Step 1: Confirm the checkpoint is clean and is the expected commit**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
$subject = git log -1 --pretty=%s
"STATUS_COUNT=$($status.Count)"
"HEAD_SUBJECT=$subject"
if ($status.Count -ne 0 -or $subject -ne 'perf: speed up frontend feedback loop') { exit 1 }
```

Expected: clean tree and Task 1 is HEAD.

- [ ] **Step 2: Verify the clean-tree changed command**

Run:

```powershell
$watch = [Diagnostics.Stopwatch]::StartNew()
npm.cmd run test:changed
$code = $LASTEXITCODE
$watch.Stop()
"TEST_CHANGED_CLEAN_EXIT=$code"
"TEST_CHANGED_CLEAN_SECONDS=$([math]::Round($watch.Elapsed.TotalSeconds, 2))"
if ($code -ne 0) { exit $code }
```

Expected: exit 0 with `No test files found`. This is the locked-down Vitest 4.1.5 behavior for a clean working tree.

- [ ] **Step 3: Verify the last-checkpoint force rerun**

Run:

```powershell
$watch = [Diagnostics.Stopwatch]::StartNew()
npm.cmd run test:changed:last
$code = $LASTEXITCODE
$watch.Stop()
"TEST_CHANGED_LAST_EXIT=$code"
"TEST_CHANGED_LAST_SECONDS=$([math]::Round($watch.Elapsed.TotalSeconds, 2))"
if ($code -ne 0) { exit $code }
```

Expected: PASS and the full suite runs. Task 1 changed `vite.config.js` and `package.json`, both default `forceRerunTriggers`; a full run is correct here and is not evidence that `--changed` is broken.

- [ ] **Step 4: Prove a normal dirty source produces a focused nonempty set**

Use the executor's patch editor to add exactly this first-line probe to `scripts/tauri.mjs`:

```js
// Vitest changed-set verification probe; remove immediately after the command.
```

Run:

```powershell
npm.cmd run test:changed
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: PASS with a nonzero related-test set containing `scripts/tauri.test.ts`.
This leaf script is not a Vitest force-rerun trigger, so a complete-suite run
would be unexpected and must be investigated rather than accepted as focused
selection.

Remove exactly the probe line with the patch editor, then run:

```powershell
git diff --exit-code -- scripts/tauri.mjs
```

Expected: exit 0; the probe file is restored byte-for-byte.

- [ ] **Step 5: Verify explicit related selection and Windows-path normalization**

Run:

```powershell
$watch = [Diagnostics.Stopwatch]::StartNew()
npm.cmd run test:related -- src\lib\api\llm.ts
$code = $LASTEXITCODE
$watch.Stop()
"TEST_RELATED_EXIT=$code"
"TEST_RELATED_SECONDS=$([math]::Round($watch.Elapsed.TotalSeconds, 2))"
if ($code -ne 0) { exit $code }
```

Expected: PASS with a nonzero related set; the wrapper accepts the PowerShell-style backslash path.

- [ ] **Step 6: Run the committed full frontend suite three times**

Run:

```powershell
$durations = @()
1..3 | ForEach-Object {
    $run = $_
    $watch = [Diagnostics.Stopwatch]::StartNew()
    npm.cmd run test
    $code = $LASTEXITCODE
    $watch.Stop()
    $seconds = [math]::Round($watch.Elapsed.TotalSeconds, 2)
    "FULL_VITEST_RUN_${run}_EXIT=$code"
    "FULL_VITEST_RUN_${run}_SECONDS=$seconds"
    if ($code -ne 0) { exit $code }
    $durations += $seconds
}
$median = ($durations | Sort-Object)[1]
"FULL_VITEST_MEDIAN_SECONDS=$median"
```

Expected: all three runs pass; each inventory is at least 156 files / 1,253 tests plus additions from Task 1; the median is materially below 130.49 seconds. Do not fail merely because the median is outside the contextual 60-70-second range.
If the median is not materially below the same-machine baseline, record the
result and stop for investigation instead of turning the timing into a flaky
shell assertion or claiming acceptance.

- [ ] **Step 7: Create the verification record with actual frontend observations**

Create `docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md` with these sections and the literal values printed in Steps 2-6; do not write placeholder tokens:

```markdown
# Daily Development Loop Performance Verification

## Scope

- Approved design: `docs/superpowers/specs/2026-07-14-daily-development-loop-performance-design.md`
- Implementation plan: `docs/superpowers/plans/2026-07-14-daily-development-loop-performance.md`
- Baseline machine: the same Windows workstation used for the approved design measurements

## Pre-change Baseline

| Operation | Seconds | Inventory |
| --- | ---: | --- |
| Full Vitest, forks | 130.49 | 156 files / 1,253 tests |
| Full Vitest, threads auto probe | 65.09 | 156 files / 1,253 tests |
| No-op cargo check | 1.15 | canonical target |
| No-op cargo test | 22.72 | tests 18.83 s |

## Frontend Checkpoint

Record the clean changed result, the expected full `test:changed:last` result,
the dirty-source focused result, the explicit related result, all three full
suite durations and inventories, and their median using the observed numeric
values from this execution.

## Cargo Profile and Cache

Status at this checkpoint: the Cargo profile has not been changed, the cleanup
decision has not been requested, and no new-profile Cargo command has run.

## Final Gates

Status at this checkpoint: final mixed frontend/Rust gates have not run because
the Cargo-profile task is still pending.
```

Replace the instruction paragraphs under `Frontend Checkpoint` with a concrete Markdown table and concise factual notes before saving. Keep the Cargo and final-gate instruction paragraphs until their owning tasks replace them with observations.

- [ ] **Step 8: Commit frontend verification evidence**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$changed = @(git status --short)
$changed
$paths = @(
    git diff --name-only
    git diff --cached --name-only
    git ls-files --others --exclude-standard
) | Sort-Object -Unique
$paths = @($paths)
if ($paths.Count -ne 1 -or $paths[0] -ne 'docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md') { exit 1 }
git add -- docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md
git diff --cached --check
git commit -m "docs: record frontend loop performance"
```

Expected: the probe source is absent from the diff and only the new evidence document is committed.

---

### Task 3: Cargo Profile, Canonical Target, and Workflow Documentation

**Files:**
- Modify: `src/lib/development-loop-performance-contract.test.ts`
- Modify: `src-tauri/Cargo.toml:52-56`
- Modify: `AGENTS.md:39-45`
- Modify: `docs/project.md:13-40`
- Modify: `docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md`

**Interfaces:**
- Produces: `[profile.dev] debug = "line-tables-only"` and `[profile.dev.package."*"] debug = false`.
- Produces: stable `<!-- daily-development-loop -->` documentation anchors and the subsystem inner-loop/canonical-target/native-debug guidance.
- Consumes: `test:rust` and the canonical `src-tauri/target` from Task 1.
- Preserves: release settings, Tauri/MCP behavior, debug assertions, the dependency graph, and all runtime behavior.

- [ ] **Step 1: Stop and request the cleanup decision**

Confirm the tree is clean, then ask the user exactly whether the one-time `cargo clean --manifest-path src-tauri/Cargo.toml` is approved for this execution. Do not edit `src-tauri/Cargo.toml` and do not run Cargo until the user explicitly answers **approved** or **declined**. Record that decision for the verification document.

- [ ] **Step 2: Add failing Cargo-profile and documentation contract cases**

Append these tests to the `daily development loop configuration` describe block in `src/lib/development-loop-performance-contract.test.ts`:

```ts
  it("uses reduced dev debug information without a custom target", () => {
    const cargoToml = readSource("src-tauri/Cargo.toml");
    expect(cargoToml).toMatch(
      /\[profile\.dev\]\s*\ndebug = "line-tables-only"/,
    );
    expect(cargoToml).toMatch(
      /\[profile\.dev\.package\."\*"\]\s*\ndebug = false/,
    );
  });

  it("keeps stable daily-loop documentation anchors", () => {
    expect(readSource("AGENTS.md")).toContain("<!-- daily-development-loop -->");
    expect(readSource("docs/project.md")).toContain("<!-- daily-development-loop -->");
  });
```

- [ ] **Step 3: Run the focused contract to verify RED without invoking Cargo**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/development-loop-performance-contract.test.ts
```

Expected: FAIL only for the missing profile blocks and documentation anchors. This Node/Vitest command does not compile or warm the Cargo profile.

- [ ] **Step 4: Add the permanent Cargo dev profile**

In `src-tauri/Cargo.toml`, immediately after `[dev-dependencies]`, add:

```toml
[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false
```

Do not add `[profile.test]`, release settings, environment-variable workarounds, or a target directory.

- [ ] **Step 5: Document the agent workflow with a stable anchor**

In `AGENTS.md`, add this block under `## 4. Validation Rules`:

```markdown
<!-- daily-development-loop -->
- Use the smallest relevant daily-loop command after a small change:
  - dirty frontend work: `npm.cmd run test:changed`;
  - the most recent linear checkpoint: `npm.cmd run test:changed:last`;
  - a known frontend source: `npm.cmd run test:related -- <forward-or-backslash-path>`;
  - focused Rust library tests: `npm.cmd run test:rust -- <test-filter>`;
  - broad Svelte/TypeScript work: also run `npm.cmd run check`;
  - Rust/Tauri work: also run `npm.cmd run check:rustfmt` and `cargo check --manifest-path src-tauri/Cargo.toml`.
- Changed/related commands are accelerators, not merge gates. An empty or unexpectedly small selection requires an explicit test or a wider run; `npm.cmd run verify` remains the full gate.
- Ordinary Cargo commands must share canonical `src-tauri/target`; do not create slice-specific `codex-*` targets for sequential work.
- For rare dependency-level native debugging, start clean, temporarily set both `[profile.dev] debug` and `[profile.dev.package."*"] debug` to `2`, set `CARGO_TARGET_DIR` to an absolute isolated native-debug directory, run `npm.cmd run tauri dev`, then restore the manifest. Never commit the temporary profile edit.
```

- [ ] **Step 6: Document the public daily workflow and native-debug escape hatch**

In `docs/project.md`, insert this block after the introductory full-verification paragraph:

First replace the primary Windows command in the existing baseline block:

```markdown
npm.cmd run verify
```

Then insert:

````markdown
<!-- daily-development-loop -->
For the daily loop after a small change, choose the narrowest applicable command:

```powershell
npm.cmd run test:changed
npm.cmd run test:changed:last
npm.cmd run test:related -- src/lib/some-model.ts
npm.cmd run test:rust -- prompt_packs::runtime::tests::load_run_runtime_config
```

The working-tree command sees uncommitted changes; the last-checkpoint command
uses `HEAD~1`, which means the first parent after a merge. Use
`npm.cmd run test -- --changed=<base>` when a different merge base is intended.
Changed/related selection follows the module graph and may be empty or
incomplete for dynamic relationships, so it is not a replacement for the full
`npm.cmd run verify` gate.

Normal Rust checks and tests share `src-tauri/target`; avoid per-task target
directories during sequential development. Ordinary dev/test builds retain
workspace line tables and omit dependency debug information. For rare native
inspection of dependency variables, begin with a clean tree, temporarily set
both `[profile.dev] debug` and `[profile.dev.package."*"] debug` to `2`, point
`CARGO_TARGET_DIR` at an absolute isolated directory, and launch the usual
MCP-enabled `npm.cmd run tauri dev`. Restore the manifest afterward and never
commit that temporary profile change.
````

Use four backticks for the outer plan fence when copying so the nested PowerShell fence remains ordinary Markdown in `docs/project.md`.

- [ ] **Step 7: Verify the source contract is GREEN before any Cargo invocation**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/development-loop-performance-contract.test.ts
```

Expected: PASS. No Cargo command has run since the profile edit.

- [ ] **Step 8: Record the pre-clean target state**

Run:

```powershell
$target = 'src-tauri/target'
$beforeBytes = if (Test-Path $target) {
    (Get-ChildItem -LiteralPath $target -Recurse -File -ErrorAction SilentlyContinue |
        Measure-Object -Property Length -Sum).Sum
} else { 0 }
$beforeGiB = [math]::Round($beforeBytes / 1GB, 2)
$codexTargets = @(Get-ChildItem -LiteralPath $target -Directory -Filter 'codex-*' -ErrorAction SilentlyContinue)
"TARGET_BEFORE_GIB=$beforeGiB"
"CODEX_TARGET_COUNT_BEFORE=$($codexTargets.Count)"
```

Expected: the current size and historical `codex-*` count are printed for evidence. Their existence is not a failure because this slice does not automatically delete individual historical directories.

- [ ] **Step 9: Execute exactly the approved cleanup branch**

If cleanup was **approved**, first close the Rust workspace/editor or disable
rust-analyzer so it cannot start a new Cargo check between the process scan and
cleanup. Then run the safety check and cleanup in the same PowerShell block:

```powershell
$blocking = @(Get-Process cargo, rustc, rust-analyzer, extractum -ErrorAction SilentlyContinue)
$tauriNodeProcesses = @(
    Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Name -eq 'node.exe' -and
            $_.CommandLine -match '(scripts[\\/]tauri\.mjs|tauri(?:\.js)?\s+dev)'
        }
)
$blocking | Select-Object Id, ProcessName, Path
$tauriNodeProcesses | Select-Object ProcessId, Name, CommandLine
if ($blocking.Count -ne 0 -or $tauriNodeProcesses.Count -ne 0) { exit 1 }
cargo clean --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: no Cargo, rustc, rust-analyzer, Extractum, or Tauri-dev process is
running and cleanup succeeds. Stop and resolve any listed process before
cleanup; do not force-stop it merely to continue the plan. If cleanup loses a
race and fails, re-establish process safety before retrying under the same
explicit approval.

If cleanup was **declined**, skip `cargo clean` completely and record `declined`; proceed directly to Step 10. Do not revisit cleanup after the new profile has warmed the target.

- [ ] **Step 10: Warm the selected profile once and capture Cargo timings**

Run:

```powershell
$watch = [Diagnostics.Stopwatch]::StartNew()
cargo check --manifest-path src-tauri/Cargo.toml --timings
$code = $LASTEXITCODE
$watch.Stop()
"FIRST_PROFILE_CARGO_CHECK_EXIT=$code"
"FIRST_PROFILE_CARGO_CHECK_SECONDS=$([math]::Round($watch.Elapsed.TotalSeconds, 2))"
if ($code -ne 0) { exit $code }
$timingReport = Get-ChildItem -LiteralPath 'src-tauri/target/cargo-timings' -Filter 'cargo-timing*.html' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
"CARGO_TIMING_REPORT=$($timingReport.FullName)"
if (-not $timingReport) { exit 1 }
```

Expected: Cargo check passes and prints the generated timing report path. Preserve the report under the canonical target for later inspection.

- [ ] **Step 11: Measure the first full Cargo test under the profile**

Run:

```powershell
$watch = [Diagnostics.Stopwatch]::StartNew()
cargo test --manifest-path src-tauri/Cargo.toml
$code = $LASTEXITCODE
$watch.Stop()
"FIRST_PROFILE_CARGO_TEST_EXIT=$code"
"FIRST_PROFILE_CARGO_TEST_SECONDS=$([math]::Round($watch.Elapsed.TotalSeconds, 2))"
if ($code -ne 0) { exit $code }
```

Expected: full Cargo test passes. Record separately the wall time and the test-execution duration printed by Cargo.

- [ ] **Step 12: Verify the focused Rust wrapper and no-op timings**

Run:

```powershell
$focusedOutput = @(
    cmd.exe /d /s /c "npm.cmd run test:rust -- prompt_packs::runtime::tests::load_run_runtime_config 2>&1"
)
$focusedCode = $LASTEXITCODE
$focusedOutput | ForEach-Object { Write-Host $_ }
$focusedText = $focusedOutput -join "`n"
if (
    $focusedCode -ne 0 -or
    $focusedText -notmatch 'test result: ok\. [1-9][0-9]* passed;'
) { exit 1 }

$checkWatch = [Diagnostics.Stopwatch]::StartNew()
cargo check --manifest-path src-tauri/Cargo.toml
$checkCode = $LASTEXITCODE
$checkWatch.Stop()
"NOOP_CARGO_CHECK_EXIT=$checkCode"
"NOOP_CARGO_CHECK_SECONDS=$([math]::Round($checkWatch.Elapsed.TotalSeconds, 2))"
if ($checkCode -ne 0) { exit $checkCode }

$testWatch = [Diagnostics.Stopwatch]::StartNew()
cargo test --manifest-path src-tauri/Cargo.toml
$testCode = $LASTEXITCODE
$testWatch.Stop()
"NOOP_CARGO_TEST_EXIT=$testCode"
"NOOP_CARGO_TEST_SECONDS=$([math]::Round($testWatch.Elapsed.TotalSeconds, 2))"
if ($testCode -ne 0) { exit $testCode }
```

Expected: focused filter and both complete no-op commands pass; no command contains `--target-dir`.
The focused wrapper must report at least one passed test (the current prefix
selects three runtime-config tests); exit code 0 with `0 passed` is a failure.

- [ ] **Step 13: Record the rebuilt target and dominant timing units**

Run:

```powershell
$target = 'src-tauri/target'
$afterBytes = (Get-ChildItem -LiteralPath $target -Recurse -File -ErrorAction SilentlyContinue |
    Measure-Object -Property Length -Sum).Sum
$afterGiB = [math]::Round($afterBytes / 1GB, 2)
$newCodexTargets = @(Get-ChildItem -LiteralPath $target -Directory -Filter 'codex-*' -ErrorAction SilentlyContinue)
"TARGET_AFTER_GIB=$afterGiB"
"CODEX_TARGET_COUNT_AFTER=$($newCodexTargets.Count)"
```

Expected: size is printed and ordinary commands created no new `codex-*` directory. Open the generated Cargo timing HTML locally, read the longest-duration compilation units from its timing table/graph, and record at least the three dominant unit names and durations; do not infer them from crate size.

- [ ] **Step 14: Replace the Cargo evidence instructions with observed facts**

In `docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md`, replace the `Cargo Profile and Cache` instruction paragraph with:

- the explicit cleanup decision (`approved` or `declined`);
- target GiB before and after;
- historical `codex-*` count before and confirmation that normal commands created none;
- first profile `cargo check --timings` and full `cargo test` wall times;
- no-op check/test wall times and Cargo's test-execution time;
- generated timing-report path relative to the repository;
- at least three dominant compilation units and their observed durations;
- a note that a cleanup-approved cold run is not compared directly with the old partially warmed baseline.

Write literal observed values and no placeholders.

- [ ] **Step 15: Review and commit the Cargo/profile slice**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$changed = @(git status --short)
$changed
$allowed = @(
    'AGENTS.md',
    'docs/project.md',
    'docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md',
    'src-tauri/Cargo.toml',
    'src/lib/development-loop-performance-contract.test.ts'
)
$paths = @(
    git diff --name-only
    git diff --cached --name-only
    git ls-files --others --exclude-standard
) | Sort-Object -Unique
$paths = @($paths)
if (@($paths | Where-Object { $_ -notin $allowed }).Count -ne 0) { exit 1 }
git add -- AGENTS.md docs/project.md docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md src-tauri/Cargo.toml src/lib/development-loop-performance-contract.test.ts
git diff --cached --check
git commit -m "perf: reduce daily Rust build overhead"
```

Expected: only the five listed files are committed; generated target contents remain ignored and unstaged.

---

### Task 4: Full Gate and Final Evidence

**Files:**
- Modify: `docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md`

**Interfaces:**
- Consumes: all frontend and Cargo changes from Tasks 1-3.
- Produces: final correctness/evidence record and a clean implementation branch.
- Preserves: unchanged `scripts/verify.mjs` sequence and full-gate authority.

- [ ] **Step 1: Confirm the implementation checkpoint is clean**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git log -3 --oneline
if ($status.Count -ne 0) { $status; exit 1 }
```

Expected: clean tree and the frontend, evidence, and Cargo/profile commits are visible.

- [ ] **Step 2: Run the explicit static checks**

Run:

```powershell
npm.cmd run check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: Svelte/TypeScript and Rust formatting checks pass.

- [ ] **Step 3: Run the unchanged authoritative gate**

Run:

```powershell
$watch = [Diagnostics.Stopwatch]::StartNew()
npm.cmd run verify
$code = $LASTEXITCODE
$watch.Stop()
"FULL_VERIFY_EXIT=$code"
"FULL_VERIFY_SECONDS=$([math]::Round($watch.Elapsed.TotalSeconds, 2))"
if ($code -ne 0) { exit $code }
```

Expected: all existing `scripts/verify.mjs` steps pass: full Vitest, Svelte check, rustfmt check, Cargo check, full Cargo test, and `git diff HEAD --check`.

- [ ] **Step 4: Record final gates and acceptance conclusions**

In the verification document, replace the `Final Gates` instruction paragraph with literal outcomes and durations for:

- focused source-contract test;
- three full frontend runs and median (refer to the existing frontend table rather than duplicating values);
- `npm.cmd run check`;
- `npm.cmd run check:rustfmt`;
- focused Rust wrapper;
- full Cargo test;
- full `npm.cmd run verify`.

Add an `## Acceptance Summary` section stating only evidence-backed conclusions: inventory retained, median materially below the same-machine baseline, focused commands executable, canonical target reused, reduced debug profile active, cleanup decision honored, and full gate passed. Do not claim a portable timing guarantee.

- [ ] **Step 5: Review the final diff and commit evidence**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$changed = @(git status --short)
$changed
$paths = @(
    git diff --name-only
    git diff --cached --name-only
    git ls-files --others --exclude-standard
) | Sort-Object -Unique
$paths = @($paths)
if ($paths.Count -ne 1 -or $paths[0] -ne 'docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md') { exit 1 }
git add -- docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md
git diff --cached --check
git commit -m "docs: finalize daily loop verification"
```

Expected: only the verification record is committed.

- [ ] **Step 6: Verify the final repository state**

Run:

```powershell
git status --short --untracked-files=all
git log -4 --oneline
git show --check --stat --oneline HEAD
```

Expected: clean tree; four slice commits are visible; the final commit has no whitespace errors. Do not push, merge, delete branches, or remove target artifacts unless the user separately requests that action.
