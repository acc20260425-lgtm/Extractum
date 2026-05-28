# Database Schema

This document describes the current local SQLite schema at a practical level.

## 1. Core tables

### 1.1 `sources`

Stores registered provider sources. Telegram and YouTube ingest are implemented
today, while RSS/forum remain provider-model placeholders.

Important fields:

- `id`
- `source_type`
- `source_subtype`
- `external_id`
- `title`
- `metadata_zstd`
- `last_sync_state`
- `last_synced_at`
- `account_id`
- `is_active`
- `is_member`
- `created_at`

Important constraints / indexes:

- canonical Telegram source identity by `(account_id, source_type, source_subtype, external_id)`
  where `source_type = 'telegram'`
- unique YouTube video source by `(source_type, source_subtype, external_id)`
  where `source_type = 'youtube' AND source_subtype = 'video'`
- unique YouTube playlist source by `(source_type, source_subtype, external_id)`
  where `source_type = 'youtube' AND source_subtype = 'playlist'`

`source_type` values currently supported by shared contracts:

- `telegram`
- `youtube`
- `rss`
- `forum`

Implemented ingest providers are `telegram` and `youtube`.

`source_subtype` is provider-local:

- Telegram uses `channel`, `supergroup`, or `group`
- YouTube uses `video` or `playlist`
- future RSS can use `feed`
- future forums can use `thread`, `board`, or `site`

Notes:

- current supported databases use `source_type = 'telegram'` with
  provider-local subtype in `source_subtype`;
- the active baseline already contains typed Telegram identity tables and the
  canonical Telegram uniqueness rules;
- pre-baseline compatibility migrations that introduced `source_subtype`,
  typed Telegram identity, and the final `sources` shape are archived under
  `docs/archive/migrations-pre-baseline-reset/`;
- Telegram source subtype is canonical in `sources.source_subtype`;
- Telegram operational peer identity lives in `telegram_sources`.
- new Telegram source rows keep `metadata_zstd` `NULL`;
- old Telegram blobs may remain in existing databases as legacy repair input;
  the cleanup policy is defined in
  `docs/superpowers/specs/2026-05-23-legacy-telegram-source-metadata-cleanup-policy-design.md`;
- normal Telegram runtime updates preserve old Telegram blobs rather than
  clearing them opportunistically;
- normal Telegram sync, Takeout, forum topic refresh, source list display, and
  source resolution use typed identity and display cache fields in
  `telegram_sources`, not Telegram source metadata blobs;
- legacy Telegram `sources.metadata_zstd` is no longer the runtime source of
  truth; cleanup is allowed only through the explicit
  `audit_legacy_telegram_source_metadata` and
  `clear_legacy_telegram_source_metadata` guarded operations after typed
  identity validation, not through startup, ordinary schema migration, or
  opportunistic sync/update/list/Takeout paths;
- uniqueness includes `account_id` because the same Telegram source can be added from multiple local accounts;
- uniqueness includes `source_subtype` because Telegram bare ids are not enough to safely describe every peer shape.
- `last_sync_state` and `last_synced_at` are advanced by normal sync and by successful Takeout import; failed or cancelled Takeout jobs leave these fields unchanged.
- YouTube source rows keep `sources.metadata_zstd` `NULL` after successful
  typed writes. Existing invalid or unbackfillable legacy YouTube blobs may
  remain inert, but normal YouTube listing, detail, jobs, and analysis do not
  decode them.
- YouTube video and playlist runtime metadata lives in `youtube_video_sources`
  and `youtube_playlist_sources`.
- YouTube source jobs are in memory in the MVP; they may update `last_synced_at`, but active job records are not restored after app restart.

### 1.2 `telegram_sources`

Stores typed operational identity for Telegram sources. Generic provider
identity stays in `sources`; Telegram peer identity, peer resolution hints, and
Telegram display cache fields live here.

Important fields:

- `source_id`
- `account_id`
- `source_subtype`
- `peer_kind`
- `peer_id`
- `resolution_strategy`
- `username`
- `access_hash`
- `avatar_cache_key`
- `identity_refreshed_at`
- `created_at`
- `updated_at`

Important constraints / indexes:

- primary key and `ON DELETE CASCADE` foreign key by `source_id`
- unique Telegram peer by `(account_id, peer_kind, peer_id)`
- lookup index by `(account_id, source_subtype)`
- username lookup index by `(account_id, username)` where `username IS NOT NULL`
- checks keep `source_subtype`, `peer_kind`, and `resolution_strategy` in the
  supported enum sets and enforce subtype/peer-kind consistency

Notes:

- `source_subtype` uses the same Telegram values as `sources.source_subtype`.
- `peer_kind = 'channel'` is used for Telegram channels and supergroups;
  `peer_kind = 'chat'` is used for small groups.
- `resolution_strategy` records how the peer was or can be resolved:
  `username`, `dialog`, `legacy_metadata`, or `unknown`.
- normal Telegram sync, Takeout, forum topic refresh, source list display, and
  NotebookLM source loading use this typed identity instead of decoding legacy
  `sources.metadata_zstd`.
- legacy metadata is decoded during startup repair and compatibility paths, not
  as the normal runtime source identity fallback.

### `telegram_migrated_history_capabilities`

Source-level Telegram capability state for explicit migrated small-group
history import. The table is keyed by `source_id` and stores private old-chat
access hints separately from `telegram_sources`.

Allowed `status` values:

- `none`
- `available`
- `unavailable`

Allowed `unavailable_reason` values are internal diagnostics:

- `not_detected`
- `missing_migrated_from_chat_id`
- `current_source_unavailable`
- `old_chat_input_unavailable`
- `revalidation_failed`

Frontend source records expose only sanitized availability status and
timestamps. They do not expose `migrated_from_chat_id`.

### 1.3 `telegram_messages`

Stores typed native identity and Telegram message context for `telegram_message`
items. `items` remains the local item/archive container; this table gives
Telegram duplicate detection and topic/ref logic a provider-native identity.

Important fields:

- `item_id`
- `source_id`
- `history_peer_kind`
- `history_peer_id`
- `telegram_message_id`
- `migration_domain`
- `is_migrated_history`
- `reply_to_msg_id`
- `reply_to_peer_kind`
- `reply_to_peer_id`
- `reply_to_top_id`
- `reaction_count`
- `created_at`
- `updated_at`

Important constraints / indexes:

- primary key and `ON DELETE CASCADE` foreign key by `item_id`
- native Telegram identity by `(source_id, history_peer_kind, history_peer_id,
  telegram_message_id)`
- lookup index by `(source_id, telegram_message_id)`
- topic fallback lookup index by `(source_id, reply_to_top_id)`

Notes:

- `history_peer_kind` and `history_peer_id` identify the Telegram
  history/origin peer for the message, not necessarily the current resolved
  source peer.
- for non-migrated current history, `history_peer_*` usually equals
  `telegram_sources.peer_*`.
- for migrated history, `history_peer_*` identifies the original Telegram
  history domain.
- `migration_domain` is a row-level marker. The first functional value is
  `migrated_from_chat`; it marks rows imported from a supergroup's old
  small-group history. It is not part of the primary duplicate identity.
- `telegram_messages.source_id` must equal `items.source_id`, and
  `telegram_messages.item_id` must point to an item whose `item_kind` is
  `telegram_message`; migration/runtime tests enforce this application
  invariant.
- `updated_at` is set on child-row creation in this slice. Duplicate skips do
  not update `telegram_messages.updated_at`.

### 1.4 Ingest provenance

Migration `23.sql` adds generic ingest provenance tables. Runtime wiring in
this slice is Telegram Takeout-only; normal `sync_source` does not write these
tables yet.

`ingest_batches` stores one durable row per actually-started locked ingest
attempt. `status` is persisted as `running`, `completed`, `failed`, or
`cancelled`; crash-interrupted imports remain `running` and can be interpreted
by query/UI code as interrupted when no in-memory job exists after restart.

`completeness` is separate from status. A completed zero-message traversal can
be `complete` when selected history traversal finished normally and no partial
flags were set. Only-my-messages fallback and migrated-history deferment are
`partial`.

`history_scope = migrated_small_group_history` identifies an explicit
historical import batch. It is a run-level scope, not a row-level migration
domain.

`item_observed_count` counts all item-level observation rows. It can be greater
than `item_inserted_count + item_duplicate_count + item_skipped_count` when
`outcome = 'failed'` rows exist, because there is no dedicated failed counter in
the foundation schema.

`telegram_takeout_batches.account_id` is a historical identity snapshot for the
Takeout run. The source/batch relationship owns provenance retention; deleting
an account must not delete the detail row while leaving the generic batch row.

`ingest_item_observations.provider_identity` is a generic text identity. For
Telegram it uses `telegram:history_peer:<kind>:<id>:message:<message_id>`,
where the peer is the message history peer from `telegram_messages`, not the
current resolved source peer.

Warning messages and terminal errors are bounded and sanitized. They must not
store raw Telegram TL payloads, session data, auth material, cookies, headers,
or compressed payload dumps.

### 1.5 `youtube_video_sources`

Stores typed runtime metadata for direct YouTube video sources. Generic identity
and display snapshot fields remain in `sources`; provider-specific title,
channel, thumbnail, canonical URL, availability, description, and provider-work
hints live here.

Important fields:

- `source_id`
- `video_id`
- `canonical_url`
- `title`
- `channel_title`
- `channel_id`
- `channel_handle`
- `channel_url`
- `author_display`
- `published_at`
- `duration_seconds`
- `description`
- `thumbnail_url`
- `tags_json`
- `chapters_json`
- `video_form`
- `availability_status`
- `caption_language_override`
- `raw_metadata_version`
- `raw_metadata_zstd`

Notes:

- `source_id` references `sources(id)` with `ON DELETE CASCADE`.
- `video_id` must match `sources.external_id`; Rust upsert/backfill code
  validates this cross-table invariant.
- `raw_metadata_zstd` is optional archive/debug/reparse/migration payload only.
  Normal listing, detail, jobs, and analysis do not decode it.

### 1.6 `youtube_playlist_sources`

Stores typed runtime metadata for YouTube playlist sources.

Important fields:

- `source_id`
- `playlist_id`
- `canonical_url`
- `title`
- `channel_title`
- `channel_id`
- `channel_handle`
- `channel_url`
- `thumbnail_url`
- `video_count`
- `availability_status`
- `raw_metadata_version`
- `raw_metadata_zstd`

Notes:

- `playlist_id` must match `sources.external_id`; Rust upsert/backfill code
  validates this cross-table invariant.
- Playlist entry payloads remain in `youtube_playlist_items.metadata_zstd`.

### 1.7 `source_identity_repair_notes`

Stores non-fatal source identity repair notes for diagnostics.

Important fields:

- `id`
- `source_id`
- `issue_code`
- `detail`
- `created_at`

Important constraints / indexes:

- `source_id` foreign key to `sources(id)` with `ON DELETE CASCADE`
- unique note by `(source_id, issue_code)`

Notes:

- fatal identity problems stop startup repair and block source commands with a
  typed repair error.
- notes are for non-fatal enrichment gaps only; duplicate or malformed identity
  rows are not silently downgraded into notes.

### 1.8 `items`

Stores locally ingested source items. Current rows include Telegram messages,
YouTube transcript items, and YouTube comment items. The table remains the
shared local corpus for provider documents.

Important fields:

- `id`
- `source_id`
- `external_id`
- `author`
- `published_at`
- `ingested_at`
- `content_zstd`
- `raw_data_zstd`
- `content_kind`
- `item_kind`
- `has_media`
- `media_kind`
- `media_metadata_zstd`
- `reply_to_msg_id`
- `reply_to_peer_kind`
- `reply_to_peer_id`
- `reply_to_top_id`
- `reaction_count`

`content_kind` values:

- `text_only`
- `text_with_media`
- `media_only`

`item_kind` values currently written by ingest:

- `telegram_message`
- `youtube_transcript`
- `youtube_comment`

Notes:

- rows may have text, media metadata, or both;
- rows without both text and useful media metadata are skipped during ingest.
- rows can be inserted by normal `sync_source` or by Takeout import;
- Takeout item rows are correlated to durable ingest batches through
  `ingest_item_observations`, not through a column on `items`;
- Telegram duplicate detection now uses `telegram_messages`, not
  `(source_id, external_id)`.
- `items.external_id` remains a compatibility/display/debug value for Telegram
  messages and is still populated with the Telegram message id string.
- Telegram context fields are nullable and are populated when Telegram exposes
  that metadata;
- `NULL` in Telegram context fields means metadata is unavailable, the row
  predates the metadata capture path, or Telegram did not expose that value;
- `reply_to_peer_kind` uses Telegram peer values (`user`, `chat`, `channel`), not Extractum source-kind values (`channel`, `supergroup`, `group`);
- `reaction_count = 0` means Telegram explicitly exposed zero aggregate reactions; `NULL` means the app cannot distinguish zero from unavailable metadata.

Important constraints / indexes:

- non-Telegram item uniqueness by `(source_id, external_id)` where
  `item_kind <> 'telegram_message'`
- non-unique compatibility lookup index on `(source_id, external_id)`
- browse index on `(source_id, published_at DESC)`
- provider item-kind browse index on `(source_id, item_kind, published_at DESC)`
- author index on `author`

Takeout implication:

- repeated Takeout runs, or a Takeout run after normal sync, rely on
  `telegram_messages` native identity handling to skip Telegram duplicates;
- migrated supergroup history has a typed identity boundary, but enabling full
  migrated-history import still requires a separate validation slice.
- Telegram Takeout writes generic ingest batch rows, Telegram Takeout batch
  detail, warnings, and item observations; it does not persist raw Telegram
  payloads as provenance.

YouTube implication:

- one synced transcript is stored as a `youtube_transcript` item, while its timestamped cues live in `youtube_transcript_segments`;
- comments are stored as `youtube_comment` items;
- YouTube description text used by analysis is synthesized from typed source
  metadata and is not stored as an `items` row.

### 1.9 `app_settings`

Simple key/value storage for app-wide settings.

Currently used for:

- active LLM profile selection
- LLM provider profile metadata
- initial sync policy settings
- legacy `llm.profile.<profile_id>.api_key` rows only as migration inputs when present

Known active keys include:

- `llm.active_provider_profile`
- `llm.profile.<profile_id>.provider`
- `llm.profile.<profile_id>.default_model`
- `llm.profile.<profile_id>.base_url`
- `sync.initial.mode`
- `sync.initial.value`

Saved LLM API keys live in OS secure storage under
`llm.profile.<profile_id>.api_key`; the backend migrates old non-empty
`app_settings` key rows after a successful secure-store write.

### 1.10 `telegram_forum_topics`

Stores the local catalog of Telegram forum topics for `supergroup` sources.

Important fields:

- `id`
- `source_id`
- `topic_id`
- `top_message_id`
- `title`
- `icon_color`
- `icon_emoji_id`
- `is_closed`
- `is_pinned`
- `is_hidden`
- `is_deleted`
- `sort_order`
- `last_seen_at`
- `updated_at`

Important constraints / indexes:

- unique topic by `(source_id, topic_id)`
- join index on `(source_id, top_message_id)`
- topic join/filter index on `items(source_id, reply_to_top_id)`
- `source_id` foreign key to `sources(id)` with `ON DELETE CASCADE`

Notes:

- `topic_id` is the stable Telegram topic identifier used by API/DTO layers;
- `top_message_id` is the Telegram root message id for the topic and is still useful metadata, but it is not the primary join key for ordinary topic messages;
- `items.reply_to_top_id` must be interpreted as the forum topic identifier for ordinary topic messages, so the primary local join is `items.reply_to_top_id -> telegram_forum_topics.topic_id`;
- `top_message_id` is only needed as a root-message fallback when the stored message itself is the topic root and therefore has no `reply_to_top_id`; that fallback first uses `telegram_messages.telegram_message_id`, while the old `CAST(items.external_id AS INTEGER)` path remains only for legacy Telegram rows that were not backfilled into `telegram_messages`;
- if `reply_to_top_id` is missing but `reply_to_msg_id = topic_id`, the row still belongs to that forum topic; this mirrors Telegram Desktop's `reply_to_top_id` / `reply_to_msg_id` fallback when deriving the topic root id;
- if no specific topic match is found and the catalog contains the real Telegram `General` topic (`topic_id = 1`), messages without explicit topic metadata are attached to that real topic;
- rows that still have no match after the full resolver go to the synthetic `Unrecognized topic` bucket; this bucket is intentionally separate from `General`;
- this distinction matters in production data: many Telegram forum messages carry `reply_to_top_id = topic_id`, not `reply_to_top_id = top_message_id`, and some omit `reply_to_top_id` while keeping `reply_to_msg_id = topic_id`, so treating `top_message_id` as the normal join key or skipping the fallbacks misclassifies topic traffic;
- topic records are retained locally even if a later catalog refresh omits them, so historical message-to-topic matches can survive.

### 1.11 `item_topic_memberships`

Stores materialized Telegram forum topic memberships for items.

Important fields:

- `item_id`
- `source_id`
- `topic_id`
- `match_kind`
- `resolver_version`
- `created_at`
- `updated_at`

Important constraints / indexes:

- one membership row per `item_id`
- `source_id` foreign key to `sources(id)` with `ON DELETE CASCADE`
- composite foreign key `(source_id, topic_id)` to
  `telegram_forum_topics(source_id, topic_id)` with `ON DELETE CASCADE`
- lookup indexes on `(source_id, topic_id)` and `(source_id, item_id)`
- `match_kind` is constrained to the supported resolver paths
- `resolver_version` must be positive

Notes:

- `item_topic_memberships` stores only real Telegram forum topic memberships.
- `Unrecognized topic` is a derived bucket for ready/current resolution state
  and is not persisted as a topic or membership row.
- Reader truth is source-level `telegram_topic_resolution_state`. Row-level
  `item_topic_memberships.resolver_version` is diagnostic and must match state
  version for ready sources.
- Full rebuilds delete and reinsert source memberships, so stale row-level
  resolver versions are cleared at the correctness boundary.

### 1.12 `telegram_topic_resolution_state`

Stores source-level state for Telegram forum topic membership materialization.

Important fields:

- `source_id`
- `resolver_version`
- `catalog_refreshed_at`
- `memberships_refreshed_at`
- `status`
- `unresolved_count`
- `pending_item_count`
- `last_error`
- `updated_at`

Important constraints / indexes:

- one state row per `source_id`
- `source_id` foreign key to `sources(id)` with `ON DELETE CASCADE`
- `resolver_version` must be positive
- `status` is constrained to `never_run`, `ready`, `dirty`, `rebuilding`, or
  `failed`
- unresolved and pending counts must be non-negative

Notes:

- `telegram_topic_resolution_state` rows are valid only for Telegram
  supergroup sources.
- Missing state is treated defensively as `never_run`.
- Missing membership means derived `Unrecognized topic` only when state is
  `ready` and current for the backend resolver version.
- `last_error` is bounded and is not a raw payload log.

### 1.13 `youtube_playlist_items`

Stores playlist membership rows and per-entry availability state.

Important fields:

- `id`
- `playlist_source_id`
- `video_source_id`
- `video_id`
- `position`
- `title_snapshot`
- `url`
- `thumbnail_url`
- `availability_status`
- `is_removed_from_playlist`
- `last_seen_at`
- `metadata_zstd`
- `created_at`
- `updated_at`

Important constraints / indexes:

- unique playlist entry by `(playlist_source_id, video_id)`
- ordering index on `(playlist_source_id, position)`
- lookup indexes on `video_source_id` and `video_id`

Notes:

- `playlist_source_id` points to the YouTube playlist source row.
- `video_source_id` points to a linked YouTube video source when the entry is available and has been materialized locally; unavailable/unlinked entries keep `NULL`.
- `availability_status` distinguishes available, upcoming, live, no-captions, auth-gated, deleted, removed, and unknown-unavailable rows.
- `is_removed_from_playlist` marks rows that disappeared from a later playlist metadata sync without deleting historical local state.
- Playlist entry rows intentionally remain typed YouTube membership/detail
  state. They are not materialized into `archive_read_items` because they are
  list state, not archived content items.

### 1.14 `youtube_transcript_segments`

Stores timestamped transcript segments for `youtube_transcript` items.

Important fields:

- `id`
- `item_id`
- `source_id`
- `segment_index`
- `start_ms`
- `end_ms`
- `text`
- `chapter_index`
- `caption_language`
- `caption_track_kind`
- `is_auto_generated`
- `metadata_zstd`

Important constraints / indexes:

- unique segment by `(item_id, segment_index)`
- segment time index on `(item_id, start_ms)`
- source lookup index on `source_id`

Notes:

- `caption_track_kind` records the selected caption track class, such as manual or auto.
- `is_auto_generated` preserves whether the selected track came from auto captions.
- Analysis trace refs can resolve segment timestamps into YouTube links.

### 1.15 `archive_read_model_state`

Source-scoped readiness gate for the provider-neutral archive/read UI model.
`items` and typed provider tables remain canonical; this table only decides
whether consumers may use derived archive rows for a source.

Important fields:

- `source_id`
- `model_version`
- `status`
- `built_at`
- `item_count`
- `row_count`
- `last_error`
- `updated_at`

Status values:

- `never_built`
- `building`
- `ready`
- `stale`
- `failed`

Notes:

- `source_id` is both the source scope and primary key.
- `model_version` stores the archive builder contract version used for this
  source. Consumers may use derived archive rows only when the source state is
  `ready` and `model_version` matches the current builder.
- `built_at` records the successful ready build timestamp.
- `item_count` and `row_count` are rebuild accounting fields for diagnostics.
- `last_error` stores bounded rebuild/backfill failure text, not canonical
  provider data.
- Missing, stale, failed, building, or old-version state keeps gated consumers
  on the canonical provider/archive path.

### 1.16 `archive_read_items`

Provider-neutral item-level archive rows for archive UI consumers such as source
browsing and Telegram NotebookLM export. The table duplicates compressed text
and compressed media metadata needed for display, but does not duplicate
`items.raw_data_zstd`; raw payload availability is represented by
`has_raw_data`.

Important fields:

- `source_id`
- `item_id`
- `ref`
- `external_id`
- `item_kind`
- `author`
- `published_at`
- `content_kind`
- `has_media`
- `media_kind`
- `content_zstd`
- `media_metadata_zstd`
- `has_raw_data`
- `forum_topic_id`
- `forum_topic_title`
- `forum_topic_top_message_id`
- `reply_to_msg_id`
- `reply_to_peer_kind`
- `reply_to_peer_id`
- `reply_to_top_id`
- `reaction_count`
- `model_version`
- `built_at`

Important constraints / indexes:

- primary key by `(source_id, item_id)`
- unique local item ref by `ref`
- source chronological lookup by `(source_id, published_at DESC, item_id DESC)`
- source/topic chronological lookup by
  `(source_id, forum_topic_id, published_at DESC, item_id DESC)`
- `source_id` and `item_id` reference canonical rows with `ON DELETE CASCADE`

Notes:

- canonical truth remains in `items`, `telegram_messages`,
  `item_topic_memberships`, `telegram_forum_topics`, and typed YouTube tables;
- this table is rebuildable derived state;
- source browsing reads these rows only through the readiness gate in
  `archive_read_model_state`;
- normal single Telegram item writes maintain ready archive rows in the same
  transaction;
- bulk ingest and YouTube refresh paths mark source archive rows stale at the
  source scope instead of rebuilding every row inline;
- source browsing and Telegram NotebookLM export use the gated archive/read UI
  model when `archive_read_model_state` is ready and current;
- for missing, building, stale, failed, or old-version archive states,
  NotebookLM export preserves the existing local provider/archive items path;
- once NotebookLM export selects the archive loader, archive row decode and
  invariant failures are surfaced as errors rather than silently falling back.

### 1.17 `accounts`

Stores configured Telegram accounts.

Important fields:

- `id`
- `label`
- `api_id`
- `api_hash`
- `phone`
- `created_at`

Notes:

- `api_hash` is retained as a legacy `NOT NULL` placeholder column and is empty for newly created or migrated accounts;
- saved Telegram `api_hash` values live in OS secure storage under `telegram.account.<account_id>.api_hash`;
- session restore state is not stored in this table and instead lives in per-account app-data session files whose contents are encrypted with keys stored in OS secure storage under `telegram.account.<account_id>.session_key`.

## 2. Analysis tables

### 2.1 `analysis_prompt_templates`

Stores saved report prompt templates.

Important fields:

- `id`
- `name`
- `template_kind`
- `body`
- `version`
- `is_builtin`
- `created_at`
- `updated_at`

### 2.2 `analysis_runs`

Stores saved report runs.

Important fields:

- `id`
- `run_type`
- `scope_type`
- `source_id`
- `source_group_id`
- `period_from`
- `period_to`
- `output_language`
- `prompt_template_id`
- `prompt_template_version`
- `provider_profile`
- `provider`
- `model`
- `youtube_corpus_mode`
- `status`
- `result_markdown`
- `trace_data_zstd`
- `scope_label_snapshot`
- `snapshot_captured_at`
- `snapshot_error`
- `error`
- `created_at`
- `completed_at`

Notes:

- `snapshot_captured_at` is set after a report run's frozen corpus has been
  persisted to `analysis_run_messages`, reloaded, and verified as usable before
  provider execution.
- `snapshot_error` is a bounded sanitized error category for
  capture-preventing failures only. Provider/model/auth/network failures after
  successful capture remain in `error` and do not populate `snapshot_error`.

### 2.3 `analysis_source_groups`

Named source groups for reusable analysis scope.

Important YouTube-related fields:

- `source_type`

Notes:

- `source_type` is `telegram` by default for existing groups.
- New groups can be `telegram` or `youtube`; mixed-provider group membership is rejected.

### 2.4 `analysis_source_group_members`

Join table between groups and sources.

### 2.5 `analysis_chat_messages`

Stores follow-up chat exchanges for a saved run.

Important fields:

- `id`
- `run_id`
- `role`
- `content`
- `created_at`

### 2.6 `analysis_documents`

`analysis_documents` is a provider-neutral materialized read model for live
analysis corpus loading. Provider/archive truth remains in `items` plus typed
provider tables such as `telegram_messages`, `youtube_video_sources`,
`youtube_playlist_sources`, and `youtube_transcript_segments`.

Important fields:

- `id`
- `source_id`
- `item_id`
- `document_key`
- `document_kind`
- `source_type`
- `source_subtype`
- `external_id`
- `author`
- `published_at`
- `document_order`
- `ref`
- `content_zstd`
- `metadata_zstd`
- `created_at`
- `updated_at`

Document kinds:

- `telegram_message`
- `youtube_transcript`
- `youtube_comment`
- `youtube_description`

Important constraints / indexes:

- unique document identity by `(source_id, document_key)`
- source/date lookup by `(source_id, published_at, document_order, id)`
- kind/source/date lookup by
  `(document_kind, source_id, published_at, document_order, id)`
- ref lookup by `ref`

Notes:

- the table is rebuildable from provider/archive truth;
- runtime writers maintain it synchronously for Telegram messages, YouTube
  comments, YouTube transcript segment rows, and YouTube video descriptions;
- source browsing and Telegram NotebookLM export use the gated archive/read UI
  model when `archive_read_model_state` is ready and current; NotebookLM export
  preserves the existing local provider/archive items path for non-ready
  archive states and surfaces archive-loader failures after selection;
- `document_order` is the numeric order key inside one
  `(published_at, source_id)` bucket;
- the live corpus reader orders by
  `published_at ASC, source_id ASC, document_order ASC, id ASC` and does not
  use `ref` as an ordering tie-breaker;
- item-backed documents use `item:<item_id>` keys and public refs shaped like
  `s<source_id>-i<item_id>`;
- YouTube transcript segment documents use
  `item:<item_id>:segment:<segment_index>` keys and refs shaped like
  `s<source_id>-i<item_id>@<start_ms>ms`;
- synthetic YouTube descriptions use the source-scoped key
  `youtube:description` and ref `s<source_id>-i0`;
- `analysis_documents.id` is internal and is not a public evidence ref;
- `ref` is not unique, so scoped callers must pair it with source, kind, run,
  or another owned boundary.

### 2.7 `analysis_run_messages`

Stores the frozen corpus snapshot for a saved run.

Important fields:

- `run_id`
- `item_id`
- `source_id`
- `external_id`
- `author`
- `published_at`
- `ref`
- `content_zstd`
- `item_kind`
- `source_type`
- `source_subtype`
- `metadata_zstd`

Purpose:

- preserve the exact text corpus used by the run;
- stabilize follow-up chat and trace resolution;
- preserve effective source-group membership for the run.
- preserve YouTube corpus metadata needed for timestamp evidence refs and synthetic description refs.

For new report runs, `analysis_run_messages` is captured before provider
execution and is the authoritative saved-run corpus for provider prompts, trace
building, evidence resolution, saved-run source context, and follow-up chat.
Completed historical runs without rows are treated as missing legacy snapshots;
saved-run read paths must not reconstruct them from current live sources.

## 3. Migration history

Baseline v1 (`0001_current_schema_baseline.sql`) is the active starting point
for supported databases. Pre-reset migrations 1 through 26 are archived under
`docs/archive/migrations-pre-baseline-reset/` and are not an automatic upgrade
path. See `docs/archive/migrations-pre-baseline-reset/README.md` for the
archive boundary. Future migrations start at `0002`.

The application performs a one-time baseline-history cutover for the one
controlled pre-reset database. The cutover validates old successful migration
history through version 26, creates a mandatory backup beside the database,
then rewrites only `_sqlx_migrations` to baseline v1 in one transaction.
Product tables are not modified.

| Version | File | Purpose |
| --- | --- | --- |
| 1 | `0001_current_schema_baseline.sql` | Current supported schema baseline |

## 4. Current behavior implications

- the analysis workspace can render media-bearing and media-only items from `items`;
- Telegram and YouTube source creation/sync are implemented; unsupported provider sync attempts return typed validation errors;
- `/analysis` loads live corpus text from provider-neutral
  `analysis_documents` rows according to the selected YouTube corpus mode;
- playlist analysis expands linked `youtube_playlist_items.video_source_id` rows and skips unavailable/unlinked playlist rows;
- YouTube source jobs are in-memory and are not resumed after app restart;
- YouTube source runtime metadata is read from typed YouTube source tables;
  `sources.metadata_zstd` is not the owner of YouTube runtime metadata;
- YouTube auth cookies are stored in OS secure storage, not SQLite;
- source browsing now uses `archive_read_items` only when
  `archive_read_model_state` is `ready` for the current builder version; all
  other states fall back to the canonical `items` plus topic joins path;
- Telegram NotebookLM export now uses `archive_read_items` only when
  `archive_read_model_state` is `ready` for the current builder version; all
  non-ready states preserve the existing local provider/archive items path, and
  archive-loader failures after selection are surfaced as errors rather than
  silently falling back;
- Takeout import fills the same `items` fields as normal sync where raw TL data exposes enough metadata;
- Telegram Takeout import persists durable ingest-batch provenance after the
  same-source ingest lock is acquired; failed or cancelled imports can leave
  partial item rows without advancing `sources.last_sync_state`;
- Telegram duplicate detection and legacy message-ref resolution use typed
  `telegram_messages` identity where available, keeping `items.external_id` as
  a compatibility value rather than the owner of Telegram message identity;
- Telegram forum topic readers use materialized `item_topic_memberships` plus
  source-level `telegram_topic_resolution_state`; Telegram NotebookLM export
  preserves that behavior through the fallback items path and, for
  ready/current archive sources, reads the materialized topic fields copied into
  `archive_read_items`. `Unrecognized topic` remains derived UI and export
  state, not a stored topic row;
- `analysis_runs.provider_profile` preserves the user-facing LLM profile id used for a run;
- `analysis_runs.youtube_corpus_mode` preserves the selected YouTube corpus scope used by the run, rather than reconstructing it from current source defaults.
- new saved analysis runs capture `analysis_run_messages` before provider
  execution and expose explicit snapshot state on run DTOs;
- saved analysis runs now use `analysis_run_messages` rather than live `items`
  for saved-run corpus, evidence resolution, saved-run source context, and
  follow-up chat;
- completed snapshotless saved runs keep report output readable but do not silently resolve source/evidence/chat against live `items`;
- new live analysis refs use local item identity (`s{source_id}-i{item_id}`);
- YouTube transcript refs can include timestamp suffixes and resolve to canonical YouTube URLs with `t=` parameters;
- legacy saved refs using Telegram message ids (`s{source_id}-m{message_id}`) remain readable;
- saved LLM API keys and Telegram `api_hash` values are owned by OS secure storage, not SQLite;
- old non-empty `llm.profile.*.api_key` and `accounts.api_hash` values are legacy migration inputs and are cleared only after successful secure-store writes;
- Telegram session files remain app-data files, but their contents are encrypted with per-account session keys stored in OS secure storage under `telegram.account.<account_id>.session_key`.
