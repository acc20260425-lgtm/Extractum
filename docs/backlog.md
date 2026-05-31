# Extractum Backlog

> **Updated:** 2026-05-31
> **Rule:** this file tracks open work only. Shipped work belongs in current-state docs and Git history.

## 1. Priority Snapshot

| Priority | Area | Next outcome |
| --- | --- | --- |
| Medium | NotebookLM export follow-ups | decide on optional link enrichment, forward metadata, and richer topic grouping |
| Medium | YouTube source follow-ups | broaden live-provider validation and decide which enrichment/resumability features matter after the MVP |
| Medium | Telegram topic/forward enrichment | model richer Forum Topics browsing/export and forward metadata when needed |
| Medium | Frontend workspace evolution | improve evidence navigation, topic filters, saved-run history, and remaining source-surface consistency |
| Medium | Media support | move beyond metadata-first storage only after explicit download and analysis policies exist |
| Medium | Stabilization and secret safety | add CI, dependency policy, event-flow validation, and secret-leak audit coverage |
| Medium | Desktop product maturity | add sanitized diagnostics, searchable settings, explicit export profiles, and local privacy/data lifecycle controls |

## 2. Planning Principles

1. Keep architecture pragmatic and local to the current codebase.
2. Prioritize correctness, data integrity, and operability risks before UI polish.
3. Validate Telegram behavior against real data when static reading is insufficient.
4. Prefer tests for pure logic, storage rules, request lifecycle boundaries, and route workflow regressions.
5. Keep closed Superpowers plans out of active docs; preserve them through Git
   history. Move historical specs or verification notes into
   `docs/superpowers/archive/` only when they remain useful as context.

## 3. Open Roadmap

### 3.1 NotebookLM Export Follow-Ups

Priority: medium.

- [ ] add optional link enrichment with explicit user opt-in and cache
- [ ] render forward context after sync persists forward metadata
- [ ] decide whether export needs richer topic grouping beyond materialized forum memberships
- [ ] consider saved-analysis-snapshot export based on `analysis_run_messages`

### 3.2 YouTube Source Follow-Ups

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

### 3.3 Media Download, Preview, And Analysis

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

### 3.4 Stabilization

Priority: medium.

- [ ] add CI for frontend tests, Svelte check, Rust tests, Rust lint, formatting, and `git diff --check`
- [ ] pin `grammers-*` dependencies to an explicit `rev` or owned release policy
- [ ] verify Telegram and LLM event-driven UI flows after the next major backend changes
- [ ] audit backend errors, frontend status text, and debug output for accidental credential exposure

### 3.5 Frontend Workspace Evolution

Priority: medium.

Guidance: `docs/frontend-architecture-evolution-analysis.md`.

- [ ] add compact Telegram topic navigation/filtering for source browsing
- [ ] extend the shipped `SourceActivityView` pattern across remaining
  non-shell source surfaces when they gain sync, Takeout, migrated-history, or
  recovery controls
- [ ] finish saved-run filtering and cleanup affordances for large histories
- [ ] evolve media evidence cards only after media download and preview policy
  is approved

### 3.6 Desktop Product Maturity

Priority: medium.

Guidance: `docs/desktop-product-evolution-analysis.md`.

- [ ] define a shared redaction policy for logs, diagnostics, debug status text,
  and future support bundles
- [ ] add a local diagnostic summary or support bundle that reports app version,
  schema state, provider health, recent job states, and sanitized logs without
  source content or secrets
- [ ] introduce a lightweight settings registry/search layer before settings
  grow around media, privacy, export, and provider controls
- [ ] define an explicit export profile contract for NotebookLM, source-group,
  saved-run, and future archive/media exports
- [ ] add a small action registry for repeated analysis, source, export,
  settings, and diagnostics commands before adding shortcuts or command palette
  UI
- [ ] add versioned validation/repair for growing settings payloads such as LLM
  profiles, provider settings, export profiles, media policy, and privacy
  options
- [ ] decide on local privacy/data lifecycle controls such as app lock, archive
  deletion, and diagnostic privacy preview after the higher-value operability
  foundations are in place

Acceptance:

- Diagnostic output is useful for support and debugging without exposing local
  archive content, prompts, provider credentials, cookies, or session material.
- Settings and export behavior remain findable, explicit, and testable as the
  desktop app gains provider, media, privacy, and export options.
- Privacy controls describe real local data lifecycle behavior and do not imply
  stronger protection than the storage model provides.

## 4. Explicit Non-Goals

| Idea | Decision | Why |
| --- | --- | --- |
| Hexagonal architecture rewrite | do not do | too heavy for current scale |
| Telegram trait abstraction mainly for tests | do not do | too much indirection for weak payoff |
| Service-heavy frontend architecture | do not do | poor fit for this Svelte app |
| E2E-first expansion before core stabilization | do not do | lower ROI than targeted storage and logic tests right now |
| Splitting every large file automatically | do not do | only split where it lowers risk or unlocks backlog work |
