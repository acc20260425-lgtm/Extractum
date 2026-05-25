# Extractum Backlog

> **Updated:** 2026-05-25
> **Rule:** this file tracks open work only. Shipped work belongs in current-state docs and Git history.

## 1. Priority Snapshot

| Priority | Area | Next outcome |
| --- | --- | --- |
| High | Takeout source import | decide migrated-history import policy on top of persisted provenance |
| High | Database schema simplification | decide whether old Telegram metadata blobs can be cleared after typed repair and real-data validation |
| Medium | Saved runs discoverability | add useful narrowing for large saved-run histories |
| Medium | NotebookLM export follow-ups | decide on optional link enrichment, source-group export, forward metadata, and richer topic grouping |
| Medium | YouTube source follow-ups | broaden live-provider validation and decide which enrichment/resumability features matter after the MVP |
| Medium | Telegram topic/forward enrichment | model richer Forum Topics browsing/export and forward metadata when needed |
| Medium | Media support | move beyond metadata-first storage only after explicit download and analysis policies exist |
| Medium | Stabilization and secret safety | add repeatable verification, CI, dependency policy, and secret-leak audit coverage |

## 2. Planning Principles

1. Keep architecture pragmatic and local to the current codebase.
2. Prioritize correctness, data integrity, and operability risks before UI polish.
3. Validate Telegram behavior against real data when static reading is insufficient.
4. Prefer tests for pure logic, storage rules, request lifecycle boundaries, and route workflow regressions.
5. Keep closed Superpowers plans out of active docs; preserve them through Git
   history. Move historical specs or verification notes into
   `docs/superpowers/archive/` only when they remain useful as context.

## 3. Open Roadmap

### 3.1 Takeout Source Import Follow-Ups

Priority: high.

- [x] ship repeatable sanitized Takeout validation diagnostics and reusable
  manual validation template
- [x] complete representative public supergroup Takeout validation after
  current durable baseline and bounded partial runs
  - Source `18` completed public-channel Takeout as batch `10` with explicit
    before/after snapshots, no warnings, and complete duplicate/fidelity
    evidence against the normal-sync baseline.
  - Source `22` public-supergroup Takeout batch `11` was bounded-cancelled as
    `partial` after `11162` observations because the estimate was `125813`;
    all observed identities matched canonical source rows, but completed
    public-supergroup evidence is still needed.
  - Source `19` public-supergroup Takeout batch `12` was bounded-cancelled as
    `partial` after `20397` observations because the estimate was `52723`;
    all observed identities matched canonical source rows, but completed
    public-supergroup evidence is still needed.
  - Source `122` completed public-supergroup Takeout as batch `13` with
    explicit before/after snapshots, `117` duplicate observations against the
    normal-sync baseline, `4447` inserts, zero warnings, and a full
    row-fidelity match across `4564` observed identities.
- [ ] compare completed small-group Takeout validation against any future additional small-group fixtures if they expose richer reply, media, or reaction shapes
- [x] validate `CHANNEL_PRIVATE` fallback on a private/left channel or supergroup
  - Offline inventory found no prior local `only_my_messages_fallback`
    evidence; source `113` was the strongest private/left-shape candidate at
    that time.
    Live Takeout retries `7`, `8`, and `9` were blocked by
    `TAKEOUT_INIT_DELAY`; batch `14` later completed without
    `only_my_messages_fallback`, `only_my_messages`, or fallback-used evidence,
    so this row remains open.
  - Read-only re-inventory after source `113` batch `14` selected source `114`
    as the strongest next candidate because prior sanitized normal-sync
    validation observed `CHANNEL_PRIVATE`, it has no prior Takeout batches, and
    its stored identity remains dialog-backed with access-hash presence and no
    username.
  - Source `114` Takeout batch `15` reproduced the typed `CHANNEL_PRIVATE`
    terminal outcome before observations, but recorded no
    `only_my_messages_fallback`, `only_my_messages`, or fallback-used evidence;
    a follow-up code investigation found the preflight path was failing before
    the fallback marker could be persisted.
  - Source `114` post-fix Takeout batch `17` completed as `partial` with
    `only_my_messages_fallback`, `only_my_messages = 1`, and
    `history_scope = partial_private_history`; Telegram exposed zero
    only-my-messages observations, so full-history import remains a separate
    concern.
- [x] validate shifted export DC behavior and the warning path when fallback to home DC is used
  - Code-backed validation proves local shifted export-DC fallback, Telegram RPC
    non-fallback, one durable `export_dc_fallback` warning, and sanitized
    diagnostics. Natural live fallback remains unobserved in the current
    environment.
- [x] compare Takeout-imported rows with normal sync rows for content, media metadata, reply/thread metadata, reaction counts, and duplicate skipping
  - Source `113` completed the normal-sync setup but Takeout batches `7` and
    `8` failed before observations with `TAKEOUT_INIT_DELAY`.
  - Source `113` retry batch `9` remained blocked before observations with
    `TAKEOUT_INIT_DELAY`.
  - Source `113` batch `14` completed with `467` observed identities, `438`
    inserts, `29` duplicate observations against the normal-sync baseline, zero
    warnings, and a full row-fidelity match.
  - Source `18` batch `10` completed after a normal-sync baseline with `42`
    duplicate observations, `425` inserts, zero warning codes, and a full
    row-fidelity match across `467` observed identities.
- [x] retry the controlled migrated small-group-to-supergroup Takeout smoke after Telegram `TAKEOUT_INIT_DELAY` expires and verify migrated-history deferment without unsafe old `chat` rows
  - Source `115` remains the selected smoke source: it is a dialog-backed
    `supergroup` with `peer_kind = channel`, no username, access-hash presence,
    one current-history item, and prior Takeout batch `2` blocked before
    observations with `TAKEOUT_INIT_DELAY`.
  - Source `115` retry batch `18` completed as `partial` with
    `migrated_history_detected = 1`, `migrated_history_imported = 0`,
    `history_scope = current_history_with_migrated_deferred`, one
    `migrated_history_deferred` warning, and zero unsafe old `chat` history
    rows. This validates detection/deferment only; migrated-history import
    enablement remains open below.
- [x] define richer incomplete-import recovery actions and user policy beyond
  the shipped read-only recovery state
  - Implemented the safe recovery-policy slice: failed, cancelled,
    interrupted, and partial-completed Takeout notices now describe the
    safe re-run policy and known warning-code limitations without adding
    discard, persisted dismiss, or true resume behavior.
- [ ] enable migrated small-group history only after provenance and real-data
  validation prove the typed Telegram identity boundary is safe
- [x] decide whether Takeout import should refresh the forum-topic catalog after successful finish
  - Policy implemented: completed Takeout imports refresh forum topics for
    eligible supergroup sources, including completed partial imports, while
    failed and cancelled attempts do not refresh.
  - Refresh failures preserve completed Takeout status and record durable
    warning code `forum_topic_refresh_failed`.
  - Source `21` / batch `4` and source `22` / batch `11` remain sanitized
    partial-run decision input, not proof of completed live behavior.

Acceptance:

- Successful Takeout import updates `last_sync_state` and `last_synced_at`.
- Failed or cancelled Takeout imports are explainable and recoverable without
  being mistaken for complete history.
- Export DC fallback and only-my-messages fallback warnings remain visible in job state.
- Migrated supergroup history has a safe provenance and validation policy
  before import is enabled.

### 3.2 Database Schema Simplification

Priority: high.

Analysis:

- Full findings are recorded in
  `docs/archive/database-schema-legacy-analysis.md`.

- [x] decide cleanup policy for old Telegram `sources.metadata_zstd` blobs
- [x] implement an explicit guarded audit/dry-run/clear helper for eligible
  legacy Telegram source metadata blobs

Acceptance:

- New provider work does not need to touch legacy Telegram subtype compatibility.
- Migrated Telegram history has durable provenance and validation policy before import is enabled.
- Normal Telegram source, sync, Takeout, browsing, and export paths keep using
  typed identity/display cache fields rather than legacy Telegram metadata
  blobs.
- Any blob cleanup is validation-aware and does not remove repair input before
  the remaining real-data checks are done.
- Legacy Telegram `sources.metadata_zstd` cleanup is not an automatic
  destructive migration, startup cleanup, or opportunistic sync/update/list/
  Takeout side effect.

### 3.3 Saved Runs Discoverability And Cleanup

Priority: medium.

- [ ] add historical search/filtering by source, source group, provider, profile, model, template, and date
- [ ] consider UI affordances for missing legacy/capture failed saved-run states

Acceptance:

- Large saved-run histories can be narrowed quickly without reconstructing the original run context.

### 3.4 NotebookLM Export Follow-Ups

Priority: medium.

- [ ] add optional link enrichment with explicit user opt-in and cache
- [ ] add source-group export if the analysis group workflow needs it
- [ ] render forward context after sync persists forward metadata
- [ ] decide whether export needs richer topic grouping beyond materialized forum memberships
- [ ] consider saved-analysis-snapshot export based on `analysis_run_messages`

### 3.5 YouTube Source Follow-Ups

Priority: medium.

- [ ] add YouTube-specific NotebookLM export enrichment with transcript segment timestamps, canonical video links, and playlist membership metadata in export output
- [ ] improve typed YouTube playlist detail/browsing for linked, unavailable,
  removed, upcoming, live, auth-gated, deleted, and unknown-unavailable entries
- [ ] add speech-to-text fallback for videos without captions
- [ ] add live chat ingest
- [ ] support media-aware analysis over thumbnails or downloaded media if a future setting explicitly allows media downloads
- [ ] make YouTube source jobs persistent/resumable across app restart
- [ ] broaden manual/live validation for auto-caption-only, no-caption, active live, upcoming, private/member/age/geo-gated, and large playlist sources

Acceptance:

- Future YouTube export enhancements do not regress the existing generic NotebookLM export.
- No media download or speech-to-text path runs without explicit user opt-in.
- Restarted apps can explain or resume interrupted YouTube work according to the selected future policy.

### 3.6 Media Download, Preview, And Analysis

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

### 3.7 Stabilization

Priority: medium.

- [ ] add a single documented full-project verification command or script
- [ ] add CI for frontend tests, Svelte check, Rust tests, Rust lint, formatting, and `git diff --check`
- [ ] pin `grammers-*` dependencies to an explicit `rev` or owned release policy
- [ ] verify Telegram and LLM event-driven UI flows after the next major backend changes
- [ ] audit backend errors, frontend status text, and debug output for accidental credential exposure

## 4. Explicit Non-Goals

| Idea | Decision | Why |
| --- | --- | --- |
| Hexagonal architecture rewrite | do not do | too heavy for current scale |
| Telegram trait abstraction mainly for tests | do not do | too much indirection for weak payoff |
| Service-heavy frontend architecture | do not do | poor fit for this Svelte app |
| E2E-first expansion before core stabilization | do not do | lower ROI than targeted storage and logic tests right now |
| Splitting every large file automatically | do not do | only split where it lowers risk or unlocks backlog work |
