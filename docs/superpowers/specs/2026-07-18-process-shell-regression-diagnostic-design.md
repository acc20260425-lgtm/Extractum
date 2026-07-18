# Process Shell Regression Diagnostic Design

**Status:** Approved in conversation; written review pending

**Date:** 2026-07-18

**Measured baseline tree:** `24c313a767a25284123b24ea3a4b8c083007c817`

**Historical candidate:** `b364756c7b5768d644321afeaeb81ec04e2481a4`

## Purpose

This experiment diagnoses the unexplained application-shell regression seen
in the reverted `extractum-process` extraction. The original valid session
measured a shell median increase from 9,135 ms to 10,177 ms: +1,042 ms, or
+11.41%. The focused process check improved from 9,171 ms to 2,049 ms, but the
candidate correctly failed its predeclared shell cap and was reverted.

The diagnostic localizes the first cumulative configuration that reproduces
the regression across three hypotheses:

1. Cargo charges a material tax for one additional workspace member;
2. the application-to-member dependency edge charges a material tax;
3. the concrete process boundary charges a material tax beyond membership and
   the dependency edge.

A fourth manifest-migration hypothesis is tested only if the first three
states leave it unresolved. The outcome determines what must happen before
Phase 4; the experiment does not itself retain a crate, change the roadmap's
thresholds, or start Phase 4.

## Fixed Decisions

- Run one isolated, preregistered session in a workflow-owned Git worktree.
- Use the fixed order `A0 -> B -> A1 -> C -> A2 -> D -> A3`.
- Use two discarded dirty-probe warm-ups after every state installation,
  including the initial A0 installation.
- Record seven application-shell samples per block and use their median.
- Require at least five of the seven samples in every block to lie within
  300 ms of that block's median.
- Record one no-op workspace check per block as diagnostic evidence.
- Treat +500 ms as the material-effect threshold.
- Invalidate the entire session if the range of all A-anchor medians exceeds
  300 ms.
- Add the contingent tail `E -> A4` only when A0 through A3 are provisionally
  drift-valid, B and C are below the material-effect threshold, and D meets or
  exceeds it.
- Budget 60–120 minutes. This includes the cold first build in the fresh
  worktree and rebuilds caused by six state transitions, or eight when E is
  triggered.

Thresholds, ordering, sample counts, validity rules, the E trigger, and the
decision table are frozen before the first warm-up. Observed results cannot
change them.

## Isolation and Baseline

The experiment runs in one new worktree created from the committed diagnostic
plan. Its measured A state must have the same `src-tauri` tree as
`24c313a767a25284123b24ea3a4b8c083007c817`; experiment documentation and
harness commits may differ outside the measured tree.

The worktree protects the main checkout in both directions:

- no experiment state is installed in the main checkout;
- the experiment uses its own `<worktree>/src-tauri/target` and never shares
  `G:/Develop/Extractum/src-tauri/target` with the main checkout.

Before measurement, `cargo metadata` must prove that `target_directory`
resolves below the experiment worktree. `CARGO_TARGET_DIR` must be unset. No
symlink, junction, Cargo configuration, or environment override may redirect
the experiment to the main checkout's target. The experiment never runs
`cargo clean`; all states intentionally reuse only the isolated experiment
target so the sequence measures normal incremental behavior.

The worktree, main checkout, resolved target paths, starting commits, Rust and
Cargo versions, host triple, power profile, Defender status, and relevant
Cargo environment variables are captured before A0. Measurement starts only
with no active `cargo`, `rustc`, Tauri development server, or other build
process using the experiment target. Rust Analyzer must be stopped or shown
idle for the full session.

## Experimental States

All state changes are limited to the isolated worktree. Every state has a
canonical tree hash and a checked manifest/lockfile shape before its first
warm-up.

### A — restored baseline

State A is the current post-revert Rust tree:

- workspace members are `extractum` and `extractum-core`;
- process modules remain inside `extractum`;
- the application has no `extractum-process` dependency.

A0, A1, A2, A3, and conditional A4 are separate measurements of identical
bytes. They are drift anchors, not independent variants.

### B — membership only

State B adds `crates/extractum-process` as a workspace member. The crate has a
minimal library target, no dependencies, no build script, and no moved
application code. `extractum` does not depend on it.

B isolates the cost of workspace membership and Cargo's additional package
and unit bookkeeping.

### C — membership plus dependency edge

State C is B plus a path dependency from `extractum` to the empty
`extractum-process` crate. No process source moves, facade changes, feature
changes, or application source references are added. Cargo metadata and the
rustc invocation must prove the direct dependency edge exists.

C isolates the additional cost of an application dependency edge while
keeping the dependency crate trivial.

### D — historical process boundary

State D reproduces the measured Rust candidate at
`b364756c7b5768d644321afeaeb81ec04e2481a4`:

- `external_process`, `child_process`, and `process_tree` move into
  `extractum-process`;
- private application facades preserve consumer paths;
- the candidate crate dependencies and workspace dependency migration are
  reproduced;
- the candidate `Cargo.toml`, `Cargo.lock`, and relevant Rust source bytes
  match the historical commit.

The TypeScript source-contract files from the historical implementation do
not participate in Cargo timing and are not a variable in D. Correctness and
tree-shape checks still prove that the reconstructed Rust candidate is the
intended one.

### E — contingent manifest bisection

E is permitted only when A0 through A3 are drift-valid, both B and C have
deltas below +500 ms, and D has a delta of at least +500 ms. An invalid
A0-through-A3 sequence or an earlier B crossing ends the sequence without E.
E starts from C, keeps all process implementations in the application, and
applies only D's workspace-dependency/manifest migration:

- `anyhow`, `parking_lot`, `tokio`, and `windows-sys` are declared in
  `[workspace.dependencies]`;
- existing application dependencies inherit those declarations;
- the empty `extractum-process` manifest receives D's normal and dev
  dependency declarations, including `tokio/test-util`;
- no process source, facade, visibility, or behavior moves.

Because the process implementation remains in the application, its
`windows-sys` edge necessarily remains there through workspace inheritance;
D's removal of that edge is a consequence of moving its owner, not a separate
manifest-only operation. E is followed by A4 so it also has two surrounding
A anchors.

## Measurement Command and Probe

The acceptance measurement in every block is the application-shell probe:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

For each dirty probe, the harness appends the same inert comment to
`src-tauri/src/lib.rs`, invokes Cargo directly, restores the canonical bytes,
and verifies the pre-probe SHA-256. The logical edit is identical in every
state even though D's canonical `lib.rs` contains the private facades.

Before every dirty warm-up, acceptance sample, or diagnostic probe, the
harness first runs an untimed canonical synchronization check on the restored
bytes. This makes Cargo's last compiled fingerprint canonical before the
dirty bytes are installed. Without that synchronization, repeated use of the
same comment could turn samples 2 through 7 into no-op checks. Synchronization
logs are retained but excluded from every median.

Every timed dirty probe must report that an `extractum` target was checked.
A successful Cargo exit with no checked application unit is not a valid shell
sample and invalidates the session; it is never silently replaced. The state
inventory separately captures `cargo metadata`, `cargo tree --workspace -e
features`, and the relevant rustc unit invocations so package edges, target
features, and feature unification can be compared rather than inferred from a
package-level `Checking` line alone.

The harness stores a recovery copy on disk before changing the probe source.
Restoration runs in a `finally` path, and the next command refuses to start
unless the canonical hash is present. An in-memory-only backup is forbidden.
Commands have explicit timeouts and process-tree termination; the harness
does not use an unbounded `Start-Process -Wait`.

## Per-Block Procedure

Every A or variant block uses exactly this order:

1. install and verify the canonical state, including manifest, lockfile, and
   source hashes;
2. run an untimed canonical synchronization check, then capture the resolved
   metadata, dependency-feature graph, and unit inventory for the state;
3. synchronize canonical bytes, perform dirty shell warm-up 1, restore bytes,
   and discard its dirty-probe timing;
4. synchronize canonical bytes, perform dirty shell warm-up 2, restore bytes,
   and discard its dirty-probe timing;
5. synchronize canonical bytes, then run one additional unchanged no-op
   workspace check; record wall time, Cargo's reported duration, exit status,
   and compiled/checked units;
6. for each of seven samples, synchronize canonical bytes, install the dirty
   probe, run the timed application-shell check, assert that `extractum` was
   checked, restore bytes, and retain all logs;
7. compute the block median from all seven successful wall times;
8. synchronize canonical bytes, then run one separate dirty diagnostic probe
   with Cargo timings and fingerprint logging; restore and verify the source
   again.

The diagnostic probe in step 8 uses stable Cargo `--timings` output and:

```text
CARGO_LOG=cargo::core::compiler::fingerprint=info
```

It is excluded from all medians. `CARGO_LOG` is never set for state
installation, canonical synchronization, either warm-up, the no-op check, or
the seven acceptance samples. Timings/fingerprint diagnostics therefore
cannot alter acceptance durations. Diagnostic probes run after acceptance
samples so their logging overhead cannot warm the measured series.

The no-op check is also excluded from the shell median. Its purpose is to
show whether membership or an edge changes an already-clean invocation before
the per-unit timing and fingerprint logs are interpreted.

## Calculations

For block `X`, let `M(X)` be the median of its seven recorded wall times.
For each variant, its local A reference is the arithmetic midpoint of the two
surrounding A medians:

```text
Aref(B) = (M(A0) + M(A1)) / 2
Aref(C) = (M(A1) + M(A2)) / 2
Aref(D) = (M(A2) + M(A3)) / 2
Aref(E) = (M(A3) + M(A4)) / 2
delta(X) = M(X) - Aref(X)
```

A variant is materially slower exactly when `delta(X) >= 500 ms`. Percentage
deltas, Cargo-reported durations, no-op durations, unit lists, timings, and
fingerprint logs are reported but do not override this diagnostic decision.
The report separately evaluates the existing shell cap as failed when
`delta(X) > 500 ms` or `100 * delta(X) / Aref(X) > 5`. If the absolute
diagnostic classification and existing dual-cap classification disagree, the
result routes to the anomalous-result rule below rather than silently changing
the +500 ms E trigger.

Because B, C, D, and E are cumulative configurations, the report also computes
the following descriptive contrasts:

```text
membership contrast       = delta(B)
edge-after-membership      = delta(C) - delta(B)
manifest-after-C           = delta(E) - delta(C)       # only when E runs
D-specific composite       = delta(D) - delta(E)       # when E runs
D-after-C composite        = delta(D) - delta(C)       # when E does not run
```

These contrasts help locate the new cost but are not independently randomized
effect estimates. The fixed sequence can establish that a cumulative
configuration crossed the threshold; it cannot prove that its newest factor
has a context-free additive cost of the same size.

The session is drift-valid only when:

```text
max(M(A0), M(A1), M(A2), M(A3)[, M(A4)])
- min(M(A0), M(A1), M(A2), M(A3)[, M(A4)]) <= 300 ms
```

If conditional E runs, A4 participates in the validity test for the entire
session. A range above 300 ms invalidates all B/C/D/E conclusions rather than
selectively keeping favorable comparisons.

Every seven-sample block must also have at least five samples within 300 ms of
its own median. All seven samples remain in the median; none is discarded as
an outlier. Failing this central-five stability rule invalidates the whole
session, which prevents a bimodal variant block from being classified solely
because one mode won four of seven observations.

## Session Validity and Retry Policy

The entire session is invalid if any of the following occurs:

- A-anchor median range exceeds 300 ms;
- any block fails the central-five stability rule;
- a required command fails, times out, or lacks its metadata;
- a dirty acceptance sample does not check an `extractum` target;
- a probe or state cannot be restored byte-for-byte;
- a state hash, dependency graph, target-directory assertion, or process
  quiescence check fails;
- machine sleep, restart, manual build activity, or another recorded
  infrastructure event interrupts the fixed sequence;
- fewer or more than the preregistered samples are used.

An invalid attempt remains immutable with its failure reason and raw
artifacts. A new whole-session attempt is allowed only after an objective
infrastructure cause has been recorded and corrected. The new attempt must use
a new attempt directory, new worktree, and fresh worktree-local target, then
repeat the full fixed sequence from cold A0; an invalid attempt's cache is
never reused. A completed session that passes every validity rule cannot be
rerun to seek better medians. Locators may not silently replace a valid
attempt, and there is no optional repeat or post-result marginal window in
this diagnostic.

## Protocol Immutability

Before A0, this design, the implementation plan, state patches, runner,
thresholds, commands, and expected tree hashes are committed and their
SHA-256 values are written into the session manifest. Measurement refuses to
run when they differ. Mid-flight edits to documentation, the runner, state
definitions, thresholds, or decision logic invalidate the attempt.

The runner writes each sample atomically, preserves raw stdout/stderr, and
never overwrites an attempt directory. A final `decision.json` is derived
from recorded samples by the frozen calculation code. Manual interpretation
may explain a result but cannot change its classification.

## Predeclared Interpretation and Roadmap Consequences

The earliest cumulative state that crosses +500 ms identifies the first
configuration sufficient to reproduce a material regression in this fixed
sequence. It implicates the newly added factor or its interaction with the
earlier configuration; it does not prove a context-free additive effect.
Later states may contain offsetting effects, so their incremental deltas are
diagnostic rather than a reason to erase an earlier crossing.

| Result | Interpretation | Required next step |
| --- | --- | --- |
| B is materially slower | The membership-only configuration is already sufficient to reproduce the tax. | Keep Phase 4 blocked. The current shell cap is at its absolute boundary or worse for a bare member; the owner must explicitly retain, replace, or waive the threshold framework before roadmap work resumes. |
| B is below threshold; C is materially slower | The threshold is first crossed after adding the application edge; the edge or its interaction with membership is implicated. | Keep Phase 4 blocked. Treat the cost as a likely one-time edge-related tax and require an explicit owner decision on how the shell cap handles it. |
| B and C are below threshold; D is materially slower | The cause is either D's manifest migration or the concrete process boundary. | Run the preregistered E tail, then apply the next two rows. |
| E is materially slower | The threshold is first crossed after the manifest migration; that migration, feature unification, dependency declarations, or their interaction with C is implicated. | Redesign the manifest change and run a new preregistered confirmation before reconsidering Phase 3. Do not blame the process boundary alone. |
| E is below threshold while D is materially slower | D's concrete boundary composite is implicated: source/test relocation, visibility/facades, dependency ownership, and removal of the app's direct `windows-sys` edge. | Redesign that boundary and permit a new, separately approved Phase 3 attempt; Phase 4 stays blocked until it succeeds or its dependency design changes. |
| B, C, and D are all below threshold, and none violates the existing 5% cap | The original +1,042 ms regression is not reproduced under controlled anchors. | Treat the original run as unstable evidence and run a new preregistered direct A/D A/B confirmation before changing Phase 3's recorded outcome. |

If the session is valid but produces a contradictory/non-monotone pattern not
covered by the table, no roadmap decision is made. The anomaly is documented
and any follow-up requires a new preregistration; thresholds are not adapted
after inspecting it.

The result is conditional on the preregistered order and its isolated
incremental cache. A anchors, canonical synchronization, and the central-five
rule control drift and repeated-probe state, but this is not a randomized or
counterbalanced estimate. Evidence of order-specific hysteresis therefore
routes to a separately preregistered counterbalanced experiment rather than a
post-hoc rerun of this session.

No diagnostic outcome automatically changes the existing roadmap, retains
`extractum-process`, or unblocks Phase 4. The verification record states the
applicable predeclared row, and any policy or architecture change is a
separate owner-approved design decision.

## Artifacts

The immutable session directory contains at least:

- session and environment manifests, including frozen hashes;
- worktree, commit, target-directory, and process-quiescence evidence;
- canonical state hashes and state-transition logs;
- recovery-copy and post-probe hash evidence;
- canonical-sync, warm-up, no-op, acceptance, and diagnostic logs for every
  block;
- all raw wall times and Cargo-reported durations;
- per-state Cargo metadata/feature graphs and per-run checked-unit
  inventories;
- Cargo timing HTML and fingerprint logs from diagnostic probes only;
- computed A references, deltas, A-anchor range, E-trigger decision, validity,
  and final classification;
- an invalid-session record for every abandoned attempt;
- a repository verification document summarizing the raw evidence without
  replacing it.

The verification document records the elapsed time honestly. The estimate is
60–120 minutes, not an acceptance condition.

## Completion and Cleanup

After the fixed sequence:

1. verify every state and probe source is restored;
2. return the experiment worktree to A and prove its measured tree matches the
   frozen baseline;
3. generate and independently recalculate the decision from raw samples;
4. write the verification document and update the roadmap only if a separate
   owner-approved decision calls for a policy change;
5. preserve all valid and invalid attempt artifacts until review completes;
6. remove the clean workflow-owned worktree only through the normal
   worktree-cleanup flow.

The main checkout's source tree and `src-tauri/target` are never cleaned,
rewritten, or used as experiment state.

## Non-Goals

- Retaining or reimplementing `extractum-process` during this experiment.
- Starting Phase 4 or redesigning Gemini Browser.
- Recalibrating the +500 ms diagnostic threshold after observation.
- Treating focused-package speedup as an answer to the shell regression.
- Changing Cargo profiles, build tools, linker settings, target directories,
  dependencies unrelated to the declared states, or machine configuration.
- Using cold-cache comparisons between states or sharing the main checkout's
  incremental cache.
