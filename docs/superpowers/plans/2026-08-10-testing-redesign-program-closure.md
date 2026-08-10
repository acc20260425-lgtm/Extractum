# Testing Redesign Program Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the completed Testing Redesign program by deleting its transition apparatus and stale authority while retaining the current daily loop and exactly three durable RepositoryIndex rules.

**Architecture:** Execute four subtractive commits. First remove the completed ledger/census/validator authority while keeping all current rules executable; then reduce RepositoryIndex and its coupled scaffolding to the three approved boundaries; then align package commands and current-state documentation; finally delete the historical program corpus, prove independent net-negative metrics, run the one intended complete gate, and record the result in the closure commit.

**Tech Stack:** Node.js ESM, TypeScript, Vitest 4 projects, Playwright 2 owners, SvelteKit/Svelte, Cargo workspace, PowerShell on Windows, Git.

## Global Constraints

- **If a closure goal requires a new mechanism, exclude the goal; do not add the mechanism.**
- Use exact baseline `c076c002a8debcce4a5bc71226077c03884367e7` for every before/after measurement.
- Use Git objects and one-off shell commands for measurement; do not add a measurement script, baseline registry, or evidence schema.
- Create no test file, rule ID, runner, project, dependency, command family, status vocabulary, manifest, allowlist, registry, ledger, evidence carrier, cache, selector, scheduler, watcher, or timing store.
- Product behavior, product architecture, `/analysis`, Rust source, migrations, and runtime configuration are unchanged.
- `testing/` must be absent at completion.
- RepositoryIndex must register exactly `rule:extractum-grid-wrapper-boundary`, `rule:telegram-crate-dependency-ownership`, and `rule:telegram-crate-manifest-boundary`.
- Preserve `scripts/telegram-grammers-feature-baseline.mjs`, `src/lib/telegram-grammers-feature-baseline.json`, and their `.gitattributes` authority line.
- Delete, rather than archive, the 18 approved Testing Redesign documents. Git at the baseline OID is their recovery path.
- The design and this plan are temporary artifacts and must be deleted by the final closure commit.
- Do not create a closure verification document. Record the baseline OID, aggregate net reduction, and successful `verify` duration in the final commit body.
- On Windows use `npm.cmd`, never `npm`, for repository scripts.
- Use focused existing owner commands during Tasks 1-3. Only Task 4 runs the intended complete `npm.cmd run verify`.
- If an OS-process owner fails only because a sandbox denies process launch,
  task termination, or termination proof, rerun the exact command through the
  current environment's approval/escalation mechanism. Do not use the Vite
  hidden-host fallback, and do not weaken or rewrite the test.
- If a deletion uncovers a durable invariant that cannot remain owned without a
  new mechanism, retain the existing item unchanged and drop that deletion
  goal. Do not create a replacement owner or deferred closure queue.
- If the complete gate fails, stop the closure, diagnose with the smallest existing non-empty owner, correct or revert the responsible deletion, and make the next fresh complete run the final attempt.

---

### Task 1: Retire the Completed Transition Authority

**Files:**

- Modify: `scripts/verify.test.ts:5-28`
- Modify: `scripts/verify.mjs:13-28`
- Modify: `scripts/testing/test-conventions.test.ts:1-70,171-238`
- Modify: `scripts/testing/repository-rules.test.ts:1-8,1041-1168`
- Modify: `vitest.config.ts:9-15`
- Modify: `.gitattributes:16`
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
- Delete: all 29 tracked objects under `testing/`

**Interfaces:**

- Consumes: `createVerifySteps()`, `runVerification()`, `createRepositoryIndex()`, `evaluateRule()`, `registeredRuleIds`, the existing 11-rule registry, and four Vitest project definitions.
- Produces: a `verify` pipeline without transition validation; current rule tests independent of the deleted ledger; four non-empty Vitest projects; no `testing/` directory; all 11 rules still registered until Task 2.

- [ ] **Step 1: Reconfirm the clean Git baseline before changing the tree**

Run this one-off PowerShell measurement directly from Git objects; do not save
it as a repository script or evidence file:

```powershell
$closureBaseline = 'c076c002a8debcce4a5bc71226077c03884367e7'
if (@(git status --porcelain).Count -ne 0) { git status --short; throw 'closure must start from a clean tree' }
git merge-base --is-ancestor $closureBaseline HEAD
if ($LASTEXITCODE -ne 0) { throw 'approved Slice 3C baseline is not an ancestor of HEAD' }

function Get-ClosureGitMetric {
  param([string[]]$Paths)
  $bytes = 0L
  $lines = 0L
  foreach ($path in $Paths) {
    $bytes += [int64](git cat-file -s "${closureBaseline}:$path")
    if ($LASTEXITCODE -ne 0) { throw "cannot size ${closureBaseline}:$path" }
    $blobLines = @(git show "${closureBaseline}:$path")
    if ($LASTEXITCODE -ne 0) { throw "cannot read ${closureBaseline}:$path" }
    $lines += $blobLines.Count
  }
  [pscustomobject]@{ Files = $Paths.Count; Bytes = $bytes; Lines = $lines }
}

$baselineTestingPaths = @(git ls-tree -r --name-only $closureBaseline -- testing)
$baselineScriptPaths = @(git ls-tree -r --name-only $closureBaseline -- scripts/testing)
$baselineProgramPaths = @(
  git ls-tree -r --name-only $closureBaseline -- docs/superpowers |
    Where-Object { $_ -match 'testing-redesign|testing-infrastructure-redesign' }
)
$baselineTesting = Get-ClosureGitMetric -Paths $baselineTestingPaths
$baselineScripts = Get-ClosureGitMetric -Paths $baselineScriptPaths
$baselineProgram = Get-ClosureGitMetric -Paths $baselineProgramPaths
$baselineRuleSource = (git show "${closureBaseline}:scripts/testing/repository-rules.mjs") -join "`n"
$baselineRuleIds = @(
  [regex]::Matches($baselineRuleSource, 'rule:[a-z0-9-]+') |
    ForEach-Object { $_.Value } |
    Sort-Object -Unique
)

if ($baselineTesting.Files -ne 29 -or $baselineTesting.Bytes -ne 2881023 -or $baselineTesting.Lines -ne 30043) { throw "unexpected testing/ baseline: $($baselineTesting | ConvertTo-Json -Compress)" }
if ($baselineScripts.Files -ne 21 -or $baselineScripts.Bytes -ne 708939 -or $baselineScripts.Lines -ne 13349) { throw "unexpected scripts/testing/ baseline: $($baselineScripts | ConvertTo-Json -Compress)" }
if ($baselineProgram.Files -ne 18 -or $baselineProgram.Bytes -ne 519798 -or $baselineProgram.Lines -ne 7901) { throw "unexpected program-corpus baseline: $($baselineProgram | ConvertTo-Json -Compress)" }
if ($baselineRuleIds.Count -ne 11) { throw "unexpected RepositoryIndex baseline: $($baselineRuleIds.Count) rules" }

$baselineTesting | Format-List
$baselineScripts | Format-List
$baselineProgram | Format-List
Write-Output "REPOSITORY_RULES=$($baselineRuleIds.Count)"
```

Expected: clean status; `c076c002` is an ancestor; the three metric records are
respectively `29 / 2,881,023 / 30,043`, `21 / 708,939 / 13,349`, and
`18 / 519,798 / 7,901`; `REPOSITORY_RULES=11`.

- [ ] **Step 2: Change the verify contract first**

Replace the first test in `scripts/verify.test.ts` with this exact current-gate contract:

```ts
it("runs distinct adapter and app e2e gates between the sidecar prerequisite and preserved static gates", () => {
  const steps = createVerifySteps({ npmExecPath: "npm-cli.js", platform: "win32" });
  expect(steps.map((step) => step.npmScript ?? step.command)).toEqual([
    "check:gemini-browser-sidecar-binary",
    "test:unit",
    "test:component",
    "test:architecture",
    "test:integration:os",
    "test:e2e",
    "test:app:e2e",
    "check",
    "check:rustfmt",
    "cargo",
    "cargo",
    "git",
  ]);
  expect(steps.flatMap((step) => step.args ?? [])).not.toContain("scripts/validate-testing-transition.mjs");
  const npmScripts = steps.filter((step) => step.npmScript).map((step) => step.npmScript);
  expect(npmScripts).not.toContain("test");
  expect(npmScripts).not.toContain("bootstrap:testing");
  expect(npmScripts).not.toContain("build:gemini-browser-sidecar");
});
```

- [ ] **Step 3: Run the changed verify contract and observe RED**

Run:

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts
```

Expected: FAIL because `createVerifySteps()` still returns the transition validator as its second step and the actual step list is one entry longer.

- [ ] **Step 4: Remove transition validation from the verify pipeline**

Delete only this entry from `createVerifySteps()` in `scripts/verify.mjs`:

```js
{ title: "node scripts/validate-testing-transition.mjs", command: process.execPath, args: ["scripts/validate-testing-transition.mjs"] },
```

The resulting step prefix must be exactly:

```js
return [
  npmStep("npm run check:gemini-browser-sidecar-binary", "check:gemini-browser-sidecar-binary", { npmExecPath, platform }),
  npmStep("npm run test:unit", "test:unit", { npmExecPath, platform }),
  npmStep("npm run test:component", "test:component", { npmExecPath, platform }),
  npmStep("npm run test:architecture", "test:architecture", { npmExecPath, platform }),
  npmStep("npm run test:integration:os", "test:integration:os", { npmExecPath, platform }),
```

Do not reorder or remove any later step.

- [ ] **Step 5: Decouple current test conventions from the ledger and extractor**

In `scripts/testing/test-conventions.test.ts`:

1. Remove these imports:

```ts
import ts from "typescript";
import sourceContractLedger from "../../testing/source-contract-ledger.json";
import { discoverSourceReaders } from "./extract-source-contract-ledger.mjs";
```

2. Delete `isInspectedVitestSource()` and `directTextReaderViolations()` completely.
3. Delete the two `directTextReaderViolations(...)` assertions from the `keeps production source inspection behind the structured rule runner` test.
4. Keep the existing 11-ID `registeredRuleIds` expectation unchanged in this task.
5. Keep the `telegram-contract-paths.behavior.test.ts` ownership expectation until Task 2 deletes that test.

The retained structured-runner test ends immediately after the 11-ID assertion:

```ts
expect(registeredRuleIds).toEqual([
  "rule:analysis-evidence-highlight-token-styling",
  "rule:analysis-source-browser-canonical-composition",
  "rule:analysis-source-browser-explicit-subject-contract",
  "rule:analysis-source-group-activity-boundary",
  "rule:analysis-source-group-tab-leaf-boundary",
  "rule:analysis-source-reader-surface-composition",
  "rule:extractum-grid-wrapper-boundary",
  "rule:extractum-llm-public-api-boundary",
  "rule:telegram-crate-dependency-ownership",
  "rule:telegram-crate-manifest-boundary",
  "rule:telegram-phase-8b-authority-integrity",
]);
```

- [ ] **Step 6: Make repository-rule tests independent of the ledger while preserving enforcement**

In `scripts/testing/repository-rules.test.ts`:

1. Remove the `sourceContractLedger` import.
2. Remove the `allowedRuleIds` construction and the ledger-derived size/membership assertions.
3. Rename the registry test to `registers the current structured rule set` and keep the exact 11-ID expectation from Step 4.
4. Retarget the existing `accepts the current repository snapshot for all six
   analysis source-reader rules` case in place instead of adding or moving a
   test. Rename it as below and replace its `analysisRuleIds` loop with the
   existing registry:

```ts
it("accepts the current repository for every registered rule", () => {
  const index = realAuthorityIndex();
  for (const id of registeredRuleIds) {
    expect(evaluateRule({ id, index }), id).toEqual({ id, violations: [] });
  }
});
```

Keep all existing positive/mutation fixture coverage in this task.

- [ ] **Step 7: Remove the retired OS observation owner**

Delete only this element from `OS_INTEGRATION_FILES` in `vitest.config.ts`:

```ts
"scripts/testing/run-observation.behavior.test.ts",
```

The OS project remains non-empty with the four `scripts/process-shell-diagnostic/` owners.

- [ ] **Step 8: Delete the transition script families and `testing/` tree**

Use `apply_patch` to delete every file named in this task, including all of `testing/`. Delete `scripts/validate-testing-transition.mjs` only after Steps 3-5 have severed its live consumers.

Remove only this `.gitattributes` line now:

```gitattributes
testing/source-contract-redisposition-evidence/*.json -text
```

Keep all three Telegram JSON lines for Task 2, especially the Grammers baseline line.

- [ ] **Step 9: Run the focused current-owner tests**

Run:

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
```

Expected: PASS; all three files are selected, the current 11 rules accept the repository, and no import resolves through `testing/`.

Run:

```powershell
npm.cmd run test:integration:os
```

Expected: PASS with a non-empty four-file OS selection.

- [ ] **Step 10: Prove the transition authority is gone from live code**

Run:

```powershell
$closureTransitionPattern = 'runner-census|source-contract-ledger|source-contract-redisposition|validate-testing-transition|extract-source-contract-ledger|testing-transition|run-observation|slice-1-baseline|slice-1-rust-feasibility|timing-log'
$closureTransitionHits = @(rg -n $closureTransitionPattern package.json scripts src vitest.config.ts .gitattributes 2>$null)
if ($closureTransitionHits.Count -ne 0) { $closureTransitionHits; throw 'live transition references remain' }
if (Test-Path -LiteralPath 'testing') { throw 'testing directory still exists' }
```

Expected: exit 0, no hits, and no `testing/` directory.

- [ ] **Step 11: Commit the retired transition authority**

Run:

```powershell
git status --short
git diff --check
git add -A -- testing scripts/testing scripts/validate-testing-transition.mjs scripts/verify.mjs scripts/verify.test.ts vitest.config.ts .gitattributes
git diff --cached --check
git commit -m "test: retire completed testing transition"
```

Expected: the commit contains only the files named in this task; no product or Rust file is changed.

---

### Task 2: Reduce RepositoryIndex and Remove Coupled Scaffolding

**Files:**

- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `scripts/testing/test-conventions.test.ts`
- Modify: `docs/value-registry.md:80-106,955-993`
- Modify: `.gitattributes:13-15`
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

**Interfaces:**

- Consumes: the ledger-free 11-rule engine and real-tree registry test from Task 1.
- Produces: exactly three registered and executable rules; retained Grammers feature authority; no analysis/LLM/Phase 8B rule code, fixtures, modules, generators, JSON, or workflow values.

- [ ] **Step 1: Change both registry expectations to exactly three rules**

In both `scripts/testing/repository-rules.test.ts` and `scripts/testing/test-conventions.test.ts`, replace the 11-ID expectation with:

```ts
expect(registeredRuleIds).toEqual([
  "rule:extractum-grid-wrapper-boundary",
  "rule:telegram-crate-dependency-ownership",
  "rule:telegram-crate-manifest-boundary",
]);
```

Also remove this soon-to-be-deleted ownership assertion from `test-conventions.test.ts`:

```ts
expectProjectOwners(projectConventions, "src/lib/telegram-contract-paths.behavior.test.ts", ["unit-node"]);
```

- [ ] **Step 2: Run the registry tests and observe RED**

Run:

```powershell
npm.cmd run test:unit -- scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
```

Expected: FAIL because `registeredRuleIds` still contains 11 IDs.

- [ ] **Step 3: Reduce `repository-rules.mjs` to the retained dependency graph**

Keep only this import:

```js
import { generateFeatureBaseline } from "../telegram-grammers-feature-baseline.mjs";
```

Delete the `node:crypto`, `telegram-8b-test-identities`, and `telegram-8b-symbol-map` imports.

Keep only these top-level rule constants and their literal values:

```js
const DATA_GRID_PATH = "src/lib/components/extractum-ui/DataGrid.svelte";
const TREE_DATA_GRID_PATH = "src/lib/components/extractum-ui/TreeDataGrid.svelte";
const GRID_RUNTIME_PATH = "src/lib/components/extractum-ui/data-grid-date-format.ts";
const APPROVED_SVAR_GRID_PATHS = new Set([DATA_GRID_PATH, TREE_DATA_GRID_PATH, GRID_RUNTIME_PATH]);
const GRAMMERS_BASELINE_PATH = "src/lib/telegram-grammers-feature-baseline.json";
```

Keep the existing `EXPECTED_PRODUCER_DEPENDENCIES` and `EXPECTED_APP_TELEGRAM_DEPENDENCIES` arrays unchanged.

Delete these rule-specific constants completely:

```text
ALLOWED_LLM_MACROS
ALLOWED_LLM_MODULE_ATTRIBUTES
ALLOWED_LLM_RENAMED_IMPORTS
ALLOWED_LLM_TYPE_ALIASES
ANALYSIS_SURFACE_PATH
CANONICAL_LEAF_PATHS
EMPTY_STATE_MODULE
EXPECTED_LLM_CREDENTIAL_TYPES
EXPECTED_LLM_PUBLIC_METHODS
EXPECTED_LLM_ROOT
EXPECTED_LLM_RUST_FILES
FROZEN_STAGING_SHA256
HIGHLIGHT_STYLE_TARGETS
LLM_BUILD_PATH
LLM_GEMINI_PATH
LLM_LIB_PATH
LLM_OPENAI_COMPAT_PATH
LLM_PROVIDER_PATH
LLM_RUNNER_PATH
LLM_SCHEDULER_PATH
LLM_SRC_PREFIX
LLM_STREAMING_PATH
LLM_TYPES_PATH
PHASE_8A_PLAN_PATH
PHASE_8B_PLAN_PATH
PHASE_8_DESIGN_PATH
PHASE_8_ROADMAP_PATH
RUST_IDENTIFIER_AT
RUST_IDENTIFIER_CONTINUE
RUST_IDENTIFIER_GLOBAL
RUST_IDENTIFIER_SOURCE
SOURCE_ACTIVITY_MODULE
SOURCE_BROWSER_SHELL_MODULE
SOURCE_BROWSER_SHELL_PATH
SOURCE_GROUP_ACTIVITY_MODULE
SOURCE_GROUP_ACTIVITY_PATH
SOURCE_GROUP_SOURCES_PATH
STAGING_SHA_PATH
SYMBOL_MAP_PATH
TELEGRAM_PATH
TEST_IDENTITIES_PATH
TRANSITIONAL_SOURCE_COMPONENTS
```

Delete these evaluator/helper functions completely:

```text
compactRust
componentsFromModule
conjunctionTerms
evaluateAnalysisEvidenceHighlightTokenStyling
evaluateAnalysisSourceBrowserCanonicalComposition
evaluateAnalysisSourceBrowserExplicitSubjectContract
evaluateAnalysisSourceGroupActivityBoundary
evaluateAnalysisSourceGroupTabLeafBoundary
evaluateAnalysisSourceReaderSurfaceComposition
evaluateExtractumLlmPublicApiBoundary
evaluateTelegramPhase8BAuthorityIntegrity
exactActivityTab
exactPositiveActivityBranch
hasComponentFromModule
hasExactCloneStruct
identifier
importedDefaultSource
matchingBrace
namedRustBlock
namedRustImplBlocks
resolveOwnedAlias
retainedPhase8StatusViolations
routeApiImport
rustAssociatedItemHazards
rustIdentifierAt
rustIdentifierValues
rustImplMethods
rustInherentHazards
rustInherentMethods
rustMacroHazards
rustModuleItemHazards
rustStructFields
rustStructuralIdentifierHazards
rustTypeItems
rustUseAliases
sanitizeRust
selectorHas
stringLiteral
uniqueMatch
withoutCfgTestModules
```

Keep these existing functions unchanged:

```text
dependencyTuple
evaluateExtractumGridWrapperBoundary
evaluateRule
evaluateTelegramCrateDependencyOwnership
evaluateTelegramCrateManifestBoundary
exactDependencyInventory
exactDependencyKinds
hasNamedComponentFromModule
resolvedNode
sameJson
selectorContainsGlobalClass
sortedJson
workspacePackage
```

Replace the evaluator registry with:

```js
const evaluators = new Map([
  ["rule:extractum-grid-wrapper-boundary", evaluateExtractumGridWrapperBoundary],
  ["rule:telegram-crate-dependency-ownership", evaluateTelegramCrateDependencyOwnership],
  ["rule:telegram-crate-manifest-boundary", evaluateTelegramCrateManifestBoundary],
]);
```

Do not generalize or replace any retained evaluator.

- [ ] **Step 4: Reduce repository-rule fixtures to the same three boundaries**

In `scripts/testing/repository-rules.test.ts`:

1. Delete the LLM, analysis, and Phase 8B constants and source fixtures.
2. Keep `extractumGridBoundarySources` and its `rule:extractum-grid-wrapper-boundary` fixture.
3. Keep `grammersBaseline`, `cargoMetadata()`, `cargoIndex()`, `realAuthorityIndex()`, and the two retained Telegram structured fixtures.
4. Delete the `rule:telegram-phase-8b-authority-integrity` structured fixture.
5. Delete `analysisRuleIds` and the per-analysis fixture loop. Rename its
   enclosing `describe` to `current repository structured rules` and retain
   only the Task 1 repurposed current-repository case inside it.
6. Delete the LLM-rule `describe` and the analysis parse-failure test. Keep the
   unknown-rule test, mutation coverage, and reordered Grammers baseline test.

The retained fixture registry must have this shape:

```ts
const ruleFixtures: Record<string, RuleFixture> = {
  "rule:extractum-grid-wrapper-boundary": {
    positive: extractumGridBoundarySources,
    mutations: {
      "imports SVAR from a feature component": {
        ...extractumGridBoundarySources,
        "src/lib/components/research-projects/FeatureGrid.svelte": `
          <script lang="ts">import { Grid } from "@svar-ui/svelte-grid";</script>
          <Grid />
        `,
      },
      "drops the tree wrapper scoped SVAR style": {
        ...extractumGridBoundarySources,
        [TREE_DATA_GRID_PATH]: extractumGridBoundarySources[TREE_DATA_GRID_PATH].replace(
          '<style>.extractum-tree-data-grid :global(.wx-cell) { padding: 4px; }</style>',
          "",
        ),
      },
    },
  },
};
```

The registry coverage assertion remains:

```ts
expect([...Object.keys(ruleFixtures), ...Object.keys(telegramStructuredFixtures)].sort())
  .toEqual(registeredRuleIds);
```

- [ ] **Step 5: Delete contract-path and Phase 8B scaffolding**

Use `apply_patch` to delete every file named in this task. Remove apparatus consumers before deleting `telegram-contract-paths.ts`.

From `.gitattributes`, delete only:

```gitattributes
src/lib/telegram-8b-test-identities.json text eol=lf
src/lib/telegram-8b-symbol-map.json text eol=lf
```

Retain exactly:

```gitattributes
src/lib/telegram-grammers-feature-baseline.json text eol=lf
```

The staging JSON has no `.gitattributes` line, so do not add one before deleting it.

- [ ] **Step 6: Remove completed workflow vocabularies from the value registry**

In `docs/value-registry.md`:

1. Delete the entire `## Telegram crate-preparation agent lifecycle` section, from its heading through the `done: retained` row.
2. Do not add omitted `8b-preparation` values.
3. Rewrite the Vitest project introduction and table to exactly four current owners:

```markdown
## Vitest testing project names

These stable tooling-only values select the four Vitest projects. They are
owned by `vitest.config.ts`, persisted only in checked-in configuration, and
have no product database, API, UI, or fixture impact.

| Value | Type | Meaning | Owner | Persistence/API/UI/fixture impact | Current usage |
| --- | --- | --- | --- | --- | --- |
| `unit-node` | Vitest project name | Pure Node and TypeScript tests. | `vitest.config.ts` | Checked-in config only; none elsewhere. | `test:unit` |
| `component` | Vitest project name | Svelte Testing Library tests in jsdom. | `vitest.config.ts` | Checked-in config only; none elsewhere. | `test:component` |
| `architecture` | Vitest project name | Structured architecture rules. | `vitest.config.ts` | Checked-in config only; none elsewhere. | `test:architecture` |
| `os-integration` | Vitest project name | Tests that own real operating-system resources. | `vitest.config.ts` | Checked-in config only; none elsewhere. | `test:integration:os` |
```

4. Delete the complete `## Testing tooling decision evidence` section.
5. Delete the complete `## Source-contract migration ledger dispositions` section.
6. Leave the following `## Process-shell diagnostic artifact classifications` section and all product vocabularies unchanged.

- [ ] **Step 7: Run the retained rule gates**

Run:

```powershell
npm.cmd run test:unit -- scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
```

Expected: PASS; the registry and fixture sets both contain exactly three IDs, and all three accept the current repository.

Run:

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --check
```

Expected: exit 0 with no output; the retained JSON exactly matches current Cargo metadata.

- [ ] **Step 8: Prove the coupled scaffolding is absent and the retained authority remains**

Run:

```powershell
$closureRemovedPaths = @(
  'src/lib/analysis-contract-paths.ts',
  'src/lib/prompt-pack-contract-paths.ts',
  'src/lib/telegram-contract-paths.ts',
  'src/lib/telegram-contract-paths.behavior.test.ts',
  'src/lib/telegram-8b-staging-sha256.json',
  'src/lib/telegram-8b-symbol-map.json',
  'src/lib/telegram-8b-test-identities.json',
  'scripts/telegram-staging-sha256.mjs',
  'scripts/telegram-8b-symbol-map.mjs',
  'scripts/telegram-8b-test-identities.mjs',
  '.superpowers/sdd/reviewfix-rules-report.md'
)
$closureUnexpected = @($closureRemovedPaths | Where-Object { Test-Path -LiteralPath $_ })
if ($closureUnexpected.Count -ne 0) { $closureUnexpected; throw 'coupled scaffolding remains' }
if (-not (Test-Path -LiteralPath 'scripts/telegram-grammers-feature-baseline.mjs')) { throw 'Grammers generator was removed' }
if (-not (Test-Path -LiteralPath 'src/lib/telegram-grammers-feature-baseline.json')) { throw 'Grammers authority was removed' }

$closureRemovedRulePattern = 'rule:analysis-|rule:extractum-llm-public-api-boundary|rule:telegram-phase-8b-authority-integrity|evaluateAnalysis|evaluateExtractumLlmPublicApiBoundary|evaluateTelegramPhase8BAuthorityIntegrity'
$closureRemovedRuleHits = @(rg -n $closureRemovedRulePattern scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts 2>$null)
if ($closureRemovedRuleHits.Count -ne 0) { $closureRemovedRuleHits; throw 'retired RepositoryIndex code remains' }

$closureRegistryHits = @(
  Select-String -LiteralPath 'docs/value-registry.md' -Pattern '^## Telegram crate-preparation agent lifecycle$|^## Testing tooling decision evidence$|^## Source-contract migration ledger dispositions$|8b-checkpoint-|8c-extracted|legacy-contract|runner census|ADOPT_NX|REJECT_NX'
)
if ($closureRegistryHits.Count -ne 0) { $closureRegistryHits; throw 'retired testing workflow vocabulary remains' }

$closureAttributeHits = @(rg -n 'telegram-8b-|source-contract-redisposition-evidence' .gitattributes 2>$null)
if ($closureAttributeHits.Count -ne 0) { $closureAttributeHits; throw 'retired generated authority remains in .gitattributes' }
$closureGrammersAttributes = @(Select-String -LiteralPath '.gitattributes' -SimpleMatch 'src/lib/telegram-grammers-feature-baseline.json text eol=lf')
if ($closureGrammersAttributes.Count -ne 1) { throw 'Grammers .gitattributes authority is missing or duplicated' }
```

Expected: exit 0; no unexpected paths, retired rule code, workflow values, or
generated-authority attributes; exactly one retained Grammers attribute.

- [ ] **Step 9: Commit the three-rule current architecture**

Run:

```powershell
git status --short
git diff --check
git add -A -- scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts docs/value-registry.md .gitattributes src/lib/analysis-contract-paths.ts src/lib/prompt-pack-contract-paths.ts src/lib/telegram-contract-paths.ts src/lib/telegram-contract-paths.behavior.test.ts src/lib/telegram-8b-staging-sha256.json src/lib/telegram-8b-symbol-map.json src/lib/telegram-8b-test-identities.json scripts/telegram-staging-sha256.mjs scripts/telegram-8b-symbol-map.mjs scripts/telegram-8b-test-identities.mjs .superpowers/sdd/reviewfix-rules-report.md
git diff --cached --check
git commit -m "test: retain three current repository boundaries"
```

Expected: the commit contains only the rule reduction and named coupled cleanup.

---

### Task 3: Align the Current Command and Documentation Surface

**Files:**

- Modify: `package.json:41-49`
- Modify: `docs/project.md:13-59,94-107`
- Modify: `AGENTS.md:19-29`
- Delete: `scripts/verify-stability.mjs`
- Delete: `scripts/verify-stability.test.ts`

**Interfaces:**

- Consumes: the four Vitest projects, two Playwright commands, retained Rust/package gates, and transition-free `verify` from Tasks 1-2.
- Produces: no broken project-runs composite, no one-time stability command, one current testing-loop description, and matching agent workflow instructions.

- [ ] **Step 1: Establish RED for the retired package scripts**

Run this assertion before editing `package.json`:

```powershell
node --input-type=module -e "import fs from 'node:fs'; const value=JSON.parse(fs.readFileSync('package.json','utf8')); const retired=['test:project-runs','verify:project-runs','verify:stability']; const present=retired.filter((name)=>Object.hasOwn(value.scripts,name)); if(present.length) throw new Error('retired scripts remain: '+present.join(', '));"
```

Expected: exit 1 with `retired scripts remain: test:project-runs, verify:project-runs, verify:stability`.

- [ ] **Step 2: Remove the broken and one-time command surface**

Delete these exact keys from `package.json`:

```json
"test:project-runs": "node scripts/run-vitest.mjs run src/lib/project-runs-screen-contract.test.ts src/lib/api/prompt-packs.test.ts",
"verify:project-runs": "npm run test:project-runs && npm run check && npm run test:rust:prompt-pack-runs",
"verify:stability": "node scripts/verify-stability.mjs",
```

Retain this focused Rust command unchanged:

```json
"test:rust:prompt-pack-runs": "cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib prompt_pack_run",
```

Delete `scripts/verify-stability.mjs` and `scripts/verify-stability.test.ts`.

- [ ] **Step 3: Replace the verification and daily-loop opening in `docs/project.md`**

Replace `docs/project.md` from `## Verification` through the paragraph ending `avoid per-task target directories during sequential development.` with:

````markdown
## Verification

Run baseline full-project verification before committing or merging:

```bash
npm.cmd run verify
```

`verify` checks the existing Gemini Browser sidecar prerequisite, then runs the
four Vitest projects (`unit-node`, `component`, `architecture`, and
`os-integration`), the Gemini adapter and application Playwright owners,
Svelte/TypeScript checks, Rust formatting, Cargo workspace check/tests, and
`git diff HEAD --check`. It does not build the sidecar, select tests by change,
or create migration evidence.

The six JavaScript/browser owner commands are explicit:

```powershell
npm.cmd run test:unit
npm.cmd run test:component
npm.cmd run test:architecture
npm.cmd run test:integration:os
npm.cmd run test:e2e
npm.cmd run test:app:e2e
```

Before the first `verify` in a fresh checkout or worktree, and after changing
sidecar packaging, bootstrap its ignored Gemini Browser sidecar binary:

```powershell
npm ci
npm.cmd run bootstrap:testing
npm.cmd run verify
```

`bootstrap:testing` may download the `pkg` runtime cache. Release builds remain
separate: `tauri build` invokes `build:tauri-prereqs`.

<!-- daily-development-loop -->
For the daily loop after a small change, choose the narrowest applicable command:

```powershell
npm.cmd run test:changed
npm.cmd run test:changed:last
npm.cmd run test:related -- src/lib/api/prompt-packs.ts
npm.cmd run check
npm.cmd run test:rust:prompt-pack-runs
```

The working-tree command sees uncommitted changes; the last-checkpoint command
uses `HEAD~1`, which means the first parent after a merge. Use
`npm.cmd run test -- --changed=HEAD~2` when that earlier checkpoint is the
intended merge base.

Use `npm.cmd run check` after broad Svelte/TypeScript work. The changed/related
commands stay inside Vitest, so they never select Rust or Playwright. They can
select an `os-integration` Vitest owner when statically related, but do not
guarantee complete OS-process coverage, and the static module graph may return
an empty or incomplete selection for dynamic relationships. An empty or
unexpectedly small result requires an explicit project, package, browser, or
wider owner command. Accelerators never replace `npm.cmd run verify`.

Canonical full Rust checks and tests use `--workspace --all-targets`. Focused
root-package filters select `-p extractum` explicitly. Every workspace member
shares `src-tauri/target`; avoid per-task target directories during sequential
development. Focused Rust package commands are accelerators, not completion
evidence; a Rust slice closes only on the full `verify` workspace gates.

### Current test-loop boundaries

The current testing loop has no 15-second fail-closed cross-stack selector,
TestingManifest, NoTestAllowlist, runner census, source-contract ledger, Nx
integration, coverage ratchet, test-quarantine system, repeated performance
series, or stability-audit command. Dependency-cruiser is outside the closed program
and may return only through a separate decision that replaces a larger custom
mechanism instead of adding one. The retained Rust type and trait tests do not
provide fail-closed detection for an arbitrarily newly named future credential
accessor.
````

- [ ] **Step 4: Remove stale project-runs documentation**

Delete the complete block beginning with:

```markdown
For the YouTube Summary / Prompt Pack project-runs slice, use the narrower
verification scripts while iterating:
```

and ending with:

```markdown
The first Rust run after a clean target can be slow because Cargo warms the
test target; subsequent runs are expected to be much faster.
```

Do not delete the following prompt-pack architecture paragraphs.

- [ ] **Step 5: Mirror the Vitest-only limitation in `AGENTS.md`**

Insert this bullet immediately after the four daily-loop command bullets:

```markdown
- `test:changed`, `test:changed:last`, and `test:related` are Vitest-only
  accelerators, so they never cover Rust or Playwright. They may select an
  `os-integration` Vitest owner when statically related, but do not guarantee
  complete OS-process coverage; an empty or unexpectedly small selection
  requires an explicit owner command or wider run.
```

Leave the bootstrap, focused Rust, Cargo feature, and full-gate rules unchanged.

- [ ] **Step 6: Run the current command-surface assertions**

Run:

```powershell
node --input-type=module -e "import fs from 'node:fs'; const value=JSON.parse(fs.readFileSync('package.json','utf8')); const retired=['test:project-runs','verify:project-runs','verify:stability']; const present=retired.filter((name)=>Object.hasOwn(value.scripts,name)); if(present.length) throw new Error('retired scripts remain: '+present.join(', ')); const retained=['test:unit','test:component','test:architecture','test:integration:os','test:e2e','test:app:e2e','test:rust:prompt-pack-runs','verify']; const missing=retained.filter((name)=>!Object.hasOwn(value.scripts,name)); if(missing.length) throw new Error('retained scripts missing: '+missing.join(', '));"
```

Expected: exit 0 with no output.

Run:

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/test-conventions.test.ts
```

Expected: PASS with both files selected.

Run:

```powershell
$closureCommandHits = @(rg -n 'test:project-runs|verify:project-runs|verify:stability|scripts/verify-stability' package.json scripts docs/project.md AGENTS.md 2>$null)
if ($closureCommandHits.Count -ne 0) { $closureCommandHits; throw 'retired command references remain' }
```

Expected: exit 0 with no hits.

- [ ] **Step 7: Commit the current command and documentation surface**

Run:

```powershell
git status --short
git diff --check
git add -A -- package.json docs/project.md AGENTS.md scripts/verify-stability.mjs scripts/verify-stability.test.ts
git diff --cached --check
git commit -m "docs: consolidate the current testing loop"
```

Expected: the commit changes only the current command surface, its deleted audit, and matching current-state documentation.

---

### Task 4: Remove the Program Corpus and Close on a Fresh Complete Gate

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

**Interfaces:**

- Consumes: three green focused-task commits, current `docs/project.md`/`AGENTS.md`, exact baseline metrics, existing bootstrap prerequisite, and the retained full repository gate.
- Produces: no Testing Redesign corpus in the working tree; independently net-negative permanent artifacts; one green complete gate; final Git commit body as the only closure record.

- [ ] **Step 1: Confirm a clean pre-closure checkpoint**

Run:

```powershell
git status --short
git log -5 --oneline
```

Expected: empty status; the three task commits appear above the committed plan,
which appears above `ff040cd2`.

- [ ] **Step 2: Delete the 18 baseline documents and both temporary closure documents**

Use `apply_patch` to delete exactly the 20 files listed in this task. Do not move them into `docs/superpowers/archive/` and do not create a replacement index or verification note.

- [ ] **Step 3: Prove the stale program authority has left the working tree**

Run:

```powershell
$closureProgramHits = @(rg -n 'testing-redesign|testing-infrastructure-redesign|ADOPT_NX|REJECT_NX' docs AGENTS.md README.md 2>$null)
if ($closureProgramHits.Count -ne 0) { $closureProgramHits; throw 'stale testing-redesign authority remains' }
```

Expected: exit 0 with no hits. Product documentation must not link to or restate the removed program.

- [ ] **Step 4: Prove every independent net-negative acceptance metric**

Run:

```powershell
$closureBaseline = 'c076c002a8debcce4a5bc71226077c03884367e7'
if (Test-Path -LiteralPath 'testing') { throw 'testing directory still exists' }

$closureScriptFiles = @(Get-ChildItem -LiteralPath 'scripts/testing' -File)
$closureScriptLines = 0
foreach ($closureScriptFile in $closureScriptFiles) {
  $closureScriptLines += @(Get-Content -LiteralPath $closureScriptFile.FullName).Count
}
$closureScriptGitPaths = @(git ls-tree -r --name-only HEAD -- scripts/testing)
$closureScriptBlobBytes = 0L
foreach ($closureScriptGitPath in $closureScriptGitPaths) {
  $closureScriptBlobBytes += [int64](git cat-file -s "HEAD:$closureScriptGitPath")
  if ($LASTEXITCODE -ne 0) { throw "cannot size HEAD:$closureScriptGitPath" }
}
if ($closureScriptGitPaths.Count -ne $closureScriptFiles.Count) { throw 'scripts/testing has untracked or missing files' }
if ($closureScriptFiles.Count -ge 21) { throw "scripts/testing file count is not net-negative: $($closureScriptFiles.Count)" }
if ($closureScriptLines -ge 13349) { throw "scripts/testing line count is not net-negative: $closureScriptLines" }

node --input-type=module -e "const {registeredRuleIds}=await import('./scripts/testing/repository-rules.mjs'); const expected=['rule:extractum-grid-wrapper-boundary','rule:telegram-crate-dependency-ownership','rule:telegram-crate-manifest-boundary']; if(JSON.stringify(registeredRuleIds)!==JSON.stringify(expected)) throw new Error('unexpected rule registry: '+JSON.stringify(registeredRuleIds)); console.log('RULES='+registeredRuleIds.length);"
if ($LASTEXITCODE -ne 0) { throw 'RepositoryIndex registry check failed' }

node --input-type=module -e "import fs from 'node:fs'; import {execFileSync} from 'node:child_process'; const stable=(value)=>JSON.stringify(Object.fromEntries(Object.entries(value??{}).sort(([left],[right])=>left.localeCompare(right)))); const baseline=JSON.parse(execFileSync('git',['show','c076c002a8debcce4a5bc71226077c03884367e7:package.json'],{encoding:'utf8'})); const current=JSON.parse(fs.readFileSync('package.json','utf8')); for(const section of ['dependencies','devDependencies']) if(stable(current[section])!==stable(baseline[section])) throw new Error(section+' changed'); const expectedScripts={...baseline.scripts}; for(const name of ['test:project-runs','verify:project-runs','verify:stability']) delete expectedScripts[name]; if(stable(current.scripts)!==stable(expectedScripts)) throw new Error('package script surface changed beyond the three approved deletions');"
if ($LASTEXITCODE -ne 0) { throw 'dependency or package-script surface changed' }

$closureAddedFiles = @(git diff --diff-filter=A --name-only $closureBaseline --)
if ($closureAddedFiles.Count -ne 0) { $closureAddedFiles; throw 'closure introduced permanent files' }

Write-Output "SCRIPTS_TESTING_FILES=$($closureScriptFiles.Count)"
Write-Output "SCRIPTS_TESTING_BLOB_BYTES=$closureScriptBlobBytes"
Write-Output "SCRIPTS_TESTING_LINES=$closureScriptLines"
```

Expected: `RULES=3`, `SCRIPTS_TESTING_FILES` below 21, a factual
`SCRIPTS_TESTING_BLOB_BYTES`, `SCRIPTS_TESTING_LINES` below 13,349, no
dependency or retained package-command drift, no added file relative to the
baseline, and no `testing/` directory.

- [ ] **Step 5: Check the sidecar prerequisite before spending the full-gate time**

Run:

```powershell
npm.cmd run check:gemini-browser-sidecar-binary
```

Expected: PASS. If it reports a missing binary in a fresh checkout/worktree, run exactly:

```powershell
npm.cmd run bootstrap:testing
npm.cmd run check:gemini-browser-sidecar-binary
```

Expected after bootstrap: PASS. Do not run bootstrap merely to refresh an already valid binary.

- [ ] **Step 6: Stage only the 20 document deletions**

Run:

```powershell
$closureDocs = @(
  'docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md',
  'docs/superpowers/specs/2026-08-05-testing-redesign-slice-3b-design.md',
  'docs/superpowers/specs/2026-08-08-testing-redesign-slice-3c-design.md',
  'docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md',
  'docs/superpowers/plans/2026-08-02-testing-redesign-slice-1-measurement.md',
  'docs/superpowers/plans/2026-08-02-testing-redesign-slice-2a-migration-preflight.md',
  'docs/superpowers/plans/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md',
  'docs/superpowers/plans/2026-08-03-testing-redesign-slice-3a-source-contracts.md',
  'docs/superpowers/plans/2026-08-05-testing-redesign-slice-3b-component-contracts.md',
  'docs/superpowers/plans/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md',
  'docs/superpowers/plans/2026-08-08-testing-redesign-slice-3c-source-contract-completion.md',
  'docs/superpowers/verification/2026-08-02-testing-redesign-slice-1-measurement.md',
  'docs/superpowers/verification/2026-08-02-testing-redesign-slice-2a-migration-preflight.md',
  'docs/superpowers/verification/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md',
  'docs/superpowers/verification/2026-08-03-testing-redesign-slice-3a-source-contracts.md',
  'docs/superpowers/verification/2026-08-05-testing-redesign-slice-3b-component-contracts.md',
  'docs/superpowers/verification/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md',
  'docs/superpowers/verification/2026-08-08-testing-redesign-slice-3c.md',
  'docs/superpowers/specs/2026-08-10-testing-redesign-program-closure-design.md',
  'docs/superpowers/plans/2026-08-10-testing-redesign-program-closure.md'
)
git status --short
git diff --check
git add -A -- $closureDocs
git diff --cached --check
git diff --cached --name-status
```

Expected: exactly 20 staged deletions and no unstaged change.

- [ ] **Step 7: Run the intended final gate and commit in the same PowerShell process**

Keep timing and commit-message construction in one PowerShell process so no
temporary evidence file or unknown value is needed:

```powershell
$closureVerifyTimer = [System.Diagnostics.Stopwatch]::StartNew()
npm.cmd run verify
$closureVerifyExit = $LASTEXITCODE
$closureVerifyTimer.Stop()
$closureVerifySeconds = [Math]::Round($closureVerifyTimer.Elapsed.TotalSeconds, 1)
Write-Output "VERIFY_EXIT=$closureVerifyExit"
Write-Output "VERIFY_SECONDS=$closureVerifySeconds"
if ($closureVerifyExit -ne 0) { throw 'final verify failed; closure remains open' }

$closureScriptFiles = @(Get-ChildItem -LiteralPath 'scripts/testing' -File)
$closureScriptLines = 0
foreach ($closureScriptFile in $closureScriptFiles) {
  $closureScriptLines += @(Get-Content -LiteralPath $closureScriptFile.FullName).Count
}
$closureScriptGitPaths = @(git ls-tree -r --name-only HEAD -- scripts/testing)
$closureScriptBlobBytes = 0L
foreach ($closureScriptGitPath in $closureScriptGitPaths) {
  $closureScriptBlobBytes += [int64](git cat-file -s "HEAD:$closureScriptGitPath")
  if ($LASTEXITCODE -ne 0) { throw "cannot size HEAD:$closureScriptGitPath" }
}
if ($closureScriptGitPaths.Count -ne $closureScriptFiles.Count) { throw 'scripts/testing has untracked or missing files' }
$closureReduction = "Reduction: testing/ 29 files, 2881023 Git blob bytes, and 30043 lines -> absent; scripts/testing/ 21 files, 708939 Git blob bytes, and 13349 lines -> $($closureScriptFiles.Count) files, $closureScriptBlobBytes Git blob bytes, and $closureScriptLines lines; RepositoryIndex 11 -> 3 rules; program corpus 18 files, 519798 Git blob bytes, and 7901 lines -> absent."
$closureVerification = "Verification: npm.cmd run verify PASS in $closureVerifySeconds seconds."
git commit -m "chore: close testing redesign program" -m "Baseline: c076c002a8debcce4a5bc71226077c03884367e7." -m $closureReduction -m $closureVerification
if ($LASTEXITCODE -ne 0) { throw 'closure commit failed' }
```

Expected: every retained gate passes, `VERIFY_EXIT=0`, one factual
`VERIFY_SECONDS` value is printed, and the commit succeeds with the exact
baseline OID, actual post-closure script counts, aggregate reductions, and
successful gate duration. Duration is recorded, not compared to 818.8 seconds
or any target.

If `verify` fails, the `throw` prevents the commit. Do not create evidence. If
the failure is solely sandbox access, use the approval/escalation rule in the
global constraints and leave the staged set unchanged.

For a repository failure, use the smallest existing non-empty owner named by
the failed step. Before changing code, recreate the exact `$closureDocs` array
from Step 6 and restore those temporary deletions so the correction cannot be
mixed into the closure commit:

```powershell
git restore --staged -- $closureDocs
git restore --worktree -- $closureDocs
git status --short
```

Expected: all 20 documents are restored; only the focused correction becomes a
working-tree change. Correct or revert the responsible deletion, rerun its
focused owner, stage only the corrected Task 1-3 files, inspect the staged diff,
and create a separate `fix: correct testing closure gate` commit. Require a
clean status after that commit. Then repeat Task 4 from Step 2: delete the same
20 documents again, rerun the independent checks, stage exactly those 20
deletions, and make the next fresh complete run the final attempt.

- [ ] **Step 8: Verify final repository and commit state**

Run:

```powershell
git status --short
git show --stat --format=fuller HEAD
git log -1 --format=%B
```

Expected: clean status; the final commit deletes exactly the 20 documents; its body contains `c076c002a8debcce4a5bc71226077c03884367e7`, the actual reduction, and the successful `verify` duration. The Testing Redesign program is closed with no new verification record.
