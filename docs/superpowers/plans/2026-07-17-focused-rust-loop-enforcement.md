# Focused Rust Loop Enforcement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforce the approved focused Rust development loop through repository instructions and a stable source contract without changing runtime behavior or Superpowers skills.

**Architecture:** `AGENTS.md` becomes the agent-neutral source of truth for the focused/package/completion command hierarchy. A Vitest source-contract test reads only the anchored policy block, normalizes Windows line endings, and protects machine-significant commands and decision rules. The crate roadmap is marked complete only after the contract and full repository gate pass.

**Tech Stack:** Markdown, TypeScript, Vitest, Cargo workspace commands, PowerShell on Windows

## Global Constraints

- Approved spec: `docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md` at or after commit `ad5f71d8`.
- Modify no repository or installed Superpowers skill.
- Modify no Cargo manifest, production Rust/TypeScript/Svelte source, `package.json`, or `scripts/verify.mjs`.
- Keep `npm.cmd run verify` and both `--workspace --all-targets` Rust gates unchanged.
- Focused commands must use `--manifest-path src-tauri/Cargo.toml` and explicit `-p <package>` selection.
- A filtered run that executes `0 tests` is not verification.
- Preserve the shared canonical `src-tauri/target`; add no target-directory override.
- Normalize CRLF/LF in the source contract and assert semantic fragments rather than complete prose or formatting.
- The first Rust check in a session may be a cold 39.7–55.6 second run; do not diagnose that expected warm-up as a loop failure by itself.
- Full workspace verification remains the end-of-slice correctness gate.

## Rust Verification Loops

**Affected Rust packages:** None; this slice changes repository guidance, one TypeScript source contract, roadmap status, and verification evidence only.

**Inner-loop check:** Not applicable because no Rust source or manifest changes.

**Inner-loop tests:** `npm.cmd run test -- src/lib/focused-rust-loop-contract.test.ts`

**Task checkpoint:** `npm.cmd run test`

**End-of-slice gates:** `npm.cmd run verify`, which runs the full frontend suite, Svelte check, rustfmt check, `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`, and `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets`.

---

### Task 1: Enforce the focused Rust loop in repository guidance

**Files:**
- Create: `src/lib/focused-rust-loop-contract.test.ts`
- Modify: `AGENTS.md:39-58`

**Interfaces:**
- Consumes: approved commands, failure rules, cold-start note, and deferred-integration rule from `docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md`
- Produces: the stable `<!-- focused-rust-loop -->` policy block consumed by `focused-rust-loop-contract.test.ts`

- [ ] **Step 1: Verify the clean approved baseline**

Run:

```powershell
git status --short
git merge-base --is-ancestor ad5f71d8 HEAD
if ($LASTEXITCODE -ne 0) { throw 'Approved focused-loop spec is not in HEAD ancestry' }
```

Expected: `git status --short` prints nothing and the ancestry check exits 0. If the worktree is not clean, stop and ask the user rather than touching unrelated changes.

- [ ] **Step 2: Write the failing source-contract test**

Create `src/lib/focused-rust-loop-contract.test.ts` with exactly:

```typescript
import { describe, expect, it } from "vitest";

import agentGuidanceRaw from "../../AGENTS.md?raw";

const agentGuidance = agentGuidanceRaw.replace(/\r\n/g, "\n");
const policyAnchor = "<!-- focused-rust-loop -->";
const policyStart = agentGuidance.indexOf(policyAnchor);
const nextHeading = policyStart < 0 ? -1 : agentGuidance.indexOf("\n## ", policyStart);
const focusedPolicy =
  policyStart < 0
    ? ""
    : agentGuidance.slice(policyStart, nextHeading < 0 ? undefined : nextHeading);

const focusedCheck =
  "cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets";
const focusedTest =
  "cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> -- --exact";
const packageCheckpoint =
  "cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets";
const workspaceCheck =
  "cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets";
const workspaceTest =
  "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets";

describe("focused Rust loop repository policy", () => {
  it("owns canonical focused package commands", () => {
    expect(focusedPolicy, "missing focused Rust loop policy anchor").not.toBe("");
    expect(focusedPolicy).toContain(focusedCheck);
    expect(focusedPolicy).toContain(focusedTest);
    expect(focusedPolicy).toContain(packageCheckpoint);
    expect(focusedPolicy).toContain("-p extractum");
    expect(focusedPolicy).toContain("src-tauri/target");
  });

  it("separates focused feedback from full completion gates", () => {
    expect(focusedPolicy, "missing focused Rust loop policy anchor").not.toBe("");
    expect(focusedPolicy).toContain("`0 tests` is not verification");
    expect(focusedPolicy).toContain(workspaceCheck);
    expect(focusedPolicy).toContain(workspaceTest);
    expect(focusedPolicy).toContain("npm.cmd run verify");
    expect(focusedPolicy).toContain("accelerators, not completion evidence");
  });

  it("documents plan shape, cold starts, and deferred integration feedback", () => {
    expect(focusedPolicy, "missing focused Rust loop policy anchor").not.toBe("");
    expect(focusedPolicy).toContain("`## Rust Verification Loops`");
    expect(focusedPolicy).toContain("first Rust check in a session may be cold and slower");
    expect(focusedPolicy).toContain("public cross-crate interface");
    expect(focusedPolicy).toContain("immediate dependent package");
  });
});
```

- [ ] **Step 3: Run the contract and verify RED**

Run:

```powershell
$artifactDir = Join-Path $env:TEMP 'extractum-focused-rust-loop'
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null
$redLog = Join-Path $artifactDir 'contract-red.log'
$redExitFile = Join-Path $artifactDir 'contract-red-exit.txt'
cmd.exe /d /c "npm.cmd run test -- src/lib/focused-rust-loop-contract.test.ts > `"$redLog`" 2>&1"
$redExit = $LASTEXITCODE
Set-Content -LiteralPath $redExitFile -Value $redExit -Encoding ascii
$output = Get-Content -LiteralPath $redLog -Raw
$output
if ($redExit -eq 0) { throw 'Focused-loop contract passed before the policy existed' }
if ($output -notmatch 'Tests\s+3 failed') { throw 'RED run did not execute all three failing contract tests' }
```

Expected: exit code is nonzero; all three named tests fail with `missing focused Rust loop policy anchor`. The literal output and exit code remain in `%TEMP%\extractum-focused-rust-loop` for the verification document. If the command passes or Vitest reports zero matched tests, stop and fix the RED setup before continuing.

- [ ] **Step 4: Replace the conflicting validation sentence and daily-loop block**

In `AGENTS.md`, replace the current Validation Rule:

```markdown
- When no Superpowers workflow is active, run `npm.cmd run check:rustfmt` and `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets` after Rust or Tauri backend changes.
```

with:

```markdown
- When no Superpowers workflow is active, use the focused Rust loop below during implementation and run the full completion gates before claiming Rust or Tauri backend work complete.
```

Then replace the complete block from `<!-- daily-development-loop -->` through the native-debugging bullet immediately before `## 5. Data Grid & Date Formatting` with:

```markdown
<!-- daily-development-loop -->
- Use the smallest relevant daily-loop command after a small change:
  - dirty frontend work: `npm.cmd run test:changed`;
  - the most recent linear checkpoint: `npm.cmd run test:changed:last`;
  - a known frontend source: `npm.cmd run test:related -- <forward-or-backslash-path>`;
  - focused root-package Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib <test-filter>`;
  - broad Svelte/TypeScript work: also run `npm.cmd run check`.
- Changed/related commands are accelerators, not merge gates. An empty or unexpectedly small selection requires an explicit test or a wider run; `npm.cmd run verify` remains the full gate.

<!-- focused-rust-loop -->
- Every implementation plan that changes Rust must include a `## Rust Verification Loops` section naming affected packages, narrow RED/GREEN tests, focused checks, package checkpoints, and end-of-slice workspace gates.
- After a small Rust change, use the owning package explicitly:
  - exact RED/GREEN test: `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> -- --exact`;
  - focused check: `cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
  - task checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`.
- Use `-p extractum` while code belongs to the application; use the extracted domain package after it moves. Check every directly affected package separately.
- A filtered Cargo run that reports `0 tests` is not verification. List tests first when the exact name is unknown, then run a non-empty selection.
- The first Rust check in a session may be cold and slower; this expected warm-up is not a loop violation or, by itself, evidence of a build problem.
- A focused check of an extracted package does not compile downstream consumers. When a public cross-crate interface changes, add a checkpoint for the immediate dependent package; unchanged internal work does not pay that cost after every edit.
- Focused checks are accelerators, not completion evidence. At the end of every Rust slice run:
  - `npm.cmd run check:rustfmt`;
  - `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`;
  - `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets`;
  - `npm.cmd run verify`.
- Every workspace member shares canonical `src-tauri/target`; do not create slice-specific `codex-*` targets for sequential work.
- For rare dependency-level native debugging, start clean, temporarily set both `[profile.dev] debug` and `[profile.dev.package."*"] debug` to `2`, set `CARGO_TARGET_DIR` to an absolute isolated native-debug directory, run `npm.cmd run tauri dev`, then restore the manifest. Never commit the temporary profile edit.
```

- [ ] **Step 5: Run the contract and verify GREEN with a non-empty inventory**

Run:

```powershell
$artifactDir = Join-Path $env:TEMP 'extractum-focused-rust-loop'
$log = Join-Path $artifactDir 'contract-green.log'
cmd.exe /d /c "npm.cmd run test -- src/lib/focused-rust-loop-contract.test.ts > `"$log`" 2>&1"
$exitCode = $LASTEXITCODE
$output = Get-Content -LiteralPath $log -Raw
$output
if ($exitCode -ne 0) { throw "Focused-loop contract failed with exit code $exitCode" }
if ($output -notmatch 'Tests\s+3 passed') { throw 'Focused-loop contract did not execute all three tests' }
```

Expected: exit code 0 and Vitest reports `3 passed`; the script rejects a zero-test false green.

- [ ] **Step 6: Run the complete frontend test inventory**

Run:

```powershell
npm.cmd run test
```

Expected: exit code 0. The inventory must include `src/lib/focused-rust-loop-contract.test.ts`; do not use a fixed repository-wide file/test count because unrelated mainline additions may change it.

- [ ] **Step 7: Review and commit the policy deliverable**

Run:

```powershell
git diff --check
git diff -- AGENTS.md
Get-Content -LiteralPath 'src/lib/focused-rust-loop-contract.test.ts'
git status --short
```

Expected: only `AGENTS.md` and `src/lib/focused-rust-loop-contract.test.ts` are changed for this task; the diff contains no Superpowers skill, Cargo, production source, package, or verify-script changes.

Commit:

```powershell
git add -- AGENTS.md src/lib/focused-rust-loop-contract.test.ts
git diff --cached --check
git commit -m "test: enforce focused Rust development loop"
```

### Task 2: Close the roadmap item and record full verification

**Files:**
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md:25-27,343`
- Create: `docs/superpowers/verification/2026-07-17-focused-rust-loop-enforcement.md`

**Interfaces:**
- Consumes: green `<!-- focused-rust-loop -->` policy and `focused-rust-loop-contract.test.ts` from Task 1
- Produces: completed roadmap status and durable RED/GREEN/full-gate evidence

- [ ] **Step 1: Verify Task 1 is committed and green**

Run:

```powershell
git status --short
npm.cmd run test -- src/lib/focused-rust-loop-contract.test.ts
```

Expected: the worktree is clean before Task 2 and the focused contract reports exactly three passing tests.

- [ ] **Step 2: Mark enforcement complete in the roadmap**

In the completed/in-flight list, replace:

```markdown
  requirements, and unchanged end-of-slice workspace gates. Approved.
```

with:

```markdown
  requirements, and unchanged end-of-slice workspace gates. Enforced through
  `AGENTS.md` and `src/lib/focused-rust-loop-contract.test.ts`.
```

In `Deferred Items and Triggers`, replace the focused-loop row with exactly:

```markdown
| Focused-loop metric respec | **Completed 2026-07-17**: branch (a)+(b), commands, thresholds, failure policy, and plan structure are enforced through `AGENTS.md` and `src/lib/focused-rust-loop-contract.test.ts` |
```

- [ ] **Step 3: Prove the roadmap no longer claims enforcement is pending**

Run:

```powershell
$roadmap = Get-Content -LiteralPath 'docs/superpowers/specs/2026-07-17-crate-roadmap.md' -Raw
if ($roadmap -match 'enforcement pending') { throw 'Focused-loop roadmap item is still pending' }
if ($roadmap -notmatch '\*\*Completed 2026-07-17\*\*') { throw 'Focused-loop roadmap completion marker is missing' }
if ($roadmap -notmatch 'focused-rust-loop-contract\.test\.ts') { throw 'Roadmap does not name the enforcement contract' }
```

Expected: exit code 0 with no output.

- [ ] **Step 4: Run the full frontend suite separately**

Run:

```powershell
$artifactDir = Join-Path $env:TEMP 'extractum-focused-rust-loop'
$fullLog = Join-Path $artifactDir 'frontend-full.log'
cmd.exe /d /c "npm.cmd run test > `"$fullLog`" 2>&1"
$exitCode = $LASTEXITCODE
$output = Get-Content -LiteralPath $fullLog -Raw
$output
if ($exitCode -ne 0) { throw "Full frontend suite failed with exit code $exitCode" }
if ($output -notmatch 'Test Files\s+\d+ passed' -or $output -notmatch 'Tests\s+\d+ passed') {
  throw 'Full frontend suite did not report a non-empty passing inventory'
}
```

Expected: exit code 0 and a non-empty inventory including the three focused-loop contract tests. The literal output remains in `%TEMP%\extractum-focused-rust-loop\frontend-full.log`.

- [ ] **Step 5: Run the canonical end-of-slice gate**

Run:

```powershell
$artifactDir = Join-Path $env:TEMP 'extractum-focused-rust-loop'
$verifyLog = Join-Path $artifactDir 'verify.log'
cmd.exe /d /c "npm.cmd run verify > `"$verifyLog`" 2>&1"
$exitCode = $LASTEXITCODE
$output = Get-Content -LiteralPath $verifyLog -Raw
$output
if ($exitCode -ne 0) { throw "Repository verification failed with exit code $exitCode" }
$requiredStages = @(
  '=== npm run test ===',
  '=== npm run check ===',
  '=== npm run check:rustfmt ===',
  '=== cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets ===',
  '=== cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets ===',
  '=== git diff HEAD --check ===',
  'All verification checks passed.'
)
foreach ($stage in $requiredStages) {
  if (-not $output.Contains($stage)) { throw "Verification log is missing stage: $stage" }
}
```

Expected: `All verification checks passed.` after every required stage. The literal output remains in `%TEMP%\extractum-focused-rust-loop\verify.log`; a failure or missing stage blocks completion.

- [ ] **Step 6: Record verification evidence from literal artifacts**

Read the saved artifacts before writing evidence:

```powershell
$artifactDir = Join-Path $env:TEMP 'extractum-focused-rust-loop'
Get-Content -LiteralPath (Join-Path $artifactDir 'contract-red-exit.txt')
Select-String -LiteralPath (Join-Path $artifactDir 'contract-red.log') -Pattern 'Test Files|Tests\s'
Select-String -LiteralPath (Join-Path $artifactDir 'contract-green.log') -Pattern 'Test Files|Tests\s'
Select-String -LiteralPath (Join-Path $artifactDir 'frontend-full.log') -Pattern 'Test Files|Tests\s'
Select-String -LiteralPath (Join-Path $artifactDir 'verify.log') -Pattern '^===|All verification checks passed\.'
```

Create `docs/superpowers/verification/2026-07-17-focused-rust-loop-enforcement.md` with `apply_patch`. Use these headings in order: `Purpose`, `RED Evidence`, `GREEN Evidence`, `Full Frontend Inventory`, `Completion Gate`, and `Scope`. Populate them only from the literal artifacts:

- record the RED command, its actual nonzero exit code, and the exact `Test Files`/`Tests` summary lines from `contract-red.log`;
- record the GREEN command and exact three-test summary from `contract-green.log`;
- record the exact full frontend `Test Files` and `Tests` counts from `frontend-full.log`;
- list every observed `=== ... ===` verify stage and quote the final success line from `verify.log`;
- state that runtime behavior, Cargo manifests, production source, `scripts/verify.mjs`, and Superpowers skills were unchanged;
- name `AGENTS.md` and `src/lib/focused-rust-loop-contract.test.ts` as the enforcement owners.

Do not write `PASS`, a count, or a stage that is absent from its literal artifact. Do not modify the approved specification while recording evidence.

- [ ] **Step 7: Run final focused evidence after the documentation edit**

Run:

```powershell
npm.cmd run test -- src/lib/focused-rust-loop-contract.test.ts
git diff --check
```

Expected: exactly three focused contract tests pass and `git diff --check` exits 0.

- [ ] **Step 8: Enforce the final scope allowlist**

Run:

```powershell
$allowed = @(
  'docs/superpowers/specs/2026-07-17-crate-roadmap.md',
  'docs/superpowers/verification/2026-07-17-focused-rust-loop-enforcement.md'
)
$changed = @(
  git diff --name-only HEAD
  git ls-files --others --exclude-standard
) | Where-Object { $_ } | Sort-Object -Unique
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
if ($unexpected.Count -ne 0) {
  throw "Unexpected Task 2 paths: $($unexpected -join ', ')"
}
$missing = @($allowed | Where-Object { $_ -notin $changed })
if ($missing.Count -ne 0) {
  throw "Missing Task 2 paths: $($missing -join ', ')"
}
```

Expected: exit code 0; Task 2 changes exactly the roadmap and verification document.

- [ ] **Step 9: Review and commit the completed enforcement slice**

Run:

```powershell
git diff -- docs/superpowers/specs/2026-07-17-crate-roadmap.md
Get-Content -LiteralPath 'docs/superpowers/verification/2026-07-17-focused-rust-loop-enforcement.md'
git status --short
```

Commit:

```powershell
git add -- docs/superpowers/specs/2026-07-17-crate-roadmap.md docs/superpowers/verification/2026-07-17-focused-rust-loop-enforcement.md
git diff --cached --check
git commit -m "docs: record focused Rust loop enforcement"
git status --short
```

Expected: both files are committed and the final `git status --short` prints nothing.
