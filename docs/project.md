# Project State

This document is the shortest current-state snapshot of the repository.

## Stack

- frontend: `SvelteKit 2`, `Svelte 5`, `TypeScript`
- desktop shell: `Tauri 2`
- backend: `Rust`
- local storage: `SQLite`
- LLM: reusable Gemini and OpenAI-compatible provider profiles

## Verification

Run baseline full-project verification before committing or merging:

```bash
npm.cmd run verify
```

This command runs frontend tests, Svelte checks, Rust check/tests, and
`git diff HEAD --check`. It is a baseline local gate; CI, Rust formatting/lint
policy, and broader live Telegram/LLM event-flow validation remain separate
stabilization work.

<!-- daily-development-loop -->
For the daily loop after a small change, choose the narrowest applicable command:

```powershell
npm.cmd run test:changed
npm.cmd run test:changed:last
npm.cmd run test:related -- src/lib/some-model.ts
npm.cmd run test:rust -- prompt_packs::runtime::tests::load_run_runtime_config
```

The working-tree command sees uncommitted changes; the last-checkpoint command
uses `HEAD~1`, which means the first parent after a merge. Use
`npm.cmd run test -- --changed=<base>` when a different merge base is intended.
Changed/related selection follows the module graph and may be empty or
incomplete for dynamic relationships, so it is not a replacement for the full
`npm.cmd run verify` gate.

Canonical full Rust checks and tests use `--workspace --all-targets`. Focused
root-package filters select `-p extractum` explicitly. Every workspace member
shares `src-tauri/target`; avoid per-task target directories during sequential
development. Ordinary dev/test builds retain
workspace line tables and omit dependency debug information. For rare native
inspection of dependency variables, begin with a clean tree, temporarily set
both `[profile.dev] debug` and `[profile.dev.package."*"] debug` to `2`, point
`CARGO_TARGET_DIR` at an absolute isolated directory, and launch the usual
MCP-enabled `npm.cmd run tauri dev`. Restore the manifest afterward and never
commit that temporary profile change.

For the YouTube Summary / Prompt Pack project-runs slice, use the narrower
verification scripts while iterating:

```bash
npm run test:project-runs
npm run test:rust:prompt-pack-runs
npm run verify:project-runs
```

`test:project-runs` runs the focused Vitest contract/API tests.
`test:rust:prompt-pack-runs` runs the focused Rust prompt-pack run tests.
`verify:project-runs` composes those with the full Svelte/TypeScript check.
The first Rust run after a clean target can be slow because Cargo warms the
test target; subsequent runs are expected to be much faster.

`src-tauri/crates/extractum-prompt-packs/src/` owns Prompt Pack lifecycle,
YouTube Summary orchestration, validation, and prompt-pack-table persistence.
`src-tauri/src/prompt_packs/` is the private application compatibility facade:
it owns Tauri commands/events/spawning, pool acquisition, profile/secret
resolution, foreign source reads, and concrete Gemini Browser adapters.
Migrations and bundled assets remain app-owned at `src-tauri/migrations/` and
`src-tauri/prompt-packs/`.

For the local Python YouTube pipeline research prototype, use:

```bash
python -m unittest discover research/youtube_pipeline/tests
```

This covers the legacy direct-LLM strategy runner helpers and the deterministic
parts of the file-backed agentic workflow. It does not exercise live LLM
providers or Codex sub-agent orchestration.

## Secret/config policy

Commit shared source, docs, migrations, tests, and stable project config only.
Do not commit local runtime state, generated logs, SQLite databases, Telegram
session files, cookie exports, private keys, or `.env*` files.

Runtime secrets are intentionally split from repository files:

- saved LLM API keys, Telegram `api_hash` values, Telegram session encryption
  keys, and YouTube cookies live in OS secure storage;
- Telegram session files live under the Tauri app-data directory and are
  encrypted with per-account keys from OS secure storage;
- the live SQLite database is app runtime state, not a repository artifact;
- local MCP/tooling state such as `.codex*`, `.kilo`, `.superpowers`,
  `.playwright-mcp`, `.worktrees`, `tmp`, `artifacts`, and `kilo.json` stays
  ignored.

## Tauri security boundary

`npm.cmd run tauri dev` is the MCP-enabled development command. It invokes the repository wrapper, which applies the dev-only MCP overlay. Direct `npx tauri dev` is not an MCP workflow. Production and debug builds do not expose the global Tauri object, MCP, or fixture commands; the bridge binds only to `127.0.0.1` in development.

The frontend receives no SQL permissions. Rust owns database access. LLM credentials remain in OS secure storage and are bound to the provider plus normalized URL origin. Remote plaintext `http://` endpoints are rejected; only HTTPS plus localhost/loopback HTTP is accepted. Loading a keyed legacy profile materializes its effective URL into backend-owned settings and fails closed if the write fails.

To inspect the production CSP, build with `npm.cmd run tauri build -- --no-bundle --features csp-verification`. That verification-only feature enables DevTools; ordinary release builds do not.

Workspace-local live DB backups and validation snapshots are private artifacts
even when they are ignored by git. Keep them only as long as they are needed
for debugging or audit evidence, store durable copies outside the repository
when retention is required, and record only sanitized source ids, counters,
states, or warning codes in committed documentation.

## Dependency policy

The `grammers-*` crates are owned git dependencies because Extractum's
Telegram behavior depends on upstream runtime details. Treat updates to
`grammers-client`, `grammers-session`, and `grammers-mtsender` as explicit
dependency work, not incidental lockfile churn.

The historical GitHub repository
[`Lonami/grammers`](https://github.com/Lonami/grammers/) is archived and now
points to the canonical upstream at
[`codeberg.org/Lonami/grammers`](https://codeberg.org/Lonami/grammers). Treat
any migration from the old GitHub git URL to the Codeberg git URL as explicit
dependency work under this policy.

Current pinned upstream: Codeberg rev
`1f901ce6e973fdcf0e74267f3d8efad5c729daaa` for the `grammers-*` `0.9.0`
line. The older GitHub lock rev
`fa7692e49f301f16dc671c2f305ac1a32cad1a8e` was not available from the
Codeberg repository during migration.

Migration validation evidence recorded on 2026-06-01: the Codeberg-pinned
`grammers-*` line passed focused live Telegram sync smoke checks through the
running Tauri app on source `119` / `СтатусБанк` (`inserted = 0`,
`skipped = 0`, `last_message_id = 363`, no warnings) and source `122` /
`tools_ui_1c` (`inserted = 21`, `skipped = 3`, `last_message_id = 12262`, no
warnings). The evidence is intentionally limited to sanitized source ids,
counts, and sync-state values; no Telegram sessions, API hashes, or raw
private payloads are recorded here.

For any `grammers-*` update:

- update the related `grammers-*` crates together unless there is a documented
  reason to split them;
- record the old and new upstream commit revisions from `Cargo.lock`;
- explain why the update is needed and whether it affects Telegram sync,
  Takeout import, session handling, or source identity behavior;
- run `npm run verify` at minimum, plus focused Telegram validation when the
  upstream change touches runtime behavior;
- keep the update in a dedicated dependency commit or clearly isolated slice.

Do not refresh `grammers-*` from the upstream branch as part of unrelated
feature, fix, formatting, or documentation work.

## Frontend component policy

New product-facing Svelte screens must use Extractum-owned wrapper components
from `src/lib/components/extractum-ui/*` for shadcn-svelte and SVAR behavior.
Feature screens should not import raw shadcn-svelte or SVAR widgets directly;
direct imports are reserved for wrapper components, low-level wrapper tests, or
explicit short-lived experiments.

SVAR Grid usage must go through the Extractum grid wrapper layer. The wrapper
owns the stable height container, stable row ids, wrapper-managed selection,
empty states, density, theme bridge, and any narrowly scoped `.wx-*` selector
overrides. Feature screens should pass product-facing props and view-model rows
instead of reading SVAR API state ad hoc.

Add or keep raw-source import-boundary tests for new UI slices that use shadcn
or SVAR so feature screens continue to depend on `extractum-ui` wrappers rather
than lower-level library imports.

## Product slice

The app is a local source ingest and analysis workspace. Telegram and YouTube
are implemented ingest providers today, while RSS/forum remain future provider
families behind the shared source model.

Implemented:

- result-first `/analysis` workspace with compact source rail, central report/source canvas, shared workspace tools for setup and opened runs, and evidence/chat/chunks/runs companion panel
- Source Browser for live Telegram sources, YouTube videos, YouTube playlists,
  live source groups, and available saved run snapshots, with provider-aware
  default tabs, playlist `Videos`, group `Sources`, frozen snapshot browsing,
  universal loaded item browsing, YouTube comments, structured metadata, and
  consolidated live source Activity
- typed event-driven `/analysis` workspace UI state transitions, ready for a future state-machine library if the workflow outgrows the local reducer
- collapsible desktop app sidebar and mobile off-canvas navigation drawer
- Telegram account management and sign-in flow
- startup session restore
- source management for Telegram channels, supergroups, and groups
- source management for YouTube videos and playlists
- provider-ready source records with `source_type` and `source_subtype`
- capability-driven source UI for Telegram sync, Takeout, membership, topics, and YouTube sync actions
- typed Telegram source identity and typed Telegram message identity for duplicate detection and legacy ref resolution
- typed YouTube video/playlist source metadata outside the generic source metadata blob
- Telegram history sync into local SQLite
- YouTube metadata, transcript, comment, and playlist membership sync into local SQLite
- provider-dispatched source sync for Telegram and YouTube
- Takeout source import for existing Telegram sources with TDesktop-first pagination
- explicit migrated small-group history import for Telegram supergroups when
  Takeout has detected the historical scope
- durable ingest batch, warning, and item-observation provenance for Takeout
  attempts
- representative Takeout validation coverage for public, private/dialog-backed,
  private/left fallback, richer small-group, duplicate/fidelity,
  migrated-history, and export-DC fallback scenarios
- explicit migrated-history scope controls for browsing, NotebookLM export, and
  analysis, with current-history-only defaults
- materialized Telegram forum topic memberships with source-level resolver state
- media-aware sync metadata for text-bearing and media-only items
- Telegram reply/thread/reaction context metadata for newly synced items
- configurable initial sync window
- source groups for analysis
- saved reports
- follow-up chat on saved runs
- analysis report launch guards for missing source context, unusable provider profiles, source runtime problems, and large selected corpora
- single-source and Telegram source-group NotebookLM export with local reply/thread/reaction metadata
- reusable LLM provider profiles with active-profile selection
- configurable OpenAI-compatible `base_url` support in `/settings`
- provider smoke testing from `/settings`
- YouTube cookie/settings controls in `/settings`
- read-only `/diagnostics` operator surface for sanitized local health, runtime,
  provider, source, ingest, and privacy-boundary summaries
- immutable saved run corpus snapshots
- provider-neutral analysis refs for new live corpus rows
- YouTube timestamp evidence refs for transcript segments
- Prompt Pack `youtube_summary` MVP runs with preflight, deterministic
  source/material snapshots, combined transcript-analysis execution, stage
  artifacts, canonical result persistence, audit events, validation findings,
  and YouTube-specific result projections
- local Python YouTube summary research prototype under
  `research/youtube_pipeline`, including legacy direct-LLM strategies and a
  file-backed agentic MoC workflow. This prototype is not production-integrated
  into the Tauri app; normal product YouTube Summary runs use Prompt Packs.
- YouTube Summary remembers the last local API/Gemini Browser runtime and browser mode, and its launch button names the provider that will receive the run.
- dedicated `/projects/runs` Prompt Pack runs screen with SVAR grid browsing,
  run label update, confirmed terminal run deletion, confirmed active run
  cancellation, and a from-scratch report workspace for videos, claims,
  evidence, warnings, validation findings, audit events, artifacts, and
  canonical JSON
- optional YouTube comment enrichment on generic source item rows for direct
  comment browsing without a separate comments pagination endpoint
- source-level YouTube metadata detail, including bounded raw metadata JSON for
  the live Metadata tab
- typed app errors across Tauri commands
- guarded audit and clear commands for eligible legacy Telegram source metadata
  blobs
- OS secure storage for saved LLM API keys and Telegram `api_hash` values
- encrypted Telegram session file contents with per-account OS secure storage keys
- OS secure storage for YouTube cookies

Not implemented yet:

- RSS or forum ingestion
- full media download / previews
- media-aware analysis beyond the current text-first corpus
- YouTube-specific NotebookLM export enrichment
- persistent/resumable YouTube sync jobs across app restart
- richer Telegram Forum Topics browsing/export beyond the current materialized membership filters
- Telegram forward metadata enrichment
- production integration of the research-only agentic YouTube summary workflow

## Main routes

- `/accounts`
  - create and delete local Telegram accounts
  - observe runtime status updates
- `/auth/[id]`
  - initialize Telegram login
  - send code
  - sign in
  - log out
- `/sources`
  - compatibility redirect for older entry paths
  - sends users to the main analysis workspace
- `/settings`
  - manage reusable LLM provider profiles
  - set the active profile used by default
  - edit provider-specific `base_url` settings for OpenAI-compatible providers
  - refresh available models
  - run a live provider smoke test with the currently edited form
  - configure YouTube cookies and runtime settings
- `/diagnostics`
  - display the sanitized `get_diagnostic_summary` backend contract
  - show app/build, SQLite/migration, secure storage, `yt-dlp`, provider,
    source, item, run, LLM request, YouTube job, ingest, and privacy-boundary
    aggregates
  - refresh manually without polling, raw JSON, log viewing, copy actions, or
    support-bundle export
- `/projects`
  - manage durable research projects and project-source membership
- `/projects/library`
  - browse Library sources through the project-oriented navigation surface
  - launch YouTube Summary runs for synced YouTube video/playlist sources
- `/projects/runs`
  - browse Prompt Pack project runs through the Extractum SVAR grid wrapper
  - update optional run labels
  - delete terminal Prompt Pack runs after confirmation
  - cancel active Prompt Pack runs after confirmation
  - inspect Prompt Pack report components without using the legacy analysis
    report viewer
- `/analysis`
  - use the result-first research workspace layout
  - keep NotebookLM export, template editing, and group editing reachable from
    shared canvas-level workspace tools in setup and opened-run states
  - switch between report output/setup and source material in the central canvas
  - switch source context through the compact analysis rail
  - inspect evidence, follow-up chat, live chunk summaries, and saved runs in the companion panel
  - browse live Telegram sources through Timeline, Items, Metadata, and
    Activity tabs
  - browse live YouTube videos through Transcript, Comments, Items, Metadata,
    and Activity tabs
  - browse live YouTube playlists through Videos, Items, Metadata, and Activity
    tabs
  - browse live source groups through Sources, Items, Metadata, and Activity
    tabs
  - browse available saved run snapshots through frozen snapshot tabs without
    live source actions
  - add Telegram sources manually or from dialogs
  - add YouTube videos and playlists by URL
  - sync Telegram source history
  - sync YouTube metadata, transcripts, comments, and playlists
  - start/cancel Takeout source imports and monitor import progress
  - start explicit migrated small-group history imports for eligible Telegram
    supergroups
  - configure the first sync policy
  - manage report templates
  - manage source groups
  - run reports
  - monitor active queued/running reports separately from history
  - browse saved runs through global history or the current analysis scope
  - inspect trace refs
  - ask follow-up questions against saved runs

## Backend command areas

### Accounts / auth

- account CRUD
- `tg_init`
- `tg_send_code`
- `tg_sign_in`
- `tg_logout`
- runtime account status refresh / restore

### Sources

- `list_telegram_sources`
- `add_telegram_source`
- `preview_youtube_source`
- `add_youtube_source`
- `list_sources`
- `delete_source`
- `sync_source`
- `get_youtube_runtime_status`
- `list_youtube_source_summaries`
- `get_youtube_video_detail`
- `get_youtube_playlist_detail`
- YouTube source-job commands for metadata, transcript, comments, playlist sync, retry, cancel, and listing
- `start_takeout_source_import`
- `start_takeout_migrated_history_import`
- `cancel_takeout_source_import`
- `list_takeout_source_import_jobs`
- `list_source_items`
- `get_sync_settings`
- `save_sync_settings`
- `audit_legacy_telegram_source_metadata`
- `clear_legacy_telegram_source_metadata`

### Analysis

- report generation
- active runs listing and restoration
- saved runs listing, scoped/global history browsing, and detail loading
- trace resolution
- follow-up chat
- prompt template CRUD
- source group CRUD

### Prompt Packs

- list Prompt Pack library entries and versions
- run YouTube Summary preflight
- start and cancel YouTube Summary Prompt Pack runs
- list recent and active Prompt Pack runs
- update optional Prompt Pack run labels
- delete terminal Prompt Pack runs
- list Prompt Pack stage runs, stage artifacts, audit events, validation
  findings, and canonical results

### Settings / LLM

- load and save LLM profiles
- switch the active LLM profile
- list provider models for Gemini and OpenAI-compatible endpoints
- stream provider test requests and analysis/chat requests through the resolved profile

### Diagnostics

- `get_diagnostic_summary`

## Important persistence

- `accounts`: local Telegram account metadata; saved Telegram `api_hash` secrets live in OS secure storage
- Telegram session files remain app-data files, but their contents are encrypted with per-account session keys stored in OS secure storage under `telegram.account.<account_id>.session_key`.
- `sources`: registered provider sources with provider-local `source_subtype`
  values and shared sync state
- `telegram_sources`: typed Telegram peer identity, resolution hints, and
  display cache fields
- `telegram_messages`: typed Telegram message identity and message context for
  Telegram item rows
- `youtube_video_sources` and `youtube_playlist_sources`: typed YouTube
  runtime metadata for registered YouTube sources
- `items`: ingested source items; currently Telegram messages, YouTube
  transcripts, and YouTube comments with provider item kinds
- `item_topic_memberships`: materialized real Telegram forum topic memberships
  for items
- `telegram_topic_resolution_state`: source-level state for forum topic
  membership freshness and unresolved counts
- `youtube_playlist_items`: YouTube playlist membership and availability rows
- `youtube_transcript_segments`: timestamped caption/transcript cues
- `ingest_batches`, `telegram_takeout_batches`, `ingest_item_observations`,
  and `ingest_batch_warnings`: durable ingest/Takeout provenance for started
  locked import attempts
- `analysis_documents`: provider-neutral read model for live analysis corpus
  loading
- `archive_read_model_state` and `archive_read_items`: source-scoped readiness
  and provider-neutral archive rows for browsing and Telegram NotebookLM export
- Takeout import job progress records are in-memory runtime state
- no persistent table exists for YouTube source jobs; job records are in-memory runtime state
- `app_settings`: app-level key/value storage, including active LLM profile, per-profile non-secret provider metadata, and sync policy
- `analysis_runs`: saved report runs
- `analysis_run_messages`: frozen corpus snapshot for saved runs
- `analysis_chat_messages`: follow-up chat history
- `prompt_pack_versions`, `prompt_pack_stage_templates`, and
  `prompt_pack_schema_assets`: bundled Prompt Pack library, stage prompt
  templates, and schema assets
- `prompt_pack_runs`: Prompt Pack run headers, status/progress, request and
  preflight payloads, model/config choices, and optional user-owned run labels
- `prompt_pack_run_scopes`, `prompt_pack_run_source_snapshots`,
  `prompt_pack_run_source_origins`, and `prompt_pack_run_material_snapshots`:
  deterministic run input boundary for YouTube sources, playlist expansion,
  inclusion/skipping reasons, transcripts, descriptions, and comments
- `prompt_pack_stage_runs` and `prompt_pack_stage_artifacts`: stage execution
  status plus compressed prompt/raw/parsed/metrics/error artifacts
- `prompt_pack_results` and `prompt_pack_result_*`: canonical result JSON plus
  queryable source refs, claims, evidence, warnings, limitations, quality
  flags, validation findings, audit refs, and YouTube-specific projections

## LLM scheduling and analysis caps

LLM scheduling allows two running requests per `(provider, profile)` and prioritizes interactive requests over background work. Analysis report runs run a backend preflight before run creation and are capped at `10_000` messages, `80` estimated chunks, `1_500_000` estimated input characters, and `80` background requests.

## Current practical constraints

- analysis corpus still requires text content;
- media-only items are stored and visible, but not yet analyzed;
- RSS and forum ingestion commands are not implemented yet;
- YouTube analysis is text-based and uses synced transcripts, synthetic descriptions, and comments; audio/video binaries are not downloaded;
- YouTube source jobs are process-local and are not resumed after app restart;
- YouTube support requires `yt-dlp` on `PATH`;
- the `/diagnostics` page is a read-only, manually refreshed summary surface;
  it intentionally does not expose raw JSON, logs, support bundles, copy
  actions, frontend environment probes, source titles, URLs, provider profile
  labels, local paths, source content, prompts, credentials, cookies, or
  session material;
- older item rows may have `NULL` Telegram context metadata because there is no
  background backfill; mutable Telegram metadata added after ingest, such as
  reactions on already-observed messages, is not backfilled by duplicate
  observations;
- saved LLM API keys and Telegram `api_hash` values use OS secure storage;
- YouTube cookies, when enabled, use OS secure storage and are written only to temporary backend cookie files for `yt-dlp`;

## External process lifecycle

`yt-dlp`, the Gemini sidecar, and Extractum-started CDP Chrome are owned backend processes. New starts are rejected once shutdown begins. On Windows each owned child is assigned to a Job Object; this contains descendants created after assignment, but cannot retroactively contain processes created before assignment.

Application exit closes external-process admission and starts one shared three-second graceful deadline at the first accepted exit request. Waiting for in-progress spawn/install permits and concurrent cleanup of YouTube, the Gemini sidecar, and owned Chrome all consume that same budget; an independent OS-thread watchdog enforces an approximately four-second hard cap with the original exit code. Gemini uses one Tokio JSONL transport in development and packaged builds; the packaged binary is resolved beside the executable and must remain declared in `bundle.externalBin`. A cancelled Gemini request taints its transport, so shutdown skips protocol `Stop` and terminates the owned process instead.
- Telegram session files remain app-data files, but their contents are encrypted with per-account session keys stored in OS secure storage under `telegram.account.<account_id>.session_key`;
- Telegram peer resolution can still fall back to dialog scanning, especially for private sources.
- Takeout import does not download media bytes.
- Normal Takeout imports current supergroup history by default; migrated
  small-group history is imported only through the explicit historical-scope
  action and remains excluded from default projections unless the user opts in
  where supported.

## Reading order for implementation work

1. `src-tauri/src/sources/mod.rs`
2. `src-tauri/src/source_ingest.rs`
3. `src-tauri/src/youtube/`
4. `src-tauri/src/takeout_import/mod.rs`
5. `src-tauri/src/takeout_import/raw_parse.rs`
6. `src-tauri/src/analysis/`
7. `src-tauri/src/llm/`
8. `src-tauri/src/diagnostics/`
9. `src-tauri/crates/extractum-prompt-packs/src/`
10. `src-tauri/src/prompt_packs/` for application adapters and command facade
11. `src/routes/projects/`
12. `src/lib/components/research-projects/`
13. `research/youtube_pipeline/` for research-only YouTube summary pipeline work
14. `src/routes/analysis/+page.svelte`
15. `src/lib/components/analysis/`
16. `src/routes/settings/+page.svelte`
17. `src/routes/diagnostics/+page.svelte`
18. `src/lib/diagnostics-view-model.ts`
19. `src/routes/sources/+page.svelte`
20. `src-tauri/src/error.rs`
21. `src-tauri/src/migrations.rs`

Related deep dive: `docs/takeout-source-import.md`.

Historical verification notes live under
`docs/superpowers/archive/verification/`.
