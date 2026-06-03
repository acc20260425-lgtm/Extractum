# Takeout Import Read-Only Recovery - Historical Note

> Status: shipped and archived.

## Decision

The frontend displays Takeout import recovery state as read-only diagnostics.
It does not resume, retry, purge, or repair incomplete imports from the recovery
notice itself.

## Rationale

- Recovery UI should explain state without creating another mutation path.
- Active import jobs are more important than stale recovery summaries.
- Sanitized DTOs keep incomplete import details useful without exposing raw
  Telegram export data.

## Preserved Contract

- Latest recovery attempt is the visible summary.
- Active jobs suppress recovery prompts for the same source.
- Recovery reasons are coarse and sanitized.
- Mutation actions require a separate explicit workflow.
