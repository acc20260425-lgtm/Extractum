# Takeout Representative Validation and Fallback Coverage - Historical Note

> Status: archived. Current verification evidence lives under
> `docs/superpowers/verification/`.

## Decision

Takeout validation should use sanitized helper output and a representative
matrix of source shapes instead of storing raw Telegram/provider data in docs.

## Rationale

- Representative coverage is useful only if it can be shared without leaking
  private source content.
- Fallback behavior differs by source shape, access state, and import outcome,
  so matrix categories were more valuable than one-off success notes.
- Validation docs should capture aggregate counters, warning categories, and
  privacy-safe outcomes.

## Preserved Contract

- Keep helper output sanitized.
- Track source-shape coverage explicitly.
- Preserve the privacy boundary: no raw messages, access hashes, paths, or
  provider payloads in committed docs.
- Treat the active verification matrix as the current evidence source.
