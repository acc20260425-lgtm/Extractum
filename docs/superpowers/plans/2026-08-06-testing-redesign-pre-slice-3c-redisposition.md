# Testing Redesign: Pre-Slice 3C Redisposition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reclassify all 436 validator-open source-contract ledger rows through the approved ordered rule, preserve all 235 closed rows byte-for-byte, publish reproducible cost and review evidence, and leave the repository green without implementing any Slice 3C replacement.

**Architecture:** A committed JSON artifact owns the human disposition decisions and a small dependency-free Node carrier validates and applies them. The carrier compares the working ledger with the exact review-base ledger obtained from Git, permits only resolution-field edits, checks the class/P0/review/timing invariants, and proves idempotence in memory. Classification is calibrated on adversarial tail cases, then applied to the two large contracts and the remaining tail; independent review covers every mandated cohort before one atomic ledger cutover.

**Tech Stack:** Node.js ESM, TypeScript/Vitest 4.1.5 for carrier tests, JSON, Git plumbing, existing source-contract transition validator, PowerShell timing on Windows.

## Global Constraints

- Work only in `G:/Develop/Extractum/.worktrees/codex-slice-3` on `codex/testing-redesign-slice-3`; the review base is the full commit `a54507d63420bb870c3870c91d7e22b050abae3e`.
- The scope is exactly the 436 rows reported open at the review base. SC-000355, SC-000366, and the other 233 closed rows are outside the review; all 235 closed rows remain byte-equivalent.
- Change ledger resolution decisions only. Do not remove or rewrite tests, change product code, Vitest projects, RepositoryIndex, repository rules, public npm commands, verify scheduling, completed 3A/3B evidence, or the program index `Current Detailed Plan`.
- Keep every open row's identity, path, title/manual metadata, `sourceHash`, `authorityHash`, `assertionCount`, lineage, and invariant text unchanged. A factual invariant correction requires another approved specification amendment and is not part of this plan.
- Apply the approved class order exactly: P0; D1; D2; D3; D4; A1; T1; B1; B2; B3; D5; otherwise `UNCLASSIFIED`. Never introduce a per-row override.
- D1 uses the operational future-defect test. D1 is forbidden when any future code, configuration, or test change could violate the invariant as a defect.
- P0 contains at least SC-000555--SC-000560, SC-000511--SC-000515, and SC-000420. Add a row only with an exact citation to `AGENTS.md` section 2, 6, or 7, or an approved program-specification section.
- P0 membership and normative criticality are separate. B3 requires an exact normative `criticalityRef`; a P0 row with no A1/T1/B1/B2 owner terminates at B3.
- D1--D5 reasons begin with the full class name and continue with row-specific text. D5 additionally enumerates all intentionally lost behavior and exact assertion ordinals.
- Independent classification uses a reviewer context that contains only the approved class catalog, normative-source catalog, source declaration/invariant/ordinals, and candidate-owner inventory; it must not contain the author's proposed class, reason, artifact decision, conversation history, or diff. In the current Codex harness, create that context through a separate subagent with `fork_turns: "none"`; another harness must provide an equivalent clean-context primitive or stop explicitly.
- A mixed decision removes top-level resolution fields and writes at least two subgroups whose ordinals exactly and non-overlappingly partition `1..assertionCount`; every subgroup has its own invariant and resolution.
- The committed carrier remains outside `verify`. Its `check` command is mandatory checkpoint evidence; the existing transition validator remains unchanged.
- Run timing observations and the complete `verify` unsandboxed. Use one unretained warm-up plus three retained runs per component mechanism. Use `npm.cmd` for repository scripts and canonical `src-tauri/target`.
- The 90-second value is an objective and disclosure, not a B3 admissibility filter. The binding 3C+ ceiling is 46 proposed jsdom rows. This checkpoint does not propose Playwright owners: a browser-only B3 candidate stops for a program amendment. Net gate seconds are disclosure only.
- Do not add Nx, Nx Cloud, GitHub Actions, branch protection, git hooks, remote cache, or an external reporting service.

---

## File Map

| Path | Responsibility |
| --- | --- |
| `scripts/testing/source-contract-redisposition-review.mjs` | Pure schema/rule validation, review-base loading, allowlisted application, idempotence proof, and `check`/`apply` CLI. |
| `scripts/testing/source-contract-redisposition-review.test.ts` | RED/GREEN coverage for scope pinning, immutable fields, class rules, P0/B3/D5, mixed partitions, review coverage, forecast arithmetic, and idempotence. |
| `testing/source-contract-redisposition-review.json` | Exact review base, catalogs, 436 decisions pinned by `id + sourceHash`, retained timing samples, independent-review records, disposition forecast, and acceptance summaries. |
| `testing/source-contract-ledger.json` | Atomic output of the approved review; only top-level resolution fields or complete mixed `subgroups` change in the 436 scoped rows. |
| `docs/superpowers/verification/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md` | Commands, retained measurements, before/after counts, projected owners, D5 loss table, independent-review outcome, and final gate evidence. |

## Artifact Interfaces

The artifact is one JSON object with this stable shape. Arrays are sorted by the key named below so review diffs remain deterministic.

```ts
type ReasonClass =
  | "D1_COMPLETED_HISTORY_ONLY"
  | "D2_IMPLEMENTATION_SHAPE"
  | "D3_NON_OBSERVABLE_VISUAL"
  | "D4_DUPLICATE_EVIDENCE"
  | "A1_EXISTING_STRUCTURED_OWNER"
  | "T1_EXISTING_TOOL_OWNER"
  | "B1_EXISTING_BEHAVIOR_OWNER"
  | "B2_NEW_CHEAP_BEHAVIOR"
  | "B3_PROTECTED_EXPENSIVE_BEHAVIOR"
  | "D5_ACCEPTED_LOSS"
  | "UNCLASSIFIED";

type Resolution =
  | { disposition: "delete"; deletionReason: string }
  | { disposition: "architecture" | "tool_owned" | "behavior"; replacementIds: string[] }
  | { subgroups: Array<{
      assertionOrdinals: number[];
      invariant: string;
      disposition: "delete" | "architecture" | "tool_owned" | "behavior";
      deletionReason?: string;
      replacementIds?: string[];
    }> };

type Decision = {
  id: string;
  sourceHash: string;
  authorRunId: string;
  class: ReasonClass;
  reason: string;
  ownerEvidence?: string[];
  criticalityRef?: string;
  lostBehavior?: Array<{ assertionOrdinals: number[]; behavior: string }>;
  resolution?: Resolution; // absent only while class is UNCLASSIFIED in the committed draft
};

type ReviewArtifact = {
  schemaVersion: 1;
  reviewBaseCommit: "a54507d63420bb870c3870c91d7e22b050abae3e";
  ledgerFrozenAtCommit: string;
  scope: { openRows: 436; closedRows: 235; closedRowsDigest: string };
  reasonClasses: Array<{ id: Exclude<ReasonClass, "UNCLASSIFIED">; rule: string }>;
  protectedRows: Array<{ id: string; criticalityRef: string }>;
  protectedRowsDigest: string;
  criticalitySources: Array<{ id: string; citation: string }>;
  mechanisms: {
    componentReplacement: { command: string; warmupExitCode: number; retainedSeconds: number[]; medianSeconds: number };
    componentStartup: { command: string; warmupExitCode: number; retainedSeconds: number[]; medianSeconds: number };
    legacyOwner: { command: "npm.cmd run test:legacy-contract"; warmupExitCode: number; retainedSeconds: number[]; medianSeconds: number; baseFiles: string[] };
    verify: { command: "npm.cmd run verify"; seconds: number; exitCode: 0; historicalSeconds: [208.1, 321.3, 383.4] };
  };
  decisions: Decision[];
  independentReview: {
    authorRunId: string;
    reviewer: { agentTaskId: string; contextPolicy: "blind-no-proposed-class-or-reason" };
    blindResults: Array<{ id: string; sourceHash: string; reviewerRunId: string; class: Exclude<ReasonClass, "UNCLASSIFIED">; reason: string; ownerEvidence?: string[]; criticalityRef?: string }>;
    calibrations: Array<{ class: Exclude<ReasonClass, "UNCLASSIFIED">; rowIds: string[]; adjacentClass: string; result: "agree" | "rule_changed" | "no_match" }>;
    mandatoryCohorts: Array<{ name: string; rowIds: string[]; comparison: "agree" | "rule_changed" }>;
    deterministicSample: { algorithm: "sha256-id-lowest-10-percent"; population: string[]; populationDigest: string; rowIds: string[]; iterations: number; comparison: "agree" | "rule_changed" };
    disagreements: Array<{ rowIds: string[]; oldClass: string; newClass: string; groupRuleChange: string }>;
  };
  forecast: {
    beforeByDisposition: Record<string, number>;
    afterByDisposition: Record<string, number>;
    futureOwnersByMechanism: Record<string, { rows: number; assertionOrdinals: number }>;
    proposedNewJsdomRows: number;
    proposedNewJsdomOrdinals: number;
    removableLegacyFiles: number;
    removableLegacySeconds: number;
    upperBoundPerRow: number;
    scalablePerRow: number;
    upperForecastSeconds: number;
    scalableForecastSeconds: number;
    grossAddedForecastSeconds: number;
    netGateForecastSeconds: number;
    replacementUnitCeiling: 46;
    legacyDominatesFullUnitCeiling: boolean;
    scalableForecastPercent: number;
    netGateForecastPercent: number;
  };
  acceptedLoss: { rows: number; assertionOrdinals: number; items: Array<{ id: string; assertionOrdinals: number[]; behavior: string }> };
};
```

The carrier exports these exact functions for its Vitest suite:

```js
export function canonicalJson(value) {}
export function sha256Text(value) {}
export function resolutionForDecision(decision) {}
export function applyReview({ artifact, baseLedger, currentLedger }) {}
export function validateReview({ artifact, baseLedger, currentLedger }) {}
export async function loadBaseLedger({ repoRoot, commit }) {}
```

`applyReview` returns `{ ledger, changedPaths }`, where every path is formatted as `rows[<id>].disposition`, `rows[<id>].replacementIds`, `rows[<id>].deletionReason`, or `rows[<id>].subgroups`. `validateReview` returns a sorted `string[]`; an empty array means valid.

## Rollback Boundary

Tasks 1 and 2 add only the carrier and a draft artifact; neither changes the ledger. Tasks 3 and 4 refine the review artifact while `apply` remains blocked until every mandatory review and timing field is complete. Task 5 first normalizes ledger formatting in a semantic-no-op commit, then makes the single atomic resolution cutover in a second commit; either boundary can be reverted independently. Task 6 adds evidence only. Never carry a partially written ledger across commits; regenerate it with `apply` or revert the cutover commit.

---

### Task 1: Build the fail-closed review carrier

**Files:**

- Create: `scripts/testing/source-contract-redisposition-review.mjs`
- Create: `scripts/testing/source-contract-redisposition-review.test.ts`
- Reference: `scripts/testing/extract-source-contract-ledger.mjs`
- Reference: `scripts/testing/validate-testing-transition.test.ts`

**Interfaces:**

- Consumes the `ReviewArtifact` and exported functions defined above.
- Produces `check` and `apply` CLI modes plus pure helpers used by all later tasks.
- `check` reads `testing/source-contract-redisposition-review.json`, obtains the base ledger with `git show <reviewBaseCommit>:testing/source-contract-ledger.json`, validates current and simulated output, prints decisions grouped by reason class and legacy path plus the exact changed-JSON-path allowlist, and exits 0 only with no issues.
- `apply` runs the same validation, writes canonical JSON plus one trailing newline to `testing/source-contract-ledger.json`, reapplies in memory, and refuses to write unless the second output is byte-identical.

- [ ] **Step 1: Write RED tests for base pinning and exact scope**

Add fixtures with two open rows and one closed row. Assert that `validateReview` rejects a wrong base commit, wrong `frozenAtCommit`, a missing or duplicate decision, an extra decision, `sourceHash` drift, a changed closed row, and a closed-row digest mismatch.

```ts
expect(validateReview({ artifact, baseLedger, currentLedger })).toEqual([]);
expect(validateReview({ artifact: { ...artifact, decisions: [] }, baseLedger, currentLedger }))
  .toContain("scope: expected one decision for every base-open row");
```

- [ ] **Step 2: Run the focused test and observe RED**

Run: `node scripts/run-vitest.mjs run --project unit-node scripts/testing/source-contract-redisposition-review.test.ts`

Expected: FAIL because the module does not exist.

- [ ] **Step 3: Implement canonicalization, Git base loading, and scope validation**

Use only Node built-ins: `node:crypto`, `node:fs/promises`, `node:child_process`, `node:path`, and `node:url`. Resolve the repository root from `import.meta.url`; invoke Git through `execFileSync("git", ["show", ...])` with `shell: false`. Sort decisions by numeric SC ID and produce deterministic issue ordering.

Closed-row bytes are `canonicalJson(baseLedger.rows.filter(row => !baseOpenIds.has(row.id)))`; compare their SHA-256 with `scope.closedRowsDigest` and with the same canonical rows from the current ledger.

- [ ] **Step 4: Write RED tests for the resolution allowlist and class ladder**

Cover every mapping: D1--D5 to `delete`, A1 to `architecture`, T1 to `tool_owned`, and B1--B3 to `behavior`. Reject `UNCLASSIFIED`, class-code-only deletion text, replacement IDs on delete, absent replacement IDs on retained decisions, individual override fields, changed immutable row fields, any changed envelope field or `sourceReaderExceptions`, P0 digest drift, equal/missing `authorRunId` and reviewer `agentTaskId`, a decision whose `authorRunId` differs from the artifact author run, a blind result whose `reviewerRunId` differs from the reviewer task, blind `sourceHash` drift, an unsupported `contextPolicy`, stale sample populations/digests, non-deterministic sample IDs, and mechanically incorrect author/blind comparisons. The distinct recorded run IDs validate the observable consequence of separate executions; the harness-specific clean-context mechanism remains an execution precondition.

```ts
expect(validateReview({ artifact: withDecision("SC-000001", {
  class: "B3_PROTECTED_EXPENSIVE_BEHAVIOR",
  criticalityRef: "AGENTS_SECURITY",
  resolution: { disposition: "behavior", replacementIds: ["test:vitest:src/future.test.ts#keeps key binding"] }
}), baseLedger, currentLedger })).toEqual([]);
```

- [ ] **Step 5: Implement class, P0, criticality, owner, and D5 checks**

Require P0 rows to avoid D1--D5. Require B3 citations to resolve through `criticalitySources`; require every protected row to carry a valid criticality source. Require D4/A1/T1/B1 owner evidence to name an exact existing test/rule/tool owner, while B2/B3 may name unresolved future `test:vitest:` or `test:cargo:` IDs. Reject every proposed `test:playwright:` replacement with `browser owner requires program amendment`. For a simple D5 row, require `lostBehavior` to cover exactly `1..assertionCount`; for a D5 mixed subgroup, require it to cover exactly that subgroup's ordinals. Reject both partial coverage and ordinals outside the relevant set.

- [ ] **Step 6: Write RED tests for mixed rows and idempotent apply**

Test exact ordinal partition, duplicate/missing/out-of-range ordinals, subgroup invariants, forbidden top-level resolution, mixed P0 deletion, stable row ordering, unchanged non-resolution bytes, exactly one trailing newline in serialized output, and identical bytes after two applications.

```ts
const first = applyReview({ artifact, baseLedger, currentLedger });
const second = applyReview({ artifact, baseLedger, currentLedger: first.ledger });
expect(canonicalJson(second.ledger)).toBe(canonicalJson(first.ledger));
```

- [ ] **Step 7: Implement mixed application and CLI modes**

For a simple decision, delete any existing `subgroups` and replace exactly the three top-level resolution keys. For a mixed decision, delete top-level `disposition`, `replacementIds`, and `deletionReason`, then assign the artifact's subgroups. Never mutate inputs. `check` must simulate `apply`, validate `changedPaths`, and test the second application before printing success.

- [ ] **Step 8: Run focused GREEN and repository checks**

Run: `node scripts/run-vitest.mjs run --project unit-node scripts/testing/source-contract-redisposition-review.test.ts`

Expected: PASS with non-empty coverage of both CLI-neutral helpers and all failure classes.

Run: `npm.cmd run check`

Expected: PASS.

- [ ] **Step 9: Commit the carrier checkpoint**

Run: `git diff --check`

Run: `git add scripts/testing/source-contract-redisposition-review.mjs scripts/testing/source-contract-redisposition-review.test.ts`

Run: `git commit -m "test: add source-contract redisposition carrier"`

---

### Task 2: Create the pinned draft and calibrate timing

**Files:**

- Create: `testing/source-contract-redisposition-review.json`
- Modify: `scripts/testing/source-contract-redisposition-review.test.ts`
- Reference: `testing/source-contract-ledger.json`
- Reference: `src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`
- Reference: `src/lib/analysis-report-canvas.behavior.component.test.ts`
- Reference: `src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts`
- Reference: `src/lib/components/research-projects/SourceStatusCell.component.test.ts`

**Interfaces:**

- Consumes the carrier schema from Task 1 and the review base `a54507d63420bb870c3870c91d7e22b050abae3e`.
- Produces a draft with all 436 exact `id + sourceHash` decisions initially marked `UNCLASSIFIED`, immutable catalogs, retained component/legacy timing observations, the base legacy file listing, and the fresh complete-gate observation.
- The draft is expected to fail `check` only for classification/review completeness; any scope, hash, timing, or schema issue is a task failure.

- [ ] **Step 1: Add a RED integration test for the real artifact envelope**

Import the JSON artifact statically as a JSON module in this dedicated carrier test, load the Git base with `loadBaseLedger`, and assert exactly 436 decisions, 235 complementary closed rows, unique IDs, exact source hashes, and the full P0 seed set. Do not read it through `fs` from the test: a new filesystem reader would become a frozen-ledger obligation.

Run: `node scripts/run-vitest.mjs run --project unit-node scripts/testing/source-contract-redisposition-review.test.ts`

Expected: FAIL because the artifact does not exist.

- [ ] **Step 2: Create the deterministic draft artifact**

Populate `reviewBaseCommit: "a54507d63420bb870c3870c91d7e22b050abae3e"`, copy the ledger's exact `frozenAtCommit`, record the current author task/run ID, calculate the closed-row digest through the carrier helper, declare all ten reason classes in ladder order, add exact normative citations, and add one decision per validator-open base row sorted by numeric SC ID. Each draft decision keeps the exact `sourceHash` and author run ID, uses `class: "UNCLASSIFIED"`, and contains no fabricated resolution.

Run the focused integration test again.

Expected: PASS for envelope/scope assertions while the carrier CLI still rejects `UNCLASSIFIED`.

- [ ] **Step 3: Measure the three-file Slice 3B mechanism**

Run this exact command once as an unretained warm-up, then three more times. For every run use a fresh PowerShell stopwatch, require `$LASTEXITCODE -eq 0`, and record only the three retained wall times in seconds:

```powershell
node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/projects-workspace.behavior.component.test.ts src/lib/analysis-report-canvas.behavior.component.test.ts src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts
```

Store the exact command, warm-up exit code, retained values in execution order, and their median as `mechanisms.componentReplacement`.

- [ ] **Step 4: Measure fixed component startup**

Use the same one-warm-up/three-retained procedure for:

```powershell
node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/SourceStatusCell.component.test.ts
```

Store the exact command and observations as `mechanisms.componentStartup`. Do not counterbalance or model AB/BA order.

- [ ] **Step 5: Measure one fresh complete gate**

Before the complete gate, measure the removable legacy owner with the same one-warm-up/three-retained protocol:

`npm.cmd run test:legacy-contract`

Store its three retained times and median as `mechanisms.legacyOwner`. Also run
`node scripts/run-vitest.mjs list --project legacy-contract --filesOnly`, normalize the output to sorted repository-relative paths, and store the exact base listing as `mechanisms.legacyOwner.baseFiles`. This is the baseline later used to prove unchanged owner membership.

Then measure the complete gate:

Preflight that no repository-owned Vitest, Playwright, Cargo, or Rust process is active. Run exactly once, unsandboxed, with a PowerShell stopwatch:

`npm.cmd run verify`

Expected: exit 0. Store its wall time, exit code, exact gate inventory from `scripts/verify.mjs`, and historical comparison `[208.1, 321.3, 383.4]`. Do not rerun a red observation unless process spawn/infrastructure failed; preserve both observations if that exception occurs.

- [ ] **Step 6: Implement and test timing arithmetic validation**

Calculate medians from retained arrays and require:

```text
upperBoundPerRow = T3B / 46
scalablePerRow = max(0, T3B - Tstartup) / 46
upperForecast = upperBoundPerRow * proposedNewJsdomRows
scalableForecast = scalablePerRow * proposedNewJsdomRows
grossAddedForecast = scalableForecast
netGateForecast = max(0, scalableForecast - Tlegacy)
replacementUnitCeiling = 46
netGateForecastPercent = netGateForecast / freshVerifySeconds * 100
legacyDominatesFullUnitCeiling = Tlegacy >= scalablePerRow * replacementUnitCeiling
```

Require `proposedNewJsdomRows <= replacementUnitCeiling`. Reject any Playwright replacement before forecast calculation; it is an explicit program-amendment stop rather than a measured mechanism. Add tests that perturb every derived number, the 46-unit ceiling, the legacy-dominance disclosure, the baseline listing, and jsdom owner grouping and assert a deterministic validation error. The draft may use zero proposed rows until classification is complete, but must contain correct zero forecasts and the measured removable legacy seconds.

- [ ] **Step 7: Commit the measured draft**

Run: `git diff --check`

Run: `git add testing/source-contract-redisposition-review.json scripts/testing/source-contract-redisposition-review.test.ts`

Run: `git commit -m "test: pin redisposition scope and timing evidence"`

---

### Task 3: Calibrate the ladder and review the mandatory cohorts

**Files:**

- Modify: `testing/source-contract-redisposition-review.json`
- Reference: `AGENTS.md`
- Reference: `testing/source-contract-ledger.json`
- Reference: `scripts/testing/repository-rules.mjs`
- Reference: `scripts/testing/repository-rules.test.ts`
- Reference: `src/lib/analysis-crate-boundary-contract.test.ts`
- Reference: `src/lib/crate-extraction-shell-cap-contract.test.ts`

**Interfaces:**

- Consumes the draft and class validator from Tasks 1--2.
- Produces calibrated group rules plus reviewed decisions for all 26 large-contract rows, all P0/security/import/process rows, and all emerging B3/D5/mixed decisions. The 124 live manual rows are classified with their file families in Task 4 and enter blind review only through overlaps with these risk cohorts or the deterministic sample.
- No decision is accepted solely because of its current ledger disposition.

- [ ] **Step 1: Select adversarial calibration cases from the 410-row tail**

For each class, record two or three rows closest to an adjacent-class boundary. Include D1/D2, D3/D5, D4/A1, A1/T1, B1/B2, B2/B3, and B3/D5. For every selected row inspect the complete frozen declaration, its invariant and ordinals, candidate owner implementation, and normative source if claimed.

The author records proposed calibration decisions first. Then dispatch a separate reviewer agent with a clean context containing only each declaration, invariant, ordinals, candidate-owner inventory, class catalog, and normative sources. Do not provide the author's proposed class, reason, artifact decision, discussion, or diff. Store the reviewer's independently written class/reason under `blindResults`; only then calculate each calibration comparison. If no truthful example exists for a class, retain the class catalog but record an empty calibration with `result: "no_match"`; do not manufacture a row.

- [ ] **Step 2: Classify the exact protected and normative-critical cohorts**

Review SC-000555--SC-000560, SC-000511--SC-000515, and SC-000420 first. Then search the open rows for invariants governed by `AGENTS.md` sections 2, 6, or 7 and approved testing-program sections. Add P0 rows only with exact citations. Apply A1/T1/B1/B2 before B3; never assign D1--D5 to P0.

Record security, import-boundary, and process-lifecycle row IDs as explicit mandatory cohorts even when they overlap P0 or manual populations.

Freeze the complete P0 catalog at the end of this step and record its digest in the artifact. If any later tail inspection discovers a credible P0 candidate, stop tail work, return to this step, expand and independently review the protected catalog, rerun every affected group, update the digest, and only then resume. Never append P0 after a tail decision has been accepted without that rollback.

- [ ] **Step 3: Classify and independently review the 26 large-contract rows**

Review all 16 open rows in `analysis-crate-boundary-contract.test.ts` and the 10 open rows in `crate-extraction-shell-cap-contract.test.ts`. SC-000355 remains closed and unchanged. Prefer completed-history deletion, existing repository-rule/Cargo owners, or focused Rust/Node seams before proposing a new behavior owner.

Give the clean-context reviewer the complete 26-row input without proposed decisions. Record the independent output, then calculate the complete 26-ID cohort comparison. Do not give the large files a special class; the same ladder applies.

- [ ] **Step 4: Validate the partial artifact without applying it**

Run: `node scripts/testing/source-contract-redisposition-review.mjs check`

Expected: non-zero only because unreviewed tail decisions remain `UNCLASSIFIED`; there must be no schema, scope, P0, owner, mixed, timing, blind-review, or already-classified-row error.

- [ ] **Step 5: Commit the calibrated mandatory pass**

Run: `git diff --check`

Run: `git add testing/source-contract-redisposition-review.json`

Run: `git commit -m "test: calibrate mandatory redisposition cohorts"`

If the unmatched-group stop condition occurs, leave Tasks 1--3 committed, keep `apply` blocked by `UNCLASSIFIED`, and mark this checkpoint awaiting an approved class amendment. Resume from Task 3 Step 1 after that amendment; do not improvise a local exception.

---

### Task 4: Classify the complete tail and close independent review

**Files:**

- Modify: `testing/source-contract-redisposition-review.json`
- Reference: all 84 remaining live legacy test files named by the artifact
- Reference: existing Vitest/Playwright test declarations, repository rules, tool owners, Cargo metadata, and public seams cited by decisions

**Interfaces:**

- Consumes the calibrated group rules from Task 3.
- Produces zero `UNCLASSIFIED` rows, complete existing/future owner projections, a complete D5 acceptance table, exact new mixed evidence, and all mandated independent-review records.

- [ ] **Step 1: Freeze and validate the seven tail-family manifests**

Derive families in this exact first-match order after excluding the 26 large-contract rows. Store every family's sorted paths and row IDs in the artifact and make the carrier recompute the base counts:

| Family | First-match path predicate | Rows | Files |
| --- | --- | ---: | ---: |
| testing/process infrastructure | starts with `scripts/` | 24 | 7 |
| analysis | contains `analysis` | 152 | 24 |
| projects/library | matches `research-projects`, `project-`, `projects/`, or `library` | 73 | 19 |
| prompt packs/YouTube | matches `prompt-pack` or `youtube-summary` | 53 | 11 |
| Gemini browser | matches `gemini-browser` or `provider-test-console` | 29 | 3 |
| Rust/security boundaries | matches `crate`, `rust-workspace`, `tauri-security`, `external-process`, `hidden-child`, or `focused-rust` | 25 | 6 |
| other product | every remaining tail path | 54 | 14 |

Expected: exactly 410 unique rows in 84 unique files, with no overlap or remainder.

- [ ] **Step 2: Classify testing/process infrastructure and Rust/security boundaries**

Apply calibrated group rules to the 49 rows in these two families. Prefer existing structured/Cargo/process owners. Recheck all protected and normative citations; discovery of a new P0 candidate returns execution to Task 3 Step 2 before either family is accepted.

- [ ] **Step 3: Classify the analysis family**

Apply the ladder to the 152 rows in stable `(path, numeric SC id)` order. Reuse one named group rule for declarations with the same invariant shape and owner mechanism; expand the result to explicit per-row artifact decisions. Inspect ordinal-level exceptions before expanding a rule to avoid hiding mixed rows.

- [ ] **Step 4: Classify projects/library and prompt-pack/YouTube families**

Apply the same group-rule process to the 126 rows in these two families. Prefer existing component/unit/Rust behavior owners before B2/B3. Keep route, component, and Cargo owner identities explicit rather than merging them under a domain label.

- [ ] **Step 5: Classify Gemini browser and other-product families**

Apply the ladder to the remaining 83 rows. A browser-facing invariant does not imply Playwright: test B1/B2 and jsdom seams first. If a row reaches B3 and only Playwright is truthful, leave it `UNCLASSIFIED`, record `browser owner requires program amendment`, and stop Task 4. Do not measure or propose a browser owner inside this checkpoint.

Across Steps 2--5, `manual` remains extraction metadata only. Process its 124 rows in their ordinary families; it neither supplies a class nor triggers automatic 100-percent blind review. Never use a generic D1, D2, or D5 sentence. Each deletion reason names the exact historical fact, implementation detail, unobservable visual, duplicate owner, or accepted behavior loss. When only some ordinals match, emit a mixed resolution with exact subgroup invariants and a full partition.

- [ ] **Step 6: Build the deterministic ten-percent remainder sample**

Construct the remainder after excluding the 26 large rows, all P0/security/import/process rows, all B3/D5 rows, all new mixed rows, and all calibration rows. Manual rows remain eligible unless excluded by one of those risk cohorts. Sort by `sha256(id)` ascending and take `ceil(population.length * 0.10)` IDs. Store both the complete population and selected IDs so the carrier can recompute the sample.

Send the sampled declarations, invariants, ordinals, candidate owners, catalog, and normative sources to the same clean-context reviewer without author decisions. The reviewer returns its own class/reason list before seeing the artifact. The carrier compares outputs. A disagreement updates the group rule, reclassifies all matching rows, and is recorded under `disagreements`.

- [ ] **Step 7: Complete 100-percent review of expensive/loss/mixed decisions**

Prepare a blind packet for every B3 and D5 decision and every newly mixed row after all group-rule changes. The reviewer sees neither the proposed class nor reason and returns a complete independent decision list before comparison. For B3 the independent result must prove both the absence of a cheaper truthful seam and the exact normative citation. For D5 it must verify non-P0 status, absence of normative criticality, exact lost behavior, and exact ordinal coverage.

After every group-rule change from Step 6 or Step 7, recompute the excluded risk cohorts and the Step 6 remainder population. If the population changes, deterministically select the ten-percent sample again and repeat its blind review. Continue until one full iteration changes neither the group rules nor the sample population; record the final population digest and iteration count. The carrier rejects a comparison whose sample was derived from an earlier population.

- [ ] **Step 8: Calculate the final forecast and loss summaries**

Count proposed owners by class/mechanism in rows and assertion ordinals. Assert that no replacement ID uses the Playwright namespace. Set `proposedNewJsdomRows`/ordinals from B3 jsdom rows and subgroups, then recalculate component, gross, removable-legacy, and net fields. Require `proposedNewJsdomRows <= 46`. If the count exceeds 46, keep the completed classifications committed, leave `apply` blocked, and mark the checkpoint awaiting a separate program amendment. Resume at this step after approval; do not change valid row classes to force the number down.

Publish `legacyDominatesFullUnitCeiling = Tlegacy >= scalablePerRow * 46` and the actual comparison of `Tlegacy` with `grossAddedForecastSeconds`. These are explicit diagnostics: net seconds remain disclosure and never determine whether the 46-unit constraint passes.

Build `acceptedLoss.items` from every D5 simple or subgroup decision, sorted by ID then first ordinal. Require the summary's row count, ordinal count, and complete behavior list to equal the decisions mechanically.

- [ ] **Step 9: Run full carrier check and focused tests**

Run: `node scripts/testing/source-contract-redisposition-review.mjs check`

Expected: PASS; `436 decisions, 235 closed rows preserved, 0 unclassified`, with idempotence and forecast checks green.

Run: `node scripts/run-vitest.mjs run --project unit-node scripts/testing/source-contract-redisposition-review.test.ts`

Expected: PASS.

- [ ] **Step 10: Commit the completed review artifact**

Run: `git diff --check`

Run: `git add testing/source-contract-redisposition-review.json`

Run: `git commit -m "test: complete source-contract redisposition review"`

---

### Task 5: Apply the ledger-only cutover atomically

**Files:**

- Modify: `testing/source-contract-ledger.json`
- Reference: `testing/source-contract-redisposition-review.json`
- Reference: `scripts/testing/source-contract-redisposition-review.mjs`

**Interfaces:**

- Consumes the fully checked artifact from Task 4.
- Produces one canonical ledger edit limited to the approved resolution paths; source files remain present, so the transition validator still reports 436 open rows.

- [ ] **Step 1: Prove the pre-apply artifact is green**

Run: `node scripts/testing/source-contract-redisposition-review.mjs check`

Expected: PASS with zero `UNCLASSIFIED` and an idempotent simulated ledger.

- [ ] **Step 2: Normalize ledger formatting in a semantic-no-op commit**

Run the repository's exact carrier canonicalization over the current ledger without applying decisions:

```powershell
node --input-type=module -e "import fs from 'node:fs'; import { canonicalJson } from './scripts/testing/source-contract-redisposition-review.mjs'; const p='testing/source-contract-ledger.json'; fs.writeFileSync(p, canonicalJson(JSON.parse(fs.readFileSync(p, 'utf8'))) + '\n');"
```

Compare `JSON.parse` of the pre-normalized Git bytes with the normalized working file:

```powershell
node --input-type=module -e "import fs from 'node:fs'; import assert from 'node:assert/strict'; import { execFileSync } from 'node:child_process'; const p='testing/source-contract-ledger.json'; const before=JSON.parse(execFileSync('git',['show','HEAD:'+p],{encoding:'utf8'})); const after=JSON.parse(fs.readFileSync(p,'utf8')); assert.deepStrictEqual(after,before);"
```

Expected: exit 0. Then run:

`node scripts/testing/source-contract-redisposition-review.mjs check`

Expected: PASS; normalization changes formatting only and preserves all parsed rows and envelope fields.

Run: `git add testing/source-contract-ledger.json`

Run: `git commit -m "style: normalize source-contract ledger json"`

This commit intentionally absorbs the known repository-wide JSON layout change before the semantic cutover, so the next diff contains resolution changes rather than 1,604 interleaved formatting lines.

- [ ] **Step 3: Apply once through the committed carrier**

Run: `node scripts/testing/source-contract-redisposition-review.mjs apply`

Expected: exit 0 and a summary containing exactly 436 scoped rows; only `testing/source-contract-ledger.json` changes.

- [ ] **Step 4: Prove post-apply idempotence and allowlist**

Run: `node scripts/testing/source-contract-redisposition-review.mjs check`

Expected: PASS.

Run: `node scripts/testing/source-contract-redisposition-review.mjs apply`

Expected: exit 0 with `0 byte changes`.

Use the carrier's `check` output as the primary review surface: it prints decisions grouped by class and legacy path plus the complete changed-JSON-path list. Confirm no envelope, exception, identity, path, title/manual, hash, count, lineage, or invariant path appears. Keep `git diff -- testing/source-contract-ledger.json` as a secondary audit now that normalization is isolated.

- [ ] **Step 5: Run the unchanged transition validator**

Run: `node scripts/validate-testing-transition.mjs`

Expected: PASS with `671 rows, 436 open`. Unresolved future replacement IDs remain valid because their legacy declarations are still live.

- [ ] **Step 6: Prove runner ownership did not change**

Run: `node scripts/run-vitest.mjs list --project legacy-contract --filesOnly`

Expected: byte-identical normalized path listing to `mechanisms.legacyOwner.baseFiles`; disposition edits do not alter `vitest.config.ts` path-derived ownership. Do not rely on the number 73 without this stored baseline.

- [ ] **Step 7: Commit the atomic ledger cutover**

Run: `git diff --check`

Run: `git add testing/source-contract-ledger.json`

Run: `git commit -m "test: apply pre-slice 3c ledger redisposition"`

---

### Task 6: Publish acceptance evidence and close the checkpoint

**Files:**

- Create: `docs/superpowers/verification/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md`
- Reference: `testing/source-contract-redisposition-review.json`
- Reference: `testing/source-contract-ledger.json`
- Do not modify: `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`

**Interfaces:**

- Consumes the committed carrier, artifact, ledger cutover, retained timing observations, and independent-review results.
- Produces the human-readable acceptance record. It does not become a second source of decisions; every number and row list is copied or mechanically rendered from the artifact.

- [ ] **Step 1: Write the evidence record from artifact values**

Include: review/base/frozen commits; exact scope reconciliation; before/after disposition counts; class counts; projected existing versus future owners by mechanism; timing commands and all retained observations; jsdom upper/scalable forecasts; explicit confirmation that no Playwright owner was proposed; removable legacy time/files; gross and net forecasts; the binding 46-row result; legacy-dominance diagnostics; fresh verify comparison; independent blind-review populations and disagreements; and confirmation that the program index was intentionally not changed.

Add a dedicated `Accepted D5 Losses` table with one row per artifact item and columns `SC ID`, `ordinals`, and `lost behavior`. State total D5 row and ordinal counts immediately above it.

- [ ] **Step 2: Run read-only independent review of the final diff**

Give a fresh reviewer the approved spec subsection, this plan, the review artifact, carrier, carrier tests, and the carrier's class/path-grouped decision report. Require findings-first review of scope pinning, class ordering, P0/B3/D5 boundaries, all mandatory cohorts, mixed partitions, owner evidence, timing arithmetic, closed-row preservation, and JSON-path allowlist. The normalized ledger diff is secondary evidence, not the primary 592-KB review object.

If findings exist, hand all findings to one fix owner in one wave. Rerun only the carrier test and `check` while fixing. Do not rerun the complete gate until the review is clean.

- [ ] **Step 3: Run the final narrow checkpoint once**

Run: `node scripts/run-vitest.mjs run --project unit-node scripts/testing/source-contract-redisposition-review.test.ts`

Expected: PASS.

Run: `node scripts/testing/source-contract-redisposition-review.mjs check`

Expected: PASS.

Run: `node scripts/validate-testing-transition.mjs`

Expected: PASS with 436 open rows.

Run: `npm.cmd run check`

Expected: PASS.

- [ ] **Step 4: Run one final unsandboxed complete gate**

Preflight repository-owned processes, then run exactly once:

`npm.cmd run verify`

Expected: PASS. Record the observed duration and result in the verification document. A retry is permitted only for a documented infrastructure/spawn failure; preserve both observations.

- [ ] **Step 5: Commit the acceptance checkpoint**

Run: `git diff --check`

Run: `git status --short`

Expected: only the verification document is uncommitted.

Run: `git add docs/superpowers/verification/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md`

Run: `git commit -m "docs: verify pre-slice 3c redisposition"`

- [ ] **Step 6: Confirm the handoff boundary**

Run: `git status --short`

Expected: empty.

Run: `git log --oneline -7`

Expected: carrier, measured draft, calibrated review, completed review, formatting normalization, ledger cutover, and verification commits are visible. Only after this checkpoint is approved may a separate detailed Slice 3C implementation plan be written from the new ledger state.
