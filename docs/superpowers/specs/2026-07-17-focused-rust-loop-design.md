# Focused Rust Development Loop Design

**Status:** Approved; timing policy simplified 2026-07-19
**Date:** 2026-07-17
**Last revised:** 2026-07-19

**Current extraction-timing authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

The 2026-07-19 roadmap revision makes compile-time measurement advisory,
cancels the exact Phase 3 reapplication, and supersedes the automatic timing
gates in the 2026-07-18 shell-cap revision. Historical measurements and
decisions remain governed by the protocols frozen for their original sessions.

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

The project owner therefore selected the crate-roadmap branch `(a) + (b)`,
later simplified on 2026-07-19:

- hot-module extraction phases 4–6 use the focused package loop as their
  advisory compile-time metric;
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
- one discarded warm-up plus three recorded samples per state;
- raw values, median, and exact probe-restoration check;
- the mandatory workspace-check command whose emitted duration is recorded;
- an explicit statement that timing is advisory and cannot automatically
  retain, reject, or revert the slice.

## Extraction Performance Protocol

### Primary metric

For hot-module phases 4–6, the advisory performance metric is the median of
`cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`.
Focused test durations are recorded as diagnostic evidence and must pass, but
they are not mixed into the compile-time comparison.

The comparison uses the same logical domain edit:

- before extraction: an inert edit in the domain source followed by
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum
  --all-targets`;
- after extraction: the same inert edit in the moved source followed by
  `cargo check --manifest-path src-tauri/Cargo.toml -p <new-package>
  --all-targets`.

### Sampling

Baseline and candidate measurements each use:

1. the shared canonical target directory and sequential Cargo commands;
2. one discarded warm-up;
3. three recorded samples;
4. the raw values and median of three;
5. byte-for-byte probe restoration in a `finally` path;
6. one SHA-256 source check and one clean-worktree check after the complete
   series.

Do not add an active-process scanner, quiet-window coordinator, Job Object,
power-profile or Defender capture, formal stability test, automatic retry, or
per-sample artifact ledger. Avoid knowingly running a competing build, but do
not turn that expectation into another measurement harness.

There is no separate application-shell A/B series. Record the duration emitted
by the mandatory end-of-slice workspace check instead.

### Advisory interpretation

Report the focused baseline median, candidate median, absolute delta, and
percentage delta. The values inform the owner and future roadmap work; they do
not automatically retain, reject, or revert a correct slice. The historical
25% / 2.0-second focused gate, 2,000 ms / 20% shell cap, and cumulative ledger
are no longer active policy.

For this rule, one completed crate-extraction slice contributes one ordinary
workspace result: the duration already emitted by its successful mandatory
end-of-slice `cargo check --manifest-path src-tauri/Cargo.toml --workspace
--all-targets`. Two consecutive completed crate-extraction slices whose
ordinary workspace results are each at or above 15,000 ms trigger a separate
owner-approved performance investigation. A completed result below 15,000 ms
breaks the sequence; failed, canceled, and incomplete slices contribute no
result. Consecutive means adjacent completed extraction slices in roadmap order;
historical measurements do not seed the sequence. Focused checks, tests,
diagnostics, and same-slice reruns do not count. Do not rerun the check or add
timing samples for this rule. Timing never fails or reverts either slice.

## Failure Classification

- **Measurement failure:** a timing command did not start, did not finish, or
  probe restoration cannot be proven. Record no performance conclusion and
  stop the timing procedure. There is no protocol-mandated retry.
- **Incomplete advisory series:** preserve the values already observed, mark
  the comparison incomplete, and continue only after exact source restoration
  is proven. Do not manufacture a median or convert this into a candidate
  failure.
- **Baseline failure:** the baseline command or test fails. Stop and restore a
  valid baseline before measuring a candidate.
- **Candidate correctness failure:** the candidate does not compile in its
  focused package or its focused tests fail. This is a correctness failure,
  independently of any timing result.
- **Advisory regression:** record the values and interpretation. Timing alone
  cannot reject or revert the slice. Two consecutive completed crate-extraction
  slices whose ordinary workspace results are each at or above 15,000 ms
  trigger the separate owner-approved performance investigation defined above;
  same-slice reruns and non-completion checks do not count.
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
- the crate-extraction timing contract pins one warm-up plus three samples per
  state, advisory-only interpretation, absence of shell A/B and cumulative
  ledger, and the canceled Phase 3 reapplication.

The contract normalizes CRLF/LF and avoids exact prose, indentation, or whole
paragraph assertions. `AGENTS.md` may reference this spec rather than duplicate
the advisory measurement protocol; its correctness loops remain unchanged.

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
