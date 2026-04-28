# Extractum Unified Backlog

> **Updated:** 2026-04-27
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

### 2.2. Main technical bottlenecks

- `src-tauri/src/sources.rs` is still oversized and owns too many responsibilities
- `src-tauri/src/analysis/store.rs` is still too large for comfortable evolution
- frontend types for `sources` and `accounts` still live in route files
- deprecated `$app/stores` usage still exists in parts of the Svelte app
- test coverage is still too thin for confidence around Telegram edge cases and request lifecycle behavior

### 2.3. Main open product and infrastructure gaps

- `telegram_source_kind` behavior still needs runtime validation against real Telegram accounts
- private source identity and resolution are still fragile
- secrets still live in SQLite-backed settings
- provider configuration is still too rigid around a single default profile
- saved analysis runs are not discoverable enough outside current scope
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
| Saved runs UX | too scope-bound | better discoverability and filtering |
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

- [x] repository and docs studied
- [x] heavy modules and active risk areas identified
- [ ] re-check the actual test count and current baseline commands before implementation
- [ ] record current `cargo clippy` and `npm run check` status

---

### Phase 1. Low-Risk Core Refactoring

Status: open.

Goal: remove duplication and isolate pure logic before product-facing changes.

#### 1.1. Extract `src-tauri/src/compression.rs`

- [x] move `compress_text`, `decompress_text`, and `compress_json_bytes`
- [x] remove duplicate compression logic from `analysis/mod.rs`
- [x] update imports in `sources.rs` and analysis modules
- [x] add unit tests for round-trips and boundary cases

Notes:

- completed via shared module `src-tauri/src/compression.rs`
- `analysis/store.rs` snapshot compression now also uses the shared helper
- verification completed with `cargo fmt` and `cargo test`
- current Rust test count after this step: `37 passed`

#### 1.2. Extract `src-tauri/src/media.rs`

- [x] move `ExtractedItemPayload`, `ExtractedMediaPayload`, `ItemMediaMetadata`, and `DocumentSignals`
- [x] move `extract_item_payload`, `extract_media_payload`, `derive_content_kind`, and `media_label`
- [x] add unit tests for media extraction branches

Notes:

- completed via shared module `src-tauri/src/media.rs`
- `sources.rs` now imports shared media extraction types and helpers instead of owning them locally
- media-focused unit coverage now lives with the shared module
- verification completed with `cargo fmt` and `cargo test`
- current Rust test count after this step: `38 passed`

#### 1.3. Centralize frontend types

- [ ] create `src/lib/types/sources.ts`
- [ ] create `src/lib/types/accounts.ts`
- [ ] update route files to import shared types
- [ ] remove local type duplication

#### 1.4. Remove deprecated page store usage

- [x] replace `$app/stores` with `$app/state` where required
- [x] verify `+layout.svelte`, `sources/+page.svelte`, and `auth/[id]/+page.svelte`

Notes:

- completed in `src/routes/+layout.svelte`, `src/routes/sources/+page.svelte`, and `src/routes/auth/[id]/+page.svelte`
- project search confirms no remaining `$app/stores` imports under `src/`
- changed files passed targeted `svelte_autofixer` validation

Expected outcome: simpler base modules, lower follow-up risk, cleaner frontend reuse.

---

### Phase 2. Ingest Refactor And Telegram Runtime Validation

Status: open.

Goal: improve ingest maintainability while validating Telegram behavior against reality.

#### 2.1. Split `sync_source`

- [x] extract `load_source`
- [x] extract `get_authorized_client`
- [x] extract `resolve_and_refresh_peer`
- [x] extract `determine_sync_policy`
- [x] extract `extract_items_from_messages`
- [x] extract `persist_items`
- [x] extract `finalize_sync`
- [x] add characterization tests and storage-focused tests

Notes:

- `sync_source` is now an orchestration layer over focused helpers for source loading, account/client auth, peer resolution, sync policy, ingest persistence, and finalization
- added storage-focused tests covering missing-source loading, initial-vs-incremental sync policy behavior, and final source-state persistence updates
- verification completed with `cargo test`
- current Rust test count after this step: `41 passed`

#### 2.2. Telegram Runtime Validation

Status: open.

Priority: high.

Goal: verify the current `telegram_source_kind` model against real Telegram accounts and real dialogs.

Why it matters: compile-time checks cannot cover Telegram peer shapes. `grammers` can expose broadcast channels, supergroups, small groups, forbidden/min peers, migrated groups, and private peers with subtly different identity data.

Scope:

- [ ] verify that `list_telegram_sources` returns broadcast channels, supergroups, and regular small groups
- [x] verify that source avatars load for channels and groups
- [ ] verify that adding from the dialog list stores the expected `telegram_source_kind`
- [x] verify that manual add by `@username` works for public channels and public groups
- [ ] verify that sync works for `channel`, `supergroup`, and `group`
- [ ] verify behavior when the user is no longer a member of a group or channel
- [ ] verify behavior for migrated small-group-to-supergroup dialogs

Acceptance criteria:

- [ ] the Add Source dialog shows channels, supergroups, and groups with correct labels
- [ ] a source added from account A does not affect the same source added from account B
- [ ] sync inserts messages for each supported kind without resolving to the wrong peer
- [x] unsupported or inaccessible Telegram peers produce friendly typed errors

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

Scope:

- [x] audit current `SourceMetadata` coverage for dialog-picked private sources
- [x] store peer identity data when `grammers` exposes it
- [x] keep manual numeric add constrained to dialogs unless metadata is sufficient
- [x] improve errors for private sources that disappeared from dialogs
- [x] document supported Telegram source refs: `@username`, `t.me/name`, and dialog-picked private source

Acceptance criteria:

- [ ] private sources added from dialogs continue syncing when Telegram session data can resolve them
- [x] if a private source cannot be resolved, the app explains the likely reason and suggests re-adding from dialogs
- [x] public username sources still sync through username resolution
- [x] existing sources with older metadata continue to work through fallback dialog scanning

Notes:

- `SourceMetadata` now normalizes legacy `username` / `added_from` / `access_hash` payloads into explicit `peer_identity` metadata with `strategy`, `username`, and `access_hash`
- `resolve_source_peer` now follows an explicit rules pipeline for username-backed public sources: username resolution -> compatibility dialog scan
- `resolve_source_peer` now follows an explicit rules pipeline for dialog-backed sources: stored peer identity -> optional username fallback -> compatibility dialog scan
- `channel` and `supergroup` can use stored `access_hash` identity when added from dialogs or otherwise resolved with enough metadata
- legacy small `group` sources still remain dialog-dependent because access-hash-only identity is not treated as stable support for that kind
- supported source refs are now documented as `@username`, `t.me/name`, and dialog-backed sources; numeric/manual refs remain dialog-constrained

Implementation plan:

##### 2.3.1. Audit current peer identity behavior

Goal: document what is stored today and where private-source fragility still comes from.

- [x] describe the current resolution chain in `resolve_source_peer`: username -> metadata identity -> dialog scan
- [x] record the current `SourceMetadata` contract and which fields are actually used during sync
- [x] separate already-supported flows from fragile flows for public sources, dialog-picked private sources, and legacy sources
- [x] document the main risk cases: private source without username, small `group`, and dialog-dependent resolution

Definition of done:

- [x] there is a short written summary in backlog notes or code comments that explains the current contract and its limitations

##### 2.3.2. Design the new peer identity model

Goal: replace loosely related metadata fields with an explicit peer identity model.

- [x] define a nested metadata structure such as `peer_identity`
- [x] include enough fields to distinguish `username`-resolved sources from dialog-picked private sources
- [x] capture the intended resolution strategy explicitly rather than inferring it indirectly
- [x] keep all new fields backward-compatible with existing rows by using optional fields where needed

Definition of done:

- [x] the target `SourceMetadata` shape is decided and maps cleanly to supported Telegram source flows

##### 2.3.3. Implement backward-compatible metadata serialization

Goal: introduce the new metadata model without breaking stored sources.

- [x] update `SourceMetadata` in `src-tauri/src/sources.rs`
- [x] update `encode_source_metadata`
- [x] update `decode_source_metadata`
- [x] keep existing legacy payloads decodable without a database migration
- [x] add unit tests for old payload decode and new payload roundtrip

Definition of done:

- [x] old metadata payloads still decode successfully
- [x] new metadata payloads roundtrip through compression and JSON encoding

##### 2.3.4. Expand metadata captured by `add_telegram_source`

Goal: store enough identity data at add time to make later sync more predictable.

- [x] when adding from dialogs, persist explicit dialog-picked peer identity data
- [x] when adding by username, persist explicit username-based resolution metadata
- [x] preserve avatar cache handling while expanding metadata
- [x] keep the persisted source record shape backward-compatible for the rest of the app

Definition of done:

- [x] newly added public and dialog-picked private sources store explicit identity strategy information

##### 2.3.5. Refactor `resolve_source_peer` into an explicit resolution pipeline

Goal: make peer resolution deterministic and understandable instead of relying on loosely ordered fallbacks.

- [x] make resolution strategy selection explicit based on stored metadata
- [x] prefer username resolution for username-based public sources
- [x] prefer stored peer identity for dialog-picked private `channel` and `supergroup` sources
- [x] keep dialog scanning only as a compatibility fallback where appropriate
- [x] return clearer failure reasons when no resolution path succeeds

Definition of done:

- [x] `resolve_source_peer` reads as an explicit rules pipeline rather than a chain of opportunistic fallbacks

##### 2.3.6. Define support boundaries by Telegram source kind

Goal: avoid pretending that all source kinds support the same identity guarantees.

- [x] define the expected private/public behavior for `channel`
- [x] define the expected private/public behavior for `supergroup`
- [x] define the practical limitations for `group`
- [x] ensure the code and docs use the same support language for each kind

Definition of done:

- [x] supported and limited flows are explicitly documented for `channel`, `supergroup`, and `group`

##### 2.3.7. Tighten manual add rules

Goal: prevent users from creating private-source records that cannot be synced predictably later.

- [ ] keep `@username` and `t.me/name` as supported manual refs for public sources
- [ ] keep dialog-picked private sources as the preferred supported private flow
- [ ] reject or clearly discourage numeric/manual private adds when metadata is insufficient
- [ ] return friendly errors when a numeric source cannot be found in dialogs

Definition of done:

- [ ] manual add no longer suggests unsupported private-source flows are stable

##### 2.3.8. Improve typed errors for private-source resolution

Goal: explain failure modes clearly enough that the user knows what to do next.

- [x] add a clear private-source resolution failure message that recommends re-adding from dialogs
- [x] distinguish username resolution failure from stored-identity failure where possible
- [x] keep source-kind mismatch errors structured and user-facing
- [x] avoid generic internal failures for expected private-source edge cases

Definition of done:

- [x] private-source sync failures return user-actionable typed errors

##### 2.3.9. Add targeted tests for new resolution contracts

Goal: lock in the new behavior and reduce regressions around private-source handling.

- [x] test old metadata decode compatibility
- [x] test new metadata roundtrip coverage
- [x] test stored peer identity resolution for `channel` and `supergroup`
- [x] test that `group` does not pretend to support unsupported identity-only resolution
- [x] test friendly failure behavior for private sources that can no longer be resolved
- [ ] test unsupported manual/private add cases

Definition of done:

- [ ] the new resolution contracts are covered by unit or storage-oriented tests where feasible

##### 2.3.10. Refresh documentation

Goal: align the docs with the new private-source contract.

- [x] update `docs/architecture-deep-dive.md`
- [x] update `docs/backlog.md`
- [x] document the supported Telegram source refs: `@username`, `t.me/name`, and dialog-picked private source
- [x] document the remaining limitations for small `group` sources and legacy fallback behavior

Definition of done:

- [x] a contributor can understand the new private-source rules from docs without re-deriving them from `sources.rs`

Recommended implementation sequence:

- [x] Stage A: audit current behavior, design the metadata model, and land backward-compatible serialization
- [x] Stage B: update add-time metadata capture and refactor the resolution pipeline
- [ ] Stage C: tighten manual-add rules and add the remaining unsupported manual/private coverage

Phase completion gate:

- [ ] private dialog-picked `channel` and `supergroup` sources resolve predictably through stored identity when Telegram provides sufficient data
- [x] legacy sources remain functional through compatibility fallbacks
- [ ] unsupported private-source flows are rejected or clearly explained instead of behaving unpredictably
- [x] documentation reflects the new peer identity contract and the remaining limitations

Expected outcome: ingest code becomes easier to change, and Telegram behavior is validated beyond static reading of the code.

---

### Phase 3. Analysis Storage Refactor And Saved Runs UX

Status: open.

Goal: simplify analysis storage internals and improve run history discoverability.

#### 3.1. Split `analysis/store.rs`

- [ ] move corpus-loading logic into `analysis/corpus.rs`
- [ ] move chat-related storage helpers into `analysis/chat.rs` where appropriate
- [ ] narrow `store.rs` to run CRUD, snapshots, and mapping responsibilities
- [ ] update imports in `report.rs` and related modules

#### 3.2. Saved Runs Discoverability

Status: open.

Priority: medium.

Goal: make previous analysis runs easy to find even when the current analysis scope changes.

Why it matters: the current Saved Runs panel is scoped to the selected source or source group. That can make older runs look missing when the user switches scope or opens Analysis without the original target selected.

Scope:

- [ ] decide whether Saved Runs should default to global history or scoped history
- [ ] add explicit scope filters if both behaviors are useful
- [ ] preserve the ability to open completed runs regardless of current composer scope
- [ ] consider search/filter by source, source group, provider, model, template, status, and date
- [ ] keep active-run restoration separate from historical run browsing

Acceptance criteria:

- [ ] users can find previous saved runs without reconstructing the original source/group selection
- [ ] scoped filtering remains available when useful
- [ ] running and queued runs remain visually distinct from completed and failed history

Expected outcome: storage logic gets clearer, and saved reports become easier to revisit.

---

### Phase 4. LLM Configuration, Secret Storage, And Concurrency

Status: open.

Goal: make the LLM subsystem more extensible, safer, and more robust under load.

#### 4.1. LLM Provider Configuration

Status: open.

Priority: high.

Goal: turn Gemini and OmniRoute support into a provider configuration model that can grow beyond the current hard-coded default profile.

Why it matters: the backend now has a modular LLM implementation, but the product still exposes only one active profile and hard-codes OmniRoute's OpenAI-compatible `base_url`.

Scope:

- [ ] add provider profile management beyond the single `default` profile
- [ ] decide whether `base_url` should be stored for OpenAI-compatible providers and exposed in `/settings`
- [ ] validate model list and Test Provider flows for Gemini and OmniRoute
- [ ] make provider labels, placeholders, and error messages provider-neutral where appropriate
- [ ] update analysis run metadata if user-facing provider profile names become editable

Acceptance criteria:

- [ ] users can configure Gemini and OmniRoute without code changes
- [ ] OpenAI-compatible providers can reuse the same backend path with a configured `base_url`
- [ ] Test Provider always uses the saved provider, model, and key the user sees in settings

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

1. Phase 1: extract shared utilities and shared types
2. Phase 2: refactor ingest and validate Telegram runtime behavior
3. Phase 3: refactor analysis storage and improve saved runs discoverability

### Next priority

4. Phase 4: provider configuration, secret storage, and LLM concurrency
5. Phase 5: media download/preview and media-aware analysis
6. Phase 6: stabilization and documentation refresh

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

1. extract `media.rs`
2. create `src/lib/types/sources.ts` and `src/lib/types/accounts.ts`
3. replace deprecated page store usage
4. refactor `sync_source`
5. validate real Telegram runtime behavior on actual accounts and dialogs

### Session Handoff

Current implementation checkpoint:

- shared compression helpers extracted into `src-tauri/src/compression.rs`
- `sources.rs`, `analysis/mod.rs`, and `analysis/store.rs` switched to the shared compression helpers
- formatting and Rust tests passed after the change

Recommended next step:

- continue with **Phase 1.3**, centralizing frontend `sources` and `accounts` types

---

**Status:** active unified backlog
**Rule going forward:** use this file for ongoing planning and prioritization
