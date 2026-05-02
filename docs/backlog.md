# Extractum Unified Backlog

> **Updated:** 2026-05-02
> **Working rule:** this file tracks open follow-up work only

---

## 1. Purpose

This backlog tracks open technical and product work only.

Released work should stay in the codebase and in current-state documentation, not in the active backlog.

---

## 2. Main Open Gaps

- Telegram runtime behavior still needs broader validation against real accounts and dialogs
- private-source resolution is better defined but still needs more real-world validation
- LLM API keys and Telegram credentials still live in SQLite-backed storage
- LLM concurrency policy still needs refinement beyond the current request-scoped scheduling baseline
- saved-run history still lacks richer filtering for larger archives
- media download, preview, and media-aware analysis are still incomplete

---

## 3. Planning Principles

1. Keep architecture pragmatic and local to the current codebase.
2. Prioritize work that reduces correctness and operability risk.
3. Validate Telegram behavior against real data when static reading is insufficient.
4. Prefer tests for pure logic, storage rules, and request lifecycle boundaries.
5. Treat secret handling and request isolation as higher priority than aesthetic refactors.

---

## 4. Active Goal Areas

| Area | Current state | Target |
|---|---|---|
| Telegram runtime correctness | partially validated | validated on real accounts and dialogs |
| Private source resolution | explicit rules exist, but runtime coverage is incomplete | predictable behavior for dialog-picked private sources |
| Secret storage | SQLite-backed | secure store with migration/import path |
| LLM concurrency | request isolation is in place, but limit policy is still coarse | predictable request scheduling with clearer limits |
| Saved runs UX | global history and active/history split are shipped | richer narrowing and filtering for large archives |
| Media support | metadata-first only | optional download/preview and media-aware analysis |
| Stabilization | partially recorded | repeatable verification baseline and broader tests |

---

## 5. Active Roadmap

### Phase 0. Baseline And Sanity Check

Status: partial.

Goal: confirm the exact current verification baseline before broader implementation continues.

- [x] re-check the actual Rust test count after the latest LLM/settings changes
- [ ] record current `cargo clippy` status
- [x] record current `cargo test` status
- [ ] record current `npm run check` status in an environment where `svelte-check` preprocessing is allowed to spawn `esbuild`

Current notes:

- `cargo test` passes locally with `68 passed; 0 failed`
- `npm run check` in the current sandbox fails during Svelte preprocessing with `spawn EPERM`, so it still needs a clean non-sandbox verification pass

---

### Phase 2. Telegram Runtime And Private-Source Validation

Status: partial.

Priority: high.

Goal: verify Telegram source-kind handling and private-source resolution against real accounts and dialogs.

Open checks:

- [ ] verify that `list_telegram_sources` returns broadcast channels, supergroups, and regular small groups
- [ ] verify that adding from the dialog list stores the expected `telegram_source_kind`
- [ ] verify that sync works for `channel`, `supergroup`, and `group`
- [ ] verify behavior when the user is no longer a member of a group or channel
- [ ] verify behavior for migrated small-group-to-supergroup dialogs
- [ ] validate that dialog-picked private `channel` and `supergroup` sources continue syncing through stored identity when Telegram exposes sufficient peer data
- [ ] validate cross-account isolation on two real Telegram accounts

Acceptance criteria:

- [ ] the Add Source dialog shows channels, supergroups, and groups with correct labels
- [ ] a source added from account A does not affect the same source added from account B
- [ ] sync inserts messages for each supported kind without resolving to the wrong peer
- [ ] private dialog-picked `channel` and `supergroup` sources resolve predictably through stored identity when Telegram provides sufficient data

Current notes:

- runtime spot-checks on April 27, 2026 already validated public `channel` and `supergroup` flows plus typed `validation` / `not_found` errors
- no real legacy small `group` has been validated yet
- dialog-list add flow still needs explicit UI-level validation rather than only command-level checks

---

### Phase 3. Saved Runs Discoverability

Status: partial.

Priority: medium.

Goal: make previous analysis runs easy to narrow down once archives become large.

Open work:

- [ ] add richer historical search/filtering by source, source group, provider, profile, model, template, and date

Acceptance criteria:

- [ ] large saved-run histories can be narrowed quickly without reconstructing the original run context

Current notes:

- Saved Runs already default to global history
- `Current scope` history filtering is already available
- queued and running runs already live in a separate `Active Runs` panel
- historical run summaries already prefer frozen `scope_label` snapshots

---

### Phase 4. LLM Security And Concurrency

Status: partial.

Goal: make the LLM subsystem safer and more robust under parallel load.

#### 4.2. Secure Secret Storage

Status: open.

Priority: high.

Goal: move sensitive credentials out of SQLite-backed storage.

Scope:

- [ ] move LLM API keys to a secure store appropriate for Tauri desktop apps
- [ ] review Telegram `api_hash` and session storage responsibilities together
- [ ] keep secrets profile-scoped or account-scoped as appropriate
- [ ] preserve existing settings through a migration or one-time import path
- [ ] avoid logging secrets in backend errors, frontend status text, or debug output

Acceptance criteria:

- [ ] new LLM provider keys are not persisted in plain SQLite
- [ ] existing configured keys can be migrated or re-entered without breaking the app
- [ ] `/settings` can still edit provider settings without exposing secrets unnecessarily
- [ ] Telegram account credentials are no longer trivially inspectable from the local database

#### 4.3. LLM Parallel Request Support

Status: partial.

Priority: medium.

Goal: support multiple LLM requests at the same time without mixing stream state, progress state, or UI output.

Scope:

- [ ] decide whether per-provider and per-profile concurrency limits need explicit configuration beyond the current shared default

Acceptance criteria:

- [ ] concurrency limits are intentional and documented
- [ ] request scheduling remains predictable under mixed interactive and background load

Current notes:

- the remaining open question is whether concurrency limits should become explicitly configurable or otherwise differentiated beyond the current per-`(provider, profile)` queue with a shared default limit

---

### Phase 5. Media Expansion

Status: open.

Goal: extend the current media-aware ingest into a fuller archival and analysis workflow.

#### 5.1. Media Download And Preview

Status: open.

Priority: medium.

Goal: extend media-aware ingest from metadata-only storage to optional binary media download and preview.

Scope:

- [ ] decide storage layout for downloaded media files
- [ ] add download policy controls so media does not unexpectedly consume disk
- [ ] render safe previews for common media types
- [ ] preserve existing metadata-only behavior as the default or fallback
- [ ] handle missing or deleted Telegram media gracefully

Acceptance criteria:

- [ ] users can opt into downloading media for selected sources or items
- [ ] downloaded media is stored outside SQLite with stable metadata references
- [ ] the analysis workspace can preview common downloaded media types

#### 5.2. Media-Aware Analysis

Status: open.

Priority: medium.

Goal: let analysis workflows account for media-bearing and media-only items in a controlled way.

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

---

### Phase 6. Stabilization

Status: open.

Goal: keep the verification baseline healthy as the remaining infrastructure work lands.

- [ ] run `cargo clippy`
- [ ] run `npm run check` outside the current sandbox restriction
- [ ] add frontend tests for `analysis-utils.ts` and `app-error.ts`
- [ ] verify that Telegram and LLM event-driven UI flows still behave correctly after the next major backend changes

---

## 6. Explicit Non-Goals

| Idea | Decision | Why |
|---|---|---|
| Hexagonal architecture rewrite | do not do | too heavy for current scale |
| Telegram trait abstraction mainly for tests | do not do | too much indirection for weak payoff |
| Service-heavy frontend architecture | do not do | poor fit for this Svelte app |
| E2E-first expansion before core stabilization | do not do | lower ROI than targeted storage and logic tests right now |
| Splitting every large file automatically | do not do | only split where it lowers risk or unlocks backlog work |

---

## 7. Execution Priority

### Near-term priority

1. finish the remaining Telegram runtime and private-source validation
2. decide whether saved-run history now needs richer metadata/date filters before deeper Phase 4 work
3. implement secure secret storage for LLM and Telegram credentials
4. finish the remaining LLM concurrency policy work after the shipped request-scoped scheduling and cancellation baseline

### Next priority

5. expand media download/preview and media-aware analysis
6. tighten stabilization and test coverage around the new infrastructure

---

## 8. Immediate Next Steps

If implementation resumes directly from this file, the recommended opening sequence is:

1. validate the remaining Telegram runtime cases on real accounts and dialogs
2. validate dialog-picked private `channel` and `supergroup` sources against the stored-identity path
3. decide whether saved-run history now needs richer metadata/date filters before Phase 4
4. continue with secure secret storage for LLM and Telegram credentials

### Session Handoff

Current implementation checkpoint:

- current work should start from the open items above, not from already-shipped profile, snapshot, or UI-separation work

Recommended next step:

- continue with secure secret storage if the team is staying on the Phase 4 track; after that, decide whether the remaining LLM concurrency work should be limited to better limit policy/configuration or deferred behind Telegram validation

---

**Status:** active open backlog
