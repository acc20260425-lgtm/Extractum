# Sanitized Diagnostics Design - Historical Note

> Status: shipped and archived. This note preserves the privacy model behind
> the diagnostics DTO.

## Decision

The diagnostics backend exposes an explicit allow-list DTO. Every field is
designed as aggregate state, sanitized status, or coarse metadata. Redaction is
treated as defense in depth, not as the main privacy boundary.

## Rationale

- Diagnostics should be useful without revealing raw Telegram/provider data,
  local file contents, tokens, usernames, message text, paths with private
  context, or unbounded error strings.
- The safe shape must be enforced before data reaches the frontend. The UI
  should not need to decide whether backend fields are sensitive.
- Counts, readiness states, and recent sanitized warning categories are enough
  for support triage.

## Current Contract

- Backend modules live in `src-tauri/src/diagnostics/`.
- Frontend API and type mirrors live in `src/lib/api/diagnostics.ts` and
  `src/lib/types/diagnostics.ts`.
- Display normalization lives in `src/lib/diagnostics-view-model.ts`.

## Still-Relevant Constraints

- Prefer aggregate counts over item samples.
- Prefer explicit enums and labels over raw backend messages.
- Keep filesystem diagnostics coarse.
- Keep browser/frontend diagnostics free of raw local storage or route payloads.
- If a future diagnostic needs raw material, add a separate reviewed workflow
  instead of extending this read-only DTO casually.
