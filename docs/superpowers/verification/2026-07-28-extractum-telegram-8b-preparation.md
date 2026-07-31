# Extractum Telegram Phase 8B Preparation Verification

## CP2 Escape-Hatch LLM Review

# Phase 8B Checkpoint 2 Escape-Hatch LLM Review

## Review identity

- Checkpoint: Phase 8B preparation Checkpoint 2, “Normalize Existing Workspace Dependencies Without Graph Drift.”
- Retained predecessor: Phase 8B preparation Checkpoint 1 at `9cad3941d8c9c314aa1d7fd63aed163f7586461a`.
- Candidate base/HEAD: `58fea66c870b9144bcd13becd72dc577d0d46428`.
- Candidate worktree patch-id: `62091e08c0b9cd7d10ded86faa80532d100fb559`.
- Reviewer independence: I am a fresh semantic reviewer. I did not implement Checkpoint 2 and did not participate in its implementation conversation. I received the bounded review packet and independently read the complete authority, inventory, candidate Rust context, and forwarding paths described below.
- Review standard: the schema-v1 symbol map and its transition arrays were used only as an input checklist. Literal searches were used only to navigate after the complete reads; no search pattern, occurrence count, or exit status was treated as semantic authorization.

The retained authority chain from CP1 to the candidate base is complete and contains no intervening commit:

1. `1c0961516c338073ba9578edb18a61c7b1285897` — `docs: define generated Telegram bridge-use authority`
2. `56b9d3995f10f0070f4e2d0f94fb048181218ae3` — `docs: move Telegram escape-hatch authority to LLM review`
3. `091b51345fdd4ff03bbc2fe835a26e1e7ee5a8cb` — `docs: harden Telegram LLM review gate`
4. `30603f11a3e6ea359b0d459497169603f0026269` — `test: simplify Telegram escape-hatch review authority`
5. `58fea66c870b9144bcd13becd72dc577d0d46428` — `docs: correct Phase 8B Checkpoint 2 status gate`

`git merge-base --is-ancestor 9cad3941d8c9c314aa1d7fd63aed163f7586461a HEAD` exited 0.

## Candidate identity and scoped change

The final pre-verdict refresh produced:

- `HEAD=58fea66c870b9144bcd13becd72dc577d0d46428`, equal to the dispatched candidate base.
- `patch-id=62091e08c0b9cd7d10ded86faa80532d100fb559`, equal to the dispatched candidate patch-id.
- The complete candidate worktree has exactly four modified paths:
  - `src-tauri/Cargo.toml`
  - `src/lib/crate-extraction-shell-cap-contract.test.ts`
  - `src/lib/gemini-browser-crate-boundary-contract.test.ts`
  - `src/lib/telegram-crate-boundary-contract.test.ts`
- I read the complete four-file candidate diff and the complete resulting `src-tauri/Cargo.toml`. The manifest change only promotes the seven Task 2 dependencies to `[workspace.dependencies]` and changes the app specifications to workspace inheritance. The contract changes assert that normalization and admit the CP2 retained-status vocabulary. No Rust source path is part of Task 2 or the candidate diff.

The complete production-and-test Rust evidence is empty in every dimension:

- `git diff --no-ext-diff --binary 9cad3941d8c9c314aa1d7fd63aed163f7586461a..HEAD -- '*.rs'` emitted no diff and exited 0.
- `git diff --no-ext-diff --binary 9cad3941d8c9c314aa1d7fd63aed163f7586461a -- '*.rs'` emitted no diff and exited 0.
- `git status --short -- '*.rs'` emitted nothing.
- Unstaged Rust name-status inventory emitted nothing.
- Staged Rust name-status inventory emitted nothing.
- Untracked non-ignored Rust inventory emitted nothing.

The PowerShell/Git warnings about the inaccessible user-level ignore file and possible future LF-to-CRLF conversion in three TypeScript files do not alter those results: all commands exited successfully, the candidate patch-id matched exactly, and no warning named a Rust path.

## Complete authority and inventory inputs

I read these authority artifacts completely, not as filtered excerpts:

- `docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md`, HEAD/worktree blob `019940d2d3a16f38deb7868ed1e4c09f4c4776cd`.
- `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`, HEAD/worktree blob `b51a221bcedfaaccc862cd82d92bc0f53d033aef`.
- `src/lib/telegram-8b-symbol-map.json`, HEAD/worktree blob `c0acda302fc70617eb1c3f9f1dcb9d849f3ab059`, equal to the dispatched expected blob.

The map is `schemaVersion: 1`. I read all 306 symbol-disposition rows, the complete 67-entry `restrictedFinalSymbols` list, the restricted bridge fence authority (`normalizedLfBytes: 2184`, SHA-256 `87a520b5917ac82c95527210120616e3e9e5b66141d6492ce46a036d75b2c5ca`), and every complete transition inventory:

```text
cp3RawHandleCallsites:
  sources::store::add_telegram_source
  sources::store::list_telegram_sources
  sources::sync::sync_telegram_source
  takeout_import::run_export_dc_spike_for_handle
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

cp4RawHandleCallsites:
  sources::sync::sync_telegram_source
  takeout_import::run_export_dc_spike_for_handle
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

cp4ResolvedSyncPeerPeerConsumers:
  sources::sync::sync_telegram_source
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

cp5RawHandleCallsites:
  takeout_import::run_export_dc_spike_for_handle
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

cp5ResolvedSyncPeerPeerConsumers:
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

cp7RawBridgeSymbolsAndCallsites:
  <empty>
```

The current six raw-handle owners exactly match `cp3RawHandleCallsites`. The CP4 inventories remove only the two store owners and retain the three exact peer consumers; the CP5 inventories remove the sync owner/consumer and retain the three Takeout raw owners and two Takeout peer consumers; CP7 is empty. These are future transition checklists, not CP2 executable source authority.

The relevant symbol rows agree with the prose authority:

- `telegram/media.rs::{DocumentSignals,derive_content_kind,derive_document_media_kind,extract_item_payload}` move at CP3.
- The same four `media.rs` compatibility names are redirected/retained through CP6 and removed at CP7.
- `TelegramClientHandle::{raw_client,raw_session}` and `TelegramSession::raw_memory_session` move at CP3 and are deleted at CP7.
- `telegram::{get_client,get_authorized_client}` and `ResolvedSyncPeer::peer` are existing transitional symbols deleted at CP7.
- `legacy_peer_ref_from_descriptor` is a new CP4 transitional copy deleted at CP7; therefore its absence at CP2 is required, not a missing CP2 definition.

## Rust paths read completely

I read every line of the required unchanged Rust context:

- `src-tauri/src/telegram/media.rs`
- `src-tauri/src/telegram.rs`
- `src-tauri/src/media.rs`
- `src-tauri/src/takeout_import/raw_parse.rs`
- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/telegram/runtime.rs`
- `src-tauri/src/telegram/session.rs`
- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/peer_resolution.rs`
- `src-tauri/src/takeout_import/mod.rs`
- `src-tauri/src/sources/mod.rs`
- `src-tauri/src/telegram_session_store.rs`
- `src-tauri/src/takeout_import/forum_topics.rs`
- `src-tauri/src/sources/topics.rs`

I also read these complete adjacent producer/forwarding paths to remove possible ambiguity:

- `src-tauri/src/sources/identity.rs`
- `src-tauri/src/sources/avatar.rs`
- `src-tauri/src/takeout_import/export_dc.rs`

## Exact media compatibility definitions and physical uses

The exact media compatibility set contains four items and only four items:

1. `DocumentSignals`
2. `derive_content_kind`
3. `derive_document_media_kind`
4. `extract_item_payload`

Definitions remain in `src-tauri/src/telegram/media.rs`:

- `DocumentSignals` is `pub(crate)` and has exactly five `pub(crate)` fields:
  - `mime_type: Option<String>`
  - `has_video: bool`
  - `has_audio: bool`
  - `is_voice: bool`
  - `is_animated: bool`
- `derive_content_kind` is `pub(crate)`.
- `derive_document_media_kind` is `pub(crate)`.
- `extract_item_payload` is `pub(crate)`.

Every physical source use is accounted for:

- `DocumentSignals`
  - `telegram::media::collect_document_signals` returns it, constructs it, fills all applicable fields from a Grammers document, and supplies it to `derive_document_media_kind`.
  - `takeout_import::raw_parse::extract_document_media_payload` constructs it, fills the same five signals from raw TL document/media flags, and supplies it to `derive_document_media_kind`.
  - `telegram::media` unit tests construct the voice, video, and image cases and pass each to `derive_document_media_kind`.
  - There is no sixth field, second signal type, or other consumer.
- `derive_content_kind`
  - Called by `telegram::media::extract_item_payload`.
  - Called by `takeout_import::raw_parse::parse_raw_message`.
  - Called by the `telegram::media` content-kind unit test for text-only, text-with-media, and media-only cases.
- `derive_document_media_kind`
  - Called by `telegram::media::extract_document_media_payload`.
  - Called by `takeout_import::raw_parse::extract_document_media_payload`.
  - Called by the three `telegram::media` document-kind unit-test cases.
- `extract_item_payload`
  - Imported from `crate::media` and called once in `sources::sync::persist_items` for each live Grammers message.

The complete media forwarding chains are:

```text
sources::sync_source
  -> sources::sync_telegram_source
  -> sources::persist_items
  -> media facade::extract_item_payload
  -> telegram::media::extract_item_payload
  -> extract_media_payload / derive_content_kind

start_takeout_source_import
  -> run_takeout_import_job
  -> run_takeout_source_import
  -> run_started_takeout_source_import
  -> run_started_takeout_source_import_inner
  -> import_takeout_history_ranges
  -> import_takeout_history_pages
  -> raw_parse::parse_raw_message
  -> derive_content_kind
  -> extract_raw_media_payload
  -> extract_document_media_payload
  -> DocumentSignals / derive_document_media_kind

start_takeout_migrated_history_import
  -> run_takeout_migrated_history_import_job
  -> run_takeout_migrated_history_import
  -> import_takeout_history_ranges
  -> import_takeout_history_pages
  -> the same raw_parse chain
```

Routing remains explicit:

- `telegram.rs` privately declares `mod media` and explicitly `pub(crate) use`s the four names from that leaf.
- `media.rs` explicitly `pub(crate) use`s the same four names from `crate::telegram`.
- `raw_parse.rs` explicitly imports only the three compatibility items it uses there.
- `sync.rs` explicitly imports only `extract_item_payload`.

`TelegramMediaPayload` appears beside the compatibility names, but it is not a fifth compatibility item: the authority names it as the single typed media payload in the curated terminal root API, and the symbol map gives it its own move/replace disposition. Likewise, `ItemMediaMetadata` and `media_label` are provider-neutral `extractum_core` media APIs, and the `CONTENT_KIND_*` constants are leaf/test support rather than an additional compatibility facade item. There is no fifth media compatibility definition or facade export.

## Exact raw bridge definitions and physical uses

The complete authorized raw bridge state at CP2 is:

- `TelegramClientHandle::raw_client`
  - Defined `pub(crate)` in `telegram/runtime.rs`.
  - Six physical calls, exactly at the six CP3 logical owners:
    - `sources::store::list_telegram_sources`
    - `sources::store::add_telegram_source`
    - `sources::sync::sync_telegram_source`
    - `takeout_import::run_export_dc_spike_for_handle`
    - `takeout_import::run_takeout_migrated_history_import`
    - `takeout_import::run_takeout_source_import`
- `TelegramClientHandle::raw_session`
  - Defined `pub(crate)` in `telegram/runtime.rs`.
  - Its body is the exact wrapper call `self.session.raw_memory_session()`.
  - Three physical owner calls:
    - `takeout_import::run_export_dc_spike_for_handle`
    - `takeout_import::run_takeout_migrated_history_import`
    - `takeout_import::run_takeout_source_import`
- `TelegramSession::raw_memory_session`
  - Defined `pub(super)` in `telegram/session.rs`.
  - Ten physical call expressions:
    - Production: `session::memory_session_to_saved`.
    - Production: `TelegramClientHandle::raw_session`.
    - Production: `runtime::initialize_grammers_client`.
    - Test: `session::encrypted_session_load_round_trips` once.
    - Test: `runtime::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner` twice.
    - Test: `runtime::failed_sign_in_retains_pending_attempt` twice.
    - Test: `runtime::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt` twice.
- `telegram::get_client`
  - Defined `pub(crate)` and delegates only to `TelegramRuntime::initialized_client`.
  - Called only by `list_telegram_sources` and `add_telegram_source`.
- `telegram::get_authorized_client`
  - Defined `pub(crate)` and delegates only to `TelegramRuntime::authorized_client`.
  - Called only by `sources::sync::sync_telegram_source`, `run_takeout_export_dc_spike`, `run_takeout_migrated_history_import`, and `run_takeout_source_import`.
- `ResolvedSyncPeer::peer`
  - Defined `pub(crate)` as `PeerRef` in `sources/peer_resolution.rs`.
  - Written only by `resolve_and_refresh_peer` when it constructs `ResolvedSyncPeer`.
  - Read nine times by exactly three logical consumers:
    - `sources::sync::sync_telegram_source`: twice, for topic refresh and item persistence.
    - `takeout_import::run_takeout_migrated_history_import`: twice, for resolved-peer provenance and migrated-chat revalidation.
    - `takeout_import::run_takeout_source_import` and its required forwarding chain: once in the owner for resolved-peer provenance, then four times in the inner function for input-peer conversion, peer validation, migration detection, and completed-Takeout forum-topic refresh.
  - `sources/mod.rs` explicitly re-exports `ResolvedSyncPeer` and `resolve_and_refresh_peer` only as `pub(crate)`.
- `legacy_peer_ref_from_descriptor`
  - No definition, import, re-export, wrapper, alias, call, function item, field, or macro-generated occurrence exists at CP2.
  - Its schema row says `<new>`, first checkpoint 4, `new-then-delete`, removal checkpoint 7. It must not exist yet.

The session bridge’s complete production forwarding is also unchanged:

```text
telegram_session_store::save_session / legacy rewrite
  -> telegram::encode_session_json
  -> session::memory_session_to_saved
  -> TelegramSession::raw_memory_session

TelegramRuntime::initialize_account
  -> runtime::initialize_grammers_client
  -> TelegramSession::raw_memory_session
  -> Arc clone into SenderPool

Takeout raw owners
  -> TelegramClientHandle::raw_session
  -> TelegramSession::raw_memory_session
  -> Arc clone
  -> export_dc::prepare_export_dc_alias
  -> MemorySession home-DC/DC-option operations
```

## Logical owners and full raw forwarding chains

### `sources::store::list_telegram_sources`

```text
Tauri command list_telegram_sources
  -> telegram::get_client
  -> TelegramRuntime::initialized_client
  -> TelegramClientHandle
  -> raw_client
  -> cloned Grammers Client
  -> Client::iter_dialogs
  -> dialog.peer()
  -> telegram_source_info_from_peer
  -> peer_photo_data_url_with_timeout
  -> peer_photo_bytes_with_timeout
  -> peer_photo_bytes
```

The command itself is the logical owner. Only the cloned client and dialog peers are forwarded; the handle is not forwarded or wrapped again.

### `sources::store::add_telegram_source`

```text
Tauri command add_telegram_source
  -> telegram::get_client
  -> TelegramRuntime::initialized_client
  -> TelegramClientHandle
  -> raw_client
  -> cloned Grammers Client
  -> resolve_telegram_source
     -> resolve_telegram_source_by_username
     or resolve_telegram_source_from_dialogs
  -> optional avatar download/cache
  -> typed source persistence
```

The command itself is the logical owner. No additional handle/raw-client facade is introduced.

### `sources::sync::sync_telegram_source`

```text
Tauri command sync_source
  -> provider dispatch
  -> sync_telegram_source
  -> telegram::get_authorized_client
  -> TelegramRuntime::authorized_client
  -> TelegramClientHandle
  -> raw_client
  -> cloned Grammers Client
  -> resolve_and_refresh_peer
  -> ResolvedSyncPeer
     -> peer -> refresh_forum_topics -> fetch_all_forum_topics
     -> peer -> persist_items -> Client::iter_messages
                            -> extract_item_payload
  -> finalize_sync with refreshed_avatar_cache_key
```

`sync_telegram_source` is the exact raw and peer logical owner. `persist_items` and topic refresh receive a cloned client reference and copied `PeerRef`; neither receives the handle nor creates another bridge.

### `takeout_import::run_export_dc_spike_for_handle`

```text
Tauri command run_takeout_export_dc_spike
  -> telegram::get_authorized_client
  -> TelegramRuntime::authorized_client
  -> TelegramClientHandle moved directly to run_export_dc_spike_for_handle
  -> raw_client -> cloned Client
  -> raw_session -> raw_memory_session -> cloned Arc<MemorySession>
  -> Client self-user check
  -> prepare_export_dc_alias(session)
  -> export_dc_invoke(client) for init, split ranges, and finish
  -> export_dc_invoke_with
     -> Client::invoke_in_dc or fallback Client::invoke
```

The outer command owns the `get_authorized_client` physical use; `run_export_dc_spike_for_handle` is the exact inventory owner of both raw handle calls.

### `takeout_import::run_takeout_migrated_history_import`

```text
Tauri command start_takeout_migrated_history_import
  -> spawned run_takeout_migrated_history_import_job
  -> run_takeout_migrated_history_import
  -> telegram::get_authorized_client
  -> raw_client -> cloned Client
  -> raw_session -> raw_memory_session -> cloned Arc<MemorySession>
  -> resolve_and_refresh_peer
  -> ResolvedSyncPeer.peer
     -> peer_ref_identity for provenance
     -> revalidate_migrated_from_chat_id
        -> export_dc_invoke_with_provenance
  -> prepare_export_dc_alias(session)
  -> export_dc_invoke_with_provenance(client) for Takeout init/splits
  -> takeout_history_count_probe
     -> takeout_get_history or takeout_search_my_messages
     -> export_dc_invoke_with_provenance
  -> import_takeout_history_ranges
     -> import_takeout_history_pages
     -> takeout_history_page_response
     -> takeout_get_history or takeout_search_my_messages
     -> export_dc_invoke_with_provenance
  -> finish_takeout_session
```

The owner acquires both raw values once. All later functions receive only typed references/copies to those values; none reacquires a handle or defines a wrapper.

### `takeout_import::run_takeout_source_import`

The mandatory full forwarding chain is present and unchanged:

```text
Tauri command start_takeout_source_import
  -> spawned run_takeout_import_job
  -> run_takeout_source_import
     -> telegram::get_authorized_client
     -> raw_client -> cloned Client
     -> raw_session -> raw_memory_session -> cloned Arc<MemorySession>
     -> resolve_and_refresh_peer -> ResolvedSyncPeer
     -> ResolvedSyncPeer.peer -> peer_ref_identity for provenance
     -> Client self-user check
     -> prepare_export_dc_alias(session)
     -> export_dc_invoke_with_provenance(client) for Takeout init
     -> run_started_takeout_source_import
        -> forwards the complete ResolvedSyncPeer, &Client, alias, takeout id,
           warnings, fallback state, and attempt state unchanged
        -> run_started_takeout_source_import_inner
           -> ResolvedSyncPeer.peer -> InputPeer conversion
           -> ResolvedSyncPeer.peer -> validate_takeout_peer
           -> ResolvedSyncPeer.peer -> detect_supergroup_migration
           -> split/count/history branches
              -> takeout_history_count_probe
              -> import_takeout_history_ranges
              -> import_takeout_history_pages
              -> takeout_history_page_response
              -> takeout_get_history or takeout_search_my_messages
              -> export_dc_invoke_with_provenance
           -> finish_takeout_session
           -> ResolvedSyncPeer.peer
              -> refresh_forum_topics_after_completed_takeout
              -> sources::refresh_forum_topics
              -> fetch_all_forum_topics
           -> refreshed_avatar_cache_key -> finalize_sync
```

`run_takeout_source_import` is the single logical raw/peer owner across this three-function chain. The two started-import functions forward the already-produced values; they do not become additional owners, aliases, or raw accessors.

### Common `ResolvedSyncPeer` producer

All three peer consumers share one unchanged producer:

```text
resolve_and_refresh_peer
  -> load_telegram_source_identity
  -> resolve_source_peer_from_typed_identity
     -> typed_peer_resolution_plan
     -> stored branch: TelegramSourceIdentity::peer_ref
     or username/dialog branch:
        peer_ref_for_typed_identity
        -> peer_ref_for_source_subtype
  -> refresh_source_avatar_cache
  -> ResolvedSyncPeer { peer, refreshed_avatar_cache_key }
```

`TelegramSourceIdentity::peer_ref`, the test-only `source_peer_ref_from_identity`, `peer_ref_for_typed_identity`, and `peer_ref_for_source_subtype` are existing app-local peer construction helpers recorded by the 306-row map for CP4 replacement. They are not a second accessor from `TelegramClientHandle`, do not cross a staged package boundary at CP2, and do not expose another field on `ResolvedSyncPeer`. The map’s CP4 plan replaces their relevant conversion with the not-yet-existing `legacy_peer_ref_from_descriptor`.

## Aliases, re-exports, wrappers, function items, shadowing, macros, globs, and same-named values

- Aliases: no exception symbol is imported with `as`, renamed, type-aliased, or bound under a second callable/accessor name. `Engine as _` in avatar code is unrelated.
- Re-exports:
  - The exact four media items have only the explicit `telegram/media.rs -> telegram.rs -> media.rs` route described above.
  - `TelegramClientHandle` and `TelegramSession` are explicitly re-exported from private leaf modules through `telegram.rs`.
  - `ResolvedSyncPeer` is explicitly re-exported `pub(crate)` through `sources/mod.rs`.
  - No raw accessor/helper is independently re-exported from a second facade.
- Wrappers:
  - `get_client` wraps only `initialized_client`.
  - `get_authorized_client` wraps only `authorized_client`.
  - `raw_session` wraps only `raw_memory_session`.
  - `run_takeout_export_dc_spike` forwards one returned handle to the named owner.
  - `run_started_takeout_source_import` forwards the already-created values to its inner function.
  - No other wrapper returns a raw client/session or `ResolvedSyncPeer::peer`.
- Function items: every exact helper occurrence is a definition, explicit import/re-export, type/construction use, or direct call/method call. No exception function/method is captured, stored, returned as a function item, passed as a callback, or invoked through UFCS under another owner.
- Shadowing/bindings:
  - Owners bind raw results to local `client` and `session`; the declared handle type and immediate method call make the producer unambiguous.
  - `memory_session_to_saved` intentionally shadows its `TelegramSession` parameter with the borrowed `MemorySession` returned by the one recorded physical accessor call.
  - `run_export_dc_spike_for_handle` names its typed parameter `handle`; it is a `TelegramClientHandle`, not a Tauri `AppHandle`.
  - `resolved_peer` bindings are all produced by `resolve_and_refresh_peer`; there is no second producer with the same local name.
- Macros:
  - No reviewed path contains `macro_rules!`, `paste!`, or `concat_idents!` that can synthesize an exception name.
  - `#[tauri::command]`, derive attributes, and test attributes wrap ordinary functions/types but do not generate or authorize a second raw bridge.
- Globs:
  - There is no exception-related production glob import or re-export and no relevant public module glob.
  - The complete reviewed context has test-only lexical `use super::*` imports in `sources/store.rs`, `sources/peer_resolution.rs`, `sources/identity.rs`, `telegram/runtime.rs`, `telegram/session.rs`, and `telegram_session_store.rs`. None is a re-export, and their physical exception calls are already listed above.
  - The source-wide `diagnostics/mod.rs` `pub(crate) use dto::*` is unrelated to Telegram and routes none of these names.
- Module visibility:
  - `lib.rs` declares `mod media`, `mod telegram`, and `mod sources`; `telegram.rs` declares private `mod media`, `mod runtime`, and `mod session`.
  - There is no newly public Telegram/media/source module and no public module added by CP2.
- Unrelated same-named fields and values:
  - Grammers dialog `.peer()` methods, helper parameters named `peer`, and raw TL request literal fields named `peer` are downstream values, not `ResolvedSyncPeer::peer` definitions or alternate producers.
  - Raw TL/application fields such as `peer_id`, `saved_peer_id`, `reply_to_peer_id`, `peer_kind`, and `peer_identity` are distinct names and typed identity data.
  - `ItemMediaMetadata::mime_type` values in Telegram parsing and other application domains are not `DocumentSignals::mime_type`; `has_video`, `has_audio`, `is_voice`, and `is_animated` occur as `DocumentSignals` fields only.
  - `TelegramClientHandle`’s private `client` and `session` fields and `TelegramSession`’s private `inner` field are backing storage, not additional accessors.
  - `ResolvedSyncPeer::refreshed_avatar_cache_key` is the sole second field on that struct and is provider-neutral cache metadata, not a raw bridge.

There is therefore no public module, exception-related glob, fifth media compatibility item, additional raw handle/session accessor, second resolved-peer field, hidden alias, function-item escape, or macro-created bridge.

## Required CP2 removals

There are no escape-hatch removals required at Checkpoint 2. Task 2 changes only dependency ownership in `Cargo.toml` plus structural contracts. The exact CP1 media and raw bridge definitions, physical uses, logical owners, and forwarding chains must remain unchanged. `legacy_peer_ref_from_descriptor` remains absent until its declared CP4 introduction.

## CP1 blob equality

Every required reviewed Rust worktree blob is byte-identical to the retained CP1 blob:

```text
src-tauri/src/telegram/media.rs                    38f6958524be7af8c1710e41cd577673bf331ae3
src-tauri/src/telegram.rs                          18242273e973276846ce1994470a118ba5e6cb04
src-tauri/src/media.rs                             2385f770a541d6658df5b9b39e7e448f52924059
src-tauri/src/takeout_import/raw_parse.rs          11dc7eb948e15747837eb8d395c1071f24648826
src-tauri/src/sources/sync.rs                      056dcb083137792ad7d396d516640e480d2f591b
src-tauri/src/telegram/runtime.rs                  a897668d10a7a079e527c1503ca00e35def11b09
src-tauri/src/telegram/session.rs                  e558cc0cc92f3c5bc6702008a07036d722c26232
src-tauri/src/sources/store.rs                     b4c8a662f8a066ff36ca1e2a499603162b27773d
src-tauri/src/sources/peer_resolution.rs           b09c263056f9e5cb9a0b8531f75a82dc3fde1090
src-tauri/src/takeout_import/mod.rs                594dd7f4059a2f17a3c588511ba5ae1318be1a98
src-tauri/src/sources/mod.rs                       a8cfbd18032f6206abaf53bfb948523a6c11699d
src-tauri/src/telegram_session_store.rs            80a495c9630a4a2d0d82d1a04ad08007fed15651
src-tauri/src/takeout_import/forum_topics.rs       fd1ed71606a0b2ad7d3f24fb54c3d7e7c9aa88a6
src-tauri/src/sources/topics.rs                    9e592e1ad7a60d09086da06ed2e84b99cf8c2611
```

The three additional complete context reads are also byte-identical to CP1:

```text
src-tauri/src/sources/identity.rs                  5c243c306bcab733da220be0628838dabbde88bc
src-tauri/src/sources/avatar.rs                    98f285c174548baf4421b741ec652e52274bbf92
src-tauri/src/takeout_import/export_dc.rs          dcc0f3b09e4280f1ca6fe40541da3950bd43c03f
```

## Explicit CP2 no-Rust-change findings

- No escape-hatch definition change: the complete CP1-to-candidate production-and-test Rust diff is empty, all definition-host blobs equal CP1, and the definitions/visibilities remain exactly as inventoried.
- No escape-hatch physical-use change: the complete CP1-to-candidate production-and-test Rust diff is empty, all consumer/test blobs equal CP1, and every physical use listed above is unchanged.
- No escape-hatch logical-owner change: the complete CP1-to-candidate production-and-test Rust diff is empty, the six raw owners and three resolved-peer consumers exactly reconcile with the schema-v1 transition inventories, and no new owner exists.
- No escape-hatch forwarding-chain change: the complete CP1-to-candidate production-and-test Rust diff is empty, every forwarding-path blob equals CP1, and the full `run_takeout_source_import -> run_started_takeout_source_import -> run_started_takeout_source_import_inner` chain plus all other branches remain unchanged.

## Findings

1. Candidate identity and authority chain match the dispatched review packet exactly.
2. Task 2 and the complete candidate patch contain no Rust source change.
3. The current media compatibility facade is the exact four-item set with the exact five-field `DocumentSignals`; no fifth compatibility item exists.
4. All raw definitions, physical calls, logical owners, common producers, and forwarding chains reconcile without an unlisted or ambiguous escape hatch.
5. `legacy_peer_ref_from_descriptor` is correctly absent at CP2 and scheduled for CP4.
6. No CP2 removal is required.
7. No blocking finding remains.

Escape-hatch review verdict: CLEAN

## Superseded CP3 Escape-Hatch LLM Review (invalidated)

This record was invalidated by a subsequent rustfmt Rust change and is retained only for audit history.

### Review identity and independence

- Review date: 2026-07-31.
- Reviewer: independent, read-only LLM reviewer; I did not implement this checkpoint and made no repository edits.
- Retained predecessor: `951b88b004ba4493a73bd2ccf93a2e8aa31dae6d`.
- Actual implementation-diff base and authority-correction HEAD: `0d91cbd800bd94f6dcbfef48072b227f099cb9a8`.
- Reviewed candidate: the complete uncommitted CP3 working-tree Rust diff on that HEAD.
- A final identity/status recheck still showed HEAD `0d91cbd800bd94f6dcbfef48072b227f099cb9a8` and exactly the expected 20 changed Rust paths. No reviewer action changed the candidate.

### Authority and artifact inputs

The review read completely:

- `docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md`, including the global constraints, forward-only amendment, mandatory CP2–CP8 LLM gate, and Task 3.
- `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`.
- `src/lib/telegram-8b-symbol-map.json`: schema version 1, all 306 symbol-disposition rows, all six transition inventories, and all 67 restricted-final-symbol entries.
- Restricted bridge fence authority: 2,184 normalized-LF bytes, SHA-256 `87a520b5917ac82c95527210120616e3e5b66141d6492ce46a036d75b2c5ca`.
- The active `cp3RawHandleCallsites` inventory:
  - `sources::store::add_telegram_source`
  - `sources::store::list_telegram_sources`
  - `sources::sync::sync_telegram_source`
  - `takeout_import::run_export_dc_spike_for_handle`
  - `takeout_import::run_takeout_migrated_history_import`
  - `takeout_import::run_takeout_source_import`
- The complete later CP4, CP5, and CP7 transition arrays were also read to detect early removal, early introduction, or lifecycle drift.
- The forward-only CP3–CP4 peer-kind amendment was applied separately from the schema-v1 artifact, as required.

Literal searches were used only for navigation and independent occurrence cross-checking. Authorization below comes from reading definitions, imports, bodies, producers, consumers, and complete forwarding chains. TypeScript was not treated as Rust escape-hatch authority.

### Reviewed Rust scope

Created and read completely:

- `src-tauri/src/telegram_impl/lib.rs`
- `src-tauri/src/telegram_impl/dto.rs`
- `src-tauri/src/telegram_impl/media.rs`
- `src-tauri/src/telegram_impl/runtime.rs`
- `src-tauri/src/telegram_impl/session.rs`

Deleted and compared completely against the retained predecessor:

- `src-tauri/src/telegram/dto.rs`
- `src-tauri/src/telegram/media.rs`
- `src-tauri/src/telegram/runtime.rs`
- `src-tauri/src/telegram/session.rs`

Modified and read completely:

- `src-tauri/src/lib.rs`
- `src-tauri/src/telegram.rs`
- `src-tauri/src/telegram_session_store.rs`
- `src-tauri/src/media.rs`
- `src-tauri/src/ingest_provenance.rs`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/sources/mod.rs`
- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/takeout_import/mod.rs`
- `src-tauri/src/takeout_import/migrated_history.rs`
- `src-tauri/src/takeout_import/raw_parse.rs`

Unchanged surrounding producer/consumer context read completely:

- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/peer_resolution.rs`

### Staged-root and visibility reconciliation

`src-tauri/src/lib.rs:68-69` installs the private app module exactly as:

```rust
#[path = "telegram_impl/lib.rs"]
mod telegram_impl;
```

`telegram_impl/lib.rs` declares exactly four private leaves—`dto`, `media`, `runtime`, and `session`—and no public module or glob export.

Its permanent public surface is curated as follows:

- DTO: `TelegramItemContext`, `TelegramMessageDraft`, `TelegramMessageIdentity`, and `ITEM_KIND_TELEGRAM_MESSAGE`.
- Media: `TelegramMediaPayload`.
- Runtime: `TelegramApiHash`, `TelegramClientHandle`, `TelegramLoginAttempt`, `TelegramRuntime`, and `TelegramRuntimeStatus`.
- Session: `decode_session_json`, `encode_session_json`, `session_json_requires_existing_key`, `SessionEncryptionKey`, and `TelegramSession`.

Its temporary package-private root surface is exactly:

- `DocumentSignals`
- `derive_content_kind`
- `derive_document_media_kind`
- `extract_item_payload`
- `TELEGRAM_PEER_KIND_CHANNEL`
- `TELEGRAM_PEER_KIND_CHAT`
- `TELEGRAM_PEER_KIND_USER`

There is no public leaf module, broad facade, wildcard export, fifth media compatibility item, or additional peer-kind root bridge.

### Exact media compatibility inventory

The compatibility set contains exactly four names.

1. `DocumentSignals`

   - Definition: `telegram_impl/media.rs:16`, package-private, with the unchanged fields `mime_type`, `has_video`, `has_audio`, `is_voice`, and `is_animated`.
   - Staged-root re-export: `telegram_impl/lib.rs:14`.
   - App compatibility facade: `media.rs:2`.
   - Leaf production uses: `collect_document_signals` at `telegram_impl/media.rs:37-57` and the parameter of `derive_document_media_kind` at line 60.
   - Takeout consumer: imported at `takeout_import/raw_parse.rs:5`, constructed at lines 145-148, and populated at lines 155-189.
   - Leaf tests retain their original direct uses at `telegram_impl/media.rs:227,243-260`.

2. `derive_content_kind`

   - Definition: `telegram_impl/media.rs:29-35`.
   - Staged-root and app-facade re-exports: `telegram_impl/lib.rs:14` and `media.rs:2`.
   - Leaf production use: `extract_item_payload` at `telegram_impl/media.rs:221`.
   - Takeout raw-parser consumer: import and call at `takeout_import/raw_parse.rs:5,26`.
   - Leaf tests: `telegram_impl/media.rs:227,233-238`.
   - Live sync receives its result through `extract_item_payload`; it does not introduce another direct helper owner.

3. `derive_document_media_kind`

   - Definition: `telegram_impl/media.rs:60-80`.
   - Staged-root and app-facade re-exports: `telegram_impl/lib.rs:14` and `media.rs:2`.
   - Leaf production use: document extraction at `telegram_impl/media.rs:104`.
   - Takeout raw-parser consumer: import and call at `takeout_import/raw_parse.rs:5,191`.
   - Leaf tests: `telegram_impl/media.rs:227,249,256,262`.

4. `extract_item_payload`

   - Definition: `telegram_impl/media.rs:209-222`.
   - Staged-root and app-facade re-exports: `telegram_impl/lib.rs:14` and `media.rs:2`.
   - Sole app consumer: import and call in `sources/sync.rs:7,137`.

The frozen consumer split is therefore exact:

- `sources::sync` consumes only `extract_item_payload`.
- `takeout_import::raw_parse` consumes `DocumentSignals`, `derive_content_kind`, and `derive_document_media_kind`.

`TelegramMediaPayload` is a permanent public staged API imported directly from `telegram_impl`, not a fifth compatibility-facade item. `ItemMediaMetadata` and `media_label` remain provider-neutral app-media exports. The `CONTENT_KIND_*` constants remain leaf-internal/test implementation details and are not re-exported through the staged root or app facade.

### Raw/session definitions and every physical use

- `TelegramClientHandle::raw_client`
  - Definition: `telegram_impl/runtime.rs:45-53`, `pub(crate)`.
  - Exactly six calls:
    - `sources/store.rs:111`
    - `sources/store.rs:284`
    - `sources/sync.rs:288`
    - `takeout_import/mod.rs:407`
    - `takeout_import/mod.rs:711`
    - `takeout_import/mod.rs:967`

- `TelegramClientHandle::raw_session`
  - Definition: `telegram_impl/runtime.rs:55-57`, `pub(crate)`.
  - It is an exact wrapper over `self.session.raw_memory_session()`.
  - Exactly three calls:
    - `takeout_import/mod.rs:408`
    - `takeout_import/mod.rs:712`
    - `takeout_import/mod.rs:968`

- `TelegramSession::raw_memory_session`
  - Definition: `telegram_impl/session.rs:183-185`, `pub(super)`, with its borrowed `&Arc<MemorySession>` signature unchanged.
  - Exactly two physical calls:
    - `telegram_impl/session.rs:73`, from `memory_session_to_saved`.
    - `telegram_impl/runtime.rs:56`, through `raw_session`.
  - It correctly coexists with the permanent clone bridge during CP3–CP6.

- App `get_client`
  - Definition: `telegram.rs:430-435`, `pub(crate)`.
  - Calls `TelegramRuntime::client`.
  - Physical app calls only at `sources/store.rs:110,283`.

- App `get_authorized_client`
  - Definition: `telegram.rs:437-442`, `pub(crate)`.
  - Calls unchanged `TelegramRuntime::authorized_client`.
  - Physical calls only at:
    - `sources/sync.rs:287`
    - `takeout_import/mod.rs:390`
    - `takeout_import/mod.rs:710`
    - `takeout_import/mod.rs:966`

- `ResolvedSyncPeer::peer`
  - Definition: `sources/peer_resolution.rs:95-98`, `pub(crate)`.
  - Producer: `resolve_and_refresh_peer` resolves a typed-identity `PeerRef` at lines 647-650 and writes it through constructor shorthand at lines 652-655.
  - Package-private type re-export: `sources/mod.rs:49`.
  - Exactly nine reads:
    - Live sync: `sources/sync.rs:292,294`.
    - Migrated-history import: `takeout_import/mod.rs:724,780`.
    - Current-source Takeout chain: `takeout_import/mod.rs:980,1151,1169,1186,1337`.

No raw definition or call is missing from the active CP3 inventory, and no seventh logical owner or additional raw callsite exists.

### Six logical raw owners

1. `sources::store::list_telegram_sources`

   Chain: `list_telegram_sources` (`store.rs:106`) → `get_client` (`110`) → `TelegramRuntime::client` → private initialized lookup → `raw_client` (`111`) → dialog iteration and avatar reads. The lookup intentionally does not authorize.

2. `sources::store::add_telegram_source`

   Chain: `add_telegram_source` (`store.rs:276`) → `get_client` (`283`) → non-authorizing runtime lookup → `raw_client` (`284`) → `resolve_telegram_source`. No raw session or `ResolvedSyncPeer` field is involved.

3. `sources::sync::sync_telegram_source`

   Chain: `sync_source` dispatch (`sync.rs:255-273`) → `sync_telegram_source` (`276`) → `get_authorized_client` (`287`) → `raw_client` (`288`) → `resolve_and_refresh_peer` (`289-290`) → `resolved_peer.peer` to forum-topic refresh (`292`) and message persistence (`294`). `persist_items` invokes the authorized media helper at line 137.

4. `takeout_import::run_export_dc_spike_for_handle`

   The command wrapper `run_takeout_export_dc_spike` obtains an authorized handle at `takeout_import/mod.rs:390` and forwards it at lines 392-397. The inventoried owner begins at line 401 and calls `raw_client` and `raw_session` at lines 407-408.

5. `takeout_import::run_takeout_migrated_history_import`

   The owner begins at `takeout_import/mod.rs:686`, obtains the authorized handle at line 710, and calls `raw_client`/`raw_session` at lines 711-712. It consumes `ResolvedSyncPeer::peer` at lines 724 and 780.

6. `takeout_import::run_takeout_source_import`

   The owner begins at `takeout_import/mod.rs:944`, obtains the authorized handle at line 966, calls `raw_client`/`raw_session` at lines 967-968, consumes the resolved peer once at line 980, and forwards the complete resolved value through the required three-function chain.

These six owners exactly equal `cp3RawHandleCallsites`.

### Complete Takeout forwarding chains

#### Export-DC spike

`run_takeout_export_dc_spike`
→ `get_authorized_client`
→ handle forwarded to `run_export_dc_spike_for_handle`
→ `raw_client` plus `raw_session`
→ `raw_session` delegates to `raw_memory_session`
→ caller clones the same `Arc<MemorySession>`
→ `prepare_export_dc_alias`
→ client/alias forwarded through Takeout initialization, split-range retrieval, and finish calls at `takeout_import/mod.rs:421-454`.

No handle alias or reacquisition occurs downstream.

#### Migrated-history import

`start_takeout_migrated_history_import`
→ spawned `run_takeout_migrated_history_import_job`
→ `run_takeout_migrated_history_import`
→ authorized handle
→ raw client/session
→ `resolve_and_refresh_peer`
→ first peer read for provenance identity at line 724
→ second peer read forwarded to migrated-chat revalidation at line 780
→ client/session-derived alias forwarded through split loading, count probes, history-range import, page import, history/search calls, provenance wrapper, and Takeout finish.

The whole `ResolvedSyncPeer` is not forwarded beyond this owner. Later functions receive only the raw client reference, export alias, typed input peer, and explicit values.

#### Current-source import

`start_takeout_source_import`
→ spawned `run_takeout_import_job`
→ `run_takeout_source_import`
→ authorized handle
→ raw client/session
→ `resolve_and_refresh_peer`
→ provenance read of `resolved_peer.peer` at line 980
→ complete `ResolvedSyncPeer`, client, alias, and Takeout state forwarded at lines 1032-1046
→ `run_started_takeout_source_import`
→ unchanged forwarding at lines 1112-1126
→ `run_started_takeout_source_import_inner`
→ peer conversion at line 1151
→ validation at line 1169
→ migration detection at line 1186
→ forum-topic refresh at line 1337
→ avatar cache key forwarded separately to `finalize_sync`.

Downstream pagination receives only `&Client`, `ExportDcAlias`, and typed peer/request values. It neither reacquires a `TelegramClientHandle` nor introduces another raw owner. All outer, middle, and inner accesses are correctly attributed to logical owner `run_takeout_source_import`.

### Peer-kind bridge

The exact retained bridge is `USER`, `CHAT`, and `CHANNEL`; it is not a `GROUP` constant:

- Definitions: `telegram_impl/dto.rs:4-6`.
- Package-private staged-root re-export: `telegram_impl/lib.rs:9-11`.
- Sole non-leaf consumer: import at `sources/sync.rs:10-13`, used only by `fallback_message_identity` at lines 188-209:
  - `PeerKind::User` → `TELEGRAM_PEER_KIND_USER`
  - `PeerKind::Chat` → `TELEGRAM_PEER_KIND_CHAT`
  - `PeerKind::Channel` → `TELEGRAM_PEER_KIND_CHANNEL`

DTO validation and DTO-local tests use the leaf constants directly; they do not create another root-bridge consumer.

`TELEGRAM_KIND_GROUP` is a distinct source-subtype constant. `takeout_import::peer_ref_identity` uses existing string literals for provenance and is not another consumer of the staged-root bridge. No `TELEGRAM_PEER_KIND_GROUP` definition or use exists or is required.

### Permanent restricted clone bridge

`telegram_impl/session.rs:179-181` has the exact required definition and body:

```rust
pub(super) fn clone_memory_session(&self) -> Arc<MemorySession> {
    Arc::clone(&self.inner)
}
```

Its sole production call is the intended runtime initialization use at `telegram_impl/runtime.rs:319`:

```rust
let pool = SenderPool::new(session.clone_memory_session(), api_id);
```

The remaining calls are only the authorized pointer-identity assertions:

- `telegram_impl/session.rs:310-311`
- `telegram_impl/runtime.rs:546-547`
- `telegram_impl/runtime.rs:859-860`
- `telegram_impl/runtime.rs:889-890`

All prove `Arc::ptr_eq` against the same `MemorySession`. The method is not root-re-exported, app-consumed, or widened beyond `pub(super)`.

### Required CP3 removals and absences

- All four old leaf files are absent.
- `telegram.rs` no longer declares `mod dto`, `mod media`, `mod runtime`, or `mod session`.
- The old app-telegram future-owner re-export block is gone; permanent consumers now use explicit `crate::telegram_impl` paths.
- App-owned `TelegramState`, commands, credentials, status, diagnostics, and `get_client`/`get_authorized_client` remain in `crate::telegram`.
- `media.rs` no longer re-exports `TelegramMediaPayload` or the two test-only content-kind constants.
- All six existing `sources/items.rs` test references were changed to the unchanged `"text_only"` or `"text_with_media"` literals. No production item behavior changed.
- The old externally visible `initialized_client` form is gone; `initialized_client` is private and reused only by `client`, `authorized_client`, and runtime-local tests.
- The implicit `Client::new` construction is gone.
- `legacy_peer_ref_from_descriptor` has zero Rust occurrences. Its CP4 first-introduction boundary has not been crossed early.
- No raw, media, or peer-kind exception scheduled for later checkpoints has been removed early.
- No future CP4 descriptor bridge, alias, wrapper, visibility, or callsite was introduced early.

### Alias, re-export, wrapper, function-item, glob, and same-name audit

- No reviewed exception is imported with `as`.
- No type alias, wrapper type, callback, function-item capture, UFCS call, macro expansion, `paste!`, or `concat_idents!` obscures a reviewed use.
- Production code contains no relevant glob import or glob re-export.
- Existing `use super::*` occurrences are confined to local test modules and neither expose nor hide an app consumer.
- Explicit re-exports are limited to:
  - the staged-root permanent API;
  - the exact four package-private media compatibility names;
  - the exact three package-private peer-kind constants;
  - the package-private `ResolvedSyncPeer` re-export from `sources/mod.rs`.
- Exact wrappers are:
  - app `get_client`;
  - app `get_authorized_client`;
  - `TelegramClientHandle::raw_session`;
  - the export-spike command-to-handle-owner forwarding pair;
  - `run_started_takeout_source_import`;
  - `run_started_takeout_source_import_inner`.
- Unrelated same-named `peer` occurrences are function parameters, local variables, `dialog.peer()` calls, Grammers/TL request fields, or topic helpers—not alternate `ResolvedSyncPeer::peer` fields.
- `TelegramClientHandle.client`, `TelegramClientHandle.session`, and ordinary `client`/`session` locals are data fields or variables, not aliases for the retained methods.
- `legacy_peer_identity` in `sources/peer_resolution.rs` is an existing metadata-normalization helper and is not `legacy_peer_ref_from_descriptor`.
- Raw parsing helpers and `raw_data` fields are unrelated to the restricted raw-handle bridge.

### Behavior, identity, and ownership findings

- `dto.rs` moved byte-for-byte: old and new Git blob are both `ef757324de36cc8b59c92a5f5bd510bc872a705a`.
- `media.rs` moved byte-for-byte: old and new Git blob are both `38f6958524be7af8c1710e41cd577673bf331ae3`.
- The runtime move differs only by the authorized CP3 changes:
  - production `Arc` import removal/test-only retention;
  - public non-authorizing `client`;
  - private `initialized_client`;
  - fail-closed default configuration and permanent clone bridge;
  - the four required pointer-identity assertion updates;
  - the one new runtime test.
- The session move differs only by the exact `clone_memory_session` method and the authorized `encrypted_session_load_round_trips` pointer-identity update.
- All other modified Rust files contain path/import rewiring, the exact app-media facade reduction, the six test literals, or the `get_client` delegation change. No unrelated production body changed.
- `TelegramRuntime::client` delegates directly to private `initialized_client`; it performs no authorization probe and preserves the missing-account auth error.
- `authorized_client` retains its signature, initialized lookup, authorization probe, unauthenticated error, and ordering.
- `initialize_grammers_client` retains its exact two-parameter signature. It:
  1. constructs `ClientConfiguration::default()`;
  2. checks `auto_cache_peers`;
  3. returns the exact internal error if false;
  4. only then creates `SenderPool`;
  5. only then spawns the runner;
  6. constructs the client only through `Client::with_configuration`.
- There is no configuration parameter, alternate constructor, second sender pool, earlier spawn, or second initialization path.
- Under the pinned true default, client/session identity, authorization probing, error mapping, runner ordering, and account insertion ordering remain unchanged. A future false default fails before a detached runner exists.
- Session encryption, serialization, storage, migration, deletion, and error behavior are unchanged; `telegram_session_store.rs` changed only its import owner.
- The Takeout call order, cancellation behavior, export alias preparation, peer resolution, pagination, provenance writes, finish behavior, forum-topic refresh, and sync finalization are unchanged.
- No Cargo manifest, lockfile, dependency, feature, database, migration, or public error-contract change is present.
- No duplicate DTO, item-kind constant, media payload, runtime, session codec, or test module remains.
- Existing moved test identities were not renamed. The only new foundation identity is `telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check`.
- `node scripts/telegram-8b-test-identities.mjs --check` succeeded, and current Rust source contains no old `telegram::{dto,media,runtime,session}::tests::` prefix. Compiled-count and behavior gates remain separate mandatory evidence and are not replaced by this review.
- The TypeScript contract changes remove automatic raw-use authorization and explicitly tolerate LLM-only alias/visibility mutations; they do not classify any temporary Rust use as permitted or forbidden.

### Findings

- Every active CP3 exception definition is present with the required visibility.
- Every physical use is accounted for.
- The six raw logical owners match the committed CP3 inventory exactly.
- All nine `ResolvedSyncPeer::peer` reads belong to the three expected semantic owners.
- The exact four media names have only the two authorized app consumer areas.
- The peer-kind root bridge has one app consumer and uses the correct `USER/CHAT/CHANNEL` vocabulary.
- The permanent clone bridge has one production consumer and only the authorized tests.
- No unlisted, ambiguous, aliased, newly wrapped, early-added, or early-removed escape-hatch use was found.
- Any subsequent Rust or escape-hatch-authority change invalidates this record and requires another complete independent review.

Superseded escape-hatch review disposition: CLEAN (invalidated by subsequent Rust change)

## CP3 Escape-Hatch LLM Review

## Review identity and independence

- Review date: 2026-07-31.
- Reviewer: fresh independent read-only LLM reviewer. I did not implement Checkpoint 3, did not rely on the invalidated earlier CLEAN disposition, and made no production, test, contract, authority, or status edit. The only write made by this review is this requested scratch record.
- Selected checkpoint: Phase 8B Checkpoint 3.
- Retained predecessor: `951b88b004ba4493a73bd2ccf93a2e8aa31dae6d`.
- Forward-only authority correction and Rust implementation-diff base: `0d91cbd800bd94f6dcbfef48072b227f099cb9a8`.
- Current committed ancestor / current HEAD: `ba0cec533cfa2824f465e1a8e2184362170a92ee`, the separately committed non-Rust analysis-contract compatibility correction.
- Reviewed candidate: the complete current uncommitted CP3 Rust working-tree candidate on `ba0cec53`, including the post-rustfmt `src-tauri/src/lib.rs` module-declaration order.
- The earlier CP3 CLEAN record was treated as stale because a subsequent Rust formatting change invalidated it. This is a complete source-and-inventory re-review, not a fix-delta review.
- The exact current scoped Rust set contains 20 paths: 5 created, 4 deleted, and 11 modified. Their pre-record content fingerprint, calculated from the sorted path plus current Git blob hash or `DELETED`, was `cf891b986887797e483c3cf6dc1c07b6da862a6c36405a4cefdd311def0d7d8b`.

## Authority and inventory inputs

The review read and applied:

- the complete mandatory reusable CP2-CP8 LLM retention gate in `docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md`, including the forward-only post-CP1 amendment, lines 1872-2016, the Checkpoint 3 authority correction, the analysis-gate compatibility correction, and all of Task 3;
- the complete `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`, including the full symbol disposition, active and later transition inventories, ownership/data-flow requirements, and forward-only transitional review authority;
- the complete current `## CP3 Escape-Hatch LLM Review` section in the cumulative verification document, only as candidate context to recheck, not as a reusable verdict;
- the complete committed `src/lib/telegram-8b-symbol-map.json`: schema version 1, all 306 symbol-disposition rows, all six transition inventories, all 67 `restrictedFinalSymbols`, and restricted-bridge fence authority of 2,184 normalized-LF bytes with SHA-256 `87a520b5917ac82c95527210120616e3e5b66141d6492ce46a036d75b2c5ca`;
- the separate, design-only CP3-CP4 three-constant peer-kind staged-root bridge, which is intentionally absent from the schema-v1 artifact.

The active generated CP3 logical-owner inventory is exactly:

1. `sources::store::add_telegram_source`
2. `sources::store::list_telegram_sources`
3. `sources::sync::sync_telegram_source`
4. `takeout_import::run_export_dc_spike_for_handle`
5. `takeout_import::run_takeout_migrated_history_import`
6. `takeout_import::run_takeout_source_import`

The later CP4, CP5, and CP7 arrays were also read completely to detect an early addition, removal, or lifecycle shift. Literal searches and occurrence counts were used only to navigate and cross-check the source. No TypeScript Rust semantic analysis, TypeScript acceptance branch, search result, or occurrence count was used to authorize or reject an escape-hatch use.

## Reviewed Rust scope

Created and read completely:

- `src-tauri/src/telegram_impl/lib.rs`
- `src-tauri/src/telegram_impl/dto.rs`
- `src-tauri/src/telegram_impl/media.rs`
- `src-tauri/src/telegram_impl/runtime.rs`
- `src-tauri/src/telegram_impl/session.rs`

Deleted and compared against the complete retained source:

- `src-tauri/src/telegram/dto.rs`
- `src-tauri/src/telegram/media.rs`
- `src-tauri/src/telegram/runtime.rs`
- `src-tauri/src/telegram/session.rs`

Modified; complete diffs and all affected surrounding definitions/bodies/tests were reviewed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/telegram.rs`
- `src-tauri/src/telegram_session_store.rs`
- `src-tauri/src/media.rs`
- `src-tauri/src/ingest_provenance.rs`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/sources/mod.rs`
- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/takeout_import/mod.rs`
- `src-tauri/src/takeout_import/migrated_history.rs`
- `src-tauri/src/takeout_import/raw_parse.rs`

Unchanged producer/consumer context was read and verified byte-unchanged against `0d91cbd8`:

- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/peer_resolution.rs`

The actual Rust path set equals the literal 20-path Task 3 Rust set with no missing or extra Rust path. DTO and media are exact moves: old/new Git blobs are respectively `ef757324de36cc8b59c92a5f5bd510bc872a705a` and `38f6958524be7af8c1710e41cd577673bf331ae3`.

## Post-rustfmt `lib.rs` order

The current app-root sequence at lines 68-71 is:

```rust
mod telegram;
#[path = "telegram_impl/lib.rs"]
mod telegram_impl;
mod telegram_session_store;
```

This remains the exact required private path override. The reordered declarations are ordinary module items. None of the three modules or the app root uses `#[macro_use]`, `macro_rules!`, a declarative public macro, or another order-dependent macro export. The reorder introduces, removes, or renames no escape-hatch token; changes no module path, visibility, definition, re-export, wrapper, producer, consumer, function item, or forwarding edge; and leaves all physical-use and logical-owner inventories below identical. It therefore has no semantic inventory effect.

## Staged root, permanent surface, and temporary root surface

`telegram_impl/lib.rs` declares exactly four private leaf modules: `dto`, `media`, `runtime`, and `session`. It declares no public module and no glob.

Its permanent curated surface is:

- DTO: `TelegramItemContext`, `TelegramMessageDraft`, `TelegramMessageIdentity`, and `ITEM_KIND_TELEGRAM_MESSAGE`;
- media: `TelegramMediaPayload`;
- runtime: `TelegramApiHash`, `TelegramClientHandle`, `TelegramLoginAttempt`, `TelegramRuntime`, and `TelegramRuntimeStatus`;
- session: `decode_session_json`, `encode_session_json`, `session_json_requires_existing_key`, `SessionEncryptionKey`, and `TelegramSession`.

Its temporary package-private CP3 surface is exactly:

- media: `DocumentSignals`, `derive_content_kind`, `derive_document_media_kind`, and `extract_item_payload`;
- peer kinds: `TELEGRAM_PEER_KIND_CHANNEL`, `TELEGRAM_PEER_KIND_CHAT`, and `TELEGRAM_PEER_KIND_USER`.

There is no public leaf module, wildcard, fifth media compatibility item, fourth peer-kind constant, raw-handle root export, or other staged-root exception.

## Exact media compatibility definitions and physical uses

The active CP3-CP6 media set contains exactly four names.

### `DocumentSignals`

- Definition: `telegram_impl/media.rs:16-22`, `pub(crate)`, with exactly five `pub(crate)` fields: `mime_type`, `has_video`, `has_audio`, `is_voice`, and `is_animated`.
- Staged-root re-export: `telegram_impl/lib.rs:13-15`.
- App compatibility re-export: `media.rs:1-3`.
- Leaf production uses: result/construction in `collect_document_signals` and parameter of `derive_document_media_kind`.
- App raw-parser uses: import in `takeout_import/raw_parse.rs`, construction/default at lines 145-148, field population at lines 155-189.
- Leaf test uses remain local to `telegram_impl/media.rs`.

### `derive_content_kind`

- Definition: `telegram_impl/media.rs:29-35`.
- Re-exported through the staged root and the app media facade.
- Leaf production use: `extract_item_payload`.
- Direct app raw-parser use: `takeout_import/raw_parse.rs:26`.
- Live sync obtains the result only through `extract_item_payload`; it is not another direct helper owner.
- Remaining direct uses are leaf-local tests.

### `derive_document_media_kind`

- Definition: `telegram_impl/media.rs:60-80`.
- Re-exported through the staged root and the app media facade.
- Leaf production use: `extract_document_media_payload`.
- Direct app raw-parser use: `takeout_import/raw_parse.rs:191`.
- Remaining direct uses are leaf-local tests.

### `extract_item_payload`

- Definition: `telegram_impl/media.rs:209-222`.
- Re-exported through the staged root and the app media facade.
- Sole app consumer: import and call in `sources/sync.rs:7,137`.

The exact app consumer split is therefore unchanged:

- `sources::sync` consumes only `extract_item_payload`;
- `takeout_import::raw_parse` consumes `DocumentSignals`, `derive_content_kind`, and `derive_document_media_kind`.

`TelegramMediaPayload` is permanent staged API, not a fifth media compatibility item. Provider-neutral `ItemMediaMetadata` and `media_label` remain outside this exception. The three content-kind constants are not staged-root or app-facade exports.

## Exact raw/session bridge definitions and every physical use

### `TelegramClientHandle::raw_client`

- Definition: `telegram_impl/runtime.rs:45-53`, `pub(crate)`.
- Exactly six call expressions:
  - `sources/store.rs:111`
  - `sources/store.rs:284`
  - `sources/sync.rs:288`
  - `takeout_import/mod.rs:407`
  - `takeout_import/mod.rs:711`
  - `takeout_import/mod.rs:967`

### `TelegramClientHandle::raw_session`

- Definition: `telegram_impl/runtime.rs:55-57`, `pub(crate)`.
- Exact wrapper body: `self.session.raw_memory_session()`.
- Exactly three call expressions:
  - `takeout_import/mod.rs:408`
  - `takeout_import/mod.rs:712`
  - `takeout_import/mod.rs:968`

### `TelegramSession::raw_memory_session`

- Definition: `telegram_impl/session.rs:183-185`, `pub(super)`, retaining the borrowed `&Arc<MemorySession>` signature.
- Exactly two call expressions:
  - `telegram_impl/session.rs:73`, from `memory_session_to_saved`;
  - `telegram_impl/runtime.rs:56`, through `raw_session`.

### App `get_client`

- Definition: `telegram.rs:430-435`, `pub(crate)`.
- Exact body delegates to permanent non-authorizing `TelegramRuntime::client`.
- Exactly two calls: `sources/store.rs:110,283`.

### App `get_authorized_client`

- Definition: `telegram.rs:437-442`, `pub(crate)`.
- Exact body delegates to unchanged `TelegramRuntime::authorized_client`.
- Exactly four calls: `sources/sync.rs:287` and `takeout_import/mod.rs:390,710,966`.

### `ResolvedSyncPeer::peer`

- Definition: unchanged `sources/peer_resolution.rs:95-98`, `pub(crate)`.
- Producer: unchanged `resolve_and_refresh_peer`; `resolve_source_peer_from_typed_identity` produces the `PeerRef`, and constructor shorthand stores it at lines 652-655.
- Type re-export: unchanged `sources/mod.rs:49`.
- Exactly nine field reads:
  - live sync: `sources/sync.rs:292,294`;
  - migrated-history Takeout: `takeout_import/mod.rs:724,780`;
  - current-source Takeout: `takeout_import/mod.rs:980,1151,1169,1186,1337`.

### `legacy_peer_ref_from_descriptor`

- Zero definitions and zero uses, as required at CP3. Its first permitted introduction is CP4; it was not introduced early under an alias or replacement spelling.

There is no seventh raw logical owner, extra raw access, additional `ResolvedSyncPeer::peer` read, or alternative raw producer.

## Logical owners and complete forwarding chains

### 1. `sources::store::list_telegram_sources`

`list_telegram_sources`
-> app `get_client`
-> permanent non-authorizing `TelegramRuntime::client`
-> private `initialized_client`
-> `TelegramClientHandle::raw_client`
-> cloned raw client
-> dialog iteration, peer mapping, and avatar read.

This chain neither probes authorization nor accesses a raw session or `ResolvedSyncPeer`.

### 2. `sources::store::add_telegram_source`

`add_telegram_source`
-> app `get_client`
-> permanent non-authorizing `TelegramRuntime::client`
-> private `initialized_client`
-> `TelegramClientHandle::raw_client`
-> cloned raw client
-> `resolve_telegram_source`
-> existing persistence flow.

There is no raw-session or resolved-sync-peer continuation.

### 3. `sources::sync::sync_telegram_source`

`sync_source`
-> `sync_telegram_source`
-> app `get_authorized_client`
-> `TelegramRuntime::authorized_client`
-> private initialized lookup plus authorization probe
-> `raw_client`
-> `resolve_and_refresh_peer`
-> produced `ResolvedSyncPeer`
-> first `peer` read into forum-topic refresh
-> second `peer` read into `persist_items`
-> message iteration
-> `extract_item_payload`
-> persistence.

The same owner is the sole app consumer of the peer-kind root bridge through `fallback_message_identity`.

### 4. `takeout_import::run_export_dc_spike_for_handle`

`run_takeout_export_dc_spike`
-> app `get_authorized_client`
-> handle forwarded once to `run_export_dc_spike_for_handle`
-> `raw_client` plus `raw_session`
-> `raw_session` delegates to `raw_memory_session`
-> caller clones the same session `Arc`
-> self check
-> `prepare_export_dc_alias`
-> initialized Takeout call
-> split-range call
-> finish call.

The handle is not aliased or reacquired downstream.

### 5. `takeout_import::run_takeout_migrated_history_import`

`start_takeout_migrated_history_import`
-> spawned `run_takeout_migrated_history_import_job`
-> `run_takeout_migrated_history_import`
-> app `get_authorized_client`
-> raw client and raw session
-> session `Arc` clone and export-DC alias
-> `resolve_and_refresh_peer`
-> first `ResolvedSyncPeer::peer` read for provenance identity
-> second peer read into migrated-chat revalidation
-> split selection
-> history count/search and range/page import operations
-> provenance-wrapped invocations
-> Takeout finish.

The whole `ResolvedSyncPeer` does not leave this logical owner. Later helpers receive the client, alias, peer/request values, and explicit state only.

### 6. `takeout_import::run_takeout_source_import`

`start_takeout_source_import`
-> spawned `run_takeout_import_job`
-> `run_takeout_source_import`
-> app `get_authorized_client`
-> raw client and raw session
-> session `Arc` clone and export-DC alias
-> `resolve_and_refresh_peer`
-> first `ResolvedSyncPeer::peer` read for provenance identity at line 980
-> complete `ResolvedSyncPeer`, raw client, alias, Takeout ID, warnings, fallback state, and attempt state forwarded to `run_started_takeout_source_import`
-> unchanged forwarding to `run_started_takeout_source_import_inner`
-> peer conversion at line 1151
-> validation at line 1169
-> migration detection at line 1186
-> forum-topic refresh at line 1337
-> avatar cache key forwarded separately to `finalize_sync`.

The required three-function chain is complete:

```text
run_takeout_source_import
  -> run_started_takeout_source_import
  -> run_started_takeout_source_import_inner
```

Downstream count/page/import helpers receive only `&Client`, `ExportDcAlias`, typed peer/request values, and explicit app-owned state. They do not reacquire a `TelegramClientHandle`, call a raw accessor, or become another logical inventory owner.

These six logical owners exactly equal `cp3RawHandleCallsites`.

## CP3-CP4 peer-kind staged-root bridge

The exact temporary bridge is:

- definitions: `telegram_impl/dto.rs:4-6`;
- package-private staged-root re-export: `telegram_impl/lib.rs:9-11`;
- sole non-leaf/app consumer: explicit import in `sources/sync.rs:10-13`, used only by `fallback_message_identity` at lines 188-209;
- mapping: `PeerKind::User` -> `TELEGRAM_PEER_KIND_USER`, `PeerKind::Chat` -> `TELEGRAM_PEER_KIND_CHAT`, `PeerKind::Channel` -> `TELEGRAM_PEER_KIND_CHANNEL`.

DTO validation and DTO-local tests use the leaf constants directly and do not create another staged-root consumer. `TELEGRAM_KIND_GROUP` is an unrelated application source-subtype constant. No `TELEGRAM_PEER_KIND_GROUP` exists. The bridge remains scheduled for removal with the mapper at CP5 and has not been widened or duplicated.

## Permanent restricted clone bridge

`TelegramSession::clone_memory_session` is defined exactly at `telegram_impl/session.rs:179-181`:

```rust
pub(super) fn clone_memory_session(&self) -> Arc<MemorySession> {
    Arc::clone(&self.inner)
}
```

It has one production call expression, `telegram_impl/runtime.rs:319`, as the `SenderPool::new` session input. It has eight test call expressions forming the four required pointer-identity pairs:

- `telegram_impl/session.rs:310-311`;
- `telegram_impl/runtime.rs:546-547`;
- `telegram_impl/runtime.rs:859-860`;
- `telegram_impl/runtime.rs:889-890`.

Each pair is compared with `Arc::ptr_eq`. The bridge is not root-re-exported, app-consumed, aliased, or widened beyond `pub(super)`. It correctly coexists with the borrowed raw bridge at CP3.

## Required CP3 removals and lifecycle reconciliation

The artifact contains exactly 66 rows whose removal checkpoint is 3:

- 8 old DTO rows;
- 13 old media rows;
- 20 old runtime rows;
- 20 old session rows;
- 4 old `telegram.rs` module-root rows;
- 1 old app-media `TelegramMediaPayload` compatibility row.

All 66 dispositions are satisfied:

- all four `src-tauri/src/telegram/{dto,media,runtime,session}.rs` leaves are absent;
- all four old `telegram.rs` leaf declarations and the old future-owner re-export blocks are absent;
- each moved definition/test has one staged owner; DTO and media are byte-identical moves;
- the app media facade no longer re-exports permanent `TelegramMediaPayload`;
- the app media facade no longer re-exports the two content-kind constants under `#[cfg(test)]`;
- the six `sources/items.rs` test references now use the unchanged `"text_only"` / `"text_with_media"` literals, with no production behavior change;
- old externally visible `initialized_client` is gone; the method is private and is called only by permanent `client`, unchanged `authorized_client`, and runtime-local tests;
- implicit `grammers_client::Client::new` is absent;
- `initialize_grammers_client` retains the exact `(api_id, &TelegramSession)` signature, constructs default `ClientConfiguration`, rejects false `auto_cache_peers` before `SenderPool::new` and `tokio::spawn`, then calls `Client::with_configuration`;
- no old `telegram::{dto,media,runtime,session}::tests::` identity remains and the sole new CP3 identity is `telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check`.

No active exception scheduled for CP5 or CP7 was removed early. No CP4 descriptor, `legacy_peer_ref_from_descriptor`, `ResolvedSyncPeer::descriptor`, live module, new raw wrapper, or additional callsite was introduced early.

## Aliases, re-exports, wrappers, function items, globs, and same-named values

- No reviewed temporary definition or use is renamed with `as`.
- No type alias, alias binding, callback, stored function item, UFCS spelling, wrapper type, trait method, macro expansion, `paste!`, `concat_idents!`, or shadowed binding obscures a reviewed use.
- There is no relevant production glob import or glob re-export. Test-local `use super::*` does not expose or hide an app consumer.
- Temporary re-exports are limited to the exact four media names in staged root and app facade and the exact three peer-kind constants in staged root.
- The package-private `ResolvedSyncPeer` re-export in `sources/mod.rs` is unchanged and forwards the same type, not a duplicate field owner.
- Exact raw/forwarding wrappers are app `get_client`, app `get_authorized_client`, `TelegramClientHandle::raw_session`, `run_takeout_export_dc_spike` -> `run_export_dc_spike_for_handle`, `run_started_takeout_source_import`, and `run_started_takeout_source_import_inner`.
- `TelegramClientHandle.client` and `.session` are ordinary private data fields, not alternate spellings of `raw_client` or `raw_session`.
- Unrelated `peer` names are function parameters, locals, Grammers/TL request fields, `dialog.peer()` results, topic values, or other domain fields. None is a second `ResolvedSyncPeer::peer`.
- `legacy_peer_identity` in unchanged `sources/peer_resolution.rs` is metadata normalization and is not `legacy_peer_ref_from_descriptor`.
- `raw_data`, raw parsing helper names, raw TL values, and serialization fields are unrelated to the transitional handle/session accessors.

## Behavior and ownership findings

- The runtime move differs from the retained leaf only by the authorized CP3 changes: test-only `Arc` import gating, permanent non-authorizing `client`, private `initialized_client`, fail-closed client configuration, permanent clone bridge use, four pointer-identity assertion updates, and one new runtime test.
- The session move differs only by the permanent clone bridge and the authorized encrypted-session pointer-identity assertion.
- Every other modified Rust file contains only explicit owner-path rewiring, exact app-media facade reduction, the six test literals, or app `get_client` delegation. App-owned store and peer-resolution producer bodies are unchanged.
- Non-authorizing `TelegramRuntime::client` delegates directly to private initialized lookup and performs no authorization probe. `authorized_client` retains the initialized lookup, authorization probe, unauthenticated error, and ordering.
- Session serialization, encryption, AAD, storage adapter behavior, migration, deletion, and errors remain unchanged.
- Source/store, live sync, Takeout cancellation, export-DC aliasing, provenance, pagination, durable ordering, finish, forum-topic refresh, and final sync ownership are unchanged.
- No manifest, lockfile, dependency, feature, migration, database schema, public error, IPC, event, command, or transaction-owner change is in the Rust candidate.
- No duplicate DTO, item-kind constant, media payload, runtime, session codec, or moved test module remains.
- The post-rustfmt module order changes none of these findings or any inventory count.

## Read-only verification evidence

The following fresh read-only checks completed with exit code 0 after the source review:

- `node scripts/telegram-8b-symbol-map.mjs --check`;
- `node scripts/telegram-8b-test-identities.mjs --check`;
- `npm.cmd run check:rustfmt`;
- scoped `git diff --check`;
- exact 20-path comparison;
- `git diff --quiet 0d91cbd8 -- src-tauri/src/sources/store.rs src-tauri/src/sources/peer_resolution.rs`.

Rust compilation and behavior suites remain separate mandatory checkpoint gates; this record neither replaces them nor uses TypeScript as escape-hatch authorization.

## Findings

- Active media compatibility definitions: 4; authorized app consumer areas: 2.
- Active raw/session/app-helper/field temporary definitions: 6 present (`raw_client`, `raw_session`, `raw_memory_session`, `get_client`, `get_authorized_client`, `ResolvedSyncPeer::peer`); CP4-only `legacy_peer_ref_from_descriptor`: 0.
- Raw logical owners: exactly 6.
- `raw_client` calls: 6; `raw_session` calls: 3; `raw_memory_session` calls: 2; app `get_client` calls: 2; app `get_authorized_client` calls: 4; `ResolvedSyncPeer::peer` reads: 9.
- Peer-kind root definitions/re-exports: exactly 3; non-leaf/app consumers: exactly 1.
- Permanent clone bridge: 1 definition, 1 production call, and 8 test calls in 4 pointer-identity pairs.
- Required CP3 removal rows: 66 satisfied; old leaf files/declarations: 0 remaining.
- Unlisted, ambiguous, aliased, newly wrapped, early-added, or early-removed escape-hatch findings: 0.

Escape-hatch review verdict: CLEAN
