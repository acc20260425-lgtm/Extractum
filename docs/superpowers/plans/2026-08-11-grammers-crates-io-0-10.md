# Grammers crates.io 0.10.0 Source Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the complete eight-package Grammers graph from the Codeberg Git pin to the equivalent crates.io artifacts while preserving direct feature policy and enforcing the resolved registry graph in baseline schema v2.

**Architecture:** Keep the four direct Cargo declarations exact at `=0.10.0`. Split baseline schema v2 into `directPackages` for the existing feature policy and `resolvedPackages` for the eight-package `{name, version, source}` identity; compare generated and committed baselines through `sameJson`, and enforce direct manifest requirements separately.

**Tech Stack:** Cargo/Rust, Node.js ESM, TypeScript, Vitest, JSON, PowerShell, Tauri workspace verification.

## Global Constraints

- Historical specifications, plans, verification records, migration notes, and diagnostic snapshots remain unchanged.
- Product behavior, Telegram schema handling, session/persistence formats, public Rust interfaces, and frontend interfaces do not change.
- Stop and re-plan if registry resolution changes a Rust API or any `directPackages[*].required` feature list.
- The four direct dependencies use exact requirements `=0.10.0` with their current feature settings.
- The canonical resolved graph is seven packages at `0.10.0` plus `grammers-tl-parser` at `1.2.2`, all from `registry+https://github.com/rust-lang/crates.io-index`.
- No Git-sourced `grammers-*` package may remain in locked Cargo metadata.
- `src/lib/telegram-grammers-feature-baseline.json` is generated with `--write`; do not hand-edit it.
- Preserve `required` arrays exactly. Review `universe`/`forbidden` diffs as published-crate facts.
- On Windows use `npm.cmd`, not `npm`.
- Use canonical `src-tauri/target`; do not create a slice-specific Cargo target directory.
- In a fresh checkout or worktree, run `npm.cmd run bootstrap:testing` before the final `npm.cmd run verify`.
- Rollback the source migration by reverting the Task 2, Task 3, and Task 4 commits. Task 1 is a behavior-neutral fixture refactor and may remain.

## File Map

- Modify `src-tauri/Cargo.toml`: replace four Codeberg specifications with exact crates.io requirements.
- Modify `src-tauri/Cargo.lock`: resolve all eight Grammers packages from crates.io.
- Modify `scripts/telegram-grammers-feature-baseline.mjs`: generate schema v2.
- Regenerate `src/lib/telegram-grammers-feature-baseline.json`: store canonical direct policy and resolved identity.
- Modify `scripts/testing/repository-rules.mjs`: consume `directPackages`, enforce direct `req`, and remove duplicate source logic.
- Modify `scripts/testing/repository-rules.test.ts`: model eight metadata packages and add policy mutations. Grammers-to-Grammers resolve edges remain empty because no evaluator reads them.
- Modify `docs/project.md`: describe crates.io as current source and baseline ownership of the full graph.
- Preserve `docs/superpowers/verification/2026-08-11-grammers-crates-io-provenance.md`.

## Rust Verification Loops

Affected package: `extractum-telegram`. No Rust source is expected to change.

- Dependency RED: after the manifest/lock switch and before generator changes, `node scripts/telegram-grammers-feature-baseline.mjs --check` must fail on the old Codeberg identity.
- Policy RED: after schema-v2 fixture/mutations and before generator/rule changes, `npm.cmd run test:related -- scripts/testing/repository-rules.test.ts` must fail with a non-empty selection.
- Policy GREEN: rerun the related test and baseline `--check` after implementation.
- Focused Rust check: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
- Package checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
- Feature-off proof: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`.
- End-of-slice gate: `npm.cmd run verify`.

---

### Task 1: Expand the Synthetic Package Inventory Without Changing Policy

**Files:**
- Modify: `scripts/testing/repository-rules.test.ts:89-189`
- Test: `scripts/testing/repository-rules.test.ts`

**Interfaces:**
- Consumes: current schema-v1 baseline and Codeberg source.
- Produces: `grammersResolvedVersions`, `grammersSource`, `grammersPackageId`, eight package records, and eight nodes with current direct feature data.

- [ ] **Step 1: Define the final eight-package inventory above the baseline**

Insert above `grammersBaseline`:

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

const grammersRevision = "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981";
const grammersSource =
  `git+https://codeberg.org/Lonami/grammers?rev=${grammersRevision}#${grammersRevision}`;
const grammersPackageId = (name: string, version: string) =>
  `${name} ${version} (${grammersSource})`;
```

Use `revision: grammersRevision` in the existing schema-v1 baseline.

- [ ] **Step 2: Build all eight package records once**

Replace the four-record `grammers` construction with:

```ts
const grammers = Object.entries(grammersResolvedVersions).map(([name, version]) => {
  const directPolicy = grammersBaseline.packages.find((entry) => entry.name === name);
  return {
    id: grammersPackageId(name, version),
    name,
    version,
    source: grammersSource,
    features: Object.fromEntries((directPolicy?.universe ?? []).map((feature) => [feature, []])),
    targets: [{ kind: ["lib"], name: name.replaceAll("-", "_") }],
    dependencies: [],
  };
});
```

Keep the producer connected only to the existing four direct packages.

- [ ] **Step 3: Create eight resolved nodes without unused transitive edges**

Replace the current Grammers node mapping with:

```ts
...grammers.map((entry) => ({
  id: entry.id,
  features: [
    ...(grammersBaseline.packages.find(({ name }) => name === entry.name)?.required ?? []),
  ],
  deps: [],
})),
```

No current or planned evaluator reads Grammers-to-Grammers `deps`; do not model them.

- [ ] **Step 4: Verify the fixture-only refactor remains GREEN**

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
```

Expected: PASS with a non-empty Vitest selection.

- [ ] **Step 5: Commit the fixture refactor**

```powershell
git diff --check
git diff -- scripts/testing/repository-rules.test.ts
git add -- scripts/testing/repository-rules.test.ts
git commit -m "test: model complete grammers package inventory"
```

Expected: only the synthetic metadata builder changes.

---

### Task 2: Switch Cargo and Specify Schema-v2 Policy (RED)

**Files:**
- Modify: `src-tauri/Cargo.toml:12-15`
- Modify: `src-tauri/Cargo.lock`
- Modify: `scripts/testing/repository-rules.test.ts:89-307`
- Test: `scripts/testing/repository-rules.test.ts`

**Interfaces:**
- Consumes: Task 1 inventory helpers.
- Produces: crates.io lock resolution plus a failing schema-v2 fixture contract for Task 3.

- [ ] **Step 1: Replace the four direct Git declarations**

```toml
grammers-client = { version = "=0.10.0", default-features = false }
grammers-mtsender = { version = "=0.10.0" }
grammers-session = { version = "=0.10.0", default-features = false, features = ["serde"] }
grammers-tl-types = { version = "=0.10.0", features = ["deserializable-functions"] }
```

- [ ] **Step 2: Resolve the lockfile and compile the complete package target**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
```

Expected: Cargo updates `src-tauri/Cargo.lock` and compilation passes. Stop and re-plan on any API incompatibility.

- [ ] **Step 3: Capture dependency-source RED before touching the generator**

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --check
```

Expected: FAIL because schema-v1 generation still requires Codeberg metadata. Record the exact failure in the task report.

- [ ] **Step 4: Change the synthetic fixture to schema v2**

Replace the Git helpers with:

```ts
const grammersSource = "registry+https://github.com/rust-lang/crates.io-index";
const grammersPackageId = (name: string, version: string) =>
  `${grammersSource}#${name}@${version}`;
```

Remove `grammersRevision`. Rename `grammersBaseline.packages` to
`grammersBaseline.directPackages`, preserving the current four synthetic
feature records, and add:

```ts
resolvedPackages: Object.entries(grammersResolvedVersions).map(([name, version]) => ({
  name,
  version,
  source: grammersSource,
})),
```

Set `schemaVersion: 2`. In the package/node builders, change only
`grammersBaseline.packages` to `grammersBaseline.directPackages`; keep the
Task 1 builder and empty node `deps`.

- [ ] **Step 5: Model exact direct manifest requirements**

Add `req` and replace the existing conditional `source` property in each
`normalProducerDependencies` record with:

```ts
req: String(name).startsWith("grammers-") ? "=0.10.0" : "*",
source: String(name).startsWith("grammers-") ? grammersSource : null,
```

- [ ] **Step 6: Add source, version, and requirement mutations**

Replace the old revision mutation under
`rule:telegram-crate-dependency-ownership` with:

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

Rename reordered-baseline access and assertion from `packages` to
`directPackages`.

- [ ] **Step 7: Prove the schema-v2 contract is RED**

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
```

Expected: FAIL with a non-empty selection because production still generates schema v1 and does not enforce dependency `req`.

- [ ] **Step 8: Commit the intentional RED checkpoint**

```powershell
git diff --check
git add -- src-tauri/Cargo.toml src-tauri/Cargo.lock scripts/testing/repository-rules.test.ts
git diff --cached --check
git commit -m "test: specify grammers crates.io policy"
```

Expected: this checkpoint intentionally leaves the focused repository-rule test RED; Task 3 must immediately implement the policy before broader gates.

---

### Task 3: Implement Schema-v2 Generation and Repository Policy (GREEN)

**Files:**
- Modify: `scripts/telegram-grammers-feature-baseline.mjs:11-244`
- Modify: `scripts/testing/repository-rules.mjs:205-256`
- Regenerate: `src/lib/telegram-grammers-feature-baseline.json`
- Test: `scripts/testing/repository-rules.test.ts`

**Interfaces:**
- Consumes: Task 2 schema-v2 fixture and real crates.io metadata.
- Produces: canonical `{schemaVersion, directPackages, resolvedPackages}` and exact direct requirement enforcement.

- [ ] **Step 1: Update generator metadata and baseline types**

Use these definitions:

```js
/**
 * @typedef {object} CargoMetadataPackage
 * @property {string} id
 * @property {string} name
 * @property {string} version
 * @property {string | null} [source]
 * @property {Record<string, unknown>} features
 */

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

Remove the `revision` and Git `exactSource` constants.

- [ ] **Step 2: Generate complete resolved identity**

Rename the existing four-package feature result from `packages` to
`directPackages`. Remove the direct package source assertion and add:

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

Return:

```js
return {
  schemaVersion: 2,
  directPackages,
  resolvedPackages,
};
```

Keep direct-package inventory and feature calculations unchanged apart from the rename.

- [ ] **Step 3: Update the repository rule**

Read direct policy from `baseline.directPackages`. Add:

```js
for (const dependency of producerGrammers) {
  if (dependency.req !== "=0.10.0") {
    violations.push(`${dependency.name}: direct manifest requirement must be =0.10.0`);
  }
}
```

Delete the redundant source check:

```js
const expectedSource = `git+https://codeberg.org/Lonami/grammers?rev=${baseline.revision}#${baseline.revision}`;
if (selected.source !== expectedSource) violations.push(`${expected.name}: source revision drifted`);
```

Inventory, versions, and sources now flow through generated baseline plus
`sameJson`.

- [ ] **Step 4: Regenerate the canonical artifact**

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --write
```

Expected: schema version 2, four complete `directPackages`, eight
`resolvedPackages`, and no `revision`, top-level `version`, or top-level
`packages` field.

- [ ] **Step 5: Run policy GREEN checks**

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
node scripts/telegram-grammers-feature-baseline.mjs --check
```

Expected: both PASS; Vitest selection is non-empty and exercises all three new mutations.

- [ ] **Step 6: Review the generated artifact and required-feature policy**

```powershell
git diff -- src/lib/telegram-grammers-feature-baseline.json
```

Confirm from the diff:

- `schemaVersion` changes from `1` to `2`;
- `revision` is removed;
- `packages` is renamed to `directPackages`;
- none of the four `required` arrays changes;
- `resolvedPackages` contains exactly eight expected records and the canonical registry source.

Stop and investigate if any `required` array changes.

- [ ] **Step 7: Print independent metadata evidence without duplicating policy assertions**

```powershell
$metadata = cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 |
  ConvertFrom-Json
$grammers = $metadata.packages |
  Where-Object { $_.name -like "grammers-*" } |
  Sort-Object name
$producer = $metadata.packages | Where-Object name -eq "extractum-telegram"
$grammers | Select-Object name, version, source | Format-Table -AutoSize
$producer.dependencies |
  Where-Object { $_.name -like "grammers-*" } |
  Select-Object name, req, source |
  Format-Table -AutoSize
```

Expected report evidence: eight resolved rows and four direct requirement rows. Enforcement remains owned by the passing checks in Step 5.

- [ ] **Step 8: Review lockfile scope and commit GREEN implementation**

```powershell
git diff --check
git diff -- src-tauri/Cargo.lock scripts/telegram-grammers-feature-baseline.mjs scripts/testing/repository-rules.mjs src/lib/telegram-grammers-feature-baseline.json
git add -- scripts/telegram-grammers-feature-baseline.mjs scripts/testing/repository-rules.mjs src/lib/telegram-grammers-feature-baseline.json
git diff --cached --check
git commit -m "build: enforce grammers crates.io graph"
```

Expected: generator, rule, and generated artifact are GREEN; lockfile review shows only registry migration closure from Task 2.

---

### Task 4: Update Current Dependency Documentation

**Files:**
- Modify: `docs/project.md:198-226`
- Preserve: `docs/superpowers/verification/2026-08-11-grammers-0-10-upgrade.md`
- Preserve: `docs/superpowers/verification/2026-08-11-grammers-crates-io-provenance.md`

**Interfaces:**
- Consumes: Task 3 exact direct requirements and schema-v2 graph.
- Produces: current project policy distinguishing the direct line from the resolved baseline.

- [ ] **Step 1: Replace only current-state policy**

Replace the active Git-dependency statement with:

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
`superpowers/verification/2026-08-11-grammers-crates-io-provenance.md` using the existing relative-link style. Preserve historical GitHub/Codeberg text and verification records.

- [ ] **Step 2: Run focused policy checks**

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --check
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
git diff --check
```

Expected: all PASS; Vitest selection is non-empty.

- [ ] **Step 3: Commit documentation**

```powershell
git diff -- docs/project.md
git add -- docs/project.md
git diff --cached --check
git commit -m "docs: document grammers crates.io policy"
```

Expected: only current-state policy changes.

---

### Task 5: Run Package and Workspace Completion Gates

**Files:**
- Modify only if checkbox tracking is committed: `docs/superpowers/plans/2026-08-11-grammers-crates-io-0-10.md`
- Verify: all Task 1-4 files

**Interfaces:**
- Consumes: committed registry migration and documentation.
- Produces: final completion evidence with a clean worktree.

- [ ] **Step 1: Run the package checkpoint**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
```

Expected: PASS with a non-zero test count.

- [ ] **Step 2: Prove the feature-off production surface**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features
```

Expected: PASS.

- [ ] **Step 3: Run the full workspace gate**

If this is a fresh checkout/worktree, first run:

```powershell
npm.cmd run bootstrap:testing
```

Then run:

```powershell
npm.cmd run verify
```

Expected: PASS. Diagnose any failure with `systematic-debugging`; do not report completion without an authoritative full GREEN run.

- [ ] **Step 4: Run final generated-state and worktree checks**

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --check
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 | Out-Null
git diff HEAD --check
git status --short
```

Expected: checks PASS and status is empty except intentional plan checkbox tracking.

- [ ] **Step 5: Commit plan tracking only if changed**

```powershell
git add -- docs/superpowers/plans/2026-08-11-grammers-crates-io-0-10.md
git diff --cached --check
git commit -m "docs: record grammers crates.io verification"
```

Do not create an empty commit if the plan file did not change.

- [ ] **Step 6: Report completion evidence**

Report implementation commit hashes, eight resolved names/versions/sources,
unchanged `directPackages[*].required`, focused policy results, package and
feature-off results, final `npm.cmd run verify`, and clean status.
