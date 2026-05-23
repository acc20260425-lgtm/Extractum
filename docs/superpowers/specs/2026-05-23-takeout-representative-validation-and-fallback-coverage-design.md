# Takeout Representative Validation And Fallback Coverage Design

## Goal

Prove, with repeatable sanitized evidence, that Telegram Takeout import behaves
safely and predictably for representative source kinds and fallback scenarios
before enabling migrated-history import, richer recovery actions, or related
product behavior changes.

This is a validation-first hybrid slice. Real Telegram behavior still requires
manual runs with real accounts, but the evidence collection should be
repeatable and less fragile than a purely manual checklist.

## Current Backlog Risk

The active Takeout follow-up backlog asks for:

- representative validation on public channels, supergroups, and small groups;
- `CHANNEL_PRIVATE` fallback validation;
- shifted export DC fallback validation;
- comparison between normal sync rows and Takeout-imported rows;
- migrated small-group-to-supergroup smoke after `TAKEOUT_INIT_DELAY`;
- evidence before any migrated-history enablement;
- evidence before deciding whether Takeout finish should refresh forum-topic
  catalog state.

Read-only recovery already explains incomplete, interrupted, failed, cancelled,
and partial Takeout attempts. This slice does not change that behavior. Its job
is to make the next validation passes repeatable and safe to document.

## Scope

Build a reusable validation harness made of two parts:

1. Backend diagnostic helpers that read local SQLite state and derived ingest
   provenance.
2. A manual validation checklist and verification template under
   `docs/superpowers/verification/`.

The backend helpers should be callable by tests and future operator-facing
diagnostic commands, but this slice does not need to expose a polished user UI.

## Non-Goals

- Do not call Telegram from diagnostic helpers.
- Do not read or export session, auth, cookie, header, API, or transport
  material.
- Do not include raw Telegram TL payloads, compressed dumps, raw provider JSON,
  message text, usernames, phone numbers, or private source titles in evidence.
- Do not enable migrated small-group history import.
- Do not add resume, purge, retry, discard, or automatic recovery actions.
- Do not change read-only recovery semantics.
- Do not add UI polish.
- Do not expand media byte download, thumbnail download, preview download, or
  custom emoji handling.
- Do not change forum-topic refresh behavior; only collect evidence for a
  later decision.

## Privacy Boundary

Diagnostic output must be safe to paste into repository docs without manual
redaction.

Allowed evidence:

- local numeric ids such as `source_id`, `batch_id`, and aggregate item counts;
- source subtype and coarse source classification;
- durable batch status, completeness, timestamps, and counters;
- warning codes;
- boolean flags such as `migrated_history_detected`,
  `migrated_history_imported`, export DC attempted, and export DC fallback used;
- identity-shape aggregates, such as peer kind counts, nullability counts, and
  duplicate native-identity counts;
- row-fidelity aggregates over field presence, kind, and nullability.

Forbidden evidence:

- message text or message snippets;
- source titles, usernames, invite links, phone numbers, or account labels that
  identify a real person/source;
- Telegram session data, auth keys, API id/hash, cookies, headers, or transport
  details;
- raw TL objects, raw provider payloads, compressed dumps, screenshots that
  reveal private content, or warning message bodies.

## Diagnostic Helpers

Add backend helpers that accept only local identifiers, such as `source_id`,
`batch_id`, or an explicit comparison tuple. They must read local database
tables only.

### Source Snapshot Summary

For a given `source_id`, return a sanitized source-level summary:

- `source_id`;
- source type and subtype;
- account id only if needed for local isolation checks;
- `last_sync_state` and `last_synced_at`;
- item count;
- Telegram typed-row count;
- max local Telegram message id where safe;
- peer-kind distribution;
- reply/thread field presence counts;
- reaction metadata presence counts;
- media metadata presence/kind counts;
- topic-related aggregate counts if available.

The helper must not return source title, username, external id, raw metadata,
message text, or item payloads.

### Takeout Batch Summary

For a given `batch_id`, return a sanitized Takeout provenance summary:

- batch id and source id;
- durable status and completeness;
- inserted, duplicate, skipped, observed, failed, and warning counters;
- sorted unique warning codes;
- started, finished, and updated timestamps;
- Takeout detail flags, including migrated-history flags and export DC flags;
- max message id and split metadata as numeric/coarse values only.

The helper should include warning codes, not warning messages.

### Row Fidelity Comparison

Compare normal-sync material and Takeout-observed material for a source using
stable, non-content-bearing dimensions:

- typed Telegram native identity shape;
- content kind distribution;
- content text presence or empty/non-empty shape, not text itself;
- media metadata presence and media kind;
- reply and thread metadata presence;
- reaction count presence and aggregate counts;
- duplicate identity counts;
- item availability/readiness state if relevant.

The comparison should report mismatches as aggregate categories with counts and
sample local ids only when those ids are safe to paste into docs. It must not
emit message bodies or raw payloads.

### Duplicate Observation Summary

Summarize repeated import behavior:

- number of inserted observations;
- number of duplicate observations;
- number of skipped observations;
- number of failed observations;
- duplicate grouping by source-local typed identity shape;
- whether repeated Takeout after normal sync and repeated Takeout after
  previous Takeout behave as expected.

### Warning Visibility Check

Summarize whether expected warning codes remain visible through durable
provenance and current read-only recovery state:

- `only_my_messages_fallback`;
- `export_dc_fallback`;
- `migrated_history_deferred`;
- `finish_takeout_failed`, when relevant.

The check should verify codes and counts only. It should not expose warning
message bodies.

## Manual Validation Matrix

Create a reusable verification document at:

```text
docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md
```

The document should use status values:

- `not run`;
- `passed`;
- `failed`;
- `blocked`;
- `needs follow-up`.

It should include these cases:

| Case | Required evidence |
| --- | --- |
| Public channel Takeout | before/after source summary, Takeout batch summary, duplicate summary, warnings |
| Public supergroup Takeout | before/after source summary, Takeout batch summary, topic/reply/thread aggregate shape, warnings |
| Private or dialog-backed supergroup Takeout | before/after source summary, fallback/warning evidence if applicable |
| Small group Takeout | source subtype and peer-kind shape, before/after source summary, batch summary |
| Repeated Takeout after normal sync | row-fidelity comparison, duplicate observation summary |
| Repeated Takeout after previous Takeout | duplicate observation summary and latest batch summary |
| `CHANNEL_PRIVATE` fallback | `only_my_messages_fallback` warning code, partial/incomplete evidence, no hidden RPC error |
| Shifted export DC fallback | export DC attempted/fallback flags, `export_dc_fallback` warning code, no hidden Telegram RPC error |
| Migrated small-group-to-supergroup smoke | migrated-history detected, deferred warning, partial completeness, no old `chat` history imported |
| Forum-topic decision input | whether Takeout materially changes topic membership/catalog aggregates |

The verification document should include a short safety reminder at the top:
do not paste message text, source titles, usernames, phone numbers, session
data, auth material, headers, cookies, raw TL payloads, or dumps.

## Expected Validation Procedure

For each manual case:

1. Record the app commit and whether the working tree is clean.
2. Record the local source id and coarse source classification.
3. Capture a source snapshot before the run.
4. Run normal sync or Takeout manually through the existing app flow.
5. Capture the relevant source snapshot, batch summary, duplicate summary,
   warning visibility check, and row-fidelity comparison.
6. Paste only sanitized helper output into the verification doc.
7. Mark the row `passed`, `failed`, `blocked`, or `needs follow-up`.

Live provider errors should be recorded as typed/coarse outcomes. They should
not be hidden by diagnostic tooling, but they should also not leak raw provider
payloads.

## Migrated-History Boundary

The migrated-history smoke verifies the existing safe behavior:

- migrated history can be detected;
- `migrated_history_deferred` is recorded;
- the batch can be partial/incomplete;
- old small-group `chat` history remains unimported.

This slice does not enable migrated-history import. Any future enablement must
wait for a separate design that uses this validation evidence.

## Forum-Topic Boundary

The validation doc should collect whether successful Takeout import changes
topic-related aggregates enough to justify refreshing the forum-topic catalog
after finish.

This slice does not decide or implement that behavior. It only produces input
for a later product decision.

## Testing Strategy

Add unit/storage tests for the diagnostic query helpers:

- summaries read only local database state;
- summary output excludes source names, usernames, raw metadata, message text,
  warning messages, and raw payloads;
- warning codes are sorted and deduplicated where applicable;
- row-fidelity comparison is stable and content-free;
- duplicate observation summary counts inserted, duplicate, skipped, and failed
  observations correctly;
- migrated-history and export DC flags appear as booleans/coarse values only.

Tests should use local SQLite fixtures and must not require live Telegram.

## Acceptance

The slice is complete when:

- the design and implementation plan are committed;
- diagnostic helpers are implemented with tests;
- helper output is safe to paste into docs without manual redaction;
- `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
  exists as a reusable validation template;
- the backlog can distinguish shipped validation tooling from live validation
  rows that still need real-account execution;
- no migrated-history import, recovery action, forum-topic behavior, UI polish,
  or media-download behavior has changed.

## Self-Review Checklist

- No live provider boundary crossed: diagnostic helpers operate only on
  `source_id`, `batch_id`, explicit local ids, and local SQLite state.
- Evidence is safe to paste into docs: helper output requires no manual secret,
  username, phone, title, message-text, payload, cookie, header, or dump
  redaction.
- Comparison shape is stable, not content-bearing: row fidelity compares
  presence, kind, nullability, counts, and identity shape instead of content.
- Recovery semantics unchanged: findings are documented and do not trigger
  retry, resume, purge, discard, or automatic behavior.
- Migrated-history boundary unchanged: the smoke records deferment and does not
  import old small-group history.
- Forum-topic behavior unchanged: validation can inform a later decision but
  does not refresh topic catalog state in this slice.
