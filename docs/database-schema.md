# Database Schema

This document describes the current local SQLite schema at a practical level.

## 1. Core tables

### 1.1 `sources`

Stores registered Telegram sources.

Important fields:

- `id`
- `source_type`
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

`telegram_source_kind` values:

- `channel`
- `supergroup`
- `group`

Notes:

- older rows that used `source_type = 'telegram_channel'` are migrated to `source_type = 'telegram'`;
- uniqueness includes `account_id` because the same Telegram source can be added from multiple local accounts;
- uniqueness includes `telegram_source_kind` because Telegram bare ids are not enough to safely describe every peer shape.

### 1.2 `items`

Stores synced Telegram messages.

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

`content_kind` values:

- `text_only`
- `text_with_media`
- `media_only`

Notes:

- rows may have text, media metadata, or both;
- rows without both text and useful media metadata are skipped during ingest.

Important constraints / indexes:

- unique item by `(source_id, external_id)`
- browse index on `(source_id, published_at DESC)`
- author index on `author`

### 1.3 `app_settings`

Simple key/value storage for app-wide settings.

Currently used for:

- LLM provider settings
- temporary Gemini API key storage
- initial sync policy settings

Known active keys include:

- `sync.initial.mode`
- `sync.initial.value`

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

## 4. Current behavior implications

- `/sources` can render media-bearing and media-only items from `items`;
- `/analysis` still loads only text-bearing corpus rows;
- saved analysis runs now prefer `analysis_run_messages` over live `items`;
- `app_settings` still contains secrets temporarily, which remains a security debt.
