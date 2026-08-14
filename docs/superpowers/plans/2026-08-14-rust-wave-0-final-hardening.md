# Rust Wave 0 Final Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close every Important and Minor finding from the Wave 0 whole-range review so the branch is locally ready for a separately authorized push and remote evidence run.

**Architecture:** A repository-owned PowerShell wrapper authenticates and runs the pinned cargo-deny binary; generated canonical policy artifacts own exact Cargo/npm direct edges; a read-only duplicate checker validates current state and optionally compares an explicit Git base; and workflows separate advisory scanning from issue writes while pinning every remote Action. Two bounded Rust-only corrections and factual documentation/evidence finish the slice without changing product behavior.

**Tech Stack:** Rust 1.95.0 / Edition 2021, Cargo, PowerShell 7/Windows PowerShell, Node.js ESM, TypeScript, Vitest 4, cargo-deny 0.20.2, Git, GitHub Actions, serde/serde_json, Tauri 2.

## Global Constraints

- Implement against approved spec commit `e55242320809954dba63616475bae39a32c5c497`; where it conflicts with the original Wave 0 design, the addendum controls.
- Do not upgrade Rust, Edition 2021, Cargo/npm dependencies, or cargo-deny `0.20.2`; do not change `src-tauri/Cargo.lock` or any Cargo manifest.
- Do not resolve, suppress, or add exceptions for the currently known RustSec advisories.
- Checks are read-only. The only writers are `node scripts/rust-dependency-policy.mjs --write scripts/testing/rust-dependency-policy.json` and `node scripts/rust-duplicate-baseline.mjs --write scripts/testing/rust-duplicate-baseline.json`.
- Cargo commands run from the repository root with `--manifest-path src-tauri/Cargo.toml`; test/check commands in this plan use `--locked`.
- The Windows duplicate target is exactly `x86_64-pc-windows-msvc`; the tool cache is exactly `src-tauri/target/.extractum-tools/cargo-deny/0.20.2-x86_64-pc-windows-msvc`.
- The extracted cargo-deny digest is exactly `f7292fab58c706638c999e64c4ba82e5128ae628130ba55e3266a768ee431fbf`; the version output is exactly `cargo-deny 0.20.2`.
- Do not add ambient/global cargo-deny lookup, `cargo install`, automatic policy regeneration, or fallback to `cargo deny`.
- Do not change product behavior, IPC, persisted data, migrations, public JSON, frontend contracts, or product UI.
- Modify only files listed in the approved Authorized Implementation Scope. A newly required file outside it stops implementation for a design amendment.
- Use `npm.cmd` for repository scripts on Windows. Do not claim remote CI, advisory issue behavior, bundle, or release evidence locally.
- Preserve the six ordered implementation commits verbatim and do not squash them. Every numbered task receives scope/spec/code review before the next task; review corrections are adjacent commits.

---

## File Map and Cross-Task Interfaces

### Local tool orchestration

- Create `scripts/run-cargo-deny.ps1`: operator-facing `Setup | Deterministic | Advisories` dispatcher, cache authenticator, installer caller, and absolute-path policy runner.
- Create `scripts/run-cargo-deny.test.ps1`: isolated PowerShell mutation/integration harness for cache, binary digest, version, transaction, environment, and absolute invocation contracts.
- Modify `scripts/setup-cargo-deny.ps1`: authenticate archive, then extracted executable, then version; publish transactionally and restore environment/cache on failure.
- Modify `.github/tools/cargo-deny.json`: retain schema/version/target/url/archive `sha256`, add exact `binarySha256`.
- Modify `.github/actions/setup-cargo-deny/action.yml`: call `run-cargo-deny.ps1 -Mode Setup -AddToGitHubPath`.
- Modify `package.json`: wrapper-backed supply-chain/advisory aliases, duplicate checker alias, and two residual locked Rust aliases.
- Modify `scripts/verify.test.ts`: only the `test:rust` and `test:rust:prompt-pack-runs` locked positive/negative contracts.

### Manifest and direct-dependency policy

- Create `scripts/rust-dependency-policy.mjs`: schema-2 generator/writer and reusable canonical inventory helpers.
- Create `scripts/rust-dependency-policy.test.ts`: generator, writer-preservation, exact-edge, pair, Cargo-only, pin, and prerelease mutations.
- Modify `scripts/testing/rust-dependency-policy.json`: generated schema-2 candidate plus manually reviewed pair/Cargo-only requirements.
- Modify `scripts/testing/repository-index.mjs`: cache raw root/member manifests and expose section-aware relevant TOML facts.
- Modify `scripts/testing/repository-index.test.ts`: parser/cache/path-boundary tests.
- Modify `scripts/testing/repository-rules.mjs`: consume exact manifest and dependency-policy facts; remove major/minor range extraction.
- Modify `scripts/testing/repository-rules.test.ts`: syntax ownership, workspace inventory, direct-edge, Tauri, exact-pin, prerelease, and duplicate current-state fixtures.
- Modify `scripts/testing/test-conventions.test.ts`: register new generator/test and stable rule inventory.

### Duplicate policy

- Modify `scripts/rust-duplicate-baseline.mjs`: generator plus current/base validation, state machine, environment/CLI precedence, structured classifications, and explicit writer.
- Modify `scripts/rust-duplicate-baseline.test.ts`: pure function and fixture-repository CLI tests with an injected clock.
- Modify `scripts/testing/rust-duplicate-baseline.json`: regenerate exact active graph only when required.
- Modify `scripts/testing/rust-supply-chain-exceptions.json`: exact schema-1 top-level structure and sorted `duplicateGrowthExceptions` lifecycle records; current Wave 0 remains empty unless the explicit base comparison proves otherwise.
- Modify `.github/workflows/rust-fast.yml`: full-history checkout and required explicit base SHA.
- Modify `.github/workflows/rust-full.yml` and `.github/workflows/rust-release.yml`: explicit duplicate mode `off`, no base SHA.

### CI, bounded corrections, and docs

- Modify `scripts/testing/github-rust-workflows.test.ts`: exact immutable Action map, duplicate-mode contracts, least-privilege advisory scan/writer contracts, and negative mutations.
- Modify `deny.toml`: remove unused LLVM exception allowance; deny both unused-license conditions.
- Modify `src-tauri/crates/extractum-gemini-browser/src/types.rs`: literal golden sidecar response in both directions.
- Modify `src-tauri/src/llm/mod.rs`: private owned-result enum and clone-free configured provider access.
- Modify `README.md`, `AGENTS.md`, `docs/project.md`: fresh-checkout wrapper-backed instructions.
- Modify `docs/value-registry.md`: five tooling-only value families with owner/persistence/API/UI/fixture fields.
- Modify `docs/superpowers/plans/2026-08-12-rust-infrastructure-wave-0.md` and `docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md`: bounded factual corrections only.
- Create `docs/superpowers/verification/2026-08-14-rust-wave-0-final-hardening.md`: exact local evidence and explicitly pending remote evidence.

### Shared JavaScript interfaces

The implementation uses these exact names and shapes so later tasks do not invent parallel evaluators:

```js
// scripts/rust-dependency-policy.mjs
export function cargoRequirementIdentity(entry) // string
export function npmRequirementIdentity(entry) // string
export function generateRustDependencyPolicy({ metadata, packageJson, committedPolicy }) // schema-2 object
export function validateReviewedPolicySeed(committedPolicy) // void; schema/root/reviewed Tauri records only
export function validateRustDependencyPolicy({ generated, committed }) // string[]

// scripts/testing/repository-index.mjs
index.getCargoManifest(relativePath) // frozen { path, tables }
// tables is a frozen object keyed by section name; each relevant key maps to
// { value: string|boolean, occurrences: number }.

// scripts/rust-duplicate-baseline.mjs
export function generateRustDuplicateBaseline(treeText) // schema-1 baseline
export function validateDuplicateBaseline(value) // canonical baseline or throws
export function validateSupplyChainExceptions(value, baseline, { now, enforceExpiry }) // canonical exceptions or throws
export function validateCurrentDuplicateState({ treeText, baseline, exceptions, now }) // { baseline, exceptions }
export function classifyDuplicateTransition({ baseBaseline, baseExceptions, currentBaseline, currentExceptions, now })
// => { classification: "CLEAN"|"REVIEW"|"REDUCTION"|"VIOLATION",
//      addedDuplicateNames: string[], removedDuplicateNames: string[],
//      reductions: Array<{ package: string, previousCount: number, currentCount: number }>,
//      violations: string[] }
export function parseDuplicateArguments(argv) // { check: boolean, writePath: string|null, baseSha: string|null }
export function resolveDuplicateInvocation({ argv, env })
// => { writePath: string|null, baseSha: string|null, githubMode: "required"|"off"|null }
export function checkRustDuplicatePolicy({ treeText, currentBaseline, currentExceptions, baseBaseline, baseExceptions, now }) // structured result
export function runRustDuplicateCli({ argv, env, now, stdout, stderr, root }) // integer exit code
```

The repository rule calls the shared current-state validators; it does not reimplement duplicate arithmetic or exception schemas. The CLI owns Git/history and human output.

## Rust Verification Loops

Affected Rust packages are `extractum-gemini-browser` and root application package `extractum` only.

- Gemini Browser RED/GREEN: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact`.
- Gemini Browser focused check: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked`.
- Gemini Browser package checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked`.
- Root application characterization before/after: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact`.
- Root application focused check: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked`.
- Root application package checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked`.
- The secret move intentionally has no fabricated RED: the existing complete, key-only, base-only, empty-key, and empty-base cases are recorded green before and after. Clone absence is proved by bounded diff review, compiler, focused check, owner test, Clippy, and the final full gate.
- End-of-slice workspace gates, in order: `npm.cmd run test:unit`, `npm.cmd run check`, `npm.cmd run bootstrap:testing`, `npm.cmd run check:rust:fast`, then `npm.cmd run verify`.
- A filtered run reporting zero tests is invalid evidence. List tests first if either exact name changes.

---

### Task 1: Bootstrap Pinned cargo-deny Locally

**Files:**
- Create: `scripts/run-cargo-deny.ps1`
- Create: `scripts/run-cargo-deny.test.ps1`
- Modify: `scripts/setup-cargo-deny.ps1`
- Modify: `.github/tools/cargo-deny.json`
- Modify: `.github/actions/setup-cargo-deny/action.yml`
- Modify: `package.json`
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/project.md`
- Test: `scripts/run-cargo-deny.test.ps1`

**Interfaces:**
- Consumes: existing cargo-deny archive URL/hash and transactional installer; root `deny.toml`; `src-tauri/Cargo.toml`.
- Produces: `run-cargo-deny.ps1 -Mode Setup|Deterministic|Advisories [-AddToGitHubPath]`; authenticated stable cache; wrapper-backed npm aliases; shared CI setup action.

- [ ] **Step 1: Add the manifest and script-contract RED assertions**

In `scripts/run-cargo-deny.test.ps1`, create isolated directories under `[System.IO.Path]::GetTempPath()`, always remove only that resolved test directory in `finally`, and add helpers with these signatures:

```powershell
function Assert-Equal([object]$Actual, [object]$Expected, [string]$Message) { if ($Actual -ne $Expected) { throw "$Message; expected '$Expected', got '$Actual'" } }
function Assert-Throws([scriptblock]$Action, [string]$Pattern) { try { & $Action; throw "Expected failure: $Pattern" } catch { if ($_.Exception.Message -notmatch $Pattern) { throw } } }
function Invoke-Wrapper([string]$Mode, [string[]]$Extra = @()) {
    $stdoutPath = Join-Path $testRoot ([guid]::NewGuid().ToString('N') + '.stdout.txt')
    $stderrPath = Join-Path $testRoot ([guid]::NewGuid().ToString('N') + '.stderr.txt')
    try {
        $arguments = @(
            '-NoProfile',
            '-ExecutionPolicy', 'Bypass',
            '-File', $wrapperPath,
            '-Mode', $Mode
        ) + $Extra
        & powershell.exe @arguments 1> $stdoutPath 2> $stderrPath
        [pscustomobject]@{
            ExitCode = $LASTEXITCODE
            Stdout = if (Test-Path -LiteralPath $stdoutPath) { Get-Content -LiteralPath $stdoutPath -Raw } else { '' }
            Stderr = if (Test-Path -LiteralPath $stderrPath) { Get-Content -LiteralPath $stderrPath -Raw } else { '' }
        }
    } finally {
        Remove-Item -LiteralPath $stdoutPath, $stderrPath -Force -ErrorAction SilentlyContinue
    }
}
```

Every multi-case assertion consumes the returned `ExitCode`, `Stdout`, and `Stderr`; a failed child must never exit the harness itself. Assert `binarySha256`, the exact three modes, rejection of `-AddToGitHubPath` outside Setup, stable cache path, absence of `cargo install`/ambient `cargo deny`, and absolute policy arguments. With `$denyPath = Join-Path $repositoryRoot 'deny.toml'` and `$cargoManifestPath = Join-Path $repositoryRoot 'src-tauri/Cargo.toml'`, assert the exact arrays `@('--config', $denyPath, '--manifest-path', $cargoManifestPath, '--locked', 'check', 'bans', 'licenses', 'sources')` and `@('--config', $denyPath, '--manifest-path', $cargoManifestPath, '--locked', 'check', 'advisories')`; mutate away `--locked` and require the contract to fail. Use source mutations copied into the isolated directory to prove omitting cache-hit digest/version checks, staged digest/version checks, or rollback logic makes the contract fail.

- [ ] **Step 2: Run the PowerShell harness and observe RED**

Run:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-cargo-deny.test.ps1
```

Expected: FAIL because `scripts/run-cargo-deny.ps1` and `binarySha256` do not exist.

- [ ] **Step 3: Add the executable digest and make setup authenticate before execution**

Add to `.github/tools/cargo-deny.json`:

```json
"binarySha256": "f7292fab58c706638c999e64c4ba82e5128ae628130ba55e3266a768ee431fbf"
```

In `setup-cargo-deny.ps1`, validate the manifest has exactly schema 1 plus nonempty version/target/url and 64-lowercase-hex archive/binary hashes. Immediately after locating the extracted file, do this before `& $extractedBinary.FullName --version`:

```powershell
$actualBinarySha256 = (Get-FileHash -LiteralPath $extractedBinary.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualBinarySha256 -ne $manifest.binarySha256.ToLowerInvariant()) {
    throw "cargo-deny binary checksum mismatch: expected $($manifest.binarySha256), got $actualBinarySha256"
}
```

Keep backup/publish/rollback transactional. A wrong staged version may execute only the authenticated `--version` probe and must not be copied to the stable path. Restore `PATH` and the exact original `GITHUB_PATH` length/content on every failure.

- [ ] **Step 4: Implement the sole operator wrapper**

Create `run-cargo-deny.ps1` with an enum-validated parameter and cache selection:

```powershell
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Setup', 'Deterministic', 'Advisories')]
    [string]$Mode,
    [switch]$AddToGitHubPath
)
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$manifest = Get-Content -LiteralPath (Join-Path $repositoryRoot '.github/tools/cargo-deny.json') -Raw | ConvertFrom-Json
$toolDirectory = Join-Path $repositoryRoot "src-tauri/target/.extractum-tools/cargo-deny/$($manifest.version)-$($manifest.target)"
$binary = Join-Path $toolDirectory 'cargo-deny.exe'
```

Factor local functions `Test-AuthenticatedBinary([string]$Path, [object]$Manifest) -> bool` and `Invoke-AuthenticatedVersion([string]$Path, [string]$Version)`. Hash first, then execute exact `--version`. On missing/mismatch call setup for the same directory, then reauthenticate. `Setup` runs no policy and optionally publishes the stable directory when `-AddToGitHubPath` is present; other modes reject the switch.

The wrapper, not the installer call, owns GitHub PATH publication. Call `setup-cargo-deny.ps1 -InstallDirectory $toolDirectory` without `-AddToGitHubPath`/`-PrependToProcessPath`, reauthenticate, then for Setup use:

```powershell
function Add-GitHubPathOnce([string]$Directory) {
    if ([string]::IsNullOrWhiteSpace($env:GITHUB_PATH)) { throw 'GITHUB_PATH is required when -AddToGitHubPath is specified.' }
    $normalized = [System.IO.Path]::GetFullPath($Directory)
    $existing = if (Test-Path -LiteralPath $env:GITHUB_PATH -PathType Leaf) {
        @(Get-Content -LiteralPath $env:GITHUB_PATH | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object { [System.IO.Path]::GetFullPath($_) })
    } else { @() }
    if (-not ($existing | Where-Object { [string]::Equals($_, $normalized, [System.StringComparison]::OrdinalIgnoreCase) })) {
        Add-Content -LiteralPath $env:GITHUB_PATH -Value $normalized
    }
}
```

Call it only after authentication succeeds. This yields exactly one normalized stable-directory line for cold and warm Setup, and no process `PATH` mutation. `Setup` without the switch only authenticates/installs.

Policy modes execute by absolute path:

```powershell
$arguments = @('--config', (Join-Path $repositoryRoot 'deny.toml'), '--manifest-path', (Join-Path $repositoryRoot 'src-tauri/Cargo.toml'), '--locked', 'check')
if ($Mode -eq 'Deterministic') { $arguments += @('bans', 'licenses', 'sources') }
else { $arguments += 'advisories' }
& $binary @arguments
exit $LASTEXITCODE
```

`--locked` is a cargo-deny root option and therefore precedes `check`; the check names remain positional values after `check`. Never prepend or inspect user Cargo bins and never run a policy when authentication or setup fails.

- [ ] **Step 5: Complete adversarial wrapper tests**

The harness must cover: missing cache; wrong-digest fake that prints `cargo-deny 0.20.2` but is never run; correctly hashed fixture with wrong version that is run only with `--version`; archive digest failure; extracted digest failure; first-install failure leaves no accepted exe; failed reinstall restores the prior authenticated exe but the invocation fails; `PATH`/`GITHUB_PATH` restoration; no ambient executable; two successive absolute-path Deterministic/Advisories invocations from the stable cache; root-level `--locked`; policy exit-code propagation. For source-order mutations, find the `Get-FileHash` token and require its character offset to be lower than both the `& $extractedBinary.FullName --version` invocation offset and the final `Copy-Item -LiteralPath $extractedBinary.FullName -Destination $installedBinary -Force` offset.

For GitHub PATH publication, use this exact cold/warm assertion after arranging the cold fixture with no cached executable:

```powershell
$env:GITHUB_PATH = Join-Path $testRoot 'github-path.txt'
New-Item -ItemType File -Path $env:GITHUB_PATH -Force | Out-Null
$processPathBefore = $env:PATH
$cold = Invoke-Wrapper 'Setup' @('-AddToGitHubPath')
Assert-Equal $cold.ExitCode 0 'cold Setup failed'
$warm = Invoke-Wrapper 'Setup' @('-AddToGitHubPath')
Assert-Equal $warm.ExitCode 0 'warm Setup failed'
$stableNormalized = [System.IO.Path]::GetFullPath($stableToolDirectory)
$matchingLines = @(Get-Content -LiteralPath $env:GITHUB_PATH |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    ForEach-Object { [System.IO.Path]::GetFullPath($_) } |
    Where-Object { [string]::Equals($_, $stableNormalized, [System.StringComparison]::OrdinalIgnoreCase) })
Assert-Equal $matchingLines.Count 1 'stable tool directory must be published exactly once'
Assert-Equal $env:PATH $processPathBefore 'wrapper must restore process PATH'
```

A mutation that makes the installer and wrapper both append, or that appends on every warm call, must fail the count assertion.

- [ ] **Step 6: Wire aliases and composite action**

Set these exact package scripts while preserving the four-part `check:rust:fast` composition:

```json
"check:rust:supply-chain": "powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-cargo-deny.ps1 -Mode Deterministic",
"check:rust:advisories": "powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-cargo-deny.ps1 -Mode Advisories"
```

Change the composite run line to:

```yaml
& './scripts/run-cargo-deny.ps1' -Mode Setup -AddToGitHubPath
```

Do not give the composite action its own cache directory or installer logic.

- [ ] **Step 7: Update fresh-checkout operator docs and their contracts**

In `README.md`, `docs/project.md`, and `AGENTS.md`, state that `npm.cmd run check:rust:fast` authenticates/self-bootstraps the repository-pinned binary, no global install is required, and retain this sequence:

```powershell
npm.cmd ci
npm.cmd run bootstrap:testing
npm.cmd run check:rust:fast
npm.cmd run verify
```

Add harness assertions/targeted repository tests that these three docs mention the wrapper-backed alias and contain no instruction matching `cargo install cargo-deny` or a global cargo-deny prerequisite.

- [ ] **Step 8: Run Task 1 GREEN checks**

Run:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-cargo-deny.test.ps1
npm.cmd run check:rust:supply-chain
npm.cmd run check:rust:fast
```

Expected: PASS; deterministic output contains successful `bans`, `licenses`, and `sources`; both policy invocations use the authenticated stable absolute path. No baseline, exception, lockfile, or global tool path changes.

- [ ] **Step 9: Commit and review Task 1**

```powershell
git add scripts/run-cargo-deny.ps1 scripts/run-cargo-deny.test.ps1 scripts/setup-cargo-deny.ps1 .github/tools/cargo-deny.json .github/actions/setup-cargo-deny/action.yml package.json README.md AGENTS.md docs/project.md
git commit -m "build: bootstrap pinned cargo-deny locally"
```

Expected: commit contains only Task 1 files. Run immediate scope/spec/code review; any fix is a separate adjacent commit and is re-reviewed before Task 2.

---

### Task 2: Harden Rust Manifest and Direct-Dependency Policy

**Files:**
- Create: `scripts/rust-dependency-policy.mjs`
- Create: `scripts/rust-dependency-policy.test.ts`
- Modify: `scripts/testing/rust-dependency-policy.json`
- Modify: `scripts/testing/repository-index.mjs`
- Modify: `scripts/testing/repository-index.test.ts`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `scripts/testing/test-conventions.test.ts`

**Interfaces:**
- Consumes: locked Cargo metadata, all seven root/member manifests, root `package.json`, and the committed schema-1 policy only as a one-time migration input.
- Produces: schema-2 canonical direct inventories, section-aware TOML facts, exact Tauri pair/Cargo-only enforcement, and explicit generator/writer.

- [ ] **Step 1: Write section-aware manifest parser RED tests**

Add `getCargoManifest` fixtures proving comments and unrelated tables do not match relevant keys, while duplicate `[workspace.package]`/`[package]` tables, duplicate dotted keys, literal member values, missing root ownership, and repository escape paths fail. The positive fixture must parse:

```toml
[workspace.package]
rust-version = "1.95" # canonical MSRV
edition = "2021"

[package]
rust-version.workspace = true
edition.workspace = true
```

Expected fact shape:

```js
{
  path: "src-tauri/Cargo.toml",
  tables: {
    "workspace.package": {
      "rust-version": { value: "1.95", occurrences: 1 },
      edition: { value: "2021", occurrences: 1 },
    },
    package: {
      "rust-version.workspace": { value: true, occurrences: 1 },
      "edition.workspace": { value: true, occurrences: 1 },
    },
  },
}
```

- [ ] **Step 2: Run manifest parser RED**

Run:

```powershell
npm.cmd run test:unit -- scripts/testing/repository-index.test.ts scripts/testing/repository-rules.test.ts
```

Expected: FAIL because `getCargoManifest` and syntax-owned inheritance checks do not exist.

- [ ] **Step 3: Implement and cache relevant TOML facts**

Implement these functions in `repository-index.mjs`:

```js
function scanTomlLine(line) {
  let quote = null;
  let escaped = false;
  let equals = -1;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (escaped) { escaped = false; continue; }
    if (quote && character === "\\") { escaped = true; continue; }
    if (character === '"' || character === "'") {
      quote = quote === character ? null : quote ?? character;
      continue;
    }
    if (!quote && character === "#") return { text: line.slice(0, index), equals };
    if (!quote && character === "=" && equals < 0) equals = index;
  }
  if (quote) throw new Error("unterminated TOML string");
  return { text: line, equals };
}

function parseRelevantCargoManifest(relativePath, source) {
  const relevantTables = new Set(["workspace.package", "package"]);
  const acceptedKeys = new Set(["rust-version", "edition", "rust-version.workspace", "edition.workspace"]);
  const tables = {};
  let currentTable = null;
  for (const [offset, raw] of String(source).split(/\r?\n/).entries()) {
    const { text, equals } = scanTomlLine(raw);
    const trimmed = text.trim();
    if (!trimmed) continue;
    if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
      const name = trimmed.slice(1, -1).trim();
      currentTable = relevantTables.has(name) ? name : null;
      if (currentTable && Object.hasOwn(tables, currentTable)) throw new Error(`${relativePath}:${offset + 1}: duplicate [${currentTable}]`);
      if (currentTable) tables[currentTable] = {};
      continue;
    }
    if (!currentTable || equals < 0) continue;
    const key = text.slice(0, equals).trim();
    if (!acceptedKeys.has(key)) continue;
    if (Object.hasOwn(tables[currentTable], key)) throw new Error(`${relativePath}:${offset + 1}: duplicate ${currentTable}.${key}`);
    const token = text.slice(equals + 1).trim();
    const value = token === "true" ? true : token === "false" ? false
      : /^"(?:[^"\\]|\\.)*"$/.test(token) ? JSON.parse(token)
      : /^'[^']*'$/.test(token) ? token.slice(1, -1)
      : (() => { throw new Error(`${relativePath}:${offset + 1}: invalid ${currentTable}.${key}`); })();
    tables[currentTable][key] = { value, occurrences: 1 };
  }
  return freeze({ path: relativePath, tables });
}
```

Expose it through `getCargoManifest(inputPath)` using `cachedSource(cargoManifestCache, inputPath, parseRelevantCargoManifest)`. The RED tests call the public index method twice and require object identity equality to prove caching. Do not add a package dependency and do not use a whole-file regular expression.

- [ ] **Step 4: Write schema-2 generator and identity RED tests**

In `rust-dependency-policy.test.ts`, fixture locked metadata with normal/build/dev, renamed and target-specific dependencies plus root npm dependencies/devDependencies/optionalDependencies. Start with an explicit schema-2 reviewed fixture. Also pass the current raw schema-1 object and assert `validateRustDependencyPolicy` reports `schemaVersion must be exactly 2`; it must never reinterpret legacy `tauriFamily.cargoMajor`, `npmMajor`, or name-only maps as reviewed schema-2 records. Assert identities and exact objects:

```js
cargoRequirementIdentity(entry) === "extractum/tauri-build/null/build/null"
npmRequirementIdentity(entry) === "extractum/@tauri-apps/cli/devDependencies"
```

Use this harness for every policy mutation:

```ts
const EXACT_TOOLCHAIN = {
  channel: "1.95.0",
  rustVersion: "1.95",
  edition: "2021",
  target: "x86_64-pc-windows-msvc",
  workspacePackages: [
    "extractum", "extractum-analysis", "extractum-core", "extractum-gemini-browser",
    "extractum-llm", "extractum-prompt-packs", "extractum-telegram",
  ],
};
const schema2Fixture = () => generateRustDependencyPolicy(fixtureInputs());
const rawSchema1Fixture = () => ({
  schemaVersion: 1,
  toolchain: schema2Fixture().toolchain,
  exactPins: { "grammers-client": "=0.10.0" },
  approvedPrereleases: { apalis: "1.0.0-rc.8" },
  tauriFamily: { cargoMajor: 2, npmMajor: 2, mcpBridgeMinor: 11, pairs: [["tauri", "@tauri-apps/api"]] },
});
function expectPolicyViolation(mutate: (value: any) => void, message: string) {
  const generated = generateRustDependencyPolicy(fixtureInputs());
  const committed = structuredClone(generated);
  mutate(committed);
  expect(validateRustDependencyPolicy({ generated, committed })).toEqual(expect.arrayContaining([expect.stringContaining(message)]));
}
it("rejects the raw schema-1 policy", () => {
  expect(validateRustDependencyPolicy({ generated: schema2Fixture(), committed: rawSchema1Fixture() }))
    .toEqual(expect.arrayContaining([expect.stringContaining("schemaVersion must be exactly 2")]));
});
it.each([
  ["Cargo widened range", (policy: any) => { policy.directRequirements.find((entry: any) => entry.dependency === "tauri").requirement = "^2 || ^3"; }, "directRequirements"],
  ["MCP widened range", (policy: any) => { policy.directRequirements.find((entry: any) => entry.dependency === "tauri-plugin-mcp-bridge").requirement = ">=0.11"; }, "directRequirements"],
  ["missing direct edge", (policy: any) => { policy.directRequirements.pop(); }, "missing or mismatched"],
  ["duplicate direct edge", (policy: any) => { policy.directRequirements.push(structuredClone(policy.directRequirements[0])); }, "committed duplicate"],
  ["npm widened range", (policy: any) => { policy.npmRequirements.find((entry: any) => entry.name === "@tauri-apps/api").requirement = "^2 || ^3"; }, "npmRequirements"],
  ["npm moved kind", (policy: any) => { policy.npmRequirements.find((entry: any) => entry.name === "@tauri-apps/cli").kind = "dependencies"; }, "npmRequirements"],
  ["unowned exact pin", (policy: any) => { policy.exactPins.pop(); }, "exactPins"],
  ["unowned prerelease", (policy: any) => { policy.approvedPrereleases.pop(); }, "approvedPrereleases"],
  ["duplicate pair id", (policy: any) => { policy.tauriFamily.pairs[1].id = policy.tauriFamily.pairs[0].id; }, "duplicate Tauri pair id"],
  ["unsorted pairs", (policy: any) => { [policy.tauriFamily.pairs[0], policy.tauriFamily.pairs[1]] = [policy.tauriFamily.pairs[1], policy.tauriFamily.pairs[0]]; }, "tauri pairs must be sorted by id"],
  ["duplicate paired Cargo identity", (policy: any) => { policy.tauriFamily.pairs[1].cargo = structuredClone(policy.tauriFamily.pairs[0].cargo); }, "duplicate paired Cargo identity"],
  ["duplicate paired npm identity", (policy: any) => { policy.tauriFamily.pairs[1].npm = structuredClone(policy.tauriFamily.pairs[0].npm); }, "duplicate paired npm identity"],
  ["orphan pair", (policy: any) => { policy.tauriFamily.pairs[0].cargo.dependency = "not-tauri"; }, "orphan Cargo identity"],
  ["widened pair approval", (policy: any) => { policy.tauriFamily.pairs[0].cargoRequirement = "^2 || ^3"; }, "Cargo requirement drifted"],
  ["missing Cargo-only record", (policy: any) => { policy.tauriFamily.cargoOnlyRequirements = []; }, "Cargo-only"],
  ["Cargo-only extra field", (policy: any) => { policy.tauriFamily.cargoOnlyRequirements[0].approved = true; }, "expected fields"],
  ["toolchain channel", (policy: any) => { policy.toolchain.channel = "1.96.0"; }, "canonical toolchain drifted"],
  ["toolchain target", (policy: any) => { policy.toolchain.target = "x86_64-unknown-linux-gnu"; }, "canonical toolchain drifted"],
  ["workspace package removed", (policy: any) => { policy.toolchain.workspacePackages.pop(); }, "canonical toolchain drifted"],
  ["renamed pair id", (policy: any) => { policy.tauriFamily.pairs[0].id = "api"; }, "reviewed Tauri pair set drifted"],
  ["swapped pair ids", (policy: any) => {
    [policy.tauriFamily.pairs[0].id, policy.tauriFamily.pairs[1].id] =
      [policy.tauriFamily.pairs[1].id, policy.tauriFamily.pairs[0].id];
  }, "reviewed Tauri pair set drifted"],
  ["swapped pair identities", (policy: any) => {
    [policy.tauriFamily.pairs[0].cargo, policy.tauriFamily.pairs[1].cargo] =
      [policy.tauriFamily.pairs[1].cargo, policy.tauriFamily.pairs[0].cargo];
  }, "reviewed Tauri pair set drifted"],
  ["null nested Cargo identity", (policy: any) => { policy.tauriFamily.pairs[0].cargo = null; }, "tauri-api.cargo: must be a plain object"],
  ["wrong nested npm kind", (policy: any) => { policy.tauriFamily.pairs[0].npm.kind = 7; }, "tauri-api.npm.kind must be one of"],
  ["malformed Cargo-only record", (policy: any) => { policy.tauriFamily.cargoOnlyRequirements[0] = null; }, "Cargo-only record: must be a plain object"],
  ["duplicate Cargo-only identity", (policy: any) => {
    policy.tauriFamily.cargoOnlyRequirements.push(structuredClone(policy.tauriFamily.cargoOnlyRequirements[0]));
    policy.tauriFamily.cargoOnlyRequirements[1].id = "another-id";
  }, "duplicate Cargo-only identity"],
  ["duplicate Cargo-only id", (policy: any) => {
    policy.tauriFamily.cargoOnlyRequirements.push(structuredClone(policy.tauriFamily.cargoOnlyRequirements[0]));
    policy.tauriFamily.cargoOnlyRequirements[1].cargo.dependency = "other-cargo-only";
  }, "duplicate Cargo-only id"],
  ["renamed Cargo-only id", (policy: any) => { policy.tauriFamily.cargoOnlyRequirements[0].id = "mcp"; }, "reviewed Cargo-only set drifted"],
  ["unsorted Cargo-only ids", (policy: any) => {
    policy.tauriFamily.cargoOnlyRequirements.push({
      id: "aaa", cargo: { package: "extractum", dependency: "other-cargo-only", rename: null, kind: "normal", target: null }, cargoRequirement: "^1",
    });
  }, "Cargo-only records must be sorted by id"],
] as const)("rejects %s", (_name, mutate, message) => expectPolicyViolation(mutate, message));
```

Separate generator-input tests add a normal/build/dev edge, renamed/target edge, exact stable pin, and prerelease comparator to metadata, and dependency/devDependency/optionalDependency Tauri edge to `packageJson`; each asserts the exact generated object and canonical position rather than mutating committed output.

- [ ] **Step 5: Run generator RED**

Run:

```powershell
npm.cmd run test:unit -- scripts/rust-dependency-policy.test.ts
```

Expected: FAIL because the generator module is absent.

- [ ] **Step 6: Implement canonical schema-2 generation**

Implement `generateRustDependencyPolicy` from Cargo metadata package dependencies and root package maps with these loops:

```js
const cargoKind = (kind) => kind === null ? "normal" : kind;
const compareFields = (left, right, fields) => {
  for (const field of fields) {
    const compared = String(left[field] ?? "").localeCompare(String(right[field] ?? ""), "en");
    if (compared) return compared;
  }
  return 0;
};

const directRequirements = metadata.workspace_members
  .map((id) => metadata.packages.find((pkg) => pkg.id === id))
  .sort((left, right) => left.name.localeCompare(right.name, "en"))
  .flatMap((pkg) => pkg.dependencies.map((dependency) => ({
    package: pkg.name,
    dependency: dependency.name,
    rename: dependency.rename ?? null,
    kind: cargoKind(dependency.kind),
    target: dependency.target ?? null,
    requirement: dependency.req,
  })))
  .sort((left, right) => compareFields(left, right, ["package", "dependency", "rename", "kind", "target"]));

const npmRequirements = ["dependencies", "devDependencies", "optionalDependencies"]
  .flatMap((kind) => Object.entries(packageJson[kind] ?? {})
    .filter(([name]) => name.startsWith("@tauri-apps/"))
    .map(([name, requirement]) => ({ owner: "extractum", name, kind, requirement })))
  .sort((left, right) => compareFields(left, right, ["owner", "name", "kind"]));
```

Derive `exactPins` by matching the entire requirement with `/^=[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$/`. Derive `approvedPrereleases` by extracting every comparator version with `/[0-9]+\.[0-9]+\.[0-9]+-[0-9A-Za-z.-]+/g`; require exactly one distinct prerelease version per direct edge or report an unsupported ambiguous approval. Emit exact root keys:

```js
{
  schemaVersion: 2,
  toolchain: committedPolicy.toolchain,
  directRequirements,
  npmRequirements,
  exactPins,
  approvedPrereleases,
  tauriFamily: committedPolicy.tauriFamily,
}
```

Each exact/prerelease entry spreads the five identity fields and adds `requirement` or `version`. `validateReviewedPolicySeed` requires schema 2, exact root keys, canonical toolchain, and structurally valid reviewed pair/Cargo-only records, but deliberately does not require the four generated inventories to match live metadata. Only `--write` calls this seed validator before replacing the generated arrays; ordinary/default checking calls full `validateRustDependencyPolicy`. `--write` must copy `toolchain`, `tauriFamily.pairs`, and `tauriFamily.cargoOnlyRequirements` byte-for-semantic-value from that schema-2 seed. Print unresolved/candidate governed differences to stderr and leave the resulting full validation red if reviewed identities/requirements no longer resolve; never infer reviewed values.

- [ ] **Step 7: Implement exact bidirectional validation**

Implement exact-key and canonical-array checks rather than ad hoc field access:

```js
function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    && [Object.prototype, null].includes(Object.getPrototypeOf(value));
}
function requireExactKeys(value, expected, label, violations) {
  if (!isPlainObject(value)) { violations.push(`${label}: must be a plain object`); return false; }
  const actual = Object.keys(value ?? {}).sort();
  const wanted = [...expected].sort();
  if (JSON.stringify(actual) !== JSON.stringify(wanted)) violations.push(`${label}: expected fields ${wanted.join(",")}`);
  return JSON.stringify(actual) === JSON.stringify(wanted);
}

const EXACT_TOOLCHAIN = {
  channel: "1.95.0",
  rustVersion: "1.95",
  edition: "2021",
  target: "x86_64-pc-windows-msvc",
  workspacePackages: [
    "extractum", "extractum-analysis", "extractum-core", "extractum-gemini-browser",
    "extractum-llm", "extractum-prompt-packs", "extractum-telegram",
  ],
};
const REVIEWED_TAURI_PAIRS = [
  { id: "tauri-api", cargo: { package: "extractum", dependency: "tauri", rename: null, kind: "normal", target: null }, cargoRequirement: "^2", npm: { owner: "extractum", name: "@tauri-apps/api", kind: "dependencies" }, npmRequirement: "^2" },
  { id: "tauri-build-cli", cargo: { package: "extractum", dependency: "tauri-build", rename: null, kind: "build", target: null }, cargoRequirement: "^2", npm: { owner: "extractum", name: "@tauri-apps/cli", kind: "devDependencies" }, npmRequirement: "^2" },
  { id: "tauri-dialog", cargo: { package: "extractum", dependency: "tauri-plugin-dialog", rename: null, kind: "normal", target: null }, cargoRequirement: "^2", npm: { owner: "extractum", name: "@tauri-apps/plugin-dialog", kind: "dependencies" }, npmRequirement: "^2" },
  { id: "tauri-opener", cargo: { package: "extractum", dependency: "tauri-plugin-opener", rename: null, kind: "normal", target: null }, cargoRequirement: "^2", npm: { owner: "extractum", name: "@tauri-apps/plugin-opener", kind: "dependencies" }, npmRequirement: "^2" },
  { id: "tauri-sql", cargo: { package: "extractum", dependency: "tauri-plugin-sql", rename: null, kind: "normal", target: null }, cargoRequirement: "^2", npm: { owner: "extractum", name: "@tauri-apps/plugin-sql", kind: "dependencies" }, npmRequirement: "^2.4.0" },
];
const REVIEWED_CARGO_ONLY = [
  { id: "tauri-plugin-mcp-bridge", cargo: { package: "extractum", dependency: "tauri-plugin-mcp-bridge", rename: null, kind: "normal", target: null }, cargoRequirement: "^0.11" },
];

function requireNonemptyString(value, label, violations) {
  if (typeof value !== "string" || !value.trim()) { violations.push(`${label} must be a nonempty string`); return false; }
  return true;
}
function validateCargoIdentity(value, label, violations) {
  if (!requireExactKeys(value, ["package", "dependency", "rename", "kind", "target"], label, violations)) return false;
  let valid = requireNonemptyString(value.package, `${label}.package`, violations)
    && requireNonemptyString(value.dependency, `${label}.dependency`, violations);
  if (!(value.rename === null || (typeof value.rename === "string" && value.rename.trim()))) { violations.push(`${label}.rename must be null or a nonempty string`); valid = false; }
  if (!["normal", "build", "dev"].includes(value.kind)) { violations.push(`${label}.kind must be one of normal,build,dev`); valid = false; }
  if (!(value.target === null || (typeof value.target === "string" && value.target.trim()))) { violations.push(`${label}.target must be null or a nonempty string`); valid = false; }
  return valid;
}
function validateNpmIdentity(value, label, violations) {
  if (!requireExactKeys(value, ["owner", "name", "kind"], label, violations)) return false;
  let valid = requireNonemptyString(value.owner, `${label}.owner`, violations)
    && requireNonemptyString(value.name, `${label}.name`, violations);
  if (!["dependencies", "devDependencies", "optionalDependencies"].includes(value.kind)) { violations.push(`${label}.kind must be one of dependencies,devDependencies,optionalDependencies`); valid = false; }
  return valid;
}
function validateCanonicalToolchain(value, violations) {
  if (!requireExactKeys(value, ["channel", "rustVersion", "edition", "target", "workspacePackages"], "toolchain", violations)) return;
  for (const field of ["channel", "rustVersion", "edition", "target"]) requireNonemptyString(value[field], `toolchain.${field}`, violations);
  if (!Array.isArray(value.workspacePackages) || value.workspacePackages.some((entry) => typeof entry !== "string" || !entry.trim())) violations.push("toolchain.workspacePackages must be an array of nonempty strings");
  if (JSON.stringify(value) !== JSON.stringify(EXACT_TOOLCHAIN)) violations.push("canonical toolchain drifted");
}
function compareInventory(label, generated, committed, identity, violations) {
  const generatedMap = new Map();
  const committedMap = new Map();
  for (const entry of generated) {
    const key = identity(entry);
    if (generatedMap.has(key)) violations.push(`${label}: generated duplicate ${key}`);
    generatedMap.set(key, entry);
  }
  for (const entry of committed) {
    const key = identity(entry);
    if (committedMap.has(key)) violations.push(`${label}: committed duplicate ${key}`);
    committedMap.set(key, entry);
  }
  for (const [key, entry] of generatedMap) if (JSON.stringify(committedMap.get(key)) !== JSON.stringify(entry)) violations.push(`${label}: missing or mismatched ${key}`);
  for (const key of committedMap.keys()) if (!generatedMap.has(key)) violations.push(`${label}: stale ${key}`);
}

function compareSortedIdentitySets(actual, expected, label, violations) {
  if (new Set(actual).size !== actual.length) violations.push(`${label}: duplicate identity reference`);
  const left = [...actual].sort();
  const right = [...expected].sort();
  if (JSON.stringify(left) !== JSON.stringify(right)) violations.push(`${label}: governed identity set drifted`);
}

function discoverPairedCargoIdentities(entries) {
  return entries.filter((entry) => entry.package === "extractum"
      && (entry.dependency === "tauri" || entry.dependency === "tauri-build" || entry.dependency.startsWith("tauri-plugin-"))
      && entry.dependency !== "tauri-plugin-mcp-bridge")
    .map(cargoRequirementIdentity).sort();
}
function discoverCargoOnlyIdentities(entries) {
  return entries.filter((entry) => entry.package === "extractum" && entry.dependency === "tauri-plugin-mcp-bridge")
    .map(cargoRequirementIdentity).sort();
}

function validateReviewedTauriRecords(tauriFamily, violations) {
  if (!isPlainObject(tauriFamily) || !Array.isArray(tauriFamily.pairs) || !Array.isArray(tauriFamily.cargoOnlyRequirements)) return;
  const ids = new Set();
  const cargoIdentities = new Set();
  const npmIdentities = new Set();
  let previousId = null;
  for (const [index, pair] of tauriFamily.pairs.entries()) {
    const position = `tauriFamily.pairs[${index}]`;
    if (!requireExactKeys(pair, ["id", "cargo", "cargoRequirement", "npm", "npmRequirement"], "tauri pair", violations)) continue;
    const idValid = requireNonemptyString(pair.id, `${position}.id`, violations);
    const label = idValid ? pair.id : position;
    const cargoValid = validateCargoIdentity(pair.cargo, `${label}.cargo`, violations);
    const npmValid = validateNpmIdentity(pair.npm, `${label}.npm`, violations);
    requireNonemptyString(pair.cargoRequirement, `${label}.cargoRequirement`, violations);
    requireNonemptyString(pair.npmRequirement, `${label}.npmRequirement`, violations);
    if (idValid) {
      if (previousId !== null && previousId.localeCompare(pair.id, "en") >= 0) violations.push("tauri pairs must be sorted by id");
      if (ids.has(pair.id)) violations.push(`duplicate Tauri pair id ${pair.id}`);
      ids.add(pair.id); previousId = pair.id;
    }
    if (cargoValid) {
      const cargoKey = cargoRequirementIdentity(pair.cargo);
      if (cargoIdentities.has(cargoKey)) violations.push(`duplicate paired Cargo identity ${cargoKey}`);
      cargoIdentities.add(cargoKey);
    }
    if (npmValid) {
      const npmKey = npmRequirementIdentity(pair.npm);
      if (npmIdentities.has(npmKey)) violations.push(`duplicate paired npm identity ${npmKey}`);
      npmIdentities.add(npmKey);
    }
  }
  const cargoOnlyIds = new Set();
  const cargoOnlyIdentities = new Set();
  let previousCargoOnlyId = null;
  for (const [index, record] of tauriFamily.cargoOnlyRequirements.entries()) {
    if (!requireExactKeys(record, ["id", "cargo", "cargoRequirement"], "Cargo-only record", violations)) continue;
    const idValid = requireNonemptyString(record.id, `cargoOnlyRequirements[${index}].id`, violations);
    const label = idValid ? record.id : `cargoOnlyRequirements[${index}]`;
    const cargoValid = validateCargoIdentity(record.cargo, `${label}.cargo`, violations);
    requireNonemptyString(record.cargoRequirement, `${label}.cargoRequirement`, violations);
    if (idValid) {
      if (previousCargoOnlyId !== null && previousCargoOnlyId.localeCompare(record.id, "en") >= 0) violations.push("Cargo-only records must be sorted by id");
      if (cargoOnlyIds.has(record.id)) violations.push(`duplicate Cargo-only id ${record.id}`);
      cargoOnlyIds.add(record.id); previousCargoOnlyId = record.id;
    }
    if (cargoValid) {
      const cargoKey = cargoRequirementIdentity(record.cargo);
      if (cargoOnlyIdentities.has(cargoKey)) violations.push(`duplicate Cargo-only identity ${cargoKey}`);
      cargoOnlyIdentities.add(cargoKey);
    }
  }
  if (JSON.stringify(tauriFamily.pairs) !== JSON.stringify(REVIEWED_TAURI_PAIRS)) violations.push("reviewed Tauri pair set drifted");
  if (JSON.stringify(tauriFamily.cargoOnlyRequirements) !== JSON.stringify(REVIEWED_CARGO_ONLY)) violations.push("reviewed Cargo-only set drifted");
}

function validateCanonicalInventory(entries, { label, fields, identity, allowedKinds }, violations) {
  if (!Array.isArray(entries)) { violations.push(`${label}: must be an array`); return false; }
  let previous = null;
  let safe = true;
  for (const entry of entries) {
    if (!requireExactKeys(entry, fields, `${label} entry`, violations)) { safe = false; continue; }
    const identitySafe = fields.includes("package")
      ? validateCargoIdentity(Object.fromEntries(["package", "dependency", "rename", "kind", "target"].map((field) => [field, entry[field]])), `${label} identity`, violations)
      : validateNpmIdentity(Object.fromEntries(["owner", "name", "kind"].map((field) => [field, entry[field]])), `${label} identity`, violations);
    const valueField = fields.includes("version") ? "version" : "requirement";
    const valueSafe = requireNonemptyString(entry[valueField], `${label}.${valueField}`, violations);
    if (!identitySafe || !valueSafe) { safe = false; continue; }
    const key = identity(entry);
    if (previous !== null && previous.localeCompare(key, "en") >= 0) violations.push(`${label}: entries must be unique and sorted`);
    if (allowedKinds && !allowedKinds.includes(entry.kind)) violations.push(`${label}: invalid kind ${entry.kind}`);
    previous = key;
  }
  return safe;
}

export function validateReviewedPolicySeed(committedPolicy) {
  const violations = [];
  requireExactKeys(committedPolicy, ["schemaVersion", "toolchain", "directRequirements", "npmRequirements", "exactPins", "approvedPrereleases", "tauriFamily"], "policy", violations);
  if (committedPolicy?.schemaVersion !== 2) violations.push("schemaVersion must be exactly 2");
  validateCanonicalToolchain(committedPolicy?.toolchain, violations);
  for (const field of ["directRequirements", "npmRequirements", "exactPins", "approvedPrereleases"]) {
    if (!Array.isArray(committedPolicy?.[field])) violations.push(`${field} must be an array`);
  }
  requireExactKeys(committedPolicy?.tauriFamily, ["pairs", "cargoOnlyRequirements"], "tauriFamily", violations);
  if (!Array.isArray(committedPolicy?.tauriFamily?.pairs) || !Array.isArray(committedPolicy?.tauriFamily?.cargoOnlyRequirements)) violations.push("tauriFamily arrays are required");
  validateReviewedTauriRecords(committedPolicy?.tauriFamily, violations);
  if (violations.length) throw new Error(violations.join("\n"));
}

export function validateRustDependencyPolicy({ generated, committed }) {
  const violations = [];
  try { validateReviewedPolicySeed(committed); } catch (error) { violations.push(error.message); return violations; }
  try { validateReviewedPolicySeed(generated); } catch (error) { violations.push(`generated: ${error.message}`); return violations; }
  let inventoryShapesAreSafe = true;
  for (const policy of [generated, committed]) {
    inventoryShapesAreSafe = validateCanonicalInventory(policy.directRequirements, { label: "directRequirements", fields: ["package", "dependency", "rename", "kind", "target", "requirement"], identity: cargoRequirementIdentity, allowedKinds: ["normal", "build", "dev"] }, violations) && inventoryShapesAreSafe;
    inventoryShapesAreSafe = validateCanonicalInventory(policy.npmRequirements, { label: "npmRequirements", fields: ["owner", "name", "kind", "requirement"], identity: npmRequirementIdentity, allowedKinds: ["dependencies", "devDependencies", "optionalDependencies"] }, violations) && inventoryShapesAreSafe;
    inventoryShapesAreSafe = validateCanonicalInventory(policy.exactPins, { label: "exactPins", fields: ["package", "dependency", "rename", "kind", "target", "requirement"], identity: cargoRequirementIdentity, allowedKinds: ["normal", "build", "dev"] }, violations) && inventoryShapesAreSafe;
    inventoryShapesAreSafe = validateCanonicalInventory(policy.approvedPrereleases, { label: "approvedPrereleases", fields: ["package", "dependency", "rename", "kind", "target", "version"], identity: cargoRequirementIdentity, allowedKinds: ["normal", "build", "dev"] }, violations) && inventoryShapesAreSafe;
  }
  if (!inventoryShapesAreSafe) return violations;
  compareInventory("directRequirements", generated.directRequirements, committed.directRequirements, cargoRequirementIdentity, violations);
  compareInventory("npmRequirements", generated.npmRequirements, committed.npmRequirements, npmRequirementIdentity, violations);
  compareInventory("exactPins", generated.exactPins, committed.exactPins, cargoRequirementIdentity, violations);
  compareInventory("approvedPrereleases", generated.approvedPrereleases, committed.approvedPrereleases, cargoRequirementIdentity, violations);
  const cargoByIdentity = new Map(generated.directRequirements.map((entry) => [cargoRequirementIdentity(entry), entry]));
  const npmByIdentity = new Map(generated.npmRequirements.map((entry) => [npmRequirementIdentity(entry), entry]));
  const referencedCargo = [];
  const referencedNpm = [];
  for (const pair of committed.tauriFamily.pairs) {
    const cargoKey = cargoRequirementIdentity(pair.cargo);
    const npmKey = npmRequirementIdentity(pair.npm);
    const cargo = cargoByIdentity.get(cargoKey);
    const npm = npmByIdentity.get(npmKey);
    if (!cargo) violations.push(`${pair.id}: orphan Cargo identity ${cargoKey}`);
    else if (cargo.requirement !== pair.cargoRequirement) violations.push(`${pair.id}: Cargo requirement drifted`);
    if (!npm) violations.push(`${pair.id}: orphan npm identity ${npmKey}`);
    else if (npm.requirement !== pair.npmRequirement) violations.push(`${pair.id}: npm requirement drifted`);
    referencedCargo.push(cargoKey);
    referencedNpm.push(npmKey);
  }
  for (const record of committed.tauriFamily.cargoOnlyRequirements) {
    const cargoKey = cargoRequirementIdentity(record.cargo);
    const cargo = cargoByIdentity.get(cargoKey);
    if (!cargo) violations.push(`${record.id}: orphan Cargo-only identity ${cargoKey}`);
    else if (cargo.requirement !== record.cargoRequirement) violations.push(`${record.id}: Cargo-only requirement drifted`);
  }
  compareSortedIdentitySets(referencedCargo, discoverPairedCargoIdentities(generated.directRequirements), "paired Cargo", violations);
  compareSortedIdentitySets(referencedNpm, generated.npmRequirements.map(npmRequirementIdentity), "paired npm", violations);
  compareSortedIdentitySets(
    committed.tauriFamily.cargoOnlyRequirements.map((record) => cargoRequirementIdentity(record.cargo)),
    discoverCargoOnlyIdentities(generated.directRequirements), "Cargo-only", violations,
  );
  return violations;
}
```

`validateReviewedTauriRecords` uses exact-field guards for pair `{id,cargo,cargoRequirement,npm,npmRequirement}`, Cargo identity, npm identity, and Cargo-only `{id,cargo,cargoRequirement}`; it rejects nonobjects, nonstrings, duplicate IDs, duplicate Cargo/npm identities, and noncanonical ordering. `discoverPairedCargoIdentities` returns root `extractum` edges named `tauri`, `tauri-build`, or prefixed `tauri-plugin-` except MCP; `discoverCargoOnlyIdentities` returns exactly the MCP edge. `compareSortedIdentitySets` sorts both arrays, rejects duplicates on either side, and adds one mismatch violation. These helpers are module-private and directly unit-tested with duplicate/orphan/extra fixtures.

Use the five approved pair IDs/requirements from the spec (`tauri-api`, `tauri-build-cli`, `tauri-dialog`, `tauri-opener`, `tauri-sql`) and the sole Cargo-only record `tauri-plugin-mcp-bridge` with `^0.11`. Tests must cover deleted/duplicate/orphan/unpaired/multiply referenced pairs, widened Cargo/npm requirements, and empty/missing/duplicate/orphan/extra-field/extra-record/widened Cargo-only records.

- [ ] **Step 8: Replace effective-only repository rules with syntax plus exact policy**

Replace the rule body with this integration skeleton:

```js
const REPOSITORY_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
function repositoryRelativeManifestPath(manifestPath) {
  const absolute = path.resolve(manifestPath);
  const relative = path.relative(REPOSITORY_ROOT, absolute);
  if (!relative || relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) throw new Error(`manifest escapes root: ${manifestPath}`);
  return relative.split(path.sep).join("/");
}
function validateCanonicalToolchain(source, toolchain) {
  const expected = `[toolchain]\nchannel = "${toolchain.channel}"\ncomponents = ["rustfmt", "clippy"]\ntargets = ["${toolchain.target}"]\nprofile = "minimal"\n`;
  return normalizeText(source) === expected ? [] : ["rust-toolchain.toml: canonical content drifted"];
}

function evaluateRustDependencyPolicy(index) {
  const policy = index.getJson(RUST_DEPENDENCY_POLICY_PATH);
  const metadata = index.getCargoMetadata();
  const packageJson = index.getJson("package.json");
  const violations = [];
  const workspacePackages = metadata.packages.filter((pkg) => metadata.workspace_members.includes(pkg.id));
  const expectedNames = [...policy.toolchain.workspacePackages].sort();
  const actualNames = workspacePackages.map((pkg) => pkg.name).sort();
  if (JSON.stringify(actualNames) !== JSON.stringify(expectedNames)) violations.push("workspace package inventory drifted");
  for (const pkg of workspacePackages) {
    const relativeManifest = repositoryRelativeManifestPath(pkg.manifest_path);
    const manifest = index.getCargoManifest(relativeManifest);
    const packageTable = manifest.tables.package;
    if (packageTable?.["rust-version.workspace"]?.value !== true || packageTable?.["rust-version"] !== undefined) violations.push(`${pkg.name}: rust-version must inherit workspace`);
    if (packageTable?.["edition.workspace"]?.value !== true || packageTable?.edition !== undefined) violations.push(`${pkg.name}: edition must inherit workspace`);
    if (pkg.rust_version !== "1.95" || pkg.edition !== "2021" || JSON.stringify(pkg.publish) !== "[]") violations.push(`${pkg.name}: effective package policy drifted`);
  }
  const root = index.getCargoManifest("src-tauri/Cargo.toml");
  if (root.tables["workspace.package"]?.["rust-version"]?.value !== "1.95") violations.push("workspace must own rust-version 1.95");
  if (root.tables["workspace.package"]?.edition?.value !== "2021") violations.push("workspace must own Edition 2021");
  const generated = generateRustDependencyPolicy({ metadata, packageJson, committedPolicy: policy });
  violations.push(...validateRustDependencyPolicy({ generated, committed: policy }));
  violations.push(...validateCanonicalToolchain(index.getText("rust-toolchain.toml"), policy.toolchain));
  return violations;
}
```

`repositoryRelativeManifestPath` resolves the absolute metadata path, requires it to remain below repository root, and returns a forward-slash relative path. Unit tests call the registered rule with a literal member MSRV, literal Edition, removed root value, duplicate relevant key, missing/extra workspace member, outside-root manifest, and every generator mutation; each fixture asserts its exact violation. Delete `requirementMajor`, `requirementMajorMinor`, name-only exact pin maps, and all first-number fallbacks.

Use this registered-rule test body:

```ts
it("enforces manifest syntax and the generated direct-requirement policy together", () => {
  expect(evaluateRule({ id: "rule:rust-dependency-policy", index: cargoIndex() }).violations).toEqual([]);
  for (const [name, index, message] of [
    ["literal MSRV", cargoIndex({ manifestMutation: ["extractum-core", 'rust-version.workspace = true', 'rust-version = "1.95"'] }), "rust-version must inherit workspace"],
    ["literal Edition", cargoIndex({ manifestMutation: ["extractum-core", 'edition.workspace = true', 'edition = "2021"'] }), "edition must inherit workspace"],
    ["missing root MSRV", cargoIndex({ rootManifestMutation: ['rust-version = "1.95"', ""] }), "workspace must own rust-version 1.95"],
    ["outside manifest", cargoIndex({ metadataMutation: (metadata) => { metadata.packages[0].manifest_path = "C:/outside/Cargo.toml"; } }), "escapes root"],
    ["widened direct requirement", cargoIndex({ policyMutation: (policy) => { policy.directRequirements[0].requirement = "^2 || ^3"; } }), "directRequirements"],
  ] as const) {
    expect(evaluateRule({ id: "rule:rust-dependency-policy", index }).violations, name)
      .toEqual(expect.arrayContaining([expect.stringContaining(message)]));
  }
});
```

Extend `cargoIndex` with the exact optional fields shown (`manifestMutation`, `rootManifestMutation`, `metadataMutation`, `policyMutation`); it clones fixture inputs, applies one mutation, and exposes mutated manifest text through the existing `readFile` seam.

- [ ] **Step 9: Generate and manually complete the schema-2 artifact**

Before the first writer run, manually replace the schema-1 artifact with a schema-2 reviewed seed. Retain the existing `toolchain`, initialize `directRequirements`, `npmRequirements`, `exactPins`, and `approvedPrereleases` to `[]`, and enter these reviewed objects exactly:

```json
"tauriFamily": {
  "pairs": [
    { "id": "tauri-api", "cargo": { "package": "extractum", "dependency": "tauri", "rename": null, "kind": "normal", "target": null }, "cargoRequirement": "^2", "npm": { "owner": "extractum", "name": "@tauri-apps/api", "kind": "dependencies" }, "npmRequirement": "^2" },
    { "id": "tauri-build-cli", "cargo": { "package": "extractum", "dependency": "tauri-build", "rename": null, "kind": "build", "target": null }, "cargoRequirement": "^2", "npm": { "owner": "extractum", "name": "@tauri-apps/cli", "kind": "devDependencies" }, "npmRequirement": "^2" },
    { "id": "tauri-dialog", "cargo": { "package": "extractum", "dependency": "tauri-plugin-dialog", "rename": null, "kind": "normal", "target": null }, "cargoRequirement": "^2", "npm": { "owner": "extractum", "name": "@tauri-apps/plugin-dialog", "kind": "dependencies" }, "npmRequirement": "^2" },
    { "id": "tauri-opener", "cargo": { "package": "extractum", "dependency": "tauri-plugin-opener", "rename": null, "kind": "normal", "target": null }, "cargoRequirement": "^2", "npm": { "owner": "extractum", "name": "@tauri-apps/plugin-opener", "kind": "dependencies" }, "npmRequirement": "^2" },
    { "id": "tauri-sql", "cargo": { "package": "extractum", "dependency": "tauri-plugin-sql", "rename": null, "kind": "normal", "target": null }, "cargoRequirement": "^2", "npm": { "owner": "extractum", "name": "@tauri-apps/plugin-sql", "kind": "dependencies" }, "npmRequirement": "^2.4.0" }
  ],
  "cargoOnlyRequirements": [
    { "id": "tauri-plugin-mcp-bridge", "cargo": { "package": "extractum", "dependency": "tauri-plugin-mcp-bridge", "rename": null, "kind": "normal", "target": null }, "cargoRequirement": "^0.11" }
  ]
}
```

This seed is the only manual schema-1-to-2 bridge. Add this preservation test; the raw schema-1 RED test must still fail and the writer must refuse schema 1:

```ts
it("seeds schema 2 once and preserves reviewed Tauri approvals while generating inventories", () => {
  const seed = reviewedSchema2Seed();
  expect(() => validateReviewedPolicySeed(seed)).not.toThrow();
  const liveCandidate = generateRustDependencyPolicy({ ...fixtureInputs(), committedPolicy: seed });
  expect(liveCandidate.tauriFamily).toEqual(seed.tauriFamily);
  expect(liveCandidate.toolchain).toEqual(seed.toolchain);
  expect(liveCandidate.directRequirements.length).toBeGreaterThan(0);
  expect(validateRustDependencyPolicy({ generated: liveCandidate, committed: seed }))
    .toEqual(expect.arrayContaining([expect.stringContaining("directRequirements")]));
  const written = { ...liveCandidate, tauriFamily: structuredClone(seed.tauriFamily) };
  expect(validateRustDependencyPolicy({ generated: liveCandidate, committed: written })).toEqual([]);
});
```

`reviewedSchema2Seed()` returns the exact JSON object in this step, including empty generated arrays. After the seed exists, run:

```powershell
node scripts/rust-dependency-policy.mjs --write scripts/testing/rust-dependency-policy.json
```

Expected: generated complete sorted Cargo/npm/exact/prerelease inventories; the five seeded pairs and sole MCP record remain byte-for-semantic-value unchanged rather than auto-approved. Run preservation tests after the seed, then run the generator without `--write`; expected stdout equals the committed artifact semantically and no file changes.

- [ ] **Step 10: Run Task 2 GREEN and mutation suite**

```powershell
npm.cmd run test:unit -- scripts/rust-dependency-policy.test.ts scripts/testing/repository-index.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
npm.cmd run test:unit
npm.cmd run check
```

Expected: PASS; real repository snapshot has zero policy violations. Confirm `git status --short` contains no Cargo manifest or lockfile.

- [ ] **Step 11: Commit and review Task 2**

```powershell
git add scripts/rust-dependency-policy.mjs scripts/rust-dependency-policy.test.ts scripts/testing/rust-dependency-policy.json scripts/testing/repository-index.mjs scripts/testing/repository-index.test.ts scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
git commit -m "test: harden Rust manifest and dependency policy"
```

Expected: only policy/index/test files. Run immediate scope/spec/code review and re-review any adjacent correction.

---

### Task 3: Make the Windows Duplicate Policy Diff-Aware

**Files:**
- Modify: `scripts/rust-duplicate-baseline.mjs`
- Modify: `scripts/rust-duplicate-baseline.test.ts`
- Modify: `scripts/testing/rust-duplicate-baseline.json`
- Modify: `scripts/testing/rust-supply-chain-exceptions.json`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `scripts/testing/test-conventions.test.ts`
- Modify: `package.json`
- Modify: `.github/workflows/rust-fast.yml`
- Modify: `.github/workflows/rust-full.yml`
- Modify: `.github/workflows/rust-release.yml`
- Test: `scripts/testing/github-rust-workflows.test.ts`

**Interfaces:**
- Consumes: locked active Cargo tree, current baseline/exceptions, optional explicit Git base, Task 2 repository index.
- Produces: exact current-state gate; explicit local/CI base precedence; deterministic CLEAN/REVIEW/REDUCTION/VIOLATION result; workflow mode wiring.

- [ ] **Step 1: Write baseline and exception semantic-validator RED tests**

Add table-driven mutations for wrong/extra/missing top-level fields, schema/target, unsorted keys, cardinality `<=1`, noninteger values, counter arithmetic, exception array types, exact six-field entries, sorting/uniqueness, `approvedCount > previousCount >= 1`, trimmed owner/reason, RFC 3339 UTC ending `Z`, baseline attachment, and injected-clock current expiry where equality is expired:

```ts
const validBaseline = () => ({
  schemaVersion: 1,
  target: "x86_64-pc-windows-msvc",
  duplicateNameCount: 1,
  duplicateVersionInstanceCount: 2,
  duplicateCardinality: { alpha: 2 },
});
const validExceptions = () => ({
  schemaVersion: 1,
  licenseExceptions: [],
  advisoryExceptions: [],
  duplicateGrowthExceptions: [{
    package: "alpha", previousCount: 1, approvedCount: 2,
    owner: "desktop-platform", reason: "fixture growth",
    reviewAfter: "2026-09-14T00:00:00Z",
  }],
});

for (const [name, mutate, message] of baselineMutations) {
  it(`rejects baseline ${name}`, () => {
    expect(() => validateDuplicateBaseline(mutate(validBaseline()))).toThrow(message);
  });
}
for (const [name, mutate, message] of exceptionMutations) {
  it(`rejects exception ${name}`, () => {
    expect(() => validateSupplyChainExceptions(
      mutate(validExceptions()),
      validBaseline(),
      { now: new Date("2026-08-14T12:00:00Z"), enforceExpiry: true },
    )).toThrow(message);
  });
}

it.each([null, [], "cardinalities", 7])("rejects duplicateCardinality container %j", (container) => {
  expect(() => validateDuplicateBaseline({ ...validBaseline(), duplicateCardinality: container })).toThrow("duplicateCardinality must be a plain object");
});

it.each([null, [], "entry", 7])("rejects duplicate exception entry %j", (entry) => {
  expect(() => validateSupplyChainExceptions(
    { ...validExceptions(), duplicateGrowthExceptions: [entry] },
    validBaseline(),
    { now: new Date("2026-08-14T12:00:00Z"), enforceExpiry: true },
  )).toThrow("duplicate exception must be a plain object");
});

it.each([
  ["package", 7, "package must be a nonempty string"],
  ["owner", null, "missing ownership"],
  ["reason", [], "missing ownership"],
  ["previousCount", "1", "invalid counts"],
  ["approvedCount", 2.5, "invalid counts"],
  ["reviewAfter", 7, "must be a string"],
])("rejects wrong duplicate exception field type for %s", (field, value, message) => {
  const entry = { ...validExceptions().duplicateGrowthExceptions[0], [field]: value };
  expect(() => validateSupplyChainExceptions(
    { ...validExceptions(), duplicateGrowthExceptions: [entry] }, validBaseline(),
    { now: new Date("2026-08-14T12:00:00Z"), enforceExpiry: true },
  )).toThrow(message);
});

it.each(["2026-02-30T00:00:00Z", "2026-08-14T24:00:00Z", "2026-08-14T12:60:00Z", "2026-08-14T12:00:00+00:00", "2026-08-14T12:00:00.Z"])(
  "rejects noncanonical or invalid UTC reviewAfter %s",
  (reviewAfter) => {
    const entry = { ...validExceptions().duplicateGrowthExceptions[0], reviewAfter };
    expect(() => validateSupplyChainExceptions(
      { ...validExceptions(), duplicateGrowthExceptions: [entry] }, validBaseline(),
      { now: new Date("2026-08-14T12:00:00Z"), enforceExpiry: true },
    )).toThrow(/canonical|invalid/);
  },
);

it.each(["2026-09-14T00:00:00.1Z", "2026-09-14T00:00:00.123Z", "2026-09-14T00:00:00.12345678901234567890Z"])(
  "accepts RFC 3339 UTC fractional seconds of arbitrary precision: %s",
  (reviewAfter) => {
    const entry = { ...validExceptions().duplicateGrowthExceptions[0], reviewAfter };
    expect(() => validateSupplyChainExceptions(
      { ...validExceptions(), duplicateGrowthExceptions: [entry] }, validBaseline(),
      { now: new Date("2026-08-14T12:00:00Z"), enforceExpiry: true },
    )).not.toThrow();
  },
);

it("compares arbitrary RFC 3339 fractions without millisecond truncation", () => {
  const artifact = (reviewAfter: string) => ({
    ...validExceptions(),
    duplicateGrowthExceptions: [{ ...validExceptions().duplicateGrowthExceptions[0], reviewAfter }],
  });
  expect(() => validateSupplyChainExceptions(artifact("2026-08-14T12:00:00.1230Z"), validBaseline(), {
    now: new Date("2026-08-14T12:00:00.123Z"), enforceExpiry: true,
  })).toThrow("expired duplicate exception");
  expect(() => validateSupplyChainExceptions(artifact("2026-08-14T12:00:00.1234Z"), validBaseline(), {
    now: new Date("2026-08-14T12:00:00.123Z"), enforceExpiry: true,
  })).not.toThrow();
});
```

The mutation tables contain one function per named defect; do not combine two invalid properties in one fixture.

- [ ] **Step 2: Run semantic-validator RED**

```powershell
npm.cmd run test:unit -- scripts/rust-duplicate-baseline.test.ts scripts/testing/repository-rules.test.ts
```

Expected: FAIL because validators and exact current equality are absent.

- [ ] **Step 3: Implement shared semantic validation and current-state equality**

Implement the validators and shared current-state helper with this algorithm skeleton:

```js
function isPlainObject(value) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function requireExactKeys(value, expected, label) {
  if (!isPlainObject(value)) throw new Error(`${label} must be a plain object`);
  const actual = Object.keys(value).sort();
  const wanted = [...expected].sort();
  if (JSON.stringify(actual) !== JSON.stringify(wanted)) throw new Error(`${label}: expected fields ${wanted.join(",")}`);
}

function parseCanonicalUtc(value, label) {
  if (typeof value !== "string") throw new Error(`${label} must be a string`);
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.(\d+))?Z$/.exec(value);
  if (!match) throw new Error(`${label} must be canonical RFC 3339 UTC`);
  const [, yearText, monthText, dayText, hourText, minuteText, secondText, fractionText = ""] = match;
  const [year, month, day, hour, minute, second] = [yearText, monthText, dayText, hourText, minuteText, secondText].map(Number);
  if (hour > 23 || minute > 59 || second > 59) throw new Error(`${label} contains an invalid UTC time`);
  const roundTrip = new Date(0);
  roundTrip.setUTCFullYear(year, month - 1, day);
  roundTrip.setUTCHours(hour, minute, second, 0);
  if (roundTrip.getUTCFullYear() !== year || roundTrip.getUTCMonth() !== month - 1
    || roundTrip.getUTCDate() !== day || roundTrip.getUTCHours() !== hour
    || roundTrip.getUTCMinutes() !== minute || roundTrip.getUTCSeconds() !== second) {
    throw new Error(`${label} contains an invalid UTC calendar date`);
  }
  return {
    epochSeconds: BigInt(Math.trunc(roundTrip.getTime() / 1000)),
    fraction: fractionText.replace(/0+$/, ""),
  };
}

function dateUtcInstant(value, label) {
  if (!(value instanceof Date) || Number.isNaN(value.getTime())) throw new Error(`${label} must be a valid Date`);
  const milliseconds = value.getTime();
  const epochSeconds = Math.floor(milliseconds / 1000);
  const remainder = milliseconds - epochSeconds * 1000;
  return { epochSeconds: BigInt(epochSeconds), fraction: String(remainder).padStart(3, "0").replace(/0+$/, "") };
}

function compareUtcInstants(left, right) {
  if (left.epochSeconds !== right.epochSeconds) return left.epochSeconds < right.epochSeconds ? -1 : 1;
  const length = Math.max(left.fraction.length, right.fraction.length);
  for (let index = 0; index < length; index += 1) {
    const leftDigit = index < left.fraction.length ? left.fraction.charCodeAt(index) : 48;
    const rightDigit = index < right.fraction.length ? right.fraction.charCodeAt(index) : 48;
    if (leftDigit !== rightDigit) return leftDigit < rightDigit ? -1 : 1;
  }
  return 0;
}

export function validateDuplicateBaseline(value) {
  requireExactKeys(value, ["schemaVersion", "target", "duplicateNameCount", "duplicateVersionInstanceCount", "duplicateCardinality"], "duplicate baseline");
  if (value.schemaVersion !== 1) throw new Error("duplicate baseline schemaVersion must be 1");
  if (typeof value.target !== "string" || value.target !== TARGET) throw new Error(`duplicate baseline target must be ${TARGET}`);
  if (!Number.isInteger(value.duplicateNameCount) || value.duplicateNameCount < 0) throw new Error("duplicateNameCount must be a nonnegative integer");
  if (!Number.isInteger(value.duplicateVersionInstanceCount) || value.duplicateVersionInstanceCount < 0) throw new Error("duplicateVersionInstanceCount must be a nonnegative integer");
  if (!isPlainObject(value.duplicateCardinality)) throw new Error("duplicateCardinality must be a plain object");
  const names = Object.keys(value.duplicateCardinality);
  if (JSON.stringify(names) !== JSON.stringify([...names].sort())) throw new Error("duplicateCardinality must be sorted");
  for (const [name, count] of Object.entries(value.duplicateCardinality)) {
    if (!name || !Number.isInteger(count) || count <= 1) throw new Error(`invalid duplicate cardinality for ${name}`);
  }
  if (value.duplicateNameCount !== names.length) throw new Error("duplicateNameCount arithmetic mismatch");
  if (value.duplicateVersionInstanceCount !== Object.values(value.duplicateCardinality).reduce((sum, count) => sum + count, 0)) throw new Error("duplicateVersionInstanceCount arithmetic mismatch");
  return structuredClone(value);
}

export function validateSupplyChainExceptions(value, baseline, { now, enforceExpiry }) {
  validateDuplicateBaseline(baseline);
  requireExactKeys(value, ["schemaVersion", "licenseExceptions", "advisoryExceptions", "duplicateGrowthExceptions"], "supply-chain exceptions");
  if (value.schemaVersion !== 1) throw new Error("supply-chain exception schemaVersion must be 1");
  for (const field of ["licenseExceptions", "advisoryExceptions", "duplicateGrowthExceptions"]) {
    if (!Array.isArray(value[field])) throw new Error(`${field} must be an array`);
  }
  if (!(now instanceof Date) || Number.isNaN(now.getTime())) throw new Error("now must be a valid Date");
  let previousPackage = null;
  for (const entry of value.duplicateGrowthExceptions) {
    if (!isPlainObject(entry)) throw new Error("duplicate exception must be a plain object");
    requireExactKeys(entry, ["package", "previousCount", "approvedCount", "owner", "reason", "reviewAfter"], "duplicate exception");
    if (typeof entry.package !== "string" || !entry.package.trim()) throw new Error("duplicate exception package must be a nonempty string");
    if (previousPackage !== null && previousPackage.localeCompare(entry.package, "en") >= 0) throw new Error("duplicate exceptions must be unique and sorted");
    if (!Number.isInteger(entry.previousCount) || !Number.isInteger(entry.approvedCount) || entry.previousCount < 1 || entry.approvedCount <= entry.previousCount) throw new Error(`invalid counts for ${entry.package}`);
    if (typeof entry.owner !== "string" || typeof entry.reason !== "string" || !entry.owner.trim() || !entry.reason.trim()) throw new Error(`missing ownership for ${entry.package}`);
    const reviewInstant = parseCanonicalUtc(entry.reviewAfter, `reviewAfter for ${entry.package}`);
    if (baseline.duplicateCardinality[entry.package] !== entry.approvedCount) throw new Error(`exception attachment mismatch for ${entry.package}`);
    if (enforceExpiry && compareUtcInstants(dateUtcInstant(now, "now"), reviewInstant) >= 0) throw new Error(`expired duplicate exception for ${entry.package}`);
    previousPackage = entry.package;
  }
  return structuredClone(value);
}

export function validateCurrentDuplicateState({ treeText, baseline, exceptions, now }) {
  const validatedBaseline = validateDuplicateBaseline(baseline);
  const generated = generateRustDuplicateBaseline(treeText);
  if (JSON.stringify(generated) !== JSON.stringify(validatedBaseline)) throw new Error("active duplicate graph differs from committed baseline");
  return {
    baseline: validatedBaseline,
    exceptions: validateSupplyChainExceptions(exceptions, validatedBaseline, { now, enforceExpiry: true }),
  };
}
```

The current checker regenerates via locked command:

```text
cargo tree --manifest-path src-tauri/Cargo.toml --locked --target x86_64-pc-windows-msvc --workspace --prefix none --format {p}
```

Reject stale reductions and any graph/baseline drift. Repository rules call the same validators and current equality helper; remove old ceiling and loose replacement logic.

- [ ] **Step 4: Write the complete transition-state RED matrix**

Use fixture repositories with commits so `git rev-parse` and `git show` are real. Inject `now = 2026-08-14T12:00:00Z`. Cover: missing commit/artifact; malformed historical baseline/exception; identical state and active carry; pure reduction; balanced true replacement with equal aggregate/multiset/common cardinalities; invented exception on replacement; mixed growth; valid/missing growth exception; expired carry; active/expired metadata renewal; partial/full reduction; base `{P:5,A:7}` to `C:6` requiring `{P:5,A:6}`; `C<=5` requiring removal; illegal removal at six; active base `C>A`; expired base `C>A`; new exception on unchanged state; and expiry exactly equal to now.

Assert structured results, not only log strings. The balanced `alpha:2` to `beta:2` fixture uses this exact assertion:

```js
expect(result).toEqual({
  classification: "REVIEW",
  addedDuplicateNames: ["beta"],
  removedDuplicateNames: ["alpha"],
  reductions: [],
  violations: [],
});

const baselineFor = (duplicateCardinality: Record<string, number>) => ({
  schemaVersion: 1,
  target: "x86_64-pc-windows-msvc",
  duplicateNameCount: Object.keys(duplicateCardinality).length,
  duplicateVersionInstanceCount: Object.values(duplicateCardinality).reduce((sum, count) => sum + count, 0),
  duplicateCardinality,
});
const exceptionFor = (packageName: string, previousCount: number, approvedCount: number, reviewAfter = "2026-09-14T00:00:00Z") => ({
  package: packageName, previousCount, approvedCount,
  owner: "desktop-platform", reason: "fixture approval", reviewAfter,
});
const exceptionArtifact = (duplicateGrowthExceptions: any[]) => ({
  schemaVersion: 1, licenseExceptions: [], advisoryExceptions: [], duplicateGrowthExceptions,
});
const classify = (base: Record<string, number>, current: Record<string, number>, baseEntries: any[] = [], currentEntries: any[] = []) =>
  classifyDuplicateTransition({
    baseBaseline: baselineFor(base), baseExceptions: exceptionArtifact(baseEntries),
    currentBaseline: baselineFor(current), currentExceptions: exceptionArtifact(currentEntries),
    now: new Date("2026-08-14T12:00:00Z"),
  });

expect(classify({ alpha: 3 }, { alpha: 2 }).classification).toBe("REDUCTION");
expect(classify({ alpha: 2 }, { alpha: 2 })).toEqual({
  classification: "CLEAN", addedDuplicateNames: [], removedDuplicateNames: [], reductions: [], violations: [],
});
expect(classify({ alpha: 2 }, { alpha: 2 }, [exceptionFor("alpha", 1, 2)], [exceptionFor("alpha", 1, 2)]).classification).toBe("CLEAN");
expect(classify({ alpha: 2 }, { alpha: 3 }).violations).toContain("alpha: positive duplicate growth requires an exception");
expect(classify(
  { alpha: 2 }, { alpha: 2 },
  [exceptionFor("alpha", 1, 2, "2026-08-01T00:00:00Z")],
  [exceptionFor("alpha", 1, 2, "2026-08-01T00:00:00Z")],
).violations).toContain("alpha: expired approval must be renewed");
expect(classify(
  { alpha: 7 }, { alpha: 6 },
  [exceptionFor("alpha", 5, 7)], [exceptionFor("alpha", 5, 6)],
).classification).toBe("REDUCTION");
expect(classify(
  { alpha: 7 }, { alpha: 8 },
  [exceptionFor("alpha", 5, 7, "2026-08-01T00:00:00Z")], [exceptionFor("alpha", 7, 8)],
).violations).toContain("alpha: expired approval cannot authorize growth");
```

- [ ] **Step 5: Implement deterministic transition classification**

Use this deterministic classification skeleton (helper assertions append package-specific strings to `violations`):

```js
const names = [...new Set([
  ...Object.keys(baseBaseline.duplicateCardinality),
  ...Object.keys(currentBaseline.duplicateCardinality),
])].sort();
const count = (baseline, name) => baseline.duplicateCardinality[name] ?? 1;
const deltas = names.map((packageName) => ({ package: packageName, base: count(baseBaseline, packageName), current: count(currentBaseline, packageName) }));
const positives = deltas.filter(({ current, base }) => current > base);
const negatives = deltas.filter(({ current, base }) => current < base);
const commonChanged = deltas.filter(({ base, current }) => base > 1 && current > 1 && base !== current);
const added = deltas.filter(({ base, current }) => base === 1 && current > 1);
const removed = deltas.filter(({ base, current }) => base > 1 && current === 1);
const identical = positives.length === 0 && negatives.length === 0;
const pureReduction = positives.length === 0 && negatives.length > 0 && added.length === 0;
const replacement = baseBaseline.duplicateNameCount === currentBaseline.duplicateNameCount
  && baseBaseline.duplicateVersionInstanceCount === currentBaseline.duplicateVersionInstanceCount
  && !identical
  && added.length > 0
  && removed.length > 0
  && commonChanged.length === 0
  && JSON.stringify(removed.map(({ base }) => base).sort()) === JSON.stringify(added.map(({ current }) => current).sort());
```

Then build `baseByPackage`/`currentByPackage` exception maps and execute this exact table for every sorted union package:

| Base entry | Cardinality | Required current entry/action |
| --- | --- | --- |
| none | no positive nonreplacement delta | none; any entry is invented |
| none | positive nonreplacement delta | `{previousCount: base count, approvedCount: current count}`, future expiry |
| `{P,A}` | `C == A` | identical active entry, or same P/A with strictly later future `reviewAfter` |
| `{P,A}` | `P < C < A` | replace with `{P,C}` and future expiry |
| `{P,A}` | `C <= P` | remove entry |
| active `{P,A}` | `C > A` | replace with `{previousCount:A, approvedCount:C}` and future expiry |
| expired `{P,A}` | `C > A` | violation; renewal must be a separate transition |

Implement the table with this loop after the delta predicates:

```js
const baseByPackage = new Map(baseExceptions.duplicateGrowthExceptions.map((entry) => [entry.package, entry]));
const currentByPackage = new Map(currentExceptions.duplicateGrowthExceptions.map((entry) => [entry.package, entry]));
const exceptionPackages = [...new Set([...baseByPackage.keys(), ...currentByPackage.keys()])].sort();
const violations = [];
const sameEntry = (left, right) => JSON.stringify(left) === JSON.stringify(right);
const requireCounts = (entry, previousCount, approvedCount, message) => {
  if (!entry || entry.previousCount !== previousCount || entry.approvedCount !== approvedCount) violations.push(message);
};

for (const packageName of exceptionPackages) {
  const baseEntry = baseByPackage.get(packageName);
  const currentEntry = currentByPackage.get(packageName);
  const baseCount = count(baseBaseline, packageName);
  const currentCount = count(currentBaseline, packageName);
  if (!baseEntry) {
    const requiresGrowth = currentCount > baseCount && !replacement;
    if (!currentEntry) {
      if (requiresGrowth) violations.push(`${packageName}: positive duplicate growth requires an exception`);
    } else if (!requiresGrowth) {
      violations.push(`${packageName}: invented duplicate growth exception`);
    } else {
      requireCounts(currentEntry, baseCount, currentCount, `${packageName}: new exception counts must match transition`);
    }
    continue;
  }
  const P = baseEntry.previousCount;
  const A = baseEntry.approvedCount;
  const C = currentCount;
  const nowInstant = dateUtcInstant(now, "now");
  const baseReviewInstant = parseCanonicalUtc(baseEntry.reviewAfter, `base reviewAfter for ${packageName}`);
  const baseExpired = compareUtcInstants(nowInstant, baseReviewInstant) >= 0;
  if (C === A) {
    if (!currentEntry) violations.push(`${packageName}: carried approval was removed`);
    else if (sameEntry(baseEntry, currentEntry)) {
      if (baseExpired) violations.push(`${packageName}: expired approval must be renewed`);
    } else if (currentEntry.previousCount !== P || currentEntry.approvedCount !== A
      || compareUtcInstants(
        parseCanonicalUtc(currentEntry.reviewAfter, `current reviewAfter for ${packageName}`),
        baseReviewInstant,
      ) <= 0) {
      violations.push(`${packageName}: invalid metadata-only renewal`);
    }
  } else if (P < C && C < A) {
    requireCounts(currentEntry, P, C, `${packageName}: partial reduction must lower approvedCount`);
  } else if (C <= P) {
    if (currentEntry) violations.push(`${packageName}: full reduction must remove exception`);
  } else if (C > A) {
    if (baseExpired) violations.push(`${packageName}: expired approval cannot authorize growth`);
    else requireCounts(currentEntry, A, C, `${packageName}: renewed growth counts must start at prior approval`);
  }
}

for (const { package: packageName, base, current } of positives) {
  if (!replacement && !currentByPackage.has(packageName)) violations.push(`${packageName}: positive duplicate growth requires an exception`);
}
for (const packageName of added.map(({ package: name }) => name)) {
  if (replacement && currentByPackage.has(packageName) && !baseByPackage.has(packageName)) violations.push(`${packageName}: review-only replacement must not add an exception`);
}

const reductions = negatives.map(({ package: packageName, base, current }) => ({ package: packageName, previousCount: base, currentCount: current }));
const canonicalViolations = [...new Set(violations)].sort();
const result = {
  classification: canonicalViolations.length ? "VIOLATION"
    : identical ? "CLEAN"
    : replacement ? "REVIEW"
    : reductions.length ? "REDUCTION"
    : "CLEAN",
  addedDuplicateNames: added.map(({ package: packageName }) => packageName),
  removedDuplicateNames: removed.map(({ package: packageName }) => packageName),
  reductions,
  violations: canonicalViolations,
};
return result;
```

Balanced replacement is necessarily nonidentical and has nonempty added and removed sets; it requires no new exception. The classifier checks `identical` before `replacement`, so unchanged state with no exceptions and unchanged state carrying an identical active exception both return CLEAN. The second growth loop may see a package already diagnosed by the lifecycle loop, so `canonicalViolations` sorts and de-duplicates messages. Return CLEAN when identical/approved growth is valid, REDUCTION for pure/exception reductions, REVIEW for replacement, and VIOLATION for any lifecycle/growth failure.

Human output prefixes nonblocking lines with `REVIEW` or `REDUCTION` and failures with `VIOLATION`; output order is package-sorted.

- [ ] **Step 6: Write invocation-precedence RED tests**

Use a table that calls `resolveDuplicateInvocation({ argv, env })` for every combination:

```ts
const VALID_SHA = "1111111111111111111111111111111111111111";
expect(resolveDuplicateInvocation({ argv: ["--check", "--base", "abc123"], env: {} })).toEqual({ writePath: null, baseSha: "abc123", githubMode: null });
expect(resolveDuplicateInvocation({ argv: ["--check"], env: { GITHUB_ACTIONS: "true", RUST_DUPLICATE_BASE_MODE: "required", RUST_DUPLICATE_BASE_SHA: VALID_SHA } })).toEqual({ writePath: null, baseSha: VALID_SHA, githubMode: "required" });
expect(resolveDuplicateInvocation({ argv: ["--check"], env: { GITHUB_ACTIONS: "true", RUST_DUPLICATE_BASE_MODE: "off" } })).toEqual({ writePath: null, baseSha: null, githubMode: "off" });
expect(() => resolveDuplicateInvocation({ argv: ["--write", "scripts/testing/rust-duplicate-baseline.json"], env: { GITHUB_ACTIONS: "true", RUST_DUPLICATE_BASE_MODE: "off" } })).toThrow("policy writers are forbidden in GitHub Actions");
```

Add individual `toThrow` cases for local env variables, CLI/env conflict, missing/unknown CI mode, required missing/all-zero SHA, off with SHA, and CI `--base`.

- [ ] **Step 7: Implement CLI, explicit writer, and Git loader**

Implement the resolver before any Cargo/Git process runs:

```js
export function resolveDuplicateInvocation({ argv, env }) {
  const parsed = parseDuplicateArguments(argv);
  const inActions = env.GITHUB_ACTIONS === "true";
  const mode = env.RUST_DUPLICATE_BASE_MODE ?? null;
  const envSha = env.RUST_DUPLICATE_BASE_SHA ?? null;
  if (!inActions && (mode !== null || envSha !== null)) throw new Error("duplicate base environment is CI-only");
  if (inActions && parsed.writePath !== null) throw new Error("policy writers are forbidden in GitHub Actions");
  if (inActions && parsed.baseSha !== null) throw new Error("--base is local-only");
  if (parsed.writePath !== null && (parsed.check || parsed.baseSha !== null)) throw new Error("--write is mutually exclusive with checking");
  if (!inActions) return { writePath: parsed.writePath, baseSha: parsed.baseSha, githubMode: null };
  if (mode === "required") {
    if (!envSha || /^0+$/.test(envSha)) throw new Error("required duplicate base SHA is missing or zero");
    return { writePath: null, baseSha: envSha, githubMode: "required" };
  }
  if (mode === "off") {
    if (envSha !== null) throw new Error("off duplicate mode forbids a base SHA");
    return { writePath: null, baseSha: null, githubMode: "off" };
  }
  throw new Error("unknown or missing duplicate base mode");
}
```

Implement the parser without a CLI library:

```js
export function parseDuplicateArguments(argv) {
  const parsed = { check: false, writePath: null, baseSha: null };
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === "--check" && !parsed.check) { parsed.check = true; continue; }
    if ((token === "--base" || token === "--write") && index + 1 < argv.length && !argv[index + 1].startsWith("--")) {
      const value = argv[index + 1];
      index += 1;
      if (token === "--base" && parsed.baseSha === null) { parsed.baseSha = value; continue; }
      if (token === "--write" && parsed.writePath === null) { parsed.writePath = value; continue; }
    }
    throw new Error(`invalid or repeated duplicate argument: ${token}`);
  }
  if (parsed.writePath !== null && (parsed.check || parsed.baseSha !== null)) throw new Error("--write is mutually exclusive with checking");
  if (parsed.writePath === null && !parsed.check) throw new Error("expected --check or --write with one path argument");
  if (parsed.baseSha !== null && !parsed.check) throw new Error("--base requires --check");
  return parsed;
}
```

Resolve a nonnull base in Node with `execFileSync("git", ["rev-parse", "--verify", `${baseSha}^{commit}`], options).trim()`, then read the two artifacts with `execFileSync("git", ["show", `${resolvedCommit}:scripts/testing/rust-duplicate-baseline.json`], options)` and the identical command ending in `rust-supply-chain-exceptions.json`. Never infer merge base/HEAD~1 or read local base env. `--check` defaults to current-only locally. The CLI passes generated tree text and parsed current artifacts to `validateCurrentDuplicateState`, validates historical artifacts without current expiry enforcement, then calls `classifyDuplicateTransition`.

```js
function loadHistoricalArtifacts(root, baseSha) {
  const options = { cwd: root, encoding: "utf8", windowsHide: true };
  const resolvedCommit = execFileSync("git", ["rev-parse", "--verify", `${baseSha}^{commit}`], options).trim();
  const readAtCommit = (relativePath) => JSON.parse(execFileSync("git", ["show", `${resolvedCommit}:${relativePath}`], options));
  return {
    baseBaseline: readAtCommit(RUST_DUPLICATE_BASELINE_PATH),
    baseExceptions: readAtCommit(RUST_SUPPLY_CHAIN_EXCEPTIONS_PATH),
  };
}
```

Implement policy composition and the CLI boundary exactly:

```js
export function checkRustDuplicatePolicy({
  treeText, currentBaseline, currentExceptions,
  baseBaseline = null, baseExceptions = null, now,
}) {
  const current = validateCurrentDuplicateState({ treeText, baseline: currentBaseline, exceptions: currentExceptions, now });
  if (baseBaseline === null && baseExceptions === null) {
    return { classification: "CLEAN", addedDuplicateNames: [], removedDuplicateNames: [], reductions: [], violations: [] };
  }
  if (baseBaseline === null || baseExceptions === null) throw new Error("base baseline and exceptions must be supplied together");
  const validatedBase = validateDuplicateBaseline(baseBaseline);
  const validatedBaseExceptions = validateSupplyChainExceptions(baseExceptions, validatedBase, { now, enforceExpiry: false });
  return classifyDuplicateTransition({
    baseBaseline: validatedBase,
    baseExceptions: validatedBaseExceptions,
    currentBaseline: current.baseline,
    currentExceptions: current.exceptions,
    now,
  });
}

export function runRustDuplicateCli({ argv, env, now = new Date(), stdout = process.stdout, stderr = process.stderr, root = repositoryRoot() }) {
  try {
    const invocation = resolveDuplicateInvocation({ argv, env });
    const treeText = cargoTree(root);
    if (invocation.writePath !== null) {
      writeFileSync(path.resolve(root, invocation.writePath), `${JSON.stringify(generateRustDuplicateBaseline(treeText), null, 2)}\n`);
      return 0;
    }
    const currentBaseline = JSON.parse(readFileSync(path.join(root, RUST_DUPLICATE_BASELINE_PATH), "utf8"));
    const currentExceptions = JSON.parse(readFileSync(path.join(root, RUST_SUPPLY_CHAIN_EXCEPTIONS_PATH), "utf8"));
    const historical = invocation.baseSha === null ? { baseBaseline: null, baseExceptions: null }
      : loadHistoricalArtifacts(root, invocation.baseSha);
    const result = checkRustDuplicatePolicy({ treeText, currentBaseline, currentExceptions, ...historical, now });
    for (const message of result.violations) stderr.write(`VIOLATION ${message}\n`);
    if (result.classification === "REVIEW") stdout.write(`REVIEW ${result.removedDuplicateNames.join(",")} -> ${result.addedDuplicateNames.join(",")}\n`);
    for (const reduction of result.reductions) stdout.write(`REDUCTION ${reduction.package}: ${reduction.previousCount} -> ${reduction.currentCount}\n`);
    stdout.write(`${JSON.stringify(result)}\n`);
    return result.classification === "VIOLATION" ? 1 : 0;
  } catch (error) {
    stderr.write(`VIOLATION infrastructure: ${error instanceof Error ? error.message : String(error)}\n`);
    return 2;
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = runRustDuplicateCli({ argv: process.argv.slice(2), env: process.env });
}
```

CLI tests inject string-buffer `{ write(text) { chunks.push(text); } }` stdout/stderr objects and fixture roots. Assert exact exit `0` plus final JSON for CLEAN/REVIEW/REDUCTION, exit `1` plus `VIOLATION alpha: positive duplicate growth requires an exception` for the missing-growth fixture, and exit `2` plus `VIOLATION infrastructure:` for bad arguments, missing Git artifacts, or cargo/git errors. A writer fixture asserts exit 0, exactly one target file write, no current/base check output, and GHA writer exit 2 with `policy writers are forbidden in GitHub Actions`.

Set package scripts:

```json
"generate:rust-duplicate-baseline": "node scripts/rust-duplicate-baseline.mjs --write scripts/testing/rust-duplicate-baseline.json",
"check:rust:duplicates": "node scripts/rust-duplicate-baseline.mjs --check"
```

Add `npm run check:rust:duplicates` to the deterministic `check:rust:fast` composition before supply-chain policy. No checker invokes either writer.

- [ ] **Step 8: Wire exact workflow base modes**

In Rust Fast use immutable checkout (Task 4 will pin SHA), `fetch-depth: 0`, and job environment:

```yaml
env:
  RUST_DUPLICATE_BASE_MODE: required
  RUST_DUPLICATE_BASE_SHA: ${{ github.event_name == 'pull_request' && github.event.pull_request.base.sha || github.event.before }}
```

In Rust Full and each release job that invokes `check:rust:fast`, set only:

```yaml
env:
  RUST_DUPLICATE_BASE_MODE: off
```

They must not define `RUST_DUPLICATE_BASE_SHA`. Extend workflow tests for full history, PR/push SHA expression, missing/all-zero mutations, and `off`/absent SHA.

- [ ] **Step 9: Regenerate current baseline only if live equality requires it**

```powershell
node scripts/rust-duplicate-baseline.mjs --write scripts/testing/rust-duplicate-baseline.json
npm.cmd run check:rust:duplicates
```

Expected current values remain the measured active Windows graph (`duplicateNameCount: 32`, `duplicateVersionInstanceCount: 80`) unless the unchanged locked graph proves otherwise. Any difference stops for investigation; this task does not change dependencies. Keep `duplicateGrowthExceptions` empty unless the explicit historical comparison proves a spec-authorized entry is required.

- [ ] **Step 10: Run Task 3 GREEN including real base evidence**

```powershell
npm.cmd run test:unit -- scripts/rust-duplicate-baseline.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts scripts/testing/github-rust-workflows.test.ts
npm.cmd run check:rust:duplicates
npm.cmd run check:rust:duplicates -- --base 9bcc047b9ec210c1ea34024f6dc782a1e7caec49
npm.cmd run check:rust:fast
```

Expected: all tests PASS; current-only is clean; explicit-base exits 0 and prints a nonempty structured CLEAN/REVIEW/REDUCTION result without mutating artifacts.

- [ ] **Step 11: Commit and review Task 3**

```powershell
git add scripts/rust-duplicate-baseline.mjs scripts/rust-duplicate-baseline.test.ts scripts/testing/rust-duplicate-baseline.json scripts/testing/rust-supply-chain-exceptions.json scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts scripts/testing/github-rust-workflows.test.ts package.json .github/workflows/rust-fast.yml .github/workflows/rust-full.yml .github/workflows/rust-release.yml
git commit -m "test: make Rust duplicate policy diff-aware"
```

Expected: only duplicate policy, its consumers/tests, aliases, and mode wiring. Review immediately; correction commits stay adjacent.

---

### Task 4: Isolate Advisory Writes and Pin Remote Actions

**Files:**
- Modify: `.github/workflows/rust-fast.yml`
- Modify: `.github/workflows/rust-full.yml`
- Modify: `.github/workflows/rust-release.yml`
- Modify: `scripts/testing/github-rust-workflows.test.ts`

**Interfaces:**
- Consumes: Task 1 setup action; Task 3 duplicate-mode environment.
- Produces: exact immutable Action inventory, read-only `advisory-scan`, isolated `advisory-issue-writer`, advisory-gated bundle.

- [ ] **Step 1: Write least-privilege and Action-pin RED mutations**

Add central exact map:

```ts
const REMOTE_ACTIONS = new Map([
  ["actions/checkout", "d23441a48e516b6c34aea4fa41551a30e30af803 # v6"],
  ["actions/setup-node", "249970729cb0ef3589644e2896645e5dc5ba9c38 # v6"],
  ["actions/upload-artifact", "ea165f8d65b6e75b540449e92b4886f43607fa02 # v4"],
  ["actions/github-script", "ed597411d8f924073f98dfc5c65a23a2325f34cd # v8"],
  ["Swatinem/rust-cache", "6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2"],
]);
const CHECKOUT = "actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803";
const SETUP_NODE = "actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38";
const UPLOAD_ARTIFACT = "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02";
const GITHUB_SCRIPT = "actions/github-script@ed597411d8f924073f98dfc5c65a23a2325f34cd";
const RUST_CACHE = "Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6";
```

Replace the existing `Step`/`Job` declarations and `jobs`/`steps` parser bodies with the block below; retain `text` arrays for the existing artifact-name and no-install assertions. Keep raw Action parsing deliberately separate from comment-stripped structural parsing:

```ts
type Step = {
  id?: string; name?: string; shell?: string; if?: string;
  continueOnError?: string; run?: string; uses?: string; text: string[];
};
type Job = {
  id: string; text: string[]; runsOn?: string; needs?: string; if?: string;
  permissions: Record<string, string>; outputs: Record<string, string>;
  concurrency: Record<string, string>; steps: Step[];
};
function rawLines(source: string) {
  return source.replaceAll("\r\n", "\n").split("\n");
}
function lines(source: string) {
  return rawLines(source).map(stripComment);
}
function keyValue(content: string) {
  const separator = content.indexOf(":");
  return separator < 0 ? [content, ""] : [content.slice(0, separator), content.slice(separator + 1).trim()];
}
function assignStep(step: Step, key: string, value: string) {
  if (key === "id" || key === "name" || key === "shell" || key === "if" || key === "run" || key === "uses") step[key] = value;
  else if (key === "continue-on-error") step.continueOnError = value;
}
function jobs(input: string[]): Map<string, Job> {
  const parsed = new Map<string, Job>();
  let inJobs = false;
  let job: Job | undefined;
  let section: "permissions" | "outputs" | "concurrency" | "steps" | null = null;
  let step: Step | undefined;
  for (const original of input) {
    const indent = original.length - original.trimStart().length;
    const content = original.trim();
    if (!content) continue;
    if (indent === 0) { inJobs = content === "jobs:"; job = undefined; section = null; step = undefined; continue; }
    if (!inJobs) continue;
    if (indent === 2 && /^[A-Za-z0-9_-]+:$/.test(content)) {
      const id = content.slice(0, -1);
      job = { id, text: [], permissions: {}, outputs: {}, concurrency: {}, steps: [] };
      parsed.set(id, job); section = null; step = undefined; continue;
    }
    if (!job) continue;
    job.text.push(original);
    if (indent === 4) {
      step = undefined;
      const [key, value] = keyValue(content);
      if (["permissions", "outputs", "concurrency", "steps"].includes(key) && value === "") {
        section = key as "permissions" | "outputs" | "concurrency" | "steps"; continue;
      }
      section = null;
      if (key === "runs-on") job.runsOn = value;
      else if (key === "needs") job.needs = value;
      else if (key === "if") job.if = value;
      continue;
    }
    if (section === "steps" && indent === 6 && content.startsWith("- ")) {
      step = { text: [original] }; job.steps.push(step);
      const [key, value] = keyValue(content.slice(2)); assignStep(step, key, value); continue;
    }
    if (section === "steps" && step && indent > 6) step.text.push(original);
    if (section === "steps" && indent === 8 && step) {
      const [key, value] = keyValue(content); assignStep(step, key, value); continue;
    }
    if (indent === 6 && section && section !== "steps") {
      const [key, value] = keyValue(content); job[section][key] = value;
    }
  }
  return parsed;
}
```

`assertPinnedRemoteActions` receives the raw source so it alone owns `# vN`. `jobs(lines(source))`, `steps`, `stepSequence`, `find`, and `filter` consume comment-stripped values and therefore compare SHA-only `Step.uses` strings. Replace every existing structural expectation:

```ts
assertInOrder(stepSequence(fast), [
  `uses:${CHECKOUT}`, "run:rustup show active-toolchain", `uses:${RUST_CACHE}`,
  "uses:./.github/actions/setup-cargo-deny", "run:npm.cmd run check:rust:fast",
]);
assertInOrder(stepSequence(full), [
  `uses:${CHECKOUT}`, `uses:${SETUP_NODE}`, "run:rustup show active-toolchain", `uses:${RUST_CACHE}`,
  "uses:./.github/actions/setup-cargo-deny", "run:npm.cmd ci", "run:npm.cmd run bootstrap:testing",
  "run:node_modules\\.bin\\playwright.cmd install chromium", "run:npm.cmd run check:rust:fast", "run:npm.cmd run verify",
]);
expect(full.steps.filter((step) => step.uses === UPLOAD_ARTIFACT)).toHaveLength(3);
expect(scan.steps.find((step) => step.uses === GITHUB_SCRIPT)).toBeUndefined();
expect(writer.steps).toHaveLength(1);
expect(writer.steps[0].uses).toBe(GITHUB_SCRIPT);
assertInOrder(stepSequence(bundle), [
  `uses:${CHECKOUT}`, `uses:${SETUP_NODE}`, "run:rustup show active-toolchain", `uses:${RUST_CACHE}`,
  "uses:./.github/actions/setup-cargo-deny", "run:npm.cmd ci", "run:npm.cmd run bootstrap:testing",
  "run:node_modules\\.bin\\playwright.cmd install chromium", "run:npm.cmd run check:rust:fast", "run:npm.cmd run verify",
  "run:npm.cmd run tauri -- build --target x86_64-pc-windows-msvc",
  "run:npm.cmd run smoke:gemini-browser-sidecar:binary", `uses:${UPLOAD_ARTIFACT}`, `uses:${UPLOAD_ARTIFACT}`,
]);
const uploads = bundle.steps.filter((step) => step.uses === UPLOAD_ARTIFACT);
```

Search the completed test with `rg -n 'uses:.*@v[0-9]|step\.uses === ".*@v|uses === ".*@v' scripts/testing/github-rust-workflows.test.ts`; expected no structural `@vN` comparison remains.

Add this raw-line contract before comment stripping:

```ts
function assertPinnedRemoteActions(source: string) {
  const usesLines = rawLines(source)
    .filter((line) => /^\s*-?\s*uses:\s*/.test(line));
  for (const line of usesLines) {
    const value = line.replace(/^\s*-?\s*uses:\s*/, "");
    if (value.startsWith("./")) continue;
    const match = /^(?<name>[^@\s]+)@(?<sha>[0-9a-f]{40})\s+#\s+(?<version>v\d+)$/.exec(value);
    requireInvariant(match?.groups, `remote action must use full reviewed SHA and version comment: ${value}`);
    const expected = REMOTE_ACTIONS.get(match.groups.name);
    requireInvariant(expected !== undefined, `unknown remote action: ${match.groups.name}`);
    requireInvariant(`${match.groups.sha} # ${match.groups.version}` === expected, `unreviewed action ref: ${value}`);
  }
}
```

Call it for each workflow. Negative tests require `remote action must use full reviewed SHA and version comment` for tags, branches, short/tag-object SHA, expressions, or missing comments; `unknown remote action` for an unlisted owner/action; and `unreviewed action ref` for a different 40-hex SHA. Relative local actions remain accepted.

`assertReleaseWorkflow` below consumes the indentation parser's typed fields; no `directValue` or free-text fallback remains. Its table covers scanner write scope, continuation, ID, output/final failure conversion, writer condition/runner/permissions/step isolation/concurrency, and bundle dependency/gate. Each mutation invokes `assertReleaseWorkflow(mutated)` and matches the exact owning error.

- [ ] **Step 2: Run workflow RED**

```powershell
npm.cmd run test:unit -- scripts/testing/github-rust-workflows.test.ts
```

Expected: FAIL on mutable Action refs and combined advisory/write job.

- [ ] **Step 3: Pin every remote Action occurrence**

Replace all occurrences in the three workflow files with the exact SHA plus comment from `REMOTE_ACTIONS`. Preserve `Swatinem/rust-cache` peeled commit `6323deb102c322ba6fcbdcafc7e3dddab59af2b6`; never use tag object `49a0bdc70d2e1b713ca9e2869b211fcce03d3c1c`. The test parser must retain comments for `uses:` validation rather than stripping them before the Action contract is checked.

```yaml
uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6
uses: actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38 # v6
uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4
uses: actions/github-script@ed597411d8f924073f98dfc5c65a23a2325f34cd # v8
uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2
```

- [ ] **Step 4: Split the advisory scanner and writer**

Replace the combined advisory job with these complete jobs:

```yaml
  advisory-scan:
    permissions:
      contents: read
    runs-on: windows-latest
    outputs:
      scan_outcome: ${{ steps.scan.outcome }}
    steps:
      - uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6
      - run: rustup show active-toolchain
      - uses: ./.github/actions/setup-cargo-deny
      - id: scan
        continue-on-error: true
        run: npm.cmd run check:rust:advisories
      - name: Fail when advisory scan failed
        if: ${{ steps.scan.outcome == 'failure' }}
        shell: pwsh
        run: exit 1

  advisory-issue-writer:
    needs: advisory-scan
    if: ${{ always() && needs.advisory-scan.outputs.scan_outcome == 'failure' && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}
    permissions:
      issues: write
    concurrency:
      group: rust-advisory-follow-up
      cancel-in-progress: false
    runs-on: ubuntu-latest
    steps:
      - uses: actions/github-script@ed597411d8f924073f98dfc5c65a23a2325f34cd # v8
        with:
          script: |
            const title = "Security: Rust advisory follow-up";
            const body = [
              "`npm.cmd run check:rust:advisories` failed.",
              "",
              `Run: ${context.serverUrl}/${context.repo.owner}/${context.repo.repo}/actions/runs/${context.runId}`,
              "Review cargo-deny output and either update dependencies or add an approved, time-bounded exception.",
            ].join("\n");
            const issues = await github.paginate(github.rest.issues.listForRepo, {
              ...context.repo,
              state: "open",
              per_page: 100,
            });
            const existing = issues.find((issue) => !issue.pull_request && issue.title === title);
            if (existing) {
              await github.rest.issues.createComment({ ...context.repo, issue_number: existing.number, body });
            } else {
              await github.rest.issues.create({ ...context.repo, title, body });
            }
```

Parse and assert every security-relevant field rather than searching job text. This is the complete release assertion; each failure string is owned by one invariant and by the mutation table below:

```ts
const SCAN_FAIL_IF = "${{ steps.scan.outcome == 'failure' }}";
const WRITER_IF = "${{ always() && needs.advisory-scan.outputs.scan_outcome == 'failure' && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}";
const BUNDLE_IF = "${{ needs.advisory-scan.result == 'success' && (startsWith(github.ref, 'refs/tags/v') || (github.event_name == 'workflow_dispatch' && inputs.bundle)) }}";
const exactRecord = (actual: Record<string, string>, expected: Record<string, string>) =>
  JSON.stringify(Object.fromEntries(Object.entries(actual).sort())) === JSON.stringify(Object.fromEntries(Object.entries(expected).sort()));
function requiredJob(parsed: Map<string, Job>, id: string, message: string): Job {
  const job = parsed.get(id);
  if (!job) throw new Error(message);
  return job;
}

function assertReleaseWorkflow(source: string) {
  assertPinnedRemoteActions(source);
  const parsed = jobs(lines(source));
  const scan = requiredJob(parsed, "advisory-scan", "advisory scanner job is required");
  const writer = requiredJob(parsed, "advisory-issue-writer", "advisory writer job is required");
  const bundle = requiredJob(parsed, "windows-bundle", "windows bundle job is required");
  requireInvariant(scan.runsOn === "windows-latest", "advisory scanner must run on windows-latest");
  requireInvariant(exactRecord(scan.permissions, { contents: "read" }), "advisory scanner must have contents read only");
  requireInvariant(exactRecord(scan.outputs, { scan_outcome: "${{ steps.scan.outcome }}" }), "scan_outcome output is required");
  const scanStep = scan.steps.find((step) => step.id === "scan");
  requireInvariant(scanStep !== undefined, "advisory scan step id is required");
  requireInvariant(scanStep?.continueOnError === "true", "scan must continue only to expose its outcome");
  requireInvariant(scanStep?.run === "npm.cmd run check:rust:advisories", "advisory scan command drifted");
  const failStep = scan.steps.at(-1);
  requireInvariant(failStep !== scanStep && failStep?.name === "Fail when advisory scan failed", "scan failure conversion step is required");
  requireInvariant(failStep?.if === SCAN_FAIL_IF, "scan failure conversion condition drifted");
  requireInvariant(failStep?.shell === "pwsh", "scan failure conversion shell must be pwsh");
  requireInvariant(failStep?.run === "exit 1", "scan failure conversion must exit 1");

  requireInvariant(exactRecord(writer.permissions, { issues: "write" }), "advisory writer must have issues write only");
  requireInvariant(writer.runsOn === "ubuntu-latest", "advisory writer must run on ubuntu-latest");
  requireInvariant(writer.needs === "advisory-scan", "advisory writer must need advisory-scan");
  requireInvariant(writer.if === WRITER_IF, "advisory writer condition drifted");
  requireInvariant(writer.steps.length === 1, "advisory writer must contain exactly one step");
  requireInvariant(writer.steps[0]?.uses === GITHUB_SCRIPT, "advisory writer sole step must be pinned github-script");
  requireInvariant(exactRecord(writer.concurrency, { group: "rust-advisory-follow-up", "cancel-in-progress": "false" }), "advisory follow-up concurrency must serialize without cancellation");

  requireInvariant(bundle.needs === "advisory-scan", "bundle must need advisory-scan");
  requireInvariant(bundle.if === BUNDLE_IF, "bundle must require successful advisory scan");
}
```

Use one executable mutation table; every expected message above is reached by its corresponding source mutation:

```ts
const insertWriterStep = (line: string) => (source: string) => source.replace(
  "    steps:\n      - uses: actions/github-script@",
  `    steps:\n      - ${line}\n      - uses: actions/github-script@`,
);
it.each([
  ["scan runner", (source: string) => source.replace("  advisory-scan:\n    permissions:\n      contents: read\n    runs-on: windows-latest", "  advisory-scan:\n    permissions:\n      contents: read\n    runs-on: ubuntu-latest"), "advisory scanner must run on windows-latest"],
  ["scanner permissions", (source: string) => source.replace("  advisory-scan:\n    permissions:\n      contents: read", "  advisory-scan:\n    permissions:\n      issues: write"), "advisory scanner must have contents read only"],
  ["scan output", (source: string) => source.replace("    outputs:\n      scan_outcome: ${{ steps.scan.outcome }}\n", ""), "scan_outcome output is required"],
  ["scan id", (source: string) => source.replace("      - id: scan", "      - name: scan"), "advisory scan step id is required"],
  ["scan continuation", (source: string) => source.replace("continue-on-error: true", "continue-on-error: false"), "scan must continue only to expose its outcome"],
  ["scan command", (source: string) => source.replace("run: npm.cmd run check:rust:advisories", "run: npm.cmd run check:rust:supply-chain"), "advisory scan command drifted"],
  ["missing final fail", (source: string) => source.replace("      - name: Fail when advisory scan failed\n        if: ${{ steps.scan.outcome == 'failure' }}\n        shell: pwsh\n        run: exit 1\n", ""), "scan failure conversion step is required"],
  ["final fail condition", (source: string) => source.replace("if: ${{ steps.scan.outcome == 'failure' }}", "if: ${{ always() }}"), "scan failure conversion condition drifted"],
  ["final fail shell", (source: string) => source.replace("        shell: pwsh\n        run: exit 1", "        shell: bash\n        run: exit 1"), "scan failure conversion shell must be pwsh"],
  ["final fail command", (source: string) => source.replace("        run: exit 1", "        run: exit 0"), "scan failure conversion must exit 1"],
  ["writer permissions", (source: string) => source.replace("  advisory-issue-writer:\n    needs: advisory-scan\n    if: ${{ always() && needs.advisory-scan.outputs.scan_outcome == 'failure' && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}\n    permissions:\n      issues: write", "  advisory-issue-writer:\n    needs: advisory-scan\n    if: ${{ always() && needs.advisory-scan.outputs.scan_outcome == 'failure' && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}\n    permissions:\n      contents: read"), "advisory writer must have issues write only"],
  ["writer runner", (source: string) => source.replace("runs-on: ubuntu-latest", "runs-on: windows-latest"), "advisory writer must run on ubuntu-latest"],
  ["writer needs", (source: string) => source.replace("  advisory-issue-writer:\n    needs: advisory-scan", "  advisory-issue-writer:\n    needs: windows-bundle"), "advisory writer must need advisory-scan"],
  ["setup failure enabling writer", (source: string) => source.replace("always() && needs.advisory-scan.outputs.scan_outcome == 'failure'", "failure()"), "advisory writer condition drifted"],
  ["writer checkout", insertWriterStep(`uses: ${CHECKOUT} # v6`), "advisory writer must contain exactly one step"],
  ["writer cache", insertWriterStep(`uses: ${RUST_CACHE} # v2`), "advisory writer must contain exactly one step"],
  ["writer installer", insertWriterStep("run: ./scripts/setup-cargo-deny.ps1"), "advisory writer must contain exactly one step"],
  ["writer shell", insertWriterStep("run: echo forbidden"), "advisory writer must contain exactly one step"],
  ["writer artifact", insertWriterStep(`uses: ${UPLOAD_ARTIFACT} # v4`), "advisory writer must contain exactly one step"],
  ["writer repository content", insertWriterStep("run: Get-Content README.md"), "advisory writer must contain exactly one step"],
  ["writer wrong sole use", (source: string) => source.replace("uses: actions/github-script@ed597411d8f924073f98dfc5c65a23a2325f34cd # v8", "uses: ./.github/actions/setup-cargo-deny"), "advisory writer sole step must be pinned github-script"],
  ["writer concurrency group", (source: string) => source.replace("group: rust-advisory-follow-up", "group: other"), "advisory follow-up concurrency must serialize without cancellation"],
  ["writer concurrency cancellation", (source: string) => source.replace("cancel-in-progress: false", "cancel-in-progress: true"), "advisory follow-up concurrency must serialize without cancellation"],
  ["missing concurrency", (source: string) => source.replace("    concurrency:\n      group: rust-advisory-follow-up\n      cancel-in-progress: false\n", ""), "advisory follow-up concurrency must serialize without cancellation"],
  ["bundle dependency", (source: string) => source.replace("  windows-bundle:\n    needs: advisory-scan", "  windows-bundle:\n    needs: advisory-issue-writer"), "bundle must need advisory-scan"],
  ["bundle scan gate", (source: string) => source.replace("needs.advisory-scan.result == 'success' && ", ""), "bundle must require successful advisory scan"],
] as const)("rejects advisory workflow mutation: %s", (_name, mutate, message) => {
  expect(() => assertReleaseWorkflow(mutate(workflow("rust-release.yml")))).toThrow(message);
});
```

- [ ] **Step 5: Preserve the bundle block**

Change the bundle header exactly:

```yaml
  windows-bundle:
    needs: advisory-scan
    if: ${{ needs.advisory-scan.result == 'success' && (startsWith(github.ref, 'refs/tags/v') || (github.event_name == 'workflow_dispatch' && inputs.bundle)) }}
    env:
      RUST_DUPLICATE_BASE_MODE: off
    runs-on: windows-latest
```

The workflow test asserts `bundle.needs === "advisory-scan"`, the exact three gate clauses, mode `off`, and absence of `RUST_DUPLICATE_BASE_SHA`. Setup failures leave `scan_outcome` unset, skip writer, and block bundle. An actual advisory failure sets scan outcome failure, permits writer only on schedule/manual, explicitly fails scanner, and blocks bundle.

- [ ] **Step 6: Run Task 4 GREEN and static inspection**

```powershell
npm.cmd run test:unit -- scripts/testing/github-rust-workflows.test.ts
npm.cmd run test:unit
rg -n "uses:" .github/workflows/rust-fast.yml .github/workflows/rust-full.yml .github/workflows/rust-release.yml
git diff --check
```

Expected: PASS; every remote `uses:` matches the reviewed map/comment, local setup action remains relative, advisory scanner has no write, and writer has no source/build/tool step. Remote runtime remains explicitly unclaimed.

- [ ] **Step 7: Commit and review Task 4**

```powershell
git add .github/workflows/rust-fast.yml .github/workflows/rust-full.yml .github/workflows/rust-release.yml scripts/testing/github-rust-workflows.test.ts
git commit -m "ci: isolate advisory writes and pin actions"
```

Expected: only workflows and their contract test. Review exact SHA/comment map and least privilege immediately.

---

### Task 5: Close Bounded Rust Wave 0 Minor Findings

**Files:**
- Modify: `package.json`
- Modify: `scripts/verify.test.ts`
- Modify: `deny.toml`
- Modify: `src-tauri/crates/extractum-gemini-browser/src/types.rs`
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `docs/value-registry.md`
- Modify: `docs/superpowers/plans/2026-08-12-rust-infrastructure-wave-0.md`
- Modify: `docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md`
- Test: `scripts/testing/repository-rules.test.ts`

**Interfaces:**
- Consumes: all Task 1-4 mode/kind/classification names and existing Rust behavior.
- Produces: locked residual aliases, strict license slack rejection, literal golden protocol test, clone-free owned secret flow, complete value registry, factual prior docs.

- [ ] **Step 1: Add locked-alias RED mutations**

In `scripts/verify.test.ts`, require exact commands:

```ts
expect(scripts["test:rust"]).toBe("cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked");
expect(scripts["test:rust:prompt-pack-runs"]).toBe("cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib prompt_pack_run --locked");
```

Run the verifier contract against two mutated package objects with `--locked` removed, and require each to fail for its owning alias.

- [ ] **Step 2: Run alias RED, then make the two minimal edits**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts
```

Expected RED on both unlocked aliases. Add `--locked` to only those two scripts and rerun; expected PASS.

- [ ] **Step 3: Tighten unused-license policy and prove mutations**

Change `deny.toml` to:

```toml
[licenses]
unused-allowed-license = "deny"
unused-license-exception = "deny"
```

Remove only `Apache-2.0 WITH LLVM-exception` from `allow`. In `scripts/testing/repository-rules.test.ts`, add this committed-config helper and exact tests:

```ts
function assertStrictLicensePolicy(source: string) {
  const licenses = /\[licenses\]\s*([\s\S]*?)(?=\n\[(?!licenses\.private\])|$)/.exec(source)?.[1];
  expect(licenses, "missing [licenses]").toBeDefined();
  expect(licenses).toMatch(/^unused-allowed-license = "deny"$/m);
  expect(licenses).toMatch(/^unused-license-exception = "deny"$/m);
  expect(licenses).not.toContain('"Apache-2.0 WITH LLVM-exception"');
  expect(source).not.toMatch(/^\[\[licenses\.exceptions\]\]$/m);
}

it("keeps unused Rust license policy strict", () => {
  assertStrictLicensePolicy(readFileSync("deny.toml", "utf8"));
});

it.each([
  ["unused allowed license", (source: string) => source.replace('"Apache-2.0",', '"Apache-2.0",\n  "Apache-2.0 WITH LLVM-exception",')],
  ["stale license exception", (source: string) => `${source}\n[[licenses.exceptions]]\nname = "not-a-real-package"\nallow = ["MIT"]\n`],
  ["unused allowance warning", (source: string) => source.replace('unused-allowed-license = "deny"', 'unused-allowed-license = "warn"')],
  ["unused exception warning", (source: string) => source.replace('unused-license-exception = "deny"', 'unused-license-exception = "warn"')],
])("rejects %s", (_name, mutate) => {
  expect(() => assertStrictLicensePolicy(mutate(readFileSync("deny.toml", "utf8")))).toThrow();
});
```

Run `npm.cmd run test:unit -- scripts/testing/repository-rules.test.ts`; the four mutated fixtures must fail and the committed config must pass. Then run `npm.cmd run check:rust:supply-chain`; expected PASS for bans/licenses/sources with no new clarification or exception.

- [ ] **Step 4: Replace the sidecar self-round-trip with a literal golden value**

Replace only the body of `sidecar_run_result_response_keeps_wire_shape`. Build `expected_response`, then independently define:

```rust
let golden = serde_json::json!({
    "type": "run_result",
    "result": {
        "run_id": "run-1",
        "status": "ok",
        "text": "answer",
        "message": null,
        "manual_action": null,
        "artifacts": {
            "run_dir": null,
            "html": null,
            "screenshot": null,
            "telemetry": null,
            "answer_extraction": null,
            "artifact_write_error": null
        },
        "elapsed_ms": 42,
        "debug_summary": null
    }
});
assert_eq!(serde_json::to_value(&expected_response).expect("serialize sidecar response"), golden);
let decoded: GeminiBrowserSidecarResponse = serde_json::from_value(serde_json::json!({
    "type": "run_result",
    "result": {
        "run_id": "run-1",
        "status": "ok",
        "text": "answer",
        "message": null,
        "manual_action": null,
        "artifacts": {
            "run_dir": null,
            "html": null,
            "screenshot": null,
            "telemetry": null,
            "answer_extraction": null,
            "artifact_write_error": null
        },
        "elapsed_ms": 42,
        "debug_summary": null
    }
}))
    .expect("deserialize golden sidecar response");
assert_eq!(decoded, expected_response);
```

The deserializer literal must be syntactically independent; do not pass `golden` or serialized output into `from_value`.

- [ ] **Step 5: Run exact Gemini RED/GREEN and checkpoint**

First introduce a deliberate local golden mutation such as `"elapsed_ms": 43`; run the exact test and expect FAIL showing left/right mismatch. Restore `42`, then run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
```

Expected: 1 exact test PASS, focused check PASS, all-target package tests PASS.

- [ ] **Step 6: Record the existing owned-secret characterization GREEN**

Before editing the function, run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact
```

Expected: exactly 1 PASS covering complete, key-only, base-only, empty-key, and empty-base cases. Do not fabricate a runtime failure for clone ownership.

- [ ] **Step 7: Move configured secret ownership without cloning**

Extend the existing test module import so the private result type is named explicitly without changing its visibility:

```rust
use super::{
    cancelled_stream_event, configured_provider_access, failed_stream_event,
    load_provider_diagnostics_from_pool, normalize_configured_provider_overrides,
    save_profile_to_pool, ConfiguredProviderAccess, LlmUsage, ProviderKind, StreamEvent,
};
```

Add the private enum adjacent to the helper:

```rust
enum ConfiguredProviderAccess {
    Complete(LlmProviderAccess),
    Partial {
        api_key: Option<SecretString>,
        base_url: Option<String>,
    },
}
```

Change the helper to consume ownership:

```rust
fn configured_provider_access(
    provider: ProviderKind,
    api_key: Option<SecretString>,
    base_url: Option<String>,
) -> AppResult<ConfiguredProviderAccess> {
    match (api_key, base_url) {
        (Some(api_key), Some(base_url)) => Ok(ConfiguredProviderAccess::Complete(
            LlmProviderAccess::new(provider, api_key, normalize_base_url(provider, Some(&base_url))?),
        )),
        (api_key, base_url) => Ok(ConfiguredProviderAccess::Partial { api_key, base_url }),
    }
}
```

Replace the caller branch exactly:

```rust
let access = match configured_provider_access(provider_kind, configured_key, configured_base_url)? {
    ConfiguredProviderAccess::Complete(access) => access,
    ConfiguredProviderAccess::Partial { api_key, base_url } => {
        let pool = get_pool(&handle).await?;
        resolve_provider_access_from_pool(
            &pool,
            &secret_store,
            provider_kind,
            profile_id.as_deref(),
            api_key,
            base_url,
        )
        .await?
    }
};
```

Replace the characterization body with ownership-shape assertions that never expose the secret:

```rust
let provider = ProviderKind::OpenAiCompatible;
assert!(matches!(
    configured_provider_access(
        provider,
        Some(SecretString::new("configured-key".to_string())),
        Some("https://api.example.test/v1".to_string()),
    ).expect("normalize complete configured access"),
    ConfiguredProviderAccess::Complete(_)
));
for (api_key, base_url, expected_key, expected_base) in [
    (Some(SecretString::new("configured-key".to_string())), None, true, false),
    (None, Some("https://api.example.test/v1".to_string()), false, true),
] {
    match configured_provider_access(provider, api_key, base_url).expect("preserve partial configured access") {
        ConfiguredProviderAccess::Partial { api_key, base_url } => {
            assert_eq!(api_key.is_some(), expected_key);
            assert_eq!(base_url.is_some(), expected_base);
        }
        ConfiguredProviderAccess::Complete(_) => panic!("partial overrides must not become complete"),
    }
}
let (empty_key, configured_base_url) = normalize_configured_provider_overrides(Some("   "), Some(" https://api.example.test/v1 "));
assert!(matches!(
    configured_provider_access(provider, empty_key, configured_base_url).expect("empty key fallback"),
    ConfiguredProviderAccess::Partial { api_key: None, base_url: Some(_) }
));
let (configured_key, empty_base_url) = normalize_configured_provider_overrides(Some(" configured-key "), Some(""));
assert!(matches!(
    configured_provider_access(provider, configured_key, empty_base_url).expect("empty base fallback"),
    ConfiguredProviderAccess::Partial { api_key: Some(_), base_url: None }
));
```

Do not call `clone`, `expose_secret`, `into_inner`, logging, serialization, or persistence from this path.

- [ ] **Step 8: Run owned-secret GREEN, diff proof, and checkpoint**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact
rg -n -C 8 "configured_provider_access|api_key\.clone" src-tauri/src/llm/mod.rs
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
```

Expected: exact 1 PASS, no `api_key.clone()` in the bounded helper/caller, check and package checkpoint PASS; fallback cases unchanged.

- [ ] **Step 9: Extend the value registry and documentation contract**

Add one `## Rust infrastructure tooling values` section and five `###` subsections. Each subsection has exact metadata bullets:

```markdown
### cargo-deny wrapper modes
- Owner: `scripts/run-cargo-deny.ps1`.
- Persistence: checked-in repository and CI command contract only.
- Product impact: no database, persistence API, or product UI impact; command output is operational.
- Fixtures and validators: `scripts/run-cargo-deny.test.ps1`.
| Value |
| --- |
| `Setup` |
| `Deterministic` |
| `Advisories` |

### duplicate base modes
- Owner: `scripts/rust-duplicate-baseline.mjs` and Rust workflows.
- Persistence: checked-in repository and CI environment contract only.
- Product impact: no database, persistence API, or product UI impact; Actions output is operational.
- Fixtures and validators: `scripts/rust-duplicate-baseline.test.ts` and `scripts/testing/github-rust-workflows.test.ts`.
| Value |
| --- |
| `required` |
| `off` |

### Cargo direct-requirement kinds
- Owner: `scripts/rust-dependency-policy.mjs` and `scripts/testing/rust-dependency-policy.json`.
- Persistence: checked-in repository policy only.
- Product impact: no database, persistence API, or product UI impact.
- Fixtures and validators: `scripts/rust-dependency-policy.test.ts` and `scripts/testing/repository-rules.test.ts`.
| Value |
| --- |
| `normal` |
| `build` |
| `dev` |

### npm direct-requirement kinds
- Owner: `scripts/rust-dependency-policy.mjs` and `scripts/testing/rust-dependency-policy.json`.
- Persistence: checked-in repository policy only.
- Product impact: no database, persistence API, or product UI impact.
- Fixtures and validators: `scripts/rust-dependency-policy.test.ts` and `scripts/testing/repository-rules.test.ts`.
| Value |
| --- |
| `dependencies` |
| `devDependencies` |
| `optionalDependencies` |

### duplicate result classifications
- Owner: `scripts/rust-duplicate-baseline.mjs`.
- Persistence: checked-in repository/CI validator contract only.
- Product impact: no database, persistence API, or product UI impact; Actions notices are operational.
- Fixtures and validators: `scripts/rust-duplicate-baseline.test.ts` and `scripts/testing/repository-rules.test.ts`.
| Value |
| --- |
| `VIOLATION` |
| `REVIEW` |
| `REDUCTION` |
```

In `scripts/testing/repository-rules.test.ts`, add an executable contract:

```ts
function markdownSubsection(source: string, heading: string) {
  const marker = `### ${heading}\n`;
  const start = source.indexOf(marker);
  if (start < 0) throw new Error(`missing value-registry subsection: ${heading}`);
  const next = source.indexOf("\n### ", start + marker.length);
  return source.slice(start, next < 0 ? source.length : next);
}
function mutateToolingSubsection(source: string, heading: string, mutate: (section: string) => string) {
  const section = markdownSubsection(source, heading);
  return source.replace(section, mutate(section));
}
const toolingFamilies = [
  ["cargo-deny wrapper modes", ["Setup", "Deterministic", "Advisories"]],
  ["duplicate base modes", ["required", "off"]],
  ["Cargo direct-requirement kinds", ["normal", "build", "dev"]],
  ["npm direct-requirement kinds", ["dependencies", "devDependencies", "optionalDependencies"]],
  ["duplicate result classifications", ["VIOLATION", "REVIEW", "REDUCTION"]],
] as const;
function assertToolingValueRegistry(source: string) {
  for (const [heading, values] of toolingFamilies) {
    const section = markdownSubsection(source, heading);
    for (const prefix of ["- Owner:", "- Persistence:", "- Product impact:", "- Fixtures and validators:"]) expect(section).toContain(prefix);
    expect(section).toContain("no database, persistence API, or product UI impact");
    for (const value of values) expect(section).toContain(`| \`${value}\` |`);
  }
}
it("registers every Rust infrastructure tooling value family", () => {
  assertToolingValueRegistry(readFileSync("docs/value-registry.md", "utf8"));
});
it.each(toolingFamilies.flatMap(([heading, values]) => values.map((value) => [heading, value] as const)))(
  "rejects missing %s value %s", (heading, value) => {
    const source = readFileSync("docs/value-registry.md", "utf8");
    expect(() => assertToolingValueRegistry(mutateToolingSubsection(source, heading, (section) => section.replace(`| \`${value}\` |`, "")))).toThrow();
  },
);
it.each(["Owner", "Persistence", "Product impact", "Fixtures and validators"])("rejects missing registry metadata %s", (label) => {
  const source = readFileSync("docs/value-registry.md", "utf8");
  expect(() => assertToolingValueRegistry(mutateToolingSubsection(source, "cargo-deny wrapper modes", (section) => section.replace(`- ${label}:`, `- Removed ${label}:`)))).toThrow();
});
```

Extend an existing tooling family rather than adding a duplicate if the heading already exists.

- [ ] **Step 10: Correct only stale Wave 0 facts**

Use these exact factual statements in the old plan/verification record:

```markdown
Task 2 RED stopped at the first failing assertion; the final GREEN run covered both locked verifier assertions.

The exact prompt-pack cancellation test is `public_api_tests::cancellation_smoke_services_remain_test_only`.

The original Wave 0 reviewed range ends at `9bcc047b9ec210c1ea34024f6dc782a1e7caec49`; final-hardening commits and evidence are recorded separately in `docs/superpowers/verification/2026-08-14-rust-wave-0-final-hardening.md`.
```

Replace only the contradictory sentences and stale filter near their existing locations. Do not edit the original design and do not claim remote evidence.

- [ ] **Step 11: Run Task 5 GREEN**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/repository-rules.test.ts
npm.cmd run check:rust:supply-chain
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
```

Expected: all PASS. `git diff --name-only` shows only authorized Task 5 files; no Cargo manifest/lock, frontend, migration, schema, or product UI file.

- [ ] **Step 12: Commit and review Task 5**

```powershell
git add package.json scripts/verify.test.ts deny.toml src-tauri/crates/extractum-gemini-browser/src/types.rs src-tauri/src/llm/mod.rs docs/value-registry.md docs/superpowers/plans/2026-08-12-rust-infrastructure-wave-0.md docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md scripts/testing/repository-rules.test.ts
git commit -m "fix: close Rust Wave 0 minor findings"
```

Expected: one bounded minor-fix commit. Review source/API/persistence scope and both exact Rust tests before Task 6.

---

### Task 6: Verify and Record Final Hardening

**Files:**
- Create: `docs/superpowers/verification/2026-08-14-rust-wave-0-final-hardening.md`

**Interfaces:**
- Consumes: reviewed Task 1-5 commit range and every focused/full command below.
- Produces: factual local evidence containing the exact tested implementation head through Task 5 and its hardening implementation range, known advisory result, and explicit remote-evidence boundary. The later Task 6 evidence-commit SHA is recorded in the ignored final-review report and user handoff because a committed file cannot contain its own commit SHA.

- [ ] **Step 1: Create an evidence skeleton before execution**

Create the record with sections `Scope`, `Environment`, `Focused Evidence`, `Full Ordered Acceptance`, `Advisory Result`, `Authorized-Scope Review`, `Whole-Range Review`, and `Remote Evidence Pending`. Use table columns `Command`, `Started UTC`, `Finished UTC`, `Exit`, `Result summary`. Do not write PASS until the command has actually completed; use `NOT RUN` only while executing and remove every such marker before commit.

```markdown
# Rust Wave 0 Final Hardening Verification

## Scope
Tested implementation head: NOT RUN
Implementation range: NOT RUN

## Environment
| Fact | Value |
| --- | --- |
| rustc/cargo/node/npm/cargo-deny digest | NOT RUN |

## Focused Evidence
| Command | Started UTC | Finished UTC | Exit | Result summary |
| --- | --- | --- | --- | --- |

## Full Ordered Acceptance
| Command | Started UTC | Finished UTC | Exit | Result summary |
| --- | --- | --- | --- | --- |

## Advisory Result
NOT RUN

## Authorized-Scope Review
NOT RUN

## Whole-Range Review
The tested implementation candidate is reviewed here; the final evidence-commit whole-range review is recorded in the ignored SDD report and handoff.

## Remote Evidence Pending
Rust Fast, Rust Full, advisory issue behavior, bundle smoke, MSI/NSIS, and hashes remain pending separate user authorization.
```

- [ ] **Step 2: Capture immutable environment and range facts**

```powershell
rustc -Vv
cargo -V
node --version
npm.cmd --version
git rev-parse HEAD
git rev-parse 77c739a321f08fc49d7a46d846d2a93a0dafc116
git rev-parse 9bcc047b9ec210c1ea34024f6dc782a1e7caec49
Get-FileHash -Algorithm SHA256 src-tauri/target/.extractum-tools/cargo-deny/0.20.2-x86_64-pc-windows-msvc/cargo-deny.exe
```

Expected: Rust 1.95.0 / Cargo companion version, the exact tested implementation head through Task 5, exact whole-range start `77c739a321f08fc49d7a46d846d2a93a0dafc116`, original Wave 0 reviewed boundary `9bcc047b9ec210c1ea34024f6dc782a1e7caec49`, and executable SHA `f7292fab58c706638c999e64c4ba82e5128ae628130ba55e3266a768ee431fbf`. Label HEAD `tested implementation head`; do not label it as the future Task 6 evidence commit. Record actual full outputs without secrets.

- [ ] **Step 3: Run the exact ordered acceptance sequence without reordering**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/rust-dependency-policy.test.ts scripts/rust-duplicate-baseline.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts scripts/testing/github-rust-workflows.test.ts
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-cargo-deny.test.ps1
npm.cmd run check:rust:duplicates -- --base 9bcc047b9ec210c1ea34024f6dc782a1e7caec49
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
npm.cmd run test:unit
npm.cmd run check
npm.cmd run bootstrap:testing
npm.cmd run check:rust:fast
npm.cmd run verify
npm.cmd run check:rust:advisories
git diff --check
```

Expected: commands 1-14 and 16 PASS. Record command 15 (`check:rust:advisories`) truthfully: it may be nonzero for the moving RustSec DB and remains a release blocker; list exact advisory IDs/packages observed and do not create an exception.

- [ ] **Step 4: Run factual and scope scans**

```powershell
rg -n "TODO|TBD|NOT RUN|pending local|cargo install cargo-deny" docs/superpowers/verification/2026-08-14-rust-wave-0-final-hardening.md README.md AGENTS.md docs/project.md
rg -n "prompt_pack_runtime_provider_default_remains_api_and_serializes_as_api|cancellation_smoke_services_remain_test_only" docs/superpowers/plans/2026-08-12-rust-infrastructure-wave-0.md docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md
git diff --name-only 77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD
git status --short
```

Expected: no unfinished/global-install marker; corrected prompt-pack name is present; changed files are within authorized scope plus this approved plan/spec/evidence; no generated lockfile or unrelated source appears.

- [ ] **Step 5: Review the uncommitted Task 6 evidence candidate**

Review the Task 6 evidence candidate for factual accuracy before committing it. Require every recorded command/result/timestamp to refer to the tested implementation head captured in Step 2, every deterministic result to match Step 3, the advisory result to remain truthful, and all remote fields to remain pending. Fix factual defects in the candidate before Step 7. This candidate review does not claim to be the final committed-head or whole-range review.

```powershell
git diff -- docs/superpowers/verification/2026-08-14-rust-wave-0-final-hardening.md
rg -n "NOT RUN|PASS|FAIL|pending|reviewed|HEAD" docs/superpowers/verification/2026-08-14-rust-wave-0-final-hardening.md
```

- [ ] **Step 6: Finalize the evidence record with no unsupported claim**

State exactly:

```text
Local readiness: ready to push for remote evidence only after all deterministic local gates and whole-range review pass.
Merge readiness: pending user-authorized push and green Rust Fast/Rust Full for the reviewed head.
Release readiness: blocked while advisories are red and until gated MSI, NSIS, application-smoke, sidecar-smoke, and hash evidence exists.
```

Record the hardening implementation range through the tested Task 5 head separately from the original reviewed range ending at `9bcc047b9ec210c1ea34024f6dc782a1e7caec49`. State that the evidence document itself will be committed afterward and that its commit SHA belongs to the post-commit review report/handoff, not to this self-referential record.

- [ ] **Step 7: Commit Task 6, then review the exact committed head and whole range**

```powershell
git add docs/superpowers/verification/2026-08-14-rust-wave-0-final-hardening.md
git commit -m "docs: verify Rust Wave 0 final hardening"
git status --short
git log --reverse --format='%H%x09%s' 77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD
$ExpectedSubjects = @(
  'build: bootstrap pinned cargo-deny locally',
  'test: harden Rust manifest and dependency policy',
  'test: make Rust duplicate policy diff-aware',
  'ci: isolate advisory writes and pin actions',
  'fix: close Rust Wave 0 minor findings',
  'docs: verify Rust Wave 0 final hardening'
)
$History = @(git log --reverse --format='%H%x09%s' 77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD)
$Cursor = -1
foreach ($ExpectedSubject in $ExpectedSubjects) {
  $Found = -1
  for ($Index = $Cursor + 1; $Index -lt $History.Count; $Index++) {
    $Parts = $History[$Index].Split("`t", 2)
    if ($Parts.Count -eq 2 -and $Parts[1] -eq $ExpectedSubject) { $Found = $Index; break }
  }
  if ($Found -lt 0) { throw "Missing or out-of-order correction commit: $ExpectedSubject" }
  $Cursor = $Found
}
git diff --check 77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD
```

Expected: clean worktree; the six named subjects occur in exact order across the full `77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD` history while adjacent correction commits are allowed; range diff-check passes. Immediately review the committed Task 6 diff for factual/spec accuracy, then review the exact complete range `77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD`, including the evidence commit, for spec compliance and code quality. Write the exact committed evidence SHA, exact reviewed range, reviewer identity/report path, and findings to the ignored final-review report and include the SHA in the user handoff; do not amend the evidence file to embed its own SHA. Any Important/Critical finding reopens the owning task and requires an adjacent correction, every focused command named in that task, the complete ordered acceptance sequence from Step 3, and a fresh committed-head review of `77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD`.

- [ ] **Step 8: Stop at the authorization boundary**

Do not push, create a PR, dispatch Actions, write advisory issues, build/publish a release, or claim remote Fast/Full/bundle evidence. Report the local reviewed head and request separate user authorization for remote execution.

```powershell
git status --short
git rev-parse HEAD
```

---

## Final Completion Checklist

- [ ] All six named commits exist in order and were not squashed.
- [ ] Every adjacent review correction was re-reviewed before continuing.
- [ ] Both explicit policy writers remain absent from checks, setup, acceptance aliases, and workflows.
- [ ] `git diff --check` passes and the final worktree is clean.
- [ ] The active Windows duplicate graph exactly equals the current baseline and explicit-base output is recorded.
- [ ] All Rust focused checks/package checkpoints and the full `npm.cmd run verify` pass.
- [ ] Advisory failure, if still present, is recorded as a release blocker rather than suppressed.
- [ ] Whole-range review has no Important or Critical findings.
- [ ] No remote or release action has been taken without new user authorization.
