# Rust Wave 0 Final Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish Rust Wave 0 with one authenticated cargo-deny runner, exact repository policy, minimal duplicate-growth review, least-privilege workflows, and bounded Rust/documentation corrections.

**Architecture:** Implement the approved simplified design at spec commit `719043f6`. PowerShell owns cargo-deny authentication and execution; JavaScript owns generated Cargo/npm facts and duplicate comparison; workflow tests parse YAML and assert only four security properties; two focused Rust tests preserve wire shape and secret ownership.

**Tech Stack:** PowerShell 5.1, Node.js ESM, TypeScript/Vitest, `yaml` 2.x, GitHub Actions YAML, Rust 1.95.0, Cargo, cargo-deny 0.20.2.

## Global Constraints

- Work from `G:\Develop\Extractum\.worktrees\rust-infrastructure-wave-0`; use `npm.cmd` on Windows.
- The controlling design is `docs/superpowers/specs/2026-08-14-rust-wave-0-final-hardening-design.md` at `719043f6`.
- Keep Rust `1.95.0`, MSRV `1.95`, Edition `2021`, cargo-deny `0.20.2`, Cargo dependencies, and `src-tauri/Cargo.lock` unchanged.
- The sole dependency exception is direct devDependency `yaml` major 2 plus its required `package-lock.json` entry; no existing npm package may move.
- `check:rust:fast` remains the existing four-part composition: fmt, Clippy, two feature-off checks, deterministic cargo-deny. Duplicate checking remains separate.
- Do not add a TOML parser, handwritten YAML parser, YAML mirror, Action-SHA map, duplicate classifications/modes/clocks/expiry/lifecycle automation, or `scripts/run-cargo-deny.ps1`.
- Writers are explicit developer operations only; tests, aliases, and CI never call `--write`.
- Workflow tests own exactly four properties: generic full-40-hex remote pins, scanner permission, writer permission/wiring, and successful-scanner bundle gating.
- Rust Full/Release duplicate presence is review-only, not a fifth workflow-test property. Rust Full is unconditional once; Release `windows-bundle` is once; advisory jobs and advisory-only cron are zero.
- Preserve product behavior, IPC, JSON, persisted data, migrations, UI, and secret boundaries.
- No per-task code review. Review Tasks 1-4 together, then run acceptance, commit evidence, and review the whole range.
- Stop before push, PR, workflow dispatch, bundle, publication, or release.

---

## File Map and Interfaces

| Task | Exact responsibility/files |
| --- | --- |
| 1 | Rename `scripts/setup-cargo-deny.ps1` → `scripts/cargo-deny.ps1`; create `scripts/cargo-deny.test.ps1`; modify `.github/tools/cargo-deny.json`, `.github/actions/setup-cargo-deny/action.yml`, `package.json`, `README.md`, `AGENTS.md`, `docs/project.md`. |
| 2 | Create `scripts/rust-dependency-policy.mjs`/`.test.ts`; modify both existing `scripts/rust-duplicate-baseline.*`, three policy JSON artifacts, `scripts/testing/repository-rules.mjs`/`.test.ts`, and `package.json`; verify the unchanged exact registry in `scripts/testing/test-conventions.test.ts` (it may have zero diff). |
| 3 | Modify `package.json`, `package-lock.json`, three Rust workflows, and `scripts/testing/github-rust-workflows.test.ts` for duplicate placement, `yaml`, pins, and advisory isolation. |
| 4 | Modify `package.json`, `scripts/verify.test.ts`, `deny.toml`, repository-rule tests, the two named Rust sources, registry/operator docs, and the prior Wave 0 plan/verification; final acceptance creates only `docs/superpowers/verification/2026-08-15-rust-wave-0-final-hardening.md`. |

Public interfaces are fixed here and implemented concretely in their owning tasks:

```powershell
scripts/cargo-deny.ps1 [-Mode] <Setup|Deterministic|Advisories> [-AddToGitHubPath]
```

```js
export function cargoRequirementIdentity(entry) // string
export function npmRequirementIdentity(entry) // string
export function generateRustDependencyPolicy({ metadata, packageJson, reviewed }) // schema-2 object
export function validateRustDependencyPolicy({ generated, committed }) // string[]
export function generateRustDuplicateBaseline(treeText) // schema-1 object
export function validateDuplicateBaseline(value) // normalized object; throws
export function validateCurrentDuplicateState({ treeText, baseline, exceptions }) // { current, baseline, exceptions, violations: string[] }
export function compareDuplicateGrowth({ current, base, exceptions }) // string[] in stable package order
export function writeRustDuplicateBaseline({ treeText, path }) // returned/written schema-1 object
export function checkRustDuplicatePolicy({ base, cwd, treeText, stderr }) // { historicalSkipped: boolean }
```

Exception-schema validation is a private helper exercised through current-state validation and
the check CLI. The developer-only writer calls `generateRustDuplicateBaseline`, validates the
result, writes the requested file, and returns the same baseline. Cargo identity is
`package/dependency/rename/kind/target`; npm identity is `owner/name/kind`. Duplicate checking
always proves current equality before optional history. Workflow tests use `parse` from `yaml`
and own only the four named security properties. Task 4's private Rust enum is defined with its
complete implementation.

---

## Rust Verification Loops

Affected packages are `extractum-gemini-browser` and application package `extractum`. No public cross-crate Rust interface changes.

- Gemini Browser exact RED/GREEN:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact`
- Gemini Browser focused check:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked`
- Gemini Browser checkpoint:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked`
- Application exact RED/GREEN:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact`
- Application focused check:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked`
- Application checkpoint:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked`
- End-of-slice workspace gates: `npm.cmd run check:rust:fast` and `npm.cmd run verify` after `npm.cmd run bootstrap:testing`.
- If an exact filter reports zero tests, list package tests and correct the full name; zero tests is a failure, not evidence.

---

## Implementation Base Capture

- [ ] **Capture the reviewed plan HEAD before Task 1**

Persist the starting commit outside tracked scope so later sessions review only implementation commits:

```powershell
$ImplementationBase = (git rev-parse HEAD).Trim()
git rev-parse --verify ($ImplementationBase + "^{commit}")
New-Item -ItemType Directory -Force .superpowers/sdd | Out-Null
Set-Content -LiteralPath .superpowers/sdd/rust-wave-0-final-hardening-implementation-base.txt -Value $ImplementationBase -NoNewline
Get-Content .superpowers/sdd/rust-wave-0-final-hardening-implementation-base.txt
```

Expected: the printed SHA is the reviewed replacement-plan HEAD and the file remains ignored. Do not recapture it after Task 1 starts.

---

### Task 1: Build — Consolidate the Pinned cargo-deny Runner

**Files:**
- Rename `scripts/setup-cargo-deny.ps1` → `scripts/cargo-deny.ps1`; create `scripts/cargo-deny.test.ps1`; modify `.github/tools/cargo-deny.json`, `.github/actions/setup-cargo-deny/action.yml`, `package.json`, `README.md`, `AGENTS.md`, `docs/project.md`.

**Interfaces:**
- Consumes: schema-1 tool manifest, repository root, `GITHUB_PATH`, network only on cold cache.
- Produces: authenticated stable executable at `src-tauri/target/.extractum-tools/cargo-deny/<version>-<target>/cargo-deny.exe` and the three-mode CLI defined above.

- [ ] **Step 1: Add the behavioral child-process harness and RED cases**

Create a unique temp repository fixture and invoke the runner in a child so one failing case cannot exit the harness. Keep these test interfaces exact:

```powershell
Invoke-Runner -Runner <path> -Arguments <string[]> -Environment <hashtable>
# -> [pscustomobject]@{ ExitCode; Stdout; Stderr }
New-Fixture [-Version <string>]
# -> @{ Root; Runner; Archive; Log; GitHubPath; Stable }
```

`Invoke-Runner` launches `powershell.exe -NoProfile -ExecutionPolicy Bypass -File`, captures stdout/stderr/exit independently, and restores every process environment value in `finally`. The fake executable logs argv, implements only `--version`, and returns `EXTRACTUM_DENY_EXIT`.

Required public test environment contract:

| Variable | Meaning |
| --- | --- |
| `EXTRACTUM_CARGO_DENY_TESTING=1` | Enables test-only seams. |
| `EXTRACTUM_CARGO_DENY_TEST_ARCHIVE` | Supplies the local archive instead of network. |
| `EXTRACTUM_DENY_LOG` | Records fake executable argv. |
| `GITHUB_PATH` | Captures Setup publication. |

The first-install seam is `EXTRACTUM_CARGO_DENY_TEST_FAIL_PRE_PUBLISH=1`; it is active only with
`EXTRACTUM_CARGO_DENY_TESTING=1` and fires after candidate digest/version authentication but before
any stable move. The rollback seam separately uses `EXTRACTUM_CARGO_DENY_TEST_FORCE_REINSTALL=1`
and `EXTRACTUM_CARGO_DENY_TEST_FAIL_POST_PUBLISH=1`; none is a production mode.

| Case | Mutation/action | Exact evidence |
| --- | --- | --- |
| cold install | Valid archive, empty cache | Exit 0; authenticated stable binary published. |
| warm install | Remove archive after cold Setup | Exit 0 without archive/network; same stable binary reused. |
| archive digest | Replace manifest archive SHA | Nonzero; stable absent; fake never executes. |
| executable digest | Replace binary SHA | Nonzero; stable absent; fake never executes. |
| version mismatch | Fake reports `0.20.1` | Nonzero; exactly one `--version` probe; stable absent. |
| first-install failure | Set `EXTRACTUM_CARGO_DENY_TEST_FAIL_PRE_PUBLISH=1` | Nonzero after candidate authentication; stage is cleaned, no stable/backup remains, and PATH/sinks are restored. |
| rollback | Force reinstall and fail after publication | Candidate removed; previous stable digest restored byte-for-byte. |
| PATH restoration | Give child a custom process PATH | Parent process PATH equals its pre-case value after invocation. |
| GITHUB_PATH once | Cold then warm Setup with `-AddToGitHubPath` | One normalized stable-directory line total. |
| ambient exclusion | Put valid fake `cargo-deny.exe` on PATH while archive SHA is invalid | Nonzero; ambient fake never executes. |
| exit propagation | Fake deterministic execution returns 23 | Wrapper exits 23; advisory success exits 0; `-AddToGitHubPath` outside Setup is nonzero. |

The argv log must match absolute repository paths and root-level locked ordering: `--config <deny.toml> --manifest-path <src-tauri/Cargo.toml> --locked check bans licenses sources` or `... --locked check advisories`.

- [ ] **Step 2: Run the harness and verify RED**

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/cargo-deny.test.ps1
```

Expected: nonzero because `scripts/cargo-deny.ps1` and `binarySha256` do not exist.

- [ ] **Step 3: Rename and implement the single production runner**

Implement one production script with these private helpers and no PATH lookup:

```powershell
Test-Digest([string]$Path, [string]$Expected) -> bool
Test-AuthenticatedBinary([string]$Path, [pscustomobject]$Manifest) -> bool
Copy-OrDownloadArchive([pscustomobject]$Manifest, [string]$Destination) -> void
Get-AuthenticatedCargoDeny([pscustomobject]$Manifest, [string]$StableBinary) -> absolute path
Add-GitHubPathOnce([string]$Path) -> void
```

1. Validate schema/version/target plus lowercase 64-hex archive and binary digests before cache work.
2. A warm hit is valid only after binary digest verification and an exact `cargo-deny 0.20.2` probe; authenticate before every execution.
3. A cold install downloads/copies to a unique stage, verifies archive digest, extracts, verifies binary digest/version, then publishes by directory move. The pre-publish test seam fires only after authentication; its failure cleans the stage, leaves no stable directory, and restores PATH/sinks.
4. If a previous stable directory exists, move it to backup first. Any failure after that removes a published candidate and restores the backup; successful backup cleanup is non-fatal.
5. Test archive/fault seams require `EXTRACTUM_CARGO_DENY_TESTING=1`; production always uses the manifest URL.
6. Setup may append the normalized stable directory to `GITHUB_PATH` exactly once and never mutates process PATH. Other modes reject `-AddToGitHubPath`.
7. Deterministic executes absolute `deny.toml`/manifest paths with `--locked check bans licenses sources`; Advisories executes the same prefix with `check advisories`.
8. Return the authenticated binary's exact exit code. Setup returns 0 only after authentication/publication.

The dispatch arrays are exactly:

```powershell
$Common = @('--config', $DenyToml, '--manifest-path', $CargoToml, '--locked', 'check')
$DeterministicChecks = @('bans', 'licenses', 'sources')
$AdvisoryChecks = @('advisories')
```

The test-only archive override is accepted only when `EXTRACTUM_CARGO_DENY_TESTING=1`; production always downloads the manifest URL. The code never consults PATH and `Add-GitHubPathOnce` never mutates process PATH.

Add to `.github/tools/cargo-deny.json`:

```json
"binarySha256": "f7292fab58c706638c999e64c4ba82e5128ae628130ba55e3266a768ee431fbf"
```

- [ ] **Step 4: Rewire aliases, composite Setup, and factual docs**

Set exact scripts without changing fast composition:

```json
"check:rust:supply-chain": "powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/cargo-deny.ps1 -Mode Deterministic",
"check:rust:advisories": "powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/cargo-deny.ps1 -Mode Advisories",
"check:rust:fast": "npm run check:rustfmt && npm run check:rust:clippy && npm run check:rust:production && npm run check:rust:supply-chain"
```

Composite step:

```yaml
- name: Authenticate pinned cargo-deny
  shell: pwsh
  run: ./scripts/cargo-deny.ps1 -Mode Setup -AddToGitHubPath
```

Docs must name `scripts/cargo-deny.ps1`, the three modes, stable repository cache, and no global install. Remove old setup-script/operator commands.

- [ ] **Step 5: Run GREEN checks**

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/cargo-deny.test.ps1
npm.cmd run check:rust:supply-chain
rg -n "setup-cargo-deny\.ps1|run-cargo-deny\.ps1|cargo install cargo-deny" README.md AGENTS.md docs/project.md package.json .github scripts
```

Expected: harness PASS; deterministic policy PASS; search returns no obsolete production invocation (the composite directory name may remain).

- [ ] **Step 6: Commit Task 1**

```powershell
git add scripts/cargo-deny.ps1 scripts/cargo-deny.test.ps1 scripts/setup-cargo-deny.ps1 .github/tools/cargo-deny.json .github/actions/setup-cargo-deny/action.yml package.json README.md AGENTS.md docs/project.md
git commit -m "build: consolidate pinned cargo-deny runner"
```

Expected: one rename/replacement, one behavioral test, no `yaml` or workflow-security changes.

---

### Task 2: Test — Harden Rust Dependency and Duplicate Policy

**Files:**
- Create `scripts/rust-dependency-policy.mjs`/`.test.ts`; modify both existing `scripts/rust-duplicate-baseline.*`, three policy JSON artifacts, `scripts/testing/repository-rules.mjs`/`.test.ts`, and `package.json`; verify `scripts/testing/test-conventions.test.ts` and modify it only if its already-exact registry assertion needs formatting, not a new ID.

**Interfaces:**
- Consumes: locked Cargo metadata/tree, seven manifest texts, `package.json`, reviewed schema-2 Tauri records, optional Git base.
- Produces: exact bidirectional live repository policy, current duplicate equality, and optional positive-growth approval.

The dependency generator obtains facts with:

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
```

- [ ] **Step 1: Seed schema 2 manually and write dependency-policy RED tests**

Replace schema 1 with generated inventory arrays plus these reviewed records; these values live only in JSON, never as code constants:

```json
"tauriFamily": {
  "pairs": [
    {
      "id": "tauri-api",
      "cargo": { "package": "extractum", "dependency": "tauri", "rename": null, "kind": "normal", "target": null },
      "cargoRequirement": "^2",
      "npm": { "owner": "extractum", "name": "@tauri-apps/api", "kind": "dependencies" },
      "npmRequirement": "^2"
    },
    {
      "id": "tauri-build-cli",
      "cargo": { "package": "extractum", "dependency": "tauri-build", "rename": null, "kind": "build", "target": null },
      "cargoRequirement": "^2",
      "npm": { "owner": "extractum", "name": "@tauri-apps/cli", "kind": "devDependencies" },
      "npmRequirement": "^2"
    },
    {
      "id": "tauri-dialog",
      "cargo": { "package": "extractum", "dependency": "tauri-plugin-dialog", "rename": null, "kind": "normal", "target": null },
      "cargoRequirement": "^2",
      "npm": { "owner": "extractum", "name": "@tauri-apps/plugin-dialog", "kind": "dependencies" },
      "npmRequirement": "^2"
    },
    {
      "id": "tauri-opener",
      "cargo": { "package": "extractum", "dependency": "tauri-plugin-opener", "rename": null, "kind": "normal", "target": null },
      "cargoRequirement": "^2",
      "npm": { "owner": "extractum", "name": "@tauri-apps/plugin-opener", "kind": "dependencies" },
      "npmRequirement": "^2"
    },
    {
      "id": "tauri-sql",
      "cargo": { "package": "extractum", "dependency": "tauri-plugin-sql", "rename": null, "kind": "normal", "target": null },
      "cargoRequirement": "^2",
      "npm": { "owner": "extractum", "name": "@tauri-apps/plugin-sql", "kind": "dependencies" },
      "npmRequirement": "^2.4.0"
    }
  ],
  "cargoOnlyRequirements": [
    { "id": "tauri-plugin-mcp-bridge", "cargo": { "package": "extractum", "dependency": "tauri-plugin-mcp-bridge", "rename": null, "kind": "normal", "target": null }, "cargoRequirement": "^0.11" }
  ]
}
```

Use the reviewed JSON above to generate one complete synthetic metadata/package fixture. The named helper positive is synthetic coverage only; live equality belongs to the repository rule in Step 4.

| Named case | Mutation | Exact expectation |
| --- | --- | --- |
| helper positive | Generated policy vs same reviewed authority | `[]`. |
| requirement drift | First pair Cargo requirement → `^3` | Nonempty violations. |
| moved npm kind | First pair npm kind → `devDependencies` | Nonempty violations. |
| orphan identity | First pair Cargo dependency → `missing` | Nonempty violations. |
| missing pair | Remove first pair | Nonempty violations. |
| widened Cargo-only | MCP requirement → `>=0.11` | Nonempty violations. |
| writer preservation | Regenerate from reviewed schema 2 | `tauriFamily` remains byte-for-byte equal. |
| malformed nested | Cargo identity → `null` | Violation, never `TypeError`. |
| wrong container/scalar/kind | Pair array/object/string/kind mutation | Violation, never `TypeError`. |
| duplicate pair ID/Cargo identity | Copy first into second | Violation, never `TypeError`. |
| unsorted Cargo-only IDs | Prepend cloned row with ID `zzz` | Violation, never `TypeError`. |
| raw schema 1 | Pass old policy | Includes `schemaVersion must be exactly 2`. |

- [ ] **Step 2: Run dependency RED**

```powershell
npm.cmd run test:unit -- scripts/rust-dependency-policy.test.ts scripts/testing/repository-rules.test.ts
```

Expected: FAIL because the generator and schema-2 validators are absent.

- [ ] **Step 3: Implement generation and generic bijection validation**

Implement these contracts without a second Tauri map:

```js
export function cargoRequirementIdentity(entry) { /* package/dependency/rename/kind/target */ }
export function npmRequirementIdentity(entry) { /* owner/name/kind */ }
export function generateRustDependencyPolicy({ metadata, packageJson, reviewed }) { /* schema 2 */ }
export function validateRustDependencyPolicy({ generated, committed }) { /* string[] */ }
```

Exactly six non-obvious rules define the implementation:

1. Cargo identity is the JSON-safe tuple `(package, dependency, rename, kind, target)`; npm identity is `(owner, name, kind)`.
2. Every generated inventory and both reviewed ID arrays are unique and sorted by their identity/ID key; malformed containers or fields add violations before any identity helper is called.
3. `tauriFamily.pairs` is a bijection onto all governed Cargo Tauri identities except the MCP bridge and all `@tauri-apps/*` npm identities; `cargoOnlyRequirements` is a bijection onto the MCP bridge identity.
4. Cargo metadata `kind: null` normalizes only to policy `kind: "normal"`; `build` and `dev` remain distinct, so moving `tauri-build` is drift.
5. Generation copies only reviewed `toolchain` and `tauriFamily`; it never invents, renames, retargets, widens, or otherwise infers an approval.
6. A direct requirement with more than one distinct prerelease token is ambiguous and rejected; a single token produces the canonical `approvedPrereleases` fact.

The developer writer validates the reviewed-authority containers/fields first, generates from live Cargo/npm facts, and writes the result. It must not compare `generated` to itself:

```js
const reviewed = JSON.parse(readFileSync(outputPath, "utf8"));
const authorityErrors = validateReviewedAuthority(reviewed);
if (authorityErrors.length) throw new Error(authorityErrors.join("\n"));
const generated = generateRustDependencyPolicy({ metadata, packageJson, reviewed });
writeFileSync(outputPath, `${JSON.stringify(generated, null, 2)}\n`);
```

`validateReviewedAuthority` is private and checks only the reviewed `toolchain`/`tauriFamily` shape needed before generation. Generated-versus-committed equality is deliberately enforced by the live rule, not the writer.

- [ ] **Step 4: Implement anchored manifest and metadata repository rules**

Normalize CRLF and count lines without parsing TOML:

```js
const exactCount = (source, expression) => normalizeText(source).split("\n").filter((line) => expression.test(line)).length;
for (const manifest of sevenManifestPaths) {
  requireCount(manifest, /^edition\.workspace = true$/, 1);
  requireCount(manifest, /^rust-version\.workspace = true$/, 1);
}
for (const manifest of sixMemberManifestPaths) {
  rejectMatch(manifest, /^edition\s*=/m);
  rejectMatch(manifest, /^rust-version\s*=/m);
}
requireCount("src-tauri/Cargo.toml", /^edition = "2021"$/, 1);
requireCount("src-tauri/Cargo.toml", /^rust-version = "1\.95"$/, 1);
```

Wire generated-versus-committed equality into the live evaluator:

```js
import {
  generateRustDependencyPolicy,
  validateRustDependencyPolicy,
} from "../rust-dependency-policy.mjs";

function evaluateRustDependencyPolicy(index) {
  const committed = index.getJson(RUST_DEPENDENCY_POLICY_PATH);
  const generated = generateRustDependencyPolicy({
    metadata: index.getCargoMetadata(),
    packageJson: index.getJson("package.json"),
    reviewed: committed,
  });
  const violations = [];
  violations.push(...validateRustDependencyPolicy({ generated, committed }));
  return violations;
}
```

Keep the actual-repository positive. For mutations, construct a real `RepositoryIndex` whose Cargo metadata, `package.json`, and committed policy are deep clones of the live index; override only those three reads and let every other file stay real. Before each mutation, the identical factory must evaluate exactly `{ id: "rule:rust-dependency-policy", violations: [] }`.

| Mutation | Fact change | Required violations |
| --- | --- | --- |
| missing | Remove `tauri-plugin-dialog` | `directRequirements drifted`; `paired Cargo bijection drifted`. |
| extra | Add `tauri-plugin-extra ^2` | Same two violations. |
| duplicate | Duplicate the dialog dependency | Same two violations. |
| moved kind | `tauri-build`: build → normal | Same two violations. |
| renamed | Dialog rename → `dialog-renamed` | Same two violations. |
| retargeted | Dialog target → `cfg(windows)` | Same two violations. |
| requirement drift | Dialog requirement → `^3` | `directRequirements drifted`; `tauri-dialog: Cargo requirement drifted`. |

Never weaken these to generic nonempty/non-`INFRA_ERROR` assertions.

Metadata validation compares the exact sorted names from `policy.toolchain.workspacePackages`, each `rust_version` to `policy.toolchain.rustVersion`, each `edition` to `policy.toolchain.edition`, and requires `Array.isArray(publish) && publish.length === 0`. Build the complete expected `rust-toolchain.toml` from the JSON channel/target plus fixed components/profile below; do not copy the JSON values into a second code map.

Replace/harden the bodies of the three existing evaluators `rule:rust-toolchain-policy`, `rule:rust-dependency-policy`, and `rule:rust-duplicate-baseline`; do not register a new rule ID. `registeredRuleIds` stays unchanged. Verify the exact existing list in `test-conventions.test.ts`; that file may correctly have zero diff. Step 7 supplies the shared strict-current body used by the duplicate evaluator.

```toml
[toolchain]
channel = "1.95.0"
components = ["rustfmt", "clippy"]
targets = ["x86_64-pc-windows-msvc"]
profile = "minimal"
```

- [ ] **Step 5: Write duplicate current/base RED tests**

Drive both the shared helper/CLI and existing repository rule with the same canonical baseline/tree builders. Repository fixtures inject `loadCargoTree` plus committed baseline/exceptions JSON through a real `createRepositoryIndex`; no test expects a `review` property.

Strict-current parity matrix:

| Case | Committed cardinality | Live cardinality | Helper/CLI/rule expectation |
| --- | --- | --- | --- |
| equality | `{ alpha: 2 }` | `{ alpha: 2 }` | `[]`; CLI success; exact rule `{ id, violations: [] }`. |
| stale reduction | `{ alpha: 3 }` | `{ alpha: 2 }` | `current duplicate graph differs from committed baseline`. |
| growth | `{ alpha: 2 }` | `{ alpha: 3 }` | Same violation. |
| same-aggregate name replacement | `{ alpha: 2 }` | `{ beta: 2 }` | Same violation. |
| per-name third version, same aggregate | `{ alpha: 2, beta: 3 }` | `{ alpha: 3, beta: 2 }` | Same violation. |

A schema-valid unused exception such as `unused-upstream: 1 → 2` must not create a current-state violation.

Historical growth matrix (`compareDuplicateGrowth` returns arrays, never throws first):

| Case | Base → current | Exceptions | Exact violations |
| --- | --- | --- | --- |
| existing growth unapproved | `getrandom 2 → 3` | none | `getrandom: duplicate growth 2 -> 3 requires exact approval`. |
| existing growth approved | `getrandom 2 → 3` | exact `2 → 3` | `[]`. |
| new name unapproved | absent → `syn 2` | none | `syn: duplicate growth 1 -> 2 requires exact approval`. |
| new name approved | absent → `syn 2` | exact `1 → 2` | `[]`. |
| decrease/equality | `3 → 2` / `2 → 2` | none | `[]`. |
| two simultaneous | `alpha 2 → 3`, `beta 2 → 3` | none | Alpha diagnostic, then beta diagnostic. |

CLI/base matrix:

| Case | Exact result |
| --- | --- |
| invalid revision | Nonzero; contains `revision`. |
| both historical artifacts absent | Success; explicit historical-skip notice. |
| either one absent | Success; same notice. |
| malformed present historical JSON | Nonzero; JSON error. |
| current mismatch before absent-history skip | Nonzero current-drift error. |
| both present | Success, comparison performed, no skip notice. |
| writer | Import exported `writeRustDuplicateBaseline`; written JSON equals its returned baseline and records `{ syn: 2 }`. |

Add direct baseline schema/counter/order tests. Malformed exception containers/fields and duplicate
package entries go through `validateCurrentDuplicateState` or current check and fail schema, never a
separately exported validator. Current-only validation accepts a schema-valid count-mismatched
approval as unused; test it through `compareDuplicateGrowth` or base-aware CLI against matching
observed growth, where it yields the exact approval-required diagnostic. Every temp Git fixture
commits an unrelated seed so an artifact-free base commit exists, and cleanup is bounded to its
`mkdtempSync` directory.
These RED tests replace old compatible-replacement/package-set-review expectations; no test may expect a `{ review: ... }` result.

- [ ] **Step 6: Run duplicate RED**

```powershell
npm.cmd run test:unit -- scripts/rust-duplicate-baseline.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
```

Expected: FAIL because the shared strict-current validator/parity is absent and the old repository rule still accepts reductions or emits review-only replacement results.

- [ ] **Step 7: Implement shared strict-current validation and the minimal duplicate CLI**

Export exactly these six functions. Exception-schema validation stays private and is exercised through `validateCurrentDuplicateState` and `checkRustDuplicatePolicy`:

```js
export function generateRustDuplicateBaseline(treeText) -> schema-1 baseline
export function validateDuplicateBaseline(value) -> validated baseline; throws on schema/counter/order drift
export function validateCurrentDuplicateState({ treeText, baseline, exceptions }) -> { current, baseline, exceptions, violations }
export function compareDuplicateGrowth({ current, base, exceptions }) -> string[]
export function writeRustDuplicateBaseline({ treeText, path }) -> returned/written schema-1 baseline
export function checkRustDuplicatePolicy({ base, cwd, treeText, stderr }) -> { historicalSkipped }
```

1. Generate unique `name@version` values from locked `cargo tree`; keep only names with more than one version, sorted by name, with exact derived counters.
2. Strict current equality compares the entire validated generated baseline with the committed baseline. Schema, counts, cardinality, name-set growth, reductions, and replacements all drift. Validate exception schema but do not reject unused entries.
3. For an explicit base, resolve `${base}^{commit}`. Use `git cat-file -e <commit>:<path>` only to distinguish absence, and `git show` to read present content. If either historical artifact is absent, emit the exact skip notice after current equality succeeds.
4. Historical growth uses current exceptions, not base exceptions. A name absent from the base has `previousCount = 1`; an approval matches only exact package/previous/approved counts with nonempty owner/reason/reviewAfter.
5. Accumulate every unapproved or mismatched growth in stable package-name order. `compareDuplicateGrowth` returns the full `string[]`; CLI throws one newline-joined error only after collection.
6. CLI grammar is exactly `--write <path>` or `--check [--base SHA]`. The writer regenerates only the requested baseline path. Parse `process.argv.slice(2)` only inside the direct-module guard; imports never execute CLI logic. There is no env mode, merge-base, implicit history, clock, expiry, or unused-exception enforcement.

The locked tree argv remains:

```js
const CARGO_TREE_ARGS = [
  "tree", "--manifest-path", "src-tauri/Cargo.toml", "--locked",
  "--target", "x86_64-pc-windows-msvc", "--workspace",
  "--prefix", "none", "--format", "{p}",
];
```

Replace the old ceiling/review evaluator body in `repository-rules.mjs`; it consumes the same returned violations directly. Add the supply-chain path constant exactly once beside the existing `RUST_DUPLICATE_BASELINE_PATH` and other production path constants (the same spelling in the test snippet is test-file-local, not a second production declaration):

```js
import { validateCurrentDuplicateState } from "../rust-duplicate-baseline.mjs";

const RUST_SUPPLY_CHAIN_EXCEPTIONS_PATH = "scripts/testing/rust-supply-chain-exceptions.json";

function evaluateRustDuplicateBaseline(index) {
  const state = validateCurrentDuplicateState({
    treeText: index.getCargoTree(),
    baseline: index.getJson(RUST_DUPLICATE_BASELINE_PATH),
    exceptions: index.getJson(RUST_SUPPLY_CHAIN_EXCEPTIONS_PATH),
  });
  return state.violations;
}
```

Delete the old greater-than count checks, added/removed-name review calculation, and `{ review: ... }` result. Exact equality covers schema, counts, cardinalities, and the duplicate-name set in both directions: reductions, growth, replacements, and third versions all fail until the committed baseline is explicitly regenerated.

The `cat-file` probe distinguishes an absent path from malformed present JSON; content is always read with `git show`. There is no merge-base/env/default history.

Add the separate alias without changing `check:rust:fast`:

```json
"check:rust:duplicates": "node scripts/rust-duplicate-baseline.mjs --check"
```

- [ ] **Step 8: Regenerate reviewed artifacts and run policy GREEN**

```powershell
node scripts/rust-dependency-policy.mjs --write scripts/testing/rust-dependency-policy.json
node scripts/rust-duplicate-baseline.mjs --write scripts/testing/rust-duplicate-baseline.json
npm.cmd run test:unit -- scripts/rust-dependency-policy.test.ts scripts/rust-duplicate-baseline.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
npm.cmd run check:rust:duplicates
npm.cmd run check:rust:duplicates -- --base 9bcc047b9ec210c1ea34024f6dc782a1e7caec49
```

Expected: all tests PASS; helper, repository rule, and CLI agree on strict current equality, with no review-only result. The fixed base `9bcc047b9ec210c1ea34024f6dc782a1e7caec49` resolves, contains both artifacts, performs the historical comparison, and emits no skip notice; any skip for this command is a failure.

- [ ] **Step 9: Commit Task 2**

```powershell
git add scripts/rust-dependency-policy.mjs scripts/rust-dependency-policy.test.ts scripts/rust-duplicate-baseline.mjs scripts/rust-duplicate-baseline.test.ts scripts/testing/rust-dependency-policy.json scripts/testing/rust-duplicate-baseline.json scripts/testing/rust-supply-chain-exceptions.json scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts package.json
git commit -m "test: harden Rust dependency and duplicate policy"
```

Expected: commit has policy/duplicate code and the package alias only; `test-conventions.test.ts` may have zero diff because no rule ID changed. No workflow YAML, `yaml` dependency, Rust source, or Cargo lock changes.

---

### Task 3: CI — Isolate Advisory Writes and Pin Actions

**Files:**
- Modify `package.json`, `package-lock.json`, `.github/workflows/rust-fast.yml`, `.github/workflows/rust-full.yml`, `.github/workflows/rust-release.yml`, and `scripts/testing/github-rust-workflows.test.ts`.

**Interfaces:**
- Consumes: YAML workflows, Task 1 cargo-deny aliases, and Task 2 duplicate alias/current-base contract.
- Produces: exact reviewed duplicate placement plus parsed four-property security contract and scan/classify/fail/writer isolation.

- [ ] **Step 1: Add exact workflow duplicate invocations**

Edit the three workflow files once in this task. Use these Rust Fast steps; do not add duplicate placement to the four-property workflow test:

```yaml
- name: Fetch duplicate-policy base
  if: github.event_name == 'pull_request'
  shell: pwsh
  run: git fetch --no-tags --depth=1 origin ${{ github.event.pull_request.base.sha }}
- name: Check duplicate growth from PR base
  if: github.event_name == 'pull_request'
  run: npm.cmd run check:rust:duplicates -- --base ${{ github.event.pull_request.base.sha }}
- name: Check current duplicate policy on main
  if: github.event_name == 'push'
  run: npm.cmd run check:rust:duplicates
```

Add one unconditional current-only invocation to Rust Full and one current-only invocation only to Release `windows-bundle`. Advisory scan/writer get none, and scheduled cron without a bundle remains advisory-only. Step order relative to `check:rust:fast` is unregulated.

```powershell
rg -n "check:rust:duplicates|pull_request.base.sha|git fetch" .github/workflows/rust-fast.yml .github/workflows/rust-full.yml .github/workflows/rust-release.yml
```

Expected by static review: PR base-aware once, main current-only once, Full unconditional once, `windows-bundle` once, advisory jobs zero. This is review evidence, not a fifth automated workflow property.

- [ ] **Step 2: Add the sole npm dependency and prove bounded lock scope**

```powershell
npm.cmd install --save-dev yaml@^2
git diff -- package.json package-lock.json
```

Expected: root `devDependencies.yaml` plus its resolved lock entry only; no existing package version changes. If npm moves another package, restore only those unrelated lock changes before continuing.

- [ ] **Step 3: Replace workflow tests with four structural RED properties**

Parse all three files structurally with `parse` from `yaml`. Tests own exactly four properties and no runner/concurrency/order/fetch/duplicate-placement details:

| Named property | Positive contract | Single mutation | Exact failure |
| --- | --- | --- | --- |
| remote pins | Every nonlocal `uses` matches `^[^@\s]+@[0-9a-f]{40}$` | First remote ref → `actions/checkout@v6` | Contains that mutable ref. |
| scanner permission | `advisory-scan.permissions` exactly `{ contents: read }` | Add `issues: write` | `scanner permissions`. |
| writer outcome isolation | Exact deny/classify/output/fail shells and expressions; writer alone has issues-write and explicit outcome condition | Append a second classify output command | `writer outcome wiring`. |
| bundle scanner success | Bundle needs scanner, condition includes scan success, excludes `always()`/`!cancelled()` | Replace condition with `always()` | `bundle scanner success`. |

Recursive `uses` collection is generic; do not add an Action-SHA map or parsed workflow mirror.

These are the complete four named tests/mutations. Do not assert or mutate runner names, concurrency, step ordering, fetch, duplicate placement, or other workflow details.

- [ ] **Step 4: Run workflow RED**

```powershell
npm.cmd run test:unit -- scripts/testing/github-rust-workflows.test.ts
```

Expected: FAIL on mutable Action refs, mixed scanner/writer permissions, old `failure()` writer, and old advisory job structure.

- [ ] **Step 5: Pin every external Action without a code-side SHA map**

Replace refs in the three workflow files:

```yaml
uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803
uses: actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38
uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
uses: actions/github-script@ed597411d8f924073f98dfc5c65a23a2325f34cd
uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6
```

The generic regex test owns pin shape only. Do not encode these exact values in a test constant/map or duplicate entire workflow objects.

- [ ] **Step 6: Implement exact advisory scan/classify/fail and writer jobs**

Rename scanner job `advisory-scan`, keep Windows and `contents: read`, then use:

```yaml
outputs:
  advisories_failed: ${{ steps.classify.outputs.advisories_failed }}
steps:
  - uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803
  - run: rustup show active-toolchain
  - uses: ./.github/actions/setup-cargo-deny
  - id: deny
    continue-on-error: true
    run: npm.cmd run check:rust:advisories
  - id: classify
    if: always()
    shell: pwsh
    run: |
      "advisories_failed=${{ steps.deny.outcome == 'failure' }}" >> $env:GITHUB_OUTPUT
  - name: Fail when advisory scan failed
    if: ${{ steps.deny.outcome == 'failure' }}
    shell: pwsh
    run: exit 1
```

Writer job:

```yaml
advisory-issue-writer:
  needs: advisory-scan
  if: ${{ always() && needs.advisory-scan.outputs.advisories_failed == 'true' && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}
  permissions:
    issues: write
  runs-on: ubuntu-latest
  concurrency:
    group: rust-advisory-follow-up
    cancel-in-progress: false
  steps:
    - uses: actions/github-script@ed597411d8f924073f98dfc5c65a23a2325f34cd
      with:
        script: |
          const title = "Security: Rust advisory follow-up";
          const body = `npm.cmd run check:rust:advisories failed.\n\nRun: ${context.serverUrl}/${context.repo.owner}/${context.repo.repo}/actions/runs/${context.runId}`;
          const issues = await github.paginate(github.rest.issues.listForRepo, { ...context.repo, state: "open", per_page: 100 });
          const existing = issues.find((issue) => !issue.pull_request && issue.title === title);
          if (existing) await github.rest.issues.createComment({ ...context.repo, issue_number: existing.number, body });
          else await github.rest.issues.create({ ...context.repo, title, body });
```

Make `windows-bundle` need `advisory-scan` and require `needs.advisory-scan.result == 'success'` with its existing tag/manual predicate. Do not add `always()`/`!cancelled()`. Setup failure leaves output false/absent, opens no issue, and blocks bundle. Keep the Step 1 duplicate placement intact; cron advisory-only has none.

`runs-on: ubuntu-latest` and serialized `rust-advisory-follow-up` concurrency remain required writer implementation facts. Check them in the combined static review only; they are not unit-test properties or mutation cases.

- [ ] **Step 7: Run workflow and npm-scope GREEN checks**

```powershell
npm.cmd run test:unit -- scripts/testing/github-rust-workflows.test.ts
rg -n "uses:.*@(v[0-9]+|main|master)$" .github/workflows
git diff -- package.json package-lock.json
```

Expected: test PASS; mutable-ref search has no matches; dependency diff contains only direct `yaml` and its lock entry.

- [ ] **Step 8: Commit Task 3**

```powershell
git add package.json package-lock.json .github/workflows/rust-fast.yml .github/workflows/rust-full.yml .github/workflows/rust-release.yml scripts/testing/github-rust-workflows.test.ts
git commit -m "ci: isolate advisory writes with structural workflow contracts"
```

Expected: no Cargo/Rust/doc changes; the final workflow YAML contains duplicate placement, pins, and advisory isolation once, while the workflow test still lists exactly four owned properties.

---

### Task 4: Fix — Close Bounded Rust Wave 0 Findings

**Files:**
- Modify `package.json`, `scripts/verify.test.ts`, `deny.toml`, `scripts/testing/repository-rules.test.ts`, both named Rust sources, `docs/value-registry.md`, `README.md`, `AGENTS.md`, `docs/project.md`, and the prior Wave 0 plan/verification.

**Interfaces:**
- Consumes: Task 1 modes, Task 2 kinds, existing serialized sidecar types and LLM access resolver.
- Produces: locked aliases, strict licenses, independent wire golden, owned secrets, factual docs/registry.

- [ ] **Step 1: Write locked-alias and strict-license RED mutations**

In `scripts/verify.test.ts`, assert exact aliases:

```ts
function assertLockedRustAliases(scripts: Record<string, string>) {
  expect(scripts["test:rust"]).toBe("cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked");
  expect(scripts["test:rust:prompt-pack-runs"]).toBe("cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib prompt_pack_run --locked");
}
assertLockedRustAliases(packageJson.scripts);
for (const name of ["test:rust", "test:rust:prompt-pack-runs"]) {
  const mutated = structuredClone(packageJson);
  mutated.scripts[name] = mutated.scripts[name].replace(" --locked", "");
  expect(() => assertLockedRustAliases(mutated.scripts)).toThrow();
}
```

In `repository-rules.test.ts`, mutate each exact deny setting independently:

```ts
function assertStrictLicensePolicy(source: string) {
  const section = /\[licenses\]\s*([\s\S]*?)(?=\n\[(?!licenses\.private\])|$)/.exec(source)?.[1];
  expect(section).toBeDefined();
  expect(section).toMatch(/^unused-allowed-license = "deny"$/m);
  expect(section).toMatch(/^unused-license-exception = "deny"$/m);
  expect(section).not.toContain('"Apache-2.0 WITH LLVM-exception"');
  expect(source).not.toMatch(/^\[\[licenses\.exceptions\]\]$/m);
}
it.each([
  ["stale allowance", (source: string) => source.replace('"Apache-2.0",', '"Apache-2.0",\n  "Apache-2.0 WITH LLVM-exception",')],
  ["stale exception", (source: string) => `${source}\n[[licenses.exceptions]]\nname = "fake"\nallow = ["MIT"]\n`],
  ["allowance warn", (source: string) => source.replace('unused-allowed-license = "deny"', 'unused-allowed-license = "warn"')],
  ["exception warn", (source: string) => source.replace('unused-license-exception = "deny"', 'unused-license-exception = "warn"')],
])("rejects %s", (_name, mutate) => {
  expect(() => assertStrictLicensePolicy(mutate(readFileSync("deny.toml", "utf8")))).toThrow();
});
```

- [ ] **Step 2: Run policy RED and implement minimal alias/license edits**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/repository-rules.test.ts
```

Expected RED: missing alias locks and permissive license policy. Then set:

```json
"test:rust": "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked",
"test:rust:prompt-pack-runs": "cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib prompt_pack_run --locked"
```

In `deny.toml`, remove only `Apache-2.0 WITH LLVM-exception`; set both unused license lint levels to `deny`. Re-run the focused test; expected PASS.

- [ ] **Step 3: Replace the sidecar self-round-trip with an independent RED/GREEN golden**

Construct `response` as today, but compare against a literal object and deserialize independently:

```rust
let expected = serde_json::json!({
    "type": "run_result",
    "result": {
        "run_id": "run-1", "status": "ok", "text": "answer",
        "message": null, "manual_action": null,
        "artifacts": { "run_dir": null, "html": null, "screenshot": null,
            "telemetry": null, "answer_extraction": null, "artifact_write_error": null },
        "elapsed_ms": 42, "debug_summary": null
    }
});
assert_eq!(serde_json::to_value(&response).unwrap(), expected);
let decoded: GeminiBrowserSidecarResponse = serde_json::from_value(serde_json::json!({
    "type": "run_result",
    "result": {
        "run_id": "run-1", "status": "ok", "text": "answer",
        "message": null, "manual_action": null,
        "artifacts": { "run_dir": null, "html": null, "screenshot": null,
            "telemetry": null, "answer_extraction": null, "artifact_write_error": null },
        "elapsed_ms": 42, "debug_summary": null
    }
})).unwrap();
assert_eq!(decoded, response);
```

Run before the edit to record that the old test passes but lacks independent deserialization evidence; after replacement run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
```

Expected: exact test is nonempty and PASS; focused check/checkpoint PASS.

- [ ] **Step 4: Record the existing LLM characterization GREEN**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact
```

Expected: exactly one existing complete/key-only/base-only/empty-key/empty-base characterization test PASS before the ownership refactor. Do not manufacture a compile RED.

- [ ] **Step 5: Move secret ownership without cloning**

Implement private ownership transfer:

```rust
enum ConfiguredProviderAccess {
    Complete(LlmProviderAccess),
    Partial { api_key: Option<SecretString>, base_url: Option<String> },
}
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

Replace the caller completely:

```rust
let access = match configured_provider_access(provider_kind, configured_key, configured_base_url)? {
    ConfiguredProviderAccess::Complete(access) => access,
    ConfiguredProviderAccess::Partial { api_key, base_url } => {
        let pool = get_pool(&handle).await?;
        resolve_provider_access_from_pool(
            &pool, &secret_store, provider_kind, profile_id.as_deref(), api_key, base_url,
        ).await?
    }
};
```

Adapt the existing test in place, including `ConfiguredProviderAccess` in its existing `use super::{...}` list:

```rust
let complete = configured_provider_access(
    provider,
    Some(SecretString::new("configured-key".into())),
    Some("https://api.example.test/v1".into()),
)
.expect("complete");
assert!(matches!(complete, ConfiguredProviderAccess::Complete(_)));

let key_only = configured_provider_access(
    provider,
    Some(SecretString::new("configured-key".into())),
    None,
)
.expect("key only");
assert!(matches!(
    key_only,
    ConfiguredProviderAccess::Partial { api_key: Some(_), base_url: None }
));

let base_only = configured_provider_access(
    provider,
    None,
    Some("https://api.example.test/v1".into()),
)
.expect("base only");
assert!(matches!(
    base_only,
    ConfiguredProviderAccess::Partial { api_key: None, base_url: Some(_) }
));

let (empty_key, configured_base) = normalize_configured_provider_overrides(Some("   "), Some("https://api.example.test/v1"));
let empty_key_access = configured_provider_access(provider, empty_key, configured_base)
    .expect("empty key");
assert!(matches!(
    empty_key_access,
    ConfiguredProviderAccess::Partial { api_key: None, base_url: Some(_) }
));

let (configured_key, empty_base) = normalize_configured_provider_overrides(Some("configured-key"), Some("   "));
let empty_base_access = configured_provider_access(provider, configured_key, empty_base)
    .expect("empty base");
assert!(matches!(
    empty_base_access,
    ConfiguredProviderAccess::Partial { api_key: Some(_), base_url: None }
));
```

Do not expose/serialize/log/convert/persist the key.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
rg -n "api_key\.clone\(\)" src-tauri/src/llm/mod.rs
$ImplementationBase=(Get-Content .superpowers/sdd/rust-wave-0-final-hardening-implementation-base.txt -Raw).Trim()
git diff ("{0}..HEAD" -f $ImplementationBase) -- src-tauri/src/llm/mod.rs | rg "^\+.*(api_key\.clone\(\)|expose_secret)"
```

Expected: exact test/check/checkpoint PASS; both searches exit 1 with no new clone or newly added exposure. Existing unchanged test-only `ExposeSecret` imports/calls are allowed and remain outside the added diff.

- [ ] **Step 6: Update registry and factual documentation**

Add only these registry families:

```markdown
| cargo-deny mode | `Setup`, `Deterministic`, `Advisories` | `scripts/cargo-deny.ps1` | CLI-only; not persisted or exposed to UI |
| Cargo dependency kind | `normal`, `build`, `dev` | `scripts/rust-dependency-policy.mjs` | Generated policy identity |
| npm dependency kind | `dependencies`, `devDependencies`, `optionalDependencies` | `scripts/rust-dependency-policy.mjs` | Generated policy identity |
```

Do not add duplicate modes/classifications. State that unused `duplicateGrowthExceptions` are reviewed and removed manually when closing a dependency wave; there is no automated cleanup/expiry check. Correct old plan/verification claims to the actual local boundary and commands. Docs must not claim push, remote green workflows, bundle, PR, or release.

Run:

```powershell
rg -n "CLEAN|REVIEW|REDUCTION|VIOLATION|Full mode|Release mode|run-cargo-deny|global cargo-deny" docs/value-registry.md README.md AGENTS.md docs/project.md docs/superpowers/plans/2026-08-12-rust-infrastructure-wave-0.md docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md
```

Expected: no obsolete duplicate mode/classification or global-runner claim.

- [ ] **Step 7: Run Task 4 checkpoint and commit**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/repository-rules.test.ts
npm.cmd run check:rust:supply-chain
git diff --check
git add package.json scripts/verify.test.ts deny.toml scripts/testing/repository-rules.test.ts src-tauri/crates/extractum-gemini-browser/src/types.rs src-tauri/src/llm/mod.rs docs/value-registry.md README.md AGENTS.md docs/project.md docs/superpowers/plans/2026-08-12-rust-infrastructure-wave-0.md docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md
git commit -m "fix: close bounded Rust Wave 0 findings"
```

Expected: tests/policy/diff PASS and the fourth named code-task commit exists.

---

## Combined Code Review

- [ ] **Review the four-task range once for scope, spec, and code quality**

```powershell
$ImplementationBase = (Get-Content .superpowers/sdd/rust-wave-0-final-hardening-implementation-base.txt -Raw).Trim()
git rev-parse --verify ($ImplementationBase + "^{commit}")
git log --reverse --format="%H`t%s" ("{0}..HEAD" -f $ImplementationBase)
git diff --name-only ("{0}..HEAD" -f $ImplementationBase)
git diff --check ("{0}..HEAD" -f $ImplementationBase)
```

Expected named subjects, in order:

```text
build: consolidate pinned cargo-deny runner
test: harden Rust dependency and duplicate policy
ci: isolate advisory writes with structural workflow contracts
fix: close bounded Rust Wave 0 findings
```

Adjacent correction commits are allowed. Review against every authorized path and all spec sections. In particular verify: one production runner; no parser/mirror; JSON sole Tauri authority; current equality before history; four workflow properties only; Full unconditional once; `windows-bundle` once; advisory jobs/cron zero; only `yaml` dependency delta; no secret clone.

- [ ] **Apply review corrections and recheck the combined range**

For each finding, add an adjacent correction commit owned by the relevant task, rerun that task's focused commands, then rerun:

```powershell
$ImplementationBase = (Get-Content .superpowers/sdd/rust-wave-0-final-hardening-implementation-base.txt -Raw).Trim()
git rev-parse --verify ($ImplementationBase + "^{commit}")
git diff --check ("{0}..HEAD" -f $ImplementationBase)
git diff --name-only ("{0}..HEAD" -f $ImplementationBase)
```

Expected: no Important/Critical findings, no unauthorized path. Do not create per-task review gates.

---

## Final Acceptance and Evidence

- [ ] **Step 1: Run the exact ordered local acceptance**

Run from repository root, without reordering or omitting commands:

```powershell
npm.cmd ci
npm.cmd run test:unit -- scripts/verify.test.ts scripts/rust-dependency-policy.test.ts scripts/rust-duplicate-baseline.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts scripts/testing/github-rust-workflows.test.ts
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/cargo-deny.test.ps1
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
npm.cmd run test:unit
npm.cmd run check
npm.cmd run bootstrap:testing
npm.cmd run check:rust:duplicates
npm.cmd run check:rust:duplicates -- --base 9bcc047b9ec210c1ea34024f6dc782a1e7caec49
npm.cmd run check:rust:fast
npm.cmd run verify
npm.cmd run check:rust:advisories
git diff --check
```

Expected: commands 1-16 and 18 PASS. Command 17 (`check:rust:advisories`) may be nonzero because RustSec moves; record it exactly, keep it a release blocker, and do not add an exception. Confirm package/lock diff adds only direct `yaml` and its lock entry.

- [ ] **Step 2: Write the factual evidence record**

Create `docs/superpowers/verification/2026-08-15-rust-wave-0-final-hardening.md` with these concrete headings:

```markdown
# Rust Wave 0 Final Hardening Verification
## Scope and Spec Authority
## Environment and Tool Digests
## Four Task Commits and Combined Review
## Focused JavaScript, PowerShell, and Rust Evidence
## Ordered Local Acceptance
## Advisory Result
## Authorized-Scope and Dependency Diff
## Remote Evidence Pending
```

Record exact command, exit code, relevant counts, tested implementation HEAD before the evidence commit, cargo-deny binary SHA-256, and review findings. State that remote Rust Fast/Full and release bundle remain pending; do not claim actions not run.

- [ ] **Step 3: Commit only the evidence record**

```powershell
git add docs/superpowers/verification/2026-08-15-rust-wave-0-final-hardening.md
git diff --cached --check
git diff --cached --name-only
git commit -m "docs: verify Rust Wave 0 final hardening"
```

Expected: staged/committed scope is exactly the one evidence file. The record names the tested implementation head, not its own impossible future SHA.

- [ ] **Step 4: Review the final whole range and stop**

```powershell
git log --reverse --format="%H`t%s" 77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD
git diff --check 77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD
git diff --name-only 77c739a321f08fc49d7a46d846d2a93a0dafc116..HEAD
git status --short
```

Expected: all four named code-task subjects occur in order, followed by `docs: verify Rust Wave 0 final hardening`; adjacent correction commits are permitted; evidence commit is included; diff check and worktree are clean; no Important/Critical finding remains. Write the evidence commit SHA, reviewed range, reviewer/report path, and findings to the ignored final-review report and handoff. Stop without push, PR, dispatch, bundle, or release.
