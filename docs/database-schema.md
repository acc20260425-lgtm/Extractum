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

- older rows that used `source_type = 'telegram_channel'` are migrated to `source_type = 'telegram'`;
- migration `15.sql` adds `source_subtype` and backfills existing Telegram rows
  from the legacy Telegram subtype mirror;
- migration `18.sql` adds typed Telegram identity tables and a startup repair
  creates the canonical Telegram unique index only after duplicate preflight;
- migration `19.sql` is runner-managed by Rust and removes the old Telegram
  subtype compatibility mirror from the current `sources` schema;
- Telegram source subtype is canonical in `sources.source_subtype`;
- Telegram operational peer identity lives in `telegram_sources`.
- new Telegram source rows keep `metadata_zstd` `NULL`; old Telegram blobs may
  remain in existing databases as legacy repair input until a separate cleanup
  decision;
- normal Telegram runtime updates preserve old Telegram blobs rather than
  clearing them opportunistically;
- normal Telegram sync, Takeout, forum topic refresh, source list display, and
  source resolution use typed identity and display cache fields in
  `telegram_sources`, not Telegram source metadata blobs;
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
- `migration_domain` is diagnostic/future-proofing metadata in this slice and
  is not used for duplicate detection, topic matching, or ref resolution.
- `telegram_messages.source_id` must equal `items.source_id`, and
  `telegram_messages.item_id` must point to an item whose `item_kind` is
  `telegram_message`; migration/runtime tests enforce this application
  invariant.
- `updated_at` is set on child-row creation in this slice. Duplicate skips do
  not update `telegram_messages.updated_at`.

### 1.4 `youtube_video_sources`

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

### 1.5 `youtube_playlist_sources`

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

### 1.6 `source_identity_repair_notes`

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

### 1.7 `items`

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
- Takeout import does not add a separate provenance column, ingest-batch table,
  or archive table yet;
- Telegram duplicate detection now uses `telegram_messages`, not
  `(source_id, external_id)`.
- `items.external_id` remains a compatibility/display/debug value for Telegram
  messages and is still populated with the Telegram message id string.
- Telegram context fields are nullable and are populated only for rows inserted after migration `13.sql` and the updated ingest code;
- `NULL` in Telegram context fields means metadata is unavailable, the row predates the migration, or Telegram did not expose that value;
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
  migrated-history import still requires the separate Takeout provenance and
  validation slice.
- the recommended provenance direction is a generic ingest-batch table plus
  Telegram Takeout batch details and item origin/observation rows, not raw
  Telegram payload storage.

YouTube implication:

- one synced transcript is stored as a `youtube_transcript` item, while its timestamped cues live in `youtube_transcript_segments`;
- comments are stored as `youtube_comment` items;
- YouTube description text used by analysis is synthesized from typed source
  metadata and is not stored as an `items` row.

### 1.8 `app_settings`

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

### 1.9 `telegram_forum_topics`

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

### 1.10 `item_topic_memberships`

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

### 1.11 `telegram_topic_resolution_state`

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

### 1.12 `youtube_playlist_items`

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

### 1.13 `youtube_transcript_segments`

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

### 1.14 `accounts`

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
- `error`
- `created_at`
- `completed_at`

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

### 2.6 `analysis_run_messages`

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

## 3. Migration history

| Version | File | Purpose |
| --- | --- | --- |
| 1 | `1.sql` | Initialize `sources`, `items`, `app_settings` |
| 2 | `2.sql` | Add `is_member` to `sources` |
| 3 | `3.sql` | Add `accounts` table |
| 4 | `4.sql` | Add `last_synced_at` to `sources` |
| 5 | `5.sql` | Add analysis templates and runs |
| 6 | `6.sql` | Add analysis source groups |
| 7 | `7.sql` | Add `source_group_id` to `analysis_runs` |
| 8 | `8.sql` | Add analysis chat history |
| 9 | `9.sql` | Add media-aware metadata to `items` |
| 10 | `10.sql` | Add saved run snapshot storage |
| 11 | `11.sql` | Add `telegram_source_kind` and migrate Telegram channels to generic Telegram sources |
| 12 | `12.sql` | Scope source uniqueness by `account_id` |
| 13 | `13.sql` | Add Telegram reply/thread/reaction context metadata to `items` |
| 14 | `14.sql` | Add local `telegram_forum_topics` catalog and topic join indexes |
| 15 | `15.sql` | Add provider-local `source_subtype` to `sources` and backfill Telegram rows |
| 16 | `16.sql` | Add YouTube source foundation, item kinds, playlist rows, transcript segments, YouTube analysis snapshot metadata, source-group provider type, and YouTube settings defaults |
| 17 | `17.sql` | Add durable YouTube corpus mode metadata to `analysis_runs` |
| 18 | `18.sql` | Add source identity bridge tables, safe Telegram subtype backfills, and repair diagnostics storage |
| 19 | `19.sql` | Runner-managed rebuild of `sources` without `telegram_source_kind`; records the sentinel checksum for SQLx history |
| 20 | `20.sql` | Runner-managed creation and backfill of typed YouTube video/playlist source metadata tables |
| 21 | `21.sql` | Runner-managed creation/backfill of typed Telegram message identity rows and replacement item uniqueness |
| 22 | `22.sql` | Runner-managed creation and rebuild of Telegram forum topic membership materialization tables |

## 4. Current behavior implications

- the analysis workspace can render media-bearing and media-only items from `items`;
- Telegram and YouTube source creation/sync are implemented; unsupported provider sync attempts return typed validation errors;
- `/analysis` loads text-bearing Telegram rows and YouTube transcript/comment/description corpus rows according to the selected YouTube corpus mode;
- playlist analysis expands linked `youtube_playlist_items.video_source_id` rows and skips unavailable/unlinked playlist rows;
- YouTube source jobs are in-memory and are not resumed after app restart;
- YouTube source runtime metadata is read from typed YouTube source tables;
  `sources.metadata_zstd` is not the owner of YouTube runtime metadata;
- YouTube auth cookies are stored in OS secure storage, not SQLite;
- NotebookLM export can render local reply snippets, thread ids, reply peer ids, and reaction counts when those nullable `items` fields are present;
- Takeout import fills the same `items` fields as normal sync where raw TL data exposes enough metadata;
- Takeout import does not yet persist durable ingest-batch provenance; failed
  or cancelled imports can leave partial item rows without advancing
  `sources.last_sync_state`;
- Telegram duplicate detection and legacy message-ref resolution use typed
  `telegram_messages` identity where available, keeping `items.external_id` as
  a compatibility value rather than the owner of Telegram message identity;
- Telegram forum topic readers and NotebookLM export use materialized
  `item_topic_memberships` plus source-level
  `telegram_topic_resolution_state`; `Unrecognized topic` remains derived UI
  and export state, not a stored topic row;
- `analysis_runs.provider_profile` preserves the user-facing LLM profile id used for a run;
- `analysis_runs.youtube_corpus_mode` preserves the selected YouTube corpus scope used by the run, rather than reconstructing it from current source defaults.
- saved analysis runs now prefer `analysis_run_messages` over live `items`;
- completed snapshotless saved runs keep report output readable but do not silently resolve source/evidence/chat against live `items`;
- new live analysis refs use local item identity (`s{source_id}-i{item_id}`);
- YouTube transcript refs can include timestamp suffixes and resolve to canonical YouTube URLs with `t=` parameters;
- legacy saved refs using Telegram message ids (`s{source_id}-m{message_id}`) remain readable;
- saved LLM API keys and Telegram `api_hash` values are owned by OS secure storage, not SQLite;
- old non-empty `llm.profile.*.api_key` and `accounts.api_hash` values are legacy migration inputs and are cleared only after successful secure-store writes;
- Telegram session files remain app-data files, but their contents are encrypted with per-account session keys stored in OS secure storage under `telegram.account.<account_id>.session_key`.
