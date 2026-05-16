# Extractum Backlog

> **Updated:** 2026-05-16
> **Rule:** this file tracks open work only. Shipped work belongs in current-state docs and Git history.

## 1. Open Gaps

- Telegram runtime behavior needs broader validation against real accounts, dialogs, private channels/supergroups, small groups, and migrated dialogs.
- Account deletion still needs coordination with active source sync, Takeout import, source deletion, and analysis work.
- Takeout source import needs broader live validation, incomplete-batch provenance, and a migrated-history identity decision.
- Database schema simplification needs a source/item identity cleanup plan before more provider expansion.
- YouTube live-provider coverage needs auto-caption-only, no-caption, active live, upcoming, auth-gated, private/member/age/geo, and large-playlist validation.
- Large saved-run archives need richer narrowing by source, group, profile/model, template, and date.
- Media support is metadata-first only; binary download, preview, and media-aware analysis remain open.
- Analysis workspace parity still needs run-open access to NotebookLM export, prompt template management, and source group management.
- NotebookLM export follow-ups remain open for optional link enrichment, source-group export, forward metadata, and richer forum-topic grouping.
- YouTube-specific NotebookLM export enrichment remains open.
- Full Telegram Forum Topics browsing/export and forward metadata are not modeled yet.
- Stabilization needs repeatable full-project verification, CI, and a dependency pinning policy for `grammers`.
- Logs and user-facing error surfaces need a focused secret-leak audit.

## 2. Planning Principles

1. Keep architecture pragmatic and local to the current codebase.
2. Prioritize correctness, data integrity, and operability risks before UI polish.
3. Validate Telegram behavior against real data when static reading is insufficient.
4. Prefer tests for pure logic, storage rules, request lifecycle boundaries, and route workflow regressions.
5. Keep closed Superpowers plans/specs out of active docs; preserve them through Git history instead.

## 3. Active Work Areas

| Area | Open target |
| --- | --- |
| Telegram runtime validation | predictable behavior across real supported dialogs and accounts |
| Account deletion coordination | deletion cannot race active ingest or analysis work |
| Takeout source import | validated across source kinds with explicit incomplete-import provenance |
| Database schema simplification | canonical source identity, provider-native item identity, and current-schema baseline |
| YouTube source ingest | broader live validation plus optional future enrichment/resumability |
| Saved runs UX | fast narrowing for large saved-run histories |
| Analysis workspace parity | run-open NotebookLM export plus template and source-group management access |
| Media support | optional download/preview and controlled media-aware analysis |
| NotebookLM export | optional enrichment and source-group export if needed |
| Stabilization | repeatable baseline plus dependency upgrade policy |
| Secret safety | no accidental secret exposure in logs, status text, or debug output |

## 4. Open Roadmap

### 4.1 Telegram Runtime And Private-Source Validation

Priority: high.

- [ ] verify that `list_telegram_sources` returns broadcast channels, supergroups, and regular small groups
- [ ] verify that adding from the dialog list stores the expected `source_subtype` and peer identity metadata
- [ ] verify that sync works for `channel`, `supergroup`, and `group`
- [ ] verify behavior when the user is no longer a member of a group or channel
- [ ] verify behavior for migrated small-group-to-supergroup dialogs
- [ ] validate dialog-picked private `channel` and `supergroup` sources through the stored-identity path
- [ ] validate cross-account isolation on two real Telegram accounts

Acceptance:

- Add Source shows channels, supergroups, and groups with correct labels.
- A source added from account A does not affect the same source added from account B.
- Sync inserts messages for each supported kind without resolving to the wrong peer.
- Private dialog-picked sources resolve predictably when Telegram provides sufficient peer data.

### 4.2 Account Deletion Coordination

Priority: high.

- [ ] reject or cancel account deletion when any owned source has active sync, Takeout import, or delete work
- [ ] decide whether account deletion should cancel owned analysis/LLM work or block until it finishes
- [ ] return `not_found` when deleting a missing account
- [ ] add backend tests for missing-account deletion and account deletion with active source work

Acceptance:

- Account deletion cannot cascade-delete source/item rows underneath active ingest tasks.
- Runtime and secure-storage cleanup still happens after a valid delete.
- Missing account deletion reports a typed `not_found` error.

### 4.3 Takeout Source Import Follow-Ups

Priority: high.

- [ ] validate Takeout import on representative public channels, supergroups, and small groups
- [ ] validate `CHANNEL_PRIVATE` fallback on a private/left channel or supergroup
- [ ] validate shifted export DC behavior and the warning path when fallback to home DC is used
- [ ] compare Takeout-imported rows with normal sync rows for content, media metadata, reply/thread metadata, reaction counts, and duplicate skipping
- [ ] decide and implement incomplete-import provenance, such as `ingest_batches`, item batch ids, or staging/promotion
- [ ] decide how to handle migrated small-group history without corrupting `(source_id, external_id)` uniqueness
- [ ] decide whether Takeout import should refresh the forum-topic catalog after successful finish

Acceptance:

- Successful Takeout import updates `last_sync_state` and `last_synced_at`.
- Failed or cancelled Takeout imports are distinguishable from complete history.
- Export DC fallback and only-my-messages fallback warnings remain visible in job state.
- Migrated supergroup history has a safe identity policy before import is enabled.

### 4.4 Database Schema Simplification

Priority: high.

Analysis:

- Full findings are recorded in `docs/database-schema-legacy-analysis.md`.

- [ ] move remaining Telegram display/avatar metadata out of `sources.metadata_zstd`
- [ ] move YouTube identity/display metadata to typed source tables
- [ ] continue item/document identity cleanup

Acceptance:

- New provider work does not need to touch legacy Telegram subtype compatibility.
- Migrated Telegram history has a safe duplicate-detection model.
- Analysis and NotebookLM export read stable document rows without provider-specific item-table branching for normal cases.
- Fresh installs start from a clean current schema while existing databases still upgrade safely.

### 4.5 Saved Runs Discoverability And Cleanup

Priority: medium.

- [ ] add historical search/filtering by source, source group, provider, profile, model, template, and date

Acceptance:

- Large saved-run histories can be narrowed quickly without reconstructing the original run context.

### 4.6 Analysis Workspace Parity

Priority: high.

- [ ] keep `Export for NotebookLM` reachable when an analysis run is open
- [ ] keep prompt template management reachable when an analysis run is open
- [ ] keep source group management reachable when an analysis run is open

Acceptance:

- Opening a current or saved analysis run does not hide NotebookLM export.
- Opening a current or saved analysis run does not hide prompt template or source group management.
- The setup/no-run path keeps the same management actions it has today.

### 4.7 NotebookLM Export Follow-Ups

Priority: medium.

- [ ] add optional link enrichment with explicit user opt-in and cache
- [ ] add source-group export if the analysis group workflow needs it
- [ ] render forward context after sync persists forward metadata
- [ ] decide whether export needs full Forum Topics names/grouping beyond stored `reply_to_top_id`
- [ ] consider saved-analysis-snapshot export based on `analysis_run_messages`

### 4.8 YouTube Source Follow-Ups

Priority: medium.

- [ ] add YouTube-specific NotebookLM export enrichment with transcript segment timestamps, canonical video links, and playlist membership metadata in export output
- [ ] add speech-to-text fallback for videos without captions
- [ ] add live chat ingest
- [ ] support media-aware analysis over thumbnails or downloaded media if a future setting explicitly allows media downloads
- [ ] make YouTube source jobs persistent/resumable across app restart
- [ ] broaden manual/live validation for auto-caption-only, no-caption, active live, upcoming, private/member/age/geo-gated, and large playlist sources

Acceptance:

- Future YouTube export enhancements do not regress the existing generic NotebookLM export.
- No media download or speech-to-text path runs without explicit user opt-in.
- Restarted apps can explain or resume interrupted YouTube work according to the selected future policy.

### 4.9 Media Download, Preview, And Analysis

Priority: medium.

- [ ] decide storage layout for downloaded media files
- [ ] add download policy controls so media does not unexpectedly consume disk
- [ ] render safe previews for common media types
- [ ] define how media metadata should appear in text-only prompts
- [ ] decide whether downloaded media can be sent to multimodal providers
- [ ] add citation semantics for media evidence
- [ ] keep text-only analysis available for providers without multimodal support

Acceptance:

- Users can opt into downloading media for selected sources or items.
- Downloaded media is stored outside SQLite with stable metadata references.
- Reports can mention relevant media metadata with clear citations when the selected analysis mode supports it.

### 4.10 Stabilization

Priority: medium.

- [ ] add a single documented full-project verification command or script
- [ ] add CI for frontend tests, Svelte check, Rust tests, Rust lint, formatting, and `git diff --check`
- [ ] pin `grammers-*` dependencies to an explicit `rev` or owned release policy
- [ ] verify Telegram and LLM event-driven UI flows after the next major backend changes
- [ ] audit backend errors, frontend status text, and debug output for accidental credential exposure

## 5. Explicit Non-Goals

| Idea | Decision | Why |
| --- | --- | --- |
| Hexagonal architecture rewrite | do not do | too heavy for current scale |
| Telegram trait abstraction mainly for tests | do not do | too much indirection for weak payoff |
| Service-heavy frontend architecture | do not do | poor fit for this Svelte app |
| E2E-first expansion before core stabilization | do not do | lower ROI than targeted storage and logic tests right now |
| Splitting every large file automatically | do not do | only split where it lowers risk or unlocks backlog work |

## 6. Execution Priority

1. Validate remaining Telegram runtime/private-source cases on real accounts and dialogs.
2. Close account-deletion coordination before more long-running ingest expansion.
3. Validate Takeout import across representative source kinds and decide incomplete-import provenance.
4. Design the database schema simplification path before adding more provider surface.
5. Decide whether saved-run history needs richer filters before media expansion.
6. Restore run-open `/analysis` access to NotebookLM export, prompt templates, and source groups.
7. Broaden YouTube live-provider validation and decide which follow-ups matter after the MVP.
8. Continue media download/preview and media-aware analysis design.
9. Tighten verification, CI, and dependency pinning.
