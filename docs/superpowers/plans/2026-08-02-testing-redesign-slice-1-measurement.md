# Testing Redesign Slice 1 Measurement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the redesign's minimal timing primitive, rebaseline the current test/gate inventory, and decide from controlled measurements whether root `extractum` paths can have a bounded sub-15-second fast owner.

**Architecture:** Add a dependency-free JSONL timing writer and two one-shot measurement drivers under `scripts/testing/`. The baseline driver observes the current commands without changing their orchestration. The Rust driver uses unique byte-reversible source mutations, Cargo JSON compiler artifacts, and one exact zero-I/O test to separate the check floor, combined test build, direct harness execution, and canonical Cargo end-to-end floor. Generated raw data stays ignored; one self-contained verification document is committed.

**Tech Stack:** Node.js ESM, TypeScript, Vitest 4.1.5, Cargo 1.95 stable JSON messages, Rust libtest, Git, Windows PowerShell.

## Global Constraints

- Implement Slice 1 of `docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md`, approved at commit `50ce5781` or a descendant.
- This slice adds only measurement infrastructure and evidence. Do not add `TestingManifest`, a gate registry, resource probes, per-project startup benchmarks, source-migration ledger, selectors, scheduling changes, or performance state.
- Do not modify the command coverage or fail-fast behavior of `scripts/verify.mjs`; the baseline driver wraps existing commands externally.
- Persist timing rows only under ignored `artifacts/testing/timings.jsonl`. Every row has exactly `command`, `startedAt`, `duration`, `exitCode`, and `commit`.
- Capture `startedAt` immediately before spawn as a UTC ISO-8601 timestamp with milliseconds. Store duration as non-negative integer milliseconds and the full HEAD hash observed before the command starts.
- A timing-log directory/write failure prints one warning and never changes the observed command's exit code.
- Use `npm.cmd`, not plain `npm`, for npm scripts on Windows.
- Run all measurements sequentially. Before retained measurements, reject active `cargo`, `rustc`, `rust-analyzer`, `vitest`, `playwright`, `node` instances whose command line belongs to this repository, except the current measurement process.
- Use only canonical `src-tauri/target`; do not use `--target-dir`, run `cargo clean`, install a profiler, change the linker, or install `cargo-nextest`.
- Rust mutations target `src-tauri/src/readiness.rs`, append only an inert unique comment, and restore the original bytes in `finally`. Interpret no result until SHA-256 and `git diff --exit-code -- src-tauri/src/readiness.rs` prove restoration.
- Use the exact zero-I/O test `readiness::tests::mark_failed_returns_failed_state`. Reject a direct-binary result unless libtest reports exactly one passed test and zero failed tests.
- Use one disclosed warm-up and exactly three retained samples for every Rust command shape. Never replace a failed or slow retained sample.
- Store raw JSON, logs, generated reports, and temporary executable-path records under `artifacts/testing/slice-1/`; commit only source/tests and the final Markdown verification record.
- Machine durations are local observations, not portable thresholds or eligibility state.
- Preserve unrelated user changes and stage only files named by the active task.
- Do not push.

## Rust Verification Loops

- **Affected package:** root package `extractum`; the slice makes no persistent Rust source change, but its diagnostic deliberately invalidates and restores `src-tauri/src/readiness.rs`.
- **Narrow RED/GREEN test:** list first with `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list | Select-String 'readiness::tests::mark_failed_returns_failed_state'`, then run `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib readiness::tests::mark_failed_returns_failed_state -- --exact`; evidence is invalid if zero tests execute.
- **Focused check:** `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets` after the diagnostic has restored the source.
- **Package checkpoint:** `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets` after restoration; do not run it immediately before the workspace gate if the final `verify` will exercise the same state.
- **End-of-slice workspace gate:** `npm.cmd run verify` once after all source, documentation, and evidence changes are ready.

---

### Task 1: Add the Exact Five-Field Timing Writer

**Files:**
- Create: `scripts/testing/timing-log.mjs`
- Create: `scripts/testing/timing-log.test.ts`

**Interfaces:**
- Export `formatCommand(command, args)` for stable human-reproducible command strings.
- Export `createTimingRow({ command, startedAt, duration, exitCode, commit })` and reject malformed rows before writing.
- Export `readHeadCommit(repoRoot, spawnSyncImpl)`.
- Export `appendTimingRow(row, options)` and `recordTimingBestEffort(row, options)`.
- Default output: `artifacts/testing/timings.jsonl` below the supplied repository root.

- [ ] **Step 1: Write failing contract tests**

Create `scripts/testing/timing-log.test.ts` with these cases:

```ts
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createTimingRow,
  formatCommand,
  recordTimingBestEffort,
} from "./timing-log.mjs";

const roots: string[] = [];
afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

describe("minimal timing log", () => {
  it("writes exactly the approved five fields", async () => {
    const repoRoot = await mkdtemp(path.join(tmpdir(), "extractum-timing-"));
    roots.push(repoRoot);
    const row = createTimingRow({
      command: "node example.mjs --flag",
      startedAt: "2026-08-02T10:11:12.123Z",
      duration: 17.8,
      exitCode: 0,
      commit: "a".repeat(40),
    });

    await recordTimingBestEffort(row, { repoRoot });
    const text = await readFile(path.join(repoRoot, "artifacts/testing/timings.jsonl"), "utf8");
    expect(Object.keys(JSON.parse(text.trim()))).toEqual([
      "command", "startedAt", "duration", "exitCode", "commit",
    ]);
    expect(JSON.parse(text.trim()).duration).toBe(18);
  });

  it("quotes whitespace and embedded quotes deterministically", () => {
    expect(formatCommand("node", ["a b.mjs", "--name", 'a"b'])).toBe(
      'node "a b.mjs" --name "a\\"b"',
    );
  });

  it("warns without throwing when persistence fails", async () => {
    const warn = vi.fn();
    const appendFile = vi.fn().mockRejectedValue(new Error("disk unavailable"));
    const row = createTimingRow({
      command: "node example.mjs", startedAt: "2026-08-02T10:11:12.123Z",
      duration: 1, exitCode: 7, commit: "b".repeat(40),
    });
    await expect(recordTimingBestEffort(row, { appendFile, mkdir: vi.fn(), warn }))
      .resolves.toBe(false);
    expect(warn).toHaveBeenCalledOnce();
  });
});
```

- [ ] **Step 2: Run the RED test**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/timing-log.test.ts
```

Expected: FAIL because `scripts/testing/timing-log.mjs` does not exist.

- [ ] **Step 3: Implement the writer without third-party dependencies**

Create `scripts/testing/timing-log.mjs`. Use this row construction and validation shape; keep filesystem and warning functions injectable for the failure test:

```js
const FIELD_NAMES = ["command", "startedAt", "duration", "exitCode", "commit"];

export function createTimingRow(input) {
  const row = {
    command: input.command,
    startedAt: input.startedAt,
    duration: Math.max(0, Math.round(input.duration)),
    exitCode: input.exitCode,
    commit: input.commit,
  };
  if (typeof row.command !== "string" || row.command.length === 0) throw new TypeError("command");
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(row.startedAt)) throw new TypeError("startedAt");
  if (!Number.isInteger(row.duration) || !Number.isInteger(row.exitCode)) throw new TypeError("duration/exitCode");
  if (!/^[0-9a-f]{40,64}$/i.test(row.commit)) throw new TypeError("commit");
  if (Object.keys(row).join("\0") !== FIELD_NAMES.join("\0")) throw new TypeError("timing fields");
  return row;
}
```

`readHeadCommit` runs `git rev-parse HEAD` with `cwd: repoRoot`, UTF-8 output, and `shell: false`; a missing commit is an infrastructure error for a measurement driver. `recordTimingBestEffort` catches both directory creation and append failures, prints `Timing log warning: <message>`, and returns `false` rather than throwing.

- [ ] **Step 4: Run the GREEN test**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/timing-log.test.ts
```

Expected: PASS, 1 file and 3 tests.

- [ ] **Step 5: Commit the timing primitive**

Run:

```powershell
git diff --check
git add -- scripts/testing/timing-log.mjs scripts/testing/timing-log.test.ts
git diff --cached --check
git commit -m "test: add minimal timing log"
```

Expected: one commit containing only the writer and its tests.

---

### Task 2: Add a Timed Process Observation Primitive

**Files:**
- Create: `scripts/testing/run-observation.mjs`
- Create: `scripts/testing/run-observation.test.ts`

**Interfaces:**
- Export `runObservedCommand({ command, args, cwd, stdio, capture, dependencies })`.
- Return `{ command, startedAt, duration, exitCode, commit, stdout, stderr, signal }`.
- Normal exit keeps its integer code; `SIGINT` becomes 130; another signal/spawn failure becomes 3.
- Record one row after termination. A logging failure must not replace the process result.

- [ ] **Step 1: Write RED tests using an injected fake child process**

Cover four cases in `scripts/testing/run-observation.test.ts`:

1. `startedAt` is read immediately before `spawn` and duration uses monotonic time.
2. A normal exit code 9 is returned and persisted as 9.
3. `SIGINT` becomes 130 and another signal becomes 3.
4. A rejected timing append warns but the original exit code remains unchanged.

Use `EventEmitter` for the fake child and assert that the row passed to `recordTimingBestEffort` has exactly the five approved keys.

- [ ] **Step 2: Prove RED**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/run-observation.test.ts
```

Expected: FAIL because `run-observation.mjs` does not exist.

- [ ] **Step 3: Implement the process observer**

The implementation must capture the commit before timing, then use this order:

```js
const startedAt = nowDate().toISOString();
const monotonicStart = nowMonotonic();
const child = spawn(command, args, { cwd, shell: false, stdio });
```

On `close`, compute `Math.round(nowMonotonic() - monotonicStart)`, create the five-field row, await best-effort persistence, and resolve the observation. When `capture` is true, use pipes, mirror output only when requested, and retain UTF-8 `stdout`/`stderr` for Cargo JSON parsing.

- [ ] **Step 4: Prove GREEN and regression-check both timing modules**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/timing-log.test.ts scripts/testing/run-observation.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the observer**

Run:

```powershell
git add -- scripts/testing/run-observation.mjs scripts/testing/run-observation.test.ts
git diff --cached --check
git commit -m "test: add timed command observer"
```

Expected: one focused commit.

---

### Task 3: Add the Current-Command Baseline Driver

**Files:**
- Create: `scripts/testing/slice-1-baseline.mjs`
- Create: `scripts/testing/slice-1-baseline.test.ts`

**Interfaces:**
- Export `BASELINE_COMMANDS` and `runBaseline({ repoRoot, runCommand })`.
- Export `resolveNpmScript(script, extraArgs, environment)`; on Windows it invokes `npm.cmd` through `ComSpec` with `shell: false`, because `.cmd` files are not native executables.
- CLI supports only `--output <path-under-artifacts/testing/slice-1>`.
- Generate JSON containing each complete command, start, duration, exit code, and parsed inventory when the runner exposes it.
- Attempt every listed command sequentially even when an earlier observation fails; return the first non-zero exit code after writing the complete report.

- [ ] **Step 1: Write the baseline contract test**

Assert that the logical command table contains these exact observations in this order:

```js
[
  ["frontend Vitest", { npmScript: "test", extraArgs: ["--reporter=json"] }],
  ["Svelte check", { npmScript: "check" }],
  ["sidecar typecheck", { npmScript: "test:gemini-browser-sidecar:typecheck" }],
  ["sidecar unit", { npmScript: "test:gemini-browser-sidecar:unit", extraArgs: ["--reporter=json"] }],
  ["sidecar build", { npmScript: "test:gemini-browser-sidecar:build" }],
  ["adapter typecheck", { npmScript: "test:gemini-browser-adapter:typecheck" }],
  ["adapter unit", { npmScript: "test:gemini-browser-adapter:unit", extraArgs: ["--reporter=json"] }],
  ["adapter Playwright", { npmScript: "test:gemini-browser-adapter:e2e", extraArgs: ["--reporter=json"] }],
  ["Cargo check", { command: "cargo", args: ["check", "--manifest-path", "src-tauri/Cargo.toml", "--workspace", "--all-targets"] }],
  ["Cargo test", { command: "cargo", args: ["test", "--manifest-path", "src-tauri/Cargo.toml", "--workspace", "--all-targets"] }],
  ["full verify", { npmScript: "verify" }],
]
```

Also assert sequential invocation, continuation after an injected failure, first-failure exit precedence, and rejection of an output path outside `artifacts/testing/slice-1`.

- [ ] **Step 2: Run RED**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/slice-1-baseline.test.ts
```

Expected: FAIL because the baseline module is absent.

- [ ] **Step 3: Implement the driver**

Use `runObservedCommand` for every table entry. On Windows, resolve an npm descriptor to:

```js
{
  command: environment.ComSpec ?? "C:\\Windows\\System32\\cmd.exe",
  args: ["/d", "/s", "/c", "npm.cmd", "run", script, ...(extraArgs.length ? ["--", ...extraArgs] : [])],
}
```

Keep `shell: false`; the explicit command processor is part of the complete normalized timing command. Parse Vitest JSON only when valid and retain at least `numTotalTestSuites`, `numPassedTestSuites`, `numTotalTests`, `numPassedTests`, and each test file path. Parse Playwright JSON into suite/spec/test counts. Never infer a passing inventory from a failed or unparsable report; preserve the raw parse error in the generated baseline JSON.

The driver itself does not add a richer timing row. Its child commands already produce the five-field observations; its generated JSON is diagnostic evidence under the ignored artifact directory.

- [ ] **Step 4: Run GREEN**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/slice-1-baseline.test.ts scripts/testing/run-observation.test.ts scripts/testing/timing-log.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the baseline driver**

Run:

```powershell
git add -- scripts/testing/slice-1-baseline.mjs scripts/testing/slice-1-baseline.test.ts
git diff --cached --check
git commit -m "test: add slice one baseline driver"
```

Expected: one focused commit.

---

### Task 4: Add the Byte-Reversible Rust Feasibility Driver

**Files:**
- Create: `scripts/testing/slice-1-rust-feasibility.mjs`
- Create: `scripts/testing/slice-1-rust-feasibility.test.ts`

**Interfaces:**
- Export `RUST_TEST_NAME`, `RETAINED_RUNS`, `WARMUP_RUNS`, `buildRunSchedule()`, `parseCargoArtifacts(text, expectation)`, `parseExactLibtest(text)`, `appendMutation(original, token)`, and `runRustFeasibility(options)`.
- CLI supports `--output artifacts/testing/slice-1/rust-feasibility.json` and no free-form command or path override.
- Always restore original bytes in `finally`; restoration/hash failure exits 3 and invalidates the report.

- [ ] **Step 1: Write RED tests for the schedule, artifact proof, exact test, and restoration**

The tests must assert:

- one warm-up plus three retained observations for `noopCheck`, `invalidatedCheck`, `noRun`, `directBinary`, and `endToEnd`;
- each retained invalidating attempt has a unique token;
- retained invalidating order is `check → noRun/direct → endToEnd`, `endToEnd → noRun/direct → check`, then `noRun/direct → check → endToEnd`;
- `parseCargoArtifacts` accepts only a `compiler-artifact` for package `extractum`, target `extractum_lib`, `fresh: false`, and the expected test/non-test profile;
- the no-run artifact supplies an executable below canonical `src-tauri/target`;
- `parseExactLibtest` rejects zero tests, two tests, ignored-only output, or any failure;
- an injected command failure still restores the exact original `Buffer` and records the failed retained sample without replacement.

- [ ] **Step 2: Run RED**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/slice-1-rust-feasibility.test.ts
```

Expected: FAIL because the driver module is absent.

- [ ] **Step 3: Implement the fixed command shapes**

Use these commands; add `--message-format=json` to both compiler-observation commands:

```js
const cargoBase = ["--manifest-path", "src-tauri/Cargo.toml", "-p", "extractum", "--lib"];
const commands = {
  noopCheck: ["cargo", ["check", ...cargoBase, "--message-format=json"]],
  invalidatedCheck: ["cargo", ["check", ...cargoBase, "--message-format=json"]],
  noRun: ["cargo", ["test", ...cargoBase, "--no-run", "--message-format=json"]],
  endToEnd: ["cargo", ["test", ...cargoBase, RUST_TEST_NAME, "--", "--exact"]],
};
```

Run the direct executable returned by its paired `noRun` compiler artifact as:

```js
[executable, [RUST_TEST_NAME, "--exact", "--nocapture"]]
```

Before each invalidating command, append `// extractum-slice-1-probe:<cohort>:<crypto.randomUUID()>` using the file's existing bytes/newline style. After the command, restore using the originally captured `Buffer`, re-read it, and compare SHA-256. A separate mutation is mandatory for `endToEnd`; it may not reuse the `noRun` build.

Before retained no-op controls, establish the canonical restored source once, run the disclosed no-op warm-up, and then run three consecutive retained no-op checks. Do not place a no-op sample immediately after source restoration from a mutation.

- [ ] **Step 4: Generate the diagnostic fields and decision mechanically**

The JSON report must retain every warm-up and sample. For the three retained samples, calculate medians for:

- `checkFloorMs` from invalidated check;
- `combinedTestBuildMs` from no-run;
- `testBuildOverCheckMs = combinedTestBuildMs - checkFloorMs`, labeled with all confounders from the specification;
- `directHarnessMs` from paired direct-binary execution;
- `cargoEndToEndMs` from the canonical exact Cargo command.

Emit exactly one classification:

```js
if (checkFloorMs > 15_000) classification = "PACKAGE_BOUNDARY_OR_SLOW";
else if (combinedTestBuildMs > 15_000 || cargoEndToEndMs > 15_000)
  classification = "SMALLER_TEST_TARGET_REQUIRED";
else if (directHarnessMs === Math.max(checkFloorMs, combinedTestBuildMs, directHarnessMs, cargoEndToEndMs))
  classification = "HARNESS_OPTIMIZATION_REQUIRED";
else classification = "BOUNDED_FAST_OWNER_PLAUSIBLE";
```

The generated report must explain that the delta includes `cfg(test)`, root test code, dev-dependencies, `app-test-support`, different compiler units, code generation, link, and cache/process noise; it must never label the delta as pure link time.

- [ ] **Step 5: Run GREEN**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/slice-1-rust-feasibility.test.ts scripts/testing/run-observation.test.ts scripts/testing/timing-log.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit the Rust diagnostic driver**

Run:

```powershell
git add -- scripts/testing/slice-1-rust-feasibility.mjs scripts/testing/slice-1-rust-feasibility.test.ts
git diff --cached --check
git commit -m "test: add Rust feasibility diagnostic"
```

Expected: one focused commit and no Rust source diff.

---

### Task 5: Capture the Current Baseline

**Files:**
- Generate ignored: `artifacts/testing/timings.jsonl`
- Generate ignored: `artifacts/testing/slice-1/current-baseline.json`

- [ ] **Step 1: Establish a clean measurement preflight**

Run:

```powershell
git status --short --untracked-files=all
git diff --exit-code -- src-tauri/src/readiness.rs
$repoProcesses = Get-CimInstance Win32_Process | Where-Object {
  $_.ProcessId -ne $PID -and $_.CommandLine -like '*G:\Develop\Extractum*' -and
  $_.Name -match '^(cargo|rustc|rust-analyzer|node|vitest|playwright)(\.exe)?$'
}
$repoProcesses | Select-Object ProcessId, Name, CommandLine
if ($repoProcesses) { exit 1 }
```

Expected: clean tree, no Rust diff, no competing repository process. Stop and resolve; do not terminate an unknown user process automatically.

- [ ] **Step 2: Run the baseline once without retry substitution**

Run:

```powershell
node scripts/testing/slice-1-baseline.mjs --output artifacts/testing/slice-1/current-baseline.json
```

Expected: every current command is attempted sequentially, every command has one five-field JSONL row, and the driver exits zero only if every observed command passed. A missing Playwright browser is recorded as a failure, not converted into a skip.

- [ ] **Step 3: Validate raw output invariants**

Run:

```powershell
$rows = Get-Content artifacts/testing/timings.jsonl | ForEach-Object { $_ | ConvertFrom-Json }
$bad = @($rows | Where-Object {
  @($_.PSObject.Properties.Name) -join ',' -ne 'command,startedAt,duration,exitCode,commit' -or
  $_.duration -lt 0 -or $_.startedAt -notmatch '^\d{4}-\d{2}-\d{2}T.*\.\d{3}Z$'
})
"ROWS=$($rows.Count) BAD_ROWS=$($bad.Count)"
if ($bad.Count -ne 0) { exit 1 }
Get-Content artifacts/testing/slice-1/current-baseline.json
```

Expected: zero malformed rows and a literal observation for Vitest, Svelte-check, sidecar, adapter, Playwright, Cargo, and full `verify`.

---

### Task 6: Run the Controlled Rust Diagnostic

**Files:**
- Generate ignored: `artifacts/testing/slice-1/rust-feasibility.json`

- [ ] **Step 1: Prove the exact test exists before measuring**

Run:

```powershell
$listed = cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$matches = @($listed | Select-String '^readiness::tests::mark_failed_returns_failed_state: test$')
"MATCHES=$($matches.Count)"
if ($matches.Count -ne 1) { exit 1 }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib readiness::tests::mark_failed_returns_failed_state -- --exact
```

Expected: exactly one listed test and exactly one executed passing test.

- [ ] **Step 2: Record the original source identity**

Run:

```powershell
$beforeHash = (Get-FileHash src-tauri/src/readiness.rs -Algorithm SHA256).Hash
$beforeLength = (Get-Item src-tauri/src/readiness.rs).Length
"HASH=$beforeHash LENGTH=$beforeLength"
```

Expected: one baseline hash and byte length retained in the terminal transcript.

- [ ] **Step 3: Run one warm-up and three retained samples per command shape**

Run:

```powershell
node scripts/testing/slice-1-rust-feasibility.mjs --output artifacts/testing/slice-1/rust-feasibility.json
```

Expected: zero exit; each invalidated compiler sample proves `extractum_lib` rebuilt with `fresh: false`; each no-run artifact is paired with its exact executable; each direct run reports one pass; no retained failure is replaced.

- [ ] **Step 4: Prove byte restoration before reading results**

Run:

```powershell
$afterHash = (Get-FileHash src-tauri/src/readiness.rs -Algorithm SHA256).Hash
$afterLength = (Get-Item src-tauri/src/readiness.rs).Length
"HASH_RESTORED=$($beforeHash -eq $afterHash) LENGTH_RESTORED=$($beforeLength -eq $afterLength)"
git diff --exit-code -- src-tauri/src/readiness.rs
if ($beforeHash -ne $afterHash -or $beforeLength -ne $afterLength) { exit 1 }
```

Expected: both restoration checks are `True` and Git reports no diff.

- [ ] **Step 5: Inspect the literal report and classification**

Run:

```powershell
$rust = Get-Content artifacts/testing/slice-1/rust-feasibility.json -Raw | ConvertFrom-Json
$rust.summary | Format-List
"CLASSIFICATION=$($rust.classification)"
if (@($rust.samples | Where-Object { $_.retained }).Count -ne 15) { exit 1 }
```

Expected: 15 retained observations (three for each of five shapes), disclosed warm-ups, all five requested medians/delta, confounder text, and exactly one classification.

---

### Task 7: Publish Slice 1 Evidence and the Explicit Rust Decision

**Files:**
- Create: `docs/superpowers/verification/2026-08-02-testing-redesign-slice-1-measurement.md`

- [ ] **Step 1: Create the evidence record from literal generated values**

Create the document with `apply_patch` and these exact sections:

```markdown
# Testing Redesign Slice 1 Measurement Verification

## Scope and Starting Commit
## Environment and Limitations
## Five-Field Timing Writer
## Current Test Inventory
## Current Gate Durations
## Rust Diagnostic Protocol
## Rust Retained Samples
## Rust Decomposition
## Rust Fast-versus-Slow Decision
## Restoration and Verification
```

Populate it only with values from `current-baseline.json`, `rust-feasibility.json`, tool version commands, and the timing JSONL validation. Include:

- every command, duration, and exit code;
- Vitest/Playwright inventories where parsable;
- all warm-ups labeled separately from all retained Rust samples;
- medians for check floor, combined test build, delta, direct binary, and Cargo end-to-end;
- the complete delta-confounder statement;
- proof that every invalidated retained unit rebuilt and every direct binary ran exactly one test;
- source hash/length restoration;
- exactly one decision from the approved table.

Decision text must follow the classification:

- `PACKAGE_BOUNDARY_OR_SLOW`: state that root paths cannot be fast owners as-is; either plan a named smaller production package boundary or classify them slow. If no bounded seam is evident from existing module/package boundaries, select slow and return to the user before any open-ended rewrite.
- `SMALLER_TEST_TARGET_REQUIRED`: name the smallest existing source/test boundary supported by the artifact evidence; do not promise sub-15 seconds without a follow-up measurement.
- `HARNESS_OPTIMIZATION_REQUIRED`: identify direct harness/setup evidence and keep the path slow until a bounded optimization is approved.
- `BOUNDED_FAST_OWNER_PLAUSIBLE`: name the measured command shape that fits and carry it into Slice 4 planning.

Do not write `TODO`, `TBD`, portable CPU/memory claims, eligibility state, or a pure-link claim.

- [ ] **Step 2: Run focused script tests and restored Rust checks**

Run:

```powershell
node scripts/run-vitest.mjs run scripts/testing/timing-log.test.ts scripts/testing/run-observation.test.ts scripts/testing/slice-1-baseline.test.ts scripts/testing/slice-1-rust-feasibility.test.ts
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: all script tests pass and the focused Cargo check succeeds from restored source.

- [ ] **Step 3: Review the complete slice diff**

Run:

```powershell
git diff --check
git status --short --untracked-files=all
git diff -- docs/superpowers/verification/2026-08-02-testing-redesign-slice-1-measurement.md
```

Expected: only the uncommitted verification document remains after Tasks 1-4 commits; ignored `artifacts/` files do not appear.

- [ ] **Step 4: Commit the evidence checkpoint**

Run:

```powershell
git add -- docs/superpowers/verification/2026-08-02-testing-redesign-slice-1-measurement.md
git diff --cached --check
git commit -m "docs: record testing redesign slice one"
```

Expected: one documentation/evidence commit.

---

### Task 8: Run the End-of-Slice Gate and Close the Checkpoint

**Files:**
- Read: all Slice 1 commits and verification evidence

- [ ] **Step 1: Run the authoritative workspace gate once**

Run:

```powershell
npm.cmd run verify
```

Expected: all existing verification checks pass. A timing-log warning, if any, is reported separately and cannot change gate correctness.

- [ ] **Step 2: Confirm clean state and inspect commits**

Run:

```powershell
$status = @(git status --short --untracked-files=all 2>$null)
"STATUS_COUNT=$($status.Count)"
git log -6 --oneline
git show --check --stat --oneline HEAD
if ($status.Count -ne 0) { $status; exit 1 }
```

Expected: clean tree, successful evidence commit, and no tracked raw measurement artifacts.

- [ ] **Step 3: Stop for the Slice 1 decision checkpoint**

Report the literal Rust classification and decision to the user. Do not author or execute Slice 2A until the user accepts the checkpoint outcome. After acceptance, use the program index to write the detailed Slice 2A plan against the committed repository state.
