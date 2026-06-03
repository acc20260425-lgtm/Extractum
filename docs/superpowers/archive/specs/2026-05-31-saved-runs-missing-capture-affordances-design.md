# Saved Runs Missing Capture Affordances - Historical Note

> Status: shipped and archived. Current analysis workspace behavior is covered
> by root docs and the analysis component tests.

## Decision

Saved runs distinguish between available snapshots, pending capture, and missing
or unavailable capture data. The UI should not silently look live when a saved
run has no usable snapshot.

## Rationale

- Users need to know whether evidence, source browsing, and chat context are
  backed by the captured run or by live/current state.
- Missing snapshot data should be visible as a status affordance, not as a
  broken empty panel.
- The same helper contract should drive Runs, Header, Source, Evidence, and Chat
  surfaces so the wording and disabled states stay consistent.

## Preserved Behavior

- Available snapshots can open captured source/evidence context.
- Pending or unavailable snapshots show status-only affordances.
- Null snapshot metadata is handled deliberately instead of leaking confusing
  live controls.
- Stale or partial saved-run metadata should prefer safe disabled states over
  implying source access that is not actually backed by the run.

## Current Pointers

Look at the analysis run snapshot helpers and companion/source surface tests for
the current implementation details.
