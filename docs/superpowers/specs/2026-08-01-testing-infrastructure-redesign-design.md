# Testing Infrastructure Redesign Design

## Status

The design was approved on 2026-08-01 and revised after written review on the
same date. The revised specification must be reviewed by the user before
implementation planning begins.

## Executive Summary

Extractum will replace its single mixed test surface with a vertically owned
portfolio, two deliberately different feedback contracts, and one completion
gate:

- focused local feedback that starts a new process and returns in at most
  15 seconds at p95;
- warm watch feedback that returns in at most 5 seconds at p95;
- a complete local `npm.cmd run verify` gate with an initial 90-second p95
  objective that becomes binding only after measured critical-path
  ratification.

The redesign is not a small runner tuning exercise. It changes the test
portfolio, source-contract strategy, browser ownership, changed-file
selection, diagnostics, and full-gate orchestration together. Low-value tests
that assert implementation text will be deleted. Important invariants will
move to behavior tests, compiler-backed architecture rules, package metadata,
or generated declarative contracts.

The 15-second Rust objective is phased. Selector and portfolio work first make
root-package misses explicit; a program-owned root refinement phase then moves
the minimum necessary production logic behind approved smaller package
boundaries. Final program acceptance includes `extractum` and cannot be
claimed while its compile-invalidating probe remains above budget.

GitHub Actions, branch protection, Git hooks, and any other server- or
Git-enforced gate are explicitly outside this design. The authoritative gate
remains local, and the project accepts the resulting reliance on developers
and agents running it before declaring work complete.

## Context and Evidence

The 2026-08-01 timing baseline was captured on commit `1114e626` on the
current Windows workstation. Telegram Phase 8C was implemented afterward;
the design is written against current commit `88363865`, where
`extractum-telegram` is already a workspace package. Slice 1 must rebaseline
the current inventory rather than treating the earlier counts as acceptance
evidence.

| Surface | Current evidence | Consequence |
| --- | --- | --- |
| Root Vitest | 177 files and 1,632 tests; about 199.96 seconds in the measured full run | Too slow for RED/GREEN feedback |
| Chromium teardown | All assertions passed, but the full Vitest run failed in an `afterAll` timeout | Browser lifecycle is flaky even when behavior is correct |
| Svelte check | About 40.69 seconds | It cannot be part of the 15-second loop |
| Cargo workspace | 1,233 tests in about 29.90 seconds | Healthy relative to Vitest, but still too broad for every Rust edit |
| Compile-invalidating root Rust probe | About 39.71 seconds in the earlier profiling evidence | A selector alone cannot meet the 15-second Rust contract for root-package edits |
| Recent full `verify` | 172.1, 202.7, and 175.5 seconds | The full local gate needs a shorter critical path |
| Fast Node behavior cohort | 81 files and 723 tests; about 2.8 seconds of summed test execution | Most assertions are not inherently expensive |
| Source-contract cohort | About 70 files and 695 tests; about 112 seconds of summed file time | Repository parsing and textual assertions dominate the long tail |

At the timing baseline, the three largest contract files alone contained about
24,500 lines:

- `src/lib/telegram-crate-boundary-contract.test.ts`, about 10,480 lines;
- `src/lib/analysis-crate-boundary-contract.test.ts`, about 7,272 lines;
- `src/lib/analysis-application-contract.test.ts`, about 6,765 lines.

At current commit `88363865`, exact physical line counts are 11,682, 7,297,
and 6,765 respectively. This expected drift makes the Slice 1 rebaseline a
blocking input, not optional documentation.

These files repeatedly read production sources and contain custom parsing,
frozen authority data, and implementation-shaped assertions. Vitest's static
related-test graph cannot see many of those filesystem dependencies. The
current `test:changed`, `test:changed:last`, and `test:related` commands can
therefore exit successfully while selecting no relevant tests.

The root suite also mixes pure Node tests, jsdom component tests, repository
architecture checks, real Windows processes, and Chromium. In particular,
`sidecars/gemini-browser/src/answer-extractor.test.ts` launches Playwright's
Chromium from Vitest and has two independent `afterAll` hooks for page and
browser cleanup. The installed Vitest 4.1.5 uses stack-ordered hooks by
default, so the page hook runs before the browser hook; the observed timeout
does not prove a race.
It does prove that manually owned Chromium teardown inside the mixed Vitest
suite is not currently reliable or sufficiently diagnosable.

The existing `scripts/verify.mjs` runs its stages strictly sequentially. It
does not include sidecar typecheck/build, adapter E2E, or a main-app browser
smoke, while the repository workflow can also repeat the Rust workspace gate
before invoking `verify`.

### Budget authority and ratification

The 15-second one-shot, 5-second watch, and 90-second full-gate values are
design targets until preregistered measurements cover the corresponding
implemented surface. They are not inferred from desired architecture, summed
per-test durations, or a smaller convenient inventory.

Slice 1 owns the initial baseline and budget-decision artifact. Ratification is
always scoped to a named, fingerprinted cohort; it is never a repository-wide
boolean inherited by future gates. A budget is `RATIFIED` only when every
preregistered attempt is retained, the required inventory for that scope is
complete, and the scope's declared ratification formula meets the target. The
one-shot and watch formulas use nearest-rank p95; the full gate also includes
the modeled operating-headroom term defined below. A surface that does not yet
exist remains `TARGET_NOT_RATIFIED`; Slice 1 must not claim it passed.

A missed target records the failing cohort and critical path. It does not
authorize an SLA exemption, hidden inventory reduction, or replacement of a
failed attempt. Later slices implement measured remediation and repeat the
same contract. No CI, branch-protection rule, Git hook, or other remote
enforcement may be introduced as compensation.

These budget-state values are ephemeral verification metadata. Their owner,
non-persistence, and lack of product API/UI impact must be recorded in
`docs/value-registry.md` with the selector status vocabulary.

## Goals

1. Return useful local RED/GREEN feedback within 15 seconds at p95 from a new
   process, without running a full typecheck.
2. Return useful warm watcher feedback within 5 seconds at p95.
3. Fail closed when a changed file has no known test mapping.
4. Separate pure, component, architecture, OS integration, and browser tests
   so each has an explicit owner and execution policy.
5. Eliminate arbitrary production-source `includes` and regular-expression
   assertions while preserving valuable behavior and architecture invariants.
6. Make Chromium exclusively Playwright-owned.
7. Measure every complete-gate candidate and ratify the 90-second p95
   objective only when the resulting resource-constrained critical-path model
   supports it. Without explicit user approval of a revised number, 90 seconds
   remains the final program target.
8. Include root frontend, sidecar, adapter, Rust workspace, Windows
   integration, and critical browser surfaces in the authoritative local
   completion gate.
9. Produce machine-readable timings and failure classifications without
   masking flaky tests with retries.
10. Keep commands and ownership rules understandable without requiring a
    general-purpose monorepo build system.

## Non-Goals

- Do not add GitHub Actions, branch protection, required PR checks, nightly
  jobs, pre-commit hooks, pre-push hooks, Husky, or Lefthook.
- Do not treat focused feedback as completion evidence.
- Do not introduce global test retries or retry-until-green scripts.
- Do not adopt Nx, Turborepo, another general-purpose build graph, remote
  execution, or build-artifact caching. This design does introduce a narrow
  repository-owned test execution graph; its maintenance cost is an accepted
  risk, not a non-goal.
- Do not introduce Testcontainers or Docker for the SQLite-owned backend.
- Do not add Mockall where existing narrow Rust traits and handwritten fakes
  already provide adequate seams.
- Do not use one global coverage percentage as a quality target.
- Do not make full Tauri packaging or release builds part of the 90-second
  test gate. Release verification remains a separate workflow.
- Do not automatically delete Cargo, npm, Playwright, or test artifact caches.
- Do not pull experimental Python research suites into the root product gate.
  Direct edits to those trees must still map to their existing owner command.
- Do not change application runtime behavior solely to satisfy an
  implementation-shaped test.

## Considered Approaches

### A. Rebuild the portfolio vertically

Change runner boundaries and expensive tests together. Introduce explicit
test tiers, a fail-closed selector, compiler-backed architecture checks, and
Playwright browser ownership in coordinated slices.

This is selected. Runner tuning alone cannot remove the 112-second
source-contract tail, while deleting contracts alone would leave mixed
environments, false-green selection, and browser lifecycle failures.

### B. Change the runner first

Split Vitest projects and add selection before rewriting tests. This produces
commands quickly but leaves the expensive contracts on the critical path and
forces the selector to understand brittle filesystem relationships that are
scheduled for deletion.

Rejected because it optimizes around the wrong portfolio.

### C. Rewrite contracts first

Replace the large textual contracts before changing orchestration. This
reduces raw execution time but preserves a mixed suite, ambiguous ownership,
and false-green changed-file behavior during a long migration.

Rejected because it delays the feedback contract and browser reliability
work.

## Target Test Portfolio

The portfolio has five ownership tiers. A tier describes semantics and
resource policy; it is not required to map one-to-one to a single process.

| Tier | Owner and contents | Execution policy |
| --- | --- | --- |
| `unit-node` | Pure TypeScript logic, API wrappers, workflow decisions, parsers, and selectors | Vitest Node environment, threads, isolation on, no jsdom or real OS resources |
| `component` | Svelte rendering and interaction through Testing Library | Dedicated Vitest project with jsdom and one explicit cleanup owner |
| `architecture` | Import boundaries, cycles, metadata, and domain-specific structured rules | Cached structured index; no arbitrary production-source text assertions |
| `os-integration` | Real Windows processes, filesystem behavior, file-backed SQLite, sockets, and equivalent Rust integration behavior | Dedicated processes/forks, limited workers, explicit timeouts and cleanup |
| `e2e` | Browser-owned user scenarios, the Gemini adapter, and Chromium-backed extraction | Playwright Test owns browser/context/page; scoped fixtures own servers, traces, and artifacts |

### Vitest projects

Vitest configuration will expose separate `unit-node`, `component`, and
`architecture` projects. Vitest-owned Windows integration tests use an
additional `os-integration` project with a fork pool and constrained workers.
Playwright tests are never included in a Vitest project.

Every project has a stable unique name and inherits the lean shared Vite/Vitest
base through `extends: true`. The shared base owns repository aliases and
required transforms and deliberately owns no environment, setup, cleanup,
pool, worker, or timeout policy. Those policies belong to the target project;
Svelte Testing Library and DOM setup belong only to `component`. Per-file
environment directives are removed after partitioning. A mixed file that
combines component behavior with raw architecture assertions is split before
the project partition is enabled.

`TestingManifest` owns project classification. A separate filesystem census,
independent of Vitest configuration, discovers every supported test/spec file.
Manifest lint proves that every census file has exactly one runner owner, every
Vitest-owned file belongs to exactly one project, project intersections are
empty, and their union equals the independent census after Playwright-owned
files and explicitly reasoned non-runnable fixtures are removed. Vitest root
discovery is not accepted as the independent census because a missing
`include` would disappear from both sides of that comparison.

Every project command selects its stable name explicitly with `--project` and
validates that collection is non-empty. Vitest's `passWithNoTests` option,
including its implicit behavior for raw `--changed`, cannot turn an empty
project or selector result into valid evidence.

From the Slice 2 partition checkpoint until the final Slice 3 checkpoint,
Vitest exposes a transitional `legacy-contract` project. It is not a sixth
target tier. It owns every remaining test that reads arbitrary production
source and does not yet satisfy the target `architecture` policy, preserves
its current Node/filesystem execution, and is excluded from feedback and watch
mode while remaining in full `test`/`verify` inventory.

Assertion obligations are frozen before any ownership edit. A mechanical mixed-
file split may relocate those frozen IDs only through an explicit old-to-new
lineage map and cannot create a new obligation. The transitional legacy file
inventory is then frozen at the Slice 2 checkpoint after those splits.

From that checkpoint onward, no new obligation may enter and the obligation
count must decrease monotonically. File count need not be monotonic when an
approved relocation preserves the same frozen IDs and lineage. Each Slice 3
sub-slice runs the remaining legacy cohort together with its replacement
evidence. Slice 3 cannot complete until the project inventory is empty and its
project, command, manifest, config, and exception entries are removed
atomically.

The project split must not globally disable isolation. Any later proposal to
disable isolation for a subset requires an explicit state-ownership audit and
an A/B benchmark for that exact subset.

### Rust ownership

Rust behavior stays in the owning Cargo package. A Node architecture test must
not parse `.rs` implementation text to prove behavior. Cross-crate structure
uses `cargo metadata`, compiler checks, public-interface tests, or narrow Rust
tests in the owner package.

Current Telegram behavior is owned by `extractum-telegram`; its immediate
application consumer is `extractum`, which enables the narrow
`app-test-support` seam only through its dev-dependency. Analysis behavior is
owned by `extractum-analysis` or `extractum` according to the current public
boundary. The testing redesign follows these present owners and does not move
ownership back into application-level source contracts.

If compile-invalidating `extractum` edits cannot meet the 15-second feedback
contract, selection is not considered a fix. Production logic must move to an
approved smaller domain boundary, without duplicating it in a test-only crate.
The testing-infrastructure program owns the performance outcome and includes
the minimum root-package refinement necessary to meet it. Boundary choices
must remain consistent with the crate roadmap; when the roadmap does not
already define a safe owner, the required nested architecture design is a
program deliverable rather than an external prerequisite.

### Rust feedback acceptance phases

Rust feedback has two explicit acceptance phases:

1. **Selector phase:** `extractum-telegram`, `extractum-analysis`, and other
   already bounded owner packages must meet the 15-second contract. Known
   `extractum` and conservatively mapped mixed producer/consumer paths that
   still exceed it are recorded as unexpired fast-feedback debt and return
   `DEFERRED_ONLY` with exact owner commands. Debt is not permitted for the
   named internal Telegram or analysis cohorts. This phase enables fail-closed
   selection but is not final SLA acceptance.
2. **Root-refinement phase:** program-owned package-boundary work eliminates
   the designated `extractum` performance-debt entries. The final benchmark
   includes compile-invalidating `extractum` probes, and every named Rust probe
   must complete within 15 seconds in all three planned attempts.

The program may checkpoint the selector phase while root refinement remains,
but it cannot satisfy final acceptance criterion 1 until the second phase is
complete.

## Command Contract

All documented Windows commands use `npm.cmd`.

| Command | Contract |
| --- | --- |
| `npm.cmd run test:feedback` | Canonical one-shot affected-test feedback for the working tree or an explicit base; ratified fast cohorts have a 15-second p95 budget |
| `npm.cmd run test:changed` | Compatibility alias for working-tree `test:feedback` |
| `npm.cmd run test:changed:last` | Compatibility alias using the most recent linear checkpoint as its base |
| `npm.cmd run test:related -- <paths...>` | Explicit-path entry into the same selector, not raw `vitest related` |
| `npm.cmd run test:watch` | Selector-aware warm watcher for fast tiers; p95 rerun budget 5 seconds |
| `npm.cmd run check:watch` | Separate background Svelte/TypeScript checking; never blocks RED/GREEN feedback |
| `npm.cmd run test:unit` | Complete `unit-node` project |
| `npm.cmd run test:component` | Complete component project |
| `npm.cmd run test:architecture` | Complete structured architecture project |
| `npm.cmd run test:integration:os` | Complete OS integration tier |
| `npm.cmd run test:e2e` | Critical Playwright suite |
| `npm.cmd run test:manifest` | Validate the manifest schema/graph/census plus Cargo-owned test inventory and feature evidence |
| `npm.cmd run test` | All Vitest-owned projects; not the complete repository gate |
| `npm.cmd run verify` | Authoritative complete local correctness gate |
| `npm.cmd run verify:performance` | Run the preregistered twenty-run full-gate budget ratification benchmark |
| `npm.cmd run verify:coverage` | Explicit V8 coverage analysis for declared critical modules |
| `npm.cmd run verify:stability` | Explicit repeated-run flake investigation; not a retry wrapper |

Existing narrow package scripts not listed above are retained only when they
remain direct entry points into one declared owner gate; redundant orchestration
scripts are removed. The commands above are the public workflow. `AGENTS.md`
and `docs/project.md` must describe the same ownership and must stop
instructing callers to invoke raw Vitest changed/related selection.

The warm watcher covers TypeScript, Svelte, and cached architecture rules.
This design does not introduce a persistent Cargo watcher. Rust uses the
one-shot affected loop and the existing package-focused verification loop.

`npm.cmd run test` is a convenience aggregate that runs every configured
Vitest project once: five during the legacy transition and the four target
projects afterward. `verify` does not call that aggregate: it invokes
`unit-node`, `component`, `architecture`, and transitional `legacy-contract`
in its frontend group and the `os-integration` project under its own reporting
group and resource claims, so no Vitest file executes twice.

## Fail-Closed Affected-Test Selector

### Inputs

The selector accepts one of:

- the staged, unstaged, and untracked working-tree set;
- an explicit Git base through `test:feedback -- --base <ref>`;
- the current linear checkpoint base used by `test:changed:last`;
- one or more explicit repository paths.

It normalizes Windows and POSIX path separators and rejects paths outside the
repository. A missing Git base or unreadable repository state is an
infrastructure error, not an empty selection. Base mode evaluates
`<base>...HEAD` and then adds current staged, unstaged, and untracked paths.
`test:changed:last` is exactly base mode with `HEAD~1`.

A clean working tree or empty base range returns `NO_CHANGES`; it does not
claim completion. A missing explicit path or an empty explicit-path invocation
returns `MAPPING_ERROR`. Git-discovered deleted paths are classified by their
former repository path without requiring the file to exist. A rename is
classified as one deletion plus one addition, and both sides must map.

### Mapping sources

One repository-owned `TestingManifest` is the executable source of truth for
orchestration-level gate ownership, path mapping, feedback eligibility,
architecture rule descriptors, debt, quarantine, and no-test allowlists.
Architecture rule modules export their ID and input descriptor for the
manifest to import; the same paths are not copied into a second config. A
third-party tool's native configuration remains the owner of that tool's
internal analysis scope and rule semantics rather than duplicating those path
lists in the manifest.

Before path classification, selector startup performs cheap manifest schema
and reference validation without spawning Vitest or Cargo. Failure returns
`INFRA_ERROR`; malformed orchestration data is never treated as an unknown
product path or empty selection.

The exhaustive `test:manifest` contract has a static phase that is a mandatory
prerequisite of the orchestrated static and full gates. It validates
schema/version, unique IDs, normalized repository-relative paths, existing
package scripts and gate dependencies, acyclic prerequisites, valid resource-
lock identifiers and artifact roots, non-orphaned path patterns, the
independent test census, exclusive runner/project ownership, rule descriptors,
project environment/setup policy, and disjoint allowlist, debt, and quarantine
entries with complete owner/expiry metadata. It rejects mappings to deleted
JavaScript test paths or evidence IDs and patterns that no longer match a
tracked path unless the entry explicitly models a Git deletion. Every
migration replacement descriptor must resolve to an enabled `verify` gate, not
merely exist.

Cargo validation is deliberately two-level. The static phase uses `cargo
metadata` to validate package, target, and declared feature names; it does not
pretend that metadata contains libtest IDs or proves runtime feature
activation. A Cargo-owned dynamic inventory node lists tests once for every
registered package/target/feature context, rejects empty or missing mapped IDs,
and records that current inventory in the full-gate artifact. Required producer
feature-off and consumer feature-on compiler/test probes are registered evidence
requirements, not automatically separate suite runs. A workspace check/test
gate satisfies a requirement when its recorded package, target, feature, and
producer/consumer context is an exact match; the inventory node schedules an
additional probe only for a context not already covered. `verify` schedules the
static and Cargo-owned nodes separately under their proper resource claims;
the public `test:manifest` command orchestrates both without duplicate
execution.

A standalone filtered Rust feedback gate performs the same current-context
list check before applying its filter unless it can consume an inventory
artifact with an exact matching source/config/toolchain fingerprint. The list
and feature-proof cost is included in feedback eligibility and full-gate timing
rather than hidden inside nominally static lint.

An absent mapped Rust ID is a stale-manifest `MAPPING_ERROR` in feedback and a
manifest correctness failure in `verify`; failure to spawn or parse the list
command is `INFRA_ERROR`. A compiler or assertion failure from the registered
Rust command remains an ordinary correctness failure.

Manifest lint has negative fixtures for an unmapped tracked executable path,
duplicate project ownership, orphan gate, prerequisite cycle, invalid resource
lock, illegal project-environment override, expired debt, broad catch-all,
empty project inventory, and unresolved or `verify`-disabled replacement ID.

The manifest-backed selector combines four mapping sources:

1. static import relationships for ordinary TypeScript and Svelte tests;
2. explicit `TestingManifest` entries for dynamic, generated, Rust, config,
   script, and filesystem relationships;
3. declared `inputs` from architecture rules;
4. Cargo package and test-module ownership for `.rs` changes.

Cargo package and target ownership and declared feature names come from `cargo
metadata` wherever possible. Gate identity includes package, target, feature
set, and producer/consumer role. A Telegram-internal change therefore maps
differently from a public or `app-test-support` change that also requires the
`extractum` consumer checkpoint.

This distinction is file-based and conservative. `cargo metadata` does not
infer which symbols or lines changed. When internal and public/test-support
code share one file, the whole file maps to the union of producer and consumer
owners until a production refactoring separates those inputs into distinct
files. Manual file-to-gate edges are reserved for relationships the language
and tool graphs cannot express and require owner, rationale, and expiry.

Changes to the manifest, its linter, Vitest configuration, lockfiles, the
selector, architecture tooling, or shared setup expand to static manifest lint
and every affected fast project. A changed test file selects itself. Rust
mapping names the owning package, target selector, feature context, and the
narrowest non-empty test module or exact test known to cover the change; the
owner gate validates that identity against its current runtime list.

`TestingManifest` also contains the small allowlist of paths that genuinely
need no tests, such as prose-only documentation and non-executable assets.
Each allowlist entry has a reason. A broad catch-all glob is forbidden.

Correctness mapping and fast-feedback eligibility are separate facts. A
mapped test becomes feedback-eligible only after a compile-invalidating or
runner-start benchmark ratifies it inside the 15-second envelope. Changes to
`Cargo.toml`, feature sets, test identity, shared runner configuration, or the
owning gate invalidate the corresponding eligibility evidence and make the
selector fail closed until it is remeasured.

### TestingManifest schema evolution

`TestingManifest` is introduced in Slice 1 with an explicit schema version and
the gate-registry, resource, census, and benchmark-scope fields required at that
checkpoint. Slice 2 atomically adds named project and legacy ownership. Slice 3
adds architecture-rule and migration-evidence descriptors. Checkpoint 4A adds
affected-path mapping, transition debt, and feedback eligibility.

At every checkpoint the schema version, producers, consumers, fixtures, and
lint rules change atomically. Lint requires every field of the current version,
rejects unknown fields and obsolete compatibility shapes, and never requires a
future-slice field from an earlier schema version. A checkpoint cannot retain a
partially upgraded manifest or a reader that silently ignores new fields.

### Transitional fast-feedback debt

Product code that is currently covered only by a slow gate is not allowlisted
and is not forced to gain a fast test in one big-bang change. `TestingManifest`
contains a separate temporary `slowOnlyProductDebt` registry. Each entry has:

- stable debt ID and exact path or narrow path pattern;
- reason the current correctness owner is slow;
- owner, tracking reference, and intended fast-assurance seam;
- non-empty deferred gate IDs and exact slow owner commands;
- remediation milestone and removal slice;
- last measured duration and expiry date.

The registry is disjoint from no-test allowlists and quarantines, and broad
catch-all entries are forbidden. A directly changed slow test needs no debt
entry, but a product input without a fast seam returns `MAPPING_ERROR` unless
it has a current debt entry. An unexpired entry contributes a deferred
obligation; it does not force an early return before fast evidence for other
inputs executes. A missing or expired entry returns `MAPPING_ERROR`, and
manifest lint also fails the full static gate.

New debt cannot be added merely to remove a named acceptance probe from
measurement. In Rust checkpoint 4A it is limited to the explicitly baselined
root `extractum` and mixed producer/consumer cohorts; the already bounded
internal `extractum-telegram` and `extractum-analysis` cohorts must already be
`RATIFIED`. Final repository-wide SLA activation requires all production-
source debt entries to be removed; directly edited slow test files may
continue to produce a deferred obligation by design.

### Feedback budget envelope

The selector distinguishes an affected set inside the ratified fast envelope
from a broader but fully mapped set. Configuration, lockfile, shared setup, or
multi-owner changes expand to complete owner gates only when that shape has
measured evidence within 15 seconds.

If the complete selection shape is outside the ratified envelope, the selector
does not start a partial run. It returns `DEFERRED_ONLY` within the end-to-end
deadline and prints every required owner command plus `npm.cmd run verify`.
This is distinct from a mixed selection whose complete eligible fast subset is
inside the envelope while another input has only a registered slow owner; the
mixed selection runs that fast subset before reporting its deferred outcome.

Crossing the deadline after an eligible run has started returns
`BUDGET_EXCEEDED`. A named single-owner acceptance probe, including each
compile-invalidating Rust probe, cannot be preclassified as broad or deferred
to remove it from the sample.

### Selection flow

For every invocation the selector:

1. cheaply validates the manifest schema and references;
2. discovers and normalizes changed paths;
3. classifies every path before starting a runner;
4. resolves each input's fast evidence and known deferred slow gates;
5. rejects the invocation if any path is unknown, any debt is invalid or
   expired, or slow-only product code has no valid debt;
6. computes the complete eligible fast subset and records which product inputs
   still have only a changed slow-test owner or transition debt;
7. preclassifies a complete mapped selection outside the ratified feedback
   envelope as `DEFERRED_ONLY` without starting a partial suite;
8. otherwise runs the complete eligible fast subset under the feedback budget,
   even when another input remains deferred;
9. applies result precedence, reports every deferred gate without claiming
   completion, and writes timings to a gitignored local artifact.

If no primary failure occurred and at least one input still has only deferred
assurance, the final result is `DEFERRED_ONLY`, even when the eligible fast
subset was non-empty and passed. `PASS` is possible only when a non-empty fast
set passed and every product input received fast assurance. It may still list
supplementary completion gates, such as a broader E2E scenario, that do not
represent an uncovered input. Both results are labelled `feedback-only`; only
`verify` can report repository completion.

### Result statuses and exit codes

| Status | Exit | Meaning |
| --- | ---: | --- |
| `PASS` | 0 | A non-empty mapped fast set passed; deferred completion gates may remain |
| `NO_APPLICABLE_TESTS` | 0 | Every input is explicitly allowlisted, and every reason is printed |
| `NO_CHANGES` | 0 | The requested working-tree/base comparison is empty; no completion claim is made |
| `TEST_FAILED` | 1 | At least one selected assertion failed |
| `MAPPING_ERROR` | 2 | A path is unknown, or product code has neither a fast assurance seam nor valid transition debt |
| `INFRA_ERROR` | 3 | Git discovery, spawning, parsing, artifact writing, or cleanup failed |
| `BUDGET_EXCEEDED` | 4 | The feedback wall-clock budget expired |
| `DEFERRED_ONLY` | 5 | Every path is mapped, but at least one input remains covered only by a slow/debt gate, or the complete set is outside the ratified envelope; eligible fast evidence may be non-empty and is reported with every owner command |
| `INTERRUPTED` | 130 | The user interrupted the invocation; owned descendants were terminated and prior observations are retained |

The JSON result contains the status, normalized inputs, per-input assurance,
selected tests, deferred gates and debt IDs, primary and secondary failures,
durations, tool versions, and suggested reproduction commands.
These status values are owned by the testing orchestrator, are ephemeral, are
not persisted in application data, and are not exposed in the product UI.
Their ownership must be registered in `docs/value-registry.md`.

Path classification and statically detectable mapping errors are resolved
before a runner starts; a dynamic Rust ID is validated by listing before test
execution. After a runner starts, interruption is always primary exit 130;
previously observed failures remain secondary. Without interruption, an
observed assertion failure is primary over a later budget, cleanup, or artifact
failure. If no assertion failed, expiration of the feedback deadline is
primary over descendant-cleanup or artifact failure; another infrastructure
failure is primary otherwise.
`DEFERRED_ONLY`, `PASS`, and the two empty-input success statuses are considered
only after no execution failure remains. A deferred obligation can therefore
never hide a fast assertion failure, timeout, or infrastructure failure.

An empty Vitest or Cargo selection can never be translated to `PASS`. When an
exact Cargo test name is not known, the implementation lists tests first and
then runs a verified non-empty filter.

## Source-Contract Replacement

### Assertion disposition

Every current source-dependent assertion obligation receives exactly one
disposition:

1. **Behavior**: replace it with a unit, component, Rust, integration, or E2E
   test through a public seam.
2. **Architecture**: express it through an import graph, AST, compiler
   metadata, Cargo metadata, or a structured repository rule.
3. **Tool-owned**: delegate generic unused-code, dependency, or cycle checks
   to a maintained third-party tool.
4. **Delete**: remove exact-name, statement-order, formatting, substring, and
   other implementation-detail assertions with no independent product or
   architecture value.

Large golden snapshots of production source are not an acceptable
replacement. Each migration slice records the disposition of the assertions
it removes so silent coverage loss can be reviewed.

Before project ownership changes, the audit freezes a canonical immutable
baseline containing the commit, tool versions, source-contract file census,
and baseline hash. The migration unit is one source-dependent assertion
obligation, not one test declaration: a single test title can contain many
independent `expect` calls or parameterized source checks.

Each obligation has an immutable ID derived from repository path, full nested
suite/test title or title template, lexical assertion ordinal from the frozen
AST, and normalized parameter/authority-data fingerprint. It records the
plain-language invariant, owner package, disposition, migration state,
replacement test/rule/tool IDs, or an obligation-specific deletion rationale.
A loop, helper, or parameterized test records the complete parameter-set
fingerprint so dropping one case cannot silently preserve the same row.

Ledger state is one of `PENDING`, `DUAL_RUN`, or `RETIRED`; disposition is a
separate immutable field. `PENDING` requires legacy evidence to exist.
`DUAL_RUN` requires both legacy and replacement evidence to resolve and
execute. `RETIRED` requires legacy evidence to be absent and either enabled
replacement evidence or an obligation-specific `Delete` rationale. Allowed
transitions are `PENDING` to `DUAL_RUN` to `RETIRED`; direct `PENDING` to
`RETIRED` is allowed only for `Delete`.

A validation command proves that every baseline obligation appears exactly
once, every pending obligation remains in the legacy inventory, every non-
deleted replacement ID resolves, and its owner gate is enabled in `verify`.
One replacement may cover several obligations only when the ledger explicitly
assigns them the same stated invariant. No checkpoint permits a `DUAL_RUN` row;
obligations outside the current Slice 3 sub-slice may remain `PENDING`, while
final Slice 3 completion permits neither `PENDING` nor `DUAL_RUN`. The completed
ledger remains as checked-in verification evidence after migration.

These ledger-state values are persisted only in tooling evidence, not
application data, and have no product API/UI impact. Slice 1 registers their
ownership and transition contract in `docs/value-registry.md` before the
baseline is frozen.

### Generic tool ownership

Use `dependency-cruiser` for generic TypeScript/Svelte import boundaries,
forbidden dependency directions, and cycles. It uses the project's installed
TypeScript and Svelte compilers rather than bundling another transpiler.
Its rule names are stable migration-evidence IDs, its configuration points to
the repository TypeScript configuration, and configuration/schema validation
must pass before graph findings are authoritative.

Use Knip for unused files, exports, and dependencies. Knip runs in the full
static gate, not the 15-second affected loop. Entry points and workspaces must
be explicit so SvelteKit, scripts, sidecars, and research adapters are not
mistaken for dead code.

Knip declares the root, Gemini sidecar, and Gemini adapter workspaces with
complete intended `entry` and `project` sets. Custom values are treated as full
replacements for Knip defaults, not additive fragments; plugin overrides also
state the complete intended override. Tool-native configuration owns these
sets, while `TestingManifest` references only the Knip gate and its stable
evidence IDs.

Both tools start with reviewed, narrow rules. Existing findings are either
fixed or entered into an exact, reasoned baseline; a blanket repository ignore
or generated allow-all list is not acceptable.

Before either tool becomes authoritative, a small checked-in canary proves the
configured graph resolves Svelte files, `$lib` and `$app` aliases, `?raw`
imports, root scripts, the Gemini sidecar, and the Gemini adapter. Knip's
workspace configuration declares root SvelteKit entry points, script entry
points, the sidecar entry, adapter entry/config/test surfaces, and their
project globs explicitly. The full gate runs its comprehensive unused-file,
export, and dependency analysis rather than a production-only subset.
These canaries run in the static gate. Exact baseline exceptions require a
reason, owner, tracking reference, and expiry. A tool finding cannot satisfy a
migration obligation unless its stable evidence ID resolves and is scheduled
by `verify`.

Use `@vitest/coverage-v8` for explicit coverage analysis. Its version must be
compatible with the installed Vitest major and locked in `package-lock.json`.

Do not add `ts-morph` initially. TypeScript and `svelte/compiler` already
provide the necessary ASTs. A later proposal may reconsider this only if the
custom adapter remains materially harder to maintain after the first rules
are implemented.

### Repository index

Domain-specific structured rules use one `RepositoryIndex` service:

- TypeScript is parsed with the installed TypeScript compiler API.
- Svelte is parsed with `svelte/compiler`.
- dependency-cruiser provides the generic JavaScript/TypeScript/Svelte import
  graph.
- Cargo structure comes from `cargo metadata` and owner-package compiler
  evidence rather than Node-side Rust text parsing.
- Every file owned by the custom `RepositoryIndex` is read and parsed at most
  once per index version. Independent third-party tools may parse their own
  inputs and are measured as separate gates.
- One-shot runs reuse a disk cache keyed by content hash, parser version,
  and rule-set version; correctness never depends on modification time alone.
- Watch mode keeps the index in memory and invalidates only changed files and
  their derived relationships.

Each custom rule declares a stable ID, its input paths, and an evaluator. It
returns diagnostics containing rule ID, repository-relative path, line, and a
plain-language reason. A parse failure is `INFRA_ERROR`; it is never treated as
zero violations.

### Migration order and guardrail

Migrate the three largest Telegram and analysis files first, followed by the
remaining source-contract cohort. During a slice, old and new evidence may
coexist only long enough to prove equivalence; the slice removes the old
evidence before its checkpoint.

At completion, no test may read arbitrary production source solely to assert
`includes`, `match`, or a regular expression. Fixture readers and tests of the
indexer itself remain allowed. A structured architecture rule prevents new
test code outside approved helpers from reading production roots through
`node:fs` or raw-source imports. The equivalent Rust-owned policy rejects
test-only `include_str!`, `include_bytes!`, filesystem reads, or regular-
expression scans over production `.rs` files outside explicitly registered
indexer and fixture owners. Moving a text contract from TypeScript to Rust is
not a valid migration.

## Browser and Component Ownership

`sidecars/gemini-browser/src/answer-extractor.test.ts` moves from Vitest to a
Playwright-owned suite. Playwright Test owns its built-in worker-scoped browser
and test-scoped isolated context/page fixtures and performs their cleanup. The
suite does not launch or close those resources manually. A custom worker
fixture owns only the server or IPC test seam it starts, and test artifacts are
scoped through `testInfo`.

The critical E2E tier includes:

- the existing Gemini browser adapter Playwright scenarios;
- Chromium-backed answer extraction currently launched from Vitest;
- a small main-application browser smoke for critical navigation and error
  rendering, using an explicit Tauri IPC test seam where browser execution
  cannot provide real Tauri IPC.

The smoke is deliberately narrow; this redesign does not attempt to reproduce
the entire Tauri desktop stack in a browser.

The Playwright server fixture owns startup and reads the URL it actually
started. Tests do not assume Vite's default port or attach to an unrelated
already-running server.

Playwright retains trace, screenshot, console, and server logs on failure.
Global retries remain zero. The suite starts with one worker where a shared
server or artifact path exists. Parallel workers require worker-scoped servers
and `testInfo`-scoped artifacts plus a measured improvement.

Component tests stay in jsdom. The project has one explicit Testing Library
cleanup owner; per-file cleanup is removed unless a file owns an additional
resource. `happy-dom` is not introduced without a compatibility benchmark for
the actual Svelte component cohort.

## Complete Local Verification Gate

### Gate registry

The gate-registry portion of `TestingManifest` is the single executable source
of truth for local verification. Each gate declares:

- ID and human title;
- command and arguments;
- owned inputs;
- prerequisite gates;
- reporting class (`light`, `cpu-heavy`, `browser`, or `os-integration`);
- numeric CPU-slot and memory-reservation claims;
- zero or more exclusive resource-lock IDs;
- platform constraints;
- timeout budget;
- expected artifact locations.

Both `test:feedback` and `verify` consume this ownership data. Documentation
lists the public commands but does not duplicate executable path mappings.

### Full graph

`npm.cmd run verify` covers these groups:

| Group | Required contents |
| --- | --- |
| `static` | `svelte-check`, sidecar and adapter typechecks, sidecar build, the static phase of `test:manifest`, dependency-cruiser, Knip, `git diff --check` for the local worktree, and generated/lockfile drift checks owned by changed build steps |
| `frontend` | Complete `unit-node`, `component`, and `architecture` projects, transitional `legacy-contract` until Slice 3 retires it, and owned sidecar/adapter unit tests |
| `rust` | Rustfmt, Cargo-owned manifest inventory/feature proofs, and workspace `cargo check --all-targets` plus `cargo test --all-targets` |
| `os-integration` | Vitest- and script-owned real Windows process, filesystem, socket, and SQLite tests |
| `e2e` | Critical Playwright suite after its required app or sidecar readiness gates |

These groups are reporting namespaces, not indivisible scheduler jobs. Each
declared gate is scheduled independently when its prerequisites, numeric
resource claims, and exclusive locks permit. The reference-environment
scheduler policy freezes total CPU slots and memory capacity in the benchmark
artifact; the sum of active claims may not exceed either capacity, and two
active gates may not hold the same exclusive lock. Slice 1 chooses those
numeric values from the recorded overlap and peak-memory probes, not an
informal `heavy` label. For example, `svelte-check`, sidecar typecheck, adapter
typecheck, dependency-cruiser, and Knip overlap only when the frozen claims
permit; their durations are not assumed to be either fully serial or fully
parallel.

Rust tests that use real OS resources belong semantically to the
`os-integration` tier but execute exactly once under their Cargo owner unless
they are moved to a distinct Cargo integration target. The registry must not
schedule the same Rust test again in the separate `os-integration` group.

Every orchestrated Cargo invocation declares the `cargo-exclusive` lock so
concurrent gates do not contend over canonical `src-tauri/target`. Browser and
Windows tests that cannot safely overlap declare the same `host-exclusive`
lock; a gate that does not need exclusivity does not acquire it merely because
of its reporting group. The scheduler lets already-running independent gates
finish after a failure, does not start failed dependents, and prints an
aggregate result.

The scheduler must terminate child process trees on timeout or interruption,
including Windows descendants. A gate or hard-safety timeout is an
infrastructure failure. Only expiration of the feedback command's declared
15-second workflow deadline produces `BUDGET_EXCEEDED`.

Local reports live under one gitignored artifact root and include group/test
durations and failure diagnostics. They are not committed.

### Full-gate result contract

`verify` exits with code 0 only when every required gate completes. User
interruption is always primary exit 130 after terminating owned descendants;
any assertion or infrastructure failures observed before interruption remain
under `secondaryFailures` because the requested run is incomplete.

Without interruption, a static, test, or behavior failure is primary exit 1.
A concurrent or later timeout, unsupported host, required capability skip,
spawn failure, cleanup failure, or artifact failure remains secondary and
cannot erase the correctness diagnostic. If no correctness check failed, any
of those infrastructure conditions is primary exit 3; additional conditions
remain secondary. A dependent gate that was not started records its causal
`blockedBy` gate and does not become a new root failure.

The required Windows integration tier cannot be skipped on a non-Windows host
or because the host lacks a required permission. Such a run is incomplete and
returns exit 3 with a capability diagnostic. Only an explicitly optional
diagnostic outside `verify` may report a successful capability skip.

### Full-gate budget ratification

The 90-second number is an initial objective, not an evidence-backed budget at
design time. Budget state is scoped rather than global: each artifact names the
feedback cohort, watch matrix, or full execution-graph fingerprint it covers.
Slice 1 may ratify an existing measured cohort, but it cannot ratify the future
full-gate scope before that inventory exists.

Slice 1 measures every existing gate, including the sidecar and adapter
commands currently outside `verify`, with one discarded warm-up and three
retained warm samples. It also records CPU/memory overlap and computes the
resource-constrained critical path for the proposed scheduler.

A not-yet-implemented gate receives no invented duration. Instead, the model
publishes the maximum remaining critical-path allowance for that gate. Each
later slice measures its new gates and recomputes the model. After all final
mandatory gates exist, twenty complete runs provide the observed nearest-rank
p95 term.

The single authoritative full-gate formula is
`max(observed_p95, 1.15 * modeled_critical_path_p95) <= 90 seconds`. Both terms
use the same complete inventory, scheduler policy, and reference environment.
Thus an observed p95 of 85 seconds is insufficient when the modeled p95 is also
85 seconds; it is not ambiguously accepted by the observed term alone. If the
formula fails, the program must optimize the graph, remove duplicated work, or
return to the user with measured evidence for an explicit budget revision. It
must not silently weaken the target or claim final acceptance.

The full-gate fingerprint includes the normalized execution-relevant manifest
fields (excluding generated timing/result metadata), independent test census,
mandatory gate IDs, commands and arguments, prerequisites,
numeric resource claims, lock IDs, runner/build configuration, lockfiles,
toolchains, and reference OS/hardware policy. A change to any of those inputs
immediately invalidates the artifact to `TARGET_NOT_RATIFIED`; a stale artifact
cannot authorize the 90-second claim. `verify` remains a mandatory correctness
gate while replacement measurement is pending and reports that its performance
budget is not currently ratified.

Ordinary product or test-body edits do not invalidate the graph fingerprint by
themselves; their performance effect is observed through the rolling overage
rule below. This avoids rerunning twenty samples for every implementation edit
without allowing a persistent measured regression to retain ratified status.

After ratification, every `verify` run compares its wall time with the ratified
budget. Exceeding it prints a prominent non-failing performance warning and
records the overage and slow critical path in the local artifact. Correctness
exit codes remain unchanged.

The orchestrator also keeps a machine-local rolling counter of comparable,
complete, correctness-passing runs for the same graph fingerprint and reference
environment. One under-budget run resets the counter; failed, interrupted, or
infrastructure-incomplete runs neither increment nor reset it. Three
consecutive over-budget runs invalidate the performance artifact to
`TARGET_NOT_RATIFIED` while preserving each run's correctness exit code.
`npm.cmd run verify:performance` then owns the complete twenty-run
reratification; individual convenient runs cannot restore `RATIFIED`.

### Avoiding duplicated Rust completion work

Focused Rust commands remain RED/GREEN accelerators. The project workflow must
not require a complete workspace test immediately before `verify`, because
`verify` already owns that completion evidence. A Rust slice runs narrow
RED/GREEN tests and package checkpoints while developing, then runs the single
end-of-slice `verify` gate.

Within `verify`, one Cargo execution may satisfy both a workspace gate and a
manifest feature-proof requirement only when their package/target/feature and
producer/consumer evidence keys match exactly. The current workspace
`cargo test --all-targets` therefore satisfies the `extractum` consumer
feature-on proof when that dev-feature context is recorded; the orchestrator
must not run the root suite again. A producer feature-off or other unmatched
context remains a distinct focused command under the same `cargo-exclusive`
lock.

## Coverage, Flake, and Quarantine Policy

`verify:coverage` uses V8 coverage outside the daily feedback loop. It covers
declared critical TypeScript modules and uses a baseline ratchet: a critical
module may not lose covered statements or branches without an explicit design
decision. There is no repository-wide percentage target.

The initial critical-module manifest contains the new selector, gate-registry
loader, repository index/cache, and process-cleanup helpers introduced by this
redesign. Adding existing product modules remains an owner decision rather
than an implicit expansion of this infrastructure project.

`verify:stability` repeats named suspect suites and records every individual
result. It never retries a failed result and replaces it with a later pass.
The command is diagnostic evidence, not a correctness gate that can rescue a
failure.

A skipped or quarantined test requires a repository-owned entry with:

- test ID or exact path/name;
- reason;
- owner;
- tracking reference;
- expiry date.

`verify` fails when an entry is expired, no longer matches a test, or a skip is
present without an entry. This enforcement is local because the design has no
CI or hooks.

Tests that create temporary directories, processes, sockets, browser
contexts, or databases own deterministic cleanup. Permission-based Windows
capability skips must produce an explicit skipped diagnostic rather than a
silent early return.

## Third-Party Tool Decisions

| Tool | Decision | Reason |
| --- | --- | --- |
| Vitest projects and sharding | Keep/adopt where measured | Existing runner; projects express environment ownership and sharding remains available for large explicit runs |
| Playwright | Expand ownership | Already installed and is the correct lifecycle owner for Chromium |
| dependency-cruiser | Adopt | Replaces custom generic import/cycle contracts with maintained rules |
| Knip | Adopt | Replaces duplicated unused-file/export/dependency checks |
| `@vitest/coverage-v8` | Adopt | Provides explicit runtime coverage without instrumenting daily feedback |
| `cargo-nextest` | A/B experiment only | May improve reporting/isolation, but process-per-test can be slower for the large Windows binary |
| `sccache` | A/B experiment only | May improve clean or branch builds but does not fix test harness runtime or linking by itself |
| Nx/Turborepo | Do not adopt | The custom graph is deliberately limited to test ownership/scheduling and does not need a second artifact cache or general build system |
| Testcontainers | Do not adopt | Production storage is embedded SQLite and existing providers use local fakes |
| Mockall | Do not adopt globally | Existing narrow traits and handwritten fakes avoid extra proc-macro cost |
| Global retries | Prohibited | They hide ownership and flake defects |

The implementation checks Node and tool compatibility before adding new
packages and records locked versions. Absence of optional benchmark tools is
not a failure and does not authorize an implicit machine-global installation.

## Performance Measurement Contract

### Environment

The reference environment is the current Windows development workstation.
Every benchmark report records commit, Windows version, processor count, Node,
npm, Vitest, Playwright, Rust, and Cargo versions plus the executed test
inventory.

Dependency installation is outside all timing budgets. Filesystem, npm, and
Cargo caches are not forcibly flushed unless a measurement is explicitly
labelled cold.

### Vitest new-process startup floor

Slice 1 records baseline proxy timings for trivial existing Node, Svelte/jsdom,
and architecture tests under the current mixed configuration. Those proxies
are diagnostic and cannot ratify projects that do not yet exist. After Slice 2
partitions the suite, `TestingManifest` names one fixed existing one-test probe
for each fast Vitest project. A probe must collect and execute exactly one real
test; an empty selection or `passWithNoTests` result is invalid.

The post-partition benchmark uses one disclosed unscored warm-up followed by
twenty predeclared new-process attempts per fast project in alternating order,
with installed dependencies and normal warm caches. It records child spawn to
first test start, assertion execution, last test completion to process exit,
and total selector/reporting time. Every one of the twenty scheduled attempts
remains in the report.

Startup-floor timing has no separate invented threshold and does not relax the
15-second end-to-end contract. If startup alone consumes that envelope, the
project remains `TARGET_NOT_RATIFIED` until plugin/config loading, filtering,
or process orchestration is redesigned. A change to project inventory,
plugins, setup, pool, workers, timeouts, or shared configuration invalidates
this evidence and feedback eligibility until the measurement is repeated.

### One-shot feedback

One-shot timing starts when `test:feedback` starts its new Node process and
ends when the final human and JSON result are complete. It includes Git
discovery, mapping, runner startup, selected tests, and reporting. It does not
use a persistent watcher or daemon. Normal disk and build caches are reused.
The 15-second budget is end-to-end, including cancellation, descendant cleanup,
and final reporting; the runner reserves time for those stages before its test
execution deadline.

Before execution, the final acceptance artifact freezes stable probe IDs,
byte-exact reversible mutations, expected owner/test selections or deferred
gates, and alternating order for twelve probes:

- two pure TypeScript behavior inputs;
- two Svelte component inputs;
- two structured architecture inputs;
- one widest selection proven to remain inside the ratified feedback envelope;
- one mapped multi-owner selection proven to return `DEFERRED_ONLY` without
  starting a partial suite;
- one compile-invalidating `extractum` Rust input;
- one compile-invalidating `extractum-telegram` Rust input;
- one compile-invalidating Telegram public or `app-test-support` input that
  exercises both producer and `extractum` consumer ownership;
- one compile-invalidating `extractum-analysis` Rust input.

Each probe runs three times in alternating order for 36 total attempts. Rust
probes start with warm dependency caches but modify an owning source file so
the package compilation/test target is genuinely invalidated; every temporary
edit is restored byte-for-byte. The report publishes median, nearest-rank p95,
and maximum.

During the selector phase the same Rust probes are retained, but valid
transitional debt can leave the root/consumer cohorts above target and the
budget state `TARGET_NOT_RATIFIED`. Final repository-wide activation requires
the overall p95 to meet 15 seconds and every named Rust probe to complete
within 15 seconds in all three attempts; aggregate arithmetic cannot hide a
failing root cohort.

Every scheduled attempt remains in the report. Assertion or infrastructure
failure fails reliability acceptance and cannot be replaced by an extra green
attempt. Expiration of the selector's 15-second deadline is instead a retained
over-target `BUDGET_EXCEEDED` timing sample; it does not automatically override
the declared p95 formula. With 36 attempts, nearest-rank p95 permits at most the
single maximum sample to exceed 15 seconds, while the named Rust cohort rule
above remains deliberately stricter at three of three.

The benchmark harness has a separate 60-second hard safety timeout. Failure to
produce and clean up a final result by that bound is `INFRA_ERROR` and fails
acceptance. Both budget expiration and hard-timeout timing are recorded at the
observed termination time. The performance distribution is never computed from
a hand-picked successful subset.

The matrix fingerprint includes those probe definitions, selector and manifest
versions, fast project inventories, mapping/eligibility data, Cargo
package/target/feature contexts, and relevant runner/lockfile versions. A
change to any of them returns the cohort to `TARGET_NOT_RATIFIED` until the
complete 36-attempt matrix is repeated.

### Warm watcher

Watcher timing starts at the file-change event and ends when the affected
result is printed. Before execution, the benchmark artifact freezes stable
probe IDs, byte-exact reversible mutations, expected owner/project sets, and
alternating order for twenty sequential edits: seven TypeScript behavior,
seven Svelte component, and six structured architecture edits. The probes use
at least two files in each category so one unusually cheap file cannot define
the cohort. Every planned attempt remains in the report; a failed mutation or
unexpected selection fails acceptance rather than being replaced.

Initialization time is reported separately and is not part of the 5-second
rerun budget. The watch-matrix fingerprint includes probe definitions, project
inventories, watcher/config/plugin versions, and invalidation rules; changing
one returns the budget to `TARGET_NOT_RATIFIED` until the twenty-edit matrix is
repeated.

### Full local gate

The full-gate sample uses installed dependencies and warm build caches and
contains twenty consecutive planned runs of an unchanged inventory. It
publishes median, nearest-rank p95, and maximum. Any correctness or
infrastructure failure fails reliability acceptance and remains in the
evidence; it is not replaced. A cold build is reported separately and cannot
be averaged into the warm target.

Until the complete implemented inventory meets the candidate 90-second p95,
the artifact remains `TARGET_NOT_RATIFIED`. The full gate stays complete and
mandatory but has a measured duration, not a claimed SLA. A missed target is
resolved through further portfolio/orchestration work or an explicit
user-approved design amendment; the non-failing per-run warning is regression
visibility, not a mechanism for accepting a missed target.

Machine-specific durations remain verification evidence; they are not encoded
as millisecond assertions inside ordinary unit tests. The orchestrator itself
does enforce the 15-second feedback budget because that is a productized
developer-workflow contract.

## Failure Handling

- No failing measurement is silently rerun and substituted into an aggregate.
- A changed test inventory invalidates a before/after comparison.
- A parser or metadata error stops the affected architecture gate.
- A selector deadline expiration reports the slowest completed work and a
  reproduction command; it is primary `BUDGET_EXCEEDED` unless an assertion
  failure or user interruption already has higher precedence. Failure to return
  by the benchmark's separate hard safety timeout is `INFRA_ERROR`.
- A child-spawn or process-cleanup failure records `INFRA_ERROR` and identifies
  any descendant process that could not be terminated; it remains secondary
  when the precedence contract names a higher primary result.
- A missing Playwright browser reports a bootstrap instruction; it is not
  presented as a behavior failure.
- A Cargo filter that executes zero tests is invalid evidence.
- A deferred slow gate is always named in feedback output and can only be
  discharged by its explicit command or full `verify`.
- Local artifact-write failure is an infrastructure failure because the
  machine-readable result is part of the command contract; it cannot replace
  an already primary correctness failure or interruption.

## Migration Sequence

### Slice 1: Registry and observability

Introduce the first version of `TestingManifest` with its gate registry,
resource and independent-census fields, plus the result schema, timing
artifacts, baseline report, and aggregate local runner without changing test
ownership. Preserve the current `verify` coverage while establishing
before/after evidence.

The baseline is mandatory and records the current commit, complete inventories,
physical source-contract counts, tool versions, and these timing cohorts:

- one trivial `unit-node` test from a new process;
- one trivial Svelte/jsdom component test from a new process;
- one trivial architecture test from a new process;
- every existing `verify` gate separately;
- sidecar typecheck/build and adapter typecheck/unit/E2E currently outside
  `verify`;
- the current full serial gate;
- safe overlap probes needed to model the resource-constrained scheduler.

Each startup-floor and gate cohort uses one discarded warm-up and three
retained warm attempts. Slice 1 publishes runner/config startup separately
from assertion execution so the remaining 15-second budget is visible. It
also creates the initial budget-decision artifact: implemented surfaces are
either `RATIFIED` or measured misses, while future surfaces remain
`TARGET_NOT_RATIFIED` with no invented duration.

Slice 1 registers `RATIFIED` and `TARGET_NOT_RATIFIED` in
`docs/value-registry.md` when it introduces them; documentation of a new status
cannot be deferred to a later cleanup slice.

The same checkpoint registers manifest-controlled reporting classes and
exclusive lock IDs as tooling-only vocabularies with no persistence, product
API, or UI impact.

After the current source-contract census is rebaselined, Slice 1 also freezes
the immutable assertion-obligation ledger and registers its `PENDING`,
`DUAL_RUN`, and `RETIRED` tooling states before Slice 2 can relocate any
obligation through a lineage map.

### Slice 2: Project and browser ownership

Split Vitest projects, establish project-specific worker/resource policies,
move Chromium extraction to Playwright, and remove competing component
cleanup ownership. Atomically advance the manifest schema for named project
and legacy ownership. Assign every unmigrated source-contract file to the
transitional `legacy-contract` project and prohibit new entries. Keep behavior
assertions equivalent. Repeat the preregistered new-process startup-floor
benchmark against the real named projects; the Slice 1 proxies are not
post-partition eligibility evidence.

### Slice 3: Source-contract replacement

Migrate the three largest Telegram and analysis contracts first, then the
remaining cohort. Add dependency-cruiser, Knip, the repository index, and the
guardrail against new arbitrary source-text tests. Remove the
`legacy-contract` project after its inventory reaches zero.

This is an epic composed of bounded atomic sub-slices: Telegram, analysis
crate boundary, analysis application, and then the remaining contract
clusters. Each sub-slice records assertion dispositions, removes its replaced
old path, runs the applicable package checkpoints, and ends with `verify`.

### Slice 4: Fail-closed feedback

Introduce the `TestingManifest` path mappings, one-shot selector, result
statuses, budget, watch invalidation, Rust/package mapping, and compatibility
aliases. This slice has three checkpoints:

1. **4A — selector correctness:** every production path has a fast correctness
   owner or valid transitional debt entry, and every directly changed slow
   test has an explicit deferred command. Only benchmark-ratified classes can
   return `PASS`. Rust debt is limited to the baselined root and mixed
   producer/consumer cohorts; internal `extractum-telegram` and
   `extractum-analysis` probes must be ratified before this checkpoint closes.
2. **4B — Rust fast-seam closure:** root and producer/consumer performance
   debt is removed by moving the minimum necessary production logic behind an
   approved smaller package boundary. Test-target or harness refinements may
   support that work but cannot replace the required compile-invalidating
   product probes or count as success while those probes remain over budget.
3. **4C — repository-wide activation:** all production-source debt is gone,
   every named Rust probe meets the budget, and the full 15-second matrix is
   `RATIFIED`.

Checkpoint 4A introduces selector statuses and exit 5 atomically with their
workflow documentation. `docs/value-registry.md` records ownership and lack of
persistence/API/UI impact; `AGENTS.md` requires every printed owner command;
and `docs/project.md` explains both changed-slow-test and transition-debt cases.
No path can be allowlisted merely to enable a checkpoint.

### Slice 5: Documentation and workflow policy

Consolidate `AGENTS.md`, `docs/project.md`, and `docs/value-registry.md`; remove
the duplicated Rust completion flow; and document coverage, stability,
quarantine, and local artifact handling. This slice checks wording consistency
but is not the first publication of budget or selector status values. The
agent workflow continues to treat exit 5 as a known `DEFERRED_ONLY` handoff: it
runs every printed owner command, such as `npm.cmd run test:e2e`, rather than
diagnosing the selector itself as broken.

### Slice 6: Optional acceleration experiments

Benchmark `cargo-nextest` and `sccache` independently. The nextest A/B uses
twenty paired warm runner attempts and proves equivalent unit, integration,
and doctest coverage; any inventory nextest does not execute remains an
explicit Cargo gate. The sccache A/B separately measures clean package rebuilds
and compile-invalidating incremental edits with cache enabled and disabled; an
unchanged warm full gate is not sccache evidence.

Adopt an optional tool only when its relevant paired median improves by at
least 20%, its p95 does not regress, inventory and correctness are unchanged,
and diagnostics remain at least as useful.

Each slice ends with a clean checkpoint and a successful authoritative gate.
Old and new ownership paths may coexist only inside the slice that migrates
them.

## Planning Decomposition

This is a program-level design, not one atomic implementation branch. After
written-spec approval, implementation planning is split by the six ordered
slices above. Slice 3 is further split into the four named contract clusters,
and Slice 4 is split into checkpoints 4A, 4B, and 4C.
Each plan references this specification, defines its own RED/GREEN tests and
rollback boundary, and reaches a clean checkpoint before the next plan starts.
The final performance and portfolio acceptance criteria apply after the last
required slice, while every intermediate plan preserves or strengthens the
then-current authoritative gate.

## Rust Verification Loops

Any implementation plan produced from this design must name every affected
Cargo package for each Rust-changing slice. Initial source-contract migration
owners include `extractum`, `extractum-telegram`, and `extractum-analysis`;
other workspace packages are included only when a migrated invariant or test
seam directly affects them. Telegram migration must checkpoint both the
producer and its immediate `extractum` consumer where the existing
`app-test-support` seam participates.

For every Rust behavior replacement:

1. list tests first when the exact name is not already known;
2. select the manifest-owned target and feature context: `--lib`,
   `--test <target>`, or another explicitly registered Cargo test target, plus
   any required `--features`; then run the narrow RED test with
   `cargo test --manifest-path src-tauri/Cargo.toml -p <package> <target-selector> <feature-args> <full-test-name> -- --exact`;
3. implement or move the behavior seam;
4. run the same non-empty exact GREEN test;
5. checkpoint the package's normal/default surface with
   `cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
6. checkpoint the same normal/default surface with
   `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
7. when the manifest-owned RED/GREEN context is not covered by those default
   commands, run the additional registered feature-on check and non-empty test
   checkpoint for that exact package/target/feature context;
8. checkpoint an immediate dependent package when a public cross-crate
   interface changes;
9. end the Rust-changing slice with `npm.cmd run verify`.

All commands reuse canonical `src-tauri/target`. The plan must not introduce
slice-specific `codex-*` targets. An exact or filtered run that executes zero
tests is not verification. Listing and execution use the same package, target,
and feature context; a target split introduced for the 15-second loop is not
silently forced back through `--lib`.

If a cross-package test-only seam is necessary, the producer exposes a narrow,
non-default `*-test-support` feature. The plan must prove the producer's normal
surface with
`cargo check --manifest-path src-tauri/Cargo.toml -p <producer> --lib --no-default-features`
and prove the seam through the consumer's
`cargo test --manifest-path src-tauri/Cargo.toml -p <consumer> --all-targets`.
The normal dependency edge remains feature-free.

## Documentation and Ownership

- `AGENTS.md` owns agent-facing command selection and the mandatory end-of-
  slice full gate.
- `docs/project.md` owns the developer-facing daily loop and explains that
  feedback success is not completion.
- `docs/value-registry.md` owns the ephemeral selector, budget, ledger-state,
  project/reporting-class, and resource-lock vocabularies and records that they
  have no application persistence, product API, or UI impact.
- `TestingManifest` owns executable input mappings, feedback eligibility,
  project classification, rule descriptors, allowlists, transition debt,
  quarantine metadata, gates, prerequisites, and resource policies.
- Package scripts expose stable public entry points and must not duplicate
  mapping logic.
- The benchmark report owns environment-specific timing evidence.

## Acceptance Criteria

1. One-shot affected feedback completes in at most 15 seconds at p95 over the
   defined 36-attempt matrix, the budget artifact is `RATIFIED`, and the
   `extractum`, internal `extractum-telegram`, Telegram producer/consumer, and
   `extractum-analysis` probes each pass all three attempts within 15 seconds.
   An ordinary deadline miss is retained under the p95 formula; the separate
   60-second hard safety timeout is an infrastructure failure.
2. The Slice 1 startup proxies and Slice 2 per-project new-process startup
   floors are reported separately; every post-partition probe collects exactly
   one real test, retains all twenty scheduled attempts, and cannot use
   `passWithNoTests` as evidence.
3. Warm watcher reruns complete in at most 5 seconds at p95 over the frozen
   seven-TypeScript, seven-Svelte, and six-architecture edit matrix; all probe
   IDs, byte-exact mutations, expected selections, and order are retained, and
   its budget artifact is `RATIFIED`.
4. An unknown changed file returns `MAPPING_ERROR` in less than 2 seconds with
   a concrete suggested command.
5. A known prose-only or asset-only change returns an explicit
   `NO_APPLICABLE_TESTS` result with its allowlist reason.
6. A clean comparison returns `NO_CHANGES`, while a directly changed slow test
   returns non-zero `DEFERRED_ONLY` with its exact owner command.
7. A mixed fast-plus-deferred change set runs the complete eligible fast
   subset and cannot return `PASS`: failure, timeout, infrastructure error, or
   interruption keeps its defined precedence; otherwise it returns
   `DEFERRED_ONLY` with completed fast evidence and every owner command.
8. Checkpoint 4A has no unknown or expired product-code mapping: every
   slow-only production path has valid transitional debt and returns
   `DEFERRED_ONLY`. Rust debt is limited to the baselined root and mixed
   producer/consumer cohorts, while internal Telegram and analysis probes are
   already `RATIFIED`; final checkpoint 4C has no production-source debt entry.
9. A mapped broad or multi-owner selection outside the ratified envelope
   returns `DEFERRED_ONLY` without starting a partial suite and prints every
   required owner command.
10. No product-code change can pass with an empty fast test selection.
11. The complete local gate has a measured critical path, complete inventory,
    and exact execution-graph fingerprint. It is `RATIFIED` only when
    `max(observed_p95, 1.15 * modeled_critical_path_p95) <= 90 seconds` over the
    final inventory; otherwise performance acceptance remains open pending
    measured remediation or an explicit user-approved target revision.
12. Changing the mandatory census, gate command/arguments, prerequisites,
    resource claims/locks, runner configuration, lockfiles, toolchains, or
    reference environment invalidates full-gate ratification to
    `TARGET_NOT_RATIFIED` before the stale artifact can be used.
13. After budget ratification, a single `verify` run over budget prints a
    non-failing warning and records the critical-path overage without changing
    correctness exit precedence.
14. Three consecutive comparable over-budget `verify` runs invalidate
    performance state to `TARGET_NOT_RATIFIED`; only the complete twenty-run
    `verify:performance` benchmark can restore ratification.
15. The full gate covers root frontend, sidecar typecheck/build and unit tests,
    adapter typecheck/unit/E2E, complete Rust workspace checks/tests, Windows
    integration, and the critical Playwright tier.
16. The independent filesystem census assigns every runnable test file to
    exactly one runner; every Vitest file belongs to exactly one named project,
    project intersections are empty, and the transitional `legacy-contract`
    project and all of its command/manifest/config entries have been removed.
17. Selector startup rejects a malformed manifest before runner launch, and
    exhaustive `test:manifest` passes its scheduler-graph, census, ownership,
    Cargo-declaration, registry-disjointness, expiry, and replacement-gating
    checks plus all declared negative and schema-transition fixtures; no
    checkpoint uses mixed producer/consumer schema versions.
18. Every registered Rust mapping resolves against a current non-empty package/
    target/feature inventory; a filtered feedback run lists or consumes an
    exact-fingerprint inventory before executing a non-empty selection.
    Producer feature-off and consumer feature-on proofs run in their registered
    contexts rather than being inferred from `cargo metadata`, and an exact-
    matching workspace execution is reused instead of rerunning the root suite.
19. The source-contract migration ledger accounts for every baseline assertion
    obligation exactly once and resolves every non-deleted replacement ID to
    evidence enabled in `verify`; every relocation has lineage, and final
    completion has no `PENDING` or `DUAL_RUN` row.
20. No test reads arbitrary production source solely for substring or regular-
    expression assertions; allowed fixture/indexer exceptions are explicit.
21. The Chromium suite passes twenty consecutive executions without lifecycle
    timeouts or leaked browser processes.
22. Global retries are zero in every runner and orchestration command.
23. Every skipped or quarantined test has a non-expired owned registry entry.
24. Feedback and full-gate failures produce readable diagnostics and
    machine-readable local timing artifacts.
25. New tooling is locked, compatible with the repository toolchain, and does
    not add a second general-purpose build graph.
26. GitHub Actions, branch protection, and Git hooks remain absent.
27. `AGENTS.md`, `docs/project.md`, and `docs/value-registry.md` describe the
    implemented command and status contracts without contradiction.

## Accepted Risk

Because this design intentionally omits CI and Git hooks, no repository-owned
mechanism prevents a human from merging code without running `verify`.
Documentation and agent workflow rules can make the gate mandatory in process,
but cannot enforce it. This risk is explicit and is not compensated for by
weakening the local gate or by describing focused feedback as completion.

The narrow custom test graph is also a material maintenance risk. A stale
mapping, feature context, duration profile, or resource edge can produce wrong
feedback even when individual runners are correct. The design accepts that
cost because the repository spans Vitest, Playwright, Cargo, Windows process
tests, sidecars, and dynamic filesystem relationships that no current single
runner owns.

The mitigation boundary is strict: `TestingManifest` models only correctness
owners, project/feedback eligibility, gate dependencies, resource claims and
locks, plus the small debt/allowlist/quarantine registries needed to fail
closed. It does not cache build/test outputs, provide remote execution, or
become a general task runner. `test:manifest`, inventory contracts, expiry
metadata, fingerprints, and measured eligibility make stale entries fail
closed. Generic dependency and dead-code rules remain delegated to dependency-
cruiser and Knip instead of growing the custom graph.

## Tooling Basis

- Vitest projects and configuration: <https://vitest.dev/guide/projects>
- Vitest performance guidance: <https://vitest.dev/guide/improving-performance>
- Playwright fixtures: <https://playwright.dev/docs/test-fixtures>
- Playwright trace viewer: <https://playwright.dev/docs/trace-viewer>
- dependency-cruiser rules: <https://github.com/sverweij/dependency-cruiser/blob/main/doc/rules-reference.md>
- Knip configuration: <https://knip.dev/reference/configuration>
- cargo-nextest: <https://nexte.st/>
- sccache: <https://github.com/mozilla/sccache>
