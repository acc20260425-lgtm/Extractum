# Prompt Packs Crate Boundary Design

**Status:** Owner-approved; implementation not started
**Date:** 2026-07-20

**Roadmap authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

**Verification-loop authority:**
[`2026-07-17-focused-rust-loop-design.md`](2026-07-17-focused-rust-loop-design.md)

This specification defines the just-in-time Phase 6 boundary for
`extractum-prompt-packs`. It supersedes only the short Phase 6 placeholder in
the crate roadmap. It does not change the retained Phase 4 Gemini Browser or
Phase 5 LLM boundaries, authorize Phase 7, or reopen the canceled process-crate
work.

The design was informed by the local, intentionally untracked architecture
audit `reference/2026-07-20-crate-extraction-refactoring-analysis.md`. That
audit was taken before Phase 5 was implemented, so its conclusions were
refreshed against current `HEAD`. This specification is self-contained and
does not depend on the reference file remaining present.

## Purpose

Phase 6 extracts prompt-pack lifecycle, YouTube Summary execution, validation,
and domain persistence into `extractum-prompt-packs`. The application retains
Tauri integration, migration registration, credential resolution, foreign
source reads, concrete Gemini Browser operations, task spawning, and IPC event
emission.

The extraction is an ownership and compilation-boundary change. It must not
redesign the prompt-pack product, normalize persisted values, change provider
behavior, collapse the current preflight sequence, migrate schemas, or alter
frontend contracts.

## Decision

The selected boundary is a SQL-owning domain crate behind a private application
facade:

1. `extractum-prompt-packs` owns prompt-pack behavior and every runtime
   query/DML/transaction operation against prompt-pack-owned tables, excluding
   app-owned migrations and the crate-private test schema fixture;
2. the app obtains and passes `SqlitePool`; migrations remain app-owned;
3. the app loads foreign source data through a narrow reader that returns
   fully owned crate DTOs;
4. the app resolves credential-backed `ResolvedLlmProfile` values, while the
   crate directly uses `extractum-llm` requests, scheduler, model policy, and
   completion APIs;
5. concrete Gemini Browser status, submission, and cancellation stay app-side
   behind an object-safe prompt-pack browser port;
6. the crate emits typed domain events through a synchronous, infallible event
   sink; the app maps them to the existing IPC payload and ignores Tauri emit
   failures exactly as it does today;
7. `src-tauri/src/prompt_packs/mod.rs` remains a private, explicit
   compatibility facade, and the new crate root exposes a curated API rather
   than reproducing the current broad `pub mod` surface.

This is the owner-approved hybrid option. It deliberately does not introduce a
generic repository layer, universal completion executor, shared SQLite crate,
or Tauri dependency in the domain crate.

## Alternatives Considered

### Keep all SQL in the application

This would make the new crate superficially persistence-neutral, but SQLx is
present throughout prompt-pack lifecycle, validation, artifacts, projections,
and result assembly. Replacing those operations with a repository interface
would create a large speculative API and move transaction ownership away from
the domain that defines it.

Rejected: prompt-pack-owned SQL remains with prompt-pack behavior.

### Put both LLM and Browser execution behind one app completion port

This would minimize direct lower-crate dependencies, but keep the hottest
provider orchestration in `extractum`, make the fixed three-crate layering
mostly nominal, and duplicate request/scheduler policy in adapters.

Rejected: API LLM execution uses `extractum-llm` directly; only repeated
app-owned Browser side effects use a port.

### Move the directory mechanically with Tauri

This is the shortest file move, but it carries `AppHandle`, `State`, `Emitter`,
pool acquisition, and application state into the new package. The resulting
crate would not provide the focused domain loop intended by the roadmap.

Rejected: mixed seams are prepared while code is still app-owned, then the
physical move is mechanical.

### Let the crate read foreign source tables directly

This preserves the fewest source lines but makes the prompt-pack crate depend
on application-owned `sources`, `items`, and YouTube schema details. It also
creates a future cycle risk if those read models later move under analysis or a
producer crate.

Rejected: the app owns a narrow source reader; a future lower
`extractum-sources` crate may own it only through a separate approved phase.

## Fresh Evidence Snapshot

The refreshed snapshot was taken on 2026-07-20 at
`6f0794b21c4b635b3214130983c14a5ca63ff24e` with a clean worktree.

- `src-tauri/src/prompt_packs` contains 46 files and 19,037 physical lines:
  21 root modules / 8,736 lines and 25 `youtube_summary` files / 10,301
  lines.
- Eight dedicated test/support files account for 3,475 lines.
- The static source inventory contains exactly 225 Rust test declarations:
  100 in top-level prompt-pack modules and 125 under `youtube_summary`. The
  normative identity is the logical source module/file plus leaf test name;
  `now_string_uses_current_utc_time` intentionally exists in two different
  modules, so a leaf-name-only uniqueness check would be incorrect. The
  implementation plan must confirm the executable Cargo inventory before
  freezing the baseline; a filtered run reporting zero tests is not evidence.
- Since 2026-06-01, 118 commits touched `prompt_packs`; 92 of them (78.0%)
  touched no other categorized Rust domain. Joint touches were led by `lib`
  12, `gemini_browser` 9, `migrations` 7, `llm` 5, `analysis` 4, `youtube` 3,
  and `projects` 2. This supports the ownership boundary but does not make the
  current mixed runtime files mechanically movable.
- Direct Tauri usage is confined to six files: `completion_transport.rs`,
  `library.rs`, `result_commands.rs`, `runtime.rs`, `seed.rs`, and
  `stage_execution.rs`.
- SQLx appears in 36 files covering 17,515 whole-file lines; 30 are
  production-path files. A pool-taking crate is materially smaller than a
  repository-port rewrite.
- `crate::db::get_pool` appears only in `library.rs`, `result_commands.rs`,
  `runtime.rs`, and `seed.rs`, defining natural thin app wrappers.
- The only production Rust consumer outside the current private module is
  `src-tauri/src/lib.rs`. There is no current prompt-pack-to-analysis or
  analysis-to-prompt-pack Rust dependency.
- The current app module has many `pub mod` declarations, but its parent
  `mod prompt_packs` is private. Copying those declarations into a public
  crate root would accidentally create a large API.
- The eight bundled JSON assets are compile-time inputs to seeding, stage
  request policy, and validation. Their canonical repository path is also
  persisted as `src-tauri/prompt-packs/youtube_summary/1.0.0`.

## Reference Audit Disposition

The design adopts the audit's preparation-first extraction model and its
value/capability/port rule:

- pass the resolved profile as a value;
- pass `SqlitePool` and `LlmSchedulerState` as concrete capabilities owned by
  lower-level behavior;
- use ports only for repeated app-owned source and Browser side effects;
- keep cross-domain coordination and Tauri event emission in the app.

It also adopts the audit's explicit Phase 6 recommendations: direct LLM use,
a Browser executor port, typed prompt-pack events, app-owned migrations,
foreign source snapshots, and a private curated facade.

The refreshed design adds the following concrete dispositions:

- Phase 5 is now retained, so `extractum-llm` is a real dependency rather than
  a forecast;
- the crate directly depends on SQLx; `extractum-core` stays SQLx-free because
  prompt packs do not consume the deferred `sql_helpers` or `tx` modules;
- domain persistence tests may use a crate-private test schema fixture that
  embeds the single canonical migration SQL files under `#[cfg(test)]` only;
  migration registration and cross-domain integration remain app tests;
- bundled assets remain at their canonical product path and are included
  through one crate asset module instead of being copied or relocated.

## Target Dependency Structure

```text
extractum
  |-- Tauri commands, state registration, spawning, IPC events
  |-- get_pool and migration registration
  |-- profile/credential resolution
  |-- AppPromptPackSourceReader
  |-- TauriGeminiBrowserPort
  |-- TauriPromptPackEventSink
  `-- extractum-prompt-packs
        |-- prompt-pack lifecycle and state
        |-- prompt-pack-owned SQL persistence
        |-- validation, artifacts, projections, result assembly
        |-- YouTube Summary orchestration
        |-- extractum-llm
        |-- extractum-gemini-browser portable types
        `-- extractum-core
```

There is one production path edge from `extractum` to
`extractum-prompt-packs`. There is no reverse edge and no dependency from
`extractum-core`, `extractum-llm`, or `extractum-gemini-browser` upward into
prompt packs.

The new crate must not depend on `extractum-analysis`. The app source adapter
must not later be moved into analysis; it may remain app-owned or move into a
future lower source/read-model crate through a separately approved design.

## Schema Ownership

### Prompt-pack-owned schema

The crate owns queries, transactions, row mapping, and persistence semantics
for the 32 tables created by `0006_prompt_pack_mvp.sql` and evolved by
`0007` through `0011`:

- `prompt_packs`, versions, stage templates, and schema assets;
- runs, scopes, source snapshots, origins, material snapshots, stage runs,
  and stage artifacts;
- canonical results, source refs, claims, evidence, ref edges, unknowns,
  verification tasks, warnings, limitations, quality flags, and audit refs;
- YouTube videos, segments, key points, quotes, action items, open questions,
  and synthesis items;
- validation findings, audit events, and quarantine artifacts.

The implementation contract must use an exact table allowlist rather than
only a `prompt_pack_` prefix because `prompt_packs` is also domain-owned.

### Foreign schema

Production prompt-pack code currently reads `sources`,
`youtube_video_sources`, `youtube_playlist_items`,
`youtube_transcript_segments`, and `items`. Those reads move into the
application source adapter. Production crate code must not contain these table
names or import app source modules.

There is no production prompt-pack query of `projects` at the current HEAD.
Phase 6 therefore adds no speculative project reader; `projects` remains only
the target of the existing foreign-key relationship described below.

`prompt_pack_runs.project_id` and snapshot/scope source IDs retain their
existing foreign keys to `projects` and `sources`. Migrations and referential
behavior remain app-owned. The extraction adds no migration and performs no
production write to a foreign-domain table.

### Migration owner

`src-tauri/src/migrations.rs`, `src-tauri/migrations/**`, startup registration,
checksums, and schema-upgrade behavior remain in `extractum`. Passing a pool to
the crate does not transfer migration ownership.

## Source Read Boundary

The crate defines an object-safe, domain-specific source reader. The app
implements it with a local wrapper such as `AppPromptPackSourceReader`; it
cannot implement a crate-owned trait directly for external `SqlitePool`
because of Rust's orphan rule.

The port uses an explicit boxed-future ABI rather than `async fn` in a dyn
trait. It is one narrow, domain-specific interface, but it deliberately exposes
the six semantic read operations that the current code repeats:

```rust
pub type PromptPackPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

pub trait PromptPackSourceReader: Send + Sync + 'static {
    fn load_source(&self, source_id: i64)
        -> PromptPackPortFuture<'_, Option<PromptPackSourceRecord>>;
    fn load_video(&self, request: YoutubeVideoReadRequest)
        -> PromptPackPortFuture<'_, Option<PromptPackYoutubeVideoRecord>>;
    fn load_playlist_items(&self, playlist_source_id: i64)
        -> PromptPackPortFuture<'_, Vec<PromptPackPlaylistItemRecord>>;
    fn load_transcript_segments(&self, source_id: i64)
        -> PromptPackPortFuture<'_, Vec<PromptPackTranscriptSegment>>;
    fn select_comment_candidates(&self, request: CommentCandidateReadRequest)
        -> PromptPackPortFuture<'_, Vec<PromptPackCommentCandidate>>;
    fn load_comment_body(&self, request: CommentBodyReadRequest)
        -> PromptPackPortFuture<'_, String>;
}
```

Names may be refined during the preparation checkpoint, but the contract is
fixed:

- inputs and outputs are crate-owned values;
- results are fully owned; no row or transaction borrow crosses `await`;
- the reader never receives a prompt-pack write transaction;
- the methods expose no generic table/query/repository vocabulary;
- the crate retains eligibility, inclusion, token, comment-cap, snapshot-ID,
  and material-ref policy.

The implementation must preserve the current fresh-read sequence rather than
cache a request-wide source graph:

1. command/start preflight performs its source/video/playlist/transcript reads;
2. skeleton creation repeats that preflight and those reads;
3. snapshot formation reads source/video metadata, playlist origins, and
   transcript segments again after run insertion;
4. comment selection reads ordered candidates and their text for token
   estimates, then each selected external ID is read again through
   `load_comment_body` before persistence.

Therefore a runnable start makes more than two source-port calls. The same
semantic method may be called repeatedly, and the app adapter must execute a
fresh SQL query on every call. In particular, candidate text used for token
estimation and the later persisted comment body are not assumed identical.
Consolidating, memoizing, or freezing these calls is a future behavior change,
not part of Phase 6.

The resolved value must preserve all current distinctions and ordering:

- caller `source_ids` order;
- playlist order by `position`, then row `id`;
- transcript order by `segment_index`, then row `id`;
- missing source versus missing video metadata versus unlinked playlist item;
- deterministic comment ordering and the domain-requested count limit;
- nullable source subtype, title, description, and external IDs;
- video/channel/publication/canonical-URL metadata and transcript segment
  timestamps;
- comment text after the existing decompression/fallback behavior.

Flattening away any of these distinctions is a behavior change and is not
authorized by this extraction.

## Runtime Boundary

### Start and preflight ordering

The Tauri wrapper passes the request, pool, run state, source reader, Browser
port, and event sink to a crate service. The crate owns the resulting
idempotency/readiness/preflight sequence and returns the existing IPC outcome
plus the minimum execution directive the app needs for spawning.

The preparation must preserve the current idempotency and side-effect order,
including its repeated reads:

1. an empty `client_request_id` fails validation before Browser or source I/O;
2. a non-empty existing request returns the existing run before Browser
   readiness or source I/O;
3. only a new Browser run performs the readiness check;
4. the crate performs the existing second idempotency lookup before preflight;
5. a runnable new request performs the existing second preflight inside run
   skeleton creation, including a fresh source read;
6. skeleton creation retains its own empty-ID and existing-run guards before
   insertion.

This slice does not add new unique-conflict recovery semantics.

Preflight budgeting also remains deliberately separate from actual model
resolution. Both the public preflight and both start-path preflights pass the
existing fixed `ModelBudget`: API uses `input_token_limit: Some(32_000)` and
Gemini Browser uses `None`. Only later, inside spawned API execution, does the
crate resolve the effective model and its actual input limit for
`GemAnalysisInputBudget`. Phase 6 must not substitute the resolved model limit
into preflight. A new API characterization pins `32_000`; the existing
Browser-unbounded characterization remains.

### Start-to-spawn handoff

The crate start service returns a non-serialized service result such as:

```rust
pub struct StartServiceOutcome {
    pub response: StartYoutubeSummaryRunOutcomeDto,
    pub execution_ticket: Option<RunExecutionTicket>,
}
```

The response remains byte-compatible IPC data; the ticket has private fields
and is only an internal cross-crate capability. Preparation borrows the ticket;
on failure the app hands that same ticket to the crate terminal-failure service
rather than reconstructing a run identity or policy. After producing a Started
response, the crate preserves the current condition
`run_status == "queued" && track_if_absent(run_id)`: it applies the queued
event to `PromptPackRunState`, publishes that event through the sink, and
returns a ticket only when tracking was newly acquired. This also covers an
idempotently found queued run that is not currently tracked; it is not limited
to a newly inserted row. The thin app wrapper immediately spawns exactly one
task for a returned ticket and then returns `response`. It does not reconstruct
the condition from serialized response fields or mutate run tracking itself.

### LLM path

The app resolves persisted profile selection and credential material into an
owned `extractum_llm::ResolvedLlmProfile`. The crate then owns:

- effective-model selection and input/output-limit lookup;
- `LlmChatRequest` and `LlmMessage` construction;
- `LlmRequestMetadata` construction with prompt-pack ownership fields;
- background-priority scheduling and queue callbacks;
- provider execution and completion mapping;
- cancellation mapping from `LlmRequestError` into prompt-pack stage outcome.

The app must not pass `SecretStoreState`, profile SQL, `AppHandle`, or Tauri
`State<'_>` into the crate. The owned spawn resolves app state and passes
`&LlmSchedulerState` only for the duration of the awaited crate call; it is not
stored in a `'static` runtime object.

Runtime configuration contains the profile ID and model override, so execution
uses an explicit two-phase handoff after the start command has already returned
its queued response:

1. inside the app-owned spawned task, the crate consumes the execution ticket,
   loads prompt-pack runtime config, and returns a prepared provider request;
2. for API execution that request exposes only the owned `profile_id` and
   `model_override` needed by the app's existing credential/profile resolver;
   Browser preparation carries its owned Browser config and needs no profile;
3. the app resolves an owned `ResolvedLlmProfile` only for the API variant and
   passes it back with the prepared execution value;
4. the crate resolves the effective model and actual model input limit, obtains
   the child cancellation token, publishes the started event, and executes the
   run through the LLM scheduler or Browser port.

Profile resolution must not move before the command response or before the
spawn. A preparation, profile-resolution, or effective-model failure follows
the current spawned-task path: the app delegates the error to a crate-owned
terminal-failure service, which marks the persisted run failed, applies state,
and emits the existing terminal event. Such a failure emits no started event.
The app does not implement failure policy; it only calls the service and logs
the existing secondary failure if terminal persistence itself fails.

The resolved profile is an ephemeral owned capability. Its secret is never
persisted in prompt-pack tables, serialized into an IPC/event DTO, or logged.
Existing profile scoping and the application's credential-store boundary stay
unchanged; this extraction neither resolves nor expands the documented
temporary security debt of persisted LLM profile settings.

### Gemini Browser path

The crate defines an object-safe `PromptPackBrowserExecutor` implemented by a
local app adapter such as `TauriGeminiBrowserPort`. Its methods return boxed
futures and owned results. The app implementation delegates to the retained
Gemini Browser facade for readiness, submission, and cancellation.

The crate owns deterministic Browser run IDs and source labels. The port must
return the complete portable `GeminiBrowserRunResult`, not only completion
text. The crate preserves:

- rejection of `Ready`, empty text, and `Ok + timeout_latest` results;
- `run_id`, status, message, provider mode, completion reason, and elapsed
  time;
- provenance persistence before completion-text validation;
- pre-enqueue cancellation, cancellation callback, and post-completion
  cancellation check;
- the current rule that cancelled Browser work cannot persist success
  provenance.

The crate may use portable config/status/result types from
`extractum-gemini-browser`, but it must not import the app's
`GeminiBrowserState`, `AppBrowserExecutor`, `sidecar`, command facade, or
`AppHandle`.

### Events

The crate owns a typed `PromptPackEvent` lifecycle model and a synchronous
event sink:

```rust
pub trait PromptPackEventSink: Send + Sync + 'static {
    fn emit(&self, event: PromptPackEvent);
}
```

The sink is passed as an owned `Arc` so it can be used by scheduler callbacks.
It is infallible by contract. The crate applies any asynchronous terminal
effect to `PromptPackRunState` before publishing the event. The local app sink
only maps the typed event to the existing serializable
`PromptPackRunEvent`, calls Tauri `emit`, and deliberately ignores emit
failure. Event emission must never turn a successful or failed run into a
different outcome.

The public IPC constant remains exactly `prompt-pack-run-event`. The complete
registered status/event/phase vocabulary in `docs/value-registry.md` remains
unchanged; the extraction neither invents nor removes values. Existing actual
queued, started, terminal, cancellation, repair, and Gem-part sequences and
messages remain byte-compatible.

### Cancellation and task spawning

`PromptPackRunState` and cancellation-token behavior move to the crate; the
app continues to register the value as managed Tauri state. The app owns
`tauri::async_runtime::spawn`, startup cleanup invocation, and command
wrappers. The crate owns run tracking, child tokens, stage cancellation,
LLM-run cancellation requests, terminal cleanup policy, interrupted-run SQL,
and the final domain outcome.

## Error Strategy

Prompt packs continue to use
`extractum_core::error::{AppError, AppErrorKind, AppResult}` for validation,
not-found, conflict, persistence, and internal failures. This matches the LLM
crate because prompt-pack consumers already expect the shared app error
taxonomy.

`YoutubeSummaryStageExecutionError` remains the typed recoverable distinction
between cancellation and failure. `LlmRequestError` and Gemini Browser typed
status values are mapped explicitly. String-prefix or message-substring error
classification is forbidden.

The app preserves the current command error JSON `{ kind, message }`, while
event `error` remains the existing nullable string field. No new error kind or
serialized value is introduced.

## Application-Owned Integration

`extractum` retains:

- all 14 production Tauri command functions and registrations;
- both `#[cfg(dev)]` cancellation-smoke commands;
- `get_pool`, `AppHandle`, `State`, `Manager`, `Emitter`, and task spawning;
- startup seeding and interrupted-run cleanup entry points;
- profile and credential resolution;
- source/YouTube/items SQL reads and the concrete source adapter;
- concrete Gemini Browser readiness, submission, and cancellation adapter;
- mapping `PromptPackEvent` to the legacy IPC DTO and channel;
- migrations, release startup, and cross-domain integration tests.

Every command wrapper obtains app state or the pool, constructs crate-owned
inputs through constructors, delegates, and returns the same response type.
It contains no prompt-pack SQL beyond adapter-owned foreign reads.

The five current result commands must gain crate-owned pool/service functions;
their app wrappers retain only command attributes, pool acquisition, and
delegation. The same rule applies to library, seed, run-list/update/delete,
stage-list, cancellation, and cleanup paths.

## Crate-Owned Behavior

`extractum-prompt-packs` owns:

- prompt-pack library reads and built-in pack seeding behavior;
- run lifecycle, idempotency, state, cancellation, progress, and cleanup SQL;
- runtime-config parsing and persisted Browser provenance;
- stage request policy and all five stage-dispatch bridges;
- API LLM scheduling and provider completion mapping;
- Browser prompt formatting, identity, result policy, and provenance mapping;
- JSON repair and output normalization;
- stage input/artifact persistence;
- schemas, validation, quarantine, canonical result assembly, projections,
  and audit persistence;
- YouTube Summary preflight, source eligibility policy, snapshot formation,
  transcript and Gem analysis, synthesis, result validation, and terminal
  outcome;
- typed events and source/Browser port contracts.

## Current-File Disposition

### Root prompt-pack files

| Current file | Phase 6 disposition |
| --- | --- |
| `completion_transport.rs` | Split during preparation: remove Tauri/AppHandle and concrete Browser calls; move LLM execution, Browser policy, cancellation, and provenance behavior. |
| `dto.rs` | Split: domain request/response/run DTOs move; legacy IPC event DTO and channel mapping remain app-side. |
| `gemini_browser_stage.rs` | Move. |
| `json_repair.rs` | Move. |
| `library.rs` | Move DTOs and pool query; retain one Tauri/get-pool wrapper. |
| `models.rs` | Move. |
| `projections.rs` | Move. |
| `result_builder.rs` | Move. |
| `result_commands.rs` | Split into five crate pool functions and five thin app commands. |
| `run_control.rs` | Move state and cancellation behavior. |
| `run_store.rs` | Move. |
| `runtime.rs` | Split: commands/spawn/profile resolution/IPC stay; domain orchestration, lifecycle SQL, runtime selection, and tests follow their behavior. |
| `runtime_config.rs` | Move. |
| `seed.rs` | Move assets/hash/pool seeding; retain startup get-pool wrapper. |
| `stage_execution.rs` | Move after `AppHandle` is replaced by explicit runtime capabilities. |
| `stage_io.rs` | Move. |
| `stage_output_normalization.rs` | Move. |
| `stage_request_policy.rs` | Move. |
| `store.rs` | Move. |
| `validation.rs` | Move. |
| `mod.rs` | Remain as a private explicit app facade; create a separate curated crate `lib.rs`. |

### `youtube_summary` files

All production behavior in `entities.rs`, `execution.rs`,
`execution_result.rs`, `gem_analysis.rs`, `mod.rs`, `outputs.rs`,
`preflight.rs`, `progress.rs`, `result_validation.rs`, `snapshots.rs`,
`store.rs`, `synthesis_execution.rs`, `synthesis_input.rs`, `tail_stages.rs`,
`transcript_execution.rs`, and `types.rs` moves after seam preparation.

`sources.rs` is replaced by the crate-owned source DTO/port contract plus an
app-owned SQL adapter. It is not copied into both packages.

The eight dedicated test files and test portions of production files follow
the exact ownership map frozen by the implementation plan. `test_support.rs`
is split into crate-private domain fixtures and app integration support; no
test helper is public.

## Bundled Assets

The following eight assets remain single-owned at the canonical location:

```text
src-tauri/prompt-packs/youtube_summary/1.0.0/
|-- pack.json
|-- runtime/synthesis.json
|-- runtime/transcript_analysis.json
|-- schemas/canonical-result.json
|-- schemas/stage-io-youtube-summary-synthesis-output.json
|-- schemas/stage-io-youtube-summary-transcript-analysis-input.json
|-- schemas/stage-io-youtube-summary-transcript-analysis-output.json
`-- stages/transcript_analysis.json
```

One private asset module owns eight named `include_str!` constants. During the
app-side preparation checkpoints, `CARGO_MANIFEST_DIR` is `src-tauri`, so that
module uses `concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/...")`. The
mechanical move changes that single centralized prefix to
`concat!(env!("CARGO_MANIFEST_DIR"), "/../../prompt-packs/...")` because the
new manifest directory is `src-tauri/crates/extractum-prompt-packs`. Seeding,
policy, and validation reuse those constants; no source-depth-relative include
survives preparation.

The bytes, SHA-384 content hashes, schema IDs, runtime configuration, and
persisted `bundled_source_path` remain unchanged. This phase does not add
runtime filesystem reads, `build.rs`, packaging-copy logic, or a second asset
copy.

## Manifest and Dependency Contract

The new member is
`src-tauri/crates/extractum-prompt-packs`, with one app path dependency.

Expected direct production roots after preparation are:

- `extractum-core`;
- `extractum-gemini-browser`;
- `extractum-llm`;
- `jsonschema` with the existing `0.46.5`, `default-features = false`
  declaration;
- `serde` and `serde_json`;
- `sha2`;
- `sqlx = { workspace = true }`, preserving the canonical declaration
  `sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }`
  without adding `default-features = false`;
- `tokio = { workspace = true, features = ["macros", "sync"] }`;
- `tokio-util = { workspace = true }`.

`sha2` and `sqlx` remain direct app dependencies, so their existing canonical
versions/features move to `[workspace.dependencies]` and both packages use
workspace inheritance. `jsonschema` has no non-prompt-pack production use; its
existing declaration transfers to the new crate rather than leaving an unused
app dependency. Production direct `time` usage must first route through
`extractum-core::time`.

The approved dev dependencies are exact:

```toml
[dev-dependencies]
tempfile.workspace = true
time.workspace = true
tokio = { workspace = true, features = ["io-util", "net", "rt", "time"] }
```

`tempfile` is required by the two crate-owned Browser cancellation tests in
`runtime.rs`. The completion-transport tests require Tokio I/O, networking,
runtime, and time support. No moved test uses Tokio's `test-util` API.

Forbidden direct roots include:

- Tauri and every Tauri plugin;
- `keyring`, `secrecy`, and `reqwest`;
- `parking_lot`, `url`, and `zstd`;
- Apalis, Grammers, Windows/process crates, diagnostics infrastructure,
  `extractum`, and `extractum-analysis`.

`zstd` access uses `extractum-core::compression`; provider HTTP and secrets
remain behind `extractum-llm`; Gemini internals remain behind
`extractum-gemini-browser` and the app port.

`src-tauri/Cargo.lock` must be committed with the manifest change. The new
package stanza has this exact dependency list, including dev dependencies:

```text
extractum-core
extractum-gemini-browser
extractum-llm
jsonschema
serde
serde_json
sha2
sqlx
tempfile
time
tokio
tokio-util
```

The app stanza gains `extractum-prompt-packs`, loses direct `jsonschema`, and
retains direct `sha2`, `sqlx`, and `tempfile` for its other production/tests.
The lockfile contract also asserts one app edge, no source/checksum for the new
path package, no reverse edge in the three lower crates, and no registry
resolution churn. In particular, the current resolutions remain
`jsonschema 0.46.5`, `sha2 0.10.9`, `sqlx 0.8.6`, `tempfile 3.27.0`,
`time 0.3.47`, `tokio 1.52.1`, and `tokio-util 0.7.18`.

## Public Rust API and Visibility

The crate root uses private named modules and explicit re-exports. Public glob
exports, public submodules, public test helpers, and public row/schema
implementation types are forbidden.

The curated surface is limited to:

- command-facing request/response, library, run-summary, result, artifact,
  finding, and audit DTOs;
- constructors and the minimum accessors required by thin app wrappers;
- `PromptPackRunState` and the lifecycle methods used by app state wiring;
- `PromptPackEvent` and `PromptPackEventSink`;
- `PromptPackSourceReader`, its boxed-future alias, and the owned DTOs for its
  six source-read operations;
- `PromptPackBrowserExecutor`, its owned request/result inputs, and boxed
  future alias;
- explicit pool/service functions needed by the app commands, startup seed,
  cleanup, start, and execution adapter.

Input DTO fields do not become public merely because the app must construct
them. They receive complete constructors and only required metadata getters.
Response DTO fields remain encapsulated unless a real app consumer reads them.
Serde derives do not justify public fields.

Every `pub(crate)`, `pub(super)`, or private item widened across the crate edge
must be enumerated in the implementation plan and consumed either by the app
facade or another public API item. Widening for test convenience is forbidden.

## IPC and Behavioral Compatibility

The app retains these 14 production command names and registrations exactly:

1. `get_prompt_pack_library`;
2. `preflight_youtube_summary_run`;
3. `start_youtube_summary_run`;
4. `cancel_prompt_pack_run`;
5. `update_prompt_pack_run`;
6. `delete_prompt_pack_run`;
7. `list_prompt_pack_runs`;
8. `list_active_prompt_pack_runs`;
9. `list_prompt_pack_run_stages`;
10. `get_prompt_pack_result`;
11. `list_prompt_pack_stage_artifacts`;
12. `get_prompt_pack_stage_artifact`;
13. `get_prompt_pack_validation_findings`;
14. `list_prompt_pack_audit_events`.

The two dev commands remain separately gated:
`seed_prompt_pack_cancellation_smoke_fixture` and
`clear_prompt_pack_cancellation_smoke_fixture`.

Before seam preparation, characterization tests must pin:

- every command attribute, registration, argument spelling, and response
  shape;
- serialized `StartYoutubeSummaryRunOutcomeDto` started/blocked variants;
- representative queued, started, repair/Gem-stage, terminal, and cancellation
  events, including null fields and exact messages;
- the exact `prompt-pack-run-event` channel and camelCase keys;
- validation, not-found, conflict, and internal `AppError` JSON;
- duplicate request ordering, Browser readiness gating, and empty-ID behavior;
- queued-event-before-spawn ordering, execution-ticket issuance, and the rule
  that profile resolution happens only inside the spawned task;
- fixed API preflight budget `32_000`, Browser-unbounded preflight, and later
  resolved-model budgeting only for Gem execution;
- the complete fresh source-read sequence, including candidate comment text
  followed by a second selected-body read;
- Browser result policy, cancellation, and provenance ordering;
- startup seeding, cleanup, asset hashes, and `bundled_source_path`.

No existing Tauri command, frontend API, event listener, persisted value,
SQLite schema, or asset format changes in this slice. Existing value-registry
paths are updated only for ownership/path changes; no new registry value is
added.

## Frozen Rust-Test Ownership

The baseline is the exact set of 225 logical module/file plus leaf-name
identities in Appendix A, not only the numeric total. New characterization
tests are recorded separately and do not change that baseline.

The approved final partition is 223 identities in
`extractum-prompt-packs` and two in `extractum`. The two app-owned baseline
identities are:

- `prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer`;
- `prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled`.

They remain app-owned because they directly characterize foreign transcript
and comment SQL ordering/selection. Every other Appendix A identity is
crate-owned. In particular, all 40 `runtime.rs` tests are portable domain
tests: their Browser cancellation cases use a generic hook, fake cancellation
tokens, and the lower crate's portable run-log API, not Tauri/AppHandle or the
concrete app adapter. The five preflight tests are domain-policy tests and move
against a fake source reader. Of the eleven snapshot tests, nine move and the
two named above stay.

Tests otherwise follow behavior:

- pure/domain, validation, scheduling, prompt-pack persistence, and pool-level
  tests move;
- Tauri/AppHandle/get-pool, IPC emission, source SQL adapter, concrete Browser
  adapter, migration registration, startup, and cross-domain tests remain in
  the app;
- mixed test modules are split at the prepared seam rather than assigned
  wholesale.

New app-side source-adapter characterizations, recorded outside the frozen 225,
must cover caller order, playlist/transcript order, missing and nullable cases,
comment limit/order/decompression, and exact row-to-owned-DTO conversion. A
new crate-side scripted/counting fake must prove the complete repeated-read
sequence for runnable start, snapshot formation, comment selection, and fresh
selected-body loading. The two mixed app tests must receive separate crate
characterizations for their material-reference and token-cap behavior so that
splitting them loses no domain assertion.

A dedicated source contract must prove every frozen logical test identity
occurs exactly once in the approved owner. It must qualify duplicate leaf
names by their logical module, and reject disabled, renamed, or copied legacy
tests, including `#[cfg(any())]`, false cfg sentinels, commented test blocks,
and `legacy_disabled_*` substitutes. A manual staged rename/diff review is
still required because a scanner cannot prove the absence of arbitrary
renamed copies.

## Crate-Private Test Schema Fixture

Many domain tests currently reach the app-only
`apply_all_migrations_for_test_pool` transitively. The new crate must not
dev-depend on `extractum` and must not export test support.

The selected solution is a crate-private `#[cfg(test)]` schema fixture:

- migration SQL remains single-owned under `src-tauri/migrations`;
- the fixture may `include_str!` an explicit ordered allowlist of canonical
  SQL files, with no copied SQL literals;
- it imports no application Rust module, Tauri migration type, or migration
  runner;
- it is intentionally a prompt-pack test schema bootstrap, not a second
  production migration engine and not “all application migrations”;
- app integration and migration-registration tests continue using the app
  helper.

The approved ordered allowlist is the twelve canonical application files
`0001` through `0012`. This retains the baseline, source, project, prompt-pack,
and current project redesign prerequisites; applying only `0006`–`0011` is
insufficient.

The dedicated boundary contract must enforce this as a standing
source-contract assertion, not a one-time planning check. Its parser:

1. reads the initial `vec![...]` prefix of `build_migrations()` up to
   `migrations.extend(apalis_sqlite_migrations())`;
2. resolves each ordered migration function to its `sql: *_SQL` constant and
   that constant to its canonical `include_str!("../migrations/...")` path;
3. reads the fixture's explicit ordered `(path, include_str!(...))` allowlist;
4. normalizes both sides to repository-relative SQL paths and requires exact
   ordered equality.

An unparseable registry shape is a contract failure, not permission to skip the
check. Any added, removed, reordered, or newly registered non-Apalis migration
therefore fails the contract until the fixture and its consumed-shape
characterizations are updated in the same change, or a new owner-approved
design explicitly changes the policy. The contract also characterizes every
table, column, index, and foreign-key shape consumed by moved tests.

Apalis migration files and the checksum-sensitive inline Apalis SQL are not
part of this reduced fixture. The fixture must not create or claim equivalence
with the app's complete 20-migration history table behavior.

Because `publish = false`, a test-only relative dependency on canonical
workspace SQL is acceptable. Production scans must distinguish that narrow
allowance from forbidden app Rust or migration coupling.

## Source-Boundary Contracts

Before creating the crate, add a deliberately RED
`src/lib/prompt-pack-crate-boundary-contract.test.ts`. It must enforce:

- the exact workspace member and one app path edge;
- exact production/dev dependency roots and features;
- exact Cargo.lock package edges;
- no reverse lower-crate edge and no app/analysis dependency;
- curated explicit crate re-exports and no glob/public test helper;
- the complete visibility-widening allowlist;
- the private app facade and unchanged `src-tauri/src/lib.rs` consumer paths;
- all 225 baseline logical test identities exactly once in their approved
  owners;
- moved-not-copied production and test sources;
- no Tauri, AppHandle, pool acquisition, app migration Rust, app source
  modules, or Tauri emit in production crate code;
- production SQL limited to the exact prompt-pack table allowlist;
- the exact six source operations, their fresh-call rule, and the
  source/Browser/event ports with object-safe boxed futures or synchronous sink
  ABI as specified;
- the opaque execution-ticket and two-phase profile-resolution handoffs;
- canonical asset bytes/path and centralized include ownership;
- the narrow test-only canonical migration SQL allowance and the standing exact
  ordered parity between its fixture list and the registered non-Apalis
  migration prefix.

Update without weakening:

- `rust-workspace-core-contract.test.ts`;
- `gemini-browser-crate-boundary-contract.test.ts`;
- `llm-crate-boundary-contract.test.ts`;
- `crate-extraction-shell-cap-contract.test.ts`;
- `focused-rust-loop-contract.test.ts`;
- `development-loop-performance-contract.test.ts`;
- all six existing raw-path prompt-pack contracts for completion transport,
  run control, run store, runtime config, stage execution, and stage request
  policy.

The implementation plan must inventory any additional raw path readers before
the move so `npm.cmd run verify` is not the first stale-path detector.

## Preparation Checkpoints

All behavioral seam work occurs while prompt-pack code is still app-owned and
the full package can compile it together.

Checkpoints 1–4 each end in a separately identifiable green commit. Every such
checkpoint is behavior-preserving, leaves the existing app-owned module as the
production implementation, runs its named non-empty focused tests, and passes:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Checkpoint 4 additionally runs the full app-package test checkpoint before the
RED contract is introduced. A checkpoint may contain smaller RED/GREEN commits
during development, but its boundary commit must contain no failing test or
dependency on an uncommitted later checkpoint. The slice may stop after any
completed green checkpoint and retain the independently useful preparation;
that outcome is a valid paused Phase 6, not a failed all-or-nothing extraction.

Checkpoint 5 is different by construction: it is a separate intentionally RED
source-contract commit. Existing application behavior and app-package checks
remain green; the new contract must fail only on the absent member/path/move.
It is not combined with the mechanical extraction commit.

### Checkpoint 1 — freeze and characterize

- confirm a clean identity and executable test inventory;
- freeze Appendix A and the production file/table/symbol maps;
- add missing command, IPC, event, error, idempotency, Browser, asset, and
  startup characterization;
- prove all existing tests before changing ownership.

### Checkpoint 2 — safe public construction

- add constructors/accessors needed by future app wrappers;
- remove external struct-literal dependence on fields that will become
  private;
- enumerate every planned visibility widening.

### Checkpoint 3 — isolate app side effects

- introduce the owned source DTOs, six-operation reader, and local SQL adapter;
- introduce the object-safe Browser port and local Tauri adapter;
- introduce typed events, the synchronous sink, and exact IPC mapper;
- introduce the opaque start-service execution ticket and retain profile
  resolution plus task spawning in app code;
- remove `AppHandle`, `State`, `Emitter`, and `get_pool` from future crate
  behavior;
- preserve repeated/fresh source reads, fixed preflight budgets,
  idempotency/queued-event/spawn/profile-resolution order, and cancellation
  ordering.

### Checkpoint 4 — isolate pool APIs and assets

- create pool/service functions for all command-owned prompt-pack SQL;
- centralize all eight compile-time assets;
- route production time/compression/error use through `extractum-core`;
- prepare the crate-private test schema fixture and the frozen 223/2 test
  partition;
- run all 225 logical test identities under their pre-move owner.

### Checkpoint 5 — RED boundary contract

- add the dedicated contract and all existing allowlist/path updates needed to
  describe the approved future state;
- prove the new contract fails only because the member/path/move does not yet
  exist;
- commit that intentional RED separately.

After Checkpoint 5, dependency roots, features, public API, app adapters,
fixture ownership, and all consumer changes are frozen. The physical move may
not repair an unresolved seam.

## Mechanical Move Rule

The extraction commit creates one workspace member and one app path edge,
moves prepared source/test ownership, adjusts module-relative imports and
asset includes, updates Cargo.lock, and rewires the private app facade.

It does not change algorithms, SQL text, serialized values, messages, timeout
or cancellation rules, preflight count, provider policy, schemas, or
frontend behavior. A manual moved-not-copied review compares every prepared
production and test item; generated formatting-only differences are reviewed
separately.

## Rust Verification Loops

Affected packages are `extractum-prompt-packs`, its immediate consumer
`extractum`, and the lower `extractum-llm`, `extractum-gemini-browser`, and
`extractum-core` contracts. Lower packages do not require repeated compilation
unless their own source/API changes; reverse-edge source contracts always run.

Before the move, exact RED/GREEN behavior tests run under `-p extractum`.
After the move, each test runs under its approved final owner. The plan must
name non-empty exact tests spanning:

- DTO/IPC serialization and errors;
- source ordering and start idempotency;
- cancellation and terminal state cleanup;
- LLM scheduling and Browser result/provenance behavior;
- prompt-pack persistence rollback;
- pure validation and result validation;
- full YouTube Summary execution.

For crate-owned behavior:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib <full-test-name> -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
```

Every public cross-crate interface checkpoint also runs the immediate
consumer:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

End-of-slice gates remain:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

All commands use canonical `src-tauri/target`. A filtered Cargo run reporting
zero tests is a failure, not verification.

## Advisory Focused Timing

Timing remains deliberately small and cannot veto a correct extraction.

- Use one stable prepared portable prompt-pack source and one inert marker.
- Baseline targets the owning app package; candidate targets
  `extractum-prompt-packs`.
- Run one discarded warm-up and three recorded focused checks per state.
- Record raw milliseconds and the median only for a complete series.
- Restore source bytes in `finally`; prove SHA-256 identity and a clean
  worktree once after each series.
- Do not add quiet-window scans, Defender/power capture, Job Objects, process
  coordinators, stability thresholds, retries, shell A/B, or a cumulative
  ledger.
- Failure or interruption records `incomplete / no performance conclusion`
  and does not alter retention.

Record the duration already produced by the one mandatory workspace check.
Phase 5's ordinary result was 10,410 ms, below 15,000 ms, so even a Phase 6
result at or above 15,000 ms would be only the first member of a possible
adjacent pair and would not itself trigger investigation.

## Release and Startup Evidence

After correctness gates, run the existing no-bundle release build and a
bounded startup smoke. The smoke proves migration registration, prompt-pack
seeding, command/state wiring, and process survival without making a live
provider request or changing account data. MSI/WiX remains excluded under the
existing roadmap rule.

Infrastructure failure in release/startup machinery is recorded separately
from a confirmed application early exit. Measurement infrastructure never
reclassifies a correctness failure.

## Failure and Rollback

A characterization, preparation, contract, package, workspace, release, or
confirmed startup failure stops progression and retention of the current
checkpoint or extraction candidate; it does not automatically invalidate an
earlier green checkpoint. Fix the failure in the checkpoint that owns it; do
not hide behavioral corrections inside the mechanical move.

Rollback follows the checkpoint ladder:

1. before a checkpoint reaches its green boundary commit, correct or abandon
   only that in-progress work; do not rewrite earlier checkpoint history;
2. after any green Checkpoint 1–4 boundary, the owner may pause the slice,
   retain all completed green commits, and record the last completed checkpoint
   in a short durable disposition and the roadmap;
3. if a completed preparation checkpoint is not independently worth retaining,
   revert that checkpoint and any later dependent preparation checkpoints in
   reverse order with ordinary `git revert`; earlier green checkpoints remain;
4. if work stops after the Checkpoint 5 RED commit but before extraction,
   revert only that unique RED commit and retain Checkpoints 1–4;
5. if the mechanical extraction or its completion gates fail, preserve the
   failed candidate in Git history, revert the extraction commit first and the
   RED contract commit second, then decide each earlier green preparation
   checkpoint independently.

Resuming later starts from the last retained green checkpoint after confirming
its identity and gates. No stop condition authorizes `git reset`, destructive
path checkout, or manual deletion of evidence.

An advisory timing failure is not a correctness failure. Restore the probe,
record no conclusion, and continue only after source identity and clean-tree
proof.

If the candidate is not retained, write a verification disposition and restore
the roadmap to the truthful retained-checkpoint or non-retained Phase 6 state.

## Non-Goals

- No prompt-pack feature, schema, provider, or frontend redesign.
- No single-preflight optimization or new idempotency recovery behavior.
- No generic repository, unit-of-work, service locator, or universal
  completion executor.
- No `extractum-sqlite`, source, analysis, or test-support crate.
- No SQLx, Tokio, or domain DTOs added to `extractum-core`.
- No migration move, rewrite, deletion, or new migration.
- No public test helpers or upward dev dependency.
- No app-wide import rewrite; the private facade preserves current consumer
  paths.
- No live credentialed provider gate or account mutation.
- No build-tooling, target-directory, linker, or measurement-harness change.

## Acceptance Criteria

Phase 6 may be recorded as implemented and retained only when:

1. the approved 223/2 post-split ownership of all 225 baseline test identities
   is enforced and every logical identity occurs once in the correct package;
2. all new IPC/event/error/source/Browser/asset characterizations pass;
3. production crate code contains no Tauri, AppHandle, app pool acquisition,
   app migration/source imports, foreign-table SQL, or direct Tauri/IPC event
   emission; typed publication through `PromptPackEventSink` is required;
4. prompt-pack-owned runtime queries, DML, and transaction behavior reside in
   the crate, excluding app-owned migrations and the test-only fixture;
5. the source reader preserves ordering, missing-value distinctions, the full
   fresh-read sequence, and repeated preflight semantics;
6. LLM execution is direct through `extractum-llm`, while concrete Browser
   operations and profile resolution remain app-side, with the queued response,
   execution-ticket, and spawned profile-resolution order unchanged;
7. command names/signatures, event channel/payloads, AppError JSON, persisted
   values, asset bytes/hashes/path, and startup behavior are unchanged;
8. the crate root, visibility map, manifest, lockfile, reverse-edge rules, and
   moved-not-copied inventory satisfy the dedicated boundary contract;
9. the crate-private schema fixture uses only canonical single-owned SQL,
   remains test-only, matches its declared reduced scope, and is held in exact
   ordered parity with the registered non-Apalis migration prefix by a standing
   source contract;
10. crate, immediate-app, workspace, repository, release, and startup gates
    pass;
11. advisory timing and ordinary workspace timing are recorded honestly and
    do not decide retention;
12. a durable verification document and roadmap update describe the observed
    result and name Phase 7 only as a future fresh JIT design.

## Implementation-Plan Requirements

The implementation plan must be written only after this specification is
reviewed. It must include:

- the exact Appendix A mapping of 223 crate-owned and two named app-owned
  baseline identities;
- an exact production/test file map, visibility-widening map, source DTO map,
  port API, root re-export allowlist, table allowlist, dependency/features
  allowlist, asset map, migration-fixture allowlist, its registry-parity parser
  and assertions, and Cargo.lock assertions;
- named RED/GREEN tests and the `## Rust Verification Loops` section required
  by repository policy;
- preparation commits before the intentionally RED contract and mechanical
  move, with one separately identifiable green boundary per Checkpoint 1–4 and
  the checkpoint rollback/pause ladder;
- the small advisory timing protocol without additional machinery;
- fail-fast commands, scoped staging, non-destructive rollback, release/startup
  evidence, verification document, and roadmap/contract completion update.

The plan must not infer implementation authorization merely from the existence
of this approved design; execution begins only when the owner explicitly asks
for it.

## Appendix A: Frozen 225-Test Baseline

The following logical file/name pairs are the normative pre-extraction set.
The implementation plan must confirm their executable Cargo identities and
assign each pair to exactly one final owner. Duplicate leaf names in different
logical modules remain separate tests.

### Top-level prompt-pack modules (100)

#### `completion_transport.rs` (2)

- `browser_model_context_has_no_api_fields`
- `api_model_context_retains_profile_and_override`

#### `dto.rs` (2)

- `preflight_request_defaults_to_api_runtime_provider`
- `start_request_accepts_gemini_browser_runtime_provider`

#### `gemini_browser_stage.rs` (3)

- `ok_browser_result_maps_to_completion_text`
- `ready_result_is_not_prompt_completion`
- `timeout_latest_ok_result_is_not_prompt_completion`

#### `library.rs` (1)

- `get_prompt_pack_library_returns_active_youtube_summary_pack`

#### `projections.rs` (5)

- `persist_final_result_sets_terminal_status_after_projection_rows_exist`
- `persist_final_result_does_not_overwrite_cancelled_run_status`
- `persist_final_result_projects_youtube_synthesis_items`
- `persist_final_result_uses_current_time_for_run_completion`
- `low_level_result_persistence_rolls_back_when_projection_insert_fails`

#### `result_builder.rs` (11)

- `build_canonical_result_assigns_backend_owned_ids`
- `gem_analysis_final_output_builds_canonical_single_video_result`
- `build_canonical_result_uses_current_created_at`
- `build_canonical_result_includes_synthesis_output`
- `build_canonical_result_preserves_synthesis_common_claim_text`
- `build_canonical_result_marks_single_video_synthesis_not_applicable`
- `build_canonical_result_marks_multi_video_synthesis_failed`
- `build_canonical_result_marks_multi_video_synthesis_skipped_insufficient_successes`
- `build_canonical_result_keeps_partial_result_flag_when_synthesis_is_skipped`
- `build_canonical_result_uses_intermediate_graph_claims_and_evidence`
- `build_canonical_result_rejects_incomplete_intermediate_graph`

#### `runtime.rs` (40)

- `now_string_uses_current_utc_time`
- `prompt_pack_run_state_tracks_active_and_cancel_requested_runs`
- `prompt_pack_run_state_cancels_child_tokens`
- `prompt_pack_cancellation_smoke_fixture_tracks_active_run`
- `prompt_pack_cancellation_smoke_fixture_clear_cancels_tokens_and_deletes_rows`
- `prompt_pack_run_cancellation_allows_completed_stage_future`
- `prompt_pack_run_cancellation_interrupts_stage_future`
- `prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job`
- `prompt_pack_browser_stage_cancelled_while_active_stops_sidecar`
- `cancelled_browser_stage_does_not_persist_success_provenance`
- `prompt_pack_browser_stage_cancelled_before_enqueue_is_tolerated`
- `terminal_event_removes_run_from_active_state`
- `browser_prompt_formatter_preserves_role_order_and_content`
- `browser_prompt_formatter_rejects_unsupported_roles`
- `browser_run_identity_includes_repair_attempt_when_present`
- `browser_run_id_accepts_optional_gem_discriminator`
- `browser_stage_result_maps_to_prompt_pack_completion_without_tokens`
- `cleanup_interrupted_prompt_pack_runs_marks_stale_active_rows_interrupted`
- `load_run_runtime_config_reads_api_and_browser_rows`
- `load_run_runtime_config_rejects_unsupported_provider`
- `load_run_runtime_config_rejects_malformed_browser_config`
- `list_prompt_pack_runs_returns_recent_runs_for_project`
- `browser_runtime_start_gate_maps_unready_status_to_preflight_failure`
- `browser_runtime_start_gate_allows_ready_status`
- `list_prompt_pack_run_stages_returns_browser_provenance`
- `persist_browser_stage_provenance_records_result_identity`
- `update_prompt_pack_run_updates_user_label_only`
- `delete_prompt_pack_run_rejects_active_runs`
- `gem_analysis_part_llm_request_preserves_part_and_frozen_input`
- `gem_analysis_part_repair_llm_request_preserves_attempt_and_repair_context`
- `transcript_analysis_llm_request_embeds_frozen_stage_input`
- `transcript_analysis_llm_request_uses_detailed_report_prompt_for_control_preset`
- `transcript_analysis_llm_request_describes_candidate_indexes_and_forbids_backend_refs`
- `synthesis_llm_request_describes_allowed_refs_and_forbids_direct_intermediate_refs`
- `transcript_analysis_output_budget_is_clamped_to_model_limit`
- `transcript_analysis_output_budget_comes_from_stage_runtime_config`
- `transcript_analysis_stage_max_prompt_token_budget_reads_runtime_config`
- `gem_input_budget_uses_lower_known_model_limit`
- `detailed_report_control_preset_uses_larger_transcript_analysis_output_budget`
- `synthesis_output_budget_comes_from_stage_runtime_config`

#### `seed.rs` (5)

- `seed_youtube_summary_pack_is_idempotent`
- `seed_youtube_summary_pack_writes_required_schema_assets`
- `seed_youtube_summary_pack_rejects_bundled_hash_conflict`
- `seed_youtube_summary_pack_rejects_user_collision`
- `seed_youtube_summary_pack_preserves_unknown_newer_bundled_version`

#### `stage_io.rs` (3)

- `build_transcript_analysis_stage_input_uses_frozen_registries`
- `transcript_analysis_stage_input_serializes_contract_keys_as_snake_case`
- `insert_stage_artifact_uses_current_time`

#### `stage_output_normalization.rs` (1)

- `synthesis_runtime_normalization_defaults_readable_arrays`

#### `store.rs` (2)

- `prompt_pack_runs_client_request_id_is_unique_when_present`
- `prompt_pack_runs_allow_null_client_request_id_for_pre_existing_rows`

#### `validation.rs` (25)

- `transcript_analysis_output_rejects_unknown_material_ref`
- `transcript_analysis_output_rejects_llm_assigned_final_ids`
- `transcript_analysis_output_rejects_structural_schema_errors`
- `extract_json_payload_accepts_fenced_json_object`
- `extract_json_payload_accepts_leading_and_trailing_prose`
- `extract_json_payload_rejects_malformed_braces`
- `extract_json_payload_rejects_multiple_json_objects`
- `synthesis_output_validator_accepts_valid_output`
- `synthesis_output_validator_rejects_missing_summary_text`
- `synthesis_output_validator_rejects_wrong_stage_io_version`
- `synthesis_output_validator_rejects_wrong_schema_version`
- `synthesis_output_accepts_provider_string_items_for_readable_arrays`
- `synthesis_output_validator_rejects_wrong_stage`
- `synthesis_output_validator_rejects_non_array_fields`
- `synthesis_output_validator_rejects_structural_schema_errors`
- `synthesis_output_validator_rejects_backend_owned_ids`
- `synthesis_output_validator_rejects_unknown_source_ref`
- `synthesis_output_validator_rejects_provider_authored_claim_ref`
- `synthesis_output_rejects_unknown_claim_ref`
- `synthesis_output_rejects_direct_segment_key_point_or_quote_refs_inside_synthesis_candidate`
- `synthesis_output_rejects_non_array_or_non_string_ref_values`
- `invalid_synthesis_output_is_written_to_quarantine_artifacts`
- `synthesis_quarantine_artifact_uses_current_time`
- `invalid_synthesis_output_with_unknown_source_ref_is_quarantined`
- `invalid_synthesis_output_surfaces_quarantine_write_failure`

### `youtube_summary` modules (125)

#### `entities_tests.rs` (11)

- `graph_constants_match_contract`
- `build_source_graph_assigns_backend_refs_and_allowed_refs`
- `textless_segment_is_kept_as_structural_navigation`
- `blank_key_point_is_skipped_with_graph_warning`
- `malformed_candidate_container_is_rejected`
- `invalid_material_ref_is_rejected`
- `evidence_quote_candidate_index_to_missing_quote_is_dropped_with_warning`
- `provider_output_must_not_supply_backend_refs_or_ids`
- `evidence_index_pointing_to_skipped_quote_candidate_is_dropped_with_warning`
- `key_point_index_pointing_to_skipped_segment_candidate_is_dropped_with_warning`
- `graph_builder_uses_persisted_prompt_input_material_registry`

#### `execution_tests.rs` (19)

- `execute_queued_run_with_stage_executor_finishes_complete`
- `gem_analysis_executes_passport_comments_and_deep_recap_in_order`
- `gem_analysis_skips_comments_when_trimmed_comment_text_is_empty`
- `gem_analysis_repairs_invalid_required_part_once`
- `gem_analysis_input_budget_blocks_before_first_provider_call`
- `gem_analysis_required_part_failure_fails_stage`
- `gem_analysis_optional_comments_failure_persists_report_with_failure_note`
- `gem_analysis_does_not_start_next_part_after_cancellation_checkpoint`
- `youtube_summary_invalid_final_result_records_result_level_findings`
- `execute_queued_run_repairs_malformed_transcript_json`
- `execution_graph_build_failure_after_failed_repair_marks_transcript_failed_once`
- `execute_queued_run_repairs_malformed_synthesis_json`
- `execute_multi_video_run_with_one_provider_failure_finishes_partial`
- `youtube_summary_single_video_run_skips_synthesis`
- `youtube_summary_run_executes_synthesis_after_transcript_stages`
- `youtube_summary_run_marks_partial_when_synthesis_fails`
- `youtube_summary_run_marks_partial_when_synthesis_output_is_invalid`
- `youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial`
- `execute_multi_video_run_stops_after_transcript_when_cancelled_before_synthesis`

#### `facade_tests.rs` (1)

- `now_string_uses_current_utc_time`

#### `gem_analysis.rs` (11)

- `gem_materials_load_formats_timestamped_transcript_from_metadata`
- `gem_materials_load_skips_empty_comment_rows`
- `gem_materials_part_prompt_inputs_are_isolated`
- `gem_materials_input_budget_rejects_over_cap`
- `assemble_gem_markdown_nests_part_markdown_under_backend_headings`
- `assemble_gem_transcript_output_contains_empty_candidate_arrays`
- `gem_analysis_part_types_cover_comments_and_stage_variants`
- `parse_part_output_accepts_matching_non_empty_markdown`
- `parse_part_output_rejects_wrong_part`
- `parse_part_output_rejects_empty_markdown`
- `parse_part_output_accepts_json_fence_with_internal_markdown_code_block`

#### `outputs_tests.rs` (15)

- `execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts`
- `execute_synthesis_stage_normalizes_provider_string_readable_items`
- `execute_synthesis_stage_rejects_invalid_output_without_success_artifacts`
- `execute_synthesis_stage_rejects_unknown_claim_ref_with_quarantine`
- `repaired_synthesis_stage_rejects_unknown_claim_ref_with_quarantine`
- `execute_synthesis_stage_requires_complete_intermediate_graph`
- `execute_transcript_analysis_stage_persists_raw_and_parsed_artifacts`
- `execute_transcript_analysis_stage_persists_default_warning_candidates`
- `execute_transcript_analysis_stage_persists_intermediate_entities_artifact`
- `transcript_stage_metrics_can_include_gem_analysis_extension`
- `transcript_success_artifacts_roll_back_when_parsed_insert_fails`
- `repaired_transcript_analysis_persists_intermediate_entities_for_repair_attempt`
- `repaired_transcript_success_artifacts_roll_back_when_parsed_insert_fails`
- `malformed_intermediate_candidates_are_quarantined_without_graph_artifact`
- `repair_graph_build_failure_does_not_write_repaired_parsed_output`

#### `preflight_tests.rs` (5)

- `preflight_explicit_video_without_transcript_is_blocking_failure`
- `preflight_playlist_video_without_transcript_is_skipped`
- `browser_runtime_preflight_does_not_apply_api_input_limit`
- `preflight_gem_analysis_allows_exactly_one_included_video`
- `preflight_gem_analysis_blocks_multiple_included_videos`

#### `result_validation.rs` (47)

- `validate_youtube_summary_canonical_result_valid_minimal_has_no_errors`
- `duplicate_source_ref_id_returns_error`
- `missing_required_top_level_array_returns_error`
- `canonical_result_schema_shape_error_returns_finding`
- `canonical_result_schema_allows_runtime_string_limitations`
- `run_id_mismatch_returns_error`
- `blank_video_id_returns_error`
- `duplicate_video_id_returns_error`
- `duplicate_claim_id_returns_error`
- `duplicate_evidence_id_returns_error`
- `synthesis_object_missing_required_array_returns_error`
- `duplicate_synthesis_item_id_across_item_kinds_returns_error`
- `video_with_unknown_source_ref_returns_error`
- `evidence_with_unknown_claim_id_returns_error`
- `synthesis_top_level_unknown_claim_ref_returns_error`
- `nested_synthesis_unknown_claim_ref_returns_error_when_top_level_union_empty`
- `nested_synthesis_unknown_video_ref_returns_error`
- `synthesis_missing_nested_claim_ref_in_top_level_union_returns_error`
- `synthesis_extra_top_level_claim_ref_returns_error`
- `synthesis_duplicate_top_level_claim_ref_returns_error_at_field_path`
- `synthesis_missing_nested_evidence_ref_in_top_level_union_returns_error`
- `synthesis_extra_top_level_evidence_ref_returns_error`
- `synthesis_missing_source_ref_derived_from_video_ref_returns_error`
- `synthesis_extra_top_level_source_ref_not_in_nested_items_returns_error`
- `synthesis_order_difference_in_top_level_union_is_allowed`
- `synthesis_unknown_video_ref_does_not_cascade_to_source_union_error`
- `synthesis_null_skips_derived_traversal_validation`
- `video_source_refs_missing_self_ref_returns_error`
- `video_source_refs_malformed_shape_returns_error`
- `video_source_refs_with_non_string_item_returns_error`
- `video_source_refs_unknown_ref_returns_error`
- `missing_video_source_refs_is_allowed`
- `video_claim_refs_unknown_ref_returns_error`
- `video_evidence_refs_unknown_ref_returns_error`
- `video_claim_refs_malformed_shape_returns_error`
- `video_evidence_refs_with_non_string_item_returns_error`
- `complete_standard_result_with_empty_videos_returns_error`
- `complete_narrative_only_result_allows_empty_videos`
- `single_video_with_synthesis_object_returns_error`
- `known_quality_flag_emits_advisory_finding_without_error`
- `unknown_quality_flag_is_ignored_by_mvp_validator`
- `validation_persistence_writes_warning_findings_and_persists_result`
- `validation_persistence_replaces_previous_result_level_findings_on_success`
- `validation_error_writes_findings_marks_run_failed_and_skips_result`
- `validation_error_keeps_stage_level_findings`
- `validation_error_removes_stale_persisted_result_and_projections`
- `validation_wrapper_rolls_back_result_findings_when_persistence_fails_after_validation`

#### `snapshots_tests.rs` (11)

- `start_freezes_one_canonical_video_snapshot_with_multiple_origins`
- `start_returns_existing_run_for_duplicate_client_request_id`
- `start_with_recomputed_blocking_preflight_returns_response_without_run`
- `start_with_runtime_blocking_failure_returns_preflight_without_run`
- `duplicate_start_ignores_runtime_blocking_failure`
- `transcript_snapshot_text_is_rendered_from_structured_segments`
- `transcript_text_for_source_uses_segment_renderer`
- `start_persists_gemini_browser_runtime_and_config_snapshot`
- `duplicate_client_request_id_preserves_existing_runtime_provider`
- `comment_snapshot_selection_is_deterministic_when_enabled`
- `gem_analysis_freezes_comments_even_when_include_comments_is_false`

#### `synthesis_input_tests.rs` (5)

- `build_synthesis_stage_input_collects_successful_transcript_outputs`
- `build_synthesis_stage_input_uses_latest_parsed_output_wrappers`
- `build_synthesis_stage_input_merges_intermediate_graphs_and_allowed_refs`
- `build_synthesis_stage_input_orders_graph_by_source_snapshot_id`
- `load_merged_intermediate_entities_rejects_duplicate_refs_across_sources`
