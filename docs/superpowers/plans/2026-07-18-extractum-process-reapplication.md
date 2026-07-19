# Extractum Process Exact Reapplication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reapply the exact historical `extractum-process` candidate from `b364756c`, retain it only after current correctness and completion gates pass, and record fresh non-gating shell diagnostics for the Phase 4 baseline decision.

**Architecture:** Execute in a new workflow-owned Windows worktree. First freeze and commit a machine-readable 14-path Git-object manifest, then prove the current preimage matches the historical parent and replay the candidate without manual conflict resolution. Keep the exact Rust/contract patch in one code-only commit; collect measurements under `%TEMP%`; finally commit a separate verification/roadmap slice whose contract distinguishes a valid cumulative baseline from an invalid non-gating session.

**Tech Stack:** Git object database, Rust 2021/Cargo workspaces, PowerShell 5.1, TypeScript, Vitest 4, Markdown/JSON evidence.

## Global Constraints

- Use `superpowers:using-git-worktrees` to create a clean isolated worktree and branch. Do not execute from `main`, any preserved diagnostic worktree, or a worktree sharing the main copy's `src-tauri/target`.
- Historical candidate: `b364756c7b5768d644321afeaeb81ec04e2481a4`; parent: `306a9370c90fd008a3b3259f77f4f48349806d05`; historical revert: `c47372dcd2fa97d8fe05f01d26a0c4f9eb888c83`.
- The exact-candidate boundary is the frozen 14-path manifest below. Any preimage, postimage, mode, path, staged patch, or final committed-patch mismatch is material. Stop immediately; do not resolve conflicts, edit the candidate, or reinterpret it as Phase 3.
- Never use ancestry as identity evidence: `b364756c` is already an ancestor of current history through the historical candidate/revert sequence.
- Never use broad `git checkout ... -- src/lib`, PowerShell text pipelines for binary Git patches, `cargo fmt`, `cargo clean`, a custom `CARGO_TARGET_DIR`, or the v1 process-shell diagnostic harness.
- All Cargo commands before the final repository wrapper use `--locked`. Run Cargo commands sequentially; a target lock or `Access denied` is infrastructure contention, not a candidate failure.
- Preserve the exact candidate `Cargo.lock` blob `6368e32cd3a3853d4a7114ce256258e834bafdd4`. Recheck all 14 path states after `npm.cmd run verify` and the Tauri build, because those wrappers do not promise `--locked`.
- Do not repair the known weak workspace-dependency regex in historical `src/lib/process-crate-boundary-contract.test.ts`; changing blob `ec44db1923d1194ad4c6bb07bd2fc643b1f1414f` would create a new candidate.
- Phase 3 is Windows/MSVC-only. Preserve the non-Windows stub byte-for-byte, but do not install or invoke a Linux target as an acceptance gate.
- Fresh shell diagnostics are strictly non-gating: one discarded warm-up and five recorded samples before and after, using the same fixed inert suffix in `src-tauri/src/lib.rs` and `cargo check --manifest-path src-tauri/Cargo.toml --locked --workspace --all-targets`.
- A shell series is stable only when at least four of five samples have absolute deviation `<= 300 ms` from their own median. Either unstable series invalidates the complete session; do not perform a stability retry or marginal repeat.
- The fresh diagnostic session has zero retries. A command-start failure, timeout, unconfirmed process-tree termination, target-lock/`.cargo-lock` contention, `Access denied`/`Отказано в доступе`, quiet-window failure, or runner/artifact/restoration failure makes that series and the complete diagnostic session invalid; after confirmed child termination and exact source restoration, continue correctness gates without rejecting the candidate. If termination or restoration cannot be confirmed, stop all further child commands and use `source-recovery.bin` for manual recovery.
- Qualify the scratch runner before the zero-retry baseline: the existing descendant Job Object preflight plus one synthetic `self-test` cycle on a `%TEMP%` copy must pass before the baseline series starts. The initial environment/quiet preflight has its own stable one-shot claim before its scan; runner qualification remains outside the measured baseline, but none of their evidence may be selectively discarded. After a recorded, reviewed plan/runner correction, qualification restarts only against a newly committed implementation base in a fresh workflow-owned worktree and scratch session; a failed qualification consumes no baseline attempt and never evaluates the candidate.
- Fresh shell values, including a delta above `2,000 ms / 20%` or a valid post median above `15,000 ms`, cannot reject this exact Phase 3 candidate. Only correctness, completion, or identity failure can prevent retention.
- Only a complete valid baseline/post session may seed the roadmap ledger. If the session is invalid, Phase 3 may retain but Phase 4 remains blocked until a valid five-sample baseline exists.
- The historical plan, historical verification, both regression-diagnostic specs/plans/verifications/artifacts/worktrees, the v2 technical body, and `scripts/process-shell-diagnostic/**` are immutable.
- Do not start Phase 4 in this plan. Successful Phase 3 plus a valid post median authorizes a separate Phase 4 implementation plan; invalid diagnostics preserve its baseline prerequisite.
- In the startup smoke, loading/compiling/constructing the Job Object helper, atomic launch/assignment, process observation, and cleanup are infrastructure operations. They may only write `startup-smoke-infrastructure-failure.json` and stop without retaining or reverting. `completion-failure.json` is allowed only when `StartAssigned` returned an assigned application process, its exit code was readable during the five-second observation window, and cleanup was subsequently confirmed.
- Every PowerShell block is a fresh process and is self-contained. In the same invocation, execute `Set-StrictMode -Version Latest` immediately before each shown `Run` body (or immediately after a scratch script's `param` block), reload persisted variables, and explicitly inspect native exit codes. Do not depend on variables or `$ErrorActionPreference` from an earlier block; set `ErrorActionPreference = 'Continue'` locally while capturing native stderr.
- Every patch fingerprint is rendered with the canonical Git options `-O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/`. These flags are identity data, not presentation preferences; do not omit or reorder their semantic effect through local Git config.

---

## Frozen Historical Identity Manifest

The implementation commits the same data as JSON before touching candidate paths. `null` means the path is absent. Every non-null mode is `100644`.

- Parent commit tree: `9099643760f53c2f056ab289551ccd9d5cb71515`
- Candidate commit tree: `269161fb9aed218b614a3e36151a1e7a67324bd9`
- Parent `src-tauri` tree: `fd9711a041432ef420e7b09d56a46131a2a52a2a`
- Candidate `src-tauri` tree: `77e2d163ccc8bddf3ea051cb995909888cae9aba`
- Canonical no-renames binary patch Git-blob OID: `f93b19fcd2aa5c2abb868ffe27c17940dae90690`
- Canonical no-renames stable patch-id: `fb767db0e8d2a9c6e743da4446b1f4da2c43f775`

| Change | Path | Parent blob | Candidate blob |
| --- | --- | --- | --- |
| M | `src-tauri/Cargo.lock` | `f552ed4ad5d150d8a3214aaa189df75b51986227` | `6368e32cd3a3853d4a7114ce256258e834bafdd4` |
| M | `src-tauri/Cargo.toml` | `c7739a16b80e6427e15984efdbf3b7eac69f6c24` | `c2037473a1257dd33a8e5b5fe81905e77dad084a` |
| A | `src-tauri/crates/extractum-process/Cargo.toml` | `null` | `3e078647dc293d95f401e15b8842776fae003ddb` |
| A | `src-tauri/crates/extractum-process/src/child_process.rs` | `null` | `9599017ed2ad826bc73f8e72f084042eacd8b58a` |
| A | `src-tauri/crates/extractum-process/src/external_process.rs` | `null` | `3cf7f073923b513381df09b7443090a4a41adc11` |
| A | `src-tauri/crates/extractum-process/src/lib.rs` | `null` | `4f7819ef7d2773b735b5edc61e162e4e034efb66` |
| A | `src-tauri/crates/extractum-process/src/process_tree.rs` | `null` | `365283e9f8accf4db91feca73bd8437db3b08c50` |
| D | `src-tauri/src/child_process.rs` | `76d16d26356fe8fc8342143dac3d56314e080dcb` | `null` |
| D | `src-tauri/src/external_process.rs` | `5974b458aeafdd7e7b9c7fde8ee669a2598deac9` | `null` |
| M | `src-tauri/src/lib.rs` | `fc2aae39b42e6b2638be167546e9442d3cc3a1e8` | `d84b653870eda9378c0d490894801850a97db68d` |
| D | `src-tauri/src/process_tree.rs` | `3a24972ac47ced9954b184dd7b6cdd6b7088eac6` | `null` |
| M | `src/lib/external-process-lifecycle-contract.test.ts` | `5ef63ee7bb78cca9be64d51b9daf96c1565d4619` | `4c3eed3493cdd2e20a99252d4f80f386c2e0e681` |
| M | `src/lib/hidden-child-process-contract.test.ts` | `daf3ccbe70982ec6f9a7cb91273fee3668838991` | `13cd27f9e6cdd22559633d870730a6ebb50e9f6b` |
| A | `src/lib/process-crate-boundary-contract.test.ts` | `null` | `ec44db1923d1194ad4c6bb07bd2fc643b1f1414f` |

The blob/mode table proves each path state. The no-renames full-index binary patch OID and patch-id cover the complete hunk stream without relying on rename similarity or an ambiguous hunk count.

## File Map

### Identity slice

- Create: `docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json` — durable machine-readable copy of the frozen identity manifest.
- Create: `src/lib/process-crate-reapplication-identity-contract.test.ts` — freezes all manifest constants, paths, modes, blobs, and patch fingerprints.

### Exact candidate slice

- Replay exactly: `src-tauri/Cargo.lock`
- Replay exactly: `src-tauri/Cargo.toml`
- Replay exactly: `src-tauri/crates/extractum-process/Cargo.toml`
- Replay exactly: `src-tauri/crates/extractum-process/src/{lib,child_process,external_process,process_tree}.rs`
- Delete exactly: `src-tauri/src/{child_process,external_process,process_tree}.rs`
- Replay exactly: `src-tauri/src/lib.rs`
- Replay exactly: `src/lib/external-process-lifecycle-contract.test.ts`
- Replay exactly: `src/lib/hidden-child-process-contract.test.ts`
- Replay exactly: `src/lib/process-crate-boundary-contract.test.ts`

### Evidence slice

- Create: `docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md` — identity, environment, raw diagnostics, inventories, gates, and final decision.
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md` — Phase 3 outcome, cumulative ledger, and conditional Phase 4 prerequisite.
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts` — replaces the pending Phase 3 assertions with retained/ledger evidence assertions.

### Scratch only

- Create below `%TEMP%/extractum-process-reapplication-*`: environment, Job Object and runner qualification artifacts, stable atomic environment/baseline/post claims, synthetic source copy, source recovery bytes, raw logs, samples, validity summaries, inventories, consumer hashes, completion results, and report preview.

## Rust Verification Loops

Affected packages are the new owner `extractum-process` and its immediate dependent `extractum`. Every Cargo command is sequential and uses the canonical worktree-local `src-tauri/target`.

Narrow package RED before replay:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum-process --lib external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets -- --exact
```

Expected RED: nonzero with Cargo reporting that package `extractum-process` does not exist. A zero-test result is not RED evidence.

Narrow package GREEN after replay:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum-process --lib external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets -- --exact
```

Expected GREEN: exactly one selected test passes.

Focused package loop and checkpoints:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
cargo check --manifest-path src-tauri/Cargo.toml --locked -p extractum-process --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum-process --all-targets
cargo check --manifest-path src-tauri/Cargo.toml --locked -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum --lib youtube::process_runtime::
```

Expected: process check passes, exactly 20 process tests pass, dependent check passes, and the YouTube selection is non-empty and passes.

Matched application-shell probe before and after:

```text
source: src-tauri/src/lib.rs
edit: append one UTF-8 inert Rust line comment, then restore exact bytes
command: cargo check --manifest-path src-tauri/Cargo.toml --locked --workspace --all-targets
```

Fresh focused performance is not rerun. The owner accepted the historical focused improvement `9,171 -> 2,049 ms` (`7,122 ms / 77.66%`) and historical shell change `9,135 -> 10,177 ms` (`1,042 ms / 11.4067%`) under the current `2,000 ms / 20%` cap. Fresh shell diagnostics use five samples with `4/5` within `<= 300 ms`; they are written under `%TEMP%` and cannot select a negative performance path. A valid post median is compared with the absolute `15,000 ms` roadmap ceiling; an invalid session leaves the ledger pending.

End-of-slice gates:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --locked --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --locked --workspace --all-targets
npm.cmd run verify
npm.cmd run tauri -- build --no-bundle
```

The release executable must then remain alive for five seconds in a hidden startup smoke and be stopped by PID with confirmed cleanup. MSI/WiX remains excluded because its failure predates this slice.

### Task 1: Freeze the candidate identity before reconstruction

**Files:**

- Create: `src/lib/process-crate-reapplication-identity-contract.test.ts`
- Create: `docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json`

**Interfaces:**

- Consumes: the frozen constants in this plan and historical Git objects.
- Produces: a committed JSON manifest read by all preimage/postimage checks.

- [ ] **Step 1: Write the identity-manifest RED contract**

Create `src/lib/process-crate-reapplication-identity-contract.test.ts` with this complete content:

```ts
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const manifestPath = path.join(
  process.cwd(),
  "docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json",
);
const manifest = existsSync(manifestPath)
  ? JSON.parse(readFileSync(manifestPath, "utf8"))
  : null;

const expectedEntries = [
  ["M", "src-tauri/Cargo.lock", "f552ed4ad5d150d8a3214aaa189df75b51986227", "6368e32cd3a3853d4a7114ce256258e834bafdd4"],
  ["M", "src-tauri/Cargo.toml", "c7739a16b80e6427e15984efdbf3b7eac69f6c24", "c2037473a1257dd33a8e5b5fe81905e77dad084a"],
  ["A", "src-tauri/crates/extractum-process/Cargo.toml", null, "3e078647dc293d95f401e15b8842776fae003ddb"],
  ["A", "src-tauri/crates/extractum-process/src/child_process.rs", null, "9599017ed2ad826bc73f8e72f084042eacd8b58a"],
  ["A", "src-tauri/crates/extractum-process/src/external_process.rs", null, "3cf7f073923b513381df09b7443090a4a41adc11"],
  ["A", "src-tauri/crates/extractum-process/src/lib.rs", null, "4f7819ef7d2773b735b5edc61e162e4e034efb66"],
  ["A", "src-tauri/crates/extractum-process/src/process_tree.rs", null, "365283e9f8accf4db91feca73bd8437db3b08c50"],
  ["D", "src-tauri/src/child_process.rs", "76d16d26356fe8fc8342143dac3d56314e080dcb", null],
  ["D", "src-tauri/src/external_process.rs", "5974b458aeafdd7e7b9c7fde8ee669a2598deac9", null],
  ["M", "src-tauri/src/lib.rs", "fc2aae39b42e6b2638be167546e9442d3cc3a1e8", "d84b653870eda9378c0d490894801850a97db68d"],
  ["D", "src-tauri/src/process_tree.rs", "3a24972ac47ced9954b184dd7b6cdd6b7088eac6", null],
  ["M", "src/lib/external-process-lifecycle-contract.test.ts", "5ef63ee7bb78cca9be64d51b9daf96c1565d4619", "4c3eed3493cdd2e20a99252d4f80f386c2e0e681"],
  ["M", "src/lib/hidden-child-process-contract.test.ts", "daf3ccbe70982ec6f9a7cb91273fee3668838991", "13cd27f9e6cdd22559633d870730a6ebb50e9f6b"],
  ["A", "src/lib/process-crate-boundary-contract.test.ts", null, "ec44db1923d1194ad4c6bb07bd2fc643b1f1414f"],
].map(([change, entryPath, parentBlob, candidateBlob]) => ({
  change,
  path: entryPath,
  parent_mode: parentBlob === null ? null : "100644",
  parent_blob: parentBlob,
  candidate_mode: candidateBlob === null ? null : "100644",
  candidate_blob: candidateBlob,
}));

describe("extractum-process exact reapplication identity", () => {
  it("freezes the historical candidate and every no-renames path state", () => {
    expect(manifest).toEqual({
      schema_version: 1,
      historical_candidate: "b364756c7b5768d644321afeaeb81ec04e2481a4",
      historical_parent: "306a9370c90fd008a3b3259f77f4f48349806d05",
      historical_revert: "c47372dcd2fa97d8fe05f01d26a0c4f9eb888c83",
      parent_tree: "9099643760f53c2f056ab289551ccd9d5cb71515",
      candidate_tree: "269161fb9aed218b614a3e36151a1e7a67324bd9",
      parent_src_tauri_tree: "fd9711a041432ef420e7b09d56a46131a2a52a2a",
      candidate_src_tauri_tree: "77e2d163ccc8bddf3ea051cb995909888cae9aba",
      no_renames_patch_blob: "f93b19fcd2aa5c2abb868ffe27c17940dae90690",
      no_renames_patch_id: "fb767db0e8d2a9c6e743da4446b1f4da2c43f775",
      entries: expectedEntries,
    });
    expect(new Set(expectedEntries.map((entry) => entry.path)).size).toBe(14);
  });
});
```

- [ ] **Step 2: Run the contract and verify RED**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
npm.cmd run test -- src/lib/process-crate-reapplication-identity-contract.test.ts
```

Expected: one test is collected and fails because `manifest` is `null`. An import or TypeScript failure is not the intended RED.

- [ ] **Step 3: Add the durable JSON manifest**

Create `docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json` with exactly:

```json
{
  "schema_version": 1,
  "historical_candidate": "b364756c7b5768d644321afeaeb81ec04e2481a4",
  "historical_parent": "306a9370c90fd008a3b3259f77f4f48349806d05",
  "historical_revert": "c47372dcd2fa97d8fe05f01d26a0c4f9eb888c83",
  "parent_tree": "9099643760f53c2f056ab289551ccd9d5cb71515",
  "candidate_tree": "269161fb9aed218b614a3e36151a1e7a67324bd9",
  "parent_src_tauri_tree": "fd9711a041432ef420e7b09d56a46131a2a52a2a",
  "candidate_src_tauri_tree": "77e2d163ccc8bddf3ea051cb995909888cae9aba",
  "no_renames_patch_blob": "f93b19fcd2aa5c2abb868ffe27c17940dae90690",
  "no_renames_patch_id": "fb767db0e8d2a9c6e743da4446b1f4da2c43f775",
  "entries": [
    {
      "change": "M",
      "path": "src-tauri/Cargo.lock",
      "parent_mode": "100644",
      "parent_blob": "f552ed4ad5d150d8a3214aaa189df75b51986227",
      "candidate_mode": "100644",
      "candidate_blob": "6368e32cd3a3853d4a7114ce256258e834bafdd4"
    },
    {
      "change": "M",
      "path": "src-tauri/Cargo.toml",
      "parent_mode": "100644",
      "parent_blob": "c7739a16b80e6427e15984efdbf3b7eac69f6c24",
      "candidate_mode": "100644",
      "candidate_blob": "c2037473a1257dd33a8e5b5fe81905e77dad084a"
    },
    {
      "change": "A",
      "path": "src-tauri/crates/extractum-process/Cargo.toml",
      "parent_mode": null,
      "parent_blob": null,
      "candidate_mode": "100644",
      "candidate_blob": "3e078647dc293d95f401e15b8842776fae003ddb"
    },
    {
      "change": "A",
      "path": "src-tauri/crates/extractum-process/src/child_process.rs",
      "parent_mode": null,
      "parent_blob": null,
      "candidate_mode": "100644",
      "candidate_blob": "9599017ed2ad826bc73f8e72f084042eacd8b58a"
    },
    {
      "change": "A",
      "path": "src-tauri/crates/extractum-process/src/external_process.rs",
      "parent_mode": null,
      "parent_blob": null,
      "candidate_mode": "100644",
      "candidate_blob": "3cf7f073923b513381df09b7443090a4a41adc11"
    },
    {
      "change": "A",
      "path": "src-tauri/crates/extractum-process/src/lib.rs",
      "parent_mode": null,
      "parent_blob": null,
      "candidate_mode": "100644",
      "candidate_blob": "4f7819ef7d2773b735b5edc61e162e4e034efb66"
    },
    {
      "change": "A",
      "path": "src-tauri/crates/extractum-process/src/process_tree.rs",
      "parent_mode": null,
      "parent_blob": null,
      "candidate_mode": "100644",
      "candidate_blob": "365283e9f8accf4db91feca73bd8437db3b08c50"
    },
    {
      "change": "D",
      "path": "src-tauri/src/child_process.rs",
      "parent_mode": "100644",
      "parent_blob": "76d16d26356fe8fc8342143dac3d56314e080dcb",
      "candidate_mode": null,
      "candidate_blob": null
    },
    {
      "change": "D",
      "path": "src-tauri/src/external_process.rs",
      "parent_mode": "100644",
      "parent_blob": "5974b458aeafdd7e7b9c7fde8ee669a2598deac9",
      "candidate_mode": null,
      "candidate_blob": null
    },
    {
      "change": "M",
      "path": "src-tauri/src/lib.rs",
      "parent_mode": "100644",
      "parent_blob": "fc2aae39b42e6b2638be167546e9442d3cc3a1e8",
      "candidate_mode": "100644",
      "candidate_blob": "d84b653870eda9378c0d490894801850a97db68d"
    },
    {
      "change": "D",
      "path": "src-tauri/src/process_tree.rs",
      "parent_mode": "100644",
      "parent_blob": "3a24972ac47ced9954b184dd7b6cdd6b7088eac6",
      "candidate_mode": null,
      "candidate_blob": null
    },
    {
      "change": "M",
      "path": "src/lib/external-process-lifecycle-contract.test.ts",
      "parent_mode": "100644",
      "parent_blob": "5ef63ee7bb78cca9be64d51b9daf96c1565d4619",
      "candidate_mode": "100644",
      "candidate_blob": "4c3eed3493cdd2e20a99252d4f80f386c2e0e681"
    },
    {
      "change": "M",
      "path": "src/lib/hidden-child-process-contract.test.ts",
      "parent_mode": "100644",
      "parent_blob": "daf3ccbe70982ec6f9a7cb91273fee3668838991",
      "candidate_mode": "100644",
      "candidate_blob": "13cd27f9e6cdd22559633d870730a6ebb50e9f6b"
    },
    {
      "change": "A",
      "path": "src/lib/process-crate-boundary-contract.test.ts",
      "parent_mode": null,
      "parent_blob": null,
      "candidate_mode": "100644",
      "candidate_blob": "ec44db1923d1194ad4c6bb07bd2fc643b1f1414f"
    }
  ]
}
```

- [ ] **Step 4: Run GREEN and repository checking**

Run sequentially:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
npm.cmd run test -- src/lib/process-crate-reapplication-identity-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Identity contract failed.' }
npm.cmd run check
if ($LASTEXITCODE -ne 0) { throw 'TypeScript/Svelte check failed.' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Identity slice whitespace check failed.' }
```

Expected: one identity test passes, repository checking passes, and no whitespace error is reported.

- [ ] **Step 5: Commit only the durable identity slice**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
git status --short
git add -- docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json src/lib/process-crate-reapplication-identity-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Could not stage identity files.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Staged identity whitespace check failed.' }
$expectedIdentityPaths = @(
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json'
  'src/lib/process-crate-reapplication-identity-contract.test.ts'
) | Sort-Object
$actualIdentityPaths = @(git diff --cached --name-only | Sort-Object)
if (@(Compare-Object $expectedIdentityPaths $actualIdentityPaths).Count -ne 0) {
  throw 'Staged identity inventory mismatch.'
}
git commit -m "test: freeze exact process candidate identity"
if ($LASTEXITCODE -ne 0) { throw 'Identity commit failed.' }
```

Expected staged inventory is exactly the two files named above and the worktree is clean after commit.

### Task 2: Qualify the historical preimage and record the fresh baseline

**Files:**

- Read: `docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json`
- Read: current baseline Rust sources and tests
- Create outside repository: `%TEMP%/extractum-process-reapplication-*`

**Interfaces:**

- Consumes: Task 1's committed manifest and a clean current baseline.
- Produces: preimage proof, environment evidence, fresh baseline inventory, 12 consumer hashes, and the non-gating baseline shell series used by Task 4.

- [ ] **Step 1: Require an isolated clean Windows worktree and exclusive target**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRaw = @(git rev-parse --show-toplevel)
$repoCode = $LASTEXITCODE
if ($repoCode -ne 0 -or $repoRaw.Count -ne 1) { throw 'Not inside the repository.' }
$repo = ([string]$repoRaw[0]).Trim()
$repoFull = [IO.Path]::GetFullPath($repo).TrimEnd('\')
$targetFull = [IO.Path]::GetFullPath(
  (Join-Path $repoFull 'src-tauri/target')).TrimEnd('\')
$branchRaw = @(git branch --show-current)
$branchCode = $LASTEXITCODE
if ($branchCode -ne 0 -or $branchRaw.Count -ne 1) {
  throw 'Could not identify the current branch.'
}
$branch = ([string]$branchRaw[0]).Trim()
if ([string]::IsNullOrWhiteSpace($branch) -or $branch -eq 'main') {
  throw 'Run the plan on a named isolated feature branch, not main.'
}
$status = @(git status --porcelain=v1)
if ($LASTEXITCODE -ne 0 -or $status.Count -ne 0) {
  throw "Starting worktree is not clean: $($status -join '; ')"
}
if (-not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
  throw 'CARGO_TARGET_DIR must be unset.'
}
$hostTriple = (rustc -vV | Select-String '^host:' | ForEach-Object {
  $_.Line.Substring(5).Trim()
})
if ($hostTriple -ne 'x86_64-pc-windows-msvc') {
  throw "Phase 3 requires x86_64-pc-windows-msvc, got $hostTriple"
}
$identityRaw = @(git rev-parse HEAD)
$identityCode = $LASTEXITCODE
$baseRaw = @(git rev-parse 'HEAD^')
$baseCode = $LASTEXITCODE
if ($identityCode -ne 0 -or $baseCode -ne 0 -or
    $identityRaw.Count -ne 1 -or $baseRaw.Count -ne 1) {
  throw 'Could not resolve the identity commit and implementation base.'
}
$identityCommit = ([string]$identityRaw[0]).Trim()
$implementationBase = ([string]$baseRaw[0]).Trim()
$sessionId = '{0}-{1}' -f `
  ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')), `
  ([guid]::NewGuid().ToString('N'))
$scratch = Join-Path $env:TEMP "extractum-process-reapplication-$sessionId"
New-Item -ItemType Directory -Path $scratch | Out-Null
New-Item -ItemType Directory -Path (Join-Path $scratch 'attempts') | Out-Null

$quietScript = Join-Path $scratch 'assert-quiet-window.ps1'
@'
param([Parameter(Mandatory = $true)][string]$ArtifactPath)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$all = @(Get-CimInstance Win32_Process -ErrorAction Stop)
$blocking = @($all | Where-Object {
  if ($_.ProcessId -eq $PID) { return $false }
  $name = [string]$_.Name
  $command = [string]$_.CommandLine
  $nativeBuild = $name -match '^(cargo.*|rustc|rust-analyzer|extractum|tauri|vite)\.exe$'
  $nodeBuild = $name -match '^(node|npm|npx)(\.exe|\.cmd)?$' -and
    $command -match '(?i)(vite|tauri|svelte-kit|npm(?:\.cmd)?\s+run\s+(verify|check|build|test)|cargo)'
  $nativeBuild -or $nodeBuild
})
[ordered]@{
  checked_at = [DateTimeOffset]::Now.ToString('o')
  cim_available = $true
  blocking_count = $blocking.Count
  blocking = @($blocking | ForEach-Object {
    [ordered]@{
      process_id = $_.ProcessId
      name = $_.Name
      command_line = $_.CommandLine
    }
  })
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $ArtifactPath
if ($blocking.Count -ne 0) {
  throw "Quiet window is not exclusive; see $ArtifactPath"
}
'@ | Set-Content -LiteralPath $quietScript
$gitCommonRaw = @(git rev-parse --git-common-dir)
if ($LASTEXITCODE -ne 0 -or $gitCommonRaw.Count -ne 1) {
  throw 'Could not resolve the shared Git directory before environment claim.'
}
$gitCommon = (Resolve-Path -LiteralPath ([string]$gitCommonRaw[0])).Path
$claimMaterial = '{0}|{1}' -f $gitCommon.ToLowerInvariant(), `
  $implementationBase
$claimHasher = [Security.Cryptography.SHA256]::Create()
try {
  $claimKey = -join ($claimHasher.ComputeHash(
    [Text.Encoding]::UTF8.GetBytes($claimMaterial)) | ForEach-Object {
      $_.ToString('x2')
    })
} finally { $claimHasher.Dispose() }
$claimRoot = Join-Path $env:TEMP `
  "extractum-process-reapplication-claims/$claimKey"
New-Item -ItemType Directory -Path $claimRoot -Force | Out-Null
$environmentClaimPath = Join-Path $claimRoot 'environment-preflight.json'
if (Test-Path -LiteralPath $environmentClaimPath) {
  throw "Environment preflight is already claimed at $environmentClaimPath; never launch the initial quiet-window again for this implementation base."
}
$environmentClaimPayload = [ordered]@{
  stage = 'environment-preflight'
  claimed_at = [DateTimeOffset]::Now.ToString('o')
  scratch = $scratch
  repository = $repoFull
  target = $targetFull
  identity_commit = $identityCommit
  implementation_base = $implementationBase
  quiet_artifact = (Join-Path $scratch 'quiet-initial.json')
} | ConvertTo-Json
$environmentClaimBytes =
  [Text.UTF8Encoding]::new($false).GetBytes($environmentClaimPayload)
$environmentClaimTempPath = '{0}.{1}.tmp' -f `
  $environmentClaimPath, ([guid]::NewGuid().ToString('N'))
$environmentClaimStream = $null
try {
  $environmentClaimStream = [IO.File]::Open(
    $environmentClaimTempPath,
    [IO.FileMode]::CreateNew,
    [IO.FileAccess]::Write,
    [IO.FileShare]::None)
  $environmentClaimStream.Write(
    $environmentClaimBytes, 0, $environmentClaimBytes.Length)
  $environmentClaimStream.Flush($true)
} catch [IO.IOException] {
  throw "Could not atomically claim the environment preflight: $($_.Exception.Message)"
} finally {
  if ($null -ne $environmentClaimStream) {
    $environmentClaimStream.Dispose()
  }
}
try {
  [IO.File]::Move($environmentClaimTempPath, $environmentClaimPath)
} catch [IO.IOException] {
  throw "Could not atomically publish the environment preflight claim: $($_.Exception.Message)"
}
$scratch | Set-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt')
$claimRoot | Set-Content -LiteralPath `
  (Join-Path $scratch 'diagnostic-claim-root.txt')
$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$initialQuietStartError = $null
try {
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $quietScript `
    -ArtifactPath (Join-Path $scratch 'quiet-initial.json') `
    1> (Join-Path $scratch 'quiet-initial.stdout.log') `
    2> (Join-Path $scratch 'quiet-initial.stderr.log')
  $initialQuietCode = $LASTEXITCODE
} catch {
  $initialQuietCode = -1
  $initialQuietStartError = $_.Exception.Message
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
$initialQuietValid = ($initialQuietCode -eq 0)
if (-not $initialQuietValid -and
    -not (Test-Path -LiteralPath (Join-Path $scratch 'quiet-initial.json'))) {
  [ordered]@{
    checked_at = [DateTimeOffset]::Now.ToString('o')
    cim_available = $false
    blocking_count = $null
    error = if ($null -ne $initialQuietStartError) {
      "quiet-window subprocess start failed: $initialQuietStartError"
    } else { 'quiet-window subprocess failed before writing its normal artifact' }
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'quiet-initial.json')
}

$mainWorktree = $null
$currentWorktree = $null
foreach ($line in @(git worktree list --porcelain)) {
  if ($line -like 'worktree *') { $currentWorktree = $line.Substring(9) }
  elseif ($line -eq 'branch refs/heads/main') { $mainWorktree = $currentWorktree }
}
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate Git worktrees.' }
if ([string]::IsNullOrWhiteSpace($mainWorktree)) {
  throw 'Could not identify the main worktree.'
}
$mainFull = [IO.Path]::GetFullPath($mainWorktree).TrimEnd('\')
if ($repoFull -eq $mainFull) { throw 'Measurement worktree is the main worktree.' }
$power = try {
  $powerRaw = (powercfg /getactivescheme | Out-String).Trim()
  if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($powerRaw)) {
    $powerRaw
  } else { 'unavailable' }
} catch { 'unavailable' }
$defender = try {
  $mp = Get-MpComputerStatus -ErrorAction Stop
  "real_time=$($mp.RealTimeProtectionEnabled); antivirus=$($mp.AntivirusEnabled)"
} catch { "unavailable: $($_.Exception.Message)" }
$rustcVersionRaw = @(rustc --version)
$rustcVersionCode = $LASTEXITCODE
$cargoVersionRaw = @(cargo --version)
$cargoVersionCode = $LASTEXITCODE
if ($rustcVersionCode -ne 0 -or $cargoVersionCode -ne 0 -or
    $rustcVersionRaw.Count -ne 1 -or $cargoVersionRaw.Count -ne 1) {
  throw 'Could not record rustc/cargo versions.'
}
$rustcVersion = ([string]$rustcVersionRaw[0]).Trim()
$cargoVersion = ([string]$cargoVersionRaw[0]).Trim()
[ordered]@{
  session_id = $sessionId
  repository = $repoFull
  main_worktree = $mainFull
  branch = $branch
  identity_commit = $identityCommit
  implementation_base = $implementationBase
  diagnostic_claim_root = $claimRoot
  environment_preflight_claim_sha256 =
    (Get-FileHash -LiteralPath $environmentClaimPath -Algorithm SHA256).Hash
  rustc = $rustcVersion
  cargo = $cargoVersion
  host = $hostTriple
  power = $power
  defender = $defender
  target = $targetFull
  main_target = (Join-Path $mainFull 'src-tauri/target')
  cargo_target_dir_environment = $null
  initial_quiet_valid = $initialQuietValid
  started_at = [DateTimeOffset]::Now.ToString('o')
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath `
  (Join-Path $scratch 'environment.json')
```

Expected: a clean non-main branch, Windows/MSVC host, a fresh scratch directory, and both the pre-identity `implementation_base` and committed `identity_commit`. Before the CIM scan, a stable `environment-preflight.json` is atomically claimed from canonical Git common directory plus `implementation_base`; an existing claim forbids a second initial quiet-window launch for the same approved plan base. The scan covers Cargo/Rust/Tauri/Vite/npm/node build activity. A failed/unavailable scan records `initial_quiet_valid=false`; it does not stop correctness work, but Step 5 must invalidate the zero-retry diagnostic session without starting its Cargo probe. If Cargo later reports `target/**/.cargo-lock: Access denied`, classify it as infrastructure contention; do not delete lock files or the target.

- [ ] **Step 2: Verify every historical object and current preimage**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$manifestPath = 'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json

function Get-RevisionEntry([string]$revision, [string]$entryPath) {
  $spec = '{0}:{1}' -f $revision, $entryPath
  $line = @(git ls-tree $revision -- $entryPath)
  if ($LASTEXITCODE -ne 0) { throw "git ls-tree failed for $spec" }
  if ($line.Count -eq 0) { return $null }
  if ($line.Count -ne 1 -or $line[0] -notmatch `
      '^(?<mode>[0-9]{6}) blob (?<blob>[0-9a-f]{40})\t') {
    throw "Unexpected tree entry for $spec"
  }
  [pscustomobject]@{ mode = $Matches.mode; blob = $Matches.blob }
}

function Get-GitSingleLine([string]$label, [string[]]$arguments) {
  $raw = @(& git @arguments)
  $code = $LASTEXITCODE
  if ($code -ne 0 -or $raw.Count -ne 1) {
    throw "Git failed or returned a non-singleton result for $label"
  }
  return ([string]$raw[0]).Trim()
}

if ((Get-GitSingleLine 'historical candidate type' `
      @('cat-file', '-t', [string]$manifest.historical_candidate)) -ne 'commit') {
  throw 'Historical candidate object is missing.'
}
if ((Get-GitSingleLine 'historical parent type' `
      @('cat-file', '-t', [string]$manifest.historical_parent)) -ne 'commit') {
  throw 'Historical parent object is missing.'
}
if ((Get-GitSingleLine 'historical revert type' `
      @('cat-file', '-t', [string]$manifest.historical_revert)) -ne 'commit') {
  throw 'Historical revert object is missing.'
}
if ((Get-GitSingleLine 'historical candidate parent' `
      @('rev-parse', "$($manifest.historical_candidate)^")) -ne
    $manifest.historical_parent) {
  throw 'Historical candidate parent link mismatch.'
}
if ((Get-GitSingleLine 'historical parent tree' `
      @('rev-parse', "$($manifest.historical_parent)^{tree}")) -ne `
    $manifest.parent_tree) { throw 'Historical parent tree mismatch.' }
if ((Get-GitSingleLine 'historical candidate tree' `
      @('rev-parse', "$($manifest.historical_candidate)^{tree}")) -ne `
    $manifest.candidate_tree) { throw 'Historical candidate tree mismatch.' }

foreach ($entry in $manifest.entries) {
  $parentEntry = Get-RevisionEntry $manifest.historical_parent $entry.path
  $candidateEntry = Get-RevisionEntry $manifest.historical_candidate $entry.path
  $headEntry = Get-RevisionEntry 'HEAD' $entry.path
  if ($null -eq $entry.parent_blob) {
    if ($null -ne $parentEntry -or $null -ne $headEntry) {
      throw "Expected absent parent/preimage path: $($entry.path)"
    }
  } else {
    if ($null -eq $parentEntry -or $null -eq $headEntry -or
        $parentEntry.mode -ne $entry.parent_mode -or
        $parentEntry.blob -ne $entry.parent_blob -or
        $headEntry.mode -ne $entry.parent_mode -or
        $headEntry.blob -ne $entry.parent_blob) {
      throw "Parent/preimage mismatch: $($entry.path)"
    }
  }
  if ($null -eq $entry.candidate_blob) {
    if ($null -ne $candidateEntry) {
      throw "Expected absent candidate path: $($entry.path)"
    }
  } elseif ($null -eq $candidateEntry -or
      $candidateEntry.mode -ne $entry.candidate_mode -or
      $candidateEntry.blob -ne $entry.candidate_blob) {
    throw "Candidate manifest mismatch: $($entry.path)"
  }
}

$headTauri = Get-GitSingleLine 'current src-tauri tree' `
  @('rev-parse', 'HEAD:src-tauri')
if ($headTauri -ne $manifest.parent_src_tauri_tree) {
  throw "Current src-tauri tree mismatch: $headTauri"
}
$candidateTauri = Get-GitSingleLine 'historical candidate src-tauri tree' `
  @('rev-parse', "$($manifest.historical_candidate):src-tauri")
if ($candidateTauri -ne $manifest.candidate_src_tauri_tree) {
  throw 'Historical candidate src-tauri tree mismatch.'
}

$patchBlobRaw = @(cmd.exe /d /c `
  "git diff -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ 306a9370c90fd008a3b3259f77f4f48349806d05 b364756c7b5768d644321afeaeb81ec04e2481a4 | git hash-object --stdin")
$patchBlobCode = $LASTEXITCODE
$patchBlob = if ($patchBlobRaw.Count -eq 1) {
  ([string]$patchBlobRaw[0]).Trim()
} else { $null }
if ($patchBlobCode -ne 0 -or $patchBlobRaw.Count -ne 1 -or
    $patchBlob -ne $manifest.no_renames_patch_blob) {
  throw "Historical binary patch mismatch: $patchBlob"
}
$patchIdRaw = @(cmd.exe /d /c `
  "git diff -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ 306a9370c90fd008a3b3259f77f4f48349806d05 b364756c7b5768d644321afeaeb81ec04e2481a4 | git patch-id --stable")
$patchIdCode = $LASTEXITCODE
$patchId = if ($patchIdRaw.Count -eq 1) {
  (([string]$patchIdRaw[0]).Trim() -split '\s+')[0]
} else { $null }
if ($patchIdCode -ne 0 -or $patchIdRaw.Count -ne 1 -or
    $patchId -ne $manifest.no_renames_patch_id) {
  throw "Historical patch-id mismatch: $patchId"
}
cmd.exe /d /c `
  "git show --format= -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ b364756c7b5768d644321afeaeb81ec04e2481a4 | git apply --cached --check"
if ($LASTEXITCODE -ne 0) {
  throw 'Historical patch does not apply cleanly to the current index.'
}
[ordered]@{
  checked_at = [DateTimeOffset]::Now.ToString('o')
  entries = $manifest.entries.Count
  head_src_tauri_tree = $headTauri
  candidate_src_tauri_tree = $candidateTauri
  patch_blob = $patchBlob
  patch_id = $patchId
  result = '14/14 preimages and historical postimages match'
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath `
  (Join-Path $scratch 'identity-preflight.json')
```

Expected: `14/14` entries, both subtree hashes, and both patch fingerprints match. Any failure closes the exact-candidate path before Rust changes.

- [ ] **Step 3: Capture RED, baseline tests, inventory, and consumers**

Run Cargo commands sequentially:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$metadataStdout = Join-Path $scratch 'baseline-metadata.stdout.log'
$metadataStderr = Join-Path $scratch 'baseline-metadata.stderr.log'
$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
  cargo metadata --manifest-path src-tauri/Cargo.toml --locked `
    --format-version 1 --no-deps 1> $metadataStdout 2> $metadataStderr
  $metadataCode = $LASTEXITCODE
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
if ($metadataCode -ne 0) { throw 'Locked baseline metadata failed.' }
$metadata = Get-Content -LiteralPath $metadataStdout -Raw | ConvertFrom-Json
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
$expectedWorkspace = [IO.Path]::GetFullPath((Join-Path (Get-Location) 'src-tauri')).TrimEnd('\')
$expectedTarget = [IO.Path]::GetFullPath((Join-Path $expectedWorkspace 'target')).TrimEnd('\')
$actualWorkspace = [IO.Path]::GetFullPath([string]$metadata.workspace_root).TrimEnd('\')
$actualTarget = [IO.Path]::GetFullPath([string]$metadata.target_directory).TrimEnd('\')
$mainTarget = [IO.Path]::GetFullPath([string]$environment.main_target).TrimEnd('\')
if ($actualWorkspace -ne $expectedWorkspace -or $actualTarget -ne $expectedTarget -or
    $actualTarget -eq $mainTarget -or
    -not $actualTarget.StartsWith($actualWorkspace + '\', [StringComparison]::OrdinalIgnoreCase)) {
  throw "Cargo target isolation failed: workspace=$actualWorkspace target=$actualTarget"
}
[ordered]@{
  workspace_root = $actualWorkspace
  target_directory = $actualTarget
  main_target = $mainTarget
  isolated = $true
} | ConvertTo-Json | Set-Content -LiteralPath `
  (Join-Path $scratch 'target-isolation.json')

$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
  $redOutput = @(& cargo test --manifest-path src-tauri/Cargo.toml --locked `
    -p extractum-process --lib `
    external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets `
    -- --exact 2>&1)
  $redCode = $LASTEXITCODE
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
$redOutput | Set-Content -LiteralPath (Join-Path $scratch 'narrow-red.log')
if ($redCode -eq 0 -or
    ($redOutput -join "`n") -notmatch 'package ID specification.*extractum-process') {
  throw 'Narrow RED did not fail for the absent package as expected.'
}

$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
  $characterizationOutput = @(& cargo test --manifest-path src-tauri/Cargo.toml `
    --locked -p extractum --lib `
    external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets `
    -- --exact 2>&1)
  $characterizationCode = $LASTEXITCODE
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
$characterizationOutput | Set-Content -LiteralPath `
  (Join-Path $scratch 'baseline-characterization.log')
if ($characterizationCode -ne 0 -or
    ($characterizationOutput -join "`n") -notmatch 'test result: ok\. 1 passed') {
  throw 'Baseline characterization did not report exactly one passing test.'
}
npm.cmd run test -- `
  src/lib/process-crate-reapplication-identity-contract.test.ts `
  src/lib/external-process-lifecycle-contract.test.ts `
  src/lib/hidden-child-process-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Baseline source contracts failed.' }

$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
  $workspaceInventory = @(& cargo test --manifest-path src-tauri/Cargo.toml `
    --locked --workspace --all-targets -- --list 2>&1)
  $workspaceInventoryCode = $LASTEXITCODE
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
if ($workspaceInventoryCode -ne 0) { throw 'Baseline workspace inventory failed.' }
$workspaceNamesRaw = @($workspaceInventory | Where-Object { $_ -match ': test$' } |
  ForEach-Object { ($_ -replace ': test$', '').Trim() })
$workspaceNames = @($workspaceNamesRaw | Sort-Object -Unique)
if ($workspaceNamesRaw.Count -ne $workspaceNames.Count) {
  throw 'Baseline inventory contains duplicate test names.'
}
$processNames = @($workspaceNames | Where-Object {
  $_ -match '^(external_process|child_process|process_tree)::'
})
if ($workspaceNames.Count -eq 0 -or $processNames.Count -ne 20) {
  throw "Unexpected baseline inventory: total=$($workspaceNames.Count) process=$($processNames.Count)"
}
$workspaceNames | Set-Content -LiteralPath `
  (Join-Path $scratch 'baseline-test-names.txt')
$workspaceNamesRaw | Set-Content -LiteralPath `
  (Join-Path $scratch 'baseline-test-names-raw.txt')
$processNames | Set-Content -LiteralPath `
  (Join-Path $scratch 'baseline-process-test-names.txt')

$consumerPaths = @(
  'src-tauri/src/diagnostics/runtime.rs'
  'src-tauri/src/gemini_browser/cdp_chrome.rs'
  'src-tauri/src/gemini_browser/commands.rs'
  'src-tauri/src/gemini_browser/sidecar.rs'
  'src-tauri/src/youtube/captions.rs'
  'src-tauri/src/youtube/comments.rs'
  'src-tauri/src/youtube/jobs.rs'
  'src-tauri/src/youtube/metadata.rs'
  'src-tauri/src/youtube/preview.rs'
  'src-tauri/src/youtube/process_runtime.rs'
  'src-tauri/src/youtube/runtime.rs'
  'src-tauri/src/youtube/ytdlp.rs'
)
@($consumerPaths | ForEach-Object {
  [ordered]@{
    path = $_
    sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash
  }
}) | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath `
  (Join-Path $scratch 'baseline-consumer-hashes.json')
```

Expected: intended package-absence RED, one passing baseline characterization test, three source-contract files passing, nonzero workspace inventory, exactly 20 baseline process tests, and 12 consumer hashes.

- [ ] **Step 4: Create the byte-restoring shell-series runner in scratch**

First create `$scratch/job-object.ps1` with this complete Windows Job Object helper. It makes descendant ownership independent of parent-PID lineage after intermediate processes exit:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($null -eq ('ExtractumOwnedJob' -as [type])) {
  Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

public sealed class ExtractumOwnedJob : IDisposable
{
    private IntPtr handle;

    public bool LastProcessCreated { get; private set; }
    public bool LastProcessAssigned { get; private set; }
    public bool LastLaunchTerminationConfirmed { get; private set; }
    public uint LastProcessId { get; private set; }

    [StructLayout(LayoutKind.Sequential)]
    private struct BasicLimitInformation
    {
        public long PerProcessUserTimeLimit;
        public long PerJobUserTimeLimit;
        public uint LimitFlags;
        public UIntPtr MinimumWorkingSetSize;
        public UIntPtr MaximumWorkingSetSize;
        public uint ActiveProcessLimit;
        public UIntPtr Affinity;
        public uint PriorityClass;
        public uint SchedulingClass;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct IoCounters
    {
        public ulong ReadOperationCount;
        public ulong WriteOperationCount;
        public ulong OtherOperationCount;
        public ulong ReadTransferCount;
        public ulong WriteTransferCount;
        public ulong OtherTransferCount;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct ExtendedLimitInformation
    {
        public BasicLimitInformation BasicLimitInformation;
        public IoCounters IoInfo;
        public UIntPtr ProcessMemoryLimit;
        public UIntPtr JobMemoryLimit;
        public UIntPtr PeakProcessMemoryUsed;
        public UIntPtr PeakJobMemoryUsed;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BasicAccountingInformation
    {
        public long TotalUserTime;
        public long TotalKernelTime;
        public long ThisPeriodTotalUserTime;
        public long ThisPeriodTotalKernelTime;
        public uint TotalPageFaultCount;
        public uint TotalProcesses;
        public uint ActiveProcesses;
        public uint TotalTerminatedProcesses;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct StartupInfo
    {
        public int cb;
        public string lpReserved;
        public string lpDesktop;
        public string lpTitle;
        public uint dwX;
        public uint dwY;
        public uint dwXSize;
        public uint dwYSize;
        public uint dwXCountChars;
        public uint dwYCountChars;
        public uint dwFillAttribute;
        public uint dwFlags;
        public short wShowWindow;
        public short cbReserved2;
        public IntPtr lpReserved2;
        public IntPtr hStdInput;
        public IntPtr hStdOutput;
        public IntPtr hStdError;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct ProcessInformation
    {
        public IntPtr hProcess;
        public IntPtr hThread;
        public uint dwProcessId;
        public uint dwThreadId;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct SecurityAttributes
    {
        public int nLength;
        public IntPtr lpSecurityDescriptor;
        [MarshalAs(UnmanagedType.Bool)]
        public bool bInheritHandle;
    }

    private const uint GenericRead = 0x80000000;
    private const uint GenericWrite = 0x40000000;
    private const uint FileShareRead = 0x00000001;
    private const uint FileShareWrite = 0x00000002;
    private const uint CreateAlways = 2;
    private const uint OpenExisting = 3;
    private const uint FileAttributeNormal = 0x00000080;
    private const uint StartfUseShowWindow = 0x00000001;
    private const uint StartfUseStdHandles = 0x00000100;
    private const uint CreateSuspended = 0x00000004;
    private const uint CreateNoWindow = 0x08000000;
    private const uint WaitObject0 = 0x00000000;
    private static readonly IntPtr InvalidHandleValue = new IntPtr(-1);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr CreateJobObject(IntPtr attributes, string name);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetInformationJobObject(
        IntPtr job, int infoClass, IntPtr info, uint length);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool QueryInformationJobObject(
        IntPtr job, int infoClass, IntPtr info, uint length, IntPtr returnLength);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool AssignProcessToJobObject(IntPtr job, IntPtr process);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool TerminateJobObject(IntPtr job, uint exitCode);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr CreateFile(
        string fileName, uint desiredAccess, uint shareMode,
        ref SecurityAttributes securityAttributes, uint creationDisposition,
        uint flagsAndAttributes, IntPtr templateFile);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CreateProcess(
        string applicationName, StringBuilder commandLine,
        IntPtr processAttributes, IntPtr threadAttributes,
        [MarshalAs(UnmanagedType.Bool)] bool inheritHandles,
        uint creationFlags, IntPtr environment, string currentDirectory,
        ref StartupInfo startupInfo, out ProcessInformation processInformation);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint ResumeThread(IntPtr thread);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool TerminateProcess(IntPtr process, uint exitCode);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CloseHandle(IntPtr handle);

    private static void RecordCloseFailure(
        IntPtr value, string label, List<Exception> failures)
    {
        if (value == IntPtr.Zero || value == InvalidHandleValue)
            return;
        if (!CloseHandle(value))
            failures.Add(new Win32Exception(
                Marshal.GetLastWin32Error(), "CloseHandle(" + label + ")"));
    }

    public ExtractumOwnedJob()
    {
        handle = CreateJobObject(IntPtr.Zero, null);
        if (handle == IntPtr.Zero)
            throw new Win32Exception(Marshal.GetLastWin32Error());

        var limits = new ExtendedLimitInformation();
        limits.BasicLimitInformation.LimitFlags = 0x00002000;
        int length = Marshal.SizeOf(typeof(ExtendedLimitInformation));
        IntPtr pointer = Marshal.AllocHGlobal(length);
        try
        {
            Marshal.StructureToPtr(limits, pointer, false);
            if (!SetInformationJobObject(handle, 9, pointer, (uint)length))
                throw new Win32Exception(Marshal.GetLastWin32Error());
        }
        catch (Exception setupError)
        {
            if (!CloseHandle(handle))
            {
                var closeError = new Win32Exception(
                    Marshal.GetLastWin32Error(), "CloseHandle(job constructor)");
                throw new AggregateException(
                    "Job Object setup and cleanup both failed.",
                    setupError,
                    closeError);
            }
            handle = IntPtr.Zero;
            throw;
        }
        finally
        {
            Marshal.FreeHGlobal(pointer);
        }
    }

    public Process StartAssigned(
        string applicationPath, string arguments, string currentDirectory,
        string stdoutPath, string stderrPath, bool hidden)
    {
        LastProcessCreated = false;
        LastProcessAssigned = false;
        LastLaunchTerminationConfirmed = true;
        LastProcessId = 0;
        var security = new SecurityAttributes
        {
            nLength = Marshal.SizeOf(typeof(SecurityAttributes)),
            bInheritHandle = true
        };
        IntPtr stdoutHandle = IntPtr.Zero;
        IntPtr stderrHandle = IntPtr.Zero;
        IntPtr stdinHandle = IntPtr.Zero;
        var processInfo = new ProcessInformation();
        Process process = null;
        bool resumed = false;
        bool launchCleanupConfirmed = true;
        try
        {
            stdoutHandle = CreateFile(
                stdoutPath, GenericWrite, FileShareRead | FileShareWrite,
                ref security, CreateAlways, FileAttributeNormal, IntPtr.Zero);
            if (stdoutHandle == InvalidHandleValue)
                throw new Win32Exception(Marshal.GetLastWin32Error(), "open stdout");
            stderrHandle = CreateFile(
                stderrPath, GenericWrite, FileShareRead | FileShareWrite,
                ref security, CreateAlways, FileAttributeNormal, IntPtr.Zero);
            if (stderrHandle == InvalidHandleValue)
                throw new Win32Exception(Marshal.GetLastWin32Error(), "open stderr");
            stdinHandle = CreateFile(
                "NUL", GenericRead, FileShareRead | FileShareWrite,
                ref security, OpenExisting, FileAttributeNormal, IntPtr.Zero);
            if (stdinHandle == InvalidHandleValue)
                throw new Win32Exception(Marshal.GetLastWin32Error(), "open stdin");

            var startup = new StartupInfo
            {
                cb = Marshal.SizeOf(typeof(StartupInfo)),
                dwFlags = StartfUseStdHandles | (hidden ? StartfUseShowWindow : 0),
                wShowWindow = 0,
                hStdInput = stdinHandle,
                hStdOutput = stdoutHandle,
                hStdError = stderrHandle
            };
            string command = "\"" + applicationPath + "\"";
            if (!String.IsNullOrWhiteSpace(arguments))
                command += " " + arguments;
            uint flags = CreateSuspended | (hidden ? CreateNoWindow : 0);
            if (!CreateProcess(
                    applicationPath, new StringBuilder(command),
                    IntPtr.Zero, IntPtr.Zero, true, flags, IntPtr.Zero,
                     currentDirectory, ref startup, out processInfo))
                throw new Win32Exception(Marshal.GetLastWin32Error(), "CreateProcessW");
            LastProcessCreated = true;
            LastLaunchTerminationConfirmed = false;
            LastProcessId = processInfo.dwProcessId;
            if (!AssignProcessToJobObject(handle, processInfo.hProcess))
                throw new Win32Exception(
                    Marshal.GetLastWin32Error(), "AssignProcessToJobObject");
            LastProcessAssigned = true;
            process = Process.GetProcessById((int)processInfo.dwProcessId);
            // Force Process to own a second, durable handle before the native
            // creation handle is closed and before the process can run/exit.
            IntPtr durableHandle = process.Handle;
            if (durableHandle == IntPtr.Zero)
                throw new InvalidOperationException("Process durable handle is null");
            if (ResumeThread(processInfo.hThread) == UInt32.MaxValue)
                throw new Win32Exception(Marshal.GetLastWin32Error(), "ResumeThread");
            resumed = true;
            return process;
        }
        catch (Exception startError)
        {
            Exception cleanupError = null;
            if (processInfo.hProcess != IntPtr.Zero && !resumed)
            {
                launchCleanupConfirmed = false;
                if (!TerminateProcess(processInfo.hProcess, 1))
                    cleanupError = new InvalidOperationException(
                        "UNCONFIRMED_PROCESS pid=" + processInfo.dwProcessId +
                        ": could not terminate the suspended process after launch failure",
                        new AggregateException(
                            startError,
                            new Win32Exception(Marshal.GetLastWin32Error())));
                else
                {
                    uint waitResult = WaitForSingleObject(processInfo.hProcess, 10000);
                    if (waitResult != WaitObject0)
                        cleanupError = new InvalidOperationException(
                            "UNCONFIRMED_PROCESS pid=" + processInfo.dwProcessId +
                            ": suspended process termination wait returned " + waitResult,
                            startError);
                    else
                    {
                        launchCleanupConfirmed = true;
                        LastLaunchTerminationConfirmed = true;
                    }
                }
            }
            if (process != null)
                process.Dispose();
            if (cleanupError != null)
                throw cleanupError;
            throw;
        }
        finally
        {
            var closeFailures = new List<Exception>();
            RecordCloseFailure(processInfo.hThread, "primary thread", closeFailures);
            RecordCloseFailure(processInfo.hProcess, "created process", closeFailures);
            RecordCloseFailure(stdinHandle, "stdin", closeFailures);
            RecordCloseFailure(stdoutHandle, "stdout", closeFailures);
            RecordCloseFailure(stderrHandle, "stderr", closeFailures);
            if (closeFailures.Count != 0)
            {
                var closeError = new AggregateException(
                    "One or more native launch handles could not be closed.",
                    closeFailures);
                if (resumed && process != null)
                {
                    var resumedCleanupFailures = new List<Exception>();
                    bool empty = false;
                    try
                    {
                        if (!TerminateJobObject(handle, 1))
                            throw new Win32Exception(
                                Marshal.GetLastWin32Error(),
                                "TerminateJobObject after native handle cleanup failure");
                        uint waitResult = WaitForSingleObject(process.Handle, 10000);
                        if (waitResult != WaitObject0)
                            throw new InvalidOperationException(
                                "resumed process termination wait returned " + waitResult);
                        empty = WaitForEmpty(10000) && ActiveProcesses == 0;
                    }
                    catch (Exception cleanupQueryError)
                    {
                        resumedCleanupFailures.Add(cleanupQueryError);
                    }
                    LastLaunchTerminationConfirmed = empty;
                    try
                    {
                        process.Dispose();
                    }
                    catch (Exception processDisposeError)
                    {
                        resumedCleanupFailures.Add(processDisposeError);
                    }
                    if (!LastLaunchTerminationConfirmed)
                    {
                        resumedCleanupFailures.Insert(0, closeError);
                        throw new InvalidOperationException(
                            "UNCONFIRMED_PROCESS pid=" + processInfo.dwProcessId +
                            ": native handle cleanup failed after resume and the " +
                            "assigned process tree could not be confirmed terminated",
                            new AggregateException(resumedCleanupFailures));
                    }
                    if (resumedCleanupFailures.Count != 0)
                    {
                        resumedCleanupFailures.Insert(0, closeError);
                        throw new AggregateException(
                            "Assigned process tree terminated, but managed/native cleanup failed.",
                            resumedCleanupFailures);
                    }
                }
                else if (!launchCleanupConfirmed)
                {
                    throw new InvalidOperationException(
                        "UNCONFIRMED_PROCESS pid=" + processInfo.dwProcessId +
                        ": launch and native handle cleanup both failed",
                        closeError);
                }
                throw closeError;
            }
        }
    }

    public uint ActiveProcesses
    {
        get
        {
            int length = Marshal.SizeOf(typeof(BasicAccountingInformation));
            IntPtr pointer = Marshal.AllocHGlobal(length);
            try
            {
                if (!QueryInformationJobObject(handle, 1, pointer, (uint)length, IntPtr.Zero))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                var value = (BasicAccountingInformation)Marshal.PtrToStructure(
                    pointer, typeof(BasicAccountingInformation));
                return value.ActiveProcesses;
            }
            finally
            {
                Marshal.FreeHGlobal(pointer);
            }
        }
    }

    public void Terminate(uint exitCode)
    {
        if (!TerminateJobObject(handle, exitCode))
            throw new Win32Exception(Marshal.GetLastWin32Error());
    }

    public bool WaitForEmpty(int timeoutMilliseconds)
    {
        var watch = Stopwatch.StartNew();
        do
        {
            if (ActiveProcesses == 0)
                return true;
            System.Threading.Thread.Sleep(50);
        }
        while (watch.ElapsedMilliseconds < timeoutMilliseconds);
        return ActiveProcesses == 0;
    }

    public void Dispose()
    {
        if (handle != IntPtr.Zero)
        {
            if (!CloseHandle(handle))
                throw new Win32Exception(
                    Marshal.GetLastWin32Error(), "CloseHandle(Job Object)");
            handle = IntPtr.Zero;
        }
    }
}
'@
}
```

Execute the helper once in a fresh PowerShell process with this exact preflight:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$jobHelper = Join-Path $scratch 'job-object.ps1'
$jobProbe = Join-Path $scratch 'job-object-preflight.ps1'
$descendantParent = Join-Path $scratch 'job-object-descendant-parent.ps1'
$descendantReady = Join-Path $scratch `
  ("job-object-descendant-{0}.ready" -f ([guid]::NewGuid().ToString('N')))
@'
param([Parameter(Mandatory = $true)][string]$ReadyPath)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$child = Start-Process -FilePath 'powershell.exe' -ArgumentList @(
  '-NoLogo', '-NoProfile', '-Command', 'Start-Sleep -Seconds 10'
) -PassThru -WindowStyle Hidden -ErrorAction Stop
try {
  $child.Id | Set-Content -LiteralPath $ReadyPath
  Start-Sleep -Seconds 10
} finally {
  $child.Refresh()
  if (-not $child.HasExited) {
    $child.Kill()
    [void]$child.WaitForExit(10000)
  }
}
'@ | Set-Content -LiteralPath $descendantParent
@'
param(
  [Parameter(Mandatory = $true)][string]$JobHelperPath,
  [Parameter(Mandatory = $true)][string]$ParentScriptPath,
  [Parameter(Mandatory = $true)][string]$ReadyPath,
  [Parameter(Mandatory = $true)][string]$ResultPath
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$result = [ordered]@{
  passed = $false
  termination_confirmed = $false
  active_after = $null
  job_helper_sha256 = $null
  launch_process_created = $false
  launch_process_assigned = $false
  launch_termination_confirmed = $false
  launch_process_id = $null
  descendant_pid = $null
  descendant_absent = $false
  error = $null
}
$job = $null
$process = $null
try {
  $result.job_helper_sha256 =
    (Get-FileHash -LiteralPath $JobHelperPath -Algorithm SHA256).Hash
  . $JobHelperPath
  $job = [ExtractumOwnedJob]::new()
  $powershellPath = (Get-Command powershell.exe -ErrorAction Stop).Source
  $arguments = '-NoLogo -NoProfile -ExecutionPolicy Bypass -File "{0}" -ReadyPath "{1}"' -f `
    $ParentScriptPath, $ReadyPath
  $probeRoot = Split-Path -Parent $JobHelperPath
  $process = $job.StartAssigned(
    $powershellPath,
    $arguments,
    (Get-Location).Path,
    (Join-Path $probeRoot 'job-object-descendant.stdout.log'),
    (Join-Path $probeRoot 'job-object-descendant.stderr.log'),
    $true)
  $deadline = [DateTimeOffset]::Now.AddSeconds(10)
  while (-not (Test-Path -LiteralPath $ReadyPath) -and
      [DateTimeOffset]::Now -lt $deadline) {
    Start-Sleep -Milliseconds 50
  }
  if (-not (Test-Path -LiteralPath $ReadyPath)) {
    throw 'Assigned parent did not report its descendant.'
  }
  $descendantPid = 0
  $descendantText = (Get-Content -LiteralPath $ReadyPath -Raw).Trim()
  if (-not [int]::TryParse($descendantText, [ref]$descendantPid)) {
    throw 'Assigned parent reported an invalid descendant PID.'
  }
  $result.descendant_pid = $descendantPid
  if ([uint32]$job.ActiveProcesses -lt 2) {
    throw 'Job Object did not inherit the assigned parent descendant.'
  }
  $job.Terminate(1)
  if (-not $process.WaitForExit(10000)) {
    throw 'Assigned descendant parent did not terminate within ten seconds.'
  }
  if (-not $job.WaitForEmpty(10000) -or [uint32]$job.ActiveProcesses -ne 0) {
    throw 'Job Object remained active after termination.'
  }
  $result.descendant_absent =
    $null -eq (Get-Process -Id $descendantPid -ErrorAction SilentlyContinue) -and
    $null -eq (Get-CimInstance Win32_Process -Filter `
      "ProcessId = $descendantPid" -ErrorAction Stop)
  if (-not $result.descendant_absent) {
    throw 'Assigned descendant remained observable after Job Object termination.'
  }
  $result.passed = $true
} catch {
  $result.error = $_.Exception.Message
} finally {
  try {
    if ($null -ne $process) {
      $process.Refresh()
      if (-not $process.HasExited) {
        $process.Kill()
        [void]$process.WaitForExit(10000)
      }
    }
    if ($null -ne $job) {
      $result.launch_process_created = [bool]$job.LastProcessCreated
      $result.launch_process_assigned = [bool]$job.LastProcessAssigned
      $result.launch_termination_confirmed =
        [bool]$job.LastLaunchTerminationConfirmed
      $result.launch_process_id = if ([uint32]$job.LastProcessId -ne 0) {
        [int]$job.LastProcessId
      } else { $null }
      if ([uint32]$job.ActiveProcesses -ne 0) {
        $job.Terminate(1)
        [void]$job.WaitForEmpty(10000)
      }
      $result.active_after = [uint32]$job.ActiveProcesses
      $rootAbsent = if ($null -ne $process) {
        $process.Refresh()
        $process.HasExited
      } elseif ($result.launch_process_created) {
        $result.launch_termination_confirmed -and
          $null -eq (Get-Process -Id $result.launch_process_id `
            -ErrorAction SilentlyContinue)
      } else { $true }
      $result.termination_confirmed = $rootAbsent -and
        $result.active_after -eq 0
    } else {
      # Constructor failure happened before a Job or child could exist.
      $result.launch_termination_confirmed = $true
      $result.termination_confirmed = $true
    }
    if ($null -ne $result.descendant_pid) {
      $result.descendant_absent =
        $null -eq (Get-Process -Id $result.descendant_pid `
          -ErrorAction SilentlyContinue) -and
        $null -eq (Get-CimInstance Win32_Process -Filter `
          "ProcessId = $($result.descendant_pid)" -ErrorAction Stop)
      if (-not $result.descendant_absent) {
        $result.termination_confirmed = $false
      }
    }
  } catch {
    $result.termination_confirmed = $false
    $cleanupText = "cleanup failure: $($_.Exception.Message)"
    $result.error = if ([string]::IsNullOrWhiteSpace([string]$result.error)) {
      $cleanupText
    } else { "$($result.error); $cleanupText" }
  } finally {
    try {
      if ($null -ne $job) { $job.Dispose() }
    } catch {
      $result.termination_confirmed = $false
      $disposeText = "Job disposal failure: $($_.Exception.Message)"
      $result.error = if ([string]::IsNullOrWhiteSpace([string]$result.error)) {
        $disposeText
      } else { "$($result.error); $disposeText" }
    }
    $result | ConvertTo-Json | Set-Content -LiteralPath $ResultPath
  }
}
if ($result.passed -and $result.termination_confirmed) { exit 0 }
if ($result.termination_confirmed) { exit 2 }
exit 3
'@ | Set-Content -LiteralPath $jobProbe
$jobProbeResultPath = Join-Path $scratch 'job-object-preflight.json'
$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $jobProbe `
    -JobHelperPath $jobHelper -ParentScriptPath $descendantParent `
    -ReadyPath $descendantReady -ResultPath $jobProbeResultPath `
    1> (Join-Path $scratch 'job-object-preflight.stdout.log') `
    2> (Join-Path $scratch 'job-object-preflight.stderr.log')
  $jobProbeCode = $LASTEXITCODE
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
$jobProbeResult = if (Test-Path -LiteralPath $jobProbeResultPath) {
  Get-Content -LiteralPath $jobProbeResultPath -Raw | ConvertFrom-Json
} else { $null }
$jobHelperHash = (Get-FileHash -LiteralPath $jobHelper -Algorithm SHA256).Hash
if ($jobProbeCode -eq 0) {
  if ($null -eq $jobProbeResult -or -not [bool]$jobProbeResult.passed -or
      -not [bool]$jobProbeResult.termination_confirmed -or
      -not [bool]$jobProbeResult.descendant_absent -or
      $jobProbeResult.job_helper_sha256 -ne $jobHelperHash) {
    throw 'Job Object preflight exit/artifact mismatch.'
  }
} elseif ($jobProbeCode -eq 2 -and $null -ne $jobProbeResult -and
    -not [bool]$jobProbeResult.passed -and
    [bool]$jobProbeResult.termination_confirmed) {
  throw 'Job Object qualification failed safely before measurement. Preserve scratch, correct and review the committed plan/runner, and restart with fresh scratch; no diagnostic attempt was consumed.'
} else {
  throw 'Job Object preflight termination is unconfirmed; stop all further child commands.'
}
```

Exit `0` enables runner qualification. Exit `2` plus `termination_confirmed=true` stops before measurement for a recorded, reviewed harness correction and a fresh workflow/scratch restart; it does not consume the zero-retry diagnostic attempt or evaluate the candidate. Any missing/inconsistent artifact, exit `3`, or unconfirmed termination stops all further child commands. Never fall back to parent-PID-only proof.

Then create `$scratch/invoke-shell-series.ps1` with this complete content. It is scratch infrastructure, not repository source:

```powershell
param(
  [Parameter(Mandatory = $true)][string]$Stage,
  [Parameter(Mandatory = $true)][string]$SourcePath,
  [Parameter(Mandatory = $true)][string]$ExpectedSha256,
  [Parameter(Mandatory = $true)][string]$AttemptRoot,
  [Parameter(Mandatory = $true)][string]$JobHelperPath,
  [ValidateSet('series', 'self-test')][string]$Mode = 'series'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$runs = $null
$resolved = $null
$original = $null
$commandPath = $null
$commandArgs = @()
$selfTestObservationPath = $null
$probeSuffix = [Text.Encoding]::UTF8.GetBytes(
  "`n// cargo-measurement-probe: extractum-process-reapplication-shell`n"
)
try {
  . $JobHelperPath
  New-Item -ItemType Directory -Path $AttemptRoot -Force | Out-Null
  $runs = Join-Path $AttemptRoot 'runs'
  New-Item -ItemType Directory -Path $runs -Force | Out-Null
  $resolved = (Resolve-Path -LiteralPath $SourcePath).Path
  $original = [IO.File]::ReadAllBytes($resolved)
  $recoveryPath = Join-Path $AttemptRoot 'source-recovery.bin'
  [IO.File]::WriteAllBytes($recoveryPath, $original)
  if ($Mode -eq 'self-test') {
    $commandPath = (Get-Command powershell.exe -ErrorAction Stop).Source
    $selfTestObservationPath = Join-Path $AttemptRoot `
      'self-test-observations.txt'
    $escapedSource = $resolved.Replace("'", "''")
    $escapedRecovery = $recoveryPath.Replace("'", "''")
    $escapedObservations = $selfTestObservationPath.Replace("'", "''")
    $suffixBase64 = [Convert]::ToBase64String($probeSuffix)
    $probeScript = @(
      '$ErrorActionPreference = ''Stop'''
      ('$sourcePath = ''{0}''' -f $escapedSource)
      ('$recoveryPath = ''{0}''' -f $escapedRecovery)
      ('$observationPath = ''{0}''' -f $escapedObservations)
      ('$suffixBase64 = ''{0}''' -f $suffixBase64)
      '$original = [IO.File]::ReadAllBytes($recoveryPath)'
      '$actual = [IO.File]::ReadAllBytes($sourcePath)'
      '$observations = @()'
      'if (Test-Path -LiteralPath $observationPath) {'
      '  $observations = @(Get-Content -LiteralPath $observationPath)'
      '}'
      'if ($observations.Count -eq 0) {'
      '  $state = ''sync'''
      '  $expected = $original'
      '} elseif ($observations.Count -eq 1 -and '
      '    $observations[0] -match ''^sync\|[0-9A-F]{64}$'') {'
      '  $state = ''timed'''
      '  $suffix = [Convert]::FromBase64String($suffixBase64)'
      '  $expected = New-Object byte[] ($original.Length + $suffix.Length)'
      '  [Array]::Copy($original, 0, $expected, 0, $original.Length)'
      '  [Array]::Copy($suffix, 0, $expected, $original.Length, $suffix.Length)'
      '} else { exit 43 }'
      'if (-not [Collections.StructuralComparisons]::StructuralEqualityComparer.Equals('
      '    $actual, $expected)) { exit 41 }'
      '$hash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash'
      '"$state|$hash" | Add-Content -LiteralPath $observationPath -Encoding ASCII'
      'exit 0'
    ) -join "`n"
    $encodedProbe = [Convert]::ToBase64String(
      [Text.Encoding]::Unicode.GetBytes($probeScript)
    )
    $commandArgs = @('-NoLogo', '-NoProfile', '-EncodedCommand', $encodedProbe)
  } else {
    $commandPath = (Get-Command cargo.exe -ErrorAction Stop).Source
    $commandArgs = @(
      'check', '--manifest-path', 'src-tauri/Cargo.toml', '--locked',
      '--workspace', '--all-targets'
    )
  }
} catch {
  $bootstrapError = $_.Exception.Message
  $bootstrapRestored = if ($null -ne $resolved) {
    try {
      (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash -eq
        $ExpectedSha256
    } catch { $false }
  } else { $true }
  try {
    New-Item -ItemType Directory -Path $AttemptRoot -Force | Out-Null
    [ordered]@{
      stage = $Stage
      mode = $Mode
      phase = 'bootstrap'
      classification = 'infrastructure_invalid'
      error = $bootstrapError
      child_started = $false
      termination_confirmed = $true
      source_restored = $bootstrapRestored
    } | ConvertTo-Json | Set-Content -LiteralPath `
      (Join-Path $AttemptRoot 'runner-infrastructure-failure.json')
  } catch {
    Write-Error "Runner bootstrap artifact failed: $($_.Exception.Message)"
  }
  exit 2
}

function Test-InfrastructureLog([string]$stdoutPath, [string]$stderrPath) {
  $text = @(
    if (Test-Path -LiteralPath $stdoutPath) {
      Get-Content -LiteralPath $stdoutPath -Raw -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $stderrPath) {
      Get-Content -LiteralPath $stderrPath -Raw -ErrorAction SilentlyContinue
    }
  ) -join "`n"
  return $text -match '(?i)(\.cargo-lock|access(?: is)? denied|os error 5|used by another process|cannot access the file)'
}

function Get-LiveProcessTreeIds([int]$rootId) {
  $all = @(Get-CimInstance Win32_Process -ErrorAction Stop)
  $known = [System.Collections.Generic.HashSet[int]]::new()
  [void]$known.Add($rootId)
  $changed = $true
  while ($changed) {
    $changed = $false
    foreach ($item in $all) {
      if ($known.Contains([int]$item.ParentProcessId) -and
          $known.Add([int]$item.ProcessId)) {
        $changed = $true
      }
    }
  }
  return @($all | Where-Object { $known.Contains([int]$_.ProcessId) } |
    ForEach-Object { [int]$_.ProcessId })
}

function Invoke-BoundedCommand(
  [string]$label,
  [string]$stdoutPath,
  [string]$stderrPath,
  [int]$timeoutMs
) {
  $result = [ordered]@{
    label = $label
    started = $false
    process_id = $null
    exit_code = $null
    elapsed_ms = $null
    timed_out = $false
    taskkill_exit_code = $null
    job_assigned = $false
    job_active_processes = $null
    launch_process_created = $false
    launch_process_assigned = $false
    launch_termination_confirmed = $false
    launch_process_id = $null
    remaining_tree_ids = @()
    owned_tree_ids = @()
    termination_confirmed = $false
    infrastructure = $false
    error = $null
  }
  $process = $null
  $watch = $null
  $job = $null
  try {
    # Create the ownership boundary before starting the measured process. The
    # helper creates the process suspended, assigns it, then resumes it. The
    # stopwatch starts immediately before that atomic launch; Job queries and
    # CIM evidence happen only after it stops.
    $job = [ExtractumOwnedJob]::new()
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $process = $job.StartAssigned(
      $commandPath,
      ($commandArgs -join ' '),
      (Get-Location).Path,
      $stdoutPath,
      $stderrPath,
      $true)
    $result.started = $true
    $result.launch_process_created = [bool]$job.LastProcessCreated
    $result.launch_process_assigned = [bool]$job.LastProcessAssigned
    $result.launch_process_id = [int]$job.LastProcessId
    $result.process_id = $process.Id
    $result.job_assigned = $true
    if (-not $process.WaitForExit($timeoutMs)) {
      $watch.Stop()
      $result.elapsed_ms = $watch.ElapsedMilliseconds
      $result.timed_out = $true
      $result.infrastructure = $true
      $killLog = Join-Path $runs "$label-taskkill.log"
      $savedErrorActionPreference = $ErrorActionPreference
      $ErrorActionPreference = 'Continue'
      try {
        $result.owned_tree_ids = @(
          @($result.owned_tree_ids) + @(Get-LiveProcessTreeIds $process.Id) |
            Sort-Object -Unique
        )
        & taskkill.exe /PID $process.Id /T /F 1> $killLog 2>&1
        $result.taskkill_exit_code = $LASTEXITCODE
      } finally {
        $ErrorActionPreference = $savedErrorActionPreference
      }
      [void]$process.WaitForExit(10000)
      [void]$job.WaitForEmpty(10000)
      $result.job_active_processes = [uint32]$job.ActiveProcesses
      if ($result.job_active_processes -ne 0) {
        $job.Terminate(1)
        [void]$process.WaitForExit(10000)
        [void]$job.WaitForEmpty(10000)
        $result.job_active_processes = [uint32]$job.ActiveProcesses
      }
      $remainingBySnapshot = @($result.owned_tree_ids | Where-Object {
        $null -ne (Get-Process -Id $_ -ErrorAction SilentlyContinue)
      })
      $result.remaining_tree_ids = @(
        $remainingBySnapshot + @(Get-LiveProcessTreeIds $process.Id) |
          Sort-Object -Unique
      )
      $result.termination_confirmed = $result.job_assigned -and
        $result.job_active_processes -eq 0 -and
        $result.remaining_tree_ids.Count -eq 0
      if (-not $result.termination_confirmed) {
        $result.error = 'timeout; Job Object/process-tree termination unconfirmed'
      } else {
        $result.error = 'timeout; Job Object process tree terminated'
        $job.Dispose()
        $job = $null
      }
      return [pscustomobject]$result
    }
    $process.WaitForExit()
    $watch.Stop()
    $result.elapsed_ms = $watch.ElapsedMilliseconds
    $result.exit_code = $process.ExitCode
    $result.job_active_processes = [uint32]$job.ActiveProcesses
    if ($result.job_active_processes -ne 0) {
      $result.infrastructure = $true
      $result.error = 'root exited while Job Object still owned live descendants'
      $job.Terminate(1)
      [void]$process.WaitForExit(10000)
      [void]$job.WaitForEmpty(10000)
      $result.job_active_processes = [uint32]$job.ActiveProcesses
    }
    $remainingBySnapshot = @($result.owned_tree_ids | Where-Object {
      $null -ne (Get-Process -Id $_ -ErrorAction SilentlyContinue)
    })
    $result.remaining_tree_ids = @(
      $remainingBySnapshot + @(Get-LiveProcessTreeIds $process.Id) |
        Sort-Object -Unique
    )
    $result.termination_confirmed = $result.job_assigned -and
      $result.job_active_processes -eq 0 -and
      $result.remaining_tree_ids.Count -eq 0
    if (-not $result.termination_confirmed) {
      $result.infrastructure = $true
      $result.error = 'exited child remains observable by Job Object or CIM'
    } elseif ($result.exit_code -ne 0 -and
        (Test-InfrastructureLog $stdoutPath $stderrPath)) {
      $result.infrastructure = $true
      $result.error = 'target lock/access contention'
    }
    if ($result.termination_confirmed) {
      $job.Dispose()
      $job = $null
    }
  } catch {
    $runnerError = $_.Exception.Message
    if ($null -ne $watch -and $watch.IsRunning) {
      $watch.Stop()
      $result.elapsed_ms = $watch.ElapsedMilliseconds
    }
    $result.infrastructure = $true
    $result.error = "command-start/runner failure: $runnerError"
    if ($result.started) {
      try {
        $result.owned_tree_ids = @(Get-LiveProcessTreeIds $process.Id |
          Sort-Object -Unique)
        $killLog = Join-Path $runs "$label-runner-failure-taskkill.log"
        $savedErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = 'Continue'
        try {
          & taskkill.exe /PID $process.Id /T /F 1> $killLog 2>&1
          $result.taskkill_exit_code = $LASTEXITCODE
        } finally {
          $ErrorActionPreference = $savedErrorActionPreference
        }
        [void]$process.WaitForExit(10000)
        if ($result.job_assigned) {
          $result.job_active_processes = [uint32]$job.ActiveProcesses
          if ($result.job_active_processes -ne 0) {
            $job.Terminate(1)
            [void]$process.WaitForExit(10000)
            [void]$job.WaitForEmpty(10000)
            $result.job_active_processes = [uint32]$job.ActiveProcesses
          }
        }
        $result.remaining_tree_ids = @($result.owned_tree_ids | Where-Object {
          $null -ne (Get-Process -Id $_ -ErrorAction SilentlyContinue)
        })
        $result.termination_confirmed = $result.job_assigned -and
          $result.job_active_processes -eq 0 -and
          $result.remaining_tree_ids.Count -eq 0
        if ($result.termination_confirmed) {
          $job.Dispose()
          $job = $null
        }
      } catch {
        $result.termination_confirmed = $false
        $result.error += "; cleanup failure: $($_.Exception.Message)"
      }
    } else {
      if ($null -eq $job) {
        # Constructor failure happened before a Job or child could exist.
        $result.launch_termination_confirmed = $true
        $result.termination_confirmed = $true
      } else {
        try {
          $result.launch_process_created = [bool]$job.LastProcessCreated
          $result.launch_process_assigned = [bool]$job.LastProcessAssigned
          $result.launch_termination_confirmed =
            [bool]$job.LastLaunchTerminationConfirmed
          $result.launch_process_id = if ([uint32]$job.LastProcessId -ne 0) {
            [int]$job.LastProcessId
          } else { $null }
          $result.process_id = $result.launch_process_id
          $result.job_assigned = $result.launch_process_assigned
          $result.job_active_processes = [uint32]$job.ActiveProcesses
          $result.remaining_tree_ids = if (
              $null -ne $result.launch_process_id -and
              $null -ne (Get-Process -Id $result.launch_process_id `
                -ErrorAction SilentlyContinue)) {
            @($result.launch_process_id)
          } else { @() }
          $result.termination_confirmed =
            (-not $result.launch_process_created -or
              $result.launch_termination_confirmed) -and
            $result.job_active_processes -eq 0 -and
            $result.remaining_tree_ids.Count -eq 0
          if (-not $result.termination_confirmed) {
            $result.error += '; structured launch cleanup proof is incomplete'
          } else {
            $job.Dispose()
            $job = $null
          }
        } catch {
          $result.termination_confirmed = $false
          $result.error += "; launch-state/cleanup inspection failed: $($_.Exception.Message)"
        }
      }
    }
  }
  return [pscustomobject]$result
}

function Invoke-One([string]$label, [bool]$recorded) {
  $metaPath = Join-Path $runs "$label-meta.json"
  $syncOut = Join-Path $runs "$label-sync.stdout.log"
  $syncErr = Join-Path $runs "$label-sync.stderr.log"
  $timedOut = Join-Path $runs "$label.stdout.log"
  $timedErr = Join-Path $runs "$label.stderr.log"
  $meta = [ordered]@{
    stage = $Stage
    mode = $Mode
    label = $label
    recorded = $recorded
    sync_exit_code = $null
    timed_exit_code = $null
    elapsed_ms = $null
    sync = $null
    timed = $null
    sync_expected_sha256 = if ($Mode -eq 'self-test') {
      $ExpectedSha256
    } else { $null }
    sync_observed_sha256 = $null
    timed_expected_sha256 = $null
    timed_observed_sha256 = $null
    self_test_observations = @()
    termination_confirmed = $true
    restored = $false
    classification = $null
    error = $null
  }
  $safeToRestore = $true
  try {
    $startingHash = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
    if ($startingHash -ne $ExpectedSha256) {
      throw "INFRASTRUCTURE: starting source hash mismatch for $label"
    }
    $safeToRestore = $false
    $sync = Invoke-BoundedCommand "$label-sync" $syncOut $syncErr 900000
    $meta.sync = $sync
    $meta.sync_exit_code = $sync.exit_code
    $meta.termination_confirmed = $sync.termination_confirmed
    if (-not $sync.termination_confirmed) {
      throw "INFRASTRUCTURE: synchronization termination unconfirmed for $label"
    }
    $safeToRestore = $true
    if ($sync.infrastructure) {
      throw "INFRASTRUCTURE: synchronization $($sync.error) for $label"
    }
    if ($sync.exit_code -ne 0) {
      throw "COMMAND_FAILURE: synchronization command failed for $label"
    }
    if ($Mode -eq 'self-test') {
      $syncObservations = @(
        Get-Content -LiteralPath $selfTestObservationPath -ErrorAction Stop |
          ForEach-Object { [string]$_ }
      )
      $expectedSyncLine = "sync|$ExpectedSha256"
      if ($syncObservations.Count -ne 1 -or
          $syncObservations[0] -cne $expectedSyncLine) {
        throw 'INFRASTRUCTURE: self-test sync did not observe exact original bytes'
      }
      $meta.sync_observed_sha256 = $ExpectedSha256
      $meta.self_test_observations = @($syncObservations)
    }
    $combined = New-Object byte[] ($original.Length + $probeSuffix.Length)
    [Array]::Copy($original, 0, $combined, 0, $original.Length)
    [Array]::Copy($probeSuffix, 0, $combined, $original.Length, $probeSuffix.Length)
    [IO.File]::WriteAllBytes($resolved, $combined)
    if ($Mode -eq 'self-test') {
      $meta.timed_expected_sha256 =
        (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
    }
    $safeToRestore = $false
    $timed = Invoke-BoundedCommand "$label-timed" $timedOut $timedErr 900000
    $meta.timed = $timed
    $meta.timed_exit_code = $timed.exit_code
    $meta.elapsed_ms = $timed.elapsed_ms
    $meta.termination_confirmed = $timed.termination_confirmed
    if (-not $timed.termination_confirmed) {
      throw "INFRASTRUCTURE: timed termination unconfirmed for $label"
    }
    $safeToRestore = $true
    if ($timed.infrastructure) {
      throw "INFRASTRUCTURE: timed $($timed.error) for $label"
    }
    if ($timed.exit_code -ne 0) {
      throw "COMMAND_FAILURE: timed command failed for $label"
    }
    if ($Mode -eq 'self-test') {
      $timedObservations = @(
        Get-Content -LiteralPath $selfTestObservationPath -ErrorAction Stop |
          ForEach-Object { [string]$_ }
      )
      $expectedTimedLine = "timed|$($meta.timed_expected_sha256)"
      if ($timedObservations.Count -ne 2 -or
          $timedObservations[0] -cne "sync|$ExpectedSha256" -or
          $timedObservations[1] -cne $expectedTimedLine) {
        throw 'INFRASTRUCTURE: self-test timed command did not observe exact suffixed bytes'
      }
      $meta.timed_observed_sha256 = $meta.timed_expected_sha256
      $meta.self_test_observations = @($timedObservations)
    }
  } catch {
    $meta.error = $_.Exception.Message
  } finally {
    if ($safeToRestore) {
      try {
        [IO.File]::WriteAllBytes($resolved, $original)
        $meta.restored =
          ((Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash -eq
            $ExpectedSha256)
      } catch {
        $meta.error = "INFRASTRUCTURE: restoration failure: $($_.Exception.Message)"
        $meta.restored = $false
      }
    }
    if ($null -eq $meta.error) { $meta.classification = 'ok' }
    elseif ($meta.error -like 'COMMAND_FAILURE:*') {
      $meta.classification = 'command_failed'
    } else {
      $meta.classification = 'infrastructure_invalid'
    }
    $meta | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $metaPath
  }
  if (-not $meta.termination_confirmed -or -not $meta.restored) { return 2 }
  if ($null -ne $meta.error) {
    if ($meta.error -like 'COMMAND_FAILURE:*') { return 1 }
    return 2
  }
  return 0
}

try {
  $labels = if ($Mode -eq 'self-test') {
    @('self-test')
  } else {
    @('warmup', 'sample-1', 'sample-2', 'sample-3', 'sample-4', 'sample-5')
  }
  foreach ($label in $labels) {
    $recorded = $Mode -eq 'self-test' -or $label -ne 'warmup'
    $code = Invoke-One $label $recorded
    if ($code -ne 0) {
      [ordered]@{
        stage = $Stage
        mode = $Mode
        failed_label = $label
        exit_code = $code
        classification = if ($code -eq 1) { 'command_failed' } else { 'infrastructure_invalid' }
        source_restored =
          ((Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash -eq
            $ExpectedSha256)
      } | ConvertTo-Json | Set-Content -LiteralPath `
        (Join-Path $AttemptRoot 'failure.json')
      exit $code
    }
  }

  if ($Mode -eq 'self-test') {
    $meta = Get-Content -LiteralPath `
      (Join-Path $runs 'self-test-meta.json') -Raw | ConvertFrom-Json
    if ($meta.classification -ne 'ok' -or -not [bool]$meta.restored -or
        -not [bool]$meta.termination_confirmed -or
        $meta.sync_exit_code -ne 0 -or $meta.timed_exit_code -ne 0 -or
        -not [bool]$meta.sync.job_assigned -or
        -not [bool]$meta.timed.job_assigned -or
        $meta.sync.job_active_processes -ne 0 -or
        $meta.timed.job_active_processes -ne 0 -or
        $meta.sync_expected_sha256 -cne $ExpectedSha256 -or
        $meta.sync_observed_sha256 -cne $ExpectedSha256 -or
        $meta.timed_expected_sha256 -cne $meta.timed_observed_sha256 -or
        @($meta.self_test_observations).Count -ne 2 -or
        $meta.self_test_observations[0] -cne "sync|$ExpectedSha256" -or
        $meta.self_test_observations[1] -cne
          "timed|$($meta.timed_expected_sha256)") {
      throw 'Synthetic runner cycle did not prove launch, ownership, exit, and restoration.'
    }
    [ordered]@{
      stage = $Stage
      mode = $Mode
      passed = $true
      source_sha256 = $ExpectedSha256
      sync_observed_sha256 = $meta.sync_observed_sha256
      timed_observed_sha256 = $meta.timed_observed_sha256
      probe_suffix_base64 = [Convert]::ToBase64String($probeSuffix)
      observations = @($meta.self_test_observations)
      source_restored = $true
      meta_file = 'runs/self-test-meta.json'
    } | ConvertTo-Json | Set-Content -LiteralPath `
      (Join-Path $AttemptRoot 'summary.json')
    exit 0
  }

  $samples = @(Get-ChildItem -LiteralPath $runs -Filter 'sample-*-meta.json' |
    Sort-Object Name | ForEach-Object {
      (Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json).elapsed_ms
    })
  if ($samples.Count -ne 5) { throw "Expected five samples, got $($samples.Count)" }
  $sorted = @($samples | Sort-Object)
  $median = [int64]$sorted[2]
  $stableCount = @($samples | Where-Object {
    [Math]::Abs([int64]$_ - $median) -le 300
  }).Count
  [ordered]@{
    stage = $Stage
    samples_ms = @($samples)
    median_ms = $median
    stable_count = $stableCount
    required_stable_count = 4
    max_absolute_deviation_ms = 300
    series_valid = ($stableCount -ge 4)
    source_sha256 = $ExpectedSha256
  } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath `
    (Join-Path $AttemptRoot 'summary.json')
  exit 0
} catch {
  $outerError = $_.Exception.Message
  $knownMeta = @()
  $metaInspectionFailed = $false
  try {
    $knownMeta = @(
      Get-ChildItem -LiteralPath $runs -Filter '*-meta.json' `
        -ErrorAction Stop | ForEach-Object {
          Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
        }
    )
  } catch { $metaInspectionFailed = $true }
  $expectedMetaCount = if ($Mode -eq 'self-test') { 1 } else { 6 }
  $outerTerminationConfirmed = -not $metaInspectionFailed -and
    $knownMeta.Count -eq $expectedMetaCount -and
    @($knownMeta | Where-Object {
      -not [bool]$_.termination_confirmed
    }).Count -eq 0
  [ordered]@{
    stage = $Stage
    mode = $Mode
    phase = 'runner-summary'
    classification = 'infrastructure_invalid'
    error = $outerError
    child_started = ($knownMeta.Count -ne 0)
    termination_confirmed = $outerTerminationConfirmed
    source_restored = try {
      (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash -eq $ExpectedSha256
    } catch { $false }
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $AttemptRoot 'runner-infrastructure-failure.json')
  exit 2
}
```

Run one synthetic cycle through the actual runner before any baseline probe. It uses the already-qualified descendant Job Object helper, a deterministic `%TEMP%` fixture plus working copy, and two bounded observing `powershell.exe -EncodedCommand` launches; it does not invoke Cargo, touch `src-tauri/target`, read or mutate the measured source:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
function Stop-RunnerQualification(
  [string]$message,
  [int]$exitCode,
  [string]$attempt,
  [bool]$sourceRestored,
  [bool]$recoveryMatches,
  [bool]$terminationConfirmed
) {
  [ordered]@{
    gate = 'runner-self-test'
    classification = 'infrastructure_invalid'
    error = $message
    exit_code = $exitCode
    attempt = $attempt
    source_restored = $sourceRestored
    recovery_matches = $recoveryMatches
    termination_confirmed = $terminationConfirmed
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'runner-self-test-failure.json')
  if (-not $sourceRestored -or -not $terminationConfirmed) {
    throw 'Synthetic runner termination/restoration is unconfirmed; stop all further child commands.'
  }
  throw 'Synthetic runner qualification failed safely before measurement. Correct and review the committed plan/runner, then restart with fresh scratch; no diagnostic attempt was consumed.'
}
$qualificationPath = Join-Path $scratch 'runner-qualification.json'
if (Test-Path -LiteralPath $qualificationPath) {
  throw 'Runner is already qualified in this scratch session; do not rerun qualification.'
}
if ((Test-Path -LiteralPath `
      (Join-Path $scratch 'runner-self-test-current.txt')) -or
    (Test-Path -LiteralPath `
      (Join-Path $scratch 'runner-self-test-failure.json'))) {
  throw 'Prior qualification evidence must be archived after a committed harness correction before qualification is repeated.'
}
$jobHelperPath = Join-Path $scratch 'job-object.ps1'
$runnerPath = Join-Path $scratch 'invoke-shell-series.ps1'
$jobPreflightPath = Join-Path $scratch 'job-object-preflight.json'
$jobPreflight = Get-Content -LiteralPath $jobPreflightPath -Raw |
  ConvertFrom-Json
$jobHelperHash = (Get-FileHash -LiteralPath $jobHelperPath -Algorithm SHA256).Hash
$runnerHash = (Get-FileHash -LiteralPath $runnerPath -Algorithm SHA256).Hash
if (-not [bool]$jobPreflight.passed -or
    -not [bool]$jobPreflight.termination_confirmed -or
    -not [bool]$jobPreflight.descendant_absent -or
    $jobPreflight.job_helper_sha256 -cne $jobHelperHash) {
  throw 'Job Object qualification is missing, failed, or stale; do not run the runner self-test.'
}
$fixtureOriginal = Join-Path $scratch 'runner-self-test-original.bin'
$selfTestSource = Join-Path $scratch 'runner-self-test-working.rs'
$fixtureBytes = [Text.UTF8Encoding]::new($false).GetBytes(
  "extractum-runner-self-test-fixture`r`nexact-bytes-v1`n"
)
$declaredSuffix = [Text.Encoding]::UTF8.GetBytes(
  "`n// cargo-measurement-probe: extractum-process-reapplication-shell`n"
)
$declaredSuffixBase64 = [Convert]::ToBase64String($declaredSuffix)
$declaredTimedBytes = New-Object byte[] `
  ($fixtureBytes.Length + $declaredSuffix.Length)
[Array]::Copy($fixtureBytes, 0, $declaredTimedBytes, 0, $fixtureBytes.Length)
[Array]::Copy(
  $declaredSuffix, 0, $declaredTimedBytes, $fixtureBytes.Length,
  $declaredSuffix.Length)
$declaredHasher = [Security.Cryptography.SHA256]::Create()
try {
  $declaredTimedHash = -join ($declaredHasher.ComputeHash($declaredTimedBytes) |
    ForEach-Object { $_.ToString('X2') })
} finally { $declaredHasher.Dispose() }
[IO.File]::WriteAllBytes($fixtureOriginal, $fixtureBytes)
[IO.File]::WriteAllBytes($selfTestSource, $fixtureBytes)
$selfTestHash = (Get-FileHash -LiteralPath $selfTestSource -Algorithm SHA256).Hash
$selfTestId = 'runner-self-test-{0}-{1}' -f `
  ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')), `
  ([guid]::NewGuid().ToString('N'))
$selfTestAttempt = Join-Path $scratch $selfTestId
$selfTestAttempt | Set-Content -LiteralPath `
  (Join-Path $scratch 'runner-self-test-current.txt')
$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File `
    $runnerPath `
    -Stage 'runner-self-test' -Mode 'self-test' `
    -SourcePath $selfTestSource -ExpectedSha256 $selfTestHash `
    -AttemptRoot $selfTestAttempt `
    -JobHelperPath $jobHelperPath `
    1> (Join-Path $scratch 'runner-self-test.stdout.log') `
    2> (Join-Path $scratch 'runner-self-test.stderr.log')
  $selfTestCode = $LASTEXITCODE
} catch {
  $selfTestCode = -1
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
$sourceRestored =
  ((Get-FileHash -LiteralPath $selfTestSource -Algorithm SHA256).Hash -eq
    $selfTestHash)
$recoveryPath = Join-Path $selfTestAttempt 'source-recovery.bin'
$recoveryMatches = (Test-Path -LiteralPath $recoveryPath) -and
  ((Get-FileHash -LiteralPath $recoveryPath -Algorithm SHA256).Hash -eq
    $selfTestHash)
$metaFiles = @(Get-ChildItem -LiteralPath (Join-Path $selfTestAttempt 'runs') `
  -Filter '*-meta.json' -ErrorAction SilentlyContinue)
$meta = if ($metaFiles.Count -eq 1) {
  Get-Content -LiteralPath $metaFiles[0].FullName -Raw | ConvertFrom-Json
} else { $null }
$syncResult = if ($null -ne $meta) { $meta.sync } else { $null }
$timedResult = if ($null -ne $meta) { $meta.timed } else { $null }
$terminationConfirmed = $null -ne $meta -and
  [bool]$meta.termination_confirmed -and
  ($null -eq $syncResult -or [bool]$syncResult.termination_confirmed) -and
  ($null -eq $timedResult -or [bool]$timedResult.termination_confirmed)
$bootstrapFailurePath = Join-Path $selfTestAttempt `
  'runner-infrastructure-failure.json'
if ($null -eq $meta -and (Test-Path -LiteralPath $bootstrapFailurePath)) {
  $bootstrapFailure = Get-Content -LiteralPath $bootstrapFailurePath -Raw |
    ConvertFrom-Json
  $terminationConfirmed = [bool]$bootstrapFailure.termination_confirmed -and
    -not [bool]$bootstrapFailure.child_started
}
if ($selfTestCode -ne 0) {
  Stop-RunnerQualification 'Runner returned a nonzero self-test exit.' `
    $selfTestCode $selfTestAttempt $sourceRestored $recoveryMatches `
    $terminationConfirmed
}
$summaryPath = Join-Path $selfTestAttempt 'summary.json'
if (-not (Test-Path -LiteralPath $summaryPath) -or $metaFiles.Count -ne 1) {
  Stop-RunnerQualification 'Synthetic runner success artifacts are incomplete.' `
    2 $selfTestAttempt $sourceRestored $recoveryMatches $terminationConfirmed
}
$summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
$unexpectedFailures = @(
  @('failure.json', 'runner-infrastructure-failure.json') |
    ForEach-Object { Join-Path $selfTestAttempt $_ } |
    Where-Object { Test-Path -LiteralPath $_ }
)
$observations = @($meta.self_test_observations)
if (-not [bool]$summary.passed -or $summary.mode -ne 'self-test' -or
    $meta.mode -ne 'self-test' -or $meta.classification -ne 'ok' -or
    -not [bool]$meta.restored -or -not $terminationConfirmed -or
    $meta.sync_exit_code -ne 0 -or $meta.timed_exit_code -ne 0 -or
    -not [bool]$meta.sync.job_assigned -or
    -not [bool]$meta.timed.job_assigned -or
    $meta.sync.job_active_processes -ne 0 -or
    $meta.timed.job_active_processes -ne 0 -or
    $meta.sync_expected_sha256 -cne $selfTestHash -or
    $meta.sync_observed_sha256 -cne $selfTestHash -or
    $summary.probe_suffix_base64 -cne $declaredSuffixBase64 -or
    $meta.timed_expected_sha256 -cne $declaredTimedHash -or
    $meta.timed_observed_sha256 -cne $declaredTimedHash -or
    $observations.Count -ne 2 -or
    $observations[0] -cne "sync|$selfTestHash" -or
    $observations[1] -cne "timed|$declaredTimedHash" -or
    -not $sourceRestored -or -not $recoveryMatches -or
    $unexpectedFailures.Count -ne 0) {
  Stop-RunnerQualification `
    'Synthetic runner artifacts do not prove a clean mutation/restoration cycle.' `
    2 $selfTestAttempt $sourceRestored $recoveryMatches $terminationConfirmed
}
[ordered]@{
  passed = $true
  qualified_at = [DateTimeOffset]::Now.ToString('o')
  job_helper_sha256 = $jobHelperHash
  runner_sha256 = $runnerHash
  job_preflight_sha256 =
    (Get-FileHash -LiteralPath $jobPreflightPath -Algorithm SHA256).Hash
  self_test_attempt = $selfTestAttempt
  self_test_summary_sha256 =
    (Get-FileHash -LiteralPath $summaryPath -Algorithm SHA256).Hash
  self_test_meta_sha256 =
    (Get-FileHash -LiteralPath $metaFiles[0].FullName -Algorithm SHA256).Hash
  fixture_sha256 = $selfTestHash
  declared_probe_suffix_base64 = $declaredSuffixBase64
  declared_timed_sha256 = $declaredTimedHash
  sync_observed_sha256 = $meta.sync_observed_sha256
  timed_observed_sha256 = $meta.timed_observed_sha256
  observations = @($observations)
  source_restored = $sourceRestored
  recovery_matches = $recoveryMatches
  termination_confirmed = $terminationConfirmed
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $qualificationPath
```

Expected: the earlier descendant preflight proves forced tree termination with no live Job Object processes and binds that proof to the helper hash. The runner self-test proves that its sync child saw the exact fixture, its timed child saw the exact fixture plus the literal fixed suffix, and the working copy and recovery bytes ended exact. The receipt binds both qualified scratch programs and all qualification evidence. All of this must pass before Step 5. A qualification failure is not a baseline sample and does not consume the zero-retry session.

If qualification fails safely, preserve the entire failed scratch directory, correct and review the committed plan/runner, and restart Task 1 only after that correction creates a new committed implementation base, in a fresh workflow-owned worktree and scratch session. The old scratch and its consumed environment-preflight claim remain evidence; neither is deleted, overwritten, or reused. No baseline claim exists yet, so this does not retry the zero-retry baseline. Missing correction evidence, an unchanged implementation base, or unconfirmed termination/restoration forbids a restart.

After creating and qualifying it, inspect both scratch files and confirm they contain `--locked`, `source-recovery.bin`, the fixed `extractum-process-reapplication-shell` suffix, bounded waits, atomic suspended creation/Job Object assignment/resume, authoritative active-process checks, `taskkill /T /F` as supplementary cleanup evidence, confirmed termination before restoration, separate command/infrastructure classifications, explicit `series`/`self-test` modes, five measured sample labels, and the `300` ms stability calculation.

- [ ] **Step 5: Run the single predeclared baseline stability series**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
if (-not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
  throw 'CARGO_TARGET_DIR changed after environment capture.'
}
$baselineRepoRaw = @(git rev-parse --show-toplevel)
$baselineRepoCode = $LASTEXITCODE
$baselineHeadRaw = @(git rev-parse HEAD)
$baselineHeadCode = $LASTEXITCODE
$baselineStatus = @(git status --porcelain=v1)
$baselineStatusCode = $LASTEXITCODE
if ($baselineRepoCode -ne 0 -or $baselineHeadCode -ne 0 -or
    $baselineStatusCode -ne 0 -or $baselineRepoRaw.Count -ne 1 -or
    $baselineHeadRaw.Count -ne 1) {
  throw 'Could not bind the baseline claim to the current worktree.'
}
$baselineRepo = [IO.Path]::GetFullPath(
  ([string]$baselineRepoRaw[0]).Trim()).TrimEnd('\')
$baselineTarget = [IO.Path]::GetFullPath(
  (Join-Path $baselineRepo 'src-tauri/target')).TrimEnd('\')
$baselineHead = ([string]$baselineHeadRaw[0]).Trim()
if ($baselineRepo -ne [string]$environment.repository -or
    $baselineTarget -ne [string]$environment.target -or
    $baselineHead -ne [string]$environment.identity_commit -or
    $baselineStatus.Count -ne 0) {
  throw 'Baseline claim requires the captured clean identity worktree and target.'
}
$qualificationPath = Join-Path $scratch 'runner-qualification.json'
$qualification = if (Test-Path -LiteralPath $qualificationPath) {
  Get-Content -LiteralPath $qualificationPath -Raw | ConvertFrom-Json
} else { $null }
if ($null -eq $qualification -or -not [bool]$qualification.passed -or
    -not [bool]$qualification.termination_confirmed -or
    -not [bool]$qualification.source_restored -or
    -not [bool]$qualification.recovery_matches) {
  throw 'Runner qualification is missing or failed; stop before claiming the diagnostic session.'
}
$jobHelperPath = Join-Path $scratch 'job-object.ps1'
$runnerPath = Join-Path $scratch 'invoke-shell-series.ps1'
$jobPreflightPath = Join-Path $scratch 'job-object-preflight.json'
$qualifiedAttempt = [string]$qualification.self_test_attempt
if ([string]::IsNullOrWhiteSpace($qualifiedAttempt)) {
  throw 'Runner qualification has no self-test attempt; stop before claiming the diagnostic session.'
}
$qualifiedSummary = Join-Path $qualifiedAttempt 'summary.json'
$qualifiedMeta = Join-Path $qualifiedAttempt 'runs/self-test-meta.json'
$currentSelfTest = Join-Path $scratch 'runner-self-test-current.txt'
$qualificationCurrent = if (Test-Path -LiteralPath $currentSelfTest) {
  (Get-Content -LiteralPath $currentSelfTest -Raw).Trim()
} else { '' }
if ($qualifiedAttempt -ne $qualificationCurrent -or
    -not (Test-Path -LiteralPath $jobHelperPath) -or
    -not (Test-Path -LiteralPath $runnerPath) -or
    -not (Test-Path -LiteralPath $jobPreflightPath) -or
    -not (Test-Path -LiteralPath $qualifiedSummary) -or
    -not (Test-Path -LiteralPath $qualifiedMeta) -or
    (Get-FileHash -LiteralPath $jobHelperPath -Algorithm SHA256).Hash -cne
      $qualification.job_helper_sha256 -or
    (Get-FileHash -LiteralPath $runnerPath -Algorithm SHA256).Hash -cne
      $qualification.runner_sha256 -or
    (Get-FileHash -LiteralPath $jobPreflightPath -Algorithm SHA256).Hash -cne
      $qualification.job_preflight_sha256 -or
    (Get-FileHash -LiteralPath $qualifiedSummary -Algorithm SHA256).Hash -cne
      $qualification.self_test_summary_sha256 -or
    (Get-FileHash -LiteralPath $qualifiedMeta -Algorithm SHA256).Hash -cne
      $qualification.self_test_meta_sha256 -or
    (Test-Path -LiteralPath `
      (Join-Path $scratch 'runner-self-test-failure.json'))) {
  throw 'Runner qualification is missing, failed, or stale; stop before claiming the diagnostic session.'
}
$sourcePath = 'src-tauri/src/lib.rs'
$sourceHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash
$attemptId = 'baseline-{0}-{1}' -f `
  ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')), `
  ([guid]::NewGuid().ToString('N'))
$attempt = Join-Path $scratch "attempts/$attemptId"
$gitCommonRaw = @(git rev-parse --git-common-dir)
if ($LASTEXITCODE -ne 0 -or $gitCommonRaw.Count -ne 1) {
  throw 'Could not resolve the shared Git directory before diagnostic claim.'
}
$gitCommon = (Resolve-Path -LiteralPath ([string]$gitCommonRaw[0])).Path
$claimMaterial = '{0}|{1}' -f $gitCommon.ToLowerInvariant(), `
  $environment.implementation_base
$claimHasher = [Security.Cryptography.SHA256]::Create()
try {
  $claimKey = -join ($claimHasher.ComputeHash(
    [Text.Encoding]::UTF8.GetBytes($claimMaterial)) | ForEach-Object {
      $_.ToString('x2')
    })
} finally { $claimHasher.Dispose() }
$claimRoot = Join-Path $env:TEMP `
  "extractum-process-reapplication-claims/$claimKey"
$persistedClaimRoot = (Get-Content -LiteralPath `
  (Join-Path $scratch 'diagnostic-claim-root.txt') -Raw).Trim()
if ($persistedClaimRoot -ne $claimRoot -or
    [string]$environment.diagnostic_claim_root -ne $claimRoot) {
  throw 'Persisted diagnostic claim root does not match the recomputed stable root.'
}
$environmentClaimPath = Join-Path $claimRoot 'environment-preflight.json'
if (-not (Test-Path -LiteralPath $environmentClaimPath)) {
  throw 'Stable environment preflight claim is missing; never recreate it.'
}
$environmentClaim = Get-Content -LiteralPath $environmentClaimPath -Raw |
  ConvertFrom-Json
$environmentClaimHash =
  (Get-FileHash -LiteralPath $environmentClaimPath -Algorithm SHA256).Hash
if ($environmentClaim.stage -ne 'environment-preflight' -or
    $environmentClaim.scratch -ne $scratch -or
    $environmentClaim.repository -ne $baselineRepo -or
    $environmentClaim.target -ne $baselineTarget -or
    $environmentClaim.identity_commit -ne $environment.identity_commit -or
    $environmentClaim.implementation_base -ne
      $environment.implementation_base -or
    $environmentClaim.quiet_artifact -ne
      (Join-Path $scratch 'quiet-initial.json') -or
    $environmentClaimHash -cne
      $environment.environment_preflight_claim_sha256) {
  throw 'Environment preflight claim is missing, mismatched, or stale.'
}
$initialQuietPath = Join-Path $scratch 'quiet-initial.json'
$initialQuiet = Get-Content -LiteralPath $initialQuietPath -Raw |
  ConvertFrom-Json
$derivedInitialQuietValid = [bool]$initialQuiet.cim_available -and
  [int]$initialQuiet.blocking_count -eq 0
if ([bool]$environment.initial_quiet_valid -ne $derivedInitialQuietValid) {
  throw 'Environment initial quiet-window result is inconsistent with its artifact.'
}
$environmentHash = (Get-FileHash -LiteralPath `
  (Join-Path $scratch 'environment.json') -Algorithm SHA256).Hash
$initialQuietHash =
  (Get-FileHash -LiteralPath $initialQuietPath -Algorithm SHA256).Hash
$baselineClaimPath = Join-Path $claimRoot 'baseline.json'
if (Test-Path -LiteralPath $baselineClaimPath) {
  throw "Zero-retry baseline is already claimed at $baselineClaimPath; never launch quiet-window or Cargo again."
}
$claimPayload = [ordered]@{
  stage = 'baseline'
  claimed_at = [DateTimeOffset]::Now.ToString('o')
  scratch = $scratch
  attempt = $attempt
  repository = $baselineRepo
  target = $baselineTarget
  head_commit = $baselineHead
  implementation_base = $environment.implementation_base
  identity_commit = $environment.identity_commit
  environment_preflight_claim_sha256 = $environmentClaimHash
  environment_sha256 = $environmentHash
  initial_quiet_sha256 = $initialQuietHash
  qualification_sha256 =
    (Get-FileHash -LiteralPath $qualificationPath -Algorithm SHA256).Hash
  runner_sha256 = $qualification.runner_sha256
  source_sha256 = $sourceHash
} | ConvertTo-Json
$claimBytes = [Text.UTF8Encoding]::new($false).GetBytes($claimPayload)
$baselineClaimTempPath = '{0}.{1}.tmp' -f `
  $baselineClaimPath, ([guid]::NewGuid().ToString('N'))
$claimStream = $null
try {
  $claimStream = [IO.File]::Open(
    $baselineClaimTempPath,
    [IO.FileMode]::CreateNew,
    [IO.FileAccess]::Write,
    [IO.FileShare]::None)
  $claimStream.Write($claimBytes, 0, $claimBytes.Length)
  $claimStream.Flush($true)
} catch [IO.IOException] {
  throw "Could not atomically claim the zero-retry baseline: $($_.Exception.Message)"
} finally {
  if ($null -ne $claimStream) { $claimStream.Dispose() }
}
try {
  [IO.File]::Move($baselineClaimTempPath, $baselineClaimPath)
} catch [IO.IOException] {
  throw "Could not atomically publish the zero-retry baseline claim: $($_.Exception.Message)"
}
[ordered]@{ path = $sourcePath; sha256 = $sourceHash } |
  ConvertTo-Json | Set-Content -LiteralPath `
  (Join-Path $scratch 'baseline-source.json')
$attempt | Set-Content -LiteralPath (Join-Path $scratch 'baseline-current.txt')
$diagnosticInvalidReason = if (-not [bool]$environment.initial_quiet_valid) {
  'initial quiet-window preflight failed after the one-shot session claim'
} else {
  $savedErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File `
      (Join-Path $scratch 'assert-quiet-window.ps1') -ArtifactPath `
      (Join-Path $scratch 'quiet-baseline.json') `
      1> (Join-Path $scratch 'quiet-baseline.stdout.log') `
      2> (Join-Path $scratch 'quiet-baseline.stderr.log')
    $quietCode = $LASTEXITCODE
  } catch {
    $quietCode = -1
  } finally {
    $ErrorActionPreference = $savedErrorActionPreference
  }
  if ($quietCode -ne 0) {
    'baseline quiet-window preflight failed after the one-shot session claim'
  } else { $null }
}
$quietCode = if ($null -ne $diagnosticInvalidReason) { 1 } else { 0 }
if ($quietCode -ne 0) {
  New-Item -ItemType Directory -Path $attempt -Force | Out-Null
  [ordered]@{
    stage = 'baseline'
    classification = 'infrastructure_invalid'
    error = $diagnosticInvalidReason
    child_started = $false
    termination_confirmed = $true
    source_restored = $true
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $attempt 'runner-infrastructure-failure.json')
  $probeCode = 2
} else {
  $savedErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  $probeStartError = $null
  try {
    & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File `
      (Join-Path $scratch 'invoke-shell-series.ps1') -Stage 'baseline' `
      -Mode 'series' -SourcePath $sourcePath -ExpectedSha256 $sourceHash `
      -AttemptRoot $attempt `
      -JobHelperPath (Join-Path $scratch 'job-object.ps1') `
      1> (Join-Path $scratch 'baseline-runner.stdout.log') `
      2> (Join-Path $scratch 'baseline-runner.stderr.log')
    $probeCode = $LASTEXITCODE
  } catch {
    $probeCode = 2
    $probeStartError = $_.Exception.Message
  } finally {
    $ErrorActionPreference = $savedErrorActionPreference
  }
  if ($null -ne $probeStartError) {
    New-Item -ItemType Directory -Path $attempt -Force | Out-Null
    [ordered]@{
      stage = 'baseline'
      phase = 'runner-process-start'
      classification = 'infrastructure_invalid'
      error = $probeStartError
      child_started = $false
      termination_confirmed = $true
      source_restored = $true
    } | ConvertTo-Json | Set-Content -LiteralPath `
      (Join-Path $attempt 'runner-infrastructure-failure.json')
  }
}
if ((Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash -ne $sourceHash) {
  throw 'Baseline probe bytes were not restored; stop all child commands.'
}
if ($probeCode -eq 1) {
  $commandFailurePath = Join-Path $attempt 'failure.json'
  $commandFailure = if (Test-Path -LiteralPath $commandFailurePath) {
    Get-Content -LiteralPath $commandFailurePath -Raw | ConvertFrom-Json
  } else { $null }
  $failedMetaPath = if ($null -ne $commandFailure) {
    Join-Path $attempt "runs/$($commandFailure.failed_label)-meta.json"
  } else { $null }
  $failedMeta = if ($null -ne $failedMetaPath -and
      (Test-Path -LiteralPath $failedMetaPath)) {
    Get-Content -LiteralPath $failedMetaPath -Raw | ConvertFrom-Json
  } else { $null }
  $commandFailureProven = $null -ne $commandFailure -and
    $commandFailure.exit_code -eq 1 -and
    $commandFailure.stage -eq 'baseline' -and
    $commandFailure.classification -eq 'command_failed' -and
    [bool]$commandFailure.source_restored -and
    $null -ne $failedMeta -and $failedMeta.mode -eq 'series' -and
    $failedMeta.stage -eq 'baseline' -and
    $failedMeta.label -eq $commandFailure.failed_label -and
    $failedMeta.classification -eq 'command_failed' -and
    $failedMeta.error -like 'COMMAND_FAILURE:*' -and
    [bool]$failedMeta.restored -and [bool]$failedMeta.termination_confirmed
  if (-not $commandFailureProven) {
    [ordered]@{
      stage = 'baseline'
      phase = 'command-failure-routing'
      classification = 'infrastructure_invalid'
      error = 'Exit 1 lacked coherent command-failure/restoration/termination evidence.'
      child_started = ($null -ne $failedMeta)
      termination_confirmed = if ($null -ne $failedMeta) {
        [bool]$failedMeta.termination_confirmed
      } else { $false }
      source_restored = $true
    } | ConvertTo-Json | Set-Content -LiteralPath `
      (Join-Path $attempt 'runner-infrastructure-failure.json')
    $probeCode = 2
  } else {
    throw 'Confirmed baseline Cargo failure.'
  }
}
if ($probeCode -eq 2) {
  $runMeta = @(Get-ChildItem -LiteralPath (Join-Path $attempt 'runs') `
    -Filter '*-meta.json' -ErrorAction SilentlyContinue | ForEach-Object {
      Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
    })
  if (@($runMeta | Where-Object { -not $_.termination_confirmed }).Count -ne 0) {
    throw 'Baseline child termination is unconfirmed; stop all further child commands.'
  }
  $partialSamples = @(Get-ChildItem -LiteralPath (Join-Path $attempt 'runs') `
    -Filter 'sample-*-meta.json' -ErrorAction SilentlyContinue |
    Sort-Object Name | ForEach-Object {
      $meta = Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
      if ($null -ne $meta.elapsed_ms -and $meta.timed_exit_code -eq 0) {
        [int64]$meta.elapsed_ms
      }
    })
  $failurePath = if (Test-Path -LiteralPath `
      (Join-Path $attempt 'runner-infrastructure-failure.json')) {
    Join-Path $attempt 'runner-infrastructure-failure.json'
  } else {
    Join-Path $attempt 'failure.json'
  }
  $failure = Get-Content -LiteralPath $failurePath -Raw | ConvertFrom-Json
  if ((Split-Path -Leaf $failurePath) -eq
      'runner-infrastructure-failure.json' -and
      ($null -eq $failure.PSObject.Properties['termination_confirmed'] -or
        -not [bool]$failure.termination_confirmed)) {
    throw 'Baseline infrastructure routing cannot confirm child termination; stop all further child commands.'
  }
  [ordered]@{
    stage = 'baseline'
    samples_ms = @($partialSamples)
    median_ms = $null
    stable_count = $null
    required_stable_count = 4
    max_absolute_deviation_ms = 300
    series_valid = $false
    invalid_reason = "zero-retry infrastructure failure: $($failure | ConvertTo-Json -Compress)"
    source_sha256 = $sourceHash
  } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath `
    (Join-Path $attempt 'summary.json')
}
if ($probeCode -notin @(0, 2)) { throw "Unexpected baseline probe exit $probeCode" }
$summaryPath = Join-Path $attempt 'summary.json'
$summary = Get-Content -LiteralPath $summaryPath -Raw |
  ConvertFrom-Json
function Test-CompletedSeriesSummary(
  [object]$value,
  [string]$expectedStage,
  [string]$expectedSourceHash
) {
  $samples = @($value.samples_ms)
  if ($value.stage -ne $expectedStage -or
      $value.source_sha256 -cne $expectedSourceHash -or
      [int]$value.required_stable_count -ne 4 -or
      [int]$value.max_absolute_deviation_ms -ne 300 -or
      @($samples | Where-Object { $null -eq $_ }).Count -ne 0) {
    return $false
  }
  if ($null -ne $value.median_ms -and $null -ne $value.stable_count) {
    if ($samples.Count -ne 5) { return $false }
    $sorted = @($samples | ForEach-Object { [int64]$_ } | Sort-Object)
    $median = [int64]$sorted[2]
    $stableCount = @($samples | Where-Object {
      [Math]::Abs([int64]$_ - $median) -le 300
    }).Count
    return [int64]$value.median_ms -eq $median -and
      [int]$value.stable_count -eq $stableCount -and
      [bool]$value.series_valid -eq ($stableCount -ge 4)
  }
  return -not [bool]$value.series_valid -and
    $null -eq $value.median_ms -and $null -eq $value.stable_count -and
    $samples.Count -le 5 -and
    $null -ne $value.PSObject.Properties['invalid_reason'] -and
    -not [string]::IsNullOrWhiteSpace([string]$value.invalid_reason)
}
if (-not (Test-CompletedSeriesSummary $summary 'baseline' $sourceHash)) {
  throw 'Baseline summary is incomplete or structurally incoherent.'
}
$baselineClaimHash =
  (Get-FileHash -LiteralPath $baselineClaimPath -Algorithm SHA256).Hash
$baselineCompletionPath = Join-Path $attempt 'baseline-completion.json'
$baselineCompletionPayload = [ordered]@{
  stage = 'baseline-completion'
  completed_at = [DateTimeOffset]::Now.ToString('o')
  scratch = $scratch
  attempt = $attempt
  repository = $baselineRepo
  target = $baselineTarget
  head_commit = $baselineHead
  baseline_claim_sha256 = $baselineClaimHash
  summary_sha256 =
    (Get-FileHash -LiteralPath $summaryPath -Algorithm SHA256).Hash
  qualification_sha256 =
    (Get-FileHash -LiteralPath $qualificationPath -Algorithm SHA256).Hash
  runner_sha256 = $qualification.runner_sha256
  source_sha256 = $sourceHash
  source_restored = $true
  termination_confirmed = $true
  probe_exit_code = $probeCode
} | ConvertTo-Json
$baselineCompletionBytes =
  [Text.UTF8Encoding]::new($false).GetBytes($baselineCompletionPayload)
$baselineCompletionTempPath = '{0}.{1}.tmp' -f `
  $baselineCompletionPath, ([guid]::NewGuid().ToString('N'))
$baselineCompletionStream = $null
$baselineCompletionError = $null
try {
  $baselineCompletionStream = [IO.File]::Open(
    $baselineCompletionTempPath,
    [IO.FileMode]::CreateNew,
    [IO.FileAccess]::Write,
    [IO.FileShare]::None)
  $baselineCompletionStream.Write(
    $baselineCompletionBytes, 0, $baselineCompletionBytes.Length)
  $baselineCompletionStream.Flush($true)
} catch {
  $baselineCompletionError = $_.Exception.Message
} finally {
  if ($null -ne $baselineCompletionStream) {
    $baselineCompletionStream.Dispose()
  }
}
if ($null -eq $baselineCompletionError) {
  try {
    [IO.File]::Move($baselineCompletionTempPath, $baselineCompletionPath)
  } catch {
    $baselineCompletionError = $_.Exception.Message
  }
}
if ($null -ne $baselineCompletionError) {
  $artifactFailurePath = Join-Path $claimRoot `
    'diagnostic-artifact-failure.json'
  $artifactFailurePayload = [ordered]@{
    stage = 'diagnostic-artifact-failure'
    failure_stage = 'baseline-completion'
    recorded_at = [DateTimeOffset]::Now.ToString('o')
    scratch = $scratch
    attempt = $attempt
    error = $baselineCompletionError
    baseline_claim_sha256 = $baselineClaimHash
    summary_sha256 =
      (Get-FileHash -LiteralPath $summaryPath -Algorithm SHA256).Hash
    source_sha256 = $sourceHash
    source_restored = $true
    termination_confirmed = $true
  } | ConvertTo-Json
  $artifactFailureBytes =
    [Text.UTF8Encoding]::new($false).GetBytes($artifactFailurePayload)
  $artifactFailureTempPath = '{0}.{1}.tmp' -f `
    $artifactFailurePath, ([guid]::NewGuid().ToString('N'))
  $artifactFailureStream = $null
  try {
    $artifactFailureStream = [IO.File]::Open(
      $artifactFailureTempPath,
      [IO.FileMode]::CreateNew,
      [IO.FileAccess]::Write,
      [IO.FileShare]::None)
    $artifactFailureStream.Write(
      $artifactFailureBytes, 0, $artifactFailureBytes.Length)
    $artifactFailureStream.Flush($true)
  } finally {
    if ($null -ne $artifactFailureStream) {
      $artifactFailureStream.Dispose()
    }
  }
  [IO.File]::Move($artifactFailureTempPath, $artifactFailurePath)
}
$summary | Format-List
```

Expected: a current hash-bound qualification receipt and the matching stable environment-preflight claim are required before the stable baseline claim. The baseline claim binds the environment and initial quiet-window artifacts and is durably created before consuming that recorded result or launching the baseline quiet-window or any Cargo child. One warm-up is discarded, five samples are recorded, source bytes are restored, and median/stability are written. An existing claim forbids every later baseline quiet/Cargo launch. Exit `2` writes a synthetic invalid summary only after every started child is confirmed terminated and the source hash is exact; an unproven exit `1` is infrastructure and fail-stops when termination is not proven.

- [ ] **Step 6: Checkpoint the unchanged baseline**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
$claimRoot = [string]$environment.diagnostic_claim_root
$baselineClaimPath = Join-Path $claimRoot 'baseline.json'
$baselineClaim = Get-Content -LiteralPath $baselineClaimPath -Raw |
  ConvertFrom-Json
$baselineCompletionPath = Join-Path `
  ([string]$baselineClaim.attempt) 'baseline-completion.json'
$artifactFailurePath = Join-Path $claimRoot `
  'diagnostic-artifact-failure.json'
$environmentClaimPath = Join-Path $claimRoot 'environment-preflight.json'
$qualificationPath = Join-Path $scratch 'runner-qualification.json'
$runnerPath = Join-Path $scratch 'invoke-shell-series.ps1'
$initialQuietPath = Join-Path $scratch 'quiet-initial.json'
$environmentClaim = Get-Content -LiteralPath $environmentClaimPath -Raw |
  ConvertFrom-Json
$qualification = Get-Content -LiteralPath $qualificationPath -Raw |
  ConvertFrom-Json
$environmentClaimHash =
  (Get-FileHash -LiteralPath $environmentClaimPath -Algorithm SHA256).Hash
$qualificationHash =
  (Get-FileHash -LiteralPath $qualificationPath -Algorithm SHA256).Hash
$runnerHash = (Get-FileHash -LiteralPath $runnerPath -Algorithm SHA256).Hash
function Test-RecoverySeriesSummary([object]$value, [string]$sourceHash) {
  $samples = @($value.samples_ms)
  if ($value.stage -ne 'baseline' -or $value.source_sha256 -cne $sourceHash -or
      [int]$value.required_stable_count -ne 4 -or
      [int]$value.max_absolute_deviation_ms -ne 300) { return $false }
  if ($null -ne $value.median_ms -and $null -ne $value.stable_count) {
    if ($samples.Count -ne 5) { return $false }
    $sorted = @($samples | ForEach-Object { [int64]$_ } | Sort-Object)
    $median = [int64]$sorted[2]
    $stable = @($samples | Where-Object {
      [Math]::Abs([int64]$_ - $median) -le 300
    }).Count
    return [int64]$value.median_ms -eq $median -and
      [int]$value.stable_count -eq $stable -and
      [bool]$value.series_valid -eq ($stable -ge 4)
  }
  return -not [bool]$value.series_valid -and
    $null -eq $value.median_ms -and $null -eq $value.stable_count -and
    $samples.Count -le 5 -and
    -not [string]::IsNullOrWhiteSpace([string]$value.invalid_reason)
}
$baselineCompletionValid = $false
if (Test-Path -LiteralPath $baselineCompletionPath) {
  try {
    $baselineCompletion = Get-Content -LiteralPath $baselineCompletionPath -Raw |
      ConvertFrom-Json
    $baselineSummaryPath = Join-Path `
      ([string]$baselineClaim.attempt) 'summary.json'
    $baselineSummary = Get-Content -LiteralPath $baselineSummaryPath -Raw |
      ConvertFrom-Json
    $baselineCompletionValid =
      [bool]$qualification.passed -and
      [bool]$qualification.termination_confirmed -and
      $qualification.runner_sha256 -ceq $runnerHash -and
      $environmentClaim.stage -eq 'environment-preflight' -and
      $environmentClaim.scratch -eq $scratch -and
      $environmentClaim.repository -eq $environment.repository -and
      $environmentClaim.target -eq $environment.target -and
      $environmentClaim.identity_commit -eq $environment.identity_commit -and
      $environmentClaim.implementation_base -eq $environment.implementation_base -and
      $environmentClaimHash -ceq $environment.environment_preflight_claim_sha256 -and
      $baselineClaim.stage -eq 'baseline' -and
      $baselineClaim.scratch -eq $scratch -and
      $baselineClaim.repository -eq $environment.repository -and
      $baselineClaim.target -eq $environment.target -and
      $baselineClaim.head_commit -eq $environment.identity_commit -and
      $baselineClaim.implementation_base -eq $environment.implementation_base -and
      $baselineClaim.identity_commit -eq $environment.identity_commit -and
      $baselineClaim.environment_preflight_claim_sha256 -ceq $environmentClaimHash -and
      $baselineClaim.environment_sha256 -ceq
        (Get-FileHash -LiteralPath (Join-Path $scratch 'environment.json') `
          -Algorithm SHA256).Hash -and
      $baselineClaim.initial_quiet_sha256 -ceq
        (Get-FileHash -LiteralPath $initialQuietPath -Algorithm SHA256).Hash -and
      $baselineClaim.qualification_sha256 -ceq $qualificationHash -and
      $baselineClaim.runner_sha256 -ceq $runnerHash -and
      (Test-RecoverySeriesSummary $baselineSummary $baselineClaim.source_sha256) -and
      $baselineCompletion.stage -eq 'baseline-completion' -and
      $baselineCompletion.scratch -eq $scratch -and
      $baselineCompletion.attempt -eq $baselineClaim.attempt -and
      $baselineCompletion.repository -eq $environment.repository -and
      $baselineCompletion.target -eq $environment.target -and
      $baselineCompletion.head_commit -eq $environment.identity_commit -and
      $baselineCompletion.baseline_claim_sha256 -ceq
        (Get-FileHash -LiteralPath $baselineClaimPath -Algorithm SHA256).Hash -and
      $baselineCompletion.summary_sha256 -ceq
        (Get-FileHash -LiteralPath $baselineSummaryPath -Algorithm SHA256).Hash -and
      $baselineCompletion.qualification_sha256 -ceq $qualificationHash -and
      $baselineCompletion.runner_sha256 -ceq $runnerHash -and
      $baselineCompletion.source_sha256 -ceq $baselineClaim.source_sha256 -and
      [bool]$baselineCompletion.source_restored -and
      [bool]$baselineCompletion.termination_confirmed -and
      [int]$baselineCompletion.probe_exit_code -in @(0, 2)
  } catch { $baselineCompletionValid = $false }
}
if (-not $baselineCompletionValid -and
    -not (Test-Path -LiteralPath $artifactFailurePath)) {
  if ((Get-FileHash -LiteralPath 'src-tauri/src/lib.rs' -Algorithm SHA256).Hash `
      -cne $baselineClaim.source_sha256) {
    throw 'Interrupted baseline cannot prove exact source restoration.'
  }
  $allProcesses = @(Get-CimInstance Win32_Process -ErrorAction Stop)
  $liveRunner = @($allProcesses | Where-Object {
    $_.ProcessId -ne $PID -and
    [string]$_.CommandLine -match [regex]::Escape($runnerPath)
  })
  $blockingBuild = @($allProcesses | Where-Object {
    $name = [string]$_.Name
    $command = [string]$_.CommandLine
    $name -match '^(cargo.*|rustc|rust-analyzer|extractum|tauri|vite)\.exe$' -or
      ($name -match '^(node|npm|npx)(\.exe|\.cmd)?$' -and
        $command -match '(?i)(vite|tauri|svelte-kit|cargo)')
  })
  if ($blockingBuild.Count -ne 0 -or $liveRunner.Count -ne 0) {
    throw 'Interrupted baseline termination is not independently confirmed.'
  }
  $summaryPath = Join-Path ([string]$baselineClaim.attempt) 'summary.json'
  $artifactFailurePayload = [ordered]@{
    stage = 'diagnostic-artifact-failure'
    failure_stage = 'baseline-completion-missing-or-invalid'
    recorded_at = [DateTimeOffset]::Now.ToString('o')
    scratch = $scratch
    attempt = $baselineClaim.attempt
    error = 'Atomic baseline completion receipt is missing, malformed, or mismatched.'
    baseline_claim_sha256 =
      (Get-FileHash -LiteralPath $baselineClaimPath -Algorithm SHA256).Hash
    summary_sha256 = if (Test-Path -LiteralPath $summaryPath) {
      (Get-FileHash -LiteralPath $summaryPath -Algorithm SHA256).Hash
    } else { $null }
    source_sha256 = $baselineClaim.source_sha256
    source_restored = $true
    termination_confirmed = $true
  } | ConvertTo-Json
  $artifactFailureBytes =
    [Text.UTF8Encoding]::new($false).GetBytes($artifactFailurePayload)
  $artifactFailureTempPath = '{0}.{1}.tmp' -f `
    $artifactFailurePath, ([guid]::NewGuid().ToString('N'))
  $artifactFailureStream = [IO.File]::Open(
    $artifactFailureTempPath,
    [IO.FileMode]::CreateNew,
    [IO.FileAccess]::Write,
    [IO.FileShare]::None)
  try {
    $artifactFailureStream.Write(
      $artifactFailureBytes, 0, $artifactFailureBytes.Length)
    $artifactFailureStream.Flush($true)
  } finally { $artifactFailureStream.Dispose() }
  [IO.File]::Move($artifactFailureTempPath, $artifactFailurePath)
}
git status --short
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect baseline status.' }
if (@(git status --porcelain=v1).Count -ne 0) {
  throw 'Baseline measurement changed repository bytes.'
}
cargo check --manifest-path src-tauri/Cargo.toml --locked -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Baseline extractum check failed.' }
```

Expected: clean repository and a passing baseline app-package check. Do not commit scratch artifacts.

### Task 3: Reapply and commit the exact historical candidate

**Files:**

- Modify exactly: the 14 candidate path states from the frozen manifest
- Read: Task 2 scratch evidence
- Must not modify: identity files, roadmap, policy docs, or any unlisted source

**Interfaces:**

- Consumes: a verified historical preimage and the committed identity manifest.
- Produces: one code-only reapplication commit whose no-renames raw diff, blobs, modes, subtree, and patch fingerprints match `b364756c` exactly.

- [ ] **Step 1: Reconfirm the preimage immediately before replay**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$status = @(git status --porcelain=v1)
if ($LASTEXITCODE -ne 0 -or $status.Count -ne 0) {
  throw "Candidate preimage is not clean: $($status -join '; ')"
}
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File `
  (Join-Path $scratch 'assert-quiet-window.ps1') -ArtifactPath `
  (Join-Path $scratch 'quiet-replay.json')
if ($LASTEXITCODE -ne 0) { throw 'Candidate replay quiet window is not exclusive.' }

foreach ($entry in $manifest.entries) {
  $spec = 'HEAD:{0}' -f $entry.path
  $line = @(git ls-tree HEAD -- $entry.path)
  if ($LASTEXITCODE -ne 0) { throw "Could not inspect $spec" }
  if ($null -eq $entry.parent_blob) {
    if ($line.Count -ne 0) { throw "New path already exists: $($entry.path)" }
  } else {
    if ($line.Count -ne 1 -or $line[0] -notmatch `
        '^(?<mode>[0-9]{6}) blob (?<blob>[0-9a-f]{40})\t' -or
        $Matches.mode -ne $entry.parent_mode -or
        $Matches.blob -ne $entry.parent_blob) {
      throw "Material preimage mismatch: $($entry.path)"
    }
  }
}
$preimageTauriRaw = @(git rev-parse 'HEAD:src-tauri')
$preimageTauriCode = $LASTEXITCODE
if ($preimageTauriCode -ne 0 -or $preimageTauriRaw.Count -ne 1) {
  throw 'Could not resolve the current src-tauri preimage.'
}
$preimageTauri = ([string]$preimageTauriRaw[0]).Trim()
if ($preimageTauri -ne $manifest.parent_src_tauri_tree) {
  throw 'Current src-tauri no longer matches the frozen parent tree.'
}
cmd.exe /d /c `
  "git show --format= -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ b364756c7b5768d644321afeaeb81ec04e2481a4 | git apply --cached --check"
if ($LASTEXITCODE -ne 0) {
  throw 'Exact historical patch no longer applies; stop the exact-candidate path.'
}
```

Expected: clean/exclusive worktree, 14 matching preimages, matching full `src-tauri` parent tree, and a clean apply check.

- [ ] **Step 2: Apply the historical commit without committing or resolving**

Run exactly:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$candidate = 'b364756c7b5768d644321afeaeb81ec04e2481a4'
git cherry-pick --no-commit $candidate
$applyCode = $LASTEXITCODE
if ($applyCode -ne 0) {
  $scratch = (Get-Content -LiteralPath `
    (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
  $manifest = Get-Content -LiteralPath `
    'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
    -Raw | ConvertFrom-Json
  @(git status --porcelain=v1) | Set-Content -LiteralPath `
    (Join-Path $scratch 'replay-conflict-status.txt')
  @(git ls-files -u) | Set-Content -LiteralPath `
    (Join-Path $scratch 'replay-conflict-unmerged-index.txt')
  @(git diff --raw) | Set-Content -LiteralPath `
    (Join-Path $scratch 'replay-conflict-worktree-raw.txt')
  @(git diff --cached --raw) | Set-Content -LiteralPath `
    (Join-Path $scratch 'replay-conflict-index-raw.txt')
  git cherry-pick --abort
  if ($LASTEXITCODE -ne 0 -or @(git status --porcelain=v1).Count -ne 0) {
    $paths = @($manifest.entries.path)
    git restore --source=HEAD --staged --worktree -- $paths
  }
  $cleanupRestoreCode = $LASTEXITCODE
  $cleanupStatus = @(git status --porcelain=v1)
  $cleanupStatusCode = $LASTEXITCODE
  $cleanupTauriRaw = @(git rev-parse 'HEAD:src-tauri')
  $cleanupTauriCode = $LASTEXITCODE
  $cleanupTauri = if ($cleanupTauriRaw.Count -eq 1) {
    ([string]$cleanupTauriRaw[0]).Trim()
  } else { $null }
  if ($cleanupRestoreCode -ne 0 -or $cleanupStatusCode -ne 0 -or
      $cleanupTauriCode -ne 0 -or $cleanupTauriRaw.Count -ne 1 -or
      $cleanupStatus.Count -ne 0 -or
      $cleanupTauri -ne $manifest.parent_src_tauri_tree) {
    throw 'Replay conflict cleanup failed; stop without manual resolution.'
  }
  throw 'Historical cherry-pick conflicted; evidence is preserved and the exact preimage is restored.'
}
```

Expected: candidate changes are staged with no conflict. On conflict, status/unmerged/raw evidence is preserved, the bounded 14-path cleanup restores the exact clean parent preimage, and execution stops. Do not edit or manually resolve any staged file.

- [ ] **Step 3: Prove the staged postimage and complete hunk stream**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
try {
$expectedPaths = @($manifest.entries.path | Sort-Object)
$actualPaths = @(git diff --cached --name-only --no-renames | Sort-Object)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate staged paths.' }
$pathDelta = @(Compare-Object $expectedPaths $actualPaths)
if ($pathDelta.Count -ne 0) {
  throw "Staged allowlist mismatch: $($pathDelta | Out-String)"
}

foreach ($entry in $manifest.entries) {
  $line = @(git ls-files --stage -- $entry.path)
  if ($LASTEXITCODE -ne 0) { throw "Could not inspect staged $($entry.path)" }
  if ($null -eq $entry.candidate_blob) {
    if ($line.Count -ne 0) { throw "Deleted path remains staged: $($entry.path)" }
  } else {
    if ($line.Count -ne 1 -or $line[0] -notmatch `
        '^(?<mode>[0-9]{6}) (?<blob>[0-9a-f]{40}) 0\t' -or
        $Matches.mode -ne $entry.candidate_mode -or
        $Matches.blob -ne $entry.candidate_blob) {
      throw "Material staged postimage mismatch: $($entry.path)"
    }
  }
}

$indexTreeRaw = @(git write-tree)
$indexTreeCode = $LASTEXITCODE
if ($indexTreeCode -ne 0 -or $indexTreeRaw.Count -ne 1) {
  throw 'Could not materialize the staged tree.'
}
$indexTree = ([string]$indexTreeRaw[0]).Trim()
$tauriLine = @(git ls-tree $indexTree -- src-tauri)
if ($LASTEXITCODE -ne 0 -or $tauriLine.Count -ne 1 -or
    $tauriLine[0] -notmatch '^040000 tree (?<tree>[0-9a-f]{40})\t' -or
    $Matches.tree -ne $manifest.candidate_src_tauri_tree) {
  throw 'Staged src-tauri subtree differs from the historical candidate.'
}

$stagedPatchBlobRaw = @(cmd.exe /d /c `
  "git diff --cached -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ HEAD | git hash-object --stdin")
$stagedPatchBlobCode = $LASTEXITCODE
$stagedPatchBlob = if ($stagedPatchBlobRaw.Count -eq 1) {
  ([string]$stagedPatchBlobRaw[0]).Trim()
} else { $null }
if ($stagedPatchBlobCode -ne 0 -or $stagedPatchBlobRaw.Count -ne 1 -or
    $stagedPatchBlob -ne $manifest.no_renames_patch_blob) {
  throw "Staged binary patch mismatch: $stagedPatchBlob"
}
$stagedPatchIdRaw = @(cmd.exe /d /c `
  "git diff --cached -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ HEAD | git patch-id --stable")
$stagedPatchIdCode = $LASTEXITCODE
$stagedPatchId = if ($stagedPatchIdRaw.Count -eq 1) {
  (([string]$stagedPatchIdRaw[0]).Trim() -split '\s+')[0]
} else { $null }
if ($stagedPatchIdCode -ne 0 -or $stagedPatchIdRaw.Count -ne 1 -or
    $stagedPatchId -ne $manifest.no_renames_patch_id) {
  throw "Staged patch-id mismatch: $stagedPatchId"
}
git diff --cached --exit-code $manifest.historical_candidate -- $expectedPaths
if ($LASTEXITCODE -ne 0) {
  throw 'Staged candidate paths differ from the historical candidate tree.'
}
if (@(git diff --name-only).Count -ne 0) {
  throw 'Unstaged candidate drift exists.'
}

[ordered]@{
  checked_at = [DateTimeOffset]::Now.ToString('o')
  paths = $actualPaths.Count
  index_tree = $indexTree
  src_tauri_tree = $Matches.tree
  patch_blob = $stagedPatchBlob
  patch_id = $stagedPatchId
  result = '14/14 staged postimages match'
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath `
  (Join-Path $scratch 'identity-staged.json')
} catch {
  $proofFailure = $_
  @(git status --porcelain=v1) | Set-Content -LiteralPath `
    (Join-Path $scratch 'staged-identity-failure-status.txt')
  @(git ls-files -u) | Set-Content -LiteralPath `
    (Join-Path $scratch 'staged-identity-failure-unmerged-index.txt')
  @(git diff --raw) | Set-Content -LiteralPath `
    (Join-Path $scratch 'staged-identity-failure-worktree-raw.txt')
  @(git diff --cached --raw) | Set-Content -LiteralPath `
    (Join-Path $scratch 'staged-identity-failure-index-raw.txt')
  $paths = @($manifest.entries.path)
  git restore --source=HEAD --staged --worktree -- $paths
  $cleanupRestoreCode = $LASTEXITCODE
  $cleanupStatus = @(git status --porcelain=v1)
  $cleanupStatusCode = $LASTEXITCODE
  $cleanupTauriRaw = @(git rev-parse 'HEAD:src-tauri')
  $cleanupTauriCode = $LASTEXITCODE
  $cleanupTauri = if ($cleanupTauriRaw.Count -eq 1) {
    ([string]$cleanupTauriRaw[0]).Trim()
  } else { $null }
  if ($cleanupRestoreCode -ne 0 -or $cleanupStatusCode -ne 0 -or
      $cleanupTauriCode -ne 0 -or $cleanupTauriRaw.Count -ne 1 -or
      $cleanupStatus.Count -ne 0 -or
      $cleanupTauri -ne $manifest.parent_src_tauri_tree) {
    throw 'Staged-identity cleanup failed; stop without manual repair.'
  }
  throw "Staged identity failed; exact preimage restored: $($proofFailure.Exception.Message)"
}
```

Expected: exactly 14 staged paths, candidate `src-tauri` tree, exact patch OID/id, no unstaged drift, and `14/14` postimages.

- [ ] **Step 4: Commit the proven exact patch before running candidate gates**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$paths = @($manifest.entries.path)
function Stop-PrecommitReplay([string]$message) {
  @(git status --porcelain=v1) | Set-Content -LiteralPath `
    (Join-Path $scratch 'precommit-failure-status.txt')
  @(git diff --cached --raw) | Set-Content -LiteralPath `
    (Join-Path $scratch 'precommit-failure-index-raw.txt')
  git restore --source=HEAD --staged --worktree -- $paths
  $cleanupRestoreCode = $LASTEXITCODE
  $cleanupStatus = @(git status --porcelain=v1)
  $cleanupStatusCode = $LASTEXITCODE
  $cleanupTauriRaw = @(git rev-parse 'HEAD:src-tauri')
  $cleanupTauriCode = $LASTEXITCODE
  $cleanupTauri = if ($cleanupTauriRaw.Count -eq 1) {
    ([string]$cleanupTauriRaw[0]).Trim()
  } else { $null }
  if ($cleanupRestoreCode -ne 0 -or $cleanupStatusCode -ne 0 -or
      $cleanupTauriCode -ne 0 -or $cleanupTauriRaw.Count -ne 1 -or
      $cleanupStatus.Count -ne 0 -or
      $cleanupTauri -ne $manifest.parent_src_tauri_tree) {
    throw 'Precommit replay cleanup failed; stop without manual repair.'
  }
  throw "$message Exact parent preimage restored."
}
$expectedPaths = @($paths | Sort-Object)
$actualPaths = @(git diff --cached --name-only --no-renames | Sort-Object)
if (@(Compare-Object $expectedPaths $actualPaths).Count -ne 0 -or
    @(git diff --name-only).Count -ne 0) {
  Stop-PrecommitReplay 'Candidate commit scope changed after staged proof.'
}
$patchBlobRaw = @(cmd.exe /d /c `
  "git diff --cached -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ HEAD | git hash-object --stdin")
$patchBlobCode = $LASTEXITCODE
$patchBlob = if ($patchBlobRaw.Count -eq 1) {
  ([string]$patchBlobRaw[0]).Trim()
} else { $null }
if ($patchBlobCode -ne 0 -or $patchBlobRaw.Count -ne 1 -or
    $patchBlob -ne $manifest.no_renames_patch_blob) {
  Stop-PrecommitReplay 'Candidate staged patch drifted before commit.'
}
git diff --cached --check
if ($LASTEXITCODE -ne 0) {
  Stop-PrecommitReplay 'Candidate staged diff has whitespace errors.'
}
git commit -C $manifest.historical_candidate
if ($LASTEXITCODE -ne 0) { Stop-PrecommitReplay 'Exact candidate commit failed.' }
$reapplicationRaw = @(git rev-parse HEAD)
$reapplicationCode = $LASTEXITCODE
if ($reapplicationCode -ne 0 -or $reapplicationRaw.Count -ne 1) {
  throw 'Exact candidate was committed but its commit id could not be resolved.'
}
$reapplicationCommit = ([string]$reapplicationRaw[0]).Trim()
$reapplicationCommit | Set-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt')
```

Expected: one code-only commit with the historical author/message and exact 14-path patch. Committing here ensures every later confirmed correctness failure has one unified, auditable revert path. Do not amend it.

- [ ] **Step 5: Prove the freshly committed replay before executing it**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
function Stop-CommittedIdentity([string]$message) {
  [ordered]@{
    gate = 'committed-candidate-identity'
    classification = 'state_invalid'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'identity-failure.json')
  throw $message
}
function Stop-IdentityInfrastructure([string]$message) {
  [ordered]@{
    gate = 'committed-candidate-identity-probe'
    classification = 'infrastructure_invalid'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'candidate-gate-infrastructure-failure.json')
  throw "$message Do not enter the negative branch."
}
$historicalRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev $manifest.historical_candidate)
if ($LASTEXITCODE -ne 0) {
  Stop-IdentityInfrastructure 'Could not compute the historical raw manifest.'
}
$reappliedRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev HEAD)
if ($LASTEXITCODE -ne 0) {
  Stop-IdentityInfrastructure 'Could not compute the reapplication raw manifest.'
}
if (@(Compare-Object $historicalRaw $reappliedRaw).Count -ne 0 -or
    $reappliedRaw.Count -ne 14) {
  Stop-CommittedIdentity 'Freshly committed raw manifest differs from the historical candidate.'
}
$freshTauriRaw = @(git rev-parse 'HEAD:src-tauri')
$freshTauriCode = $LASTEXITCODE
if ($freshTauriCode -ne 0 -or $freshTauriRaw.Count -ne 1) {
  Stop-IdentityInfrastructure 'Could not read the freshly committed src-tauri tree.'
}
$freshTauri = ([string]$freshTauriRaw[0]).Trim()
if ($freshTauri -ne
    $manifest.candidate_src_tauri_tree) {
  Stop-CommittedIdentity 'Freshly committed src-tauri tree mismatch.'
}
$commitPatchBlobRaw = @(cmd.exe /d /c `
  "git show --format= -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ HEAD | git hash-object --stdin")
$patchBlobCode = $LASTEXITCODE
$commitPatchIdRaw = @(cmd.exe /d /c `
  "git show --format= -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ HEAD | git patch-id --stable")
$patchIdCode = $LASTEXITCODE
$commitPatchBlob = if ($commitPatchBlobRaw.Count -eq 1) {
  ([string]$commitPatchBlobRaw[0]).Trim()
} else { $null }
$commitPatchId = if ($commitPatchIdRaw.Count -eq 1) {
  (([string]$commitPatchIdRaw[0]).Trim() -split '\s+')[0]
} else { $null }
if ($patchBlobCode -ne 0 -or $patchIdCode -ne 0 -or
    $commitPatchBlobRaw.Count -ne 1 -or $commitPatchIdRaw.Count -ne 1) {
  Stop-IdentityInfrastructure 'Could not compute canonical committed patch fingerprints.'
}
$status = @(git status --porcelain=v1)
if ($LASTEXITCODE -ne 0) {
  Stop-IdentityInfrastructure 'Could not inspect post-commit worktree status.'
}
if ($commitPatchBlob -ne $manifest.no_renames_patch_blob -or
    $commitPatchId -ne $manifest.no_renames_patch_id -or $status.Count -ne 0) {
  Stop-CommittedIdentity 'Freshly committed patch fingerprint or cleanliness mismatch.'
}
```

Expected: 14 raw entries, exact subtree, canonical patch OID/id, and a clean worktree. An identity failure enters Task 4 Step 5; never repair the commit.

- [ ] **Step 6: Run candidate contracts and the focused Rust GREEN loop**

Run sequentially and stop on the first failure; do not edit the candidate:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
function Invoke-CandidateGate([string]$name, [scriptblock]$command) {
  $savedErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  $startFailure = $null
  try {
    $output = @(& $command 2>&1)
    $code = $LASTEXITCODE
  } catch {
    $output = @($_.Exception.Message)
    $code = -1
    $startFailure = $_.Exception.Message
  } finally {
    $ErrorActionPreference = $savedErrorActionPreference
  }
  $log = Join-Path $scratch "candidate-$name.log"
  $output | Set-Content -LiteralPath $log
  if ($code -ne 0) {
    $text = $output -join "`n"
    $infrastructure = $null -ne $startFailure -or
      $text -match '(?i)(\.cargo-lock|access(?: is)? denied|os error 5|used by another process|cannot access the file)'
    $artifact = if ($infrastructure) {
      'candidate-gate-infrastructure-failure.json'
    } else {
      'candidate-correctness-failure.json'
    }
    [ordered]@{
      gate = $name
      exit_code = $code
      classification = if ($infrastructure) { 'infrastructure_invalid' } else { 'command_failed' }
      log = $log
      command_start_error = $startFailure
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch $artifact)
    if ($infrastructure) {
      throw "Candidate gate infrastructure failure: $name; do not enter the negative branch."
    }
    throw "Confirmed candidate correctness failure: $name"
  }
  return ,$output
}
function Stop-CandidateAssertion([string]$gate, [string]$message) {
  [ordered]@{
    gate = $gate
    exit_code = 0
    classification = 'command_failed'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'candidate-correctness-failure.json')
  throw $message
}

$metadataOutput = Invoke-CandidateGate 'locked-metadata' {
  cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 --no-deps
}
$metadataJsonLines = @($metadataOutput | ForEach-Object { [string]$_ } |
  Where-Object { $_.TrimStart().StartsWith('{') })
if ($metadataJsonLines.Count -ne 1) {
  Stop-CandidateAssertion 'candidate-metadata-json' `
    'Candidate metadata did not emit exactly one JSON document.'
}
$metadata = $metadataJsonLines[0] | ConvertFrom-Json
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
$expectedWorkspace = [IO.Path]::GetFullPath((Join-Path (Get-Location) 'src-tauri')).TrimEnd('\')
$expectedTarget = [IO.Path]::GetFullPath((Join-Path $expectedWorkspace 'target')).TrimEnd('\')
$actualWorkspace = [IO.Path]::GetFullPath([string]$metadata.workspace_root).TrimEnd('\')
$actualTarget = [IO.Path]::GetFullPath([string]$metadata.target_directory).TrimEnd('\')
if ($actualWorkspace -ne $expectedWorkspace -or $actualTarget -ne $expectedTarget -or
    $actualTarget -eq [IO.Path]::GetFullPath([string]$environment.main_target).TrimEnd('\')) {
  Stop-CandidateAssertion 'candidate-target-isolation' `
    'Candidate Cargo metadata redirected workspace or target.'
}
$null = Invoke-CandidateGate 'source-contracts' {
  npm.cmd run test -- `
    src/lib/process-crate-reapplication-identity-contract.test.ts `
    src/lib/process-crate-boundary-contract.test.ts `
    src/lib/external-process-lifecycle-contract.test.ts `
    src/lib/hidden-child-process-contract.test.ts
}
$greenOutput = Invoke-CandidateGate 'narrow-green' {
  cargo test --manifest-path src-tauri/Cargo.toml --locked `
    -p extractum-process --lib `
    external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets `
    -- --exact
}
if (($greenOutput -join "`n") -notmatch 'test result: ok\. 1 passed') {
  Stop-CandidateAssertion 'narrow-green-selection' `
    'Narrow GREEN did not report exactly one passing test.'
}
$null = Invoke-CandidateGate 'process-check' {
  cargo check --manifest-path src-tauri/Cargo.toml --locked `
    -p extractum-process --all-targets
}
$processOutput = Invoke-CandidateGate 'process-tests' {
  cargo test --manifest-path src-tauri/Cargo.toml --locked `
    -p extractum-process --all-targets
}
if (($processOutput -join "`n") -notmatch 'test result: ok\. 20 passed') {
  Stop-CandidateAssertion 'process-test-count' `
    'Process package checkpoint did not report exactly 20 passing tests.'
}
$null = Invoke-CandidateGate 'dependent-check' {
  cargo check --manifest-path src-tauri/Cargo.toml --locked `
    -p extractum --all-targets
}
$youtubeOutput = Invoke-CandidateGate 'youtube-process-runtime' {
  cargo test --manifest-path src-tauri/Cargo.toml --locked `
    -p extractum --lib youtube::process_runtime::
}
if (($youtubeOutput -join "`n") -notmatch 'test result: ok\. [1-9][0-9]* passed') {
  Stop-CandidateAssertion 'youtube-test-selection' `
    'YouTube process-runtime checkpoint selected zero tests.'
}
```

Expected: four Vitest files pass, narrow Rust test is exactly `1/1`, the process package reports `20 passed`, and both app checkpoints pass. A candidate failure closes this exact replay; do not patch it.

- [ ] **Step 7: Compare fresh inventories and unchanged consumers**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
function Stop-InventoryFailure(
  [string]$message,
  [bool]$infrastructure,
  [AllowNull()][string]$log
) {
  $artifact = if ($infrastructure) {
    'candidate-gate-infrastructure-failure.json'
  } else {
    'candidate-correctness-failure.json'
  }
  [ordered]@{
    gate = 'workspace-test-inventory'
    classification = if ($infrastructure) { 'infrastructure_invalid' } else { 'command_failed' }
    error = $message
    log = $log
  } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch $artifact)
  throw $message
}
$baselineNames = @(Get-Content -LiteralPath `
  (Join-Path $scratch 'baseline-test-names.txt'))
$baselineProcess = @(Get-Content -LiteralPath `
  (Join-Path $scratch 'baseline-process-test-names.txt'))
$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$inventoryStartFailure = $null
try {
  $postInventory = @(& cargo test --manifest-path src-tauri/Cargo.toml --locked `
    --workspace --all-targets -- --list 2>&1)
  $postInventoryCode = $LASTEXITCODE
} catch {
  $postInventory = @($_.Exception.Message)
  $postInventoryCode = -1
  $inventoryStartFailure = $_.Exception.Message
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
$inventoryLog = Join-Path $scratch 'candidate-workspace-inventory.log'
$postInventory | Set-Content -LiteralPath $inventoryLog
if ($postInventoryCode -ne 0) {
  $inventoryText = $postInventory -join "`n"
  $infrastructure = $null -ne $inventoryStartFailure -or $inventoryText -match `
    '(?i)(\.cargo-lock|access(?: is)? denied|os error 5|used by another process|cannot access the file)'
  Stop-InventoryFailure 'Candidate workspace inventory command failed.' `
    $infrastructure $inventoryLog
}
$postNamesRaw = @($postInventory | Where-Object { $_ -match ': test$' } |
  ForEach-Object { ($_ -replace ': test$', '').Trim() })
$postNames = @($postNamesRaw | Sort-Object -Unique)
if ($postNamesRaw.Count -ne $postNames.Count) {
  Stop-InventoryFailure 'Candidate inventory contains duplicate test names.' `
    $false $inventoryLog
}
$postProcess = @($postNames | Where-Object {
  $_ -match '^(external_process|child_process|process_tree)::'
})
$missing = @($baselineNames | Where-Object { $_ -notin $postNames })
$extra = @($postNames | Where-Object { $_ -notin $baselineNames })
$extraProcess = @($postProcess | Where-Object { $_ -notin $baselineProcess })
$missingProcess = @($baselineProcess | Where-Object { $_ -notin $postProcess })
if ($missing.Count -ne 0 -or $extra.Count -ne 0 -or
    $postNames.Count -ne $baselineNames.Count -or
    $postProcess.Count -ne 20 -or $extraProcess.Count -ne 0 -or
    $missingProcess.Count -ne 0) {
  Stop-InventoryFailure 'Candidate test inventory lost, duplicated, or renamed tests.' `
    $false $inventoryLog
}

$baselineConsumersParsed = Get-Content -LiteralPath `
  (Join-Path $scratch 'baseline-consumer-hashes.json') -Raw | ConvertFrom-Json
$baselineConsumers = @($baselineConsumersParsed)
$consumerDrift = @($baselineConsumers | Where-Object {
  -not (Test-Path -LiteralPath $_.path) -or
  (Get-FileHash -LiteralPath $_.path -Algorithm SHA256).Hash -ne $_.sha256
})
if ($baselineConsumers.Count -ne 12 -or $consumerDrift.Count -ne 0) {
  Stop-InventoryFailure 'One or more of the 12 consumers changed.' $false $null
}
[ordered]@{
  baseline_total = $baselineNames.Count
  candidate_total = $postNames.Count
  missing = $missing.Count
  extra = $extra.Count
  baseline_unique = $baselineNames.Count
  candidate_raw = $postNamesRaw.Count
  candidate_unique = $postNames.Count
  process_before = $baselineProcess.Count
  process_after = $postProcess.Count
  consumers_unchanged = $baselineConsumers.Count
} | ConvertTo-Json | Set-Content -LiteralPath `
  (Join-Path $scratch 'inventory-comparison.json')
```

Expected: baseline and candidate raw inventories contain no duplicate names, their complete unique sets and totals are exactly equal, process names are exactly `20/20`, and consumers are `12/12` unchanged.

- [ ] **Step 8: Checkpoint the committed candidate after focused gates**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
function Stop-PostGateIdentity([string]$message) {
  [ordered]@{
    gate = 'post-focused-candidate-identity'
    classification = 'state_invalid'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'identity-failure.json')
  throw $message
}
function Stop-PostGateInfrastructure([string]$message) {
  [ordered]@{
    gate = 'post-focused-identity-probe'
    classification = 'infrastructure_invalid'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'candidate-gate-infrastructure-failure.json')
  throw "$message Do not enter the negative branch."
}
$headCommitRaw = @(git rev-parse HEAD)
$headCommitCode = $LASTEXITCODE
if ($headCommitCode -ne 0 -or $headCommitRaw.Count -ne 1) {
  Stop-PostGateInfrastructure 'Could not read HEAD.'
}
$headCommit = ([string]$headCommitRaw[0]).Trim()
$status = @(git status --porcelain=v1)
if ($LASTEXITCODE -ne 0) { Stop-PostGateInfrastructure 'Could not read status.' }
if ($headCommit -ne $reapplicationCommit -or $status.Count -ne 0) {
  Stop-PostGateIdentity 'Focused gates changed HEAD or repository bytes.'
}
$headLockRaw = @(git rev-parse 'HEAD:src-tauri/Cargo.lock')
$headLockCode = $LASTEXITCODE
if ($headLockCode -ne 0 -or $headLockRaw.Count -ne 1) {
  Stop-PostGateInfrastructure 'Could not read candidate Cargo.lock identity.'
}
$headLock = ([string]$headLockRaw[0]).Trim()
if ($headLock -ne
    '6368e32cd3a3853d4a7114ce256258e834bafdd4') {
  Stop-PostGateIdentity 'Candidate Cargo.lock drifted during focused gates.'
}
$headTauriRaw = @(git rev-parse 'HEAD:src-tauri')
$headTauriCode = $LASTEXITCODE
if ($headTauriCode -ne 0 -or $headTauriRaw.Count -ne 1) {
  Stop-PostGateInfrastructure 'Could not read candidate src-tauri identity.'
}
$headTauri = ([string]$headTauriRaw[0]).Trim()
if ($headTauri -ne
    $manifest.candidate_src_tauri_tree) {
  Stop-PostGateIdentity 'Candidate src-tauri tree drifted during focused gates.'
}
if (-not (Test-Path -LiteralPath (Join-Path $scratch 'inventory-comparison.json'))) {
  Stop-PostGateInfrastructure 'Focused inventory evidence is missing.'
}
```

Expected: HEAD is still the exact reapplication commit, the worktree is clean, lock/subtree identity is unchanged, and inventory evidence exists.

- [ ] **Step 9: Reprove the committed replay after focused execution**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
function Stop-FinalCandidateIdentity([string]$message) {
  [ordered]@{
    gate = 'final-focused-candidate-identity'
    classification = 'state_invalid'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'identity-failure.json')
  throw $message
}
function Stop-FinalIdentityInfrastructure([string]$message) {
  [ordered]@{
    gate = 'final-focused-identity-probe'
    classification = 'infrastructure_invalid'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'candidate-gate-infrastructure-failure.json')
  throw "$message Do not enter the negative branch."
}
$historicalRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev $manifest.historical_candidate)
if ($LASTEXITCODE -ne 0) {
  Stop-FinalIdentityInfrastructure 'Could not read historical raw diff.'
}
$reappliedRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev HEAD)
if ($LASTEXITCODE -ne 0) {
  Stop-FinalIdentityInfrastructure 'Could not read reapplication raw diff.'
}
$rawDelta = @(Compare-Object $historicalRaw $reappliedRaw)
if ($rawDelta.Count -ne 0 -or $reappliedRaw.Count -ne 14) {
  Stop-FinalCandidateIdentity "Committed raw manifest mismatch: $($rawDelta | Out-String)"
}
$finalTauriRaw = @(git rev-parse 'HEAD:src-tauri')
$finalTauriCode = $LASTEXITCODE
if ($finalTauriCode -ne 0 -or $finalTauriRaw.Count -ne 1) {
  Stop-FinalIdentityInfrastructure 'Could not read final candidate src-tauri tree.'
}
$finalTauri = ([string]$finalTauriRaw[0]).Trim()
if ($finalTauri -ne
    $manifest.candidate_src_tauri_tree) {
  Stop-FinalCandidateIdentity 'Committed src-tauri tree mismatch.'
}
$commitPatchBlobRaw = @(cmd.exe /d /c `
  "git show --format= -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ HEAD | git hash-object --stdin")
$patchBlobCode = $LASTEXITCODE
if ($patchBlobCode -ne 0 -or $commitPatchBlobRaw.Count -ne 1) {
  Stop-FinalIdentityInfrastructure 'Could not compute the committed patch blob.'
}
$commitPatchBlob = ([string]$commitPatchBlobRaw[0]).Trim()
if ($commitPatchBlob -ne $manifest.no_renames_patch_blob) {
  Stop-FinalCandidateIdentity 'Committed patch blob mismatch.'
}
$commitPatchIdRaw = @(cmd.exe /d /c `
  "git show --format= -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ HEAD | git patch-id --stable")
$patchIdCode = $LASTEXITCODE
if ($patchIdCode -ne 0 -or $commitPatchIdRaw.Count -ne 1) {
  Stop-FinalIdentityInfrastructure 'Could not compute the committed patch-id.'
}
$commitPatchId = (([string]$commitPatchIdRaw[0]).Trim() -split '\s+')[0]
if ($commitPatchId -ne $manifest.no_renames_patch_id) {
  Stop-FinalCandidateIdentity 'Committed patch-id mismatch.'
}
$status = @(git status --porcelain=v1)
if ($LASTEXITCODE -ne 0) {
  Stop-FinalIdentityInfrastructure 'Could not inspect final candidate status.'
}
if ($status.Count -ne 0) {
  Stop-FinalCandidateIdentity 'Worktree is dirty after candidate commit.'
}
```

Expected: 14 raw entries match byte-for-byte, `src-tauri` tree is exact, patch OID/id match, and the worktree is clean.

### Task 4: Record diagnostics, run completion gates, and close Phase 3

**Files:**

- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Create: `docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Read: all Task 2 scratch artifacts and the Task 3 reapplication commit

**Interfaces:**

- Consumes: the clean exact code commit, baseline series, inventory, consumer hashes, and identity manifest.
- Produces: post series, validity/cumulative classification, full completion evidence, retained or reverted outcome, and the Phase 4 prerequisite state.

- [ ] **Step 1: Run the single predeclared post-reapplication shell series**

Run from the clean candidate commit:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$postRepoRaw = @(git rev-parse --show-toplevel)
$postRepoCode = $LASTEXITCODE
$postHeadRaw = @(git rev-parse HEAD)
$postHeadCode = $LASTEXITCODE
$postStatus = @(git status --porcelain=v1)
$postStatusCode = $LASTEXITCODE
$postHead = if ($postHeadRaw.Count -eq 1) {
  ([string]$postHeadRaw[0]).Trim()
} else { $null }
if ($postRepoCode -ne 0 -or $postHeadCode -ne 0 -or
    $postStatusCode -ne 0 -or $postRepoRaw.Count -ne 1) {
  throw 'Post-series Git preflight failed as infrastructure; do not start diagnostics.'
}
if ($postHeadRaw.Count -ne 1 -or $postHead -ne $reapplicationCommit -or
    $postStatus.Count -ne 0) {
  throw 'Post series requires the clean exact reapplication commit at HEAD.'
}
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
if (-not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
  throw 'CARGO_TARGET_DIR changed after environment capture.'
}
$postRepo = [IO.Path]::GetFullPath(
  ([string]$postRepoRaw[0]).Trim()).TrimEnd('\')
$postTarget = [IO.Path]::GetFullPath(
  (Join-Path $postRepo 'src-tauri/target')).TrimEnd('\')
if ($postRepo -ne [string]$environment.repository -or
    $postTarget -ne [string]$environment.target) {
  throw 'Post claim requires the captured worktree and canonical target.'
}
$qualificationPath = Join-Path $scratch 'runner-qualification.json'
$qualification = Get-Content -LiteralPath $qualificationPath -Raw |
  ConvertFrom-Json
$gitCommonRaw = @(git rev-parse --git-common-dir)
if ($LASTEXITCODE -ne 0 -or $gitCommonRaw.Count -ne 1) {
  throw 'Could not recompute the shared Git directory before post claim.'
}
$gitCommon = (Resolve-Path -LiteralPath ([string]$gitCommonRaw[0])).Path
$claimMaterial = '{0}|{1}' -f $gitCommon.ToLowerInvariant(), `
  $environment.implementation_base
$claimHasher = [Security.Cryptography.SHA256]::Create()
try {
  $claimKey = -join ($claimHasher.ComputeHash(
    [Text.Encoding]::UTF8.GetBytes($claimMaterial)) | ForEach-Object {
      $_.ToString('x2')
    })
} finally { $claimHasher.Dispose() }
$claimRoot = Join-Path $env:TEMP `
  "extractum-process-reapplication-claims/$claimKey"
$persistedClaimRoot = (Get-Content -LiteralPath `
  (Join-Path $scratch 'diagnostic-claim-root.txt') -Raw).Trim()
if ($persistedClaimRoot -ne $claimRoot -or
    [string]$environment.diagnostic_claim_root -ne $claimRoot) {
  throw 'Persisted diagnostic claim root does not match the recomputed stable root.'
}
$artifactFailurePath = Join-Path $claimRoot `
  'diagnostic-artifact-failure.json'
if (Test-Path -LiteralPath $artifactFailurePath) {
  $artifactFailure = Get-Content -LiteralPath $artifactFailurePath -Raw |
    ConvertFrom-Json
  if ($artifactFailure.stage -ne 'diagnostic-artifact-failure' -or
      $artifactFailure.scratch -ne $scratch -or
      -not [bool]$artifactFailure.source_restored -or
      -not [bool]$artifactFailure.termination_confirmed) {
    throw 'Diagnostic artifact-failure routing is unsafe or stale.'
  }
  [ordered]@{
    stage = 'post'
    classification = 'infrastructure_invalid'
    skipped = $true
    reason = 'baseline diagnostic artifact chain already failed; post was not claimed or launched'
    artifact_failure_sha256 =
      (Get-FileHash -LiteralPath $artifactFailurePath -Algorithm SHA256).Hash
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'post-skipped.json')
  return
}
$baselineClaimPath = Join-Path $claimRoot 'baseline.json'
$baselineClaim = Get-Content -LiteralPath $baselineClaimPath -Raw |
  ConvertFrom-Json
$environmentClaimPath = Join-Path $claimRoot 'environment-preflight.json'
$environmentClaim = Get-Content -LiteralPath $environmentClaimPath -Raw |
  ConvertFrom-Json
$environmentClaimHash =
  (Get-FileHash -LiteralPath $environmentClaimPath -Algorithm SHA256).Hash
$environmentHash = (Get-FileHash -LiteralPath `
  (Join-Path $scratch 'environment.json') -Algorithm SHA256).Hash
$initialQuietPath = Join-Path $scratch 'quiet-initial.json'
$initialQuietHash =
  (Get-FileHash -LiteralPath $initialQuietPath -Algorithm SHA256).Hash
$currentQualificationHash =
  (Get-FileHash -LiteralPath $qualificationPath -Algorithm SHA256).Hash
$baselineClaimHash =
  (Get-FileHash -LiteralPath $baselineClaimPath -Algorithm SHA256).Hash
$baselineAttempt = (Get-Content -LiteralPath `
  (Join-Path $scratch 'baseline-current.txt') -Raw).Trim()
$baselineSummaryPath = Join-Path $baselineAttempt 'summary.json'
$baselineCompletionPath = Join-Path $baselineAttempt 'baseline-completion.json'
$baselineSummary = Get-Content -LiteralPath $baselineSummaryPath -Raw |
  ConvertFrom-Json
$baselineCompletion = Get-Content -LiteralPath $baselineCompletionPath -Raw |
  ConvertFrom-Json
$baselineSummaryHash =
  (Get-FileHash -LiteralPath $baselineSummaryPath -Algorithm SHA256).Hash
$baselineCompletionHash =
  (Get-FileHash -LiteralPath $baselineCompletionPath -Algorithm SHA256).Hash
function Test-CompletedSeriesSummary(
  [object]$value,
  [string]$expectedStage,
  [string]$expectedSourceHash
) {
  $samples = @($value.samples_ms)
  if ($value.stage -ne $expectedStage -or
      $value.source_sha256 -cne $expectedSourceHash -or
      [int]$value.required_stable_count -ne 4 -or
      [int]$value.max_absolute_deviation_ms -ne 300 -or
      @($samples | Where-Object { $null -eq $_ }).Count -ne 0) {
    return $false
  }
  if ($null -ne $value.median_ms -and $null -ne $value.stable_count) {
    if ($samples.Count -ne 5) { return $false }
    $sorted = @($samples | ForEach-Object { [int64]$_ } | Sort-Object)
    $median = [int64]$sorted[2]
    $stableCount = @($samples | Where-Object {
      [Math]::Abs([int64]$_ - $median) -le 300
    }).Count
    return [int64]$value.median_ms -eq $median -and
      [int]$value.stable_count -eq $stableCount -and
      [bool]$value.series_valid -eq ($stableCount -ge 4)
  }
  return -not [bool]$value.series_valid -and
    $null -eq $value.median_ms -and $null -eq $value.stable_count -and
    $samples.Count -le 5 -and
    $null -ne $value.PSObject.Properties['invalid_reason'] -and
    -not [string]::IsNullOrWhiteSpace([string]$value.invalid_reason)
}
if (-not [bool]$qualification.passed -or
    -not [bool]$qualification.termination_confirmed -or
    $environmentClaim.stage -ne 'environment-preflight' -or
    $environmentClaim.scratch -ne $scratch -or
    $environmentClaim.repository -ne $postRepo -or
    $environmentClaim.target -ne $postTarget -or
    $environmentClaim.identity_commit -ne $environment.identity_commit -or
    $environmentClaim.implementation_base -ne
      $environment.implementation_base -or
    $environmentClaim.quiet_artifact -ne $initialQuietPath -or
    $environmentClaimHash -cne
      $environment.environment_preflight_claim_sha256 -or
    $baselineClaim.stage -ne 'baseline' -or
    $baselineClaim.scratch -ne $scratch -or
    $baselineClaim.attempt -ne $baselineAttempt -or
    $baselineClaim.repository -ne $postRepo -or
    $baselineClaim.target -ne $postTarget -or
    $baselineClaim.head_commit -ne $environment.identity_commit -or
    $baselineClaim.qualification_sha256 -cne $currentQualificationHash -or
    $baselineClaim.environment_preflight_claim_sha256 -cne
      $environmentClaimHash -or
    $baselineClaim.environment_sha256 -cne $environmentHash -or
    $baselineClaim.initial_quiet_sha256 -cne $initialQuietHash -or
    $baselineClaim.implementation_base -ne $environment.implementation_base -or
    $baselineClaim.identity_commit -ne $environment.identity_commit -or
    $baselineClaim.runner_sha256 -cne $qualification.runner_sha256 -or
    $baselineSummary.source_sha256 -cne $baselineClaim.source_sha256 -or
    -not (Test-CompletedSeriesSummary $baselineSummary 'baseline' `
      $baselineClaim.source_sha256) -or
    $baselineCompletion.stage -ne 'baseline-completion' -or
    $baselineCompletion.scratch -ne $scratch -or
    $baselineCompletion.attempt -ne $baselineAttempt -or
    $baselineCompletion.repository -ne $postRepo -or
    $baselineCompletion.target -ne $postTarget -or
    $baselineCompletion.head_commit -ne $environment.identity_commit -or
    $baselineCompletion.baseline_claim_sha256 -cne $baselineClaimHash -or
    $baselineCompletion.summary_sha256 -cne $baselineSummaryHash -or
    $baselineCompletion.qualification_sha256 -cne
      $currentQualificationHash -or
    $baselineCompletion.runner_sha256 -cne $qualification.runner_sha256 -or
    $baselineCompletion.source_sha256 -cne $baselineClaim.source_sha256 -or
    -not [bool]$baselineCompletion.source_restored -or
    -not [bool]$baselineCompletion.termination_confirmed -or
    [int]$baselineCompletion.probe_exit_code -notin @(0, 2) -or
    (Get-FileHash -LiteralPath (Join-Path $scratch 'job-object.ps1') `
      -Algorithm SHA256).Hash -cne $qualification.job_helper_sha256 -or
    (Get-FileHash -LiteralPath (Join-Path $scratch 'invoke-shell-series.ps1') `
      -Algorithm SHA256).Hash -cne $qualification.runner_sha256) {
  throw 'Post runner qualification or baseline claim is missing, failed, or stale; stop before claiming post.'
}
$sourcePath = 'src-tauri/src/lib.rs'
$sourceHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash
$attemptId = 'post-{0}-{1}' -f `
  ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')), `
  ([guid]::NewGuid().ToString('N'))
$attempt = Join-Path $scratch "attempts/$attemptId"
$postClaimPath = Join-Path $claimRoot 'post.json'
if (Test-Path -LiteralPath $postClaimPath) {
  throw "Zero-retry post series is already claimed at $postClaimPath; never launch quiet-window or Cargo again."
}
$postClaimPayload = [ordered]@{
  stage = 'post'
  claimed_at = [DateTimeOffset]::Now.ToString('o')
  scratch = $scratch
  attempt = $attempt
  repository = $postRepo
  target = $postTarget
  head_commit = $postHead
  reapplication_commit = $reapplicationCommit
  implementation_base = $environment.implementation_base
  identity_commit = $environment.identity_commit
  environment_preflight_claim_sha256 = $environmentClaimHash
  baseline_claim_sha256 = $baselineClaimHash
  baseline_completion_sha256 = $baselineCompletionHash
  baseline_summary_sha256 = $baselineSummaryHash
  qualification_sha256 = $currentQualificationHash
  runner_sha256 = $qualification.runner_sha256
  source_sha256 = $sourceHash
} | ConvertTo-Json
$postClaimBytes = [Text.UTF8Encoding]::new($false).GetBytes($postClaimPayload)
$postClaimTempPath = '{0}.{1}.tmp' -f `
  $postClaimPath, ([guid]::NewGuid().ToString('N'))
$postClaimStream = $null
try {
  $postClaimStream = [IO.File]::Open(
    $postClaimTempPath,
    [IO.FileMode]::CreateNew,
    [IO.FileAccess]::Write,
    [IO.FileShare]::None)
  $postClaimStream.Write($postClaimBytes, 0, $postClaimBytes.Length)
  $postClaimStream.Flush($true)
} catch [IO.IOException] {
  throw "Could not atomically claim the zero-retry post series: $($_.Exception.Message)"
} finally {
  if ($null -ne $postClaimStream) { $postClaimStream.Dispose() }
}
try {
  [IO.File]::Move($postClaimTempPath, $postClaimPath)
} catch [IO.IOException] {
  throw "Could not atomically publish the zero-retry post claim: $($_.Exception.Message)"
}
[ordered]@{ path = $sourcePath; sha256 = $sourceHash } |
  ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'post-source.json')
$attempt | Set-Content -LiteralPath (Join-Path $scratch 'post-current.txt')
$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File `
    (Join-Path $scratch 'assert-quiet-window.ps1') -ArtifactPath `
    (Join-Path $scratch 'quiet-post.json') `
    1> (Join-Path $scratch 'quiet-post.stdout.log') `
    2> (Join-Path $scratch 'quiet-post.stderr.log')
  $quietCode = $LASTEXITCODE
} catch {
  $quietCode = -1
} finally {
  $ErrorActionPreference = $savedErrorActionPreference
}
$diagnosticInvalidReason = if ($quietCode -ne 0) {
  'post quiet-window preflight failed after the one-shot post claim'
} else { $null }
if ($quietCode -ne 0) {
  New-Item -ItemType Directory -Path $attempt -Force | Out-Null
  [ordered]@{
    stage = 'post'
    classification = 'infrastructure_invalid'
    error = $diagnosticInvalidReason
    child_started = $false
    termination_confirmed = $true
    source_restored = $true
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $attempt 'runner-infrastructure-failure.json')
  $probeCode = 2
} else {
  $savedErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  $probeStartError = $null
  try {
    & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File `
      (Join-Path $scratch 'invoke-shell-series.ps1') -Stage 'post' `
      -Mode 'series' -SourcePath $sourcePath -ExpectedSha256 $sourceHash `
      -AttemptRoot $attempt `
      -JobHelperPath (Join-Path $scratch 'job-object.ps1') `
      1> (Join-Path $scratch 'post-runner.stdout.log') `
      2> (Join-Path $scratch 'post-runner.stderr.log')
    $probeCode = $LASTEXITCODE
  } catch {
    $probeCode = 2
    $probeStartError = $_.Exception.Message
  } finally {
    $ErrorActionPreference = $savedErrorActionPreference
  }
  if ($null -ne $probeStartError) {
    New-Item -ItemType Directory -Path $attempt -Force | Out-Null
    [ordered]@{
      stage = 'post'
      phase = 'runner-process-start'
      classification = 'infrastructure_invalid'
      error = $probeStartError
      child_started = $false
      termination_confirmed = $true
      source_restored = $true
    } | ConvertTo-Json | Set-Content -LiteralPath `
      (Join-Path $attempt 'runner-infrastructure-failure.json')
  }
}
if ((Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash -ne $sourceHash) {
  throw 'Post probe bytes were not restored; stop all child commands.'
}
if ($probeCode -eq 1) {
  $commandFailurePath = Join-Path $attempt 'failure.json'
  $commandFailure = if (Test-Path -LiteralPath $commandFailurePath) {
    Get-Content -LiteralPath $commandFailurePath -Raw | ConvertFrom-Json
  } else { $null }
  $failedMetaPath = if ($null -ne $commandFailure) {
    Join-Path $attempt "runs/$($commandFailure.failed_label)-meta.json"
  } else { $null }
  $failedMeta = if ($null -ne $failedMetaPath -and
      (Test-Path -LiteralPath $failedMetaPath)) {
    Get-Content -LiteralPath $failedMetaPath -Raw | ConvertFrom-Json
  } else { $null }
  $commandFailureProven = $null -ne $commandFailure -and
    $commandFailure.exit_code -eq 1 -and
    $commandFailure.stage -eq 'post' -and
    $commandFailure.classification -eq 'command_failed' -and
    [bool]$commandFailure.source_restored -and
    $null -ne $failedMeta -and $failedMeta.mode -eq 'series' -and
    $failedMeta.stage -eq 'post' -and
    $failedMeta.label -eq $commandFailure.failed_label -and
    $failedMeta.classification -eq 'command_failed' -and
    $failedMeta.error -like 'COMMAND_FAILURE:*' -and
    [bool]$failedMeta.restored -and [bool]$failedMeta.termination_confirmed
  if ($commandFailureProven) {
    [ordered]@{
      gate = 'post-shell-probe'
      exit_code = 1
      classification = 'command_failed'
      attempt = $attempt
    } | ConvertTo-Json | Set-Content -LiteralPath `
      (Join-Path $scratch 'candidate-correctness-failure.json')
    throw 'Confirmed candidate Cargo failure during post probe.'
  }
  [ordered]@{
    stage = 'post'
    phase = 'command-failure-routing'
    classification = 'infrastructure_invalid'
    error = 'Exit 1 lacked coherent command-failure/restoration/termination evidence.'
    child_started = ($null -ne $failedMeta)
    termination_confirmed = if ($null -ne $failedMeta) {
      [bool]$failedMeta.termination_confirmed
    } else { $false }
    source_restored = $true
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $attempt 'runner-infrastructure-failure.json')
  $probeCode = 2
}
if ($probeCode -eq 2) {
  $runMeta = @(Get-ChildItem -LiteralPath (Join-Path $attempt 'runs') `
    -Filter '*-meta.json' -ErrorAction SilentlyContinue | ForEach-Object {
      Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
    })
  if (@($runMeta | Where-Object { -not $_.termination_confirmed }).Count -ne 0) {
    throw 'Post child termination is unconfirmed; stop all further child commands.'
  }
  $partialSamples = @(Get-ChildItem -LiteralPath (Join-Path $attempt 'runs') `
    -Filter 'sample-*-meta.json' -ErrorAction SilentlyContinue |
    Sort-Object Name | ForEach-Object {
      $meta = Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
      if ($null -ne $meta.elapsed_ms -and $meta.timed_exit_code -eq 0) {
        [int64]$meta.elapsed_ms
      }
    })
  $failurePath = if (Test-Path -LiteralPath `
      (Join-Path $attempt 'runner-infrastructure-failure.json')) {
    Join-Path $attempt 'runner-infrastructure-failure.json'
  } else {
    Join-Path $attempt 'failure.json'
  }
  $failure = Get-Content -LiteralPath $failurePath -Raw | ConvertFrom-Json
  if ((Split-Path -Leaf $failurePath) -eq
      'runner-infrastructure-failure.json' -and
      ($null -eq $failure.PSObject.Properties['termination_confirmed'] -or
        -not [bool]$failure.termination_confirmed)) {
    throw 'Post infrastructure routing cannot confirm child termination; stop all further child commands.'
  }
  [ordered]@{
    stage = 'post'
    samples_ms = @($partialSamples)
    median_ms = $null
    stable_count = $null
    required_stable_count = 4
    max_absolute_deviation_ms = 300
    series_valid = $false
    invalid_reason = "zero-retry infrastructure failure: $($failure | ConvertTo-Json -Compress)"
    source_sha256 = $sourceHash
  } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath `
    (Join-Path $attempt 'summary.json')
}
if ($probeCode -notin @(0, 2)) { throw "Unexpected post probe exit $probeCode" }
$postSummaryPath = Join-Path $attempt 'summary.json'
$postSummary = Get-Content -LiteralPath $postSummaryPath -Raw |
  ConvertFrom-Json
if (-not (Test-CompletedSeriesSummary $postSummary 'post' $sourceHash)) {
  throw 'Post summary is incomplete or structurally incoherent.'
}
$postClaimHash =
  (Get-FileHash -LiteralPath $postClaimPath -Algorithm SHA256).Hash
$postCompletionPath = Join-Path $attempt 'post-completion.json'
$postCompletionPayload = [ordered]@{
  stage = 'post-completion'
  completed_at = [DateTimeOffset]::Now.ToString('o')
  scratch = $scratch
  attempt = $attempt
  repository = $postRepo
  target = $postTarget
  head_commit = $postHead
  reapplication_commit = $reapplicationCommit
  post_claim_sha256 = $postClaimHash
  summary_sha256 =
    (Get-FileHash -LiteralPath $postSummaryPath -Algorithm SHA256).Hash
  qualification_sha256 = $currentQualificationHash
  runner_sha256 = $qualification.runner_sha256
  source_sha256 = $sourceHash
  source_restored = $true
  termination_confirmed = $true
  probe_exit_code = $probeCode
} | ConvertTo-Json
$postCompletionBytes =
  [Text.UTF8Encoding]::new($false).GetBytes($postCompletionPayload)
$postCompletionTempPath = '{0}.{1}.tmp' -f `
  $postCompletionPath, ([guid]::NewGuid().ToString('N'))
$postCompletionStream = $null
$postCompletionError = $null
try {
  $postCompletionStream = [IO.File]::Open(
    $postCompletionTempPath,
    [IO.FileMode]::CreateNew,
    [IO.FileAccess]::Write,
    [IO.FileShare]::None)
  $postCompletionStream.Write(
    $postCompletionBytes, 0, $postCompletionBytes.Length)
  $postCompletionStream.Flush($true)
} catch {
  $postCompletionError = $_.Exception.Message
} finally {
  if ($null -ne $postCompletionStream) {
    $postCompletionStream.Dispose()
  }
}
if ($null -eq $postCompletionError) {
  try {
    [IO.File]::Move($postCompletionTempPath, $postCompletionPath)
  } catch {
    $postCompletionError = $_.Exception.Message
  }
}
if ($null -ne $postCompletionError) {
  $artifactFailurePath = Join-Path $claimRoot `
    'diagnostic-artifact-failure.json'
  $artifactFailurePayload = [ordered]@{
    stage = 'diagnostic-artifact-failure'
    failure_stage = 'post-completion'
    recorded_at = [DateTimeOffset]::Now.ToString('o')
    scratch = $scratch
    attempt = $attempt
    error = $postCompletionError
    post_claim_sha256 = $postClaimHash
    summary_sha256 =
      (Get-FileHash -LiteralPath $postSummaryPath -Algorithm SHA256).Hash
    source_sha256 = $sourceHash
    source_restored = $true
    termination_confirmed = $true
  } | ConvertTo-Json
  $artifactFailureBytes =
    [Text.UTF8Encoding]::new($false).GetBytes($artifactFailurePayload)
  $artifactFailureTempPath = '{0}.{1}.tmp' -f `
    $artifactFailurePath, ([guid]::NewGuid().ToString('N'))
  $artifactFailureStream = $null
  try {
    $artifactFailureStream = [IO.File]::Open(
      $artifactFailureTempPath,
      [IO.FileMode]::CreateNew,
      [IO.FileAccess]::Write,
      [IO.FileShare]::None)
    $artifactFailureStream.Write(
      $artifactFailureBytes, 0, $artifactFailureBytes.Length)
    $artifactFailureStream.Flush($true)
  } finally {
    if ($null -ne $artifactFailureStream) {
      $artifactFailureStream.Dispose()
    }
  }
  [IO.File]::Move($artifactFailureTempPath, $artifactFailurePath)
}
$postSummary | Format-List
```

Expected: the post step recomputes the stable claim root, revalidates the bound qualification/baseline claim, and durably creates its one-shot post claim before quiet-window or Cargo. One warm-up is discarded, five post samples are recorded, and exact facade bytes are restored. An existing post claim forbids a retry. Exit `2` produces an invalid summary only after confirmed termination/restoration; exit `1` enters Step 5 only with coherent command-failure, stage/label, restoration, and termination evidence.

- [ ] **Step 2: Classify validity and cumulative ledger eligibility without a performance gate**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$environmentPath = Join-Path $scratch 'environment.json'
$environment = Get-Content -LiteralPath $environmentPath -Raw |
  ConvertFrom-Json
$classificationRepo = [IO.Path]::GetFullPath(
  ([string]$environment.repository)).TrimEnd('\')
$classificationTarget = [IO.Path]::GetFullPath(
  ([string]$environment.target)).TrimEnd('\')
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$classificationHead = $reapplicationCommit
$claimRoot = [string]$environment.diagnostic_claim_root
$persistedClaimRoot = (Get-Content -LiteralPath `
  (Join-Path $scratch 'diagnostic-claim-root.txt') -Raw).Trim()
if ($persistedClaimRoot -ne $claimRoot -or
    [string]::IsNullOrWhiteSpace($claimRoot)) {
  throw 'Classification claim root does not match the stable root.'
}
$artifactFailurePath = Join-Path $claimRoot `
  'diagnostic-artifact-failure.json'
$postClaimPath = Join-Path $claimRoot 'post.json'
if (-not (Test-Path -LiteralPath $artifactFailurePath) -and
    (Test-Path -LiteralPath $postClaimPath)) {
  $recoveryPostClaim = Get-Content -LiteralPath $postClaimPath -Raw |
    ConvertFrom-Json
  $recoveryPostCompletion = Join-Path `
    ([string]$recoveryPostClaim.attempt) 'post-completion.json'
  $recoverySummaryPath = Join-Path `
    ([string]$recoveryPostClaim.attempt) 'summary.json'
  $recoveryQualificationPath = Join-Path $scratch 'runner-qualification.json'
  $recoveryRunnerPath = Join-Path $scratch 'invoke-shell-series.ps1'
  $recoveryEnvironmentClaimPath = Join-Path $claimRoot `
    'environment-preflight.json'
  $recoveryBaselineClaimPath = Join-Path $claimRoot 'baseline.json'
  $recoveryBaselineClaim = Get-Content -LiteralPath `
    $recoveryBaselineClaimPath -Raw | ConvertFrom-Json
  $recoveryBaselineSummaryPath = Join-Path `
    ([string]$recoveryBaselineClaim.attempt) 'summary.json'
  $recoveryBaselineCompletionPath = Join-Path `
    ([string]$recoveryBaselineClaim.attempt) 'baseline-completion.json'
  $recoveryQualification = Get-Content -LiteralPath `
    $recoveryQualificationPath -Raw | ConvertFrom-Json
  $recoveryQualificationHash = (Get-FileHash -LiteralPath `
    $recoveryQualificationPath -Algorithm SHA256).Hash
  $recoveryRunnerHash = (Get-FileHash -LiteralPath `
    $recoveryRunnerPath -Algorithm SHA256).Hash
  function Test-PostRecoverySummary([object]$value, [string]$sourceHash) {
    $samples = @($value.samples_ms)
    if ($value.stage -ne 'post' -or $value.source_sha256 -cne $sourceHash -or
        [int]$value.required_stable_count -ne 4 -or
        [int]$value.max_absolute_deviation_ms -ne 300) { return $false }
    if ($null -ne $value.median_ms -and $null -ne $value.stable_count) {
      if ($samples.Count -ne 5) { return $false }
      $sorted = @($samples | ForEach-Object { [int64]$_ } | Sort-Object)
      $median = [int64]$sorted[2]
      $stable = @($samples | Where-Object {
        [Math]::Abs([int64]$_ - $median) -le 300
      }).Count
      return [int64]$value.median_ms -eq $median -and
        [int]$value.stable_count -eq $stable -and
        [bool]$value.series_valid -eq ($stable -ge 4)
    }
    return -not [bool]$value.series_valid -and
      $null -eq $value.median_ms -and $null -eq $value.stable_count -and
      $samples.Count -le 5 -and
      -not [string]::IsNullOrWhiteSpace([string]$value.invalid_reason)
  }
  $recoveryPostCompletionValid = $false
  if (Test-Path -LiteralPath $recoveryPostCompletion) {
    try {
      $recoveryCompletion = Get-Content -LiteralPath `
        $recoveryPostCompletion -Raw | ConvertFrom-Json
      $recoverySummary = Get-Content -LiteralPath $recoverySummaryPath -Raw |
        ConvertFrom-Json
      $recoveryPostCompletionValid =
        [bool]$recoveryQualification.passed -and
        [bool]$recoveryQualification.termination_confirmed -and
        $recoveryQualification.runner_sha256 -ceq $recoveryRunnerHash -and
        $recoveryPostClaim.stage -eq 'post' -and
        $recoveryPostClaim.scratch -eq $scratch -and
        $recoveryPostClaim.repository -eq $classificationRepo -and
        $recoveryPostClaim.target -eq $classificationTarget -and
        $recoveryPostClaim.head_commit -eq $reapplicationCommit -and
        $recoveryPostClaim.reapplication_commit -eq $reapplicationCommit -and
        $recoveryPostClaim.implementation_base -eq $environment.implementation_base -and
        $recoveryPostClaim.identity_commit -eq $environment.identity_commit -and
        $recoveryPostClaim.environment_preflight_claim_sha256 -ceq
          (Get-FileHash -LiteralPath $recoveryEnvironmentClaimPath `
            -Algorithm SHA256).Hash -and
        $recoveryPostClaim.baseline_claim_sha256 -ceq
          (Get-FileHash -LiteralPath $recoveryBaselineClaimPath `
            -Algorithm SHA256).Hash -and
        $recoveryPostClaim.baseline_completion_sha256 -ceq
          (Get-FileHash -LiteralPath $recoveryBaselineCompletionPath `
            -Algorithm SHA256).Hash -and
        $recoveryPostClaim.baseline_summary_sha256 -ceq
          (Get-FileHash -LiteralPath $recoveryBaselineSummaryPath `
            -Algorithm SHA256).Hash -and
        $recoveryPostClaim.qualification_sha256 -ceq
          $recoveryQualificationHash -and
        $recoveryPostClaim.runner_sha256 -ceq $recoveryRunnerHash -and
        $recoveryPostClaim.source_sha256 -ceq
          (Get-FileHash -LiteralPath 'src-tauri/src/lib.rs' `
            -Algorithm SHA256).Hash -and
        (Test-PostRecoverySummary $recoverySummary `
          $recoveryPostClaim.source_sha256) -and
        $recoveryCompletion.stage -eq 'post-completion' -and
        $recoveryCompletion.scratch -eq $scratch -and
        $recoveryCompletion.attempt -eq $recoveryPostClaim.attempt -and
        $recoveryCompletion.repository -eq $classificationRepo -and
        $recoveryCompletion.target -eq $classificationTarget -and
        $recoveryCompletion.head_commit -eq $reapplicationCommit -and
        $recoveryCompletion.reapplication_commit -eq $reapplicationCommit -and
        $recoveryCompletion.post_claim_sha256 -ceq
          (Get-FileHash -LiteralPath $postClaimPath -Algorithm SHA256).Hash -and
        $recoveryCompletion.summary_sha256 -ceq
          (Get-FileHash -LiteralPath $recoverySummaryPath -Algorithm SHA256).Hash -and
        $recoveryCompletion.qualification_sha256 -ceq
          $recoveryQualificationHash -and
        $recoveryCompletion.runner_sha256 -ceq $recoveryRunnerHash -and
        $recoveryCompletion.source_sha256 -ceq
          $recoveryPostClaim.source_sha256 -and
        [bool]$recoveryCompletion.source_restored -and
        [bool]$recoveryCompletion.termination_confirmed -and
        [int]$recoveryCompletion.probe_exit_code -in @(0, 2)
    } catch { $recoveryPostCompletionValid = $false }
  }
  if (-not $recoveryPostCompletionValid) {
    if ((Get-FileHash -LiteralPath 'src-tauri/src/lib.rs' -Algorithm SHA256).Hash `
          -cne $recoveryPostClaim.source_sha256) {
      throw 'Interrupted post cannot prove exact source restoration.'
    }
    $runnerPath = Join-Path $scratch 'invoke-shell-series.ps1'
    $allProcesses = @(Get-CimInstance Win32_Process -ErrorAction Stop)
    $liveRunner = @($allProcesses | Where-Object {
      $_.ProcessId -ne $PID -and
      [string]$_.CommandLine -match [regex]::Escape($runnerPath)
    })
    $blockingBuild = @($allProcesses | Where-Object {
      $name = [string]$_.Name
      $command = [string]$_.CommandLine
      $name -match '^(cargo.*|rustc|rust-analyzer|extractum|tauri|vite)\.exe$' -or
        ($name -match '^(node|npm|npx)(\.exe|\.cmd)?$' -and
          $command -match '(?i)(vite|tauri|svelte-kit|cargo)')
    })
    if ($blockingBuild.Count -ne 0 -or $liveRunner.Count -ne 0) {
      throw 'Interrupted post termination is not independently confirmed.'
    }
    $artifactFailurePayload = [ordered]@{
      stage = 'diagnostic-artifact-failure'
      failure_stage = 'post-completion-missing-or-invalid'
      recorded_at = [DateTimeOffset]::Now.ToString('o')
      scratch = $scratch
      attempt = $recoveryPostClaim.attempt
      error = 'Atomic post completion receipt is missing, malformed, or mismatched.'
      post_claim_sha256 =
        (Get-FileHash -LiteralPath $postClaimPath -Algorithm SHA256).Hash
      summary_sha256 = if (Test-Path -LiteralPath $recoverySummaryPath) {
        (Get-FileHash -LiteralPath $recoverySummaryPath -Algorithm SHA256).Hash
      } else { $null }
      source_sha256 = $recoveryPostClaim.source_sha256
      source_restored = $true
      termination_confirmed = $true
    } | ConvertTo-Json
    $artifactFailureBytes =
      [Text.UTF8Encoding]::new($false).GetBytes($artifactFailurePayload)
    $artifactFailureTempPath = '{0}.{1}.tmp' -f `
      $artifactFailurePath, ([guid]::NewGuid().ToString('N'))
    $artifactFailureStream = [IO.File]::Open(
      $artifactFailureTempPath,
      [IO.FileMode]::CreateNew,
      [IO.FileAccess]::Write,
      [IO.FileShare]::None)
    try {
      $artifactFailureStream.Write(
        $artifactFailureBytes, 0, $artifactFailureBytes.Length)
      $artifactFailureStream.Flush($true)
    } finally { $artifactFailureStream.Dispose() }
    [IO.File]::Move($artifactFailureTempPath, $artifactFailurePath)
  }
}
if (Test-Path -LiteralPath $artifactFailurePath) {
  $artifactFailure = Get-Content -LiteralPath $artifactFailurePath -Raw |
    ConvertFrom-Json
  if ($artifactFailure.stage -ne 'diagnostic-artifact-failure' -or
      $artifactFailure.scratch -ne $scratch -or
      -not [bool]$artifactFailure.source_restored -or
      -not [bool]$artifactFailure.termination_confirmed) {
    throw 'Diagnostic artifact failure lacks safe invalid-session routing proof.'
  }
  function Get-OptionalInvalidSummary([string]$pointerName, [string]$stage) {
    try {
      $pointerPath = Join-Path $scratch $pointerName
      if (Test-Path -LiteralPath $pointerPath) {
        $attemptPath = (Get-Content -LiteralPath $pointerPath -Raw).Trim()
        $summaryPath = Join-Path $attemptPath 'summary.json'
        if (Test-Path -LiteralPath $summaryPath) {
          return Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
        }
      }
    } catch { }
    return [pscustomobject][ordered]@{
      stage = $stage
      samples_ms = @()
      median_ms = $null
      stable_count = $null
      required_stable_count = 4
      max_absolute_deviation_ms = 300
      series_valid = $false
      invalid_reason = 'diagnostic artifact chain failed before a bound summary'
      source_sha256 = $null
    }
  }
  $invalidBaseline = Get-OptionalInvalidSummary `
    'baseline-current.txt' 'baseline'
  $invalidPost = Get-OptionalInvalidSummary 'post-current.txt' 'post'
  [ordered]@{
    gating = $false
    repeat_used = $false
    baseline_attempt = if (Test-Path -LiteralPath `
        (Join-Path $scratch 'baseline-current.txt')) {
      (Get-Content -LiteralPath `
        (Join-Path $scratch 'baseline-current.txt') -Raw).Trim()
    } else { $null }
    post_attempt = if (Test-Path -LiteralPath `
        (Join-Path $scratch 'post-current.txt')) {
      (Get-Content -LiteralPath `
        (Join-Path $scratch 'post-current.txt') -Raw).Trim()
    } else { $null }
    baseline = $invalidBaseline
    post = $invalidPost
    session_valid = $false
    invalid_reason = 'diagnostic artifact/completion chain failure'
    artifact_failure = $artifactFailure
    artifact_failure_sha256 =
      (Get-FileHash -LiteralPath $artifactFailurePath -Algorithm SHA256).Hash
    delta_ms = $null
    delta_percent = $null
    cumulative_ceiling_ms = 15000
    remaining_ms = $null
    cumulative_ceiling_exceeded = $false
    candidate_rejected_by_diagnostics = $false
  } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath `
    (Join-Path $scratch 'measurement-summary.json')
  Get-Content -LiteralPath (Join-Path $scratch 'measurement-summary.json') -Raw
  return
}
if (-not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
  throw 'CARGO_TARGET_DIR changed after environment capture.'
}
$classificationRepoRaw = @(git rev-parse --show-toplevel)
$classificationRepoCode = $LASTEXITCODE
$classificationHeadRaw = @(git rev-parse HEAD)
$classificationHeadCode = $LASTEXITCODE
$classificationStatus = @(git status --porcelain=v1)
$classificationStatusCode = $LASTEXITCODE
if ($classificationRepoCode -ne 0 -or $classificationHeadCode -ne 0 -or
    $classificationStatusCode -ne 0 -or
    $classificationRepoRaw.Count -ne 1 -or
    $classificationHeadRaw.Count -ne 1) {
  throw 'Could not bind classification to the current worktree.'
}
$actualClassificationRepo = [IO.Path]::GetFullPath(
  ([string]$classificationRepoRaw[0]).Trim()).TrimEnd('\')
$actualClassificationTarget = [IO.Path]::GetFullPath(
  (Join-Path $actualClassificationRepo 'src-tauri/target')).TrimEnd('\')
$actualClassificationHead = ([string]$classificationHeadRaw[0]).Trim()
if ($actualClassificationRepo -ne $classificationRepo -or
    $actualClassificationTarget -ne $classificationTarget -or
    $actualClassificationHead -ne $classificationHead -or
    $classificationStatus.Count -ne 0) {
  throw 'Classification requires the captured clean candidate worktree and target.'
}
$gitCommonRaw = @(git rev-parse --git-common-dir)
if ($LASTEXITCODE -ne 0 -or $gitCommonRaw.Count -ne 1) {
  throw 'Could not recompute the stable diagnostic claim root.'
}
$gitCommon = (Resolve-Path -LiteralPath ([string]$gitCommonRaw[0])).Path
$claimMaterial = '{0}|{1}' -f $gitCommon.ToLowerInvariant(), `
  $environment.implementation_base
$claimHasher = [Security.Cryptography.SHA256]::Create()
try {
  $claimKey = -join ($claimHasher.ComputeHash(
    [Text.Encoding]::UTF8.GetBytes($claimMaterial)) | ForEach-Object {
      $_.ToString('x2')
    })
} finally { $claimHasher.Dispose() }
$recomputedClaimRoot = Join-Path $env:TEMP `
  "extractum-process-reapplication-claims/$claimKey"
if ($recomputedClaimRoot -ne $claimRoot) {
  throw 'Classification claim root does not match the stable root.'
}
$environmentClaimPath = Join-Path $claimRoot 'environment-preflight.json'
$baselineClaimPath = Join-Path $claimRoot 'baseline.json'
$qualificationPath = Join-Path $scratch 'runner-qualification.json'
$initialQuietPath = Join-Path $scratch 'quiet-initial.json'
$environmentClaim = Get-Content -LiteralPath $environmentClaimPath -Raw |
  ConvertFrom-Json
$baselineClaim = Get-Content -LiteralPath $baselineClaimPath -Raw |
  ConvertFrom-Json
$postClaim = Get-Content -LiteralPath $postClaimPath -Raw |
  ConvertFrom-Json
$qualification = Get-Content -LiteralPath $qualificationPath -Raw |
  ConvertFrom-Json
$baselineAttempt = (Get-Content -LiteralPath `
  (Join-Path $scratch 'baseline-current.txt') -Raw).Trim()
$postAttempt = (Get-Content -LiteralPath `
  (Join-Path $scratch 'post-current.txt') -Raw).Trim()
$attemptsRoot = [IO.Path]::GetFullPath(
  (Join-Path $scratch 'attempts')).TrimEnd('\') + '\'
$baselineAttemptFull = [IO.Path]::GetFullPath($baselineAttempt)
$postAttemptFull = [IO.Path]::GetFullPath($postAttempt)
if (-not $baselineAttemptFull.StartsWith(
      $attemptsRoot, [StringComparison]::OrdinalIgnoreCase) -or
    -not $postAttemptFull.StartsWith(
      $attemptsRoot, [StringComparison]::OrdinalIgnoreCase)) {
  throw 'Claimed measurement attempts escaped the session attempts root.'
}
$baselineSummaryPath = Join-Path $baselineAttempt 'summary.json'
$postSummaryPath = Join-Path $postAttempt 'summary.json'
$baselineCompletionPath = Join-Path $baselineAttempt 'baseline-completion.json'
$postCompletionPath = Join-Path $postAttempt 'post-completion.json'
$baseline = Get-Content -LiteralPath $baselineSummaryPath -Raw |
  ConvertFrom-Json
$post = Get-Content -LiteralPath $postSummaryPath -Raw |
  ConvertFrom-Json
$baselineCompletion = Get-Content -LiteralPath $baselineCompletionPath -Raw |
  ConvertFrom-Json
$postCompletion = Get-Content -LiteralPath $postCompletionPath -Raw |
  ConvertFrom-Json
$environmentClaimHash =
  (Get-FileHash -LiteralPath $environmentClaimPath -Algorithm SHA256).Hash
$environmentHash =
  (Get-FileHash -LiteralPath $environmentPath -Algorithm SHA256).Hash
$initialQuietHash =
  (Get-FileHash -LiteralPath $initialQuietPath -Algorithm SHA256).Hash
$qualificationHash =
  (Get-FileHash -LiteralPath $qualificationPath -Algorithm SHA256).Hash
$baselineClaimHash =
  (Get-FileHash -LiteralPath $baselineClaimPath -Algorithm SHA256).Hash
$postClaimHash =
  (Get-FileHash -LiteralPath $postClaimPath -Algorithm SHA256).Hash
$baselineSummaryHash =
  (Get-FileHash -LiteralPath $baselineSummaryPath -Algorithm SHA256).Hash
$postSummaryHash =
  (Get-FileHash -LiteralPath $postSummaryPath -Algorithm SHA256).Hash
$baselineCompletionHash =
  (Get-FileHash -LiteralPath $baselineCompletionPath -Algorithm SHA256).Hash
$postCompletionHash =
  (Get-FileHash -LiteralPath $postCompletionPath -Algorithm SHA256).Hash
$currentSourceHash = (Get-FileHash -LiteralPath 'src-tauri/src/lib.rs' `
  -Algorithm SHA256).Hash
$runnerHash = (Get-FileHash -LiteralPath `
  (Join-Path $scratch 'invoke-shell-series.ps1') -Algorithm SHA256).Hash
$jobHelperHash = (Get-FileHash -LiteralPath `
  (Join-Path $scratch 'job-object.ps1') -Algorithm SHA256).Hash
function Test-CompletedSeriesSummary(
  [object]$value,
  [string]$expectedStage,
  [string]$expectedSourceHash
) {
  $samples = @($value.samples_ms)
  if ($value.stage -ne $expectedStage -or
      $value.source_sha256 -cne $expectedSourceHash -or
      [int]$value.required_stable_count -ne 4 -or
      [int]$value.max_absolute_deviation_ms -ne 300 -or
      @($samples | Where-Object { $null -eq $_ }).Count -ne 0) {
    return $false
  }
  if ($null -ne $value.median_ms -and $null -ne $value.stable_count) {
    if ($samples.Count -ne 5) { return $false }
    $sorted = @($samples | ForEach-Object { [int64]$_ } | Sort-Object)
    $median = [int64]$sorted[2]
    $stableCount = @($samples | Where-Object {
      [Math]::Abs([int64]$_ - $median) -le 300
    }).Count
    return [int64]$value.median_ms -eq $median -and
      [int]$value.stable_count -eq $stableCount -and
      [bool]$value.series_valid -eq ($stableCount -ge 4)
  }
  return -not [bool]$value.series_valid -and
    $null -eq $value.median_ms -and $null -eq $value.stable_count -and
    $samples.Count -le 5 -and
    $null -ne $value.PSObject.Properties['invalid_reason'] -and
    -not [string]::IsNullOrWhiteSpace([string]$value.invalid_reason)
}
if (-not [bool]$qualification.passed -or
    -not [bool]$qualification.termination_confirmed -or
    $qualification.runner_sha256 -cne $runnerHash -or
    $qualification.job_helper_sha256 -cne $jobHelperHash -or
    $environmentClaim.stage -ne 'environment-preflight' -or
    $environmentClaim.scratch -ne $scratch -or
    $environmentClaim.repository -ne $classificationRepo -or
    $environmentClaim.target -ne $classificationTarget -or
    $environmentClaim.identity_commit -ne $environment.identity_commit -or
    $environmentClaim.implementation_base -ne
      $environment.implementation_base -or
    $environmentClaim.quiet_artifact -ne $initialQuietPath -or
    $environmentClaimHash -cne
      $environment.environment_preflight_claim_sha256 -or
    $baselineClaim.stage -ne 'baseline' -or
    $baselineClaim.scratch -ne $scratch -or
    $baselineClaim.attempt -ne $baselineAttempt -or
    $baselineClaim.repository -ne $classificationRepo -or
    $baselineClaim.target -ne $classificationTarget -or
    $baselineClaim.head_commit -ne $environment.identity_commit -or
    $baselineClaim.implementation_base -ne $environment.implementation_base -or
    $baselineClaim.identity_commit -ne $environment.identity_commit -or
    $baselineClaim.environment_preflight_claim_sha256 -cne
      $environmentClaimHash -or
    $baselineClaim.environment_sha256 -cne $environmentHash -or
    $baselineClaim.initial_quiet_sha256 -cne $initialQuietHash -or
    $baselineClaim.qualification_sha256 -cne $qualificationHash -or
    $baselineClaim.runner_sha256 -cne $runnerHash -or
    -not (Test-CompletedSeriesSummary $baseline 'baseline' `
      $baselineClaim.source_sha256) -or
    $baselineCompletion.stage -ne 'baseline-completion' -or
    $baselineCompletion.scratch -ne $scratch -or
    $baselineCompletion.attempt -ne $baselineAttempt -or
    $baselineCompletion.repository -ne $classificationRepo -or
    $baselineCompletion.target -ne $classificationTarget -or
    $baselineCompletion.head_commit -ne $environment.identity_commit -or
    $baselineCompletion.baseline_claim_sha256 -cne $baselineClaimHash -or
    $baselineCompletion.summary_sha256 -cne $baselineSummaryHash -or
    $baselineCompletion.qualification_sha256 -cne $qualificationHash -or
    $baselineCompletion.runner_sha256 -cne $runnerHash -or
    $baselineCompletion.source_sha256 -cne $baselineClaim.source_sha256 -or
    -not [bool]$baselineCompletion.source_restored -or
    -not [bool]$baselineCompletion.termination_confirmed -or
    [int]$baselineCompletion.probe_exit_code -notin @(0, 2) -or
    $postClaim.stage -ne 'post' -or
    $postClaim.scratch -ne $scratch -or
    $postClaim.attempt -ne $postAttempt -or
    $postClaim.repository -ne $classificationRepo -or
    $postClaim.target -ne $classificationTarget -or
    $postClaim.head_commit -ne $reapplicationCommit -or
    $postClaim.reapplication_commit -ne $reapplicationCommit -or
    $postClaim.implementation_base -ne $environment.implementation_base -or
    $postClaim.identity_commit -ne $environment.identity_commit -or
    $postClaim.environment_preflight_claim_sha256 -cne $environmentClaimHash -or
    $postClaim.baseline_claim_sha256 -cne $baselineClaimHash -or
    $postClaim.baseline_completion_sha256 -cne $baselineCompletionHash -or
    $postClaim.baseline_summary_sha256 -cne $baselineSummaryHash -or
    $postClaim.qualification_sha256 -cne $qualificationHash -or
    $postClaim.runner_sha256 -cne $runnerHash -or
    $postClaim.source_sha256 -cne $currentSourceHash -or
    -not (Test-CompletedSeriesSummary $post 'post' `
      $postClaim.source_sha256) -or
    $postCompletion.stage -ne 'post-completion' -or
    $postCompletion.scratch -ne $scratch -or
    $postCompletion.attempt -ne $postAttempt -or
    $postCompletion.repository -ne $classificationRepo -or
    $postCompletion.target -ne $classificationTarget -or
    $postCompletion.head_commit -ne $reapplicationCommit -or
    $postCompletion.reapplication_commit -ne $reapplicationCommit -or
    $postCompletion.post_claim_sha256 -cne $postClaimHash -or
    $postCompletion.summary_sha256 -cne $postSummaryHash -or
    $postCompletion.qualification_sha256 -cne $qualificationHash -or
    $postCompletion.runner_sha256 -cne $runnerHash -or
    $postCompletion.source_sha256 -cne $postClaim.source_sha256 -or
    -not [bool]$postCompletion.source_restored -or
    -not [bool]$postCompletion.termination_confirmed -or
    [int]$postCompletion.probe_exit_code -notin @(0, 2)) {
  throw 'Measurement claim/completion chain is incomplete, mismatched, or stale.'
}
$sessionValid = [bool]$baseline.series_valid -and [bool]$post.series_valid -and
  $null -ne $baseline.median_ms -and $null -ne $post.median_ms
$deltaMs = if ($null -ne $baseline.median_ms -and $null -ne $post.median_ms) {
  [int64]$post.median_ms - [int64]$baseline.median_ms
} else { $null }
$deltaPercent = if ($null -ne $deltaMs -and [int64]$baseline.median_ms -ne 0) {
  100.0 * [double]$deltaMs / [double]$baseline.median_ms
} else { $null }
$remainingMs = if ($sessionValid) { 15000 - [int64]$post.median_ms } else { $null }
[ordered]@{
  gating = $false
  repeat_used = $false
  baseline_attempt = $baselineAttempt
  post_attempt = $postAttempt
  environment_preflight_claim_sha256 = $environmentClaimHash
  baseline_claim_sha256 = $baselineClaimHash
  baseline_completion_sha256 = $baselineCompletionHash
  post_claim_sha256 = $postClaimHash
  post_completion_sha256 = $postCompletionHash
  baseline = $baseline
  post = $post
  session_valid = $sessionValid
  delta_ms = $deltaMs
  delta_percent = $deltaPercent
  cumulative_ceiling_ms = 15000
  remaining_ms = $remainingMs
  cumulative_ceiling_exceeded = ($sessionValid -and $remainingMs -lt 0)
  candidate_rejected_by_diagnostics = $false
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath `
  (Join-Path $scratch 'measurement-summary.json')
Get-Content -LiteralPath (Join-Path $scratch 'measurement-summary.json') -Raw
```

Expected: the result explicitly says `gating=false`, `repeat_used=false`, and `candidate_rejected_by_diagnostics=false`. Only `session_valid=true` supplies `remaining_ms`; a negative remaining value is recorded honestly but does not reject exact Phase 3.

- [ ] **Step 3: Run all current command-based completion gates sequentially**

Run this exact sequence. A confirmed nonzero result is a completion failure; a command-start/target-lock failure is infrastructure and stops without classification until corrected.

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$completedGates = [System.Collections.Generic.List[string]]::new()
function Invoke-CompletionGate([string]$gate, [scriptblock]$command) {
  $savedErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  $startFailure = $null
  try {
    $output = @(& $command 2>&1)
    $code = $LASTEXITCODE
  } catch {
    $output = @($_.Exception.Message)
    $code = -1
    $startFailure = $_.Exception.Message
  } finally {
    $ErrorActionPreference = $savedErrorActionPreference
  }
  $log = Join-Path $scratch "completion-$gate.log"
  $output | Set-Content -LiteralPath $log
  if ($code -ne 0) {
    $text = $output -join "`n"
    $infrastructure = $null -ne $startFailure -or
      $text -match '(?i)(\.cargo-lock|access(?: is)? denied|os error 5|used by another process|cannot access the file)'
    $classification = if ($infrastructure) {
      'infrastructure_invalid'
    } else {
      'command_failed'
    }
    $artifact = if ($infrastructure) {
      'completion-infrastructure-failure.json'
    } else {
      'completion-failure.json'
    }
    [ordered]@{
      gate = $gate
      exit_code = $code
      classification = $classification
      log = $log
      command_start_error = $startFailure
      completed_before = @($completedGates)
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath `
      (Join-Path $scratch $artifact)
    if ($infrastructure) {
      throw "Completion infrastructure failure: $gate; do not enter the negative branch."
    }
    throw "Confirmed completion failure: $gate"
  }
  $completedGates.Add($gate)
}

Invoke-CompletionGate 'source-contracts' {
  npm.cmd run test -- `
    src/lib/process-crate-reapplication-identity-contract.test.ts `
    src/lib/process-crate-boundary-contract.test.ts `
    src/lib/external-process-lifecycle-contract.test.ts `
    src/lib/hidden-child-process-contract.test.ts `
    src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CompletionGate 'typescript-svelte-check' { npm.cmd run check }
Invoke-CompletionGate 'locked-cargo-metadata' {
  cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 --no-deps
}
Invoke-CompletionGate 'rustfmt-check' { npm.cmd run check:rustfmt }
Invoke-CompletionGate 'locked-workspace-check' {
  cargo check --manifest-path src-tauri/Cargo.toml --locked --workspace --all-targets
}
Invoke-CompletionGate 'locked-workspace-tests' {
  cargo test --manifest-path src-tauri/Cargo.toml --locked --workspace --all-targets
}
Invoke-CompletionGate 'repository-verify' { npm.cmd run verify }
Invoke-CompletionGate 'release-no-bundle' { npm.cmd run tauri -- build --no-bundle }
@($completedGates) | ConvertTo-Json | Set-Content -LiteralPath `
  (Join-Path $scratch 'completion-gates.json')
```

Expected: all eight named gates pass. `npm.cmd run verify` intentionally repeats the full repository gate. MSI/WiX is not run.

- [ ] **Step 4: Reprove identity after wrappers, then run the hidden startup smoke**

Routing invariant: helper load/compile, Job Object construction, atomic start/assignment, observation, and cleanup are infrastructure. They must never create `completion-failure.json` or authorize Step 5. Only a readable early exit from the application returned by `StartAssigned`, followed by confirmed zero-process cleanup, is a completion failure.

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
function Stop-WrapperIdentity([string]$message) {
  [ordered]@{
    gate = 'post-wrapper-candidate-identity'
    classification = 'state_invalid'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'identity-failure.json')
  throw $message
}
function Stop-WrapperIdentityInfrastructure([string]$message) {
  [ordered]@{
    gate = 'post-wrapper-identity-probe'
    classification = 'infrastructure_invalid'
    error = $message
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'completion-infrastructure-failure.json')
  throw "$message Do not enter the negative branch."
}
$committedLockRaw = @(git rev-parse "$reapplicationCommit`:src-tauri/Cargo.lock")
$committedLockCode = $LASTEXITCODE
if ($committedLockCode -ne 0 -or $committedLockRaw.Count -ne 1) {
  Stop-WrapperIdentityInfrastructure 'Could not read committed Cargo.lock identity.'
}
$committedLock = ([string]$committedLockRaw[0]).Trim()
if ($committedLock -ne
    '6368e32cd3a3853d4a7114ce256258e834bafdd4') {
  Stop-WrapperIdentity 'Committed Cargo.lock blob is not the historical candidate blob.'
}
$headLockRaw = @(git rev-parse 'HEAD:src-tauri/Cargo.lock')
$headLockCode = $LASTEXITCODE
if ($headLockCode -ne 0 -or $headLockRaw.Count -ne 1) {
  Stop-WrapperIdentityInfrastructure 'Could not read post-wrapper Cargo.lock identity.'
}
$headLock = ([string]$headLockRaw[0]).Trim()
if ($headLock -ne
    '6368e32cd3a3853d4a7114ce256258e834bafdd4') {
  Stop-WrapperIdentity 'Cargo.lock changed during completion wrappers.'
}
$headTauriRaw = @(git rev-parse 'HEAD:src-tauri')
$headTauriCode = $LASTEXITCODE
if ($headTauriCode -ne 0 -or $headTauriRaw.Count -ne 1) {
  Stop-WrapperIdentityInfrastructure 'Could not read post-wrapper src-tauri identity.'
}
$headTauri = ([string]$headTauriRaw[0]).Trim()
if ($headTauri -ne
    $manifest.candidate_src_tauri_tree) {
  Stop-WrapperIdentity 'Candidate src-tauri tree changed during completion wrappers.'
}
$historicalRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev $manifest.historical_candidate)
if ($LASTEXITCODE -ne 0) {
  Stop-WrapperIdentityInfrastructure 'Could not compute historical raw identity.'
}
$reappliedRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev $reapplicationCommit)
if ($LASTEXITCODE -ne 0) {
  Stop-WrapperIdentityInfrastructure 'Could not compute reapplication raw identity.'
}
if (@(Compare-Object $historicalRaw $reappliedRaw).Count -ne 0) {
  Stop-WrapperIdentity 'Committed candidate identity changed during completion wrappers.'
}
$wrapperStatus = @(git status --porcelain=v1)
if ($LASTEXITCODE -ne 0) {
  Stop-WrapperIdentityInfrastructure 'Could not inspect post-wrapper worktree status.'
}
if ($wrapperStatus.Count -ne 0) {
  Stop-WrapperIdentity 'Completion wrappers changed tracked repository bytes.'
}

$process = $null
$launchFailure = $null
$completionFailure = $null
$completionExitCode = $null
$cleanupFailure = $null
$smokeJob = $null
$smokeJobAssigned = $false
$smokeJobActiveProcesses = $null
$taskkillCode = $null
$ownedSmokeIds = [System.Collections.Generic.HashSet[int]]::new()
function Get-LiveSmokeTreeIds([int]$rootId) {
  $all = @(Get-CimInstance Win32_Process -ErrorAction Stop)
  $known = [System.Collections.Generic.HashSet[int]]::new()
  [void]$known.Add($rootId)
  $changed = $true
  while ($changed) {
    $changed = $false
    foreach ($item in $all) {
      if ($known.Contains([int]$item.ParentProcessId) -and
          $known.Add([int]$item.ProcessId)) {
        $changed = $true
      }
    }
  }
  return @($all | Where-Object { $known.Contains([int]$_.ProcessId) } |
    ForEach-Object { [int]$_.ProcessId })
}
try {
  try {
    . (Join-Path $scratch 'job-object.ps1')
    $smokeJob = [ExtractumOwnedJob]::new()
    $exe = (Resolve-Path -LiteralPath `
      'src-tauri/target/release/extractum.exe' -ErrorAction Stop).Path
    $process = $smokeJob.StartAssigned(
      $exe,
      '',
      (Get-Location).Path,
      (Join-Path $scratch 'startup-smoke.stdout.log'),
      (Join-Path $scratch 'startup-smoke.stderr.log'),
      $true)
    $smokeJobAssigned = $true
    foreach ($id in @(Get-LiveSmokeTreeIds $process.Id)) {
      [void]$ownedSmokeIds.Add($id)
    }
  } catch {
    $launchFailure = $_.Exception.Message
  }
  if ($null -eq $launchFailure) {
    try {
      foreach ($second in 1..5) {
        Start-Sleep -Seconds 1
        $process.Refresh()
        foreach ($id in @(Get-LiveSmokeTreeIds $process.Id)) {
          [void]$ownedSmokeIds.Add($id)
        }
        if ($process.HasExited) {
          $completionExitCode = $process.ExitCode
          $completionFailure =
            "Release executable exited early with code $completionExitCode"
          break
        }
      }
    } catch {
      $launchFailure = "Startup observation infrastructure failed: $($_.Exception.Message)"
    }
  }
} finally {
  if ($null -ne $process) {
    try {
      $process.Refresh()
      if (-not $process.HasExited) {
        foreach ($id in @(Get-LiveSmokeTreeIds $process.Id)) {
          [void]$ownedSmokeIds.Add($id)
        }
        $taskkillLog = Join-Path $scratch 'startup-smoke-taskkill.log'
        $savedErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = 'Continue'
        try {
          & taskkill.exe /PID $process.Id /T /F 1> $taskkillLog 2>&1
          $taskkillCode = $LASTEXITCODE
        } finally {
          $ErrorActionPreference = $savedErrorActionPreference
        }
        [void]$process.WaitForExit(10000)
      }
      if ($null -eq $smokeJob) { throw 'Startup Job Object was not created.' }
      $smokeJobActiveProcesses = [uint32]$smokeJob.ActiveProcesses
      if ($smokeJobActiveProcesses -ne 0) {
        $smokeJob.Terminate(1)
        [void]$process.WaitForExit(10000)
        [void]$smokeJob.WaitForEmpty(10000)
        $smokeJobActiveProcesses = [uint32]$smokeJob.ActiveProcesses
      }
      $remainingBySnapshot = @($ownedSmokeIds | Where-Object {
        $null -ne (Get-Process -Id $_ -ErrorAction SilentlyContinue)
      })
      $remainingTree = @(
        $remainingBySnapshot + @(Get-LiveSmokeTreeIds $process.Id) |
          Sort-Object -Unique
      )
      if (-not $smokeJobAssigned -or $smokeJobActiveProcesses -ne 0 -or
          $remainingTree.Count -ne 0) {
        throw "Startup ownership/cleanup unconfirmed: assigned=$smokeJobAssigned; " +
          "job_active=$smokeJobActiveProcesses; remaining=$($remainingTree -join ',')"
      }
    } catch {
      $cleanupFailure = $_.Exception.Message
    }
  }
  if ($null -ne $smokeJob -and
      ($null -eq $process -or $smokeJobActiveProcesses -eq 0)) {
    try {
      $smokeJob.Dispose()
      $smokeJob = $null
    } catch {
      $disposeText = "Job Object disposal failed: $($_.Exception.Message)"
      $cleanupFailure = if ([string]::IsNullOrWhiteSpace([string]$cleanupFailure)) {
        $disposeText
      } else { "$cleanupFailure; $disposeText" }
    }
  }
}
if ($null -ne $cleanupFailure) {
  [ordered]@{
    gate = 'startup-smoke-cleanup'
    classification = 'infrastructure_invalid'
    error = $cleanupFailure
    process_id = if ($null -ne $process) { $process.Id } else { $null }
    job_assigned = $smokeJobAssigned
    job_active_processes = $smokeJobActiveProcesses
    taskkill_exit_code = $taskkillCode
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'startup-smoke-infrastructure-failure.json')
  throw 'Startup cleanup is unconfirmed; stop without retaining or reverting.'
}
if ($null -ne $launchFailure) {
  [ordered]@{
    gate = 'startup-smoke-launch'
    classification = 'infrastructure_invalid'
    error = $launchFailure
    process_id = if ($null -ne $process) { $process.Id } else { $null }
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'startup-smoke-infrastructure-failure.json')
  throw 'Startup launch/observation failed as infrastructure; do not revert.'
}
if ($null -ne $completionFailure) {
  if ($null -eq $process -or $null -eq $completionExitCode -or
      -not $smokeJobAssigned -or
      $smokeJobActiveProcesses -ne 0) {
    [ordered]@{
      gate = 'startup-smoke-routing'
      classification = 'infrastructure_invalid'
      error = 'Early-exit evidence lacks confirmed Job Object assignment/cleanup.'
      process_id = if ($null -ne $process) { $process.Id } else { $null }
      job_assigned = $smokeJobAssigned
      job_active_processes = $smokeJobActiveProcesses
    } | ConvertTo-Json | Set-Content -LiteralPath `
      (Join-Path $scratch 'startup-smoke-infrastructure-failure.json')
    throw 'Startup smoke routing proof failed as infrastructure; do not revert.'
  }
  [ordered]@{
    gate = 'startup-smoke'
    application_exit_code = $completionExitCode
    classification = 'command_failed'
    error = $completionFailure
  } | ConvertTo-Json | Set-Content -LiteralPath `
    (Join-Path $scratch 'completion-failure.json')
  throw 'Startup smoke is a confirmed completion failure.'
}
[ordered]@{
  gate = 'startup-smoke'
  passed = $true
  observed_seconds = 5
  cleanup_confirmed = $true
  job_assigned = $smokeJobAssigned
  job_active_processes = $smokeJobActiveProcesses
  taskkill_exit_code = $taskkillCode
} | ConvertTo-Json | Set-Content -LiteralPath `
  (Join-Path $scratch 'startup-smoke.json')
```

Expected: committed identity remains exact, repository remains clean, release executable stays alive for five seconds, and full process-tree cleanup is confirmed. Resolve/start/observation/cleanup/disposal failures are infrastructure-only. A completion failure requires `StartAssigned` to return an assigned application process, a readable actual application exit code inside the five-second window, and subsequently confirmed zero-process cleanup.

If the smoke writes `startup-smoke-infrastructure-failure.json`, stop this gate and investigate; never enter Step 5. A retry is allowed only after separately recording confirmed process-tree termination and the corrected objective infrastructure cause. Before that retry, preserve the current veto artifact in history without deleting or overwriting it:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$smokeInfrastructure = Join-Path $scratch `
  'startup-smoke-infrastructure-failure.json'
if (-not (Test-Path -LiteralPath $smokeInfrastructure)) {
  throw 'No current startup-smoke infrastructure artifact to archive.'
}
$history = Join-Path $scratch ('startup-smoke-history/{0}' -f `
  [DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff'))
New-Item -ItemType Directory -Path $history -Force | Out-Null
Move-Item -LiteralPath $smokeInfrastructure -Destination `
  (Join-Path $history 'startup-smoke-infrastructure-failure.json')
```

- [ ] **Step 5: Route any identity, correctness, or completion failure before success documentation**

This is the only negative branch. Do not enter it for unstable/non-gating measurements. If a confirmed candidate/identity/completion failure occurred after the reapplication commit:

First prove the failure is material and restore any dirty candidate paths to the committed bytes:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$startupSmokeInfrastructure = Join-Path $scratch `
  'startup-smoke-infrastructure-failure.json'
if (Test-Path -LiteralPath $startupSmokeInfrastructure) {
  throw 'Current startup-smoke infrastructure evidence forbids the negative/revert branch.'
}
$materialArtifacts = @(
  'identity-failure.json'
  'candidate-correctness-failure.json'
  'completion-failure.json'
) | ForEach-Object { Join-Path $scratch $_ } | Where-Object {
  Test-Path -LiteralPath $_
}
if ($materialArtifacts.Count -eq 0) {
  throw 'No recorded identity/correctness/completion failure; negative branch forbidden.'
}
$paths = @($manifest.entries.path)
@(git status --porcelain=v1) | Set-Content -LiteralPath `
  (Join-Path $scratch 'negative-precleanup-status.txt')
@(git diff --raw) | Set-Content -LiteralPath `
  (Join-Path $scratch 'negative-precleanup-worktree-raw.txt')
@(git diff --cached --raw) | Set-Content -LiteralPath `
  (Join-Path $scratch 'negative-precleanup-index-raw.txt')
git restore --source=$reapplicationCommit --staged --worktree -- $paths
$negativeRestoreCode = $LASTEXITCODE
$negativeHeadRaw = @(git rev-parse 'HEAD')
$negativeHeadCode = $LASTEXITCODE
$negativeStatus = @(git status --porcelain=v1)
$negativeStatusCode = $LASTEXITCODE
$negativeTauriRaw = @(git rev-parse 'HEAD:src-tauri')
$negativeTauriCode = $LASTEXITCODE
$negativeHead = if ($negativeHeadRaw.Count -eq 1) {
  ([string]$negativeHeadRaw[0]).Trim()
} else { $null }
$negativeTauri = if ($negativeTauriRaw.Count -eq 1) {
  ([string]$negativeTauriRaw[0]).Trim()
} else { $null }
if ($negativeRestoreCode -ne 0 -or $negativeHeadCode -ne 0 -or
    $negativeStatusCode -ne 0 -or $negativeTauriCode -ne 0 -or
    $negativeHeadRaw.Count -ne 1 -or $negativeTauriRaw.Count -ne 1 -or
    $negativeHead -ne $reapplicationCommit -or $negativeStatus.Count -ne 0 -or
    $negativeTauri -ne $manifest.candidate_src_tauri_tree) {
  throw 'Could not restore the clean committed candidate; do not revert.'
}
$materialArtifacts | Set-Content -LiteralPath `
  (Join-Path $scratch 'negative-material-artifacts.txt')
```

Render the complete pre-revert report in scratch:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$artifactPaths = @(Get-Content -LiteralPath `
  (Join-Path $scratch 'negative-material-artifacts.txt'))
if ($artifactPaths.Count -ne 1) {
  throw 'Negative report requires exactly one terminal material artifact.'
}
$failurePath = $artifactPaths[0]
$failure = Get-Content -LiteralPath $failurePath -Raw | ConvertFrom-Json
$measurementText = if (Test-Path -LiteralPath `
    (Join-Path $scratch 'measurement-summary.json')) {
  (Get-Content -LiteralPath (Join-Path $scratch 'measurement-summary.json') `
    -Raw | ConvertFrom-Json) | ConvertTo-Json -Depth 8 -Compress
} else { 'not collected' }
$completedText = if (Test-Path -LiteralPath `
    (Join-Path $scratch 'completion-gates.json')) {
  $completedParsed = Get-Content -LiteralPath `
    (Join-Path $scratch 'completion-gates.json') -Raw | ConvertFrom-Json
  @($completedParsed) -join ', '
} elseif ($failure.PSObject.Properties.Name -contains 'completed_before') {
  @($failure.completed_before) -join ', '
} else { 'none' }
$logText = if ($failure.PSObject.Properties.Name -contains 'log' -and
    -not [string]::IsNullOrWhiteSpace([string]$failure.log)) {
  [string]$failure.log
} else { 'not applicable' }
$errorText = if ($failure.PSObject.Properties.Name -contains 'error' -and
    -not [string]::IsNullOrWhiteSpace([string]$failure.error)) {
  [string]$failure.error
} else { 'not supplied; inspect the failure artifact and referenced log' }
$executionDate = ([DateTimeOffset]::Parse([string]$environment.started_at)).ToString('yyyy-MM-dd')
$reportLines = @(
  '# Extractum Process Exact Reapplication Verification'
  ''
  ('**Date:** {0}' -f $executionDate)
  ('**Execution started:** `{0}`' -f $environment.started_at)
  '**Historical candidate:** `b364756c7b5768d644321afeaeb81ec04e2481a4`'
  '**Historical parent:** `306a9370c90fd008a3b3259f77f4f48349806d05`'
  '**Historical revert:** `c47372dcd2fa97d8fe05f01d26a0c4f9eb888c83`'
  ('**Reapplication commit:** `{0}`' -f $reapplicationCommit)
  '**Outcome:** `not_retained`'
  ''
  '## Historical Integrity'
  ''
  '- The 2026-07-17 `not_retained` result remains valid under its frozen protocol and is not rewritten.'
  '- The fresh replay matched the frozen 14-path candidate before execution.'
  ''
  '## Environment'
  ''
  ('- Repository: `{0}`' -f $environment.repository)
  ('- Branch: `{0}`' -f $environment.branch)
  ('- Host: `{0}`' -f $environment.host)
  ''
  '## Exact Candidate Identity'
  ''
  '- Historical path states: `14 / 14`'
  ('- Reapplication commit: `{0}`' -f $reapplicationCommit)
  ''
  '## Test and Consumer Inventory'
  ''
  '- Completed gates before failure: `' + $completedText + '`'
  ''
  '## Non-Gating Shell Diagnostics'
  ''
  '- These diagnostics were strictly non-gating.'
  ('- Measurement artifact: `{0}`' -f $measurementText)
  ''
  '## Failed Gate'
  ''
  ('- Failure artifact: `{0}`' -f $failurePath)
  ('- Gate: `{0}`' -f $failure.gate)
  ('- Classification: `{0}`' -f $failure.classification)
  ('- Error: `{0}`' -f $errorText)
  ('- Log: `{0}`' -f $logText)
  ''
  '## Revert Verification'
  ''
  '- Revert commit: `pending`'
  '- Parent `src-tauri` tree: `pending`'
  ''
  '## Decision'
  ''
  '- The exact replay was not retained because of the recorded identity, correctness, or completion failure.'
  '- Performance diagnostics did not select this outcome.'
  '- A new owner-approved Phase 3 design is required; Phase 4 remains blocked.'
)
$reportLines -join "`n" | Set-Content -LiteralPath `
  (Join-Path $scratch 'negative-report-preview.md')
Get-Content -LiteralPath (Join-Path $scratch 'negative-report-preview.md') -Raw
```

Use `apply_patch` to create `docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md` byte-for-byte from `negative-report-preview.md` before reverting. The preview is the full template; do not add ad hoc headings or omit `not collected`/`not applicable` evidence. Its sections are, in order:

```markdown
## Historical Integrity
## Environment
## Exact Candidate Identity
## Test and Consumer Inventory
## Non-Gating Shell Diagnostics
## Failed Gate
## Revert Verification
## Decision
```

The Decision must say that the 2026-07-17 historical `not_retained` result remains valid and unchanged, this fresh exact replay also was not retained for the named correctness/identity/completion failure, performance diagnostics did not select the outcome, and a new owner-approved design is required.

Then revert and mechanically prove the parent state:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$report = 'docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md'
if (-not (Test-Path -LiteralPath $report) -or
    (Get-Content -LiteralPath $report -Raw -Encoding UTF8) -notmatch `
      '\*\*Outcome:\*\* `not_retained`') {
  throw 'Negative report must exist before revert.'
}
git revert --no-edit $reapplicationCommit
if ($LASTEXITCODE -ne 0) { throw 'Exact reapplication revert failed.' }
$revertRaw = @(git rev-parse HEAD)
$revertCode = $LASTEXITCODE
if ($revertCode -ne 0 -or $revertRaw.Count -ne 1) {
  throw 'Revert completed but its commit id could not be resolved.'
}
$revertCommit = ([string]$revertRaw[0]).Trim()
$revertCommit | Set-Content -LiteralPath (Join-Path $scratch 'revert-commit.txt')
$revertedTauriRaw = @(git rev-parse 'HEAD:src-tauri')
$revertedTauriCode = $LASTEXITCODE
if ($revertedTauriCode -ne 0 -or $revertedTauriRaw.Count -ne 1) {
  throw 'Could not resolve the reverted src-tauri tree.'
}
$revertedTauri = ([string]$revertedTauriRaw[0]).Trim()
if ($revertedTauri -ne
    'fd9711a041432ef420e7b09d56a46131a2a52a2a') {
  throw 'Revert did not restore the parent src-tauri tree.'
}
foreach ($path in @(
  'src-tauri/src/child_process.rs'
  'src-tauri/src/external_process.rs'
  'src-tauri/src/process_tree.rs'
)) {
  if (-not (Test-Path -LiteralPath $path)) { throw "Revert did not restore $path" }
}
foreach ($path in @(
  'src-tauri/crates/extractum-process'
  'src/lib/process-crate-boundary-contract.test.ts'
)) {
  if (Test-Path -LiteralPath $path) { throw "Revert left candidate path $path" }
}
```

Use `apply_patch` to append the literal revert SHA to the report and to update the roadmap and shell-cap contract together:

- preserve the historical 2026-07-17 paragraphs and explicitly label them historical;
- update the upper `Completed and in-flight slices` Phase 3 bullet to retain the historical 2026-07-17 result and append the fresh failed exact-reapplication/revert outcome with the new verification link;
- set the Phase 3 heading to `Phase 3 — \`extractum-process\` (blocked: exact reapplication failed)`;
- append the failed gate, reapplication SHA, revert SHA, and negative verification link;
- keep exactly this one pending row: `| Reapplied Phase 3 | unavailable — candidate not retained | pending | Phase 4 blocked pending owner-approved Phase 3 design |`;
- replace the Phase 4 prerequisite with `Phase 4 remains blocked pending a new owner-approved Phase 3 design; the exact reapplication failed a current identity, correctness, or completion gate.`;
- change `src/lib/crate-extraction-shell-cap-contract.test.ts` to require that blocked heading, negative outcome/link/SHAs, and pending ledger row. It must not continue to require `approved for exact-candidate reapplication`.

For the negative contract, add the `node:fs`/`node:path` imports, `processReapplicationVerification` loader, and `roadmapSummary` extraction shown in success Step 6, then replace the pending Phase 3 assertions with this exact branch:

```ts
    expect(roadmapBudget.match(/\| Reapplied Phase 3 \|/g)).toHaveLength(1);
    expect(roadmapBudget).toContain(
      "| Reapplied Phase 3 | unavailable — candidate not retained | " +
        "pending | Phase 4 blocked pending owner-approved Phase 3 design |",
    );
    expect(phase3Roadmap).toContain(
      "Phase 3 — `extractum-process` (blocked: exact reapplication failed)",
    );
    expect(phase4Roadmap).toContain(
      "Phase 4 remains blocked pending a new owner-approved Phase 3 design; " +
        "the exact reapplication failed a current identity, correctness, or completion gate.",
    );
    expect(processReapplicationVerification).toContain(
      "**Outcome:** `not_retained`",
    );
    const failedReplay = processReapplicationVerification.match(
      /^\*\*Reapplication commit:\*\* `([0-9a-f]{40})`$/m,
    );
    const failedRevert = processReapplicationVerification.match(
      /^- Revert commit: `([0-9a-f]{40})`$/m,
    );
    expect(failedReplay).not.toBeNull();
    expect(failedRevert).not.toBeNull();
    expect(phase3Roadmap).toContain(failedReplay?.[1]);
    expect(phase3Roadmap).toContain(failedRevert?.[1]);
    expect(phase3Roadmap).toContain(
      "2026-07-18-extractum-process-reapplication.md",
    );
    expect(roadmapSummary).toContain("fresh exact reapplication failed");
```

Also remove the old approved-reapplication and unconditional Phase 4 assertions. Use `apply_patch` to replace `- Revert commit: \`pending\`` with the literal revert SHA and `- Parent \`src-tauri\` tree: \`pending\`` with `- Parent \`src-tauri\` tree: \`fd9711a041432ef420e7b09d56a46131a2a52a2a\`` before running the contract.

Run the post-revert gates:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
npm.cmd run test -- `
  src/lib/external-process-lifecycle-contract.test.ts `
  src/lib/hidden-child-process-contract.test.ts `
  src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Post-revert source contracts failed.' }
cargo check --manifest-path src-tauri/Cargo.toml --locked --workspace --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Post-revert locked workspace check failed.' }
cargo test --manifest-path src-tauri/Cargo.toml --locked --workspace --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Post-revert locked workspace tests failed.' }
npm.cmd run verify
if ($LASTEXITCODE -ne 0) { throw 'Post-revert repository verification failed.' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Negative evidence whitespace check failed.' }
```

Stage and commit the negative evidence exactly:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$negativePaths = @(
  'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md'
  'src/lib/crate-extraction-shell-cap-contract.test.ts'
) | Sort-Object
git add -- $negativePaths
if ($LASTEXITCODE -ne 0) { throw 'Could not stage negative evidence.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Negative staged whitespace check failed.' }
$actualNegativePaths = @(git diff --cached --name-only | Sort-Object)
if (@(Compare-Object $negativePaths $actualNegativePaths).Count -ne 0) {
  throw 'Negative staged allowlist mismatch.'
}
git commit -m "docs: record failed process crate reapplication"
if ($LASTEXITCODE -ne 0) { throw 'Negative evidence commit failed.' }
$status = @(git status --porcelain=v1)
if ($LASTEXITCODE -ne 0 -or $status.Count -ne 0) {
  throw 'Negative branch is not clean after evidence commit.'
}
```

Expected: the negative execution ends with identity commit, exact candidate commit, automatic revert commit, negative evidence commit, and a clean branch. Stop this plan and do not start Phase 4.

An infrastructure-only artifact (`candidate-gate-infrastructure-failure.json`, `completion-infrastructure-failure.json`, runner exit `2`, or `startup-smoke-infrastructure-failure.json` from helper load/compilation/construction, atomic launch/assignment, observation, cleanup, or disposal) never authorizes this branch. Preserve it, regain a separately recorded confirmed quiet/terminated state, correct the objective infrastructure cause, and resume the same correctness gate; the diagnostic series itself remains invalid and is never retried.

- [ ] **Step 6: Write the retained-evidence RED contract**

In `src/lib/crate-extraction-shell-cap-contract.test.ts`, add these Node imports before the Vitest import:

```ts
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
```

Immediately after the `normalize` helper, add:

```ts
const processReapplicationVerificationPath = path.join(
  process.cwd(),
  "docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md",
);
const processReapplicationVerification = existsSync(
  processReapplicationVerificationPath,
)
  ? normalize(readFileSync(processReapplicationVerificationPath, "utf8"))
  : "";
const formatMilliseconds = (value: number) =>
  String(value).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
```

Immediately after `roadmapBudget`, add:

```ts
const roadmapSummary = sectionBetween(
  crateRoadmap,
  "Completed and in-flight slices governed by their own documents:",
  "## Evidence Base",
);
```

Inside `records the cumulative roadmap and moot anomaly disposition`, replace the two pending-row assertions and the old Phase 3 heading assertion with:

```ts
    const reappliedRows = roadmapBudget
      .split("\n")
      .filter((line) => line.startsWith("| Reapplied Phase 3 |"));
    expect(reappliedRows).toHaveLength(1);
    const reappliedCells = reappliedRows[0]
      .split("|")
      .slice(1, -1)
      .map((cell) => cell.trim());
    expect(reappliedCells).toHaveLength(4);
    const validMedian = reappliedCells[1].match(/^([0-9,]+) ms$/);
    const invalidSession =
      reappliedCells[1] === "unavailable — invalid diagnostic session";
    expect([Boolean(validMedian), invalidSession].filter(Boolean)).toHaveLength(1);
    const verificationLedgerRow = processReapplicationVerification.match(
      /^- Roadmap ledger row: `(\| Reapplied Phase 3 \|.*\|)`$/m,
    );
    expect(verificationLedgerRow).not.toBeNull();
    expect(reappliedRows[0]).toBe(verificationLedgerRow?.[1]);
    expect(roadmapBudget).toContain(verificationLedgerRow?.[1]);
    const verificationSession = processReapplicationVerification.match(
      /^- Session valid: `(true|false)`$/m,
    );
    const verificationPost = processReapplicationVerification.match(
      /^- Post median: `([0-9,]+ ms|unavailable)`$/m,
    );
    const verificationCommit = processReapplicationVerification.match(
      /^\*\*Reapplication commit:\*\* `([0-9a-f]{40})`$/m,
    );
    expect(verificationSession).not.toBeNull();
    expect(verificationPost).not.toBeNull();
    expect(verificationCommit).not.toBeNull();
    expect(phase3Roadmap).toContain(verificationCommit?.[1]);
    if (validMedian) {
      expect(verificationSession?.[1]).toBe("true");
      expect(verificationPost?.[1]).toBe(reappliedCells[1]);
    } else {
      expect(verificationSession?.[1]).toBe("false");
      expect(verificationPost?.[1]).toBe("unavailable");
    }
    expect(processReapplicationVerification).toContain(
      "- Completion gates: `source-contracts, typescript-svelte-check, " +
        "locked-cargo-metadata, rustfmt-check, locked-workspace-check, " +
        "locked-workspace-tests, repository-verify, release-no-bundle`",
    );
    expect(processReapplicationVerification).toContain(
      "- Startup smoke passed: `true`",
    );
    expect(processReapplicationVerification).toContain(
      "- Startup smoke observed seconds: `5`",
    );
    expect(processReapplicationVerification).toContain(
      "- Startup smoke cleanup confirmed: `true`",
    );
    expect(phase3Roadmap).toContain(
      "Phase 3 — `extractum-process` (completed: exact candidate reapplied)",
    );
    expect(phase3Roadmap).toContain(
      "2026-07-18-extractum-process-reapplication.md",
    );
    if (validMedian) {
      const median = Number(validMedian[1].replaceAll(",", ""));
      if (median <= 15_000) {
        expect(reappliedCells[2]).toBe(
          `${formatMilliseconds(15_000 - median)} ms`,
        );
        expect(phase4Roadmap).toContain(
          "A valid post-Phase 3 shell baseline is recorded in the ledger",
        );
        expect(phase4Roadmap).toContain(
          "Phase 4 implementation planning is authorized",
        );
      } else {
        expect(reappliedCells[2]).toBe(
          `exceeded by ${formatMilliseconds(median - 15_000)} ms`,
        );
        expect(phase4Roadmap).toContain(
          "valid post-Phase 3 shell baseline exceeds the 15,000 ms cumulative ceiling",
        );
        expect(phase4Roadmap).toContain(
          "new owner-approved policy revision is required",
        );
      }
    } else {
      expect(reappliedCells[2]).toBe("pending");
      expect(phase4Roadmap).toContain(
        "Phase 4 remains blocked until a valid post-Phase 3 shell baseline is recorded",
      );
    }
```

Remove the old unconditional Phase 4 assertions for `remains blocked until the exact Phase 3 candidate is integrated` and `valid shell baseline exists`; keep the assertion that no additional v2/v3 approval is required. Near the end of the same test, add:

```ts
    expect(processReapplicationVerification).toContain(
      "**Historical candidate:** `b364756c7b5768d644321afeaeb81ec04e2481a4`",
    );
    expect(processReapplicationVerification).toMatch(
      /\*\*Reapplication commit:\*\* `[0-9a-f]{40}`/,
    );
    expect(processReapplicationVerification).toContain(
      "**Outcome:** `retained`",
    );
    expect(processReapplicationVerification).toContain(
      "14 / 14 exact path states",
    );
    expect(processReapplicationVerification).toContain(
      "Process tests before/after: 20 / 20",
    );
    expect(processReapplicationVerification).toContain(
      "Consumers unchanged: 12 / 12",
    );
    expect(processReapplicationVerification).toContain(
      "strictly non-gating",
    );
    expect(roadmapSummary).toContain(
      "2026-07-17 execution was measured and not retained",
    );
    expect(roadmapSummary).toContain(
      "2026-07-18-extractum-process-reapplication.md",
    );
    expect(roadmapSummary).toContain("exact candidate was reapplied and retained");
```

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
```

Expected: the file is collected and fails because the new verification is absent and the roadmap still says `approved for exact-candidate reapplication`. Do not weaken assertions in response.

- [ ] **Step 7: Create literal verification evidence and update the roadmap**

Read these immutable scratch inputs before editing:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
$identityPreflight = Get-Content -LiteralPath `
  (Join-Path $scratch 'identity-preflight.json') -Raw | ConvertFrom-Json
$identityStaged = Get-Content -LiteralPath `
  (Join-Path $scratch 'identity-staged.json') -Raw | ConvertFrom-Json
$inventory = Get-Content -LiteralPath `
  (Join-Path $scratch 'inventory-comparison.json') -Raw | ConvertFrom-Json
$measurement = Get-Content -LiteralPath `
  (Join-Path $scratch 'measurement-summary.json') -Raw | ConvertFrom-Json
$completion = Get-Content -LiteralPath `
  (Join-Path $scratch 'completion-gates.json') -Raw | ConvertFrom-Json
$smoke = Get-Content -LiteralPath `
  (Join-Path $scratch 'startup-smoke.json') -Raw | ConvertFrom-Json
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$environment, $identityPreflight, $identityStaged, $inventory, $measurement,
  $completion, $smoke | Format-List
```

Render the fixed header first:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$executionDate = ([DateTimeOffset]::Parse([string]$environment.started_at)).ToString('yyyy-MM-dd')
$verificationHeader = @(
  '# Extractum Process Exact Reapplication Verification'
  ''
  ('**Date:** {0}' -f $executionDate)
  ('**Execution started:** `{0}`' -f $environment.started_at)
  '**Historical candidate:** `b364756c7b5768d644321afeaeb81ec04e2481a4`'
  '**Historical parent:** `306a9370c90fd008a3b3259f77f4f48349806d05`'
  '**Historical revert:** `c47372dcd2fa97d8fe05f01d26a0c4f9eb888c83`'
  ('**Reapplication commit:** `{0}`' -f $reapplicationCommit)
  '**Outcome:** `retained`'
) -join "`n"
$verificationHeader
```

Use `apply_patch` to create `docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md`. Its first lines must equal the rendered header byte-for-byte. Follow it with these headings, in order, and copy every runtime value literally from the artifacts:

```markdown
## Historical Integrity
## Environment
## Exact Candidate Identity
## Test and Consumer Inventory
## Non-Gating Shell Diagnostics
## Current Completion Gates
## Decision
```

The rendered file must say `14 / 14 exact path states`, `Process tests before/after: 20 / 20`, `Consumers unchanged: 12 / 12`, and that the new shell series is `strictly non-gating`. Include every raw sample actually collected, medians when available, both stability counts, validity, diagnostic delta/percentage, numeric `remaining_ms`, and boolean `cumulative_ceiling_exceeded`. Completed series have five-value arrays; a zero-retry infrastructure-invalid series instead includes its partial array and literal invalid reason. Include these exact machine-checked list items, filled literally from scratch and the chosen roadmap row:

```markdown
- Session valid: `true` or `false`
- Post median: `<comma-formatted integer> ms` or `unavailable`
- Roadmap ledger row: `| Reapplied Phase 3 | ... |`
- Completion gates: `source-contracts, typescript-svelte-check, locked-cargo-metadata, rustfmt-check, locked-workspace-check, locked-workspace-tests, repository-verify, release-no-bundle`
- Startup smoke passed: `true`
- Startup smoke observed seconds: `5`
- Startup smoke cleanup confirmed: `true`
```

If the session is invalid, state that neither median entered the ledger. If it is valid above `15,000 ms`, state that exact Phase 3 remains accepted but later retention needs a policy revision. State explicitly that the old `not_retained` result was correct under its frozen 2026-07-17 protocol and was not rewritten.

Then update `docs/superpowers/specs/2026-07-17-crate-roadmap.md` with `apply_patch`:

1. Replace the Phase 3 heading with:

   ```markdown
   ### Phase 3 — `extractum-process` (completed: exact candidate reapplied)
   ```

2. In the upper `Completed and in-flight slices` summary, rewrite the Phase 3 bullet so it says both that the `2026-07-17 execution was measured and not retained` under its historical protocol and that the `exact candidate was reapplied and retained`, with a link to `2026-07-18-extractum-process-reapplication.md`. Preserve the detailed historical failure/revert paragraphs below and append the literal reapplication SHA, identity-manifest link, new verification link, `20 / 20`, `12 / 12`, completion-gate result, and retained owner-decision explanation.
3. In the target crate map, replace `phase 3, if justified` with `phase 3, done: exact historical candidate`.
4. Replace the single pending ledger row with exactly one of these data-derived forms:

   - valid and `post <= 15,000`: literal post median; literal `15,000 - post` remaining; disposition `valid non-gating diagnostic; Phase 4 baseline available`;
   - valid and `post > 15,000`: literal post median; `exceeded by` the literal excess; disposition `exact Phase 3 retained by owner exception; later retention requires policy revision`;
   - invalid: `| Reapplied Phase 3 | unavailable — invalid diagnostic session | pending | Phase 4 baseline still required |`.

5. Replace the Phase 4 prerequisite paragraph with the matching exact branch:

   ```markdown
   A valid post-Phase 3 shell baseline is recorded in the ledger. Phase 4 implementation planning is authorized after the retained Phase 3 branch is integrated. No additional v2/v3 diagnostic approval is required.
   ```

   or:

   ```markdown
   Phase 4 remains blocked because the valid post-Phase 3 shell baseline exceeds the 15,000 ms cumulative ceiling; a new owner-approved policy revision is required before any later slice can be retained. No additional v2/v3 diagnostic approval is required.
   ```

   or:

   ```markdown
   Phase 4 remains blocked until a valid post-Phase 3 shell baseline is recorded. No additional v2/v3 diagnostic approval is required.
   ```

Do not edit the v2/v3 disposition, old plan, old verification, or any policy threshold. This slice reuses existing `retained`/`not_retained` evidence vocabulary and the registered diagnostic values `ok`, `command_failed`, `infrastructure_invalid`, and `state_invalid`; it adds no new product or artifact status/kind/reason token. Persistence, API, UI, fixtures, and `docs/value-registry.md` remain unchanged.

Before GREEN, validate the complete scratch-to-report-to-roadmap chain:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$measurement = Get-Content -LiteralPath `
  (Join-Path $scratch 'measurement-summary.json') -Raw | ConvertFrom-Json
$completionParsed = Get-Content -LiteralPath `
  (Join-Path $scratch 'completion-gates.json') -Raw | ConvertFrom-Json
$completion = @($completionParsed)
$smoke = Get-Content -LiteralPath (Join-Path $scratch 'startup-smoke.json') `
  -Raw | ConvertFrom-Json
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$report = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md' `
  -Raw -Encoding UTF8
$roadmap = Get-Content -LiteralPath `
  'docs/superpowers/specs/2026-07-17-crate-roadmap.md' -Raw -Encoding UTF8
function Format-Milliseconds([int64]$value) {
  return $value.ToString('N0', [Globalization.CultureInfo]::InvariantCulture)
}
$expectedGates = @(
  'source-contracts'
  'typescript-svelte-check'
  'locked-cargo-metadata'
  'rustfmt-check'
  'locked-workspace-check'
  'locked-workspace-tests'
  'repository-verify'
  'release-no-bundle'
)
if (@(Compare-Object $expectedGates $completion).Count -ne 0 -or
    $completion.Count -ne $expectedGates.Count) {
  throw 'Completion gate artifact is not the exact eight-gate sequence.'
}
for ($index = 0; $index -lt $expectedGates.Count; $index++) {
  if ([string]$completion[$index] -ne $expectedGates[$index]) {
    throw "Completion gate order mismatch at index $index"
  }
}
$sessionValid = [bool]$measurement.session_valid
$expectedPost = if ($sessionValid) {
  '{0} ms' -f (Format-Milliseconds ([int64]$measurement.post.median_ms))
} else { 'unavailable' }
if ($sessionValid) {
  $postMedian = [int64]$measurement.post.median_ms
  if ($postMedian -le 15000) {
    $expectedRow = '| Reapplied Phase 3 | {0} ms | {1} ms | valid non-gating diagnostic; Phase 4 baseline available |' -f `
      (Format-Milliseconds $postMedian), (Format-Milliseconds (15000 - $postMedian))
  } else {
    $expectedRow = '| Reapplied Phase 3 | {0} ms | exceeded by {1} ms | exact Phase 3 retained by owner exception; later retention requires policy revision |' -f `
      (Format-Milliseconds $postMedian), (Format-Milliseconds ($postMedian - 15000))
  }
} else {
  $expectedRow = '| Reapplied Phase 3 | unavailable — invalid diagnostic session | pending | Phase 4 baseline still required |'
}
$expectedReportLines = @(
  ('**Reapplication commit:** `{0}`' -f $reapplicationCommit)
  ('- Session valid: `{0}`' -f $sessionValid.ToString().ToLowerInvariant())
  ('- Post median: `{0}`' -f $expectedPost)
  ('- Roadmap ledger row: `{0}`' -f $expectedRow)
  ('- Completion gates: `{0}`' -f ($expectedGates -join ', '))
  '- Startup smoke passed: `true`'
  '- Startup smoke observed seconds: `5`'
  '- Startup smoke cleanup confirmed: `true`'
)
foreach ($line in $expectedReportLines) {
  if (-not $report.Replace("`r`n", "`n").Split("`n").Contains($line)) {
    throw "Verification does not contain exact scratch-derived line: $line"
  }
}
$roadmapRows = @($roadmap.Replace("`r`n", "`n").Split("`n") | Where-Object {
  $_ -like '| Reapplied Phase 3 |*'
})
if ($roadmapRows.Count -ne 1 -or $roadmapRows[0] -ne $expectedRow) {
  throw 'Roadmap ledger row does not equal the scratch-derived row.'
}
if (-not [bool]$smoke.passed -or [int]$smoke.observed_seconds -ne 5 -or
    -not [bool]$smoke.cleanup_confirmed) {
  throw 'Startup smoke artifact is not the reported passing five-second smoke.'
}
```

Expected: the reapplication SHA, validity, post median, exact ledger row, ordered eight-gate list, and smoke fields all agree mechanically across scratch, verification, and roadmap.

- [ ] **Step 8: Run retained-evidence GREEN and the final repository gate**

Run sequentially:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
npm.cmd run test -- `
  src/lib/process-crate-reapplication-identity-contract.test.ts `
  src/lib/process-crate-boundary-contract.test.ts `
  src/lib/external-process-lifecycle-contract.test.ts `
  src/lib/hidden-child-process-contract.test.ts `
  src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Final source contracts failed.' }
npm.cmd run check
if ($LASTEXITCODE -ne 0) { throw 'Final TypeScript/Svelte check failed.' }
npm.cmd run verify
if ($LASTEXITCODE -ne 0) { throw 'Final repository verification failed.' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Final worktree whitespace check failed.' }
```

Expected: all five named contract files pass, repository checking passes, the complete verify wrapper passes, and no whitespace errors appear.

- [ ] **Step 9: Reverify candidate immutability and historical scope**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$environment = Get-Content -LiteralPath (Join-Path $scratch 'environment.json') `
  -Raw | ConvertFrom-Json
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$historicalRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev $manifest.historical_candidate)
$historicalRawCode = $LASTEXITCODE
$reappliedRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev $reapplicationCommit)
$reappliedRawCode = $LASTEXITCODE
if ($historicalRawCode -ne 0 -or $reappliedRawCode -ne 0 -or
    @(Compare-Object $historicalRaw $reappliedRaw).Count -ne 0) {
  throw 'Reapplication commit no longer matches historical candidate.'
}
$finalLockRaw = @(git rev-parse 'HEAD:src-tauri/Cargo.lock')
$finalLockCode = $LASTEXITCODE
if ($finalLockCode -ne 0 -or $finalLockRaw.Count -ne 1) {
  throw 'Could not resolve the final Cargo.lock identity.'
}
$finalLock = ([string]$finalLockRaw[0]).Trim()
if ($finalLock -ne
    '6368e32cd3a3853d4a7114ce256258e834bafdd4') {
  throw 'Final Cargo.lock does not match the exact candidate.'
}
$immutablePaths = @(
  'docs/superpowers/plans/2026-07-18-extractum-process-reapplication.md'
  'docs/superpowers/plans/2026-07-17-extractum-process-extraction.md'
  'docs/superpowers/verification/2026-07-17-extractum-process-extraction.md'
  'docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md'
  'docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md'
  'docs/superpowers/specs/2026-07-18-process-shell-regression-diagnostic-design.md'
  'docs/superpowers/plans/2026-07-18-process-shell-regression-diagnostic.md'
  'docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md'
  'docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md'
  'scripts/process-shell-diagnostic'
)
git diff --quiet $environment.implementation_base HEAD -- $immutablePaths
if ($LASTEXITCODE -ne 0) {
  throw 'The implementation plan, governing specs, historical evidence, or v1 harness changed.'
}
$allowedUncommitted = @(
  'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md'
  'src/lib/crate-extraction-shell-cap-contract.test.ts'
) | Sort-Object
$actualUncommitted = @(git status --short | ForEach-Object {
  $_.Substring(3).Replace('\', '/')
} | Sort-Object)
if (@(Compare-Object $allowedUncommitted $actualUncommitted).Count -ne 0) {
  throw 'Unexpected final evidence-slice path inventory.'
}
```

Expected: exact candidate diff remains immutable, lock blob is exact, historical files are untouched, and only the three evidence-slice paths are uncommitted.

- [ ] **Step 10: Commit the retained evidence slice**

Run:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
git add -- `
  docs/superpowers/specs/2026-07-17-crate-roadmap.md `
  docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md `
  src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Could not stage retained evidence.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Staged evidence whitespace check failed.' }
$expectedEvidencePaths = @(
  'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md'
  'src/lib/crate-extraction-shell-cap-contract.test.ts'
) | Sort-Object
$actualEvidencePaths = @(git diff --cached --name-only | Sort-Object)
if (@(Compare-Object $expectedEvidencePaths $actualEvidencePaths).Count -ne 0) {
  throw 'Staged evidence inventory mismatch.'
}
git commit -m "docs: record retained process crate reapplication"
if ($LASTEXITCODE -ne 0) { throw 'Retained evidence commit failed.' }
```

Expected: one evidence-only commit and a clean worktree.

- [ ] **Step 11: Request independent review and close the branch**

Invoke `superpowers:requesting-code-review` with `environment.implementation_base` (the commit before Task 1), `environment.identity_commit`, current HEAD, this plan, the shell-cap revision design, the active process-boundary design, and both verification files. Require the reviewer to check:

- all 14 parent/candidate path states, subtree hashes, patch OID/id, and raw-diff equality;
- exact code-only candidate commit and absence of manual repairs;
- locked manifest/lockfile behavior and `20/20`, `12/12`, inventory results;
- non-gating measurement semantics, validity calculation, and cumulative ledger arithmetic;
- completion gates, release smoke, and cleanup evidence;
- historical immutability and conditional Phase 4 disposition.

Fix every valid Important issue only in the evidence/contract slice. Any requested change to a candidate-manifest path is a material mismatch: stop instead of editing it. After review fixes, rerun Step 8, then use this review-scope check instead of Step 9's pre-commit exact-three-files assertion:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$allowedReviewPaths = @(
  'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication.md'
  'src/lib/crate-extraction-shell-cap-contract.test.ts'
)
$actualReviewPaths = @(git status --short | ForEach-Object {
  $_.Substring(3).Replace('\', '/')
})
if (@($actualReviewPaths | Where-Object { $_ -notin $allowedReviewPaths }).Count -ne 0) {
  throw 'Review fix escaped the evidence/contract allowlist.'
}
```

Re-run identity against the pinned code commit, never current `HEAD` (which is now an evidence or review-fix commit):

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$scratch = (Get-Content -LiteralPath `
  (Join-Path $env:TEMP 'extractum-process-reapplication-current.txt') -Raw).Trim()
$reapplicationCommit = (Get-Content -LiteralPath `
  (Join-Path $scratch 'reapplication-commit.txt') -Raw).Trim()
$manifest = Get-Content -LiteralPath `
  'docs/superpowers/verification/2026-07-18-extractum-process-reapplication-identity.json' `
  -Raw | ConvertFrom-Json
$historicalRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev $manifest.historical_candidate)
$historicalRawCode = $LASTEXITCODE
$reappliedRaw = @(git diff-tree --no-commit-id --raw -r --no-renames `
  --no-abbrev $reapplicationCommit)
$reappliedRawCode = $LASTEXITCODE
$pinnedTauriRaw = @(git rev-parse "$reapplicationCommit`:src-tauri")
$pinnedTauriCode = $LASTEXITCODE
$pinnedTauri = if ($pinnedTauriRaw.Count -eq 1) {
  ([string]$pinnedTauriRaw[0]).Trim()
} else { $null }
if ($historicalRawCode -ne 0 -or $reappliedRawCode -ne 0 -or
    $pinnedTauriCode -ne 0 -or $pinnedTauriRaw.Count -ne 1 -or
    @(Compare-Object $historicalRaw $reappliedRaw).Count -ne 0 -or
    $reappliedRaw.Count -ne 14 -or
    $pinnedTauri -ne $manifest.candidate_src_tauri_tree) {
  throw 'Review-fix checkpoint changed or cannot prove the pinned candidate identity.'
}
$patchBlobRaw = @(cmd.exe /d /c `
  "git show --format= -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ $reapplicationCommit | git hash-object --stdin")
$patchBlobCode = $LASTEXITCODE
$patchIdRaw = @(cmd.exe /d /c `
  "git show --format= -O NUL --no-renames --full-index --binary --unified=3 --no-color --indent-heuristic --diff-algorithm=myers --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ $reapplicationCommit | git patch-id --stable")
$patchIdCode = $LASTEXITCODE
$patchBlob = if ($patchBlobRaw.Count -eq 1) {
  ([string]$patchBlobRaw[0]).Trim()
} else { $null }
$patchId = if ($patchIdRaw.Count -eq 1) {
  (([string]$patchIdRaw[0]).Trim() -split '\s+')[0]
} else { $null }
if ($patchBlobCode -ne 0 -or $patchIdCode -ne 0 -or
    $patchBlobRaw.Count -ne 1 -or $patchIdRaw.Count -ne 1 -or
    $patchBlob -ne $manifest.no_renames_patch_blob -or
    $patchId -ne $manifest.no_renames_patch_id) {
  throw 'Review-fix checkpoint canonical patch fingerprint mismatch.'
}
```

Commit only the non-empty subset of allowed review paths as a separate review-fix commit, and leave the clean branch ready for the `superpowers:finishing-a-development-branch` flow. Do not begin Phase 4.

## Expected Commit Shape

Successful execution produces exactly three logical commits before optional review fixes:

1. `test: freeze exact process candidate identity`
2. `refactor: extract process infrastructure crate` — code-only replay of `b364756c`
3. `docs: record retained process crate reapplication`

The second commit must have the exact frozen no-renames patch even though its commit SHA differs because its parent includes the identity slice.
