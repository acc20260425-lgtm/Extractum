# Telegram Crate Boundary Design

**Status:** Implemented and retained; [verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md)
**Date:** 2026-07-26

**Roadmap authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

**Verification-loop authority:**
[`2026-07-17-focused-rust-loop-design.md`](2026-07-17-focused-rust-loop-design.md)

**Approved Phase 8B narrow-API authority:**
[`2026-07-28-extractum-telegram-8b-preparation.md`](../plans/2026-07-28-extractum-telegram-8b-preparation.md)

**Phase 8C bounded design:**
[`2026-08-01-telegram-8c-extraction-design.md`](2026-08-01-telegram-8c-extraction-design.md)

The linked document records its current approval status and is never, by
itself, implementation authority. After explicit owner approval, it becomes
the forward-only normative authority for 8C where it explicitly supersedes
this umbrella design. It does not rewrite retained 8A or 8B history; every
parent requirement not explicitly superseded remains in force.

**Owner-directed 2026-07-30 forward amendment:** after retained Checkpoint 1,
transitional escape-hatch use-sites are reviewed by an LLM. The TypeScript
contract does not perform Rust semantic use-site analysis; automatic evidence
for those uses is limited to compilation and behavior.

**Owner-directed 2026-07-30 Checkpoint 3 correction:** for unexecuted CP3 and
CP4 only, the three package-private peer-kind constants have an exact
staged-root bridge used solely by `sources::sync::fallback_message_identity`.
Task 5 removes that bridge with the mapper. The CP3 staging allowlist also
includes the shell-cap status contract. Neither correction changes retained
CP1/CP2 history or the schema-v1 authority artifacts.

This specification defines the just-in-time Phase 8 boundary for
`extractum-telegram`. It supersedes the short Phase 8 placeholder in the crate
roadmap. Approval of this document does not authorize implementation. Phase 8
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
Grammers boundary and follows every moved owned type to its transitive
fan-in/fan-out fixed point rather than assigning files wholesale by historical
module name.

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
4. the boundary consists only of owned values, the shared
   `extractum-core` `AppResult` contract, and opaque runtime/login/client
   handles plus the concrete stateful opaque `DialogListing` and
   `TakeoutTransport` capabilities;
5. no public crate API exposes Grammers, SQLx, Tauri, keyring, raw TL,
   `RemoteCall`, or `InvocationError`;
6. the physical work is divided into contract/session preparation (8A), full
   live-source and Takeout boundary preparation while everything still
   compiles in the app (8B), and one mechanical extraction plus final
   dependency removal (8C);
7. each sub-slice is separately green and recoverable, while only 8C completes
   the dependency-removal outcome;
8. timing is reduced to the already-mandatory ordinary workspace check and is
   advisory only.

The two concrete stateful opaque capabilities are the sole architecture
amendment approved with the Phase 8B plan. `DialogListing` is only the frozen
budgeted dialog/avatar operation and `TakeoutTransport` is only the frozen
concrete Takeout operation set. Neither type is a generic invocation,
repository, event, callback, cursor, or stream abstraction. No `PeerLocator`,
public raw/generic cursor or stream other than the exact opaque
`DialogListing`, or generic invocation/provenance callback is authorized.

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
- Following moved values through their known type fan-out adds four files,
  1,714 physical lines, and 21 test attributes. The required app-root module
  wiring adds `lib.rs` (427 lines / 0 tests), for a 19-file / 13,422-line /
  140-test complete known production move-and-touch surface.
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
  part of Phase 8; provider-neutral media metadata already belongs to
  `extractum-core`, while the Telegram-ingest middle layer is assigned below.

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
- the private `crate::media` compatibility facade over provider-neutral core
  metadata plus the Telegram-owned ingest payload.

The app may hold `TelegramRuntime` as managed state, but the runtime's internal
clients, login tokens, sessions, and runners remain private to the crate.

## Media Value Ownership

Phase 8 completes rather than reverses the historical Phase 1 split:

- `extractum-core::media_metadata` remains the single owner of the
  provider-neutral `ItemMediaMetadata`, its codec, and `media_label`;
- `extractum-telegram` becomes the single owner of the Telegram-ingest payload
  and classification layer that Phase 1 intentionally left beside the
  Grammers adapter;
- application `media.rs` remains a private compatibility facade and contains
  no independent definition after 8C.

The middle layer is dependency-pure but semantically the output of Telegram
live/raw adapters; moving it into core would make provider-specific ingest
DTOs foundational merely to avoid a public Telegram value. Phase 8 therefore
uses the lower owning domain, `extractum-telegram`, rather than broadening
core.

The disposition is exact and forbids duplicate or conversion-only types:

- current `ExtractedMediaPayload` is renamed once to public
  `TelegramMediaPayload` with the same `kind` and
  `ItemMediaMetadata` fields;
- current `ExtractedItemPayload` is not retained as a second public DTO; its
  `content`, `content_kind`, and optional media fields become fields of
  `TelegramMessageDraft`;
- current `SourceItemInsert` becomes the single `TelegramMessageDraft`
  provider-to-storage hand-off, also carrying its existing author, published
  time, canonical raw payload, reply/context, and identity data;
- `DocumentSignals`, the content-kind constants, `derive_content_kind`, and
  `derive_document_media_kind` move with the Telegram adapter; only items
  required by app construction/tests are curated public API, and the remaining
  helpers stay crate-private.

App SQL accepts `TelegramMessageDraft` directly. There is no app mirror DTO,
no `ExtractedItemPayload` to `TelegramMessageDraft` mapping, and no duplicate
media payload. The two current media classification tests move to
`extractum-telegram` through the frozen test-identity map.

## Transitive Type Perimeter

The 14-file / 11,281-line / 119-test direct Grammers scan is the dependency
perimeter, not the complete move-and-touch perimeter. The symbol map must
follow every moved value through type fan-out until it reaches only retained
app consumers or crate-owned producers.

The known closure adds four production files with 1,714 physical lines and 21
test attributes:

- `sources/types.rs` (432 lines / 8 tests) defines
  `TelegramMessageIdentity`, its validation, the three Telegram history-peer
  kind constants, and `ITEM_KIND_TELEGRAM_MESSAGE`;
- `ingest_provenance.rs` (910 lines / 7 tests) remains app-owned but consumes
  the identity and item-kind value for provider keys/provenance;
- `takeout_import/migrated_history.rs` (315 lines / 6 tests) remains app-owned
  for capability SQL and policy but constructs the identity.
- `sources/mod.rs` (57 lines / 0 tests) remains the app-owned module shell but
  must remove or redirect the old local re-exports of the moved DTOs,
  item-kind constant, dead generic insert, and `SourceItemInsert` name.

The transitive symbol closure is therefore 18 production files, 12,995
physical lines, and 140 test attributes. Phase wiring also touches the
427-line / zero-test `src-tauri/src/lib.rs` to register the private staging
facade, so the full known production move-and-touch surface is 19 files,
13,422 physical lines, and 140 test attributes. The literal identity map
covers all 140, not only the original 119. Of the 21 type-closure identities,
`sources::types::tests::telegram_message_identity_validation_rejects_invalid_values`
moves with the identity; the other 20 baseline primaries remain app-owned.
One of those 20,
`sources::types::tests::item_kind_constants_match_persisted_wire_values`, is a
predeclared mixed subject: its app identity retains only the two YouTube wire
assertions, while crate companion
`telegram_item_kind_constant_matches_persisted_wire_value` owns the
`ITEM_KIND_TELEGRAM_MESSAGE == "telegram_message"` assertion.
The closure therefore has 21 immutable baseline entries and 22 final
executable identities: 20 app primaries, one crate primary, and this one crate
companion.

Ownership is exact:

- `TelegramMessageIdentity`, its `validate()` behavior, and its three
  crate-private peer-kind constants move from `sources/types.rs` to
  `telegram_impl/dto.rs`;
- `TelegramItemContext` moves from `sources/items.rs` to
  `telegram_impl/dto.rs`;
- `ITEM_KIND_TELEGRAM_MESSAGE` moves once with the Telegram DTOs and is
  publicly re-exported for app persistence/provenance; no second literal owner
  is introduced, and its persisted wire value is tested by the owning crate
  companion;
- `TelegramMessageDraft`, `TelegramMessageIdentity`, and
  `TelegramItemContext` are public owned values of `extractum-telegram`;
- `ingest_provenance.rs` and `takeout_import/migrated_history.rs` change only
  their type/constant import paths and remain application modules.
- `sources/mod.rs` remains an application module shell and owns no duplicate
  Telegram DTO or constant after its re-exports are updated.
- `src-tauri/src/lib.rs` remains the application root and owns only the private
  `telegram_impl` module declaration; command registration does not move.

`TelegramMessageIdentity::validate()` continues to return the shared core
`AppResult` and preserves these exact messages:

- `Unsupported Telegram history peer kind '{}'`;
- `Telegram history peer id must be positive`;
- `Telegram message id must be positive`.

Checks run in this exact order: supported peer kind, positive peer ID, then
positive message ID; the first failing branch wins. Every branch returns
`AppErrorKind::Validation`. Its serialized shape remains exactly
`{"kind":"validation","message":"<the exact message above>"}`, with the
unsupported peer kind substituted verbatim between the existing single
quotes. 8A adds exact characterization for all three messages, kind,
serialization, and precedence before moving the type.

The rename also removes an extraction-created dead seam rather than carrying
it into the public crate API. The current generic `insert_source_item` has no
production caller, and `SourceItemInsert.external_id` is read only by that
`#[allow(dead_code)]` function and tests. Production Telegram insertion already
derives SQLite `external_id` from
`TelegramMessageIdentity.telegram_message_id`. Therefore:

- `TelegramMessageDraft` has no `external_id` field;
- 8A removes the dead generic `insert_source_item`, its dead-code allowance,
  and the unused field after proving the zero-production-caller inventory;
- the baseline
  `insert_source_item_writes_payload_and_skips_duplicates` identity maps to
  `insert_telegram_source_item_writes_payload_and_skips_duplicates` and
  preserves its payload and duplicate assertions through the retained
  production Telegram insertion path;
- raw-parse tests assert the draft's `TelegramMessageIdentity` instead of the
  removed `external_id`;
- the optional draft identity and the existing Takeout missing-identity error
  remain unchanged.

## Three Separately Green Sub-Slices

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
- verify the exact 43 helper-dependent plus three credential-SQL app
  assignments below, classify each of the other 73 identities by subject, and
  add the 21-identity transitive type closure to commit the literal immutable
  140-entry map before any test module is split;
- introduce owned Telegram DTOs, preserve the shared core error contract, and
  prepare opaque runtime/login/session concepts behind the private
  `crate::telegram` facade;
- establish the exact single-owner media disposition above, preserve the
  `crate::media` facade, and prepare `TelegramMessageDraft` as the single
  `SourceItemInsert` replacement rather than a mapped duplicate;
- move the identity/context/item-kind vocabulary into its declared DTO owner,
  preserve all identity validation strings, and remove the dead generic insert
  seam exactly as declared above;
- complete the separately green core-facade inventory and identity-seam
  normalization checkpoint below before moving any DTO or test into the
  staging tree;
- separate session codec behavior from app path/keyring/file operations;
- introduce `TelegramApiHash` and `SessionEncryptionKey` secret wrappers;
- pin exact command, event, status, session, secret, and error compatibility;
- preserve every current dependency declaration until the preparation is
  green.

#### First 8A checkpoint: core-facade inventory and identity-seam normalization

At the frozen evidence snapshot, the exact 19-file production
move-and-touch perimeter contains 166 `crate::<root>` references. Exactly 50
are mechanically equivalent paths resolving through three application
re-export facades:

- 43 `crate::error` references;
- five `crate::compression` references;
- two `crate::time` references.

The counts separate predictable path noise from ownership work, but the
standing mechanical-move rule forbids a mass rewrite of app consumers.
Therefore 8A does not replace all 50 references across mixed and app-owned
regions. Its first separately committed GREEN checkpoint records the
166-reference inventory and changes exactly the six `crate::error` paths in
the final-owner `TelegramMessageIdentity::validate` implementation and its
moving
`sources::types::tests::telegram_message_identity_validation_rejects_invalid_values`
test to direct `extractum_core::error` paths. This includes both
`AppErrorKind::Validation` assertions. The new exact message, serialization,
and precedence characterizations use the same direct-core path from birth.

No production control flow or observable behavior, error kind/message/JSON,
existing test identity, visibility, or dependency declaration changes in that
checkpoint. The other 44 known core-facade references remain unchanged in this
checkpoint. App-owned consumers retain their facade paths; future-owner
references normalize only as their symbols enter staging during 8B. App-owned
sentinels in
`TelegramSourceKind`/`SourceItemsCursor`, item/peer/sync compression, and
Takeout persistence time calls prove that the checkpoint did not expand into a
consumer-wide rewrite.

During 8B, a separately green portable-tree import checkpoint requires zero
`crate::error`, `crate::compression`, or `crate::time` references inside the
exact `src-tauri/src/telegram_impl/**` staging tree and requires direct
`extractum_core` paths there. It changes paths only as final-owner symbols enter
their portable files; it does not mass-rewrite app-owned source. Every other
`crate::<root>` reference remains input to the 8B symbol/coupling map and must
be resolved by ownership rather than global replacement.

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
- build the exact `src-tauri/src/telegram_impl/**` staging layout declared
  below, including the concrete Takeout file split;
- complete the separately green workspace-dependency normalization checkpoint
  without changing the app's direct dependency graph;
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

`8B preparation retained; 8C pending` is not an extraction success or an
independently claimed dependency benefit. Its full disposition is:

> 8B preparation retained; 8C pending: all runtime/session/live-source/Takeout
> package seams are prepared and contract-GREEN inside `extractum`; no crate or
> workspace edge exists yet, all four Grammers roots remain app-owned, and
> Phase 8's dependency-removal outcome is incomplete.

This means only that the separately green preparation is recoverable. If 8C is
deferred or canceled, the owner explicitly chooses whether to retain or revert
that overhead; the roadmap does not report Phase 8 as successful.

### 8C — single mechanical extraction and dependency cleanup

The bounded Phase 8C design linked above specifies the post-8B extraction
protocol, including the exact cross-package test-support preparation and its
preparation-commit content authority. Until that design is explicitly
approved, the clauses in this section remain the current 8C authority. After
approval, the bounded design takes precedence only for the clauses it
explicitly names.

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

The boundary uses the following concepts. Existing-symbol visibility is frozen
by the allowlist below; each implementation plan must additionally freeze the
exact signatures of newly introduced operations before its RED contract:

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
- `TelegramMessageIdentity`: the current five-field history peer/message
  identity plus unchanged `validate() -> AppResult<()>`;
- `TelegramItemContext`: the current five reply/reaction fields;
- `TelegramMessageDraft`: owned optional `TelegramMessageIdentity`,
  `TelegramItemContext`, content, content kind, author, published time,
  canonical raw data, item kind, and optional media needed by app persistence;
  it is the single renamed/re-homed form of current `SourceItemInsert`, absorbs
  the current `ExtractedItemPayload` fields, and deliberately omits the dead
  `external_id` field;
- `ITEM_KIND_TELEGRAM_MESSAGE`: the existing persisted/provider kind constant
  with one crate owner;
- `PeerDescriptor`: owned Telegram identity, typed peer kind, membership, and
  metadata needed by resolution and persistence;
- `ForumTopicSnapshot`: owned topic data;
- `TelegramMediaPayload`: the single renamed/re-homed form of current
  `ExtractedMediaPayload`, containing `ItemMediaMetadata` from core;
- `DialogListing`, `LiveMessageBatch`, and `LiveMessage`: private-field,
  owned live-operation values;
- `TakeoutAttempt`, `TakeoutFallbackKind`, `TakeoutFallback`, `TakeoutPeer`,
  `TakeoutTransport`, `MessageRange`, `TakeoutCount`, `TakeoutPage`, and
  `TakeoutMessage`: private-field owned Takeout operation values.

Public fallible operations return
`extractum_core::error::AppResult<T>` directly. The crate does not re-export
`AppError`/`AppResult` and does not define a parallel public `TelegramError`.

### Public visibility allowlist

8A and 8B establish the final cross-crate visibility while code still compiles
inside the application. 8C changes no visibility. The complete
existing-symbol widening/rename allowlist is:

- `SourceItemInsert` becomes public `TelegramMessageDraft`; its public fields
  are exactly `telegram_identity`, `telegram_context`, `content`,
  `content_kind`, `author`, `published_at`, `raw_data`, `item_kind`, and
  `media`;
- `TelegramMessageIdentity` and its five fields `history_peer_kind`,
  `history_peer_id`, `telegram_message_id`, `migration_domain`, and
  `is_migrated_history` become public, as does `validate`;
- `TelegramItemContext` and its five fields `reply_to_msg_id`,
  `reply_to_peer_kind`, `reply_to_peer_id`, `reply_to_top_id`, and
  `reaction_count` become public;
- `ExtractedMediaPayload` becomes public `TelegramMediaPayload` with exactly
  public `kind` and `metadata` fields;
- current `ResolvedTelegramSource` becomes public `PeerDescriptor` with
  exactly public `external_id`, `title`, `source_subtype`, `is_member`,
  `username`, `access_hash`, and `avatar_bytes` fields;
- current `ForumTopicSnapshot` and its fields `topic_id`, `top_message_id`,
  `title`, `icon_color`, `icon_emoji_id`, `is_closed`, `is_pinned`,
  `is_hidden`, and `sort_order` become public;
- `ITEM_KIND_TELEGRAM_MESSAGE` becomes one public constant.

No existing free function is widened in place. `ExtractedItemPayload` is
absorbed rather than exported. `DocumentSignals`, media/content helpers and
constants, raw Takeout/TL request and pagination types, session envelope
structs, protocol classifiers, and test helpers remain private. Opaque runtime,
client, login, session, secret, live-operation, and Takeout values are
introduced as new final seam types rather than widening a raw existing type.
Every field on `DialogListing`, `LiveMessageBatch`, `LiveMessage`,
`TakeoutAttempt`, `TakeoutFallback`, `TakeoutPeer`, `TakeoutTransport`,
`MessageRange`, `TakeoutCount`, `TakeoutPage`, and `TakeoutMessage` remains
private.

The 8A/8B source contract must enumerate the public items above and the exact
new operation signatures, reject any other `pub` item, and prove the same
allowlist in the staged tree. A need to widen or rename any other existing
symbol requires a design amendment before code changes.

The public API must not contain:

- `grammers_client`, `grammers_session`, `grammers_mtsender`, or
  `grammers_tl_types`;
- `Client`, `MemorySession`, `LoginToken`, `PeerRef`, raw `tl::*`,
  `RemoteCall`, or `InvocationError`;
- `SqlitePool`, `SqliteConnection`, SQL rows, or transaction types;
- `AppHandle`, Tauri state/event/runtime types, or command attributes;
- keyring entries or the generic application `SecretStore`;
- application modules or the app-local `crate::error` facade;
- secret-bearing public fields or secret getters.

Public widening is allowlist-only. Each `pub(crate)` to `pub` change must
already appear in the specification allowlist, be mapped by the implementation
plan, and be enforced by the source-boundary contract.

## Phase 8B Narrow API and Lifecycle Authority

The approved Phase 8B plan freezes the following execution-state, count,
signature, visibility, transition, symbol-disposition, portable-tree, and
generated-artifact authority. These clauses are copied here before RED and
supersede every broader narrative clause in this design. The explicitly
forward-only amendment below applies to unexecuted work after retained
Checkpoint 1 and does not rewrite Checkpoint 1 history:
- [ ] Inspect `git status --short`, the complete scoped diff, and the staged diff before every commit. Stage only the task allowlist; preserve unrelated user changes.
- [ ] Every Checkpoint 1–8 is independently GREEN, separately committed, and
  truthfully represented in the roadmap/design status in that same commit.
  Implementation and pre-status gates remain on the preceding retained status.
  The exact uncommitted `Checkpoint N retained` pair may be written only as the
  candidate lifecycle selector immediately before the final checkpoint gates;
  it is not retained until those gates pass and the same commit records it. A
  failed candidate gate restores the preceding pair before any further fix.
- [ ] Use the canonical shared `src-tauri/target`. Do not create a worktree, alternate target, timing harness, process scanner, quiet-window protocol, retry loop, or temporary Cargo profile.
- [ ] Phase 8B keeps exactly six workspace members. It creates no `extractum-telegram` package, member, manifest, path dependency, package metadata node, or resolved edge.
- [ ] All four Grammers roots remain direct dependencies of `extractum` throughout 8B. Dependency-removal benefit is not claimed before 8C.
- [ ] Do not edit migrations, schemas, frontend runtime/UI code, command signatures/registration, IPC/event payloads, persisted status values, secret identifiers, session format/path/AAD, transaction ownership, outer cancellation-select boundaries, message order/limits/cutoffs, fallback rules, or durable progress boundaries.
- [ ] Do not introduce SQL, Tauri, keyring, filesystem, app-module, `crate::error`, `crate::compression`, or `crate::time` imports into `src-tauri/src/telegram_impl/**`.
- [ ] Staged-to-staged references use only `self::` and `super::`. No staged
  file contains `crate::`, an application alias, or an application re-export.
  Raw Grammers types remain private implementation details, parameters/results
  of the literal final restricted bridge allowlist, or members of the exact
  CP3→CP6 transitional raw bridge frozen below; none is externally public or
  root-re-exported.
- [ ] Except for the exact CP3→CP6 package-private media compatibility facade
  and the exact CP3→CP6 application-facing members of the transitional raw
  bridge frozen below, every application reference to staged API uses an
  explicit `crate::telegram_impl::...` path. Do not add
  `use crate::telegram_impl as ...`, another facade, a glob, or a duplicate
  DTO.
- [ ] Public fallible operations return `extractum_core::error::AppResult<T>` directly. The root does not re-export `AppError`/`AppResult` and defines no `TelegramError`.
- [ ] The public API contains no `Client`, `MemorySession`, `LoginToken`, `PeerRef`, `RemoteCall`, `InvocationError`, raw `tl::*`, SQLx, Tauri, keyring, secret getter, or app type.
- [ ] At CP7/CP8, restricted `pub(super)` bridges are allowed only for the
  literal 69-entry final internal bridge allowlist frozen below. They are not
  root-re-exported. Before CP7, the only additional staged
  restricted/package bridges are the exact CP3→CP6 transitional raw bridge and
  exact CP3→CP4 three-constant peer-kind root bridge named in the next bullet.
  TypeScript does not authorize or reject those temporary definitions or uses;
  the mandatory LLM review does. Every externally reachable public item
  outside the frozen API remains forbidden.
- [ ] The only lifecycle-gated exceptions to the terminal API/visibility rules
  are: (a) the exact four-item CP3→CP6 media compatibility set, including its
  staged-root export and package-private `src-tauri/src/media.rs` facade for the
  exact frozen consumers; and (b) the exact CP3→CP6 transitional raw bridge
  `TelegramClientHandle::{raw_client,raw_session}`,
  `TelegramSession::raw_memory_session`,
  `telegram::{get_client,get_authorized_client}`, and
  `{ResolvedSyncPeer::peer,legacy_peer_ref_from_descriptor}`. The staged raw
  accessors retain only their existing package/super visibility and exact
  callsites; and (c) the exact CP3→CP4 package-private staged-root re-export of
  `{TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER}`
  for the sole `sources::sync::fallback_message_identity` consumer. Task 5
  removes (c) with that mapper. All exception sets are removed/demoted by their
  stated lifecycle and authorize no public module, glob, fifth media item, or
  additional raw/constant bridge.
- [ ] For every unexecuted checkpoint after retained CP1, a fresh independent
  LLM that did not implement the checkpoint reviews the complete scoped
  production/test Rust diff and enough unchanged surrounding source to account
  for every transitional escape-hatch definition, physical use, producer, and
  forwarding continuation against the readable inventories below. The final
  CLEAN record is appended to the cumulative committed verification document
  before candidate status. The
  TypeScript contract must not resolve Rust names/types/imports/scopes,
  reconstruct producer or call graphs, trace data flow, or enforce
  source-use anchors/edges/occurrence fences. For these use-sites, automatic
  evidence is limited to Rust compilation and the named behavior tests. An
  unlisted or ambiguous use blocks the checkpoint. It may be removed followed
  by a clean LLM re-review; retaining it requires an approved plan/design
  amendment.
- [ ] Remove `TelegramClientHandle::{raw_client,raw_session}` and app helpers `get_client`/`get_authorized_client` only after their last consumers use owned operations; none may exist at Checkpoint 7 or 8.
- [ ] App persistence remains incremental. It processes every `LiveMessage` and every `TakeoutMessage` in order, records max IDs before skip/parse decisions exactly as today, and returns a per-entry error only after prior entries have been durably handled.
- [ ] Every live batch performs exactly one raw `messages.getHistory` invoke with `1 <= limit <= 100`; do not build it by draining `MessageIter::next`. It preserves newest-to-oldest order, exact offset-id/offset-date advancement, the pinned Grammers terminal rule, and per-message conversion after app cutoffs.
- [ ] The raw live batch reproduces the pinned Grammers peer-map/session-cache update before returning and performs no second remote call.
- [ ] Grammers client construction explicitly materializes `grammers_client::client::ClientConfiguration::default()`, rejects `auto_cache_peers == false` with `AppError::internal("Grammers client configuration must enable auto_cache_peers")` before `SenderPool::new` or `tokio::spawn`, and calls `Client::with_configuration`. The raw live batch may reproduce the enabled cache side effect only under this fail-closed construction invariant.
- [ ] `initialize_grammers_client` retains the exact configuration-free signature frozen in Task 3. It must not accept `ClientConfiguration`, `auto_cache_peers`, a configuration factory, or any equivalent injected policy. Making the otherwise unreachable false-default branch testable by parameterizing this function moves the invariant to its caller, is forbidden, and requires an approved plan/design amendment.
- [ ] The one declared error-behavior exception is `Messages::NotModified` for `messages.getHistory { hash: 0 }`: pinned Grammers panics, while Extractum deliberately returns `AppErrorKind::Network` with exact message `Telegram returned messagesNotModified for live history batch`. This aligns live history with the existing Takeout page/count typed-rejection convention; it neither unwinds nor accepts `NotModified.count`. No other panic/error-kind/message/JSON behavior may change.
- [ ] The app records an export-DC attempt before each concrete remote call and drains fallback metadata after success, error, or cancellation. On a terminal remote error, failure to record export-DC fallback metadata is best-effort and the original remote error wins. Only-my-messages persistence is mandatory and completes before the explicit search continuation.
- [ ] Avatar download remains staged; avatar cache keys/paths/writes/cleanup and base64/data-URL presentation remain app-owned. The app creates `DialogListing` with the exact 4,000 ms list budget, converts each returned descriptor before requesting the next one, and staged `next()` preserves dialog-next/avatar interleaving plus the exact 750 ms per-photo timeout.
- [ ] The immutable 140 primaries and three declared companions remain the only baseline decompositions. Any fourth decomposition or identity rename stops for a plan/design amendment.
- [ ] Every exact Rust RED is a runtime RED: a compiling test executes exactly once, fails for the named semantic reason, and contains no compiler error. A compile failure or zero-test filter is never RED evidence.
- [ ] No live credentialed Telegram mutation is a gate. Deterministic tests prove login, source, session, and Takeout behavior.
- [ ] If implementation requires another public type, dependency, raw escape hatch, transaction owner, wire change, cross-domain port, or generic invocation/repository/event abstraction, stop before code and amend the approved design and plan.

### Phase 8B Status State Machine

Task 1 installs this closed vocabulary atomically in `telegram-contract-paths.ts`, the Telegram boundary contract, and the shell-cap contract:

1. roadmap `8A preparation retained`; design `Approved; 8A preparation retained; 8B not started` — starting state;
2. roadmap `8B preparation Checkpoint 1 retained` through
   `8B preparation Checkpoint 8 retained`; design
   `Approved; 8B preparation Checkpoint N retained` — the exact uncommitted
   pair selects the candidate lifecycle for the final named gates and becomes
   retained only when those gates pass and the checkpoint commit records it;
3. roadmap `8B preparation retained; 8C pending`; design `Approved; 8B preparation retained; 8C pending` — only after Checkpoint 8 release/startup evidence and the durable verification document;
4. `done: retained` and `not retained` remain future/rollback states and are not produced by this plan.

`TelegramLifecycle` gains exact `8b-checkpoint-1` through `8b-checkpoint-8` values; the existing terminal 8B layout becomes `8b-preparation`. Per-path resolution is checkpoint-aware:

| Staged owner path | First physical owner checkpoint |
| --- | ---: |
| `telegram_impl/{lib,dto,media,runtime,session}.rs` | 3 |
| `telegram_impl/live/{mod,avatar,peer}.rs` | 4 |
| `telegram_impl/live/{messages,topics}.rs` | 5 |
| `telegram_impl/error.rs` | 4 |
| `telegram_impl/takeout/**` | 7 |

Checkpoint 6 adds the two raw-parse companion assertions at their current `takeout_import::raw_parse::tests` temporary IDs; Checkpoint 7 moves them to the staged IDs. No lifecycle infers a path from existence alone.

## Normative Immutable Test Authority

The immutable map is included here by exact content-addressed reference:

```text
source_path = docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
start_heading = ## Literal Immutable 140-Test Identity Map
end_marker = LF + ### Exact New-Test Identity Map
normalized_lf_bytes = 37436
sha256 = ceab6cef728d396bf2136207f2130974dee2cc0be3c5184eabd8c8de5e58b3ca
```

The retained 8A addition table is independently content-addressed:

```text
source_path = docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
start_heading = ### Exact New-Test Identity Map
end_marker = LF + --- + LF
normalized_lf_bytes = 3717
sha256 = a8dce5a0a00ac8cdcf83ef7eab2304f482e7c3967ec26ab8c8270d6fde42f539
```

Task 1 makes the contract normalize CRLF to LF, slice exactly between those headings, verify byte count and SHA-256 before parsing, and continue to require:

```text
140 immutable primaries = 99 app + 41 future-owner
3 companions = item-kind + 2 deferred raw-parse companions
143 eventual baseline-derived identities
18 Phase-8A additions = 1 companion + 17 additional verification
8A retained = 141 baseline-derived / 158 tracked
8B before new seam tests = 143 baseline-derived / 160 tracked
```

No task edits the imported table or its 8A exact-addition table. A hash mismatch stops execution and requires a reviewed authority amendment.

### Exact Phase 8B New-Test Table

These 15 seam tests are additional verification, not new baseline companions.
Fourteen belong to the staged future owner; one is the app-owned live batch
coordinator test required to preserve incremental durability and the raw
message limit:

| Checkpoint | Exact identity | Subject |
| ---: | --- | --- |
| 3 | `telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check` | non-authorized opaque lookup |
| 4 | `telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure` | 750 ms owned-byte avatar behavior |
| 4 | `telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget` | dialog/avatar interleaving, 4 s cutoff, order, and owned descriptors |
| 4 | `telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes` | app-owned plan primitives and exact outcomes |
| 5 | `telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule` | one raw invoke, cache update under the validated `auto_cache_peers` invariant, 1..=100 limit, offsets, ordering, terminal rule, typed `NotModified` rejection |
| 5 | `telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload` | per-entry conversion and empty skip |
| 5 | `telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor` | owned topics/deletions/pagination |
| 5 | `sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error` | app coordinator durability, raw-message limit, and no post-error fetch |
| 7 | `telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error` | attempt snapshot and fallback queue after success/error |
| 7 | `telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges` | self-check/init/split selection |
| 7 | `telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity` | migration detect/revalidate |
| 7 | `telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome` | classified fallback queue and owned only-my count |
| 7 | `telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages` | concrete page/search operations |
| 7 | `telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping` | concrete finish behavior |
| 7 | `telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots` | post-Takeout remote topic result |

The two deferred companions are separate and retain their design-mandated IDs:

```text
telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids
telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id
```

Starting from 719 current library identities, Phase 8B adds exactly 17 tests:
two companions plus 15 additional seam tests. The final executable library set
is therefore exactly 736 unique IDs. The Phase 8 tracked subset is exactly 175:
104 app identities and 71 staged future-owner identities. Any other count
requires a plan amendment before implementation continues.

Checkpoint accounting is exact:

| Retained checkpoint | Unique `extractum` library IDs | Present baseline-derived | Present Phase-8 tracked |
| ---: | ---: | ---: | ---: |
| 1 | 719 | 141 | 158 |
| 2 | 719 | 141 | 158 |
| 3 | 720 | 141 | 159 |
| 4 | 723 | 141 | 162 |
| 5 | 727 | 141 | 166 |
| 6 | 729 | 143 | 168 |
| 7 | 736 | 143 | 175 |
| 8 | 736 | 143 | 175 |

## Exact Portable Tree

Checkpoint 7 must produce exactly these 19 Rust files and no other file below the root:

```text
src-tauri/src/telegram_impl/
  lib.rs
  dto.rs
  error.rs
  runtime.rs
  session.rs
  media.rs
  live/
    mod.rs
    avatar.rs
    peer.rs
    messages.rs
    topics.rs
  takeout/
    mod.rs
    types.rs
    transport.rs
    export_dc.rs
    operations.rs
    pagination.rs
    raw_parse.rs
    forum_topics.rs
```

Checkpoint 8 writes `src/lib/telegram-8b-staging-sha256.json` with schema:

```json
{
  "schemaVersion": 1,
  "algorithm": "sha256",
  "root": "src-tauri/src/telegram_impl",
  "files": [
    { "path": "dto.rs", "sha256": "generated lowercase SHA-256" }
  ]
}
```

The real artifact contains 19 records sorted by forward-slash relative path. `scripts/telegram-staging-sha256.mjs --write` is the only generator; `--check` recomputes from bytes and fails on a missing/extra path, order drift, root drift, or hash mismatch. The 8C plan must compare the same relative-path/hash records against `src-tauri/crates/extractum-telegram/src`.

## Frozen Final Public API

Task 0 copies this exact signature authority into the design. Task 1 makes the
source contract reject any other terminal root re-export, `pub` item, public
field, or public method and lifecycle-gates the exact CP3→CP6 media exception
described below.

```rust
// telegram_impl/dto.rs
pub const ITEM_KIND_TELEGRAM_MESSAGE: &str = "telegram_message";

pub struct TelegramMessageIdentity {
    pub history_peer_kind: String,
    pub history_peer_id: i64,
    pub telegram_message_id: i64,
    pub migration_domain: Option<String>,
    pub is_migrated_history: bool,
}
impl TelegramMessageIdentity {
    pub fn validate(&self) -> extractum_core::error::AppResult<()>;
}

pub struct TelegramItemContext {
    pub reply_to_msg_id: Option<i64>,
    pub reply_to_peer_kind: Option<String>,
    pub reply_to_peer_id: Option<String>,
    pub reply_to_top_id: Option<i64>,
    pub reaction_count: Option<i64>,
}

pub struct TelegramMessageDraft {
    pub telegram_identity: Option<TelegramMessageIdentity>,
    pub telegram_context: TelegramItemContext,
    pub content: Option<String>,
    pub content_kind: &'static str,
    pub author: Option<String>,
    pub published_at: i64,
    pub raw_data: Vec<u8>,
    pub item_kind: String,
    pub media: Option<TelegramMediaPayload>,
}

pub struct PeerDescriptor {
    pub external_id: String,
    pub title: String,
    pub source_subtype: String,
    pub is_member: bool,
    pub username: Option<String>,
    pub access_hash: Option<i64>,
    pub avatar_bytes: Option<Vec<u8>>,
}

pub struct ForumTopicSnapshot {
    pub topic_id: i64,
    pub top_message_id: i64,
    pub title: String,
    pub icon_color: i64,
    pub icon_emoji_id: Option<i64>,
    pub is_closed: bool,
    pub is_pinned: bool,
    pub is_hidden: bool,
    pub sort_order: i64,
}

// telegram_impl/media.rs
pub struct TelegramMediaPayload {
    pub kind: String,
    pub metadata: extractum_core::media_metadata::ItemMediaMetadata,
}

// telegram_impl/session.rs
pub struct SessionEncryptionKey(/* private secret container */);
impl SessionEncryptionKey {
    pub fn try_from_encoded(
        encoded: secrecy::SecretString,
    ) -> extractum_core::error::AppResult<Self>;
    pub fn generate() -> (Self, secrecy::SecretString);
}

pub struct TelegramSession { /* private Grammers session */ }
impl TelegramSession {
    pub fn empty() -> Self;
    pub(super) fn clone_memory_session(
        &self,
    ) -> std::sync::Arc<grammers_session::storages::MemorySession>;
}

pub fn session_json_requires_existing_key(
    json: &str,
) -> extractum_core::error::AppResult<bool>;
pub fn decode_session_json(
    json: &str,
    account_id: i64,
    key: Option<&SessionEncryptionKey>,
) -> extractum_core::error::AppResult<TelegramSession>;
pub async fn encode_session_json(
    session: &TelegramSession,
    account_id: i64,
    key: &SessionEncryptionKey,
) -> extractum_core::error::AppResult<String>;

// telegram_impl/runtime.rs
pub struct TelegramApiHash(/* private secret container */);
impl TelegramApiHash {
    pub fn new(value: secrecy::SecretString) -> Self;
}

pub enum TelegramRuntimeStatus {
    Ready,
    ReauthRequired,
}

pub struct TelegramClientHandle { /* private client/session capability */ }
pub struct TelegramLoginAttempt { /* private login token and phone */ }
pub struct TelegramRuntime { /* private account map/test callbacks */ }

impl TelegramRuntime {
    pub fn new() -> Self;
    pub async fn initialize_account(
        &self,
        account_id: i64,
        api_id: i32,
        api_hash: TelegramApiHash,
        session: TelegramSession,
    ) -> extractum_core::error::AppResult<TelegramRuntimeStatus>;
    pub async fn is_authenticated(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<bool>;
    pub async fn request_login_code(
        &self,
        account_id: i64,
        phone: String,
    ) -> extractum_core::error::AppResult<()>;
    pub async fn sign_in(
        &self,
        account_id: i64,
        code: String,
    ) -> extractum_core::error::AppResult<TelegramSession>;
    pub async fn client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle>;
    pub async fn authorized_client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle>;
    pub async fn clear_account(&self, account_id: i64, sign_out: bool);
}

// telegram_impl/live/peer.rs
pub struct DialogListing { /* private dialog iterator/start/budget/client */ }

// telegram_impl/live/messages.rs
pub struct LiveMessageBatch { /* private messages/offset/terminal fields */ }
pub struct LiveMessage { /* private raw message plus owned peer lookup */ }

impl LiveMessageBatch {
    pub fn take_messages(&mut self) -> Vec<LiveMessage>;
    pub fn is_terminal(&self) -> bool;
    pub fn next_offset_id(&self) -> i32;
    pub fn next_offset_date(&self) -> i32;
}
impl LiveMessage {
    pub fn message_id(&self) -> i64;
    pub fn published_at(&self) -> i64;
    pub fn into_draft(
        self,
        source_title: Option<&str>,
    ) -> extractum_core::error::AppResult<Option<TelegramMessageDraft>>;
}

// telegram_impl/runtime.rs; delegates through live::* restricted facades
impl TelegramClientHandle {
    pub fn dialog_listing(
        &self,
        avatar_budget_ms: u64,
    ) -> DialogListing;
    pub async fn resolve_dialog_peer(
        &self,
        peer_id: i64,
        expected_subtype: Option<&str>,
    ) -> extractum_core::error::AppResult<PeerDescriptor>;
    pub async fn resolve_username(
        &self,
        username: &str,
        expected_subtype: Option<&str>,
    ) -> extractum_core::error::AppResult<Option<PeerDescriptor>>;
    pub async fn peer_avatar_bytes(
        &self,
        peer: &PeerDescriptor,
    ) -> Option<Vec<u8>>;
    pub async fn fetch_message_batch(
        &self,
        peer: &PeerDescriptor,
        offset_id: i32,
        offset_date: i32,
        limit: usize,
    ) -> extractum_core::error::AppResult<LiveMessageBatch>;
    pub async fn fetch_forum_topics(
        &self,
        peer: &PeerDescriptor,
    ) -> extractum_core::error::AppResult<
        Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>,
    >;
}

impl DialogListing {
    pub async fn next(
        &mut self,
    ) -> extractum_core::error::AppResult<Option<PeerDescriptor>>;
}

// telegram_impl/takeout/types.rs
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TakeoutFallbackKind {
    ExportDc,
    OnlyMyMessages,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TakeoutAttempt { /* private home_dc_id/export_dc_id fields */ }
impl TakeoutAttempt {
    pub fn home_dc_id(&self) -> i32;
    pub fn export_dc_id(&self) -> i32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TakeoutFallback { /* private kind/warning/provenance fields */ }
impl TakeoutFallback {
    pub fn kind(&self) -> TakeoutFallbackKind;
    pub fn warning(&self) -> &str;
    pub fn provenance_message(&self) -> Option<&str>;
}

// telegram_impl/takeout/transport.rs
pub struct TakeoutTransport { /* private client/DC/fallback state */ }
impl TakeoutTransport {
    pub fn export_dc_attempt(&self) -> TakeoutAttempt;
    pub fn drain_fallbacks(&mut self) -> Vec<TakeoutFallback>;
}

// telegram_impl/takeout/types.rs
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TakeoutPeer { /* private subtype/kind/id/access-hash fields */ }
impl TakeoutPeer {
    pub fn from_descriptor(
        descriptor: &PeerDescriptor,
    ) -> extractum_core::error::AppResult<Self>;
    pub fn peer_kind(&self) -> &'static str;
    pub fn peer_id(&self) -> i64;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageRange { /* private min/max fields */ }
impl MessageRange {
    pub fn min_id(&self) -> i32;
    pub fn max_id(&self) -> i32;
}

pub struct TakeoutCount { /* private count/only-my fields */ }
impl TakeoutCount {
    pub fn count(&self) -> i64;
    pub fn only_my_messages(&self) -> bool;
}

pub struct TakeoutPage { /* private cursor/messages/continuation fields */ }
impl TakeoutPage {
    pub fn take_messages(&mut self) -> Vec<TakeoutMessage>;
    pub fn has_next(&self) -> bool;
    pub fn only_my_messages(&self) -> bool;
    pub fn take_pagination_fallback_warning(&mut self) -> Option<String>;
}

pub struct TakeoutMessage { /* private raw TL message */ }
impl TakeoutMessage {
    pub fn message_id(&self) -> i64;
    pub fn into_draft(
        self,
        source_title: Option<&str>,
    ) -> extractum_core::error::AppResult<Option<TelegramMessageDraft>>;
}

// telegram_impl/runtime.rs; delegates through takeout::* restricted facades
impl TelegramClientHandle {
    pub async fn takeout_self_check(
        &self,
    ) -> extractum_core::error::AppResult<()>;
    pub async fn prepare_takeout(
        &self,
    ) -> extractum_core::error::AppResult<TakeoutTransport>;
    pub async fn takeout_forum_topics(
        &self,
        peer: &PeerDescriptor,
    ) -> extractum_core::error::AppResult<
        Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>,
    >;
}

impl TakeoutTransport {
    pub async fn init(
        &mut self,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<i64>;
    pub async fn message_ranges(
        &mut self,
        takeout_id: i64,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<(i64, Vec<MessageRange>)>;
    pub async fn validate_peer(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<()>;
    pub async fn detect_supergroup_migration(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<Option<i64>>;
    pub async fn revalidate_migrated_peer(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
    ) -> extractum_core::error::AppResult<Option<(i64, TakeoutPeer)>>;
    pub async fn history_count(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<TakeoutCount>;
    pub async fn search_my_history_count(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
    ) -> extractum_core::error::AppResult<TakeoutCount>;
    pub async fn history_page(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        count: &TakeoutCount,
        previous: Option<&TakeoutPage>,
    ) -> extractum_core::error::AppResult<TakeoutPage>;
    pub async fn search_my_history_page(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        count: &TakeoutCount,
        previous: Option<&TakeoutPage>,
    ) -> extractum_core::error::AppResult<TakeoutPage>;
    pub async fn finish(
        &mut self,
        takeout_id: i64,
        success: bool,
    ) -> extractum_core::error::AppResult<()>;
}
```

`TelegramApiHash`, `TelegramRuntimeStatus`, `TelegramClientHandle`,
`DialogListing`,
`TelegramLoginAttempt`, `TelegramRuntime`, `SessionEncryptionKey`,
`TelegramSession`, `session_json_requires_existing_key`,
`decode_session_json`, and `encode_session_json` retain their Phase 8A
external visibility/signatures. There are two explicit internal/API-surface
changes: add public `TelegramRuntime::client`, and add restricted
`TelegramSession::clone_memory_session(&self) -> Arc<MemorySession>` while the
borrowed restricted
`TelegramSession::raw_memory_session(&self) -> &Arc<MemorySession>` coexists
only through CP6 and is deleted at CP7. This is a coexist-then-replace
transition, not an in-place signature change. `initialized_client` stays
private and `authorized_client` stays public.
Private derives and test-only constructors are allowed; no additional
externally reachable production `pub` item exists at CP7/CP8. The exact
CP3→CP6 media compatibility exports and the exact package-private CP3→CP4
peer-kind constant exports named below are the only lifecycle-gated staged-root
exceptions; each is removed/demoted at its stated checkpoint.

The exact restricted internal bridge allowlist is below. A name on the
`live::{...}` or `takeout::{...}` line is a `pub(super)` parent facade called
only by `runtime.rs`, except that
`takeout/forum_topics.rs` is the one additional caller of
`live::fetch_forum_topics`. A leaf name is `pub(super)` to its immediate
parent. Public inherent methods listed in the API block are not duplicated
here. `live::peer::peer_ref_from_descriptor` remains private through the
retained CP4 source and becomes the listed `pub(super)` leaf only at CP5.
`media::extract_raw_item_payload` is introduced as a `pub(super)` leaf at CP5
solely for owned raw live-message conversion; the existing CP3-CP6
`extract_item_payload(&grammers_client::message::Message)` compatibility
signature remains unchanged.

```text
live::{dialog_listing,resolve_dialog_peer,resolve_username,peer_avatar_bytes,fetch_message_batch,fetch_forum_topics}
live::avatar::{peer_photo_bytes_with_timeout,peer_photo_bytes}
live::peer::{DialogListing::new,resolve_dialog_peer,resolve_username,peer_avatar_bytes}
live::peer::peer_ref_from_descriptor
live::messages::fetch_message_batch
live::topics::fetch_forum_topics
media::{derive_content_kind,derive_document_media_kind_from_parts,extract_item_payload,extract_raw_item_payload}
error::{is_non_forum_topic_refresh_error,is_channel_private_error,should_fallback_export_dc_error}
session::TelegramSession::{clone_memory_session,cache_peer_infos}
takeout::{takeout_self_check,prepare_takeout,takeout_forum_topics}
takeout::export_dc::{prepare_export_dc_alias,export_dc_invoke,finish_takeout_session}
takeout::forum_topics::takeout_forum_topics
takeout::operations::takeout_self_check
takeout::pagination::TakeoutPaginationProfile
takeout::pagination::TakeoutPageRequest
takeout::pagination::TakeoutPageRequest::offset_id
takeout::pagination::TakeoutPageRequest::add_offset
takeout::pagination::TakeoutPageRequest::limit
takeout::pagination::TakeoutPaginationCursor
takeout::pagination::TakeoutPaginationCursor::new
takeout::pagination::TakeoutCursorAdvance
takeout::pagination::TakeoutCursorAdvance::cursor
takeout::pagination::TakeoutCursorAdvance::advanced
takeout::pagination::TakeoutCursorAdvance::reached_range_start
takeout::pagination::TakeoutPaginationFallbackReason
takeout::pagination::select_history_splits
takeout::pagination::takeout_page_request
takeout::pagination::next_takeout_cursor
takeout::pagination::should_restart_with_descending_fallback
takeout::pagination::takeout_pagination_fallback_warning
takeout::pagination::parse_takeout_page
takeout::raw_parse::{parse_raw_message,peer_ref_identity,messages_response_count}
takeout::transport::TakeoutTransport::{new,queue_fallback,client,session,home_dc_id,export_dc_id}
takeout::types::TakeoutAttempt::new
takeout::types::TakeoutFallback::new
takeout::types::TakeoutPeer::{new,source_subtype,access_hash}
takeout::types::MessageRange::new
takeout::types::TakeoutCount::new
takeout::types::TakeoutPage::{from_parts,pagination_state}
takeout::types::TakeoutMessage::from_raw
```

The pagination subsection is intentionally one fully qualified symbol per
line; no nested brace syntax is legal. Task 1's
`scripts/telegram-8b-symbol-map.mjs` parses this entire fenced allowlist,
expands only the remaining single-level brace groups, and materializes the
sorted canonical 69-entry result as `restrictedFinalSymbols` in the generated
symbol artifact. The TypeScript contract consumes that generated array instead
of maintaining or parsing a second pagination list. Flattening also
deliberately closes two omissions in the old nested notation by listing the
`TakeoutPageRequest` and `TakeoutCursorAdvance` types themselves. This is not
formatting-only drift: the sibling-visible `takeout_page_request` and
`next_takeout_cursor` functions return those types, respectively, so the types
must themselves be `pub(super)`. The separate `pagination_state` bridge retains
its frozen `(TakeoutPaginationProfile, TakeoutPaginationCursor, usize)` return
tuple.

Every listed bridge is spelled `pub(super)` at its leaf or is reached through
one `pub(super)` parent facade; no `pub(crate)`, `pub(in ...)`, root export, or
other restricted production visibility is allowed. This keeps staged files
free of `crate::` and lets sibling modules collaborate without expanding the
future crate API. This paragraph governs the CP7/CP8 final inventory. The exact
CP3→CP6 raw bridge is a separately generated transitional inventory, may retain
only the existing package/super visibility frozen above, and is excluded from
`restrictedFinalSymbols`.

The final media bridge that replaces the transitional `DocumentSignals`
cross-module construction is exact:

```rust
pub(super) fn derive_document_media_kind_from_parts(
    mime_type: Option<&str>,
    has_video: bool,
    has_audio: bool,
    is_voice: bool,
    is_animated: bool,
) -> &'static str;
```

`DocumentSignals` and all five fields are private again at CP7. The bridge
reuses the existing classifier and introduces no second classification rule.

The pagination types on the allowlist are `pub(super)` only to the `takeout`
parent; they are neither root exports nor public API. `TakeoutPage` stores that
restricted state and exposes it to sibling operations only as:

```rust
pub(super) fn pagination_state(
    &self,
) -> (
    TakeoutPaginationProfile,
    TakeoutPaginationCursor,
    usize,
);
```

This is the sole internal cursor-state bridge and does not authorize a public
cursor/stream.

The live batch implementation is additionally frozen as follows:

- `fetch_message_batch` validates `1 <= limit <= 100`, constructs exactly one
  raw `tl::functions::messages::GetHistory` request with the supplied
  `offset_id` and `offset_date`, and performs exactly one `Client::invoke`;
- it maps the response's messages/users/chats into private owned lookup state
  sufficient to preserve current author, reply, media, raw-data, and fallback
  identity behavior without a second remote call;
- before returning, it reproduces pinned Grammers `build_peer_map` behavior and
  updates the runtime's in-memory `TelegramSession` peer cache only after
  runtime construction has validated that
  `ClientConfiguration.auto_cache_peers` is true, and then for exactly those
  response peers for which pinned Grammers `Peer::auth().is_some()` is true.
  This preserves both levels of the pinned condition. The inner predicate
  includes ordinary chats with default auth and excludes min users/channels
  whose auth is unavailable; it is not an access-hash-only predicate;
- newest-to-oldest response order is retained;
- `Messages::Messages` is terminal; `Messages::Slice` and
  `Messages::ChannelMessages` are terminal exactly when empty or when the first
  returned message ID is less than or equal to the request limit, matching the
  pinned Grammers `fill_buffer` rule;
- for this `GetHistory` request whose `hash` is exactly zero,
  `Messages::NotModified` is the one intentional divergence from pinned
  `fill_buffer`: pinned Grammers panics, whereas the owned live operation
  returns `AppErrorKind::Network` with exact message
  `Telegram returned messagesNotModified for live history batch`. It does not
  use `NotModified.count`. This matches the existing Extractum Takeout
  convention, whose operation-specific messages remain exactly
  `Telegram returned messagesNotModified for Takeout history page` and
  `Telegram returned messagesNotModified for Takeout history count probe`;
- a non-terminal batch advances both offsets from its last raw message;
- the app checks prior-ID and date cutoffs before `into_draft`, then updates
  `max_message_id`, converts, and persists/skips in that order. It never asks
  for the next batch after a cutoff or conversion/persistence error.

The Takeout transition protocol is also part of the API authority:

1. before each standalone concrete `TakeoutTransport` call, the app performs
   the existing cancellation pre-check and durably deduplicates both
   `transport.export_dc_attempt().home_dc_id()` and
   `transport.export_dc_attempt().export_dc_id()`. For a classified
   history-to-search pair, the one outer pre-check covers both calls; the app
   records the current attempt again immediately before search but performs no
   second cancellation pre-check or select;
2. one existing outer `run_takeout_step_with_cancel` future contains the
   concrete history call and, when needed, fallback persistence plus the
   explicit only-my search continuation; no second cancellation select or
   pre-check is inserted between them;
3. the app captures the outer future result and every completed recorder result
   without `?`; after success, error, or cancellation it drains all fallback
   metadata and applies the exact `FallbackRecordState` protocol in Task 7B
   before propagating the captured result. A completed recorder failure is
   never called again;
4. `validate_peer` may return `Ok(())` with an `OnlyMyMessages` fallback;
5. `history_count` or `history_page` returns the classified channel-private
   error and queues `OnlyMyMessages`; after the app durably records that
   fallback, it immediately calls the corresponding
   `search_my_history_count` or `search_my_history_page` concrete operation;
6. no progress event, durable page update, second select/pre-check, or unrelated
   job-state decision is inserted between that fallback record and search
   continuation; dropping the one outer future preserves the existing
   cancellation point;
7. a validation-level `OnlyMyMessages` warning is metadata only: every range
   still tries `history_count` first. Only a classified history error makes the
   returned `TakeoutCount::only_my_messages()` route that range's pages through
   search;
8. shifted-export-DC → home-DC remains internal to one concrete call. Its
   `ExportDc` fallback is drained after either success or error;
9. pagination fallback remains page-local, may recur for separate ranges, and
   is exposed only through `TakeoutPage::take_pagination_fallback_warning`.

The root re-export allowlist is exactly:

```text
ITEM_KIND_TELEGRAM_MESSAGE
TelegramMessageIdentity
TelegramItemContext
TelegramMessageDraft
PeerDescriptor
ForumTopicSnapshot
TelegramMediaPayload
TelegramApiHash
TelegramRuntimeStatus
TelegramClientHandle
DialogListing
TelegramLoginAttempt
TelegramRuntime
SessionEncryptionKey
TelegramSession
session_json_requires_existing_key
decode_session_json
encode_session_json
LiveMessageBatch
LiveMessage
TakeoutAttempt
TakeoutFallbackKind
TakeoutFallback
TakeoutPeer
TakeoutTransport
MessageRange
TakeoutCount
TakeoutPage
TakeoutMessage
```

There are no public modules or glob exports. `DocumentSignals`, all
media/content helpers, raw peer/TL/request/response/pagination types, protocol
classifiers, session envelopes, test callbacks, and constructors not named
above remain private or appear only on the exact restricted bridge allowlist.

At CP3 through CP6 only, `telegram_impl/lib.rs` additionally re-exports this
exact compatibility set for the not-yet-moved app consumers:

```text
DocumentSignals (including its five current fields)
derive_content_kind
derive_document_media_kind
extract_item_payload
```

`src-tauri/src/media.rs` re-exports that same set package-privately during the
transition. Separately, at CP3 and CP4 only, `telegram_impl/lib.rs`
package-privately re-exports
`{TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER}`
for the sole `sources::sync::fallback_message_identity` consumer. Task 5 moves
that mapper into `telegram_impl/live/messages.rs` and removes all three root
exports. No other transitional staged-root item is legal. Task 5 also removes
the `extract_item_payload` live-sync consumer; Task 7 moves raw parsing,
deletes the remaining app compatibility re-exports, removes the four media
root exports, and demotes the leaf helpers/fields to their final private or
exact `pub(super)` visibility before CP7 is retained.

## Symbol-Level Source Map

The two file/responsibility tables immediately below are a readable layout
summary only. They do not authorize a catch-all move, an unnamed helper, or
checkpoint drift. The literal production-symbol disposition table that
follows them is normative and is materialized by
`scripts/telegram-8b-symbol-map.mjs`.

The exact private-parent ownership correction below splits the former grouped
Takeout export disposition across its leaf and parent owners. The normative
source table and the existing generator's grouped-row invariant therefore
contain 100 rows instead of 99; the expanded schema-v1 artifact is regenerated
with the corrected owners. This is a mechanical ownership-map correction, not
Rust-semantic parsing or behavioral test authority.

| Final staged owner | Current production source and exact responsibility |
| --- | --- |
| `telegram_impl/lib.rs` | New private module shell and curated exports; replaces the future-owner module declarations/re-exports at the top of `telegram.rs`. |
| `telegram_impl/dto.rs` | Move all production/tests from `telegram/dto.rs`; move `ResolvedTelegramSource` into public `PeerDescriptor`; move `ForumTopicSnapshot` from `sources/topics.rs`. |
| `telegram_impl/error.rs` | Private shell at CP4; move `is_non_forum_topic_refresh_error` at CP5, then `is_channel_private_error` and export-DC local-transport classification at CP7; terminal `AppError` construction only. |
| `telegram_impl/runtime.rs` | Move all production/tests from `telegram/runtime.rs`; add `TelegramRuntime::client`; own every public `TelegramClientHandle` method and delegate through the exact restricted live/Takeout parent facades; remove raw client/session adapters at Checkpoint 7. |
| `telegram_impl/session.rs` | Move all production/tests from `telegram/session.rs`, including the opaque session/key, envelope/legacy codec, AAD, base64, encryption logic, and permanent restricted `clone_memory_session`/peer-cache bridges. |
| `telegram_impl/media.rs` | Move all production/tests from `telegram/media.rs`, including the Grammers media adapter and private content/media classifiers. |
| `telegram_impl/live/mod.rs` | Private live module shell and the frozen parent facades; their signatures directly name the raw client before delegating to leaves. |
| `telegram_impl/live/avatar.rs` | Move `TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS`, `peer_photo_bytes_with_timeout`, and raw avatar download from `sources/avatar.rs`; expose owned bytes only. |
| `telegram_impl/live/peer.rs` | Own `DialogListing` and its public `next`; move username/dialog transport lookup, raw peer mapping/reconstruction, subtype/member/access-hash helpers, and budgeted dialog/avatar listing from `sources/{peer_resolution,identity,store}.rs`. |
| `telegram_impl/live/messages.rs` | Move raw history fetch and message/media/author/reply/raw-payload/fallback-identity adaptation from `sources/{sync,items}.rs`; own the single-invoke batch seam. |
| `telegram_impl/live/topics.rs` | Move `fetch_all_forum_topics`, cursor/date helpers, raw mapping, and the private topic fetch shared with Takeout. |
| `telegram_impl/takeout/mod.rs` | Private Takeout module shell, curated internal exports, the single private `TAKEOUT_FILE_MAX_SIZE`/`takeout_init_request_for_source_subtype` owner shared by child modules, and the frozen parent facades; their signatures directly name raw client/request types before delegating to leaves. |
| `telegram_impl/takeout/types.rs` | Own `TakeoutAttempt`, `TakeoutFallback*`, `TakeoutPeer`, `MessageRange`, `TakeoutCount`, `TakeoutPage`, and `TakeoutMessage`; `TakeoutPage` stores only the exact sibling-restricted cursor/profile state and exposes the frozen tuple bridge. |
| `telegram_impl/takeout/transport.rs` | Own client/session/DC state, attempt snapshots, and fallback queueing; expose only the exact restricted accessors used by concrete siblings. |
| `telegram_impl/takeout/export_dc.rs` | Move the raw, provenance-free half of `export_dc_invoke_with_provenance`, `ExportDcAlias`, DC selection, shifted/home invocation/fallback mechanics, init-request characterization tests, and all seven mapped tests. |
| `telegram_impl/takeout/operations.rs` | Move self-check, init/finish, validation, migration, split/count/history/search operations, and only-my classification. |
| `telegram_impl/takeout/pagination.rs` | Move all production/tests from `takeout_import/pagination.rs`; own split selection, the exact `pub(super)` TDesktop/descending state types, request/advance fields, restart/warning, and response classification. |
| `telegram_impl/takeout/raw_parse.rs` | Move all production/tests from `takeout_import/raw_parse.rs`, `peer_ref_identity`, raw response count/classification, and both companion tests. |
| `telegram_impl/takeout/forum_topics.rs` | Own the post-Takeout remote topic operation and delegate by relative path to the private live topic fetcher. |

Application residual ownership is exact:

| App file | Residual responsibility |
| --- | --- |
| `src-tauri/src/lib.rs` | App composition/commands and `#[path = "telegram_impl/lib.rs"] mod telegram_impl;`. |
| `src-tauri/src/telegram.rs` | Wire-status mapping, account/credential SQL and secret lookup, managed state, restore, events, and six `tg_*` commands; delete app client lookup helpers. |
| `src-tauri/src/telegram_session_store.rs` | App-data path, keyring, temp/atomic file lifecycle, legacy migration orchestration, and adapter tests. |
| `src-tauri/src/media.rs` | Provider-neutral core re-exports only at CP7/CP8; its exact four-item Telegram compatibility facade is transitional through CP6. |
| `src-tauri/src/sources/store.rs` | Tauri commands, the exact 4,000 ms avatar-budget input, avatar presentation/cache/persistence/SQL, and UI DTO mapping. |
| `src-tauri/src/sources/items.rs` | SQL, transactions, insert/upsert/list/read models, `PreparedSourceItem`, outcomes, and app tests. |
| `src-tauri/src/sources/peer_resolution.rs` | Resolution plans, aggregate failure wording, manual parser, typed DB identity, metadata/cache coordination, and final pure `ResolvedSyncPeer`; its exact raw field plus `legacy_peer_ref_from_descriptor` are transitional through CP6 and deleted at CP7. Dialog not-found/subtype errors are staged. |
| `src-tauri/src/sources/identity.rs` | DB rows, policy enums, normalization, SQL loads, readiness; no raw peer conversion. |
| `src-tauri/src/sources/avatar.rs` | Data URL, 4 s budget, cache key/path/read/write/cleanup. |
| `src-tauri/src/sources/sync.rs` | Provider/lock/settings policy, bounded batch loop, message-by-message persistence, counters/finalization, and command wrapper. |
| `src-tauri/src/sources/topics.rs` | UI/read models, refresh coordination, subtype/SQL checks, upsert/read commands. |
| `src-tauri/src/takeout_import/mod.rs` | Commands, jobs/start records, cancellation, provenance, warnings, range/page selection loop, persistence, progress/events, terminal finalization. |
| `src-tauri/src/takeout_import/forum_topics.rs` | Completion policy, warning/provenance recording, and all three app tests. |
| `src-tauri/src/takeout_import/migrated_history.rs` | Capability SQL, migration policy/errors, and identity construction. |
| `src-tauri/src/ingest_provenance.rs` | Entirely app-owned; only imports change. |
| `src-tauri/src/sources/types.rs` | App-owned; at CP4 add only `SourceSyncTarget::is_member`, populated by the existing `sources.is_member` column through `sources::store::load_source`, so stored descriptors preserve membership without network work. |
| `src-tauri/src/sources/mod.rs` | App module shell and non-Telegram exports; redirects exact staged public values only where existing app modules require them. |

### Literal Machine-Checked Production-Symbol Disposition

This table is bounded to current production symbols that move, split, change
shape, or are deleted, plus every new boundary/replacement symbol. A brace
group is an exact comma-separated identifier list, not a wildcard; the
generator expands it into one JSON row per current identifier. Singleton
targets apply to every current identifier, equal-cardinality groups align
positionally, and one current identifier may name multiple `finalTargets`.
Any other cardinality is rejected. `=` means the final identifier is exactly
the current identifier. `retained` means the symbol
remains app-owned after the named rewrite. No `*`, `all helpers`, `as needed`,
or unnamed fragment is legal in the generated artifact.

| Current path | Current exact symbol(s) | Final path | Final exact symbol(s) | Semantic owner | First checkpoint | Removal checkpoint | Disposition |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| `telegram/dto.rs` | `{ITEM_KIND_TELEGRAM_MESSAGE,TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER,TelegramMessageIdentity,TelegramMessageIdentity::validate,TelegramItemContext,TelegramMessageDraft}` | `telegram_impl/dto.rs` | `=` | staged | 3 | 3 | move |
| `telegram/media.rs` | `{CONTENT_KIND_TEXT_ONLY,CONTENT_KIND_TEXT_WITH_MEDIA,CONTENT_KIND_MEDIA_ONLY,TelegramMediaPayload,DocumentSignals,trimmed_non_empty,derive_content_kind,collect_document_signals,derive_document_media_kind,contact_summary,extract_document_media_payload,extract_media_payload,extract_item_payload}` | `telegram_impl/media.rs` | `=` | staged | 3 | 3 | move |
| `<new>` | `extract_raw_item_payload` | `telegram_impl/media.rs` | `extract_raw_item_payload` | staged-internal | 5 | retained | new-restricted-bridge |
| `<new>` | `derive_document_media_kind_from_parts` | `telegram_impl/media.rs` | `derive_document_media_kind_from_parts` | staged-internal | 7 | retained | new-restricted-bridge |
| `telegram/runtime.rs` | `{TelegramApiHash,TelegramApiHash::new,TelegramRuntimeStatus,TelegramClientInner,TelegramClientHandle,TelegramLoginAttemptToken,TelegramLoginAttempt,TelegramRuntimeAccount,detach_replaced,TelegramRuntime,TelegramRuntime::new,TelegramRuntime::initialize_account,TelegramRuntime::is_authenticated,TelegramRuntime::request_login_code,TelegramRuntime::sign_in,TelegramRuntime::authorized_client,TelegramRuntime::initialized_client,TelegramRuntime::clear_account,TelegramRuntime::handle_is_authorized,initialize_grammers_client}` | `telegram_impl/runtime.rs` | `=` | staged | 3 | 3 | move |
| `<new>` | `TelegramRuntime::client` | `telegram_impl/runtime.rs` | `TelegramRuntime::client` | staged | 3 | retained | new |
| `telegram/runtime.rs` | `{TelegramClientHandle::raw_client,TelegramClientHandle::raw_session}` | `telegram_impl/runtime.rs` | `=` | transitional | 3 | 7 | move-then-delete |
| `telegram/session.rs` | `{SESSION_KEY_BYTES,ENVELOPE_VERSION,ENVELOPE_ALGORITHM,SavedSession,EncryptedSessionEnvelope,SessionEncryptionKey,SessionEncryptionKey::try_from_encoded,SessionEncryptionKey::generate,encode_base64,decode_base64,associated_data,memory_session_to_saved,saved_to_telegram_session,encrypt_saved_session,decrypt_saved_session,TelegramSession,TelegramSession::empty,session_json_requires_existing_key,decode_session_json,encode_session_json}` | `telegram_impl/session.rs` | `=` | staged | 3 | 3 | move |
| `telegram/session.rs` | `TelegramSession::raw_memory_session` | `telegram_impl/session.rs` | `=` | transitional | 3 | 7 | move-then-delete |
| `<new>` | `TelegramSession::clone_memory_session` | `telegram_impl/session.rs` | `TelegramSession::clone_memory_session` | staged-internal | 3 | retained | new-restricted-bridge |
| `telegram.rs` | `{mod::dto,mod::media,mod::runtime,mod::session}` | `telegram_impl/lib.rs` | `{mod::dto,mod::media,mod::runtime,mod::session}` | staged | 3 | 3 | replace-module-root |
| `media.rs` | `TelegramMediaPayload` | `telegram_impl/lib.rs` | `TelegramMediaPayload` | staged | 3 | 3 | replace-compat-reexport |
| `media.rs` | `{derive_content_kind,derive_document_media_kind,extract_item_payload,DocumentSignals}` | `telegram_impl/lib.rs` | `=` | transitional | 3 | 7 | redirect-compat-reexport |
| `media.rs` | `{derive_content_kind,derive_document_media_kind,extract_item_payload,DocumentSignals}` | `media.rs` | `=` | transitional | 3 | 7 | retain-compat-reexport |
| `telegram.rs` | `{get_client,get_authorized_client}` | `telegram.rs` | `=` | transitional | existing | 7 | delete |
| `sources/peer_resolution.rs` | `ResolvedTelegramSource` | `telegram_impl/dto.rs` | `PeerDescriptor` | staged | 4 | 4 | replace |
| `sources/peer_resolution.rs` | `resolve_telegram_source_by_username` | `telegram_impl/live/peer.rs` | `resolve_username` | staged | 4 | 4 | split-stage |
| `sources/peer_resolution.rs` | `resolve_telegram_source_by_username` | `sources/peer_resolution.rs` | `resolve_telegram_source` | app | 4 | retained | split-app-dispatch |
| `sources/peer_resolution.rs` | `resolve_telegram_source_from_dialogs` | `telegram_impl/live/peer.rs` | `resolve_dialog_peer` | staged | 4 | 4 | split-stage |
| `sources/peer_resolution.rs` | `resolve_telegram_source_from_dialogs` | `sources/peer_resolution.rs` | `resolve_telegram_source` | app | 4 | retained | split-app-dispatch |
| `sources/peer_resolution.rs` | `{dialog_lookup_not_found_message,dialog_lookup_not_found_error,telegram_source_subtype_matches,validate_expected_telegram_source_subtype,resolved_telegram_source_from_peer,telegram_group_kind,telegram_group_is_member,peer_access_hash}` | `telegram_impl/live/peer.rs` | `{dialog_lookup_not_found_message,dialog_lookup_not_found_error,telegram_source_subtype_matches,validate_expected_telegram_source_subtype,peer_descriptor_from_peer,telegram_group_kind,telegram_group_is_member,peer_access_hash}` | staged | 4 | 4 | move-or-rename |
| `sources/peer_resolution.rs` | `telegram_source_info_from_peer` | `telegram_impl/live/peer.rs` | `peer_descriptor_from_peer` | staged | 4 | 4 | split-stage |
| `sources/peer_resolution.rs` | `telegram_source_info_from_peer` | `sources/store.rs` | `peer_descriptor_to_source_info` | app | 4 | retained | split-app-projection |
| `sources/peer_resolution.rs` | `{source_peer_ref_from_identity,peer_ref_for_source_subtype,peer_ref_for_typed_identity}` | `telegram_impl/live/peer.rs` | `peer_ref_from_descriptor` | staged | 4 | 4 | replace |
| `sources/identity.rs` | `TelegramSourceIdentity::peer_ref` | `telegram_impl/live/peer.rs` | `peer_ref_from_descriptor` | staged | 4 | 4 | replace |
| `sources/peer_resolution.rs` | `{source_peer_ref_from_identity,peer_ref_for_source_subtype,peer_ref_for_typed_identity}` | `sources/peer_resolution.rs` | `legacy_peer_ref_from_descriptor` | transitional | 4 | 4 | replace-with-transitional-copy |
| `sources/identity.rs` | `TelegramSourceIdentity::peer_ref` | `sources/peer_resolution.rs` | `legacy_peer_ref_from_descriptor` | transitional | 4 | 4 | replace-with-transitional-copy |
| `<new>` | `legacy_peer_ref_from_descriptor` | `sources/peer_resolution.rs` | `legacy_peer_ref_from_descriptor` | transitional | 4 | 7 | new-then-delete |
| `sources/peer_resolution.rs` | `{typed_peer_resolution_plan,resolve_source_peer_from_typed_identity,resolve_and_refresh_peer,refresh_source_avatar_cache}` | `sources/peer_resolution.rs` | `=` | app | 4 | retained | rewrite-owned-seam |
| `<new>` | `peer_descriptor_from_stored_identity` | `sources/peer_resolution.rs` | `peer_descriptor_from_stored_identity` | app | 4 | retained | new |
| `sources/peer_resolution.rs` | `ResolvedSyncPeer::peer` | `sources/peer_resolution.rs` | `ResolvedSyncPeer::peer` | transitional | existing | 7 | retain-then-delete |
| `<new>` | `ResolvedSyncPeer::descriptor` | `sources/peer_resolution.rs` | `ResolvedSyncPeer::descriptor` | app | 4 | retained | new |
| `sources/types.rs` | `SourceSyncTarget` | `sources/types.rs` | `SourceSyncTarget` | app | 4 | retained | add-is-member-field |
| `<new>` | `SourceSyncTarget::is_member` | `sources/types.rs` | `SourceSyncTarget::is_member` | app | 4 | retained | new |
| `sources/store.rs` | `load_source` | `sources/store.rs` | `load_source` | app | 4 | retained | rewrite-owned-query |
| `sources/store.rs` | `add_telegram_source` | `sources/store.rs` | `add_telegram_source` | app | 4 | retained | rewrite-owned-seam |
| `sources/avatar.rs` | `{TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS,peer_photo_bytes_with_timeout,peer_photo_bytes}` | `telegram_impl/live/avatar.rs` | `=` | staged | 4 | 4 | move |
| `sources/avatar.rs` | `peer_photo_data_url_with_timeout` | `sources/store.rs` | `peer_descriptor_to_source_info` | app | 4 | 4 | replace |
| `sources/store.rs` | `list_telegram_sources::dialog_and_avatar_segment` | `telegram_impl/live/mod.rs` | `dialog_listing` | staged-internal | 4 | 4 | split-stage-facade |
| `sources/store.rs` | `list_telegram_sources::dialog_and_avatar_segment` | `telegram_impl/live/peer.rs` | `DialogListing::next` | staged | 4 | 4 | split-stage-iterator |
| `<new>` | `DialogListing` | `telegram_impl/live/peer.rs` | `DialogListing` | staged | 4 | retained | new |
| `<new>` | `DialogListing::next` | `telegram_impl/live/peer.rs` | `DialogListing::next` | staged | 4 | retained | new |
| `<new>` | `DialogListing::new` | `telegram_impl/live/peer.rs` | `DialogListing::new` | staged-internal | 4 | retained | new-restricted-bridge |
| `<new>` | `peer_avatar_bytes` | `telegram_impl/live/peer.rs` | `peer_avatar_bytes` | staged-internal | 4 | retained | new-restricted-leaf |
| `<new>` | `{dialog_listing,resolve_dialog_peer,resolve_username,peer_avatar_bytes}` | `telegram_impl/live/mod.rs` | `=` | staged-internal | 4 | retained | new-restricted-facade |
| `<new>` | `{TelegramClientHandle::dialog_listing,TelegramClientHandle::resolve_dialog_peer,TelegramClientHandle::resolve_username,TelegramClientHandle::peer_avatar_bytes}` | `telegram_impl/runtime.rs` | `=` | staged | 4 | retained | new |
| `sources/store.rs` | `list_telegram_sources` | `sources/store.rs` | `list_telegram_sources` | app | 4 | retained | split-app-command |
| `sources/items.rs` | `{message_author,extract_telegram_context,reply_peer_context,build_raw_payload}` | `telegram_impl/live/messages.rs` | `{raw_message_author,extract_telegram_context,reply_peer_context,build_raw_payload}` | staged | 5 | 5 | move-or-rename |
| `sources/sync.rs` | `fallback_message_identity` | `telegram_impl/live/messages.rs` | `fallback_message_identity` | staged | 5 | 5 | move |
| `sources/sync.rs` | `persist_items::raw_history_segment` | `telegram_impl/live/messages.rs` | `fetch_message_batch` | staged-internal | 5 | 5 | split-stage |
| `sources/sync.rs` | `{persist_items,sync_telegram_source}` | `sources/sync.rs` | `=` | app | 5 | retained | rewrite-owned-seam |
| `<new>` | `run_telegram_batch_loop` | `sources/sync.rs` | `run_telegram_batch_loop` | app | 5 | retained | new-private-test-seam |
| `<new>` | `TelegramSession::cache_peer_infos` | `telegram_impl/session.rs` | `TelegramSession::cache_peer_infos` | staged-internal | 5 | retained | new-restricted-bridge |
| `<new>` | `{LiveMessageBatch,LiveMessage}` | `telegram_impl/live/messages.rs` | `=` | staged | 5 | retained | new |
| `<new>` | `{LiveMessageBatch::take_messages,LiveMessageBatch::is_terminal,LiveMessageBatch::next_offset_id,LiveMessageBatch::next_offset_date,LiveMessage::message_id,LiveMessage::published_at,LiveMessage::into_draft}` | `telegram_impl/live/messages.rs` | `=` | staged | 5 | retained | new |
| `<new>` | `{fetch_message_batch,fetch_forum_topics}` | `telegram_impl/live/mod.rs` | `=` | staged-internal | 5 | retained | new-restricted-facade |
| `<new>` | `{TelegramClientHandle::fetch_message_batch,TelegramClientHandle::fetch_forum_topics}` | `telegram_impl/runtime.rs` | `=` | staged | 5 | retained | new |
| `sources/topics.rs` | `ForumTopicSnapshot` | `telegram_impl/dto.rs` | `ForumTopicSnapshot` | staged | 5 | 5 | move |
| `sources/topics.rs` | `{fetch_all_forum_topics,forum_topic_page_cursor,forum_topic_message_date}` | `telegram_impl/live/topics.rs` | `{fetch_forum_topics,forum_topic_page_cursor,forum_topic_message_date}` | staged | 5 | 5 | move-or-rename |
| `sources/topics.rs` | `is_non_forum_topic_refresh_error` | `telegram_impl/error.rs` | `is_non_forum_topic_refresh_error` | staged | 5 | 5 | move |
| `sources/topics.rs` | `refresh_forum_topics` | `sources/topics.rs` | `refresh_forum_topics` | app | 5 | retained | rewrite-owned-seam |
| `takeout_import/mod.rs` | `export_dc_invoke_with_provenance::raw_segment` | `telegram_impl/takeout/export_dc.rs` | `export_dc_invoke` | staged-internal | 7 | 7 | split-stage-raw-invoke |
| `takeout_import/mod.rs` | `export_dc_invoke_with_provenance::durable_segment` | `takeout_import/mod.rs` | `run_takeout_transport_step_with_provenance` | app | 7 | retained | split-app-coordinator |
| `takeout_import/mod.rs` | `{record_export_dc_attempt_if_needed,record_export_dc_fallback_if_needed,record_only_my_messages_fallback_if_needed}` | `takeout_import/mod.rs` | `=` | app | 7 | retained | rewrite-owned-values |
| `takeout_import/mod.rs` | `record_channel_private_fallback_if_supported` | `takeout_import/mod.rs` | `record_only_my_messages_fallback_if_needed` | app | 7 | 7 | replace |
| `takeout_import/mod.rs` | `peer_ref_identity` | `telegram_impl/takeout/raw_parse.rs` | `peer_ref_identity` | staged | 7 | 7 | move |
| `takeout_import/mod.rs` | `{validate_takeout_peer,detect_supergroup_migration,revalidate_migrated_from_chat_id}` | `telegram_impl/takeout/operations.rs` | `{TakeoutTransport::validate_peer,TakeoutTransport::detect_supergroup_migration,TakeoutTransport::revalidate_migrated_peer}` | staged | 7 | 7 | replace |
| `takeout_import/mod.rs` | `takeout_get_history` | `telegram_impl/takeout/operations.rs` | `TakeoutTransport::history_count` | staged | 7 | 7 | split-stage-count |
| `takeout_import/mod.rs` | `takeout_get_history` | `telegram_impl/takeout/operations.rs` | `TakeoutTransport::history_page` | staged | 7 | 7 | split-stage-page |
| `takeout_import/mod.rs` | `takeout_search_my_messages` | `telegram_impl/takeout/operations.rs` | `TakeoutTransport::search_my_history_count` | staged | 7 | 7 | split-stage-search-count |
| `takeout_import/mod.rs` | `takeout_search_my_messages` | `telegram_impl/takeout/operations.rs` | `TakeoutTransport::search_my_history_page` | staged | 7 | 7 | split-stage-search-page |
| `takeout_import/mod.rs` | `{supports_only_my_messages_fallback,is_channel_private_error,messages_response_count}` | `{telegram_impl/takeout/operations.rs,telegram_impl/error.rs,telegram_impl/takeout/raw_parse.rs}` | `{supports_only_my_messages_fallback,is_channel_private_error,messages_response_count}` | staged | 7 | 7 | move |
| `takeout_import/mod.rs` | `{takeout_history_count_probe,takeout_history_page_response}` | `takeout_import/mod.rs` | `=` | app | 7 | retained | rewrite-owned-seam |
| `takeout_import/mod.rs` | `TakeoutHistoryProbe` | `telegram_impl/takeout/types.rs` | `TakeoutCount` | staged | 7 | 7 | replace |
| `takeout_import/mod.rs` | `CountedMessageRange` | `takeout_import/mod.rs` | `CountedMessageRange` | app | 7 | retained | replace-fields-with-owned-values |
| `takeout_import/mod.rs` | `{run_export_dc_spike_for_handle,run_takeout_source_import,run_takeout_migrated_history_import,run_started_takeout_source_import,run_started_takeout_source_import_inner,import_takeout_history_ranges,import_takeout_history_pages}` | `takeout_import/mod.rs` | `=` | app | 7 | retained | rewrite-owned-seam |
| `takeout_import/export_dc.rs` | `{EXPORT_DC_SHIFT,ExportDcAlias,prepare_export_dc_alias,export_dc_id_for_home_dc,export_dc_invoke,export_dc_invoke_with,finish_takeout_session}` | `telegram_impl/takeout/export_dc.rs` | `=` | staged | 7 | 7 | move |
| `takeout_import/export_dc.rs` | `{TAKEOUT_FILE_MAX_SIZE,takeout_init_request_for_source_subtype}` | `telegram_impl/takeout/mod.rs` | `=` | staged | 7 | 7 | move |
| `takeout_import/export_dc.rs` | `should_fallback_export_dc_error` | `telegram_impl/error.rs` | `should_fallback_export_dc_error` | staged | 7 | 7 | move |
| `takeout_import/export_dc.rs` | `ExportDcAttemptState` | `telegram_impl/takeout/transport.rs` | `ExportDcAttemptTracker` | staged | 7 | 7 | split-stage-tracker |
| `takeout_import/export_dc.rs` | `ExportDcAttemptState` | `telegram_impl/takeout/types.rs` | `TakeoutAttempt` | staged | 7 | 7 | split-stage-attempt-value |
| `takeout_import/pagination.rs` | `{TAKEOUT_HISTORY_PAGE_LIMIT,TakeoutPaginationProfile,TakeoutPageRequest,TakeoutPaginationCursor,TakeoutPaginationCursor::new,TakeoutCursorAdvance,TakeoutPaginationFallbackReason,select_history_splits,fallback_message_range,takeout_page_request,next_takeout_cursor,should_restart_with_descending_fallback,takeout_pagination_fallback_warning}` | `telegram_impl/takeout/pagination.rs` | `=` | staged | 7 | 7 | move |
| `takeout_import/pagination.rs` | `{TakeoutPageRequest::offset_id,TakeoutPageRequest::add_offset,TakeoutPageRequest::limit,TakeoutCursorAdvance::cursor,TakeoutCursorAdvance::advanced,TakeoutCursorAdvance::reached_range_start}` | `telegram_impl/takeout/pagination.rs` | `=` | staged-internal | 7 | 7 | move-restricted-fields |
| `takeout_import/pagination.rs` | `{ParsedTakeoutPage,parse_takeout_page}` | `{telegram_impl/takeout/types.rs,telegram_impl/takeout/pagination.rs}` | `{TakeoutPage,parse_takeout_page}` | staged | 7 | 7 | absorb-and-rewrite |
| `takeout_import/pagination.rs` | `{message_range_min_id,message_range_max_id}` | `telegram_impl/takeout/types.rs` | `{MessageRange::min_id,MessageRange::max_id}` | staged | 7 | 7 | replace |
| `takeout_import/raw_parse.rs` | `parse_raw_message` | `telegram_impl/takeout/raw_parse.rs` | `parse_raw_message` | staged-internal | 7 | 7 | move |
| `takeout_import/raw_parse.rs` | `{extract_raw_media_payload,extract_photo_media_payload,extract_document_media_payload,extract_raw_telegram_context,raw_message_identity,reaction_count,raw_message_author,peer_context,peer_id_string,raw_summary_media,apply_larger_photo_size,contact_summary,trimmed_non_empty}` | `telegram_impl/takeout/raw_parse.rs` | `=` | staged | 7 | 7 | move |
| `takeout_import/forum_topics.rs` | `refresh_forum_topics_after_completed_takeout::remote_segment` | `telegram_impl/takeout/forum_topics.rs` | `takeout_forum_topics` | staged-internal | 7 | 7 | split-stage |
| `takeout_import/forum_topics.rs` | `refresh_forum_topics_after_completed_takeout` | `takeout_import/forum_topics.rs` | `refresh_forum_topics_after_completed_takeout` | app | 7 | retained | split-app-policy |
| `<new>` | `{TakeoutAttempt,TakeoutFallbackKind,TakeoutFallback,TakeoutPeer,MessageRange,TakeoutCount,TakeoutPage,TakeoutMessage}` | `telegram_impl/takeout/types.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutAttempt::home_dc_id,TakeoutAttempt::export_dc_id,TakeoutFallback::kind,TakeoutFallback::warning,TakeoutFallback::provenance_message,TakeoutPeer::from_descriptor,TakeoutPeer::peer_kind,TakeoutPeer::peer_id,MessageRange::min_id,MessageRange::max_id,TakeoutCount::count,TakeoutCount::only_my_messages,TakeoutPage::take_messages,TakeoutPage::has_next,TakeoutPage::only_my_messages,TakeoutPage::take_pagination_fallback_warning,TakeoutMessage::message_id,TakeoutMessage::into_draft}` | `telegram_impl/takeout/types.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutAttempt::new,TakeoutFallback::new,TakeoutPeer::new,TakeoutPeer::source_subtype,TakeoutPeer::access_hash,MessageRange::new,TakeoutCount::new,TakeoutPage::from_parts,TakeoutPage::pagination_state,TakeoutMessage::from_raw}` | `telegram_impl/takeout/types.rs` | `=` | staged-internal | 7 | retained | new-restricted-bridge |
| `<new>` | `TakeoutTransport` | `telegram_impl/takeout/transport.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutTransport::export_dc_attempt,TakeoutTransport::drain_fallbacks}` | `telegram_impl/takeout/transport.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutTransport::new,TakeoutTransport::queue_fallback,TakeoutTransport::client,TakeoutTransport::session,TakeoutTransport::home_dc_id,TakeoutTransport::export_dc_id}` | `telegram_impl/takeout/transport.rs` | `=` | staged-internal | 7 | retained | new-restricted-bridge |
| `<new>` | `takeout_self_check` | `telegram_impl/takeout/operations.rs` | `takeout_self_check` | staged-internal | 7 | retained | new-restricted-leaf |
| `<new>` | `{takeout_self_check,prepare_takeout,takeout_forum_topics}` | `telegram_impl/takeout/mod.rs` | `=` | staged-internal | 7 | retained | new-restricted-facade |
| `<new>` | `{TelegramClientHandle::takeout_self_check,TelegramClientHandle::prepare_takeout}` | `telegram_impl/runtime.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `TelegramClientHandle::takeout_forum_topics` | `telegram_impl/runtime.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutTransport::init,TakeoutTransport::message_ranges,TakeoutTransport::validate_peer,TakeoutTransport::detect_supergroup_migration,TakeoutTransport::revalidate_migrated_peer,TakeoutTransport::history_count,TakeoutTransport::search_my_history_count,TakeoutTransport::history_page,TakeoutTransport::search_my_history_page,TakeoutTransport::finish}` | `telegram_impl/takeout/operations.rs` | `=` | staged | 7 | retained | new |

The `raw_memory_session` and `clone_memory_session` rows are an intentional
coexist-then-replace pair: CP3 through CP6 require both exact methods, while
CP7 and later require the restricted owned-`Arc` bridge exactly once and zero
occurrences of the borrowed bridge name in production or tests.

The schema-v1 generator also freezes these logical-owner transition
inventories. They remain the readable checklist for the forward-only LLM
review below; TypeScript does not reconcile them against Rust use-sites:

```text
CP3 raw handle callsites:
  sources::store::{list_telegram_sources,add_telegram_source}
  sources::sync::sync_telegram_source
  takeout_import::{run_export_dc_spike_for_handle,run_takeout_migrated_history_import,run_takeout_source_import}
CP4 raw handle callsites:
  sources::sync::sync_telegram_source
  takeout_import::{run_export_dc_spike_for_handle,run_takeout_migrated_history_import,run_takeout_source_import}
CP4 ResolvedSyncPeer::peer consumers:
  sources::sync::sync_telegram_source
  takeout_import::{run_takeout_migrated_history_import,run_takeout_source_import}
CP5 raw handle callsites:
  takeout_import::{run_export_dc_spike_for_handle,run_takeout_migrated_history_import,run_takeout_source_import}
CP5 ResolvedSyncPeer::peer consumers:
  takeout_import::{run_takeout_migrated_history_import,run_takeout_source_import}
CP7 raw bridge symbols/callsites: empty
```

Separately from those schema-v1 inventories, the design itself adds this
LLM-only CP3→CP4 review input; it is intentionally not generated or written
into the retained artifact:

```text
peer-kind staged-root bridge:
  telegram_impl::{TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER}
  sole consumer: sources::sync::fallback_message_identity
```

### Forward-Only Transitional Escape-Hatch Review Authority

This subsection applies only to unexecuted work after retained Checkpoint 1.
It does not reinterpret the Checkpoint 1 commit, change its schema-v1 artifact,
or rewrite its historical RED/GREEN instructions.

The logical-owner arrays above are a human-review checklist, not executable
Rust source authority. The committed schema-v1 `transitionInventories` field
remains so the plan, generated artifact, and reviewer share the same readable
checkpoint vocabulary. It is not upgraded to schema v2, and no TypeScript test
may derive, accept, or reject a Rust escape-hatch use-site from those arrays.

A fresh independent LLM reviewer that did not implement the checkpoint and has
no prior implementation-conversation context performs every CP2–CP8 gate. Its
bounded packet includes the checkpoint and retained predecessor, the complete
scoped Rust diff including production and test Rust, enough unchanged
surrounding source, this plan/design authority, and committed
`src/lib/telegram-8b-symbol-map.json`. The reviewer reads the artifact's
complete schema-v1 `transitionInventories` field and symbol-disposition rows as
input checklists together with all complete transitional exception sets:

```text
media compatibility facade:
  DocumentSignals
  derive_content_kind
  derive_document_media_kind
  extract_item_payload

raw bridge:
  TelegramClientHandle::{raw_client,raw_session}
  TelegramSession::raw_memory_session
  telegram::{get_client,get_authorized_client}
  ResolvedSyncPeer::peer
  legacy_peer_ref_from_descriptor

peer-kind staged-root bridge (CP3→CP4 only):
  telegram_impl::{TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER}
  sole consumer: sources::sync::fallback_message_identity
```

The schema-v1 artifact is intentionally not expanded merely to make every
checkpoint look uniform. CP2 uses the explicit no-Rust-change rule; CP3 uses
`cp3RawHandleCallsites`; CP4 uses both CP4 arrays; CP5 uses both CP5 arrays;
CP6 carries the CP5 arrays forward unless the reviewed diff removes a listed
use; and CP7/CP8 use/carry forward the empty
`cp7RawBridgeSymbolsAndCallsites` array. The artifact is not the sole authority:
the media facade, symbol rows, complete exception sets, and plan/design rules
remain required review inputs.

At CP2, the reviewer verifies from the complete scoped diff that no Rust
escape-hatch definition, physical use, logical ownership, or forwarding chain
changed. At CP3 through CP6, the reviewer accounts for the active inventories
and the complete exception sets above. At CP5 the reviewer additionally
confirms that the three peer-kind root exports and their sole app consumer
disappear when the mapper moves; no replacement bridge is allowed.

At CP7 and CP8 the reviewer must find no retained transitional definition,
callsite, forwarding use, alias, re-export, or replacement spelling. An
unlisted use or uncertain attribution blocks the checkpoint. The implementation
may remove it and obtain a clean LLM re-review; retaining it requires an
approved plan/design amendment. It is never accepted by teaching the
TypeScript contract more Rust syntax.

Review follows the whole value path rather than only the function containing
the terminal access. In particular, the retained Takeout path is reviewed as
one two-hop chain:

```text
run_takeout_source_import
  -> run_started_takeout_source_import
  -> run_started_takeout_source_import_inner
```

The intermediate forwarding function is part of the review even though it is
not a logical inventory owner. Every physical access in the inner function and
the direct access in the outer function are attributed to
`run_takeout_source_import`. Equivalent producer, alias, function-item,
shadowing, wrapper, or forwarding changes are reviewed semantically by the
LLM, not inferred by automation.

For each remaining checkpoint commit, the independent LLM review inspects the
complete scoped Rust diff plus enough unchanged surrounding source to follow
every affected producer and continuation. Its review evidence records:

- the selected checkpoint and retained predecessor;
- every listed exception definition and physical use that remains;
- the logical owner and full forwarding chain for indirect uses;
- every use expected to disappear at that checkpoint;
- any new alias, re-export, wrapper, function item, or unrelated same-named
  field that could obscure the inventory;
- a blocking finding for every unlisted or ambiguous escape hatch.

The evidence is review output, not a generated artifact and not input to a
source parser. It must be recorded in the active task thread before the
candidate status/final gates. After a final `CLEAN` verdict, CP2 creates and
CP3–CP8 append the complete record to cumulative committed document
`docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`
before candidate status. A `BLOCKED` review is not retained as checkpoint
evidence. Every review after `BLOCKED` receives the complete current scoped
Rust diff and required unchanged context, even when resolution was added
context, clarification, or an approved authority amendment rather than a Rust
edit. Any Rust or escape-hatch-authority change after `CLEAN` also invalidates
that verdict and requires a fresh independent full review, never a
fix-delta-only review. The CP8 section contains only the complete CP8 record and
one final CLEAN marker. A separate aggregate table outside all seven exact
review sections references their SHAs/sections and records plain `CLEAN`
outcomes without repeating verdict-marker lines. No TypeScript test parses,
validates, or gates on this prose, and no test passes merely because the prose
exists.

For transitional escape-hatch authorization,
`src/lib/telegram-crate-boundary-contract.test.ts` must not tokenize or mask
Rust, isolate Rust functions, resolve types/imports/bindings/scopes/shadowing,
infer producer results or call graphs, trace forwarding/data flow, count
transitional member occurrences, or enforce generated use-site anchors, edges,
or occurrence fences for any transitional exception set. Their schema-v1
symbol rows and inventories remain document/artifact review metadata, but
TypeScript does not reconcile their definitions or use-sites against Rust
source. It may continue to verify document/artifact serialization and
lifecycle vocabulary. There is no global reservation of an unrelated
application field named `peer`.
Literal source search may be used only as reviewer navigation. No search
pattern, occurrence count, or exit status authorizes or blocks an escape-hatch
use or serves as CP7/CP8 terminal evidence.

Automatic evidence for these escape-hatch uses is limited to Rust compilation
and the named Rust focused/package/workspace behavior tests. Compilation
proves the reviewed source is valid and type-correct; Rust behavior tests prove
the frozen observable contracts. A TypeScript source contract is not evidence
that a use is allowed. Neither substitutes for the required LLM review, and
the LLM review does not weaken any compile or behavior gate.

Synthetic `::segment` keys are source fragments, not Rust identifiers. Their
JSON rows carry these exact pre-move anchors, each required in its enclosing
function at CP1 and forbidden there after the removal checkpoint:

```text
list_telegram_sources::dialog_and_avatar_segment:
  client.iter_dialogs()
  peer_photo_data_url_with_timeout(&client, dialog.peer())
persist_items::raw_history_segment:
  client.iter_messages(peer)
  messages.next()
export_dc_invoke_with_provenance::raw_segment:
  export_dc_invoke(client, alias, request, warnings, fallback_used).await
export_dc_invoke_with_provenance::durable_segment:
  record_export_dc_attempt_if_needed
  record_export_dc_fallback_if_needed
refresh_forum_topics_after_completed_takeout::remote_segment:
  refresh_forum_topics(pool, client, peer, source).await
```

The app-retained production symbols not named above are out of the move scope
and may receive import-only edits. A body change to any such symbol must be
listed as `rewrite-owned-seam` before implementation.

## Generated Grammers Feature Baseline

Checkpoint 1 creates
`src/lib/telegram-grammers-feature-baseline.json` and
`scripts/telegram-grammers-feature-baseline.mjs`. The script accepts exactly
`--write` or `--check`, invokes locked Cargo metadata, rejects a
malformed/missing package-graph result, and writes stable UTF-8 JSON with this
schema:

```json
{
  "schemaVersion": 1,
  "revision": "1f901ce6e973fdcf0e74267f3d8efad5c729daaa",
  "packages": [
    {
      "name": "grammers-client",
      "required": [],
      "forbidden": ["default"],
      "universe": ["default"]
    }
  ]
}
```

The real arrays are sorted and contain every feature key exposed by each
pinned package. The generator uses the resolved `extractum` node only to find
the four exact direct-dependency package IDs. It reads required features from
each Grammers dependency's own `resolve.nodes[].features` entry and reads that
package's feature universe from the matching `packages[].features` keys.
Forbidden is exactly `universe - required`. The frozen observations are:

| Package | Required | Forbidden |
| --- | --- | --- |
| `grammers-client` | none | `default`, `fs`, `html`, `html5ever`, `markdown`, `parse_invite_link`, `proxy`, `pulldown-cmark`, `url` |
| `grammers-session` | `serde` | `default`, `sqlite-storage` |
| `grammers-mtsender` | none | `hickory-resolver`, `proxy`, `tokio-socks`, `url` |
| `grammers-tl-types` | `default`, `deserializable-functions`, `impl-debug`, `impl-from-enum`, `impl-from-type`, `tl-api`, `tl-mtproto` | `impl-serde` |

`--check` fails on revision, package, order, required, forbidden, universe, or
format drift. The Telegram boundary contract executes `--check`, so
`npm.cmd run verify` remains the standing gate.


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

The crate returns the exact owned `LiveMessageBatch`/`LiveMessage` seam rather
than a public Grammers iterator or stream. Each batch is exactly one raw
`messages.getHistory` invoke with `1 <= limit <= 100`; the app processes each
owned message incrementally before requesting another batch and never receives
a borrowed Grammers value.

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

There is no public generic `invoke<R: RemoteCall>` equivalent and no generic
invocation/provenance callback. The only Takeout transport capability is the
exact `TakeoutTransport` API frozen below, including stateful attempt/fallback
draining after success, error, or cancellation.

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

`extractum_core::error::{AppError, AppResult}` remains the Rust boundary,
command, and IPC error contract. This is a dependency on lower-layer core, not
on the application. Public fallible crate operations return `AppResult<T>`
directly; the app facade performs no error conversion.

No public `TelegramError` or parallel terminal-error taxonomy is introduced.
Grammers `InvocationError` matching and any typed flood-wait, retry, transport,
or fallback decisions remain crate-private. They may control retries or
fallbacks, but a terminal failure is constructed directly as the existing
`AppError` kind and exact message at the owning operation boundary.

This follows the `extractum-llm`, `extractum-analysis`, and
`extractum-prompt-packs` precedent because the core taxonomy already expresses
every public Telegram terminal failure. `extractum-gemini-browser` is
different: its public domain error carries recoverable timeout, cancellation,
protocol, and browser-lifecycle distinctions that do cross its crate boundary.

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

The physical final crate path is
`src-tauri/crates/extractum-telegram`. The workspace-member spelling
`crates/extractum-telegram` is relative to `src-tauri/Cargo.toml`; no
repository-root `crates/` directory is created.

8A does not create a crate or change dependency ownership.

8B contains a named, separately green **workspace dependency normalization**
checkpoint in the existing `src-tauri/Cargo.toml`:

1. promote `base64`, `chacha20poly1305`, `rand_core`, and all four pinned
   Grammers roots from app-inline declarations into
   `[workspace.dependencies]`;
2. make the application inherit the same roots without removing any direct
   app edge or changing any effective version, Git revision, default feature,
   or explicit feature;
3. run `cargo metadata --locked` and regenerate the lockfile only if Cargo's
   resolved package/dependency data changes.

The normalized declarations preserve this exact baseline:

```toml
base64 = "0.22"
chacha20poly1305 = { version = "0.10", features = ["std"] }
grammers-client = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", default-features = false }
grammers-session = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", default-features = false, features = ["serde"] }
grammers-mtsender = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa" }
grammers-tl-types = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", features = ["deserializable-functions"] }
rand_core = { version = "0.6", features = ["getrandom"] }
```

Promotion alone is expected to leave the resolved graph unchanged, so the
checkpoint must not fabricate a `Cargo.lock` hunk. It records either the
actual generated hunk or byte identity of the lockfile.

8C adds `crates/extractum-telegram` as a workspace member, makes its package
manifest inherit the prepared roots, adds one application path dependency on
the crate, and removes the app's direct Grammers plus session-only crypto
edges. That package-graph change must have its own generated
`src-tauri/Cargo.lock` hunk.

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

This list is the audited expected baseline, not permission to add capabilities
speculatively. The implementation plan must verify actual imports against the
frozen 8B file map and may remove an unused root. It may also add a root that
is a direct, demonstrated dependency of an already-approved moved file, or
refine features of a non-Grammers dependency, without amending this design.
The manifest contract must name that evidence. Any change to the direct
default-feature or explicit-feature declaration policy of the four pinned
Grammers roots requires a design amendment.

A new root or feature that introduces a new capability or owner — including
SQL, Tauri/IPC, keyring, independent HTTP/network transport, filesystem,
process, or OS integration not already frozen in the move map — requires a
design amendment. Neighboring crates using a dependency is not evidence that
this crate needs it.

The expected production Tokio features include `rt`, `sync`, and `time`;
`time` is required by the current avatar timeout. Test-only `macros`,
`test-util`, or other features belong in the exact dev-dependency contract.
`tokio-util` is not implied merely because adjacent crates use it: Takeout job
cancellation stays app-owned. If the frozen final file map proves another
portable Tokio/Tokio-util use, the implementation plan may declare the minimal
root/features and cite the exact source use.

The four Grammers roots keep the exact current Codeberg revision. Their
direct-declaration policy is frozen separately from Cargo's resolved feature
closure:

- `grammers-client`: `default-features = false`, no explicit features;
- `grammers-session`: `default-features = false`, explicit `serde`;
- `grammers-mtsender`: default features enabled by the omitted
  `default-features` key, no explicit features;
- `grammers-tl-types`: default features enabled by the omitted
  `default-features` key, explicit `deserializable-functions`;
- the Takeout owner carries the `deserializable-functions` requirement.

This direct declaration policy is normative. The package-defined and resolved
feature sets, including the effective `grammers-mtsender` result, are derived
only from locked Cargo metadata and the generated baseline below; they are not
hard-coded as prose assertions.

Resolved feature closure is not maintained as a literal array in this design.
At the clean identity commit of the 8B implementation plan, a canonical
metadata step must generate and commit an order-independent feature baseline
for all four pinned packages. For each package, the baseline records the
required set from `resolve.nodes[].features` and the forbidden set consisting
of every key in `packages[].features` that is not required. Arrays and package
records are sorted before serialization. 8B and 8C must require every
generated required feature, reject every generated forbidden feature, and
prove that the package feature-key universe still matches the generated
baseline. This catches feature unification drift without copying a
hand-maintained resolved array into the plan or standing contract.

Canonical versions and Git pins belong in `[workspace.dependencies]`; package
manifests inherit them. `base64` remains an app dependency where independently
used, while session-only crypto roots leave the app when their last app use is
moved.

Every manifest checkpoint must pass `cargo metadata --locked`. 8B proves
normalization preserved the app's direct dependency graph; 8C proves the new
crate owns the direct Grammers edges and the app does not.

### Direct-dependency proof

The primary final absence proof parses JSON from:

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
```

The standing contract must:

1. resolve `metadata.workspace_members` back to packages and find exactly one
   `extractum` at the expected app manifest plus one `extractum-telegram` at
   the canonicalized physical
   `src-tauri/crates/extractum-telegram/Cargo.toml`;
2. require exactly one `extractum-telegram` declaration total from `extractum`,
   with normal kind, no target, rename, or source, and the canonicalized
   expected path; require exactly one corresponding immediate resolved edge
   whose sole dependency-kind entry is normal and target-free;
3. require zero direct `grammers-client`, `grammers-session`,
   `grammers-mtsender`, or `grammers-tl-types` declarations from `extractum`
   across normal, development, build, and target-qualified dependency kinds;
4. map the app resolve node's immediate dependency package IDs back through
   `packages[].id` and require zero immediate resolved Grammers packages,
   rather than comparing Cargo-normalized alias strings;
5. require all four direct Grammers roots on `extractum-telegram`, with the
   exact Codeberg source/revision and the exact direct default-feature and
   explicit-feature declaration policy above;
6. load the generated, order-independent feature baseline captured at the
   clean 8B identity commit; require every feature in each required set,
   require zero resolved features from each forbidden set, and require the
   current package feature-key universe to equal the recorded universe;
7. parse the declared Git `rev` query and resolved source commit fragment and
   require each to equal the full
   `1f901ce6e973fdcf0e74267f3d8efad5c729daaa`, not merely contain a
   substring;
8. remove the app's immediate `extractum-telegram` edge from a copy of the
   resolve graph and prove that no Grammers package remains reachable from the
   app node.

Grammers packages are intentionally allowed elsewhere in `packages` and as
transitive app dependencies through `extractum-telegram`; the graph-cut proves
that every such app path passes through the owning crate. A global package-name
ban would reject the desired graph and is forbidden.

The secondary source scan covers every app-owned Rust source and test. It
checks imports and fully qualified paths, raw `tl::` uses, aliases, re-exports,
and raw Grammers/TL type names. A passing grep without the metadata proof is
not dependency evidence.

## 8B Staging Layout

The 2,828-line `takeout_import/mod.rs` is not moved or split ad hoc during 8C.
8B prepares the future crate layout inside the application at the exact
physical path:

```text
src-tauri/src/telegram_impl/
  lib.rs
  dto.rs
  error.rs
  runtime.rs
  session.rs
  media.rs
  live/
    mod.rs
    avatar.rs
    peer.rs
    messages.rs
    topics.rs
  takeout/
    mod.rs
    types.rs
    transport.rs
    export_dc.rs
    operations.rs
    pagination.rs
    raw_parse.rs
    forum_topics.rs
```

The root staging files have unambiguous owners:

- `dto.rs` owns `TelegramMessageDraft`, `TelegramMessageIdentity`,
  `TelegramItemContext`, `PeerDescriptor`, `ForumTopicSnapshot`, and the one
  public `ITEM_KIND_TELEGRAM_MESSAGE` constant;
- `media.rs` owns `TelegramMediaPayload`, `DocumentSignals`, content/media
  classification constants and helpers, and the Grammers media adapter;
- `error.rs` contains only crate-private protocol/retry/fallback
  classification and terminal `AppError` construction; it exports no parallel
  error type. The filename is retained for that private failure-translation
  role and does not imply a public Telegram error taxonomy;
- `live/peer.rs` owns the concrete stateful opaque `DialogListing`;
  `live/messages.rs` owns private-field `LiveMessageBatch` and `LiveMessage`;
- `takeout/types.rs` owns `TakeoutAttempt`, `TakeoutFallbackKind`,
  `TakeoutFallback`, `TakeoutPeer`, `MessageRange`, `TakeoutCount`,
  `TakeoutPage`, and `TakeoutMessage`; `takeout/transport.rs` owns the concrete
  stateful opaque `TakeoutTransport`. These public values are curated through
  `lib.rs`.

The application connects this tree only through private
`#[path = "telegram_impl/lib.rs"] mod telegram_impl;`. By the end of 8B every
file in this staging tree is package-portable: it imports no application
module, SQLx, Tauri, keyring type, or app-local `crate::error`. Fallible staged
code imports `extractum_core::error::{AppError, AppResult}` directly.

The staging tree has one exact internal-path convention. Every reference from
one staged module to another uses only `self::`/`super::` relative module
paths. No staged source contains `crate::`, an application-root alias, or an
application-root re-export. All public visibility needed by the final crate is
already present and reviewed by the end of 8B. Conversely, every app-owned
source outside the staging tree refers to staged public API only through the
exact `crate::telegram_impl::` prefix; bare or aliased staging paths are
forbidden. That consumer prefix remains unchanged in 8C behind a private
compatibility facade. Standing scanners enforce both sides of this convention.

The Takeout staging boundary is concrete and permits no catch-all ownership:

- `transport.rs` owns only the concrete client/session/DC state, attempt
  snapshots, fallback queue, and exact restricted accessors frozen below;
- `export_dc.rs` owns export-DC selection and invocation;
- `operations.rs` owns Telegram self-check, init/finish, peer validation,
  migration detect/revalidate, count/history/search/page operations;
- `raw_parse.rs` owns peer/TL identity conversion and raw response
  classification/parsing;
- `pagination.rs` owns range, page, and cursor rules;
- `forum_topics.rs` owns the remote forum-topic operation;
- `types.rs` owns only the exact Takeout inputs, outputs, attempts, fallback
  metadata, and sibling-restricted pagination state frozen below.

The app-owned `takeout_import/mod.rs` keeps commands, jobs, cancellation and
selection loops, persistence, provenance recording, warnings, progress/event
emission, and terminal batch finalization. The app applies the exact
one-outer-select attempt/fallback protocol frozen below; only the app records
that metadata as provenance.

8C then moves `src-tauri/src/telegram_impl/**` to
`src-tauri/crates/extractum-telegram/src/**`, adds the path dependency, and
leaves app coordinators and all consumer source files in place. The move
preserves every staged source file byte-for-byte at the same relative path.
Inside the moved tree, no module path, import, visibility, or source text
changes in 8C.

After moving the staged `lib.rs`, 8C creates a new private compatibility facade
at the vacated `src-tauri/src/telegram_impl/lib.rs` path. It contains only an
explicit curated `pub(crate) use extractum_telegram::{...};` allowlist; no glob
is authorized. The existing
`#[path = "telegram_impl/lib.rs"] mod telegram_impl;` declaration and every
consumer `crate::telegram_impl::` path remain byte-identical. Outside the moved
tree, the only Rust source addition is this facade; the remaining allowed
changes are generated manifest and lockfile edits.

The last green 8B commit records a content-hash manifest keyed by paths
relative to `src-tauri/src/telegram_impl`. 8C records the same manifest keyed
relative to `src-tauri/crates/extractum-telegram/src` and requires exact path
and byte-hash equality for the moved staged files before any completion claim;
the newly created app facade is recorded separately and is not substituted
for the moved `lib.rs` in that comparison. A mismatch stops the slice as
non-mechanical and requires an amended preparation step rather than an
in-place extraction fix.

## Current-File Disposition

The implementation plans must freeze a symbol-level move map. The complete
known disposition is:

| Current path | Physical LOC | Crate-owned portion | App-owned portion |
| --- | ---: | --- | --- |
| `takeout_import/mod.rs` | 2,828 | raw transport, migration probes, validation, remote operations, response classification | commands, jobs, cancellation, provenance, persistence and finalization |
| `sources/items.rs` | 2,136 | `TelegramMessageDraft`, `TelegramItemContext`, message/author/reply/raw/media conversion | SQL and cross-domain item transaction; dead generic insert removed |
| `sources/peer_resolution.rs` | 1,198 | remote resolve and peer mapping | planning, DB identity, cache and orchestration |
| `ingest_provenance.rs` | 910 | none; consumes public identity/item-kind values | provenance SQL, provider-key formatting and batch policy |
| `sources/topics.rs` | 817 | remote topic retrieval and mapping | SQL upsert/read models and refresh coordination |
| `telegram.rs` | 712 | client/runtime/login/session ownership | commands, SQL/secret resolution, status/event mapping |
| `takeout_import/raw_parse.rs` | 645 | raw TL-to-owned draft conversion | no Grammers-bearing app portion |
| `takeout_import/pagination.rs` | 569 | raw page/range parsing and cursor rules | no Grammers-bearing app portion |
| `sources/sync.rs` | 550 | live message retrieval and pure identity mapping | locks, settings, persistence and finalization |
| `telegram_session_store.rs` | 463 | saved-session model, codec, encryption, Grammers conversion | app-data path, keyring adapter, file lifecycle |
| `sources/types.rs` | 432 | `TelegramMessageIdentity`, validation, peer-kind constants and item-kind constant | all other source/read-model values and policies |
| `lib.rs` | 427 | none; registers the staged module and retained private compatibility facade | app root and command registration remain application-owned |
| `sources/identity.rs` | 380 | Grammers peer conversion | DB rows, normalization and source identity policy |
| `takeout_import/export_dc.rs` | 360 | export-DC and concrete raw invocation | app error/event adaptation if any remains |
| `takeout_import/migrated_history.rs` | 315 | none; consumes public identity | capability SQL, migration policy and identity construction |
| `media.rs` | 275 | Telegram media payload/classification and Grammers adapter | private compatibility facade; provider-neutral core values remain in core |
| `takeout_import/forum_topics.rs` | 238 | remote Telegram topic operation | batch warnings and refresh coordination |
| `sources/avatar.rs` | 110 | Telegram photo transport | cache paths, writes, cleanup and app presentation |
| `sources/mod.rs` | 57 | none; removes or redirects re-exports of moved values | app module shell; no duplicate DTO/constant owner or dead insert re-export |

The original 14-file direct perimeter is 11,281 physical lines; its four
largest files are 61.9% and its eight largest are 83.8%. The complete known
19-file production move-and-touch perimeter is 13,422 physical lines. These
verified figures are descriptive only and do not replace the symbol-level
move map.

The immutable ownership/move surface remains 19 paths / 140 tests.
`src-tauri/src/sources/store.rs` is the sole known dependent-only raw-client
consumer outside that map. Its 24 app tests stay outside the ownership map and
run as broad store regressions in 8A Checkpoint 5; none directly invokes
`list_telegram_sources` or `add_telegram_source`, so the Checkpoint 5 runtime
lookup test plus lifecycle-gated source contract—not this suite—prove the
caller-lock rewrite. The complete Phase 8 implementation touch surface is
therefore the 19 ownership/move paths plus this one dependent-only path: 19
ownership/move paths plus one dependent-only consumer with 24 regressions.

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

The committed map is a literal immutable baseline, not a count recomputed from
the changing tree. Every entry has the form:

```text
{
  baseline_package,
  baseline_full_id,
  staged_path,
  final_owner,
  final_full_id,
  companion_final_ids
}
```

`companion_final_ids` is normally empty. It is populated only when an existing
mixed-subject baseline identity must retain one primary owner while a new test
captures the other side of the split.

8B proves the app-only executable identities against that same map. 8C proves
the union of app and crate identities against it. A renamed or deleted path
cannot silently remove an identity from the baseline.

### Application fixture audit

The audit of the original 119 direct-perimeter tests quantifies the deferred
`sources::test_support` problem before producer code moves:

- exactly 43 SQL/integration tests use one or more of the seven app-only
  helpers `memory_pool_with_sources`,
  `memory_pool_with_source_items_and_topics`,
  `create_analysis_documents_table`,
  `create_ingest_provenance_tables`,
  `create_migrated_history_capability_tables`,
  `create_item_identity_indexes`, and
  `create_archive_read_model_tables`;
- those 43 identities remain in `extractum`, because their subject is
  app-owned tables, transactions, provenance, topic membership, watermarks, or
  cross-domain derived writes;
- three credential SQL identities also remain in `extractum` because account
  SQL remains app-owned:
  `legacy_api_hash_migrates_to_secret_store_and_blanks_column`,
  `legacy_api_hash_remains_when_secret_write_fails`, and
  `missing_secure_api_hash_for_blank_legacy_account_is_auth_error`;
- the remaining 73 identities have no aggregate owner. 8A must assign each one
  from its actual subject before any test move; app diagnostics, routing,
  cancellation, warning, request/read-model, storage/codec, and other
  non-protocol tests remain app-owned during Phase 8. Moving an identity to
  `extractum-core` is a separate slice, not a third Phase 8 owner.

The exact 43 app-fixture identities are frozen by module:

- `sources::identity::tests` (2):
  `load_telegram_identity_returns_typed_row`,
  `load_telegram_runtime_source_pairs_source_with_typed_identity`;
- `sources::sync::tests` (3):
  `determine_sync_policy_only_applies_initial_settings_on_first_sync`,
  `finalize_sync_updates_source_state_and_typed_avatar_cache`,
  `finalize_sync_preserves_existing_legacy_metadata_blob`;
- `sources::items::tests` (19):
  `insert_source_item_writes_payload_and_skips_duplicates`,
  `insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload`,
  `telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate`,
  `telegram_insert_writes_analysis_document_in_same_writer_transaction`,
  `single_telegram_insert_maintains_ready_archive_model`,
  `telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows`,
  `takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build`,
  `insert_telegram_source_item_resolves_topic_membership_only_for_new_item`,
  `scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item`,
  `insert_telegram_source_item_allows_same_message_id_in_different_history_domains`,
  `migrated_small_group_insert_skips_current_history_derived_writes`,
  `migrated_insert_idempotency_uses_old_chat_native_identity`,
  `youtube_transcript_upsert_targets_non_telegram_partial_unique_index`,
  `upsert_youtube_transcript_item_updates_existing_text_and_returns_id`,
  `youtube_comment_upsert_targets_non_telegram_partial_unique_index`,
  `youtube_comment_upsert_writes_analysis_document_and_updates_content`,
  `list_source_items_enriches_youtube_comment_rows_from_raw_payload`,
  `list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed`,
  `upsert_youtube_comment_item_updates_existing_text_and_reaction_count`;
- `sources::topics::tests` (5):
  `forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind`,
  `forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists`,
  `list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket`,
  `upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted`,
  `topic_refresh_rebuilds_materialized_memberships`;
- `takeout_import::tests` (12):
  `channel_private_count_probe_records_fallback_before_search_continuation`,
  `export_dc_fallback_provenance_records_once_before_finalize`,
  `channel_private_validation_preflight_records_fallback_and_continues`,
  `takeout_subtype_load_uses_typed_identity_not_legacy_kind`,
  `takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists`,
  `takeout_parsed_items_with_same_message_id_insert_under_different_history_peers`,
  `takeout_duplicate_parsed_item_updates_topic_unresolved_count_once`,
  `locked_start_conflict_creates_no_provenance_rows`,
  `locked_start_allows_only_one_batch_for_same_source`,
  `migrated_history_start_records_use_same_source_takeout_lock`,
  `migrated_history_start_requires_available_capability`,
  `historical_batch_completion_does_not_advance_source_watermark`;
- `takeout_import::forum_topics::tests` (2):
  `takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize`,
  `takeout_forum_topic_refresh_success_records_no_warning`.

Mixed production files and test modules split by subject rather than moving as
whole files. Crate-local tests may use Grammers TL values, `MemorySession`,
pure DTOs, injected futures, and in-memory secret implementations. They may
not copy app schemas or SQL fixture helpers. No app dev dependency, reverse
dependency, or fixture crate is introduced.

Two of the 43 retained app identities currently combine raw TL parsing with SQL
assertions:

- `takeout_parsed_items_with_same_message_id_insert_under_different_history_peers`;
- `takeout_duplicate_parsed_item_updates_topic_unresolved_count_once`.

8A first characterizes their complete current assertions. 8B then keeps each
baseline identity app-side with preconstructed `TelegramMessageDraft` inputs
for its storage/transaction subject, moves the raw TL fixture constructor into
the portable staging tree, and adds these staged companion identities. They
still execute under `extractum` during 8B but have final owner
`extractum-telegram`:

- `raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids`;
- `raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id`.

The literal map records those companion IDs and the boundary contract proves
that the app plus crate assertions jointly cover the characterized behavior.
The second crate companion proves identical native identity for equal peer and
message IDs; its retained app baseline proves the duplicate storage path is
skipped and increments the unresolved count only once.

The third known mixed identity comes from the 21-test type closure:

- the app keeps the baseline full ID
  `sources::types::tests::item_kind_constants_match_persisted_wire_values`
  with only `youtube_transcript` and `youtube_comment` persisted-wire
  assertions;
- `extractum-telegram` adds companion
  `dto::tests::telegram_item_kind_constant_matches_persisted_wire_value` with
  staged ID
  `telegram_impl::dto::tests::telegram_item_kind_constant_matches_persisted_wire_value`
  and the sole
  `ITEM_KIND_TELEGRAM_MESSAGE == "telegram_message"` assertion.

These are the three known mandatory decompositions. If the 8A classification
proves another genuinely mixed subject, its approved plan must name and
characterize the split before changing the baseline test; a design amendment
is required only if the split changes behavior or the approved ownership/API
boundary.

Across the complete identity map, 140 immutable baseline identities map to
143 final executable identities: one primary for every baseline plus the three
predeclared crate companions.

`sources::items::tests::media_metadata_roundtrip_through_zstd` is explicitly
not a Telegram-crate candidate. It remains an app facade/storage contract for
all of Phase 8; any later move to `extractum-core` is separately designed. The
existing
`media-metadata-core-contract.test.ts` must remain GREEN throughout Phase 8.

Required standing contracts include:

1. a Phase 8 roadmap/status contract in
   `crate-extraction-shell-cap-contract.test.ts`;
2. a new `telegram-crate-boundary-contract.test.ts`;
3. exact workspace-member allowlist updates in all existing crate contracts;
4. a curated crate-root/public-API allowlist;
5. primary metadata-graph proof of final direct dependency ownership, plus the
   secondary absence scan over app Rust source/tests;
6. the exact schema-v1 8B prepared implementation symbol map, its readable
   transitional review inventories, and the intentionally absent
   crate/workspace edge; the map is not executable Rust use-site authority;
7. the literal immutable 140-entry test-identity map, the exact 43
   helper-dependent plus three credential-SQL app assignments, and the two
   raw-TL/SQL companion-test decompositions, plus the 21-identity type-closure
   addendum and its declared item-kind companion decomposition;
8. moved-not-copied source and test ownership;
9. exact Grammers Git revision and feature ownership;
10. actual `Cargo.lock` changes or proved byte identity for each manifest
    checkpoint;
11. frozen command/event/status/session/secret string contracts.

8B ends with GREEN preparation contracts. 8C begins by committing the final
boundary contract intentionally RED only because the crate, workspace edge,
and physical moves are absent, then makes that same contract GREEN without
inventing another seam. Deleting or renaming a path must not make a contract
silently stop checking the underlying symbol or test identity.

A filtered Cargo run that reports zero tests is not evidence. Implementation
helpers must fail closed when a requested exact or prefix suite selects no
tests.

## Draft and Approval Synchronization

The prior draft commit atomically:

1. add this specification with status `Draft for owner review`;
2. update the Phase 8 roadmap entry to `design drafted; awaiting owner
   approval`, link this document, and replace the stale whole-file scope with
   the fresh evidence and selected three-slice boundary;
3. update `crate-extraction-shell-cap-contract.test.ts` to recognize that exact
   draft state and the Phase 8 timing override;
4. run the focused contract test.

After the owner reviewed the committed text and explicitly approved it, this
separate synchronization commit atomically:

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
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
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

For the 8B live command-registration step, pinned
`tauri-plugin-mcp-bridge` 0.11.0 cannot invoke arbitrary application commands
through its `ipc_execute_command` handler; that handler accepts only the
bridge plugin's own commands. The live MCP smoke therefore uses
`webview_execute_js` to evaluate
`(async () => await window.__TAURI__.core.invoke('tg_get_account_statuses', { accountIds: [] }))()`
in the connected main webview and requires the exact result `[]`. This is the
real Tauri IPC path used by the application. The evidence must not be obtained
by upgrading the bridge or changing `Cargo.toml`/`Cargo.lock`.

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
plan-declared preparation checkpoint states, `8A preparation retained`,
`8B preparation retained; 8C pending`, `done: retained`, and `not retained`.

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
  Grammers revision change. Removing the exact dead generic insert seam and
  draft field declared above is boundary work, not an unrelated cleanup.
- No focused timing series, process scanner, quiet-window rule, retry policy,
  cumulative timing/measurement ledger, or new measurement runner. The
  cumulative CP2–CP8 LLM review document required above is not measurement
  infrastructure.
- No TypeScript implementation of Rust import, name, type, scope, shadowing,
  call-graph, or data-flow resolution and no generated escape-hatch
  use-site/edge/occurrence authority. For unexecuted work after retained CP1,
  a fresh independent LLM reviews those source forms and records CLEAN evidence
  durably; Cargo and behavior tests provide the automatic evidence.

## Acceptance Criteria

The design outcome is complete only when:

1. 8A, 8B, and 8C each have an approved bounded implementation plan, retained
   verification evidence, and truthful roadmap status;
2. the exact baseline test identity map is preserved with every identity owned
   once;
3. all twelve account/Telegram commands retain their signatures, registration,
   and observable behavior;
4. existing Telegram and Takeout event names, payloads, statuses, order,
   `AppError` kind/message/JSON, and error strings are unchanged; no parallel
   public error taxonomy or app-facade conversion table exists;
5. session encryption, serialization, account binding, path, key identifiers,
   and legacy migration are byte/string compatible;
6. app-owned source, item, topic, ingest, account, and read-model transactions
   remain single-owner workflows with their current ordering;
7. `extractum-telegram` has no Tauri, SQLx, keyring, application, or foreign
   domain dependency;
8. `extractum-core` uniquely owns provider-neutral media metadata, while
   `extractum-telegram` uniquely owns `TelegramMediaPayload`, classification,
   `TelegramMessageIdentity`, `TelegramItemContext`, the item-kind constant,
   and the single `TelegramMessageDraft` persistence hand-off; no mirror DTO,
   duplicate payload, conversion-only layer, dead generic insert, or draft
   `external_id` exists;
9. all 140 baseline identities are explicitly classified by subject; the
   original 119 retain the exact 43 helper-dependent and three credential-SQL
   app assignments, the 21-identity type closure has one identity-validation
   move plus 20 app primary owners, the three declared mixed identities have
   named crate companions, the final union contains exactly 143 executable
   identities, and no app/crate test-dependency cycle exists;
10. the crate public API matches the existing-symbol visibility allowlist plus
    the predeclared new operation signatures, returns core `AppResult`
    directly, and contains no public `TelegramError`, Grammers type, or
    secret-bearing public field/getter;
11. parsed Cargo metadata proves the application package has one path edge to
    `extractum-telegram` and no direct Grammers dependency while permitting the
    intended transitive Grammers graph only through that edge;
12. app Rust, including tests, contains no Grammers import, path, raw TL type,
    alias/re-export, or direct dependency workaround; existing
    `crate::telegram_impl::` consumer paths remain byte-identical behind the
    private explicit facade; the first 8A checkpoint directly normalizes the
    six identity-seam error paths, and the final 8B staging tree contains no
    `crate::error`, `crate::compression`, or `crate::time` reference; every
    unexecuted CP2→CP8 checkpoint has an independent LLM review of the complete
    transitional exception set and all indirect forwarding paths recorded in
    the cumulative committed verification document, while the TypeScript
    contract performs no Rust use-site analysis; CP7/CP8 review finds no
    transitional escape hatch, and the named Rust compile and behavior gates
    pass;
13. the exact Grammers revision and direct declaration policy live with the
    crate dependency owner, the generated required/forbidden feature baseline
    passes without unclassified feature keys, and the lockfile is current;
14. crate, app, workspace, frontend/full verification, release build, and
    startup gates pass;
15. timing is recorded as advisory evidence only and does not decide
    correctness or retention.

## Implementation-Plan Requirements

Phase 8 is executed through three plans, in order:

1. `8A` contract and session preparation;
2. `8B` complete live-source and Takeout preparation inside the app;
3. `8C` single mechanical extraction and dependency cleanup.

Each plan must:

- start at the current clean HEAD and record drift from this evidence snapshot;
- refresh direct Grammers paths, follow moved-type fan-in/fan-out to a fixed
  point, verify the known 19-file/140-test surface, and refresh manifest
  roots/features plus exact Cargo test identities;
- include a symbol-level source map and the committed literal 140-entry
  test-identity map with `baseline_package`, `baseline_full_id`, `staged_path`,
  `final_owner`, `final_full_id`, and normally empty
  `companion_final_ids`;
- preserve the exact 43 helper-dependent plus three credential-SQL app
  assignments, classify the residual 73 individually, implement the two
  predeclared raw-TL/SQL companion-test decompositions, classify the 21
  type-closure identities as declared, implement its predeclared item-kind
  companion decomposition, and forbid a reverse dev-dependency or copied app
  schema;
- map the dead generic-insert baseline test to the retained Telegram insert
  path while removing `insert_source_item` and draft `external_id`;
- make the first 8A implementation checkpoint record the complete root
  inventory and directly normalize exactly the six identity-seam error paths;
  preserve all behavior/test identities and app-owned facade consumers, then
  make the 8B portable-tree checkpoint prove absence of all three declared
  core-facade roots inside staging while leaving every other `crate::<root>`
  for the ownership map;
- make 8B produce the exact `src-tauri/src/telegram_impl/**` staging tree and
  make 8C move it mechanically to
  `src-tauri/crates/extractum-telegram/src/**`;
- make 8B enforce relative-only internal staging paths and the exact external
  `crate::telegram_impl::` prefix, then make 8C prove byte-for-byte
  relative-path/content-hash identity while retaining that consumer prefix
  through the explicit private compatibility facade;
- preserve the retained CP1 schema-v1 symbol map and change only the
  unexecuted CP2→CP8 instructions: keep the logical-owner arrays as a readable
  LLM-review checklist, require the committed artifact plus complete diff and
  producer/continuation review by a fresh independent reviewer at each
  checkpoint, record every CLEAN verdict in the cumulative committed
  verification document, and forbid TypeScript Rust use-site analysis or
  generated anchors/edges/occurrence fences; automatic evidence for those uses
  is the named Rust compilation and behavior gates;
- reproduce the exact existing-symbol public visibility allowlist and state
  every new operation signature before its RED contract;
- name RED/GREEN tests and non-empty suite helpers before implementation;
- include exact manifest and `Cargo.lock` expectations plus the canonical
  metadata command and committed generated required/forbidden Grammers
  feature baseline;
- make the parsed direct-dependency metadata contract primary and the complete
  app Rust/source scan secondary;
- update boundary/status/workspace-member contracts atomically;
- use separately green commits for preparation, RED contract, extraction, and
  verification disposition;
- include `## Rust Verification Loops`, release/startup evidence where
  required, advisory timing, and a recoverable rollback ladder;
- stop and amend the design if an unexpected dependency, raw public type,
  transaction owner, wire change, or cross-domain port is required.

Implementation may not begin from approval alone. The owner
must explicitly authorize the next plan after reviewing it.
