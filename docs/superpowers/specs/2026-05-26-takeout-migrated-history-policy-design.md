# Takeout Migrated-History Policy Design

## Goal

Decide the product and technical policy for Telegram small-group history that
Telegram exposes after a group has migrated into a supergroup.

The chosen policy is: migrated small-group history is a separate historical
scope. It must not be silently merged into the current supergroup history by a
normal Takeout rerun.

This design records the policy boundary and the implementation constraints for
future work. It does not enable migrated-history import.

## Current State

Extractum already stores Telegram message identity by native history peer:

- `telegram_messages.history_peer_kind`;
- `telegram_messages.history_peer_id`;
- `telegram_messages.telegram_message_id`;
- `telegram_messages.migration_domain`;
- `telegram_messages.is_migrated_history`.

The storage layer can represent overlapping Telegram message ids from different
history peers. Current supergroup rows use the `channel` history peer. Old
small-group rows, if imported in the future, would use the old `chat` history
peer and must remain distinguishable.

The current Takeout supergroup path detects migration with
`channels.getFullChannel`. When `migrated_from_chat_id` is present, it records:

- `migrated_history_detected = 1`;
- `migrated_history_imported = 0`;
- warning code `migrated_history_deferred`;
- `history_scope = current_history_with_migrated_deferred`, or `mixed_partial`
  when another partial-history condition also applies.

Source `115` / batch `18` validated the current safe behavior: migrated history
was detected and deferred, no old `chat` rows were imported, and the batch was
classified as completed / partial.

## Chosen Approach

Use a "historical scope, explicit opt-in" policy.

The current supergroup source keeps its current history as the default history
view. Migrated small-group history remains visible as a detected historical
scope, but it is not imported by the normal Takeout path and not retried
automatically by recovery copy.

Future migrated-history import, if implemented, must be a separate explicit
action or mode. It should be presented and tested as importing a historical
scope, not as completing the current supergroup history.

## Alternatives Considered

### Strict Defer-Only

Keep the existing detect-and-defer behavior as the permanent product policy.

This is the smallest and safest runtime choice, but it prevents users from ever
bringing in old small-group history through Extractum, even when they explicitly
want that context.

### Separate Synthetic Source

Create a separate local source for the old small-group history.

This gives the strongest `source_id` boundary, but it introduces harder product
questions around source naming, exports, analysis grouping, duplicates, and how
to explain that the historical source is related to the current supergroup.

### Historical Scope, Explicit Opt-In

Keep one current supergroup source, treat old small-group history as a separate
historical scope, and require an explicit future action before importing it.

This is the recommended option because it preserves today's safety boundary
while leaving a clear path for later import with durable provenance.

## Policy Contract

Normal Takeout import for a migrated supergroup:

- imports current supergroup history only;
- may detect old small-group history;
- records deferment provenance and warning code;
- completes as partial when migrated history is detected;
- does not set `migrated_history_imported = 1`;
- does not import old `chat` history rows;
- does not imply that a normal rerun will automatically import the historical
  scope.

Future historical-scope import:

- must be opt-in or otherwise explicitly requested;
- must use a separate code path or mode from normal current-history Takeout;
- must keep old rows under their native `chat` history peer;
- must mark rows with `is_migrated_history = 1`;
- must set a stable `migration_domain`, such as `migrated_from_chat`;
- must update provenance only after actual historical-scope import work;
- must not rewrite old `chat` identity into current `channel` identity.

## Data Flow

Current Takeout flow remains:

```text
resolve current supergroup peer
  -> validate current peer
  -> detect migrated_from_chat_id
  -> record migrated_history_deferred when detected
  -> select current-history split
  -> import current-history messages
  -> finalize source state and batch
```

Recovery and diagnostics should describe the deferment as a separate historical
scope, not as an ordinary transient failure.

A future opt-in historical importer would use a distinct flow:

```text
resolve current supergroup peer
  -> confirm migrated historical scope is available
  -> explicitly select historical migrated scope
  -> import old chat-history rows with migrated markers
  -> record historical-scope provenance
  -> leave current-history provenance distinguishable
```

## Error Handling

`migrated_history_deferred` is not an application crash, data corruption, or
provider failure. It is a durable signal that Extractum intentionally did not
cross the source's current-history boundary.

The following outcomes are data-integrity failures:

- normal current-history Takeout imports `history_peer_kind = chat` rows for a
  migrated supergroup;
- normal current-history Takeout sets `migrated_history_imported = 1`;
- imported historical rows lose their native history peer;
- migrated rows become indistinguishable from current supergroup rows.

The following outcomes are acceptable under this policy:

- completed / partial Takeout with `migrated_history_deferred`;
- warning-code-only recovery state explaining the deferment;
- repeated normal Takeout reruns that keep the historical scope deferred.

## User-Facing Recovery Semantics

Recovery copy should avoid implying that a safe rerun will make the migrated
source fully complete. A rerun may refresh or re-import current history, but it
must not automatically import the old small-group historical scope.

The safe user-facing interpretation is:

```text
Current supergroup history was imported. Older small-group history was detected
as a separate historical scope and is not imported by normal reruns.
```

UI and docs should continue to use warning codes and aggregate state. They must
not expose Telegram private content, source titles, usernames, raw peer ids,
access hashes, warning bodies, or raw TL/provider payloads.

## Tests And Validation

The first implementation slice after this design should focus on policy
enforcement and documentation, not enabling migrated-history import.

Recommended checks:

- a Rust provenance regression showing deferred migrated history remains
  `migrated_history_imported = 0`, records one
  `migrated_history_deferred` warning code, and finalizes as partial;
- a Rust storage regression showing overlapping Telegram message ids are
  separated by native peer identity, not by `items.external_id` alone;
- a diagnostic or recovery-copy test showing migrated-history deferment is
  explained as a separate historical scope and not as automatic rerun work;
- docs updates in `docs/backlog.md`, `docs/takeout-source-import.md`, and the
  representative validation matrix that record the selected policy.

If a later implementation enables opt-in historical import, it needs its own
design and tests for:

- explicit opt-in behavior;
- duplicate handling across current and historical scopes;
- cancellation after historical observations;
- partial historical import provenance;
- export and analysis filtering or labeling;
- sanitized diagnostics for historical-scope rows.

## Non-Goals

- Do not enable migrated small-group history import in this slice.
- Do not add a UI button or command for historical-scope import yet.
- Do not add schema changes unless a later implementation proves they are
  necessary.
- Do not clear legacy Telegram metadata blobs as part of this policy decision.
- Do not change normal Telegram sync behavior.
- Do not run live Telegram validation for this policy-only design.

## Privacy Boundary

Tracked docs and diagnostics may mention:

- local source ids and batch ids;
- source subtype and peer kind;
- boolean migrated-history flags;
- warning codes;
- status, completeness, history scope, and aggregate counters.

Tracked docs and diagnostics must not include:

- message text;
- source titles;
- usernames;
- phone numbers;
- account labels identifying people or sources;
- session/auth material;
- raw access hashes;
- raw TL/provider payloads;
- warning message bodies.

## Acceptance

- The project policy says migrated small-group history is a separate historical
  scope.
- Normal Takeout reruns remain current-history imports and do not import old
  `chat` rows.
- `migrated_history_deferred` is documented as intentional deferment, not an
  auto-retry promise.
- Future migrated-history import is explicitly scoped as separate opt-in work.
- Privacy and sanitized diagnostics boundaries remain unchanged.
