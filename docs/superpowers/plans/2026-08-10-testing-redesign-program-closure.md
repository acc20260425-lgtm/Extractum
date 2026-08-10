# Testing Redesign Program Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the completed Testing Redesign program by deleting its
transition apparatus and stale authority while retaining the current daily
loop and exactly three durable RepositoryIndex rules.

**Architecture:** Make four subtractive commits: retire transition authority,
reduce RepositoryIndex and coupled scaffolding, align commands and current
documentation, then delete the program corpus and close on one complete gate.
Measurements use the approved `c076c002` baseline from the design; this plan
adds no measurement or evidence mechanism.

**Tech Stack:** Node.js ESM, TypeScript, Vitest, Playwright, SvelteKit, Cargo, PowerShell, Git.

## Global Constraints

- **If a closure goal requires a new mechanism, exclude the goal; do not add
  the mechanism.**
- When this plan and the actual code disagree, the code is authoritative.
  Adjust the plan silently; do not create a separate plan-hardening commit.
- Start from a clean tree with
  `c076c002a8debcce4a5bc71226077c03884367e7` in `HEAD` ancestry. Baseline
  figures already live in the approved design and are not re-measured.
- Product behavior, product architecture, `/analysis`, Rust source, migrations, and runtime configuration are out of scope.
- Add no test file, rule, evaluator ability, runner, project, dependency,
  command family, status vocabulary, manifest, allowlist, registry, ledger,
  evidence carrier, cache, selector, scheduler, watcher, or timing store.
- `testing/` must be absent at completion. `scripts/testing/` must finish below
  its baseline of 21 files and 13,349 physical lines.
- RepositoryIndex must register exactly
  `rule:extractum-grid-wrapper-boundary`,
  `rule:telegram-crate-dependency-ownership`, and
  `rule:telegram-crate-manifest-boundary`.
- Preserve `scripts/telegram-grammers-feature-baseline.mjs`,
  `src/lib/telegram-grammers-feature-baseline.json`, and their
  `.gitattributes` line.
- The existing real-tree RepositoryIndex test case may be retargeted to the
  retained registry; do not add a test case or execution layer.
- Delete rather than archive the 18 program documents. Delete this design and plan in the final commit; Git is the recovery path.
- Do not create a closure verification document. Record the baseline OID,
  aggregate reduction, and actual successful `verify` duration in the final
  commit body.
- Keep the existing daily frontend and focused Rust loops. Changed/related selection is Vitest-only; focused commands are accelerators, not completion evidence.
- On Windows use `npm.cmd` for repository scripts. For sandbox-only process failures, rerun the exact command through the environment's approval mechanism; do not change the test.
- If deleting an item exposes a durable invariant that cannot remain owned without a new mechanism, retain the item unchanged and drop that deletion goal.

---

### Task 1: Retire the Completed Transition Authority

**Files:**

- Modify: `scripts/verify.mjs`
- Modify: `scripts/verify.test.ts`
- Modify: `scripts/testing/test-conventions.test.ts`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `vitest.config.ts`
- Modify: `.gitattributes`
- Delete: `scripts/validate-testing-transition.mjs`
- Delete: `scripts/testing/extract-source-contract-ledger.mjs`
- Delete: `scripts/testing/run-observation.mjs`
- Delete: `scripts/testing/run-observation.behavior.test.ts`
- Delete: `scripts/testing/slice-1-baseline.mjs`
- Delete: `scripts/testing/slice-1-baseline.behavior.test.ts`
- Delete: `scripts/testing/slice-1-rust-feasibility.mjs`
- Delete: `scripts/testing/slice-1-rust-feasibility.test.ts`
- Delete: `scripts/testing/source-contract-redisposition-review.mjs`
- Delete: `scripts/testing/source-contract-redisposition-review.test.ts`
- Delete: `scripts/testing/testing-transition.mjs`
- Delete: `scripts/testing/timing-log.mjs`
- Delete: `scripts/testing/timing-log.test.ts`
- Delete: `scripts/testing/validate-testing-transition.test.ts`
- Delete: all 29 tracked files under `testing/`

**Interfaces:** Removes completed ledger/census/evidence validation from `verify`; leaves all current RepositoryIndex rules executable until Task 2.

- [ ] **Step 1: Confirm the starting checkpoint**

```powershell
if (@(git status --porcelain).Count -ne 0) { git status --short; throw 'closure requires a clean tree' }
git merge-base --is-ancestor c076c002a8debcce4a5bc71226077c03884367e7 HEAD
if ($LASTEXITCODE -ne 0) { throw 'Slice 3C baseline is not an ancestor of HEAD' }
```

Expected: clean status and exit 0.

- [ ] **Step 2: Remove the transition owner and sever its imports**

- Remove only the transition-validator step from `createVerifySteps()` and update its existing test to describe the remaining gates without duplicating the full step implementation.
- Remove ledger/extractor imports and ledger-derived assertions from `test-conventions.test.ts` and `repository-rules.test.ts`.
- Retarget the existing real-tree RepositoryIndex case in place from the six analysis IDs to `registeredRuleIds`; do not add or move a test case.
- Remove `run-observation.behavior.test.ts` from the OS Vitest owner list. Keep the four process-shell diagnostic owners.
- Delete the files listed above and the complete `testing/` tree. Remove only `testing/source-contract-redisposition-evidence/*.json -text` from `.gitattributes`.

- [ ] **Step 3: Run the remaining owners**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
npm.cmd run test:integration:os
```

Expected: both commands pass; the OS project remains non-empty.

- [ ] **Step 4: Commit**

```powershell
git diff --check
git add -A -- testing scripts/testing scripts/validate-testing-transition.mjs scripts/verify.mjs scripts/verify.test.ts vitest.config.ts .gitattributes
git diff --cached --check
git commit -m "test: retire completed testing transition"
```

---

### Task 2: Retain Three Repository Boundaries and Remove Coupled Scaffolding

**Files:**

- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `scripts/testing/test-conventions.test.ts`
- Modify: `docs/value-registry.md`
- Modify: `.gitattributes`
- Delete: `src/lib/analysis-contract-paths.ts`
- Delete: `src/lib/prompt-pack-contract-paths.ts`
- Delete: `src/lib/telegram-contract-paths.ts`
- Delete: `src/lib/telegram-contract-paths.behavior.test.ts`
- Delete: `src/lib/telegram-8b-staging-sha256.json`
- Delete: `src/lib/telegram-8b-symbol-map.json`
- Delete: `src/lib/telegram-8b-test-identities.json`
- Delete: `scripts/telegram-staging-sha256.mjs`
- Delete: `scripts/telegram-8b-symbol-map.mjs`
- Delete: `scripts/telegram-8b-test-identities.mjs`
- Delete: `.superpowers/sdd/reviewfix-rules-report.md`

**Interfaces:** Produces exactly three registered real-tree rules and preserves the Grammers generator/JSON authority pair.

- [ ] **Step 1: Reduce RepositoryIndex by reachability**

- Keep the three approved evaluators and only the imports, constants, helpers, and fixtures they transitively use.
- Delete all analysis, Extractum LLM inventory, and Telegram Phase 8B evaluators and fixtures. Do not replace them or preserve unused parser helpers.
- Keep the unknown-rule check, mutation coverage, reordered Grammers-baseline check, and the existing real-tree case.
- Make both registry assertions exact:

```ts
expect(registeredRuleIds).toEqual([
  "rule:extractum-grid-wrapper-boundary",
  "rule:telegram-crate-dependency-ownership",
  "rule:telegram-crate-manifest-boundary",
]);
```

- [ ] **Step 2: Delete coupled scaffolding and transitional metadata**

- Delete every file listed in this task after its apparatus consumers are gone.
- Remove the two deleted Phase 8B JSON entries from `.gitattributes`; retain the Grammers baseline entry unchanged.
- In `docs/value-registry.md`, delete the Telegram crate-preparation lifecycle, Testing tooling decision evidence, and source-contract ledger disposition sections.
- Reduce the Vitest project table to `unit-node`, `component`, `architecture`, and `os-integration`; remove `legacy-contract` and runner-census wording.

- [ ] **Step 3: Run the retained rule and compiler-visible owners**

```powershell
npm.cmd run test:unit -- scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
node scripts/telegram-grammers-feature-baseline.mjs --check
npm.cmd run check
```

Expected: all commands pass; the registry and fixtures cover exactly three IDs.

- [ ] **Step 4: Commit**

```powershell
git diff --check
git add -A -- scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts docs/value-registry.md .gitattributes src/lib/analysis-contract-paths.ts src/lib/prompt-pack-contract-paths.ts src/lib/telegram-contract-paths.ts src/lib/telegram-contract-paths.behavior.test.ts src/lib/telegram-8b-staging-sha256.json src/lib/telegram-8b-symbol-map.json src/lib/telegram-8b-test-identities.json scripts/telegram-staging-sha256.mjs scripts/telegram-8b-symbol-map.mjs scripts/telegram-8b-test-identities.mjs .superpowers/sdd/reviewfix-rules-report.md
git diff --cached --check
git commit -m "test: retain three current repository boundaries"
```

---

### Task 3: Consolidate the Current Command and Documentation Surface

**Files:**

- Modify: `package.json`
- Modify: `docs/project.md`
- Modify: `AGENTS.md`
- Delete: `scripts/verify-stability.mjs`
- Delete: `scripts/verify-stability.test.ts`

**Interfaces:** Leaves one current daily loop, six explicit JavaScript/browser owner commands, focused Rust accelerators, and the complete `verify` gate.

- [ ] **Step 1: Remove dead commands and the one-time audit**

- Delete `test:project-runs`, `verify:project-runs`, and `verify:stability` from `package.json`.
- Delete both stability-audit files. Keep `test:rust:prompt-pack-runs` and `run-vitest` unchanged.

- [ ] **Step 2: Rewrite current-state documentation**

- In `docs/project.md`, name `test:unit`, `test:component`, `test:architecture`, `test:integration:os`, `test:e2e`, and `test:app:e2e`, plus the bootstrap prerequisite, complete `verify` composition, changed/related commands, broad frontend `check`, and focused Rust role.
- State only the operational boundaries a reader can act on: there is no cross-stack changed-file selection, `changed` / `related` run Vitest only, and the repository has no CI workflow.
- Remove the stale project-runs block without deleting the following prompt-pack architecture text.
- Add the same concise Vitest-only limitation to `AGENTS.md`; keep its bootstrap and Rust loops unchanged.

- [ ] **Step 3: Run current owners**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/test-conventions.test.ts
npm.cmd run check
```

Expected: both commands pass and every documented command exists in `package.json`.

- [ ] **Step 4: Commit**

```powershell
git diff --check
git add -A -- package.json docs/project.md AGENTS.md scripts/verify-stability.mjs scripts/verify-stability.test.ts
git diff --cached --check
git commit -m "docs: consolidate the current testing loop"
```

---

### Task 4: Delete the Program Corpus and Close

**Files:**

- Delete: `docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md`
- Delete: `docs/superpowers/specs/2026-08-05-testing-redesign-slice-3b-design.md`
- Delete: `docs/superpowers/specs/2026-08-08-testing-redesign-slice-3c-design.md`
- Delete: `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`
- Delete: `docs/superpowers/plans/2026-08-02-testing-redesign-slice-1-measurement.md`
- Delete: `docs/superpowers/plans/2026-08-02-testing-redesign-slice-2a-migration-preflight.md`
- Delete: `docs/superpowers/plans/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md`
- Delete: `docs/superpowers/plans/2026-08-03-testing-redesign-slice-3a-source-contracts.md`
- Delete: `docs/superpowers/plans/2026-08-05-testing-redesign-slice-3b-component-contracts.md`
- Delete: `docs/superpowers/plans/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md`
- Delete: `docs/superpowers/plans/2026-08-08-testing-redesign-slice-3c-source-contract-completion.md`
- Delete: `docs/superpowers/verification/2026-08-02-testing-redesign-slice-1-measurement.md`
- Delete: `docs/superpowers/verification/2026-08-02-testing-redesign-slice-2a-migration-preflight.md`
- Delete: `docs/superpowers/verification/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md`
- Delete: `docs/superpowers/verification/2026-08-03-testing-redesign-slice-3a-source-contracts.md`
- Delete: `docs/superpowers/verification/2026-08-05-testing-redesign-slice-3b-component-contracts.md`
- Delete: `docs/superpowers/verification/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md`
- Delete: `docs/superpowers/verification/2026-08-08-testing-redesign-slice-3c.md`
- Delete: `docs/superpowers/specs/2026-08-10-testing-redesign-program-closure-design.md`
- Delete: `docs/superpowers/plans/2026-08-10-testing-redesign-program-closure.md`

**Interfaces:** Produces no active Testing Redesign corpus, one final green `verify`, and a closure commit body as the only closure record.

- [ ] **Step 1: Delete exactly the 20 documents**

Use `apply_patch`; do not archive them or create a replacement index or verification record.

- [ ] **Step 2: Run the single final inventory check**

```powershell
if (Test-Path -LiteralPath 'testing') { throw 'testing/ remains' }
$scriptFiles = @(Get-ChildItem -LiteralPath 'scripts/testing' -File)
$scriptLines = ($scriptFiles | Get-Content | Measure-Object -Line).Lines
if ($scriptFiles.Count -ge 21 -or $scriptLines -ge 13349) { throw 'scripts/testing is not net-negative' }
node --input-type=module -e "const {registeredRuleIds}=await import('./scripts/testing/repository-rules.mjs'); const expected=['rule:extractum-grid-wrapper-boundary','rule:telegram-crate-dependency-ownership','rule:telegram-crate-manifest-boundary']; if(JSON.stringify(registeredRuleIds)!==JSON.stringify(expected)) throw new Error(JSON.stringify(registeredRuleIds));"
if ($LASTEXITCODE -ne 0) { throw 'unexpected RepositoryIndex registry' }
$retiredHits = @(
  rg -n 'validate-testing-transition|testing-transition|source-contract-ledger|source-contract-redisposition|runner-census|run-observation|slice-1-|timing-log|telegram-8b-|analysis-contract-paths|prompt-pack-contract-paths|telegram-contract-paths|verify-stability|test:project-runs|verify:project-runs|rule:analysis-|rule:extractum-llm-public-api|rule:telegram-phase-8b' package.json scripts src vitest.config.ts .gitattributes 2>$null
  rg -n 'Telegram crate-preparation agent lifecycle|Testing tooling decision evidence|Source-contract migration ledger dispositions|8b-checkpoint-|8c-extracted|ADOPT_NX|REJECT_NX|legacy-contract' docs/value-registry.md 2>$null
  rg -n 'testing-redesign|testing-infrastructure-redesign' docs AGENTS.md README.md 2>$null
)
if ($retiredHits.Count -ne 0) { $retiredHits; throw 'retired authority remains' }
if (@(Select-String -LiteralPath '.gitattributes' -SimpleMatch 'src/lib/telegram-grammers-feature-baseline.json text eol=lf').Count -ne 1) { throw 'Grammers authority line is missing or duplicated' }
if (@(git diff --diff-filter=A --name-only c076c002a8debcce4a5bc71226077c03884367e7 --).Count -ne 0) { throw 'closure introduced a permanent file' }
```

Expected: no hits; `testing/` absent; `scripts/testing/` net-negative; exactly three rules; no permanent added file.

- [ ] **Step 3: Stage the corpus, run the complete gate, and commit**

```powershell
npm.cmd run check:gemini-browser-sidecar-binary
if ($LASTEXITCODE -ne 0) { npm.cmd run bootstrap:testing }
git add -A -- docs/superpowers
$stagedDocs = @(git diff --cached --name-status)
if ($stagedDocs.Count -ne 20 -or @($stagedDocs | Where-Object { $_ -notmatch '^D\s' }).Count -ne 0) { throw 'expected exactly 20 staged document deletions' }
git diff --cached --check
$verifyTimer = [System.Diagnostics.Stopwatch]::StartNew()
npm.cmd run verify
$verifyExit = $LASTEXITCODE
$verifyTimer.Stop()
if ($verifyExit -ne 0) { throw 'final verify failed; closure remains open' }
$verifySeconds = [Math]::Round($verifyTimer.Elapsed.TotalSeconds, 1)
$scriptFiles = @(Get-ChildItem -LiteralPath 'scripts/testing' -File)
$scriptLines = ($scriptFiles | Get-Content | Measure-Object -Line).Lines
$reduction = "Reduction: testing/ 29 files -> absent; scripts/testing/ 21 files and 13349 lines -> $($scriptFiles.Count) files and $scriptLines lines; RepositoryIndex 11 -> 3 rules; program corpus 18 -> 0 files."
git commit -m "chore: close testing redesign program" -m "Baseline: c076c002a8debcce4a5bc71226077c03884367e7." -m $reduction -m "Verification: npm.cmd run verify PASS in $verifySeconds seconds."
if ($LASTEXITCODE -ne 0) { throw 'closure commit failed' }
git status --short
```

Expected: the sidecar prerequisite exists, all retained gates pass, the commit contains exactly the 20 deletions, and final status is clean.

If the complete gate fails for a repository reason: restore the 20 documents, make the correction as a separate focused commit, return to a clean tree, and repeat Task 4. Do not create evidence or a retry protocol.
