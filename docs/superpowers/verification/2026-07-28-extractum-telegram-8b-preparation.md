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
