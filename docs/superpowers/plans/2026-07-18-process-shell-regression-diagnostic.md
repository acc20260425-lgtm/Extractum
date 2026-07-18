# Process Shell Regression Diagnostic Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax as immutable execution gates.

**Goal:** Build and execute a frozen, crash-safe A/B/C/D diagnostic that
identifies which cumulative Cargo workspace change reproduces the reverted
`extractum-process` application-shell regression, with a conditional E
manifest bisection and a terminal environment-precision outcome.

**Architecture:** Implement a committed Node ESM harness with a pure protocol
engine, direct child-process runtime, Git-backed state fixtures, and an
attempt coordinator. Develop and validate the harness in a workflow-owned
worktree, then pin each measurement attempt to the frozen protocol commit in
its own detached worktree and worktree-local Cargo target. Keep raw artifacts
outside Git, restore every attempt to A, and generate the repository
verification record from immutable JSON rather than manual transcription.

**Tech Stack:** Node.js ESM and built-in modules, Vitest 4, Git worktrees and
alternate indexes, Cargo 1.95 stable timings, Rust 2021, PowerShell 5.1 on
Windows/MSVC.

## Global Constraints

- The normative design is
  `docs/superpowers/specs/2026-07-18-process-shell-regression-diagnostic-design.md`.
- The measured baseline commit is
  `24c313a767a25284123b24ea3a4b8c083007c817`, whose `src-tauri` tree is
  `fd9711a041432ef420e7b09d56a46131a2a52a2a`. Historical D is commit
  `b364756c7b5768d644321afeaeb81ec04e2481a4`, whose `src-tauri` tree is
  `77e2d163ccc8bddf3ea051cb995909888cae9aba`.
- Use the fixed block order `A0 -> B -> A1 -> C -> A2 -> D -> A3`; append
  `E -> A4` only when A0–A3 are valid, B and C are below +500 ms, and D is at
  or above +500 ms.
- Every block uses two discarded dirty warm-ups, one recorded no-op, seven
  recorded dirty samples, and one excluded diagnostic probe. Before every
  dirty invocation, run an untimed canonical synchronization check.
- The dirty probe appends the same inert comment to
  `src-tauri/src/lib.rs`, requires `Checking extractum`, restores the exact
  canonical bytes in `finally`, and verifies SHA-256 from an on-disk recovery
  copy.
- A material effect is `delta >= 500 ms`. All A-anchor medians must have range
  no greater than 300 ms. Every block needs at least five of seven samples
  within 300 ms of its median.
- The existing shell cap is reported separately as failed when
  `delta > 500 ms` or percentage regression is greater than 5%. A disagreement
  with the material-effect classification routes to anomaly.
- The timeout for every Git, Cargo, PowerShell environment, or Node helper
  child is fixed at 30 minutes before A0. Timeout kills only that child process
  tree and invalidates the attempt. A repeated post-kill CIM inventory must
  prove the tree dead; otherwise the run records `termination_unconfirmed`,
  restores canonical source bytes only from the already-fsynced local recovery
  copy, durably records pending full-state recovery, performs no later child
  command in that invocation, and requires a new operator quiescence
  attestation before exact A recovery on resume.
- `CARGO_LOG=cargo::core::compiler::fingerprint=info`, Cargo `--timings`, and
  `-vv` appear only on the excluded diagnostic probe, never on canonical
  synchronization, warm-ups, no-op, or acceptance samples.
- B is an empty dependency-free `extractum-process` workspace member. C adds
  only the app-to-empty-crate path edge. D is reconstructed only with
  `git checkout b364756c7b5768d644321afeaeb81ec04e2481a4 -- src-tauri` and
  exact path/mode/blob/tree/diff verification.
- Every A restoration uses
  `git restore --source=24c313a767a25284123b24ea3a4b8c083007c817 --staged --worktree -- src-tauri`.
  A tree-ish checkout by pathspec is forbidden because it does not remove
  state-added paths that are absent from A.
- E starts from C. Its empty crate has exactly `anyhow`, `parking_lot`, and
  `tokio` workspace dependencies, target-specific `windows-sys.workspace =
  true`, and `tokio/test-util` as its only dev feature. Process code and
  facades remain in the app.
- A0–A4 have identical tracked `src-tauri` bytes. B/C/E are installed only
  from committed patches whose expected subtree hashes are frozen before A0.
- Every measurement attempt uses a new detached worktree and its own
  `<attempt-worktree>/src-tauri/target`. `CARGO_TARGET_DIR` is unset; never run
  `cargo clean`, pass `--target-dir`, share the main target, or measure in the
  implementation worktree.
- The first durable per-attempt preflight immediately before A0 fixes the
  authoritative toolchain/main-target environment baseline. Later attempts
  must match it before worktree creation; only power/Defender deltas tied to
  the immediately preceding nonempty corrected-cause disposition are allowed
  and recorded (Defender `QuickScanAge` is descriptive, not an invariant).
- The workflow-owned implementation worktree path is fixed before Task 1 as
  `G:/Develop/Extractum/.worktrees/process-shell-diagnostic-implementation`.
  Every Task 7/8 command block resolves that literal path again and uses
  `git -C` or an absolute manifest path; no block relies on PowerShell state
  from an earlier tool invocation.
- The first unexplained stability invalidation increments the monotonic
  `unexplained_stability_invalid_count` to 1 and permits one identical fresh
  retry. Count 2 terminates as `environment_precision_insufficient`. Explained
  stability and non-stability failures neither increment nor reset the count.
- A non-stability retry requires a recorded, corrected objective cause. It
  keeps the count and uses a new worktree/target. A valid session cannot be
  rerun. No locator may silently replace a valid or terminal session.
- Protocol source, design, plan, patches, thresholds, commands, and state
  hashes are committed and SHA-256-frozen before A0. Any edit after A0
  invalidates the current attempt; do not patch the running protocol.
- The owner approved a pre-A0 preregistration amendment on 2026-07-18 after
  Cargo 1.95 demonstrated that `cargo metadata --no-deps` leaves the resolver
  and lockfile untouched. Task 3 generates each state lock delta with
  resolver-capable `cargo metadata` (without `--no-deps`), then proves the
  result stable with the same command plus `--locked`; `cargo generate-lockfile`
  remains forbidden.
- This plan and its normative design are preregistration inputs committed
  before Task 0. Do not edit or tick their checkboxes during execution; track
  progress in the execution session/plan tool. Any amendment requires a new
  reviewed preregistration commit before A0, and both Markdown files remain
  byte-for-byte unchanged once the protocol lock is generated.
- Run only on Windows `x86_64-pc-windows-msvc`. Do not kill rust-analyzer or
  unrelated user processes automatically; require the user to stop them.
- The diagnostic does not retain `extractum-process`, start Phase 4, change
  roadmap policy, or alter product APIs/UI. Register the persisted diagnostic
  classification strings in `docs/value-registry.md` and explicitly record
  their owner plus the absence of SQLite, product API, UI, and fixture impact.
- Use `npm.cmd`, not `npm`, for repository scripts. Inspect the dirty worktree
  before every commit and stage only plan-owned files.

## File Map

**Create:**

- `scripts/process-shell-diagnostic/protocol.mjs` — frozen constants,
  statistics, E trigger, causal classification, and retry reducer.
- `scripts/process-shell-diagnostic/protocol.test.ts` — pure protocol RED/GREEN
  tests, including bimodality and retry edge cases.
- `scripts/process-shell-diagnostic/runtime.mjs` — direct process execution,
  timeouts, atomic artifacts, Cargo parsing, and byte-safe dirty probes.
- `scripts/process-shell-diagnostic/runtime.test.ts` — command, timeout,
  atomic-write, and restoration tests.
- `scripts/process-shell-diagnostic/git-state.mjs` — A/B/C/D/E installation,
  subtree/inventory verification, target isolation, and patch hashing.
- `scripts/process-shell-diagnostic/git-state.test.ts` — command-sequence and
  exact manifest-shape contracts.
- `scripts/process-shell-diagnostic/states/B.patch` — membership-only state.
- `scripts/process-shell-diagnostic/states/C.patch` — membership plus app edge.
- `scripts/process-shell-diagnostic/states/E.patch` — C plus exact D-style
  workspace dependency migration without process source movement.
- `scripts/process-shell-diagnostic/attempt.mjs` — one fixed measurement
  attempt and per-block orchestration.
- `scripts/process-shell-diagnostic/attempt.test.ts` — fake-runtime ordering,
  E-tail, failure, and restoration tests.
- `scripts/process-shell-diagnostic/coordinator.mjs` — immutable session
  ledger, fresh detached worktrees, retries, locator enforcement, and CLI.
- `scripts/process-shell-diagnostic/coordinator.test.ts` — monotonic retry and
  no-replacement integration contracts.
- `scripts/process-shell-diagnostic/freeze.mjs` — deterministic protocol/state
  hash generation and verification.
- `scripts/process-shell-diagnostic/report.mjs` — deterministic Markdown
  verification renderer.
- `scripts/process-shell-diagnostic/report.test.ts` — report fixtures for a
  causal result and `environment_precision_insufficient`.
- `scripts/process-shell-diagnostic/protocol-lock.json` — generated committed
  hashes for every frozen input and A/B/C/D/E subtree.
- `docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md`
  — generated evidence summary after the run.

**Use transiently but never retain:**

- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and
  `src-tauri/src/lib.rs` — canonical A inputs and dirty probe source.

**Read but do not modify:**

- `docs/superpowers/specs/2026-07-17-crate-roadmap.md` — remains blocked until
  the owner separately approves a consequence from the decision table.

**Modify:**

- `docs/value-registry.md` — register the diagnostic artifact classifications
  defined by `protocol.mjs`; this documents persisted evidence values without
  changing product behavior.

**Transient only in attempt worktrees:**

- `src-tauri/crates/extractum-process/**` and candidate facade/source changes.
- `<attempt-worktree>/src-tauri/target/**`.
- `%TEMP%/extractum-process-shell-sessions/process-shell-session-<session-id>/`
  — immutable manifest, numbered ledger, attempts, decision, aggregate ledger,
  artifact index, and raw command evidence.
- `G:/Develop/Extractum/.worktrees/process-shell-session-<session-id>/**` —
  preserved detached attempt worktrees and their worktree-local Cargo targets.

## Interfaces

```text
protocol.mjs
  summarizeBlock(samplesMs) -> BlockSummary
  evaluateAttempt(blockSamples) -> AttemptEvaluation
  reduceRetry(retryState, invalidation) -> RetryDecision

runtime.mjs
  runWindowsProcess(spec) -> Promise<CommandResult>
  runCargoCheck(spec) -> Promise<CargoResult>
  runDirtyCargoProbe(spec) -> Promise<CargoResult>
  writeAtomicBytesExclusive(path, bytes) -> Promise<void>
  writeAtomicJsonExclusive(path, value) -> Promise<void>
  sha256File(path) -> Promise<string>

git-state.mjs
  installState({ state, worktree, mainRoot, protocolLock, artifactDir })
    -> Promise<StateEvidence>
  verifyTargetIsolation({ metadata, worktree, mainRoot }) -> Promise<void>

attempt.mjs
  runAttempt({ worktree, mainRoot, sessionDir, attemptId, protocolLock })
    -> Promise<AttemptResult>

coordinator.mjs
  startSession({ mainRoot, protocolRoot, scratchParent, processAttested })
    -> Promise<SessionResult>
  resumeSession({ sessionDir, correctedCause?, unexplainedStability?, processAttested? })
    -> Promise<SessionResult>
  snapshotDirectory(root) -> Promise<DirectorySnapshot>
```

Artifact schemas and string unions are defined once in `protocol.mjs`; later
tasks import them rather than duplicating decision logic.

## Rust Verification Loops

**Affected packages:** transient states affect `extractum`, `extractum-core`,
and the temporary `extractum-process`. No Rust source or manifest change is
retained after the attempt returns to A.

Harness RED/GREEN and state contracts:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/protocol.test.ts
npm.cmd run test -- scripts/process-shell-diagnostic/runtime.test.ts
npm.cmd run test -- scripts/process-shell-diagnostic/git-state.test.ts
npm.cmd run test -- scripts/process-shell-diagnostic/attempt.test.ts scripts/process-shell-diagnostic/coordinator.test.ts scripts/process-shell-diagnostic/report.test.ts
```

Run Rust characterization only in a separate validation worktree/target before
A0. The exact D narrow test and focused checks are:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum-process --lib external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-process --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-process --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

For B, C, and E, validate each patch in that same non-measurement worktree:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-process --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

These validation builds must never populate an attempt target. Measurement
uses only the fixed full-workspace shell command from the design.

After the experiment restores A, run the end-of-slice gates from the
implementation worktree:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

---

### Task 0: Create the Fixed Isolated Implementation Worktree

**Files:** none yet; this task creates only the workflow-owned worktree and its
local dependency installation.

- [ ] **Step 1: Invoke the worktree skill and prove the parent is ignored**

Use `superpowers:using-git-worktrees`. From main, run:

```powershell
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticImplementationRoot = Join-Path $diagnosticMainRoot '.worktrees\process-shell-diagnostic-implementation'
$diagnosticPreregInputs = @(
    'docs/superpowers/specs/2026-07-18-process-shell-regression-diagnostic-design.md',
    'docs/superpowers/plans/2026-07-18-process-shell-regression-diagnostic.md'
)
if (Test-Path -LiteralPath $diagnosticImplementationRoot) {
    throw "Implementation worktree already exists: $diagnosticImplementationRoot"
}
foreach ($diagnosticInput in $diagnosticPreregInputs) {
    git -C "$diagnosticMainRoot" ls-files --error-unmatch -- "$diagnosticInput" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Preregistration input is not tracked at HEAD: $diagnosticInput" }
}
git -C "$diagnosticMainRoot" diff --quiet HEAD -- $diagnosticPreregInputs
if ($LASTEXITCODE -ne 0) { throw 'Preregistration plan/spec have unstaged changes.' }
git -C "$diagnosticMainRoot" diff --cached --quiet HEAD -- $diagnosticPreregInputs
if ($LASTEXITCODE -ne 0) { throw 'Preregistration plan/spec have staged changes outside HEAD.' }
git -C "$diagnosticMainRoot" check-ignore -q .worktrees
if ($LASTEXITCODE -ne 0) { throw '.worktrees must be ignored before creation.' }
git -C "$diagnosticMainRoot" worktree add -b process-shell-diagnostic "$diagnosticImplementationRoot" HEAD
if ($LASTEXITCODE -ne 0) { throw 'Implementation worktree creation failed.' }
```

Expected: the approved plan/spec are tracked and byte-clean at one preregistration
HEAD; the fixed path is a new branch worktree at that exact HEAD; main remains
unchanged.

- [ ] **Step 2: Install dependencies and establish a clean baseline**

Run in one invocation with a scoped location:

```powershell
$diagnosticImplementationRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
Push-Location -LiteralPath $diagnosticImplementationRoot
try {
    npm.cmd ci
    if ($LASTEXITCODE -ne 0) { throw 'npm dependency installation failed.' }
    npm.cmd run check
    if ($LASTEXITCODE -ne 0) { throw 'Baseline check failed before implementation.' }
    $diagnosticBaselineStatus = @(git status --short)
    if ($LASTEXITCODE -ne 0) { throw 'Could not inspect implementation baseline status.' }
    if ($diagnosticBaselineStatus.Count -ne 0) {
        throw "Dependency installation changed the implementation worktree: $($diagnosticBaselineStatus -join '; ')"
    }
} finally {
    Pop-Location
}
```

Expected: dependency installation and the existing repository check pass;
status is clean. All Tasks 1–8 run from this worktree unless they explicitly
name the non-measurement validation worktree or a coordinator attempt worktree.

### Task 1: Implement the Pure Protocol and Retry Engine

**Files:**

- Create: `scripts/process-shell-diagnostic/protocol.test.ts`
- Create: `scripts/process-shell-diagnostic/protocol.mjs`
- Modify: `docs/value-registry.md`

**Interfaces:**

- Produces all frozen constants and pure functions listed in the global
  interface map.
- Later tasks pass only raw block wall times and explicit invalidation records;
  they never reimplement thresholds or decision branches.

- [ ] **Step 1: Write the protocol RED tests**

Create `scripts/process-shell-diagnostic/protocol.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import {
  PROTOCOL,
  evaluateAttempt,
  reduceRetry,
  summarizeBlock,
} from "./protocol.mjs";

const samples = (value: number) => Array(PROTOCOL.samplesPerBlock).fill(value);

function base(overrides: Record<string, number[]> = {}) {
  return {
    A0: samples(9_000),
    B: samples(9_000),
    A1: samples(9_000),
    C: samples(9_000),
    A2: samples(9_000),
    D: samples(9_000),
    A3: samples(9_000),
    ...overrides,
  };
}

describe("process shell diagnostic protocol", () => {
  it("accepts five clustered samples and keeps both outliers", () => {
    const summary = summarizeBlock([9_000, 9_020, 9_040, 9_060, 9_080, 10_000, 11_000]);
    expect(summary).toEqual({
      samplesMs: [9_000, 9_020, 9_040, 9_060, 9_080, 10_000, 11_000],
      medianMs: 9_060,
      samplesWithinBand: 5,
      stable: true,
    });
  });

  it("rejects a four-versus-three bimodal block", () => {
    expect(summarizeBlock([9_000, 9_000, 9_000, 9_000, 10_000, 10_000, 10_000]).stable).toBe(false);
  });

  it("invalidates anchor drift above 300 ms", () => {
    const result = evaluateAttempt(base({ A3: samples(9_301) }));
    expect(result.kind).toBe("stability_invalid");
    expect(result.reasons).toContain("anchor_range_exceeded");
  });

  it("requests E only when B and C are fast and D crosses 500 ms", () => {
    const result = evaluateAttempt(base({ D: samples(9_500) }));
    expect(result.kind).toBe("needs_e");
    expect(result.eRequired).toBe(true);
  });

  it("classifies a fast E and slow D as the D-specific boundary composite", () => {
    const result = evaluateAttempt({
      ...base({ D: samples(9_500) }),
      E: samples(9_000),
      A4: samples(9_000),
    });
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("boundary_composite");
    expect(result.contrasts.dSpecificCompositeMs).toBe(500);
  });

  it("classifies a slow E as manifest-related", () => {
    const result = evaluateAttempt({
      ...base({ D: samples(9_700) }),
      E: samples(9_600),
      A4: samples(9_000),
    });
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("manifest_related");
  });

  it("classifies B at the threshold as membership configuration", () => {
    const result = evaluateAttempt(base({ B: samples(9_500) }));
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("membership_configuration");
  });

  it("classifies C at the threshold as edge-related configuration", () => {
    const result = evaluateAttempt(base({ C: samples(9_500) }));
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("edge_related_configuration");
  });

  it("routes a 5-percent versus 500-ms disagreement to anomaly", () => {
    const result = evaluateAttempt(base({ B: samples(9_480) }));
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("threshold_disagreement");
    expect(result.metrics.B.material).toBe(false);
    expect(result.metrics.B.shellCapFailed).toBe(true);
  });

  it("records not reproduced only when all declared variants stay below both caps", () => {
    const result = evaluateAttempt(base());
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("not_reproduced");
  });

  it("keeps unexplained stability count through infrastructure retries", () => {
    const first = reduceRetry(
      { unexplainedStabilityInvalidCount: 0, terminal: false },
      { kind: "stability_invalid", objectiveCauseCorrected: false },
    );
    expect(first).toMatchObject({ action: "retry", unexplainedStabilityInvalidCount: 1 });

    const blocked = reduceRetry(first.state, {
      kind: "infrastructure_invalid",
      objectiveCauseCorrected: false,
    });
    expect(blocked).toMatchObject({ action: "await_correction", unexplainedStabilityInvalidCount: 1 });

    const corrected = reduceRetry(blocked.state, {
      kind: "infrastructure_invalid",
      objectiveCauseCorrected: true,
    });
    expect(corrected).toMatchObject({ action: "retry", unexplainedStabilityInvalidCount: 1 });

    const second = reduceRetry(corrected.state, {
      kind: "stability_invalid",
      objectiveCauseCorrected: false,
    });
    expect(second).toMatchObject({
      action: "environment_precision_insufficient",
      unexplainedStabilityInvalidCount: 2,
      state: { terminal: true },
    });
  });

  it("does not count a stability failure with a corrected objective cause", () => {
    const result = reduceRetry(
      { unexplainedStabilityInvalidCount: 1, terminal: false },
      { kind: "stability_invalid", objectiveCauseCorrected: true },
    );
    expect(result).toMatchObject({ action: "retry", unexplainedStabilityInvalidCount: 1 });
  });

  it("completes a valid attempt and forbids another reduction", () => {
    const complete = reduceRetry(
      { unexplainedStabilityInvalidCount: 0, terminal: false },
      { kind: "valid", objectiveCauseCorrected: false },
    );
    expect(complete).toMatchObject({ action: "complete", state: { terminal: true } });
    expect(() => reduceRetry(complete.state, {
      kind: "valid",
      objectiveCauseCorrected: false,
    })).toThrow("retry state is terminal");
  });
});
```

- [ ] **Step 2: Run the protocol tests and verify RED**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/protocol.test.ts
```

Expected: FAIL because `./protocol.mjs` does not exist. A zero-test run is not
RED; stop if Vitest does not collect all 13 tests.

- [ ] **Step 3: Implement the protocol engine**

Create `scripts/process-shell-diagnostic/protocol.mjs`:

```js
export const PROTOCOL = Object.freeze({
  version: 1,
  baselineCommit: "24c313a767a25284123b24ea3a4b8c083007c817",
  candidateCommit: "b364756c7b5768d644321afeaeb81ec04e2481a4",
  baseSequence: Object.freeze(["A0", "B", "A1", "C", "A2", "D", "A3"]),
  conditionalSequence: Object.freeze(["E", "A4"]),
  samplesPerBlock: 7,
  warmupsPerBlock: 2,
  effectThresholdMs: 500,
  anchorRangeLimitMs: 300,
  sampleBandMs: 300,
  samplesRequiredInBand: 5,
  shellPercentCap: 5,
  commandTimeoutMs: 30 * 60 * 1_000,
  expectedCheckedPackage: "extractum",
  probeSuffix: "\n// process-shell-diagnostic-probe\n",
  cargoArgs: Object.freeze([
    "check",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--workspace",
    "--all-targets",
  ]),
});

function finiteNumber(value, label) {
  if (!Number.isFinite(value)) throw new Error(`${label} must be finite`);
  return value;
}

export function median(values) {
  if (values.length === 0) throw new Error("median requires samples");
  const sorted = values.map((value, index) => finiteNumber(value, `sample ${index}`)).sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? sorted[middle]
    : (sorted[middle - 1] + sorted[middle]) / 2;
}

export function summarizeBlock(samplesMs) {
  if (samplesMs.length !== PROTOCOL.samplesPerBlock) {
    throw new Error(`expected ${PROTOCOL.samplesPerBlock} samples, got ${samplesMs.length}`);
  }
  const canonical = [...samplesMs];
  const medianMs = median(canonical);
  const samplesWithinBand = canonical.filter(
    (sample) => Math.abs(sample - medianMs) <= PROTOCOL.sampleBandMs,
  ).length;
  return {
    samplesMs: canonical,
    medianMs,
    samplesWithinBand,
    stable: samplesWithinBand >= PROTOCOL.samplesRequiredInBand,
  };
}

function range(values) {
  return Math.max(...values) - Math.min(...values);
}

function metric(variant, leftAnchor, rightAnchor) {
  const aReferenceMs = (leftAnchor.medianMs + rightAnchor.medianMs) / 2;
  const deltaMs = variant.medianMs - aReferenceMs;
  const percentDelta = (100 * deltaMs) / aReferenceMs;
  return {
    variantMedianMs: variant.medianMs,
    aReferenceMs,
    deltaMs,
    percentDelta,
    material: deltaMs >= PROTOCOL.effectThresholdMs,
    shellCapFailed:
      deltaMs > PROTOCOL.effectThresholdMs || percentDelta > PROTOCOL.shellPercentCap,
  };
}

function summarizeRequired(blockSamples, names) {
  const summaries = {};
  for (const name of names) {
    if (!Object.hasOwn(blockSamples, name)) throw new Error(`missing block ${name}`);
    summaries[name] = summarizeBlock(blockSamples[name]);
  }
  return summaries;
}

function stabilityReasons(summaries, anchorNames) {
  const reasons = [];
  for (const [name, summary] of Object.entries(summaries)) {
    if (!summary.stable) reasons.push(`block_unstable:${name}`);
  }
  const anchorRangeMs = range(anchorNames.map((name) => summaries[name].medianMs));
  if (anchorRangeMs > PROTOCOL.anchorRangeLimitMs) reasons.push("anchor_range_exceeded");
  return { reasons, anchorRangeMs };
}

function buildMetrics(summaries, includeE) {
  const metrics = {
    B: metric(summaries.B, summaries.A0, summaries.A1),
    C: metric(summaries.C, summaries.A1, summaries.A2),
    D: metric(summaries.D, summaries.A2, summaries.A3),
  };
  if (includeE) metrics.E = metric(summaries.E, summaries.A3, summaries.A4);
  return metrics;
}

function contrasts(metrics) {
  return {
    membershipMs: metrics.B.deltaMs,
    edgeAfterMembershipMs: metrics.C.deltaMs - metrics.B.deltaMs,
    manifestAfterCMs: metrics.E ? metrics.E.deltaMs - metrics.C.deltaMs : null,
    dSpecificCompositeMs: metrics.E ? metrics.D.deltaMs - metrics.E.deltaMs : null,
    dAfterCCompositeMs: metrics.E ? null : metrics.D.deltaMs - metrics.C.deltaMs,
  };
}

export function evaluateAttempt(blockSamples) {
  const summaries = summarizeRequired(blockSamples, PROTOCOL.baseSequence);
  let stability = stabilityReasons(summaries, ["A0", "A1", "A2", "A3"]);
  if (stability.reasons.length > 0) {
    return { kind: "stability_invalid", eRequired: false, summaries, ...stability };
  }

  let metrics = buildMetrics(summaries, false);
  const eRequired = !metrics.B.material && !metrics.C.material && metrics.D.material;
  const hasE = Object.hasOwn(blockSamples, "E") || Object.hasOwn(blockSamples, "A4");
  if (eRequired && !hasE) {
    return { kind: "needs_e", eRequired: true, summaries, metrics, ...stability };
  }
  if (!eRequired && hasE) throw new Error("E/A4 present when E is not permitted");

  if (eRequired) {
    Object.assign(summaries, summarizeRequired(blockSamples, ["E", "A4"]));
    stability = stabilityReasons(summaries, ["A0", "A1", "A2", "A3", "A4"]);
    if (stability.reasons.length > 0) {
      return { kind: "stability_invalid", eRequired: true, summaries, ...stability };
    }
    metrics = buildMetrics(summaries, true);
  }

  const disagreement = Object.entries(metrics)
    .filter(([, value]) => value.material !== value.shellCapFailed)
    .map(([name]) => name);

  let classification;
  if (disagreement.length > 0) classification = "threshold_disagreement";
  else if (metrics.B.material) classification = "membership_configuration";
  else if (metrics.C.material) classification = "edge_related_configuration";
  else if (metrics.D.material && metrics.E?.material) classification = "manifest_related";
  else if (metrics.D.material) classification = "boundary_composite";
  else classification = "not_reproduced";

  return {
    kind: "valid",
    classification,
    disagreement,
    eRequired,
    summaries,
    metrics,
    contrasts: contrasts(metrics),
    ...stability,
  };
}

export function reduceRetry(state, invalidation) {
  if (state.terminal) throw new Error("retry state is terminal");
  const current = state.unexplainedStabilityInvalidCount;
  if (invalidation.kind === "valid") {
    const nextState = { unexplainedStabilityInvalidCount: current, terminal: true };
    return { action: "complete", unexplainedStabilityInvalidCount: current, state: nextState };
  }
  if (invalidation.kind === "stability_invalid") {
    if (invalidation.objectiveCauseCorrected) {
      return { action: "retry", unexplainedStabilityInvalidCount: current, state: { ...state } };
    }
    const nextCount = current + 1;
    if (nextCount >= 2) {
      const nextState = { unexplainedStabilityInvalidCount: nextCount, terminal: true };
      return {
        action: "environment_precision_insufficient",
        unexplainedStabilityInvalidCount: nextCount,
        state: nextState,
      };
    }
    const nextState = { unexplainedStabilityInvalidCount: nextCount, terminal: false };
    return { action: "retry", unexplainedStabilityInvalidCount: nextCount, state: nextState };
  }
  if (invalidation.kind === "infrastructure_invalid") {
    const action = invalidation.objectiveCauseCorrected ? "retry" : "await_correction";
    return { action, unexplainedStabilityInvalidCount: current, state: { ...state } };
  }
  throw new Error(`unknown invalidation kind: ${invalidation.kind}`);
}
```

- [ ] **Step 4: Run protocol GREEN**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/protocol.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Protocol GREEN failed.' }
```

Expected: 1 file and 13 tests PASS.

- [ ] **Step 5: Register the persisted diagnostic classifications**

Append this section to `docs/value-registry.md`:

```markdown
## Process-shell diagnostic artifact classifications

Representative source:

- `scripts/process-shell-diagnostic/protocol.mjs`

These values classify an immutable experimental `decision.json`. The harness
owns them. They are persisted only in temporary raw artifacts and the committed
verification document; they do not enter SQLite, product APIs, UI state, or
product fixtures.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `membership_configuration` | kind | Membership configuration | The empty dependency-free member already crosses the material threshold. | diagnostic protocol | taxonomy | none | n/a | yes | `decision.json`, verification document |
| `edge_related_configuration` | kind | Edge-related configuration | B is below threshold and C crosses it after adding only the app dependency edge. | diagnostic protocol | taxonomy | none | n/a | yes | `decision.json`, verification document |
| `manifest_related` | kind | Manifest-related | Conditional E reproduces D while retaining process code in the app. | diagnostic protocol | taxonomy | none | n/a | yes | `decision.json`, verification document |
| `boundary_composite` | kind | Boundary composite | D reproduces the effect but E does not, isolating the remaining D-specific code boundary/facade composite. | diagnostic protocol | taxonomy | none | n/a | yes | `decision.json`, verification document |
| `not_reproduced` | kind | Not reproduced | Every declared variant remains below both the material threshold and shell cap. | diagnostic protocol | terminal | none | n/a | yes | `decision.json`, verification document |
| `threshold_disagreement` | kind | Threshold disagreement | The absolute material threshold and existing shell-cap rule disagree. | diagnostic protocol | terminal | inspect_error | n/a | yes | `decision.json`, verification document |
| `environment_precision_insufficient` | kind | Environment precision insufficient | Two unexplained stability-invalid attempts show that the environment cannot support the preregistered precision. | diagnostic protocol | terminal | inspect_error | n/a | yes | session ledger, `decision.json`, verification document |
```

Expected review result: owner is the diagnostic harness; persistence is limited
to evidence artifacts; SQLite/API/UI/fixture impact is explicitly `none`.

- [ ] **Step 6: Commit the pure engine and registry entry**

Run:

```powershell
$diagnosticTask1Status = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect Task 1 status.' }
$diagnosticTask1Status | ForEach-Object { Write-Output $_ }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 1 diff check failed.' }
git add -- scripts/process-shell-diagnostic/protocol.mjs scripts/process-shell-diagnostic/protocol.test.ts docs/value-registry.md
if ($LASTEXITCODE -ne 0) { throw 'Could not stage Task 1 files.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Task 1 staged diff check failed.' }
git commit -m "test: define process shell diagnostic protocol"
if ($LASTEXITCODE -ne 0) { throw 'Task 1 commit failed.' }
```

Expected: only the two Task 1 files and the registry documentation are
committed.

### Task 2: Implement Direct Process Execution and Crash-Safe Probes

**Files:**

- Create: `scripts/process-shell-diagnostic/runtime.test.ts`
- Create: `scripts/process-shell-diagnostic/runtime.mjs`

**Interfaces:**

- Consumes `PROTOCOL` from Task 1.
- Produces direct `spawn -> close` wall timing, bounded Windows tree kill,
  immutable logs/metadata, ordinary/diagnostic Cargo invocations, and the only
  function allowed to mutate `src-tauri/src/lib.rs`.
- Full process environments are never persisted; only the allowlisted Cargo
  variables and a `cargo_log_enabled` boolean enter artifacts.

- [ ] **Step 1: Write runtime RED tests**

Create `scripts/process-shell-diagnostic/runtime.test.ts`:

```ts
import { mkdtemp, readFile, readdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { describe, expect, it } from "vitest";

import {
  assertCommandOk,
  ProtocolError,
  cargoInvocation,
  parseCargoOutput,
  runDirtyCargoProbe,
  runWindowsProcess,
  sha256File,
  terminateWindowsTree,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

async function scratch() {
  return mkdtemp(path.join(os.tmpdir(), "extractum-psd-runtime-"));
}

describe("process shell diagnostic runtime", () => {
  it("writes JSON once and refuses a duplicate artifact", async () => {
    const dir = await scratch();
    const target = path.join(dir, "value.json");
    await writeAtomicJsonExclusive(target, { value: 1 });
    await expect(writeAtomicJsonExclusive(target, { value: 2 })).rejects.toMatchObject({
      kind: "duplicate_artifact",
    });
    expect(JSON.parse(await readFile(target, "utf8"))).toEqual({ value: 1 });
  });

  it("lets exactly one concurrent publisher claim an artifact", async () => {
    const dir = await scratch();
    const target = path.join(dir, "race.json");
    const settled = await Promise.allSettled([
      writeAtomicJsonExclusive(target, { writer: 1 }),
      writeAtomicJsonExclusive(target, { writer: 2 }),
    ]);
    expect(settled.filter((entry) => entry.status === "fulfilled")).toHaveLength(1);
    expect(settled.filter((entry) => entry.status === "rejected")).toHaveLength(1);
    expect([1, 2]).toContain(JSON.parse(await readFile(target, "utf8")).writer);
  });

  it("strips every CARGO_LOG case variant from ordinary Cargo", () => {
    const invocation = cargoInvocation({
      diagnostic: false,
      baseEnv: { Path: "x", CARGO_LOG: "a", cargo_log: "b" },
    });
    expect(invocation.args).not.toContain("--timings");
    expect(invocation.args).not.toContain("-vv");
    expect(Object.keys(invocation.env).map((key) => key.toUpperCase())).not.toContain("CARGO_LOG");
  });

  it("adds timings, verbosity, and fingerprint logging only to diagnostic Cargo", () => {
    const invocation = cargoInvocation({ diagnostic: true, baseEnv: { Path: "x" } });
    expect(invocation.args).toContain("--timings");
    expect(invocation.args).toContain("-vv");
    expect(invocation.env.CARGO_LOG).toBe("cargo::core::compiler::fingerprint=info");
  });

  it("parses Cargo duration and checked packages without treating Compiling as Checking", () => {
    expect(parseCargoOutput(" Compiling helper v0.1.0\n Checking extractum v0.2.0\n Finished `dev` profile in 9.17s\n")).toEqual({
      cargoReportedMs: 9_170,
      checkedPackages: ["extractum"],
      extractumChecked: true,
      extractumLibRustcObserved: false,
      extractumProcessExtern: false,
    });
    const app = parseCargoOutput(
      "Running `rustc --crate-name extractum_lib --extern extractum_process=target\\debug\\deps\\libextractum_process.rmeta`\nFinished `dev` profile in 9.17s\n",
    );
    expect(app).toMatchObject({ extractumLibRustcObserved: true, extractumProcessExtern: true });
    expect(parseCargoOutput(
      "Running `rustc --crate-name helper --extern extractum_process=target\\debug\\deps\\libextractum_process.rmeta`\nFinished `dev` profile in 9.17s\n",
    ).extractumProcessExtern).toBe(false);
  });

  it("parses Cargo hour/minute/second durations from cold builds", () => {
    expect(parseCargoOutput("Finished `dev` profile in 1m 01.25s\n").cargoReportedMs).toBe(61_250);
    expect(parseCargoOutput("Finished `dev` profile in 1h 02m 03s\n").cargoReportedMs).toBe(3_723_000);
  });

  it("captures direct child stdout/stderr through close", async () => {
    const dir = await scratch();
    const result = await runWindowsProcess({
      label: "echo",
      command: process.execPath,
      args: ["-e", "console.log('out'); console.error('err')"],
      cwd: dir,
      env: { ...process.env, EXTRACTUM_TEST_SECRET: "do-not-persist" },
      artifactDir: dir,
      timeoutMs: 10_000,
      taskkillExe: path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "taskkill.exe"),
    });
    expect(result.classification).toBe("ok");
    expect(await readFile(result.stdoutPath, "utf8")).toContain("out");
    expect(await readFile(result.stderrPath, "utf8")).toContain("err");
    const intent = await readFile(path.join(dir, "runs", "echo.intent.json"), "utf8");
    expect(intent).not.toContain("do-not-persist");
  });

  it.runIf(process.platform === "win32")("does not return until an owned grandchild tree is dead", async () => {
    const dir = await scratch();
    const pidFile = path.join(dir, "grandchild.pid");
    const result = await runWindowsProcess({
      label: "timeout",
      command: process.execPath,
      args: ["-e", [
        "const { spawn } = require('node:child_process');",
        "const { writeFileSync } = require('node:fs');",
        `const child = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore' });`,
        `writeFileSync(${JSON.stringify(pidFile)}, String(child.pid));`,
        "setInterval(() => {}, 1000);",
      ].join(" ")],
      cwd: dir,
      env: process.env,
      artifactDir: dir,
      timeoutMs: 500,
      taskkillExe: path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "taskkill.exe"),
    });
    expect(result.timedOut).toBe(true);
    expect(result.taskkill).toMatchObject({ args: ["/PID", String(result.pid), "/T", "/F"] });
    expect(result.classification).toBe("timeout");
    expect(result.taskkill.survivors).toEqual([]);
    const grandchildPid = Number(await readFile(pidFile, "utf8"));
    expect(() => process.kill(grandchildPid, 0)).toThrow();
  });

  it("runs sync before mutation and restores from the disk recovery copy", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    const calls: string[] = [];
    const result = await runDirtyCargoProbe({
      label: "A0-sample-1",
      worktree: dir,
      artifactDir: dir,
      sourcePath,
      expectedCanonicalSha256,
      cargoExe: "cargo.exe",
      taskkillExe: "taskkill.exe",
      timeoutMs: 1_000,
      requireExtractum: true,
      runCargoFn: async (spec: { label: string }) => {
        calls.push(spec.label);
        if (spec.label.endsWith(".dirty")) {
          expect(await readFile(sourcePath, "utf8")).toContain("process-shell-diagnostic-probe");
          return { classification: "ok", extractumChecked: true };
        }
        return { classification: "ok", extractumChecked: false };
      },
    });
    expect(result.classification).toBe("ok");
    expect(calls).toEqual(["A0-sample-1.sync", "A0-sample-1.dirty"]);
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    expect(await readdir(path.join(dir, "recovery"))).toContain("A0-sample-1.lib.rs");
  });

  it("downgrades any survivor-proof exception to termination_unconfirmed evidence", async () => {
    const dir = await scratch();
    const evidence = await terminateWindowsTree(
      {
        pid: 4242,
        cwd: dir,
        env: { SystemRoot: "C:\\Windows" },
        taskkillExe: "taskkill.exe",
        artifactDir: dir,
        label: "proof-failure",
      },
      {
        captureOwnedWindowsPidsFn: async () => [4242, 4243],
        runTaskkillFn: async () => ({ closeObserved: true, exitCode: 0 }),
        survivingPidsFn: async () => {
          throw Object.assign(new Error("access denied"), { code: "EPERM" });
        },
      },
    );
    expect(evidence).toMatchObject({
      confirmed: false,
      terminationErrors: [{ phase: "termination-proof", kind: "Error", message: "access denied" }],
    });
    expect(JSON.parse(await readFile(
      path.join(dir, "runs", "proof-failure.termination-unconfirmed.json"),
      "utf8",
    ))).toMatchObject({ operatorActionRequired: true, confirmed: false });
  });

  it("halts with durable pending recovery after an unconfirmed Cargo-tree termination", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    const calls: string[] = [];
    await expect(runDirtyCargoProbe({
      label: "failure",
      worktree: dir,
      artifactDir: dir,
      sourcePath,
      expectedCanonicalSha256,
      cargoExe: "cargo.exe",
      taskkillExe: "taskkill.exe",
      timeoutMs: 1_000,
      requireExtractum: true,
      runCargoFn: async (spec: { label: string }) => {
        calls.push(spec.label);
        if (spec.label.endsWith(".sync")) return { classification: "ok", extractumChecked: false };
        return { classification: "termination_unconfirmed", extractumChecked: false };
      },
    })).rejects.toMatchObject({ kind: "termination_unconfirmed" });
    expect(calls).toEqual(["failure.sync", "failure.dirty"]);
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    expect(JSON.parse(await readFile(
      path.join(dir, "recovery", "failure.recovery-pending.json"),
      "utf8",
    ))).toMatchObject({
      label: "failure",
      canonical_sha256: expectedCanonicalSha256,
      source_restored_locally: true,
      operator_action_required: true,
    });
  });

  it("never downgrades unconfirmed termination when local restore/publication also fail", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    await expect(runDirtyCargoProbe({
      label: "compound-failure",
      worktree: dir,
      artifactDir: dir,
      sourcePath,
      expectedCanonicalSha256,
      cargoExe: "cargo.exe",
      taskkillExe: "taskkill.exe",
      timeoutMs: 1_000,
      runCargoFn: async (spec: { label: string }) => spec.label.endsWith(".sync")
        ? { classification: "ok" }
        : { classification: "termination_unconfirmed", timedOut: true },
      restoreSourceFn: async () => {
        throw Object.assign(new Error("source busy"), { code: "EPERM" });
      },
      writeJsonFn: async () => {
        throw new Error("artifact volume unavailable");
      },
    })).rejects.toMatchObject({
      kind: "termination_unconfirmed",
      details: {
        operatorActionRequired: true,
        restorationError: { message: "source busy" },
        pendingPublicationError: { message: "artifact volume unavailable" },
      },
    });
  });

  it("does not mutate when canonical sync fails", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    await expect(runDirtyCargoProbe({
      label: "sync-failure",
      worktree: dir,
      artifactDir: dir,
      sourcePath,
      expectedCanonicalSha256,
      cargoExe: "cargo.exe",
      taskkillExe: "taskkill.exe",
      timeoutMs: 1_000,
      requireExtractum: true,
      runCargoFn: async () => ({ classification: "timeout", timedOut: true, extractumChecked: false }),
    })).rejects.toMatchObject({ kind: "command_timeout" });
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    await expect(readdir(path.join(dir, "recovery"))).rejects.toMatchObject({ code: "ENOENT" });
  });
});
```

- [ ] **Step 2: Run runtime RED**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/runtime.test.ts
```

Expected: FAIL because `runtime.mjs` does not exist; Vitest must collect thirteen
tests on Windows.

- [ ] **Step 3: Implement the direct process and artifact primitives**

Create `scripts/process-shell-diagnostic/runtime.mjs` with this complete
content:

```js
import { spawn } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { constants } from "node:fs";
import {
  copyFile,
  link,
  mkdir,
  open,
  readFile,
  readdir,
  unlink,
} from "node:fs/promises";
import path from "node:path";
import process from "node:process";

import { PROTOCOL } from "./protocol.mjs";

export class ProtocolError extends Error {
  constructor(kind, message, details = {}) {
    super(message);
    this.name = "ProtocolError";
    this.kind = kind;
    this.details = details;
  }
}

export function hasTerminationUnconfirmed(value, seen = new Set()) {
  if (!value || typeof value !== "object" || seen.has(value)) return false;
  seen.add(value);
  if (value.classification === "termination_unconfirmed" || value.kind === "termination_unconfirmed") {
    return true;
  }
  return Object.values(value).some((entry) => hasTerminationUnconfirmed(entry, seen));
}

export async function sha256File(filePath) {
  const bytes = await readFile(filePath);
  return createHash("sha256").update(bytes).digest("hex");
}

export async function writeAtomicBytesExclusive(target, bytes) {
  await mkdir(path.dirname(target), { recursive: true });
  const temporary = `${target}.${process.pid}.${randomUUID()}.tmp`;
  const handle = await open(temporary, "wx");
  try {
    await handle.writeFile(bytes);
    await handle.sync();
  } finally {
    await handle.close();
  }
  try {
    // A same-directory hard link publishes the fully synced bytes atomically
    // and fails with EEXIST instead of replacing an existing artifact.
    await link(temporary, target);
  } catch (error) {
    if (error.code === "EEXIST") {
      throw new ProtocolError("duplicate_artifact", `artifact already exists: ${target}`);
    }
    throw error;
  } finally {
    await unlink(temporary).catch((error) => {
      if (error.code !== "ENOENT") throw error;
    });
  }
}

export async function writeAtomicJsonExclusive(target, value) {
  return writeAtomicBytesExclusive(target, Buffer.from(`${JSON.stringify(value, null, 2)}\n`, "utf8"));
}

function allowlistedEnvironment(env) {
  const names = [
    "CARGO_BUILD_TARGET",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_INCREMENTAL",
    "CARGO_TARGET_DIR",
    "RUSTFLAGS",
  ];
  const result = {};
  for (const name of names) {
    const entry = Object.entries(env).find(([key]) => key.toUpperCase() === name);
    result[name] = entry?.[1] ?? null;
  }
  result.cargo_log_enabled = Object.keys(env).some((key) => key.toUpperCase() === "CARGO_LOG");
  return result;
}

function closeResult(child) {
  return new Promise((resolve) => {
    let spawnError = null;
    let exitObserved = false;
    child.once("error", (error) => { spawnError = error.message; });
    child.once("exit", () => { exitObserved = true; });
    child.once("close", (exitCode, signal) => resolve({
      exitCode,
      signal,
      spawnError,
      exitObserved,
      closeObserved: true,
    }));
  });
}

async function bounded(promise, timeoutMs, timeoutValue) {
  let timer;
  try {
    return await Promise.race([
      promise,
      new Promise((resolve) => {
        timer = setTimeout(() => resolve(timeoutValue), timeoutMs);
      }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

function processAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}

async function captureOwnedWindowsPids(rootPid, cwd, env) {
  const script = [
    `$root = [int]${rootPid}`,
    "$all = @(Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId)",
    "$ids = @($root)",
    "do {",
    "  $next = @($all | Where-Object { $ids -contains [int]$_.ParentProcessId -and $ids -notcontains [int]$_.ProcessId } | ForEach-Object { [int]$_.ProcessId })",
    "  $ids += $next",
    "} while ($next.Count -gt 0)",
    "$ids | Sort-Object -Unique | ConvertTo-Json -Compress",
  ].join("; ");
  const child = spawn(path.join(env.SystemRoot, "System32", "WindowsPowerShell", "v1.0", "powershell.exe"), [
    "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script,
  ], { cwd, env, shell: false, windowsHide: true, stdio: ["ignore", "pipe", "pipe"] });
  let stdout = "";
  let stderr = "";
  child.stdout.on("data", (chunk) => { stdout += chunk.toString("utf8"); });
  child.stderr.on("data", (chunk) => { stderr += chunk.toString("utf8"); });
  const closed = closeResult(child);
  const result = await bounded(closed, 15_000, { timedOut: true });
  if (result.timedOut) {
    child.kill("SIGKILL");
    await bounded(closed, 5_000, null);
    throw new ProtocolError("owned_tree_inventory_timeout", `root ${rootPid}`);
  }
  if (result.exitCode !== 0) throw new ProtocolError("owned_tree_inventory_failed", stderr.trim());
  const parsed = JSON.parse(stdout.trim());
  return [...new Set((Array.isArray(parsed) ? parsed : [parsed]).map(Number))];
}

async function runTaskkill({ pid, cwd, env, taskkillExe, artifactDir, label }) {
  const stdoutPath = path.join(artifactDir, "runs", `${label}.stdout.log`);
  const stderrPath = path.join(artifactDir, "runs", `${label}.stderr.log`);
  const stdout = await open(stdoutPath, "wx");
  const stderr = await open(stderrPath, "wx");
  const args = ["/PID", String(pid), "/T", "/F"];
  let child;
  try {
    child = spawn(taskkillExe, args, {
      cwd,
      env,
      shell: false,
      windowsHide: true,
      stdio: ["ignore", stdout.fd, stderr.fd],
    });
    const closed = closeResult(child);
    let result = await bounded(closed, 15_000, { taskkillTimedOut: true });
    if (result.taskkillTimedOut) {
      child.kill("SIGKILL");
      result = {
        ...result,
        afterFallback: await bounded(closed, 5_000, { closeObserved: false }),
      };
    }
    return { ...result, args, stdoutPath, stderrPath };
  } finally {
    await stdout.close();
    await stderr.close();
  }
}

async function survivingPids(pids, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  let survivors = pids.filter(processAlive);
  while (survivors.length && Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, 100));
    survivors = pids.filter(processAlive);
  }
  return survivors;
}

export async function terminateWindowsTree(
  { pid, cwd, env, taskkillExe, artifactDir, label },
  injected = {},
) {
  const captureFn = injected.captureOwnedWindowsPidsFn ?? captureOwnedWindowsPids;
  const taskkillFn = injected.runTaskkillFn ?? runTaskkill;
  const survivorsFn = injected.survivingPidsFn ?? survivingPids;
  const aliveFn = injected.processAliveFn ?? processAlive;
  const writeJsonFn = injected.writeJsonFn ?? writeAtomicJsonExclusive;
  const observed = new Set([pid]);
  const inventoryErrors = [];
  const terminationErrors = [];
  let primary = null;
  let survivors = [pid];
  let confirmed = false;
  try {
    try {
      for (const ownedPid of await captureFn(pid, cwd, env)) observed.add(ownedPid);
    } catch (error) {
      inventoryErrors.push({ phase: "pre-kill", kind: error.kind ?? error.name, message: error.message });
    }
    primary = await taskkillFn({
      pid, cwd, env, taskkillExe, artifactDir, label: `${label}.taskkill-primary`,
    });
    for (let pass = 1; pass <= 3; pass += 1) {
      try {
        // ParentProcessId remains queryable after the root exits, so a child
        // born between snapshots is still discovered before proof succeeds.
        for (const ownedPid of await captureFn(pid, cwd, env)) observed.add(ownedPid);
      } catch (error) {
        inventoryErrors.push({ phase: `post-kill-${pass}`, kind: error.kind ?? error.name, message: error.message });
      }
      survivors = await survivorsFn([...observed], 1_000);
      if (survivors.length === 0 && inventoryErrors.length === 0) {
        await new Promise((resolve) => setTimeout(resolve, 250));
        try {
          for (const ownedPid of await captureFn(pid, cwd, env)) observed.add(ownedPid);
        } catch (error) {
          inventoryErrors.push({ phase: `confirm-${pass}`, kind: error.kind ?? error.name, message: error.message });
        }
        survivors = [...observed].filter(aliveFn);
        if (survivors.length === 0 && inventoryErrors.length === 0) {
          confirmed = true;
          break;
        }
      }
      for (const survivor of [...survivors].reverse()) {
        await taskkillFn({
          pid: survivor,
          cwd,
          env,
          taskkillExe,
          artifactDir,
          label: `${label}.taskkill-pass-${pass}-${survivor}`,
        });
      }
    }
  } catch (error) {
    terminationErrors.push({
      phase: "termination-proof",
      kind: error.kind ?? error.name,
      message: error.message ?? String(error),
    });
    confirmed = false;
  }
  const evidence = {
    ...(primary ?? {}),
    args: ["/PID", String(pid), "/T", "/F"],
    observedPids: [...observed].sort((left, right) => left - right),
    survivors,
    inventoryErrors,
    terminationErrors,
    confirmed: confirmed && terminationErrors.length === 0,
  };
  if (!evidence.confirmed) {
    try {
      await writeJsonFn(path.join(artifactDir, "runs", `${label}.termination-unconfirmed.json`), {
        schemaVersion: 1,
        rootPid: pid,
        ...evidence,
        operatorActionRequired: true,
      });
    } catch (error) {
      evidence.evidencePublicationError = {
        kind: error.kind ?? error.name,
        message: error.message ?? String(error),
      };
    }
  }
  return evidence;
}

export async function runWindowsProcess({
  label,
  command,
  args,
  cwd,
  env,
  artifactDir,
  timeoutMs,
  taskkillExe,
}) {
  const runs = path.join(artifactDir, "runs");
  await mkdir(runs, { recursive: true });
  const stdoutPath = path.join(runs, `${label}.stdout.log`);
  const stderrPath = path.join(runs, `${label}.stderr.log`);
  const metaPath = path.join(runs, `${label}.process.json`);
  await writeAtomicJsonExclusive(path.join(runs, `${label}.intent.json`), {
    schema_version: 1,
    label,
    command,
    args,
    cwd,
    environment: allowlistedEnvironment(env),
  });

  const stdout = await open(stdoutPath, "wx");
  const stderr = await open(stderrPath, "wx");
  const startedAt = new Date().toISOString();
  const startedNs = process.hrtime.bigint();
  let child;
  let outcome;
  let timedOut = false;
  let taskkill = null;
  const streamCloseErrors = [];
  try {
    child = spawn(command, args, {
      cwd,
      env,
      shell: false,
      windowsHide: true,
      stdio: ["ignore", stdout.fd, stderr.fd],
    });
    const closed = closeResult(child);
    const first = await bounded(
      closed.then((value) => ({ kind: "closed", value })),
      timeoutMs,
      { kind: "timeout" },
    );
    if (first.kind === "closed") {
      outcome = first.value;
    } else {
      timedOut = true;
      if (!child.pid) {
        outcome = await closed;
      } else {
        try {
          taskkill = await terminateWindowsTree({
            pid: child.pid,
            cwd,
            env,
            taskkillExe,
            artifactDir,
            label,
          });
        } catch (error) {
          // Termination evidence must dominate unexpected helper failures too.
          taskkill = {
            confirmed: false,
            terminationErrors: [{
              phase: "terminateWindowsTree-call",
              kind: error.kind ?? error.name,
              message: error.message ?? String(error),
            }],
          };
        }
        outcome = await bounded(
          closed,
          10_000,
          { closeObserved: false, exitObserved: false, exitCode: null, signal: null, spawnError: null },
        );
      }
    }
  } finally {
    for (const [stream, handle] of [["stdout", stdout], ["stderr", stderr]]) {
      try {
        await handle.close();
      } catch (error) {
        streamCloseErrors.push({
          stream,
          kind: error.kind ?? error.name,
          message: error.message ?? String(error),
        });
      }
    }
  }

  const elapsedMs = Number(process.hrtime.bigint() - startedNs) / 1_000_000;
  let classification = "ok";
  if (outcome.spawnError) classification = "spawn_error";
  else if (!outcome.closeObserved) classification = "termination_unconfirmed";
  else if (timedOut && (!taskkill || taskkill.confirmed !== true)) classification = "termination_unconfirmed";
  else if (timedOut) classification = "timeout";
  else if (outcome.exitCode !== 0) classification = "command_failed";

  const result = {
    schemaVersion: 1,
    label,
    command,
    args,
    cwd,
    startedAt,
    endedAt: new Date().toISOString(),
    elapsedMs,
    pid: child?.pid ?? null,
    exitCode: outcome.exitCode ?? null,
    signal: outcome.signal ?? null,
    spawnError: outcome.spawnError ?? null,
    exitObserved: outcome.exitObserved ?? false,
    closeObserved: outcome.closeObserved ?? false,
    timedOut,
    taskkill,
    streamCloseErrors,
    operatorActionRequired: classification === "termination_unconfirmed",
    stdoutPath,
    stderrPath,
    classification,
  };
  try {
    await writeAtomicJsonExclusive(metaPath, result);
  } catch (error) {
    if (classification !== "termination_unconfirmed") throw error;
    result.evidencePublicationError = {
      kind: error.kind ?? error.name,
      message: error.message ?? String(error),
    };
  }
  if (classification !== "termination_unconfirmed" && streamCloseErrors.length > 0) {
    throw new ProtocolError("process_log_close_failed", "stdout/stderr artifact close failed", {
      streamCloseErrors,
      metaPath,
    });
  }
  return result;
}

export function cargoInvocation({ diagnostic, baseEnv = process.env }) {
  const env = Object.fromEntries(
    Object.entries(baseEnv).filter(([key]) => key.toUpperCase() !== "CARGO_LOG"),
  );
  const args = [...PROTOCOL.cargoArgs];
  if (diagnostic) {
    args.push("--timings", "-vv");
    env.CARGO_LOG = "cargo::core::compiler::fingerprint=info";
  }
  return { args, env };
}

export function parseCargoOutput(output) {
  const durations = [...output.matchAll(
    /Finished[^\r\n]*? in (?:(\d+)h\s*)?(?:(\d+)m\s*)?([0-9.]+)s/g,
  )].map((match) => (
    Number(match[1] ?? 0) * 3_600_000
    + Number(match[2] ?? 0) * 60_000
    + Number(match[3]) * 1_000
  ));
  const checkedPackages = [...output.matchAll(/^\s*Checking\s+([^\s]+)/gm)].map((match) => match[1]);
  const extractumLibRustcLines = output.split(/\r?\n/).filter((line) =>
    /\bRunning\b/.test(line) && /--crate-name\s+extractum_lib\b/.test(line),
  );
  return {
    cargoReportedMs: durations.length === 1 ? durations[0] : null,
    checkedPackages: [...new Set(checkedPackages)],
    extractumChecked: checkedPackages.includes(PROTOCOL.expectedCheckedPackage),
    extractumLibRustcObserved: extractumLibRustcLines.length > 0,
    extractumProcessExtern: extractumLibRustcLines.some((line) =>
      /--extern\s+extractum_process(?:=|\s)/.test(line),
    ),
  };
}

async function timingFiles(worktree) {
  const directory = path.join(worktree, "src-tauri", "target", "cargo-timings");
  try {
    return new Set((await readdir(directory)).filter((name) => /^cargo-timing-.+\.html$/.test(name)));
  } catch (error) {
    if (error.code === "ENOENT") return new Set();
    throw error;
  }
}

export async function runCargoCheck({
  label,
  worktree,
  artifactDir,
  cargoExe,
  taskkillExe,
  timeoutMs,
  diagnostic = false,
}) {
  const beforeTimings = diagnostic ? await timingFiles(worktree) : new Set();
  const invocation = cargoInvocation({ diagnostic });
  const processResult = await runWindowsProcess({
    label,
    command: cargoExe,
    args: invocation.args,
    cwd: worktree,
    env: invocation.env,
    artifactDir,
    timeoutMs,
    taskkillExe,
  });
  // Once unconfirmed termination is observed, do not parse logs, copy
  // timings, or publish a derived Cargo artifact in this invocation.
  if (hasTerminationUnconfirmed(processResult)) assertCommandOk(processResult, label);
  const stdout = await readFile(processResult.stdoutPath, "utf8");
  const stderr = await readFile(processResult.stderrPath, "utf8");
  const parsed = parseCargoOutput(`${stdout}\n${stderr}`);
  let timingArtifact = null;
  if (diagnostic && processResult.classification === "ok") {
    const afterTimings = await timingFiles(worktree);
    const created = [...afterTimings].filter((name) => !beforeTimings.has(name));
    if (created.length !== 1) {
      throw new ProtocolError("timing_artifact_count", `expected one timing HTML, got ${created.length}`);
    }
    const source = path.join(worktree, "src-tauri", "target", "cargo-timings", created[0]);
    const target = path.join(artifactDir, "timings", `${label}.html`);
    await mkdir(path.dirname(target), { recursive: true });
    await copyFile(source, target, constants.COPYFILE_EXCL);
    timingArtifact = { path: target, sha256: await sha256File(target) };
  }
  const result = { ...processResult, ...parsed, timingArtifact };
  await writeAtomicJsonExclusive(path.join(artifactDir, "runs", `${label}.cargo.json`), result);
  return result;
}

export function assertCommandOk(result, label, failureKind = "command_failed") {
  if (hasTerminationUnconfirmed(result)) {
    throw new ProtocolError("termination_unconfirmed", label, {
      result,
      operatorActionRequired: true,
    });
  }
  if (
    result?.timedOut === true
    || result?.classification === "timeout"
  ) {
    throw new ProtocolError("command_timeout", label, { result });
  }
  if (result?.classification !== "ok") {
    throw new ProtocolError(failureKind, label, { result });
  }
}

async function fsyncPath(filePath, flags) {
  const handle = await open(filePath, flags);
  try {
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function restoreSourceFromRecovery({ sourcePath, recoveryPath }) {
  const recoveryBytes = await readFile(recoveryPath);
  const source = await open(sourcePath, "w");
  try {
    await source.writeFile(recoveryBytes);
    await source.sync();
  } finally {
    await source.close();
  }
}

export async function runDirtyCargoProbe({
  label,
  worktree,
  artifactDir,
  sourcePath,
  expectedCanonicalSha256,
  cargoExe,
  taskkillExe,
  timeoutMs,
  diagnostic = false,
  requireExtractum = false,
  runCargoFn = runCargoCheck,
  restoreSourceFn = restoreSourceFromRecovery,
  writeJsonFn = writeAtomicJsonExclusive,
}) {
  if (await sha256File(sourcePath) !== expectedCanonicalSha256) {
    throw new ProtocolError("canonical_hash_mismatch", label);
  }
  const shared = { worktree, artifactDir, cargoExe, taskkillExe, timeoutMs };
  const sync = await runCargoFn({ label: `${label}.sync`, diagnostic: false, ...shared });
  assertCommandOk(sync, label, "canonical_sync_failed");
  if (await sha256File(sourcePath) !== expectedCanonicalSha256) {
    throw new ProtocolError("canonical_hash_mismatch", `${label} after sync`);
  }

  const recoveryPath = path.join(artifactDir, "recovery", `${label}.lib.rs`);
  await mkdir(path.dirname(recoveryPath), { recursive: true });
  await copyFile(sourcePath, recoveryPath, constants.COPYFILE_EXCL);
  await fsyncPath(recoveryPath, "r");
  if (await sha256File(recoveryPath) !== expectedCanonicalSha256) {
    throw new ProtocolError("recovery_hash_mismatch", label);
  }

  let dirtyResult;
  let dirtyError = null;
  let recoveryPending = false;
  let restorationError = null;
  try {
    const source = await open(sourcePath, "a");
    try {
      await source.writeFile(PROTOCOL.probeSuffix, "utf8");
      await source.sync();
    } finally {
      await source.close();
    }
    dirtyResult = await runCargoFn({ label: `${label}.dirty`, diagnostic, ...shared });
    assertCommandOk(dirtyResult, label, "cargo_failed");
    if (requireExtractum && !dirtyResult.extractumChecked) {
      throw new ProtocolError("extractum_not_checked", label, { dirtyResult });
    }
  } catch (error) {
    dirtyError = error;
    recoveryPending = hasTerminationUnconfirmed(error);
  } finally {
    try {
      await restoreSourceFn({ sourcePath, recoveryPath });
    } catch (error) {
      restorationError = error;
    }
  }

  if (recoveryPending) {
    let pendingPublicationError = null;
    try {
      await writeJsonFn(
        path.join(artifactDir, "recovery", `${label}.recovery-pending.json`),
        {
        schema_version: 1,
        label,
        source_path: sourcePath,
        recovery_path: recoveryPath,
        canonical_sha256: expectedCanonicalSha256,
        recovery_sha256: await sha256File(recoveryPath).catch(() => null),
        source_restored_locally: await sha256File(sourcePath)
          .then((value) => value === expectedCanonicalSha256)
          .catch(() => false),
        operator_action_required: true,
        restoration_error: restorationError
          ? { kind: restorationError.kind ?? restorationError.name, message: restorationError.message }
          : null,
        },
      );
    } catch (error) {
      pendingPublicationError = error;
    }
    throw new ProtocolError("termination_unconfirmed", label, {
      operatorActionRequired: true,
      original: dirtyError?.details ?? { message: dirtyError?.message ?? String(dirtyError) },
      restorationError: restorationError
        ? { kind: restorationError.kind ?? restorationError.name, message: restorationError.message }
        : null,
      pendingPublicationError: pendingPublicationError
        ? { kind: pendingPublicationError.kind ?? pendingPublicationError.name, message: pendingPublicationError.message }
        : null,
    });
  }
  if (restorationError) throw restorationError;
  const restoredSha256 = await sha256File(sourcePath);
  if (restoredSha256 !== expectedCanonicalSha256) {
    throw new ProtocolError("source_restore_failed", label, { recoveryPath, restoredSha256 });
  }
  await writeJsonFn(path.join(artifactDir, "recovery", `${label}.restored.json`), {
    schema_version: 1,
    label,
    recovery_path: recoveryPath,
    canonical_sha256: expectedCanonicalSha256,
    restored_sha256: restoredSha256,
  });
  if (dirtyError) throw dirtyError;
  return dirtyResult;
}
```

- [ ] **Step 4: Run runtime GREEN twice**

Run twice to expose duplicate-label or leaked-process behavior:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/runtime.test.ts
if ($LASTEXITCODE -ne 0) { throw 'First runtime GREEN failed.' }
npm.cmd run test -- scripts/process-shell-diagnostic/runtime.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Second runtime GREEN failed.' }
```

Expected each time: one file, thirteen tests PASS; Task Manager shows no owned
hanging Node child. If the timeout test reports `termination_unconfirmed`,
inspect its taskkill logs before changing code; do not weaken the assertion
after seeing measurement data.

- [ ] **Step 5: Commit the runtime layer**

Run:

```powershell
$diagnosticTask2Status = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect Task 2 status.' }
$diagnosticTask2Status | ForEach-Object { Write-Output $_ }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 2 diff check failed.' }
git add -- scripts/process-shell-diagnostic/runtime.mjs scripts/process-shell-diagnostic/runtime.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Could not stage Task 2 files.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Task 2 staged diff check failed.' }
git commit -m "feat: add crash-safe cargo diagnostic runtime"
if ($LASTEXITCODE -ne 0) { throw 'Task 2 commit failed.' }
```

Expected: only the Task 2 runtime and test are committed.

### Task 3: Build Exact State Fixtures and Git Verification

**Files:**

- Create: `scripts/process-shell-diagnostic/git-state.test.ts`
- Create: `scripts/process-shell-diagnostic/git-state.mjs`
- Create: `scripts/process-shell-diagnostic/states/B.patch`
- Create: `scripts/process-shell-diagnostic/states/C.patch`
- Create: `scripts/process-shell-diagnostic/states/E.patch`

**Interfaces:**

- Consumes `PROTOCOL`, `ProtocolError`, `runWindowsProcess`,
  `writeAtomicJsonExclusive`, and `sha256File`.
- Produces `installState({ state, worktree, mainRoot, protocolLock,
  artifactDir })`, `verifyTargetIsolation({ metadata, worktree, mainRoot })`,
  `validateStateManifests(...)`, `validateLockDelta(...)`, and frozen Git anchors used by the freezer and
  attempt runner.

- [ ] **Step 1: Write the state-contract RED tests**

Create `scripts/process-shell-diagnostic/git-state.test.ts`:

```ts
import { mkdtemp, mkdir, readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  aRestoreArgs,
  D_BLOB_ANCHORS,
  STATE_TREE_ANCHORS,
  dCheckoutArgs,
  validateLockDelta,
  validateStateManifests,
  verifyTargetIsolation,
} from "./git-state.mjs";

const emptyManifest = `[package]
name = "extractum-process"
version.workspace = true
edition.workspace = true
publish = false
`;

const eManifest = `${emptyManifest}
[dependencies]
anyhow.workspace = true
parking_lot.workspace = true
tokio.workspace = true

[target.'cfg(windows)'.dependencies]
windows-sys.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
`;

const root = (edge: boolean, migrated: boolean) => `[workspace]
members = [".", "crates/extractum-core", "crates/extractum-process"]
resolver = "2"

[workspace.dependencies]
${migrated ? `anyhow = "1.0"
parking_lot = "0.12"
tokio = { version = "1", features = ["full"] }
windows-sys = { version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }
` : ""}
[dependencies]
${migrated ? `anyhow = { workspace = true }
parking_lot = { workspace = true }
tokio = { workspace = true }
` : ""}${edge ? `extractum-process = { path = "crates/extractum-process" }
` : ""}
[target.'cfg(windows)'.dependencies]
windows-sys = ${migrated ? `{ workspace = true }` : `{ version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }`}
`;

const processPaths = [
  "src-tauri/crates/extractum-process/Cargo.toml",
  "src-tauri/crates/extractum-process/src/lib.rs",
];

const lock = ({
  rootDependencies = ["anyhow", "serde"],
  processDependencies = null as null | string[],
  anyhowVersion = "1.0.100",
} = {}) => `version = 4

[[package]]
name = "anyhow"
version = "${anyhowVersion}"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fixed"

[[package]]
name = "extractum"
version = "0.2.0"
dependencies = [
${rootDependencies.map((name) => ` "${name}",`).join("\n")}
]

${processDependencies === null ? "" : `[[package]]
name = "extractum-process"
version = "0.2.0"
dependencies = [
${processDependencies.map((name) => ` "${name}",`).join("\n")}
]

`}[[package]]
name = "serde"
version = "1.0.228"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fixed-serde"
`;

describe("process shell diagnostic Git states", () => {
  it("freezes commit, subtree, and D blob anchors", () => {
    expect(STATE_TREE_ANCHORS).toEqual({
      A: "fd9711a041432ef420e7b09d56a46131a2a52a2a",
      D: "77e2d163ccc8bddf3ea051cb995909888cae9aba",
    });
    expect(D_BLOB_ANCHORS).toEqual({
      "src-tauri/Cargo.lock": "6368e32cd3a3853d4a7114ce256258e834bafdd4",
      "src-tauri/Cargo.toml": "c2037473a1257dd33a8e5b5fe81905e77dad084a",
      "src-tauri/crates/extractum-process/Cargo.toml": "3e078647dc293d95f401e15b8842776fae003ddb",
      "src-tauri/crates/extractum-process/src/child_process.rs": "9599017ed2ad826bc73f8e72f084042eacd8b58a",
      "src-tauri/crates/extractum-process/src/external_process.rs": "3cf7f073923b513381df09b7443090a4a41adc11",
      "src-tauri/crates/extractum-process/src/lib.rs": "4f7819ef7d2773b735b5edc61e162e4e034efb66",
      "src-tauri/crates/extractum-process/src/process_tree.rs": "365283e9f8accf4db91feca73bd8437db3b08c50",
      "src-tauri/src/lib.rs": "d84b653870eda9378c0d490894801850a97db68d",
    });
  });

  it("requires every generated B/C/E patch to carry a Cargo.lock text hunk", async () => {
    for (const state of ["B", "C", "E"]) {
      const patch = await readFile(new URL(`./states/${state}.patch`, import.meta.url), "utf8");
      expect(patch).toContain("diff --git a/src-tauri/Cargo.lock b/src-tauri/Cargo.lock");
      expect(patch).toMatch(/--- a\/src-tauri\/Cargo\.lock\r?\n\+\+\+ b\/src-tauri\/Cargo\.lock\r?\n@@ /);
    }
  });

  it("uses the exact approved A restore and D checkout commands", () => {
    expect(aRestoreArgs()).toEqual([
      "restore",
      "--source=24c313a767a25284123b24ea3a4b8c083007c817",
      "--staged",
      "--worktree",
      "--",
      "src-tauri",
    ]);
    expect(dCheckoutArgs()).toEqual([
      "checkout",
      "b364756c7b5768d644321afeaeb81ec04e2481a4",
      "--",
      "src-tauri",
    ]);
  });

  it("accepts B only with a dependency-free empty crate and no app edge", () => {
    expect(() => validateStateManifests({
      state: "B",
      rootManifest: root(false, false),
      processManifest: emptyManifest,
      processPaths,
    })).not.toThrow();
    expect(() => validateStateManifests({
      state: "B",
      rootManifest: root(true, false),
      processManifest: emptyManifest,
      processPaths,
    })).toThrow("B must not contain the app edge");
  });

  it("accepts C only with the path edge and the same empty crate", () => {
    expect(() => validateStateManifests({
      state: "C",
      rootManifest: root(true, false),
      processManifest: emptyManifest,
      processPaths,
    })).not.toThrow();
    expect(() => validateStateManifests({
      state: "C",
      rootManifest: root(false, false),
      processManifest: emptyManifest,
      processPaths,
    })).toThrow("C must contain exactly the app path edge");
  });

  it("accepts E's four named roots and only tokio/test-util as dev input", () => {
    expect(() => validateStateManifests({
      state: "E",
      rootManifest: root(true, true),
      processManifest: eManifest,
      processPaths,
    })).not.toThrow();
  });

  it("rejects an E manifest that omits target-specific windows-sys", () => {
    expect(() => validateStateManifests({
      state: "E",
      rootManifest: root(true, true),
      processManifest: eManifest.replace("windows-sys.workspace = true\n", ""),
      processPaths,
    })).toThrow("unexpected target dependency keys");
  });

  it("rejects any moved process source in B, C, or E", () => {
    expect(() => validateStateManifests({
      state: "E",
      rootManifest: root(true, true),
      processManifest: eManifest,
      processPaths: [...processPaths, "src-tauri/crates/extractum-process/src/process_tree.rs"],
    })).toThrow("unexpected process crate paths");
  });

  it("accepts only the state-local root/process lock delta", () => {
    const baselineLock = lock();
    expect(() => validateLockDelta({
      state: "B",
      baselineLock,
      stateLock: lock({ processDependencies: [] }),
    })).not.toThrow();
    expect(() => validateLockDelta({
      state: "C",
      baselineLock,
      stateLock: lock({ rootDependencies: ["anyhow", "extractum-process", "serde"], processDependencies: [] }),
    })).not.toThrow();
    expect(() => validateLockDelta({
      state: "E",
      baselineLock,
      stateLock: lock({
        rootDependencies: ["anyhow", "extractum-process", "serde"],
        processDependencies: ["anyhow", "parking_lot", "tokio", "windows-sys"],
      }),
    })).not.toThrow();
  });

  it("rejects any third-party lock resolution drift", () => {
    expect(() => validateLockDelta({
      state: "C",
      baselineLock: lock(),
      stateLock: lock({
        rootDependencies: ["anyhow", "extractum-process", "serde"],
        processDependencies: [],
        anyhowVersion: "1.0.101",
      }),
    })).toThrow("third-party lock package changed");
  });

  it("accepts only the exact worktree-local target directory", async () => {
    const parent = await mkdtemp(path.join(os.tmpdir(), "extractum-psd-target-"));
    const worktree = path.join(parent, "attempt");
    const mainRoot = path.join(parent, "main");
    await mkdir(path.join(worktree, "src-tauri", "target"), { recursive: true });
    await mkdir(path.join(mainRoot, "src-tauri", "target"), { recursive: true });
    await expect(verifyTargetIsolation({
      metadata: {
        workspace_root: path.join(worktree, "src-tauri"),
        target_directory: path.join(worktree, "src-tauri", "target"),
      },
      worktree,
      mainRoot,
    })).resolves.toBeUndefined();
    await expect(verifyTargetIsolation({
      metadata: {
        workspace_root: path.join(worktree, "src-tauri"),
        target_directory: path.join(mainRoot, "src-tauri", "target"),
      },
      worktree,
      mainRoot,
    })).rejects.toMatchObject({ kind: "target_not_isolated" });
  });
});
```

- [ ] **Step 2: Run state-contract RED**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/git-state.test.ts
```

Expected: FAIL during module resolution because `git-state.mjs` does not
exist. The source declares eleven cases, including the B/C/E
`Cargo.lock`-hunk contract; a zero-exit or skipped-suite run is not RED.

- [ ] **Step 3: Author the three complete A-to-state patches mechanically**

Perform this step only in the clean implementation worktree. Begin every
fixture from the frozen A bytes:

```powershell
git restore --source=24c313a767a25284123b24ea3a4b8c083007c817 --staged --worktree -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'Frozen A restore failed.' }
git diff --quiet -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'Frozen A worktree is not clean.' }
git diff --cached --quiet HEAD -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'Frozen A index is not clean.' }
New-Item -ItemType Directory -Force -Path 'scripts\process-shell-diagnostic\states' | Out-Null
```

For B, apply these exact source edits with `apply_patch`:

```diff
*** Begin Patch
*** Update File: src-tauri/Cargo.toml
@@
-members = [".", "crates/extractum-core"]
+members = [".", "crates/extractum-core", "crates/extractum-process"]
*** Add File: src-tauri/crates/extractum-process/Cargo.toml
+[package]
+name = "extractum-process"
+version.workspace = true
+edition.workspace = true
+publish = false
*** Add File: src-tauri/crates/extractum-process/src/lib.rs
+#![forbid(unsafe_code)]
+
+// Membership/edge diagnostic fixture intentionally has no public API.
*** End Patch
```

Generate only the corresponding lockfile change, stage the complete state,
emit a full-index patch without shell redirection, then reverse that exact
patch back to A:

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'B lock resolution failed.' }
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'B lock is not stable under --locked.' }
git diff -- src-tauri/Cargo.lock
if ($LASTEXITCODE -ne 0) { throw 'B lock diff inspection failed.' }
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked
if ($LASTEXITCODE -ne 0) { throw 'B final locked metadata failed.' }
git add -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'B staging failed.' }
git diff --cached --binary --full-index --output=scripts/process-shell-diagnostic/states/B.patch -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'B patch generation failed.' }
git apply -R --index scripts/process-shell-diagnostic/states/B.patch
if ($LASTEXITCODE -ne 0) { throw 'B reverse application failed.' }
git diff --quiet -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'B reverse application left a worktree diff.' }
git diff --cached --quiet HEAD -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'B reverse application left an index diff.' }
```

Expected: metadata lists three workspace members; B's generated lock entry for
`extractum-process` has no dependency array; both final quiet checks exit 0.

For C, apply this complete A-to-C edit with `apply_patch`:

```diff
*** Begin Patch
*** Update File: src-tauri/Cargo.toml
@@
-members = [".", "crates/extractum-core"]
+members = [".", "crates/extractum-core", "crates/extractum-process"]
@@
 extractum-core = { path = "crates/extractum-core" }
+extractum-process = { path = "crates/extractum-process" }
*** Add File: src-tauri/crates/extractum-process/Cargo.toml
+[package]
+name = "extractum-process"
+version.workspace = true
+edition.workspace = true
+publish = false
*** Add File: src-tauri/crates/extractum-process/src/lib.rs
+#![forbid(unsafe_code)]
+
+// Membership/edge diagnostic fixture intentionally has no public API.
*** End Patch
```

Then generate and reverse C exactly as B:

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'C lock resolution failed.' }
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'C lock is not stable under --locked.' }
git diff -- src-tauri/Cargo.lock
if ($LASTEXITCODE -ne 0) { throw 'C lock diff inspection failed.' }
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked
if ($LASTEXITCODE -ne 0) { throw 'C final locked metadata failed.' }
git add -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'C staging failed.' }
git diff --cached --binary --full-index --output=scripts/process-shell-diagnostic/states/C.patch -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'C patch generation failed.' }
git apply -R --index scripts/process-shell-diagnostic/states/C.patch
if ($LASTEXITCODE -ne 0) { throw 'C reverse application failed.' }
git diff --quiet -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'C reverse application left a worktree diff.' }
git diff --cached --quiet HEAD -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'C reverse application left an index diff.' }
```

Expected: the app package has one direct `extractum-process` path edge; the
process package itself still has no dependencies; both final quiet checks exit
0.

For E, apply the complete A-to-E manifest-only edit below. It intentionally
keeps the app's Windows edge and creates only the same empty `src/lib.rs` used
by B/C:

```diff
*** Begin Patch
*** Update File: src-tauri/Cargo.toml
@@
-members = [".", "crates/extractum-core"]
+members = [".", "crates/extractum-core", "crates/extractum-process"]
@@
 [workspace.dependencies]
+anyhow = "1.0"
+parking_lot = "0.12"
 serde = { version = "1", features = ["derive"] }
 serde_json = "1"
 time = { version = "0.3", features = ["formatting", "parsing", "macros"] }
+tokio = { version = "1", features = ["full"] }
+windows-sys = { version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }
 zstd = "0.13"
@@
-parking_lot = "0.12"
+parking_lot = { workspace = true }
@@
-tokio = { version = "1", features = ["full"] }
+tokio = { workspace = true }
 tokio-util = "0.7"
-anyhow = "1.0"
+anyhow = { workspace = true }
@@
 extractum-core = { path = "crates/extractum-core" }
+extractum-process = { path = "crates/extractum-process" }
@@
 [dev-dependencies]
-tokio = { version = "1", features = ["test-util"] }
+tokio = { workspace = true, features = ["test-util"] }
@@
 [target.'cfg(windows)'.dependencies]
-windows-sys = { version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }
+windows-sys = { workspace = true }
*** Add File: src-tauri/crates/extractum-process/Cargo.toml
+[package]
+name = "extractum-process"
+version.workspace = true
+edition.workspace = true
+publish = false
+
+[dependencies]
+anyhow.workspace = true
+parking_lot.workspace = true
+tokio.workspace = true
+
+[target.'cfg(windows)'.dependencies]
+windows-sys.workspace = true
+
+[dev-dependencies]
+tokio = { workspace = true, features = ["test-util"] }
*** Add File: src-tauri/crates/extractum-process/src/lib.rs
+#![forbid(unsafe_code)]
+
+// Membership/edge diagnostic fixture intentionally has no public API.
*** End Patch
```

Generate and reverse E:

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'E lock resolution failed.' }
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'E lock is not stable under --locked.' }
git diff -- src-tauri/Cargo.lock
if ($LASTEXITCODE -ne 0) { throw 'E lock diff inspection failed.' }
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked
if ($LASTEXITCODE -ne 0) { throw 'E final locked metadata failed.' }
git add -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'E staging failed.' }
git diff --cached --binary --full-index --output=scripts/process-shell-diagnostic/states/E.patch -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'E patch generation failed.' }
git apply -R --index scripts/process-shell-diagnostic/states/E.patch
if ($LASTEXITCODE -ne 0) { throw 'E reverse application failed.' }
git diff --quiet -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'E reverse application left a worktree diff.' }
git diff --cached --quiet HEAD -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'E reverse application left an index diff.' }
git apply --check --index scripts/process-shell-diagnostic/states/B.patch
if ($LASTEXITCODE -ne 0) { throw 'B patch no longer applies to A.' }
git apply --check --index scripts/process-shell-diagnostic/states/C.patch
if ($LASTEXITCODE -ne 0) { throw 'C patch no longer applies to A.' }
git apply --check --index scripts/process-shell-diagnostic/states/E.patch
if ($LASTEXITCODE -ne 0) { throw 'E patch no longer applies to A.' }
```

Expected: E metadata contains all four named dependency roots on the empty
crate, the app retains its inherited `windows-sys` edge, no process source has
moved, and all six quiet/check commands exit 0.

For B/C/E, the resolver-capable metadata call is used only to minimally extend
the existing baseline lock. Both the generating call and its `--locked`
verification intentionally omit `--no-deps`; that option suppresses the
resolver and cannot materialize or verify a lock delta. `cargo generate-lockfile`
is forbidden because it can re-resolve unrelated packages. Before Task 3
commits, the `validateLockDelta` tests and production check below must prove
that every third-party package block (name, version, source, checksum, and
dependencies) is byte-identical to baseline. Only the `extractum` and new
`extractum-process` package records may differ according to the state contract.

- [ ] **Step 4: Implement manifest, tree, inventory, and target verification**

Create `scripts/process-shell-diagnostic/git-state.mjs`:

```js
import { lstat, readFile, realpath } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

import { PROTOCOL } from "./protocol.mjs";
import {
  assertCommandOk,
  ProtocolError,
  runWindowsProcess,
  sha256File,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

export const STATE_TREE_ANCHORS = Object.freeze({
  A: "fd9711a041432ef420e7b09d56a46131a2a52a2a",
  D: "77e2d163ccc8bddf3ea051cb995909888cae9aba",
});

export const D_BLOB_ANCHORS = Object.freeze({
  "src-tauri/Cargo.lock": "6368e32cd3a3853d4a7114ce256258e834bafdd4",
  "src-tauri/Cargo.toml": "c2037473a1257dd33a8e5b5fe81905e77dad084a",
  "src-tauri/crates/extractum-process/Cargo.toml": "3e078647dc293d95f401e15b8842776fae003ddb",
  "src-tauri/crates/extractum-process/src/child_process.rs": "9599017ed2ad826bc73f8e72f084042eacd8b58a",
  "src-tauri/crates/extractum-process/src/external_process.rs": "3cf7f073923b513381df09b7443090a4a41adc11",
  "src-tauri/crates/extractum-process/src/lib.rs": "4f7819ef7d2773b735b5edc61e162e4e034efb66",
  "src-tauri/crates/extractum-process/src/process_tree.rs": "365283e9f8accf4db91feca73bd8437db3b08c50",
  "src-tauri/src/lib.rs": "d84b653870eda9378c0d490894801850a97db68d",
});

const WINDOWS_TABLE = "target.'cfg(windows)'.dependencies";
const PROCESS_PATHS = [
  "src-tauri/crates/extractum-process/Cargo.toml",
  "src-tauri/crates/extractum-process/src/lib.rs",
];

export function aRestoreArgs() {
  return [
    "restore",
    `--source=${PROTOCOL.baselineCommit}`,
    "--staged",
    "--worktree",
    "--",
    "src-tauri",
  ];
}

export function dCheckoutArgs() {
  return ["checkout", PROTOCOL.candidateCommit, "--", "src-tauri"];
}

function normalizedState(state) {
  if (/^A(?:\d+|-final)?$/.test(state)) return "A";
  if (["B", "C", "D", "E"].includes(state)) return state;
  throw new ProtocolError("unknown_state", state);
}

export function parseTomlSections(text) {
  const sections = new Map([["", new Map()]]);
  let current = "";
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (line === "" || line.startsWith("#")) continue;
    const header = line.match(/^\[([^\]]+)\]$/);
    if (header) {
      current = header[1];
      if (!sections.has(current)) sections.set(current, new Map());
      continue;
    }
    const equals = line.indexOf("=");
    if (equals < 1) throw new Error(`unsupported TOML line: ${rawLine}`);
    const key = line.slice(0, equals).trim();
    const value = line.slice(equals + 1).trim();
    if (sections.get(current).has(key)) throw new Error(`duplicate TOML key ${current}.${key}`);
    sections.get(current).set(key, value);
  }
  return sections;
}

function table(sections, name) {
  return sections.get(name) ?? new Map();
}

function exactKeys(entries, expected, label) {
  const actual = [...entries.keys()].sort();
  const wanted = [...expected].sort();
  if (JSON.stringify(actual) !== JSON.stringify(wanted)) {
    throw new Error(`unexpected ${label} keys: ${actual.join(",")}`);
  }
}

function requireValue(entries, key, value, label) {
  if (entries.get(key) !== value) {
    throw new Error(`${label}.${key} must be ${value}, got ${entries.get(key) ?? "missing"}`);
  }
}

function lockPackages(text) {
  const normalized = text.replaceAll("\r\n", "\n");
  const firstPackage = normalized.indexOf("[[package]]");
  if (firstPackage < 0) throw new Error("Cargo.lock has no package records");
  const preamble = normalized.slice(0, firstPackage).trim();
  const blocks = normalized.slice(firstPackage).split(/(?=^\[\[package\]\]$)/m).filter(Boolean);
  const records = blocks.map((block) => {
    const name = block.match(/^name = "([^"]+)"$/m)?.[1];
    const version = block.match(/^version = "([^"]+)"$/m)?.[1];
    const source = block.match(/^source = "([^"]+)"$/m)?.[1] ?? "workspace";
    if (!name || !version) throw new Error("Cargo.lock package lacks name/version");
    const dependencyBody = block.match(/^dependencies = \[\n([\s\S]*?)^\]$/m)?.[1] ?? "";
    const dependencies = [...dependencyBody.matchAll(/"([^"]+)"/g)]
      .map((match) => match[1].split(" ")[0])
      .sort();
    return { key: `${name}\0${version}\0${source}`, name, version, source, block: block.trim(), dependencies };
  });
  return { preamble, records };
}

function onlyPackage(records, name) {
  const matches = records.filter((record) => record.name === name);
  if (matches.length !== 1) throw new Error(`expected one ${name} lock package, got ${matches.length}`);
  return matches[0];
}

function sameValues(actual, expected, label) {
  if (JSON.stringify([...actual].sort()) !== JSON.stringify([...expected].sort())) {
    throw new Error(`${label}: ${JSON.stringify(actual)} != ${JSON.stringify(expected)}`);
  }
}

export function validateLockDelta({ state, baselineLock, stateLock }) {
  const baseline = lockPackages(baselineLock);
  const candidate = lockPackages(stateLock);
  if (baseline.preamble !== candidate.preamble) throw new Error("Cargo.lock preamble changed");
  const excluded = new Set(["extractum", "extractum-process"]);
  const baselineThirdParty = new Map(
    baseline.records.filter((record) => !excluded.has(record.name)).map((record) => [record.key, record.block]),
  );
  const candidateThirdParty = new Map(
    candidate.records.filter((record) => !excluded.has(record.name)).map((record) => [record.key, record.block]),
  );
  if (JSON.stringify([...baselineThirdParty].sort()) !== JSON.stringify([...candidateThirdParty].sort())) {
    throw new Error("third-party lock package changed");
  }
  if (baseline.records.some((record) => record.name === "extractum-process")) {
    throw new Error("baseline unexpectedly contains extractum-process");
  }
  const baselineRoot = onlyPackage(baseline.records, "extractum");
  const candidateRoot = onlyPackage(candidate.records, "extractum");
  const processPackage = onlyPackage(candidate.records, "extractum-process");
  const expectedRoot = state === "B"
    ? baselineRoot.dependencies
    : [...baselineRoot.dependencies, "extractum-process"];
  sameValues(candidateRoot.dependencies, expectedRoot, `${state} root lock dependencies`);
  const expectedProcess = state === "E" ? ["anyhow", "parking_lot", "tokio", "windows-sys"] : [];
  sameValues(processPackage.dependencies, expectedProcess, `${state} process lock dependencies`);
}

export function validateStateManifests({
  state,
  rootManifest,
  processManifest,
  processPaths,
}) {
  if (!["B", "C", "E"].includes(state)) return;
  const rootSections = parseTomlSections(rootManifest);
  const processSections = parseTomlSections(processManifest);
  requireValue(
    table(rootSections, "workspace"),
    "members",
    '[".", "crates/extractum-core", "crates/extractum-process"]',
    "workspace",
  );
  if (JSON.stringify([...processPaths].sort()) !== JSON.stringify(PROCESS_PATHS)) {
    throw new Error(`unexpected process crate paths: ${processPaths.join(",")}`);
  }

  const packageTable = table(processSections, "package");
  exactKeys(packageTable, ["name", "version.workspace", "edition.workspace", "publish"], "package");
  requireValue(packageTable, "name", '"extractum-process"', "package");
  requireValue(packageTable, "version.workspace", "true", "package");
  requireValue(packageTable, "edition.workspace", "true", "package");
  requireValue(packageTable, "publish", "false", "package");

  const appDependencies = table(rootSections, "dependencies");
  const processDependencies = table(processSections, "dependencies");
  const processTargetDependencies = table(processSections, WINDOWS_TABLE);
  const processDevDependencies = table(processSections, "dev-dependencies");

  if (state === "B") {
    if (appDependencies.has("extractum-process")) throw new Error("B must not contain the app edge");
    exactKeys(processDependencies, [], "dependency");
    exactKeys(processTargetDependencies, [], "target dependency");
    exactKeys(processDevDependencies, [], "dev dependency");
    return;
  }

  if (state === "C") {
    if (appDependencies.get("extractum-process") !== '{ path = "crates/extractum-process" }') {
      throw new Error("C must contain exactly the app path edge");
    }
    exactKeys(processDependencies, [], "dependency");
    exactKeys(processTargetDependencies, [], "target dependency");
    exactKeys(processDevDependencies, [], "dev dependency");
    return;
  }

  requireValue(
    appDependencies,
    "extractum-process",
    '{ path = "crates/extractum-process" }',
    "dependencies",
  );

  const workspaceDependencies = table(rootSections, "workspace.dependencies");
  requireValue(workspaceDependencies, "anyhow", '"1.0"', "workspace.dependencies");
  requireValue(workspaceDependencies, "parking_lot", '"0.12"', "workspace.dependencies");
  requireValue(workspaceDependencies, "tokio", '{ version = "1", features = ["full"] }', "workspace.dependencies");
  requireValue(
    workspaceDependencies,
    "windows-sys",
    '{ version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }',
    "workspace.dependencies",
  );
  for (const key of ["anyhow", "parking_lot", "tokio"]) {
    requireValue(appDependencies, key, "{ workspace = true }", "dependencies");
  }
  requireValue(table(rootSections, WINDOWS_TABLE), "windows-sys", "{ workspace = true }", WINDOWS_TABLE);
  exactKeys(processDependencies, ["anyhow.workspace", "parking_lot.workspace", "tokio.workspace"], "dependency");
  for (const key of ["anyhow.workspace", "parking_lot.workspace", "tokio.workspace"]) {
    requireValue(processDependencies, key, "true", "dependencies");
  }
  exactKeys(processTargetDependencies, ["windows-sys.workspace"], "target dependency");
  requireValue(processTargetDependencies, "windows-sys.workspace", "true", WINDOWS_TABLE);
  exactKeys(processDevDependencies, ["tokio"], "dev dependency");
  requireValue(
    processDevDependencies,
    "tokio",
    '{ workspace = true, features = ["test-util"] }',
    "dev-dependencies",
  );
}

function normalizedPath(value) {
  return path.resolve(value).replaceAll("/", "\\").toLowerCase();
}

async function rejectReparsePoint(candidate) {
  try {
    const information = await lstat(candidate);
    if (information.isSymbolicLink()) {
      throw new ProtocolError("target_not_isolated", `reparse point is forbidden: ${candidate}`);
    }
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
}

export async function verifyTargetIsolation({ metadata, worktree, mainRoot }) {
  const targetOverride = Object.keys(process.env).find(
    (key) => key.toUpperCase() === "CARGO_TARGET_DIR",
  );
  if (targetOverride !== undefined) {
    throw new ProtocolError("target_not_isolated", `CARGO_TARGET_DIR is set as ${targetOverride}`);
  }
  const workspaceRoot = metadata.workspace_root;
  const targetDirectory = metadata.target_directory;
  if (typeof workspaceRoot !== "string" || typeof targetDirectory !== "string") {
    throw new ProtocolError("target_metadata_missing", "cargo metadata omitted workspace_root or target_directory");
  }
  const expectedWorkspace = path.join(worktree, "src-tauri");
  const expected = path.join(worktree, "src-tauri", "target");
  const mainTarget = path.join(mainRoot, "src-tauri", "target");
  if (normalizedPath(workspaceRoot) !== normalizedPath(expectedWorkspace)) {
    throw new ProtocolError("target_not_isolated", `expected workspace ${expectedWorkspace}, got ${workspaceRoot}`);
  }
  if (normalizedPath(targetDirectory) !== normalizedPath(expected)) {
    throw new ProtocolError("target_not_isolated", `expected ${expected}, got ${targetDirectory}`);
  }
  if (normalizedPath(targetDirectory) === normalizedPath(mainTarget)) {
    throw new ProtocolError("target_not_isolated", "attempt target equals main target");
  }
  await rejectReparsePoint(path.join(worktree, "src-tauri"));
  await rejectReparsePoint(expected);
  try {
    const [resolvedTarget, resolvedMain] = await Promise.all([realpath(expected), realpath(mainTarget)]);
    if (normalizedPath(resolvedTarget) === normalizedPath(resolvedMain)) {
      throw new ProtocolError("target_not_isolated", "attempt target resolves to main target");
    }
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
}

function taskkillExe() {
  if (!process.env.SystemRoot) throw new ProtocolError("missing_system_root", "SystemRoot is required");
  return path.join(process.env.SystemRoot, "System32", "taskkill.exe");
}

async function git({ args, label, worktree, artifactDir, rawOutput = false }) {
  const result = await runWindowsProcess({
    label,
    command: "git.exe",
    args,
    cwd: worktree,
    env: process.env,
    artifactDir,
    timeoutMs: PROTOCOL.commandTimeoutMs,
    taskkillExe: taskkillExe(),
  });
  if (result.classification !== "ok") {
    throw new ProtocolError("git_command_failed", args.join(" "), { result });
  }
  if (rawOutput) return result.stdoutPath;
  return (await readFile(result.stdoutPath, "utf8")).trim().replaceAll("\r\n", "\n");
}

async function verifyOnlySrcTauriChanged(spec) {
  const names = new Set();
  for (const [suffix, args] of [
    ["unstaged", ["diff", "--name-only", "HEAD"]],
    ["staged", ["diff", "--cached", "--name-only", "HEAD"]],
  ]) {
    const output = await git({ ...spec, label: `${spec.label}.outside.${suffix}`, args });
    for (const name of output.split("\n").filter(Boolean)) names.add(name);
  }
  const outside = [...names].filter((name) => !name.startsWith("src-tauri/"));
  if (outside.length > 0) throw new ProtocolError("state_changed_outside_src_tauri", outside.join(","));
}

export async function installState({ state, worktree, mainRoot, protocolLock, artifactDir }) {
  const [resolvedWorktree, resolvedMainRoot] = await Promise.all([
    realpath(worktree),
    realpath(mainRoot),
  ]);
  if (normalizedPath(resolvedWorktree) === normalizedPath(resolvedMainRoot)) {
    throw new ProtocolError("worktree_not_isolated", "state installation cannot run in main");
  }
  const kind = normalizedState(state);
  const prefix = `state-${state}`;
  const shared = { worktree, artifactDir, label: prefix };
  await git({
    ...shared,
    label: `${prefix}.restore-a`,
    args: aRestoreArgs(),
  });

  if (["B", "C", "E"].includes(kind)) {
    const patchRelative = `scripts/process-shell-diagnostic/states/${kind}.patch`;
    const patchPath = await git({
      ...shared,
      label: `${prefix}.canonical-patch-blob`,
      args: ["cat-file", "blob", `HEAD:${patchRelative}`],
      rawOutput: true,
    });
    const actualPatchSha256 = await sha256File(patchPath);
    if (actualPatchSha256 !== protocolLock.states[kind].patchSha256) {
      throw new ProtocolError("state_patch_hash_mismatch", `${kind}: ${actualPatchSha256}`);
    }
    await git({ ...shared, label: `${prefix}.patch-check`, args: ["apply", "--check", "--index", patchPath] });
    await git({ ...shared, label: `${prefix}.patch-apply`, args: ["apply", "--index", patchPath] });
  } else if (kind === "D") {
    await git({ ...shared, label: `${prefix}.candidate-checkout`, args: dCheckoutArgs() });
  }

  await verifyOnlySrcTauriChanged(shared);
  await git({ ...shared, label: `${prefix}.worktree-index-clean`, args: ["diff", "--quiet"] });
  const rootTree = await git({ ...shared, label: `${prefix}.write-tree`, args: ["write-tree"] });
  const srcTauriTree = await git({
    ...shared,
    label: `${prefix}.subtree`,
    args: ["rev-parse", `${rootTree}:src-tauri`],
  });
  const expectedTree = protocolLock.states[kind].srcTauriTree;
  if (srcTauriTree !== expectedTree) {
    throw new ProtocolError("state_tree_mismatch", `${kind}: ${srcTauriTree} != ${expectedTree}`);
  }
  if (kind === "A" && srcTauriTree !== STATE_TREE_ANCHORS.A) {
    throw new ProtocolError("baseline_tree_mismatch", srcTauriTree);
  }

  const processPathsText = await git({
    ...shared,
    label: `${prefix}.process-paths`,
    args: ["ls-files", "--", "src-tauri/crates/extractum-process"],
  });
  const processPaths = processPathsText.split("\n").filter(Boolean).sort();
  if (["B", "C", "E"].includes(kind)) {
    const rootManifest = await readFile(path.join(worktree, "src-tauri", "Cargo.toml"), "utf8");
    const processManifest = await readFile(
      path.join(worktree, "src-tauri", "crates", "extractum-process", "Cargo.toml"),
      "utf8",
    );
    validateStateManifests({ state: kind, rootManifest, processManifest, processPaths });
    const baselineLockPath = await git({
      ...shared,
      label: `${prefix}.baseline-lock-blob`,
      args: ["cat-file", "blob", `${PROTOCOL.baselineCommit}:src-tauri/Cargo.lock`],
      rawOutput: true,
    });
    validateLockDelta({
      state: kind,
      baselineLock: await readFile(baselineLockPath, "utf8"),
      stateLock: await readFile(path.join(worktree, "src-tauri", "Cargo.lock"), "utf8"),
    });
  }

  if (kind === "D") {
    if (srcTauriTree !== STATE_TREE_ANCHORS.D) throw new ProtocolError("candidate_tree_mismatch", srcTauriTree);
    const expectedInventory = await git({
      ...shared,
      label: `${prefix}.expected-inventory`,
      args: ["ls-tree", "-r", PROTOCOL.candidateCommit, "--", "src-tauri"],
    });
    const actualInventory = await git({
      ...shared,
      label: `${prefix}.actual-inventory`,
      args: ["ls-tree", "-r", rootTree, "--", "src-tauri"],
    });
    if (actualInventory !== expectedInventory) {
      throw new ProtocolError("candidate_inventory_mismatch", "D path/mode/blob inventory differs");
    }
    for (const [filePath, blob] of Object.entries(D_BLOB_ANCHORS)) {
      if (!actualInventory.includes(`100644 blob ${blob}\t${filePath}`)) {
        throw new ProtocolError("candidate_blob_mismatch", `${filePath}:${blob}`);
      }
    }
    await git({
      ...shared,
      label: `${prefix}.required-diff`,
      args: ["diff", "--quiet", PROTOCOL.candidateCommit, "--", "src-tauri"],
    });
    await git({
      ...shared,
      label: `${prefix}.cached-diff`,
      args: ["diff", "--cached", "--quiet", PROTOCOL.candidateCommit, "--", "src-tauri"],
    });
  }

  const sourcePath = path.join(worktree, "src-tauri", "src", "lib.rs");
  const evidence = {
    schemaVersion: 1,
    state,
    kind,
    mainRoot,
    worktree,
    rootTree,
    srcTauriTree,
    canonicalLibSha256: await sha256File(sourcePath),
    processPaths,
  };
  await writeAtomicJsonExclusive(path.join(artifactDir, "states", `${state}.json`), evidence);
  return evidence;
}
```

- [ ] **Step 5: Run state GREEN and patch applicability checks**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/git-state.test.ts
if ($LASTEXITCODE -ne 0) { throw 'State GREEN failed.' }
git apply --check --index scripts/process-shell-diagnostic/states/B.patch
if ($LASTEXITCODE -ne 0) { throw 'B patch applicability check failed.' }
git apply --check --index scripts/process-shell-diagnostic/states/C.patch
if ($LASTEXITCODE -ne 0) { throw 'C patch applicability check failed.' }
git apply --check --index scripts/process-shell-diagnostic/states/E.patch
if ($LASTEXITCODE -ne 0) { throw 'E patch applicability check failed.' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 3 diff check failed.' }
```

Expected: one file, eleven tests PASS; every patch contains a `Cargo.lock`
hunk; all three patches apply cleanly to A;
the lock contract rejects any third-party package drift; `git diff --check`
prints nothing.

- [ ] **Step 6: Commit the frozen state definitions**

Run:

```powershell
$diagnosticTask3Status = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect Task 3 status.' }
$diagnosticTask3Status | ForEach-Object { Write-Output $_ }
git add -- scripts/process-shell-diagnostic/git-state.mjs scripts/process-shell-diagnostic/git-state.test.ts scripts/process-shell-diagnostic/states/B.patch scripts/process-shell-diagnostic/states/C.patch scripts/process-shell-diagnostic/states/E.patch
if ($LASTEXITCODE -ne 0) { throw 'Could not stage Task 3 files.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Task 3 staged diff check failed.' }
git commit -m "test: freeze process shell diagnostic states"
if ($LASTEXITCODE -ne 0) { throw 'Task 3 commit failed.' }
```

Expected: only Task 3's verifier, tests, and three complete patches are
committed; tracked `src-tauri` remains A.

### Task 4: Orchestrate One Complete Measurement Attempt

**Files:**

- Create: `scripts/process-shell-diagnostic/attempt.test.ts`
- Create: `scripts/process-shell-diagnostic/attempt.mjs`

**Interfaces:**

- Consumes state installation, target verification, runtime probes, and the
  pure protocol evaluator.
- Produces `runAttempt({ worktree, mainRoot, sessionDir, attemptId,
  protocolLock }, deps?) -> AttemptResult`. Dependency injection exists only
  for deterministic tests; production uses the Task 1–3 functions.
- An attempt writes once beneath `<sessionDir>/attempts/<attemptId>` and always
  installs/verifies `A-final` before returning any result.

- [ ] **Step 1: Write attempt-order RED tests**

Create `scripts/process-shell-diagnostic/attempt.test.ts`:

```ts
import path from "node:path";
import { describe, expect, it } from "vitest";

import { evaluateAttempt } from "./protocol.mjs";
import { runAttempt } from "./attempt.mjs";

function fixture(overrides: Record<string, unknown> = {}) {
  const calls: Array<Record<string, unknown>> = [];
  const writes: Array<{ target: string; value: Record<string, unknown> }> = [];
  const stateValues: Record<string, number> = {
    A0: 9_000,
    B: 9_000,
    A1: 9_000,
    C: 9_000,
    A2: 9_000,
    D: 9_000,
    A3: 9_000,
    E: 9_000,
    A4: 9_000,
  };
  Object.assign(stateValues, overrides.stateValues ?? {});
  let currentBlock = "A0";

  const deps = {
    installStateFn: async ({ state }: { state: string }) => {
      currentBlock = state === "A-final" ? currentBlock : state;
      calls.push({ kind: "install", state });
      return {
        state,
        kind: state.startsWith("A") ? "A" : state,
        srcTauriTree: state.startsWith("A") ? "a-tree" : `${state}-tree`,
        canonicalLibSha256: `${state}-sha`,
      };
    },
    captureStateInventoryFn: async ({ block }: { block: string }) => {
      calls.push({ kind: "inventory", block });
      return {
        metadata: { target_directory: `G:\\attempt\\src-tauri\\target` },
        extractumProcessDirectDependency: ["C", "D", "E"].includes(block),
      };
    },
    verifyTargetPreflightFn: async ({ block }: { block: string }) => {
      calls.push({ kind: "target-preflight", block });
      return { targetDirectory: "G:\\attempt\\src-tauri\\target" };
    },
    runCargoCheckFn: async ({ label }: { label: string }) => {
      calls.push({ kind: "cargo", label });
      return overrides.cargoResult ?? {
        classification: "ok",
        elapsedMs: 50,
        cargoReportedMs: 40,
        closeObserved: true,
      };
    },
    runDirtyProbeFn: async ({ label, diagnostic }: { label: string; diagnostic: boolean }) => {
      calls.push({ kind: "dirty", label, diagnostic });
      if (overrides.failLabel === label) throw Object.assign(new Error("forced"), { kind: "forced_failure" });
      if (overrides.timeoutLabel === label) {
        throw Object.assign(new Error("timed out"), {
          kind: "cargo_failed",
          details: { dirtyResult: { classification: "timeout", timedOut: true } },
        });
      }
      if (overrides.terminationLabel === label) {
        throw Object.assign(new Error("termination unconfirmed"), {
          kind: "cargo_failed",
          details: { dirtyResult: { classification: "termination_unconfirmed", timedOut: true } },
        });
      }
      return {
        classification: "ok",
        elapsedMs: stateValues[currentBlock],
        cargoReportedMs: stateValues[currentBlock] - 20,
        extractumChecked: overrides.missingCheckedLabel !== label,
        extractumLibRustcObserved: true,
        extractumProcessExtern: ["C", "D", "E"].includes(currentBlock),
        closeObserved: true,
        timingArtifact: diagnostic ? { path: `${label}.html`, sha256: "f".repeat(64) } : null,
      };
    },
    evaluateAttemptFn: evaluateAttempt,
    writeJsonFn: async (target: string, value: Record<string, unknown>) => {
      writes.push({ target, value });
    },
  };
  return { calls, deps, writes };
}

const spec = {
  worktree: "G:\\attempt",
  mainRoot: "G:\\main",
  sessionDir: "G:\\artifacts",
  attemptId: "attempt-001",
  protocolLock: {
    states: {
      A: { srcTauriTree: "a-tree" },
      B: { srcTauriTree: "B-tree" },
      C: { srcTauriTree: "C-tree" },
      D: { srcTauriTree: "D-tree" },
      E: { srcTauriTree: "E-tree" },
    },
  },
};

describe("process shell diagnostic attempt", () => {
  it("runs the fixed seven-block sequence and restores A", async () => {
    const value = fixture();
    const result = await runAttempt(spec, value.deps);
    expect(result.kind).toBe("valid");
    expect(result.evaluation.classification).toBe("not_reproduced");
    expect(value.calls.filter((call) => call.kind === "install").map((call) => call.state)).toEqual([
      "A0", "B", "A1", "C", "A2", "D", "A3", "A-final",
    ]);
    for (const block of ["A0", "B", "A1", "C", "A2", "D", "A3"]) {
      const dirty = value.calls.filter((call) => call.kind === "dirty" && String(call.label).startsWith(`${block}.`));
      expect(dirty).toHaveLength(10);
      expect(dirty.filter((call) => call.diagnostic)).toEqual([
        expect.objectContaining({ label: `${block}.diagnostic`, diagnostic: true }),
      ]);
      expect(value.calls).toEqual(expect.arrayContaining([
        expect.objectContaining({ kind: "cargo", label: `${block}.inventory-sync` }),
        expect.objectContaining({ kind: "cargo", label: `${block}.noop-sync` }),
        expect.objectContaining({ kind: "cargo", label: `${block}.noop` }),
      ]));
      const inventoryIndex = value.calls.findIndex((call) => call.kind === "inventory" && call.block === block);
      const preflightIndex = value.calls.findIndex((call) =>
        call.kind === "target-preflight" && call.block === block,
      );
      const canonicalSyncIndex = value.calls.findIndex((call) =>
        call.kind === "cargo" && call.label === `${block}.inventory-sync`,
      );
      expect(preflightIndex).toBeLessThan(canonicalSyncIndex);
      expect(canonicalSyncIndex).toBeLessThan(inventoryIndex);
      expect(result.blocks[block].inventory.extractumProcessDirectDependency).toBe(["C", "D", "E"].includes(block));
      expect(result.blocks[block].diagnostic.extractumProcessExtern).toBe(["C", "D", "E"].includes(block));
    }
  });

  it("appends E and A4 only after D crosses while B and C stay fast", async () => {
    const value = fixture({ stateValues: { D: 9_600 } });
    const result = await runAttempt(spec, value.deps);
    expect(result.kind).toBe("valid");
    expect(result.evaluation).toMatchObject({ eRequired: true, classification: "boundary_composite" });
    expect(value.calls.filter((call) => call.kind === "install").map((call) => call.state)).toEqual([
      "A0", "B", "A1", "C", "A2", "D", "A3", "E", "A4", "A-final",
    ]);
    expect(result.blocks.E.diagnostic.extractumProcessExtern).toBe(true);
  });

  it("retains all seven samples and never replaces one", async () => {
    const value = fixture();
    const result = await runAttempt(spec, value.deps);
    expect(result.blocks.B.samples.map((sample: { wallMs: number }) => sample.wallMs)).toEqual(Array(7).fill(9_000));
    expect(result.blocks.B.summary.samplesWithinBand).toBe(7);
  });

  it("classifies a command failure as infrastructure-invalid and still restores A", async () => {
    const value = fixture({ failLabel: "C.sample-4" });
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: ["protocol_violation"] });
    expect(value.calls.at(-1)).toEqual({ kind: "install", state: "A-final" });
  });

  it("persists a nested dirty-probe timeout as command_timeout", async () => {
    const value = fixture({ timeoutLabel: "C.sample-4" });
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: ["command_timeout"] });
    expect(value.calls.at(-1)).toEqual({ kind: "install", state: "A-final" });
  });

  it("persists then halts without A-final install after unconfirmed termination", async () => {
    const value = fixture({ terminationLabel: "C.sample-4" });
    await expect(runAttempt(spec, value.deps)).rejects.toMatchObject({
      kind: "termination_unconfirmed",
      details: { operatorActionRequired: true },
    });
    expect(value.calls.at(-1)).not.toEqual({ kind: "install", state: "A-final" });
    expect(value.writes.at(-1)).toMatchObject({
      target: path.join("G:\\artifacts", "attempts", "attempt-001", "attempt-result.json"),
      value: {
        kind: "infrastructure_invalid",
        reasons: ["command_timeout"],
        finalState: null,
      },
    });
  });

  it("preserves the termination sentinel when attempt-result publication fails", async () => {
    const value = fixture({ terminationLabel: "C.sample-4" });
    const baseWrite = value.deps.writeJsonFn;
    value.deps.writeJsonFn = async (target: string, result: Record<string, unknown>) => {
      if (target.endsWith("attempt-result.json")) throw new Error("attempt artifact unavailable");
      return baseWrite(target, result);
    };
    await expect(runAttempt(spec, value.deps)).rejects.toMatchObject({
      kind: "termination_unconfirmed",
      details: {
        operatorActionRequired: true,
        persistenceError: { message: "attempt artifact unavailable" },
      },
    });
    expect(value.calls.at(-1)).not.toEqual({ kind: "install", state: "A-final" });
  });

  it("persists a missing extractum checked unit as metadata_invalid", async () => {
    const value = fixture({ missingCheckedLabel: "C.sample-4" });
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: ["metadata_invalid"] });
  });

  it.each([
    { classification: "timeout", commandResult: { classification: "timeout", timedOut: true }, reason: "command_timeout" },
    { classification: "command_failed", commandResult: { classification: "command_failed", timedOut: false }, reason: "command_failed" },
  ])("classifies a $classification Cargo result as $reason", async ({ commandResult, reason }) => {
    const value = fixture({ cargoResult: commandResult });
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: [reason] });
  });

  it("overrides any prior result if final A restoration fails", async () => {
    const value = fixture();
    const original = value.deps.installStateFn;
    value.deps.installStateFn = async (input: { state: string }) => {
      if (input.state === "A-final") throw Object.assign(new Error("restore"), { kind: "final_restore_failed" });
      return original(input);
    };
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: ["restore_failed"] });
  });

  it("writes one terminal attempt result beneath the numbered attempt", async () => {
    const value = fixture();
    await runAttempt(spec, value.deps);
    expect(value.writes).toHaveLength(8);
    expect(value.writes.at(-1)?.target).toBe(path.join("G:\\artifacts", "attempts", "attempt-001", "attempt-result.json"));
  });
});
```

- [ ] **Step 2: Run attempt RED**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/attempt.test.ts
```

Expected: FAIL because `attempt.mjs` does not exist; the fixed-order and exact
reason-classification cases must all be collected.

- [ ] **Step 3: Implement the fixed per-block and per-attempt procedure**

Create `scripts/process-shell-diagnostic/attempt.mjs`:

```js
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

import { installState, verifyTargetIsolation } from "./git-state.mjs";
import { PROTOCOL, evaluateAttempt, summarizeBlock } from "./protocol.mjs";
import {
  assertCommandOk,
  hasTerminationUnconfirmed,
  ProtocolError,
  runCargoCheck,
  runDirtyCargoProbe,
  runWindowsProcess,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

function ordinaryEnvironment() {
  return Object.fromEntries(
    Object.entries(process.env).filter(([key]) => key.toUpperCase() !== "CARGO_LOG"),
  );
}

function taskkillExe() {
  if (!process.env.SystemRoot) throw new ProtocolError("missing_system_root", "SystemRoot is required");
  return path.join(process.env.SystemRoot, "System32", "taskkill.exe");
}

function requireCargoMetadata(result, label) {
  assertCommandOk(result, label, "required_cargo_metadata_missing");
  if (
    result.closeObserved !== true ||
    !Number.isFinite(result.elapsedMs) ||
    !Number.isFinite(result.cargoReportedMs)
  ) {
    throw new ProtocolError("required_cargo_metadata_missing", label, { result });
  }
}

function extractumProcessDirectDependency(metadata, block) {
  const app = metadata.packages?.find((pkg) => pkg.name === "extractum");
  if (!app) throw new ProtocolError("extractum_metadata_missing", block);
  return app.dependencies.some((dependency) =>
    dependency.name === "extractum-process"
    && dependency.kind === null
    && typeof dependency.path === "string",
  );
}

async function evidenceCommand({ label, args, worktree, artifactDir }) {
  const result = await runWindowsProcess({
    label,
    command: "cargo.exe",
    args,
    cwd: worktree,
    env: ordinaryEnvironment(),
    artifactDir,
    timeoutMs: PROTOCOL.commandTimeoutMs,
    taskkillExe: taskkillExe(),
  });
  assertCommandOk(result, label, "state_inventory_failed");
  if (result.closeObserved !== true) throw new ProtocolError("state_inventory_failed", label, { result });
  return {
    result,
    stdout: await readFile(result.stdoutPath, "utf8"),
    stderr: await readFile(result.stderrPath, "utf8"),
  };
}

export async function captureStateInventory({ block, worktree, mainRoot, artifactDir }) {
  const metadataRun = await evidenceCommand({
    label: `${block}.metadata`,
    args: [
      "metadata",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--format-version",
      "1",
      "--no-deps",
      "--locked",
    ],
    worktree,
    artifactDir,
  });
  let metadata;
  try {
    metadata = JSON.parse(metadataRun.stdout);
  } catch (error) {
    throw new ProtocolError("cargo_metadata_parse_failed", block, { message: error.message });
  }
  await verifyTargetIsolation({ metadata, worktree, mainRoot });
  const directProcessDependency = extractumProcessDirectDependency(metadata, block);

  const treeRun = await evidenceCommand({
    label: `${block}.feature-tree`,
    args: [
      "tree",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--workspace",
      "-e",
      "features",
      "--locked",
    ],
    worktree,
    artifactDir,
  });
  return {
    metadata,
    extractumProcessDirectDependency: directProcessDependency,
    metadataProcess: metadataRun.result,
    featureTreePath: treeRun.result.stdoutPath,
    featureTreeSha256Input: treeRun.stdout,
  };
}

export async function verifyTargetPreflight({ block, worktree, mainRoot, artifactDir }) {
  const metadataRun = await evidenceCommand({
    label: `${block}.target-preflight-metadata`,
    args: [
      "metadata",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--format-version",
      "1",
      "--no-deps",
      "--locked",
    ],
    worktree,
    artifactDir,
  });
  let metadata;
  try {
    metadata = JSON.parse(metadataRun.stdout);
  } catch (error) {
    throw new ProtocolError("cargo_metadata_parse_failed", `${block}.target-preflight`, {
      message: error.message,
    });
  }
  await verifyTargetIsolation({ metadata, worktree, mainRoot });
  return {
    targetDirectory: metadata.target_directory,
    metadataProcess: metadataRun.result,
  };
}

const productionDependencies = {
  installStateFn: installState,
  verifyTargetPreflightFn: verifyTargetPreflight,
  captureStateInventoryFn: captureStateInventory,
  runCargoCheckFn: runCargoCheck,
  runDirtyProbeFn: runDirtyCargoProbe,
  evaluateAttemptFn: evaluateAttempt,
  writeJsonFn: writeAtomicJsonExclusive,
};

async function runBlock({
  block,
  spec,
  attemptDir,
  deps,
}) {
  const stateEvidence = await deps.installStateFn({
    state: block,
    worktree: spec.worktree,
    mainRoot: spec.mainRoot,
    protocolLock: spec.protocolLock,
    artifactDir: attemptDir,
  });
  const cargoShared = {
    worktree: spec.worktree,
    artifactDir: attemptDir,
    cargoExe: "cargo.exe",
    taskkillExe: taskkillExe(),
    timeoutMs: PROTOCOL.commandTimeoutMs,
  };

  // cargo metadata --no-deps is the only allowed pre-build command: it proves
  // this state resolves to the isolated attempt target before Cargo may write.
  const targetPreflight = await deps.verifyTargetPreflightFn({
    block,
    worktree: spec.worktree,
    mainRoot: spec.mainRoot,
    artifactDir: attemptDir,
  });
  // The measured cache order is canonical sync first, then the resolved
  // metadata, feature graph, and unit inventory captured for evidence.
  const inventorySync = await deps.runCargoCheckFn({
    label: `${block}.inventory-sync`,
    diagnostic: false,
    ...cargoShared,
  });
  requireCargoMetadata(inventorySync, `${block}.inventory-sync`);
  const inventory = await deps.captureStateInventoryFn({
    block,
    worktree: spec.worktree,
    mainRoot: spec.mainRoot,
    artifactDir: attemptDir,
  });
  const expectsProcessExtern = ["C", "D", "E"].includes(block);
  if (inventory.extractumProcessDirectDependency !== expectsProcessExtern) {
    throw new ProtocolError("metadata_edge_mismatch", block, {
      expected: expectsProcessExtern,
      actual: inventory.extractumProcessDirectDependency,
    });
  }

  const dirtyShared = {
    ...cargoShared,
    sourcePath: path.join(spec.worktree, "src-tauri", "src", "lib.rs"),
    expectedCanonicalSha256: stateEvidence.canonicalLibSha256,
    requireExtractum: true,
  };
  const warmups = [];
  for (let index = 1; index <= PROTOCOL.warmupsPerBlock; index += 1) {
    const warmup = await deps.runDirtyProbeFn({
      label: `${block}.warmup-${index}`,
      diagnostic: false,
      ...dirtyShared,
    });
    requireCargoMetadata(warmup, `${block}.warmup-${index}`);
    if (!warmup.extractumChecked) throw new ProtocolError("extractum_not_checked", `${block}.warmup-${index}`);
    warmups.push(warmup);
  }

  const noOpSync = await deps.runCargoCheckFn({
    label: `${block}.noop-sync`,
    diagnostic: false,
    ...cargoShared,
  });
  requireCargoMetadata(noOpSync, `${block}.noop-sync`);
  const noOp = await deps.runCargoCheckFn({
    label: `${block}.noop`,
    diagnostic: false,
    ...cargoShared,
  });
  requireCargoMetadata(noOp, `${block}.noop`);

  const samples = [];
  for (let index = 1; index <= PROTOCOL.samplesPerBlock; index += 1) {
    const sample = await deps.runDirtyProbeFn({
      label: `${block}.sample-${index}`,
      diagnostic: false,
      ...dirtyShared,
    });
    requireCargoMetadata(sample, `${block}.sample-${index}`);
    if (!sample.extractumChecked) throw new ProtocolError("extractum_not_checked", `${block}.sample-${index}`);
    samples.push({
      index,
      wallMs: sample.elapsedMs,
      cargoReportedMs: sample.cargoReportedMs,
      checkedPackages: sample.checkedPackages ?? [PROTOCOL.expectedCheckedPackage],
      processMetadataPath: sample.stdoutPath ? sample.stdoutPath.replace(/\.stdout\.log$/, ".process.json") : null,
    });
  }
  const summary = summarizeBlock(samples.map((sample) => sample.wallMs));

  const diagnostic = await deps.runDirtyProbeFn({
    label: `${block}.diagnostic`,
    diagnostic: true,
    ...dirtyShared,
  });
  requireCargoMetadata(diagnostic, `${block}.diagnostic`);
  if (!diagnostic.extractumChecked) throw new ProtocolError("extractum_not_checked", `${block}.diagnostic`);
  if (!diagnostic.timingArtifact) throw new ProtocolError("timing_artifact_missing", block);
  if (!diagnostic.extractumLibRustcObserved) {
    throw new ProtocolError("extractum_lib_rustc_missing", block);
  }
  if (diagnostic.extractumProcessExtern !== expectsProcessExtern) {
    throw new ProtocolError("rustc_edge_mismatch", block, {
      expected: expectsProcessExtern,
      actual: diagnostic.extractumProcessExtern,
    });
  }

  const result = {
    schemaVersion: 1,
    block,
    stateEvidence,
    targetPreflight,
    inventorySync,
    inventory,
    warmups,
    noOpSync,
    noOp,
    samples,
    summary,
    diagnostic,
  };
  await deps.writeJsonFn(path.join(attemptDir, "blocks", `${block}.json`), result);
  return result;
}

function errorReason(error) {
  const kind = String(error?.kind ?? error?.name ?? "unknown").toLowerCase();
  function containsTimeout(value, seen = new Set()) {
    if (!value || typeof value !== "object" || seen.has(value)) return false;
    seen.add(value);
    if (
      value.timedOut === true
      || value.classification === "timeout"
      || value.classification === "termination_unconfirmed"
      || value.kind === "command_timeout"
    ) return true;
    return Object.values(value).some((entry) => containsTimeout(entry, seen));
  }
  if (kind.includes("timeout") || containsTimeout(error)) return "command_timeout";
  const exact = {
    canonical_sync_failed: "command_failed",
    cargo_failed: "command_failed",
    canonical_hash_mismatch: "restore_failed",
    recovery_hash_mismatch: "restore_failed",
    source_restore_failed: "restore_failed",
    extractum_not_checked: "metadata_invalid",
    extractum_lib_rustc_missing: "metadata_invalid",
    rustc_edge_mismatch: "metadata_invalid",
    metadata_edge_mismatch: "metadata_invalid",
    required_cargo_metadata_missing: "metadata_invalid",
    state_inventory_failed: "metadata_invalid",
  };
  if (exact[kind]) return exact[kind];
  if (/(restore|recovery|probe_source)/.test(kind)) return "restore_failed";
  if (/(target|workspace_root|cargo_target_dir)/.test(kind)) return "target_invalid";
  if (/(metadata|timing|checked_package|cargo_reported|inventory|rustc|extern|edge)/.test(kind)) return "metadata_invalid";
  if (/(state|tree|blob|patch|candidate|manifest)/.test(kind)) return "state_invalid";
  if (/(platform|host|attestation|environment|quiescence)/.test(kind)) return "environment_invalid";
  if (/(cargo|git|command|spawn|exit)/.test(kind)) return "command_failed";
  return "protocol_violation";
}

async function persistTerminationResultAndThrow({ deps, resultPath, result, error, attemptId }) {
  let persistenceError = null;
  try {
    await deps.writeJsonFn(resultPath, result);
  } catch (writeError) {
    persistenceError = writeError;
  }
  throw new ProtocolError("termination_unconfirmed", attemptId, {
    operatorActionRequired: true,
    attemptResult: result,
    cause: error?.details ?? null,
    persistenceError: persistenceError
      ? { kind: persistenceError.kind ?? persistenceError.name, message: persistenceError.message }
      : null,
  });
}

export async function runAttempt(spec, injected = {}) {
  const deps = { ...productionDependencies, ...injected };
  const attemptDir = path.join(spec.sessionDir, "attempts", spec.attemptId);
  const startedAt = new Date().toISOString();
  const blocks = {};
  let result;

  try {
    for (const block of PROTOCOL.baseSequence) {
      blocks[block] = await runBlock({ block, spec, attemptDir, deps });
    }
    let evaluation = deps.evaluateAttemptFn(
      Object.fromEntries(Object.entries(blocks).map(([name, value]) => [name, value.samples.map((sample) => sample.wallMs)])),
    );
    if (evaluation.kind === "needs_e") {
      for (const block of PROTOCOL.conditionalSequence) {
        blocks[block] = await runBlock({ block, spec, attemptDir, deps });
      }
      evaluation = deps.evaluateAttemptFn(
        Object.fromEntries(Object.entries(blocks).map(([name, value]) => [name, value.samples.map((sample) => sample.wallMs)])),
      );
    }
    if (!["valid", "stability_invalid"].includes(evaluation.kind)) {
      throw new ProtocolError("incomplete_evaluation", evaluation.kind);
    }
    result = {
      schemaVersion: 1,
      attemptId: spec.attemptId,
      kind: evaluation.kind,
      reasons: evaluation.reasons ?? [],
      startedAt,
      endedAt: new Date().toISOString(),
      blocks,
      evaluation,
    };
  } catch (error) {
    result = {
      schemaVersion: 1,
      attemptId: spec.attemptId,
      kind: "infrastructure_invalid",
      reasons: [errorReason(error)],
      startedAt,
      endedAt: new Date().toISOString(),
      blocks,
      error: {
        name: error?.name ?? "Error",
        kind: error?.kind ?? error?.name ?? "unknown",
        category: errorReason(error),
        message: error?.message ?? String(error),
        details: error?.details ?? null,
      },
    };
    if (hasTerminationUnconfirmed(error)) {
      result.finalState = null;
      await persistTerminationResultAndThrow({
        deps,
        resultPath: path.join(attemptDir, "attempt-result.json"),
        result,
        error,
        attemptId: spec.attemptId,
      });
    }
  }

  try {
    result.finalState = await deps.installStateFn({
      state: "A-final",
      worktree: spec.worktree,
      mainRoot: spec.mainRoot,
      protocolLock: spec.protocolLock,
      artifactDir: attemptDir,
    });
  } catch (error) {
    result = {
      schemaVersion: 1,
      attemptId: spec.attemptId,
      kind: "infrastructure_invalid",
      reasons: [errorReason(error)],
      startedAt,
      endedAt: new Date().toISOString(),
      blocks,
      error: {
        name: error?.name ?? "Error",
        kind: error?.kind ?? error?.name ?? "unknown",
        category: errorReason(error),
        message: error?.message ?? String(error),
        details: error?.details ?? null,
      },
    };
    if (hasTerminationUnconfirmed(error)) {
      result.finalState = null;
      await persistTerminationResultAndThrow({
        deps,
        resultPath: path.join(attemptDir, "attempt-result.json"),
        result,
        error,
        attemptId: spec.attemptId,
      });
    }
  }

  await deps.writeJsonFn(path.join(attemptDir, "attempt-result.json"), result);
  return result;
}
```

- [ ] **Step 4: Run attempt GREEN**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/attempt.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Attempt GREEN failed.' }
```

Expected: one file, twelve tests PASS. The fake log proves 10 dirty calls per
block (2 warm-ups + 7 samples + 1 diagnostic), exactly one no-op after its own
sync, the direct `--extern extractum_process` edge only in C/D/E, conditional
E/A4 behavior, and final A restoration.

- [ ] **Step 5: Commit the attempt runner**

Run:

```powershell
$diagnosticTask4Status = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect Task 4 status.' }
$diagnosticTask4Status | ForEach-Object { Write-Output $_ }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 4 diff check failed.' }
git add -- scripts/process-shell-diagnostic/attempt.mjs scripts/process-shell-diagnostic/attempt.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Could not stage Task 4 files.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Task 4 staged diff check failed.' }
git commit -m "feat: orchestrate process shell diagnostic attempts"
if ($LASTEXITCODE -ne 0) { throw 'Task 4 commit failed.' }
```

Expected: only Task 4's runner and tests are committed.

### Task 5: Coordinate Immutable Sessions, Fresh Worktrees, and Retries

**Files:**

- Create: `scripts/process-shell-diagnostic/coordinator.test.ts`
- Create: `scripts/process-shell-diagnostic/coordinator.mjs`
- Modify: `docs/value-registry.md`

**Interfaces:**

- Consumes `runAttempt`, `reduceRetry`, the runtime process/artifact layer, and
  the committed `protocol-lock.json` (Task 6 adds cryptographic verification
  before production start).
- Produces `startSession({ mainRoot, protocolRoot, scratchParent,
  processAttested }, deps?)`, `resumeSession({ sessionDir, correctedCause?,
  unexplainedStability?, processAttested? }, deps?)`, and `parseCli(argv)`.
- Storage is append-only: the external locator is the bootstrap reservation WAL
  written before `sessionDir`; `attempt_started` is the attempt reservation WAL
  written before `attemptDir`; numbered recovery events make source restoration
  replayable. Aggregate `session-ledger.json` is projected first; terminal
  `decision.json` is its last-published commit marker. Both are deterministic
  projections of the immutable numbered records.
- The session-manifest environment is bootstrap evidence, not the measurement
  baseline. The first durable `attempt_environment` is authoritative immediately
  before A0; every later attempt must match its toolchain, Cargo environment,
  main tree, and full main-target snapshot before worktree creation or Cargo.
  Power/Defender observations remain per-attempt evidence so an explicitly
  corrected infrastructure cause is recordable rather than silently normalized.

- [ ] **Step 1: Write coordinator RED tests**

Create `scripts/process-shell-diagnostic/coordinator.test.ts`:

```ts
import { mkdtemp, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  assertControlCommandResult,
  parseCli,
  resumeSession,
  snapshotDirectory,
  startSession,
} from "./coordinator.mjs";

async function paths() {
  const root = await mkdtemp(path.join(os.tmpdir(), "extractum-psd-session-"));
  const mainRoot = path.join(root, "main");
  const protocolRoot = path.join(root, "protocol");
  const scratchParent = path.join(root, "scratch");
  await mkdir(path.join(protocolRoot, "scripts", "process-shell-diagnostic"), { recursive: true });
  await mkdir(mainRoot, { recursive: true });
  await mkdir(scratchParent, { recursive: true });
  await writeFile(
    path.join(protocolRoot, "scripts", "process-shell-diagnostic", "protocol-lock.json"),
    `${JSON.stringify({
      schemaVersion: 1,
      states: {
        A: { srcTauriTree: "a-tree" },
        B: { srcTauriTree: "b-tree" },
        C: { srcTauriTree: "c-tree" },
        D: { srcTauriTree: "d-tree" },
        E: { srcTauriTree: "e-tree" },
      },
    })}\n`,
    "utf8",
  );
  return { mainRoot, protocolRoot, scratchParent };
}

function attempt(kind: string, classification = "not_reproduced") {
  return (spec: Record<string, string>) => ({
    schemaVersion: 1,
    attemptId: spec.attemptId,
    kind,
    reasons: kind === "stability_invalid" ? ["anchor_range_exceeded"] : kind === "infrastructure_invalid" ? ["command_failed"] : [],
    evaluation: kind === "valid"
      ? { kind: "valid", classification }
      : kind === "stability_invalid"
        ? { kind: "stability_invalid", reasons: ["anchor_range_exceeded"] }
        : null,
    finalState: { kind: "A", srcTauriTree: "a-tree" },
    worktree: spec.worktree,
  });
}

function fake(attempts: Array<(spec: Record<string, string>) => Record<string, unknown>>) {
  const queue = [...attempts];
  const worktrees: string[] = [];
  const targets: string[] = [];
  return {
    worktrees,
    targets,
    dependencies: {
      uuidFn: () => "session-fixed",
      nowFn: () => "2026-07-18T12:00:00.000Z",
      processEnv: {},
      resolveProtocolCommitFn: async () => "a".repeat(40),
      captureEnvironmentFn: async () => ({
        platform: "win32",
        host: "x86_64-pc-windows-msvc",
        cargo: "cargo 1.95.0",
        rustc: "rustc 1.95.0",
        power: "Balanced",
        defender: "unavailable: Access denied",
        processQuiescence: [],
        operatorProcessAttestation: true,
        cargoEnvironment: {},
        mainRoot: "G:\\main",
        mainSrcTauriTree: "a-tree",
        mainTargetDirectory: "G:\\main\\src-tauri\\target",
        mainTargetSnapshot: { exists: true, records: [], digest: "baseline-target" },
      }),
      createDetachedWorktreeFn: async ({ worktree }: { worktree: string }) => {
        await mkdir(worktree, { recursive: true });
        worktrees.push(worktree);
      },
      restoreAttemptWorktreeFn: async () => ({ kind: "A", srcTauriTree: "a-tree" }),
      runAttemptFn: async (spec: Record<string, string>) => {
        targets.push(path.join(spec.worktree, "src-tauri", "target"));
        const next = queue.shift();
        if (!next) throw new Error("unexpected extra attempt");
        const result = next(spec);
        await writeFile(
          path.join(spec.sessionDir, "attempts", spec.attemptId, "attempt-result.json"),
          `${JSON.stringify(result, null, 2)}\n`,
          { encoding: "utf8", flag: "wx" },
        );
        return result;
      },
    },
  };
}

describe("process shell diagnostic coordinator", () => {
  it("pins one locator and completes one valid session", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const result = await startSession(
      { ...input, processAttested: true },
      value.dependencies,
    );
    expect(result).toMatchObject({
      status: "completed",
      classification: "not_reproduced",
      retryState: { unexplainedStabilityInvalidCount: 0, terminal: true },
    });
    expect(value.worktrees).toEqual([
      path.join(input.mainRoot, ".worktrees", "process-shell-session-session-fixed", "attempt-001"),
    ]);
    expect(value.targets).toEqual([
      path.join(value.worktrees[0], "src-tauri", "target"),
    ]);
    await expect(startSession(
      { ...input, processAttested: true },
      value.dependencies,
    )).rejects.toMatchObject({ kind: "session_locator_exists" });
    const decision = JSON.parse(await readFile(path.join(result.sessionDir, "decision.json"), "utf8"));
    expect(decision).toMatchObject({ classification: "not_reproduced", attemptId: "attempt-001" });
  });

  it("uses two explicit unexplained dispositions before the precision terminal", async () => {
    const input = await paths();
    const value = fake([attempt("stability_invalid"), attempt("stability_invalid")]);
    const first = await startSession({ ...input, processAttested: true }, value.dependencies);
    expect(first).toMatchObject({
      status: "awaiting_stability_disposition",
      retryState: { unexplainedStabilityInvalidCount: 0 },
    });
    const second = await resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true },
      value.dependencies,
    );
    expect(second).toMatchObject({
      status: "awaiting_stability_disposition",
      retryState: { unexplainedStabilityInvalidCount: 1 },
    });
    const terminal = await resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true },
      value.dependencies,
    );
    expect(terminal).toMatchObject({
      status: "completed",
      classification: "environment_precision_insufficient",
      retryState: { unexplainedStabilityInvalidCount: 2, terminal: true },
    });
    expect(value.worktrees).toHaveLength(2);
    expect(new Set(value.targets).size).toBe(2);
  });

  it("keeps count one through a corrected infrastructure retry", async () => {
    const input = await paths();
    const value = fake([
      attempt("stability_invalid"),
      attempt("infrastructure_invalid"),
      attempt("valid", "boundary_composite"),
    ]);
    const first = await startSession({ ...input, processAttested: true }, value.dependencies);
    const second = await resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true },
      value.dependencies,
    );
    expect(second.status).toBe("awaiting_correction");
    expect(second.retryState.unexplainedStabilityInvalidCount).toBe(1);
    const stillWaiting = await resumeSession({ sessionDir: first.sessionDir }, value.dependencies);
    expect(stillWaiting.status).toBe("awaiting_correction");
    expect(value.worktrees).toHaveLength(2);
    const final = await resumeSession(
      { sessionDir: first.sessionDir, correctedCause: "Defender scan ended; the approved exclusion was verified.", processAttested: true },
      value.dependencies,
    );
    expect(final).toMatchObject({
      status: "completed",
      classification: "boundary_composite",
      retryState: { unexplainedStabilityInvalidCount: 1, terminal: true },
    });
  });

  it("invalidates target or toolchain drift before a later attempt creates its worktree", async () => {
    const input = await paths();
    const value = fake([attempt("stability_invalid"), attempt("valid")]);
    const baseCapture = value.dependencies.captureEnvironmentFn;
    let captureNumber = 0;
    const captureEnvironmentFn = async () => {
      captureNumber += 1;
      const environment = await baseCapture();
      return {
        ...environment,
        mainRoot: input.mainRoot,
        mainTargetDirectory: path.join(input.mainRoot, "src-tauri", "target"),
        mainTargetSnapshot: {
          exists: true,
          records: [],
          digest: captureNumber >= 3 ? "changed-target" : "baseline-target",
        },
      };
    };
    const first = await startSession(
      { ...input, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    );
    const second = await resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    );
    expect(second.status).toBe("awaiting_correction");
    expect(value.worktrees).toHaveLength(1);
    expect(JSON.parse(await readFile(second.attempts[1].resultPath, "utf8"))).toMatchObject({
      reasons: ["coordinator_failure"],
      error: { kind: "attempt_environment_drift" },
    });
  });

  it("requires a corrected-cause disposition before power or Defender drift", async () => {
    async function runCase(firstKind: "stability_invalid" | "infrastructure_invalid", corrected: boolean) {
      const input = await paths();
      const value = fake([attempt(firstKind), attempt("valid")]);
      const baseCapture = value.dependencies.captureEnvironmentFn;
      let captureNumber = 0;
      const captureEnvironmentFn = async () => {
        captureNumber += 1;
        return {
          ...await baseCapture(),
          mainRoot: input.mainRoot,
          mainTargetDirectory: path.join(input.mainRoot, "src-tauri", "target"),
          power: captureNumber >= 3 ? "High performance" : "Balanced",
        };
      };
      const first = await startSession(
        { ...input, processAttested: true },
        { ...value.dependencies, captureEnvironmentFn },
      );
      const options = corrected
        ? { sessionDir: first.sessionDir, correctedCause: "Power plan fixed by operator", processAttested: true }
        : { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true };
      return {
        result: await resumeSession(options, { ...value.dependencies, captureEnvironmentFn }),
        worktrees: value.worktrees,
      };
    }

    const unexplained = await runCase("stability_invalid", false);
    expect(unexplained.result.status).toBe("awaiting_correction");
    expect(unexplained.worktrees).toHaveLength(1);
    const corrected = await runCase("infrastructure_invalid", true);
    expect(corrected.result).toMatchObject({ status: "completed", classification: "not_reproduced" });
    expect(corrected.worktrees).toHaveLength(2);
  });

  it("recovers a durable attempt result without creating a duplicate attempt", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const crash = Object.assign(new Error("simulated crash"), { simulatedCrash: true });
    await expect(startSession(
      { ...input, processAttested: true },
      { ...value.dependencies, afterAttemptObservedFn: async () => { throw crash; } },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered).toMatchObject({ status: "completed", classification: "not_reproduced" });
    expect(value.worktrees).toHaveLength(1);
  });

  it("pins a pre-recovery result path and digest across a crash inside recovery", async () => {
    const input = await paths();
    const unproven = (spec: Record<string, string>) => ({
      ...attempt("valid")(spec),
      finalState: null,
    });
    const value = fake([unproven]);
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterAttemptObservedFn: async () => {
          throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    let recoveryCrash = false;
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!recoveryCrash && event.type === "attempt_recovery_started") {
            recoveryCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    const events = (await readdir(path.join(sessionDir, "ledger"))).sort();
    const ledger = await Promise.all(events.map((name) =>
      readFile(path.join(sessionDir, "ledger", name), "utf8").then(JSON.parse),
    ));
    const reservation = ledger.find((event) => event.type === "attempt_recovery_started");
    expect(reservation).toMatchObject({
      sourceResultPath: path.join(sessionDir, "attempts", "attempt-001", "attempt-result.json"),
      sourceResultSha256: expect.stringMatching(/^[0-9a-f]{64}$/),
    });
    expect(JSON.parse(await readFile(recovered.attempts[0].resultPath, "utf8"))).toMatchObject({
      reasons: ["final_restore_evidence_missing"],
      sourceResultPath: reservation.sourceResultPath,
    });
  });

  it("allows only a completed nonzero optional environment command", () => {
    expect(() => assertControlCommandResult(
      { classification: "command_failed", closeObserved: true, timedOut: false, exitCode: 1 },
      "environment-defender",
      true,
    )).not.toThrow();
    for (const { result, expectedKind } of [
      {
        result: { classification: "timeout", closeObserved: true, timedOut: true },
        expectedKind: "command_timeout",
      },
      {
        result: { classification: "termination_unconfirmed", closeObserved: false, timedOut: true },
        expectedKind: "termination_unconfirmed",
      },
    ]) {
      let thrown: unknown = null;
      try {
        assertControlCommandResult(result, "environment-defender", true);
      } catch (error) {
        thrown = error;
      }
      expect(thrown).toMatchObject({ kind: expectedKind });
    }
  });

  it("halts after unconfirmed termination and recovers only on a freshly attested resume", async () => {
    const input = await paths();
    const value = fake([]);
    let captureCount = 0;
    let restoreCount = 0;
    let verifyCount = 0;
    const baseVerify = (value.dependencies as Record<string, unknown>).verifyFrozenProtocolFn as
      | ((input: { repoRoot: string }) => Promise<Record<string, unknown>>)
      | undefined;
    const captureEnvironmentFn = async () => {
      captureCount += 1;
      return value.dependencies.captureEnvironmentFn();
    };
    const restoreAttemptWorktreeFn = async () => {
      restoreCount += 1;
      return { kind: "A", srcTauriTree: "a-tree" };
    };
    const runAttemptFn = async (spec: Record<string, string>) => {
      const result = {
        ...attempt("infrastructure_invalid")(spec),
        finalState: null,
      };
      await writeFile(
        path.join(spec.sessionDir, "attempts", spec.attemptId, "attempt-result.json"),
        `${JSON.stringify(result, null, 2)}\n`,
        { encoding: "utf8", flag: "wx" },
      );
      throw Object.assign(new Error("owned process tree may still be alive"), {
        kind: "termination_unconfirmed",
        details: { operatorActionRequired: true, attemptResult: result },
      });
    };
    const deps = {
      ...value.dependencies,
      captureEnvironmentFn,
      restoreAttemptWorktreeFn,
      runAttemptFn,
      ...(baseVerify
        ? {
            verifyFrozenProtocolFn: async (input: { repoRoot: string }) => {
              verifyCount += 1;
              return baseVerify(input);
            },
          }
        : {}),
    };
    await expect(startSession({ ...input, processAttested: true }, deps)).rejects.toMatchObject({
      kind: "termination_unconfirmed",
    });
    expect(captureCount).toBe(2);
    expect(restoreCount).toBe(0);
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const verifyCountBeforeRejectedResume = verifyCount;
    await expect(resumeSession({ sessionDir }, deps)).rejects.toMatchObject({
      kind: "process_attestation_missing",
    });
    expect(verifyCount).toBe(verifyCountBeforeRejectedResume);
    expect(captureCount).toBe(2);
    expect(restoreCount).toBe(0);
    const recovered = await resumeSession({ sessionDir, processAttested: true }, deps);
    expect(recovered.status).toBe("awaiting_correction");
    expect(captureCount).toBe(3);
    expect(restoreCount).toBe(1);
    expect(value.worktrees).toHaveLength(1);
  });

  it("replays one durable retry disposition without consuming it twice", async () => {
    const input = await paths();
    const value = fake([attempt("stability_invalid"), attempt("valid")]);
    const first = await startSession({ ...input, processAttested: true }, value.dependencies);
    let crashed = false;
    await expect(resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!crashed && event.type === "retry_disposition") {
            crashed = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const recovered = await resumeSession(
      { sessionDir: first.sessionDir, processAttested: true },
      value.dependencies,
    );
    expect(recovered).toMatchObject({
      status: "completed",
      retryState: { unexplainedStabilityInvalidCount: 1, terminal: true },
    });
    expect(value.worktrees).toHaveLength(2);
  });

  it("materializes missing terminal projections across both publication windows", async () => {
    for (const point of ["session_completed", "session-ledger.json"]) {
      const input = await paths();
      const value = fake([attempt("valid")]);
      let crashed = false;
      await expect(startSession(
        { ...input, processAttested: true },
        {
          ...value.dependencies,
          afterDurableWriteFn: async ({
            target,
            value: event,
          }: { target: string; value: { type?: string } }) => {
            if (!crashed && (event.type === point || target.endsWith(point))) {
              crashed = true;
              throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
            }
          },
        },
      )).rejects.toThrow("simulated crash");
      const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
      const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
      expect(recovered.status).toBe("completed");
      expect(JSON.parse(await readFile(path.join(sessionDir, "session-ledger.json"), "utf8"))).toMatchObject({
        terminal: true,
      });
      expect(JSON.parse(await readFile(path.join(sessionDir, "decision.json"), "utf8"))).toMatchObject({
        attemptId: "attempt-001",
      });
    }
  });

  it.each(["session-manifest.json", "process-shell-diagnostic.locator.json", "session_started"])(
    "recovers bootstrap crash after %s",
    async (point) => {
      const input = await paths();
      const value = fake([attempt("valid")]);
      let crashed = false;
      await expect(startSession(
        { ...input, processAttested: true },
        {
          ...value.dependencies,
          afterDurableWriteFn: async ({ target, value: event }: { target: string; value: { type?: string } }) => {
            if (!crashed && (target.endsWith(point) || event.type === point)) {
              crashed = true;
              throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
            }
          },
        },
      )).rejects.toThrow("simulated crash");
      const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
      const recovered = await resumeSession(
        { sessionDir, processAttested: true },
        value.dependencies,
      );
      expect(recovered.status).toBe("completed");
      expect(value.worktrees).toHaveLength(1);
    },
  );

  it("retries bootstrap materialization in a new artifact directory after two crashes", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let locatorCrash = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ target }: { target: string }) => {
          if (!locatorCrash && target.endsWith("process-shell-diagnostic.locator.json")) {
            locatorCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const baseCapture = value.dependencies.captureEnvironmentFn;
    let bootstrapCrashes = 0;
    const captureEnvironmentFn = async (spec: { artifactDir: string }) => {
      if (path.basename(spec.artifactDir).startsWith("bootstrap-recovery-")) {
        await mkdir(spec.artifactDir, { recursive: true });
        if (bootstrapCrashes < 2) {
          bootstrapCrashes += 1;
          throw new Error(`bootstrap capture crash ${bootstrapCrashes}`);
        }
      }
      return baseCapture();
    };
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    )).rejects.toThrow("bootstrap capture crash 1");
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    )).rejects.toThrow("bootstrap capture crash 2");
    const recovered = await resumeSession(
      { sessionDir, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    );
    expect(recovered.status).toBe("completed");
    expect((await readdir(sessionDir)).filter((name) => name.startsWith("bootstrap-recovery-"))).toHaveLength(3);
  });

  it("recovers an attempt reservation committed before its directory exists", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let crashed = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!crashed && event.type === "attempt_started") {
            crashed = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts[0].reasons).toEqual(["coordinator_interrupted"]);
    expect(value.worktrees).toHaveLength(0);
  });

  it("replays a durable coordinator failure before any normal result", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let crashed = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        createDetachedWorktreeFn: async () => { throw new Error("forced worktree failure"); },
        afterDurableWriteFn: async ({ target }: { target: string }) => {
          if (!crashed && target.endsWith("coordinator-failure.json")) {
            crashed = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts[0].reasons).toEqual(["coordinator_failure"]);
  });

  it("replays an interruption published after recovery and before attempt_finished", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let reservationCrash = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!reservationCrash && event.type === "attempt_started") {
            reservationCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    let interruptionCrash = false;
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ target }: { target: string }) => {
          if (!interruptionCrash && target.endsWith("coordinator-interruption.json")) {
            interruptionCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts).toHaveLength(1);
  });

  it("abandons a late normal result after recovery_started and completes a new recovery", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let reservationCrash = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!reservationCrash && event.type === "attempt_started") {
            reservationCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    let recoveryCrash = false;
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!recoveryCrash && event.type === "attempt_recovery_started") {
            recoveryCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const lateResultPath = path.join(sessionDir, "attempts", "attempt-001", "attempt-result.json");
    await mkdir(path.dirname(lateResultPath), { recursive: true });
    await writeFile(lateResultPath, `${JSON.stringify(attempt("valid")({
      attemptId: "attempt-001",
      worktree: path.join(input.mainRoot, ".worktrees", "late"),
    }), null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts[0].reasons).toEqual(["coordinator_interrupted"]);
  });

  it("points coordinator failure rows at their real immutable artifact", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const result = await startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        createDetachedWorktreeFn: async () => {
          const error = Object.assign(new Error("forced worktree failure"), { kind: "worktree_create_failed" });
          throw error;
        },
      },
    );
    expect(result.status).toBe("awaiting_correction");
    const finished = result.attempts[0];
    expect(finished.resultPath).toMatch(/coordinator-failure\.json$/);
    expect(JSON.parse(await readFile(finished.resultPath, "utf8"))).toMatchObject({
      kind: "infrastructure_invalid",
      reasons: ["coordinator_failure"],
    });
  });

  it("content-hashes the complete main target tree, not only its root timestamp", async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "extractum-target-snapshot-"));
    await mkdir(path.join(root, "debug", "incremental"), { recursive: true });
    const artifact = path.join(root, "debug", "incremental", "unit.bin");
    await writeFile(artifact, "before", "utf8");
    const before = await snapshotDirectory(root);
    await writeFile(artifact, "after!", "utf8");
    const after = await snapshotDirectory(root);
    expect(before.digest).not.toBe(after.digest);
    expect(before.records.map((record) => record.path)).toContain("debug/incremental/unit.bin");
  });

  it("rejects shared-target environment before creating a worktree", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    await expect(startSession(
      { ...input, processAttested: true },
      { ...value.dependencies, processEnv: { cargo_target_dir: "G:\\shared" } },
    )).rejects.toMatchObject({ kind: "cargo_target_dir_set" });
    expect(value.worktrees).toHaveLength(0);
  });

  it("requires an explicit operator quiescence attestation", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    await expect(startSession(input, value.dependencies)).rejects.toMatchObject({
      kind: "process_attestation_missing",
    });
  });

  it("parses boolean and value CLI arguments without ambiguity", () => {
    expect(parseCli([
      "start", "--main-root", "G:\\main", "--protocol-root", "G:\\protocol",
      "--scratch-parent", "G:\\scratch", "--process-attested",
    ])).toEqual({
      command: "start",
      options: {
        mainRoot: "G:\\main",
        protocolRoot: "G:\\protocol",
        scratchParent: "G:\\scratch",
        processAttested: true,
      },
    });
    expect(parseCli([
      "resume", "--session-dir", "G:\\scratch\\session", "--unexplained-stability", "--process-attested",
    ])).toEqual({
      command: "resume",
      options: { sessionDir: "G:\\scratch\\session", unexplainedStability: true, processAttested: true },
    });
  });
});
```

- [ ] **Step 2: Run coordinator RED**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/coordinator.test.ts
```

Expected: FAIL because `coordinator.mjs` does not exist; twenty-four tests must be
collected.

- [ ] **Step 3: Implement environment capture, append-only ledger, and CLI**

Create `scripts/process-shell-diagnostic/coordinator.mjs`:

```js
import { createHash, randomUUID } from "node:crypto";
import { access, mkdir, readFile, readdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

import { installState } from "./git-state.mjs";
import { runAttempt } from "./attempt.mjs";
import { PROTOCOL, reduceRetry } from "./protocol.mjs";
import {
  hasTerminationUnconfirmed,
  ProtocolError,
  runWindowsProcess,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

const LOCATOR_NAME = "process-shell-diagnostic.locator.json";
const INITIAL_RETRY_STATE = Object.freeze({
  unexplainedStabilityInvalidCount: 0,
  terminal: false,
});

async function assertMissing(filePath, kind) {
  try {
    await access(filePath);
  } catch (error) {
    if (error.code === "ENOENT") return;
    throw error;
  }
  throw new ProtocolError(kind, `path already exists: ${filePath}`);
}

function assertCargoTargetDirUnset(environment) {
  const entry = Object.entries(environment).find(([key]) => key.toUpperCase() === "CARGO_TARGET_DIR");
  if (entry) throw new ProtocolError("cargo_target_dir_set", `${entry[0]} must be absent`, { value: entry[1] });
}

function taskkillExe() {
  if (!process.env.SystemRoot) throw new ProtocolError("missing_system_root", "SystemRoot is required");
  return path.join(process.env.SystemRoot, "System32", "taskkill.exe");
}

async function controlCommand({ label, command, args, cwd, artifactDir, allowFailure = false }) {
  const result = await runWindowsProcess({
    label,
    command,
    args,
    cwd,
    env: Object.fromEntries(
      Object.entries(process.env).filter(([key]) => key.toUpperCase() !== "CARGO_LOG"),
    ),
    artifactDir,
    timeoutMs: PROTOCOL.commandTimeoutMs,
    taskkillExe: taskkillExe(),
  });
  if (hasTerminationUnconfirmed(result)) {
    throw new ProtocolError("termination_unconfirmed", label, {
      result,
      operatorActionRequired: true,
    });
  }
  const stdout = await readFile(result.stdoutPath, "utf8");
  const stderr = await readFile(result.stderrPath, "utf8");
  assertControlCommandResult(result, label, allowFailure, stderr);
  return { result, stdout: stdout.trim(), stderr: stderr.trim() };
}

export function assertControlCommandResult(result, label, allowFailure = false, stderr = "") {
  if (hasTerminationUnconfirmed(result)) {
    throw new ProtocolError("termination_unconfirmed", label, {
      result,
      stderr,
      operatorActionRequired: true,
    });
  }
  if (result?.timedOut === true || result?.classification === "timeout") {
    throw new ProtocolError("command_timeout", label, { result, stderr });
  }
  const completedNonzero = result?.classification === "command_failed"
    && result?.closeObserved === true
    && result?.timedOut !== true
    && Number.isInteger(result?.exitCode)
    && result.exitCode !== 0;
  if (result?.classification !== "ok" && !(allowFailure && completedNonzero)) {
    throw new ProtocolError("preflight_command_failed", label, { result, stderr });
  }
}

async function resolveProtocolCommitProduction({ protocolRoot, artifactDir }) {
  const value = await controlCommand({
    label: "protocol-head",
    command: "git.exe",
    args: ["rev-parse", "HEAD"],
    cwd: protocolRoot,
    artifactDir,
  });
  if (!/^[0-9a-f]{40}$/.test(value.stdout)) {
    throw new ProtocolError("protocol_commit_invalid", value.stdout);
  }
  return value.stdout;
}

async function captureEnvironmentProduction({
  mainRoot,
  artifactDir,
  processAttested,
  protocolLock,
}) {
  const rustc = await controlCommand({
    label: "environment-rustc",
    command: "rustc.exe",
    args: ["-vV"],
    cwd: mainRoot,
    artifactDir,
  });
  const cargo = await controlCommand({
    label: "environment-cargo",
    command: "cargo.exe",
    args: ["-V"],
    cwd: mainRoot,
    artifactDir,
  });
  const host = rustc.stdout.match(/^host:\s+(.+)$/m)?.[1] ?? null;
  if (process.platform !== "win32" || host !== "x86_64-pc-windows-msvc") {
    throw new ProtocolError("unsupported_host", `${process.platform}:${host}`);
  }

  const active = await controlCommand({
    label: "environment-build-processes",
    command: "powershell.exe",
    args: [
      "-NoLogo",
      "-NoProfile",
      "-Command",
      "$names=@('cargo.exe','rustc.exe','rust-analyzer.exe'); Get-CimInstance Win32_Process -ErrorAction Stop | Where-Object { $names -contains $_.Name -or $_.CommandLine -match '(?i)(tauri\\s+dev|vite(\\.js)?\\s+--host|npm\\.cmd\\s+run\\s+tauri)' } | Select-Object ProcessId,Name,CommandLine | ConvertTo-Json -Compress",
    ],
    cwd: mainRoot,
    artifactDir,
  });
  if (active.stdout !== "") {
    throw new ProtocolError("build_process_active", active.stdout);
  }

  const mainStatus = await controlCommand({
    label: "environment-main-src-status",
    command: "git.exe",
    args: ["status", "--porcelain=v1", "--untracked-files=all", "--", "src-tauri"],
    cwd: mainRoot,
    artifactDir,
  });
  if (mainStatus.stdout !== "") throw new ProtocolError("main_src_tauri_dirty", mainStatus.stdout);
  const mainTree = await controlCommand({
    label: "environment-main-src-tree",
    command: "git.exe",
    args: ["rev-parse", "HEAD:src-tauri"],
    cwd: mainRoot,
    artifactDir,
  });
  if (mainTree.stdout !== protocolLock.states.A.srcTauriTree) {
    throw new ProtocolError("main_baseline_tree_mismatch", mainTree.stdout);
  }

  const power = await controlCommand({
    label: "environment-power",
    command: "powercfg.exe",
    args: ["/GETACTIVESCHEME"],
    cwd: mainRoot,
    artifactDir,
    allowFailure: true,
  });
  const defender = await controlCommand({
    label: "environment-defender",
    command: "powershell.exe",
    args: [
      "-NoLogo",
      "-NoProfile",
      "-Command",
      "Get-MpComputerStatus | Select-Object RealTimeProtectionEnabled,AntivirusEnabled,QuickScanAge | ConvertTo-Json -Compress",
    ],
    cwd: mainRoot,
    artifactDir,
    allowFailure: true,
  });
  const cargoEnvironment = {};
  for (const name of ["CARGO_BUILD_TARGET", "CARGO_ENCODED_RUSTFLAGS", "CARGO_INCREMENTAL", "CARGO_TARGET_DIR", "RUSTFLAGS"]) {
    const entry = Object.entries(process.env).find(([key]) => key.toUpperCase() === name);
    cargoEnvironment[name] = entry?.[1] ?? null;
  }
  const mainTargetDirectory = path.join(mainRoot, "src-tauri", "target");
  const mainTargetSnapshot = await snapshotDirectory(mainTargetDirectory);
  return {
    platform: process.platform,
    architecture: process.arch,
    host,
    cargo: cargo.stdout,
    rustc: rustc.stdout,
    node: process.version,
    power: power.result.classification === "ok" ? power.stdout : `unavailable: ${power.stderr || power.result.classification}`,
    defender: defender.result.classification === "ok"
      ? defender.stdout
      : `unavailable: ${defender.stderr || defender.result.classification}`,
    processQuiescence: [],
    operatorProcessAttestation: processAttested,
    cargoEnvironment,
    mainRoot,
    mainSrcTauriTree: mainTree.stdout,
    mainTargetDirectory,
    mainTargetSnapshot,
  };
}

async function createDetachedWorktreeProduction({
  protocolRoot,
  worktree,
  protocolCommit,
  artifactDir,
}) {
  await mkdir(path.dirname(worktree), { recursive: true });
  await controlCommand({
    label: `${path.basename(worktree)}-worktree-add`,
    command: "git.exe",
    args: ["worktree", "add", "--detach", worktree, protocolCommit],
    cwd: protocolRoot,
    artifactDir,
  });
  const head = await controlCommand({
    label: `${path.basename(worktree)}-worktree-head`,
    command: "git.exe",
    args: ["rev-parse", "HEAD"],
    cwd: worktree,
    artifactDir,
  });
  if (head.stdout !== protocolCommit) {
    throw new ProtocolError("detached_worktree_commit_mismatch", `${head.stdout} != ${protocolCommit}`);
  }
}

export async function snapshotDirectory(root) {
  const records = [];
  async function walk(current) {
    const entries = await readdir(current, { withFileTypes: true });
    for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
      const absolute = path.join(current, entry.name);
      const relative = path.relative(root, absolute).replaceAll("\\", "/");
      if (entry.isSymbolicLink()) {
        throw new ProtocolError("target_reparse_point", relative);
      }
      if (entry.isDirectory()) {
        records.push({ path: relative, type: "directory" });
        await walk(absolute);
      } else if (entry.isFile()) {
        const bytes = await readFile(absolute);
        records.push({ path: relative, type: "file", bytes: bytes.length, sha256: sha256Bytes(bytes) });
      } else {
        throw new ProtocolError("target_special_file", relative);
      }
    }
  }
  try {
    await walk(root);
  } catch (error) {
    if (error.code === "ENOENT") {
      const value = { exists: false, records: [] };
      return { ...value, digest: sha256Bytes(canonicalJson(value)) };
    }
    throw error;
  }
  const value = { exists: true, records };
  return { ...value, digest: sha256Bytes(canonicalJson(value)) };
}

const DEFAULT_DEPENDENCIES = Object.freeze({
  uuidFn: randomUUID,
  nowFn: () => new Date().toISOString(),
  processEnv: process.env,
  resolveProtocolCommitFn: resolveProtocolCommitProduction,
  captureEnvironmentFn: captureEnvironmentProduction,
  createDetachedWorktreeFn: createDetachedWorktreeProduction,
  runAttemptFn: runAttempt,
  restoreAttemptWorktreeFn: installState,
  writeJsonFn: writeAtomicJsonExclusive,
  afterDurableWriteFn: async () => {},
  afterAttemptObservedFn: async () => {},
});

function canonicalJson(value) {
  return Buffer.from(`${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function sha256Bytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function writeNewJson(target, value, deps) {
  await deps.writeJsonFn(target, value);
  await deps.afterDurableWriteFn({ target, value });
}

async function publishJsonIdempotent(target, value, deps) {
  try {
    await writeNewJson(target, value, deps);
    return;
  } catch (error) {
    if (error.kind !== "duplicate_artifact") throw error;
  }
  const existing = await readFile(target);
  if (!existing.equals(canonicalJson(value))) {
    throw new ProtocolError("immutable_projection_conflict", target);
  }
}

async function readLedger(sessionDir) {
  const directory = path.join(sessionDir, "ledger");
  let names;
  try {
    names = (await readdir(directory)).filter((name) => /^\d{6}\.json$/.test(name)).sort();
  } catch (error) {
    if (error.code === "ENOENT") return [];
    throw error;
  }
  const entries = [];
  for (let index = 0; index < names.length; index += 1) {
    const expected = `${String(index + 1).padStart(6, "0")}.json`;
    if (names[index] !== expected) throw new ProtocolError("ledger_sequence_gap", `${expected} != ${names[index]}`);
    const entry = JSON.parse(await readFile(path.join(directory, names[index]), "utf8"));
    if (entry.sequence !== index + 1) throw new ProtocolError("ledger_sequence_mismatch", names[index]);
    entries.push(entry);
  }
  return entries;
}

async function appendLedger(sessionDir, value, deps) {
  const sequence = (await readLedger(sessionDir)).length + 1;
  const entry = { schemaVersion: 1, sequence, recordedAt: deps.nowFn(), ...value };
  await writeNewJson(
    path.join(sessionDir, "ledger", `${String(sequence).padStart(6, "0")}.json`),
    entry,
    deps,
  );
  return entry;
}

function latestRetryState(entries) {
  return [...entries].reverse().find((entry) => entry.retryState)?.retryState
    ?? { ...INITIAL_RETRY_STATE };
}

function attemptRows(entries) {
  return entries.filter((entry) => entry.type === "attempt_finished").map((entry) => ({
    attemptId: entry.attemptId,
    status: entry.kind,
    startedAt: entry.startedAt,
    endedAt: entry.endedAt,
    reasons: entry.reasons,
    worktree: entry.worktree,
    targetDirectory: entry.targetDirectory,
    environment: entry.environment,
    resultPath: entry.resultPath,
  }));
}

function snapshot(manifest, entries, status, classification = null) {
  return {
    schemaVersion: 1,
    sessionDir: manifest.sessionDir,
    status,
    classification,
    retryState: latestRetryState(entries),
    attempts: attemptRows(entries),
  };
}

function decisionValue({ result, classification, retryState }) {
  return {
    schemaVersion: 1,
    classification,
    attemptId: result?.attemptId ?? null,
    unexplainedStabilityInvalidCount: retryState.unexplainedStabilityInvalidCount,
    evaluation: result?.evaluation ?? null,
  };
}

async function materializeTerminal(manifest, entries, deps) {
  const terminal = [...entries].reverse().find((entry) => entry.type === "session_completed");
  if (!terminal?.decision) throw new ProtocolError("terminal_event_incomplete", "session_completed lacks decision");
  const aggregate = {
    schemaVersion: 1,
    sessionId: manifest.sessionId,
    unexplainedStabilityInvalidCount: terminal.retryState.unexplainedStabilityInvalidCount,
    terminal: true,
    attempts: attemptRows(entries),
    events: entries,
  };
  await publishJsonIdempotent(path.join(manifest.sessionDir, "session-ledger.json"), aggregate, deps);
  // decision.json is the final commit marker; its presence proves that the
  // aggregate peer was already published byte-identically.
  await publishJsonIdempotent(path.join(manifest.sessionDir, "decision.json"), terminal.decision, deps);
  return snapshot(manifest, entries, "completed", terminal.decision.classification);
}

async function completeSession({ manifest, result, classification, retryState, deps }) {
  let entries = await readLedger(manifest.sessionDir);
  let terminal = [...entries].reverse().find((entry) => entry.type === "session_completed");
  const decision = decisionValue({ result, classification, retryState });
  if (!terminal) {
    terminal = await appendLedger(manifest.sessionDir, {
      type: "session_completed",
      classification,
      retryState,
      decision,
    }, deps);
    entries = await readLedger(manifest.sessionDir);
  } else if (JSON.stringify(terminal.decision) !== JSON.stringify(decision)) {
    throw new ProtocolError("terminal_recovery_conflict", "durable terminal decision differs");
  }
  return materializeTerminal(manifest, entries, deps);
}

function manifestFromLocator(locatorPath, locatorRecord, environment) {
  return {
    schemaVersion: 1,
    sessionId: locatorRecord.sessionId,
    createdAt: locatorRecord.createdAt,
    mainRoot: locatorRecord.mainRoot,
    protocolRoot: locatorRecord.protocolRoot,
    scratchParent: locatorRecord.scratchParent,
    sessionDir: locatorRecord.sessionDir,
    worktreeParent: locatorRecord.worktreeParent,
    locatorPath,
    locatorRecord,
    locatorSha256: sha256Bytes(canonicalJson(locatorRecord)),
    protocolLockPath: locatorRecord.protocolLockPath,
    protocolLock: locatorRecord.protocolLock,
    protocol: locatorRecord.protocol,
    environment,
  };
}

async function loadManifest(sessionDir, deps, { processAttested = false } = {}) {
  const locatorPath = path.join(path.dirname(sessionDir), LOCATOR_NAME);
  const locatorRecord = JSON.parse(await readFile(locatorPath, "utf8"));
  if (path.resolve(locatorRecord.sessionDir) !== path.resolve(sessionDir)) {
    throw new ProtocolError("session_locator_path_mismatch", locatorRecord.sessionDir);
  }
  await mkdir(sessionDir, { recursive: true });
  let manifest = await readOptionalJson(locatorRecord.sessionManifestPath);
  if (!manifest) {
    if (processAttested !== true) {
      throw new ProtocolError("process_attestation_missing", "bootstrap recovery needs fresh quiescence evidence");
    }
    const recoveryNumber = (await readdir(sessionDir, { withFileTypes: true }))
      .filter((entry) => entry.isDirectory() && /^bootstrap-recovery-\d{3}$/.test(entry.name)).length + 1;
    const recoveryDir = path.join(
      sessionDir,
      `bootstrap-recovery-${String(recoveryNumber).padStart(3, "0")}`,
    );
    const environment = await deps.captureEnvironmentFn({
      mainRoot: locatorRecord.mainRoot,
      artifactDir: recoveryDir,
      processAttested: true,
      protocolLock: locatorRecord.protocolLock,
    });
    manifest = manifestFromLocator(locatorPath, locatorRecord, environment);
    await publishJsonIdempotent(locatorRecord.sessionManifestPath, manifest, deps);
  }
  if (
    manifest.locatorRecord.sessionId !== manifest.sessionId
    || path.resolve(manifest.locatorRecord.sessionDir) !== path.resolve(sessionDir)
    || sha256Bytes(canonicalJson(manifest.locatorRecord)) !== manifest.locatorSha256
  ) throw new ProtocolError("session_locator_anchor_mismatch", "manifest locator anchor is invalid");
  await publishJsonIdempotent(manifest.locatorPath, manifest.locatorRecord, deps);
  return manifest;
}

async function assertResumeMaySpawn(sessionDir, processAttested) {
  const manifestExists = await pathExists(path.join(sessionDir, "session-manifest.json"));
  const entries = await readLedger(sessionDir);
  const finished = new Set(
    entries.filter((entry) => entry.type === "attempt_finished").map((entry) => entry.attemptId),
  );
  const unfinishedAttempt = [...entries].reverse().find((entry) =>
    entry.type === "attempt_started" && !finished.has(entry.attemptId),
  );
  const haltedAttempt = unfinishedAttempt && [...entries].reverse().find((entry) =>
    entry.type === "attempt_termination_unconfirmed" && entry.attemptId === unfinishedAttempt.attemptId,
  );
  if ((!manifestExists || unfinishedAttempt) && processAttested !== true) {
    throw new ProtocolError(
      "process_attestation_missing",
      haltedAttempt
        ? `attempt ${haltedAttempt.attemptId} halted with unconfirmed termination; fresh quiescence is required`
        : unfinishedAttempt
          ? `unfinished attempt ${unfinishedAttempt.attemptId} requires fresh quiescence before any child command`
          : "bootstrap recovery requires fresh quiescence before any child command",
    );
  }
}

async function readOptionalJson(filePath) {
  try {
    return JSON.parse(await readFile(filePath, "utf8"));
  } catch (error) {
    if (error.code === "ENOENT") return null;
    throw error;
  }
}

async function readOptionalResult(filePath) {
  try {
    const bytes = await readFile(filePath);
    return {
      result: JSON.parse(bytes.toString("utf8")),
      resultPath: filePath,
      resultSha256: sha256Bytes(bytes),
    };
  } catch (error) {
    if (error.code === "ENOENT") return null;
    throw error;
  }
}

async function readReservedResult(reservation) {
  if (!reservation.sourceResultPath) return null;
  const selected = await readOptionalResult(reservation.sourceResultPath);
  if (!selected || selected.resultSha256 !== reservation.sourceResultSha256) {
    throw new ProtocolError("recovery_source_result_mismatch", reservation.attemptId, {
      expectedPath: reservation.sourceResultPath,
      expectedSha256: reservation.sourceResultSha256,
      actualSha256: selected?.resultSha256 ?? null,
    });
  }
  return selected;
}

async function finishAttempt({ manifest, startedEvent, result, resultPath, environment, deps }) {
  await appendLedger(manifest.sessionDir, {
    type: "attempt_finished",
    attemptId: startedEvent.attemptId,
    kind: result.kind,
    startedAt: startedEvent.recordedAt,
    endedAt: deps.nowFn(),
    reasons: result.reasons ?? [],
    worktree: startedEvent.worktree,
    targetDirectory: startedEvent.targetDirectory,
    environment: environment ?? startedEvent.environment ?? null,
    resultPath,
    classification: result.evaluation?.classification ?? null,
    retryState: startedEvent.retryState,
  }, deps);
  if (result.kind === "valid") {
    const reduced = reduceRetry(startedEvent.retryState, { kind: "valid", objectiveCauseCorrected: false });
    return completeSession({
      manifest,
      result,
      classification: result.evaluation.classification,
      retryState: reduced.state,
      deps,
    });
  }
  const entries = await readLedger(manifest.sessionDir);
  return snapshot(
    manifest,
    entries,
    result.kind === "stability_invalid" ? "awaiting_stability_disposition" : "awaiting_correction",
  );
}

function coordinatorArtifactPath(manifest, attemptId, name) {
  return path.join(manifest.sessionDir, "coordinator", attemptId, name);
}

function finalAProven(manifest, result) {
  return result?.finalState?.kind === "A"
    && result.finalState?.srcTauriTree === manifest.protocolLock.states.A.srcTauriTree;
}

function environmentInvariantProjection(environment) {
  const names = [
    "platform",
    "architecture",
    "host",
    "cargo",
    "rustc",
    "node",
    "cargoEnvironment",
    "mainRoot",
    "mainSrcTauriTree",
    "mainTargetDirectory",
    "mainTargetSnapshot",
  ];
  return Object.fromEntries(names.map((name) => [name, environment?.[name] ?? null]));
}

function operationalEnvironmentProjection(environment) {
  let defender = environment?.defender ?? null;
  try {
    const parsed = JSON.parse(defender);
    defender = {
      RealTimeProtectionEnabled: parsed.RealTimeProtectionEnabled ?? null,
      AntivirusEnabled: parsed.AntivirusEnabled ?? null,
    };
  } catch {
    // Preserve an unavailable/access-denied string, but intentionally ignore
    // naturally drifting QuickScanAge when structured status is available.
  }
  return { power: environment?.power ?? null, defender };
}

function assertAttemptEnvironmentCompatible(entries, current) {
  const baseline = entries.find((entry) => entry.type === "attempt_environment");
  if (!baseline) return { environmentBaseline: true, correctedEnvironmentDelta: null };
  const expected = environmentInvariantProjection(baseline.environment);
  const actual = environmentInvariantProjection(current);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new ProtocolError("attempt_environment_drift", "attempt preflight differs from first measurement baseline", {
      expected,
      actual,
    });
  }
  const operationalExpected = operationalEnvironmentProjection(baseline.environment);
  const operationalActual = operationalEnvironmentProjection(current);
  const operationalChanged = JSON.stringify(operationalActual) !== JSON.stringify(operationalExpected);
  const disposition = [...entries].reverse().find((entry) => entry.type === "retry_disposition");
  const correctedCause = typeof disposition?.correctedCause === "string"
    ? disposition.correctedCause.trim()
    : "";
  if (operationalChanged && correctedCause === "") {
    throw new ProtocolError(
      "attempt_environment_drift",
      "power/Defender drift requires the immediately preceding corrected-cause disposition",
      { operationalExpected, operationalActual },
    );
  }
  return {
    environmentBaseline: false,
    correctedEnvironmentDelta: operationalChanged
      ? { correctedCause, before: operationalExpected, after: operationalActual }
      : null,
  };
}

async function pathExists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

async function normalizeResultArtifact({ manifest, startedEvent, result, resultPath, recoveredFinalState, deps }) {
  if (result.kind === "infrastructure_invalid") return { result, resultPath };
  if (finalAProven(manifest, result)) return { result, resultPath };
  const wrapperPath = coordinatorArtifactPath(manifest, startedEvent.attemptId, "coordinator-failure.json");
  const wrapper = {
    schemaVersion: 1,
    attemptId: startedEvent.attemptId,
    kind: "infrastructure_invalid",
    reasons: ["final_restore_evidence_missing"],
    evaluation: null,
    finalState: recoveredFinalState ?? result.finalState ?? null,
    blocks: result.blocks ?? {},
    sourceResultPath: resultPath,
    error: { kind: "final_restore_evidence_missing" },
  };
  await publishJsonIdempotent(wrapperPath, wrapper, deps);
  return { result: wrapper, resultPath: wrapperPath };
}

async function ensureAttemptRecovered(manifest, startedEvent, selected, deps, processAttested) {
  let entries = await readLedger(manifest.sessionDir);
  const completed = [...entries].reverse().find((entry) =>
    entry.type === "attempt_recovery_completed" && entry.attemptId === startedEvent.attemptId,
  );
  if (completed) return completed;
  if (processAttested !== true) {
    throw new ProtocolError("process_attestation_missing", "unfinished attempt recovery needs fresh quiescence evidence");
  }
  const recoveryNumber = entries.filter((entry) =>
    entry.type === "attempt_recovery_started" && entry.attemptId === startedEvent.attemptId,
  ).length + 1;
  const recoveryId = `recovery-${String(recoveryNumber).padStart(3, "0")}`;
  const recoveryDir = coordinatorArtifactPath(manifest, startedEvent.attemptId, path.join("recoveries", recoveryId));
  await appendLedger(manifest.sessionDir, {
    type: "attempt_recovery_started",
    attemptId: startedEvent.attemptId,
    recoveryId,
    worktree: startedEvent.worktree,
    sourceResultPath: selected?.resultPath ?? null,
    sourceResultSha256: selected?.resultSha256 ?? null,
    retryState: startedEvent.retryState,
  }, deps);
  const environment = await deps.captureEnvironmentFn({
    mainRoot: manifest.mainRoot,
    artifactDir: recoveryDir,
    processAttested: true,
    protocolLock: manifest.protocolLock,
  });
  entries = await readLedger(manifest.sessionDir);
  const worktreeCreated = entries.some((entry) =>
    entry.type === "worktree_created" && entry.attemptId === startedEvent.attemptId,
  );
  const exists = await pathExists(startedEvent.worktree);
  if (!exists && worktreeCreated) {
    throw new ProtocolError("created_worktree_missing", startedEvent.worktree);
  }
  let finalState = null;
  if (exists) {
    finalState = await deps.restoreAttemptWorktreeFn({
      state: "A-final",
      worktree: startedEvent.worktree,
      mainRoot: manifest.mainRoot,
      protocolLock: manifest.protocolLock,
      artifactDir: recoveryDir,
    });
    if (!finalAProven(manifest, { finalState })) {
      throw new ProtocolError("recovery_a_mismatch", startedEvent.attemptId, { finalState });
    }
  }
  return appendLedger(manifest.sessionDir, {
    type: "attempt_recovery_completed",
    attemptId: startedEvent.attemptId,
    recoveryId,
    worktree: startedEvent.worktree,
    worktreeAbsent: !exists,
    finalState,
    environment,
    retryState: startedEvent.retryState,
  }, deps);
}

async function recoverStartedAttempt(manifest, startedEvent, deps, { processAttested = false } = {}) {
  const attemptDir = path.join(manifest.sessionDir, "attempts", startedEvent.attemptId);
  const entries = await readLedger(manifest.sessionDir);
  const environment = [...entries].reverse().find((entry) =>
    entry.type === "attempt_environment" && entry.attemptId === startedEvent.attemptId,
  )?.environment ?? startedEvent.environment ?? null;
  const normalPath = path.join(attemptDir, "attempt-result.json");
  const failurePath = coordinatorArtifactPath(manifest, startedEvent.attemptId, "coordinator-failure.json");
  const interruptionPath = coordinatorArtifactPath(manifest, startedEvent.attemptId, "coordinator-interruption.json");
  const recoveryReservation = [...entries].reverse().find((entry) =>
    entry.type === "attempt_recovery_started" && entry.attemptId === startedEvent.attemptId,
  );
  // A durable coordinator failure is terminal evidence and always outranks an
  // earlier recovery reservation. This makes failure publication replay-safe.
  let selected = await readOptionalResult(failurePath)
    ?? (recoveryReservation
      ? await readReservedResult(recoveryReservation)
      : await readOptionalResult(interruptionPath)
        ?? await readOptionalResult(normalPath));
  let result = selected?.result ?? null;
  let resultPath = selected?.resultPath ?? null;
  let recovery = null;
  if (!result || !finalAProven(manifest, result)) {
    recovery = await ensureAttemptRecovered(manifest, startedEvent, selected, deps, processAttested);
  }
  if (!result) {
    resultPath = interruptionPath;
    result = {
      schemaVersion: 1,
      attemptId: startedEvent.attemptId,
      kind: "infrastructure_invalid",
      reasons: ["coordinator_interrupted"],
      evaluation: null,
      finalState: recovery.finalState,
      blocks: {},
      recoveryId: recovery.recoveryId,
      worktreeAbsent: recovery.worktreeAbsent,
      error: { kind: "coordinator_interrupted" },
    };
    await publishJsonIdempotent(resultPath, result, deps);
  }
  if (result.attemptId !== startedEvent.attemptId) {
    throw new ProtocolError("attempt_result_identity_mismatch", startedEvent.attemptId);
  }
  const normalized = await normalizeResultArtifact({
    manifest,
    startedEvent,
    result,
    resultPath,
    recoveredFinalState: recovery?.finalState ?? null,
    deps,
  });
  return finishAttempt({
    manifest,
    startedEvent,
    result: normalized.result,
    resultPath: normalized.resultPath,
    environment: environment ?? recovery?.environment ?? null,
    deps,
  });
}

async function launchAttempt(manifest, retryState, deps, processAttested) {
  assertCargoTargetDirUnset(deps.processEnv);
  const entries = await readLedger(manifest.sessionDir);
  const attemptNumber = entries.filter((entry) => entry.type === "attempt_started").length + 1;
  const attemptId = `attempt-${String(attemptNumber).padStart(3, "0")}`;
  const attemptDir = path.join(manifest.sessionDir, "attempts", attemptId);
  const worktree = path.join(manifest.worktreeParent, attemptId);
  const targetDirectory = path.join(worktree, "src-tauri", "target");
  await mkdir(path.dirname(attemptDir), { recursive: true });
  await mkdir(path.dirname(worktree), { recursive: true });
  await assertMissing(attemptDir, "attempt_directory_exists");
  await assertMissing(worktree, "attempt_worktree_exists");
  await assertMissing(targetDirectory, "fresh_target_already_exists");
  if (processAttested !== true) {
    throw new ProtocolError("process_attestation_missing", "every fresh attempt needs a new quiescence attestation");
  }
  const startedEvent = await appendLedger(manifest.sessionDir, {
    type: "attempt_started",
    attemptId,
    startedAt: deps.nowFn(),
    worktree,
    targetDirectory,
    processAttested: true,
    environment: null,
    retryState,
  }, deps);

  let coordinatorFailure = null;
  try {
    await mkdir(attemptDir, { recursive: false });
    const attemptEnvironment = await deps.captureEnvironmentFn({
      mainRoot: manifest.mainRoot,
      artifactDir: attemptDir,
      processAttested: true,
      protocolLock: manifest.protocolLock,
    });
    const environmentDisposition = assertAttemptEnvironmentCompatible(
      await readLedger(manifest.sessionDir),
      attemptEnvironment,
    );
    await appendLedger(manifest.sessionDir, {
      type: "attempt_environment",
      attemptId,
      environment: attemptEnvironment,
      ...environmentDisposition,
      retryState,
    }, deps);
    await appendLedger(manifest.sessionDir, {
      type: "worktree_creation_started",
      attemptId,
      worktree,
      retryState,
    }, deps);
    await deps.createDetachedWorktreeFn({
      protocolRoot: manifest.protocolRoot,
      worktree,
      protocolCommit: manifest.protocol.protocolCommit,
      artifactDir: attemptDir,
    });
    await appendLedger(manifest.sessionDir, {
      type: "worktree_created",
      attemptId,
      worktree,
      retryState,
    }, deps);
    await assertMissing(targetDirectory, "fresh_target_already_exists");
    const resultPath = path.join(attemptDir, "attempt-result.json");
    const result = await deps.runAttemptFn({
      worktree,
      mainRoot: manifest.mainRoot,
      sessionDir: manifest.sessionDir,
      attemptId,
      protocolLock: manifest.protocolLock,
    });
    const persisted = JSON.parse(await readFile(resultPath, "utf8"));
    if (JSON.stringify(persisted) !== JSON.stringify(result)) {
      throw new ProtocolError("attempt_return_artifact_mismatch", attemptId);
    }
    await deps.afterAttemptObservedFn({ attemptId, resultPath });
  } catch (error) {
    if (error?.simulatedCrash === true) throw error;
    const terminationUnconfirmed = hasTerminationUnconfirmed(error);
    coordinatorFailure = {
      schemaVersion: 1,
      attemptId,
      kind: "infrastructure_invalid",
      reasons: ["coordinator_failure"],
      evaluation: null,
      finalState: null,
      blocks: {},
      error: {
        name: error?.name ?? "Error",
        kind: error?.kind ?? "coordinator_failure",
        message: error?.message ?? String(error),
      },
    };
    if (terminationUnconfirmed) {
      let markerError = null;
      let failurePublicationError = null;
      try {
        await appendLedger(manifest.sessionDir, {
          type: "attempt_termination_unconfirmed",
          attemptId,
          worktree,
          targetDirectory,
          resultPath: path.join(attemptDir, "attempt-result.json"),
          operatorActionRequired: true,
          retryState,
        }, deps);
      } catch (writeError) {
        markerError = writeError;
      }
      try {
        await publishJsonIdempotent(
          coordinatorArtifactPath(manifest, attemptId, "coordinator-failure.json"),
          coordinatorFailure,
          deps,
        );
      } catch (writeError) {
        failurePublicationError = writeError;
      }
      throw new ProtocolError("termination_unconfirmed", attemptId, {
        operatorActionRequired: true,
        resultPath: path.join(attemptDir, "attempt-result.json"),
        markerError: markerError
          ? { kind: markerError.kind ?? markerError.name, message: markerError.message }
          : null,
        failurePublicationError: failurePublicationError
          ? { kind: failurePublicationError.kind ?? failurePublicationError.name, message: failurePublicationError.message }
          : null,
      });
    }
    await publishJsonIdempotent(
      coordinatorArtifactPath(manifest, attemptId, "coordinator-failure.json"),
      coordinatorFailure,
      deps,
    );
  }
  return recoverStartedAttempt(manifest, startedEvent, deps, { processAttested: true });
}

export async function startSession(options, overrides = {}) {
  const deps = { ...DEFAULT_DEPENDENCIES, ...overrides };
  if (options.processAttested !== true) {
    throw new ProtocolError("process_attestation_missing", "operator must attest build-process quiescence");
  }
  assertCargoTargetDirUnset(deps.processEnv);
  const mainRoot = path.resolve(options.mainRoot);
  const protocolRoot = path.resolve(options.protocolRoot);
  const scratchParent = path.resolve(options.scratchParent);
  const locatorPath = path.join(scratchParent, LOCATOR_NAME);
  await mkdir(scratchParent, { recursive: true });
  await assertMissing(locatorPath, "session_locator_exists");
  const orphanSessions = (await readdir(scratchParent, { withFileTypes: true }))
    .filter((entry) => entry.isDirectory() && entry.name.startsWith("process-shell-session-"));
  if (orphanSessions.length) {
    throw new ProtocolError("orphan_session_exists", "resume or audit the existing session directory", {
      sessions: orphanSessions.map((entry) => path.join(scratchParent, entry.name)),
    });
  }
  const sessionId = deps.uuidFn();
  const sessionDir = path.join(scratchParent, `process-shell-session-${sessionId}`);
  const protocolLockPath = path.join(
    protocolRoot,
    "scripts",
    "process-shell-diagnostic",
    "protocol-lock.json",
  );
  const protocolLock = JSON.parse(await readFile(protocolLockPath, "utf8"));
  const bootstrapDir = path.join(scratchParent, `bootstrap-${sessionId}`);
  const protocolCommit = await deps.resolveProtocolCommitFn({ protocolRoot, artifactDir: bootstrapDir });
  const locatorRecord = {
    schemaVersion: 1,
    sessionId,
    createdAt: deps.nowFn(),
    mainRoot,
    protocolRoot,
    scratchParent,
    sessionDir,
    worktreeParent: path.join(mainRoot, ".worktrees", `process-shell-session-${sessionId}`),
    sessionManifestPath: path.join(sessionDir, "session-manifest.json"),
    protocolLockPath,
    protocolLock,
    protocol: { protocolCommit },
  };
  // The external locator is the bootstrap reservation WAL. Nothing creates the
  // final session directory until this exact recovery seed is durable.
  await publishJsonIdempotent(locatorPath, locatorRecord, deps);
  await mkdir(sessionDir, { recursive: true });
  const environment = await deps.captureEnvironmentFn({
    mainRoot,
    artifactDir: path.join(sessionDir, "bootstrap"),
    processAttested: true,
    protocolLock,
  });
  const manifest = manifestFromLocator(locatorPath, locatorRecord, environment);
  await publishJsonIdempotent(locatorRecord.sessionManifestPath, manifest, deps);
  await appendLedger(sessionDir, {
    type: "session_started",
    sessionId,
    protocolCommit,
    retryState: { ...INITIAL_RETRY_STATE },
  }, deps);
  return launchAttempt(manifest, { ...INITIAL_RETRY_STATE }, deps, true);
}

export async function resumeSession(options, overrides = {}) {
  const deps = { ...DEFAULT_DEPENDENCIES, ...overrides };
  const sessionDir = path.resolve(options.sessionDir);
  // This filesystem-only gate runs before Task 6's Git-backed loadManifest
  // verifier. A possibly live descendant is never followed by another child
  // command until the operator supplies a new quiescence attestation.
  await assertResumeMaySpawn(sessionDir, options.processAttested === true);
  const manifest = await loadManifest(sessionDir, deps, { processAttested: options.processAttested === true });
  let entries = await readLedger(sessionDir);
  if (entries.some((entry) => entry.type === "session_completed")) {
    return materializeTerminal(manifest, entries, deps);
  }
  if (!entries.some((entry) => entry.type === "session_started")) {
    if (options.processAttested !== true) {
      throw new ProtocolError("process_attestation_missing", "bootstrap recovery needs a fresh quiescence attestation");
    }
    await appendLedger(sessionDir, {
      type: "session_started",
      sessionId: manifest.sessionId,
      protocolCommit: manifest.protocol.protocolCommit,
      retryState: { ...INITIAL_RETRY_STATE },
    }, deps);
    entries = await readLedger(sessionDir);
  }
  const finishedIds = new Set(entries.filter((entry) => entry.type === "attempt_finished").map((entry) => entry.attemptId));
  const unfinished = [...entries].reverse().find((entry) =>
    entry.type === "attempt_started" && !finishedIds.has(entry.attemptId),
  );
  if (unfinished) {
    return recoverStartedAttempt(manifest, unfinished, deps, {
      processAttested: options.processAttested === true,
    });
  }

  const lastAttempt = [...entries].reverse().find((entry) => entry.type === "attempt_finished");
  if (!lastAttempt) {
    if (options.processAttested !== true) {
      throw new ProtocolError("process_attestation_missing", "first attempt recovery needs a fresh quiescence attestation");
    }
    return launchAttempt(manifest, { ...INITIAL_RETRY_STATE }, deps, true);
  }
  const retryState = latestRetryState(entries);
  if (lastAttempt.kind === "valid") {
    const result = JSON.parse(await readFile(lastAttempt.resultPath, "utf8"));
    const reduced = reduceRetry(lastAttempt.retryState, { kind: "valid", objectiveCauseCorrected: false });
    return completeSession({
      manifest,
      result,
      classification: result.evaluation.classification,
      retryState: reduced.state,
      deps,
    });
  }
  const correctedCause = typeof options.correctedCause === "string" && options.correctedCause.trim()
    ? options.correctedCause.trim()
    : null;
  const unexplained = options.unexplainedStability === true;
  if (correctedCause && unexplained) {
    throw new ProtocolError("retry_disposition_ambiguous", "choose corrected cause or unexplained stability");
  }
  const priorDisposition = entries.find((entry) =>
    entry.type === "retry_disposition" && entry.attemptId === lastAttempt.attemptId,
  );
  if (priorDisposition) {
    if (
      (correctedCause && correctedCause !== priorDisposition.correctedCause)
      || (unexplained && priorDisposition.unexplainedStability !== true)
    ) throw new ProtocolError("retry_replay_conflict", lastAttempt.attemptId);
    if (priorDisposition.retryAction === "environment_precision_insufficient") {
      return completeSession({
        manifest,
        result: null,
        classification: "environment_precision_insufficient",
        retryState: priorDisposition.retryState,
        deps,
      });
    }
    if (priorDisposition.retryAction !== "retry") {
      throw new ProtocolError("retry_action_invalid", priorDisposition.retryAction);
    }
    if (options.processAttested !== true) {
      throw new ProtocolError("process_attestation_missing", "recovered retry needs a fresh quiescence attestation");
    }
    return launchAttempt(manifest, priorDisposition.retryState, deps, true);
  }
  if (lastAttempt.kind === "infrastructure_invalid" && !correctedCause) {
    return snapshot(manifest, entries, "awaiting_correction");
  }
  if (lastAttempt.kind === "stability_invalid" && !correctedCause && !unexplained) {
    return snapshot(manifest, entries, "awaiting_stability_disposition");
  }
  if (lastAttempt.kind === "infrastructure_invalid" && unexplained) {
    throw new ProtocolError("retry_disposition_invalid", "infrastructure failure needs a corrected cause");
  }
  const terminatesForPrecision =
    lastAttempt.kind === "stability_invalid" &&
    unexplained &&
    retryState.unexplainedStabilityInvalidCount >= 1;
  if (!terminatesForPrecision && options.processAttested !== true) {
    throw new ProtocolError("process_attestation_missing", "a retry needs a fresh quiescence attestation");
  }
  const reduced = reduceRetry(retryState, {
    kind: lastAttempt.kind,
    objectiveCauseCorrected: correctedCause !== null,
  });
  await appendLedger(sessionDir, {
    type: "retry_disposition",
    attemptId: lastAttempt.attemptId,
    invalidationKind: lastAttempt.kind,
    correctedCause,
    unexplainedStability: unexplained,
    retryAction: reduced.action,
    retryState: reduced.state,
  }, deps);
  if (reduced.action === "environment_precision_insufficient") {
    return completeSession({
      manifest,
      result: null,
      classification: "environment_precision_insufficient",
      retryState: reduced.state,
      deps,
    });
  }
  if (reduced.action !== "retry") throw new ProtocolError("retry_action_invalid", reduced.action);
  return launchAttempt(manifest, reduced.state, deps, true);
}

function parseFlags(tokens) {
  const values = {};
  const booleans = new Set(["--process-attested", "--unexplained-stability"]);
  for (let index = 0; index < tokens.length; index += 1) {
    const flag = tokens[index];
    if (!flag.startsWith("--") || Object.hasOwn(values, flag)) throw new Error(`invalid or duplicate flag ${flag}`);
    if (booleans.has(flag)) values[flag] = true;
    else {
      if (tokens[index + 1] === undefined || tokens[index + 1].startsWith("--")) throw new Error(`missing value for ${flag}`);
      values[flag] = tokens[index + 1];
      index += 1;
    }
  }
  return values;
}

function required(values, flag) {
  if (!values[flag]) throw new Error(`missing required flag ${flag}`);
  return values[flag];
}

export function parseCli(argv) {
  const [command, ...tokens] = argv;
  const values = parseFlags(tokens);
  if (command === "start") return {
    command,
    options: {
      mainRoot: required(values, "--main-root"),
      protocolRoot: required(values, "--protocol-root"),
      scratchParent: required(values, "--scratch-parent"),
      processAttested: values["--process-attested"] === true,
    },
  };
  if (command === "resume") {
    const options = {
      sessionDir: required(values, "--session-dir"),
      unexplainedStability: values["--unexplained-stability"] === true,
      processAttested: values["--process-attested"] === true,
    };
    if (values["--corrected-cause"]) options.correctedCause = values["--corrected-cause"];
    if (!options.unexplainedStability) delete options.unexplainedStability;
    if (!options.processAttested) delete options.processAttested;
    return { command, options };
  }
  throw new Error(`expected start or resume, got ${command ?? "missing"}`);
}

async function main() {
  const parsed = parseCli(process.argv.slice(2));
  const result = parsed.command === "start"
    ? await startSession(parsed.options)
    : await resumeSession(parsed.options);
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  await main();
}
```
- [ ] **Step 4: Run coordinator GREEN and the combined harness tests**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/coordinator.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Coordinator GREEN failed.' }
npm.cmd run test -- scripts/process-shell-diagnostic/protocol.test.ts scripts/process-shell-diagnostic/runtime.test.ts scripts/process-shell-diagnostic/git-state.test.ts scripts/process-shell-diagnostic/attempt.test.ts scripts/process-shell-diagnostic/coordinator.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Combined Task 5 harness tests failed.' }
npm.cmd run check
if ($LASTEXITCODE -ne 0) { throw 'Task 5 repository check failed.' }
```

Expected: twenty-four coordinator tests PASS, including the locator/attempt/recovery
crash matrix; every prior harness test remains GREEN,
and `svelte-check` reports zero errors.

- [ ] **Step 5: Extend the diagnostic registry with attempt/session/ledger values**

Insert these rows into the Task 1 process-shell diagnostic table in
`docs/value-registry.md`:

```markdown
| `valid` | kind | Valid attempt | Every command and validity rule passed and a terminal diagnostic classification exists. | diagnostic coordinator | terminal | none | n/a | yes | attempt result, numbered ledger |
| `stability_invalid` | kind | Stability invalid | Anchor range or central-five stability invalidated the complete attempt. | diagnostic coordinator | terminal | inspect_error | n/a | yes | attempt result, numbered ledger |
| `infrastructure_invalid` | kind | Infrastructure invalid | A command, metadata, restore, state, target, or environment contract invalidated the attempt. | diagnostic coordinator | terminal | inspect_error | n/a | yes | attempt result, numbered ledger |
| `awaiting_stability_disposition` | status | Awaiting stability disposition | Resume must record a corrected cause or explicitly consume one unexplained-stability result. | diagnostic coordinator | transitional | choose | n/a | yes | coordinator result |
| `awaiting_correction` | status | Awaiting correction | An objective infrastructure cause must be recorded and corrected before a fresh attempt. | diagnostic coordinator | transitional | configure | n/a | yes | coordinator result |
| `completed` | status | Completed | A valid or terminal precision outcome permanently closes this same-protocol session. | diagnostic coordinator | terminal | none | n/a | yes | coordinator result, session ledger |
| `ok` | process classification | Process OK | The owned child reached `close` with exit code zero before the deadline. | diagnostic runtime | terminal | none | n/a | yes | raw process JSON |
| `command_failed` | process classification | Process command failed | The owned child reached `close` with a nonzero exit code before the deadline. | diagnostic runtime | terminal | inspect_error | n/a | yes | raw process JSON |
| `timeout` | process classification | Process timeout | The deadline expired and bounded tree termination was positively confirmed. | diagnostic runtime | terminal | inspect_error | n/a | yes | raw process JSON |
| `termination_unconfirmed` | process classification | Termination unconfirmed | The child did not close or its timed-out process tree could not be proven dead; no later child command may run in that invocation, and resume needs fresh operator attestation. | diagnostic runtime | terminal | inspect_error | n/a | yes | raw process JSON |
| `spawn_error` | process classification | Process spawn error | The owned child could not be started. | diagnostic runtime | terminal | inspect_error | n/a | yes | raw process JSON |
| `complete` | retry action | Complete | A valid attempt terminally completes the retry reducer. | diagnostic protocol | terminal | none | n/a | yes | reducer result, session completion |
| `retry` | retry action | Retry | The identical frozen protocol may start one fresh attempt under the monotonic retry state. | diagnostic protocol | transitional | retry | n/a | yes | retry disposition, numbered ledger |
| `await_correction` | retry action | Await correction | An objective infrastructure cause must be recorded as corrected before retry. | diagnostic protocol | transitional | configure | n/a | yes | reducer result, coordinator status |
| `environment_precision_insufficient` | retry action | Precision terminal | The second unexplained stability invalidation terminates same-protocol retry. | diagnostic protocol | terminal | inspect_error | n/a | yes | retry disposition, numbered ledger |
| `session_started` | kind | Session started | Immutable manifest and locator were created. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `attempt_started` | kind | Attempt started | Exact attempt/worktree/target paths were durably reserved before the attempt directory or worktree is created. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `attempt_environment` | kind | Attempt environment | Fresh preflight and operator attestation were captured; the first event fixes the measurement baseline, while later power/Defender deltas require and record the immediately preceding corrected cause. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `worktree_creation_started` | kind | Worktree creation started | Creation of the named detached worktree has begun; a missing path now requires explicit failure evidence. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `worktree_created` | kind | Worktree created | The detached worktree exists at the frozen protocol commit. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `attempt_recovery_started` | kind | Attempt recovery started | A uniquely numbered fresh-quiescence/A-restore recovery begins and irrevocably pins the preexisting source-result path plus SHA-256, or pins their absence so late output is ignored. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `attempt_recovery_completed` | kind | Attempt recovery completed | The reserved worktree was restored to exact A, or its pre-creation absence was proven, before interruption projection. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `attempt_termination_unconfirmed` | kind | Attempt termination unconfirmed | The coordinator durably halted an unfinished attempt because an owned descendant may still be alive; no recovery child may start until a newly attested resume. | diagnostic coordinator | transitional | inspect_error | n/a | yes | numbered ledger |
| `attempt_finished` | kind | Attempt finished | The immutable attempt result or interruption invalidation was recorded. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `retry_disposition` | kind | Retry disposition | The monotonic reducer consumed an explicit corrected/unexplained disposition. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `session_completed` | kind | Session completed | A valid or terminal result forbids another same-protocol attempt. | diagnostic coordinator | taxonomy | none | n/a | yes | numbered ledger |
| `anchor_range_exceeded` | reason | Anchor range exceeded | A-anchor median range is above 300 ms. | diagnostic protocol | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `block_unstable:<block>` | reason namespace | Block unstable | Fewer than five of seven samples in the named frozen block are within 300 ms of its median. | diagnostic protocol | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `command_timeout` | reason | Command timeout | A bounded child command exceeded the frozen timeout. | diagnostic attempt runner | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `command_failed` | reason | Command failed | Cargo, Git, spawn, or another required control command failed. | diagnostic attempt runner | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `restore_failed` | reason | Restore failed | Canonical/recovery bytes could not be proven restored. | diagnostic attempt runner | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `target_invalid` | reason | Target invalid | Workspace or target isolation evidence violated the frozen contract. | diagnostic attempt runner | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `metadata_invalid` | reason | Metadata invalid | Required Cargo duration, timing, checked-package, direct-rustc-edge, or inventory evidence was absent or inconsistent. | diagnostic attempt runner | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `state_invalid` | reason | State invalid | Patch, tree, blob, manifest, or state evidence disagreed with the lock. | diagnostic attempt runner | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `environment_invalid` | reason | Environment invalid | Platform, host, attestation, or quiescence evidence violated the preregistration. | diagnostic attempt runner | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `protocol_violation` | reason | Protocol violation | An otherwise unmapped harness invariant failed; detailed `error.kind` remains diagnostic, not a decision taxonomy. | diagnostic attempt runner | terminal | inspect_error | n/a | yes | attempt result, ledger |
| `coordinator_failure` | reason | Coordinator failure | Worktree creation, pinning, or coordinator control failed outside an attempt probe. | diagnostic coordinator | terminal | inspect_error | n/a | yes | coordinator failure, ledger |
| `coordinator_interrupted` | reason | Coordinator interrupted | Resume found an unfinished attempt without a durable result artifact. | diagnostic coordinator | terminal | inspect_error | n/a | yes | interruption result, ledger |
| `final_restore_evidence_missing` | reason | Final restore evidence missing | A result could not prove exact final A and is therefore infrastructure-invalid. | diagnostic coordinator | terminal | inspect_error | n/a | yes | failure result, ledger |
```

Retain the section note that the harness owns these values and that SQLite,
product API, UI, and product fixture impact are all `none`.

- [ ] **Step 6: Commit the coordinator and registry extension**

Run:

```powershell
$diagnosticTask5Status = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect Task 5 status.' }
$diagnosticTask5Status | ForEach-Object { Write-Output $_ }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 5 diff check failed.' }
git add -- scripts/process-shell-diagnostic/coordinator.mjs scripts/process-shell-diagnostic/coordinator.test.ts docs/value-registry.md
if ($LASTEXITCODE -ne 0) { throw 'Could not stage Task 5 files.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Task 5 staged diff check failed.' }
git commit -m "feat: coordinate immutable diagnostic sessions"
if ($LASTEXITCODE -ne 0) { throw 'Task 5 commit failed.' }
```

Expected: only Task 5's coordinator, tests, and diagnostic registry rows are
committed.

### Task 6: Freeze Every Protocol Input and Build the Evidence Renderer

**Files:**

- Create: `scripts/process-shell-diagnostic/freeze.mjs`
- Create: `scripts/process-shell-diagnostic/report.test.ts`
- Create: `scripts/process-shell-diagnostic/report.mjs`
- Modify: `scripts/process-shell-diagnostic/coordinator.mjs`
- Modify: `scripts/process-shell-diagnostic/coordinator.test.ts`
- Create: `scripts/process-shell-diagnostic/protocol-lock.json` (generated only
  after all other protocol inputs are committed)

**Interfaces:**

- Produces `buildProtocolLock({ repoRoot })`, `verifyFrozenProtocol({
  repoRoot }) -> { ProtocolPin, protocolLock }`, `prepareArtifactIndex(sessionDir, locatorPath)`, and
  `renderVerification({ sessionManifest, measurementEnvironment, ledger,
  decision, attemptResults, artifactIndex })`.
- Exposes the single frozen `REPORT_PATH`, crash-replay-safe `--verify-only`,
  `deriveLedgerProjection(sessionId, events)`, and
  `assertRetryProtocol({ events, attemptResults, decision })`; no arbitrary
  report destination or trusted aggregate/retry projection is accepted.
- `ProtocolPin` is exactly `{ protocolCommit, lockPath, lockBlob, lockSha256,
  protocolVersion }`; the lock hashes every input except itself, while the
  session pins the lock-containing commit and blob.
- The report independently recalculates medians, local A references, deltas,
  stability, E eligibility, shell caps, classification, and contrasts before
  rendering recorded decision data. Its primary environment table comes from
  the first authoritative `attempt_environment`, not the earlier bootstrap
  snapshot in `session-manifest.json`.

- [ ] **Step 1: Write report and independent-recalculation RED tests**

Create `scripts/process-shell-diagnostic/report.test.ts`:

```ts
import { access, link, mkdtemp, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { assertProtocolWorktreeStatus } from "./freeze.mjs";
import {
  assertProtocolPinMatches,
  assertRetryProtocol,
  cleanupOwnedAtomicTemps,
  deriveLedgerProjection,
  publishReportPair,
  renderVerification,
  verifyReportProtocol,
} from "./report.mjs";
import { writeAtomicBytesExclusive } from "./runtime.mjs";

const samples = (value: number) => Array(7).fill(value);
const summary = (value: number) => ({
  samplesMs: samples(value),
  medianMs: value,
  samplesWithinBand: 7,
  stable: true,
});
const metric = (variant: number, reference: number) => {
  const deltaMs = variant - reference;
  return {
    variantMedianMs: variant,
    aReferenceMs: reference,
    deltaMs,
    percentDelta: 100 * deltaMs / reference,
    material: deltaMs >= 500,
    shellCapFailed: deltaMs > 500 || (100 * deltaMs / reference) > 5,
  };
};
const block = (value: number, extractumProcessExtern = false) => ({
  samples: samples(value).map((wallMs, index) => ({
    index: index + 1,
    wallMs,
    cargoReportedMs: wallMs - 20,
    checkedPackages: ["extractum"],
  })),
  summary: summary(value),
  noOp: { elapsedMs: 80, cargoReportedMs: 60 },
  diagnostic: {
    extractumProcessExtern,
    timingArtifact: { path: `timing-${value}.html`, sha256: "f".repeat(64) },
  },
});

const sessionManifest = {
  schemaVersion: 1,
  sessionId: "session-fixed",
  createdAt: "2026-07-18T08:00:00.000Z",
  sessionDir: "G:\\raw\\session-fixed",
  protocolRoot: "G:\\protocol",
  protocol: {
    protocolCommit: "1".repeat(40),
    lockPath: "scripts/process-shell-diagnostic/protocol-lock.json",
    lockBlob: "2".repeat(40),
    lockSha256: "3".repeat(64),
    protocolVersion: 1,
  },
  protocolLock: { schemaVersion: 1, protocolVersion: 1, states: {} },
  environment: {
    platform: "win32",
    host: "x86_64-pc-windows-msvc",
    cargo: "cargo 1.95.0",
    rustc: "rustc 1.95.0",
    power: "Balanced",
    defender: "unavailable: Access denied",
    mainTargetDirectory: "G:\\Develop\\Extractum\\src-tauri\\target",
  },
};

const evaluation = {
  kind: "valid",
  classification: "boundary_composite",
  eRequired: true,
  anchorRangeMs: 0,
  disagreement: [],
  summaries: {
    A0: summary(9_000), B: summary(9_100), A1: summary(9_000),
    C: summary(9_200), A2: summary(9_000), D: summary(9_800),
    A3: summary(9_000), E: summary(9_250), A4: summary(9_000),
  },
  metrics: {
    B: metric(9_100, 9_000),
    C: metric(9_200, 9_000),
    D: metric(9_800, 9_000),
    E: metric(9_250, 9_000),
  },
  contrasts: {
    membershipMs: 100,
    edgeAfterMembershipMs: 100,
    manifestAfterCMs: 50,
    dSpecificCompositeMs: 550,
    dAfterCCompositeMs: null,
  },
};

const causal = {
  sessionManifest,
  measurementEnvironment: {
    ...sessionManifest.environment,
    power: "High performance",
  },
  ledger: {
    schemaVersion: 1,
    sessionId: "session-fixed",
    unexplainedStabilityInvalidCount: 0,
    terminal: true,
    attempts: [{
      attemptId: "attempt-001",
      status: "valid",
      startedAt: "2026-07-18T08:01:00.000Z",
      endedAt: "2026-07-18T09:18:00.000Z",
      reasons: [],
    }],
  },
  decision: {
    schemaVersion: 1,
    classification: "boundary_composite",
    attemptId: "attempt-001",
    unexplainedStabilityInvalidCount: 0,
    evaluation,
  },
  attemptResults: [{
    attemptId: "attempt-001",
    kind: "valid",
    evaluation,
    blocks: Object.fromEntries(
      Object.entries(evaluation.summaries).map(([name, value]) => [
        name,
        block(value.medianMs, ["C", "D", "E"].includes(name)),
      ]),
    ),
  }],
  artifactIndex: { sha256: "4".repeat(64), files: 211, bytes: 123_456 },
};

describe("process shell diagnostic report", () => {
  it("renders causal raw evidence, no-op timings, and cumulative contrasts", () => {
    const report = renderVerification(causal);
    expect(report).toContain("**Outcome:** `boundary_composite`");
    expect(report).toContain("**Protocol-lock blob:** `2222222222222222222222222222222222222222`");
    expect(report).toContain("| D | 9800, 9800, 9800, 9800, 9800, 9800, 9800 | 9800 ms | 7/7 | 80 ms | 60 ms |");
    expect(report).toContain("| D | 9780, 9780, 9780, 9780, 9780, 9780, 9780 | true |");
    expect(report).toContain("| D-specific composite | +550 ms |");
    expect(report).toContain("descriptive contrasts between cumulative configurations");
    expect(report).toContain("does not automatically retain `extractum-process` or unblock Phase 4");
    expect(report).toContain("High performance");
    expect(report).not.toContain("| power | Balanced |");
    expect(report).toContain("## Retry and invalidation audit");
    expect(report).toContain("### Attempt error details");
    expect(report).toContain("### Corrected environment deltas");
  });

  it("rejects a recorded decision that disagrees with independent arithmetic", () => {
    const corrupted = structuredClone(causal);
    corrupted.decision.evaluation.metrics.D.deltaMs = 799;
    expect(() => renderVerification(corrupted)).toThrow("independent recalculation mismatch");
  });

  it("rejects a decision detached from the raw attempt samples", () => {
    const corrupted = structuredClone(causal);
    for (const sample of corrupted.attemptResults[0].blocks.D.samples) sample.wallMs = 9_100;
    expect(() => renderVerification(corrupted)).toThrow("raw attempt evidence mismatch");
  });

  it("derives the aggregate from numbered events and rejects an attempt after valid", () => {
    const retryState = { unexplainedStabilityInvalidCount: 0, terminal: false };
    const finished = causal.ledger.attempts[0];
    const events = [
      { schemaVersion: 1, sequence: 1, type: "session_started", sessionId: "session-fixed", retryState },
      {
        schemaVersion: 1,
        sequence: 2,
        type: "attempt_started",
        attemptId: "attempt-001",
        retryState,
      },
      {
        schemaVersion: 1,
        sequence: 3,
        type: "attempt_finished",
        attemptId: "attempt-001",
        kind: "valid",
        startedAt: finished.startedAt,
        endedAt: finished.endedAt,
        reasons: [],
        worktree: "G:\\attempt-001",
        targetDirectory: "G:\\attempt-001\\src-tauri\\target",
        environment: causal.measurementEnvironment,
        resultPath: "G:\\raw\\attempt-001.json",
        classification: "boundary_composite",
        retryState,
      },
      {
        schemaVersion: 1,
        sequence: 4,
        type: "session_completed",
        classification: "boundary_composite",
        retryState: { unexplainedStabilityInvalidCount: 0, terminal: true },
        decision: causal.decision,
      },
    ];
    const derived = deriveLedgerProjection("session-fixed", events);
    expect(derived).toMatchObject({ terminal: true, attempts: [{ attemptId: "attempt-001", status: "valid" }] });
    expect(() => assertRetryProtocol({
      events,
      attemptResults: causal.attemptResults,
      decision: causal.decision,
    })).not.toThrow();
    const illegal = structuredClone(events);
    illegal.splice(3, 0, {
      schemaVersion: 1,
      sequence: 4,
      type: "attempt_started",
      attemptId: "attempt-002",
      retryState,
    });
    illegal[4].sequence = 5;
    expect(() => assertRetryProtocol({
      events: illegal,
      attemptResults: causal.attemptResults,
      decision: causal.decision,
    })).toThrow("retry protocol replay mismatch");
    const beforeSessionStart = [events[1], events[0], ...events.slice(2)];
    expect(() => deriveLedgerProjection("session-fixed", beforeSessionStart)).toThrow(
      "session_started must be first",
    );
    expect(() => assertRetryProtocol({
      events: beforeSessionStart,
      attemptResults: causal.attemptResults,
      decision: causal.decision,
    })).toThrow("event before session_started");
  });

  it("renders terminal environment precision without a causal claim", () => {
    const value = structuredClone(causal);
    value.ledger.unexplainedStabilityInvalidCount = 2;
    const invalidBlocks = () => ({
      A0: block(9_000), B: block(9_100), A1: block(9_400),
      C: block(9_200, true), A2: block(9_000), D: block(9_300, true), A3: block(9_000),
    });
    value.ledger.attempts = [
      { attemptId: "attempt-001", status: "stability_invalid", startedAt: "08:00", endedAt: "09:00", reasons: ["anchor_range_exceeded"], resultPath: "attempt-001.json" },
      { attemptId: "attempt-002", status: "stability_invalid", startedAt: "09:10", endedAt: "10:10", reasons: ["anchor_range_exceeded"], resultPath: "attempt-002.json" },
    ];
    value.ledger.events = [
      { type: "retry_disposition", attemptId: "attempt-001", unexplainedStability: true, retryAction: "retry" },
      { type: "retry_disposition", attemptId: "attempt-002", unexplainedStability: true, retryAction: "environment_precision_insufficient" },
    ];
    value.decision = {
      schemaVersion: 1,
      classification: "environment_precision_insufficient",
      attemptId: null,
      unexplainedStabilityInvalidCount: 2,
      evaluation: null,
    };
    value.attemptResults = value.ledger.attempts.map((attempt) => ({
      attemptId: attempt.attemptId,
      kind: "stability_invalid",
      reasons: ["anchor_range_exceeded"],
      blocks: invalidBlocks(),
      evaluation: {
        kind: "stability_invalid",
        eRequired: false,
        anchorRangeMs: 400,
        reasons: ["anchor_range_exceeded"],
        summaries: Object.fromEntries(Object.entries(invalidBlocks()).map(([name, value]) => [name, value.summary])),
      },
    }));
    const report = renderVerification(value);
    expect(report).toContain("No B/C/D/E causal classification is made.");
    expect(report).toContain("**Unexplained stability-invalid count:** 2");
    expect(report).toContain("## Attempt attempt-001 raw measurements");
    expect(report).toContain("## Attempt attempt-002 raw measurements");
    expect(report).not.toContain("## Variant metrics");
  });

  it("is deterministic and uses only recorded timestamps", () => {
    expect(renderVerification(structuredClone(causal))).toBe(renderVerification(causal));
  });

  it("rejects a reporter-time protocol pin mismatch before publication", () => {
    const verified = {
      ...sessionManifest.protocol,
      protocolLock: structuredClone(sessionManifest.protocolLock),
    };
    expect(() => assertProtocolPinMatches(sessionManifest, verified)).not.toThrow();
    expect(() => assertProtocolPinMatches(sessionManifest, {
      ...verified,
      lockSha256: "9".repeat(64),
    })).toThrow("report protocol pin mismatch");
  });

  it("allows only its output and replays index-link and between-publication crashes", async () => {
    const outputRelative = "docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md";
    assertProtocolWorktreeStatus(`?? ${outputRelative}`, [outputRelative]);
    for (const status of [` M ${outputRelative}`, "?? unrelated.txt"]) {
      let thrown: unknown = null;
      try {
        assertProtocolWorktreeStatus(status, [outputRelative]);
      } catch (error) {
        thrown = error;
      }
      expect(thrown).toMatchObject({ kind: "protocol_worktree_dirty" });
    }

    let verifyInput: Record<string, unknown> | null = null;
    const verified = {
      ...sessionManifest.protocol,
      protocolLock: structuredClone(sessionManifest.protocolLock),
    };
    await verifyReportProtocol({
      sessionManifest,
      output: path.join(sessionManifest.protocolRoot, ...outputRelative.split("/")),
      runningProtocolRoot: sessionManifest.protocolRoot,
      verifyFn: async (input: Record<string, unknown>) => {
        verifyInput = input;
        return verified;
      },
    });
    expect(verifyInput).toEqual({
      repoRoot: sessionManifest.protocolRoot,
      allowedUntrackedPaths: [outputRelative],
    });
    let arbitraryOutputReachedGit = false;
    await expect(verifyReportProtocol({
      sessionManifest,
      output: path.join(sessionManifest.protocolRoot, "unrelated", "important.txt"),
      runningProtocolRoot: sessionManifest.protocolRoot,
      verifyFn: async () => {
        arbitraryOutputReachedGit = true;
        return verified;
      },
    })).rejects.toThrow("report output must equal");
    expect(arbitraryOutputReachedGit).toBe(false);

    const root = await mkdtemp(path.join(os.tmpdir(), "extractum-report-replay-"));
    const artifactIndex = {
      path: path.join(root, "artifact-index.json"),
      content: Buffer.from("index\n"),
    };
    const output = path.join(root, "verification.md");
    const reportBytes = Buffer.from("report\n");
    const staleTemp = `${output}.4242.123e4567-e89b-12d3-a456-426614174000.tmp`;
    const unrelated = `${output}.not-owned.tmp`;
    await writeFile(staleTemp, "partial", "utf8");
    await writeFile(unrelated, "keep", "utf8");
    await cleanupOwnedAtomicTemps(output, { processAliveFn: () => false });
    await expect(access(staleTemp)).rejects.toMatchObject({ code: "ENOENT" });
    await expect(access(unrelated)).resolves.toBeUndefined();

    // Simulate a hard kill after the atomic index temp was linked to its final
    // target but before the writer unlinked the sibling temp.
    const strandedIndexTemp = `${artifactIndex.path}.4242.123e4567-e89b-12d3-a456-426614174001.tmp`;
    await writeFile(strandedIndexTemp, artifactIndex.content);
    await link(strandedIndexTemp, artifactIndex.path);
    await cleanupOwnedAtomicTemps(artifactIndex.path, { processAliveFn: () => false });
    await expect(access(strandedIndexTemp)).rejects.toMatchObject({ code: "ENOENT" });
    await publishReportPair({ artifactIndex, output, reportBytes });
    expect(await readFile(artifactIndex.path)).toEqual(artifactIndex.content);
    expect(await readFile(output)).toEqual(reportBytes);

    const peerRoot = await mkdtemp(path.join(os.tmpdir(), "extractum-report-peer-replay-"));
    const peerArtifactIndex = {
      path: path.join(peerRoot, "artifact-index.json"),
      content: artifactIndex.content,
    };
    const peerOutput = path.join(peerRoot, "verification.md");
    let crashed = false;
    await expect(publishReportPair({
      artifactIndex: peerArtifactIndex,
      output: peerOutput,
      reportBytes,
    }, async (target: string, bytes: Buffer) => {
      await writeAtomicBytesExclusive(target, bytes);
      if (!crashed) {
        crashed = true;
        throw new Error("simulated publication crash");
      }
    })).rejects.toThrow("simulated publication crash");
    await publishReportPair({ artifactIndex: peerArtifactIndex, output: peerOutput, reportBytes });
    expect(await readFile(peerArtifactIndex.path)).toEqual(peerArtifactIndex.content);
    expect(await readFile(peerOutput)).toEqual(reportBytes);
  });
});
```

- [ ] **Step 2: Run report RED**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/report.test.ts
```

Expected: FAIL because `report.mjs` does not exist; eight tests must be
collected.

- [ ] **Step 3: Implement deterministic, self-reference-free protocol locking**

Create `scripts/process-shell-diagnostic/freeze.mjs`:

```js
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

import { D_BLOB_ANCHORS } from "./git-state.mjs";
import { PROTOCOL } from "./protocol.mjs";
import {
  ProtocolError,
  runWindowsProcess,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

export const LOCK_PATH = "scripts/process-shell-diagnostic/protocol-lock.json";
export const FROZEN_INPUTS = Object.freeze([
  "docs/superpowers/specs/2026-07-18-process-shell-regression-diagnostic-design.md",
  "docs/superpowers/plans/2026-07-18-process-shell-regression-diagnostic.md",
  "docs/value-registry.md",
  "scripts/process-shell-diagnostic/protocol.mjs",
  "scripts/process-shell-diagnostic/protocol.test.ts",
  "scripts/process-shell-diagnostic/runtime.mjs",
  "scripts/process-shell-diagnostic/runtime.test.ts",
  "scripts/process-shell-diagnostic/git-state.mjs",
  "scripts/process-shell-diagnostic/git-state.test.ts",
  "scripts/process-shell-diagnostic/attempt.mjs",
  "scripts/process-shell-diagnostic/attempt.test.ts",
  "scripts/process-shell-diagnostic/coordinator.mjs",
  "scripts/process-shell-diagnostic/coordinator.test.ts",
  "scripts/process-shell-diagnostic/freeze.mjs",
  "scripts/process-shell-diagnostic/report.mjs",
  "scripts/process-shell-diagnostic/report.test.ts",
  "scripts/process-shell-diagnostic/states/B.patch",
  "scripts/process-shell-diagnostic/states/C.patch",
  "scripts/process-shell-diagnostic/states/E.patch",
]);

const A_TREE = "fd9711a041432ef420e7b09d56a46131a2a52a2a";
const D_TREE = "77e2d163ccc8bddf3ea051cb995909888cae9aba";

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function taskkillExe() {
  if (!process.env.SystemRoot) throw new ProtocolError("missing_system_root", "SystemRoot is required");
  return path.join(process.env.SystemRoot, "System32", "taskkill.exe");
}

async function removeOwnedTemp(directory) {
  const resolved = path.resolve(directory);
  const temporaryRoot = `${path.resolve(tmpdir())}${path.sep}`.toLowerCase();
  if (!resolved.toLowerCase().startsWith(temporaryRoot)) {
    throw new ProtocolError("unsafe_temp_cleanup", resolved);
  }
  await rm(resolved, { recursive: true, force: true });
}

async function createGit(repoRoot) {
  const artifactDir = await mkdtemp(path.join(tmpdir(), "extractum-process-freeze-"));
  let sequence = 0;
  async function bytes(args, env = process.env) {
    sequence += 1;
    const result = await runWindowsProcess({
      label: `git-${String(sequence).padStart(3, "0")}`,
      command: "git.exe",
      args,
      cwd: repoRoot,
      env,
      artifactDir,
      timeoutMs: PROTOCOL.commandTimeoutMs,
      taskkillExe: taskkillExe(),
    });
    if (result.classification !== "ok") {
      throw new ProtocolError("freeze_git_failed", args.join(" "), { result });
    }
    return readFile(result.stdoutPath);
  }
  return {
    bytes,
    text: async (args, env) => (await bytes(args, env)).toString("utf8").trim(),
    close: async () => removeOwnedTemp(artifactDir),
  };
}

export function assertProtocolWorktreeStatus(status, allowedUntrackedPaths = []) {
  const allowed = new Set(allowedUntrackedPaths.map((value) => value.replaceAll("\\", "/")));
  const unexpected = status.split(/\r?\n/).filter(Boolean).filter((line) => {
    if (!line.startsWith("?? ")) return true;
    const candidate = line.slice(3).replaceAll("\\", "/");
    return !allowed.has(candidate);
  });
  if (unexpected.length) {
    throw new ProtocolError("protocol_worktree_dirty", unexpected.join("\n"));
  }
}

async function assertInputsCommitted(git, allowedUntrackedPaths = []) {
  const status = await git.text(["status", "--porcelain=v1", "--untracked-files=all"]);
  assertProtocolWorktreeStatus(status, allowedUntrackedPaths);
  const tracked = (await git.text(["ls-files", "--", "scripts/process-shell-diagnostic"]))
    .split(/\r?\n/).filter(Boolean).sort();
  const expected = FROZEN_INPUTS.filter((entry) => entry.startsWith("scripts/")).sort();
  const allowed = new Set([...expected, LOCK_PATH]);
  const missing = expected.filter((entry) => !tracked.includes(entry));
  const extra = tracked.filter((entry) => !allowed.has(entry));
  if (missing.length || extra.length) {
    throw new ProtocolError("protocol_input_inventory_mismatch", "frozen script inventory differs", { missing, extra });
  }
}

async function inputRecord(git, filePath) {
  const blob = await git.text(["rev-parse", `HEAD:${filePath}`]);
  const blobBytes = await git.bytes(["cat-file", "blob", blob]);
  return {
    path: filePath,
    size: blobBytes.length,
    sha256: sha256(blobBytes),
    gitBlob: blob,
  };
}

async function patchedState(repoRoot, git, name) {
  const indexRoot = await mkdtemp(path.join(tmpdir(), `extractum-process-${name}-index-`));
  const indexPath = path.join(indexRoot, "index");
  const environment = { ...process.env, GIT_INDEX_FILE: indexPath };
  const patch = `scripts/process-shell-diagnostic/states/${name}.patch`;
  try {
    const patchBlob = await git.text(["rev-parse", `HEAD:${patch}`]);
    const patchBytes = await git.bytes(["cat-file", "blob", patchBlob]);
    const canonicalPatch = path.join(indexRoot, `${name}.patch`);
    await writeFile(canonicalPatch, patchBytes, { flag: "wx" });
    await git.text(["read-tree", PROTOCOL.baselineCommit], environment);
    await git.text([
      "apply", "--cached", "--whitespace=nowarn", "--",
      canonicalPatch,
    ], environment);
    const rootTree = await git.text(["write-tree"], environment);
    const srcTauriTree = await git.text(["rev-parse", `${rootTree}:src-tauri`], environment);
    const changedPaths = (await git.text([
      "diff", "--cached", "--name-status", "--find-renames=50%",
      PROTOCOL.baselineCommit, "--", "src-tauri",
    ], environment)).split(/\r?\n/).filter(Boolean);
    return {
      source: patch,
      base: "A",
      patch,
      patchSha256: sha256(patchBytes),
      srcTauriTree,
      changedPaths,
    };
  } finally {
    await removeOwnedTemp(indexRoot);
  }
}

async function stateRecords(repoRoot, git) {
  const aTree = await git.text(["rev-parse", `${PROTOCOL.baselineCommit}:src-tauri`]);
  const dTree = await git.text(["rev-parse", `${PROTOCOL.candidateCommit}:src-tauri`]);
  const parentTree = await git.text(["rev-parse", `${PROTOCOL.candidateCommit}^:src-tauri`]);
  if (aTree !== A_TREE || dTree !== D_TREE || parentTree !== A_TREE) {
    throw new ProtocolError("historical_tree_mismatch", "A, D, or D parent differs", { aTree, dTree, parentTree });
  }
  const dChangedPaths = (await git.text([
    "diff", "--name-status", "--find-renames=50%",
    PROTOCOL.baselineCommit, PROTOCOL.candidateCommit, "--", "src-tauri",
  ])).split(/\r?\n/).filter(Boolean);
  const states = {
    A: { source: PROTOCOL.baselineCommit, srcTauriTree: aTree, changedPaths: [] },
    B: await patchedState(repoRoot, git, "B"),
    C: await patchedState(repoRoot, git, "C"),
    D: {
      source: PROTOCOL.candidateCommit,
      srcTauriTree: dTree,
      changedPaths: dChangedPaths,
      blobs: { ...D_BLOB_ANCHORS },
      absentPaths: [
        "src-tauri/src/child_process.rs",
        "src-tauri/src/external_process.rs",
        "src-tauri/src/process_tree.rs",
      ],
    },
    E: await patchedState(repoRoot, git, "E"),
  };
  if (new Set(Object.values(states).map((state) => state.srcTauriTree)).size !== 5) {
    throw new ProtocolError("state_tree_collision", "A/B/C/D/E must have five distinct trees", { states });
  }
  return states;
}

export async function buildProtocolLock({ repoRoot, allowedUntrackedPaths = [] }) {
  const git = await createGit(repoRoot);
  try {
    await assertInputsCommitted(git, allowedUntrackedPaths);
    const frozenInputs = [];
    for (const filePath of [...FROZEN_INPUTS].sort()) {
      frozenInputs.push(await inputRecord(git, filePath));
    }
    return {
      schemaVersion: 1,
      protocolVersion: PROTOCOL.version,
      baselineCommit: PROTOCOL.baselineCommit,
      candidateCommit: PROTOCOL.candidateCommit,
      generatedBy: "scripts/process-shell-diagnostic/freeze.mjs",
      frozenInputs,
      states: await stateRecords(repoRoot, git),
    };
  } finally {
    await git.close();
  }
}

export async function verifyFrozenProtocol({ repoRoot, allowedUntrackedPaths = [] }) {
  const actual = await buildProtocolLock({ repoRoot, allowedUntrackedPaths });
  const git = await createGit(repoRoot);
  try {
    const protocolCommit = await git.text(["rev-parse", "HEAD"]);
    const lockBlob = await git.text(["rev-parse", `${protocolCommit}:${LOCK_PATH}`]);
    const recordedBytes = await git.bytes(["cat-file", "blob", lockBlob]);
    const recorded = JSON.parse(recordedBytes.toString("utf8"));
    if (JSON.stringify(recorded) !== JSON.stringify(actual)) {
      throw new ProtocolError("protocol_lock_mismatch", "protocol-lock.json differs from canonical frozen Git blobs");
    }
    return {
      protocolCommit,
      lockPath: LOCK_PATH,
      lockBlob,
      lockSha256: sha256(recordedBytes),
      protocolVersion: recorded.protocolVersion,
      protocolLock: recorded,
    };
  } finally {
    await git.close();
  }
}

function option(name) {
  const index = process.argv.indexOf(name);
  if (index < 0 || !process.argv[index + 1]) throw new Error(`missing ${name}`);
  return process.argv[index + 1];
}

async function main() {
  const action = process.argv[2];
  const repoRoot = path.resolve(option("--repo-root"));
  if (action === "generate") {
    await writeAtomicJsonExclusive(
      path.join(repoRoot, ...LOCK_PATH.split("/")),
      await buildProtocolLock({ repoRoot }),
    );
    process.stdout.write(`${LOCK_PATH}\n`);
    return;
  }
  if (action === "verify") {
    const { protocolLock: _protocolLock, ...pin } = await verifyFrozenProtocol({ repoRoot });
    process.stdout.write(`${JSON.stringify(pin, null, 2)}\n`);
    return;
  }
  throw new Error(`expected generate or verify, got ${action ?? "missing"}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  await main();
}
```

All hashes above are of canonical committed Git blob bytes. The verifier never
compares checkout bytes, and `patchedState` applies a temporary file recreated
from the committed patch blob. This is deliberate: a fresh Windows worktree
may materialize CRLF through `core.autocrlf`, while the frozen protocol identity
must remain invariant across checkouts.

`protocol-lock.json` is intentionally absent from `FROZEN_INPUTS`. The
generated lock therefore has no self-reference; its containing Git commit and
blob are pinned later by `verifyFrozenProtocol` and the session manifest.

- [ ] **Step 4: Implement independent recalculation, artifact indexing, and report rendering**

Create `scripts/process-shell-diagnostic/report.mjs`:

```js
import { createHash } from "node:crypto";
import { lstat, readFile, readdir, unlink } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";

import { snapshotDirectory } from "./coordinator.mjs";
import { verifyFrozenProtocol } from "./freeze.mjs";
import { sha256File, writeAtomicBytesExclusive } from "./runtime.mjs";

const ORDER = ["A0", "B", "A1", "C", "A2", "D", "A3", "E", "A4"];
export const REPORT_PATH = "docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md";
const RUNNING_PROTOCOL_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const INTERPRETATION = {
  membership_configuration: "The membership-only configuration is sufficient to reproduce the material tax; threshold policy needs an explicit owner decision.",
  edge_related_configuration: "The first crossing occurs after the app edge; that edge or its interaction with membership is implicated.",
  manifest_related: "E reproduces D, implicating the manifest migration, feature unification, declarations, or their interaction with C.",
  boundary_composite: "D reproduces the effect while E does not; the remaining concrete boundary/facade/ownership composite is implicated.",
  not_reproduced: "The original regression is not reproduced; a separately preregistered direct A/D confirmation is required.",
  threshold_disagreement: "The absolute diagnostic rule and existing dual shell cap disagree; this is an anomaly with no roadmap decision.",
};
const REQUIRED_NEXT_STEP = {
  membership_configuration: "Keep Phase 4 blocked; the owner must explicitly retain, replace, or waive the shell-threshold framework before roadmap work resumes.",
  edge_related_configuration: "Keep Phase 4 blocked; require an explicit owner decision on how the shell cap handles the likely one-time edge-related tax.",
  manifest_related: "Redesign the manifest migration and run a new preregistered confirmation before reconsidering Phase 3.",
  boundary_composite: "Redesign the concrete boundary and seek separate approval for a new Phase 3 attempt; Phase 4 remains blocked.",
  not_reproduced: "Run a new preregistered direct A/D A/B confirmation before changing Phase 3's recorded outcome.",
  threshold_disagreement: "Document the anomaly and preregister any follow-up; do not adapt thresholds or change the roadmap from this result.",
};

export function assertProtocolPinMatches(sessionManifest, verified) {
  const { protocolLock, ...pin } = verified;
  const pinKeys = ["protocolCommit", "lockPath", "lockBlob", "lockSha256", "protocolVersion"];
  if (pinKeys.some((key) => pin[key] !== sessionManifest.protocol[key])
    || JSON.stringify(protocolLock) !== JSON.stringify(sessionManifest.protocolLock)) {
    throw new Error("report protocol pin mismatch");
  }
}

function sameAbsolutePath(left, right) {
  const normalizedLeft = path.normalize(path.resolve(left));
  const normalizedRight = path.normalize(path.resolve(right));
  return process.platform === "win32"
    ? normalizedLeft.toLowerCase() === normalizedRight.toLowerCase()
    : normalizedLeft === normalizedRight;
}

export function assertFixedReportOutput({
  sessionManifest,
  output,
  runningProtocolRoot = RUNNING_PROTOCOL_ROOT,
}) {
  const protocolRoot = path.resolve(sessionManifest.protocolRoot);
  if (!sameAbsolutePath(protocolRoot, runningProtocolRoot)) {
    throw new Error("session protocol root differs from the running frozen reporter");
  }
  const expectedOutput = path.resolve(protocolRoot, ...REPORT_PATH.split("/"));
  if (!sameAbsolutePath(output, expectedOutput)) {
    throw new Error(`report output must equal ${REPORT_PATH}`);
  }
  return { protocolRoot, expectedOutput };
}

export async function verifyReportProtocol({
  sessionManifest,
  output,
  runningProtocolRoot = RUNNING_PROTOCOL_ROOT,
  verifyFn = verifyFrozenProtocol,
}) {
  const { protocolRoot } = assertFixedReportOutput({ sessionManifest, output, runningProtocolRoot });
  const verified = await verifyFn({
    repoRoot: protocolRoot,
    allowedUntrackedPaths: [REPORT_PATH],
  });
  assertProtocolPinMatches(sessionManifest, verified);
}

function regexEscape(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function pidAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}

export async function cleanupOwnedAtomicTemps(target, injected = {}) {
  const aliveFn = injected.processAliveFn ?? pidAlive;
  const parent = path.dirname(target);
  const basename = path.basename(target);
  const pattern = new RegExp(
    `^${regexEscape(basename)}\\.(\\d+)\\.[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\\.tmp$`,
    "i",
  );
  for (const entry of await readdir(parent, { withFileTypes: true })) {
    const match = entry.name.match(pattern);
    if (!match) continue;
    const candidate = path.join(parent, entry.name);
    const stat = await lstat(candidate);
    if (!stat.isFile() || stat.isSymbolicLink()) {
      throw new Error(`refusing non-regular report temp cleanup: ${candidate}`);
    }
    if (aliveFn(Number(match[1]))) {
      throw new Error(`report publisher is still alive for temp: ${candidate}`);
    }
    await unlink(candidate);
  }
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

function summarize(samplesMs) {
  const medianMs = median(samplesMs);
  const samplesWithinBand = samplesMs.filter((value) => Math.abs(value - medianMs) <= 300).length;
  return { samplesMs: [...samplesMs], medianMs, samplesWithinBand, stable: samplesWithinBand >= 5 };
}

function metric(variant, left, right) {
  const aReferenceMs = (left.medianMs + right.medianMs) / 2;
  const deltaMs = variant.medianMs - aReferenceMs;
  const percentDelta = 100 * deltaMs / aReferenceMs;
  return {
    variantMedianMs: variant.medianMs,
    aReferenceMs,
    deltaMs,
    percentDelta,
    material: deltaMs >= 500,
    shellCapFailed: deltaMs > 500 || percentDelta > 5,
  };
}

function samplesFromRawAttempt(attemptResult) {
  return Object.fromEntries(Object.entries(attemptResult.blocks ?? {}).map(([name, block]) => {
    if (!Array.isArray(block.samples) || block.samples.length !== 7) {
      throw new Error(`raw attempt evidence mismatch: ${attemptResult.attemptId}/${name} does not have seven samples`);
    }
    const values = block.samples.map((sample) => sample.wallMs);
    if (values.some((value) => !Number.isFinite(value))) {
      throw new Error(`raw attempt evidence mismatch: ${attemptResult.attemptId}/${name} has a non-numeric wall sample`);
    }
    return [name, values];
  }));
}

function independentEvaluation(attemptResult) {
  const raw = samplesFromRawAttempt(attemptResult);
  for (const name of ["A0", "B", "A1", "C", "A2", "D", "A3"]) {
    if (!raw[name]) throw new Error(`raw attempt evidence mismatch: missing base block ${attemptResult.attemptId}/${name}`);
  }
  const hasE = Boolean(raw.E || raw.A4);
  if (Boolean(raw.E) !== Boolean(raw.A4)) {
    throw new Error("raw attempt evidence mismatch: conditional E/A4 pair");
  }
  const summaries = Object.fromEntries(Object.entries(raw).map(([name, values]) => [name, summarize(values)]));
  let anchorNames = ["A0", "A1", "A2", "A3"];
  const anchorMedians = anchorNames.map((name) => summaries[name].medianMs);
  let anchorRangeMs = Math.max(...anchorMedians) - Math.min(...anchorMedians);
  let reasons = Object.entries(summaries)
    .filter(([, value]) => !value.stable)
    .map(([name]) => `block_unstable:${name}`);
  if (anchorRangeMs > 300) reasons.push("anchor_range_exceeded");
  if (reasons.length) {
    if (hasE) throw new Error("raw attempt evidence mismatch: E/A4 ran after base stability failure");
    return { kind: "stability_invalid", eRequired: false, anchorRangeMs, reasons, summaries };
  }
  const metrics = {
    B: metric(summaries.B, summaries.A0, summaries.A1),
    C: metric(summaries.C, summaries.A1, summaries.A2),
    D: metric(summaries.D, summaries.A2, summaries.A3),
  };
  if (summaries.E) metrics.E = metric(summaries.E, summaries.A3, summaries.A4);
  const eRequired = !metrics.B.material && !metrics.C.material && metrics.D.material;
  if (hasE !== eRequired) {
    throw new Error("raw attempt evidence mismatch: conditional E/A4 contract");
  }
  if (hasE) {
    anchorNames = [...anchorNames, "A4"];
    anchorRangeMs = Math.max(...anchorNames.map((name) => summaries[name].medianMs))
      - Math.min(...anchorNames.map((name) => summaries[name].medianMs));
    reasons = Object.entries(summaries)
      .filter(([, value]) => !value.stable)
      .map(([name]) => `block_unstable:${name}`);
    if (anchorRangeMs > 300) reasons.push("anchor_range_exceeded");
    if (reasons.length) {
      return { kind: "stability_invalid", eRequired: true, anchorRangeMs, reasons, summaries };
    }
  }
  const disagreement = Object.entries(metrics)
    .filter(([, value]) => value.material !== value.shellCapFailed)
    .map(([name]) => name);
  let classification;
  if (disagreement.length) classification = "threshold_disagreement";
  else if (metrics.B.material) classification = "membership_configuration";
  else if (metrics.C.material) classification = "edge_related_configuration";
  else if (metrics.D.material && metrics.E?.material) classification = "manifest_related";
  else if (metrics.D.material) classification = "boundary_composite";
  else classification = "not_reproduced";
  return {
    kind: "valid",
    classification,
    eRequired,
    anchorRangeMs,
    disagreement,
    summaries,
    metrics,
    contrasts: {
      membershipMs: metrics.B.deltaMs,
      edgeAfterMembershipMs: metrics.C.deltaMs - metrics.B.deltaMs,
      manifestAfterCMs: metrics.E ? metrics.E.deltaMs - metrics.C.deltaMs : null,
      dSpecificCompositeMs: metrics.E ? metrics.D.deltaMs - metrics.E.deltaMs : null,
      dAfterCCompositeMs: metrics.E ? null : metrics.D.deltaMs - metrics.C.deltaMs,
    },
  };
}

function evaluationProjection(value) {
  return {
    kind: value.kind,
    classification: value.classification,
    eRequired: value.eRequired,
    anchorRangeMs: value.anchorRangeMs,
    reasons: value.reasons ?? [],
    disagreement: value.disagreement ?? [],
    summaries: Object.fromEntries(Object.entries(value.summaries).map(([name, summary]) => [name, {
      samplesMs: summary.samplesMs,
      medianMs: summary.medianMs,
      samplesWithinBand: summary.samplesWithinBand,
      stable: summary.stable,
    }])),
    metrics: value.metrics ?? null,
    contrasts: value.contrasts ?? null,
  };
}

function sameStrings(left, right) {
  return JSON.stringify([...(left ?? [])].sort()) === JSON.stringify([...(right ?? [])].sort());
}

export function deriveLedgerProjection(sessionId, events) {
  const starts = new Map();
  const finished = new Set();
  const attempts = [];
  let terminal = null;
  let sessionStarted = false;
  for (const [index, event] of events.entries()) {
    if (terminal) throw new Error("ledger projection mismatch: event follows session_completed");
    if (event.type === "session_started") {
      if (index !== 0 || sessionStarted || event.sessionId !== sessionId) {
        throw new Error("ledger projection mismatch: session_started identity/count");
      }
      sessionStarted = true;
    } else if (!sessionStarted) {
      throw new Error("ledger projection mismatch: session_started must be first");
    } else if (event.type === "attempt_started") {
      if (starts.has(event.attemptId)) throw new Error("ledger projection mismatch: duplicate attempt_started");
      starts.set(event.attemptId, event);
    } else if (event.type === "attempt_finished") {
      if (!starts.has(event.attemptId) || finished.has(event.attemptId)) {
        throw new Error("ledger projection mismatch: orphan/duplicate attempt_finished");
      }
      finished.add(event.attemptId);
      attempts.push({
        attemptId: event.attemptId,
        status: event.kind,
        startedAt: event.startedAt,
        endedAt: event.endedAt,
        reasons: event.reasons,
        worktree: event.worktree,
        targetDirectory: event.targetDirectory,
        environment: event.environment,
        resultPath: event.resultPath,
      });
    } else if (event.type === "session_completed") {
      terminal = event;
    }
  }
  if (!sessionStarted || !terminal || starts.size !== finished.size) {
    throw new Error("ledger projection mismatch: session is not exactly terminal and complete");
  }
  return {
    schemaVersion: 1,
    sessionId,
    unexplainedStabilityInvalidCount: terminal.retryState.unexplainedStabilityInvalidCount,
    terminal: true,
    attempts,
    events,
  };
}

function replayMismatch(message, details = null) {
  throw new Error(`retry protocol replay mismatch: ${message}${details ? ` ${JSON.stringify(details)}` : ""}`);
}

export function assertRetryProtocol({ events, attemptResults, decision }) {
  const results = new Map(attemptResults.map((result) => [result.attemptId, result]));
  if (results.size !== attemptResults.length) replayMismatch("duplicate result id");
  let state = { unexplainedStabilityInvalidCount: 0, terminal: false };
  let activeAttempt = null;
  let pendingResult = null;
  let allowNextAttempt = true;
  let attemptsStarted = 0;
  let terminalEvent = null;
  let sessionStarted = false;
  for (const [index, event] of events.entries()) {
    if (terminalEvent) replayMismatch("event after terminal", event);
    if (event.type === "session_started") {
      if (index !== 0 || sessionStarted) replayMismatch("session_started must occur exactly once and first", event);
      sessionStarted = true;
      if (!isDeepStrictEqual(event.retryState, state)) replayMismatch("initial retry state", event);
    } else if (!sessionStarted) {
      replayMismatch("event before session_started", event);
    } else if (event.type === "attempt_started") {
      if (activeAttempt || pendingResult || state.terminal || !allowNextAttempt) {
        replayMismatch("illegal attempt start", event);
      }
      attemptsStarted += 1;
      allowNextAttempt = false;
      if (!isDeepStrictEqual(event.retryState, state)) replayMismatch("attempt retry state", event);
      activeAttempt = event.attemptId;
    } else if (event.type === "attempt_finished") {
      if (activeAttempt !== event.attemptId || pendingResult) replayMismatch("attempt finish ordering", event);
      const result = results.get(event.attemptId);
      if (!result || result.kind !== event.kind) replayMismatch("attempt result identity/kind", event);
      activeAttempt = null;
      pendingResult = result;
    } else if (event.type === "retry_disposition") {
      if (!pendingResult || pendingResult.attemptId !== event.attemptId || pendingResult.kind === "valid") {
        replayMismatch("disposition without matching invalid attempt", event);
      }
      const correctedCause = typeof event.correctedCause === "string" ? event.correctedCause.trim() : "";
      const unexplained = event.unexplainedStability === true;
      if (correctedCause && unexplained) replayMismatch("ambiguous disposition", event);
      let action;
      if (pendingResult.kind === "infrastructure_invalid") {
        if (!correctedCause || unexplained) replayMismatch("infrastructure retry lacks corrected cause", event);
        action = "retry";
      } else if (pendingResult.kind === "stability_invalid") {
        if (correctedCause) action = "retry";
        else if (unexplained) {
          state = {
            unexplainedStabilityInvalidCount: state.unexplainedStabilityInvalidCount + 1,
            terminal: false,
          };
          action = state.unexplainedStabilityInvalidCount >= 2
            ? "environment_precision_insufficient"
            : "retry";
        } else replayMismatch("stability disposition missing cause/unexplained choice", event);
      } else replayMismatch("unknown invalidation kind", pendingResult);
      if (action === "environment_precision_insufficient") state = { ...state, terminal: true };
      const expectedState = { ...state };
      if (event.invalidationKind !== pendingResult.kind
        || event.retryAction !== action
        || !isDeepStrictEqual(event.retryState, expectedState)) {
        replayMismatch("disposition action/state", { event, action, expectedState });
      }
      pendingResult = null;
      allowNextAttempt = action === "retry";
    } else if (event.type === "session_completed") {
      if (activeAttempt) replayMismatch("terminal with active attempt", event);
      if (pendingResult?.kind === "valid") {
        state = { ...state, terminal: true };
        if (event.classification !== pendingResult.evaluation?.classification) {
          replayMismatch("valid terminal classification", event);
        }
        pendingResult = null;
      } else if (!(state.terminal && event.classification === "environment_precision_insufficient" && !pendingResult)) {
        replayMismatch("terminal without valid/precision outcome", event);
      }
      if (!isDeepStrictEqual(event.retryState, state) || !isDeepStrictEqual(event.decision, decision)) {
        replayMismatch("terminal state/decision", event);
      }
      terminalEvent = event;
    }
  }
  if (!sessionStarted || !terminalEvent || activeAttempt || pendingResult || attemptsStarted !== results.size) {
    replayMismatch("incomplete replay", { attemptsStarted, results: results.size });
  }
}

function assertIndependentEvidence({ ledger, decision, attemptResults }) {
  const results = new Map();
  for (const result of attemptResults) {
    if (!result?.attemptId || results.has(result.attemptId)) {
      throw new Error("raw attempt evidence mismatch: missing or duplicate attempt id");
    }
    results.set(result.attemptId, result);
  }
  if (results.size !== ledger.attempts.length) {
    throw new Error("raw attempt evidence mismatch: every ledger attempt needs exactly one immutable result");
  }
  const calculations = new Map();
  for (const row of ledger.attempts) {
    const result = results.get(row.attemptId);
    if (!result || result.kind !== row.status || !sameStrings(result.reasons, row.reasons)) {
      throw new Error(`raw attempt evidence mismatch: ledger/result projection for ${row.attemptId}`);
    }
    if (["valid", "stability_invalid"].includes(result.kind)) {
      const calculation = independentEvaluation(result);
      calculations.set(row.attemptId, calculation);
      if (calculation.kind !== result.kind) {
        throw new Error(`raw attempt evidence mismatch: recalculated kind for ${row.attemptId}`);
      }
      if (JSON.stringify(evaluationProjection(result.evaluation)) !== JSON.stringify(evaluationProjection(calculation))) {
        throw new Error(`raw attempt evidence mismatch: recalculated evaluation for ${row.attemptId}`);
      }
    } else if (result.evaluation != null) {
      throw new Error(`raw attempt evidence mismatch: infrastructure result carries an evaluation for ${row.attemptId}`);
    }
  }

  if (decision.classification === "environment_precision_insufficient") {
    if (decision.attemptId !== null || decision.evaluation !== null) {
      throw new Error("independent recalculation mismatch: precision outcome carries a causal decision");
    }
    const consumed = (ledger.events ?? []).filter((event) =>
      event.type === "retry_disposition" && event.unexplainedStability === true,
    );
    if (
      decision.unexplainedStabilityInvalidCount !== 2
      || ledger.unexplainedStabilityInvalidCount !== 2
      || consumed.length !== 2
      || new Set(consumed.map((event) => event.attemptId)).size !== 2
      || consumed.some((event) => calculations.get(event.attemptId)?.kind !== "stability_invalid")
    ) throw new Error("independent recalculation mismatch: environment precision evidence");
    return calculations;
  }

  const terminal = results.get(decision.attemptId);
  const recalculated = calculations.get(decision.attemptId);
  if (!terminal || terminal.kind !== "valid" || !recalculated) {
    throw new Error("independent recalculation mismatch: terminal valid attempt");
  }
  if (JSON.stringify(evaluationProjection(decision.evaluation)) !== JSON.stringify(evaluationProjection(recalculated))) {
    throw new Error("independent recalculation mismatch: recorded decision evaluation");
  }
  if (decision.classification !== recalculated.classification) {
    throw new Error("independent recalculation mismatch: terminal classification");
  }
  return calculations;
}

function safeCell(value) {
  return String(value ?? "n/a").replaceAll("|", "\\|").replaceAll("\r", " ").replaceAll("\n", " ");
}

function environmentCell(name, value) {
  if (name === "mainTargetSnapshot") {
    return safeCell(JSON.stringify({ exists: value.exists, digest: value.digest, records: value.records.length }));
  }
  return safeCell(typeof value === "object" ? JSON.stringify(value) : value);
}

function signedMs(value) {
  return value == null ? "n/a" : `${value >= 0 ? "+" : ""}${value} ms`;
}

function attemptRows(ledger) {
  return ledger.attempts.map((attempt) =>
    `| ${attempt.attemptId} | ${attempt.status} | ${safeCell(attempt.reasons?.join(", ") || "none")} | ${safeCell(attempt.startedAt)} | ${safeCell(attempt.endedAt)} |`,
  ).join("\n");
}

function attemptEnvironmentRows(ledger) {
  return ledger.attempts.map((attempt) =>
    `| ${attempt.attemptId} | ${safeCell(attempt.environment?.host)} | ${safeCell(attempt.environment?.power)} | ${safeCell(attempt.environment?.defender)} | ${safeCell(attempt.targetDirectory)} |`,
  ).join("\n");
}

function retryDispositionRows(ledger) {
  const rows = (ledger.events ?? []).filter((event) => event.type === "retry_disposition").map((event) =>
    `| ${event.attemptId} | ${event.invalidationKind} | ${event.unexplainedStability === true} | ${safeCell(event.correctedCause || "none")} | ${event.retryAction} | ${event.retryState?.unexplainedStabilityInvalidCount ?? "n/a"} |`,
  );
  return rows.length ? rows.join("\n") : "| none | none | false | none | none | 0 |";
}

function attemptErrorRows(attemptResults) {
  return attemptResults.map((attempt) =>
    `| ${attempt.attemptId} | ${attempt.kind} | ${safeCell(attempt.error?.kind || "none")} | ${safeCell(attempt.error?.category || "none")} | ${safeCell(attempt.error?.message || "none")} |`,
  ).join("\n");
}

function correctedEnvironmentRows(ledger) {
  const rows = (ledger.events ?? []).filter((event) =>
    event.type === "attempt_environment" && event.correctedEnvironmentDelta,
  ).map((event) =>
    `| ${event.attemptId} | ${safeCell(event.correctedEnvironmentDelta.correctedCause)} | ${safeCell(JSON.stringify(event.correctedEnvironmentDelta.before))} | ${safeCell(JSON.stringify(event.correctedEnvironmentDelta.after))} |`,
  );
  return rows.length ? rows.join("\n") : "| none | none | none | none |";
}

function elapsedSummary(ledger) {
  const starts = ledger.attempts.map((attempt) => Date.parse(attempt.startedAt)).filter(Number.isFinite);
  const ends = ledger.attempts.map((attempt) => Date.parse(attempt.endedAt)).filter(Number.isFinite);
  if (!starts.length || !ends.length) return "unavailable from recorded timestamps";
  return `${((Math.max(...ends) - Math.min(...starts)) / 60_000).toFixed(1)} minutes`;
}

function blockRows(evaluation, attemptResult) {
  return ORDER.filter((name) => evaluation.summaries[name]).map((name) => {
    const summary = evaluation.summaries[name];
    const block = attemptResult?.blocks?.[name];
    return `| ${name} | ${summary.samplesMs.join(", ")} | ${summary.medianMs} ms | ${summary.samplesWithinBand}/7 | ${block?.noOp?.elapsedMs ?? "n/a"} ms | ${block?.noOp?.cargoReportedMs ?? "n/a"} ms |`;
  }).join("\n");
}

function cargoRows(attemptResult) {
  return ORDER.filter((name) => attemptResult?.blocks?.[name]).map((name) => {
    const block = attemptResult.blocks[name];
    return `| ${name} | ${block.samples.map((sample) => sample.cargoReportedMs).join(", ")} | ${block.diagnostic?.extractumProcessExtern === true} | ${safeCell(block.inventory?.featureTreePath)} | ${safeCell(block.diagnostic?.timingArtifact?.path)} | ${safeCell(block.diagnostic?.timingArtifact?.sha256)} |`;
  }).join("\n");
}

function stateRows(attemptResult) {
  return ORDER.filter((name) => attemptResult?.blocks?.[name]).map((name) => {
    const block = attemptResult.blocks[name];
    return `| ${name} | ${safeCell(block.stateEvidence?.srcTauriTree)} | ${safeCell(block.stateEvidence?.canonicalLibSha256)} | ${block.inventory?.extractumProcessDirectDependency === true} | ${safeCell(block.inventory?.metadata?.target_directory)} |`;
  }).join("\n");
}

function rawBlockRows(attemptResult, calculation) {
  return ORDER.filter((name) => attemptResult?.blocks?.[name]).map((name) => {
    const block = attemptResult.blocks[name];
    const summary = calculation?.summaries?.[name] ?? null;
    const wall = block.samples?.map((sample) => sample.wallMs) ?? [];
    return `| ${name} | ${wall.join(", ") || "none"} | ${summary ? `${summary.medianMs} ms` : "n/a"} | ${summary ? `${summary.samplesWithinBand}/7` : "n/a"} | ${block.noOp?.elapsedMs ?? "n/a"} ms | ${block.noOp?.cargoReportedMs ?? "n/a"} ms |`;
  }).join("\n");
}

function attemptEvidenceSections(attemptResults, calculations) {
  const lines = [];
  for (const attempt of attemptResults) {
    const calculation = calculations.get(attempt.attemptId) ?? null;
    lines.push(
      `## Attempt ${attempt.attemptId} raw measurements`, "",
      `**Recorded kind:** \`${attempt.kind}\``, "",
      `**Recalculated stability reasons:** ${safeCell(calculation?.reasons?.join(", ") || "none / infrastructure invalidation")}`, "",
      "| Block | Wall samples | Median | Within 300 ms | No-op wall | No-op Cargo |",
      "| --- | --- | ---: | ---: | ---: | ---: |",
      rawBlockRows(attempt, calculation), "",
      "### State evidence", "",
      "| Block | src-tauri tree | Canonical lib.rs SHA-256 | Metadata direct edge | Cargo target |",
      "| --- | --- | --- | --- | --- |",
      stateRows(attempt), "",
      "### Cargo-reported samples and diagnostics", "",
      "| Block | Cargo durations (ms) | `--extern extractum_process` | Feature graph | Timings HTML | SHA-256 |",
      "| --- | --- | --- | --- | --- | --- |",
      cargoRows(attempt), "",
    );
  }
  return lines;
}

function metricRows(metrics) {
  return ["B", "C", "D", "E"].filter((name) => metrics[name]).map((name) => {
    const value = metrics[name];
    return `| ${name} | ${value.variantMedianMs} ms | ${value.aReferenceMs} ms | ${signedMs(value.deltaMs)} | ${value.percentDelta.toFixed(3)}% | ${value.material} | ${value.shellCapFailed} |`;
  }).join("\n");
}

function contrastRows(contrasts) {
  return [
    ["Membership", contrasts.membershipMs],
    ["Edge after membership", contrasts.edgeAfterMembershipMs],
    ["Manifest after C", contrasts.manifestAfterCMs],
    ["D-specific composite", contrasts.dSpecificCompositeMs],
    ["D after C composite", contrasts.dAfterCCompositeMs],
  ].filter(([, value]) => value != null).map(([name, value]) => `| ${name} | ${signedMs(value)} |`).join("\n");
}

export function renderVerification({
  sessionManifest,
  measurementEnvironment,
  ledger,
  decision,
  attemptResults,
  artifactIndex,
}) {
  const calculations = assertIndependentEvidence({ ledger, decision, attemptResults });
  const lines = [
    "# Process Shell Regression Diagnostic Verification",
    "",
    `**Session:** \`${sessionManifest.sessionId}\``,
    `**Outcome:** \`${decision.classification}\``,
    `**Protocol commit:** \`${sessionManifest.protocol.protocolCommit}\``,
    `**Protocol-lock blob:** \`${sessionManifest.protocol.lockBlob}\``,
    `**Protocol-lock SHA-256:** \`${sessionManifest.protocol.lockSha256}\``,
    `**Raw artifact directory:** \`${sessionManifest.sessionDir}\``,
    `**Artifact-index SHA-256:** \`${artifactIndex.sha256}\` (${artifactIndex.files} files, ${artifactIndex.bytes} bytes)`,
    `**Recorded attempt span:** ${elapsedSummary(ledger)}`,
    "",
    "## Environment",
    "",
    "| Field | Value |",
    "| --- | --- |",
    ...Object.entries(measurementEnvironment).map(([name, value]) => `| ${name} | ${environmentCell(name, value)} |`),
    "",
    "## Attempt ledger",
    "",
    "| Attempt | Status | Reasons | Started | Ended |",
    "| --- | --- | --- | --- | --- |",
    attemptRows(ledger),
    "",
    "## Attempt environments", "",
    "| Attempt | Host | Power | Defender | Target |",
    "| --- | --- | --- | --- | --- |",
    attemptEnvironmentRows(ledger), "",
    "## Retry and invalidation audit", "",
    "| Attempt | Invalidation | Unexplained stability | Corrected cause | Action | Count |",
    "| --- | --- | --- | --- | --- | ---: |",
    retryDispositionRows(ledger), "",
    "### Attempt error details", "",
    "| Attempt | Kind | Error kind | Category | Message |",
    "| --- | --- | --- | --- | --- |",
    attemptErrorRows(attemptResults), "",
    "### Corrected environment deltas", "",
    "| Attempt | Corrected cause | Before | After |",
    "| --- | --- | --- | --- |",
    correctedEnvironmentRows(ledger), "",
    ...attemptEvidenceSections(attemptResults, calculations),
  ];
  if (decision.classification === "environment_precision_insufficient") {
    lines.push(
      "## Decision", "",
      "No B/C/D/E causal classification is made.", "",
      `**Unexplained stability-invalid count:** ${decision.unexplainedStabilityInvalidCount}`, "",
      "The machine did not support the preregistered 300 ms precision twice. Any next run requires a separately frozen anomaly protocol.", "",
      "**Required next step:** Keep Phase 4 blocked and preregister a new design with sample count, interleaving/counterbalancing, and stability rule fixed before new data.", "",
    );
  } else {
    const evaluation = calculations.get(decision.attemptId);
    lines.push(
      `**A-anchor range:** ${evaluation.anchorRangeMs} ms`,
      `**Conditional E required:** ${evaluation.eRequired}`, "",
      "## Variant metrics", "",
      "| Variant | Median | Local A reference | Delta | Delta % | Material | Shell cap failed |",
      "| --- | ---: | ---: | ---: | ---: | --- | --- |",
      metricRows(evaluation.metrics), "",
      "## Descriptive contrasts", "",
      "| Contrast | Value |", "| --- | ---: |",
      contrastRows(evaluation.contrasts), "",
      "These are descriptive contrasts between cumulative configurations; they are not independently randomized component estimates.", "",
      "## Decision", "", INTERPRETATION[decision.classification], "",
      `**Required next step:** ${REQUIRED_NEXT_STEP[decision.classification]}`, "",
    );
  }
  lines.push(
    "## Scope", "",
    "This diagnostic does not automatically retain `extractum-process` or unblock Phase 4. Any roadmap, threshold, or architecture change remains a separate owner-approved decision.", "",
    "The result is conditional on the fixed incremental-cache order. Evidence of order-specific hysteresis requires a separately preregistered counterbalanced experiment, not a post-hoc rerun.", "",
  );
  return `${lines.join("\n")}\n`;
}

async function walkArtifacts(root, current = root) {
  const records = [];
  for (const entry of await readdir(current, { withFileTypes: true })) {
    const absolute = path.join(current, entry.name);
    const relative = path.relative(root, absolute).replaceAll("\\", "/");
    if (relative === "artifact-index.json") continue;
    if (relative === "worktrees" && entry.isDirectory()) continue;
    if (entry.isSymbolicLink()) throw new Error(`artifact symlink is forbidden: ${relative}`);
    if (entry.isDirectory()) records.push(...await walkArtifacts(root, absolute));
    else if (entry.isFile()) {
      const bytes = await readFile(absolute);
      records.push({ path: relative, bytes: bytes.length, sha256: await sha256File(absolute) });
    }
  }
  return records.sort((left, right) => left.path.localeCompare(right.path));
}

function sha256Bytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

export async function prepareArtifactIndex(sessionDir, locatorPath) {
  const records = await walkArtifacts(sessionDir);
  const locatorBytes = await readFile(locatorPath);
  records.push({
    path: "@external/session-locator.json",
    source: locatorPath,
    bytes: locatorBytes.length,
    sha256: sha256Bytes(locatorBytes),
  });
  records.sort((left, right) => left.path.localeCompare(right.path));
  const target = path.join(sessionDir, "artifact-index.json");
  const bytes = Buffer.from(`${JSON.stringify({ schemaVersion: 1, sessionDir, records }, null, 2)}\n`, "utf8");
  return {
    path: target,
    content: bytes,
    sha256: sha256Bytes(bytes),
    files: records.length,
    bytes: records.reduce((sum, record) => sum + record.bytes, 0),
  };
}

async function publishBytesIdempotent(target, bytes, writeFn = writeAtomicBytesExclusive) {
  try {
    await writeFn(target, bytes);
    return;
  } catch (error) {
    if (error.kind !== "duplicate_artifact") throw error;
  }
  const existing = await readFile(target);
  if (!existing.equals(bytes)) throw new Error(`immutable publication conflict: ${target}`);
}

export async function publishReportPair(
  { artifactIndex, output, reportBytes },
  writeFn = writeAtomicBytesExclusive,
) {
  await publishBytesIdempotent(artifactIndex.path, artifactIndex.content, writeFn);
  await publishBytesIdempotent(output, reportBytes, writeFn);
}

async function readNumberedLedger(sessionDir) {
  const directory = path.join(sessionDir, "ledger");
  const names = (await readdir(directory)).filter((name) => /^\d{6}\.json$/.test(name)).sort();
  const events = [];
  for (let index = 0; index < names.length; index += 1) {
    const expected = `${String(index + 1).padStart(6, "0")}.json`;
    if (names[index] !== expected) throw new Error(`numbered ledger gap: ${expected} != ${names[index]}`);
    const event = JSON.parse(await readFile(path.join(directory, names[index]), "utf8"));
    if (event.sequence !== index + 1) throw new Error(`numbered ledger sequence mismatch: ${names[index]}`);
    events.push(event);
  }
  return events;
}

function option(name) {
  const index = process.argv.indexOf(name);
  if (index < 0 || !process.argv[index + 1]) throw new Error(`missing ${name}`);
  return process.argv[index + 1];
}

async function main() {
  const sessionDir = path.resolve(option("--session-dir"));
  const output = path.resolve(option("--output"));
  const sessionManifest = JSON.parse(
    await readFile(path.join(sessionDir, "session-manifest.json"), "utf8"),
  );
  // Validate the fixed workflow-owned destination against this running frozen
  // reporter before deleting even an exact atomic-temp sibling.
  assertFixedReportOutput({ sessionManifest, output });
  // A hard kill can strand only the atomic writer's exact sibling temp. Remove
  // dead-PID regular files matching that workflow-owned pattern before Git
  // status verification; never touch the output or a near-match.
  await cleanupOwnedAtomicTemps(output);
  if (process.argv.includes("--verify-only")) {
    await verifyReportProtocol({ sessionManifest, output });
    process.stdout.write(`${sessionManifest.protocol.lockSha256}\n`);
    return;
  }
  const [recordedLedger, decision] = await Promise.all([
    readFile(path.join(sessionDir, "session-ledger.json"), "utf8").then(JSON.parse),
    readFile(path.join(sessionDir, "decision.json"), "utf8").then(JSON.parse),
  ]);
  const numberedEvents = await readNumberedLedger(sessionDir);
  if (!isDeepStrictEqual(numberedEvents, recordedLedger.events)) {
    throw new Error("aggregate ledger differs from contiguous numbered events");
  }
  const ledger = deriveLedgerProjection(sessionManifest.sessionId, numberedEvents);
  if (!isDeepStrictEqual(ledger, recordedLedger)) {
    throw new Error("aggregate ledger projection differs from numbered events");
  }
  const locatorBytes = await readFile(sessionManifest.locatorPath);
  const locator = JSON.parse(locatorBytes.toString("utf8"));
  if (
    locator.sessionId !== sessionManifest.sessionId
    || !sameAbsolutePath(locator.sessionDir, sessionDir)
    || !sameAbsolutePath(sessionManifest.sessionDir, sessionDir)
    || !sameAbsolutePath(locator.sessionManifestPath, path.join(sessionDir, "session-manifest.json"))
    || !isDeepStrictEqual(locator, sessionManifest.locatorRecord)
    || sha256Bytes(locatorBytes) !== sessionManifest.locatorSha256
  ) throw new Error("session locator differs from immutable manifest anchor");
  await verifyReportProtocol({ sessionManifest, output });
  // The index writer uses the same atomic sibling scheme. Only after both the
  // external locator and frozen reporter have authenticated this session path
  // may a dead publisher's exact index temp be removed.
  await cleanupOwnedAtomicTemps(path.join(sessionDir, "artifact-index.json"));
  const measurementBaselines = numberedEvents.filter((event) =>
    event.type === "attempt_environment" && event.environmentBaseline === true,
  );
  if (measurementBaselines.length !== 1) {
    throw new Error(`expected exactly one authoritative attempt environment, got ${measurementBaselines.length}`);
  }
  const measurementEnvironment = measurementBaselines[0].environment;
  const finalMainTargetSnapshot = await snapshotDirectory(measurementEnvironment.mainTargetDirectory);
  if (JSON.stringify(finalMainTargetSnapshot) !== JSON.stringify(measurementEnvironment.mainTargetSnapshot)) {
    throw new Error("main target content changed during the diagnostic session");
  }
  const attemptResults = [];
  for (const attempt of ledger.attempts) {
    if (!attempt.resultPath) throw new Error(`attempt ${attempt.attemptId} has no immutable result path`);
    attemptResults.push(JSON.parse(await readFile(attempt.resultPath, "utf8")));
  }
  assertRetryProtocol({ events: numberedEvents, attemptResults, decision });
  const artifactIndex = await prepareArtifactIndex(sessionDir, sessionManifest.locatorPath);
  const reportBytes = Buffer.from(
    renderVerification({
      sessionManifest,
      measurementEnvironment,
      ledger,
      decision,
      attemptResults,
      artifactIndex,
    }),
    "utf8",
  );
  // All ledger, locator, raw-result, and arithmetic checks above complete before
  // either publication. A crash between the two writes is recoverable: rerun
  // accepts only byte-identical output and creates the missing peer.
  await publishReportPair({ artifactIndex, output, reportBytes });
  process.stdout.write(`${output}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  await main();
}
```


The artifact index never traverses the external `worktreeParent` (and
defensively excludes any `worktrees/**` directory inside the session). Those
rebuildable trees and Cargo targets remain preserved until review, while the
index covers immutable manifests, ledger, raw command logs, recovery evidence,
block/attempt JSON, fingerprint logs, and copied timings HTML.
Before either verify-only or full rendering, the reporter first requires its
running worktree to equal the manifest's protocol root and `--output` to equal
the single frozen `REPORT_PATH`. Only then does it remove a dead publisher's
regular `<verification-name>.<pid>.<uuid>.tmp` sibling. Full rendering also
authenticates the external locator and protocol pin before removing the same
exact pattern for fixed `artifact-index.json`, then walks the session. It
refuses live-PID, reparse, directory, and near-match entries; neither final
publication is deleted or overwritten. This makes hard kills before or after
either atomic link replayable without broad cleanup authority.

- [ ] **Step 5: Make coordinator start/resume/attempt creation verify the exact protocol pin**

Add this import to `scripts/process-shell-diagnostic/coordinator.mjs`:

```js
import { verifyFrozenProtocol } from "./freeze.mjs";
```

Add the verifier to `DEFAULT_DEPENDENCIES` and remove the now-unused
`resolveProtocolCommitFn` entry:

```js
const DEFAULT_DEPENDENCIES = Object.freeze({
  uuidFn: randomUUID,
  nowFn: () => new Date().toISOString(),
  processEnv: process.env,
  verifyFrozenProtocolFn: verifyFrozenProtocol,
  captureEnvironmentFn: captureEnvironmentProduction,
  createDetachedWorktreeFn: createDetachedWorktreeProduction,
  runAttemptFn: runAttempt,
  restoreAttemptWorktreeFn: installState,
  writeJsonFn: writeAtomicJsonExclusive,
  afterDurableWriteFn: async () => {},
  afterAttemptObservedFn: async () => {},
});
```

Delete `resolveProtocolCommitProduction`. Add these helpers beside the locator
functions; the parsed lock is kept separate from the smaller pin stored in the
manifest:

```js
function splitVerifiedProtocol(verified) {
  const { protocolLock, ...protocol } = verified;
  return { protocol, protocolLock };
}

function assertSameProtocolPin(actual, expected) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new ProtocolError("protocol_pin_mismatch", "worktree/session protocol pin differs", {
      actual,
      expected,
    });
  }
}
```

In `startSession`, immediately after resolving the three roots—and before
creating the scratch parent, locator, session directory, or attempt—verify the
Git-blob-backed lock. Delete both the checkout-path
`readFile(protocolLockPath)` and the later `resolveProtocolCommitFn` call:

```js
const verifiedProtocol = await deps.verifyFrozenProtocolFn({ repoRoot: protocolRoot });
const { protocol, protocolLock } = splitVerifiedProtocol(verifiedProtocol);
const protocolLockPath = path.join(protocolRoot, ...protocol.lockPath.split("/"));
```

Preserve Task 5's locator-as-reservation ordering. The final recovery seed and
materialization block is exactly:

```js
const locatorRecord = {
  schemaVersion: 1,
  sessionId,
  createdAt: deps.nowFn(),
  mainRoot,
  protocolRoot,
  scratchParent,
  sessionDir,
  worktreeParent: path.join(mainRoot, ".worktrees", `process-shell-session-${sessionId}`),
  sessionManifestPath: path.join(sessionDir, "session-manifest.json"),
  protocolLockPath,
  protocolLock,
  protocol,
};
await publishJsonIdempotent(locatorPath, locatorRecord, deps);
await mkdir(sessionDir, { recursive: true });
const environment = await deps.captureEnvironmentFn({
  mainRoot,
  artifactDir: path.join(sessionDir, "bootstrap"),
  processAttested: true,
  protocolLock,
});
const manifest = manifestFromLocator(locatorPath, locatorRecord, environment);
await publishJsonIdempotent(locatorRecord.sessionManifestPath, manifest, deps);
```

The locator is the only reservation point: no final `sessionDir` exists before
it. Use `protocol.protocolCommit` in `session_started`.

Inside `loadManifest`, immediately after reading/validating the locator and
before creating a missing session directory, capturing recovery environment,
or materializing a missing manifest, reverify the locator seed from Git blobs:

```js
const resumed = splitVerifiedProtocol(
  await deps.verifyFrozenProtocolFn({ repoRoot: locatorRecord.protocolRoot }),
);
assertSameProtocolPin(resumed.protocol, locatorRecord.protocol);
if (JSON.stringify(resumed.protocolLock) !== JSON.stringify(locatorRecord.protocolLock)) {
  throw new ProtocolError("protocol_lock_seed_mismatch", "verified lock differs from locator WAL");
}
```

In `launchAttempt`, immediately after `createDetachedWorktreeFn` and before
checking/creating the target or invoking Cargo, run:

```js
const attemptVerified = splitVerifiedProtocol(
  await deps.verifyFrozenProtocolFn({ repoRoot: worktree }),
);
assertSameProtocolPin(attemptVerified.protocol, manifest.protocol);
if (JSON.stringify(attemptVerified.protocolLock) !== JSON.stringify(manifest.protocolLock)) {
  throw new ProtocolError("attempt_protocol_lock_mismatch", attemptId);
}
```

Add one shared completion verifier:

```js
async function verifyAttemptCompletionPins(manifest, startedEvent, deps) {
  for (const [scope, repoRoot] of [
    ["attempt", startedEvent.worktree],
    ["protocol", manifest.protocolRoot],
  ]) {
    const completed = splitVerifiedProtocol(await deps.verifyFrozenProtocolFn({ repoRoot }));
    assertSameProtocolPin(completed.protocol, manifest.protocol);
    if (JSON.stringify(completed.protocolLock) !== JSON.stringify(manifest.protocolLock)) {
      throw new ProtocolError(
        "post_attempt_protocol_lock_mismatch",
        `${startedEvent.attemptId}:${scope}`,
      );
    }
  }
}
```

After `runAttemptFn` returns and its durable result is byte-compared, but before
`afterAttemptObservedFn` or `attempt_finished`, call
`verifyAttemptCompletionPins(manifest, startedEvent, deps)`. Keep it inside
`launchAttempt`'s existing `try`, so a mismatch becomes its immutable
`coordinator-failure.json`.

The same check is mandatory on crash replay. In `recoverStartedAttempt`, after
any required A recovery and identity validation but before
`normalizeResultArtifact`, run the helper whenever the pinned selected source
is the normal `attempt-result.json`. If it throws, publish the same
`coordinator-failure.json` value built by `launchAttempt`'s catch, and replace
the selected normal result/path with that failure before `finishAttempt`.
Extract the existing failure object literal into one helper so both paths have
identical bytes and reason `coordinator_failure`; preserve a `simulatedCrash`
throw without projection. This closes the crash window between the durable
Cargo result and its post-run pin check. A normal result may reach
`attempt_finished` only after the attempt worktree and protocol root have both
passed this exact verifier in the current process.

In the test fixture inside
`scripts/process-shell-diagnostic/coordinator.test.ts`, replace
`resolveProtocolCommitFn` with this object-entry fragment:

```text
verifyFrozenProtocolFn: async (_input: { repoRoot: string }) => ({
  protocolCommit: "a".repeat(40),
  lockPath: "scripts/process-shell-diagnostic/protocol-lock.json",
  lockBlob: "b".repeat(40),
  lockSha256: "c".repeat(64),
  protocolVersion: 1,
  protocolLock: {
    schemaVersion: 1,
    states: {
      A: { srcTauriTree: "a-tree" },
      B: { srcTauriTree: "b-tree" },
      C: { srcTauriTree: "c-tree" },
      D: { srcTauriTree: "d-tree" },
      E: { srcTauriTree: "e-tree" },
    },
  },
}),
```

Add this test:

```ts
it("pins the lock-containing commit, blob, and SHA in the immutable manifest", async () => {
  const input = await paths();
  const value = fake([attempt("valid")]);
  const result = await startSession({ ...input, processAttested: true }, value.dependencies);
  const manifest = JSON.parse(
    await readFile(path.join(result.sessionDir, "session-manifest.json"), "utf8"),
  );
  expect(manifest.protocol).toEqual({
    protocolCommit: "a".repeat(40),
    lockPath: "scripts/process-shell-diagnostic/protocol-lock.json",
    lockBlob: "b".repeat(40),
    lockSha256: "c".repeat(64),
    protocolVersion: 1,
  });
});

it("invalidates a valid Cargo result when the protocol root changes mid-flight", async () => {
  const input = await paths();
  const value = fake([attempt("valid")]);
  const baseRun = value.dependencies.runAttemptFn;
  const baseVerify = value.dependencies.verifyFrozenProtocolFn;
  let cargoCompleted = false;
  const result = await startSession(
    { ...input, processAttested: true },
    {
      ...value.dependencies,
      runAttemptFn: async (spec: Record<string, string>) => {
        const completed = await baseRun(spec);
        cargoCompleted = true;
        return completed;
      },
      verifyFrozenProtocolFn: async ({ repoRoot }: { repoRoot: string }) => {
        const verified = await baseVerify({ repoRoot });
        return cargoCompleted && path.resolve(repoRoot) === path.resolve(input.protocolRoot)
          ? { ...verified, lockSha256: "d".repeat(64) }
          : verified;
      },
    },
  );
  expect(result.status).toBe("awaiting_correction");
  expect(JSON.parse(await readFile(result.attempts[0].resultPath, "utf8"))).toMatchObject({
    kind: "infrastructure_invalid",
    reasons: ["coordinator_failure"],
    error: { kind: "protocol_pin_mismatch" },
  });
});

it("rechecks the attempt-worktree pin when resuming a durable pre-check result", async () => {
  const input = await paths();
  const value = fake([attempt("valid")]);
  const baseRun = value.dependencies.runAttemptFn;
  const baseVerify = value.dependencies.verifyFrozenProtocolFn;
  let cargoCompleted = false;
  let crashBeforePostCheck = true;
  await expect(startSession(
    { ...input, processAttested: true },
    {
      ...value.dependencies,
      runAttemptFn: async (spec: Record<string, string>) => {
        const completed = await baseRun(spec);
        cargoCompleted = true;
        return completed;
      },
      verifyFrozenProtocolFn: async ({ repoRoot }: { repoRoot: string }) => {
        if (
          cargoCompleted
          && crashBeforePostCheck
          && path.resolve(repoRoot) !== path.resolve(input.protocolRoot)
        ) {
          crashBeforePostCheck = false;
          throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
        }
        return baseVerify({ repoRoot });
      },
    },
  )).rejects.toThrow("simulated crash");
  const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
  let failurePublicationCrash = false;
  await expect(resumeSession(
    { sessionDir, processAttested: true },
    {
      ...value.dependencies,
      verifyFrozenProtocolFn: async ({ repoRoot }: { repoRoot: string }) => {
        const verified = await baseVerify({ repoRoot });
        return path.resolve(repoRoot) === path.resolve(input.protocolRoot)
          ? verified
          : { ...verified, lockSha256: "d".repeat(64) };
      },
      afterDurableWriteFn: async ({ target }: { target: string }) => {
        if (!failurePublicationCrash && target.endsWith("coordinator-failure.json")) {
          failurePublicationCrash = true;
          throw Object.assign(new Error("simulated failure-publication crash"), { simulatedCrash: true });
        }
      },
    },
  )).rejects.toThrow("simulated failure-publication crash");
  const recovered = await resumeSession(
    { sessionDir, processAttested: true },
    { ...value.dependencies, verifyFrozenProtocolFn: baseVerify },
  );
  expect(recovered.status).toBe("awaiting_correction");
  expect(JSON.parse(await readFile(recovered.attempts[0].resultPath, "utf8"))).toMatchObject({
    kind: "infrastructure_invalid",
    reasons: ["coordinator_failure"],
    error: { kind: "protocol_pin_mismatch" },
  });
});
```

- [ ] **Step 6: Run report/coordinator GREEN and all harness tests**

Run:

```powershell
npm.cmd run test -- scripts/process-shell-diagnostic/report.test.ts scripts/process-shell-diagnostic/coordinator.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Report/coordinator focused tests failed.' }
npm.cmd run test -- scripts/process-shell-diagnostic/protocol.test.ts scripts/process-shell-diagnostic/runtime.test.ts scripts/process-shell-diagnostic/git-state.test.ts scripts/process-shell-diagnostic/attempt.test.ts scripts/process-shell-diagnostic/coordinator.test.ts scripts/process-shell-diagnostic/report.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Combined diagnostic harness tests failed.' }
npm.cmd run check
if ($LASTEXITCODE -ne 0) { throw 'Repository type/check gate failed.' }
```

Expected: all six test files PASS, including eight report tests and twenty-seven
coordinator tests (the Task 5 crash matrix plus all three Task 6 protocol-pin tests);
`svelte-check` reports zero errors.

- [ ] **Step 7: Commit all lock inputs before generating the lock**

Run:

```powershell
$diagnosticExpectedTracked = @(
    'scripts/process-shell-diagnostic/coordinator.mjs',
    'scripts/process-shell-diagnostic/coordinator.test.ts'
)
$diagnosticExpectedUntracked = @(
    'scripts/process-shell-diagnostic/freeze.mjs',
    'scripts/process-shell-diagnostic/report.mjs',
    'scripts/process-shell-diagnostic/report.test.ts'
)
$diagnosticTracked = @(git diff --name-only)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate Task 6 tracked changes.' }
$diagnosticUntracked = @(git ls-files --others --exclude-standard)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate Task 6 untracked files.' }
if (@(Compare-Object $diagnosticExpectedTracked $diagnosticTracked).Count -ne 0) {
    throw "Unexpected tracked Task 6 inventory: $($diagnosticTracked -join ', ')"
}
if (@(Compare-Object $diagnosticExpectedUntracked $diagnosticUntracked).Count -ne 0) {
    throw "Unexpected untracked Task 6 inventory: $($diagnosticUntracked -join ', ')"
}
git diff --cached --quiet
if ($LASTEXITCODE -eq 1) { throw 'Task 6 began with pre-existing staged changes.' }
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect the staged Task 6 inventory.' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 6 worktree diff check failed.' }
git add -- scripts/process-shell-diagnostic/freeze.mjs scripts/process-shell-diagnostic/report.mjs scripts/process-shell-diagnostic/report.test.ts scripts/process-shell-diagnostic/coordinator.mjs scripts/process-shell-diagnostic/coordinator.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Could not stage Task 6 lock inputs.' }
$diagnosticExpectedStaged = @($diagnosticExpectedTracked + $diagnosticExpectedUntracked | Sort-Object)
$diagnosticStaged = @(git diff --cached --name-only | Sort-Object)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate staged Task 6 inputs.' }
if (@(Compare-Object $diagnosticExpectedStaged $diagnosticStaged).Count -ne 0) {
    throw "Unexpected staged Task 6 inventory: $($diagnosticStaged -join ', ')"
}
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Staged Task 6 diff check failed.' }
git commit -m "feat: freeze and report process shell diagnostics"
if ($LASTEXITCODE -ne 0) { throw 'Task 6 lock-input commit failed.' }
$diagnosticStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect status after the Task 6 commit.' }
if ($diagnosticStatus.Count -ne 0) { throw "Task 6 commit left changes: $($diagnosticStatus -join '; ')" }
```

Expected: the commit contains all Task 6 source/test changes, while
`protocol-lock.json` is still absent; final status is clean.

- [ ] **Step 8: Generate, commit, and verify the self-reference-free lock**

Run:

```powershell
$diagnosticRepoRoot = (Get-Location).Path
node scripts/process-shell-diagnostic/freeze.mjs generate --repo-root "$diagnosticRepoRoot"
if ($LASTEXITCODE -ne 0) { throw 'Protocol-lock generation failed.' }
$diagnosticLockStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect the generated lock.' }
if ($diagnosticLockStatus.Count -ne 1 -or $diagnosticLockStatus[0] -ne '?? scripts/process-shell-diagnostic/protocol-lock.json') {
    throw "Lock generation changed an unexpected inventory: $($diagnosticLockStatus -join '; ')"
}
git add -- scripts/process-shell-diagnostic/protocol-lock.json
if ($LASTEXITCODE -ne 0) { throw 'Could not stage protocol-lock.json.' }
$diagnosticLockStaged = @(git diff --cached --name-only)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate the staged protocol lock.' }
if ($diagnosticLockStaged.Count -ne 1 -or $diagnosticLockStaged[0] -ne 'scripts/process-shell-diagnostic/protocol-lock.json') {
    throw "Unexpected staged lock inventory: $($diagnosticLockStaged -join '; ')"
}
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Staged protocol-lock diff check failed.' }
git commit -m "chore: freeze process shell diagnostic protocol"
if ($LASTEXITCODE -ne 0) { throw 'Protocol-lock commit failed.' }
node scripts/process-shell-diagnostic/freeze.mjs verify --repo-root "$diagnosticRepoRoot"
if ($LASTEXITCODE -ne 0) { throw 'Committed protocol-lock verification failed.' }
$diagnosticFinalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect status after protocol freeze.' }
if ($diagnosticFinalStatus.Count -ne 0) { throw "Protocol freeze left changes: $($diagnosticFinalStatus -join '; ')" }
```

Expected: generation creates exactly one `protocol-lock.json`; verification
prints the lock-containing commit, lock blob, lock SHA-256, and protocol
version; A/D historical anchors and all B/C/E patch-derived state hashes match;
final status is clean. From this commit onward, no `FROZEN_INPUTS` file may be
edited before the session terminates.

### Task 7: Validate Every State Outside Measurement and Run the Frozen Session

**Files:**

- Read only: all frozen protocol inputs and `protocol-lock.json`
- Transient: one validation worktree/target and one or more coordinator-owned
  attempt worktrees/targets
- Write outside Git: `%TEMP%/extractum-process-shell-sessions/**`

**Interfaces:**

- Consumes the lock-containing commit and coordinator CLI from Task 6.
- Produces either one valid terminal decision, one terminal
  `environment_precision_insufficient`, or an immutable non-stability failure
  awaiting an objective correction. It never changes source or protocol files.

- [ ] **Step 1: Invoke the worktree workflow and establish exact roots**

Use `superpowers:using-git-worktrees` before creating the validation worktree.
The skill action is required because both validation and measurement must be
isolated from the main checkout. From the clean lock-containing implementation
worktree, run:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticLockCommitOutput = @(git -C "$diagnosticProtocolRoot" rev-parse HEAD)
$diagnosticLockCommitExit = $LASTEXITCODE
if ($diagnosticLockCommitExit -ne 0) { throw 'Could not resolve the frozen protocol commit.' }
$diagnosticLockCommit = ($diagnosticLockCommitOutput -join '').Trim()
if ($diagnosticLockCommit -notmatch '^[0-9a-fA-F]{40}$') { throw 'Frozen protocol commit is not a full Git object id.' }
$diagnosticValidationRoot = Join-Path $diagnosticMainRoot '.worktrees\process-shell-diagnostic-validation'
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'

if ($diagnosticProtocolRoot -eq $diagnosticMainRoot) {
    throw 'Run the frozen protocol from its workflow-owned implementation worktree, not main.'
}
if (Test-Path -LiteralPath $diagnosticValidationRoot) {
    throw "Validation worktree already exists: $diagnosticValidationRoot"
}
if (@(Get-ChildItem Env: | Where-Object Name -IEQ 'CARGO_TARGET_DIR').Count -ne 0) {
    throw 'CARGO_TARGET_DIR must be absent.'
}

node "$diagnosticProtocolRoot\scripts\process-shell-diagnostic\freeze.mjs" verify --repo-root "$diagnosticProtocolRoot"
if ($LASTEXITCODE -ne 0) { throw 'Frozen protocol verification failed.' }
$diagnosticProtocolStatus = @(git -C "$diagnosticProtocolRoot" status --short)
if ($LASTEXITCODE -ne 0) { throw 'Protocol worktree status failed.' }
if ($diagnosticProtocolStatus.Count -ne 0) {
    $diagnosticProtocolStatus | ForEach-Object { Write-Output $_ }
    throw 'Protocol worktree is not clean.'
}
```

Expected: lock verification prints the current commit/blob pin; status is
clean; `CARGO_TARGET_DIR` is absent.

- [ ] **Step 2: Create a cold validation worktree that is never measured**

Run through the worktree workflow:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticValidationRoot = Join-Path $diagnosticMainRoot '.worktrees\process-shell-diagnostic-validation'
$diagnosticLockCommitOutput = @(git -C "$diagnosticProtocolRoot" rev-parse HEAD)
$diagnosticLockCommitExit = $LASTEXITCODE
if ($diagnosticLockCommitExit -ne 0) { throw 'Could not resolve the frozen protocol commit.' }
$diagnosticLockCommit = ($diagnosticLockCommitOutput -join '').Trim()
if ($diagnosticLockCommit -notmatch '^[0-9a-fA-F]{40}$') { throw 'Frozen protocol commit is not a full Git object id.' }
git -C "$diagnosticProtocolRoot" worktree add --detach "$diagnosticValidationRoot" "$diagnosticLockCommit"
if ($LASTEXITCODE -ne 0) { throw 'Validation worktree creation failed.' }
$diagnosticValidationHeadOutput = @(git -C "$diagnosticValidationRoot" rev-parse HEAD)
if ($LASTEXITCODE -ne 0) { throw 'Could not resolve validation worktree HEAD.' }
$diagnosticValidationHead = ($diagnosticValidationHeadOutput -join '').Trim()
if ($diagnosticValidationHead -ne $diagnosticLockCommit) { throw 'Validation worktree is not at the frozen protocol commit.' }

$diagnosticValidationTarget = Join-Path $diagnosticValidationRoot 'src-tauri\target'
if (Test-Path -LiteralPath $diagnosticValidationTarget) {
    throw 'Fresh validation worktree unexpectedly contains a target directory.'
}
```

Expected: detached HEAD equals `$diagnosticLockCommit`; target is absent. This
target is separate from every later measurement target and from main.

- [ ] **Step 3: Validate A, B, C, E, and exact historical D in the validation target**

Run the harness tests first:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
Push-Location -LiteralPath $diagnosticProtocolRoot
try {
    npm.cmd run test -- scripts/process-shell-diagnostic/protocol.test.ts scripts/process-shell-diagnostic/runtime.test.ts scripts/process-shell-diagnostic/git-state.test.ts scripts/process-shell-diagnostic/attempt.test.ts scripts/process-shell-diagnostic/coordinator.test.ts scripts/process-shell-diagnostic/report.test.ts
    if ($LASTEXITCODE -ne 0) { throw 'Harness validation failed.' }
} finally {
    Pop-Location
}
```

Before any Cargo validation, smoke-test the production Git installer—not a
manual approximation—through every state and final restore:

```powershell
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticValidationRoot = (Resolve-Path -LiteralPath (Join-Path $diagnosticMainRoot '.worktrees\process-shell-diagnostic-validation')).Path
$diagnosticInstallScript = @'
import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
const [worktree, mainRoot, state, artifactDir] = process.argv.slice(1);
const { installState } = await import(pathToFileURL(path.join(worktree, "scripts", "process-shell-diagnostic", "git-state.mjs")).href);
const protocolLock = JSON.parse(await readFile(path.join(worktree, "scripts", "process-shell-diagnostic", "protocol-lock.json"), "utf8"));
const evidence = await installState({ state, worktree, mainRoot, protocolLock, artifactDir });
if (!evidence.srcTauriTree || !evidence.canonicalLibSha256) throw new Error(`missing state evidence for ${state}`);
'@
foreach ($diagnosticState in @('A0', 'B', 'C', 'D', 'E', 'A-final')) {
    $diagnosticStateArtifacts = Join-Path $env:TEMP "extractum-process-installer-smoke-$diagnosticState-$([guid]::NewGuid())"
    node --input-type=module -e $diagnosticInstallScript "$diagnosticValidationRoot" "$diagnosticMainRoot" "$diagnosticState" "$diagnosticStateArtifacts"
    if ($LASTEXITCODE -ne 0) { throw "Production installState smoke failed for $diagnosticState." }
}
$diagnosticInstalledStatus = @(git -C "$diagnosticValidationRoot" status --porcelain=v1 --untracked-files=all -- src-tauri)
$diagnosticInstalledStatusExit = $LASTEXITCODE
if ($diagnosticInstalledStatusExit -ne 0) { throw 'Could not inspect A-final installer status.' }
if ($diagnosticInstalledStatus.Count -ne 0) {
    throw 'Production A-final did not restore an exact clean src-tauri state.'
}
```

Expected: all six production calls write their real state evidence, B/C/E use
canonical committed patch blobs, D executes the literal candidate checkout and
inventory comparison, and `A-final` leaves exact A. A failure here occurs before
A0 and is fixed only by a new committed/frozen protocol.

Validate A and prove Cargo selected only the validation target:

```powershell
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticValidationRoot = (Resolve-Path -LiteralPath (Join-Path $diagnosticMainRoot '.worktrees\process-shell-diagnostic-validation')).Path
$diagnosticValidationTarget = Join-Path $diagnosticValidationRoot 'src-tauri\target'
$diagnosticManifest = Join-Path $diagnosticValidationRoot 'src-tauri\Cargo.toml'
git -C "$diagnosticValidationRoot" checkout 24c313a767a25284123b24ea3a4b8c083007c817 -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'A installation failed.' }
$diagnosticMetadataJson = cargo metadata --manifest-path "$diagnosticManifest" --format-version 1 --no-deps --locked
if ($LASTEXITCODE -ne 0) { throw 'A locked metadata failed.' }
$diagnosticMetadata = $diagnosticMetadataJson | ConvertFrom-Json
if ((Resolve-Path -LiteralPath $diagnosticMetadata.workspace_root).Path -ne (Resolve-Path -LiteralPath (Join-Path $diagnosticValidationRoot 'src-tauri')).Path) {
    throw 'Validation workspace_root mismatch.'
}
if ([IO.Path]::GetFullPath($diagnosticMetadata.target_directory) -ne [IO.Path]::GetFullPath($diagnosticValidationTarget)) {
    throw 'Validation target_directory mismatch.'
}
$diagnosticARootTreeOutput = @(git -C "$diagnosticValidationRoot" write-tree)
if ($LASTEXITCODE -ne 0) { throw 'Could not materialize the A index tree.' }
$diagnosticARootTree = ($diagnosticARootTreeOutput -join '').Trim()
$diagnosticATreeOutput = @(git -C "$diagnosticValidationRoot" rev-parse "${diagnosticARootTree}:src-tauri")
if ($LASTEXITCODE -ne 0) { throw 'Could not resolve the A src-tauri tree.' }
$diagnosticATree = ($diagnosticATreeOutput -join '').Trim()
if ($diagnosticATree -ne 'fd9711a041432ef420e7b09d56a46131a2a52a2a') {
    throw 'A tree mismatch.'
}
```

For B, C, and E, always restore A first, apply exactly one committed patch, run
the two owning-package checks, and verify `--locked` metadata:

```powershell
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticValidationRoot = (Resolve-Path -LiteralPath (Join-Path $diagnosticMainRoot '.worktrees\process-shell-diagnostic-validation')).Path
$diagnosticManifest = Join-Path $diagnosticValidationRoot 'src-tauri\Cargo.toml'
foreach ($diagnosticState in @('B', 'C', 'E')) {
    $diagnosticStateArtifacts = Join-Path $env:TEMP "extractum-process-state-validation-$diagnosticState-$([guid]::NewGuid())"
    $diagnosticInstallScript = @'
import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
const [worktree, mainRoot, state, artifactDir] = process.argv.slice(1);
const { installState } = await import(pathToFileURL(path.join(worktree, "scripts", "process-shell-diagnostic", "git-state.mjs")).href);
const protocolLock = JSON.parse(await readFile(path.join(worktree, "scripts", "process-shell-diagnostic", "protocol-lock.json"), "utf8"));
await installState({ state, worktree, mainRoot, protocolLock, artifactDir });
'@
    node --input-type=module -e $diagnosticInstallScript "$diagnosticValidationRoot" "$diagnosticMainRoot" "$diagnosticState" "$diagnosticStateArtifacts"
    if ($LASTEXITCODE -ne 0) { throw "$diagnosticState canonical state installation failed." }
    cargo metadata --manifest-path "$diagnosticManifest" --format-version 1 --no-deps --locked | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "$diagnosticState locked metadata failed." }
    cargo check --manifest-path "$diagnosticManifest" -p extractum-process --all-targets --locked
    if ($LASTEXITCODE -ne 0) { throw "$diagnosticState extractum-process check failed." }
    cargo check --manifest-path "$diagnosticManifest" -p extractum --all-targets --locked
    if ($LASTEXITCODE -ne 0) { throw "$diagnosticState extractum check failed." }
}
```

Reconstruct and validate D only through the approved command and historical
blob/tree comparison:

```powershell
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticValidationRoot = (Resolve-Path -LiteralPath (Join-Path $diagnosticMainRoot '.worktrees\process-shell-diagnostic-validation')).Path
$diagnosticManifest = Join-Path $diagnosticValidationRoot 'src-tauri\Cargo.toml'
git -C "$diagnosticValidationRoot" checkout 24c313a767a25284123b24ea3a4b8c083007c817 -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'A restore before D failed.' }
git -C "$diagnosticValidationRoot" checkout b364756c7b5768d644321afeaeb81ec04e2481a4 -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'Literal D checkout failed.' }
git -C "$diagnosticValidationRoot" diff --quiet b364756c7b5768d644321afeaeb81ec04e2481a4 -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'D differs from the historical candidate.' }
$diagnosticDRootTreeOutput = @(git -C "$diagnosticValidationRoot" write-tree)
if ($LASTEXITCODE -ne 0) { throw 'Could not materialize the D index tree.' }
$diagnosticDRootTree = ($diagnosticDRootTreeOutput -join '').Trim()
$diagnosticDTreeOutput = @(git -C "$diagnosticValidationRoot" rev-parse "${diagnosticDRootTree}:src-tauri")
if ($LASTEXITCODE -ne 0) { throw 'Could not resolve the D src-tauri tree.' }
$diagnosticDTree = ($diagnosticDTreeOutput -join '').Trim()
if ($diagnosticDTree -ne '77e2d163ccc8bddf3ea051cb995909888cae9aba') {
    throw 'D tree mismatch.'
}
$diagnosticExactOutput = @(cargo test --manifest-path "$diagnosticManifest" --locked -p extractum-process --lib external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets -- --exact 2>&1)
$diagnosticExactExit = $LASTEXITCODE
$diagnosticExactOutput | ForEach-Object { Write-Output $_ }
if ($diagnosticExactExit -ne 0) { throw 'D exact characterization test failed.' }
$diagnosticExactText = $diagnosticExactOutput -join "`n"
if ($diagnosticExactText -notmatch '(?m)^running 1 test$' -or $diagnosticExactText -notmatch '(?m)^test result: ok\. 1 passed;') {
    throw 'D exact characterization selection was empty or not exactly one passing test.'
}
cargo check --manifest-path "$diagnosticManifest" -p extractum-process --all-targets --locked
if ($LASTEXITCODE -ne 0) { throw 'D extractum-process check failed.' }
cargo test --manifest-path "$diagnosticManifest" -p extractum-process --all-targets --locked
if ($LASTEXITCODE -ne 0) { throw 'D extractum-process checkpoint failed.' }
cargo check --manifest-path "$diagnosticManifest" -p extractum --all-targets --locked
if ($LASTEXITCODE -ne 0) { throw 'D downstream extractum check failed.' }
```

Expected: the exact D test is non-empty and PASS; every focused check/test is
PASS; no command used a measurement target.

- [ ] **Step 4: Restore A and remove only the clean validation worktree**

Run:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticValidationRoot = (Resolve-Path -LiteralPath (Join-Path $diagnosticMainRoot '.worktrees\process-shell-diagnostic-validation')).Path
git -C "$diagnosticValidationRoot" checkout 24c313a767a25284123b24ea3a4b8c083007c817 -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'Final A restoration failed.' }
git -C "$diagnosticValidationRoot" diff --quiet HEAD -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'Final A worktree diff is not clean.' }
git -C "$diagnosticValidationRoot" diff --cached --quiet HEAD -- src-tauri
if ($LASTEXITCODE -ne 0) { throw 'Final A index diff is not clean.' }
$diagnosticFinalValidationStatus = @(git -C "$diagnosticValidationRoot" status --porcelain --untracked-files=all)
$diagnosticFinalValidationStatusExit = $LASTEXITCODE
if ($diagnosticFinalValidationStatusExit -ne 0) { throw 'Could not inspect final validation status.' }
if ($diagnosticFinalValidationStatus.Count -ne 0) {
    throw 'Validation worktree is not clean after A restoration.'
}
```

Then use the normal clean-worktree cleanup path from
`superpowers:using-git-worktrees` to remove only
`G:/Develop/Extractum/.worktrees/process-shell-diagnostic-validation` (resolve
the literal path again in the cleanup invocation). Expected: validation target is deleted with its
workflow-owned worktree; main and future attempt targets are untouched.

- [ ] **Step 5: Prove machine quiescence without killing user processes**

Run these read-only checks:

```powershell
$diagnosticBlocking = @(
    Get-Process -ErrorAction SilentlyContinue |
        Where-Object { $_.ProcessName -in @('cargo', 'rustc', 'rust-analyzer') }
)
$diagnosticBuildHosts = @(
    Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
        Where-Object {
            $_.CommandLine -match '(?i)(tauri\s+dev|vite(\.js)?\s+--host|npm\.cmd\s+run\s+tauri)'
        }
)
if ($diagnosticBlocking.Count -ne 0 -or $diagnosticBuildHosts.Count -ne 0) {
    $diagnosticBlocking | Format-Table Id, ProcessName
    $diagnosticBuildHosts | Select-Object ProcessId, Name, CommandLine | Format-List
    throw 'Stop or prove idle the listed processes before attesting; do not kill them automatically.'
}

```

Expected: no Cargo/Rust build, Rust Analyzer, Tauri dev, or Vite build host is
active. Only after observing this result may `--process-attested` be passed.
The coordinator records bootstrap context in the immutable session manifest,
then the first durable `attempt_environment` immediately before A0
content-hashes every directory and file under the main `src-tauri/target` and
becomes the authoritative measurement baseline. The reporter selects exactly
that event and repeats the full snapshot before it publishes anything. No
PowerShell variable from this step is used as later evidence.

- [ ] **Step 6: Start the frozen session directly and monitor it**

Run the Node coordinator directly from the clean protocol root; do not use
`Start-Process -Wait` and do not redirect acceptance output through another
shell:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'
Push-Location -LiteralPath $diagnosticProtocolRoot
try {
  node scripts/process-shell-diagnostic/coordinator.mjs start `
    --main-root "$diagnosticMainRoot" `
    --protocol-root "$diagnosticProtocolRoot" `
    --scratch-parent "$diagnosticScratchParent" `
    --process-attested
  $diagnosticStartExit = $LASTEXITCODE
  if ($diagnosticStartExit -ne 0) {
    $diagnosticLocator = Join-Path $diagnosticScratchParent 'process-shell-diagnostic.locator.json'
    if (-not (Test-Path -LiteralPath $diagnosticLocator)) { throw 'Diagnostic start failed before locator reservation.' }
    $diagnosticReservedSession = (Get-Content -LiteralPath $diagnosticLocator -Raw | ConvertFrom-Json).sessionDir
    $diagnosticManifestMissing = -not (Test-Path -LiteralPath (Join-Path $diagnosticReservedSession 'session-manifest.json'))
    $diagnosticEvents = @()
    $diagnosticLedgerDir = Join-Path $diagnosticReservedSession 'ledger'
    if (Test-Path -LiteralPath $diagnosticLedgerDir) {
      $diagnosticEvents = @(Get-ChildItem -LiteralPath $diagnosticLedgerDir -File |
        Sort-Object Name |
        ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json })
    }
    $diagnosticFinishedIds = @($diagnosticEvents | Where-Object type -eq 'attempt_finished' | ForEach-Object attemptId)
    $diagnosticSessionStartedCount = @($diagnosticEvents | Where-Object type -eq 'session_started').Count
    if ($diagnosticSessionStartedCount -gt 1) {
      throw 'Diagnostic start produced duplicate session_started events.'
    }
    $diagnosticAttemptStartedCount = @($diagnosticEvents | Where-Object type -eq 'attempt_started').Count
    $diagnosticUnfinished = @($diagnosticEvents | Where-Object {
      $_.type -eq 'attempt_started' -and $_.attemptId -notin $diagnosticFinishedIds
    })
    $diagnosticDecisionExists = Test-Path -LiteralPath (Join-Path $diagnosticReservedSession 'decision.json')
    $diagnosticAggregateExists = Test-Path -LiteralPath (Join-Path $diagnosticReservedSession 'session-ledger.json')
    if ($diagnosticDecisionExists -and -not $diagnosticAggregateExists) {
      throw 'Terminal decision commit marker exists without its aggregate peer.'
    }
    if ($diagnosticDecisionExists) {
      Write-Warning 'Coordinator returned nonzero after publishing both terminal projections; use the terminal audit branch and do not resume.'
    } else {
      Write-Warning "Coordinator halted with a locator-reserved projection/recovery gap (manifestMissing=$diagnosticManifestMissing; sessionStarted=$diagnosticSessionStartedCount; attemptsStarted=$diagnosticAttemptStartedCount; unfinished=$($diagnosticUnfinished.Count))."
      Write-Warning 'Run no child command until fresh quiescence is established; use the attested no-disposition recovery branch below.'
    }
  }
} finally {
  Pop-Location
}
```

Expected runtime is 60–120 minutes for one attempt. When an agent executes the
plan, run this as a yielded long-running process and poll it in at most
45-second waits, sending the user a short progress update at least every
60 seconds. The coordinator automatically runs A0/B/A1/C/A2/D/A3 and only the
predeclared E/A4 tail when `evaluateAttempt` returns `needs_e`.

Keep the machine awake and do not start builds, Rust Analyzer, Tauri, or Vite
during the attempt. If sleep, restart preparation, manual build activity, or
another infrastructure event is observed, interrupt the coordinator so the
append-only recovery path records `coordinator_interrupted`; do not let that
attempt become a valid terminal result.

After the command returns, resolve the immutable session path:

```powershell
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'
$diagnosticLocator = Join-Path $diagnosticScratchParent 'process-shell-diagnostic.locator.json'
$diagnosticSessionDir = (Get-Content -LiteralPath $diagnosticLocator -Raw | ConvertFrom-Json).sessionDir
Get-ChildItem -LiteralPath (Join-Path $diagnosticSessionDir 'ledger') |
    Sort-Object Name |
    Select-Object -Last 5 |
    ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw }
```

- [ ] **Step 7: Follow only the preregistered terminal/retry branch**

Apply exactly one applicable branch:

- If Step 6 returned nonzero and `decision.json` is absent, treat every
  locator-reserved state as a projection/recovery gap. This includes a missing
  manifest, no `session_started`, no first `attempt_started`, an unfinished
  attempt, a durable result not yet projected to `attempt_finished`, or a
  terminal ledger event whose aggregate/decision peer is missing. Run Step 5
  again before any other child command, then perform only attested recovery
  (no retry disposition yet):

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'
$diagnosticSessionDir = (Get-Content -LiteralPath (Join-Path $diagnosticScratchParent 'process-shell-diagnostic.locator.json') -Raw | ConvertFrom-Json).sessionDir
node "$diagnosticProtocolRoot\scripts\process-shell-diagnostic\coordinator.mjs" resume `
    --session-dir "$diagnosticSessionDir" `
    --process-attested
if ($LASTEXITCODE -ne 0) {
    throw 'Attested projection/recovery resume halted again; repeat quiescence before another resume.'
}
```

  This resume may only materialize missing bootstrap/projection peers, start
  the first reserved protocol attempt, recover an unfinished attempt after
  exact A restoration, or finish terminal projection. If it returns
  `awaiting_correction`, apply the corrected-cause branch next; it does not
  consume an unexplained-stability count by itself.
- If both `session-ledger.json` and its last-published `decision.json` commit
  marker exist, the session is terminal. Do not resume it. A decision without
  its aggregate peer is a protocol violation, not a terminal session.
- If status is `awaiting_stability_disposition` and no objective cause is
  identifiable, repeat the Step 5 quiescence checks and run the first retry:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'
$diagnosticSessionDir = (Get-Content -LiteralPath (Join-Path $diagnosticScratchParent 'process-shell-diagnostic.locator.json') -Raw | ConvertFrom-Json).sessionDir
node "$diagnosticProtocolRoot\scripts\process-shell-diagnostic\coordinator.mjs" resume `
    --session-dir "$diagnosticSessionDir" `
    --unexplained-stability `
    --process-attested
if ($LASTEXITCODE -ne 0) { throw 'First unexplained-stability resume failed.' }
```

- If a second attempt is stability-invalid with no objective cause, terminate
  without creating attempt 3:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'
$diagnosticSessionDir = (Get-Content -LiteralPath (Join-Path $diagnosticScratchParent 'process-shell-diagnostic.locator.json') -Raw | ConvertFrom-Json).sessionDir
node "$diagnosticProtocolRoot\scripts\process-shell-diagnostic\coordinator.mjs" resume `
    --session-dir "$diagnosticSessionDir" `
    --unexplained-stability
if ($LASTEXITCODE -ne 0) { throw 'Terminal precision resume failed.' }
```

  Expected: terminal `environment_precision_insufficient`, count 2, no causal
  B/C/D/E classification.
- If either stability or infrastructure invalidation has a concrete objective
  cause, first preserve its raw artifacts, correct that cause, restore the
  failed attempt worktree to A if its result lacks final-A evidence, repeat
  Step 5, and run:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'
$diagnosticSessionDir = (Get-Content -LiteralPath (Join-Path $diagnosticScratchParent 'process-shell-diagnostic.locator.json') -Raw | ConvertFrom-Json).sessionDir
$diagnosticObjectiveCauseAndCorrection = Read-Host 'Enter the exact observed objective cause and the verified correction'
if ([string]::IsNullOrWhiteSpace($diagnosticObjectiveCauseAndCorrection)) {
    throw 'A non-empty objective cause and correction record is required.'
}
node "$diagnosticProtocolRoot\scripts\process-shell-diagnostic\coordinator.mjs" resume `
    --session-dir "$diagnosticSessionDir" `
    --corrected-cause "$diagnosticObjectiveCauseAndCorrection" `
    --process-attested
if ($LASTEXITCODE -ne 0) { throw 'Corrected-cause resume failed.' }
```

  `$diagnosticObjectiveCauseAndCorrection` must contain the observed cause and
  verifiable correction, not a generic retry justification, and must not
  contain credentials or other secrets. Infrastructure invalidations without
  this record remain `awaiting_correction`.

No branch may edit a frozen file, alter the sequence, replace a sample, relax
300/500 ms thresholds, add warm-ups, or reuse an invalid attempt target.

### Task 8: Audit Restoration, Recalculate, Verify, and Commit the Record

**Files:**

- Create:
  `docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md`
- Read only: terminal session artifacts and every preserved attempt worktree
- Do not modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`

**Interfaces:**

- Consumes a terminal `decision.json`, `session-ledger.json`, immutable raw
  artifacts, and preserved worktrees.
- Produces a deterministic report whose arithmetic has already been
  independently checked, plus completion-gate evidence. No roadmap or product
  source change is part of this task.

- [ ] **Step 1: Reverify the protocol and audit every live tree before reporting**

Run from the protocol worktree:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticMainRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum').Path
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'
$diagnosticSessionDir = (Get-Content -LiteralPath (Join-Path $diagnosticScratchParent 'process-shell-diagnostic.locator.json') -Raw | ConvertFrom-Json).sessionDir
$diagnosticVerification = Join-Path $diagnosticProtocolRoot 'docs\superpowers\verification\2026-07-18-process-shell-regression-diagnostic.md'
node "$diagnosticProtocolRoot\scripts\process-shell-diagnostic\report.mjs" `
    --verify-only `
    --session-dir "$diagnosticSessionDir" `
    --output "$diagnosticVerification"
if ($LASTEXITCODE -ne 0) { throw 'Crash-replay-safe frozen protocol re-verification failed.' }

$diagnosticDecision = Get-Content -LiteralPath (Join-Path $diagnosticSessionDir 'decision.json') -Raw | ConvertFrom-Json
$diagnosticLedger = Get-Content -LiteralPath (Join-Path $diagnosticSessionDir 'session-ledger.json') -Raw | ConvertFrom-Json
$diagnosticExpectedATree = 'fd9711a041432ef420e7b09d56a46131a2a52a2a'

foreach ($diagnosticAttempt in $diagnosticLedger.attempts) {
    if (-not (Test-Path -LiteralPath $diagnosticAttempt.worktree)) {
        $diagnosticWorktreeCreated = @(
            $diagnosticLedger.events | Where-Object {
                $_.type -eq 'worktree_created' -and
                $_.attemptId -eq $diagnosticAttempt.attemptId
            }
        ).Count -gt 0
        $diagnosticFailureEvidence = $null
        if (Test-Path -LiteralPath $diagnosticAttempt.resultPath) {
            $diagnosticFailureEvidence = Get-Content -LiteralPath $diagnosticAttempt.resultPath -Raw | ConvertFrom-Json
        }
        $diagnosticMissingPathExplained =
            -not $diagnosticWorktreeCreated -and
            $diagnosticAttempt.status -eq 'infrastructure_invalid' -and
            @($diagnosticFailureEvidence.reasons | Where-Object {
                $_ -in @('coordinator_failure', 'coordinator_interrupted')
            }).Count -gt 0
        if ($diagnosticMissingPathExplained) {
            continue
        }
        throw "Attempt worktree is missing without valid pre-creation failure evidence: $($diagnosticAttempt.worktree)"
    }
    git -C "$($diagnosticAttempt.worktree)" diff --quiet
    if ($LASTEXITCODE -ne 0) { throw "Unstaged attempt bytes remain: $($diagnosticAttempt.attemptId)" }
    git -C "$($diagnosticAttempt.worktree)" diff --cached --quiet HEAD
    if ($LASTEXITCODE -ne 0) { throw "Staged attempt bytes remain: $($diagnosticAttempt.attemptId)" }
    $diagnosticAttemptStatus = @(git -C "$($diagnosticAttempt.worktree)" status --porcelain=v1 --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw "Could not inspect attempt status: $($diagnosticAttempt.attemptId)" }
    if ($diagnosticAttemptStatus.Count -ne 0) {
        throw "Attempt has tracked, staged, or untracked changes: $($diagnosticAttempt.attemptId)"
    }
    $diagnosticAttemptRootTree = (git -C "$($diagnosticAttempt.worktree)" write-tree).Trim()
    if ($LASTEXITCODE -ne 0) { throw "Could not write attempt index tree: $($diagnosticAttempt.attemptId)" }
    $diagnosticAttemptTree = (git -C "$($diagnosticAttempt.worktree)" rev-parse "${diagnosticAttemptRootTree}:src-tauri").Trim()
    if ($LASTEXITCODE -ne 0) { throw "Could not resolve attempt src-tauri tree: $($diagnosticAttempt.attemptId)" }
    if ($diagnosticAttemptTree -ne $diagnosticExpectedATree) {
        throw "Attempt A tree mismatch: $($diagnosticAttempt.attemptId) $diagnosticAttemptTree"
    }
}

$diagnosticMainStatus = @(git -C "$diagnosticMainRoot" status --porcelain=v1 --untracked-files=all -- src-tauri)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect main src-tauri status.' }
if ($diagnosticMainStatus.Count -ne 0) {
    throw 'Main src-tauri changed during the experiment.'
}
$diagnosticMainTree = (git -C "$diagnosticMainRoot" rev-parse 'HEAD:src-tauri').Trim()
if ($LASTEXITCODE -ne 0) { throw 'Could not resolve main HEAD:src-tauri.' }
if ($diagnosticMainTree -ne $diagnosticExpectedATree) {
    throw 'Main HEAD no longer has the frozen A src-tauri tree.'
}
```

Expected: protocol pin still matches; every attempt index/worktree is exact A;
main source is unchanged. A missing worktree is allowed only when no durable
`worktree_created` event exists and the immutable attempt result proves a
coordinator failure/interruption before creation. Once `worktree_created` exists,
the path must exist and be auditable at exact A even for an invalid attempt. The report's pre-publication
full-content snapshot separately proves the main target unchanged; a root
directory timestamp is not accepted as evidence.

- [ ] **Step 2: Validate all evidence, then publish/recover the artifact index and verification record**

Resolve every path in this invocation. If output from an interrupted reporting
transaction exists, do not delete it: `report.mjs` accepts it only when its
bytes equal the freshly validated deterministic projection.

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticScratchParent = Join-Path $env:TEMP 'extractum-process-shell-sessions'
$diagnosticSessionDir = (Get-Content -LiteralPath (Join-Path $diagnosticScratchParent 'process-shell-diagnostic.locator.json') -Raw | ConvertFrom-Json).sessionDir
$diagnosticVerification = Join-Path $diagnosticProtocolRoot 'docs\superpowers\verification\2026-07-18-process-shell-regression-diagnostic.md'

node "$diagnosticProtocolRoot\scripts\process-shell-diagnostic\report.mjs" `
    --session-dir "$diagnosticSessionDir" `
    --output "$diagnosticVerification"
if ($LASTEXITCODE -ne 0) { throw 'Evidence validation/report publication failed.' }
```

Expected: `report.mjs` first verifies the external locator anchor and the exact
Git-blob protocol pin (allowing at most its own untracked output on crash
replay), reconstructs the aggregate ledger from contiguous numbered events,
selects exactly one first durable authoritative `attempt_environment`, loads a mandatory
raw result artifact for every attempt, and independently recalculates valid and
stability-invalid attempts. Only then does it publish the index and Markdown.
A rerun after a crash completes a missing peer only when every already-published
byte is identical; conflicting output is never overwritten.

- [ ] **Step 3: Inspect report completeness against the frozen decision row**

Run:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticVerification = Join-Path $diagnosticProtocolRoot 'docs\superpowers\verification\2026-07-18-process-shell-regression-diagnostic.md'
Get-Content -LiteralPath $diagnosticVerification
if (-not $?) { throw 'Could not read the generated verification document.' }
git -C "$diagnosticProtocolRoot" diff --check -- 'docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md'
if ($LASTEXITCODE -ne 0) { throw 'Generated verification diff check failed.' }
```

Confirm the generated document contains the exact protocol commit/blob/SHA,
environment and Defender result (including honest `Access denied` if present),
all attempts and invalidations, every seven-sample wall/Cargo series, no-op
durations, anchor range, local A references, deltas and percentages, E trigger,
shell-cap result, direct-rustc-edge truth table, timings hashes, descriptive
contrasts, independently replayed retry dispositions and corrected causes,
per-attempt error kinds, corrected environment deltas, artifact-index SHA,
actual elapsed timestamps, final classification, and its predeclared roadmap
consequence. For `environment_precision_insufficient`, confirm it explicitly
makes no causal B/C/D/E claim.

- [ ] **Step 4: Invoke verification-before-completion and run all completion gates**

Use `superpowers:verification-before-completion` before making any pass/complete
claim. Then run from the implementation worktree—not main and not any attempt
worktree:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
Push-Location -LiteralPath $diagnosticProtocolRoot
try {
    npm.cmd run test -- scripts/process-shell-diagnostic/protocol.test.ts scripts/process-shell-diagnostic/runtime.test.ts scripts/process-shell-diagnostic/git-state.test.ts scripts/process-shell-diagnostic/attempt.test.ts scripts/process-shell-diagnostic/coordinator.test.ts scripts/process-shell-diagnostic/report.test.ts
    if ($LASTEXITCODE -ne 0) { throw 'Focused diagnostic harness tests failed.' }
    npm.cmd run check:rustfmt
    if ($LASTEXITCODE -ne 0) { throw 'Rust formatting gate failed.' }
    cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
    if ($LASTEXITCODE -ne 0) { throw 'Workspace Rust check failed.' }
    cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
    if ($LASTEXITCODE -ne 0) { throw 'Workspace Rust tests failed.' }
    npm.cmd run verify
    if ($LASTEXITCODE -ne 0) { throw 'Full repository verification failed.' }
} finally {
    Pop-Location
}
```

Expected: all focused harness tests PASS; rustfmt PASS; workspace check/test
PASS; full repository verify PASS. A filtered Rust run with zero tests is not
completion evidence; the workspace test command must execute its normal suite.

- [ ] **Step 5: Commit only the generated verification record**

Run:

```powershell
$diagnosticProtocolRoot = (Resolve-Path -LiteralPath 'G:\Develop\Extractum\.worktrees\process-shell-diagnostic-implementation').Path
$diagnosticVerificationPath = 'docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md'
$diagnosticTrackedBeforeCommit = @(git -C "$diagnosticProtocolRoot" diff --name-only)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate verification changes.' }
$diagnosticUntrackedBeforeCommit = @(git -C "$diagnosticProtocolRoot" ls-files --others --exclude-standard)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate untracked verification files.' }
if ($diagnosticTrackedBeforeCommit.Count -ne 0) {
    throw "Unexpected tracked changes before verification commit: $($diagnosticTrackedBeforeCommit -join '; ')"
}
if ($diagnosticUntrackedBeforeCommit.Count -ne 1 -or $diagnosticUntrackedBeforeCommit[0] -ne $diagnosticVerificationPath) {
    throw "Unexpected untracked verification inventory: $($diagnosticUntrackedBeforeCommit -join '; ')"
}
git -C "$diagnosticProtocolRoot" diff --cached --quiet
if ($LASTEXITCODE -eq 1) { throw 'Verification commit began with staged changes.' }
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect staged verification changes.' }
git -C "$diagnosticProtocolRoot" diff --check
if ($LASTEXITCODE -ne 0) { throw 'Verification worktree diff check failed.' }
git -C "$diagnosticProtocolRoot" add -- docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md
if ($LASTEXITCODE -ne 0) { throw 'Could not stage the verification document.' }
$diagnosticVerificationStaged = @(git -C "$diagnosticProtocolRoot" diff --cached --name-only)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate staged verification changes.' }
if ($diagnosticVerificationStaged.Count -ne 1 -or $diagnosticVerificationStaged[0] -ne $diagnosticVerificationPath) {
    throw "Unexpected staged verification inventory: $($diagnosticVerificationStaged -join '; ')"
}
git -C "$diagnosticProtocolRoot" diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Staged verification diff check failed.' }
git -C "$diagnosticProtocolRoot" commit -m "docs: record process shell regression diagnostic"
if ($LASTEXITCODE -ne 0) { throw 'Verification commit failed.' }
$diagnosticPostCommitStatus = @(git -C "$diagnosticProtocolRoot" status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect status after verification commit.' }
if ($diagnosticPostCommitStatus.Count -ne 0) {
    throw "Verification commit left changes: $($diagnosticPostCommitStatus -join '; ')"
}
```

Expected: only the verification Markdown is committed; raw artifacts and all
attempt worktrees remain outside the commit and preserved for review; final
status is clean.

- [ ] **Step 6: Request review and preserve the roadmap boundary**

Use `superpowers:requesting-code-review` on the harness, lock, raw artifact
index, independent arithmetic, restoration evidence, and report. Do not edit
the frozen roadmap or start Phase 4. After review, use
`superpowers:finishing-a-development-branch` for the owner-selected merge
path; remove preserved clean attempt worktrees only after that review/decision.
