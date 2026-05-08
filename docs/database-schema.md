# Database Schema

This document describes the current local SQLite schema at a practical level.

## 1. Core tables

### 1.1 `sources`

Stores registered provider sources. Telegram is the only implemented ingest
provider today, but the shared schema is provider-ready.

Important fields:

- `id`
- `source_type`
- `source_subtype`
- `telegram_source_kind`
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

- unique source by `(account_id, source_type, telegram_source_kind, external_id)`

`source_type` values currently supported by shared contracts:

- `telegram`
- `youtube`
- `rss`
- `forum`

Only `telegram` has implemented ingest today.

`source_subtype` is provider-local:

- Telegram uses `channel`, `supergroup`, or `group`
- future YouTube can use `video` or `playlist`
- future RSS can use `feed`
- future forums can use `thread`, `board`, or `site`

`telegram_source_kind` values:

- `channel`
- `supergroup`
- `group`

Notes:

- older rows that used `source_type = 'telegram_channel'` are migrated to `source_type = 'telegram'`;
- migration `15.sql` adds `source_subtype` and backfills existing Telegram rows
  from `telegram_source_kind`;
- `telegram_source_kind` is a Telegram compatibility field and can be `NULL` for
  future non-Telegram sources;
- uniqueness includes `account_id` because the same Telegram source can be added from multiple local accounts;
- uniqueness includes `telegram_source_kind` because Telegram bare ids are not enough to safely describe every peer shape.
- `last_sync_state` and `last_synced_at` are advanced by normal sync and by successful Takeout import; failed or cancelled Takeout jobs leave these fields unchanged.

### 1.2 `items`

Stores locally ingested source items. Current rows are Telegram messages, but
the table is the shared local corpus for future provider documents.

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

Notes:

- rows may have text, media metadata, or both;
- rows without both text and useful media metadata are skipped during ingest.
- rows can be inserted by normal `sync_source` or by Takeout import;
- Takeout import does not add a separate provenance column and does not create a separate archive table;
- Telegram context fields are nullable and are populated only for rows inserted after migration `13.sql` and the updated ingest code;
- `NULL` in Telegram context fields means metadata is unavailable, the row predates the migration, or Telegram did not expose that value;
- `reply_to_peer_kind` uses Telegram peer values (`user`, `chat`, `channel`), not Extractum source-kind values (`channel`, `supergroup`, `group`);
- `reaction_count = 0` means Telegram explicitly exposed zero aggregate reactions; `NULL` means the app cannot distinguish zero from unavailable metadata.

Important constraints / indexes:

- unique item by `(source_id, external_id)`
- browse index on `(source_id, published_at DESC)`
- author index on `author`

Takeout implication:

- repeated Takeout runs, or a Takeout run after normal sync, rely on `(source_id, external_id)` conflict handling to skip duplicates;
- migrated supergroup history is currently not inserted by Takeout because old small-group ids may collide with current supergroup ids under this key.

### 1.3 `app_settings`

Simple key/value storage for app-wide settings.

Currently used for:

- active LLM profile selection
- LLM provider profile metadata
- temporary LLM API key storage
- initial sync policy settings

Known active keys include:

- `llm.active_provider_profile`
- `llm.profile.<profile_id>.provider`
- `llm.profile.<profile_id>.default_model`
- `llm.profile.<profile_id>.base_url`
- `llm.profile.<profile_id>.api_key`
- `sync.initial.mode`
- `sync.initial.value`

### 1.4 `telegram_forum_topics`

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
- `top_message_id` is only needed as a root-message fallback when the stored message itself is the topic root and therefore has no `reply_to_top_id`; in that case the local match is `CAST(items.external_id AS INTEGER) = telegram_forum_topics.top_message_id`;
- if `reply_to_top_id` is missing but `reply_to_msg_id = topic_id`, the row still belongs to that forum topic; this mirrors Telegram Desktop's `reply_to_top_id` / `reply_to_msg_id` fallback when deriving the topic root id;
- if no specific topic match is found and the catalog contains the real Telegram `General` topic (`topic_id = 1`), messages without explicit topic metadata are attached to that real topic;
- rows that still have no match after the full resolver go to the synthetic `Unrecognized topic` bucket; this bucket is intentionally separate from `General`;
- this distinction matters in production data: many Telegram forum messages carry `reply_to_top_id = topic_id`, not `reply_to_top_id = top_message_id`, and some omit `reply_to_top_id` while keeping `reply_to_msg_id = topic_id`, so treating `top_message_id` as the normal join key or skipping the fallbacks misclassifies topic traffic;
- topic records are retained locally even if a later catalog refresh omits them, so historical message-to-topic matches can survive.

### 1.5 `accounts`

Stores configured Telegram accounts.

Important fields:

- `id`
- `label`
- `api_id`
- `api_hash`
- `phone`
- `created_at`

Notes:

- `api_hash` is still stored in SQLite today, which remains part of the current secret-storage debt;
- session restore state is not stored in this table and instead lives in the app's per-account session files.

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
- `status`
- `result_markdown`
- `trace_data_zstd`
- `scope_label_snapshot`
- `error`
- `created_at`
- `completed_at`

### 2.3 `analysis_source_groups`

Named source groups for reusable analysis scope.

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

Purpose:

- preserve the exact text corpus used by the run;
- stabilize follow-up chat and trace resolution;
- preserve effective source-group membership for the run.

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

## 4. Current behavior implications

- the analysis workspace can render media-bearing and media-only items from `items`;
- `sources` is provider-ready, but only Telegram source creation and sync are implemented today;
- unsupported provider sync attempts return typed validation errors;
- `/analysis` still loads only text-bearing corpus rows;
- NotebookLM export can render local reply snippets, thread ids, reply peer ids, and reaction counts when those nullable `items` fields are present;
- Takeout import fills the same `items` fields as normal sync where raw TL data exposes enough metadata;
- `analysis_runs.provider_profile` preserves the user-facing LLM profile id used for a run;
- saved analysis runs now prefer `analysis_run_messages` over live `items`;
- new live analysis refs use local item identity (`s{source_id}-i{item_id}`);
- legacy saved refs using Telegram message ids (`s{source_id}-m{message_id}`) remain readable;
- `app_settings` still contains secrets temporarily, which remains a security debt.
