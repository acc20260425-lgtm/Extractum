# Focused Rust Development Loop Design

**Status:** Approved in conversation
**Date:** 2026-07-17

**Current extraction-performance authority:**
[`2026-07-18-crate-extraction-shell-cap-revision-design.md`](2026-07-18-crate-extraction-shell-cap-revision-design.md)

The revision applies prospectively and to the approved exact Phase 3
reapplication. Historical measurements and decisions remain governed by the
thresholds frozen for their original sessions.

This authority supersedes only the shell-cap and marginal-repeat clauses in
`2026-07-17-process-and-gemini-browser-crate-boundary-design.md`. That design's
boundary, facade, dependency, correctness, and restoration requirements remain
active for the exact Phase 3 reapplication.

## Purpose

This specification defines the Rust inner-development loop used by LLM agents
working in this repository after the crate-roadmap Stage 0 experiment
falsified the claim that editing an extracted dependency materially
accelerates a full-workspace check. It makes focused package checks and tests
the normal feedback loop for small Rust changes while preserving
full-workspace checks as mandatory end-of-slice correctness gates.

The policy is project-specific. It is adopted through `AGENTS.md` and the
required structure of generated implementation plans. Repository or installed
Superpowers skills are not modified unless later evidence shows that the
standing repository instructions are systematically ignored.

## Evidence and Decision

The notebook-render Stage 0 preflight measured these warmed medians on the
same machine and shared `src-tauri/target`:

- application full-workspace probe: 9,090 ms;
- extracted-core full-workspace surrogate: 9,100 ms;
- focused `extractum-core` check: 1,020 ms.

The full-workspace surrogate was 0.11% slower, so moving code into a dependency
does not by itself improve the full-workspace loop: Cargo must still re-check
the large application crate above the edited dependency. The focused package
loop, however, avoided that work and was about nine times faster.

The project owner therefore selected the crate-roadmap branch `(a) + (b)`:

- hot-module extraction phases 4–6 use the focused package loop as their
  performance acceptance metric;
- dependency-hygiene phases may proceed on their architectural justification,
  with performance retained as diagnostic evidence;
- full-workspace correctness gates are never weakened or replaced.

## Two Verification Loops

### Focused inner loop

After a small Rust change, check the smallest directly affected package:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets
```

Run the narrowest meaningful test first:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> -- --exact
```

At a task checkpoint, run all targets for the affected package:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets
```

Code that still belongs to the application uses `-p extractum`. Code in an
extracted domain uses that domain's package. When a change directly affects
multiple packages, each affected package is checked independently before the
end-of-slice workspace gate.

A filtered Cargo test that executes zero tests is not verification, even when
Cargo exits successfully. If the exact name is not known, list tests first and
then run a non-empty selection:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p <package> -- --list
```

### Full completion loop

At the end of every Rust slice, run the unchanged workspace gates:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

The focused loop is an accelerator, not completion evidence for the entire
repository. A focused pass cannot excuse, replace, or weaken a failed
workspace gate.

## Expected Development Sequence

The normal Rust task sequence is:

1. write or identify one narrow failing test;
2. run that test in the owning package and observe the intended RED result;
3. make the smallest implementation change;
4. rerun the same non-empty test and observe GREEN;
5. run the owning package's focused check;
6. repeat for the next small change;
7. run the package-wide checkpoint when the task is complete;
8. run the full completion loop when the slice is complete.

This ordering keeps repeated feedback cheap without reducing final coverage.
All workspace members continue to share canonical `src-tauri/target`; the
focused loop must not create slice-specific target directories.

The first Rust check in a session may be a cold 39.7–55.6 second run rather
than a warm 1–2 second focused run. That expected warm-up cost is not a policy
violation or, by itself, evidence of a build problem; subsequent inner-loop
commands provide the useful warm comparison.

## Deferred Integration Feedback

A focused check of an extracted package proves that package and its own test
targets compile. It does not compile downstream consumers such as the
`extractum` application. This defers some integration feedback in exchange for
the faster repeated loop.

When a task changes a public cross-crate interface, its implementation plan
must add a checkpoint for the immediate dependent package, commonly
`cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.
Unchanged internal implementation work does not pay that dependent-package
cost after every edit. The end-of-slice workspace gates remain the final
backstop for all downstream consumers.

## Implementation-Plan Contract

Every future implementation plan that changes Rust must contain a `Rust
Verification Loops` section with executable commands for:

1. each affected package;
2. the narrow RED test;
3. the narrow GREEN test;
4. the focused package check;
5. the package-wide task checkpoint;
6. the end-of-slice full-workspace gates.

Focused and package-checkpoint commands must name `--manifest-path
src-tauri/Cargo.toml`, select the package with `-p`, and use a concrete test
filter when a focused test is claimed. End-of-slice commands instead select
the full workspace with `--workspace --all-targets`. Phrases such as "run
relevant Rust tests", "run cargo check", or "verify the backend" are not
executable verification instructions and do not satisfy the contract.

For a crate-extraction plan, the section must additionally name:

- the pre-extraction command against `-p extractum`;
- the post-extraction command against the new package;
- the matched logical probe source before and after the move;
- the focused improvement gate and 2,000 ms / 20% per-slice shell cap;
- the canonical 9,135 ms anchor and 15,000 ms cumulative shell ceiling;
- the five-sample, four-within-300-ms validity rule;
- the measurement artifact, invalid-session path, and negative-outcome path.

## Extraction Performance Protocol

### Primary metric

For hot-module phases 4–6, the primary performance metric is the median of
`cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`.
Focused test durations are recorded as diagnostic evidence and must pass, but
they are not mixed into the compile-time retention calculation.

The comparison uses the same logical domain edit:

- before extraction: an inert edit in the domain source followed by
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum
  --all-targets`;
- after extraction: the same inert edit in the moved source followed by
  `cargo check --manifest-path src-tauri/Cargo.toml -p <new-package>
  --all-targets`.

### Sampling

Baseline and candidate measurements each use:

1. no active `cargo`, `rustc`, `rust-analyzer`, Tauri, or competing build
   process;
2. recorded Rust/Cargo versions, power profile, and Defender state;
3. one discarded warm-up;
4. five recorded samples;
5. the median of the five samples;
6. the shared canonical target directory;
7. byte-for-byte probe restoration plus SHA-256 verification after every
   sample;
8. at least four of the five samples are within 300 ms of the series' own
   median (absolute deviation <= 300 ms).

An unstable baseline or candidate series invalidates the complete measurement
session. It is not a performance failure, and none of its medians may be used
for retention. A fresh session may start only after the quiet-window preflight
passes again.

Thresholds are fixed before candidate measurements and are never adjusted in
response to observed results.

### Retention gates

The focused domain check must improve by both:

- at least 25%; and
- at least 2.0 seconds in absolute median wall time.

The application-shell probe may regress by no more than both:

- 20%; and
- 2,000 ms in absolute median wall time.

The valid post-slice application-shell median must also be no more than
15,000 ms against the canonical pre-Phase 3 anchor of 9,135 ms. Crossing that
cumulative roadmap ceiling blocks automatic retention and requires a new
owner-approved policy revision. Unused cumulative budget does not raise the
per-slice 2,000 ms / 20% cap.

The shell probe uses an inert edit in a source file retained by `extractum`.
It is measured before and after extraction with the full-workspace check,
because it exists to detect a cost imposed on ordinary application work.

Passing only one side of either paired threshold is insufficient.
Values exactly at 2,000 ms / 20% pass. There is
no marginal-performance repeat: a valid result passes or fails directly.
Measurement invalidation and corrected infrastructure retries remain separate
from performance classification.

The exact Phase 3 reapplication is the single exception to a new gating shell
decision: its already-valid historical 10,177 ms result was accepted by the
owner revision. Fresh post-reapplication samples are non-gating ledger
diagnostics and cannot produce a performance no-go for that exact candidate.
The follow-on plan must use a frozen historical tree/blob identity manifest
covering every file and hunk that constitutes candidate `b364756c`. The
exact-candidate exception treats any mismatch against that manifest as
material: the result is a new candidate with fresh preregistered timing.

## Failure Classification

- **Infrastructure failure:** the command did not start, measurement metadata
  is absent, an unrelated process invalidated the session, or probe bytes were
  not restored. Discard the entire affected measurement session and restart
  from its warm-up.
- **Measurement invalidation:** fewer than four of five values in either shell
  series have absolute deviation <= 300 ms from that series median. Discard the
  complete session, classify no performance result, re-establish the quiet
  window, and start a fresh session from its warm-up.
- **Baseline failure:** the baseline command or test fails. Stop and restore a
  valid baseline before measuring a candidate.
- **Candidate correctness failure:** the candidate does not compile in its
  focused package or its focused tests fail. Do not measure or retain it as a
  performance success.
- **Performance no-go:** except for the exact Phase 3 reapplication described
  above, the correct candidate misses either focused-domain improvement
  threshold, either side of the 2,000 ms / 20% per-slice shell cap, or the
  15,000 ms cumulative shell ceiling. Record the negative result and follow
  the phase's already-approved retain/revert branch. A cumulative ceiling
  crossing requires a separate owner policy revision before retention.
- **Completion failure:** any end-of-slice workspace gate fails. The slice is
  incomplete regardless of focused-loop results; this class is not folded
  into candidate measurement evidence.

## Repository Enforcement

The implementation slice adds a stable `<!-- focused-rust-loop -->` policy
block to `AGENTS.md`. It defines the command hierarchy, non-empty-test rule,
plan-shape requirement, canonical target, and full completion gates.

A lightweight source-contract test protects only machine-significant
invariants:

- the policy anchor exists;
- focused check and test commands use the canonical manifest and explicit
  package selection;
- the policy rejects a zero-test result;
- full check and test commands use `--workspace --all-targets`;
- Rust plans are required to name focused and completion commands separately;
- the crate-extraction shell contract pins the current per-slice cap,
  cumulative ceiling, stability rule, and absence of a marginal-performance
  repeat.

The contract normalizes CRLF/LF and avoids exact prose, indentation, or whole
paragraph assertions. Performance thresholds remain normative in this spec;
`AGENTS.md` may reference the spec rather than duplicate its full measurement
protocol.

## Scope

The implementation slice may change:

- `AGENTS.md`;
- `docs/superpowers/specs/2026-07-17-crate-roadmap.md` for the status and link
  to this approved specification;
- one focused-loop source-contract test;
- `src/lib/crate-extraction-shell-cap-contract.test.ts` for the current
  extraction-performance policy;
- its implementation plan and verification evidence.

It does not change Cargo manifests, production Rust code, `scripts/verify.mjs`,
or any Superpowers skill. A later skill change requires evidence that agents
systematically violate the repository policy and must follow the skill
RED/GREEN pressure-testing workflow.

This revision changes no Rust source, Cargo manifest, product behavior, or
historical verification record.

## Acceptance Criteria

1. `AGENTS.md` contains an unambiguous focused Rust loop and preserves full
   workspace gates.
2. Every documented focused command explicitly names its package.
3. A zero-test filtered run is explicitly rejected as evidence.
4. The source contract demonstrates RED before the policy lands and GREEN
   afterward.
5. Before enforcement lands, the crate roadmap links this approved
   specification and marks enforcement pending; the implementation slice
   closes the item only after the `AGENTS.md` policy and contract are green.
6. Repository Superpowers skills remain unchanged.
7. The focused contract test, full frontend test suite, and `npm.cmd run
   verify` pass.
8. Runtime behavior and the Cargo workspace are unchanged.

## Non-Goals

- Replacing full-workspace correctness gates with focused checks.
- Automatically benchmarking every ordinary Rust edit.
- Adding a generic Cargo wrapper whose package and test filter are implicit.
- Creating or editing a Superpowers skill without demonstrated policy failure.
- Changing compiler, linker, cache, target-directory, or test-runner tooling.
