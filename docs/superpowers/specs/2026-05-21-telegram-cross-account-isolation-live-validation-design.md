# Telegram Cross Account Isolation Live Validation Design

## Goal

Validate the remaining Telegram 3.1 backlog risk that the same real Telegram
peer can be added and synced under two different accounts without account A
using account B runtime state, session data, source state, or item rows.

The slice name is:

```text
telegram-cross-account-isolation-live-validation
```

## Scope

Use two real Telegram accounts in the local Extractum app:

- account A: account `1`;
- account B: the second local Telegram account that restores to `ready`.

Use exactly one shared public Telegram peer, either a public channel or public
supergroup, that both accounts can access. Prefer an existing account 1 source
with a small expected sync delta, such as a previously validated public source,
then add the same peer under account B through the normal app command path. Do
not delete and re-add existing account 1 fixtures; they are already useful
validation evidence.

This slice does not cover lost-access behavior, migrated small-group behavior,
private source titles/usernames, or account deletion coordination.

## Validation Approach

Run a normal app-path probe against the live app:

1. Start the Tauri app and confirm both account A and account B are `ready`.
2. Select exactly one public channel or supergroup visible to both accounts.
3. If account A already has the selected source, keep that row. If it does not,
   add it through `add_telegram_source`.
4. Add or refresh the same public peer under account B through
   `add_telegram_source`.
5. Read both typed identities from SQLite and record only safe fields:
   `account_id`, `source_id`, `source_type`, `source_subtype`, `peer_kind`,
   `peer_id`, access-hash presence, username presence, `resolution_strategy`,
   `last_sync_state`, and `last_synced_at`.
6. Record pre-sync item counts for both `source_id` values.
7. Run `sync_source(source_id A)` and then re-read source state and item counts
   for both sources.
8. Run `sync_source(source_id B)` and then re-read source state and item counts
   for both sources.
9. Document the sync results, warnings/errors, isolation checks, and any
   limitations.

If the MCP bridge shortcut for app commands does not map arbitrary Tauri
commands, use the webview path:

```text
window.__TAURI__.core.invoke(...)
```

This is still the normal app IPC path.

## Required Pre-Flight Conditions

- The tracked workspace is clean before runtime validation begins.
- No stale Extractum/Tauri process is holding the live database before any
  direct SQLite reads.
- Both account A and account B return runtime status `ready`.
- The selected peer is public and accessible to both accounts.
- Exactly one peer is selected for this slice.
- Account A and account B resolve the selected peer to matching `peer_kind` and
  `peer_id`.
- Pre-sync source fields and item counts are captured for both `source_id`
  values before either sync runs.

Abort the probe and document it as `blocked` if any pre-flight condition fails.

## Pass Criteria

The live validation passes only if all of these checks are true:

```text
source_id A != source_id B
account_id A != account_id B
peer_kind A == peer_kind B
peer_id A == peer_id B
```

And:

```text
sync_source(source_id A) updates only source A state and source A item rows
sync_source(source_id B) updates only source B state and source B item rows
```

For each account, record:

```text
account_id
source_id
source_type = telegram
source_subtype
peer_kind
peer_id
access_hash presence
username presence
resolution_strategy
last_sync_state before/after
inserted/skipped/last_message_id
warnings/errors
```

The item-count isolation check may observe no new messages if the source is
already caught up. In that case, the source-state check still must show that
syncing source A does not mutate source B's `last_sync_state` or item count, and
syncing source B does not mutate source A's `last_sync_state` or item count.

## Abort And Needs-Follow-Up Conditions

Abort as `blocked` before documenting a result if:

- account B is not `ready`;
- the shared public source cannot be added under account B;
- the selected peer is not accessible to both accounts;
- more than one peer is involved in the evidence;
- a sync fails with an auth/runtime error unrelated to cross-account isolation.

Document as `failed` or `needs follow-up`, rather than `passed`, if:

- account A and account B resolve different `peer_kind` or `peer_id` values for
  what was expected to be the same public peer;
- `source_id A` equals `source_id B`;
- either sync mutates the other account's source row or item count;
- warnings suggest that one account used the other account's runtime/session
  state.

## Safety And Privacy

Do not write credentials, phone numbers, Telegram session data, API hashes,
private message content, private titles, or private usernames to tracked docs.

For the selected public peer, record username only as `present` or `absent`, not
as the value. Use a neutral label such as `selected public channel` or
`selected public supergroup` in tracked documentation.

This slice must not mutate Telegram server state beyond normal source add and
sync activity. It must not delete sources, reset sync state, or clear useful
validation fixtures.

## Expected Documentation Changes

Update:

- `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- `docs/backlog.md`

If the probe passes, mark the matrix row `Same source on account A and account
B` as `passed`, add a dated live-run note, and remove the cross-account row
from backlog section `3.1`. Leave the lost-access and migrated-dialog rows
open.

If the probe is blocked or inconclusive, keep the backlog row open and record
the exact blocker without sensitive details.

## Verification

Before committing the validation result, run:

```text
git diff --check
```

If production code changes become necessary, stop this docs-only slice and
switch to a RED test first.
