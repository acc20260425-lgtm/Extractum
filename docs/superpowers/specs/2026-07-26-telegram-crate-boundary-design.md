# Telegram Crate Boundary Design

**Status:** Approved; 8A preparation retained; 8B not started
**Date:** 2026-07-26

**Roadmap authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

**Verification-loop authority:**
[`2026-07-17-focused-rust-loop-design.md`](2026-07-17-focused-rust-loop-design.md)

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
   handles;
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
- `MessageRange`, `TakeoutPage`, and `TakeoutMessage`: owned Takeout operation
  values.

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
client, login, session, secret, and new owned Takeout values are introduced as
new final seam types rather than widening a raw existing type; their fields
remain private unless this specification already names them as an owned DTO.

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
- Takeout-specific public values remain in `takeout/types.rs` and are curated
  through `lib.rs`.

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

The Takeout staging boundary is concrete:

- `transport.rs` owns the raw transport half of current
  `export_dc_invoke_with_provenance`;
- `export_dc.rs` owns export-DC selection and invocation;
- `operations.rs` owns Telegram self-check, init/finish, peer validation,
  migration detect/revalidate, count/history/search/page operations, and pure
  attempt/fallback outcomes;
- `raw_parse.rs` owns peer/TL identity conversion and raw response
  classification/parsing;
- `pagination.rs` owns range, page, and cursor rules;
- `forum_topics.rs` owns the remote forum-topic operation;
- `types.rs` owns the pure Takeout inputs, outputs, attempts, and fallback
  metadata.

The app-owned `takeout_import/mod.rs` keeps commands, jobs, cancellation and
selection loops, persistence, provenance recording, warnings, progress/event
emission, and terminal batch finalization. Concrete crate operations return
pure results plus attempt/fallback metadata; only the app records that metadata
as provenance.

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
6. the exact 8B prepared implementation symbol map and intentionally absent
   crate/workspace edge;
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
  cumulative ledger, or new measurement runner.

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
    `crate::error`, `crate::compression`, or `crate::time` reference;
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
