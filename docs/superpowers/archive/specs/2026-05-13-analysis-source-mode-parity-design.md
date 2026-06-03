# Analysis Source Mode Parity - Historical Note

> Status: shipped and archived. Current source behavior has expanded since this
> design, but the source-basis decision remains useful.

## Decision

The analysis workspace distinguishes live source context from saved-run snapshot
context. UI controls should reflect the selected basis instead of reusing live
actions in snapshot mode.

## Rationale

- Live mode can expose current provider/source controls.
- Saved-run mode should preserve the run as captured and avoid implying that
  stale or missing snapshot data can be refreshed live.
- Shared UI should route through a basis-aware contract rather than checking
  ad hoc state inside each panel.

## Preserved Contract

- Show live controls only when the source basis is live/current.
- Use captured snapshot data for saved-run browsing and evidence focus.
- Prefer explicit unavailable/pending states over hidden fallbacks.
- Keep source-surface parity where behavior is genuinely shared, but allow
  snapshot mode to be more constrained.
