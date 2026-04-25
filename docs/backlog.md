# Backlog

This backlog tracks known follow-up work after expanding Telegram sources from broadcast channels to Telegram source kinds.

## Recently Completed

### Scope Telegram Source Uniqueness By Account

Status: done.

Why it mattered: the same Telegram channel or group can exist in multiple local Telegram accounts. Source uniqueness is now scoped by `account_id`, so adding the same Telegram source to another account no longer reassigns the existing source row.

Implementation notes:

- `sources` uniqueness is now `(account_id, source_type, telegram_source_kind, external_id)`.
- `add_telegram_source` uses the same conflict target.
- Migration `12.sql` updates the index without rewriting source rows.

### Add Source Dialog UX

Status: done.

Why it mattered: after including groups, the dialog list can become much longer. Users can now search, filter, and scan counts without assuming the Telegram API returned an incomplete list.

Implementation notes:

- The Add Source dialog has local search by title and username.
- Filters cover `All`, `Channels`, `Supergroups`, and `Groups`.
- The dialog shows filtered and total counts.
- Dialog rows are sorted by title, then by source kind.
- Already-added detection is scoped by account, kind, and external id.
- Loading, not-ready, not-loaded, and no-match states are distinct.

### Documentation Refresh

Status: done.

Why it mattered: project docs still described source management as broadcast-channel-only after the code moved to Telegram source kinds.

Implementation notes:

- Project, design, database, README, architecture, and agent context docs now describe Telegram sources.
- Command names now use `list_telegram_sources` and `sync_source`.
- Database docs include `telegram_source_kind` and uniqueness by `(account_id, source_type, telegram_source_kind, external_id)`.
- Migration history includes versions 11 and 12.

### Peer Identity Metadata Foundation

Status: partial.

Why it mattered: private channels and supergroups can require Telegram peer identity beyond username and bare id. The source metadata format can now preserve identity details without changing the table schema.

Implementation notes:

- `SourceMetadata` remains backward-compatible with old username-only payloads.
- New metadata can record whether a source was added from dialogs or username flow.
- New metadata can record `access_hash` for channel-backed sources when `grammers` exposes it.
- Sync can resolve channel and supergroup sources from stored `access_hash` before falling back to dialog scanning.
- Small groups still rely on dialog/session resolution because they do not use channel `access_hash`.

## Telegram Runtime Validation

Priority: high.

Goal: verify the new `telegram_source_kind` model against real Telegram accounts and real dialogs.

Why it matters: `cargo check` and `svelte-check` confirm compile-time safety, but the meaningful risk is in Telegram runtime behavior. `grammers` can expose broadcast channels, supergroups, small groups, forbidden/min peers, and migrated groups with slightly different raw shapes.

Scope:

- Verify that `list_telegram_sources` returns broadcast channels.
- Verify that `list_telegram_sources` returns supergroups.
- Verify that `list_telegram_sources` returns regular small groups if the account has any.
- Verify that profile pictures load for channels and groups.
- Verify that adding from the dialog list stores the expected `telegram_source_kind`.
- Verify that manual add by `@username` works for public channels and public groups.
- Verify that sync works for `channel`, `supergroup`, and `group`.
- Verify behavior when the user is no longer a member of a group/channel.
- Verify behavior for migrated small group to supergroup dialogs.

Acceptance criteria:

- The Add Source dialog shows channels, supergroups, and groups with correct labels.
- A source added from account A does not affect the same source added from account B.
- Sync inserts messages for each supported kind without resolving to the wrong peer.
- Any unsupported or inaccessible Telegram peer produces a friendly typed error instead of a silent failure.

Risks:

- Private sources without username may only be resolvable while they are present in dialogs.
- Telegram bare ids can be ambiguous without source kind, so adding from dialog must keep passing `telegramSourceKind`.
- Some historical/migrated group states may need extra metadata beyond bare id and username.

## Persist And Show Source Avatars

Priority: medium.

Goal: show source profile pictures not only in the Add Source dialog, but also in the Added Sources list and source detail header.

Why it matters: avatars make it easier to recognize Telegram sources, especially when channels and groups have similar names.

Scope:

- Decide where to store avatar data or avatar references.
- Prefer storing compact metadata over large base64 blobs directly in `sources`.
- Add a cache strategy for downloaded profile photos.
- Update `SourceRecord` to expose an optional avatar URL or data URL.
- Update `SourceRow` to render avatar with the same fallback initial used in the Add Source dialog.
- Update source detail header to show the avatar next to the title.
- Refresh avatar when loading dialogs or syncing a source.

Acceptance criteria:

- Added sources show an avatar when one is available.
- Sources without avatar show a stable fallback initial.
- Avatar loading failure does not break source listing or sync.
- The storage approach does not noticeably bloat SQLite for large source lists.

Risks:

- Base64 image data in SQLite can grow quickly.
- Telegram photo references may expire or require the account session to fetch.
- Avatar refresh should not block core sync work.

## Private Sources And Peer Identity

Status: partial.

Priority: high.

Goal: make private Telegram channels/groups predictable by storing enough peer identity to resolve them without relying only on username or dialog scanning.

Why it matters: public sources can be resolved by username, but private sources often cannot. Bare id plus kind helps, but Telegram access usually needs session peer cache or access hash.

Scope:

- Extend `SourceMetadata` to store peer identity data when available.
- Consider storing `access_hash` for channels and supergroups.
- Consider storing whether the source was added from dialog or by username.
- Keep manual numeric add constrained to dialogs unless metadata is sufficient.
- Improve error messages for private sources that disappeared from dialogs.
- Add migration/backward compatibility for existing metadata.
- Document which Telegram source refs are supported: `@username`, `t.me/name`, and dialog-picked private source.

Acceptance criteria:

- Private sources added from dialogs continue syncing if Telegram session cache can resolve them.
- If a private source cannot be resolved, the app explains the likely reason and suggests re-adding from dialogs.
- Public username sources still sync through username resolution.
- Existing sources with old metadata continue to work through fallback dialog scanning.

Risks:

- Access hash handling may depend on `grammers` raw peer variants and can change across library updates.
- Storing more Telegram identity data improves reliability but increases responsibility for local sensitive metadata.
- Some private or forbidden peers may remain impossible to sync after the account loses access.
