# Grammers crates.io 0.10.0 Source Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the complete eight-package Grammers graph from the Codeberg Git pin to the equivalent crates.io artifacts while preserving direct feature policy and enforcing the resolved registry graph in baseline schema v2.

**Architecture:** Keep the four direct Cargo declarations exact at `=0.10.0`. Split baseline schema v2 into `directPackages` for the existing four-package feature policy and `resolvedPackages` for the eight-package `{name, version, source}` identity; the repository rule compares the generated schema with the committed canonical artifact and separately enforces the direct manifest requirements.

**Tech Stack:** Cargo/Rust, Node.js ESM, TypeScript, Vitest, JSON, PowerShell, Tauri workspace verification.

## Global Constraints

- Historical specifications, plans, verification records, migration notes, and diagnostic snapshots remain unchanged.
- Product behavior, Telegram schema handling, session/persistence formats, public Rust interfaces, and frontend interfaces do not change.
- Stop and re-plan if registry resolution changes a Rust API or any `directPackages[*].required` feature list.
- The four direct dependencies use exact requirements `=0.10.0` with their current default-feature and requested-feature settings.
- The canonical resolved graph is seven packages at `0.10.0` plus `grammers-tl-parser` at `1.2.2`, all from `registry+https://github.com/rust-lang/crates.io-index`.
- No Git-sourced `grammers-*` package may remain in locked Cargo metadata.
- `src/lib/telegram-grammers-feature-baseline.json` is generated with `--write`; do not hand-edit it.
- Preserve the existing `required` feature arrays exactly. Review `universe`/`forbidden` diffs as published-crate facts.
- On Windows use `npm.cmd`, not `npm`.
- Use canonical `src-tauri/target`; do not create a slice-specific Cargo target directory.
- If execution starts in a fresh checkout or worktree, run `npm.cmd run bootstrap:testing` before the final `npm.cmd run verify`.

## File Map

- Modify `src-tauri/Cargo.toml`: replace the four Codeberg dependency specifications with exact crates.io requirements.
- Modify `src-tauri/Cargo.lock`: resolve the complete Grammers graph from crates.io with Cargo-managed checksums.
- Modify `scripts/telegram-grammers-feature-baseline.mjs`: generate schema v2 with `directPackages` and complete `resolvedPackages` identity.
- Regenerate `src/lib/telegram-grammers-feature-baseline.json`: store the canonical schema-v2 snapshot.
- Modify `scripts/testing/repository-rules.mjs`: enforce direct `req = "=0.10.0"`, consume `directPackages`, and remove the redundant source comparison.
- Modify `scripts/testing/repository-rules.test.ts`: model the complete eight-package graph and add source/version/manifest-requirement mutations.
- Modify `docs/project.md`: describe crates.io as the current source and the baseline as owner of the complete resolved graph.
- Preserve `docs/superpowers/verification/2026-08-11-grammers-crates-io-provenance.md`: provenance evidence is already complete and must not be rewritten during implementation.

## Rust Verification Loops

Affected package: `extractum-telegram`. No Rust source is expected to change.

- Dependency RED: after switching `src-tauri/Cargo.toml` and resolving `Cargo.lock`, but before changing the generator, run `node scripts/telegram-grammers-feature-baseline.mjs --check`; expect failure because schema v1 still requires the Codeberg identity.
- Policy RED: after changing the synthetic fixture to schema v2 and adding the new mutations, but before changing the generator/rule, run `npm.cmd run test:related -- scripts/testing/repository-rules.test.ts`; expect a non-empty failing selection.
- Policy GREEN: rerun the same related test and `node scripts/telegram-grammers-feature-baseline.mjs --check` after implementation.
- Focused Rust check: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
- Package checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
- Feature-off package proof: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`.
- End-of-slice workspace gate: `npm.cmd run verify`.

---

### Task 1: Expand the Synthetic Cargo Graph Without Changing Current Policy

**Files:**
- Modify: `scripts/testing/repository-rules.test.ts:89-189`
- Test: `scripts/testing/repository-rules.test.ts`

**Interfaces:**
- Consumes: current schema-v1 `grammersBaseline.packages` and current Codeberg source fixture.
- Produces: `grammersResolvedVersions`, `grammersResolvedEdges`, eight metadata package records, and eight resolved nodes that Task 2 can migrate to schema v2.

- [ ] **Step 1: Add the complete current resolved inventory and edges**

Immediately after `grammersBaseline`, add the exact current graph:

```ts
const grammersResolvedVersions = {
  "grammers-client": "0.10.0",
  "grammers-crypto": "0.10.0",
  "grammers-mtproto": "0.10.0",
  "grammers-mtsender": "0.10.0",
  "grammers-session": "0.10.0",
  "grammers-tl-gen": "0.10.0",
  "grammers-tl-parser": "1.2.2",
  "grammers-tl-types": "0.10.0",
} as const;

type GrammersPackageName = keyof typeof grammersResolvedVersions;
type GrammersResolvedEdge = { name: GrammersPackageName; kind: null | "build" };
const normalEdge = (name: GrammersPackageName): GrammersResolvedEdge => ({ name, kind: null });
const normalEdges = (...names: GrammersPackageName[]): GrammersResolvedEdge[] => names.map(normalEdge);

const grammersResolvedEdges: Record<GrammersPackageName, GrammersResolvedEdge[]> = {
  "grammers-client": normalEdges("grammers-crypto", "grammers-mtsender", "grammers-session", "grammers-tl-types"),
  "grammers-crypto": [],
  "grammers-mtproto": normalEdges("grammers-crypto", "grammers-tl-types"),
  "grammers-mtsender": normalEdges("grammers-crypto", "grammers-mtproto", "grammers-session", "grammers-tl-types"),
  "grammers-session": [normalEdge("grammers-tl-types")],
  "grammers-tl-gen": [normalEdge("grammers-tl-parser")],
  "grammers-tl-parser": [],
  "grammers-tl-types": [
    { name: "grammers-tl-gen", kind: "build" },
    { name: "grammers-tl-parser", kind: "build" },
  ],
};
```

- [ ] **Step 2: Build eight package records from the inventory**

Keep the current Git source and schema-v1 baseline in this task. Replace the four-record `grammers` construction with:

```ts
const revision = grammersBaseline.revision;
const exactSource = `git+https://codeberg.org/Lonami/grammers?rev=${revision}#${revision}`;
const grammers = Object.entries(grammersResolvedVersions).map(([name, version]) => {
  const directPolicy = grammersBaseline.packages.find((entry) => entry.name === name);
  return {
    id: `${name} ${version} (${exactSource})`,
    name,
    version,
    source: exactSource,
    features: Object.fromEntries((directPolicy?.universe ?? []).map((feature) => [feature, []])),
    targets: [{ kind: ["lib"], name: name.replaceAll("-", "_") }],
    dependencies: [],
  };
});
```

- [ ] **Step 3: Give all eight packages accurate resolved edges**

Replace the existing empty Grammers node list with:

```ts
...grammers.map((entry) => ({
  id: entry.id,
  features: [
    ...(grammersBaseline.packages.find(({ name }) => name === entry.name)?.required ?? []),
  ],
  deps: grammersResolvedEdges[entry.name as GrammersPackageName].map((dependency) => ({
    name: dependency.name.replaceAll("-", "_"),
    pkg: grammers.find(({ name }) => name === dependency.name)!.id,
    dep_kinds: [{ kind: dependency.kind, target: null }],
  })),
})),
```

The producer node remains connected only to its four direct Grammers dependencies.

- [ ] **Step 4: Run the current repository-rule suite**

Run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
```

Expected: PASS with a non-empty Vitest selection. Existing Git-source and feature-policy mutations still pass; the fixture now contains eight Grammers package records and their real resolve edges.

- [ ] **Step 5: Review and commit the fixture-only refactor**

Run:

```powershell
git diff --check
git diff -- scripts/testing/repository-rules.test.ts
git add -- scripts/testing/repository-rules.test.ts
git commit -m "test: model complete grammers dependency graph"
```

Expected: the commit changes only the synthetic metadata builder and keeps all current policy tests green.

---

### Task 2: Migrate Cargo and the Dependency Policy to crates.io

**Files:**
- Modify: `src-tauri/Cargo.toml:12-15`
- Modify: `src-tauri/Cargo.lock`
- Modify: `scripts/telegram-grammers-feature-baseline.mjs:11-244`
- Modify: `src/lib/telegram-grammers-feature-baseline.json`
- Modify: `scripts/testing/repository-rules.mjs:205-256`
- Modify: `scripts/testing/repository-rules.test.ts:89-307`
- Test: `scripts/testing/repository-rules.test.ts`

**Interfaces:**
- Consumes: Task 1's `grammersResolvedVersions`, `grammersResolvedEdges`, and complete synthetic graph.
- Produces: baseline schema `{ schemaVersion: 2, directPackages, resolvedPackages }`, where every resolved entry is `{ name, version, source }`; direct Cargo requirements are `=0.10.0`.

- [ ] **Step 1: Replace the four workspace Git dependencies**

Use these exact declarations in `src-tauri/Cargo.toml`:

```toml
grammers-client = { version = "=0.10.0", default-features = false }
grammers-mtsender = { version = "=0.10.0" }
grammers-session = { version = "=0.10.0", default-features = false, features = ["serde"] }
grammers-tl-types = { version = "=0.10.0", features = ["deserializable-functions"] }
```

- [ ] **Step 2: Resolve the changed graph and run the focused Rust check**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
```

Expected: Cargo updates `src-tauri/Cargo.lock`, downloads registry artifacts as needed, and the package compiles. If Rust compilation reveals an API difference, stop and re-plan; do not adapt product code in this slice.

- [ ] **Step 3: Capture the dependency-source RED signal in the only valid window**

Before editing the generator, run:

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --check
```

Expected: FAIL because schema-v1 generation still requires the Codeberg source/revision while locked metadata now uses crates.io. Record the actual failure text in the task report.

- [ ] **Step 4: Change the test fixture to the desired schema v2**

Move `grammersResolvedVersions` and `grammersResolvedEdges` above
`grammersBaseline`, then replace the baseline with this exact synthetic shape:

```ts
const registrySource = "registry+https://github.com/rust-lang/crates.io-index";
const grammersBaseline = {
  schemaVersion: 2,
  directPackages: [
    { name: "grammers-client", required: [], forbidden: ["default"], universe: ["default"] },
    { name: "grammers-mtsender", required: [], forbidden: ["proxy"], universe: ["proxy"] },
    { name: "grammers-session", required: ["serde"], forbidden: ["default"], universe: ["default", "serde"] },
    { name: "grammers-tl-types", required: ["default", "deserializable-functions"], forbidden: ["impl-serde"], universe: ["default", "deserializable-functions", "impl-serde"] },
  ],
  resolvedPackages: Object.entries(grammersResolvedVersions).map(([name, version]) => ({
    name,
    version,
    source: registrySource,
  })),
};
```

Delete the Task 1 `revision`/`exactSource` declarations and replace the Grammers
package builder with:

```ts
const grammers = grammersBaseline.resolvedPackages.map(({ name, version, source }) => {
  const directPolicy = grammersBaseline.directPackages.find((entry) => entry.name === name);
  return {
    id: `${source}#${name}@${version}`,
    name,
    version,
    source,
    features: Object.fromEntries((directPolicy?.universe ?? []).map((feature) => [feature, []])),
    targets: [{ kind: ["lib"], name: name.replaceAll("-", "_") }],
    dependencies: [],
  };
});
```

Add exact requirements and registry source to the producer dependency records:

```ts
req: String(name).startsWith("grammers-") ? "=0.10.0" : "*",
source: String(name).startsWith("grammers-") ? registrySource : null,
```

- [ ] **Step 5: Add the three required policy mutations**

Under `rule:telegram-crate-dependency-ownership`, replace the old direct revision mutation with these mutations:

```ts
"drifts a transitive Grammers source": () => {
  const metadata = clone(cargoMetadata());
  metadata.packages.find(({ name }: any) => name === "grammers-crypto")!.source =
    "git+https://codeberg.org/Lonami/grammers?rev=wrong#wrong";
  return cargoIndex(metadata);
},
"drifts a transitive Grammers version": () => {
  const metadata = clone(cargoMetadata());
  metadata.packages.find(({ name }: any) => name === "grammers-mtproto")!.version = "0.10.1";
  return cargoIndex(metadata);
},
"widens a direct Grammers manifest requirement": () => {
  const metadata = clone(cargoMetadata());
  const producer = metadata.packages.find(({ name }: any) => name === "extractum-telegram")!;
  producer.dependencies.find(({ name }: any) => name === "grammers-client")!.req = "^0.10.0";
  return cargoIndex(metadata);
},
```

Rename the reordered-baseline mutation and assertion from `packages` to `directPackages`.

- [ ] **Step 6: Run the policy test to prove the new expectation is RED**

Run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
```

Expected: FAIL with a non-empty selection because production generation still returns schema v1 and the rule does not yet enforce dependency `req`.

- [ ] **Step 7: Implement schema-v2 generation**

In `scripts/telegram-grammers-feature-baseline.mjs`, update the JSDoc model:

```js
/**
 * @typedef {object} ResolvedGrammersPackage
 * @property {string} name
 * @property {string} version
 * @property {string | null} source
 */

/**
 * @typedef {object} FeatureBaseline
 * @property {number} schemaVersion
 * @property {FeatureBaselinePackage[]} directPackages
 * @property {ResolvedGrammersPackage[]} resolvedPackages
 */
```

Update `CargoMetadataPackage` with these properties, rename the local
feature-policy result from `packages` to `directPackages`, remove `revision` and
the Git `exactSource` assertion, and build the resolved identity with:

```js
/**
 * @typedef {object} CargoMetadataPackage
 * @property {string} id
 * @property {string} name
 * @property {string} version
 * @property {string | null} [source]
 * @property {Record<string, unknown>} features
 */
```

```js
const resolvedPackages = metadataPackages
  .filter((candidate) => candidate?.name?.startsWith("grammers-"))
  .map((candidate) => {
    if (typeof candidate.version !== "string") {
      fail(`missing version for ${candidate.name}`);
    }
    if (candidate.source !== null && typeof candidate.source !== "string") {
      fail(`invalid source for ${candidate.name}`);
    }
    return {
      name: candidate.name,
      version: candidate.version,
      source: candidate.source ?? null,
    };
  })
  .sort((left, right) =>
    left.name.localeCompare(right.name)
    || left.version.localeCompare(right.version)
    || String(left.source).localeCompare(String(right.source)));
```

Return exactly:

```js
return {
  schemaVersion: 2,
  directPackages,
  resolvedPackages,
};
```

Keep the existing four-distinct-direct-package check and direct feature calculation unchanged apart from the `directPackages` rename.

- [ ] **Step 8: Enforce exact direct manifest requirements in the repository rule**

In `scripts/testing/repository-rules.mjs`, read the direct policy from `baseline.directPackages`, then add:

```js
for (const dependency of producerGrammers) {
  if (dependency.req !== "=0.10.0") {
    violations.push(`${dependency.name}: direct manifest requirement must be =0.10.0`);
  }
}
```

Delete the second per-package source construction/comparison:

```js
const expectedSource = `git+https://codeberg.org/Lonami/grammers?rev=${baseline.revision}#${baseline.revision}`;
if (selected.source !== expectedSource) violations.push(`${expected.name}: source revision drifted`);
```

Generated schema comparison now owns the complete resolved inventory, versions, and sources.

- [ ] **Step 9: Regenerate the canonical artifact**

Run:

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --write
```

Expected: the artifact has `schemaVersion: 2`, four complete
`directPackages` feature-policy records generated from Cargo metadata, and the
eight `resolvedPackages` records audited in Step 12. It has no `revision`,
top-level `version`, or top-level `packages` field.

- [ ] **Step 10: Prove the direct required-feature policy did not drift**

Run this PowerShell comparison before committing:

```powershell
$previousBaseline = git show HEAD:src/lib/telegram-grammers-feature-baseline.json |
  Out-String | ConvertFrom-Json
$currentBaseline = Get-Content src/lib/telegram-grammers-feature-baseline.json -Raw |
  ConvertFrom-Json
$previousRequired = $previousBaseline.packages |
  Select-Object name, required | ConvertTo-Json -Depth 5 -Compress
$currentRequired = $currentBaseline.directPackages |
  Select-Object name, required | ConvertTo-Json -Depth 5 -Compress
if ($previousRequired -ne $currentRequired) {
  throw "Grammers required feature policy drifted"
}
Write-Output "required feature policy unchanged"
```

Expected: `required feature policy unchanged`. If it throws, stop and investigate instead of accepting the generated artifact.

- [ ] **Step 11: Run policy GREEN checks**

Run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
node scripts/telegram-grammers-feature-baseline.mjs --check
```

Expected: both commands PASS; the Vitest selection is non-empty and includes the three new mutations.

- [ ] **Step 12: Audit the authoritative locked metadata**

Run:

```powershell
$metadata = cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 |
  ConvertFrom-Json
$grammers = @($metadata.packages | Where-Object { $_.name -like "grammers-*" } | Sort-Object name)
$expected = @{
  "grammers-client" = "0.10.0"; "grammers-crypto" = "0.10.0"
  "grammers-mtproto" = "0.10.0"; "grammers-mtsender" = "0.10.0"
  "grammers-session" = "0.10.0"; "grammers-tl-gen" = "0.10.0"
  "grammers-tl-parser" = "1.2.2"; "grammers-tl-types" = "0.10.0"
}
if ($grammers.Count -ne 8) { throw "expected eight resolved Grammers packages" }
foreach ($package in $grammers) {
  if ($expected[$package.name] -ne $package.version) { throw "version drift: $($package.name)" }
  if ($package.source -ne "registry+https://github.com/rust-lang/crates.io-index") {
    throw "source drift: $($package.name)"
  }
}
$producer = $metadata.packages | Where-Object name -eq "extractum-telegram"
$direct = @($producer.dependencies | Where-Object { $_.name -like "grammers-*" })
if ($direct.Count -ne 4 -or @($direct | Where-Object req -ne "=0.10.0").Count -ne 0) {
  throw "direct Grammers requirements drifted"
}
$grammers | Select-Object name, version, source | Format-Table -AutoSize
```

Expected: eight rows, the expected versions, only the canonical registry source, and no exception.

- [ ] **Step 13: Audit the lockfile and commit the core migration**

Run:

```powershell
git diff --check
git diff -- src-tauri/Cargo.toml src-tauri/Cargo.lock scripts/telegram-grammers-feature-baseline.mjs src/lib/telegram-grammers-feature-baseline.json scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts
git add -- src-tauri/Cargo.toml src-tauri/Cargo.lock scripts/telegram-grammers-feature-baseline.mjs src/lib/telegram-grammers-feature-baseline.json scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts
git diff --cached --check
git commit -m "build: source grammers 0.10 from crates.io"
```

Expected: only the eight-package registry resolution and policy/schema changes are committed; unrelated lockfile churn is absent.

---

### Task 3: Update Current Dependency Documentation

**Files:**
- Modify: `docs/project.md:198-226`
- Preserve: `docs/superpowers/verification/2026-08-11-grammers-0-10-upgrade.md`
- Preserve: `docs/superpowers/verification/2026-08-11-grammers-crates-io-provenance.md`

**Interfaces:**
- Consumes: Task 2's exact direct requirements and schema-v2 resolved graph.
- Produces: current project policy that distinguishes the direct `=0.10.0` line from the complete resolved baseline.

- [ ] **Step 1: Replace only the active policy paragraphs**

Replace the active statement that Grammers crates are owned Git dependencies with:

```markdown
The `grammers-*` crates are explicitly controlled crates.io dependencies because
Extractum's Telegram behavior depends on upstream runtime details. Treat updates
to the direct Grammers line or its resolved graph as explicit dependency work,
not incidental lockfile churn.
```

Replace the current-pin paragraph with:

```markdown
Current active source: crates.io. The four direct dependencies are pinned with
exact requirement `=0.10.0`; the generated Grammers feature baseline fixes the
complete resolved graph at seven `0.10.0` packages plus
`grammers-tl-parser 1.2.2`, all from the canonical crates.io registry source.
The published artifacts originate from Codeberg revision
`5c6d44ff30e02d6c9295bcf1fcb51403ad77c981`, as recorded in the sanitized
provenance verification.
```

Link `provenance verification` to
`superpowers/verification/2026-08-11-grammers-crates-io-provenance.md` using the existing relative-link style.

Do not alter the historical GitHub/Codeberg migration paragraphs, the older lock revision, or either existing verification record.

- [ ] **Step 2: Run focused documentation/policy checks**

Run:

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --check
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
git diff --check
```

Expected: baseline check PASS, related test PASS with a non-empty selection, and diff check PASS.

- [ ] **Step 3: Review and commit the documentation update**

Run:

```powershell
git diff -- docs/project.md
git add -- docs/project.md
git diff --cached --check
git commit -m "docs: document grammers crates.io policy"
```

Expected: the commit changes only current-state policy and leaves historical evidence untouched.

---

### Task 4: Run Package and Workspace Completion Gates

**Files:**
- Modify only if checkbox tracking is committed: `docs/superpowers/plans/2026-08-11-grammers-crates-io-0-10.md`
- Verify: all Task 1-3 files

**Interfaces:**
- Consumes: committed registry migration and documentation.
- Produces: final completion evidence with a clean worktree.

- [ ] **Step 1: Run the extractum-telegram package checkpoint**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
```

Expected: PASS with a non-zero test count and no API or fixture incompatibility.

- [ ] **Step 2: Prove the feature-off production surface**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features
```

Expected: PASS.

- [ ] **Step 3: Run the full workspace gate**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS. If a failure is unrelated or environmental, diagnose it with `systematic-debugging`; do not report completion until an authoritative full run passes.

- [ ] **Step 4: Repeat the final metadata and generated-artifact checks**

Run:

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --check
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 | Out-Null
git diff HEAD --check
git status --short
```

Expected: both policy commands PASS, `git diff HEAD --check` PASS, and `git status --short` is empty except for intentional plan checkbox tracking.

- [ ] **Step 5: Commit plan tracking if the execution workflow updated it**

If this plan's checkboxes changed, run:

```powershell
git add -- docs/superpowers/plans/2026-08-11-grammers-crates-io-0-10.md
git diff --cached --check
git commit -m "docs: record grammers crates.io verification"
```

If the plan file did not change, do not create an empty commit.

- [ ] **Step 6: Report completion evidence**

Report:

- all implementation commit hashes;
- the eight resolved package names, versions, and canonical registry source;
- unchanged `directPackages[*].required` policy;
- focused repository-rule test result;
- package checkpoint and feature-off check results;
- final `npm.cmd run verify` result;
- final clean `git status --short`.
