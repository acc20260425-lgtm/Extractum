# LLM Session Cargo Usage Measurement

**Date:** 2026-07-17
**Purpose:** Calibration input for the focused-loop workflow respec triggered
by the render-slice `no_go` (roadmap Decision Framework, branch (a)+(b)).
This document records how often the LLM development agent actually invokes
Rust verification commands per session and what those commands cost on this
machine, so the respec can predeclare thresholds from measured usage instead
of guesses.

## Method

All 33 session transcripts under the local project directory
(`~/.claude/projects/G--Develop-Extractum`, window 2026-06 through 2026-07-17)
were parsed programmatically. Every `tool_use` block with a `command` input
was classified against fixed regex patterns for `cargo check/test/build/fmt`
(split by `--workspace` versus focused/filtered form) and the repository npm
gates (`verify`, `test:rust`, `check`, Vitest runs). Mentions of commands in
document text were excluded; only executed tool calls counted.

## Command-Frequency Findings

Five of 33 sessions invoked verification commands directly. Two were Rust
TDD sessions; the rest were frontend sessions:

| Session | Date | Rust commands | Frontend commands |
| --- | --- | --- | --- |
| `dc1296b1` | 2026-06-27 | check ×3, filtered test ×4 | — |
| `a4f43fac` | 2026-07-12 | check ×6, filtered test ×2, fmt ×7 | — |
| `8cd03e40` | 2026-06-28 | — | svelte check ×93, vitest ×69 |
| `c0e08c47` | 2026-07-10 | — | vitest ×9 |
| `bc3b14db` | 2026-06-25 | — | vitest ×3, svelte check ×1 |

Reading:

- A direct Rust TDD session performs on the order of **5–15 focused
  check/test cycles** (median of the two observed sessions: ~11 commands).
- The measurement slices of 2026-07-15…17 are invisible in these counts by
  design: their Cargo invocations ran inside external runner scripts and npm
  gates. Their durations are recorded in their own verification documents and
  are used below.
- Frontend sessions dominate the transcript corpus and are unaffected by this
  respec; their loop was already optimized separately
  (`2026-07-14-daily-development-loop-performance.md`).

## Duration Evidence (same machine, previously recorded)

| Operation | Seconds | Source |
| --- | ---: | --- |
| No-op `cargo check` (canonical target) | 1.15 | 2026-07-14 daily-loop verification |
| Warm incremental workspace check after app-file edit | 7.6 | 2026-07-15 core-extraction probes |
| Same, 2026-07-17 session | 9.09 | notebooklm preflight (app median) |
| Same, editing the extracted `extractum-core` | 9.10 | notebooklm preflight (surrogate median) |
| **Focused** `cargo check -p extractum-core --all-targets` | **1.02** | notebooklm preflight (diagnostic median) |
| Cold-cache incremental check (first warm-up of a session) | 39.7–55.6 | 2026-07-14 profiling; 2026-07-17 warm-up |
| No-op full `cargo test` (compile + 18.8 s test run) | 22.72 | 2026-07-14 daily-loop verification |
| Full parallel Rust test harness alone | 18.4 | 2026-07-14 profiling |

## Derived Session Model

For a Rust TDD session of ~10 inner cycles on this machine today, each cycle
pays ~9 s for an incremental check plus app-scale test compilation when tests
run. Under a focused domain crate the same cycle pays ~1–2 s for the check
plus domain-only test compilation (measured 9× reduction on the check;
test-compile reduction to be measured per phase).

Estimated saving: **roughly 1.5–4 minutes of command wall time per Rust
session**, plus two structural benefits that do not show in medians —
lower risk of hitting the 120 s command timeout on cold first builds, and
more inner iterations per session budget.

## Recommended Calibration Inputs for the Respec

These are recommendations; the respec must predeclare its own thresholds
before any candidate measurement.

1. Domain focused-check gate: median `cargo check -p <domain-crate>
   --all-targets` ≤ **2.0 s** warm (measured core value: 1.02 s; the larger
   domain crates justify headroom).
2. Domain focused-test gate: measure the per-phase baseline
   (`cargo test -p <domain-crate>` compile+run) before declaring a number;
   no evidence exists yet for domain-sized test targets.
3. Full workspace gates stay unchanged as end-of-slice verification; they are
   excluded from inner-loop thresholds entirely.
4. Adoption is mechanical: plan templates and skills prescribe the focused
   commands for the inner TDD loop of extracted domains. No habit-change risk
   applies to an LLM executor.

## Limitations

- Only two directly observed Rust TDD sessions; the 5–15-cycle range is an
  order of magnitude, not a distribution.
- Durations are reused from other same-machine documents, not re-measured
  here; cross-referencing sessions from different weeks assumes a comparable
  warm-cache state.
- Commands executed inside external runner scripts are counted as their
  slice's recorded evidence, not as inner-loop cycles.
- No claim is portable to another machine or power profile.
