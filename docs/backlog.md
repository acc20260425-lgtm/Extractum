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

## Add Source Dialog UX

Priority: medium.

Goal: make the Add Source dialog usable when an account has many Telegram dialogs.

Why it matters: after including groups, the dialog list can become much longer. Without search and filters, users can miss sources or assume the list is incomplete.

Scope:

- Add a local search field for title and username.
- Add filters for `All`, `Channels`, `Supergroups`, and `Groups`.
- Add visible counters for total sources and filtered sources.
- Add a loading state that explains that profile pictures may make first load slower.
- Sort sources by title, with a secondary sort by kind.
- Keep already-added detection scoped to account, kind, and external id.
- Preserve keyboard accessibility for search, filter buttons, and Add actions.

Acceptance criteria:

- Users can find a source by typing part of its title or username.
- Users can filter the list to only groups or only channels.
- Empty states distinguish between "not loaded", "no matches", and "account not ready".
- UI remains usable on smaller desktop widths.

Risks:

- Downloading many avatars during list load can make filtering feel slow unless the list render stays lightweight.
- If filters are added before runtime validation, users may misinterpret Telegram API gaps as app filtering bugs.

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

## Documentation Refresh

Priority: low.

Goal: update project docs so they match the new Telegram source model.

Why it matters: current docs still describe source management as broadcast-channel-only in a few places.

Scope:

- Update `docs/project.md` command names from `list_telegram_channels` and `sync_channel` to `list_telegram_sources` and `sync_source`.
- Update `docs/design-document.md` source model from "Telegram channel" to "Telegram source".
- Update `docs/database-schema.md` to include `telegram_source_kind`.
- Update source uniqueness docs to mention `(account_id, source_type, telegram_source_kind, external_id)`.
- Add migration history entries for versions 11 and 12.

Acceptance criteria:

- Docs no longer imply that only broadcast channels are supported.
- Database docs match the current migrations.
- A new contributor can understand channels, groups, and supergroups without reading the full Rust implementation first.
