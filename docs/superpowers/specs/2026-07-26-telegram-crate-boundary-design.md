# Telegram Crate Boundary Design

**Status:** Draft for owner review; implementation not authorized
**Date:** 2026-07-26

**Roadmap authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

**Verification-loop authority:**
[`2026-07-17-focused-rust-loop-design.md`](2026-07-17-focused-rust-loop-design.md)

This specification defines the just-in-time Phase 8 boundary for
`extractum-telegram`. It supersedes the short Phase 8 placeholder in the crate
roadmap. Approval of this document will not authorize implementation. Phase 8
is deliberately split into three separately planned and separately retained
sub-slices, and each sub-slice requires an explicit owner instruction.

## Purpose

Phase 8 removes all direct Grammers dependencies, imports, and types from the
application package while preserving every existing Telegram, source-sync,
Takeout, account, session, event, and IPC behavior.

The current tree no longer supports the old whole-module idea of moving
`telegram`, `accounts`, `telegram_session_store`, `secret_store`, and part of
`media.rs`. Grammers types now cross live source sync, peer resolution, topic
refresh, media conversion, and Takeout. Conversely, `accounts.rs` and
`secret_store.rs` have become application-wide coordination and shared
infrastructure. The selected design therefore splits mixed files at the
Grammers boundary rather than assigning them wholesale by historical module
name.

This is an ownership and dependency-hygiene change. It does not redesign
Telegram UX, authorization semantics, source persistence, Takeout behavior,
database schema, secret identifiers, session encryption, IPC, or task
supervision.

## Decision

The selected design is a Grammers-owning integration crate behind a private
application facade:

1. `extractum-telegram` owns all four pinned Grammers dependency roots and
   every Grammers type;
2. the crate owns the Telegram client runtime, login state, session codec,
   live Telegram transport/adaptation, and raw Takeout transport/parsing;
3. the application owns Tauri commands and events, SQLite and migrations,
   generic secure storage, file locations and the current session-file
   temp-write/rename lifecycle, source and item persistence, Takeout
   job/provenance state, and every cross-domain transaction;
4. the boundary consists only of owned values, typed errors, and opaque
   runtime/login/client handles;
5. no public crate API exposes Grammers, SQLx, Tauri, keyring, raw TL,
   `RemoteCall`, or `InvocationError`;
6. the physical work is divided into contract/session preparation (8A), full
   live-source and Takeout boundary preparation while everything still
   compiles in the app (8B), and one mechanical extraction plus final
   dependency removal (8C);
7. each sub-slice is independently green and retainable;
8. timing is reduced to the already-mandatory ordinary workspace check and is
   advisory only.

The final invariant is stronger than merely hiding imports behind an
application module: after 8C the `extractum` package has no direct Grammers
dependency and no Rust source or test reference to a Grammers path or type.

## Alternatives Considered

### Boundary-first extraction in three sub-slices

Prepare the contract/session seam, finish all live-source and Takeout seams
while every raw consumer still compiles in the app, then move the complete
prepared Grammers perimeter and remove the app dependency roots in one
mechanical extraction.

Selected: it gives each step useful retained value, keeps rollback local, and
does not require a temporary public API containing raw Grammers types.

### Crate-first incremental migration

Create `extractum-telegram` immediately and migrate call sites one by one.
This starts the physical extraction sooner, but either exposes temporary raw
Grammers types across the package edge or creates disposable wrapper APIs that
must be redesigned during the same phase.

Rejected: a package existing earlier is not worth an unstable public boundary.

### One big-bang extraction

Move the full current Grammers perimeter in one preparation and extraction
plan.

Rejected: the direct perimeter is approximately 11,281 physical lines across
14 production files, and its source-sync and Takeout halves have distinct
transaction, test, and rollback risks.

## Fresh Evidence Snapshot

The refreshed read-only snapshot was taken on 2026-07-26 at
`4023cbd535cddd96908c1dd44f15913b4f408a6c` with a clean worktree.

- The application manifest has four direct pinned roots:
  `grammers-client`, `grammers-session`, `grammers-mtsender`, and
  `grammers-tl-types`.
- All four use Codeberg revision
  `1f901ce6e973fdcf0e74267f3d8efad5c729daaa`.
- `grammers-client` has 33 explicit occurrences across 12 production files;
  `grammers-session` has 17 across nine; `grammers-mtsender` has three across
  two. `grammers-tl-types` is not named directly from Rust because Takeout
  reaches TL through the client re-export.
- Two hundred source lines use `tl::`.
- The direct Grammers perimeter is 14 production Rust files, 11,281 physical
  lines, and 119 test attributes.
- The historical five-file ceiling
  (`telegram.rs`, `accounts.rs`, `telegram_session_store.rs`,
  `secret_store.rs`, and all of `media.rs`) is only 2,047 physical lines and
  22 test attributes, so it cannot produce the promised dependency outcome.
- A broad current integration selection across accounts, account deletion,
  media, sources, Takeout, Telegram, and session storage lists 251 Cargo test
  identities. This is evidence of the integration surface, not a proposed
  all-move test set.
- The app package currently lists 701 library tests in total.
- Since 2026-06-01, 20 commits touched the audited paths: 17 touched one
  classified scope, two touched two scopes, and one touched four. No semantic
  commit pairs the exact historical candidate with both sources and Takeout;
  the paired cases are formatting or dependency migration.
- `accounts.rs` has the largest application fan-out: ten app roots, six Tauri
  commands, nine SQL sites, and eight non-candidate domains.
- `secret_store.rs` has ten external consumers and serves Telegram, LLM,
  YouTube, diagnostics, and startup. It is generic application infrastructure,
  not a Telegram-owned module.
- `media.rs` has eleven external consumers, but only its Grammers adapter is
  part of Phase 8; pure media metadata already belongs to `extractum-core`.

The fresh evidence supports a physical Grammers boundary but rejects the stale
whole-file ownership map.

## Target Dependency Structure

```text
extractum
  |-- Tauri commands, managed state wrapper, startup and events
  |-- accounts SQL and cross-domain account deletion
  |-- generic SecretStore and app-data/session-file adapter
  |-- source/item/topic storage and shared transactions
  |-- Takeout jobs, cancellation, provenance and UI events
  `-- extractum-telegram
        |-- opaque Telegram runtime, client and login handles
        |-- session model, codec, encryption and legacy decoding
        |-- live peer/message/media/avatar/topic transport and mapping
        |-- concrete Takeout operations, pagination and raw TL parsing
        |-- extractum-core
        `-- all pinned Grammers roots
```

There is one production dependency edge from `extractum` to
`extractum-telegram`. There is no reverse edge. `extractum-core` remains below
both and receives no Telegram or Grammers dependency.

`extractum-telegram` must not depend on the application, Tauri, SQLx, keyring,
the future `extractum-sources` crate, `extractum-analysis`, or
`extractum-prompt-packs`.

## Ownership Boundary

### Crate-owned behavior

`extractum-telegram` owns:

- Grammers `Client`, `SenderPool`, `MemorySession`, `LoginToken`, peer, media,
  message, TL request/response, transport, and invocation-error types;
- the account client pool and runner lifecycle;
- authorization checks and the login state machine;
- conversion between an opaque crate session and the canonical saved-session
  representation;
- session JSON envelope encoding/decoding, XChaCha20-Poly1305 encryption, AAD
  binding, and legacy plaintext decoding;
- Telegram peer resolution and conversion to owned descriptors;
- live message retrieval and conversion to owned item drafts;
- raw media, author, reply, and Telegram-context conversion;
- remote avatar bytes and forum-topic snapshots;
- Takeout export-DC setup, concrete remote operations, migrated-history
  probes, page/range handling, pagination, and raw message parsing;
- Grammers-specific retry/fallback classification already present in those
  operations;
- deterministic unit tests whose subject is one of the behaviors above.

The crate owns network and protocol behavior. It does not own persistence or
application workflow completion.

### Application-owned integration

`extractum` retains:

- all twelve current account and Telegram Tauri commands:
  `list_accounts`, `get_account`, `create_account`, `set_account_phone`,
  `clear_account_phone`, `delete_account`, `tg_init`,
  `tg_is_authenticated`, `tg_get_account_statuses`, `tg_send_code`,
  `tg_sign_in`, and `tg_logout`;
- command registration, `AppHandle`, `tauri::State`, startup restore task
  registration, Tauri event emission, and best-effort emit failure handling;
- `accounts.rs`, its SQL, API-hash compensation behavior, and cross-domain
  account deletion;
- `account_deletion.rs` and blocker coordination across source ingest,
  Takeout, YouTube, Analysis, and LLM;
- `SecretStore`, `SystemSecretStore`, `SecretStoreState`, keyring service
  selection, secure-store diagnostics, and non-Telegram key namespaces;
- app-data path resolution, directory creation, session file existence checks,
  the current temp-file write followed by rename, and deletion;
- all migrations and the application SQL pool;
- provider-neutral source identity readiness, source locks, source/item/topic
  persistence, archive and analysis read-model updates, and ingest provenance;
- every transaction that combines a Telegram-specific row with a generic or
  foreign-domain row;
- Takeout job creation, cancellation state, recovery, durable batches,
  warnings, provenance, terminal status, command wrappers, task spawn, and UI
  events;
- diagnostics and all cross-domain integration tests;
- pure media metadata encoding, decoding, and labels already delegated to
  `extractum-core`.

The app may hold `TelegramRuntime` as managed state, but the runtime's internal
clients, login tokens, sessions, and runners remain private to the crate.

## Three Retainable Sub-Slices

The crate is intentionally not created in 8A or 8B. Current Takeout obtains
the same raw client and session owned by the runtime. Moving runtime/session
before Takeout is prepared would require either a public
`Client`/`MemorySession` escape hatch or a second independently constructed
runtime. The first violates the boundary and the second changes lifecycle and
session behavior. Completing every pure seam inside one package before the
move avoids both.

### 8A — contract and session preparation

8A changes ownership seams while all production code still belongs to
`extractum`.

It must:

- freeze executable test identities and characterize externally observable
  behavior before refactoring;
- introduce owned Telegram DTOs, typed error categories, and opaque
  runtime/login/session concepts behind the private `crate::telegram` facade;
- separate session codec behavior from app path/keyring/file operations;
- introduce `TelegramApiHash` and `SessionEncryptionKey` secret wrappers;
- pin exact command, event, status, session, secret, and error compatibility;
- preserve every current dependency declaration until the preparation is
  green.

8A may contain multiple separately committed preparation checkpoints. A pause
after any green checkpoint is legitimate and the roadmap must name the last
retained checkpoint exactly.

### 8B — complete app-side boundary preparation

8B keeps every implementation in `extractum` and completes all remaining
cross-boundary refactors before a package edge exists.

It must:

- remove raw Grammers values from source storage inputs and outputs;
- split Grammers conversion from app-owned SQL, filesystem, event, and
  transaction behavior in every mixed live-source file;
- prepare opaque runtime/client/login/session operations without exposing raw
  handles to their consumers;
- replace generic public-crate Takeout invocation concepts with concrete
  owned-value operations;
- move raw Takeout response normalization behind those concrete operations
  while code still compiles in one package;
- separate Takeout transport/parsing from app-owned job, cancellation,
  persistence, provenance, and event loops;
- place every crate-assigned production symbol and implementation-local test
  into its final portable file unit behind the private app facade, so 8C does
  not split a mixed Rust file;
- freeze the complete symbol and test move map while keeping all preparation
  contracts GREEN.

At 8B completion:

- all runtime, session, live-source, and Takeout consumers use the final pure
  boundary shape;
- raw Grammers values are confined to the exact prepared implementation
  files assigned to the future crate;
- no `Client`, `MemorySession`, `LoginToken`, peer/TL type, or generic raw
  invocation needs to cross a future package boundary;
- the app remains fully buildable because the physical crate edge has not yet
  been introduced.

8B is useful and retainable even if 8C is deferred: it leaves a cleaner
single-package architecture without duplicating a runtime or exposing raw
protocol types.

### 8C — single mechanical extraction and dependency cleanup

8C creates `extractum-telegram` and mechanically moves the complete prepared
Grammers perimeter:

- runtime and login state;
- client/session ownership and runner lifecycle;
- session model, codec, encryption, and legacy decoder;
- live peer resolution, message/media mapping, avatar transport, and forum
  topic transport;
- Takeout transport, raw response normalization, pagination,
  migrated-history operations, and raw parsing;
- implementation-local tests and raw fixtures assigned to the crate.

The app facade retains commands, account SQL, secret/file adapters,
persistence, transactions, Takeout coordination, and events, all already wired
to pure values by 8A/8B.

At 8C completion:

- all four direct Grammers roots leave the app package;
- no app Rust source or test names a Grammers path or raw TL type;
- the full Phase 8 outcome is complete.

8B and 8C are not combined into one implementation plan without a written
amendment to this design.

## Public Rust API

The crate root is curated. It has no public modules, glob exports, or exported
test helpers.

The boundary uses the following concepts; the implementation plan freezes the
exact signatures and constructor visibility before 8A code begins:

- `TelegramRuntime`: owns account clients, runners, login state, and runtime
  status;
- `TelegramClientHandle`: an opaque cloneable capability for one connected
  account;
- `TelegramLoginAttempt`: opaque sign-in state; it does not expose
  `LoginToken`;
- `TelegramSession`: opaque session state; it does not expose
  `MemorySession`;
- `TelegramRuntimeStatus`: a typed internal-facing status that the app maps to
  the existing wire strings;
- `TelegramApiHash`: API-hash secret material with zeroization through its
  secret container, no secret-bearing `Debug`, and no getter;
- `SessionEncryptionKey`: secret key material with length validation,
  zeroization through its secret container, no `Debug` secret output, and no
  getter;
- `TelegramMessageDraft`: owned message identity, content, author, reply,
  raw-context, and media data needed by app persistence;
- `PeerDescriptor`: owned Telegram identity, typed peer kind, membership, and
  metadata needed by resolution and persistence;
- `ForumTopicSnapshot`: owned topic data;
- `TelegramMediaPayload`: owned media payload compatible with
  `extractum-core` media metadata;
- `MessageRange`, `TakeoutPage`, and `TakeoutMessage`: owned Takeout operation
  values;
- `TelegramError`: a typed, non-IPC error returned to the app facade.

The public API must not contain:

- `grammers_client`, `grammers_session`, `grammers_mtsender`, or
  `grammers_tl_types`;
- `Client`, `MemorySession`, `LoginToken`, `PeerRef`, raw `tl::*`,
  `RemoteCall`, or `InvocationError`;
- `SqlitePool`, `SqliteConnection`, SQL rows, or transaction types;
- `AppHandle`, Tauri state/event/runtime types, or command attributes;
- keyring entries or the generic application `SecretStore`;
- application modules or `AppError`;
- secret-bearing public fields or secret getters.

Public widening is allowlist-only. Each `pub(crate)` to `pub` change must be
named in the implementation plan and enforced by the source-boundary contract.

## Data Flows

### Startup restore and login

```text
app loads account row and secure API hash
  -> app loads session file bytes and session key
  -> crate decodes/decrypts into opaque TelegramSession
  -> crate runtime starts client and runner
  -> crate returns typed status/result
  -> app maps status to existing strings and emits existing events
```

For legacy session input, the crate returns the opaque session plus canonical
encrypted envelope bytes. The app performs the current temp-file write and
rename with its existing success and failure behavior.
Successful sign-in follows the inverse path: the crate produces canonical
session envelope bytes, and the app writes them.

The API hash and session key cross the boundary only in secret wrappers. They
are never returned through public getters, serialized into diagnostics, or
included in `Debug` output.

### Live source operations

```text
app validates identity and acquires source lock
  -> crate resolves peer / fetches owned message or topic batches
  -> app opens the existing transaction
  -> app writes generic and Telegram-specific rows together
  -> app finalizes source state and existing read models
```

The crate returns owned batches rather than a public Grammers iterator or
stream. The implementation may page internally, but app persistence never
receives a borrowed Grammers value.

8A characterization must freeze message ordering, limits and date/range
selection, partial-progress behavior, fallback identity, and the existing
fetch/persist error boundary. The new page or batch seam must not turn a
currently incremental sync into an all-messages-in-memory operation or delay
durable progress past a different failure boundary.

Avatar network download belongs to the crate. Avatar cache path selection,
base64/data-URL presentation where still needed by app models, file writes,
and cache cleanup remain app-owned.

### Takeout

```text
app creates durable job/batch and owns cancellation
  -> app calls a concrete crate Takeout operation
  -> crate performs raw TL/export-DC work and returns owned page data
  -> app checks cancellation and persists the page/provenance
  -> app emits the existing job event and selects the next operation
```

There is no public generic `invoke<R: RemoteCall>` equivalent. Concrete
operations cover the existing export-DC setup, history count, history page,
search-my-messages, migration probe, topic refresh, and finish behavior.

The app remains the loop and transaction owner. The crate does not commit a
page, finish a durable batch, emit a Tauri event, or decide application job
status.

## Schema and Transaction Ownership

All physical migrations remain in `src-tauri/migrations` and are registered by
the application.

Telegram-related logical tables remain app-persisted:

- `accounts`;
- `telegram_sources`;
- `telegram_messages`;
- `telegram_forum_topics`;
- `telegram_topic_resolution_state`;
- `telegram_takeout_batches`;
- `telegram_migrated_history_capabilities`.

Shared tables and projections remain app-owned, including `sources`, `items`,
ingest batches/observations/warnings, topic memberships, identity repair,
analysis documents, archive read models, app settings, and project-source
relationships.

The following transaction families must not be fractured:

1. generic `sources` plus `telegram_sources` upsert;
2. `items` plus Telegram message/topic data, analysis document, archive model,
   and optional ingest observation;
3. ingest batch plus Telegram Takeout detail/provenance;
4. account create/delete compensation and cross-domain blocker coordination;
5. identity repair and legacy metadata cleanup.

`extractum-telegram` therefore has no SQLx production dependency and accepts no
pool, connection, transaction, or repository service locator. Its owned
values are the hand-off into existing app transaction owners.

## Session and Secret Contracts

The following values and behavior are byte- or string-compatible contracts:

- keyring service: `org.ai.extractum`;
- API-hash key: `telegram.account.<account_id>.api_hash`;
- session-key key: `telegram.account.<account_id>.session_key`;
- session file: `telegram_<account_id>.session.json` under app data;
- AAD: `org.ai.extractum.telegram.session.v1.account.<account_id>`;
- envelope version `1`;
- algorithm label `XChaCha20-Poly1305`;
- 32-byte session key;
- URL-safe base64 without padding for nonce and ciphertext;
- legacy plaintext session auto-migration on successful load;
- wrong-account, missing-key, invalid-key, malformed-envelope, encryption,
  and write-failure behavior.

8A characterization tests must pin the canonical JSON/envelope shape,
decryption/account binding, legacy migration decision, exact error strings,
temp-path construction, write/rename behavior, and compensation ordering.
Moving the codec may not silently change serialization, nonce encoding,
associated data, destination-exists behavior, or failure classification. The
slice does not reinterpret the current temp-write/rename helper as a guaranteed
cross-platform replacement primitive or repair it in place.

The generic secure-store implementation and its LLM/YouTube namespaces never
move to `extractum-telegram`.

## Errors, Events, Statuses, and Cancellation

`TelegramError` is a Rust-domain error, not a new wire value. It may classify
validation, authentication, session, transport, remote/flood-wait, canceled,
and internal failures, but the private app facade maps each path to the
existing `AppError` kind and exact serialized message.

8A characterization must freeze the current output of every migrated
error-producing path before classification changes. A mechanical move cannot
improve wording or reclassify an error.

The app retains and preserves:

- `telegram://restore-failure`;
- `telegram://account-status`;
- `sources://takeout-import`;
- Telegram statuses `not_initialized`, `restoring`, `ready`,
  `reauth_required`, and `restore_failed`;
- Takeout statuses `queued`, `running`, `cancel_requested`, `failed`,
  `cancelled`, and `completed`;
- current payload fields, ordering, and best-effort event emission behavior.

Takeout cancellation tokens and job state remain app-owned. The app checks
cancellation at the same observable boundaries around concrete crate
operations. The crate owns only transport cleanup required for a call it
started.

## Manifest and Dependency Contract

8A and 8B do not create a crate or move a Grammers dependency declaration.
8C adds `crates/extractum-telegram` as a workspace member and adds one
application path dependency on it.

The intended final crate production dependency roots are:

- `extractum-core`;
- `base64`;
- `chacha20poly1305`;
- `grammers-client`;
- `grammers-session`;
- `grammers-mtsender`;
- `grammers-tl-types`;
- `rand_core`;
- `secrecy`;
- `serde`;
- `serde_json`;
- `tokio`.

The implementation plan must verify actual use before generating the manifest
and may remove an unused root. Adding another root requires an explicit design
amendment rather than an opportunistic extraction change.

The four Grammers roots keep the exact current Codeberg revision and effective
features. In particular:

- `grammers-client` keeps default features disabled;
- `grammers-session` keeps default features disabled and `serde` enabled;
- `grammers-tl-types` keeps `deserializable-functions`;
- the Takeout owner carries the `deserializable-functions` requirement.

Canonical versions and Git pins belong in `[workspace.dependencies]`; package
manifests inherit them. `base64` remains an app dependency where independently
used, while session-only crypto roots leave the app when their last app use is
moved.

The 8C manifest patch must include the corresponding
`src-tauri/Cargo.lock` hunk and pass `cargo metadata --locked`. Any separately
justified manifest change during 8A/8B carries its own lockfile hunk and may
not pre-move a Grammers ownership edge.

## Current-File Disposition

The implementation plans must freeze a symbol-level move map. The minimum
disposition is:

| Current path | Crate-owned portion | App-owned portion |
| --- | --- | --- |
| `telegram.rs` | client/runtime/login/session ownership | commands, SQL/secret resolution, status/event mapping |
| `telegram_session_store.rs` | saved-session model, codec, encryption, Grammers conversion | app-data path, keyring adapter, file lifecycle |
| `media.rs` | Grammers-to-pure media adapter | private compatibility facade; core metadata remains in core |
| `sources/avatar.rs` | Telegram photo transport | cache paths, writes, cleanup and app presentation |
| `sources/identity.rs` | Grammers peer conversion | DB rows, normalization and source identity policy |
| `sources/items.rs` | message/author/reply/raw/media conversion | SQL and cross-domain item transaction |
| `sources/peer_resolution.rs` | remote resolve and peer mapping | planning, DB identity, cache and orchestration |
| `sources/sync.rs` | live message retrieval and pure identity mapping | locks, settings, persistence and finalization |
| `sources/topics.rs` | remote topic retrieval and mapping | SQL upsert/read models and refresh coordination |
| `takeout_import/export_dc.rs` | export-DC and concrete raw invocation | app error/event adaptation if any remains |
| `takeout_import/forum_topics.rs` | remote Telegram topic operation | batch warnings and refresh coordination |
| `takeout_import/mod.rs` | raw transport, migration probes and page operations | commands, jobs, cancellation, provenance and persistence loop |
| `takeout_import/pagination.rs` | raw page/range parsing and cursor rules | no Grammers-bearing app portion |
| `takeout_import/raw_parse.rs` | raw TL-to-owned draft conversion | no Grammers-bearing app portion |

`accounts.rs`, `account_deletion.rs`, `secret_store.rs`,
`sources/test_support.rs`, Takeout state/recovery, migrations, diagnostics, and
command registration are explicitly app-owned.

No prepared implementation is copied. The final source-boundary contract must
prove each moved production symbol and baseline test identity has exactly one
owner.

## Test Ownership and Boundary Contracts

8A begins by deriving an exact executable identity map from Cargo output. It
must not use a substring count that accidentally includes similarly named
modules. Each baseline identity affected by the symbol move map is assigned
exactly once to:

- `extractum-telegram` when its subject is protocol/runtime/session/conversion;
  or
- `extractum` when its subject is a command, secret/file adapter, SQL,
  transaction, job, event, diagnostic, or cross-domain workflow.

Raw Grammers/TL/session/error fixtures move with the owning crate test. App
tests switch to pure DTOs and opaque fake capabilities. The crate receives no
dev dependency on the application or app test-support modules.

Required standing contracts include:

1. a Phase 8 roadmap/status contract in
   `crate-extraction-shell-cap-contract.test.ts`;
2. a new `telegram-crate-boundary-contract.test.ts`;
3. exact workspace-member allowlist updates in all existing crate contracts;
4. a curated crate-root/public-API allowlist;
5. final absence of Grammers roots and Rust source/test references from the
   app package;
6. the exact 8B prepared implementation symbol map and intentionally absent
   crate/workspace edge;
7. moved-not-copied source and test ownership;
8. exact Grammers Git revision and feature ownership;
9. required `Cargo.lock` changes for manifest-changing slices;
10. frozen command/event/status/session/secret string contracts.

8B ends with GREEN preparation contracts. 8C begins by committing the final
boundary contract intentionally RED only because the crate, workspace edge,
and physical moves are absent, then makes that same contract GREEN without
inventing another seam. Deleting or renaming a path must not make a contract
silently stop checking the underlying symbol or test identity.

A filtered Cargo run that reports zero tests is not evidence. Implementation
helpers must fail closed when a requested exact or prefix suite selects no
tests.

## Draft and Approval Synchronization

This draft commit must atomically:

1. add this specification with status `Draft for owner review`;
2. update the Phase 8 roadmap entry to `design drafted; awaiting owner
   approval`, link this document, and replace the stale whole-file scope with
   the fresh evidence and selected three-slice boundary;
3. update `crate-extraction-shell-cap-contract.test.ts` to recognize that exact
   draft state and the Phase 8 timing override;
4. run the focused contract test.

After the owner reviews the committed text and explicitly approves it, a
separate synchronization commit must:

1. change this document to `Approved; implementation not started`;
2. change the roadmap to `design approved; implementation not started`;
3. update the status contract's current expected value without weakening its
   allowed state vocabulary;
4. run the focused contract again.

No implementation plan is written before the written specification is
explicitly approved.

Before each sub-slice begins, its implementation plan must add the exact
intermediate checkpoint states it can produce to the roadmap status contract.
This avoids a pause forcing either a false roadmap statement or a broken
contract.

## Rust Verification Loops

Every Phase 8 implementation plan must include the repository-required
`## Rust Verification Loops` section.

During 8A and 8B, code remains app-owned and exact RED/GREEN tests use:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib <full-test-name> -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

After 8C creates the crate, domain tests and checkpoints use:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib <full-test-name> -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Every retained sub-slice ends with:

```powershell
npm.cmd run check:rustfmt
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --no-deps
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

The canonical shared `src-tauri/target` remains in use. Phase 8 creates no
measurement worktree or slice-specific target.

## Advisory Timing

Phase 8 has an explicit owner-selected simplification relative to the
roadmap's hot-module sample-series default.

After each retained 8A, 8B, or 8C sub-slice, record only the duration already
emitted by that sub-slice's successful mandatory:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

Do not add a focused mutation probe, baseline/candidate series, discarded
warm-up, median, source restoration protocol, quiet-window scan, retry,
process-tree coordinator, measurement worktree, or timing harness. Timing
never decides retention and never causes rollback.

Intermediate 8A and 8B values are diagnostic only. If Phase 8 completes, the
final 8C ordinary workspace result is the single Phase 8 value that
participates in the roadmap's adjacent completed-phase `>= 15,000 ms` rule.
If Phase 8 stops incomplete, it contributes no result to that rule.

`npm.cmd run verify` may internally execute another workspace check as a
correctness gate. That invocation is not a timing sample and its duration is
ignored. Phase 8 does not intentionally rerun a successful check for
measurement or admit more than one timing result per retained sub-slice.
This design does not weaken the repository-required full gate or claim that
Cargo executes only once; eliminating its duplicate correctness work requires
a separate owner-approved verification-workflow change.

## Release and Startup Evidence

8B and 8C completion require the existing release `--no-bundle` build and
bounded startup smoke because each changes the application-to-runtime
boundary. The smoke:

- proves the launched executable by exact path and PID;
- proves bounded startup and clean exit;
- checks existing command registration without a credentialed Telegram call;
- performs live MCP smoke before any self-managed smoke when both are used;
- leaves app ports free;
- does not create a new process-control or timing harness.

Deterministic Rust tests, not a live account mutation, prove login, session,
source, and Takeout behavior.

## Failure and Rollback

The rollback ladder preserves useful green work:

1. every 8A preparation checkpoint is a separate green commit;
2. every 8B preparation checkpoint is a separate green commit;
3. a pause records the exact last retained checkpoint in the roadmap and a
   durable verification disposition;
4. the final 8C boundary contract is committed RED only after 8B is complete;
5. the 8C mechanical extraction is a separate commit; on failure it is
   reverted while all green 8A and 8B preparation remains;
6. manifest and lockfile changes are reverted with their owning extraction
   commit, never repaired by an unrelated follow-up mutation.

Roadmap states must distinguish at least draft, approved/not-started, the
plan-declared 8A checkpoint states, `8A retained`, `8B retained`,
`done: retained`, and `not retained`.

No failure authorizes `git reset`, destructive path checkout, deletion of
evidence, or silent resumption from a stale plan. Timing failure or slowdown is
not a correctness failure.

## Non-Goals

- No source, item, account, Takeout, or Telegram schema change.
- No migration ownership change.
- No move of `accounts.rs`, `account_deletion.rs`, generic `secret_store.rs`,
  diagnostics, source storage, Takeout jobs, or Tauri commands into the crate.
- No generic repository, SQL port, service locator, event bus, or app-wide
  runtime abstraction.
- No future `extractum-sources` work.
- No change to account deletion compensation or known partial-update
  semantics.
- No new event, status, IPC error value, or frontend behavior.
- No session format, key, path, encryption, AAD, or migration change.
- No raw Grammers public API, even temporarily.
- No live credentialed provider request as a completion gate.
- No unrelated dependency upgrade, warning cleanup, formatting sweep, or
  Grammers revision change.
- No focused timing series, process scanner, quiet-window rule, retry policy,
  cumulative ledger, or new measurement runner.

## Acceptance Criteria

The design outcome is complete only when:

1. 8A, 8B, and 8C each have an approved bounded implementation plan, retained
   verification evidence, and truthful roadmap status;
2. the exact baseline test identity map is preserved with every identity owned
   once;
3. all twelve account/Telegram commands retain their signatures, registration,
   and observable behavior;
4. existing Telegram and Takeout event names, payloads, statuses, order, and
   error strings are unchanged;
5. session encryption, serialization, account binding, path, key identifiers,
   and legacy migration are byte/string compatible;
6. app-owned source, item, topic, ingest, account, and read-model transactions
   remain single-owner workflows with their current ordering;
7. `extractum-telegram` has no Tauri, SQLx, keyring, application, or foreign
   domain dependency;
8. the crate public API is curated and contains no Grammers or secret-bearing
   public field/getter;
9. the application package has one path edge to `extractum-telegram` and no
   direct Grammers dependency;
10. app Rust, including tests, contains no Grammers import, path, raw TL type,
    or direct dependency workaround;
11. the exact Grammers revision and required features live with the crate
    dependency owner and the lockfile is current;
12. crate, app, workspace, frontend/full verification, release build, and
    startup gates pass;
13. timing is recorded as advisory evidence only and does not decide
    correctness or retention.

## Implementation-Plan Requirements

Phase 8 is executed through three plans, in order:

1. `8A` contract and session preparation;
2. `8B` complete live-source and Takeout preparation inside the app;
3. `8C` single mechanical extraction and dependency cleanup.

Each plan must:

- start at the current clean HEAD and record drift from this evidence snapshot;
- refresh direct Grammers paths, fan-in/fan-out, manifest roots/features, and
  exact Cargo test identities;
- include a symbol-level source map and a test-identity ownership map;
- state exact public API and visibility changes;
- name RED/GREEN tests and non-empty suite helpers before implementation;
- include exact manifest and `Cargo.lock` expectations;
- update boundary/status/workspace-member contracts atomically;
- use separately green commits for preparation, RED contract, extraction, and
  verification disposition;
- include `## Rust Verification Loops`, release/startup evidence where
  required, advisory timing, and a recoverable rollback ladder;
- stop and amend the design if an unexpected dependency, raw public type,
  transaction owner, wire change, or cross-domain port is required.

Implementation may not begin from this draft or from approval alone. The owner
must explicitly authorize the next plan after reviewing it.
