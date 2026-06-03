# Takeout Incomplete Recovery Policy - Historical Note

> Status: shipped and archived.

## Decision

Incomplete Takeout imports are exposed as read-only recovery state. The product
does not resume or purge incomplete imports automatically.

## Rationale

- Recovery state helps users and support understand what happened without
  mutating local data.
- Latest-attempt state is more actionable than a full historical log in the UI.
- Warning codes and aggregate counts are enough for triage while keeping raw
  payloads private.

## Preserved Contract

- Active jobs hide stale recovery prompts.
- Latest attempt wins for the visible recovery summary.
- Recovery kinds remain sanitized and coarse.
- Resume/purge decisions require a separate reviewed design.
