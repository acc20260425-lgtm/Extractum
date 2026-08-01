# Testing Infrastructure Redesign Design

## Status

The design was approved on 2026-08-01. This written specification must be
reviewed by the user before implementation planning begins.

## Executive Summary

Extractum will replace its single mixed test surface with a vertically owned
portfolio, two deliberately different feedback contracts, and one completion
gate:

- focused local feedback that starts a new process and returns in at most
  15 seconds at p95;
- warm watch feedback that returns in at most 5 seconds at p95;
- a complete local `npm.cmd run verify` gate that targets at most 90 seconds
  at p95 with installed dependencies and warm build caches.

The redesign is not a small runner tuning exercise. It changes the test
portfolio, source-contract strategy, browser ownership, changed-file
selection, diagnostics, and full-gate orchestration together. Low-value tests
that assert implementation text will be deleted. Important invariants will
move to behavior tests, compiler-backed architecture rules, package metadata,
or generated declarative contracts.

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

The three largest contract files alone contain about 24,500 lines:

- `src/lib/telegram-crate-boundary-contract.test.ts`, about 10,480 lines;
- `src/lib/analysis-crate-boundary-contract.test.ts`, about 7,272 lines;
- `src/lib/analysis-application-contract.test.ts`, about 6,765 lines.

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
default, so the page
hook runs before the browser hook; the observed timeout does not prove a race.
It does prove that manually owned Chromium teardown inside the mixed Vitest
suite is not currently reliable or sufficiently diagnosable.

The existing `scripts/verify.mjs` runs its stages strictly sequentially. It
does not include sidecar typecheck/build, adapter E2E, or a main-app browser
smoke, while the repository workflow can also repeat the Rust workspace gate
before invoking `verify`.

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
7. Shorten the complete local verification critical path to at most 90
   seconds at p95 on the reference workstation after dependency installation
   and cache warm-up.
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
- Do not adopt Nx, Turborepo, or another repository-wide build graph.
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

Project `include` and `exclude` rules are mutually exclusive. Shared Vite and
Svelte plugin setup comes from one configuration factory, while environment-
specific setup and cleanup belong to the target project. A mixed file that
combines component behavior with raw architecture assertions is split before
the project partition is enabled.

An inventory contract enumerates every Vitest test file and proves that it is
owned by exactly one project. The union of project inventories must equal the
root Vitest inventory, with no duplicate or missing path.

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
Such movement must remain consistent with the separately approved crate
roadmap; a path with no approved boundary requires an explicit architecture
design rather than an SLA exemption.

## Command Contract

All documented Windows commands use `npm.cmd`.

| Command | Contract |
| --- | --- |
| `npm.cmd run test:feedback` | Canonical one-shot affected-test feedback for the working tree or an explicit base; p95 budget 15 seconds |
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
| `npm.cmd run test` | All Vitest-owned projects; not the complete repository gate |
| `npm.cmd run verify` | Authoritative complete local correctness gate |
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

`npm.cmd run test` is a convenience aggregate that runs all four Vitest
projects once. `verify` does not call that aggregate: it invokes the first
three projects in its frontend group and the `os-integration` project in its
exclusive group, so no Vitest file executes twice.

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
gate ownership, path mapping, architecture rule descriptors, and no-test
allowlists. Architecture rule modules export their ID and input descriptor for
the manifest to import; the same paths are not copied into a second config.

The manifest-backed selector combines four mapping sources:

1. static import relationships for ordinary TypeScript and Svelte tests;
2. explicit `TestingManifest` entries for dynamic, generated, Rust, config,
   script, and filesystem relationships;
3. declared `inputs` from architecture rules;
4. Cargo package and test-module ownership for `.rs` changes.

Changes to test configuration, lockfiles, the selector, architecture tooling,
or shared setup expand to the complete affected fast project. A changed test
file selects itself. Rust mapping names the owning package and the narrowest
non-empty test module or exact test known to cover the change.

`TestingManifest` also contains the small allowlist of paths that genuinely
need no tests, such as prose-only documentation and non-executable assets.
Each allowlist entry has a reason. A broad catch-all glob is forbidden.

### Selection flow

For every invocation the selector:

1. discovers and normalizes changed paths;
2. classifies every path;
3. resolves fast tests and known deferred slow gates;
4. rejects the invocation if any path is unknown;
5. rejects a product-code change that has only a slow gate and no fast
   assurance seam;
6. returns `DEFERRED_ONLY` when the invocation selects no fast tests and all
   changed paths are known Playwright, OS-integration, or other slow tests with
   explicit owner commands;
7. runs the selected fast tests under the feedback budget;
8. reports deferred gates without claiming completion;
9. writes the result and timings to a gitignored local artifact.

A successful `PASS` may list known deferred gates, but its scope is always
labelled `feedback-only`. Only `verify` can report repository completion.

### Result statuses and exit codes

| Status | Exit | Meaning |
| --- | ---: | --- |
| `PASS` | 0 | A non-empty mapped fast set passed; deferred completion gates may remain |
| `NO_APPLICABLE_TESTS` | 0 | Every input is explicitly allowlisted, and every reason is printed |
| `NO_CHANGES` | 0 | The requested working-tree/base comparison is empty; no completion claim is made |
| `TEST_FAILED` | 1 | At least one selected assertion failed |
| `MAPPING_ERROR` | 2 | A path is unknown, or product code has no fast assurance seam |
| `INFRA_ERROR` | 3 | Git discovery, spawning, parsing, artifact writing, or cleanup failed |
| `BUDGET_EXCEEDED` | 4 | The feedback wall-clock budget expired |
| `DEFERRED_ONLY` | 5 | A changed slow test has a known owner command but cannot produce fast feedback |

The JSON result contains the status, normalized inputs, selected tests,
deferred gates, durations, tool versions, and suggested reproduction command.
These status values are owned by the testing orchestrator, are ephemeral, are
not persisted in application data, and are not exposed in the product UI.
Their ownership must be registered in `docs/value-registry.md`.

An empty Vitest or Cargo selection can never be translated to `PASS`. When an
exact Cargo test name is not known, the implementation lists tests first and
then runs a verified non-empty filter.

## Source-Contract Replacement

### Assertion disposition

Every current assertion that reads production source text receives exactly
one disposition:

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

The audit first freezes a canonical migration ledger. Every old assertion has
a stable key consisting of repository path plus full nested suite/test title
(and an ordinal only when titles collide). The ledger maps that key to the
owner package, disposition, replacement test/rule/tool ID, or deletion
rationale. A validation command proves that every baseline key appears exactly
once and that every non-deleted replacement ID resolves. The completed ledger
remains as verification evidence after migration.

### Generic tool ownership

Use `dependency-cruiser` for generic TypeScript/Svelte import boundaries,
forbidden dependency directions, and cycles. It uses the project's installed
TypeScript and Svelte compilers rather than bundling another transpiler.

Use Knip for unused files, exports, and dependencies. Knip runs in the full
static gate, not the 15-second affected loop. Entry points and workspaces must
be explicit so SvelteKit, scripts, sidecars, and research adapters are not
mistaken for dead code.

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
- resource class (`light`, `cpu-heavy`, `browser`, or `os-exclusive`);
- platform constraints;
- timeout budget;
- expected artifact locations.

Both `test:feedback` and `verify` consume this ownership data. Documentation
lists the public commands but does not duplicate executable path mappings.

### Full graph

`npm.cmd run verify` covers these groups:

| Group | Required contents |
| --- | --- |
| `static` | `svelte-check`, sidecar and adapter typechecks, sidecar build, dependency-cruiser, Knip, `git diff --check` for the local worktree, and generated/lockfile drift checks owned by changed build steps |
| `frontend` | Complete `unit-node`, `component`, and `architecture` projects, including owned sidecar and adapter unit tests |
| `rust` | Rustfmt plus workspace `cargo check --all-targets` and `cargo test --all-targets` |
| `os-integration` | Vitest- and script-owned real Windows process, filesystem, socket, and SQLite tests |
| `e2e` | Critical Playwright suite after its required app or sidecar readiness gates |

Rust tests that use real OS resources belong semantically to the
`os-integration` tier but execute exactly once under their Cargo owner unless
they are moved to a distinct Cargo integration target. The registry must not
schedule the same Rust test again in the separate `os-integration` group.

Independent groups may run concurrently, but at most two heavy groups run at
once. Browser and OS-exclusive work do not compete with another exclusive
group. Every Cargo invocation also holds one `cargo-exclusive` resource lock
so concurrent groups do not contend over canonical `src-tauri/target`. The
scheduler lets already-running independent gates finish after a failure, does
not start failed dependents, and prints an aggregate result.

The scheduler must terminate child process trees on timeout or interruption,
including Windows descendants. A timeout is an infrastructure failure for the
full gate and a `BUDGET_EXCEEDED` result for feedback.

Local reports live under one gitignored artifact root and include group/test
durations and failure diagnostics. They are not committed.

### Full-gate result contract

`verify` exits with code 0 only when every required gate completes. A static,
test, or behavior failure produces exit 1. If no correctness check failed, a
timeout, unsupported host, required capability skip, spawn failure, cleanup
failure, or artifact failure produces exit 3. User interruption produces exit
130 after terminating owned descendants.

When correctness and infrastructure failures occur together, exit 1 remains
the primary result and every infrastructure problem is retained under
`secondaryFailures`; cleanup or artifact failure cannot erase the original
assertion failure. A dependent gate that was not started records its causal
`blockedBy` gate and does not become a new root failure.

The required Windows integration tier cannot be skipped on a non-Windows host
or because the host lacks a required permission. Such a run is incomplete and
returns exit 3 with a capability diagnostic. Only an explicitly optional
diagnostic outside `verify` may report a successful capability skip.

### Avoiding duplicated Rust completion work

Focused Rust commands remain RED/GREEN accelerators. The project workflow must
not require a complete workspace test immediately before `verify`, because
`verify` already owns that completion evidence. A Rust slice runs narrow
RED/GREEN tests and package checkpoints while developing, then runs the single
end-of-slice `verify` gate.

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
| Nx/Turborepo | Do not adopt | The repository does not need another build graph for one frontend plus Cargo workspace |
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

### One-shot feedback

One-shot timing starts when `test:feedback` starts its new Node process and
ends when the final human and JSON result are complete. It includes Git
discovery, mapping, runner startup, selected tests, and reporting. It does not
use a persistent watcher or daemon. Normal disk and build caches are reused.
The 15-second budget is end-to-end, including cancellation, descendant cleanup,
and final reporting; the runner reserves time for those stages before its test
execution deadline.

The acceptance matrix contains ten predeclared reversible probes:

- two pure TypeScript behavior inputs;
- two Svelte component inputs;
- two structured architecture inputs;
- one selector/configuration input;
- one compile-invalidating `extractum` Rust input;
- one compile-invalidating `extractum-telegram` Rust input;
- one compile-invalidating `extractum-analysis` Rust input.

Each probe runs three times in alternating order for 30 total attempts. Rust
probes start with warm dependency caches but modify an owning source file so
the package compilation/test target is genuinely invalidated; every temporary
edit is restored byte-for-byte. The report publishes median, nearest-rank p95,
and maximum.

Every scheduled attempt remains in the report. Assertion failure,
infrastructure failure, or timeout fails acceptance and cannot be replaced by
an extra green attempt. Timing for a timeout is recorded at the observed
termination time. The performance distribution is never computed from a
hand-picked successful subset.

### Warm watcher

Watcher timing starts at the file-change event and ends when the affected
result is printed. The acceptance sample contains twenty sequential edits
covering the representative TypeScript, Svelte, and architecture categories.
All planned attempts remain in the report. Initialization time is reported
separately and is not part of the 5-second rerun budget.

### Full local gate

The full-gate sample uses installed dependencies and warm build caches and
contains twenty consecutive planned runs of an unchanged inventory. It
publishes median, nearest-rank p95, and maximum. Any correctness or
infrastructure failure fails reliability acceptance and remains in the
evidence; it is not replaced. A cold build is reported separately and cannot
be averaged into the warm target.

Machine-specific durations remain verification evidence; they are not encoded
as millisecond assertions inside ordinary unit tests. The orchestrator itself
does enforce the 15-second feedback budget because that is a productized
developer-workflow contract.

## Failure Handling

- No failing measurement is silently rerun and substituted into an aggregate.
- A changed test inventory invalidates a before/after comparison.
- A parser or metadata error stops the affected architecture gate.
- A selector timeout reports the slowest completed work and a reproduction
  command before returning `BUDGET_EXCEEDED`.
- A child-spawn or process-cleanup failure returns `INFRA_ERROR` and identifies
  any descendant process that could not be terminated.
- A missing Playwright browser reports a bootstrap instruction; it is not
  presented as a behavior failure.
- A Cargo filter that executes zero tests is invalid evidence.
- A deferred slow gate is always named in feedback output and can only be
  discharged by its explicit command or full `verify`.
- Local artifact-write failure is an infrastructure failure because the
  machine-readable result is part of the command contract.

## Migration Sequence

### Slice 1: Registry and observability

Introduce the gate registry, result schema, timing artifacts, baseline report,
and aggregate local runner without changing test ownership. Preserve the
current `verify` coverage while establishing before/after evidence.

### Slice 2: Project and browser ownership

Split Vitest projects, establish project-specific worker/resource policies,
move Chromium extraction to Playwright, and remove competing component
cleanup ownership. Keep behavior assertions equivalent.

### Slice 3: Source-contract replacement

Migrate the three largest Telegram and analysis contracts first, then the
remaining cohort. Add dependency-cruiser, Knip, the repository index, and the
guardrail against new arbitrary source-text tests.

This is an epic composed of bounded atomic sub-slices: Telegram, analysis
crate boundary, analysis application, and then the remaining contract
clusters. Each sub-slice records assertion dispositions, removes its replaced
old path, runs the applicable package checkpoints, and ends with `verify`.

### Slice 4: Fail-closed feedback

Introduce the `TestingManifest` path mappings, one-shot selector, result
statuses, budget, watch invalidation, Rust/package mapping, and compatibility
aliases. Do not
enable fail-closed behavior until the known production paths have explicit
mappings or intentional allowlist entries.

### Slice 5: Documentation and workflow policy

Update `AGENTS.md`, `docs/project.md`, and `docs/value-registry.md`; remove the
duplicated Rust completion flow; document coverage, stability, quarantine,
and local artifact handling.

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
slices above. Slice 3 is further split into the four named contract clusters.
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
2. run the narrow RED test with
   `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> -- --exact`;
3. implement or move the behavior seam;
4. run the same non-empty exact GREEN test;
5. run
   `cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
6. checkpoint the package with
   `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
7. checkpoint an immediate dependent package when a public cross-crate
   interface changes;
8. end the Rust-changing slice with `npm.cmd run verify`.

All commands reuse canonical `src-tauri/target`. The plan must not introduce
slice-specific `codex-*` targets. An exact or filtered run that executes zero
tests is not verification.

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
- `docs/value-registry.md` owns the ephemeral selector status vocabulary and
  records that it has no persistence, API, or UI impact.
- `TestingManifest` owns executable input mappings, rule descriptors,
  allowlists, gates, and resource policies.
- Package scripts expose stable public entry points and must not duplicate
  mapping logic.
- The benchmark report owns environment-specific timing evidence.

## Acceptance Criteria

1. One-shot affected feedback completes in at most 15 seconds at p95 over the
   defined 30-attempt matrix, including compile-invalidating `extractum`,
   `extractum-telegram`, and `extractum-analysis` probes.
2. Warm watcher reruns complete in at most 5 seconds at p95 over twenty
   representative edits.
3. An unknown changed file returns `MAPPING_ERROR` in less than 2 seconds with
   a concrete suggested command.
4. A known prose-only or asset-only change returns an explicit
   `NO_APPLICABLE_TESTS` result with its allowlist reason.
5. A clean comparison returns `NO_CHANGES`, while a directly changed slow test
   returns non-zero `DEFERRED_ONLY` with its exact owner command.
6. No product-code change can pass with an empty fast test selection.
7. Full local `npm.cmd run verify` completes in at most 90 seconds at p95 with
   installed dependencies, warm build caches, and an unchanged inventory.
8. The full gate covers root frontend, sidecar typecheck/build and unit tests,
   adapter typecheck/unit/E2E, complete Rust workspace checks/tests, Windows
   integration, and the critical Playwright tier.
9. Every Vitest file belongs to exactly one project, and the union of project
   inventories equals the root inventory without duplicates or omissions.
10. The source-contract migration ledger accounts for every baseline assertion
    exactly once and resolves every non-deleted replacement ID.
11. No test reads arbitrary production source solely for substring or regular-
   expression assertions; allowed fixture/indexer exceptions are explicit.
12. The Chromium suite passes twenty consecutive executions without lifecycle
   timeouts or leaked browser processes.
13. Global retries are zero in every runner and orchestration command.
14. Every skipped or quarantined test has a non-expired owned registry entry.
15. Feedback and full-gate failures produce readable diagnostics and
    machine-readable local timing artifacts.
16. New tooling is locked, compatible with the repository toolchain, and does
    not add a second general-purpose build graph.
17. GitHub Actions, branch protection, and Git hooks remain absent.
18. `AGENTS.md`, `docs/project.md`, and `docs/value-registry.md` describe the
    implemented command and status contracts without contradiction.

## Accepted Risk

Because this design intentionally omits CI and Git hooks, no repository-owned
mechanism prevents a human from merging code without running `verify`.
Documentation and agent workflow rules can make the gate mandatory in process,
but cannot enforce it. This risk is explicit and is not compensated for by
weakening the local gate or by describing focused feedback as completion.

## Tooling Basis

- Vitest projects and configuration: <https://vitest.dev/guide/projects>
- Vitest performance guidance: <https://vitest.dev/guide/improving-performance>
- Playwright fixtures: <https://playwright.dev/docs/test-fixtures>
- Playwright trace viewer: <https://playwright.dev/docs/trace-viewer>
- dependency-cruiser rules: <https://github.com/sverweij/dependency-cruiser/blob/main/doc/rules-reference.md>
- Knip configuration: <https://knip.dev/reference/configuration>
- cargo-nextest: <https://nexte.st/>
- sccache: <https://github.com/mozilla/sccache>
