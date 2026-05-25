# Takeout Forum-Topic Refresh Policy Design

## Goal

Refresh Telegram forum-topic catalog state after a successfully completed
Takeout import, including completed imports whose durable completeness is
`partial`, while keeping failed and cancelled Takeout attempts side-effect
limited.

This slice decides and implements the policy that was left open by the
representative Takeout validation work: Takeout can materially increase
topic-membership rows while leaving the forum-topic catalog stale.

## Background

Normal Telegram sync already refreshes forum topics before persisting regular
history. The current flow calls `refresh_forum_topics`, which:

- checks typed source identity and runs only for supergroups;
- fetches Telegram forum topics;
- upserts `telegram_forum_topics`;
- marks deleted topics;
- rebuilds materialized `item_topic_memberships`;
- treats non-forum Telegram outcomes such as `CHANNEL_FORUM_MISSING` and
  `CHANNEL_MONOFORUM_UNSUPPORTED` as a no-op.

Takeout import currently persists messages and resolves scoped topic
memberships against the existing local topic catalog. It does not refresh the
catalog when Takeout completes.

Representative validation produced useful decision input:

- source `21` / batch `4` was cancelled / partial and added `8515` topic
  memberships while `0` topic catalog rows were updated or seen during the
  batch window;
- source `22` / batch `11` was cancelled / partial and added `10030` topic
  memberships while the topic catalog aggregate stayed unchanged;
- completed public-supergroup evidence from source `122` had no topic
  membership or topic catalog evidence, so it did not answer this policy
  question.

The partial runs above do not justify refreshing after cancelled jobs. They do
justify a policy for future jobs that reach the normal completed Takeout path.

## Chosen Policy

Refresh forum topics after any `status=completed` Takeout import, including
completed imports with `completeness=partial`.

Do not refresh after:

- cancelled Takeout imports;
- failed Takeout imports;
- interrupted/running recovery states.

The policy intentionally separates "Takeout completed normally" from "Takeout
proved full-history completeness." A completed partial Takeout may be partial
because of access limits, only-my-messages fallback, or deferred migrated
history. Refreshing the topic catalog after that normal completion is safe
because it only synchronizes current forum-topic metadata; it does not claim
that the imported message history is complete.

## Scope

Add a Takeout completion hook that reuses the existing forum-topic refresh
implementation instead of creating a second topic-refresh path.

The hook should run only after Takeout history import has finished and the
Takeout session has been successfully finished. It should execute before
`finalize_ingest_batch` so any durable warning rows are included in the final
batch `warning_count`.

If forum-topic refresh succeeds, the completed Takeout batch remains a normal
completed batch with no extra warning.

If forum-topic refresh fails for an actionable reason, the completed Takeout
batch remains completed, and Extractum records a durable warning code:

```text
forum_topic_refresh_failed
```

The warning message should be static or sanitized and should not expose private
Telegram content. The warning code is the durable contract for validation docs
and UI recovery surfaces.

If Telegram reports that the source is not a forum-capable supergroup, the
refresh remains a no-op with no warning, matching normal sync behavior.

## Non-Goals

- Do not refresh forum topics after failed, cancelled, or interrupted Takeout
  attempts.
- Do not change Takeout import completeness rules.
- Do not enable migrated small-group history import.
- Do not change export-DC fallback behavior.
- Do not change normal sync topic refresh behavior except for sharing helper
  visibility if needed.
- Do not add a frontend control, UI polish, or a manual "refresh topics after
  Takeout" action.
- Do not add a migration unless implementation discovers an unavoidable schema
  need.
- Do not expose message text, source titles, usernames, phone numbers, raw
  Telegram identifiers, raw TL/provider payloads, session material, headers,
  cookies, warning message bodies, or compressed metadata.

## Architecture

Use the current backend ownership boundary:

- forum-topic Telegram calls and SQLite writes stay in Rust;
- frontend receives the existing Takeout job and recovery state surfaces;
- no new Tauri command is needed.

Implementation should make the existing topic refresh reusable by Takeout. The
preferred shape is:

1. Expose a `pub(crate)` topic-refresh function from `sources::topics` through
   `sources::mod` or a narrow internal module path.
2. In the completed Takeout path, call that helper with the existing `pool`,
   authorized `client`, resolved peer, and source.
3. Convert refresh warnings into Takeout warnings and durable provenance using
   the `forum_topic_refresh_failed` code.
4. Keep the completed Takeout path completed even if topic refresh records that
   warning.

The completed-path order should be:

1. import Takeout history;
2. finish the Takeout session successfully;
3. record export-DC fallback provenance if needed;
4. refresh forum topics for eligible supergroups;
5. record any `forum_topic_refresh_failed` warning before finalizing the batch;
6. finalize source sync state;
7. finalize the ingest batch as completed.

If implementation finds that `finalize_sync` must happen before topic refresh
for an existing invariant, the implementation plan must state that invariant
and still preserve durable warning counting before `finalize_ingest_batch`.

## Data Flow

For completed Takeout:

```text
resolved source + authorized client
  -> Takeout import and finish
  -> forum-topic refresh helper
  -> telegram_forum_topics upsert/deleted marks
  -> item_topic_memberships rebuild
  -> optional forum_topic_refresh_failed warning
  -> completed ingest batch
```

For failed or cancelled Takeout:

```text
terminal error or cancel request
  -> no forum-topic refresh
  -> existing failed/cancelled provenance
```

## Error Handling

Forum-topic refresh should not fail an otherwise completed Takeout import.

Non-forum topic errors remain silent no-ops.

Actionable refresh failures should:

- append a user-visible job warning with generic wording;
- record durable warning code `forum_topic_refresh_failed`;
- preserve completed Takeout status;
- avoid raw provider payloads and private content in any persisted message.

The implementation may reuse `record_ingest_batch_warning`, which already
sanitizes provenance text, but the preferred warning message is static product
copy such as:

```text
Forum topic refresh after Takeout failed; existing topic catalog remains available.
```

## Testing

Add Rust tests at the smallest useful seam:

- completed Takeout policy calls the forum-topic refresh hook for supergroup
  sources;
- completed partial Takeout policy also calls the hook;
- cancelled or failed Takeout paths do not call the hook;
- forum-topic refresh failure records `forum_topic_refresh_failed` and does not
  convert the Takeout batch to failed;
- non-forum topic errors remain no-warning no-ops through existing topic
  refresh behavior.

Prefer pure or database-backed unit tests over live Telegram calls. Live
Telegram validation remains a separate manual step.

Run at least:

```powershell
cargo test
npm.cmd test
npm.cmd run check
git diff --check
```

`cargo check` is acceptable during development, but the final implementation
verification should include `cargo test` because this slice changes Rust.

## Validation Docs

After implementation, update:

- `docs/backlog.md`;
- `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
  if a new sanitized local validation note is captured.

The verification matrix row `Forum-topic decision input` should move only when
there is either:

- a code-level test proving the policy for completed Takeout paths and the
  backlog decision is closed; or
- sanitized live evidence from a completed Takeout with topic memberships.

Partial cancelled historical batches remain decision input, not proof of
completed behavior.

## Acceptance Criteria

- Completed Takeout imports refresh forum-topic catalog state for eligible
  supergroup sources.
- Completed partial Takeout imports follow the same refresh policy.
- Failed and cancelled Takeout imports do not refresh forum-topic catalog state.
- Refresh failure records durable warning code `forum_topic_refresh_failed`
  without failing the completed Takeout batch.
- Non-forum topic outcomes remain silent no-ops.
- No private Telegram content or raw provider data is added to docs, tests, or
  durable validation surfaces.
- Existing normal sync topic refresh behavior remains intact.
