# Process Shell Anomaly Qualification v2 Design

**Status:** `moot` for the current crate roadmap; preserved historical design

**Date:** 2026-07-18

## Current Roadmap Disposition

The owner decision in
[`2026-07-18-crate-extraction-shell-cap-revision-design.md`](2026-07-18-crate-extraction-shell-cap-revision-design.md)
accepts the observed Phase 3 shell cost. V2 and a possible v3 no longer control
Phase 3 retention or Phase 4 entry and must not run as roadmap prerequisites.

This technical design remains preserved for a future owner-approved task that
genuinely requires sub-second precision or causal attribution. Its reviewed v1
harness remediation is deferred with it; the current v1 harness is not
production-ready infrastructure for another protocol.

**Archived v1 protocol commit:**
`783c46a1eacce8c92b5e73efbaed247ef57a99d6`

**Archived v1 verification record:**
`docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md`

## Purpose

The v1 process-shell experiment ended with
`environment_precision_insufficient`: two complete measured attempts failed
the preregistered 300 ms stability rules. That result supports no causal
B/C/D/E classification and requires a separately frozen anomaly protocol
before any new causal experiment.

V2 is that anomaly protocol. It answers one bounded question:

> Can the normal development environment reproduce the same canonical A
> dirty-check and no-op measurements across time, probe order, and two fresh
> Cargo targets within the original 300 ms precision?

V2 does not test workspace membership, dependency edges, manifest migration,
or the extracted process boundary. It cannot retain `extractum-process`,
change the roadmap, unblock Phase 4, or make a B/C/D/E causal claim. A passing
v2 only permits the owner to design and freeze a separate v3 causal protocol.

## Fixed Owner Decisions

- Use a two-stage workflow: v2 qualifies the environment; a possible v3 is a
  separate design, plan, freeze, run, review, and decision.
- Target an unattended v2 runtime of approximately one hour, excluding
  implementation, validation, and completion gates.
- Use exactly two independent detached worktrees and two initially absent,
  worktree-local Cargo targets.
- Keep the machine representative of normal development. Do not change the
  active power plan, disable Defender, add a Defender exclusion, or alter
  global or repository Git/Cargo configuration for the measurement.
- Preserve the original 300 ms precision. V2 may not relax that threshold in
  response to observed data.
- Use deterministic mirrored paired interleaving rather than random order.
- Stop after publishing `environment_qualified` or
  `environment_not_qualified`. Never launch v3 automatically.
- Preserve all v1 source history, raw artifacts, worktrees, and verification
  bytes until their separately approved cleanup point.

The sample count, schedules, thresholds, calculations, environment invariants,
recovery rules, commands, artifact schemas, and report contents are frozen in
Git before the first v2 warm-up. Observed v2 data cannot change them.

## Scope and Non-Goals

V2 measures only canonical state A. It characterizes four possible sources of
measurement imprecision:

1. within-cell dispersion;
2. fresh-target disagreement;
3. first-half versus second-half temporal drift;
4. carryover from the preceding measured trial kind.

V2 does not estimate a causal architecture effect, select a new causal
threshold, tune the sample count for v3, identify an operating-system root
cause, or apply machine-level performance changes. If v2 fails, those actions
require another owner-approved design based on the immutable v2 evidence.

## V1 Preservation and Review Disposition

The v1 verification Markdown, raw session, artifact index, locator, decision,
ledger, and preserved attempt worktrees remain byte-for-byte unchanged. V1
replay uses its pinned commit, not the evolving branch head.

The remediation creates this separate durable record:

`docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic-review-disposition.md`

That record must:

- list the three Important review findings and the minor whitespace,
  correction-evidence, and final-restoration findings;
- state that the v1 terminal classification and raw evidence remain sound;
- supplement the immutable v1 report with clearly non-causal per-attempt
  anchor ranges, local A references, deltas, percentages, E-trigger state,
  shell-cap results, and descriptive contrasts recalculated from indexed raw
  evidence;
- summarize final A trees, canonical source hashes, recovery/restoration pair
  counts, and the independently verified artifact-index inventory;
- state explicitly that the process-local `core.longpaths=true` correction
  smoke was observed by the operator but was not captured as independently
  indexed command/config evidence;
- forbid reuse of the v1 harness as a current production-ready protocol;
- point replay users to commit `783c46a1eacce8c92b5e73efbaed247ef57a99d6`;
- point future work to the corrected v2 implementation and its own lock.

Current source files may evolve after the v1 terminal record because Git keeps
the exact v1 implementation at the pinned commit. No current-source change may
be represented as part of the historical v1 run.

## Architecture and Namespaces

V2 adds qualification-specific modules under
`scripts/process-shell-diagnostic/`:

- `anomaly-protocol.mjs` and `anomaly-protocol.test.ts` own the pure schedule,
  statistics, gates, and terminal classification;
- `anomaly-attempt.mjs` and `anomaly-attempt.test.ts` own one fresh-target
  replicate and its exact-A restoration;
- `anomaly-coordinator.mjs` and `anomaly-coordinator.test.ts` own the v2
  locator, append-only ledger, two valid replicate slots, crash recovery, and
  terminal projection;
- `anomaly-report.mjs` and `anomaly-report.test.ts` independently recalculate
  raw evidence, build the artifact index, and render the deterministic report;
- `anomaly-freeze.mjs` owns v2 input hashing and verifies
  `anomaly-protocol-lock-v2.json`.

The modules reuse the existing process runner, atomic publication, Git state,
environment capture, target-isolation, exact-A installation, and sidecar
placeholder primitives only after the reviewed defects are fixed and covered
by regression tests. V2 does not call the B/C/D/E state orchestration.

V2 uses identifiers that cannot collide with v1:

- scratch parent:
  `%TEMP%/extractum-process-shell-anomaly-v2-sessions`;
- external locator: `process-shell-anomaly-v2.locator.json`;
- session directory: `process-shell-anomaly-v2-session-<uuid>`;
- worktree parent:
  `G:/Develop/Extractum/.worktrees/process-shell-anomaly-v2-session-<uuid>`;
- replicate worktrees: `replicate-001`, `replicate-002`, and fresh corrected
  infrastructure retries with the next monotonic replicate number;
- lock: `scripts/process-shell-diagnostic/anomaly-protocol-lock-v2.json`;
- report:
  `docs/superpowers/verification/2026-07-18-process-shell-anomaly-v2.md`.

The terminal report consumes exactly two valid replicate results. An
infrastructure-invalid replicate never occupies a valid slot.

## Isolation and Canonical State

Each valid replicate starts from the v2 lock-containing commit in a fresh
detached worktree. Its `<replicate-worktree>/src-tauri/target` must not exist
before preparation. `CARGO_TARGET_DIR` is unset, and `cargo metadata` must
prove that `target_directory` resolves below that replicate worktree.

The measured `src-tauri` state is exact A with the tree frozen in the v2 lock.
The protocol records the root tree, `src-tauri` tree, canonical
`src-tauri/src/lib.rs` SHA-256, manifest/lock shape, feature graph, and resolved
target path before warm-up and after final restoration.

Before the first Cargo command, the replicate materializes the ignored regular
file
`src-tauri/binaries/gemini-browser-sidecar-x86_64-pc-windows-msvc.exe` with
size `0` and SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
It is a Tauri build prerequisite, is never executed or bundled by v2, and is
recorded in the replicate result.

The main checkout, v1 worktrees, v1 targets, implementation-worktree target,
and another replicate's target are read-only and excluded from every v2 Cargo
invocation.

## Probe Semantics

Every warm-up and recorded trial starts from canonical source bytes and begins
with a reset-sync command:

`cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`

The reset-sync result is always captured. It prepares canonical Cargo target
state but is not one of the 12 measured values in a cell.

### Dirty trial (`D`)

1. Run and validate reset-sync.
2. Verify the canonical `lib.rs` hash.
3. Publish an exclusive recovery copy and its SHA-256.
4. Append the fixed probe suffix
   `\n// process-shell-diagnostic-probe\n`.
5. Run the measured Cargo command.
6. Restore canonical source bytes from the recovery copy in `finally`.
7. Verify the restored source hash and persist restoration evidence.

The measured wall and Cargo durations come only from step 5. The target may
represent dirty source after this trial; the next trial's reset-sync returns it
to canonical state.

### No-op control trial (`N`)

1. Run and validate reset-sync.
2. Verify the canonical `lib.rs` hash.
3. Run the same Cargo command again without changing source bytes.
4. Verify source bytes remain canonical.

The measured wall and Cargo durations come only from step 3.

Both trial kinds therefore have one explicit preparation command and one
measured command. Every child process uses the Windows process-tree runner,
captures stdout/stderr through close, records wall and Cargo-reported duration,
and refuses derived artifacts after unconfirmed termination.

## Warm-Up and Recorded Schedule

Each replicate performs one discarded four-trial warm-up cycle. The warm-up
uses the same schedule assigned to that target and establishes the predecessor
kind for the first recorded trial.

Replicate slot T1 uses six recorded cycles of:

`D -> N -> N -> D`

Replicate slot T2 uses six recorded cycles of:

`N -> D -> D -> N`

Each target therefore produces exactly 24 recorded trials: 12 `D` and 12 `N`.
Across both targets, each probe kind has exactly 12 observations preceded by
the same kind and 12 preceded by the other kind. Warm-up measurements are
persisted but never enter qualification arithmetic.

The schedule is literal and deterministic. Operators cannot shuffle, omit,
repeat, or append a trial. A command failure ends the replicate as
infrastructure-invalid after exact-A recovery; it does not produce a partial
statistical result.

## Captured Evidence

Every trial persists:

- replicate slot and monotonic replicate id;
- warm-up or recorded disposition;
- cycle number, position, trial kind, and predecessor kind;
- reset-sync intent, process result, stdout/stderr hashes, wall duration, and
  Cargo-reported duration;
- measured-command intent, process result, stdout/stderr hashes, wall
  duration, Cargo-reported duration, and checked-package evidence;
- canonical, mutated when applicable, recovery, and restored source hashes;
- process termination confirmation and operator-action marker;
- target and workspace identity.

Each replicate also persists the cold build, environment snapshot, metadata,
target-isolation proof, toolchain versions, power plan, Defender result,
relevant Cargo variables, preflight inventory, final canonical sync, final
exact-A tree/hash audit, and elapsed timestamps.

Power-plan and Defender values are observed, not modified. Defender
`QuickScanAge` is descriptive. All other allowlisted environment invariants
must match across valid replicates.

## Statistical Definitions

All decisions use unrounded finite wall-clock milliseconds. Rendered values
may be formatted for readability, but the report retains the raw values.

For an even-size sample, the median is the arithmetic mean of the two middle
sorted values. A value is within band when its absolute difference from the
cell median is less than or equal to 300 ms.

The four cells are `T1-D`, `T1-N`, `T2-D`, and `T2-N`, each with exactly 12
recorded values.

The following calculations are mandatory:

- `withinBandCount(cell)`: number of the 12 values within `<= 300 ms` of that
  cell's median;
- `targetDelta(D)`: absolute difference between the `T1-D` and `T2-D`
  medians;
- `targetDelta(N)`: absolute difference between the `T1-N` and `T2-N`
  medians;
- `halfDrift(cell)`: absolute difference between the median of the first six
  chronological values and the median of the last six chronological values in
  that cell;
- `predecessorEffect(D)`: absolute difference between the pooled median of the
  12 `D` values after `D` and the pooled median of the 12 `D` values after
  `N`;
- `predecessorEffect(N)`: absolute difference between the pooled median of the
  12 `N` values after `N` and the pooled median of the 12 `N` values after
  `D`.

The report independently reconstructs cell membership and predecessor groups
from the frozen schedule plus raw trial positions. It does not trust summary
fields written by the attempt runner.

## Qualification Rule

V2 returns `environment_qualified` only when all conditions hold:

1. every cell has `withinBandCount >= 9`;
2. `targetDelta(D) <= 300 ms`;
3. `targetDelta(N) <= 300 ms`;
4. every one of the four `halfDrift` values is `<= 300 ms`;
5. `predecessorEffect(D) <= 300 ms`;
6. `predecessorEffect(N) <= 300 ms`;
7. both valid replicates have exact command, metadata, environment, isolation,
   termination, and final-A evidence.

If both replicates are evidence-valid and any statistical condition fails, v2
returns the terminal `environment_not_qualified`. The result lists every
failed rule; it is not an invalid attempt and is never followed by an
identical statistical retry.

An infrastructure failure leaves the session in an operational
`awaiting_correction` state. A fresh replicate is allowed only after the
operator records a concrete objective cause plus verified correction and
repeats process-quiescence checks. The failed worktree and all raw artifacts
remain preserved. The first two evidence-valid replicates become T1 and T2;
failed replicate ids are never reused.

## Crash Safety and Process Termination

The external locator is published before session work. Numbered ledger events
are append-only, exclusive, contiguous, and projected idempotently into the
aggregate ledger and decision. Intent artifacts precede child commands, and
immutable result artifacts follow them.

Any `termination_unconfirmed` condition, including one raised during
post-result protocol-pin verification:

- publishes a durable `replicate_termination_unconfirmed` marker when
  possible;
- leaves the replicate unfinished;
- sets `operatorActionRequired: true`;
- forbids Git, Cargo, or another child process on every unattested resume;
- requires a fresh process-quiescence attestation before filesystem/Git
  recovery or exact-A restoration can run.

The resume gate that detects an unfinished or termination-marked replicate is
filesystem-only and runs before protocol Git verification. General recovery
must not catch and downgrade unconfirmed termination into an ordinary
coordinator failure.

A crash at any atomic-publication boundary can only replay identical bytes or
publish a missing peer. Conflicting durable output is terminal protocol
corruption and is never overwritten.

## Review Remediation Before Freeze

The following reviewed defects require RED tests before production changes:

1. The independent causal reporter must evaluate base stability first,
   determine whether E is required, and then evaluate E/A4 stability. An
   unstable E or A4 returns the same `stability_invalid` projection as the
   production evaluator rather than throwing.
2. Coordinator recovery must preserve `termination_unconfirmed` from
   post-result pin verification, record its marker, leave the attempt
   unfinished, and demand attestation before any later child command.
3. Invalid/precision evidence rendering must include clearly non-causal
   anchor ranges, local A references, deltas, percentages, E-trigger state,
   shell-cap results, descriptive contrasts, and final restoration evidence.
4. Every deterministic Markdown renderer must emit exactly one terminal
   newline so ordinary `git diff --check` and staged `git diff --cached
   --check` pass without a whitespace-policy exception.

These corrections describe current source after v1. They do not rewrite the
historical v1 report or claim that v1 ran corrected code.

## Freeze and Pre-Measurement Validation

The v2 lock includes every executable protocol input, threshold, schedule,
command, fixture, expected A tree/hash, schema, report renderer, design, plan,
and review disposition. The lock itself is committed, its Git blob and
SHA-256 are recorded, and the containing commit is the only allowed v2
protocol commit.

Before measurement, a separate validation workflow must prove:

- both mirrored schedules produce 12 `D`, 12 `N`, and balanced predecessor
  groups;
- the ignored sidecar prerequisite is exact;
- a fresh target is absent before preparation;
- cold build, warm-up, all trial kinds, final sync, and exact-A restoration
  work on real Windows processes;
- target isolation rejects the main, v1, implementation, validation-peer, and
  other-replicate targets;
- normal and injected crash/recovery paths preserve immutable artifacts;
- the full reporter can independently validate a synthetic qualified result
  and a synthetic not-qualified result;
- the validation worktrees and targets are removed through the normal clean
  workflow without touching main, v1, or future measurement paths.

After the lock-containing commit, no locked input may change until the v2
session is terminal. A defect discovered after the first warm-up invalidates
the affected replicate and requires an owner-approved new freeze; it cannot be
patched into the running session.

## Reporting

The v2 reporter first verifies the external locator, protocol commit, lock
blob, lock SHA-256, contiguous ledger, raw result for every replicate, full
artifact-index content, and unchanged main target snapshot. It independently
recalculates every qualification value from raw recorded trial durations.

The committed report contains:

- protocol commit/blob/SHA and v2 version;
- environment, toolchain, power, Defender, Cargo variables, main target digest,
  and process attestation;
- every replicate, infrastructure invalidation, corrected cause, and recovery;
- literal schedules and all warm-up/reset/measured command evidence;
- all four 12-value wall and Cargo series;
- cell medians and within-band counts;
- target deltas, half-series drift, and predecessor effects;
- trial source/recovery/restoration hashes and final exact-A evidence;
- artifact-index SHA-256, file count, and byte count;
- the complete list of passed and failed qualification rules;
- exactly one terminal outcome and its consequence for v3.

The report makes no B/C/D/E claim. `environment_qualified` means only that the
fixed v2 controls supported 300 ms precision. `environment_not_qualified`
keeps v3 and Phase 4 blocked.

## Value Registry and Product Impact

The remediation registers `environment_qualified`,
`environment_not_qualified`, `replicate_termination_unconfirmed`, and every
new v2 operational status/reason in `docs/value-registry.md` before freeze.
The diagnostic harness owns these values. They are persisted only in temporary
raw evidence, the artifact index, and committed verification documents.

There is no SQLite migration, product API field, frontend state, UI copy, or
product fixture impact. The values do not authorize a roadmap or architecture
change. Their user action is evidence review; their lifecycle ends at the v2
decision or an explicit infrastructure correction.

## Testing and Completion Gates

Implementation follows test-driven development one behavior at a time. Every
production behavior is preceded by a focused failing test that fails for the
expected missing behavior.

Focused tests cover:

- exact schedules, counts, predecessor balance, medians, inclusive bands,
  target deltas, half drift, predecessor effects, and both outcomes;
- dirty/no-op trial ordering, canonical reset, exclusive recovery copies,
  source restoration, and final A;
- two isolated valid slots, infrastructure retries, no statistical retry,
  locator collision prevention, crash replay, and terminal projection;
- unconfirmed termination during both ordinary execution and post-result pin
  recovery;
- independent arithmetic, tamper rejection, artifact-index replay, report
  completeness, final restoration, and terminal newline;
- the reviewed unstable-E and unstable-A4 causal reporter paths.

The Windows lifecycle test runs in a context permitted to terminate its own
test child tree. Sandbox `Access denied` is environment evidence, not a reason
to weaken the test.

After the v2 report is generated, completion requires:

1. all focused v1-remediation and v2 harness tests;
2. `npm.cmd run check:rustfmt`;
3. `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`;
4. `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets`;
5. `npm.cmd run verify`;
6. ordinary unstaged and staged Git whitespace checks;
7. independent code review of source, locks, raw index, calculations,
   restoration, and the generated report.

No check is reported as passing unless its command was run and returned zero.

## Runtime and Operational Boundary

The expected unattended v2 measurement is 45 to 60 minutes. A single child
Cargo command retains the frozen 30-minute timeout. A session crossing 75
minutes may stop only between trials, restore exact A, and become
infrastructure-invalid; it may not publish a partial qualification result.

The operator establishes process quiescence before session start and before
every corrected infrastructure resume. During measurement, no Cargo, rustc,
Rust Analyzer, Tauri, Vite, or other build activity may compete with the two
replicate targets. The coordinator is run as one yielded process and monitored
at intervals no longer than 45 seconds, with a user-visible progress update at
least every 60 seconds.

V2 ends after its terminal report and review. Even when the result is
`environment_qualified`, creating v3 requires a new owner-approved design and
plan with its own causal hypotheses, states, sample count, ordering,
thresholds, lock, validation, and session namespace.
