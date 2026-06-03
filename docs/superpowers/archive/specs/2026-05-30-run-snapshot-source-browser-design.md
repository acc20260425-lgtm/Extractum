# Run Snapshot Source Browser - Historical Note

> Status: shipped and archived. Current saved-run behavior lives in analysis
> snapshot helpers and Source Browser components.

## Decision

Available saved-run snapshots can open inside the shared Source Browser shell.
Pending, missing, or unavailable snapshots remain status-only and do not fall
back to live source browsing.

## Rationale

- Saved runs are only trustworthy when the user can tell what was captured at
  run time.
- Live fallback would make historical runs appear to contain current data.
- Reusing the shell keeps browsing interactions familiar while preserving a
  snapshot-specific basis.

## Preserved Contract

- Available snapshots enter the shell with snapshot browser data.
- Pending/unavailable snapshots render clear status affordances.
- Snapshot mode excludes live refresh or mutation actions.
- Evidence focus and "Back to evidence" must preserve snapshot basis.
