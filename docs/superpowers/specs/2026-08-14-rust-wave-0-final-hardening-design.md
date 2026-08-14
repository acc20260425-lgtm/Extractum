# Rust Wave 0 Final Hardening Design Amendment

**Date:** 2026-08-15

**Status:** Approved simplified correction design; ready for a replacement implementation plan

**Amends:** `docs/superpowers/specs/2026-08-12-rust-infrastructure-upgrade-design.md`

## Purpose and Authority

This amendment closes the remaining Important and Minor Wave 0 findings with a
smaller executable policy surface. It supersedes every earlier revision of this
file and controls wherever it conflicts with the original Rust infrastructure
design. The existing final-hardening implementation plan predates this
amendment and must be replaced before implementation begins.

The correction has four goals:

1. make pinned cargo-deny setup and execution one repository-owned operation;
2. enforce manifest and direct-dependency facts without a TOML parser or a
   second source of reviewed Tauri mappings;
3. keep duplicate checking strict for the current graph while reducing
   historical comparison to the one policy decision that matters: positive
   per-package growth; and
4. test GitHub Actions structurally while preserving least privilege and the
   release advisory gate.

The branch is locally ready for a separately authorized push only after the
four code tasks, one combined code/spec review, the exact local acceptance
sequence, the evidence commit, and a final whole-range review all complete.
Merge readiness still requires green remote Rust Fast and Rust Full runs for
the reviewed head. Release readiness additionally requires a successful
advisory scan and gated bundle evidence.

## Non-Goals

This correction does not:

- upgrade Rust 1.95.0, Edition 2021, Cargo dependencies, Cargo.lock, existing
  npm dependencies, or cargo-deny 0.20.2;
- add any npm dependency except the direct devDependency `yaml` and its exact
  lockfile entry;
- resolve, suppress, or add exceptions for current RustSec findings;
- change product behavior, IPC, persisted data, migrations, public JSON,
  frontend behavior, or product UI;
- build a general TOML or YAML policy framework;
- classify duplicate changes, manage exception expiry/lifecycle, or infer a
  merge base;
- push, create a PR, dispatch Actions, publish, or claim remote evidence.

## Simplified Architecture

The correction has four implementation boundaries:

1. `scripts/cargo-deny.ps1` owns pinned tool installation, authentication,
   cache publication, GitHub PATH publication, and policy execution.
2. Repository rules use anchored manifest-line contracts plus Cargo metadata,
   while `rust-dependency-policy.json` alone owns reviewed Tauri pairs and the
   Cargo-only MCP record.
3. The duplicate checker always proves current graph equality and optionally
   compares one explicit base commit for positive cardinality growth.
4. Workflow tests parse YAML into objects using the direct `yaml` devDependency
   and assert security properties rather than mirroring whole workflow files.

Checks are read-only. The two explicit developer writers remain:

```powershell
node scripts/rust-dependency-policy.mjs --write scripts/testing/rust-dependency-policy.json
node scripts/rust-duplicate-baseline.mjs --write scripts/testing/rust-duplicate-baseline.json
```

Neither writer is called by tests, verification aliases, or CI.

## Unified Pinned Cargo-Deny Runner

### One production script

Replace/rename the existing `scripts/setup-cargo-deny.ps1` as
`scripts/cargo-deny.ps1` and expand that one file into the only production
PowerShell entry point. In Git this is the deletion of the old path and
creation of the new path, not a second installer or wrapper. Do not create
`scripts/run-cargo-deny.ps1`. The renamed script supports exactly:

- `-Mode Setup`: authenticate or install the tool, run no policy;
- `-Mode Deterministic`: run `bans licenses sources`;
- `-Mode Advisories`: run `advisories`.

`.github/tools/cargo-deny.json` remains schema 1 and retains the release archive
URL/hash plus the reviewed extracted executable digest:

```json
"binarySha256": "f7292fab58c706638c999e64c4ba82e5128ae628130ba55e3266a768ee431fbf"
```

The stable ignored cache is
`src-tauri/target/.extractum-tools/cargo-deny/0.20.2-x86_64-pc-windows-msvc`.
The script never inspects a user Cargo bin, global cargo-deny, or `cargo
install`.

On a cache hit it verifies the binary digest before any execution, then
requires exact `--version` output `cargo-deny 0.20.2`. On a cold or rejected
cache it downloads the configured archive into a staging directory, verifies
the archive digest, extracts, verifies the executable digest before executing
it, verifies the version, and publishes the executable transactionally. A
digest-rejected file is never executed. A digest-valid but wrong-version file
is used only for the version probe and is never published or used for policy.

`Setup -AddToGitHubPath` appends the normalized stable directory exactly once
to `GITHUB_PATH` after authentication succeeds and does not mutate the current
process PATH. Other modes reject `-AddToGitHubPath`. Failure restores the prior
authenticated cache, if any, and fails the current invocation; a failed first
install leaves no accepted executable.

Policy modes invoke the authenticated executable by absolute path with these
root-level arguments before the selected checks:

```text
--config $RepositoryRoot/deny.toml --manifest-path $RepositoryRoot/src-tauri/Cargo.toml --locked check
```

They propagate cargo-deny's exit code. `check:rust:supply-chain` calls
`Deterministic`, `check:rust:advisories` calls `Advisories`, and the composite
Action calls `Setup -AddToGitHubPath`.

The `.github/actions/setup-cargo-deny/` directory and composite Action name
intentionally remain unchanged because that Action's single responsibility is
still Setup. Only the production PowerShell script it invokes is renamed.

### One behavioral test

Create only `scripts/cargo-deny.test.ps1`. It launches the runner as a child
`powershell.exe`, captures stdout, stderr, and exit code, and tests cold/warm
cache, archive and binary digest rejection, wrong version, transactional
rollback, absolute locked arguments, policy exit propagation, exactly-once
GitHub PATH publication, and absence of ambient cargo-deny fallback.

The test is behavioral. It does not assert source token order, character
offsets, implementation text, or private helper names.

## Manifest and Dependency Policy

### Anchored inheritance lines

Do not add a TOML parser. Normalize CRLF to LF and count these exact anchored
lines in each of the seven manifests:

```regex
^edition\.workspace = true$
^rust-version\.workspace = true$
```

Each line must occur exactly once in:

- `src-tauri/Cargo.toml`;
- `src-tauri/crates/extractum-analysis/Cargo.toml`;
- `src-tauri/crates/extractum-core/Cargo.toml`;
- `src-tauri/crates/extractum-gemini-browser/Cargo.toml`;
- `src-tauri/crates/extractum-llm/Cargo.toml`;
- `src-tauri/crates/extractum-prompt-packs/Cargo.toml`;
- `src-tauri/crates/extractum-telegram/Cargo.toml`.

Only the six non-root member manifests prohibit literal lines matching
`^edition\s*=` or `^rust-version\s*=`. The root manifest is intentionally
different: its `[package]` inherits both values and its `[workspace.package]`
owns the valid literals `edition = "2021"` and `rust-version = "1.95"`.
Those two exact literal lines must each occur exactly once in the root.
Therefore a whole-root-file literal prohibition would be incorrect.

Cargo metadata remains authoritative for effective values. It must report the
exact seven sorted workspace package names, Rust version 1.95, Edition 2021,
and unpublished state for every member. `rust-toolchain.toml` remains the exact
canonical Rust 1.95.0/minimal/rustfmt/clippy/Windows-target artifact. Missing,
duplicate, indented, commented-out, or altered inheritance lines fail even if
metadata happens to resolve to the same effective value.

### Canonical direct inventories

`scripts/testing/rust-dependency-policy.json` uses schema 2 and contains exact
sorted arrays for:

- every direct Cargo normal/build/dev edge in every workspace member;
- every root `@tauri-apps/*` npm edge in dependencies, devDependencies, or
  optionalDependencies;
- every exact Cargo pin; and
- every direct Cargo requirement containing a prerelease comparator.

Its `toolchain` object remains the canonical Rust 1.95.0, MSRV 1.95, Edition
2021, Windows target, and exact seven-package inventory; no generated code
duplicates those values as a second reviewed policy.

Cargo identity is `package/dependency/rename/kind/target`; npm identity is
`owner/name/kind`. Requirements are the normalized strings reported by Cargo
metadata or stored in `package.json`. Comparison is bidirectional: missing,
extra, duplicate, moved-kind, renamed, retargeted, or requirement-drifted
entries fail.

The JSON artifact is the sole reviewed source for
`tauriFamily.pairs` and `tauriFamily.cargoOnlyRequirements`. Production and
test code must not contain a second exact map of pair IDs, identities, or
requirements. The validator is generic:

- pair and Cargo-only objects have exact documented shapes;
- IDs and referenced identities are unique and canonically ordered;
- Cargo kinds are `normal`, `build`, or `dev`; npm kinds are `dependencies`,
  `devDependencies`, or `optionalDependencies`;
- every pair Cargo/npm reference resolves exactly once to generated inventory;
- resolved requirements equal the reviewed requirement strings exactly;
- every independently discovered governed Cargo/npm edge belongs to exactly
  one pair; and
- the separately discovered MCP bridge edge belongs to exactly one Cargo-only
  record and no pair.

The writer regenerates the four factual inventories but preserves the
committed reviewed `tauriFamily` values unchanged. It reports unresolved or
candidate drift and leaves validation red until a reviewer edits the JSON.
It never infers approval from manifests or `package.json`.

Tests concentrate on meaningful policy drift: changed requirements, moved
kinds, missing/extra inventory edges, duplicate/orphan/unpaired identities,
missing or widened pair/Cargo-only records, and writer preservation. Small
direct helper tests cover wrong containers, null nested identities, wrong
scalar types, invalid kinds, duplicate IDs, duplicate identities, and ordering
without reproducing the entire reviewed Tauri table in test code.

## Minimal Windows Duplicate Gate

### Current-state invariant

`scripts/rust-duplicate-baseline.mjs --check` runs the locked Windows graph:

```text
cargo tree --manifest-path src-tauri/Cargo.toml --locked --target x86_64-pc-windows-msvc --workspace --prefix none --format {p}
```

It validates the baseline's exact schema-1 fields, target, sorted cardinality
map, integer cardinalities greater than one, and counter arithmetic. The
generated current graph must equal the committed baseline exactly. There is no
ceiling or warning-only current-state path.

The supply-chain exception artifact retains exact schema-1 top-level arrays.
Each duplicate growth entry has exactly `package`, `previousCount`,
`approvedCount`, `owner`, `reason`, and `reviewAfter`; packages are unique and
sorted, counts are integers with `approvedCount > previousCount >= 1`, and the
three text fields are nonempty strings. `reviewAfter` is retained as reviewed
metadata but has no clock or expiry behavior in this correction.

Current-only checking validates current graph equality and exception schema.
It does not reject carried, unused, expired, renewed, reduced, or removed
entries and does not classify the result.

Unused `duplicateGrowthExceptions` entries are reviewed and removed manually
when closing a dependency wave. This correction has no automated unused-entry
check, expiry check, lifecycle transition, renewal, or cleanup writer.

### Optional explicit base

The read-only CLI contract is:

```text
--check [--base SHA]
```

With `--base`, resolve the supplied revision using `git rev-parse --verify
SHA^{commit}` and read both historical artifacts using `git show`:

- `scripts/testing/rust-duplicate-baseline.json`;
- `scripts/testing/rust-supply-chain-exceptions.json`.

An invalid or unresolvable revision is an infrastructure failure. If the commit
exists but either historical artifact is absent, print an explicit
`historical duplicate policy unavailable; skipping base comparison` notice and
complete the strict current-only check. Do not partially compare one artifact.
If both exist, validate both schemas; malformed present history fails.

For each package whose current cardinality is greater than its base
cardinality, require exactly one current `duplicateGrowthExceptions` entry
whose `package`, `previousCount`, and `approvedCount` equal that transition.
An absent name has base cardinality 1. The matching entry must also have
nonempty `owner`, `reason`, and `reviewAfter`. Missing, duplicate, or
count-mismatched approval fails. Decreases, unchanged names, replacement
patterns, historical exception lifecycle, extra structurally valid current
entries, and time are outside this minimal comparison.

There are no CLEAN/REVIEW/REDUCTION/VIOLATION classifications, no duplicate
base modes, no environment resolver, and no Full/Release historical-base mode
wiring. Output is pass/fail plus the explicit historical-skip notice.

### GitHub base behavior

Rust Fast retains `push` on `main` and `pull_request` triggers without duplicate
runs for branch pushes. A pull request performs this targeted fetch before the
duplicate check:

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
- name: Run the existing fast Rust gate
  run: npm.cmd run check:rust:fast
```

`check:rust:fast` remains exactly its existing four-part composition: rustfmt,
Clippy, the two production feature-off checks, and deterministic cargo-deny.
It does not call `check:rust:duplicates`.

On a PR the base-aware command already performs the strict current check, so
the workflow does not run a second current-only duplicate command. On a direct
push to `main`, it runs the current-only duplicate command exactly once. The PR
checkout does not use full history. That event SHA is the target branch tip
supplied by GitHub and is deliberately not a calculated merge base. The
absent-historical-artifact skip remains the one defined in the preceding
owning section.

Rust Full invokes `npm.cmd run check:rust:duplicates` exactly once as an
unconditional, separate current-only command without `--base`, duplicate mode,
or a base-SHA environment variable. This covers Rust Full manual dispatch. On
pull requests it intentionally repeats the current-equality proof already
included in Rust Fast's base-aware check: the command is cheap, and an event
condition would add branching to save only seconds.

In Release, exactly the `windows-bundle` job invokes that same current-only
command exactly once. This covers tags and manual dispatch. Neither
`advisory-scan` nor `advisory-issue-writer` invokes the duplicate command. A
scheduled cron run that has no bundle runs advisories only and performs no
duplicate check; this is intentional because it neither gates nor builds an
artifact.

Step order relative to `npm.cmd run check:rust:fast` is unregulated. Rust Full
and Release `windows-bundle` presence is a reviewed design fact checked during
the combined review; it is not a fifth property of
`github-rust-workflows.test.ts` and has no separate automated contract.

## Structural Workflow Contracts and Advisory Isolation

### Direct YAML dependency

Add exactly one direct npm dependency edge: `yaml` in `devDependencies`, using
major version 2. `package-lock.json` may change only to record the root direct
edge and the resolved `yaml` package. Existing dependency versions and lockfile
entries must not move. No runtime dependency is added.

Workflow tests use the documented ESM API:

```ts
import { parse } from "yaml";
const workflow = parse(source);
```

They assert the resulting object graph. Do not keep or add a handwritten YAML
indentation parser, exact workflow-text mirror, central approved Action SHA
map, or complete YAML fixture duplicate.

The workflow test contract owns exactly four security properties:

- every external `uses` value is pinned to some full 40-hex commit SHA, while
  repository-local `./` actions are exempt;
- the advisory scanner has exactly `contents: read` permission;
- the follow-up writer has exactly `issues: write` permission and runs only
  when the classify step's output, derived only from the explicit deny step,
  is the string `true`; and
- the bundle needs a successful scanner result and has no `always()` or
  `!cancelled()` bypass.

The test file contains one scope comment listing only these owned properties,
so later maintainers do not turn it back into a workflow mirror.

The test does not own exact Action SHA values, version comments, PR fetch
content or ordering, unrelated step ordering, full-history behavior, or a
textual copy of the workflows.

### Exact advisory flow

`advisory-scan` runs on Windows with only `contents: read`. Checkout,
toolchain inspection, local cargo-deny Setup, and the advisory command all run
without issue-write authority. The deny step is exact:

```yaml
- id: deny
  continue-on-error: true
  run: npm.cmd run check:rust:advisories
```

The next step always runs after the deny attempt and classifies only that
step's outcome:

```yaml
- id: classify
  if: always()
  shell: pwsh
  run: |
    "advisories_failed=${{ steps.deny.outcome == 'failure' }}" >> $env:GITHUB_OUTPUT
```

The job exports the classify-step value exactly:

```yaml
outputs:
  advisories_failed: ${{ steps.classify.outputs.advisories_failed }}
```

The test rejects log parsing, job-level `failure()`, or a setup-step outcome as
a substitute. The final step is exact:

```yaml
- name: Fail when advisory scan failed
  if: ${{ steps.deny.outcome == 'failure' }}
  shell: pwsh
  run: exit 1
```

It turns a real advisory failure back into a failed scanner job.

`advisory-issue-writer` needs `advisory-scan`, has exactly `issues: write`,
runs on Ubuntu, keeps serialized `rust-advisory-follow-up` concurrency, and
contains only a full-SHA-pinned `actions/github-script` step. Its condition is
the conjunction of:

- `always()`;
- `needs.advisory-scan.outputs.advisories_failed == 'true'`; and
- schedule or explicit workflow-dispatch event.

The resulting job condition is exact:

```yaml
if: ${{ always() && needs.advisory-scan.outputs.advisories_failed == 'true' && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}
```

It lists open issues, updates the existing advisory follow-up issue when
present, or creates one otherwise. It receives no checkout, cache, installer,
artifact, shell, repository content, or build secret.

If checkout, toolchain inspection, or cargo-deny Setup fails, the deny step is
skipped. The `always()` classify step writes `advisories_failed=false` when it
can run; if classification itself cannot run, the job output is absent. In
both cases the writer's exact string comparison is false and no advisory issue
is opened. If the deny step fails, classify writes `true`, the explicit output
enables the writer only on the allowed events, the final step makes the scanner
red, and the bundle is blocked.

The bundle job has `needs: advisory-scan` and preserves its existing tag/manual
event restriction while adding the positive predicate
`needs.advisory-scan.result == 'success'`. It does not use `always()` or
`!cancelled()` to bypass the scanner result.

## Bounded Residual Corrections

The final slice retains these already approved corrections:

1. Add `--locked` to `test:rust` and `test:rust:prompt-pack-runs`, with positive
   and unlocked-negative alias tests.
2. Remove the unused `Apache-2.0 WITH LLVM-exception` allowance and set both
   `unused-allowed-license` and `unused-license-exception` to `deny`, with
   focused mutations owned by `repository-rules.test.ts`.
3. Replace the Gemini sidecar self-round-trip with an independent literal
   golden object in both serialization and deserialization directions,
   including every run-result and artifact field.
4. Move configured LLM secret ownership through a private result enum without
   cloning, exposing, logging, serializing, converting, or persisting the key;
   complete, key-only, base-only, empty-key, and empty-base behavior stays the
   same.
5. Correct only stale Wave 0 plan/verification facts and record the final local
   evidence without unsupported remote claims.
6. Update `docs/value-registry.md` only for values still introduced here:
   cargo-deny modes `Setup`, `Deterministic`, `Advisories`; Cargo kinds
   `normal`, `build`, `dev`; and npm kinds `dependencies`, `devDependencies`,
   `optionalDependencies`. Do not add duplicate modes or duplicate result
   classifications because this design introduces neither.

## Authorized Implementation Scope

Implementation may modify only:

- rename `scripts/setup-cargo-deny.ps1` to `scripts/cargo-deny.ps1` (delete the
  old path and create the new path), and create `scripts/cargo-deny.test.ps1`;
  this is one production-script replacement, not two scripts;
- do not create or modify any `run-cargo-deny.ps1` file;
- `.github/tools/cargo-deny.json`,
  `.github/actions/setup-cargo-deny/action.yml` (the composite directory/name
  intentionally remains Setup-owned), `package.json`, `package-lock.json`, and
  `scripts/verify.test.ts`;
- `scripts/rust-dependency-policy.mjs`, its test, and
  `scripts/testing/rust-dependency-policy.json`;
- `scripts/rust-duplicate-baseline.mjs`, its test,
  `scripts/testing/rust-duplicate-baseline.json`, and
  `scripts/testing/rust-supply-chain-exceptions.json`;
- `scripts/testing/repository-rules.mjs`,
  `scripts/testing/repository-rules.test.ts`, and
  `scripts/testing/test-conventions.test.ts`;
- `.github/workflows/rust-fast.yml`, `.github/workflows/rust-full.yml`,
  `.github/workflows/rust-release.yml`, and
  `scripts/testing/github-rust-workflows.test.ts`;
- `deny.toml`;
- `src-tauri/crates/extractum-gemini-browser/src/types.rs` and
  `src-tauri/src/llm/mod.rs`;
- `README.md`, `AGENTS.md`, `docs/project.md`, and
  `docs/value-registry.md`;
- replace the superseded implementation plan
  `docs/superpowers/plans/2026-08-14-rust-wave-0-final-hardening.md`;
- factual corrections in
  `docs/superpowers/plans/2026-08-12-rust-infrastructure-wave-0.md` and
  `docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md`;
- create
  `docs/superpowers/verification/2026-08-15-rust-wave-0-final-hardening.md`.

No other Rust source, Cargo manifest, Cargo lockfile, frontend source,
migration, product configuration, dependency, or unrelated documentation is
authorized. `package.json` and `package-lock.json` changes are limited to the
single direct `yaml` devDependency and required script aliases; no existing
npm package version may change. An implementation need outside this list stops
for a reviewed design amendment.

## Verification Matrix

| Contract | Required negative evidence | Focused GREEN evidence |
| --- | --- | --- |
| Unified cargo-deny runner | cold/warm cache, wrong archive/binary hash, digest-valid wrong version, failed first install, rollback, ambient tool absent, wrong policy exit, duplicate GitHub PATH | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/cargo-deny.test.ps1`; `npm.cmd run check:rust:supply-chain` |
| Manifest inheritance | missing/duplicate/indented/commented inheritance line; literal in each non-root member; missing root workspace literal; wrong effective metadata/inventory | repository-rule and convention tests; real repository snapshot |
| Direct dependency policy | Cargo/npm requirement or kind drift, missing/extra/duplicate edge, orphan/unpaired pair, widened MCP record, malformed helper shapes, writer changes reviewed Tauri values | dependency-policy and repository-rule tests; explicit writer preservation fixture |
| Current duplicate gate | malformed baseline/exception schema, arithmetic/sort failure, stale baseline, live graph drift | duplicate and repository-rule tests; `npm.cmd run check:rust:duplicates` |
| Historical duplicate growth | invalid revision, malformed present history, one/both artifact absent skip, absent-name growth from 1, surviving-name growth, exact/missing/duplicate/mismatched approval | fixture-repository CLI tests; explicit known-base command |
| Structural workflow policy | (1) mutable/short external Action ref; (2) scanner extra/missing permission; (3) writer permission or mandatory deny/classify/fail/job-output/`advisories_failed == 'true'` wiring drift, including setup failure treated as deny failure; (4) bundle scanner-success bypass | YAML-object workflow tests using `parse`; static workflow inspection |
| npm scope | missing direct `yaml`, runtime placement, any existing package or lock version changes | package/lock mutation test and bounded diff review |
| Locked aliases and licenses | remove either `--locked`; restore either unused-license warning; add stale allowance/exception | verify and repository-rule tests; deterministic cargo-deny |
| Sidecar golden | changed tag/field/null or deserialization from serializer output | exact Gemini Browser owner test and package checkpoint |
| Secret ownership | complete/partial/empty characterization before and after; clone reintroduced | exact application owner test, bounded diff, check, Clippy, package checkpoint |
| Docs, registry, evidence | obsolete duplicate modes/classifications, global cargo-deny instruction, stale facts, unfinished or remote claims | documentation contracts, targeted stale scan, whole-range review |

## Exact Local Acceptance

Run in this order from the repository root:

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

Commands through `verify` are deterministic PASS requirements. The final
advisory command is recorded truthfully and may remain nonzero for the moving
RustSec database; it remains a release blocker and never auto-creates an
exception. Evidence must also prove that the package/lock diff adds only the
direct `yaml` devDependency and its resolved lock entry.

## Four Code Tasks, Review, and Evidence Order

Implementation uses four code tasks and does not perform per-task reviews:

1. `build: consolidate pinned cargo-deny runner` — rename the existing setup
   script into the unified runner, then update its manifest, composite action,
   aliases, behavioral PowerShell test, and operator docs.
2. `test: harden Rust dependency and duplicate policy` — anchored inheritance
   checks, generated inventories, JSON-owned Tauri mapping, generic dependency
   validation, strict current duplicate equality, optional base artifacts,
   historical skip, positive-growth approval, exact PR-base fetch, and focused
   tests.
3. `ci: isolate advisory writes with structural workflow contracts` — direct
   `yaml` devDependency and lock entry, parsed workflow security contracts,
   exact deny/classify/fail/output wiring, isolated follow-up writer, and
   successful-scanner bundle gate, plus one unconditional current-only
   duplicate invocation in Rust Full and one current-only invocation in
   Release `windows-bundle`; advisory-only scheduled runs do not invoke it.
4. `fix: close bounded Rust Wave 0 findings` — strict licenses, locked aliases,
   golden sidecar, clone-free secret ownership, reduced value registry, stale
   factual docs, and focused Rust/documentation evidence.

After Task 4, run one combined authorized-scope, spec-compliance, and code-
quality review over all four commits. Review corrections are adjacent commits
and are rechecked as one combined range; there are no per-task review gates.
Then run the exact acceptance sequence and commit only the factual record as:

5. `docs: verify Rust Wave 0 final hardening`.

Finally review the whole implementation-and-evidence range, including the
evidence commit, from `77c739a321f08fc49d7a46d846d2a93a0dafc116`
through the exact committed head. Record the reviewed head and findings in the
ignored final-review report and handoff. Implementation stops before push,
PR, workflow dispatch, bundle, or release action.

## Success Criteria

The correction is complete locally when:

- one repository PowerShell script authenticates and runs pinned cargo-deny
  locally and in CI, with one behavioral test and no ambient fallback;
- all seven manifests satisfy exact inheritance lines while only six non-root
  members prohibit literals, and metadata proves exact effective policy;
- the generated Cargo/npm inventories are bidirectional and the JSON artifact
  is the only reviewed Tauri pair/Cargo-only source;
- the active Windows duplicate graph exactly equals its baseline, explicit base
  growth needs exact attributable approval, and absent historical artifacts
  follow only the documented skip path;
- PRs fetch exactly the event base SHA while direct main pushes remain
  current-only, without full history or merge-base inference;
- Rust Full runs exactly one unconditional separate current-only duplicate
  check, and Release `windows-bundle` runs exactly one current-only duplicate
  check, both without base arguments or duplicate mode environment; advisory
  scan/writer jobs and advisory-only scheduled runs do not run it;
- workflow contracts are parsed with `yaml`, all external Actions use full
  commit SHAs, the deny/classify/fail/job-output chain owns exactly
  `advisories_failed`, the writer requires its string value `true`, setup
  failure cannot open an issue, and bundle requires scanner success;
- package and lock changes add only direct devDependency `yaml` and do not
  upgrade an existing npm package;
- locked aliases, strict license policy, sidecar golden shape, clone-free
  secret ownership, factual docs, and the reduced value registry all pass;
- exact local evidence contains no placeholders or unsupported remote claims;
  and
- the final diff stays inside authorized scope and the final whole-range review
  has no Important or Critical finding.
