# Takeout Incomplete Recovery Policy Design

## Goal

Define a richer, user-facing recovery policy for incomplete Telegram Takeout
imports without adding destructive actions, persisted dismissals, or true
resume semantics.

The slice turns the existing read-only recovery state into clearer guidance:
what happened, what is safe to do next, and which known provenance warnings
limit the promise of a repeated Takeout run.

## Background

The current Takeout recovery layer already exposes sanitized state for the
latest incomplete or unsuccessful Takeout batch per source. It reports:

- local `batch_id` and `source_id`;
- batch `status`, `recovery_kind`, and `completeness`;
- aggregate item counters;
- sorted warning codes;
- coarse terminal error text for failed batches only;
- timestamps.

The current UI presents this mostly as a generic "run Takeout again" message.
That is safe, but it undersells the policy differences between interrupted,
failed, cancelled, and partial-completed imports.

## Chosen Approach

Use the safe policy approach:

- keep the backend recovery command read-only;
- keep existing Takeout start/cancel commands as the only operational actions;
- implement richer policy text and warning-code explanations in frontend state
  helpers;
- render those helpers in the existing `TakeoutRecoveryNotice`.

Running Takeout again remains the only recovery action. Saved rows from earlier
attempts are not rolled back. Repeated Takeout attempts rely on the existing
deduplication behavior.

## Scope

This slice should cover four recovery kinds:

| Recovery kind | User-facing policy |
| --- | --- |
| `interrupted` | The prior Takeout process stopped without a tracked active job. Rows already saved locally remain available. Running Takeout again starts a fresh import and deduplicates saved messages. |
| `failed` | The prior Takeout import ended with an error. Rows saved before the error remain available. Running Takeout again is allowed and deduplicates saved messages. |
| `cancelled` | The user cancelled a Takeout import. Partial rows remain available. Running Takeout again starts a fresh import and deduplicates saved messages. |
| `partial_completed` | Takeout finished, but durable provenance says the imported history is partial. Running Takeout again may collect more available history, but it must not promise a complete archive. |

This slice should also explain known warning codes without exposing warning
message bodies:

| Warning code | Explanation |
| --- | --- |
| `only_my_messages_fallback` | Telegram limited available channel or supergroup history; the import used the only-my-messages fallback. |
| `migrated_history_deferred` | Migrated small-group history was detected and intentionally deferred. |
| `export_dc_fallback` | The import used the home-DC fallback after an export-DC path was attempted. |
| `finish_takeout_failed` | Extractum could not cleanly finish the Takeout session after a terminal error. Local provenance remains available. |

Unknown warning codes should still be displayed as codes, but they should not
produce invented explanations.

## Non-Goals

- Do not add `discard`, `purge`, `delete`, or rollback behavior for incomplete
  batches.
- Do not add persisted `dismiss`, `acknowledge`, or "mark reviewed" state.
- Do not implement true resume from a Takeout cursor, max message id, split, or
  watermark.
- Do not change import, pagination, deduplication, provenance, or batch
  finalization semantics.
- Do not enable migrated small-group history import.
- Do not change forum-topic catalog refresh behavior.
- Do not expose warning message bodies, raw Telegram identifiers, raw provider
  payloads, message text, source titles, usernames, phone numbers, session
  material, headers, cookies, or compressed metadata.

## Privacy Boundary

The recovery notice may display only data already allowed by the current
sanitized recovery DTO:

- local numeric ids only when useful for diagnostics;
- aggregate counters;
- status, completeness, recovery kind, and timestamps;
- warning codes;
- coarse terminal error text for failed batches.

The notice and helper tests must not require warning message bodies. Warning
code explanations must be static product copy, not derived from private data.

## Data Flow

1. Rust `list_takeout_import_recovery_states` continues to return sanitized
   `TakeoutImportRecoveryState` values.
2. Frontend loading keeps storing recovery states by `source_id`.
3. `visibleTakeoutRecoveryForSource` continues to hide recovery notices while a
   source has an active Takeout job.
4. Frontend helper functions derive:
   - title;
   - body;
   - severity;
   - aggregate facts;
   - warning-code explanations;
   - retry/re-run guidance.
5. `TakeoutRecoveryNotice` renders those derived values.

The existing source controls remain responsible for starting a new Takeout
import. The recovery notice should explain the safe next action, not introduce
a separate command.

## UI Policy

The notice should stay compact and operational. It should not become a wizard
or a second Takeout control surface.

For full notices, show:

- severity badge based on recovery kind;
- concise title;
- one recovery-kind-specific body sentence;
- aggregate counters;
- warning code badges;
- static explanation lines for known warning codes;
- failed terminal error when present.

For compact notices, keep the body omitted as today, but retain the title,
facts, and warning-code badges. Compact notices do not render warning-code
explanation lines; those lines belong to the full source surface.

## Testing

Backend tests should continue proving that recovery state is latest-batch only,
sanitized, warning-code based, and hidden while an active job exists.

Frontend tests should cover:

- each recovery kind maps to a distinct title and body;
- `failed`, `interrupted`, `cancelled`, and `partial_completed` severities stay
  stable;
- known warning codes map to static explanations;
- unknown warning codes remain visible without fabricated explanations;
- recovery facts keep using aggregate counters only;
- active jobs still suppress visible recovery notices.

Component tests are not required if state-helper tests cover the text and the
component remains a simple renderer.

## Acceptance Criteria

- Users can distinguish failed, cancelled, interrupted, and partial-completed
  Takeout outcomes from the notice copy.
- The only recovery action described is running Takeout again through the
  existing import flow.
- The copy explicitly says repeated Takeout runs deduplicate previously saved
  messages.
- Partial-completed imports do not promise full-history recovery.
- Known warning codes add clear, static explanations.
- No destructive recovery behavior, persisted dismissal, or true resume behavior
  is added.
- Tests prove the policy mapping without introducing private Telegram content.
