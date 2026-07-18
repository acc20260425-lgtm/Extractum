# Crate Extraction Shell-Cap Revision Design

**Status:** Approved design; written-spec review requested

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

## Historical Integrity

Historical plans, frozen protocols, raw artifacts, and verification documents
remain unchanged. In particular, the 2026-07-17 `extractum-process` report
continues to record a correct rejection under the old policy.

Current normative documentation will point to this revision for new decisions.
It may add a supersession note to the focused-loop specification and update the
crate roadmap, but it must not rewrite old measurements, retry decisions, or
the reason recorded for the original revert.

## Phase 3 Consequence

The valid historical Phase 3 timing evidence was:

- application shell: 9,135 ms -> 10,177 ms;
- regression: 1,042 ms / 11.4067%;
- focused process check: 9,171 ms -> 2,049 ms;
- focused improvement: 7,122 ms / 77.66%.

Those values satisfy the revised shell cap and the unchanged focused-domain
gate. A second performance experiment is not required before reapplying the
same `extractum-process` boundary.

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
   cap and the absence of a marginal-performance repeat;
3. update the crate roadmap to record the owner decision, reopen Phase 3 for
   exact-candidate reapplication, and keep Phase 4 blocked until integration;
4. add or update a repository contract test so future plans cannot silently
   restore the old cap or marginal window;
5. leave historical extraction and diagnostic records untouched.

No product behavior, API, persistence, UI, migration, or value-registry entry
changes as part of this policy revision.

## Verification

The implementation plan must include focused documentation-contract RED/GREEN
tests, ordinary whitespace checks, the relevant TypeScript test command, and
the repository completion gate. The later Phase 3 reapplication plan must
separately include the Rust verification loops required by `AGENTS.md`.
