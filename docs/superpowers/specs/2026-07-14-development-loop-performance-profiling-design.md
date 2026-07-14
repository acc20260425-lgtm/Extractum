# Development Loop Performance Profiling Design

## Status

Approved conversational design, pending review of this written specification.

## Context

The daily-loop optimization slice completed on 2026-07-14 reduced the
same-machine full Vitest median from 130.49 seconds to 68.72 seconds, added
focused frontend and Rust commands, reduced steady Rust artifact storage, and
preserved the full verification gate. Its final evidence is recorded in
`docs/superpowers/verification/2026-07-14-daily-development-loop-performance.md`.

The next action must not assume that frontend or Rust work is the better
optimization target. The preceding design already established that Vitest uses
the `node` environment by default: 126 of the then-current 145 test files under
`src` used `node`, while 19 files opted into jsdom with per-file directives.
There is no repository-wide DOM initialization cost for a node/DOM project
split to remove.

The remaining evidence also describes different kinds of cost:

- approximately 69 seconds for a full frontend gate;
- approximately 21 seconds for a warm full Cargo test command, of which about
  18 to 19 seconds is test execution;
- focused daily-loop commands that normally complete in seconds;
- a cold Cargo timing report captured immediately after an explicitly approved
  cache cleanup and profile change.

These values cannot by themselves determine the next optimization. This slice
profiles the three remaining areas without changing application or committed
test behavior.

## Goal

Produce reproducible evidence that identifies whether the next optimization
should target Vitest execution, Rust test execution, incremental Rust
compilation, or nothing.

The output is one verification document containing raw observations,
aggregates, limitations, and one evidence-backed recommendation.

## Non-Goals

- No permanent Vitest, Cargo, test, build, or application configuration change.
- No split into node and DOM Vitest projects.
- No committed `isolate: false` setting.
- No test rewrites, SQLite-fixture rewrites, timeout changes, or production-code
  refactors.
- No dependency changes, linker changes, crate split, cache cleanup, or target
  directory change.
- No installation of `cargo-nextest` or any other profiler without a separate
  user decision.
- No portable performance guarantee and no timing assertion in the test suite.
- No attempt to optimize the full gate merely because its absolute duration is
  larger than a different subsystem's gate duration.

## Selected Approach

Use a two-phase measurement protocol.

Phase one is observational. It collects repeated Vitest file timings, analyzes
the existing cold Cargo report, captures a representative incremental Cargo
report, and profiles the Rust test-execution floor with tools already installed
on the workstation.

Phase two is conditional. A reversible A/B experiment runs only when phase one
identifies a specific candidate. The only currently authorized frontend A/B is
`--no-isolate` on an explicit, statically reviewed node-only subset. A
`cargo-nextest` experiment is not part of this execution because the command is
not installed; the report may recommend a separately approved installation
experiment.

No temporary experiment changes committed configuration or behavior.

## Measurement Environment

All measurements run on the same Windows workstation used for the previous
baseline. The verification record captures the actual OS, CPU/logical-core
count, memory, Node, npm, Vitest, rustc, and Cargo versions observed at
execution time.

Before measurements:

1. Require a clean Git worktree.
2. Confirm no Cargo, rustc, rust-analyzer, Tauri development process, Vitest, or
   Extractum process is active.
3. Use the canonical `src-tauri/target` and existing `node_modules`.
4. Run the three measurement families sequentially; do not overlap them with
   each other or with background builds.
5. Record the starting commit and relevant configuration hashes.

Machine noise cannot be eliminated. Repetition and medians reduce its impact;
individual timings remain workstation-specific evidence.

## Vitest Profiling

### Baseline Runs

Run the complete repository suite three times through the existing
`scripts/run-vitest.mjs` wrapper. The wrapper's research-adapter exclusion and
all current per-file environments remain active.

Use Vitest's JSON reporter to store machine-readable results in a temporary
directory outside the repository. For every run, record:

- wall-clock duration;
- complete file and test inventory;
- pass/fail status;
- each file's reported start, end, and duration data.

For each file, compute the median duration across the three successful runs.
For the complete file distribution, report p50, p90, p95, and the ten highest
median durations. Classify files as jsdom only when their source declares that
environment; all other files remain in the effective node/default group.

Per-file durations overlap because Vitest runs files concurrently. Their sum is
not treated as wall time and is not used to claim a directly reclaimable number
of seconds.

### Import-Duration Evidence

On one of the three baseline runs, also enable Vitest's official experimental
import-duration printing. Use it to identify repeated expensive imports that
correlate with slow files. Import durations are diagnostic evidence, not values
that can be subtracted directly from wall time.

### Conditional Isolation A/B

Run this experiment only if the phase-one tail contains a coherent node-only
subset whose cost plausibly includes repeated environment or module setup.

Before the experiment, statically inspect the selected files and their shared
setup for process-global mutation, `process.chdir`, environment writes,
unrestored fake timers, global mocks, module-cache assumptions, and shared
external resources. An unresolved state owner disqualifies the file from the
subset.

Run the same explicit file list three times with normal isolation and three
times with `--no-isolate`, alternating the modes to reduce ordering bias. Every
run must execute the same file/test inventory and pass. Recommend a later
isolation design only when the no-isolation median improves by at least 15%
for that exact subset. A subset result is not extrapolated to a full-suite
speedup without a separate full-suite-safe design.

## Cargo Profiling

### Cold Report

Analyze the existing report:

`src-tauri/target/cargo-timings/cargo-timing-20260714T180231751Z-a54253738dfaee23.html`

Record its longest units, concurrency shape, and dependency waits. Keep it
explicitly labeled as a cold, profile-triggered build after cache cleanup. It
does not represent a normal small edit and is not averaged with incremental or
no-op measurements.

### No-Op Control

Run `cargo check` against `src-tauri/Cargo.toml` three times without changing a
source file. Record wall time and confirm the canonical target remains in use.
The median is the no-op control, not an edit-cycle measurement.

### Incremental Edit Probe

Use `src-tauri/src/prompt_packs/runtime_config.rs` as the representative small
root-crate source. Record its byte hash, add one inert comment with a focused
patch, and run `cargo check --timings` against the canonical manifest. Capture
the generated timestamped timing report and wall time. Then remove exactly that
comment with a reverse patch and confirm the original byte hash and clean Git
state.

The probe measures a small root-crate edit, not a dependency change and not a
link-heavy executable build. Report total duration, the `extractum` unit's
duration, dependency waits, and visible concurrency. Do not infer linker or
release-build conclusions from this check-only probe.

## Rust Test-Execution Profiling

### Full Harness Controls

Run the warm full Cargo test command three times and record both command wall
time and libtest's reported harness duration. Then run one full test command
with `--test-threads=1` to measure the effect of current test parallelism.

All runs use the canonical target and must retain the complete inventory.

### Module Groups

Use libtest's list output to inventory tests and group them by their top-level
Rust module. Run each top-level module filter separately and record:

- expected tests from the list;
- actually executed tests;
- command wall time;
- harness duration.

A group result is valid only when the executed count matches its inventory.
The grouped inventory must cover the full library-test inventory exactly once;
otherwise the report identifies the gap and does not use the incomplete groups
to rank subsystems.

If one top-level group dominates the observed critical path, partition only
that group by its next module segment and repeat the same inventory check.

Group wall times are diagnostic because separate filtered processes have
startup overhead and different parallel scheduling. Their sum is not treated
as the full-suite duration.

### Static Candidate Scan

For the slowest valid groups, inspect test code and fixtures for:

- disk-backed SQLite databases or file-backed pools;
- `std::thread::sleep`, Tokio sleeps, and real timeout budgets;
- explicit single-threading, shared locks, global state, and serial sections;
- external process or network-shaped fixtures;
- repeated expensive setup that is not shared safely.

Matches identify hypotheses only. They do not prove causality and do not
authorize changes in this slice.

## Interpretation and Decision Rules

The final report selects exactly one outcome:

1. **Vitest follow-up:** a repeatable node-file tail exists and the controlled
   isolation A/B meets its 15% subset threshold without correctness drift.
2. **Rust-test follow-up:** a valid module or second-level group dominates the
   harness evidence and has one or more concrete, testable SQLite, wait,
   timeout, or serialization hypotheses.
3. **Incremental Cargo follow-up:** the small-edit report shows a material
   root-crate compilation bottleneck that is absent from the no-op control.
4. **Stop optimizing:** no candidate has sufficiently repeatable evidence to
   justify another implementation slice; use the existing focused commands.

The cold Cargo report provides prioritization context but cannot by itself
select outcome 3. Likewise, the 69-second frontend gate and 21-second Cargo
test command are not compared as if both were equally frequent daily actions.

When more than one candidate is credible, recommend the one with the clearest
causal hypothesis and the cheapest safe validation, and list the other as a
secondary candidate. Do not convert machine-specific measurements into CI
thresholds.

## Failure Handling

- A failing test run is excluded from timing aggregates and recorded with its
  stage; it is not silently rerun until green.
- A changed inventory invalidates the comparison that depends on it.
- Failure to parse a reporter or timing artifact stops that measurement rather
  than substituting console estimates.
- If the temporary Rust edit cannot be restored byte-for-byte, stop all work
  and report the dirty path; do not continue to tests or commit evidence.
- If `--no-isolate` exposes a failure or state leak, stop that A/B, retain the
  normal-isolation baseline, and recommend against the tested subset.
- `cargo-nextest` is currently unavailable. Do not install it implicitly and do
  not treat its absence as a measurement failure.
- Intermediate JSON, logs, and derived tables live under the system temporary
  directory. Generated Cargo timing files remain ignored under the canonical
  target. Neither class is staged.

## Evidence Artifact

Create:

`docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md`

The document contains:

- environment and tool versions;
- exact commands and starting commit;
- all successful raw run durations and inventories;
- Vitest distribution and top-file tables;
- import-duration observations;
- Cargo cold/no-op/incremental distinctions and dominant units;
- Rust full, sequential, group, and optional second-level measurements;
- static candidate findings with explicit hypothesis labels;
- skipped conditional experiments and the reason for each skip;
- limitations and the selected recommendation.

Do not commit raw temporary logs or duplicate the Cargo HTML report into docs.

## Verification Strategy

This is a documentation-and-measurement slice. Its correctness checks are:

- clean starting tree and recorded commit;
- successful inventories for every aggregated test run;
- three successful repetitions where the design requires medians;
- exact restoration hash for the Rust edit probe;
- no changed source or configuration file at the end;
- `git diff --check` for the evidence document;
- final status containing only the intended evidence document before commit.

The unchanged `npm.cmd run verify` remains authoritative for implementation
work. Re-running it is not required solely to commit this measurement report
because this slice leaves code and configuration byte-for-byte unchanged.

## Acceptance Criteria

1. No application, test, build, or repository workflow behavior changes.
2. Vitest evidence includes three complete inventories, per-file medians,
   distribution percentiles, environment classification, and a slow-file tail.
3. Cold, no-op, and small-edit Cargo measurements remain separately labeled.
4. Rust evidence accounts for the full library-test inventory and distinguishes
   parallel full-suite, sequential, and module-filtered costs.
5. Every temporary source edit is restored byte-for-byte.
6. Conditional experiments either meet their prerequisites or are explicitly
   skipped; no new tool is installed automatically.
7. The final evidence selects one next action and states the limits of that
   recommendation.
8. Only the verification document is committed by the execution slice.

## Tooling Basis

The profiling methods use current official documentation:

- Vitest reporters and JSON output:
  <https://vitest.dev/guide/reporters.html>
- Vitest performance profiling and import durations:
  <https://vitest.dev/guide/profiling-test-performance.html>
- Vitest isolation performance guidance:
  <https://vitest.dev/guide/improving-performance.html>
- Cargo timing reports:
  <https://doc.rust-lang.org/cargo/reference/timings.html>
