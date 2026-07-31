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

## Superseded CP4 Escape-Hatch LLM Review (invalidated)

# Phase 8B Checkpoint 4 Temporary Escape-Hatch Review (v4)

## Review identity and independence

- Review date: 2026-07-31.
- Selected checkpoint: Phase 8B Checkpoint 4, `Move Peer and Avatar Transport Behind Owned Values`.
- Retained predecessor and current candidate HEAD: `d72d56d7fce6a0545fe89f6934d41d6ae7fa3f9f`.
- Reviewer: fresh independent read-only LLM reviewer. I did not implement the checkpoint, did not reuse the v2 or v3 verdicts, and reviewed v4 from the complete current package and current source. The only write made by this review is this scratch record.
- Review standard: the TypeScript contract, symbol-map generator, generated artifact, literal searches, and counts were navigation/checklist inputs only. They were not treated as semantic Rust escape-hatch authority. Authorization below comes from the complete Rust source and forwarding-path review against the approved plan/design exception sets.

## Candidate and package identity

- Review package: `C:\Users\Dima\AppData\Local\Temp\telegram-8b-task-4-review-package-v4.diff`.
- Package size: 112,989 bytes; 3,166 lines.
- Expected and actual package SHA-256 are identical: `C33C72E23EBA6226133AC92322092DA2C423B94F612A863B46B1B8815EC2BE77`.
- The package declares `d72d56d7..working-tree`; current HEAD is exactly `d72d56d7fce6a0545fe89f6934d41d6ae7fa3f9f`, with no commit after that base.
- The package's complete tracked diff is byte-identical after LF normalization to the current `git diff --unified=10 d72d56d7` for its 11 tracked paths.
- Each complete untracked file in the package is byte-identical after LF normalization to the current file: `telegram_impl/error.rs`, `telegram_impl/live/avatar.rs`, `telegram_impl/live/mod.rs`, and `telegram_impl/live/peer.rs`.
- Before this scratch write, the working tree contained exactly the package's 11 tracked changes and four untracked Rust files. No status document, cumulative verification ledger, plan, design, Cargo manifest/lock, migration, or additional production/test path was changed.

The complete reviewed candidate path set is:

```text
src-tauri/src/sources/avatar.rs
src-tauri/src/sources/identity.rs
src-tauri/src/sources/peer_resolution.rs
src-tauri/src/sources/store.rs
src-tauri/src/sources/sync.rs
src-tauri/src/sources/types.rs
src-tauri/src/takeout_import/mod.rs
src-tauri/src/telegram_impl/dto.rs
src-tauri/src/telegram_impl/error.rs
src-tauri/src/telegram_impl/lib.rs
src-tauri/src/telegram_impl/live/avatar.rs
src-tauri/src/telegram_impl/live/mod.rs
src-tauri/src/telegram_impl/live/peer.rs
src-tauri/src/telegram_impl/runtime.rs
src/lib/telegram-crate-boundary-contract.test.ts
```

## Authority and inventory inputs

I independently read and reconciled:

- the retained Phase 8A public/internal API continuity authority in `docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md`;
- the approved Phase 8B plan's global exception constraints, literal symbol dispositions, complete transition inventories, forward-only CP3 correction, mandatory reusable CP2-CP8 LLM gate, and Task 4 in `docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md`;
- the narrow API/lifecycle authority, complete transitional exception sets, owner arrays, CP3-CP4 peer-kind addition, and forward-only review authority in `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`;
- the complete active CP2 and current valid CP3 records in `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`; the superseded CP3 record was not used as authority;
- `.superpowers/sdd/telegram-8b-task-4-brief.md` and `.superpowers/sdd/telegram-8b-task-4-report.md` completely;
- `scripts/telegram-8b-symbol-map.mjs` completely;
- the complete committed `src/lib/telegram-8b-symbol-map.json`: schema version 1, 306 disposition rows, six transition inventories, and 67 restricted final symbols. Its current SHA-256 is `9CF3489121B78990B1C16209EE8CAF8293F4D8A8ED2E7D52D3F4D7AB00C88FA8`, and `node scripts/telegram-8b-symbol-map.mjs --check` exited 0;
- the full v4 package and enough complete unchanged current Rust context to trace every producer, re-export, physical access, and continuation.

The active generated CP4 owner inventories are exactly:

```text
CP4 raw handle callsites:
  sources::sync::sync_telegram_source
  takeout_import::run_export_dc_spike_for_handle
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

CP4 ResolvedSyncPeer::peer consumers:
  sources::sync::sync_telegram_source
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import
```

The complete design-only CP3-CP4 peer-kind exception remains the three named constants with sole app consumer `sources::sync::fallback_message_identity`.

## Media compatibility facade

The CP3-CP6 compatibility facade remains exactly four items:

1. `DocumentSignals` is defined once in `telegram_impl/media.rs` with `pub(crate)` visibility and exactly five `pub(crate)` fields: `mime_type`, `has_video`, `has_audio`, `is_voice`, and `is_animated`.
2. `derive_content_kind` is defined once in `telegram_impl/media.rs` as `pub(crate)`.
3. `derive_document_media_kind` is defined once in `telegram_impl/media.rs` as `pub(crate)`.
4. `extract_item_payload` is defined once in `telegram_impl/media.rs` as `pub(crate)`.

The staged root re-exports exactly those four package-privately in `telegram_impl/lib.rs`; the app `media.rs` facade re-exports exactly the same four package-privately. There is no alias, glob, wrapper facade, fifth media item, or public module.

Every app physical use is authorized and unchanged from CP3:

- `sources::sync` imports only `extract_item_payload` and calls it once in `persist_items`;
- `takeout_import::raw_parse` imports `DocumentSignals`, `derive_content_kind`, and `derive_document_media_kind`; it constructs/fills the one signal value, calls `derive_content_kind` once, and calls `derive_document_media_kind` once;
- all other occurrences are definition-local implementation or leaf-local tests.

`TelegramMediaPayload`, `ItemMediaMetadata`, `media_label`, and the content-kind constants are separate permanent/provider-neutral values, not additional compatibility items. No media exception was removed early or widened.

## Raw/session/app-helper definitions and physical uses

All active temporary definitions have their exact approved visibility and lifecycle:

| Definition | Current visibility | Physical uses | Result |
| --- | --- | ---: | --- |
| `TelegramClientHandle::raw_client` | `pub(crate)` | 4 calls | exact CP4 inventory |
| `TelegramClientHandle::raw_session` | `pub(crate)` | 3 calls | exact Takeout subset |
| `TelegramSession::raw_memory_session` | `pub(super)` | 2 calls | exact serialization/raw-session bridge |
| `telegram::get_client` | `pub(crate)` | 2 calls | exact two store owned-handle callers |
| `telegram::get_authorized_client` | `pub(crate)` | 4 calls | sync plus three Takeout workflows |
| `ResolvedSyncPeer::peer` | `pub(crate)` field | 9 reads | exact three CP4 logical consumers |
| `legacy_peer_ref_from_descriptor` | `pub(crate)` | 1 call | sole transitional field builder |

The four `raw_client` calls are exactly:

- `sources::sync::sync_telegram_source`;
- `takeout_import::run_export_dc_spike_for_handle`;
- `takeout_import::run_takeout_migrated_history_import`;
- `takeout_import::run_takeout_source_import`.

The three `raw_session` calls occur in those three Takeout owners only. Its body is exactly the direct `self.session.raw_memory_session()` wrapper. The borrowed session bridge has exactly two calls: `session::memory_session_to_saved` and that `raw_session` wrapper. No raw-session call occurs in sync, store, peer resolution, or a new helper.

`get_client` delegates only to permanent non-authorizing `TelegramRuntime::client`; its two callers are `sources::store::{list_telegram_sources,add_telegram_source}`, which now use owned handle operations and contain no raw-client access. `get_authorized_client` delegates only to `TelegramRuntime::authorized_client`; its four calls feed exactly the sync and three Takeout logical owners above.

`ResolvedSyncPeer::peer` has one definition and one producer. The nine reads are exactly:

- two in `sources::sync::sync_telegram_source`, for topic refresh and message persistence;
- two in `takeout_import::run_takeout_migrated_history_import`, for provenance identity and migrated-chat revalidation;
- five under `takeout_import::run_takeout_source_import`: one provenance read in the owner and four reads after the required forwarding chain, for input-peer conversion, peer validation, migration detection, and post-Takeout forum-topic refresh.

`legacy_peer_ref_from_descriptor` is introduced exactly at CP4. It is the sole app-side Grammers peer builder, and `resolve_and_refresh_peer` calls it exactly once to fill `ResolvedSyncPeer::peer`. The staged `live::peer::peer_ref_from_descriptor` is private inside the staged implementation and serves the approved owned `peer_avatar_bytes` operation; it is not root-re-exported or app-visible and is not a second temporary boundary hatch.

The permanent `TelegramSession::clone_memory_session` bridge remains separate from the deletion set: one `pub(super)` definition, one production call in client initialization, and eight test calls forming four pointer-identity pairs. It is not root-re-exported or app-consumed.

## Logical owners and complete forwarding chains

### `sources::sync::sync_telegram_source`

```text
sync_source
  -> sync_telegram_source
  -> telegram::get_authorized_client
  -> TelegramRuntime::authorized_client
  -> TelegramClientHandle::raw_client (one call)
  -> resolve_and_refresh_peer(handle)
     -> typed owned resolution plan
     -> PeerDescriptor
     -> legacy_peer_ref_from_descriptor (one call)
     -> ResolvedSyncPeer { peer, descriptor, refreshed_avatar_cache_key }
  -> ResolvedSyncPeer::peer -> refresh_forum_topics
  -> ResolvedSyncPeer::peer -> persist_items
     -> extract_item_payload
     -> fallback_message_identity
```

This owner has no raw-session call. It is also the sole app consumer of the peer-kind staged-root bridge through `fallback_message_identity`.

### `takeout_import::run_export_dc_spike_for_handle`

```text
run_takeout_export_dc_spike
  -> telegram::get_authorized_client
  -> handle forwarded once to run_export_dc_spike_for_handle
  -> raw_client (one call)
  -> raw_session (one call)
     -> raw_memory_session
  -> cloned client/session
  -> self check, export-DC alias, Takeout init/splits/finish
```

The handle is not aliased, reacquired, or forwarded beyond this named owner.

### `takeout_import::run_takeout_migrated_history_import`

```text
start_takeout_migrated_history_import
  -> spawned run_takeout_migrated_history_import_job
  -> run_takeout_migrated_history_import
  -> telegram::get_authorized_client
  -> raw_client + raw_session (one call each)
  -> resolve_and_refresh_peer(handle)
  -> ResolvedSyncPeer::peer -> provenance identity
  -> ResolvedSyncPeer::peer -> revalidate_migrated_from_chat_id
  -> split/count/page import operations
  -> Takeout finish
```

The complete `ResolvedSyncPeer` does not leave this logical owner, and downstream operations receive only already-produced typed/raw values.

### `takeout_import::run_takeout_source_import`

```text
start_takeout_source_import
  -> spawned run_takeout_import_job
  -> run_takeout_source_import
     -> telegram::get_authorized_client
     -> raw_client + raw_session (one call each)
     -> resolve_and_refresh_peer(handle)
     -> ResolvedSyncPeer::peer -> provenance identity
     -> run_started_takeout_source_import
        -> run_started_takeout_source_import_inner
           -> ResolvedSyncPeer::peer -> InputPeer conversion
           -> ResolvedSyncPeer::peer -> validate_takeout_peer
           -> ResolvedSyncPeer::peer -> detect_supergroup_migration
           -> count/page/import continuations
           -> ResolvedSyncPeer::peer -> completed-Takeout forum-topic refresh
           -> refreshed avatar cache key -> finalize_sync
```

Both intermediate functions forward the already-produced `ResolvedSyncPeer` and explicit state; neither acquires a handle, calls a raw accessor, defines a wrapper, or becomes another logical owner.

These four raw owners and three peer consumers exactly equal both active CP4 arrays. There is no fifth raw owner, fourth peer consumer, second peer producer, or unaccounted continuation.

## CP3-CP4 peer-kind staged-root bridge

- Definitions: exactly `TELEGRAM_PEER_KIND_CHANNEL`, `TELEGRAM_PEER_KIND_CHAT`, and `TELEGRAM_PEER_KIND_USER` in `telegram_impl/dto.rs`, each `pub(crate)`.
- Staged-root re-export: exactly those three in `telegram_impl/lib.rs`, package-private.
- Sole non-leaf/app consumer: the explicit import in `sources/sync.rs`, used only by `fallback_message_identity` to map `PeerKind::{Channel,Chat,User}`.
- DTO validation and DTO-local tests use the leaf constants directly and do not add another staged-root consumer.
- No fourth peer-kind constant, `TELEGRAM_PEER_KIND_GROUP`, alias, duplicate mapper, glob, or additional root consumer exists.
- The bridge remains scheduled for deletion at CP5; it was neither removed early nor widened.

## Required CP4 removals and lifecycle reconciliation

- The two CP3 store `raw_client` calls in `list_telegram_sources` and `add_telegram_source` are removed. Both commands retain their authorized `get_client` call but now use owned `dialog_listing`, `resolve_username`, and `resolve_dialog_peer` operations.
- Raw Grammers types and operations are removed from `sources/avatar.rs`, `sources/identity.rs`, and the dialog-listing portion of `sources/store.rs`.
- The former app peer builders are removed/replaced by the one authorized `legacy_peer_ref_from_descriptor`; staged reconstruction is private to `telegram_impl/live/peer.rs`.
- `ResolvedSyncPeer::peer` remains with its exact three CP4 logical consumers, while the approved owned `descriptor` field is added alongside it.
- No CP5 removal occurred early: the sync raw owner, sync peer consumer, four-item media facade, and three-constant peer-kind bridge all remain.
- No CP7 deletion occurred early: both handle accessors, the borrowed session accessor, both app lookup helpers, `ResolvedSyncPeer::peer`, and the legacy builder all remain for their declared lifecycle.
- No temporary definition or physical use that should disappear at CP4 remains beyond the approved CP4 sets.

## Alias, re-export, wrapper, function-item, macro, and same-name audit

- No temporary symbol is renamed with `as`, type-aliased, invoked through UFCS, stored or passed as a function item, callback, trait method, or alternate callable.
- The only `as` imports in relevant source are unrelated utility imports such as `Engine as _` and `Duration as StdDuration`.
- There is no production glob import/re-export involving an exception. Test-local `use super::*` imports are lexical test conveniences, not re-exports, and introduce no extra physical exception call.
- No `macro_rules!`, `paste!`, `concat_idents!`, generated identifier, or public macro synthesizes an exception spelling.
- Temporary re-exports are limited to the exact four media names in the staged root and app media facade and the exact three peer-kind constants in the staged root. No raw accessor, raw type, or legacy builder is root-re-exported.
- The package-private `ResolvedSyncPeer` re-export in `sources/mod.rs` forwards the one exact type for the approved Takeout chain; it does not duplicate or hide the `peer` field.
- Exact raw/forwarding wrappers are limited to app `get_client`, app `get_authorized_client`, `TelegramClientHandle::raw_session`, command-to-`run_export_dc_spike_for_handle`, and the two required started-source-import forwarding functions.
- The private `TypedPeerResolutionBackend`, listing backend, and username/dialog test seams forward owned `PeerDescriptor` operations only; they do not expose or acquire raw client/session values and are the Task 4-authorized private fakes.
- `TelegramClientHandle.client`, `.session`, `TelegramSession.inner`, and `ResolvedSyncPeer.descriptor` are ordinary private/owned backing fields, not alternate raw accessors.
- Other `peer` fields/parameters are Grammers/TL request fields or already-forwarded peer values, not additional `ResolvedSyncPeer::peer` definitions or producers. `legacy_peer_identity`, raw parser names, and `raw_data` are unrelated to the transitional legacy/raw bridge.
- The staged tree introduces no application alias/import, public module, public glob, or externally reachable raw type. Its raw Grammers parameters/results are confined to private implementation or the approved final restricted `pub(super)` live/avatar facades.

## Findings by severity

- Critical: none.
- High: none.
- Medium: none.
- Low: none.
- Informational: the candidate has exactly the approved CP4 temporary exception definitions, physical uses, owners, consumers, visibilities, and lifecycle. The v4-only deterministic test-runtime adjustments do not add or alter an escape hatch.

No unlisted, ambiguous, aliased, newly wrapped, extra, early-added, or early-removed temporary escape-hatch definition or physical use was found. The current frozen candidate satisfies the mandatory semantic escape-hatch gate and is eligible to proceed to the remaining CP4 candidate-status and retained checkpoint gates.

Superseded escape-hatch review disposition: CLEAN (invalidated by subsequent Rust change)

## CP4 Escape-Hatch LLM Review

# Phase 8B Checkpoint 4 Temporary Escape-Hatch Review (v6)

## Review identity and package

- Review date: 2026-07-31.
- Selected checkpoint: Phase 8B Checkpoint 4, `Move Peer and Avatar Transport Behind Owned Values`.
- Retained predecessor and current HEAD: `d72d56d7fce6a0545fe89f6934d41d6ae7fa3f9f`.
- Reviewer: fresh independent read-only LLM reviewer. I did not implement the candidate. The interrupted v5 audit was discarded without a verdict or scratch file, and the superseded v4 CLEAN record was treated only as invalidated history.
- Candidate status remains Checkpoint 3 in both roadmap and design. No candidate status, production, test, plan, design, ledger, Cargo, staging, or commit file was written by this review.
- Package: `C:\Users\Dima\AppData\Local\Temp\telegram-8b-task-4-review-package-v6.diff`.
- Required and actual package size: 142,884 bytes. Required and actual SHA-256: `0275A9BDB4838DA6089B6521C23FE11006351CFBA26F9FD0239C31E56857F4CD`. The package has 3,599 lines.
- With explicit UTF-8 decoding and LF normalization, every one of the package's 2,680 tracked diff lines is identical to the current `git diff --unified=10 d72d56d7` for its 14 tracked paths. All four complete untracked file bodies are identical to the current files: `telegram_impl/error.rs`, `telegram_impl/live/avatar.rs`, `telegram_impl/live/mod.rs`, and `telegram_impl/live/peer.rs`.

## Authority and review inputs

I independently read and reconciled the approved Phase 8B plan's global exception rules, complete transition inventories, forward-only CP3 correction, mandatory reusable CP2-CP8 LLM gate, owner amendment, and Task 4; the approved Phase 8 boundary design's complete exception sets and forward-only review authority; the cumulative verification ledger including the invalidated CP4 v4 record; the complete Task 4 brief and current report; the complete symbol-map generator; the generated schema-v1 inventories; the complete v6 package; and current Rust definitions, surrounding implementations, owners, consumers, producers, and forwarding paths.

The current Task 4 brief, explicit owner direction, approved plan amendment, and current report consistently require that no TypeScript Rust-function-body parser be retained; there is no brief drift affecting Rust escape-hatch authority.

The generated artifact is current and internally valid:

- schema version 1;
- 306 symbol-disposition entries;
- 67 restricted final symbols;
- SHA-256 `9CF3489121B78990B1C16209EE8CAF8293F4D8A8ED2E7D52D3F4D7AB00C88FA8`;
- `node scripts/telegram-8b-symbol-map.mjs --check` exited 0.

Its active CP4 logical-owner checklists are exactly:

```text
raw handle owners:
  sources::sync::sync_telegram_source
  takeout_import::run_export_dc_spike_for_handle
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

ResolvedSyncPeer::peer consumers:
  sources::sync::sync_telegram_source
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import
```

These arrays and the TypeScript contracts were navigation/checklist inputs only. The authorization findings below come from the Rust definitions, types, visibility, bodies, and complete forwarding paths.

## Four-item media compatibility facade

The CP3-CP6 compatibility exception remains exactly four definitions in `telegram_impl/media.rs`:

1. `DocumentSignals`, `pub(crate)`, with exactly five `pub(crate)` fields: `mime_type`, `has_video`, `has_audio`, `is_voice`, and `is_animated`.
2. `derive_content_kind`, `pub(crate)`.
3. `derive_document_media_kind`, `pub(crate)`.
4. `extract_item_payload`, `pub(crate)`.

`telegram_impl/lib.rs` package-privately re-exports exactly those four. App `src-tauri/src/media.rs` package-privately re-exports exactly the same four. There is no alias, glob, wrapper facade, public leaf module, or fifth item.

The authorized app physical uses are exact:

- `sources::sync` imports only `extract_item_payload` and calls it once from `persist_items`;
- `takeout_import::raw_parse` imports `DocumentSignals`, `derive_content_kind`, and `derive_document_media_kind`, constructs the one signal value, and calls each function once in its parsing flow;
- remaining occurrences are definition-local implementation or leaf-local tests.

`TelegramMediaPayload`, provider-neutral media metadata APIs, media labels, and content-kind constants are separate permanent APIs, not additional compatibility items. No media item was removed early or widened.

## Transitional raw/session/app bridge

The active definitions, visibility, and physical calls are:

| Definition | Visibility | Physical calls | Accounted use |
| --- | --- | ---: | --- |
| `TelegramClientHandle::raw_client` | `pub(crate)` | 4 | exact four CP4 raw owners |
| `TelegramClientHandle::raw_session` | `pub(crate)` | 3 | exact Takeout subset |
| `TelegramSession::raw_memory_session` | `pub(super)` | 2 | serialization and `raw_session` wrapper |
| `telegram::get_client` | `pub(crate)` | 2 | exact store handle consumers |
| `telegram::get_authorized_client` | `pub(crate)` | 4 | sync plus three Takeout owners |
| `ResolvedSyncPeer::peer` | `pub(crate)` field | 9 reads | exact three CP4 logical consumers |
| `legacy_peer_ref_from_descriptor` | `pub(crate)` | 1 call | sole transitional field producer |

The four `raw_client` calls are one each in `sources::sync::sync_telegram_source`, `takeout_import::run_export_dc_spike_for_handle`, `takeout_import::run_takeout_migrated_history_import`, and `takeout_import::run_takeout_source_import`. The three `raw_session` calls are in the three Takeout owners. Its body delegates directly to `self.session.raw_memory_session()`. The borrowed session accessor's other call is `memory_session_to_saved`.

`get_client` delegates only to `TelegramRuntime::client`. Its two consumers, `sources::store::{list_telegram_sources,add_telegram_source}`, now call owned-value handle methods and do not call a raw accessor. `get_authorized_client` delegates only to `TelegramRuntime::authorized_client`, and its four calls feed exactly the four raw owners.

`ResolvedSyncPeer::peer` has one field definition and one producer. Its nine reads are:

- two in `sources::sync::sync_telegram_source`, for topic refresh and item persistence;
- two in `takeout_import::run_takeout_migrated_history_import`, for provenance identity and migrated-chat revalidation;
- five under `takeout_import::run_takeout_source_import`: one provenance read in the owner and four after forwarding, for input-peer conversion, validation, migration detection, and completed-Takeout forum-topic refresh.

`legacy_peer_ref_from_descriptor` is the sole app-side Grammers builder. `resolve_and_refresh_peer` calls it once and constructs the sole `ResolvedSyncPeer` producer with the resulting raw `peer`, the owned `descriptor`, and the cache result. The separate staged `live::peer::peer_ref_from_descriptor` is private, is used only inside the owned `peer_avatar_bytes` implementation, and is neither app-visible nor root-re-exported.

The permanent `TelegramSession::clone_memory_session` remains outside the transitional deletion set: one `pub(super)` definition, one production call during client initialization, and eight test calls in four pointer-identity pairs. It is not app-consumed or root-re-exported.

## Logical owners and complete forwarding paths

### Sync

```text
sync_source
  -> sync_telegram_source
     -> get_authorized_client -> authorized_client
     -> raw_client (one call)
     -> resolve_and_refresh_peer(opaque handle)
        -> private owned-value typed-resolution adapter
        -> PeerDescriptor
        -> legacy_peer_ref_from_descriptor (one call)
        -> ResolvedSyncPeer
     -> ResolvedSyncPeer::peer -> refresh_forum_topics
     -> ResolvedSyncPeer::peer -> persist_items
        -> extract_item_payload
        -> fallback_message_identity
```

This is the sole sync raw owner and peer consumer, and the sole app consumer of the peer-kind root bridge.

### Export-DC spike

```text
run_takeout_export_dc_spike
  -> get_authorized_client
  -> run_export_dc_spike_for_handle
     -> raw_client + raw_session
        -> raw_memory_session
     -> cloned raw values used by self-check/export-DC/Takeout operations
```

The handle leaves no additional owner or helper.

### Migrated history

```text
start_takeout_migrated_history_import
  -> run_takeout_migrated_history_import_job
  -> run_takeout_migrated_history_import
     -> get_authorized_client
     -> raw_client + raw_session
     -> resolve_and_refresh_peer(opaque handle)
     -> peer read for provenance
     -> peer read for revalidate_migrated_from_chat_id
     -> remaining Takeout operations use already-produced values
```

### Current source Takeout

```text
start_takeout_source_import
  -> run_takeout_import_job
  -> run_takeout_source_import
     -> get_authorized_client
     -> raw_client + raw_session
     -> resolve_and_refresh_peer(opaque handle)
     -> peer read for provenance
     -> run_started_takeout_source_import
        -> run_started_takeout_source_import_inner
           -> peer read for InputPeer conversion
           -> peer read for validate_takeout_peer
           -> peer read for detect_supergroup_migration
           -> import continuations
           -> peer read for completed-Takeout forum-topic refresh
           -> cache key to finalize_sync
```

Both started-import functions merely forward the already-produced `ResolvedSyncPeer` and explicit state. Neither acquires a handle, calls a raw accessor, defines another wrapper, or creates another logical owner.

## Private live typed-resolution adapter

`TypedPeerResolutionBackend`, `LiveTypedPeerResolutionBackend`, both trait methods, the tuple field, and the implementation are private to `sources::peer_resolution`.

The adapter's complete state is `&TelegramClientHandle`. It cannot access `TelegramClientHandle.client` or `.session`, because those fields are private to `telegram_impl::runtime`. Its two methods call only the permanent public owned-value operations `TelegramClientHandle::resolve_username` and `TelegramClientHandle::resolve_dialog_peer`; they accept strings/integers and return only owned `PeerDescriptor` values or application errors. It has no raw-client/session accessor call, no `PeerRef` parameter/result, no raw type, no trait implementation on the public handle, no root re-export, no app facade, and no path by which the handle or its backing fields escape.

`resolve_source_peer_from_typed_identity` creates the adapter as a local variable and passes it only to the private generic coordinator. The coordinator returns `(PeerDescriptor, bool)`. The test backend stores event strings and returns owned descriptors/errors. Neither backend changes the authorized raw-owner or field-consumer arrays.

Therefore the new live adapter is a private owned-value test seam, not an alternate raw/client/session escape hatch and not a replacement spelling for one.

## CP3-CP4 peer-kind root bridge

- Exactly three `pub(crate)` definitions exist in `telegram_impl/dto.rs`: `TELEGRAM_PEER_KIND_CHANNEL`, `TELEGRAM_PEER_KIND_CHAT`, and `TELEGRAM_PEER_KIND_USER`.
- `telegram_impl/lib.rs` package-privately re-exports exactly those three.
- `sources::sync::fallback_message_identity` is the sole app/root consumer and maps `PeerKind::{Channel,Chat,User}`.
- DTO-local validation/tests use the leaf constants directly and do not add another root consumer.
- There is no fourth constant, alias, duplicate app mapper, glob, or additional root consumer.
- The bridge remains present for CP4 and scheduled for removal at CP5.

## Required CP4 removals and later lifecycle

- The two CP3 store `raw_client` calls are gone. Store retains only its two authorized `get_client` calls and owned `dialog_listing`, `resolve_username`, and `resolve_dialog_peer` operations.
- Raw Grammers imports/operations are absent from `sources/avatar.rs`, `sources/identity.rs`, and `sources/store.rs`.
- The former app peer builders were removed; the one authorized transitional legacy builder replaces them.
- `ResolvedSyncPeer::peer` remains with exactly the three CP4 consumers and now has the required owned descriptor alongside it.
- No CP5 removal occurred early: the sync raw owner/peer consumer, four media items, and three peer-kind constants remain.
- No CP7 removal occurred early: both handle raw accessors, the borrowed session accessor, both app lookup helpers, the transitional peer field, and the legacy builder remain.
- At CP5 the peer-kind bridge and sync raw/peer consumer disappear. At CP6 the CP5 raw/peer arrays and four media items carry forward unless a reviewed diff removes a listed use. At CP7 all transitional raw definitions/calls, the peer field/builder, and the media facade are removed/demoted as specified; the permanent clone bridge remains.

## Alias, re-export, wrapper, function-item, macro, and same-name audit

- No temporary name is imported or re-exported with `as`, type-aliased, invoked through UFCS, stored/passed as a function item, installed as a callback/trait method, or shadowed by an alternate callable.
- There is no exception-related production glob, public macro, `macro_rules!`, generated identifier, or public leaf module.
- Temporary re-exports are limited to the exact four media names in the staged root and app media facade and the exact three peer-kind constants in the staged root. No raw accessor, raw type, or legacy builder is root-re-exported.
- The package-private `sources::ResolvedSyncPeer` re-export forwards the one exact type for the required Takeout chain; it does not duplicate or hide the field.
- The only raw/handle wrappers are the declared app lookup helpers, `raw_session`, and command/continuation forwarding paths accounted above.
- `DialogListing` is the permanent opaque public iterator value: its raw client and dialog iterator are private fields in a private backend enum, and its public `next` returns only owned descriptors.
- Private staged live backends may hold raw values inside `telegram_impl/live`; they are implementation details behind the approved owned API and neither app-visible nor root-re-exported.
- Other fields/parameters named `peer`, `client`, `session`, `raw`, or `legacy` are raw-library internals or unrelated app values, not additional transitional field definitions, accessors, or producers.

## Findings

- Critical: none.
- High: none.
- Medium: none.
- Low: none.
- Unlisted, ambiguous, aliased, newly wrapped, extra, early-added, or early-removed temporary escape-hatch definitions/uses: none.

The frozen v6 candidate contains exactly the CP4-authorized temporary exception definitions, physical uses, logical owners, forwarding chains, visibility, and lifecycle. The isolated TypeScript historical CP3 facade fixture changes no Rust authority or escape-hatch semantics.

Escape-hatch review verdict: CLEAN

## CP5 Escape-Hatch LLM Review

# Phase 8B Checkpoint 5 Temporary Escape-Hatch Review

## Review identity and frozen candidate

- Review date/time: 2026-07-31T21:01:25.8951569+03:00.
- Selected checkpoint: Phase 8B Checkpoint 5, Task 5, `Move Live History and Topic Fetching Behind Owned Batch APIs`.
- Retained predecessor and current HEAD: `1acc4f618dcc5855b677c844624f21c67c371b47` (`refactor: stage Telegram peer and avatar boundary`).
- Reviewer: fresh independent read-only LLM reviewer. I did not implement this candidate and did not rely on the implementation report for authorization; I independently read the complete current Rust diff, complete new Rust files, unchanged producer/forwarding context, pinned Grammers implementation, approved plan/design, generated authority, and cumulative ledger.
- The retained status pair remains Checkpoint 4: the roadmap is unchanged from the predecessor and the design header remains `Approved; 8B preparation Checkpoint 4 retained`. The cumulative ledger is unchanged at SHA-256 `75c749adf4361116fcc5c315bfd3426e9cc8f885bd466ab2b708e77bb498b2f8`.
- This review wrote only this scratch record. It did not edit Rust, TypeScript, status, plan, design, symbol authority, ledger, staging, or commits.

The complete scoped Rust candidate consists of 13 tracked modifications and two complete untracked files. The tracked Rust diff is 573 insertions and 407 deletions; the new files contain 1,115 and 365 lines. I read every changed hunk, both complete new files, and the relevant unchanged definitions/callers/producers. The exact reviewed path list is:

```text
src-tauri/src/sources/items.rs
src-tauri/src/sources/sync.rs
src-tauri/src/sources/topics.rs
src-tauri/src/takeout_import/forum_topics.rs
src-tauri/src/takeout_import/mod.rs
src-tauri/src/telegram_impl/dto.rs
src-tauri/src/telegram_impl/error.rs
src-tauri/src/telegram_impl/lib.rs
src-tauri/src/telegram_impl/live/messages.rs
src-tauri/src/telegram_impl/live/mod.rs
src-tauri/src/telegram_impl/live/peer.rs
src-tauri/src/telegram_impl/live/topics.rs
src-tauri/src/telegram_impl/media.rs
src-tauri/src/telegram_impl/runtime.rs
src-tauri/src/telegram_impl/session.rs
```

The candidate was frozen by hashing the UTF-8 text `path|byte-length|sha256` for those 15 paths in the order above, joined with LF and without a final LF. Its aggregate SHA-256 is `d21577ed80d8993bc52a574e30800ecf086cffd0519044dc5812d0702b8351e5`. The constituent rows are:

```text
src-tauri/src/sources/items.rs|68218|5be8b96e92e24739a385243b9688dbbe4202b7a06e6629c0ad7bb8f40b11b286
src-tauri/src/sources/sync.rs|26884|694aa31cd92fd515c5b2719a38f56761d06669e6c5f956f71e6a6259df4a8969
src-tauri/src/sources/topics.rs|23264|d537625c03303d57b6e61fc515b6587f9dcfd1da63873ad68edf7d4243cf18cc
src-tauri/src/takeout_import/forum_topics.rs|8472|db71c36e57ccaaebdaa7b7ba082d9b8e756116760314a2e04b6e9480f33ad3f2
src-tauri/src/takeout_import/mod.rs|95248|1547ae4c0b54c0e7a8a1ce94283785bc5b751aceaf7e5ddc6883442cf184da3c
src-tauri/src/telegram_impl/dto.rs|9404|0a9caed44e37496c0a8f336048dbc7a548737e2213381c34901a59b44dc76c4b
src-tauri/src/telegram_impl/error.rs|660|0311ea20d5ae815d78960f76e7b98181b99c36886ee4a3776ad65a3ea453a305
src-tauri/src/telegram_impl/lib.rs|747|5a14b5b2bbd6e0397e2985bae29c232b9ba166bb1131c258cd77216035a0a4d2
src-tauri/src/telegram_impl/live/messages.rs|36346|927d2e927d43fc79653f800329938401b8b51de416ca00ac991393be9c9c61b1
src-tauri/src/telegram_impl/live/mod.rs|1793|c62179eb196f13a7a616bd5b25b1787bba1bd8bf3068a51973722876b524bc5a
src-tauri/src/telegram_impl/live/peer.rs|25356|62228ab09ffab1f2d03ded8a878baef9566a7b0b6d9df1005a76304c89168de8
src-tauri/src/telegram_impl/live/topics.rs|11824|bf4e4d964c04c0c722f048023f7285e9a967c5b6b4d7fa169f7dccf11328b316
src-tauri/src/telegram_impl/media.rs|9752|3773641120e18010a77c962deb048784d75f7878c67564d8d80eb710c6bb79c4
src-tauri/src/telegram_impl/runtime.rs|53648|8827953e85c7376759055e2af7b62c31cb030f15c8efa674d571c6362447e4f6
src-tauri/src/telegram_impl/session.rs|12692|569117e1e7ce31e4c51331d2a8258b28cc10445436e6a7ae606ce8c1a4e8456b
```

Any subsequent edit to any reviewed Rust path invalidates this record and requires the full fresh review required by the reusable gate.

## Authority and complete inventory inputs

I independently read and reconciled:

- the approved plan's complete global constraints, forward-only no-TypeScript-Rust-analysis amendment, complete mandatory reusable CP2-CP8 LLM gate, complete transition inventories, public/restricted fences, symbol-disposition table, lifecycle rules, Rust verification loops, and all of Task 5;
- the approved design's complete boundary, lifecycle, owned-value API, live-history/topic semantics, exception sets, 69-name final restricted fence, symbol map, and acceptance criteria;
- the complete cumulative verification ledger, including the retained CP2, corrected CP3, and valid CP4 v6 review records;
- the complete Task 5 brief and report as navigation/evidence claims only;
- all 307 expanded symbol-disposition entries, all transition arrays, and all 69 final restricted symbols in `src/lib/telegram-8b-symbol-map.json`;
- the complete `scripts/telegram-8b-symbol-map.mjs` generator, which reads plan/document structure and does not parse Rust bodies or authorize Rust use-sites;
- the complete scoped Rust candidate plus unchanged `sources/peer_resolution.rs`, `telegram.rs`, store callers, session serialization, and all three Takeout owner chains;
- pinned Grammers revision `1f901ce6e973fdcf0e74267f3d8efad5c729daaa`, specifically `MessageIter::fill_buffer`, `Client::build_peer_map`, raw-message owned mapping, `Peer::auth`, `PeerInfo` conversion/auth, and the default `ClientConfiguration`.

Authority input hashes at review time were:

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md` | 210073 | `5c8533c5a7b4e7768659f9fbbc1a633ecdbd3c45d1e2ed3e2d5830a8a22880b5` |
| `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md` | 167172 | `ca242075914936613814cc7de85f75591c6a3025ffd6c662e9186056ed0126c4` |
| cumulative verification ledger | 107340 | `75c749adf4361116fcc5c315bfd3426e9cc8f885bd466ab2b708e77bb498b2f8` |
| `scripts/telegram-8b-symbol-map.mjs` | 23161 | `2a801cd645d9074f22149319ff3243dc3b5d2d106ce90aa4a9170a9020a58be7` |
| `src/lib/telegram-8b-symbol-map.json` | 123717 | `4fdfac6ca545b3ac5eb7dd09d7cfb68d5e7341a4476f938efe63186bc9bd96f0` |
| `src/lib/telegram-crate-boundary-contract.test.ts` | 346305 | `c1da5bc38e13f7373c94ad88a908c0526a8f487e3e793bbd49315511b7533213` |
| Task 5 brief | 9154 | `917993506b897a98b28122dca82dc4d1bb68e02cac2940a38fb72a1029108261` |
| Task 5 implementation report | 10422 | `62fecd892df2028fec113ba19f9ff3da0d4cb90002680d0b161bbf45e6dc5ca9` |

The artifact is schema version 1, contains 307 unique expanded disposition entries, contains 69 sorted unique restricted final symbols, and has restricted-fence authority `{ normalizedLfBytes: 2246, sha256: b7d7d4028fc653aa1efb217f043f9a4ab96778e0e38c775703507ce9b81f5fa1 }`. `node scripts/telegram-8b-symbol-map.mjs --check` exited 0.

The complete transition inventories read from the artifact were:

```text
CP3 raw-handle owners:
  sources::store::add_telegram_source
  sources::store::list_telegram_sources
  sources::sync::sync_telegram_source
  takeout_import::run_export_dc_spike_for_handle
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

CP4 raw-handle owners:
  sources::sync::sync_telegram_source
  takeout_import::run_export_dc_spike_for_handle
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

CP4 ResolvedSyncPeer::peer consumers:
  sources::sync::sync_telegram_source
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

CP5 raw-handle owners:
  takeout_import::run_export_dc_spike_for_handle
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

CP5 ResolvedSyncPeer::peer consumers:
  takeout_import::run_takeout_migrated_history_import
  takeout_import::run_takeout_source_import

CP7 raw bridge symbols/callsites:
  empty
```

I treated those lists only as navigation/checklists. The authorization conclusions below come from the actual Rust definitions, visibility, signatures, bodies, physical calls, producers, and complete data/forwarding paths.

The complete 69-name final restricted fence was also read. At CP5, its materialized live/error/session leaf and parent facade names have the specified `pub(super)` visibility: both avatar leaves; all six `live` parent operations; the message and topic leaves; `DialogListing::new`; the four peer leaves including the newly restricted descriptor converter; `is_non_forum_topic_refresh_error`; `TelegramSession::{clone_memory_session,cache_peer_infos}`; and `media::extract_raw_item_payload`. The four media compatibility names that remain temporarily `pub(crate)` are governed by their explicit finite lifecycle rather than prematurely demoted. Remaining `takeout::*`, later error, and later media names are future CP7 targets and are not materialized early. No restricted name is root-public, and all current root-public types have owned, raw-free signatures.

## TypeScript source-analysis prohibition

There is no new TypeScript Rust parser, Rust fixture variant, or mutation test in the candidate.

- The complete HEAD-relative diff of `src/lib/telegram-crate-boundary-contract.test.ts` changes only the expected generated authority cardinalities: 306 to 307 disposition entries, 98 to 99 source rows in three error expectations, and 67 to 69 restricted names in the corresponding expectations.
- The prohibited analyzer names `phase8BTypedReceiverNames`, `phase8BProducerBoundReceiverNames`, `phase8BFunctionUsesTransitionBridge`, and `phase8BIsCanonicalBridgeTypePath` occur only as prose in the plan and do not exist in TypeScript.
- `phase8BSessionFixture()` exists in exactly the same two lines in the retained predecessor and the current file. It takes no parameter; there is no `includeRawAccessor` branch. It is an unchanged non-use-site compilation/public-surface fixture, not a newly introduced escape-hatch analyzer.
- The generator parses approved plan tables/fences into JSON and never reads, tokenizes, masks, scopes, or semantically reconciles Rust source.

Accordingly, no TypeScript result accepts or rejects a Rust escape-hatch use. The source-level ownership analysis in this record is the required LLM review; automatic evidence below is Rust compilation and Rust behavior testing (plus formatting/diff and generated-document consistency checks, none of which authorize use-sites).

## Four-item media compatibility facade and new restricted raw adapter

The temporary CP3-CP6 media compatibility facade remains exactly four `pub(crate)` definitions in `telegram_impl/media.rs`:

1. `DocumentSignals`, with exactly its five package-private fields;
2. `derive_content_kind`;
3. `derive_document_media_kind`;
4. `extract_item_payload`.

`telegram_impl/lib.rs` re-exports exactly those four package-privately, and app `src-tauri/src/media.rs` re-exports exactly those four package-privately. There is no glob, alias, public re-export, fifth facade item, or second compatibility module.

The remaining app physical uses are narrower than CP4 and fully accounted:

- `takeout_import/raw_parse.rs` imports `DocumentSignals`, `derive_content_kind`, and `derive_document_media_kind`, constructs the one signal value, and calls the two classifiers in its existing raw Takeout parse flow;
- `extract_item_payload` has no app call after the live-history move, but remains the declared temporary compatibility item through its planned lifecycle and is not widened;
- all other occurrences are the exact two package-private re-export sites, implementation-local calls, or leaf tests.

Task 5 adds `media::extract_raw_item_payload` as a separate `pub(super)` staged-internal adapter. `LiveMessage::into_draft` is its sole production caller. The adapter and the retained high-level helper delegate to the same private `extract_item_payload_from_parts`, so raw/high-level mapping cannot silently fork. The new adapter is not app-visible, not root-re-exported, not part of the four-item compatibility facade, and returns only owned application payload values.

## Remaining transitional raw/session/app definitions and exact physical uses

The active definitions and all production physical uses are:

| Definition | Visibility | Physical uses | Exact accounting |
| --- | --- | ---: | --- |
| `TelegramClientHandle::raw_client` | `pub(crate)` | 3 | one call in each of the three CP5 Takeout logical owners |
| `TelegramClientHandle::raw_session` | `pub(crate)` | 3 | one call in each of the same three owners |
| `TelegramSession::raw_memory_session` | `pub(super)` | 2 | session serialization and the `raw_session` wrapper |
| `telegram::get_client` | `pub(crate)` | 2 | `sources::store::{list_telegram_sources,add_telegram_source}`, both owned-handle consumers |
| `telegram::get_authorized_client` | `pub(crate)` | 4 | sync's owned handle plus the three Takeout owners |
| `ResolvedSyncPeer::peer` | `pub(crate)` field | 6 reads | two in migrated history and four under current-source Takeout |
| `legacy_peer_ref_from_descriptor` | `pub(crate)` | 1 | sole call in `resolve_and_refresh_peer`, the sole field producer |

`raw_client` and `raw_session` have no call in `sources::sync` or any other app owner. Each body still returns the exact raw value directly; there is no parallel accessor. `raw_session` delegates only to `self.session.raw_memory_session()`.

`get_client` delegates only to `TelegramRuntime::client`; its two store consumers use owned handle methods and never call a raw accessor. `get_authorized_client` delegates only to `TelegramRuntime::authorized_client`; its sync consumer now uses only owned handle APIs, while its three Takeout results feed exactly the three allowed raw owners.

`ResolvedSyncPeer` is unchanged and has exactly one definition with fields `peer: PeerRef`, owned `descriptor: PeerDescriptor`, and the optional refreshed avatar cache key. Its sole producer remains `resolve_and_refresh_peer`: the function obtains the owned descriptor, invokes `legacy_peer_ref_from_descriptor` once, and constructs the one result. The field's six physical reads are:

- migrated-history owner: provenance identity and `revalidate_migrated_from_chat_id` (2);
- current-source owner/forwarded continuation: provenance identity, `InputPeer` conversion, `validate_takeout_peer`, and `detect_supergroup_migration` (4).

The former current-source completed-Takeout topic-refresh raw peer read is gone; that operation now receives `&resolved_peer.descriptor` and `&TelegramClientHandle`.

`TelegramSession::clone_memory_session` is a permanent restricted bridge rather than a transitional app escape. Its production call materializes the sender pool during client initialization; other calls are staged-unit-test pointer/session checks. It is not app-visible and is not root-re-exported. The new permanent `cache_peer_infos` method is also `pub(super)` and is called only by the staged live-message operation.

## Logical raw owners and full forwarding chains

### Export-DC spike owner

```text
run_takeout_export_dc_spike
  -> get_authorized_client -> TelegramRuntime::authorized_client
  -> run_export_dc_spike_for_handle(owned TelegramClientHandle)
     -> raw_client (one call) + raw_session (one call)
        -> raw_memory_session
     -> cloned raw values used by self-check, export-DC alias, and Takeout calls
```

The handle is consumed by `run_export_dc_spike_for_handle`; no helper reacquires it, returns it, or creates another owner.

### Migrated-history owner

```text
start_takeout_migrated_history_import
  -> run_takeout_migrated_history_import_job
  -> run_takeout_migrated_history_import
     -> get_authorized_client
     -> raw_client (one call) + raw_session (one call)
     -> resolve_and_refresh_peer(opaque handle)
        -> owned PeerDescriptor
        -> legacy_peer_ref_from_descriptor (sole producer)
        -> ResolvedSyncPeer
     -> peer read for provenance identity
     -> peer read for revalidate_migrated_from_chat_id
     -> remaining Takeout work uses already produced raw/owned values
```

No nested helper accepts or obtains `TelegramClientHandle`, calls a raw accessor, or becomes a second logical owner.

### Current-source Takeout owner and mandatory full continuation

```text
start_takeout_source_import
  -> run_takeout_import_job
  -> run_takeout_source_import
     -> get_authorized_client
     -> raw_client (one call) + raw_session (one call)
     -> resolve_and_refresh_peer(opaque handle)
        -> owned PeerDescriptor + transitional PeerRef
     -> peer read for provenance identity
     -> run_started_takeout_source_import
        [forwards resolved_peer, &TelegramClientHandle, &Client, session-derived alias, state]
        -> run_started_takeout_source_import_inner
           [forwards/uses the same values; no reacquisition and no raw accessor]
           -> peer read for InputPeer conversion
           -> peer read for validate_takeout_peer
           -> peer read for detect_supergroup_migration
           -> history import
           -> finish Takeout
           -> refresh_forum_topics_after_completed_takeout
              -> refresh_forum_topics
                 -> TelegramClientHandle::fetch_forum_topics(&PeerDescriptor)
           -> finalize sync
           -> finalize ingest batch
```

The public owned handle is forwarded through both started-import functions solely to invoke the new owned forum-topic method at the end. Neither continuation calls `raw_client`, `raw_session`, `get_client`, or `get_authorized_client`; neither constructs a handle or peer; neither adds a logical owner. The original outer owner retains the raw values required by the still-unmoved Takeout path. This accounts for the complete plan-mandated three-function chain.

## CP5 removals and application boundary

Every use scheduled to disappear at CP5 is gone:

- `sources::sync::sync_telegram_source` no longer calls `raw_client` and no longer reads `ResolvedSyncPeer::peer`; it resolves an opaque handle/owned descriptor, calls the owned forum-topic API, and passes the owned descriptor into the live batch loop;
- the app raw `iter_messages`/payload/author/context/reply logic is removed from `sources::{sync,items}` and is owned by `telegram_impl/live/messages.rs` plus the restricted media adapter;
- the app raw forum-topic pagination, TL mapping, cursor helpers, and error classifier are removed from `sources/topics.rs` and owned by `telegram_impl/live/topics.rs` and `telegram_impl/error.rs`;
- the completed-Takeout topic refresh no longer receives or consumes `ResolvedSyncPeer::peer`; the exact owned handle is threaded through the full continuation and the owned descriptor is used;
- the temporary CP3-CP4 peer-kind root bridge is removed: the three `TELEGRAM_PEER_KIND_*` constants are private DTO implementation constants, are not re-exported by `telegram_impl/lib.rs`, and have no app consumer;
- `sources/items.rs`, `sources/sync.rs`, and `sources/topics.rs` contain no Grammers import, raw Grammers type, `tl::` operation, `PeerRef`, `PeerKind`, or raw `Client` use.

The new public boundary is raw-free:

- `TelegramClientHandle::fetch_message_batch` accepts only `&PeerDescriptor`, integer offsets, and `usize`, and returns owned `LiveMessageBatch`;
- `TelegramClientHandle::fetch_forum_topics` accepts only `&PeerDescriptor` and returns owned `ForumTopicSnapshot`/deleted-ID vectors;
- `LiveMessageBatch` is public with private fields and exposes only owned messages, terminal state, and integer offsets;
- `LiveMessage` is public with private raw/fetched/peer-map fields and exposes only integer metadata and consuming conversion to `TelegramMessageDraft`;
- `ForumTopicSnapshot` is an owned DTO with no Grammers field;
- no raw Grammers type appears in a root-public function signature or field.

No CP7 removal happened early: the three Takeout raw owners, both handle raw accessors, borrowed session accessor, app lookup helpers, transitional peer field, sole legacy builder, and four-item media facade remain exactly for their finite lifecycle.

## Live-message equivalence, cache invariant, and initialization guard

The new leaf performs exactly one raw `GetHistory` invocation per batch through one private backend seam. It validates `limit` in `1..=100` before invocation and sends the frozen offsets/request fields. Response order is preserved. A nonterminal page advances both ID and date offsets from the final returned message; a terminal page preserves the input offsets. `Messages::Messages` is terminal; `Slice` and `ChannelMessages` use the pinned Grammers empty/first-ID terminal rule.

The peer map reproduces pinned Grammers ordering and overwrite behavior: users are inserted first, chats second. `PeerInfo` uses the pinned conversions, and `TelegramSession::cache_peer_infos` caches exactly entries for which `PeerInfo::auth().is_some()`. This matches pinned `Peer::auth().is_some()` semantics, including ambient authority for chats; it is not an access-hash-only predicate.

The outer `auto_cache_peers` precondition is executable. `initialize_grammers_client(api_id, session)` does not accept configuration from its caller; it materializes `ClientConfiguration::default()` internally, rejects `!configuration.auto_cache_peers`, and performs that guard before constructing `SenderPool` or spawning its runner. It then uses `Client::with_configuration`. The configuration-free signature prevents moving the invariant to a caller merely to make the currently unreachable error branch testable. Test-only client construction does not alter the production invariant.

Pinned Grammers panics on `Messages::NotModified` for history hash zero. The candidate deliberately maps it to a typed network `AppError`, matching the approved plan and existing Takeout convention. The match occurs before peer caching or message conversion, so the error has no partial cache/conversion side effect.

The raw-message conversion retains owned message identity, content/media classification, author, context/reply/reaction fields, raw serialized payload, and peer lookup semantics behind private fields. The live batch test exercises populated/empty/min/no-auth peers; the draft test exercises owned mapping and empty-payload skip.

## Topic operation and sync loop behavior

The topic pagination/mapping/classifier moved intact to the staged owner. `sources/topics.rs` retains only the app policy, SQL writes, membership rebuild, and warnings. The same public owned handle operation is used by normal sync and the current-source completed-Takeout path. The Takeout wrapper retains completion/warning/provenance policy and records a refresh warning before final ingest-batch completion.

The app sync loop requests `min(remaining, 100)`, forwards both offsets, and decrements the remaining-message budget before previous-ID/date cutoff checks and before a message can be skipped by conversion. It updates max ID before conversion, persists each draft sequentially, stops immediately on conversion/persistence/fetch error, returns only after a terminal page or exhausted budget, and advances both offsets only between successful pages. Thus a skipped raw entry still consumes budget and a successful prior insert is durable if a later entry or page fails.

The exact existing Rust behavior test covers all owner-requested orchestration evidence in one test body: persistence order/durability, skipped-entry budget consumption and shrinking second-page limit, offset order, conversion-stop behavior, and a network `timeout` on the second fetch with the first page already persisted and no third fetch. No TypeScript Rust-source analysis is used as evidence for budget, order, or timeout.

## Alias, re-export, wrapper, function-item, macro, and same-name audit

- None of the temporary names is imported/re-exported with `as`, hidden behind a type alias, invoked through UFCS, captured as a function item/callback, or shadowed by an alternate callable.
- There is no exception-related `macro_rules!`, generated identifier, `paste!`, `concat_idents!`, production glob import, or public leaf module that creates an alternate path.
- Temporary re-exports are limited to the exact four media compatibility names in the staged root and app media facade. Raw accessors, raw types, the transitional peer field, and the legacy builder are not root-re-exported.
- `sources::ResolvedSyncPeer` is the one package-private re-export of the exact struct needed by Takeout continuation signatures; it neither duplicates nor conceals the field producer.
- The private test/back-end traits in live messages/topics and sync hold or model raw values only inside their owning modules. They are not implemented as public traits on `TelegramClientHandle`, not root-re-exported, and cannot expose a raw client/session/peer to app code.
- `LiveMessage` privately retains a raw TL message and peer map so its consuming owned conversion can preserve semantics. Its private fields and raw-free public methods make it an opaque owned boundary value, not an escape hatch.
- `DialogListing` remains the approved opaque public iterator: raw client/iterator state is private and `next` returns only owned descriptors.
- Other fields or locals named `client`, `session`, `peer`, `raw`, `messages`, or `legacy` are staged implementation details or unrelated app data, not additional definitions, producers, wrappers, aliases, or logical owners of the transitional exceptions.

## Fresh automatic evidence

All commands were run against the frozen candidate in `G:\Develop\Extractum` after the static ownership review:

| Command | Fresh result |
| --- | --- |
| `node scripts/telegram-8b-symbol-map.mjs --check` | exit 0 |
| `npm.cmd run check:rustfmt` | exit 0 |
| `git diff --check 1acc4f618dcc5855b677c844624f21c67c371b47` | exit 0; only Git LF/CRLF notices |
| exact live-message batch test | 1 passed, 0 failed, 726 filtered out |
| exact live-message draft test | 1 passed, 0 failed, 726 filtered out |
| exact forum-topic page test | 1 passed, 0 failed, 726 filtered out |
| exact sync durability/budget/error/timeout test | 1 passed, 0 failed, 726 filtered out |
| `cargo check --color never --manifest-path src-tauri/Cargo.toml -p extractum --all-targets` | exit 0; warnings only |
| `cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --all-targets` | 727 passed, 0 failed; binary target had 0 tests |

The four exact test identities were:

```text
telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule
telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload
telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor
sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error
```

These tests and compilation are behavior/type evidence only. The LLM review of definitions and full owner chains above is the authorization evidence for retained escape uses.

## Findings

- Critical: none.
- High: none.
- Medium: none.
- Low: none.
- Unlisted, ambiguous, aliased, newly wrapped, early-added, or prematurely retained escape-hatch definition/use: none.

The frozen CP5 candidate has exactly the three authorized raw-handle/session logical owners, exactly the two authorized `ResolvedSyncPeer::peer` logical owners, the exact remaining bridge definitions and physical uses, the complete required forwarding chain, and no new app-visible raw escape. Every CP5 removal is present, later finite-lifecycle exceptions remain, the 69-name restricted authority is consistent with checkpoint lifecycle, and no TypeScript code semantically analyzes or authorizes Rust use-sites.

Escape-hatch review verdict: CLEAN

## CP6 Escape-Hatch LLM Review

### Review identity and candidate

- Review time: `2026-07-31T22:09:47.0217922+03:00`.
- Selected checkpoint: Phase 8B Checkpoint 6, Task 6, `Decompose the Two Raw-TL/SQL Baselines`.
- Candidate checkpoint SHA: none; the candidate is intentionally uncommitted.
- Retained predecessor and current HEAD: `180c797bf04d63c33df555cbb04da51b73a2b3af` (`refactor: stage Telegram live message and topic operations`).
- Reviewer: fresh independent read-only LLM reviewer. I did not implement CP6, had no prior CP6 implementation-conversation context, and did not rely on literal-search counts, TypeScript source analysis, or an implementation report to authorize an escape-hatch use.
- I made no file edits, staging changes, or commits. The Git index was empty at the final identity check.

The complete HEAD-relative Rust candidate contains only:

```text
src-tauri/src/takeout_import/mod.rs
src-tauri/src/takeout_import/raw_parse.rs
```

Its exact final reviewed identity was:

```text
src-tauri/src/takeout_import/mod.rs
  bytes: 92448
  lines: 2790
  sha256: c06972155835b300553f7baecff4541c5287760594d625a89630610f6dd3e5ed
  diff: 81 insertions, 103 deletions

src-tauri/src/takeout_import/raw_parse.rs
  bytes: 25526
  lines: 748
  sha256: 673e3f6789a283523e2c8208f194a230fde8e98f537762c3873c2ff896382854
  diff: 89 insertions, 0 deletions
```

I read the complete diff and sufficient unchanged production, producer, caller, continuation, facade, session, and peer-resolution source. Every changed hunk is inside an existing `#[cfg(test)]` module. Any subsequent edit to either reviewed Rust path invalidates this record and requires a fresh complete review.

### Authority and inventory inputs

I read and reconciled:

- the complete mandatory reusable CP2–CP8 LLM retention gate in `docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md`;
- the complete Task 6, including its CP6 execution-preflight amendment;
- the design’s public API/visibility authority, Phase 8B narrow API and lifecycle authority, literal production-symbol disposition, complete transition inventories, and forward-only transitional escape-hatch review authority;
- the complete retained CP5 review section in `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`;
- the complete committed `src/lib/telegram-8b-symbol-map.json`, read from predecessor HEAD, including every one of its 307 expanded symbol-disposition rows, every one of its 69 restricted-final-symbol rows, and its complete `transitionInventories`.

The committed artifact identity was:

```text
schemaVersion: 1
bytes: 123717
sha256: 4fdfac6ca545b3ac5eb7dd09d7cfb68d5e7341a4476f938efe63186bc9bd96f0
Git blob: 5fbf450f50780de182de870e452d28ca69949b73
expanded symbol rows: 307
restricted final symbols: 69
```

The complete transition inventories were:

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
  empty
```

CP6 must carry both CP5 arrays forward unless this Rust diff removes a listed use. The reviewed diff removes none, so CP6 retains exactly the three CP5 raw-handle owners and exactly the two CP5 `ResolvedSyncPeer::peer` consumers.

I treated the artifact and searches only as navigation checklists. Authorization below comes from the actual Rust definitions, visibility, bodies, typed values, producers, callsites, and complete forwarding chains.

### Exact effect of the test-only decomposition

The candidate performs only the Task 6 characterization decomposition:

- The two app persistence tests in `takeout_import::tests` now construct approved owned `TelegramMessageDraft`, `TelegramMessageIdentity`, and `TelegramItemContext` values directly.
- The app tests no longer import `grammers_client::tl`, invoke `raw_parse::parse_raw_message`, or define `takeout_raw_message_for_identity_test`.
- `takeout_raw_message_for_identity_test` is now a private helper inside `takeout_import::raw_parse::tests`, implemented using the existing private `raw_message` fixture.
- Two raw-parser companion tests call the existing `parse_raw_message` and characterize distinct-history-peer identity and identical native identity.
- The SQL primary retains its required two-insert assertion and exact `SELECT COUNT(*) FROM items WHERE external_id = '42'` check.
- The duplicate primary retains inserted/duplicate outcomes and final `("ready", 1)` topic-resolution state.

This changes test ownership of raw TL fixture construction and test callsites of `parse_raw_message`. It does not change an escape-hatch definition, physical use, producer, logical owner, visibility, alias, re-export, wrapper, function item, same-named field, or forwarding chain. `parse_raw_message` is the existing package-private parser in a private module and a CP7 move target; it is not one of the CP3–CP6 transitional raw client/session/peer escape hatches. Its production body and visibility are unchanged.

### Remaining media compatibility facade

The CP3–CP6 media compatibility facade remains exactly four package-private definitions in `telegram_impl/media.rs`:

1. `DocumentSignals`, with exactly five `pub(crate)` fields;
2. `derive_content_kind`;
3. `derive_document_media_kind`;
4. `extract_item_payload`.

The exact package-private re-export layers remain:

```text
telegram_impl/lib.rs
  -> derive_content_kind
  -> derive_document_media_kind
  -> extract_item_payload
  -> DocumentSignals

src-tauri/src/media.rs
  -> the same exact four names
```

There is no public or glob export, alias, fifth compatibility item, or second facade.

The unchanged physical uses are:

- `takeout_import/raw_parse.rs` imports `DocumentSignals`, `derive_content_kind`, and `derive_document_media_kind`, constructs the raw-document signal value, calls the content classifier once, and calls the document classifier once.
- `telegram_impl/media.rs` constructs its own high-level signal value, calls `derive_content_kind` once through `extract_item_payload_from_parts`, and calls `derive_document_media_kind` once in high-level document extraction.
- Staged media unit tests call `derive_content_kind` three times, construct three `DocumentSignals` values, and call `derive_document_media_kind` three times.
- `extract_item_payload` has no current app or staged callsite; only its definition and the exact two package-private re-exports remain.

The separate permanent restricted `media::extract_raw_item_payload` adapter remains `pub(super)`, has the single staged production caller `LiveMessage::into_draft`, is not root-re-exported, and is not part of the temporary four-item facade. CP6 does not change it.

### Remaining raw/session/app definitions and exact physical uses

| Definition | Visibility | Remaining physical uses |
| --- | --- | --- |
| `TelegramClientHandle::raw_client` | `pub(crate)` | 3 calls: one in each CP5 Takeout owner |
| `TelegramClientHandle::raw_session` | `pub(crate)` | 3 calls: one in each CP5 Takeout owner |
| `TelegramSession::raw_memory_session` | `pub(super)` | 2 calls: session serialization and the `raw_session` wrapper |
| `telegram::get_client` | `pub(crate)` | 2 calls: `sources::store::{list_telegram_sources,add_telegram_source}` |
| `telegram::get_authorized_client` | `pub(crate)` | 4 calls: sync plus the three Takeout owners |
| `ResolvedSyncPeer::peer` | `pub(crate)` field | 6 reads: two in migrated history and four in current-source Takeout |
| `legacy_peer_ref_from_descriptor` | `pub(crate)` | 1 call, inside the sole `ResolvedSyncPeer` producer |

The accessor bodies remain direct and unchanged:

- `raw_client` returns the underlying Grammers client from the opaque handle.
- `raw_session` delegates only to `self.session.raw_memory_session()`.
- `raw_memory_session` returns the borrowed `Arc<MemorySession>`.
- No parallel accessor, alternate spelling, callback, or wrapper was introduced.

`get_client` delegates only to `TelegramRuntime::client`. Its two store consumers use owned handle operations and never call a raw accessor.

`get_authorized_client` delegates only to `TelegramRuntime::authorized_client`. The sync consumer uses owned descriptor, live-message, and forum-topic APIs and never calls a raw accessor or reads `ResolvedSyncPeer::peer`. Its three Takeout results feed exactly the three authorized raw owners.

`ResolvedSyncPeer` remains one struct with exactly:

```text
peer: PeerRef
descriptor: PeerDescriptor
refreshed_avatar_cache_key: Option<String>
```

Its sole producer remains `resolve_and_refresh_peer`. That producer obtains an owned `PeerDescriptor`, calls `legacy_peer_ref_from_descriptor(&descriptor)` exactly once, and constructs the single `ResolvedSyncPeer`. Sync invokes this producer but consumes only the owned descriptor and refreshed cache key; it does not read the transitional peer field. The two Takeout consumers account for every field read.

`TelegramSession::clone_memory_session` and `cache_peer_infos` remain permanent `pub(super)` restricted bridges, not transitional application escapes. Neither is app-visible or root-re-exported, and CP6 does not change either.

### Logical owners, producers, and complete forwarding chains

#### Export-DC spike owner

```text
run_takeout_export_dc_spike
  -> get_authorized_client
     -> TelegramRuntime::authorized_client
  -> run_export_dc_spike_for_handle(owned TelegramClientHandle)
     -> raw_client (one call)
     -> raw_session (one call)
        -> raw_memory_session
     -> cloned raw client/session values feed self-check,
        export-DC alias preparation, Takeout init/split, and finish calls
```

The handle is consumed by `run_export_dc_spike_for_handle`. No nested helper reacquires a handle, calls an accessor, returns a raw handle, or creates another logical owner.

#### Migrated-history owner

```text
start_takeout_migrated_history_import
  -> spawned run_takeout_migrated_history_import_job
  -> run_takeout_migrated_history_import
     -> get_authorized_client
     -> raw_client (one call)
     -> raw_session (one call)
        -> raw_memory_session
     -> resolve_and_refresh_peer(&TelegramClientHandle)
        -> owned PeerDescriptor
        -> legacy_peer_ref_from_descriptor (sole static callsite)
        -> ResolvedSyncPeer
     -> ResolvedSyncPeer::peer read for provenance identity
     -> ResolvedSyncPeer::peer read for revalidate_migrated_from_chat_id
     -> remaining Takeout work uses already-produced raw/owned values
```

Those are the owner’s exact two peer reads. No nested migrated-history helper accepts or obtains `TelegramClientHandle`, invokes a raw accessor, or becomes another logical owner.

#### Current-source Takeout owner and mandatory continuation

```text
start_takeout_source_import
  -> spawned run_takeout_import_job
  -> run_takeout_source_import
     -> get_authorized_client
     -> raw_client (one call)
     -> raw_session (one call)
        -> raw_memory_session
     -> resolve_and_refresh_peer(&TelegramClientHandle)
        -> owned PeerDescriptor
        -> legacy_peer_ref_from_descriptor
        -> ResolvedSyncPeer
     -> ResolvedSyncPeer::peer read for provenance identity
     -> run_started_takeout_source_import
        [forwards ResolvedSyncPeer, &TelegramClientHandle, &Client,
         alias, Takeout ID, warnings/fallback state, and attempts]
        -> run_started_takeout_source_import_inner
           [uses the same forwarded values; no reacquisition]
           -> ResolvedSyncPeer::peer read for InputPeer conversion
           -> ResolvedSyncPeer::peer read for validate_takeout_peer
           -> ResolvedSyncPeer::peer read for detect_supergroup_migration
           -> count/import history
           -> finish Takeout
           -> refresh_forum_topics_after_completed_takeout
              -> TelegramClientHandle::fetch_forum_topics(
                   &resolved_peer.descriptor
                 )
           -> finalize sync
           -> finalize ingest batch
```

This accounts for the mandatory full chain:

```text
run_takeout_source_import
  -> run_started_takeout_source_import
  -> run_started_takeout_source_import_inner
```

The outer owner contains its one raw-client call, one raw-session call, and first peer read. The intermediate function only forwards values and error state. The inner function performs the remaining three peer reads and uses the forwarded owned handle only for the owned forum-topic operation. Neither continuation calls `raw_client`, `raw_session`, `get_client`, or `get_authorized_client`; neither constructs another handle or resolved peer; neither adds a logical owner.

### Required CP6 removals and lifecycle reconciliation

No escape-hatch removal is scheduled for CP6, and the Task 6 Rust diff removes no CP5-inventory use. Therefore:

- the three CP5 raw-handle owners remain exactly unchanged;
- the two CP5 `ResolvedSyncPeer::peer` consumers remain exactly unchanged;
- all seven listed transitional raw/session/app definitions remain with unchanged visibility;
- all six peer-field reads and the sole legacy producer call remain;
- the four-item media compatibility facade remains through CP6;
- no CP7 removal occurs early.

Task 6 does remove raw TL fixture construction and `raw_parse::parse_raw_message` calls from the app persistence tests, but those are decomposition requirements, not removal of a transitional client/session/peer escape hatch. Equivalent raw-parser characterization now resides only in the parser’s own test module.

The CP3–CP4 peer-kind staged-root bridge remains absent as required after CP5: `TELEGRAM_PEER_KIND_{CHANNEL,CHAT,USER}` are not re-exported by `telegram_impl/lib.rs`, `sources::sync::fallback_message_identity` is gone from the app owner, and CP6 adds no replacement spelling or constant bridge.

At CP7, not CP6, the temporary media facade, both raw handle accessors, borrowed raw-session accessor, both app lookup helpers, transitional peer field, and legacy builder must be removed or demoted according to authority. CP7/CP8 must then satisfy the empty `cp7RawBridgeSymbolsAndCallsites` inventory.

### Alias, re-export, wrapper, function-item, macro, and same-name audit

- None of the temporary raw bridge names is renamed with `as`, hidden behind a type alias, invoked through UFCS, captured as a function item/callback, or shadowed by another callable.
- There is no relevant glob import, public leaf-module escape, `macro_rules!`, generated identifier, `paste!`, or `concat_idents!` path.
- The only temporary re-exports are the exact four package-private media names through the staged root and app media facade.
- Raw client/session accessors, raw Grammers types, `legacy_peer_ref_from_descriptor`, and the `ResolvedSyncPeer::peer` field are not staged-root or externally public re-exports.
- `sources::ResolvedSyncPeer` remains the one package-private re-export of the exact struct needed by Takeout continuation signatures. It does not duplicate, alias, or conceal the field or its producer.
- `get_authorized_client` is imported normally into `takeout_import`; it is not aliased. The other physical calls use their explicit `crate::telegram::...` path.
- The moved `takeout_raw_message_for_identity_test` is a private `#[cfg(test)]` raw-message fixture constructor. It wraps neither a client handle nor a session or resolved peer, does not expose a production raw value to app code, and is not an alternate escape-hatch spelling.
- The new parser companion calls to `parse_raw_message` remain inside `takeout_import::raw_parse::tests`; they create no public wrapper or function item.
- The candidate’s `TelegramMessageIdentity::{history_peer_kind,history_peer_id}` fields and raw fixture `Message::peer_id` are owned identity/test data, not aliases or same-named substitutes for `ResolvedSyncPeer::peer`.
- Other fields or locals named `peer`, including staged live-operation request/dialog peers and sync’s private owned `PeerDescriptor` test seam, are unrelated typed implementation details. None produces or forwards the transitional `PeerRef`.
- Other names containing `raw`, `client`, `session`, `messages`, or `identity` are private staged implementation data or test fixtures and do not provide another application-visible client/session/peer escape.

### Findings

- Critical: none.
- High: none.
- Medium: none.
- Low: none.
- Unlisted or ambiguous escape-hatch definition/use: none.
- New alias, re-export, wrapper, function item, producer, logical owner, forwarding continuation, visibility widening, same-named replacement, or early lifecycle transition: none.

The frozen CP6 candidate is a test-only baseline decomposition. It preserves exactly the CP5 media and raw-bridge definitions, exact physical-use inventory, three raw-handle logical owners, two peer-field logical owners, sole peer producer, and complete current-source forwarding chain. No CP6 escape-hatch removal is required, no CP5 listed use was removed, and no new or obscured application raw escape exists.

Escape-hatch review verdict: CLEAN
