# Rust Wave 0 Final Hardening Design Addendum

**Date:** 2026-08-14

**Status:** Approved correction design; ready for implementation planning

**Amends:** `docs/superpowers/specs/2026-08-12-rust-infrastructure-upgrade-design.md`

## Purpose and Readiness Boundary

This addendum closes the Important and Minor findings in
`.superpowers/sdd/wave-0-final-review.md` without reopening the Rust
infrastructure program. The original design remains authoritative where this
addendum is silent. Where the documents conflict, this addendum controls the
Wave 0 correction.

The correction has three goals:

1. make every claimed local and CI policy executable from a fresh checkout
   without ambient tools or bypassable manifest/range checks;
2. make duplicate and advisory handling fail for the right reason while
   retaining the designed review-only paths; and
3. close the bounded residual source, license, command, and documentation
   findings before the branch is presented for remote execution.

The branch is not ready to merge at the start of this correction. It becomes
**locally ready to push for remote evidence** only when the complete correction
range has passed the focused and full local matrix in this document and a
fresh whole-range review has no Important or Critical findings. It becomes
**merge-ready** only after a later, user-authorized push produces green Rust
Fast and Rust Full runs for the reviewed head. It is not release-ready while
the advisory scan is red or until a successful gated bundle run supplies MSI,
NSIS, application-smoke, sidecar-smoke, and hash evidence.

## Non-Goals

This correction does not:

- upgrade Rust, Edition 2021, Cargo dependencies, npm dependencies, or the
  cargo-deny pin;
- resolve or suppress current RustSec findings;
- change product behavior, IPC commands, persisted data, migrations, public
  JSON, or frontend contracts;
- rewrite the original design or restructure the completed Wave 0 plan;
- add a second cargo-deny installer, a global install, or an ambient `PATH`
  prerequisite;
- automatically regenerate policy artifacts during a check;
- push a branch, open a pull request, dispatch Actions, publish a release, or
  claim remote evidence during implementation.

## Correction Architecture

The corrected system has four explicit boundaries:

1. a repository-owned PowerShell wrapper provides the pinned cargo-deny binary
   to local npm commands and the existing composite action;
2. repository rules inspect manifest syntax and compare a generated canonical
   inventory of every direct Cargo edge;
3. the duplicate CLI first proves that the active graph exactly equals the
   current baseline, then optionally compares that baseline with an explicit
   base commit; and
4. the release workflow separates read-only advisory execution from the only
   job allowed to write an issue.

All checks are read-only. There are exactly two developer-only policy writers:
`node scripts/rust-dependency-policy.mjs --write
scripts/testing/rust-dependency-policy.json` and `node
scripts/rust-duplicate-baseline.mjs --write
scripts/testing/rust-duplicate-baseline.json`. Neither command is invoked by a
verification alias, test setup, local acceptance sequence, or workflow.

## Pinned Cargo-Deny for Local and CI Use

### Wrapper contract

Add `scripts/run-cargo-deny.ps1` as the sole operator-facing cargo-deny entry
point. It supports exactly these modes:

- `-Mode Setup` installs/verifies the tool but runs no policy;
- `-Mode Deterministic` runs `bans licenses sources`;
- `-Mode Advisories` runs `advisories`.

`.github/tools/cargo-deny.json` remains schema 1 and contains both the existing
release-archive `sha256` and this exact extracted-executable digest:

```json
"binarySha256": "f7292fab58c706638c999e64c4ba82e5128ae628130ba55e3266a768ee431fbf"
```

The wrapper reads version, target, archive hash, and binary hash from that
manifest and uses the stable ignored directory
`src-tauri/target/.extractum-tools/cargo-deny/0.20.2-x86_64-pc-windows-msvc`.
It does not
read a user Cargo bin directory, inspect a global cargo-deny installation, or
run `cargo install`. Before any cached executable is run, the wrapper hashes
the file and requires exact equality with `binarySha256`. Only an
already-authenticated executable receives the mandatory `--version` check for
exact output `cargo-deny 0.20.2`; both digest and version must pass on every
cache hit before policy execution.

Missing or digest/version-mismatched cache content causes the wrapper to call
`scripts/setup-cargo-deny.ps1` for that exact directory. The setup script
downloads and verifies the archive hash, extracts into its staging directory,
hashes the extracted `cargo-deny.exe` against `binarySha256` **before** any
version/self-check execution, then mandatorily runs the exact version check and
publishes the digest-and-version-authenticated executable transactionally.
Digest-rejected cached or freshly extracted candidates are never executed. A
digest-authenticated candidate whose mandatory `--version` result is wrong is
executed only for that version probe: it is never published, cached as valid,
or run with Deterministic, Advisories, or any other policy arguments. A failed
reinstall restores the previous authenticated executable, if one existed, but
the current invocation still fails.

After setup or an authenticated cache hit, Deterministic and Advisories modes
invoke that `cargo-deny.exe` by absolute path with explicit root `deny.toml` and
`src-tauri/Cargo.toml`; they never call `cargo deny` and never fall back to an
ambient executable. The wrapper propagates the policy exit code. `-Mode Setup`
may accept `-AddToGitHubPath`; the other modes reject that switch. Any temporary
environment or `GITHUB_PATH` change is rolled back on setup failure.

The npm aliases become thin wrapper calls. `check:rust:supply-chain` uses
`Deterministic`, `check:rust:advisories` uses `Advisories`, and
`check:rust:fast` continues to compose the deterministic alias. The composite
action calls the wrapper in `Setup` mode with `-AddToGitHubPath`, so local and
CI paths select the same cache layout and existing verified installer.

The documented fresh-checkout sequence remains:

```powershell
npm.cmd ci
npm.cmd run bootstrap:testing
npm.cmd run check:rust:fast
npm.cmd run verify
```

It now self-bootstraps cargo-deny. A contract test must prove that README,
`docs/project.md`, and `AGENTS.md` point to the wrapper-backed alias and do not
instruct a global cargo-deny install.

### Failure and rollback

A failed download, archive digest, extracted-binary digest, extraction, or
version check leaves a previously authenticated cached binary in place through
the setup transaction but still fails the current invocation. A failed first
install leaves no executable accepted as valid. The wrapper never executes a
candidate to establish its integrity and never deletes the shared Cargo target
or user files. Retrying the same alias is the recovery path. Reverting the
wrapper commit restores the prior command wiring without touching globally
installed software because none is installed.

## Manifest-Syntax and Direct-Dependency Policy

### Syntax-owned toolchain inheritance

Cargo metadata remains a secondary effective-value check, not evidence of
inheritance. The repository index must load the root manifest and every member
manifest named by the canonical seven-package policy. A section-aware TOML
reader parses table headers, dotted keys, strings, booleans, and comments for
the relevant tables; a whole-file regular expression is not an acceptable
parser.

The rule requires all of the following simultaneously:

- root `[workspace.package]` owns exactly `rust-version = "1.95"` and
  `edition = "2021"`;
- every member's `[package]` table, including root `extractum`, contains
  `rust-version.workspace = true` and `edition.workspace = true`;
- no member substitutes literal `rust-version` or `edition` values, even when
  they equal the workspace value;
- all seven package names and manifest paths correspond to Cargo metadata and
  the canonical workspace inventory;
- effective metadata still reports Rust 1.95, Edition 2021, and unpublished
  state for all seven packages; and
- root `rust-toolchain.toml` remains the exact canonical artifact.

Missing root ownership, duplicate relevant keys/tables, invalid values, a
same-value hard-coded member, an extra/missing workspace package, or a manifest
outside the repository is an infrastructure violation.

### Canonical direct requirement inventory

Bump `scripts/testing/rust-dependency-policy.json` to `schemaVersion: 2` and
replace name-only policy maps with a generated, sorted `directRequirements`
array covering every direct normal, build, and dev dependency edge in every
workspace package. `scripts/rust-dependency-policy.mjs` generates the complete
schema-2 candidate, prints it by default, and writes it only with explicit
`--write scripts/testing/rust-dependency-policy.json`. Each direct entry has
this exact shape:

```json
{
  "package": "extractum",
  "dependency": "tauri-build",
  "rename": null,
  "kind": "build",
  "target": null,
  "requirement": "^2"
}
```

`kind` is exactly `normal`, `build`, or `dev`; `rename` and `target` are
explicit strings or `null`. The identity key is
`package/dependency/rename/kind/target`. Entries sort by those five fields.
`requirement` is Cargo metadata's normalized requirement string. This exact
inventory, rather than a first-number extractor, is the allowed language: a
manifest change from `^2` to `^2 || ^3`, or from `^0.11` to `>=0.11`, changes
the generated entry and fails until a dependency wave deliberately updates the
policy.

The schema-2 root contains exactly `schemaVersion`, `toolchain`,
`directRequirements`, `npmRequirements`, `exactPins`,
`approvedPrereleases`, and `tauriFamily`.
The comparison is bidirectional. Every generated edge has exactly one policy
entry, every policy entry has exactly one generated edge, and duplicate policy
identities are invalid. Dev dependencies are never filtered out.

`npmRequirements` is the separate canonical, sorted inventory of every direct
root `package.json` edge whose name starts with `@tauri-apps/`. Each entry has
exactly this shape:

```json
{
  "owner": "extractum",
  "name": "@tauri-apps/cli",
  "kind": "devDependencies",
  "requirement": "^2"
}
```

`kind` is exactly `dependencies`, `devDependencies`, or
`optionalDependencies`; the identity is `owner/name/kind`, sorted by those
fields. Generation reads all three root maps and emits every matching edge.
Comparison is bidirectional and rejects a missing, extra, duplicate, moved, or
requirement-mismatched entry. Thus `^2 || ^3`, `>=2`, moving CLI from
`devDependencies` to `dependencies`, deleting a pair, or adding an unowned
`@tauri-apps/*` package all fail until the reviewed schema-2 policy is
deliberately regenerated and committed.

Exact-pin and prerelease ownership are also arrays, keyed to an exact direct
edge rather than a dependency name alone:

```json
{
  "package": "extractum-telegram",
  "dependency": "grammers-client",
  "rename": null,
  "kind": "normal",
  "target": null,
  "requirement": "=0.10.0"
}
```

```json
{
  "package": "extractum",
  "dependency": "apalis",
  "rename": null,
  "kind": "normal",
  "target": null,
  "version": "1.0.0-rc.8"
}
```

Every manifest requirement whose complete normalized range is one exact
version must have one matching `exactPins` entry, and every `exactPins` entry
must still identify an exact manifest edge. Every direct range containing a
prerelease comparator must have one matching `approvedPrereleases` entry for
that exact package edge and prerelease version, and every approval must remain
in use. An exact prerelease therefore appears in both arrays. Stable exact pins
and prereleases introduced in normal, build, or dev dependencies without these
owning-package entries fail.

`tauriFamily` contains exactly `pairs` and `cargoOnlyRequirements` arrays.
Every `tauriFamily.pairs` entry has exactly five fields: stable `id`, Cargo
identity, reviewed `cargoRequirement`, npm identity, and reviewed
`npmRequirement`. It has this exact object schema:

```json
{
  "id": "tauri-build-cli",
  "cargo": {
    "package": "extractum",
    "dependency": "tauri-build",
    "rename": null,
    "kind": "build",
    "target": null
  },
  "cargoRequirement": "^2",
  "npm": {
    "owner": "extractum",
    "name": "@tauri-apps/cli",
    "kind": "devDependencies"
  },
  "npmRequirement": "^2"
}
```

Pair IDs and both identity references are unique. Each reference resolves
exactly one canonical inventory entry, and the evaluator requires the resolved
Cargo/npm `requirement` to equal the pair's reviewed requirement string
exactly. There is no `cargoMajor`, `npmMajor`, range parsing, or first-number
fallback.

The complete reviewed Wave 0 pair set is:

| Pair ID | Cargo identity (`package/dependency/rename/kind/target`) | Cargo requirement | npm identity (`owner/name/kind`) | npm requirement |
| --- | --- | --- | --- | --- |
| `tauri-api` | `extractum/tauri/null/normal/null` | `^2` | `extractum/@tauri-apps/api/dependencies` | `^2` |
| `tauri-build-cli` | `extractum/tauri-build/null/build/null` | `^2` | `extractum/@tauri-apps/cli/devDependencies` | `^2` |
| `tauri-dialog` | `extractum/tauri-plugin-dialog/null/normal/null` | `^2` | `extractum/@tauri-apps/plugin-dialog/dependencies` | `^2` |
| `tauri-opener` | `extractum/tauri-plugin-opener/null/normal/null` | `^2` | `extractum/@tauri-apps/plugin-opener/dependencies` | `^2` |
| `tauri-sql` | `extractum/tauri-plugin-sql/null/normal/null` | `^2` | `extractum/@tauri-apps/plugin-sql/dependencies` | `^2.4.0` |

The governed Cargo set is independently discoverable: root `extractum` direct
edges named `tauri`, `tauri-build`, or starting `tauri-plugin-`, excluding the
separately reviewed Cargo-only `tauri-plugin-mcp-bridge` edge. Every governed
Cargo edge and every `npmRequirements` edge appears in exactly one pair. Every
pair points to one member of each governed set. A deleted, duplicated, orphaned,
missing, moved, Cargo-widened, or npm-widened pair/edge is a violation. The MCP
bridge Cargo-only record separately stores its exact Cargo identity and
reviewed `^0.11` requirement:

```json
{
  "id": "tauri-plugin-mcp-bridge",
  "cargo": {
    "package": "extractum",
    "dependency": "tauri-plugin-mcp-bridge",
    "rename": null,
    "kind": "normal",
    "target": null
  },
  "cargoRequirement": "^0.11"
}
```

`cargoOnlyRequirements` currently contains exactly this one record and no
other. Each entry is an exact three-field object `{id, cargo,
cargoRequirement}`; extra fields are invalid. Its stable `id` and exact
five-field Cargo identity are each unique within the array. The Cargo-only
governed set is independently discoverable and currently contains exactly the
root `extractum` direct dependency whose name is
`tauri-plugin-mcp-bridge`. Every member of that set appears in exactly one
`cargoOnlyRequirements` entry, and every entry resolves to exactly one member
of that set. The resolved canonical direct requirement must equal the reviewed
`cargoRequirement` string exactly. An empty array, missing record, duplicate
ID or Cargo identity, orphaned record, extra record or field, multiply
referenced edge, or widened requirement is a violation.

Policy generation is an explicit developer operation for dependency waves and
is never part of ordinary verification.
`--write` regenerates direct Cargo/npm inventories and exact/prerelease
candidates, but copies the committed pair and Cargo-only IDs, identities, and
reviewed requirement strings without changing them. It reports unresolved and
candidate pair or Cargo-only differences and leaves the regenerated artifact
failing until a reviewer deliberately edits the affected reviewed
values/identity. It never infers approval from a new manifest or package
requirement and never silently adds, removes, widens, or rewrites the reviewed
Cargo-only record. The resulting diff receives the same dependency-owner
review as the manifest/package change, and the ordinary checker never invokes
the writer.

## Diff-Aware Windows Duplicate Policy

### Current-state invariant

Extend the existing duplicate CLI with read-only `--check` and optional
local `--base` followed by one full or abbreviated commit ID. One shared
semantic validator is applied without weakening to both current and historical
baseline artifacts. It requires the exact top-level field set and all of these
invariants:

- `schemaVersion` is exactly `1`;
- `target` is exactly `x86_64-pc-windows-msvc`;
- `duplicateCardinality` is an alphabetically sorted object whose values are
  integers greater than one;
- `duplicateNameCount` equals the number of mapping keys;
- `duplicateVersionInstanceCount` equals the arithmetic sum of mapping values;
  and
- no additional top-level field or unsorted/noncanonical mapping is accepted.

After that identical semantic validation, the current baseline receives one
additional live-state assertion: every counter and mapping entry equals the
freshly generated active graph. That equality rejects stale ceiling slack after
a reduction; the historical baseline is compared to the current baseline, not
to the current live graph.

A shared structural/attachment validator is applied identically to current and
historical `scripts/testing/rust-supply-chain-exceptions.json`. It requires the
exact schema-1 top-level fields `schemaVersion`, `licenseExceptions`,
`advisoryExceptions`, and `duplicateGrowthExceptions`; the three exception
values must be arrays. Every duplicate-growth entry has the exact six-field
shape below, packages are unique and canonically sorted, counts are integers
with `approvedCount > previousCount >= 1`, owner/reason are nonempty, and
`reviewAfter` is syntactically valid RFC 3339 UTC. The validator receives the
corresponding validated baseline and requires that baseline's cardinality for
every present entry equal its `approvedCount`.

Expiry is a separate time/transition rule. Present current entries must have a
future `reviewAfter`; current-only mode therefore rejects any expired current
entry. Historical entries may already be expired when read from a base commit:
they still must pass the identical structure, sorting, uniqueness, date syntax,
and historical-baseline attachment checks, while their expiry controls which
transition is legal below. Neither mode invents a requirement for elevated
packages that have no exception in its artifact.

The ordinary local `check:rust:fast` alias runs `--check` without a base and
therefore validates the active graph, current baseline, and every present
current exception. It does not infer branch history.

### Explicit base comparison

Dependency waves run:

```powershell
$BaseSha = git rev-parse origin/main
npm.cmd run check:rust:duplicates -- --base $BaseSha
```

Local execution accepts historical comparison only through `--base`; it does
not read a local environment variable, guess `HEAD~1`, or calculate a merge
base. It is rejected when `GITHUB_ACTIONS=true` and is mutually exclusive with
either duplicate-base environment variable. The checker verifies that the
argument resolves to a commit and reads both
`scripts/testing/rust-duplicate-baseline.json` and
`scripts/testing/rust-supply-chain-exceptions.json` from that commit with
`git show`. An absent commit/artifact or any historical semantic-validation
failure fails closed.

Every GitHub job that can invoke duplicate checking sets
`RUST_DUPLICATE_BASE_MODE` explicitly:

- Rust Fast sets `required`, uses `fetch-depth: 0`, and supplies
  `RUST_DUPLICATE_BASE_SHA` as `github.event.pull_request.base.sha` for a pull
  request or `github.event.before` for a push to `main`;
- Rust Full and the release bundle set `off` and do not define
  `RUST_DUPLICATE_BASE_SHA`.

Under `GITHUB_ACTIONS=true`, missing/unknown mode fails. `required` rejects a
missing, all-zero, or unresolvable SHA; `off` rejects a present SHA. Both modes
reject CLI `--base`. Locally, either environment variable is rejected and the
operator must use `--base`. These precedence rules prevent Fast from silently
degrading to current-only and prevent Full/bundle from accidentally comparing
an unrelated base.

### Change classification and exceptions

After current-state equality and both base validations succeed, the checker
computes a deterministic per-package delta map before classifying the change.
Absent mapping entries have cardinality one for growth arithmetic.

- Identical baseline state passes silently; carried identical active exception
  entries remain valid when current cardinality equals `approvedCount`.
- A pure reduction has at least one negative delta, no added duplicated name,
  and no positive per-name delta. It passes and reports the lowered entries.
- A true review-only replacement requires both aggregate counters to be
  unchanged, a changed package set, unchanged cardinality for every common
  package name, and equality between the sorted multiset of removed
  cardinalities and the sorted multiset of added cardinalities. It emits a
  deterministic nonblocking review notice and needs no growth exception.
- Every other new duplicated name or positive per-name delta requires exactly
  one valid current growth exception. A surviving-name increase or unmatched
  added cardinality therefore cannot hide behind a flat or declining aggregate.

For example, base `{ alpha: 3, gamma: 2 }` to current
`{ beta: 2, gamma: 3 }` is not review-only because common `gamma` increased;
`gamma` needs an exception and the unmatched added `beta` also needs one.
An aggregate decrease with any surviving-name increase follows the same growth
path rather than the pure-reduction path.

Each `duplicateGrowthExceptions` entry has exactly this shape and no additional
fields:

```json
{
  "package": "windows-sys",
  "previousCount": 5,
  "approvedCount": 6,
  "owner": "desktop-platform",
  "reason": "Upstream Tauri graph requires both incompatible lines.",
  "reviewAfter": "2026-09-14T00:00:00Z"
}
```

For a new positive-growth entry, `previousCount` is the base cardinality, using
`1` when the package was not duplicated, and `approvedCount` is the current
cardinality. Package/previous/approved values must exactly match the applicable
transition. `owner` and `reason` must be nonempty after trimming.
`reviewAfter` is an RFC 3339 UTC instant ending in `Z`; an entry is expired
when the current UTC instant is greater than or equal to it. Tests inject the
clock; local and CI execution use the real UTC clock.

Exception lifecycle comparison is an explicit state machine. For a base entry
`{ previousCount: P, approvedCount: A }`, historical attachment proves the base
cardinality is `A`; let current cardinality be `C`:

- If `C == A`, current must carry the identical unexpired entry or perform a
  metadata-only renewal with unchanged package/P/A, nonempty ownership/reason,
  and `reviewAfter` strictly later than the base value and in the future.
  Renewal is allowed whether the base entry is active or expired. An expired
  base entry cannot be carried unchanged, so renewal is mandatory at `C == A`.
- If `P < C < A`, this is a partial negative-delta reduction. Current must
  replace the entry with `{ previousCount: P, approvedCount: C }`, retain
  nonempty ownership/reason, and use a future `reviewAfter`; metadata may be
  updated. This is not evaluated as positive growth.
- If `C <= P`, this is a full reduction to or below the pre-exception level.
  Current must remove the entry. Keeping it or replacing it is stale.
- If `C > A`, this is positive growth from the previously approved level.
  Only an unexpired base entry may transition to
  `{ previousCount: A, approvedCount: C }`; an expired base entry must first be
  renewed in a separate reviewed transition and cannot authorize more growth.

When no base entry exists, a new current entry is legal only for an actual
positive per-name delta outside the true replacement predicate, with
`previousCount` equal to base cardinality and `approvedCount` equal to current
cardinality. A valid-looking entry added on an unchanged baseline is invented
and fails. A balanced true review-only replacement must have no new exception;
attaching one to any added replacement package fails. A missing exception fails
only an actual positive delta that requires one.

Present duplicate, expired-current, malformed, attachment-mismatched,
invalidly renewed/reduced, invented, or impermissibly removed entries fail.
Exceptions never permit active-graph/current-baseline drift. These rules allow
persisted approvals to survive ordinary PRs, pushes, Full runs, and bundle runs
while forcing partial reductions to lower their approval and expiry explicitly.

The CLI returns exit code `0` for clean, reduction, and review-only results and
nonzero for infrastructure or policy violations. Its human output separates
`VIOLATION`, `REVIEW`, and `REDUCTION`; tests assert the structured result so a
review annotation cannot make the real-snapshot unit test fail.

## Advisory Execution and Least Privilege

Rename the read-only job conceptually to `advisory-scan`. It has only
`contents: read`. Checkout, toolchain inspection, pinned cargo-deny setup, and
the advisory command execute in this job without issue-write authority.

The advisory command step has `id: scan` and `continue-on-error: true`. The job
exports `scan_outcome: ${{ steps.scan.outcome }}`. A later explicit shell step
runs only when `steps.scan.outcome == 'failure'` and exits nonzero, restoring a
red job result. Therefore the bundle continues to use
`needs.advisory-scan.result == 'success'` and remains blocked by an actual
advisory failure.

A separate `advisory-issue-writer` job:

- has `needs: advisory-scan`;
- has `permissions: { issues: write }` and no checkout, cache, installer, shell,
  artifact, or repository-content step;
- owns `concurrency.group: rust-advisory-follow-up` with
  `cancel-in-progress: false`, preserving serialization around the non-atomic
  list-then-create/update issue operation;
- contains only the pinned `actions/github-script` step; and
- uses exactly the logical condition
  `always() && needs.advisory-scan.outputs.scan_outcome == 'failure'` plus the
  existing schedule-or-workflow-dispatch event restriction.

If checkout, toolchain setup, cargo-deny setup, or any pre-scan step fails, the
scan step has no `failure` outcome, the scan job is red, the bundle is blocked,
and the writer is skipped. Such failures must never open or update an issue
claiming that Rust advisories failed. A writer API failure makes the writer job
red but does not grant it access to build secrets or change the scan result.

## Immutable GitHub Actions

Every remote Action reference in all Rust workflows is pinned to the following
reviewed full commit SHA, with its mutable release line retained only as a
comment:

```yaml
actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6
actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38 # v6
actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4
actions/github-script@ed597411d8f924073f98dfc5c65a23a2325f34cd # v8
Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2
```

The contract inventory includes checkout, setup-node, upload-artifact,
github-script, and Swatinem/rust-cache and rejects tags, branches, shortened
SHAs, expressions, or unrecognized remote actions. Repository-local actions
remain relative references. An Action update is manual: review the upstream
release notes, resolve both the tag ref and its `^{}` annotated-tag peel, and
require the selected workflow SHA to be the peeled
object when the two differ. Verify with Git that the peeled object type is
`commit`, inspect the upstream commit page and `action.yml` at that commit, and
record the exact tag-object-to-commit evidence. Then update every occurrence
and the central reviewed commit map in one isolated commit and rerun workflow
mutation tests and the full local gate. The offline contract compares exact
owner/action names with that reviewed map; accepting an arbitrary 40-hex value
is forbidden. Automation may report a candidate but may not rewrite or merge
these refs.

For the accepted rust-cache v2 review, the evidence is exact: tag object
`49a0bdc70d2e1b713ca9e2869b211fcce03d3c1c` peels to commit
`6323deb102c322ba6fcbdcafc7e3dddab59af2b6`, and the latter is the only allowed
workflow/inventory value.

## Bounded Minor Corrections

The final correction slice makes exactly these additional changes:

1. Add `--locked` to `test:rust` and `test:rust:prompt-pack-runs`, and assert
   both exact aliases and both unlocked negative mutations in
   `scripts/verify.test.ts`; production verifier behavior is unchanged.
2. Remove the currently unused `Apache-2.0 WITH LLVM-exception` allowance and
   set both `unused-allowed-license = "deny"` and
   `unused-license-exception = "deny"`. The deterministic cargo-deny gate and
   separate unused-allowance/stale-exception mutations prove that neither form
   of bootstrap slack survives.
3. Correct only the stale Wave 0 prose: Task 2 RED stopped at the first failing
   assertion while final GREEN covered both; the prompt-pack exact test is
   `public_api_tests::cancellation_smoke_services_remain_test_only`; and the
   verification record distinguishes the original reviewed range through
   `9bcc047b9ec210c1ea34024f6dc782a1e7caec49` from the later hardening range.
   The original design is not edited.
4. Replace the sidecar self-round-trip assertion with a complete golden value.
   Serialization must equal a literal object containing `type`, every run
   result field, and all six artifact fields (including nulls). Deserialization
   starts independently from that literal and equals the expected boxed Rust
   value; it must not deserialize the serializer's output variable.
5. Remove the unnecessary clone of the configured LLM API key. A private
   result enum distinguishes a complete configured pair from partial
   overrides; `configured_provider_access` consumes
   `Option<SecretString>` and `Option<String>`, moves a complete key into
   `LlmProviderAccess`, and returns partial values by ownership for existing
   profile resolution. The key is never exposed, logged, serialized, converted
   back to `String`, or persisted, and key-only/base-only fallback behavior is
   unchanged. This is an explicit characterization-tested exception to the
   usual strict RED ownership-shape rule: runtime assertions cannot prove that
   a `SecretString` clone is absent. The existing complete/partial behavioral
   cases are recorded green before and after the refactor; clone removal is
   proved by the bounded diff review plus compiler, focused check, owner test,
   and Clippy evidence. No artificial failing runtime test is claimed.
6. Update `docs/value-registry.md` for every stable tooling-only value family
   introduced by this hardening slice: cargo-deny wrapper modes `Setup`,
   `Deterministic`, and `Advisories`; duplicate base modes `required` and
   `off`; Cargo direct-requirement kinds `normal`, `build`, and `dev`; npm
   direct-requirement kinds `dependencies`, `devDependencies`, and
   `optionalDependencies`; and duplicate result classifications `VIOLATION`,
   `REVIEW`, and `REDUCTION`. Each family records its script, workflow, or
   policy-artifact owner; checked-in repository/CI persistence; absence of
   product database, persistence API, and product UI impact (Actions log and
   notice rendering are operational output, not product UI); and its owning
   fixture and validator inventory. Existing registry families are extended
   rather than duplicated.

## Data Flow

### Deterministic local gate

1. `npm.cmd run check:rust:fast` runs formatting, Clippy, production-surface
   checks, current duplicate validation, and deterministic supply-chain policy.
2. Duplicate validation runs locked `cargo tree`, validates artifact structure,
   and requires exact current graph equality.
3. The cargo-deny wrapper validates/reuses the repository cache or calls the
   checksum-owning setup script, then executes the pinned binary.
4. No step writes a baseline, exception, lockfile, or global tool location.

### Dependency-wave review

1. The wave deliberately changes manifests/lockfile and regenerates canonical
   dependency and duplicate artifacts.
2. The operator supplies the reviewed pre-wave commit using `--base`.
3. The checker validates current equality, loads and semantically validates the
   base baseline and exception artifact, classifies per-name deltas, and
   validates current exception lifecycle against both base artifacts, current
   graph, and UTC clock.
4. Review-only replacement output is retained in the wave evidence; violations
   block acceptance.

### Advisory and bundle flow

1. The read-only scan job checks out the reviewed revision and runs pinned
   advisories.
2. The scan step outcome becomes a job output; an explicit final step converts
   only scan failure into job failure.
3. A scan failure blocks the bundle and, for scheduled/manual runs, enables the
   isolated issue writer through `always()`.
4. A setup/infrastructure failure blocks the bundle but cannot satisfy the
   writer condition.

## Failure Classification and Rollback

Policy and infrastructure failures fail closed and never repair committed
state automatically. A malformed manifest/policy artifact, current duplicate
mismatch, unavailable base commit, expired current exception, expired
historical exception carried unchanged, or unresolvable Action contract is a
violation, not a review notice. An expired historical exception is not
intrinsically malformed: it may transition only through future-dated metadata
renewal or the partial/full reduction and removal cases in the state table; it
cannot authorize `C > A` growth. The operator first determines
whether the source graph or the committed policy is intended; only a deliberate
dependency wave may regenerate and commit an artifact.

Review-only duplicate output is non-mutating and needs no rollback. If its
replacement is not acceptable, revert the dependency-wave manifest/lockfile
and regenerated baseline together. A growth exception is rolled back with the
growth it authorizes; a newly invented or stale exception is a failure, while
an unchanged active base/current approval is intentionally carried.

The advisory scanner and writer do not update dependency or exception files.
Retry infrastructure failures after restoring checkout/tool access. Resolve a
real advisory or commit a separately reviewed time-bounded advisory exception;
never make the issue writer green by weakening the scan. An Action-pin update
rolls back by reverting its isolated reviewed commit to the previously listed
full SHA.

The two Rust source corrections have no persistence or migration effect. Their
rollback unit is the bounded minor-fix commit, followed by the two exact tests
and owner checkpoints. No rollback path deletes caches, Cargo targets, secure
storage, user databases, or release artifacts.

## Authorized Implementation Scope

The implementation plan may modify only these categories:

- local tool orchestration: `scripts/run-cargo-deny.ps1`,
  `scripts/run-cargo-deny.test.ps1`,
  `scripts/setup-cargo-deny.ps1`, `.github/tools/cargo-deny.json`,
  `.github/actions/setup-cargo-deny/action.yml`, `package.json`, and
  `scripts/verify.test.ts` (the last file owns only the two locked-alias
  positive and negative contracts);
- executable policy: duplicate generator/checker and tests,
  `scripts/rust-dependency-policy.mjs` and its tests,
  `scripts/testing/rust-dependency-policy.json`,
  `scripts/testing/rust-duplicate-baseline.json`,
  `scripts/testing/rust-supply-chain-exceptions.json`, repository
  index/rules/tests and their registered-rule inventory;
- CI contracts: the three Rust workflow files and
  `scripts/testing/github-rust-workflows.test.ts`;
- license policy: `deny.toml`;
- bounded Rust corrections:
  `src-tauri/crates/extractum-gemini-browser/src/types.rs` and
  `src-tauri/src/llm/mod.rs` only;
- canonical operator and value-contract docs: `README.md`, `AGENTS.md`,
  `docs/project.md`, and `docs/value-registry.md`;
- stale factual corrections in
  `docs/superpowers/plans/2026-08-12-rust-infrastructure-wave-0.md` and
  `docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md`,
  plus the new factual record
  `docs/superpowers/verification/2026-08-14-rust-wave-0-final-hardening.md`.

No other Rust source, Cargo manifest, Cargo lockfile, frontend source,
migration, configuration schema, original design, or unrelated documentation
is authorized. If an implementation test demonstrates that another file is
necessary, implementation stops for a reviewed design amendment rather than
expanding scope implicitly.

## Verification Matrix

| Contract | Required mutations or negative evidence | Focused GREEN evidence | Slice/full evidence |
| --- | --- | --- | --- |
| Local cargo-deny wrapper | missing cache; fake executable with expected version text but wrong binary digest; correctly hashed wrong-version fixture; omitted cache-hit/staged version check; archive/binary checksum failure; ambient cargo-deny absent; transaction/environment restoration | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-cargo-deny.test.ps1`; two absolute-path invocations against the digest-and-version-authenticated stable cache; `npm.cmd run check:rust:supply-chain` | fresh-checkout documented sequence and `npm.cmd run check:rust:fast` |
| Manifest inheritance | replace one `rust-version.workspace = true` with literal `rust-version = "1.95"`; replace Edition inheritance with literal 2021; remove root workspace value; add duplicate relevant key | `npm.cmd run test:unit -- scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts` | `npm.cmd run test:unit`; `npm.cmd run check` |
| Direct requirements | Cargo `^2 || ^3`, `>=0.11`, new dev edge, unowned stable exact pin/prerelease, missing/stale/duplicate edge; npm `^2 || ^3`, `>=2`, moved kind, missing/extra edge; deleted pair, duplicate pair ID/identity, orphan pair, unpaired governed edge, Cargo widened against pair, npm widened against pair, and writer-generated candidate without reviewed pair update; empty/missing Cargo-only array, duplicate Cargo-only ID/identity, orphaned/extra-field/extra-record Cargo-only entry, widened MCP bridge requirement, and writer auto-approval instead of preserving the reviewed record | `npm.cmd run test:unit -- scripts/rust-dependency-policy.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts` over Cargo normal/build/dev and npm dependencies/devDependencies/optionalDependencies; writer fixture proves Cargo-only ID/identity/requirement preservation | `npm.cmd run test:unit`; real repository-rule snapshot |
| Duplicate structure/current state | malformed current and historical baseline/exception artifacts: extra fields, wrong schema/target, unsorted keys, invalid integers/arithmetic, duplicate or attachment-mismatched present exception, and expired current exception; stale reduction; graph/baseline drift | `npm.cmd run test:unit -- scripts/rust-duplicate-baseline.test.ts scripts/testing/repository-rules.test.ts`; `npm.cmd run check:rust:duplicates` | `npm.cmd run check:rust:fast`; `npm.cmd run verify` |
| Duplicate base comparison | missing commit/artifact; pure reduction; balanced true replacement with no exception; balanced replacement with invented exception; unchanged baseline with no base entry plus valid-looking invented current entry; mixed/retained growth; valid/missing growth exception; active carry; expired historical entry carried unchanged; active or expired-base future-dated metadata renewal; expired-base partial/full reduction or removal; expired-base `C>A` rejection; base `{P:5,A:7}` to `C:6` requiring `{P:5,A:6}`; `C<=5` requiring removal; illegal removal at `C:6`; positive `C>A` from active base; UTC equality expiry | fixture CLI tests with explicit local `--base`; `npm.cmd run check:rust:duplicates -- --base 9bcc047b9ec210c1ea34024f6dc782a1e7caec49` returns a nonempty structured result | Fast contract proves `required` plus SHA/full history; Full/release prove `off` and absent SHA |
| Duplicate mode | Fast missing mode/SHA; Fast all-zero SHA; `off` with SHA; local env use; CI `--base`; CLI/environment conflict | duplicate CLI and workflow mutation tests | focused workflow tests; real explicit-base command above |
| Advisory isolation | scanner given write scope; scan lacks `continue-on-error`; missing output/final fail; writer uses `failure()`; setup failure enables writer; missing/cancelling writer concurrency; bundle ignores scan result | `npm.cmd run test:unit -- scripts/testing/github-rust-workflows.test.ts` | `npm.cmd run test:unit`; workflow syntax inspection; remote behavior remains post-push evidence |
| Immutable Actions | each mutable tag, short SHA, tag-object SHA, expression ref, missing version comment, unknown remote action, or SHA differing from reviewed commit map | `npm.cmd run test:unit -- scripts/testing/github-rust-workflows.test.ts` | `npm.cmd run test:unit`; recorded peeled-tag/commit-object/action.yml evidence; `git diff --check` |
| Locked aliases/license | remove `--locked` from either alias; add unused allowed license; add stale license exception; restore either unused lint to warn | `npm.cmd run test:unit -- scripts/verify.test.ts`; `npm.cmd run check:rust:supply-chain` | `npm.cmd run check:rust:fast`; `npm.cmd run verify` |
| Sidecar golden wire shape | change a tag, field, null/default, or deserialize from serializer output | `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact` | Gemini Browser package all-target checkpoint; `npm.cmd run verify` |
| Owned secret move | complete pair, key-only, base-only, empty-key, empty-base characterization passes before/after; no fabricated runtime RED; bounded diff proves removal of clone | `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact` before and after | diff review; compiler and Clippy; `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked`; owner test; `npm.cmd run verify` |
| Value registry | omit or misspell any wrapper mode, duplicate-base mode, Cargo/npm requirement kind, or result classification; omit owner, checked-in persistence, product API/database/UI impact, or fixture/validator inventory | documentation contract test plus targeted `rg` assertions over `docs/value-registry.md` and the owning scripts, policy artifact, workflows, and fixtures | `npm.cmd run test:unit`; authorized-scope scan confirms the registry change is included and no product schema/UI file changed |
| Docs/evidence | unfinished markers, stale zero-test filter, global cargo-deny instructions, or unsupported CI claim | targeted `rg` scans and documentation contract tests | `git diff --check`; whole-range review |

The full local acceptance sequence is exact and ordered:

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

The advisory command is recorded truthfully and may remain red for the moving
RustSec database; it is not converted into a deterministic-gate success. Any
new advisory is reported rather than silently added to an exception.

## Correction Commit and Review Order

Implementation uses this order, with no squashing:

1. `build: bootstrap pinned cargo-deny locally` — archive/binary manifest
   hashes, transactional setup, absolute-path wrapper, composite action,
   aliases, wrapper contracts, and fresh-checkout docs;
2. `test: harden Rust manifest and dependency policy` — syntax ownership,
   complete Cargo/npm inventories, reviewed Tauri pair requirements,
   exact/prerelease ownership, and mutations;
3. `test: make Rust duplicate policy diff-aware` — structure/current equality,
   explicit base comparison, exact exception schema, CI base wiring, and tests;
4. `ci: isolate advisory writes and pin actions` — scanner/writer split,
   bundle dependency, full SHAs, comments, and workflow mutations;
5. `fix: close Rust Wave 0 minor findings` — locked residual aliases, strict
   unused-license policy, stale factual prose, golden sidecar fixture, and
   owned secret move;
6. `docs: verify Rust Wave 0 final hardening` — factual local commands,
   results, reviewed range, and explicitly pending remote evidence.

Each commit receives an immediate scope/spec/code review. A correction from
that review is a separate adjacent commit and is re-reviewed before the next
numbered slice. After the six slices and any adjacent corrections, run the full
matrix, review the entire original-Wave-0-plus-hardening range, and record the
exact reviewed head. Implementation stops there: pushing, PR creation, remote
Fast/Full evidence, advisory issue behavior, bundle dispatch, and release
evidence require separate user authorization.

## Success Criteria

The correction is complete locally when:

- a fresh checkout obtains the pinned cargo-deny through the repository cache
  and existing checksummed setup script without an ambient/global install;
- manifest syntax, all Cargo direct dependency kinds, all root Tauri npm edges,
  reviewed one-to-one Tauri pairs, exact pins, and prereleases are enforced
  bidirectionally without range extraction or writer auto-approval;
- the active duplicate graph exactly equals its valid current baseline, base
  comparisons validate both historical artifacts, only balanced true
  replacements remain review-only, and all other positive per-name growth has
  a complete unexpired attributable exception with a valid lifecycle;
- advisory scanning is read-only, issue writing is isolated and outcome-based,
  setup failures cannot create advisory issues, and the bundle still blocks;
- every remote Action uses the listed immutable full SHA with a version comment;
- both residual Cargo aliases are locked, unused licenses fail, stale prose is
  factual, the sidecar wire test is golden in both directions, and the API key
  moves by ownership without exposure or changed fallback behavior;
- every introduced tooling-only mode, kind, and classification is inventoried
  in `docs/value-registry.md` with owner, persistence/API/UI impact, and owning
  fixtures/validators;
- all focused and full local evidence is recorded without placeholders or
  remote claims; and
- the final worktree diff contains only authorized files and passes fresh
  whole-range review.
