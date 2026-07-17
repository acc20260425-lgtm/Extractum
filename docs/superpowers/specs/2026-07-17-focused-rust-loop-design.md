# Focused Rust Development Loop Design

**Status:** Approved in conversation
**Date:** 2026-07-17

## Purpose

This specification defines the Rust inner-development loop used by Codex in
Extractum after the crate-roadmap Stage 0 experiment falsified the claim that
editing an extracted dependency materially accelerates a full-workspace
check. It makes focused package checks and tests the normal feedback loop for
small Rust changes while preserving full-workspace checks as mandatory
end-of-slice correctness gates.

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
- the predeclared retention and shell-regression thresholds;
- the measurement artifact and negative-outcome path.

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

1. no active `cargo`, `rustc`, `rust-analyzer`, or Tauri process;
2. recorded Rust/Cargo versions, power profile, and Defender state;
3. one discarded warm-up;
4. five recorded samples;
5. the median of the five samples;
6. the shared canonical target directory;
7. byte-for-byte probe restoration plus SHA-256 verification after every
   sample.

Thresholds are fixed before candidate measurements and are never adjusted in
response to observed results.

### Retention gates

The focused domain check must improve by both:

- at least 25%; and
- at least 2.0 seconds in absolute median wall time.

The application-shell probe may regress by no more than both:

- 5%; and
- 0.5 seconds in absolute median wall time.

The shell probe uses an inert edit in a source file retained by `extractum`.
It is measured before and after extraction with the full-workspace check,
because it exists to detect a cost imposed on ordinary application work.

Passing only one side of either paired threshold is insufficient. A correct
candidate that misses the performance threshold records an honest negative
result; its thresholds are not recalibrated afterward. Any architectural-only
retention must already be authorized by the relevant phase specification.

## Failure Classification

- **Infrastructure failure:** the command did not start, measurement metadata
  is absent, an unrelated process invalidated the session, or probe bytes were
  not restored. Discard the entire affected measurement session and restart
  from its warm-up.
- **Baseline failure:** the baseline command or test fails. Stop and restore a
  valid baseline before measuring a candidate.
- **Candidate correctness failure:** the candidate does not compile in its
  focused package or its focused tests fail. Do not measure or retain it as a
  performance success.
- **Performance no-go:** the correct candidate misses either focused-domain
  improvement threshold or either application-shell regression cap. Record
  the negative result and follow the phase's already-approved retain/revert
  branch.
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
- Rust plans are required to name focused and completion commands separately.

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
- its implementation plan and verification evidence.

It does not change Cargo manifests, production Rust code, `scripts/verify.mjs`,
or any Superpowers skill. A later skill change requires evidence that agents
systematically violate the repository policy and must follow the skill
RED/GREEN pressure-testing workflow.

## Acceptance Criteria

1. `AGENTS.md` contains an unambiguous focused Rust loop and preserves full
   workspace gates.
2. Every documented focused command explicitly names its package.
3. A zero-test filtered run is explicitly rejected as evidence.
4. The source contract demonstrates RED before the policy lands and GREEN
   afterward.
5. The crate roadmap links this specification and marks the respec complete.
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
