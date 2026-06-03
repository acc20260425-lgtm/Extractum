# Analysis Snapshot Hardening - Historical Note

> Status: shipped and archived.

## Decision

Analysis captures the source/read context needed for saved-run review before
provider calls complete. Later review should prefer captured snapshot data over
live source reads.

## Rationale

- Saved runs are historical artifacts and must remain reviewable after source
  state changes.
- Capturing before provider calls protects against partial analysis output that
  has no matching evidence/source basis.
- Legacy or degraded snapshots should be visible as degraded, not silently
  upgraded with live data.

## Preserved Contract

- Capture source context before provider execution.
- Review saved runs from snapshot/read-model state where available.
- Mark degraded legacy snapshots explicitly.
- Do not let live reads overwrite the meaning of a saved run.
