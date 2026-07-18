# Crate Extraction Shell-Cap Revision Design

**Status:** Approved for implementation

**Date:** 2026-07-18

## Purpose

This policy revision allows Extractum to continue splitting Rust domains into
workspace crates while bounding the compile-time cost paid by ordinary edits
to the application shell.

The owner accepts the previously measured `extractum-process` application-shell
increase of 1,042 ms / 11.41% as a reasonable one-time crate-boundary cost.
This is an explicit governance decision made after the experiment. It does not
reinterpret the historical experiment as having passed its original
500 ms / 5% preregistered gate.

## Revised Retention Rule

For Phase 3 reapplication and future crate-extraction slices, the application
shell passes exactly when both inclusive conditions hold:

- absolute median regression is no more than 2,000 ms; and
- relative median regression is no more than 20%.

Passing only one condition is insufficient. A 1,900 ms / 21% regression fails,
and a 2,100 ms / 19% regression fails. Values exactly at 2,000 ms / 20% pass.

The old 800 ms / 8% marginal-performance window is removed. A valid timing
result is classified directly as pass or fail; there is no automatic repeat
for a near-threshold result. Existing infrastructure-failure and measurement-
validity retry rules remain unchanged.

The focused-domain retention gate remains unchanged: the focused check must
improve by at least 25% and at least 2.0 seconds. Correctness gates, restoration
requirements, and end-of-slice workspace completion gates also remain
unchanged.

## Roadmap-Level Cumulative Budget

The per-slice cap does not replace a cumulative roadmap limit. The canonical
pre-Phase 3 application-shell median is 9,135 ms, and the inclusive cumulative
ceiling is 15,000 ms. Every future retained slice must therefore satisfy both
its 2,000 ms / 20% per-slice cap and the 15,000 ms absolute roadmap ceiling.

A valid post-slice median above 15,000 ms blocks automatic retention even when
the slice passes its local cap. Continuing then requires a separate
owner-approved policy revision; unused budget from one slice does not raise
another slice's 2,000 ms / 20% cap.

The ceiling limits the accepted trajectory to approximately 5,865 ms above the
canonical pre-Phase 3 measurement instead of permitting each extraction to add
2,000 ms indefinitely.

## Measurement Validity

Future shell measurements run in the focused-loop quiet window with no active
Cargo, rustc, Rust Analyzer, Tauri, or competing build process. Baseline and
candidate series retain five recorded samples and their median, but each
series is valid only when at least four of its five samples lie within 300 ms
of that series median.

An unstable series makes the measurement session invalid. It is neither a
performance failure nor a near-threshold repeat, and no retention decision may
use its median. A fresh session may run only after the quiet-window preflight
passes again. This validity retry is distinct from the removed
marginal-performance repeat.

## Historical Integrity

Historical plans, frozen protocols, raw artifacts, and verification documents
remain unchanged. In particular, the 2026-07-17 `extractum-process` report
continues to record a correct rejection under the old policy.

Current normative documentation will point to this revision for new decisions.
It may add a supersession note to the focused-loop specification and update the
crate roadmap, but it must not rewrite old measurements, retry decisions, or
the reason recorded for the original revert.

## V2/V3 Diagnostic Disposition

The approved process-shell anomaly v2 design and its possible v3 causal track
are closed as moot for the current crate roadmap. The owner has accepted the
observed Phase 3 shell cost, so identifying whether it came from workspace
membership, a dependency edge, manifest migration, or the concrete process
boundary no longer controls retention or Phase 4.

Do not implement or run v2/v3 as a prerequisite for Phase 3 or Phase 4. The v2
design remains historical documentation and may be reopened only by a new
owner decision for work that genuinely requires sub-second measurement
precision or causal attribution.

The reviewed v1 harness defects remain documented but their remediation is
deferred with v2. Until that remediation is separately planned and completed,
the v1 harness is not production-ready infrastructure for future protocols.
Existing v1 source history, reports, raw artifacts, and preserved worktrees
retain their previously declared preservation and cleanup rules.

## Phase 3 Consequence

The valid historical Phase 3 timing evidence was:

- application shell: 9,135 ms -> 10,177 ms;
- regression: 1,042 ms / 11.4067%;
- focused process check: 9,171 ms -> 2,049 ms;
- focused improvement: 7,122 ms / 77.66%.

Those values satisfy the revised shell cap and the unchanged focused-domain
gate. A second performance experiment is not required before reapplying the
same `extractum-process` boundary.

The reapplication nevertheless records non-gating before/after shell samples,
medians, and validity counts using the rule above. These measurements cannot
reverse the owner decision or reject the exact historical candidate. A valid
post-reapplication median seeds the cumulative roadmap ledger; if the
diagnostic series is invalid, Phase 3 may still retain after correctness and
completion gates, but Phase 4 must establish a valid shell baseline before its
own performance decision.

The implementation must reconstruct the candidate from historical commit
`b364756c`, verify the intended tree/blob identity, and rerun all current
correctness and completion gates. Phase 3 is retained only after those gates
pass. Phase 4 remains blocked until the reapplied Phase 3 change is integrated;
it does not start merely because this policy document exists.

If the reconstructed boundary differs materially from the historical
candidate, it is a new candidate and requires fresh preregistered timing
evidence under the revised cap.

## Documentation and Contract Changes

Implementation will:

1. mark this document as the current shell-cap authority from its effective
   commit forward;
2. update the focused Rust loop specification to reference the 2,000 ms / 20%
   cap, the 15,000 ms cumulative ceiling, the series-validity rule, and the
   absence of a marginal-performance repeat;
3. update the crate roadmap to record the owner decision, reopen Phase 3 for
   exact-candidate reapplication, close v2/v3 as moot, establish the cumulative
   ledger, and keep Phase 4 blocked until integration;
4. add or update a repository contract test so future plans cannot silently
   restore the old cap, omit the cumulative ceiling or validity rule, or
   restore the marginal window;
5. record non-gating Phase 3 shell diagnostics during reapplication;
6. mark the v2 design as superseded for the current roadmap without rewriting
   its technical contents;
7. leave historical extraction and diagnostic records untouched.

No product behavior, API, persistence, UI, migration, or value-registry entry
changes as part of this policy revision.

## Verification

The implementation plan must include focused documentation-contract RED/GREEN
tests, ordinary whitespace checks, the relevant TypeScript test command, and
the repository completion gate. The later Phase 3 reapplication plan must
separately include the Rust verification loops required by `AGENTS.md`.
