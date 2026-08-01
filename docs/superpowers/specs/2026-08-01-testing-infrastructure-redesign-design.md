# Testing Infrastructure Redesign Design

## Status

The design was approved on 2026-08-01, revised after written review, and then
simplified by user decision on 2026-08-02. This revision must be reviewed by
the user before implementation planning begins.

## Executive Summary

Extractum will replace its single mixed test surface with vertically owned test
tiers, a fail-closed changed-file selector, and one complete local verification
gate.

The central simplification is that test selection and timing are independent:

- TestingManifest maps a path to one owner command and classifies that command
  as fast or slow.
- Selection uses only that mapping. Historical timings never make a test
  eligible, ineligible, valid, stale, or ratified.
- Every orchestrated command appends one timing row containing only command,
  duration, exitCode, and commit.
- test:feedback has a hard end-to-end limit of 15 seconds.
- test:watch prints a warning when a rerun takes more than 5 seconds.
- verify prints a warning when the complete gate takes more than 90 seconds,
  but elapsed time never changes its correctness result.
- verify:performance runs verify five times and prints the median, maximum, and
  the five slowest gates.
- A separate 20-run p95 audit is permitted only at a major program checkpoint.
  It is report-only and creates no state used by daily commands.

The redesign remains intentionally broad. It changes the test portfolio,
source-contract strategy, browser ownership, changed-file selection,
diagnostics, and complete-gate orchestration. Low-value tests that assert
implementation text will be deleted. Valuable invariants will move to behavior
tests, compiler-backed architecture rules, package metadata, or generated
declarative contracts.

GitHub Actions, branch protection, Git hooks, and other server- or
Git-enforced gates remain outside this design. The authoritative completion
gate is local.

## Context and Evidence

The original 2026-08-01 timing snapshot was captured on commit 1114e626.
Telegram Phase 8C landed afterward, so Slice 1 must rebaseline current
inventory before making implementation decisions.

| Surface | Existing evidence | Consequence |
| --- | --- | --- |
| Root Vitest | 177 files and 1,632 tests; about 199.96 seconds in the measured full run | Too slow for RED/GREEN feedback |
| Chromium teardown | Assertions passed, but the full Vitest run failed in an afterAll timeout | Browser lifecycle needs a single owner |
| Svelte check | About 40.69 seconds | It cannot be part of the 15-second loop |
| Cargo workspace | 1,233 tests in about 29.90 seconds | Too broad for every Rust edit |
| Historical root Rust check | About 39.71 seconds after compile invalidation | The cost must be decomposed before promising fast root feedback |
| Recent full verify | 172.1, 202.7, and 175.5 seconds | The 90-second value is currently an objective, not a proven result |
| Fast Node behavior cohort | 81 files and 723 tests; about 2.8 seconds of summed test execution | Most assertions are not inherently expensive |
| Source-contract cohort | About 70 files and 695 tests; about 112 seconds of summed file time | Repository parsing and textual assertions dominate the long tail |

The historical 39.71-second Rust observation is frequently easy to
misinterpret. It was a compile-invalidating Cargo check whose root extractum
library unit took about 38.26 seconds. It was not an end-to-end test run and
contains no isolated link or test-execution measurement. Slice 1 therefore
measures the missing phases instead of treating 39.71 seconds as a test
duration.

At the earlier snapshot, the three largest contract files contained about
24,500 lines:

- src/lib/telegram-crate-boundary-contract.test.ts;
- src/lib/analysis-crate-boundary-contract.test.ts;
- src/lib/analysis-application-contract.test.ts.

They repeatedly read production sources and contain custom parsing, frozen
authority data, and implementation-shaped assertions. Vitest's static related
graph cannot see many of those filesystem relationships. The current changed
and related commands can therefore exit successfully while selecting no useful
tests.

The root suite also mixes pure Node tests, jsdom component tests, repository
architecture checks, real Windows processes, and Chromium.
sidecars/gemini-browser/src/answer-extractor.test.ts currently launches
Chromium from Vitest and owns manual teardown. The observed timeout does not
prove a particular hook race, but it does prove that ownership and diagnostics
are inadequate.

The current scripts/verify.mjs is sequential and omits some sidecar, adapter,
and browser surfaces. The repository workflow can also repeat the Rust
workspace gate immediately before verify.

## Timing Principles

Timing is operational feedback, not correctness metadata.

1. A path's fast or slow class is an explicit owner decision in
   TestingManifest.
2. No benchmark result changes that class automatically.
3. No configuration change invalidates timing eligibility because timing
   eligibility does not exist.
4. No rolling counter, benchmark fingerprint, modeled critical path, CPU
   reservation, memory reservation, or performance-state vocabulary is
   maintained.
5. A missed timing target is visible as a timeout or warning. It does not
   rewrite a correctness exit code.
6. A human may change a manifest entry from fast to slow, or back, through a
   normal reviewed source change.

The thresholds have deliberately different semantics:

| Surface | Threshold | Effect |
| --- | ---: | --- |
| test:feedback | 15 seconds end to end | Hard stop and BUDGET_EXCEEDED |
| test:watch | 5 seconds per warm rerun | Warning only |
| verify | 90 seconds wall time | Warning only |
| verify:performance | Five complete runs | Report median, maximum, and five slowest gates |
| Major-checkpoint p95 audit | Twenty complete runs | Report-only nearest-rank p95 |

The 90-second value remains the desired complete-gate outcome. Until the
portfolio is optimized, verify stays authoritative even when it prints the
warning.

## Goals

1. Return useful affected-test feedback from a new process within a hard
   15-second limit.
2. Return useful warm watcher feedback and warn after 5 seconds.
3. Fail closed when a changed executable path has no known test owner.
4. Separate pure, component, architecture, OS integration, and browser tests
   so each has an explicit owner.
5. Eliminate arbitrary production-source includes and regular-expression
   assertions while preserving valuable behavior and architecture invariants.
6. Make Chromium exclusively Playwright-owned.
7. Cover root frontend, sidecar, adapter, Rust workspace, Windows integration,
   and critical browser surfaces in verify.
8. Use a fixed scheduler that is easy to understand and debug.
9. Keep timing data minimal and independent of correctness.
10. Keep commands and ownership rules understandable without a general-purpose
    monorepo build system.

## Non-Goals

- Do not add GitHub Actions, branch protection, required PR checks, nightly
  jobs, pre-commit hooks, pre-push hooks, Husky, or Lefthook.
- Do not treat focused feedback as completion evidence.
- Do not introduce global retries or retry-until-green scripts.
- Do not maintain performance ratification, benchmark eligibility, timing
  fingerprints, rolling overage counters, or automatic timing invalidation.
- Do not predict duration from a resource-constrained critical-path model.
- Do not encode numeric CPU or memory claims in test orchestration.
- Do not require startup microbenchmarks for every Vitest project.
- Do not adopt Nx, Turborepo, another general-purpose build graph, remote
  execution, or build-artifact caching.
- Do not introduce Testcontainers or Docker for the SQLite-owned backend.
- Do not add Mockall where existing narrow Rust traits and handwritten fakes
  already provide adequate seams.
- Do not use one global coverage percentage as a quality target.
- Do not make full Tauri packaging or release builds part of verify.
- Do not automatically delete Cargo, npm, Playwright, or test artifact caches.
- Do not pull experimental Python research suites into the root product gate.
- Do not change application behavior solely to satisfy an
  implementation-shaped test.

## Chosen Architectural Approach

The project will rebuild the portfolio vertically: runner boundaries,
expensive source contracts, browser ownership, selection, and orchestration
move in coordinated slices.

Changing the runner first would optimize around source contracts that are
scheduled for removal. Rewriting contracts first would leave false-green
selection and mixed lifecycle ownership in place for too long. The ordered
slices below keep every checkpoint locally verifiable without building a
second general-purpose task graph.

## Target Test Portfolio

The portfolio has five semantic tiers:

| Tier | Owner and contents | Execution policy |
| --- | --- | --- |
| unit-node | Pure TypeScript logic, API wrappers, workflow decisions, parsers, and selectors | Vitest Node environment, isolation on, no DOM or real OS resources |
| component | Svelte rendering and interaction through Testing Library | Dedicated Vitest project with jsdom and one cleanup owner |
| architecture | Import boundaries, cycles, metadata, and domain-specific structured rules | Structured repository index; no arbitrary production-source text assertions |
| os-integration | Real Windows processes, filesystem behavior, file-backed SQLite, sockets, and equivalent Rust integration behavior | Dedicated processes or forks, constrained workers, explicit cleanup |
| e2e | Browser-owned user scenarios, the Gemini adapter, and Chromium-backed extraction | Playwright owns browser, context, page, traces, and artifacts |

A tier describes semantics. It does not need to be one process or one
TestingManifest entry.

### Vitest projects and independent census

Vitest configuration exposes unit-node, component, architecture, and
os-integration projects. Playwright tests are never included in a Vitest
project.

Every project has a stable unique name and inherits only the lean shared
Vite/Vitest base. Environment, setup, cleanup, pool, worker, and timeout policy
belong to the target project. Svelte Testing Library and DOM setup belong only
to component. Mixed files are split before project ownership is enabled.

Project classification is owned by Vitest and Playwright configuration, not by
TestingManifest.

The census compares three independently produced sets:

1. filesystem candidates discovered from all supported test and spec suffixes;
2. files actually collected by Vitest and Playwright;
3. exact, reasoned exceptions.

A collected runner file must be a filesystem candidate or an exact
nonstandardTest exception. Every filesystem candidate must be collected by
exactly one runner or be an exact non-runnable fixtureException. Exceptions
contain a path, reason, and owner; broad catch-all patterns are prohibited.
Project intersections must be empty.

Every project command names its project explicitly and rejects an empty
collection. passWithNoTests cannot turn an empty project into evidence.

During source-contract migration, a transitional legacy-contract Vitest
project owns remaining arbitrary-source readers. It stays outside feedback and
watch mode but remains in test and verify. Slice 2A freezes obligations, not
physical files. Slice 2B may mechanically split mixed files while preserving
stable IDs and path lineage; after those splits it freezes the legacy file
inventory and no new obligation or legacy file may enter. The project is
removed atomically when its inventory reaches zero.

Isolation remains enabled. Disabling it for a bounded project requires a
separate state-ownership review and a small measured experiment.

### Rust ownership

Rust behavior stays in its owning Cargo package. A Node test must not parse
.rs implementation text to prove Rust behavior. Cross-crate structure uses
cargo metadata, compiler checks, public-interface tests, or narrow tests in the
owner package.

Current Telegram behavior is owned by extractum-telegram; extractum is its
immediate application consumer and enables app-test-support only through a dev
dependency. Analysis behavior remains with extractum-analysis or extractum
according to the current public boundary.

The design does not assume every Rust path can be fast. Slice 1 first determines
where the current cost lies. If a root path cannot fit the 15-second limit
without an unjustified architecture change, it is explicitly mapped to a slow
owner and test:feedback returns DEFERRED_ONLY. A smaller package or test-target
boundary is introduced only when the diagnostic shows a bounded, valuable
path to fast feedback.

## Command Contract

All documented Windows commands use npm.cmd.

| Command | Contract |
| --- | --- |
| npm.cmd run test:feedback | Canonical one-shot affected-test feedback; hard 15-second end-to-end limit |
| npm.cmd run test:changed | Compatibility alias for working-tree test:feedback |
| npm.cmd run test:changed:last | Compatibility alias using the most recent linear checkpoint |
| npm.cmd run test:related -- paths | Explicit-path entry into the same selector |
| npm.cmd run test:watch | Selector-aware warm watcher; warning after a 5-second rerun |
| npm.cmd run check:watch | Separate background Svelte/TypeScript checking |
| npm.cmd run test:unit | Complete unit-node project |
| npm.cmd run test:component | Complete component project |
| npm.cmd run test:architecture | Complete structured architecture project |
| npm.cmd run test:integration:os | Complete OS integration tier |
| npm.cmd run test:e2e | Critical Playwright suite |
| npm.cmd run test:manifest | Validate path ownership, command references, speed values, census, and runner ownership |
| npm.cmd run test | All Vitest-owned projects; not repository completion |
| npm.cmd run verify | Authoritative complete local correctness gate; warning after 90 seconds |
| npm.cmd run verify:performance | Run verify five times and print timing aggregates |
| npm.cmd run verify:performance -- --p95-audit | Major-checkpoint-only 20-run report |
| npm.cmd run verify:coverage | Explicit V8 coverage analysis for declared critical modules |
| npm.cmd run verify:stability | Explicit repeated-run flake investigation |

Existing narrow package scripts remain only when they are direct owner
commands. Redundant orchestration scripts are removed. AGENTS.md and
docs/project.md must describe the same public workflow.

The warm watcher covers TypeScript, Svelte, and the cached repository index.
This design does not introduce a persistent Cargo watcher.

npm.cmd run test executes every Vitest project once. verify invokes those
projects as individual gates, so no Vitest file is executed twice.

## Simple TestingManifest

TestingManifest is a repository-owned path routing table. Each entry contains
exactly:

- path: an exact repository-relative path or a narrow non-overlapping pattern;
- command: one stable owner command;
- speed: fast or slow.

There are no gate IDs, prerequisites, project definitions, resource claims,
locks, durations, benchmark references, evidence states, expiry dates,
fingerprints, or migration records in TestingManifest.

A path has exactly one owner command. When several checks are required, that
owner is a small stable aggregate command. Overlapping path patterns are a lint
error rather than a precedence feature.

The speed value means:

- fast: test:feedback may execute the command under its remaining 15-second
  deadline;
- slow: test:feedback never starts the command and prints it as deferred work.

Speed is a reviewed policy choice. The timing log does not update it.

A separate small NoTestAllowlist contains only a path and reason for prose or
inert assets that genuinely need no test command. It may use an exact path or
a narrow non-executable-only directory pattern; broad repository catch-alls
are prohibited. Its validator rejects any match that becomes executable. The
allowlist is separate because it is a correctness classification, not a
performance classification.

Quarantine metadata also remains separate. It must not expand the manifest
schema or turn an unknown product path into an allowed path.

### Manifest validation

test:manifest checks:

- every manifest entry has only path, command, and speed;
- current exact paths exist as tracked or untracked repository files, patterns
  match current repository candidates, all mappings are normalized and
  non-overlapping, and no pattern is a broad catch-all;
- every command is fully invokable as written or resolves to a stable package
  script; the selector never derives Cargo package, target, features, or test
  names from the path;
- speed is exactly fast or slow;
- every discovered tracked or untracked executable source path has one owner
  or an exact no-test reason;
- every changed test path maps to an owner that includes that test;
- filesystem census and actual runner collection agree;
- no-test entries are narrow and reasoned, quarantine entries are exact, and
  both are disjoint from executable ownership.

The validator uses fixtures for an untracked addition, exact mapped deletion,
deleted no-test path, and both sides of a rename, plus negative fixtures for an
unknown path, overlap, deleted command, invalid speed, empty project, duplicate
runner ownership, and broad allowlist.

Cargo package, target, and feature names are checked with cargo metadata. A
filtered Cargo owner lists tests before execution when its exact test identity
is not already known. A zero-test Cargo run is invalid evidence.

## Fail-Closed Affected-Test Selector

### Inputs

The selector accepts:

- staged, unstaged, and untracked working-tree paths;
- an explicit Git base through --base;
- the HEAD~1 base used by test:changed:last;
- one or more explicit repository paths.

It normalizes Windows and POSIX separators and rejects paths outside the
repository. Base mode evaluates base...HEAD and then adds current staged,
unstaged, and untracked paths. Deleted paths map by their former repository
path. A rename is one deletion plus one addition, and both sides must map.

Current and added paths use the current TestingManifest and
NoTestAllowlist. A deleted path, or the old side of a rename, first uses the
current routing files and then their versions from the selected Git base when
the current entry was removed with the file. Working-tree mode uses HEAD as
that base. If the base has no readable compatible routing files or neither
revision maps or allowlists the old path, selection returns MAPPING_ERROR. The
base fallback changes only path routing; it imports no timing data or
performance state.

A clean range returns NO_CHANGES. A missing explicit path, unreadable Git
state, or empty explicit-path invocation is not converted to successful test
evidence.

### Selection flow

For every invocation the selector:

1. validates the small manifest and no-test allowlist;
2. discovers and normalizes all inputs;
3. classifies every input before starting any owner command;
4. fails with MAPPING_ERROR if any path is unknown;
5. deduplicates fast and slow owner commands;
6. runs the complete fast subset sequentially under the single remaining
   end-to-end deadline;
7. never starts a slow owner;
8. reports all deferred owner commands;
9. applies result precedence and appends minimal timing rows.

No historical timing lookup occurs in this flow.

Sequential owner execution is deliberate: the selector needs no hidden
scheduler classification, Cargo is necessarily alone, and ordinary commands
remain below the global maximum of two. Typical changes should resolve to one
owner command; a recurring multi-owner case is handled by a reviewed aggregate
owner rather than dynamic scheduling metadata.

Shared configuration, lockfiles, or other paths that require a broad aggregate
are mapped directly to that aggregate owner and normally classified slow. They
therefore return DEFERRED_ONLY immediately instead of spending 15 seconds on a
predictably incomplete broad run.

If fast and slow paths are mixed, the selector runs the fast subset. It cannot
return PASS while a slow obligation remains: after successful fast work it
returns DEFERRED_ONLY and prints every slow command.

If the combined fast set exceeds the hard limit, the selector cancels its
owned process tree and returns BUDGET_EXCEEDED. The implementation must reserve
enough of the 15-second envelope for cancellation and final output. It does not
use a precomputed selection envelope.

PASS is possible only after a non-empty fast set passed and every executable
input received fast assurance. It is always labelled feedback-only.

### Result statuses and exit codes

| Status | Exit | Meaning |
| --- | ---: | --- |
| PASS | 0 | A non-empty fast owner set passed and no slow obligation remains |
| NO_APPLICABLE_TESTS | 0 | Every input has an exact no-test reason |
| NO_CHANGES | 0 | The requested comparison is empty |
| TEST_FAILED | 1 | A selected assertion or correctness command failed |
| MAPPING_ERROR | 2 | An input is unknown or manifest ownership is invalid |
| INFRA_ERROR | 3 | Git discovery, spawning, parsing, or required cleanup failed |
| BUDGET_EXCEEDED | 4 | test:feedback reached its hard 15-second limit |
| DEFERRED_ONLY | 5 | At least one mapped input has a slow owner; every owner command is printed |
| INTERRUPTED | 130 | The user interrupted the invocation |

The human diagnostic may contain normalized inputs, selected commands,
deferred commands, and reproduction instructions. The persisted timing row is
separate and contains only the four timing fields defined below.

Path mapping errors are resolved before a runner starts. After execution
starts, interruption is primary. Without interruption, an observed correctness
failure is primary over a later deadline or cleanup failure. If no correctness
failure was observed, BUDGET_EXCEEDED is primary over a cleanup warning.
DEFERRED_ONLY and success statuses are considered only after execution errors.

An empty Vitest or Cargo selection can never become PASS.

## Source-Contract Replacement

### Dispositions

Every source-dependent test receives one or more reviewed dispositions:

1. Behavior: replace it with a unit, component, Rust, integration, or E2E test
   through a public seam.
2. Architecture: express it through an import graph, AST, compiler metadata,
   Cargo metadata, or a structured repository rule.
3. Tool-owned: delegate generic unused-code, dependency, or cycle checks to a
   maintained third-party tool.
4. Delete: remove exact-name, statement-order, formatting, substring, and
   similar implementation-detail assertions with no independent value.

Large production-source snapshots are not an acceptable replacement.

### Lightweight migration ledger

The ledger is frozen in Slice 2A immediately before ownership relocation, not
in the measurement slice.

The default row represents one test declaration, not every expect call. It
contains:

- a stable ID that survives file moves;
- current path and full nested test title;
- source hash, assertion count, and parameter or authority-data hash;
- a plain-language invariant and disposition;
- replacement test, rule, or tool IDs, or a specific deletion reason;
- path lineage when the test moves.

Assertion subgroups are created only when one test has mixed dispositions,
different replacement owners, or partial retirement. This keeps simple tests
simple without allowing a large mixed test to hide deleted obligations.

The ledger has no PENDING, DUAL_RUN, or RETIRED vocabulary. Validation derives
whether a row is open or closed. A row closes only when current legacy evidence
is absent and either both its replacement IDs resolve in current runner or tool
inventory and their owner commands are present in verify, or the row has a
specific deletion reason. Merely writing a replacement ID does not close it.
A migration sub-slice may run old and new evidence together, but it closes with
the old evidence removed.

The extractor and validator are timeboxed to two implementation days.
Unsupported syntax produces an explicit manual row instead of expanding a
custom parser indefinitely. The completed ledger remains checked in as review
evidence. During migration, test:manifest invokes the bounded ledger validator
as a separate module; TestingManifest does not absorb ledger fields.

### Generic tool ownership

Use dependency-cruiser for generic TypeScript/Svelte import boundaries,
forbidden directions, and cycles. Use Knip for unused files, exports, and
dependencies. Knip runs in the complete static gate, not the 15-second loop.

Tool-native configuration owns entry points, workspaces, project sets, and
rule semantics. TestingManifest contains only the owner command for paths that
should invoke these checks.

Both tools start with narrow reviewed rules. Existing findings are fixed or
entered into exact, reasoned, expiring baselines. Blanket repository ignores
are prohibited.

A checked-in canary proves that configured analysis resolves Svelte files,
$lib and $app aliases, raw imports, root scripts, the Gemini sidecar, and the
Gemini adapter.

Use @vitest/coverage-v8 for explicit coverage analysis. Its version must match
the installed Vitest major and be locked in package-lock.json.

Do not add ts-morph initially. The installed TypeScript and Svelte compilers
already provide the required ASTs.

### Repository index

Domain-specific structured rules share one RepositoryIndex:

- TypeScript uses the installed TypeScript compiler API.
- Svelte uses svelte/compiler.
- dependency-cruiser owns its generic import graph.
- Cargo structure comes from cargo metadata and compiler evidence.
- Custom-index files are read and parsed once per content version.
- One-shot runs may reuse a disk cache keyed by content and parser/rule
  versions.
- Watch mode keeps the index in memory and invalidates changed relationships.

Each custom rule has a stable ID, declared inputs, and an evaluator. A parse
failure is INFRA_ERROR, never zero violations.

### Migration guardrail

Migrate the three largest Telegram and analysis contracts first, then the
remaining source-contract cohort.

At completion, no test may read arbitrary production source solely to assert
includes, match, or regular expressions. Fixture readers and tests of the
indexer itself remain explicit exceptions. A structured rule prevents new
test code outside approved helpers from reading production roots. The
equivalent Rust policy rejects test-only source scans over production .rs
files outside registered indexer and fixture owners.

## Browser and Component Ownership

The Chromium answer-extractor suite moves from Vitest to Playwright. Playwright
owns browser, context, page, and their cleanup. A custom worker fixture owns
only the server or IPC seam it starts. Artifacts are scoped through testInfo.

The critical E2E tier includes:

- existing Gemini browser adapter scenarios;
- Chromium-backed answer extraction currently launched from Vitest;
- a narrow main-application browser smoke for critical navigation and error
  rendering through an explicit Tauri IPC seam.

The smoke does not reproduce the complete Tauri desktop stack.

The Playwright server fixture reads the URL it actually starts. Tests do not
assume port 5173 or attach to an unrelated server.

Playwright retains trace, screenshot, console, and server logs on failure.
Global retries remain zero. A shared server starts with one worker. More
workers require worker-scoped servers, testInfo-scoped artifacts, and a
measured benefit.

Component tests stay in jsdom and have one Testing Library cleanup owner.
happy-dom is not introduced without a compatibility experiment on the actual
component cohort.

## Complete Local Verification Gate

### Inventory

verify covers:

| Group | Required contents |
| --- | --- |
| static | svelte-check, sidecar and adapter typechecks, sidecar build, test:manifest including census and active ledger validation, dependency-cruiser, Knip, git diff --check, and owned generated/lockfile drift checks |
| frontend | unit-node, component, architecture, transitional legacy-contract, and sidecar/adapter unit tests |
| rust | rustfmt, required producer feature-off and consumer feature-on proofs, workspace cargo check --all-targets, and cargo test --all-targets |
| os-integration | Real Windows process, filesystem, socket, and SQLite tests, including Cargo-owned tests that execute only once |
| e2e | Critical Playwright suite with its own server readiness fixture |

The command list is explicit in scripts/verify.mjs. It is not a generic gate
registry and is not stored in TestingManifest. The script has three plain
command arrays: general, Cargo, and browser-or-OS. There is no scheduler-class
field in a shared schema.

### Fixed scheduler

The scheduler has three rules:

1. At most two non-Cargo commands run at a time.
2. Cargo commands run one at a time and run alone; no other command overlaps a
   Cargo command.
3. At most one browser or OS-integration command occupies the two available
   non-Cargo positions, so browser and OS work never overlap one another.

The implementation may arrange these as simple phases. It has no numeric
resource model, dynamic locks, or critical-path prediction. Server startup and
readiness needed by E2E belong inside the E2E owner command instead of becoming
a general dependency graph.

Except on user interruption, verify schedules every command in the three
arrays even after an earlier command fails, then aggregates all diagnostics.
This keeps inventory and per-gate timing comparable and removes dependency or
blockedBy semantics. A required gate that cannot start because of host
capability makes verify incomplete, but the remaining independent commands
still run. The scheduler terminates owned Windows process trees on a gate
timeout or interruption and prints the causal failure.

### Result and timing warning

verify exits 0 only when every required correctness gate completes
successfully. Correctness failure exits 1, infrastructure incompleteness exits
3, and user interruption exits 130. A command that cannot start reports its
own spawn or capability diagnostic.

Interruption is always the primary result; failures already observed remain
visible as secondary diagnostics. Without interruption, any correctness
failure is primary exit 1 and infrastructure failures remain secondary. If no
correctness command failed, any infrastructure failure is primary exit 3.
Timing warnings never enter this precedence.

Elapsed time does not participate in this result. If verify is still running at
90 seconds, a timer prints a prominent warning and the currently slowest
active or completed gates. Completion prints the final wall time and preserves
the correctness exit code.

Focused Rust commands remain development accelerators. A Rust-changing slice
runs narrow tests and package checkpoints during development, then one verify
at the end. The workflow must not require a complete workspace test
immediately before verify.

An exact matching workspace Cargo execution may satisfy a producer/consumer
proof already contained in that execution. The root suite is not rerun merely
to duplicate evidence.

## Minimal Timing System

### Timing row

Every logical scheduler or selector entry produces one JSON Lines row under a
gitignored local artifact directory, and every public aggregate produces one
row for itself. This includes raw internal entries such as Knip,
dependency-cruiser, and Cargo. A top-level command records itself. An aggregate
records each direct child after that child exits or is terminated and
suppresses the child's top-level recorder, so a forced termination still has a
row and no invocation is duplicated. Helper processes inside one owner command
do not receive separate rows. The row has exactly four fields:

| Field | Meaning |
| --- | --- |
| command | Complete normalized command string |
| duration | Wall duration in integer milliseconds |
| exitCode | Normalized integer command result |
| commit | Full HEAD commit hash at command start |

Each test:watch rerun is treated as a public-command observation even though
the watcher process remains alive. Append order is the only chronology. There
is no timestamp, environment snapshot, machine ID, inventory hash,
fingerprint, status, rolling summary, or retention protocol.

exitCode uses the public command's integer contract. A normal child exit keeps
its integer code. A watch rerun records its logical selector result. A process
without an OS exit code is normalized to 130 for user interruption, 4 for the
feedback deadline, or 3 for another spawn or termination failure.

Dirty work is associated with the current HEAD commit. The row is
observational and does not claim that the worktree equals the commit.

A timing-log write failure prints a warning and does not change a correctness
result. Timing must not be able to paralyze testing.

### test:feedback

Timing begins at selector process start and ends after final human output. It
includes Git discovery, manifest validation, command startup, selected tests,
cancellation, and reporting. The command must return within 15 seconds.

Before an owner starts, the selector establishes Windows job or equivalent
process-tree ownership that guarantees descendant termination on controller
close; failure to establish that ownership is INFRA_ERROR and the owner is not
started. At 14 seconds the selector stops dispatch and closes that ownership,
leaving one fixed second for confirmed cleanup and final output. At 15 seconds
it returns BUDGET_EXCEEDED unless a correctness failure or interruption already
has precedence, and no owned descendant remains. The slowest started command
and a reproduction command are printed. No startup benchmark, timing history,
or eligibility lookup occurs first.

### test:watch

Initialization time is printed separately. Each warm rerun is measured from
the accepted file event to the rendered result. If no result exists at 5
seconds, a timer immediately prints a warning and the currently active owner;
completion prints the final duration. The warning does not stop the watcher,
change the test result, or change future selection.

The watcher uses the same fast-or-slow mapping as one-shot feedback. A slow
change prints its owner command and remains watched, but the slow command is
not started. A mixed change still reruns its fast owners and prints the slow
handoff.

### verify

verify records each gate and total wall time. If the run is incomplete at 90
seconds it prints the warning immediately; completion prints the final total.
The warning creates no counter or performance state.

### verify:performance

The normal manual command runs five complete verify invocations sequentially.
It retains all five observations and does not replace a failed or slow run.
Its report contains:

- all five total durations and exit codes;
- median total duration;
- maximum total duration;
- the five slowest gates, ranked by their median duration across the five
  runs, with each gate's maximum also shown.

The command returns non-zero when an underlying verify run has a correctness
or infrastructure failure. A performance threshold miss alone never changes
the exit code. The aggregate is console output, not a second richer timing
record.

An underlying failure does not cancel the remaining planned runs. User
interruption is the sole exception: it stops immediately with exit 130, prints
the partial observations as incomplete, and does not claim a five-run median.

At final acceptance of a major program stage, the user may explicitly run
verify:performance with --p95-audit. That mode performs exactly twenty
complete runs and adds nearest-rank p95 to the same report. It is never
triggered by an ordinary change, does not alter TestingManifest, and does not
create or invalidate a budget state. The report informs the human checkpoint
decision only.

Because test:feedback already has a hard per-run limit and test:watch has only
a warning, they do not need separate daily p95 systems.

## Slice 1 Rust Feasibility Diagnostic

Slice 1 performs one special Rust timing study before any package-boundary
promise. It uses byte-reversible compile-invalidating edits in extractum, warm
dependency caches, one disclosed warm-up, and three retained samples for each
command in one report.

The report measures:

1. a no-op cargo check control;
2. compile-invalidated cargo check for extractum --lib;
3. compile-invalidated cargo test for extractum --lib with --no-run and
   --message-format=json;
4. direct execution of the emitted test binary with one exact, trivial,
   zero-I/O test;
5. a separate compile-invalidated end-to-end cargo test for extractum --lib
   with that same exact test.

Each retained compile-invalidating attempt uses a unique inert mutation and
restores the source byte for byte. The invalidated command cohorts alternate
order, and Cargo timings or compiler-artifact output must confirm that the
expected extractum unit actually rebuilt. The no-run artifact and its direct
binary execution stay paired. The end-to-end command uses a distinct mutation
so it cannot reuse the paired no-run build. Direct execution must report
exactly one executed test.

Stable Cargo timings may identify compilation units, while JSON compiler
artifacts provide the exact test executable path. Stable Cargo does not expose
a clean pure-link duration. Therefore the report labels:

- command 2 as the compile/check floor;
- command 3 as combined test compilation, code generation, and link;
- the difference between commands 3 and 2 as the test-build-over-check delta;
- command 4 as harness and test execution;
- command 5 as the canonical Cargo end-to-end owner floor before selector
  overhead.

The test-build-over-check delta includes cfg(test), root test-code compilation,
dev-dependencies, the app-test-support feature context, different compiler
units, code generation, link, and process/cache noise. It is not pure link and
must not be presented as such.

Only if this delta is the actual blocker may a follow-up diagnostic time the
configured linker process directly. The implementation must not assume the
process is link.exe because a wrapper or lld-link may be configured. That
follow-up is not part of the everyday measurement system.

The diagnostic ends with an explicit decision:

| Observation | Decision |
| --- | --- |
| Compile/check floor already exceeds 15 seconds | A smaller production package boundary is required for fast root-path feedback, or those paths stay slow |
| Check fits but the test build or end-to-end owner exceeds 15 seconds | Investigate a smaller test target or test-build boundary |
| Direct binary execution dominates | Optimize harness setup or the selected test seam |
| A bounded change predicts a sub-15-second owner | Plan that change before selector activation |
| No bounded change predicts success | Keep the path slow and return to the user before an open-ended architecture rewrite |

This report is diagnostic evidence. It does not ratify anything and is not
repeated after every Rust configuration change.

## Coverage, Flake, and Quarantine Policy

verify:coverage uses V8 outside the daily feedback loop. It covers declared
critical TypeScript modules and uses a baseline ratchet: a critical module may
not lose covered statements or branches without an explicit design decision.
There is no repository-wide percentage target.

Initial critical modules include the selector, manifest validator, repository
index/cache, and process-cleanup helpers introduced here.

verify:stability repeats explicitly named suspect suites and records each
result. It never retries a failure and substitutes a later pass. The command is
diagnostic, not completion evidence.

A skipped or quarantined test has an exact path or test name, reason, owner,
tracking reference, and expiry. verify fails when an entry expires, no longer
matches, or a skip exists without an entry.

Tests that create temporary directories, processes, sockets, browser contexts,
or databases own deterministic cleanup. Windows capability problems produce an
explicit diagnostic rather than a silent success.

## Third-Party Tool Decisions

| Tool | Decision | Reason |
| --- | --- | --- |
| Vitest projects | Adopt | Existing runner; projects express environment ownership |
| Playwright | Expand ownership | Correct lifecycle owner for Chromium |
| dependency-cruiser | Adopt | Replaces custom generic import and cycle contracts |
| Knip | Adopt | Replaces duplicated unused-file, export, and dependency checks |
| @vitest/coverage-v8 | Adopt | Explicit coverage outside daily feedback |
| cargo-nextest | Small A/B experiment only | May improve reporting or isolation, but can add process cost |
| sccache | Small A/B experiment only | May improve compilation but does not fix harness or linking cost |
| Nx or Turborepo | Do not adopt | The project does not need a second general build graph |
| Testcontainers | Do not adopt | Production storage is embedded SQLite |
| Mockall | Do not adopt globally | Existing narrow traits and fakes are adequate |
| Global retries | Prohibited | They hide lifecycle and flake defects |

Optional A/B experiments use five paired runs, unchanged inventory, and a
manual maintenance-cost decision. They do not create automated adoption
thresholds or performance states.

New packages must be compatible with the locked Node and runner versions and
must be recorded in package-lock.json. Optional tools are not installed
machine-globally by the implementation.

## Failure Handling

- A correctness failure is never replaced by a later green retry.
- An unknown changed path fails before runner launch.
- A parser or cargo metadata error fails its owning command.
- A feedback deadline prints the slowest started command and returns
  BUDGET_EXCEEDED unless a prior correctness failure or interruption has
  precedence.
- A child spawn or required cleanup failure is INFRA_ERROR.
- A missing Playwright browser prints a bootstrap instruction.
- A Cargo filter that executes zero tests is invalid evidence.
- A slow owner is always printed and is discharged only by its exact command
  or verify.
- Timing warnings and timing-log write failures do not change correctness.

## Migration Sequence

### Slice 1: Measurement and Rust feasibility

Rebaseline current test inventories and existing gate durations. Add only the
minimal four-field timing writer needed for comparable observations.

Perform the Rust feasibility diagnostic above and decide whether root
extractum paths can receive a bounded fast seam or must initially remain slow.
Also record current Vitest, Svelte-check, sidecar, adapter, Playwright, Cargo,
and full verify wall times. These are ordinary observations, not benchmark
states.

Slice 1 does not introduce TestingManifest, a gate registry, resource probes,
project startup benchmarks, or the source-migration ledger.

### Slice 2A: Migration preflight

Build the bidirectional filesystem-versus-runner census. Freeze the lightweight
test-level source-contract ledger immediately before any test relocation.
This freezes obligations but not the physical legacy file inventory. Timebox
automatic extraction and enter unsupported constructs manually.

### Slice 2B: Project and browser ownership

Split Vitest projects, establish project-specific worker policies, move
Chromium extraction to Playwright, and remove competing component cleanup
ownership. Assign remaining source contracts to legacy-contract and prohibit
new obligations. Finish required mechanical mixed-file splits with stable-ID
lineage, then freeze the resulting legacy file inventory. Verify collection is
non-empty and the independent census closes in both directions.

No mandatory per-project startup benchmark is added.

### Slice 3: Source-contract replacement

Migrate the three largest Telegram and analysis contracts first, followed by
the remaining cohort. Add dependency-cruiser, Knip, RepositoryIndex, and the
guardrail against new arbitrary source-text tests. Remove legacy-contract after
its inventory reaches zero.

Use bounded sub-slices for Telegram, analysis crate boundary, analysis
application, and remaining clusters. Each sub-slice updates ledger rows,
removes replaced evidence, runs affected package checkpoints, and ends with
verify.

### Slice 4: Fail-closed feedback

Introduce the three-field TestingManifest entries, NoTestAllowlist, manifest
validator, one-shot selector, compatibility aliases, result statuses, and warm
watcher.

Path ownership is complete at the checkpoint. Known fast owners run under the
hard deadline. Known slow owners return DEFERRED_ONLY. Unknown paths return
MAPPING_ERROR. No benchmark artifact or timing state is required before a path
is allowed to be fast.

If Slice 1 identified a bounded Rust seam, implement it before marking the
corresponding paths fast. Otherwise keep those paths slow and print the exact
focused or full owner command. Slice 4 acceptance exercises each proposed fast
Rust owner once through the real test:feedback path so selector and reporting
overhead are included. This smoke is ordinary command evidence, creates no
eligibility record, and a deadline miss leaves the reviewed mapping slow.

If the bounded seam introduces a Cargo --test target, the same Rust slice
updates AGENTS.md to permit that exact target; it does not wait for Slice 5.
The target must exercise the production library API and must not copy
production logic or compile selected production modules through a test-only
path attribute.

### Slice 5: Fixed verify and workflow documentation

Replace sequential orchestration with the fixed three-rule scheduler. Add the
90-second warning, five-run verify:performance report, and optional explicit
20-run major-checkpoint audit.

Consolidate AGENTS.md and docs/project.md. docs/project.md receives a one-page
quick reference containing daily commands, fast/slow behavior, result statuses
and exit codes, and the 15/5/90-second semantics. Update
docs/value-registry.md for selector statuses, project names, and the
TestingManifest speed values fast and slow. Record their tooling-only owner
and lack of persistence, product API, and UI impact. No budget-state or
scheduler-lock vocabulary is introduced.

AGENTS.md explicitly treats DEFERRED_ONLY exit 5 as a handoff, not a broken
selector: the agent runs every printed owner command, such as test:e2e, before
claiming completion.

### Slice 6: Optional acceleration experiments

Only after portfolio and scheduling changes, run small five-pair experiments
for cargo-nextest or sccache if profiling still identifies them as plausible
solutions. Adopt a tool only when inventory and correctness are equivalent,
the observed improvement is useful, and the maintenance cost is acceptable.

Each slice ends with a clean checkpoint and one successful verify. A timing
warning does not make a correctness-successful checkpoint fail.

## Planning Decomposition

This is a program-level design, not one implementation branch. After written
approval, create one implementation plan per ordered slice. Slice 3 is further
split by contract cluster.

Each plan references this specification, defines RED/GREEN tests and a rollback
boundary, and reaches a clean checkpoint before the next plan starts.

## Rust Verification Loops

Every implementation plan that changes Rust contains a section titled Rust
Verification Loops and names affected packages, narrow RED/GREEN tests, focused
checks, package checkpoints, and the end-of-slice workspace gate. Its commands
follow the exact manifest-path forms in AGENTS.md.

Initial migration owners include extractum, extractum-telegram, and
extractum-analysis. Telegram public-interface changes checkpoint both the
producer and immediate extractum consumer where app-test-support participates.

For every Rust behavior replacement:

1. list tests first when the exact name is unknown;
2. run a non-empty exact RED test in the owning package, target, and feature
   context;
3. implement or move the behavior seam;
4. run the same exact GREEN test;
5. run cargo check for the owning package with --all-targets;
6. run cargo test for the owning package with --all-targets;
7. run an additional feature context only when the default commands do not
   cover the registered behavior;
8. checkpoint an immediate dependent package when a public cross-crate
   interface changes;
9. end the Rust-changing slice with npm.cmd run verify.

All commands reuse canonical src-tauri/target. A filtered run with zero tests
is not verification.

If a cross-package test-only seam is necessary, the producer exposes a narrow,
non-default test-support feature. Prove the producer's normal library surface
without default features and prove the seam through the consumer's all-targets
test. The normal dependency edge remains feature-free.

## Documentation and Ownership

- AGENTS.md owns agent-facing command selection and the mandatory end-of-slice
  verify.
- docs/project.md owns the developer daily loop and one-page quick reference.
- docs/value-registry.md owns selector statuses, project names, and the
  TestingManifest speed values fast and slow, including their lack of
  application persistence, API, and UI impact.
- TestingManifest owns only path, command, and fast-or-slow routing.
- Vitest and Playwright configuration own project and runner collection.
- scripts/verify.mjs owns the three explicit complete-gate command arrays and
  fixed scheduling loop.
- Package scripts expose stable owner commands.
- The local timing JSONL owns only command, duration, exitCode, and commit.

## Acceptance Criteria

1. TestingManifest entries contain only path, command, and speed; speed is fast
   or slow, path patterns do not overlap, and no timing evidence participates
   in selection.
2. Every discovered tracked or untracked executable source path has exactly
   one owner command or an exact no-test reason. An unknown path returns
   MAPPING_ERROR before a runner starts; deleted and renamed-old paths use the
   selected base manifest and no-test routing fallback.
3. test:feedback returns within the hard 15-second end-to-end limit. A deadline
   returns BUDGET_EXCEEDED and terminates owned descendants. A fixture with a
   hung child proves dispatch stops at 14 seconds and the controller returns
   with no remaining owned descendant by the 15-second deadline.
4. A non-empty fast selection can return PASS only after its owner commands
   pass. Empty Vitest and Cargo selections cannot pass.
5. A slow-only selection returns DEFERRED_ONLY without starting the slow owner
   and prints its exact command.
6. A mixed fast-and-slow selection runs the fast subset under the deadline,
   preserves any fast failure, and otherwise returns DEFERRED_ONLY with every
   slow owner command.
7. A clean comparison returns NO_CHANGES. A no-test-only selection returns
   NO_APPLICABLE_TESTS with every exact reason.
8. test:watch prints a warning for a rerun over 5 seconds without stopping the
   watcher, changing the result, or changing future selection.
9. verify covers root frontend, sidecar typecheck/build and tests, adapter
   typecheck/tests/E2E, complete Rust workspace checks/tests, Windows
   integration, and critical Playwright scenarios.
10. The verify scheduler proves that Cargo runs alone, browser and OS commands
    never overlap, and no more than two non-Cargo commands run concurrently.
    Except for user interruption, every listed command is attempted even after
    another command fails.
11. A verify duration over 90 seconds prints a warning and does not change its
    correctness exit code.
12. Every persisted timing row has exactly command, duration, exitCode, and
    commit. No budget state, fingerprint, rolling counter, or resource claim is
    persisted.
13. verify:performance performs exactly five sequential complete runs and
    reports all five durations, median, maximum, and five slowest gates. It
    never replaces a failed or slow sample.
14. The optional p95 audit runs only by explicit major-checkpoint invocation,
    retains exactly twenty runs, reports nearest-rank p95, and has no effect on
    daily selection or correctness exits.
15. Slice 1 publishes the Rust compile/check floor, combined test-build time,
    test-build-over-check delta with its stated confounders, direct test-binary
    time, canonical end-to-end Cargo time, and a bounded fast-versus-slow
    decision. Every retained invalidating sample proves the root unit rebuilt.
16. The independent census accounts for every filesystem candidate and every
    file collected by Vitest or Playwright, with only exact reasoned
    exceptions and one runner owner per runnable file.
17. Every named Vitest project rejects empty collection, project intersections
    are empty, and legacy-contract plus its command/config entries are removed
    after migration.
18. The lightweight ledger accounts for every baseline source-dependent test;
    mixed tests have explicit subgroups, moves have lineage, and every final
    row has replacement evidence resolved to an owner command executed by
    verify or a specific deletion reason.
19. No test reads arbitrary production source solely for substring or
    regular-expression assertions; fixture and indexer exceptions are exact.
20. Chromium is Playwright-owned, global retries are zero, failure artifacts
    are retained, and the five normal performance runs do not leak browser
    processes.
21. Every skipped or quarantined test has a non-expired owned entry.
22. New third-party tooling is locked, compatible with the repository
    toolchain, and does not add a general-purpose build graph.
23. AGENTS.md, docs/project.md, docs/value-registry.md, package scripts, and the
    implemented command behavior agree; AGENTS.md handles DEFERRED_ONLY by
    running every printed owner command.

## Accepted Risks

The simplified timing system has less statistical rigor. Daily data cannot
answer machine-normalized regression questions, and the optional p95 report is
specific to the machine and checkpoint where it ran. This is accepted because
measurement infrastructure must not dominate development or maintenance.

Fast and slow are human-maintained classifications. A mistakenly fast command
can hit the hard deadline; a mistakenly slow command can defer useful feedback.
The hard limit, visible owner command, manifest lint, and ordinary code review
bound that risk without automatic eligibility machinery.

The fixed scheduler may leave some safe parallelism unused. It is preferred to
a resource model whose claims, locks, fingerprints, and critical-path math
would need continuous maintenance.

Because this design omits CI and Git hooks, nothing repository-owned prevents a
human from merging without running verify. Documentation and agent workflow
make it mandatory in process but cannot enforce it.

A stale path mapping remains possible. Fail-closed classification,
non-overlapping patterns, bidirectional census checks, and test:manifest are
the mitigation. TestingManifest must not grow into a build graph in response.

## Tooling Basis

- Vitest projects and configuration: https://vitest.dev/guide/projects
- Vitest performance guidance: https://vitest.dev/guide/improving-performance
- Playwright fixtures: https://playwright.dev/docs/test-fixtures
- Playwright trace viewer: https://playwright.dev/docs/trace-viewer
- dependency-cruiser rules: https://github.com/sverweij/dependency-cruiser/blob/main/doc/rules-reference.md
- Knip configuration: https://knip.dev/reference/configuration
- cargo-nextest: https://nexte.st/
- sccache: https://github.com/mozilla/sccache
