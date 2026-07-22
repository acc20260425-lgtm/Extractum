# Analysis Crate Boundary Design

**Status:** Draft for owner review; implementation not started
**Date:** 2026-07-22

**Roadmap authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

**Verification-loop authority:**
[`2026-07-17-focused-rust-loop-design.md`](2026-07-17-focused-rust-loop-design.md)

This specification defines the just-in-time Phase 7 boundary for
`extractum-analysis`. It supersedes the short Phase 7 placeholder in the crate
roadmap after owner approval. It does not authorize implementation, change the
retained Phase 4–6 boundaries, or authorize Phase 8.

The design was informed by the local, intentionally untracked audit
`reference/2026-07-20-crate-extraction-refactoring-analysis.md`. Its conclusions
were refreshed against current `HEAD`, and this specification is self-contained
so it does not depend on that reference file remaining present.

## Purpose

Phase 7 extracts analysis-domain models, state, report/chat execution,
snapshot and trace behavior, and analysis-owned persistence into
`extractum-analysis`. The application remains the composition root for Tauri,
credentials, source/project resolution, foreign source reads, task spawning,
events, migrations, and cross-domain coordination.

The extraction is an ownership and compilation-boundary change. It must not
redesign analysis UX, prompts, map/reduce behavior, persistence values, IPC,
source ingestion, schema, or task supervision. Preparatory refactors are made
while code is still application-owned; the final physical move is mechanical.

## Decision

The selected design is a SQL-owning analysis crate behind a private
application facade:

1. `extractum-analysis` owns analysis behavior and all runtime SQL for exactly
   six analysis tables;
2. the app obtains and passes `SqlitePool`; migrations remain app-owned;
3. the app resolves source, source-group, and project scopes into an owned
   `ResolvedAnalysisScope` value;
4. the app implements a narrow `AnalysisCorpusReader` that returns owned
   corpus DTOs without exposing source/project schema to the crate;
5. `extractum-analysis` directly uses `extractum-llm` with explicitly passed
   scheduler and resolved profile capabilities;
6. the app owns detached task spawning and maps typed domain events to the
   existing Tauri channels through `AnalysisEventSink`;
7. the portable `AnalysisState`, internal map `JoinSet`, cancellation policy,
   snapshot behavior, and terminal outcome classification belong to the crate;
8. `src-tauri/src/analysis/mod.rs` remains a private, explicit compatibility
   facade, while the public crate root exposes a curated API with no public
   submodules or glob exports.

This is the owner-selected clean-boundary option. It deliberately rejects both
a quick physical move that retains foreign SQL and a smaller engine-only crate
that leaves the domain split across packages.

## Alternatives Considered

### Fast physical move with foreign SQL

Moving most of `src-tauri/src/analysis` as-is would be shorter, but the crate
would continue to query `sources`, `projects`, `project_sources`,
`analysis_documents`, Telegram, and YouTube tables. It would preserve the
current source/analysis coupling and create a likely reverse edge when a future
`extractum-sources` crate is extracted.

Rejected: foreign schema stays behind application adapters.

### Engine-only extraction

Moving only report/chat orchestration and trace handling would avoid the
largest SQL seams. It would also leave analysis persistence, state, lifecycle,
and DTO ownership in the application, producing two competing analysis
domains and a weaker focused loop.

Rejected: the crate owns the coherent analysis domain and its six tables.

### Generic repository or unit-of-work port

Replacing all SQLx operations with a broad repository interface would make
the crate superficially persistence-neutral, but would duplicate a large API,
obscure transaction ownership, and make tests depend on repository mocks
rather than the real SQLite contract.

Rejected: analysis-owned SQL stays concrete; only foreign reads use narrow
typed seams.

## Fresh Evidence Snapshot

The refreshed snapshot was taken on 2026-07-22 at
`d8ea06dae71e42aea4919ffa885bdc59cef7374e` with a clean worktree.

- `src-tauri/src/analysis` contains 54 Rust files and 13,187 physical lines.
- Twenty-six non-dedicated-test files account for 7,745 lines, including the
  1,392-line dev fixture implementation. Twenty-eight dedicated test files
  account for 5,442 lines.
- The executable inventory command
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::' -- --list`
  reports exactly 143 tests. Strict subject-under-test ownership assigns 95 to
  the crate and 48 to the app; the app set contains 30 concrete foreign-SQL
  integration identities plus 18 dev-fixture identities. Appendix A is
  normative.
- Since 2026-06-01, 49 commits touched analysis. Forty-one categorized commits
  touched no other Rust domain (83.7%). Joint touches were led by
  `prompt_packs` 4, `llm` 4, `lib` 3, `projects` 3, and `youtube` 2.
- The current surface contains 21 release analysis Tauri commands, three dev
  fixture commands, and three project commands that consume analysis behavior.
- Tauri/AppHandle leakage appears in nine release files and one dev-fixture
  file. `ReportPipelineContext` currently uses `AppHandle` as a pool, state,
  scheduler, and event service locator.
- The report pipeline performs two live-corpus reads: one for synchronous
  preflight and one after spawn for the authoritative saved snapshot.
- `analysis/trace.rs` contains the app's remaining direct `zstd::` calls.
- Production code outside `analysis` directly reaches analysis internals or
  tables from projects, account deletion, diagnostics, and NotebookLM export.

The co-change data supports a physical boundary, while the runtime and SQL
inventories require preparation before the move.

## Reference Audit Disposition

The design adopts the audit's value/capability/port rule:

- pass resolved scope and resolved profile as owned values;
- pass `SqlitePool`, `LlmSchedulerState`, and `AnalysisState` as concrete
  capabilities used by the domain;
- use ports only for repeated app-owned corpus and event side effects;
- keep Tauri and cross-domain coordination in the application.

It also adopts the audit's recommendations to remove `AppHandle` from the
pipeline, keep `analysis_documents` source-owned, move direct trace compression
to `extractum-core`, and expose curated APIs rather than modules.

One possible audit optimization is explicitly deferred: Phase 7 preserves the
two current live-corpus reads rather than materializing once before spawn.
Changing the read count would alter race, failure, and snapshot-timing
semantics and needs a separate owner-approved change.

## Target Dependency Structure

```text
extractum
  |-- Tauri commands, registration, startup and detached spawn
  |-- get_pool and migration registration
  |-- profile/credential resolution
  |-- scope/source/project adapters
  |-- AppAnalysisCorpusReader
  |-- TauriAnalysisEventSink
  |-- dev fixtures and cross-domain coordinators
  `-- extractum-analysis
        |-- analysis DTOs, validation and state
        |-- analysis-owned SQL persistence
        |-- snapshot, trace, report and chat execution
        |-- map/reduce JoinSet and cancellation policy
        |-- extractum-llm
        `-- extractum-core
```

There is one production dependency edge from `extractum` to
`extractum-analysis`. There is no reverse edge and no upward dependency from
`extractum-core` or `extractum-llm`.

`extractum-analysis` must not depend on `extractum-prompt-packs`, Tauri, a
future source crate, or any application module. A future lower source/read
model crate may replace the app corpus adapter only through a separate design.

## Ownership Boundary

### Crate-owned behavior

`extractum-analysis` owns:

- domain and wire-compatible analysis DTOs;
- validation and normalization that do not require app state;
- `AnalysisState` and report cancellation tokens;
- prompt-template and source-group domain behavior;
- report preparation after app-owned inputs have been resolved;
- preflight calculations, chunking, map/reduce orchestration, request building,
  parsing, trace construction, and terminal outcome classification;
- chat context selection, request construction, LLM execution, and chat
  persistence;
- immutable run snapshot persistence and reads;
- CRUD and read models over the six owned tables;
- typed run/chat events sent through `AnalysisEventSink`;
- diagnostic aggregation derived only from analysis-owned rows.

The crate owns internal concurrency. `JoinSet` is portable domain
orchestration, not Tauri lifecycle behavior, and moves with report execution.

### Application-owned integration

`extractum` retains:

- every `#[tauri::command]`, command registration, `AppHandle`, `tauri::State`,
  and app startup hook;
- `get_pool` and all migration registration;
- LLM profile and credential resolution;
- detached report/chat spawn boundaries;
- Tauri channel emission and best-effort emit-error handling;
- source identity readiness and source/project/playlist resolution;
- SQL against `analysis_documents` and every other foreign table;
- project deletion and account deletion coordination;
- NotebookLM and diagnostic composition;
- all three dev fixture commands and their cross-domain data;
- full-schema and cross-domain integration tests.

The app facade preserves existing `crate::analysis` consumer paths where
practical, but preserving an import path does not authorize public modules or
struct-field widening.

## Schema Ownership

### Analysis-owned tables

The crate owns all production queries, DML, row mapping, and transaction
semantics for exactly these six tables:

1. `analysis_runs`
2. `analysis_run_messages`
3. `analysis_chat_messages`
4. `analysis_prompt_templates`
5. `analysis_source_groups`
6. `analysis_source_group_members`

The boundary contract uses this exact allowlist. A table-name prefix is not an
acceptable substitute.

Foreign keys from these tables to app-owned identities remain intact. The
crate stores those IDs as opaque domain references; it does not acquire
ownership of the referenced tables.

### App/source-owned tables

The following remain outside the crate:

- `analysis_documents`;
- `sources` and `items`;
- `telegram_messages`;
- `youtube_playlist_items`, `youtube_video_sources`, and
  `youtube_transcript_segments`;
- `projects` and `project_sources`;
- all other source, provider, settings, account, and prompt-pack tables.

`analysis_documents` is a rebuildable source projection. Source and YouTube
ingestion update it transactionally with producer-owned writes. Moving it into
analysis would create the future reverse dependency `sources -> analysis`
while analysis still consumes source data. Its builders, rebuild functions,
indexes, and write-side tests therefore remain app-owned.

### Migration owner

`src-tauri/src/migrations.rs`, `src-tauri/migrations/**`, checksums, startup
registration, and upgrade behavior remain in the application. Existing
migrations are not moved, rewritten, renamed, or duplicated. Phase 7 adds no
production migration.

Cross-domain FK and cascade behavior remains an app integration concern. For
example, project deletion keeps one app-owned transaction: the coordinator
starts the SQLx transaction, calls a curated analysis transaction function to
delete project runs, deletes project-owned rows, and commits. This preserves
atomicity without leaving raw analysis-table SQL in `projects`.

## Resolved Scope Boundary

The app resolves a source, source group, or project into an owned value before
report preparation. The required semantic shape is:

```rust
pub struct ResolvedAnalysisScope {
    scope_kind: AnalysisScopeKind,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
    source_kind: AnalysisSourceKind,
    source_ids: Vec<i64>,
    scope_label_snapshot: String,
    skipped_unlinked_playlist_items: usize,
}
```

Fields remain private. Constructors enforce exactly one scope identity,
non-empty resolved source IDs, stable source ordering, and a non-empty fallback
label. Same-provider membership is rechecked for projects during report-scope
resolution and for groups only during group create/update, exactly as today.
Report start trusts an already-persisted group's invariant and must not add a
new rejection path for a legacy or corrupt mixed group. Accessors expose only
values needed by execution and persistence.

The app resolver owns:

- source/project existence checks;
- `project_sources` reads;
- the command-specific source identity readiness policy: the existing gate on
  `list_analysis_sources` remains, while report and project-report start do not
  gain a new readiness gate;
- YouTube playlist expansion and exclusion of removed/unlinked rows;
- source type and subtype interpretation;
- foreign source titles and project names;
- enrichment of source-group members with titles and item counts.

The crate owns source-group IDs, names, declared source type, membership rows,
and group CRUD. Before a group write, the app resolves and validates the
foreign source identities, then passes typed member values to the crate. When
reading a group, the crate returns owned membership IDs and the app enriches
the IPC DTO. The serialized group response remains unchanged.

## Corpus Read Boundary

The crate defines an object-safe, domain-specific reader with this required
ABI:

```rust
pub type AnalysisPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

pub trait AnalysisCorpusReader: Send + Sync + 'static {
    fn load_corpus(
        &self,
        request: AnalysisCorpusRequest,
    ) -> AnalysisPortFuture<'_, Vec<AnalysisCorpusMessage>>;
}
```

The request and result are fully owned crate DTOs. The port does not expose
SQLx queries, application rows, `AppHandle`, source modules, or test support.

`AppAnalysisCorpusReader` owns the current foreign reads:

- live documents from `analysis_documents`;
- migrated Telegram fallback from `items`, `sources`, and
  `telegram_messages`;
- source metadata needed for YouTube descriptions and transcripts;
- document-kind filtering, ordering, evidence refs, and metadata decoding.

`YoutubeCorpusMode` is exported as a typed parser/value. The current
`QueryBuilder` helper for document-kind predicates remains private to the app
adapter and must not become public crate API.

## Run Labels and Foreign Search Compatibility

Current run reads join live source and project labels, while
`scope_label_snapshot` is the authoritative display fallback. Removing those
joins must not change IPC fields, legacy-label fallback, query matching, or the
rule that filtering occurs before `LIMIT`.

Preparation therefore introduces typed value inputs for foreign label
enrichment and per-search-term matching:

- the app queries matching source/project IDs for normalized search terms;
- the crate combines those ID sets with predicates over its owned run, group,
  and template fields;
- the crate applies ordering and limit only after all predicates;
- the app supplies current source/project labels for returned IDs;
- the final domain DTO prefers the stored snapshot for `scope_label`, while
  preserving existing `source_title` and `project_name` fields and legacy
  fallbacks.

Fetching a broad page and filtering it in memory after `LIMIT` is forbidden.
Foreign matching, the crate-owned run query, and returned-label enrichment run
on the same app-opened SQLite read transaction/connection. This preserves the
single-snapshot behavior of the current JOIN if a source or project is renamed
or deleted concurrently.

The implementation plan must characterize multi-term matching, escaped `%`,
`_`, and `\\`, deleted scopes, empty labels, and snapshot precedence before
changing this query.

## Report Data Flow

The synchronous report-start order remains observable and must be preserved:

1. normalize request values and validate period, language, and exactly one
   scope;
2. obtain the pool and load the report template;
3. resolve the LLM profile, effective model, and input limit;
4. resolve the app-owned scope into `ResolvedAnalysisScope`;
5. call `AnalysisCorpusReader` for live-corpus preflight;
6. validate corpus size and model limits;
7. resolve duplicate active-run behavior, including stale persisted runs;
8. insert the queued run with `scope_label_snapshot` and register it in
   `AnalysisState`;
9. return a prepared execution ticket to the app;
10. the app starts one detached task and immediately returns `run_id`.

The execution ticket is an in-memory, non-serializable value. It may contain a
resolved secret-bearing LLM profile, but it exposes no secret getter and is
never logged or persisted.

Inside the spawned task, the app preserves the current post-spawn pool lookup
before entering the crate future. Terminal failed/capture-failed/cancelled
handlers also preserve their current app-side pool lookup points and
best-effort persistence. This retains their existing failure branches and
messages. The crate then preserves the current pipeline:

1. check cancellation and mark the run `running`;
2. emit `started/load_items`;
3. call the same corpus reader a second time;
4. persist exactly this second result to `analysis_run_messages` and reload the
   authoritative snapshot;
5. emit chunking progress and run map workers in a `JoinSet`;
6. preserve input chunk order when collecting map results;
7. run reduce through the background LLM scheduler;
8. build and compress trace data;
9. persist the final report/trace/status before emitting `completed`;
10. route typed terminal outcomes and always remove active state.

The two reader calls are intentional. Preflight validates read A; the frozen
snapshot and all later report/chat/trace behavior use read B. A one-read
optimization is a non-goal.

## Chat Data Flow

The chat command preserves this order:

1. trim and validate the question;
2. load the run and require `completed` with a saved report;
3. load only the captured snapshot and grounded context;
4. load and validate saved chat history;
5. construct the `analysis-chat-*` request and request ID;
6. start a detached app task and immediately return `request_id`.

Profile resolution intentionally remains inside the spawned task. A missing or
invalid profile therefore remains an asynchronous `failed` event after the
command has successfully returned its ID. Report profile resolution remains a
synchronous command error before run creation.

After profile resolution, the crate uses `extractum-llm` directly with
interactive priority and streams typed chat events. After completion, the app
performs the same second pool lookup as today and passes that pool to the
crate-owned chat transaction. Failure retains the exact `Answer completed but
chat history could not be saved: ...` event text. The crate persists the user
and assistant turns in one transaction and emits `completed` only after that
transaction succeeds. It never receives `AppHandle`.

## Runtime, Events, and Cancellation

### Spawn ownership

The app owns the outer detached task because accepting background work is an
application lifecycle decision. The crate returns prepared inputs and owns the
future executed inside that task. It does not introduce a new supervisor,
global runtime container, or service locator.

### Event sink

`AnalysisEventSink` accepts typed run and chat events synchronously through
this required ABI:

```rust
pub trait AnalysisEventSink: Send + Sync + 'static {
    fn publish_run(&self, event: AnalysisRunEvent);
    fn publish_chat(&self, event: AnalysisChatEvent);
}
```

The app implementation maps them to the existing payload DTOs and channels:

- `analysis://run`
- `analysis://chat`

The sink is infallible at the domain boundary because current Tauri emit
errors are ignored. No event failure changes persisted status or terminal
cleanup.

The following remain exact:

- run event kinds `queued`, `started`, `progress`, `delta`, `completed`,
  `failed`, and `cancelled`;
- report phases `load_items`, `chunking`, `map`, `reduce`, and `persist`;
- chat event kinds `queued`, `started`, `delta`, `completed`, `failed`, and
  `cancelled`;
- field names/casing, optional-field behavior, messages, progress values,
  request IDs, and ordering;
- request ID families `analysis-map-*`, `analysis-reduce-*`, and
  `analysis-chat-*`.

### Cancellation

`AnalysisState` moves as a portable type and remains managed by Tauri in the
app. The crate owns its token map and child-token behavior.

`cancel_analysis_run` keeps the current rules:

- only `queued` or `running` runs may be cancelled;
- both the run token and scheduler requests for the run owner are cancelled;
- a run that is persisted active but absent from both state and scheduler
  retains the current conflict message;
- map/reduce preserve both child-token and `LlmRequestControl` cancellation;
- chat gains no new analysis-specific cancel command;
- startup cleanup silently marks interrupted queued/running runs without
  emitting an event.

Terminal cleanup removes the run from active state even if terminal status
persistence or event publication fails. The implementation must characterize
this cleanup path before moving it.

## Error Strategy

`extractum_core::AppError` and `AppResult` remain the command, validation,
storage, and IPC error contract. Existing `kind` and `message` serialization
must not change.

Execution uses a typed crate-owned terminal classification equivalent to:

```rust
pub enum AnalysisExecutionError {
    Cancelled(String),
    CaptureFailed(String),
    Failed(String),
}
```

This type is not a new wire value. It exists so adapters map outcomes to the
existing statuses and events without string-prefix classification.

The design intentionally differs from the Gemini Browser domain error style:
analysis already uses `AppError` pervasively for command/storage semantics,
while its extra type represents recoverable lifecycle outcomes rather than a
new external error taxonomy. Future phases should use `AppError` when its
taxonomy is sufficient and introduce a domain error only for additional
owned lifecycle outcomes.

Provider and snapshot errors keep existing sanitization. Error text emitted or
persisted by timeout, cancellation, capture, provider, JSON, compression, and
chat-persistence paths is characterized before extraction.

## Cross-Domain Consumers

Direct production SQL against the six owned tables is removed outside the new
crate. Each consumer receives a narrow API/DTO rather than access to public
modules or internal rows.

### Projects

- `start_project_analysis` uses a project-scoped request constructor and the
  same app report coordinator as `start_analysis_report`.
- `list_project_runs` uses a project-scoped filter constructor.
- `get_project_data_range` uses the typed corpus mode and app scope/source
  adapter; it does not import SQL helpers from analysis.
- project-list run aggregates come from a batch analysis API and are composed
  with project rows app-side.
- project deletion calls a transaction-scoped analysis delete API so its
  current cross-domain transaction remains atomic.

### NotebookLM export

The crate returns the owned group record and ordered member IDs. NotebookLM's
app adapter resolves source titles/types and constructs its export DTO. It no
longer queries group tables directly.

### Account deletion

The crate exposes the existing active-run dependency query over direct source
and group membership. The app supplies active run IDs and account-owned source
IDs. `AnalysisState::active_report_run_ids` becomes a public read-only
accessor; token mutation remains behind lifecycle APIs.

The current dependency check does not associate project-scoped runs with a
project's source set. Fixing that blind spot requires additional state or a
scope-source snapshot and is explicitly deferred. Phase 7 preserves the
current behavior and records a follow-up rather than silently changing account
deletion semantics.

### Diagnostics

Analysis-run aggregation moves to the crate and returns a diagnostic DTO that
contains only the existing coarse `error_kind`, never raw error text. The app
combines it with the other diagnostic domains.

## Command and IPC Ownership

All 21 release analysis commands remain app-owned:

- run/source: `list_analysis_sources`, `list_analysis_runs`,
  `list_active_analysis_runs`, `get_analysis_run`,
  `list_analysis_run_messages`, `get_analysis_run_trace`,
  `delete_analysis_run`, `resolve_analysis_trace_refs`;
- templates: `list_analysis_prompt_templates`,
  `create_analysis_prompt_template`, `update_analysis_prompt_template`,
  `delete_analysis_prompt_template`;
- groups: `list_analysis_source_groups`, `create_analysis_source_group`,
  `update_analysis_source_group`, `delete_analysis_source_group`;
- chat: `list_analysis_chat_messages`, `clear_analysis_chat_messages`,
  `ask_analysis_run_question`;
- reports: `start_analysis_report`, `cancel_analysis_run`.

The three app-owned project commands are `start_project_analysis`,
`list_project_runs`, and `get_project_data_range`. The three `#[cfg(dev)]`
fixture commands are `seed_analysis_redesign_fixtures`,
`clear_analysis_redesign_fixtures`, and
`clear_analysis_redesign_fixture_active_runs`.

Names, parameters, camelCase mapping, return types, `AppError` JSON, serialized
DTO fields, and command registration remain unchanged. Project deletion,
startup cleanup, account deletion, NotebookLM, and diagnostics also receive
focused compatibility tests even though they are not counted among the 24
analysis-facing release commands.

No new `status`, `state`, `kind`, `mode`, `phase`, `type`, `provider`,
`subtype`, `scope`, or severity wire value is introduced. Consequently this
slice does not change `docs/value-registry.md`; any implementation that needs
a new persisted or wire value exceeds this design.

## Compression Boundary

Before the move, `analysis/trace.rs` replaces direct
`zstd::encode_all/decode_all` with
`extractum_core::compression::{compress_json_bytes, decompress_bytes}`. Both
paths use the same zstd level and preserve the canonical JSON representation.

Characterization must prove:

- existing trace bytes remain decodable;
- newly written trace data round-trips;
- Telegram and YouTube refs keep the same JSON shape;
- invalid zstd and invalid JSON retain their current internal-error mapping;
- compression errors do not change persisted or emitted text.

After confirming there is no other direct app use, the app's direct `zstd`
dependency is removed. The workspace dependency remains because
`extractum-core` owns compression.

## Manifest and Dependency Contract

The intended new manifest is:

```toml
[package]
name = "extractum-analysis"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
extractum-core = { path = "../extractum-core" }
extractum-llm = { path = "../extractum-llm" }
serde.workspace = true
serde_json.workspace = true
sqlx.workspace = true
tokio = { workspace = true, features = ["macros", "rt", "sync"] }
tokio-util.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["time"] }
```

The implementation contract must verify the exact direct roots and the exact
crate-local feature additions shown above. `sqlx.workspace = true`
intentionally inherits the canonical workspace SQLx feature set; Phase 7 does
not narrow or fork that shared root. Any additional root or crate-local feature
requires an explicit design amendment.

Forbidden direct roots include:

- `extractum`, Tauri, `tauri-build`, and Tauri plugins;
- `extractum-prompt-packs` and `extractum-gemini-browser`;
- direct `zstd`, `reqwest`, `secrecy`, and `parking_lot`;
- Apalis, Grammers, Windows/process dependencies;
- application source/project/test-support modules;
- `tempfile`, `sha2`, or `time` without a newly approved concrete need.

The application gains one path dependency and workspace membership entry.
Every lower crate must remain free of an upward `extractum-analysis` edge.
`Cargo.lock` changes are part of the mechanical move and are asserted by the
boundary contract.

## Public Rust API and Visibility

The crate root uses explicit curated re-exports. Public `mod` declarations,
glob exports, public test helpers, and public implementation rows are
forbidden.

The public surface is limited to:

- IPC/domain DTOs already consumed by the app;
- `AnalysisState` and its narrow lifecycle/read APIs;
- scope, corpus, report, chat, event, and execution value types;
- `AnalysisCorpusReader` and `AnalysisEventSink`;
- constructors for report requests and run filters;
- typed storage/coordinator functions needed by commands and named
  cross-domain consumers.

Preparation resolves current visibility hazards before the move:

- `StartAnalysisReportRequest` receives named source/group/project
  constructors; its fields are not made public;
- `AnalysisRunListFilters` receives analysis/project constructors or a
  constrained builder; its fields are not made public;
- `YoutubeCorpusMode` exposes a typed wire parser/`FromStr`, while SQL helpers
  remain private;
- `AnalysisState::active_report_run_ids` becomes a public read-only accessor;
- dev fixtures use a narrow lifecycle function rather than raw token-map
  mutation.

`AnalysisRunRow`, `AnalysisSourceGroupRow`, `StoredRunSnapshotRow`,
`CorpusMessage`, `ChunkSummary`, SQL request structs, and their fields remain
private. `AnalysisRunDetail` does not expose `scope_label_snapshot` or raw
snapshot counters merely to satisfy app construction.

Every `pub(crate) -> pub` widening must be enumerated in the implementation
plan and source contract. Compilation pressure is not authorization to expose
an internal module or field.

## Current-File Disposition

The implementation plan must produce an exact 54-file map. The governing
disposition is:

- `mod.rs`, `report_commands.rs`, and the command portions of `chat.rs`,
  `groups.rs`, and `templates.rs` remain app adapters behind the private
  facade;
- `events.rs` becomes the app's Tauri `AnalysisEventSink` adapter;
- `fixtures.rs`, `fixtures/seed.rs`, `fixtures/seed/runs.rs`, and all
  `fixtures/tests/**` remain app-owned;
- foreign SQL from `corpus/live.rs`, `corpus/source_resolution.rs`, report
  scope resolution, and group enrichment moves into app adapter files;
- portable corpus/snapshot/preflight, models, state, report/chat engines,
  requests, trace, owned stores, and their 95 tests move to the crate;
- mixed files are split during green preparation so the extraction commit
  contains no behavioral refactor.

Production logic is moved, not copied. Temporary compatibility shims may exist
only in a named green checkpoint and must be gone at final acceptance.

## Frozen Rust-Test Ownership

The baseline is 143 logical Cargo identities, keyed by full module path and
leaf name. Ownership follows the behavior actually under test, not the source
directory in which a test happens to live today:

| Owner | Groups | Count |
| --- | --- | ---: |
| `extractum-analysis` | chat 6, portable corpus 20, portable report 24, state 1, owned store 28, root 8, trace 8 | 95 |
| `extractum` | foreign corpus/scope/group/store integration 30, dev fixtures 18 | 48 |

Tests whose subject is `analysis_documents`, source/project/playlist SQL,
source-type validation, label enrichment/search, or adapter-to-snapshot
behavior remain app-owned. Replacing those tests with a fake reader would
change their subject and is forbidden. New crate tests may use a
counting/panicking fake to characterize the port itself, but receive new
identities and do not substitute for the 48 retained app identities.

Three no-live-fallback snapshot identities remain crate-owned even though
their private test setup seeds app-owned tables as negative sentinels. The
functions under test read only `analysis_run_messages`; the foreign rows prove
that no fallback is attempted. Those tests may seed the canonical test schema
directly, but must not import an app adapter, copy a production source builder,
or create a public test helper. This exact test-only allowance does not permit
foreign-table SQL in production crate code.

The boundary contract must prove every Appendix A identity exists exactly once
under its final owner. Renaming, disabling, duplicating, or silently replacing
a test is not a move.

## Crate-Private Test Schema Fixture

The crate may define one private `#[cfg(test)] test_schema` module. It embeds
canonical SQL with `include_str!` from the application migration directory;
it does not copy SQL or import the app migration runner.

The exact ordered allowlist is the current non-Apalis prefix:

1. `0001_current_schema_baseline.sql`
2. `0002_migrated_history_opt_in_schema.sql`
3. `0003_analysis_telegram_history_scope.sql`
4. `0004_source_delete_cascade_indexes.sql`
5. `0005_projects_mvp.sql`
6. `0006_prompt_pack_mvp.sql`
7. `0007_prompt_pack_run_idempotency.sql`
8. `0008_prompt_pack_run_labels.sql`
9. `0009_prompt_pack_intermediate_entities_artifacts.sql`
10. `0010_prompt_pack_runtime_provider.sql`
11. `0011_prompt_pack_stage_browser_provenance.sql`
12. `0012_projects_redesign.sql`

A standing TypeScript contract parses `build_migrations()` up to, but not
including, `apalis_sqlite_migrations()`, resolves each registration to its
canonical SQL path, and requires exact ordered equality with this fixture
allowlist. A new or reordered `0013` migration makes the contract RED until the
fixture and consumed-shape characterizations are updated in the same change.

The fixture is test-only and is not a second production migration engine.
Minimal hand-written schemas are allowed only in explicitly named isolated
failure/transaction tests, must be marked partial, and do not claim registry
parity. The 18 app fixture tests continue to use the full app migration helper.

The crate has no dev dependency on `extractum` or `sources::test_support` and
exports no test helper.

## Source-Boundary Contracts

A new `src/lib/analysis-crate-boundary-contract.test.ts` must verify:

- workspace membership, the one app path edge, and `Cargo.lock` membership;
- exact dependency roots and features plus the denylist;
- no Tauri, `AppHandle`, app pool lookup, app module import, direct `zstd::`,
  foreign-table SQL, or `sources::test_support` in production crate code;
- no foreign-table SQL in crate tests except the exact three Appendix A
  no-live-fallback negative-sentinel setups;
- the exact six-table owned allowlist;
- app ownership of migrations, `analysis_documents`, commands, event adapter,
  scope/corpus adapter, and dev fixtures;
- curated `lib.rs`, no public modules/globs/test support, and the exhaustive
  visibility-widening allowlist;
- moved-not-copied production files and exact 95/48 test ownership;
- exact ordered migration-fixture parity and consumed schema shapes;
- no reverse dependency from lower crates;
- compression ownership and removal of the app's direct `zstd` root;
- unchanged event channels, payload fields, command names/signatures, and
  `AppError` JSON.

The slice also updates all existing workspace-member/dependency allowlists,
including the core, Gemini Browser, LLM, prompt-pack, development-loop, and
focused-loop contracts.

`src/lib/analysis-redesign-safety-contract.test.ts` currently imports raw
backend files from `src-tauri/src/analysis/**`. Its imports and assertions move
atomically with their owning behavior. Snapshot-only evidence, chat
persistence, scope-label precedence, and no-live-fallback assertions remain
equally strong; a deleted path must not cause the protection to disappear.

`src/lib/crate-extraction-shell-cap-contract.test.ts` gains a Phase 7 status
vocabulary and the owner-approved Phase 7 measurement override before the
first preparation status is recorded.

## Approval Synchronization

Written owner approval closes the design phase but does not authorize
implementation. The approval commit must atomically:

1. change this document to `Approved; implementation not started`;
2. update the Phase 7 roadmap entry to `design approved; implementation not
   started`, link this specification, and replace the stale 65%/40-commit
   placeholder facts with the refreshed evidence;
3. update `crate-extraction-shell-cap-contract.test.ts` to recognize that exact
   approved status and the Phase 7 timing rule;
4. run the focused contract test before implementation planning begins.

This synchronization occurs before Checkpoint 1, so an approved specification
and a roadmap that still demands a fresh design never coexist.

## Preparation Checkpoints

Every preparation checkpoint is a separate green commit. Each keeps the app
buildable and leaves a useful retained improvement if the extraction stops.

### Checkpoint 1 — freeze and characterize

- freeze the exact 95/48 Appendix A partition against executable Cargo output;
- characterize all 24 analysis-facing release commands and three dev commands;
- pin report/chat acceptance order, both corpus reads, profile-resolution
  timing, event payload/order/messages, cancellation, cleanup, AppError JSON,
  trace compatibility, and cross-domain delete/read behavior;
- advance the already-approved roadmap state to `preparation Checkpoint 1
  retained` only after the checkpoint is green.

### Checkpoint 2 — compression and safe construction

- route trace compression through `extractum-core`;
- add request/filter/scope constructors and typed corpus-mode parsing;
- migrate every external struct literal while all consumers compile together;
- keep internal rows and fields private.

### Checkpoint 3 — scope and corpus adapters

- introduce `ResolvedAnalysisScope`, foreign label match/enrichment values,
  and `AnalysisCorpusReader`;
- move foreign SQL and source/project/playlist interpretation into app
  adapters;
- preserve filter-before-limit, exact ordering, and both corpus reads.

### Checkpoint 4 — runtime seam

- introduce typed events and `AnalysisEventSink`;
- remove `AppHandle` from portable execution contexts;
- make scheduler, pool, state, profile, corpus reader, and sink explicit;
- keep outer app spawn, profile timing, request IDs, and terminal cleanup.

### Checkpoint 5 — owned SQL and test fixture

- isolate the six-table store and transaction APIs;
- replace projects, NotebookLM, account-deletion, and diagnostic table access
  with curated analysis APIs while preserving cross-domain transactions;
- establish the private canonical migration fixture and standing parity test;
- leave all 143 tests green under the application package.

### Checkpoint 6 — intentionally RED boundary contract

Commit the exact crate boundary contract, workspace/manifest/lock expectations,
file map, public API allowlist, and 95/48 ownership map. RED must fail only
because the crate and physical move do not yet exist.

### Checkpoint 7 — mechanical extraction

Create the crate, move prepared portable files/tests, wire the private app
facade and adapters, update manifests/lockfile, and turn the boundary contract
GREEN. No new seam, behavior change, visibility decision, or test redesign is
allowed in this checkpoint.

After any green Checkpoint 1–5, execution may stop legitimately. The roadmap
must say `Phase 7 — in progress; preparation Checkpoint N retained`, and the
status contract must accept only the explicit states `design approved;
implementation not started`, `preparation Checkpoint N retained`, `done:
retained`, or `not retained`.

## Mechanical Move Rule

The extraction commit may contain file moves, module-path changes, manifest and
lockfile updates, explicit facade re-exports, and mechanically necessary import
changes. It may not introduce a new abstraction or fix an unrelated defect.

Any unexpected compile failure that requires a new port, public field,
dependency root, SQL ownership exception, or observable behavior change stops
the move. The design or preparation checkpoint is amended first and made green
before a new extraction attempt.

## Rust Verification Loops

The implementation plan must include the repository-required
`## Rust Verification Loops` section and name every affected package.

During Checkpoints 1–5, code still owned by the app uses `-p extractum` with
exact non-empty tests. After the move, domain checks use
`-p extractum-analysis`. Any public interface change also checks immediate
consumer `extractum`.

Required post-move inventories include:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --lib -- --list
# the Appendix contract finds all 95 mapped baseline identities exactly once

cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::' -- --list
# the Appendix contract finds all 48 retained baseline identities exactly once
```

Focused exact tests must cover at least:

- legacy/invalid trace decompression;
- live corpus filtering, order, migrated history, and port behavior;
- snapshot roundtrip and no-live-fallback;
- run read-model filtering and scope labels;
- report cancellation and terminal cleanup;
- chat persistence failure and asynchronous profile failure;
- group validation/enrichment;
- app fixture seed/clear behavior;
- cross-domain project deletion and account-deletion dependency behavior.

Package checkpoints are:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

End-of-slice gates are:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

A filtered Cargo run reporting zero tests is not evidence. The canonical
shared `src-tauri/target` remains in use; Phase 7 does not create an isolated
target or measurement worktree.

## Advisory Timing

For Phase 7, compile-time measurement is reduced to the duration of one
ordinary mandatory workspace check. It is recorded as advisory evidence only.

There is no focused probe, discarded warm-up, sample series, source mutation,
quiet-window scan, process coordinator, stability rule, retry, A/B harness, or
cumulative ledger. Measurement failure is recorded honestly and does not
invalidate correctness gates or retention.

This is an explicit owner-approved Phase 7 override to the roadmap's
hot-module one-warm-up/three-sample rule. The implementation must update the
roadmap and its shell-cap contract before measurement so the two authorities
do not coexist in contradiction. The ordinary result still participates in
the roadmap's coarse adjacent `>= 15,000 ms` observation rule.

## Release and Startup Evidence

After all source and test gates pass:

1. build the release application with `--no-bundle`;
2. prove startup by exact executable PID/path and a bounded observation;
3. prove the owned process exits and no matching process remains;
4. run live MCP smoke before any self-managed analysis smoke when both are
   used, and verify app ports are free before the self-managed run;
5. avoid live credentialed provider requests and account mutation.

Infrastructure failure in build/startup tooling is classified separately from
an application completion failure. The release smoke must not introduce a new
process-control harness merely for this slice.

## Failure and Rollback

The rollback ladder preserves useful green preparation:

1. every Checkpoint 1–5 is independently committed and verified;
2. a pause records the last retained checkpoint in the roadmap;
3. the RED boundary contract is a separate commit;
4. the mechanical extraction is a separate commit;
5. if extraction or completion gates fail, preserve the failed candidate in
   Git history, revert the extraction commit first and the RED contract commit
   second, then decide each earlier green checkpoint independently.

No stop condition authorizes `git reset`, destructive path checkout, or manual
deletion of evidence. Resuming begins from the last retained green checkpoint
after identity and gates are reconfirmed.

If the candidate is not retained, write a verification disposition and set
the roadmap to the truthful `not retained` or retained-checkpoint state. An
advisory timing result never triggers rollback.

## Non-Goals

- No analysis feature, UX, prompt, schema, status, or frontend redesign.
- No one-read corpus optimization.
- No change to map/reduce chunking, concurrency, scheduler priority, model
  policy, or request-ID construction.
- No new task supervisor or app-wide runtime abstraction.
- No generic repository, unit of work, service locator, SQLite crate, source
  crate, or test-support crate.
- No move of migrations, `analysis_documents`, sources, projects, Telegram,
  YouTube, NotebookLM, accounts, or diagnostics into the crate.
- No public submodules, glob exports, implementation rows, secret-bearing
  getters, or test helpers.
- No crate dev dependency on the application.
- No fix for the project-scoped account-deletion blind spot.
- No unrelated cleanup of existing warnings.
- No live provider/account mutation gate.
- No timing runner, target-directory change, linker change, or performance
  retention threshold.

## Acceptance Criteria

Phase 7 may be recorded as implemented and retained only when:

1. the Appendix A partition is exactly 95 crate and 48 app identities, each
   present once;
2. the 21 analysis commands, three project commands, three dev commands,
   startup cleanup, and named cross-domain consumers preserve their contracts;
3. production crate code contains no Tauri, `AppHandle`, app pool lookup,
   application import, foreign-table SQL, direct zstd, or test support;
4. all runtime SQL for the exact six owned tables resides in the crate, with
   app-owned cross-domain transactions using curated transaction APIs;
5. migrations and `analysis_documents` remain app/source-owned;
6. scope resolution and corpus loading use owned values/ports and preserve
   provider validation, playlist expansion, ordering, refs, filters, and both
   live reads;
7. report/chat acceptance timing, profile timing, detached spawn behavior,
   LLM priorities, request IDs, event ordering, persistence ordering,
   cancellation, cleanup, and serialized errors remain unchanged;
8. run/group foreign labels and multi-term search retain filter-before-limit
   behavior without foreign joins in the crate;
9. trace compression uses `extractum-core` and remains compatible with
   existing bytes and error mapping;
10. public API, visibility, manifest, lockfile, reverse-edge, and
    moved-not-copied contracts pass;
11. the private test fixture uses canonical SQL and remains in exact ordered
    parity with the registered non-Apalis migration prefix;
12. crate, immediate-app, workspace, repository, release, and startup gates
    pass;
13. one ordinary workspace timing result is recorded as advisory evidence,
    without additional measurement machinery;
14. a durable verification document and roadmap update record the retained or
    non-retained result and leave Phase 8 unauthorized.

## Implementation-Plan Requirements

The implementation plan may be written only after this specification is
reviewed and approved. It must include:

- the exact Appendix A inventory and final Cargo paths;
- an exact 54-file production/test disposition map;
- all 21 release analysis commands, three project commands, three dev commands,
  and cross-domain integration points;
- the exact public root allowlist and every visibility widening;
- the exact six-table allowlist, foreign-table denylist, transaction API map,
  and migration-fixture allowlist/parity parser;
- final signatures for `ResolvedAnalysisScope`, `AnalysisCorpusReader`,
  `AnalysisEventSink`, preparation/execution tickets, foreign label inputs,
  state lifecycle APIs, and constructors;
- named RED/GREEN tests and the required `## Rust Verification Loops` section;
- separately green preparation commits, one RED contract commit, one
  mechanical move commit, and the pause/rollback ladder;
- exact manifest features, workspace allowlist updates, and `Cargo.lock`
  assertions;
- the single ordinary advisory timing capture, full completion gates,
  release/startup evidence, verification document, and roadmap update.

The existence of this specification does not authorize implementation.
Execution begins only after explicit owner instruction following plan review.

## Appendix A: Frozen 143-Test Baseline

Each heading freezes a current module prefix and its required final prefix.
Each bullet is the normative leaf name. The complete baseline identity is
`<current prefix>::<leaf>`; the complete post-extraction identity is
`<final prefix>::<leaf>`. Crate-owned identities mechanically drop only the
leading app module `analysis::`. App-owned identities retain their current
prefix. The implementation plan must confirm the executable inventory
immediately before Checkpoint 1 and preserve every mapped identity exactly
once.

### `extractum-analysis` — 95 identities

#### Chat (6)

Current prefix: `analysis::chat::tests`. Final prefix: `chat::tests`.

- `analysis_chat_request_metadata_uses_run_owner`
- `build_chat_request_uses_provider_neutral_source_document_wording`
- `chat_context_labels_migrated_history_scope_from_metadata`
- `completed_chat_context_accepts_saved_snapshot_messages`
- `completed_chat_context_requires_saved_snapshot_messages`
- `empty_chat_context_uses_source_document_wording`

#### Portable corpus live policy (1)

Current prefix: `analysis::corpus::tests::live`. Final prefix:
`corpus::tests::live`.

- `youtube_corpus_mode_parses_wire_values_and_defaults`

#### Portable corpus preflight (7)

Current prefix: `analysis::corpus::tests::preflight`. Final prefix:
`corpus::tests::preflight`.

- `default_preflight_limits_are_conservative`
- `estimated_chunk_count_matches_chunk_boundary_behavior`
- `estimated_message_chars_match_report_chunk_accounting`
- `model_limit_preflight_allows_unknown_or_fitting_limits`
- `model_limit_preflight_reports_oversized_chunks`
- `preflight_limit_error_allows_runs_within_limits`
- `preflight_limit_error_reports_all_scale_dimensions`

#### Portable corpus snapshot (11)

Current prefix: `analysis::corpus::tests::snapshot`. Final prefix:
`corpus::tests::snapshot`.

- `captured_marker_with_missing_rows_returns_corrupt_snapshot_error`
- `list_run_snapshot_messages_page_does_not_fall_back_to_live_source`
- `list_run_snapshot_messages_page_reads_saved_snapshot_only`
- `list_run_snapshot_messages_page_returns_typed_internal_for_corrupt_snapshot_content`
- `list_run_snapshot_messages_page_starts_at_around_ref`
- `load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows`
- `load_run_corpus_messages_uses_snapshot_when_available`
- `run_message_cursor_uses_ref_and_published_at`
- `run_snapshot_roundtrips_frozen_corpus`
- `source_group_membership_drift_after_capture_does_not_change_saved_run_corpus`
- `trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot`

#### Portable corpus source resolution (1)

Current prefix: `analysis::corpus::tests::source_resolution`. Final prefix:
`corpus::tests::source_resolution`.

- `resolve_run_source_ids_prefers_snapshot_over_live_group_membership`

#### Report architecture (1)

Current prefix: `analysis::report::tests::architecture`. Final prefix:
`report::tests::architecture`.

- `analysis_report_workflow_file_has_no_tauri_command_adapters`

#### Report lifecycle (4)

Current prefix: `analysis::report::tests::lifecycle`. Final prefix:
`report::tests::lifecycle`.

- `interrupted_cleanup_preserves_captured_snapshot_state_marker`
- `request_analysis_run_cancel_completed_run_keeps_conflict_message`
- `request_analysis_run_cancel_missing_run_keeps_not_found_message`
- `request_analysis_run_cancel_running_but_inactive_keeps_conflict_message`

#### Report phases (5)

Current prefix: `analysis::report::tests::phases`. Final prefix:
`report::tests::phases`.

- `analysis_step_cancel_wrapper_allows_completed_future`
- `analysis_step_cancel_wrapper_interrupts_pending_future`
- `finish_map_phase_preserves_chunk_order_by_original_index`
- `finish_map_phase_propagates_map_error_without_starting_reduce`
- `finish_map_phase_rejects_missing_chunk_before_reduce`

#### Report preflight (3)

Current prefix: `analysis::report::tests::preflight`. Final prefix:
`report::tests::preflight`.

- `validate_report_preflight_allows_runs_within_limits`
- `validate_report_preflight_rejects_empty_corpus`
- `validate_report_preflight_rejects_oversized_runs`

#### Report requests (6)

Current prefix: `analysis::report::tests::requests`. Final prefix:
`report::tests::requests`.

- `build_map_request_keeps_run_scoped_request_and_profile`
- `build_reduce_request_keeps_run_scoped_request_and_profile`
- `extracts_json_inside_markdown_fence`
- `extracts_json_with_text_before_and_after`
- `parse_chunk_summary_ignores_non_json_prefix_with_braces`
- `parse_chunk_summary_rejects_malformed_payload`

#### Report scope (5)

Current prefix: `analysis::report::tests::scope`. Final prefix:
`report::tests::scope`.

- `chunk_target_chars_are_derived_from_model_input_limit_with_fallback`
- `migrated_history_opt_in_rejects_non_telegram_analysis`
- `report_run_input_carries_resolved_profile_snapshot`
- `report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape`
- `telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match`

#### State (1)

Current prefix: `analysis::state::tests`. Final prefix: `state::tests`.

- `analysis_state_cancels_report_run_child_tokens`

#### Owned store read model (15)

Current prefix: `analysis::store::tests::read_model`. Final prefix:
`store::tests::read_model`.

- `completed_run_without_capture_marker_is_capture_failed`
- `failed_terminal_run_without_capture_marker_is_capture_failed`
- `list_analysis_run_summaries_applies_query_before_limit`
- `list_analysis_run_summaries_combines_scope_and_field_filters`
- `list_analysis_run_summaries_escapes_literal_like_characters`
- `list_analysis_run_summaries_filters_source_groups_and_template_names`
- `list_analysis_run_summaries_filters_status_and_dates`
- `list_analysis_run_summaries_rejects_both_scope_ids`
- `map_run_detail_exposes_youtube_corpus_mode`
- `map_run_summary_exposes_capture_failed_snapshot_state`
- `map_run_summary_exposes_captured_snapshot_state`
- `map_run_summary_exposes_frozen_scope_label`
- `map_run_summary_exposes_null_snapshot_state_for_active_runs_before_capture`
- `map_run_summary_exposes_youtube_corpus_mode`
- `resolve_run_scope_label_prefers_frozen_value`

#### Owned store runs (7)

Current prefix: `analysis::store::tests::runs`. Final prefix:
`store::tests::runs`.

- `cancellation_after_capture_does_not_write_snapshot_error`
- `delete_saved_run_removes_run_and_saved_children`
- `delete_saved_run_returns_typed_not_found_error`
- `duplicate_lookup_keeps_project_and_source_group_scopes_separate`
- `duplicate_lookup_matches_telegram_history_scope`
- `insert_analysis_run_persists_youtube_corpus_mode`
- `provider_failure_status_update_does_not_write_snapshot_error`

#### Owned store setup (1)

Current prefix: `analysis::store::tests::setup`. Final prefix:
`store::tests::setup`.

- `fetch_prompt_template_returns_typed_not_found_error`

#### Owned store snapshot (5)

Current prefix: `analysis::store::tests::snapshot`. Final prefix:
`store::tests::snapshot`.

- `capture_run_snapshot_marks_captured_after_reload_and_replaces_rows`
- `capture_run_snapshot_rejects_missing_required_fields_without_marker`
- `mark_run_capture_failed_sets_snapshot_error`
- `sanitize_provider_error_redacts_provider_payloads`
- `sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens`

#### Root analysis tests (8)

Current prefix: `analysis::tests`. Final prefix: `tests`.

- `builtin_template_is_seeded_once`
- `chat_role_validation_returns_typed_error`
- `chat_turn_validation_returns_typed_error`
- `completed_run_without_snapshot_marker_is_capture_failed`
- `source_group_input_is_trimmed_and_deduplicated`
- `source_group_input_validation_returns_typed_error`
- `template_kind_validation_returns_typed_error`
- `trace_data_roundtrips_through_zstd`

#### Trace (8)

Current prefix: `analysis::trace::tests`. Final prefix: `trace::tests`.

- `analysis_trace_ref_serializes_youtube_fields_as_null_for_telegram_refs`
- `build_trace_refs_falls_back_to_base_item_refs`
- `build_trace_refs_handles_multibyte_excerpt`
- `build_trace_refs_marks_youtube_description_refs_as_synthetic`
- `build_trace_refs_resolves_exact_youtube_timestamp_refs`
- `clip_excerpt_truncates_on_char_boundary`
- `decode_trace_data_returns_typed_internal_for_invalid_zstd`
- `normalize_ref_accepts_item_refs`

### `extractum` — 48 identities

#### Foreign corpus live adapter (15)

Current and final prefix: `analysis::corpus::tests::live`.

- `default_analysis_corpus_excludes_migrated_history_documents`
- `description_mode_creates_synthetic_description_message`
- `explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus`
- `live_corpus_refs_use_local_item_ids`
- `load_corpus_messages_filters_telegram_to_telegram_message`
- `load_corpus_messages_filters_youtube_transcript_only_to_transcripts`
- `load_corpus_messages_includes_youtube_comment_only_in_comments_mode`
- `load_corpus_messages_orders_transcript_segments_by_document_order_not_ref`
- `load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content`
- `opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight`
- `preflight_ref_format_matches_corpus_loader_ref_format`
- `source_group_opt_in_includes_only_members_with_migrated_rows`
- `youtube_description_missing_typed_metadata_skips_without_decoding_source_blob`
- `youtube_description_rows_use_typed_metadata_with_corrupt_source_blob`
- `youtube_transcript_segment_evidence_uses_typed_source_context`

#### Foreign corpus preflight integration (3)

Current and final prefix: `analysis::corpus::tests::preflight`.

- `preflight_count_matches_loader_for_youtube_corpus_modes`
- `preflight_counts_eligible_text_messages_for_sources`
- `preflight_ignores_media_only_items_without_text_content`

#### Foreign scope and playlist resolution (5)

Current and final prefix: `analysis::corpus::tests::source_resolution`.

- `playlist_expansion_excludes_unlinked_and_removed_rows`
- `resolve_analysis_sources_loads_single_provider_project`
- `resolve_analysis_sources_preserves_no_linked_youtube_error_message`
- `resolve_analysis_sources_rejects_mixed_provider_project`
- `resolve_run_source_ids_loads_project_sources_without_snapshot`

#### Foreign group source validation (3)

Current and final prefix: `analysis::groups::tests`.

- `validate_group_source_type_accepts_matching_provider_membership`
- `validate_group_source_type_rejects_mixed_provider_membership`
- `validate_group_source_type_rejects_unknown_group_type`

#### Report capture adapter integration (1)

Current and final prefix: `analysis::report::tests::capture`.

- `capture_report_corpus_returns_reloaded_snapshot_before_provider_phases`

#### Foreign store read-model integration (2)

Current and final prefix: `analysis::store::tests::read_model`.

- `list_analysis_run_summaries_filters_project_runs`
- `list_analysis_run_summaries_matches_all_query_terms_across_any_field`

#### Foreign store setup integration (1)

Current and final prefix: `analysis::store::tests::setup`.

- `ensure_sources_exist_returns_typed_not_found_error`

#### Dev fixture active runs (2)

Current and final prefix: `analysis::fixtures::tests::active_runs`.

- `fixture_active_state_tracks_seeded_running_run`
- `fixture_cancel_waiter_marks_running_run_cancelled`

#### Dev fixture clear (3)

Current and final prefix: `analysis::fixtures::tests::clear`.

- `clear_deletes_child_rows_through_fixture_parent_ids`
- `clear_preserves_non_fixture_groups_and_members`
- `clear_removes_only_fixture_rows_and_is_idempotent`

#### Dev fixture harness (1)

Current and final prefix: `analysis::fixtures::tests::harness`.

- `fixture_test_pool_has_required_tables`

#### Dev fixture seed (7)

Current and final prefix: `analysis::fixtures::tests::seed`.

- `compressed_fixture_fields_are_readable`
- `seed_creates_fixture_runs_with_statuses_templates_and_snapshots`
- `seed_creates_post_sync_reader_content`
- `seed_creates_safe_account_prompt_profile_sources_and_group`
- `seed_creates_sources_that_pass_identity_repair`
- `seed_creates_valid_typed_youtube_detail_metadata`
- `seed_twice_keeps_one_deterministic_fixture_set`

#### Dev fixture snapshot (4)

Current and final prefix: `analysis::fixtures::tests::snapshot`.

- `capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report`
- `fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot`
- `missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages`
- `seeded_snapshot_runs_expose_captured_snapshot_state`

#### Dev fixture summary (1)

Current and final prefix: `analysis::fixtures::tests::summary`.

- `summary_serializes_with_camel_case_keys`
