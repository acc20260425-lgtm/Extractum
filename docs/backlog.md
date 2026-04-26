# Backlog

This backlog tracks active follow-up work only. Completed implementation history belongs in release notes, commit history, or the related design/database documents.

## Telegram Runtime Validation

Status: open.

Priority: high.

Goal: verify the current `telegram_source_kind` model against real Telegram accounts and real dialogs.

Why it matters: compile-time checks cannot cover Telegram peer shapes. `grammers` can expose broadcast channels, supergroups, small groups, forbidden/min peers, migrated groups, and private peers with subtly different identity data.

Scope:

- Verify that `list_telegram_sources` returns broadcast channels, supergroups, and regular small groups.
- Verify that source avatars load for channels and groups.
- Verify that adding from the dialog list stores the expected `telegram_source_kind`.
- Verify that manual add by `@username` works for public channels and public groups.
- Verify that sync works for `channel`, `supergroup`, and `group`.
- Verify behavior when the user is no longer a member of a group or channel.
- Verify behavior for migrated small-group-to-supergroup dialogs.

Acceptance criteria:

- The Add Source dialog shows channels, supergroups, and groups with correct labels.
- A source added from account A does not affect the same source added from account B.
- Sync inserts messages for each supported kind without resolving to the wrong peer.
- Unsupported or inaccessible Telegram peers produce friendly typed errors.

## Private Sources And Peer Identity

Status: partial.

Priority: high.

Goal: make private Telegram channels and groups predictable by storing enough peer identity to resolve them without relying only on username or dialog scanning.

Why it matters: public sources can be resolved by username, but private sources often cannot. Bare id plus kind helps, but Telegram access may need session peer cache, access hash, or dialog-derived identity.

Scope:

- Audit current `SourceMetadata` coverage for dialog-picked private sources.
- Store peer identity data when `grammers` exposes it.
- Keep manual numeric add constrained to dialogs unless metadata is sufficient.
- Improve errors for private sources that disappeared from dialogs.
- Document supported Telegram source refs: `@username`, `t.me/name`, and dialog-picked private source.

Acceptance criteria:

- Private sources added from dialogs continue syncing when Telegram session data can resolve them.
- If a private source cannot be resolved, the app explains the likely reason and suggests re-adding from dialogs.
- Public username sources still sync through username resolution.
- Existing sources with older metadata continue to work through fallback dialog scanning.

## Secure Secret Storage

Status: open.

Priority: high.

Goal: move sensitive credentials out of SQLite-backed `app_settings`.

Why it matters: LLM API keys and Telegram credentials are currently easy to inspect in the local database. That is acceptable only as development debt.

Scope:

- Move LLM API keys to a secure store appropriate for Tauri desktop apps.
- Review Telegram `api_hash` and session storage responsibilities.
- Keep secrets profile/account scoped.
- Preserve existing settings through a migration or one-time import path.
- Avoid logging secrets in backend errors, frontend status text, or debug output.

Acceptance criteria:

- New LLM provider keys are not persisted in plain SQLite.
- Existing configured keys can be migrated or re-entered without breaking the app.
- `/settings` can still edit provider settings without exposing secrets unnecessarily.

## LLM Provider Configuration

Status: open.

Priority: high.

Goal: turn Gemini and OmniRoute support into a provider configuration model that can grow beyond the current hard-coded default profile.

Why it matters: the backend now has a modular LLM implementation, but the product still exposes only one active profile and hard-codes OmniRoute's OpenAI-compatible `base_url`.

Scope:

- Add provider profile management beyond the single `default` profile.
- Decide whether `base_url` should be stored for OpenAI-compatible providers and exposed in `/settings`.
- Validate model list and Test Provider flows for Gemini and OmniRoute.
- Make provider labels, placeholders, and error messages provider-neutral where appropriate.
- Update analysis run metadata if user-facing provider profile names become editable.

Acceptance criteria:

- Users can configure Gemini and OmniRoute without code changes.
- OpenAI-compatible providers can reuse the same backend path with a configured `base_url`.
- Test Provider always uses the saved provider/model/key the user sees in settings.

## LLM Parallel Request Support

Status: planned.

Priority: medium.

Goal: support multiple LLM requests running at the same time without mixing stream state, progress state, or UI output.

Why it matters: analysis map chunks, report reduction, follow-up chat, and provider smoke tests can all need request-scoped lifecycle handling. The refactored LLM runner is ready for this, but no concurrency policy exists yet.

Scope:

- Define concurrency limits per provider/profile.
- Add active request tracking keyed by `request_id`.
- Add cancellation support for long-running requests.
- Keep stream buffers, usage, timeout, and callbacks request-local.
- Decide how the frontend should display multiple active streams.
- Ensure analysis progress and provider test output ignore unrelated request events.

Acceptance criteria:

- Concurrent LLM requests cannot overwrite each other's output.
- A user can cancel a long-running request.
- Provider and analysis events remain traceable by `request_id`.

## Saved Runs Discoverability

Status: open.

Priority: medium.

Goal: make previous analysis runs easy to find even when the current analysis scope changes.

Why it matters: the current `Saved Runs` panel is scoped to the selected source or source group. That can make older runs look missing when the user switches scope or opens Analysis without the original target selected.

Scope:

- Decide whether Saved Runs should default to global history or scoped history.
- Add explicit scope filters if both behaviors are useful.
- Preserve the ability to open completed runs regardless of current composer scope.
- Consider search/filter by source, source group, provider, model, template, status, and date.
- Keep active-run restoration separate from historical run browsing.

Acceptance criteria:

- Users can find previous saved runs without reconstructing the original source/group selection.
- Scoped filtering remains available when useful.
- Running/queued runs remain visually distinct from completed/failed history.

## Media Download And Preview

Status: open.

Priority: medium.

Goal: extend media-aware ingest from metadata-only storage to optional binary media download and preview.

Why it matters: `/sources` already preserves media metadata, but users cannot inspect the actual files from the local archive.

Scope:

- Decide storage layout for downloaded media files.
- Add download policy controls so media does not unexpectedly consume disk.
- Render safe previews for common media types.
- Preserve existing metadata-only behavior as the default or fallback.
- Handle missing/deleted Telegram media gracefully.

Acceptance criteria:

- Users can opt into downloading media for selected sources or items.
- Downloaded media is stored outside SQLite with stable metadata references.
- `/sources` can preview common downloaded media types.

## Media-Aware Analysis

Status: open.

Priority: medium.

Goal: let analysis workflows account for media-bearing and media-only items in a controlled way.

Why it matters: current analysis is text-first. Media-only posts are visible in `/sources` but excluded from the analysis corpus, which can hide important evidence.

Scope:

- Define how media metadata should appear in text-only prompts.
- Decide whether downloaded media can be sent to multimodal providers.
- Add citation semantics for media evidence.
- Update trace resolution and report viewer to handle media refs.
- Keep text-only analysis available for providers without multimodal support.

Acceptance criteria:

- Reports can mention relevant media metadata with clear citations.
- Media-only items do not silently disappear when the selected analysis mode supports them.
- Non-multimodal providers degrade predictably.

## Documentation Refresh

Status: open.

Priority: low.

Goal: align project docs with the current LLM and settings implementation.

Why it matters: several docs still describe the LLM flow as Gemini-only, while the app now supports Gemini and OmniRoute through a modular backend.

Scope:

- Update `README.md`, `docs/project.md`, `docs/design-document.md`, `docs/database-schema.md`, and `docs/architecture-deep-dive.md`.
- Replace Gemini-only language with provider-neutral language where appropriate.
- Document OmniRoute's OpenAI-compatible path and current hard-coded `base_url` limitation.
- Keep the secure-storage warning current.

Acceptance criteria:

- New contributors can understand the current Gemini/OmniRoute provider flow from docs.
- The docs no longer list completed LLM refactor work as future work.
