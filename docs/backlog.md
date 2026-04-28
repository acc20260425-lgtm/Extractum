# Extractum Unified Backlog

> **Updated:** 2026-04-28
> **Sources merged:** legacy `docs/backlog.md` + `.kilo/plans/1777186648259-proud-star.md`
> **Working rule:** this file is now the single source of truth for active technical and product follow-up work

---

## 1. Purpose

This document combines two previously separate planning tracks:

1. codebase refactoring needed to reduce risk and improve delivery speed;
2. product and infrastructure backlog needed to move the app forward.

The intent is to keep one practical roadmap instead of splitting work across:

- local planning artifacts in `.kilo/`;
- product backlog notes in `docs/`;
- unwritten assumptions in active implementation work.

From this point on, active planning should happen here.

---

## 2. Current State

### 2.1. Already in place

- `llm/` is already modular and supports Gemini plus an OpenAI-compatible OmniRoute path
- `analysis/` is already decomposed into multiple submodules
- typed Tauri application errors exist and are normalized in the frontend
- media-aware ingest already stores metadata for text-bearing and media-only Telegram messages
- new analysis runs already persist frozen corpus snapshots
- source identity is already scoped by `account_id` and `telegram_source_kind`
- frontend `sources` and `accounts` types are centralized under `src/lib/types/`

### 2.2. Main technical bottlenecks

- `src-tauri/src/sources.rs` is still oversized and owns too many responsibilities
- test coverage is still too thin for confidence around Telegram edge cases and request lifecycle behavior

### 2.3. Main open product and infrastructure gaps

- `telegram_source_kind` behavior still needs runtime validation against real Telegram accounts
- private source identity and resolution are still fragile
- secrets still live in SQLite-backed settings
- provider configuration is still too rigid around a single default profile
- saved analysis run history still lacks richer search and metadata filters for larger archives
- media-aware ingest exists, but binary media flow and media-aware analysis are still incomplete

---

## 3. Planning Principles

1. Keep architecture pragmatic and local to the current codebase.
2. Prioritize work that lowers risk for upcoming product tasks.
3. Validate Telegram behavior against real data when compile-time guarantees are insufficient.
4. Prefer tests for pure logic, storage rules, and request lifecycle boundaries.
5. Improve reuse on the frontend without forcing a service-heavy design.
6. Treat correctness, secret handling, and operability as more important than aesthetic refactors.

---

## 4. Goal Areas

| Area | Current state | Target |
|---|---|---|
| `sources.rs` | too large | smaller and responsibility-split |
| `analysis/store.rs` | too large | clearer storage/corpus/chat separation |
| Telegram runtime correctness | partially inferred from code | validated on real accounts and dialogs |
| Private source resolution | fallback-heavy | predictable and better explained |
| Secret storage | SQLite-backed | secure storage |
| LLM configuration | mostly single-profile and partially hard-coded | extensible provider/profile model |
| LLM concurrency | no explicit policy | request-scoped parallel lifecycle |
| Saved runs UX | global history and active/history separation landed, but deeper filtering is still limited | better discoverability and filtering |
| Media support | metadata-first only | optional download/preview and analysis support |
| Documentation | partly lagging implementation | aligned with code and current behavior |

---

## 5. Dependencies Between Workstreams

### 5.1. Best done before broader backlog expansion

- extract compression helpers into a shared module
- extract media parsing helpers into a shared module
- split `sync_source`
- split `analysis/store.rs`
- centralize frontend types

These are not abstract cleanup for its own sake. They reduce risk before:

- private peer identity work;
- media download and preview;
- media-aware analysis;
- LLM parallel request support;
- saved runs UX changes.

### 5.2. Best done after infrastructure contracts stabilize

- secure secret storage should follow clearer provider/profile storage rules
- LLM parallel request support should follow provider configuration cleanup
- media-aware analysis should follow media download and preview decisions
- documentation refresh should happen after the major LLM, storage, and UX changes settle

---

## 6. Unified Roadmap

### Phase 0. Baseline And Sanity Check

Status: partial.

Goal: confirm the exact technical baseline before larger work starts.
- [ ] re-check the actual test count and current baseline commands before implementation
- [ ] record current `cargo clippy` and `npm run check` status

Notes:

- repository and docs studied
- heavy modules and active risk areas identified

---

### Phase 1. Low-Risk Core Refactoring

Status: completed.

Goal: remove duplication and isolate pure logic before product-facing changes.

#### 1.1. Extract `src-tauri/src/compression.rs`

Status: completed.

Notes:

- completed via shared module `src-tauri/src/compression.rs`
- `analysis/store.rs` snapshot compression now also uses the shared helper
- verification completed with `cargo fmt` and `cargo test`
- current Rust test count after this step: `37 passed`

#### 1.2. Extract `src-tauri/src/media.rs`

Status: completed.

Notes:

- completed via shared module `src-tauri/src/media.rs`
- `sources.rs` now imports shared media extraction types and helpers instead of owning them locally
- media-focused unit coverage now lives with the shared module
- verification completed with `cargo fmt` and `cargo test`
- current Rust test count after this step: `38 passed`

#### 1.3. Centralize frontend types

Status: completed.

Notes:

- completed via shared modules `src/lib/types/sources.ts` and `src/lib/types/accounts.ts`
- `src/routes/accounts/+page.svelte`, `src/routes/auth/[id]/+page.svelte`, and `src/routes/sources/+page.svelte` now import shared source/account types instead of defining them locally
- targeted route search confirms no local source/account type declarations remain under `src/routes/`
- verification completed with `npm run check` on April 28, 2026

#### 1.4. Remove deprecated page store usage

Status: completed.

Notes:

- completed in `src/routes/+layout.svelte`, `src/routes/sources/+page.svelte`, and `src/routes/auth/[id]/+page.svelte`
- project search confirms no remaining `$app/stores` imports under `src/`
- changed files passed targeted `svelte_autofixer` validation

Expected outcome: simpler base modules, lower follow-up risk, cleaner frontend reuse.

---

### Phase 2. Ingest Refactor And Telegram Runtime Validation

Status: partial.

Goal: improve ingest maintainability while validating Telegram behavior against reality.

#### 2.1. Split `sync_source`

Status: completed.

Notes:

- `sync_source` is now an orchestration layer over focused helpers for source loading, account/client auth, peer resolution, sync policy, ingest persistence, and finalization
- added storage-focused tests covering missing-source loading, initial-vs-incremental sync policy behavior, and final source-state persistence updates
- verification completed with `cargo test`
- current Rust test count after this step: `41 passed`

#### 2.2. Telegram Runtime Validation

Status: partial.

Priority: high.

Goal: verify the current `telegram_source_kind` model against real Telegram accounts and real dialogs.

Why it matters: compile-time checks cannot cover Telegram peer shapes. `grammers` can expose broadcast channels, supergroups, small groups, forbidden/min peers, migrated groups, and private peers with subtly different identity data.

Scope:

- [ ] verify that `list_telegram_sources` returns broadcast channels, supergroups, and regular small groups
- [ ] verify that adding from the dialog list stores the expected `telegram_source_kind`
- [ ] verify that sync works for `channel`, `supergroup`, and `group`
- [ ] verify behavior when the user is no longer a member of a group or channel
- [ ] verify behavior for migrated small-group-to-supergroup dialogs

Acceptance criteria:

- [ ] the Add Source dialog shows channels, supergroups, and groups with correct labels
- [ ] a source added from account A does not affect the same source added from account B
- [ ] sync inserts messages for each supported kind without resolving to the wrong peer

Notes:

- backend error classification now treats dialog lookup misses and peer-resolution misses as typed `not_found` errors instead of generic internal failures
- Telegram source-kind mismatch paths now return validation-friendly messages that include the requested and actual kind
- added unit coverage for Telegram username/link parsing and source-kind mismatch reporting
- live runtime spot-check completed on April 27, 2026 against account `Life`
- `list_telegram_sources` returned real `channel` and `supergroup` dialogs for that account; no legacy small `group` dialogs were present in this specific dataset
- dialog avatar fetch works in production flow for at least part of the dialog list; this account returned 4 dialogs with avatar data during the check
- manual add by `@username` was validated for one public channel (`@turboproject` -> stored as `channel`) and one public supergroup (`@WhiteBirdChat` -> stored as `supergroup`)
- sync was validated for those runtime-added public sources before cleanup:
  - `AI Projects` (`channel`): inserted `263`, skipped `0`, first-sync policy `last 30 days`
  - `WBChat` (`supergroup`): inserted `2654`, skipped `2`, first-sync policy `last 30 days`
- typed runtime errors were validated on a real account:
  - wrong expected kind returns structured `validation`
  - numeric dialog miss returns structured `not_found`
- live validation is still pending for:
  - a real legacy small `group`
  - behavior after leaving a group or channel
  - migrated small-group-to-supergroup dialogs
  - dialog-list add flow explicitly validated through the Add Source UI rather than via command-level runtime add
  - cross-account isolation validated on two real Telegram accounts

#### 2.3. Private Sources And Peer Identity

Status: partial.

Priority: high.

Goal: make private Telegram channels and groups predictable by storing enough peer identity to resolve them without relying only on username or dialog scanning.

Why it matters: public sources can be resolved by username, but private sources often cannot. Bare id plus kind helps, but Telegram access may need session peer cache, access hash, or dialog-derived identity.

Remaining work:

- [ ] validate on real accounts that dialog-picked private `channel` and `supergroup` sources continue syncing through stored identity when Telegram exposes sufficient peer data

Acceptance criteria:

- [ ] private sources added from dialogs continue syncing when Telegram session data can resolve them

Notes:

- `SourceMetadata` now normalizes legacy `username` / `added_from` / `access_hash` payloads into explicit `peer_identity` metadata with `strategy`, `username`, and `access_hash`
- `resolve_source_peer` now follows an explicit rules pipeline for username-backed public sources: username resolution -> compatibility dialog scan
- `resolve_source_peer` now follows an explicit rules pipeline for dialog-backed sources: stored peer identity -> optional username fallback -> compatibility dialog scan
- `channel` and `supergroup` can use stored `access_hash` identity when added from dialogs or otherwise resolved with enough metadata
- legacy small `group` sources still remain dialog-dependent because access-hash-only identity is not treated as stable support for that kind
- supported source refs are now documented as `@username`, `t.me/name`, and dialog-backed sources; numeric/manual refs remain dialog-constrained
- manual add now rejects private invite links and internal `t.me/c/...` refs with explicit guidance to add those sources from dialogs
- the metadata refactor, explicit resolution pipeline, manual-add tightening, targeted tests, and documentation refresh are complete

Phase completion gate:

- [ ] private dialog-picked `channel` and `supergroup` sources resolve predictably through stored identity when Telegram provides sufficient data

Expected outcome: ingest code becomes easier to change, and Telegram behavior is validated beyond static reading of the code.

---

### Phase 3. Analysis Storage Refactor And Saved Runs UX

Status: partial.

Goal: simplify analysis storage internals and improve run history discoverability.

#### 3.1. Split `analysis/store.rs`

Status: completed.

Notes:

- corpus lookup and snapshot-read helpers now live in `src-tauri/src/analysis/corpus.rs`
- `analysis/chat.rs` now owns its local chat message load/persist helpers instead of routing those writes through `store.rs`
- `analysis/store.rs` is now focused on run/template/group lookup, run creation, snapshot persistence, and mapping helpers
- verification completed with `cargo fmt` and `cargo test`
- current Rust test count after this step: `54 passed`

#### 3.2. Saved Runs Discoverability

Status: partial.

Priority: medium.

Goal: make previous analysis runs easy to find even when the current analysis scope changes.

Why it matters: the original Saved Runs panel was scoped to the selected source or source group. Global history and active/history separation have landed, but larger archives still need richer narrowing tools.

Scope:

- [ ] add richer historical search/filtering by source, source group, provider, model, template, and date

Acceptance criteria:

- [ ] large saved-run histories can be narrowed quickly without reconstructing the original run context or scanning an unfiltered global list

Notes:

- Saved Runs now default to global history instead of inheriting the current composer scope
- an explicit `Current scope` history filter remains available when scoped browsing is useful
- completed runs can be opened regardless of the currently selected source or source group
- queued and running runs now live in a dedicated `Active Runs` panel instead of mixing with saved history
- historical run summaries now prefer frozen `scope_label` snapshots so renamed or deleted sources/groups remain identifiable

Expected outcome: storage logic gets clearer, and saved reports become easier to revisit.

---

### Phase 4. LLM Configuration, Secret Storage, And Concurrency

Status: partial.

Goal: make the LLM subsystem more extensible, safer, and more robust under load.

#### 4.1. LLM Provider Configuration

Status: completed.

Priority: high.

Goal: turn Gemini and OmniRoute support into a provider configuration model that can grow beyond the current hard-coded default profile.

Why it matters: the backend now has a modular LLM implementation, but the product still exposed only one active profile and hard-coded OmniRoute's OpenAI-compatible `base_url`.

Scope:

- [x] add provider profile management beyond the single `default` profile
- [x] decide whether `base_url` should be stored for OpenAI-compatible providers and exposed in `/settings`
- [x] validate model list and Test Provider flows for Gemini and OmniRoute
- [x] make provider labels, placeholders, and error messages provider-neutral where appropriate

Acceptance criteria:

- [x] users can configure Gemini and OmniRoute without code changes
- [x] OpenAI-compatible providers can reuse the same backend path with a configured `base_url`
- [x] Test Provider always uses the saved provider, model, and key the user sees in settings

Notes:

- completed via multi-profile backend/frontend support in `src-tauri/src/llm/` and `src/routes/settings/+page.svelte`
- `get_llm_profiles` now returns the full saved profile list plus the active profile instead of only a single `default_profile`
- LLM profiles now persist provider-specific `base_url` settings, with OpenAI-compatible requests and model-list calls using the saved or currently edited `base_url`
- `/settings` now supports selecting existing profiles, creating new profiles, saving without activation, saving and activating, and masked API key editing
- provider copy and backend error labels now use provider-neutral OpenAI-compatible wording where appropriate instead of hard-coding OmniRoute-specific phrasing
- analysis run metadata did not need a migration in this slice because profile ids remain the persisted user-facing identifier; editable display names were not introduced
- verification completed with `cargo fmt`, `cargo test`, and `npm run check` on April 28, 2026
- current Rust test count after this step: `57 passed`

#### 4.2. Secure Secret Storage

Status: open.

Priority: high.

Goal: move sensitive credentials out of SQLite-backed `app_settings`.

Why it matters: LLM API keys and Telegram credentials are currently easy to inspect in the local database. That is acceptable only as development debt.

Scope:

- [ ] move LLM API keys to a secure store appropriate for Tauri desktop apps
- [ ] review Telegram `api_hash` and session storage responsibilities
- [ ] keep secrets profile-scoped or account-scoped as appropriate
- [ ] preserve existing settings through a migration or one-time import path
- [ ] avoid logging secrets in backend errors, frontend status text, or debug output

Acceptance criteria:

- [ ] new LLM provider keys are not persisted in plain SQLite
- [ ] existing configured keys can be migrated or re-entered without breaking the app
- [ ] `/settings` can still edit provider settings without exposing secrets unnecessarily

#### 4.3. LLM Parallel Request Support

Status: planned.

Priority: medium.

Goal: support multiple LLM requests running at the same time without mixing stream state, progress state, or UI output.

Why it matters: analysis map chunks, report reduction, follow-up chat, and provider smoke tests can all need request-scoped lifecycle handling. The refactored LLM runner is ready for this, but no concurrency policy exists yet.

Scope:

- [ ] define concurrency limits per provider and profile
- [ ] add active request tracking keyed by `request_id`
- [ ] add cancellation support for long-running requests
- [ ] keep stream buffers, usage, timeout, and callbacks request-local
- [ ] decide how the frontend should display multiple active streams
- [ ] ensure analysis progress and provider test output ignore unrelated request events

Acceptance criteria:

- [ ] concurrent LLM requests cannot overwrite each other's output
- [ ] a user can cancel a long-running request
- [ ] provider and analysis events remain traceable by `request_id`

Expected outcome: better provider flexibility, safer secret handling, and cleaner request isolation.

---

### Phase 5. Media Expansion

Status: open.

Goal: extend the current media-aware ingest into a fuller archival and analysis workflow.

#### 5.1. Media Download And Preview

Status: open.

Priority: medium.

Goal: extend media-aware ingest from metadata-only storage to optional binary media download and preview.

Why it matters: `/sources` already preserves media metadata, but users cannot inspect the actual files from the local archive.

Scope:

- [ ] decide storage layout for downloaded media files
- [ ] add download policy controls so media does not unexpectedly consume disk
- [ ] render safe previews for common media types
- [ ] preserve existing metadata-only behavior as the default or fallback
- [ ] handle missing or deleted Telegram media gracefully

Acceptance criteria:

- [ ] users can opt into downloading media for selected sources or items
- [ ] downloaded media is stored outside SQLite with stable metadata references
- [ ] `/sources` can preview common downloaded media types

#### 5.2. Media-Aware Analysis

Status: open.

Priority: medium.

Goal: let analysis workflows account for media-bearing and media-only items in a controlled way.

Why it matters: current analysis is text-first. Media-only posts are visible in `/sources` but excluded from the analysis corpus, which can hide important evidence.

Scope:

- [ ] define how media metadata should appear in text-only prompts
- [ ] decide whether downloaded media can be sent to multimodal providers
- [ ] add citation semantics for media evidence
- [ ] update trace resolution and report viewer to handle media refs
- [ ] keep text-only analysis available for providers without multimodal support

Acceptance criteria:

- [ ] reports can mention relevant media metadata with clear citations
- [ ] media-only items do not silently disappear when the selected analysis mode supports them
- [ ] non-multimodal providers degrade predictably

Expected outcome: media stops being visible only in archive views and becomes usable in later workflows.

---

### Phase 6. Stabilization And Documentation

Status: open.

Goal: re-align verification, behavior, and project docs after the larger changes land.

#### 6.1. Technical stabilization

- [ ] run `cargo test`
- [ ] run `cargo clippy`
- [ ] run `npm run check`
- [ ] add frontend tests for `analysis-utils.ts` and `app-error.ts`
- [ ] verify that new Telegram and LLM flows still behave correctly in the event-driven UI

#### 6.2. Documentation Refresh

Status: open.

Priority: low.

Goal: align project docs with the current LLM and settings implementation.

Why it matters: several docs still describe the LLM flow as Gemini-only, while the app now supports Gemini and OmniRoute through a modular backend.

Scope:

- [ ] update `README.md`
- [ ] update `docs/project.md`
- [ ] update `docs/design-document.md`
- [ ] update `docs/database-schema.md`
- [ ] update `docs/architecture-deep-dive.md`
- [ ] replace Gemini-only language with provider-neutral language where appropriate
- [ ] document OmniRoute's OpenAI-compatible path and current hard-coded `base_url` limitation
- [ ] keep the secure-storage warning current

Acceptance criteria:

- [ ] new contributors can understand the current Gemini and OmniRoute provider flow from docs
- [ ] the docs no longer list completed LLM refactor work as future work

Expected outcome: code, behavior, and documentation converge again.

---

## 7. Explicit Non-Goals

| Idea | Decision | Why |
|---|---|---|
| Hexagonal architecture rewrite | do not do | too heavy for current scale |
| Telegram trait abstraction mainly for tests | do not do | too much indirection for weak payoff |
| Service-heavy frontend architecture | do not do | poor fit for this Svelte app |
| E2E-first expansion before core stabilization | do not do | lower ROI than targeted storage and logic tests right now |
| Splitting every large file automatically | do not do | only split where it lowers risk or unlocks backlog work |

---

## 8. Execution Priority

### Near-term priority

1. Phase 2: finish Telegram runtime and private-source validation on real accounts
2. Phase 3: finish saved-run history filtering if current global history still feels too broad
3. Phase 4: provider configuration, secret storage, and LLM concurrency

### Next priority

4. Phase 5: media download/preview and media-aware analysis
5. Phase 6: stabilization and documentation refresh

---

## 9. Backlog-To-Phase Mapping

| Backlog item | Phase |
|---|---|
| Telegram Runtime Validation | Phase 2 |
| Private Sources And Peer Identity | Phase 2 |
| Secure Secret Storage | Phase 4 |
| LLM Provider Configuration | Phase 4 |
| LLM Parallel Request Support | Phase 4 |
| Saved Runs Discoverability | Phase 3 |
| Media Download And Preview | Phase 5 |
| Media-Aware Analysis | Phase 5 |
| Documentation Refresh | Phase 6 |

---

## 10. Immediate Next Steps

If implementation starts directly from this file, the recommended opening sequence is:

1. validate the remaining Telegram runtime cases on real accounts and dialogs
2. validate dialog-picked private `channel` and `supergroup` sources against the stored-identity path
3. decide whether saved-run history now needs richer metadata/date filters before Phase 4
4. continue with Phase 4.2 secure secret storage once the team is ready to stay within Phase 4 work

### Session Handoff

Current implementation checkpoint:

- shared compression and media helpers are extracted into `src-tauri/src/compression.rs` and `src-tauri/src/media.rs`
- `sync_source` and `analysis/store.rs` are already split into narrower analysis and ingest helpers
- frontend `sources` and `accounts` types are centralized under `src/lib/types/`
- Saved Runs now use global history by default and keep queued/running work in a separate `Active Runs` panel
- Phase 4.1 provider configuration cleanup is complete: multi-profile LLM settings, configurable OpenAI-compatible `base_url`, and profile-aware provider test/model-list flows are now in place
- current verification checkpoints include passing `cargo test` and `npm run check`

Recommended next step:

- if the team is staying on the Phase 4 track, continue with **Phase 4.2**, moving LLM secrets out of SQLite-backed `app_settings`

---

**Status:** active unified backlog
**Rule going forward:** use this file for ongoing planning and prioritization
