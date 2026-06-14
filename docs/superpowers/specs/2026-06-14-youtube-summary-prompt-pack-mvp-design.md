# YouTube Summary Prompt Pack MVP Design

Date: 2026-06-14

Status: approved in design discussion. This document defines the first MVP slice
for `youtube_summary` as the first module of the new Prompt Pack architecture.

## Source Documents

- `docs/llm_interaction_spec.md`
- `docs/prompt-packs/prompt_pack_json_contract_v1_draft.md`
- `docs/prompt-packs/execution_model_graph_assembly_policy.md`
- `docs/prompt-packs/youtube_summary_pack_spec.md`
- `docs/prompt-packs/validation_rules.md`
- `docs/database-schema.md`

## Product Direction

`youtube_summary` is the first real Prompt Pack module. It must be designed as a
new LLM analysis architecture, not as an extension of the existing report
pipeline.

The existing `analysis_runs`, `analysis_run_messages`, `analysis_prompt_templates`,
`result_markdown`, and related report code remain the legacy report pipeline.
They are not the persistence root for this MVP.

The new persistence root is `prompt_pack_runs`. A new UI reads the
`prompt_pack_*` tables and treats Prompt Pack results as canonical structured
artifacts.

## Goals

- Create a new Prompt Pack run model independent from `analysis_runs`.
- Support `youtube_summary` for already synced YouTube video and playlist
  sources.
- Use a pipeline-skeleton backend: stages, snapshots, artifacts, validation,
  audit, and final result are modeled now, while only one real LLM stage is
  executed in MVP.
- Store the canonical Prompt Pack result JSON as the immutable contract artifact.
- Add normalized projection tables for UI navigation, filtering, and future
  search.
- Fully freeze run-local YouTube inputs so old runs do not depend on live source
  library changes.
- Support partial multi-video runs when some videos fail and others succeed.

## Non-Goals

- No URL ingest inside the `youtube_summary` flow. MVP accepts only already
  synced Library sources.
- No dependency on `analysis_runs` for the new run model.
- No full multi-stage graph assembly implementation in the first backend slice.
- No cross-video synthesis execution in MVP.
- No full prompt-pack editor UI in MVP.
- No automatic provider fallback.
- No manual editing of generated claims, evidence, or summaries.

## Major Decisions

1. Use a separate `prompt_pack_runs` root.
2. Use a pipeline-skeleton MVP.
3. Store canonical JSON plus normalized projection tables.
4. Fully snapshot source and material inputs per run.
5. Support YouTube video and playlist scopes.
6. Default to `control_preset = "standard"` and `evidence_mode = "standard"`.
7. Run one real LLM stage per video.
8. Treat cross-video synthesis as a skipped or not-implemented stage in MVP.
9. Use a spec-aware validator, not the full reference validator.
10. Use hybrid prompt-pack library storage: bundled assets seed DB versions, and
    runs snapshot the exact pack version they used.
11. Support both a new YouTube Summary screen and Library detail entry points.
12. Use existing LLM profiles in MVP, while storing per-stage resolved
    provider/model snapshots.
13. Make YouTube comments optional and default them off.
14. Require usable transcript for MVP `standard` runs.

## Architecture

The new backend contour:

```text
prompt_pack_runs
  -> run-local YouTube snapshots
  -> stage runs and artifacts
  -> canonical result JSON
  -> projection tables for UI
  -> validation findings
  -> audit and quarantine
```

Live Library data remains input only:

- `sources`
- `youtube_video_sources`
- `youtube_playlist_items`
- `youtube_transcript_segments`
- `items` for `youtube_comment`
- `analysis_documents` as an optional read-model helper

When a Prompt Pack run starts, the backend freezes every selected source and
material needed by the run. Later reads of old runs must use the run-local
snapshots, not live Library tables.

## Prompt Pack Library Storage

MVP uses hybrid storage.

Bundled Prompt Pack assets are shipped with the application and seeded into DB:

- pack metadata;
- pack versions;
- stage templates;
- schema assets;
- prompt render assets where needed.

Runtime uses DB pack versions. A run stores both a foreign key to the selected
pack version and a snapshot of the exact prompt/schema/config used.

### Seed and Update Policy

Bundled pack versions are immutable. The seed process treats
`(pack_id, pack_version)` as a semantic version identity and `content_hash` as
the byte-level identity of all bundled definition, prompt, schema, and render
assets for that version.

Rules:

- If `(pack_id, pack_version)` does not exist, seed inserts the pack version and
  all stage/schema assets.
- If `(pack_id, pack_version)` exists as `origin_kind = "bundled"` with the same
  `content_hash`, seed is idempotent. It may refresh non-semantic metadata such
  as `bundled_source_path` and `last_seeded_at`, but it must not rewrite
  definition, prompt, or schema payloads.
- If `(pack_id, pack_version)` exists with a different `content_hash`, seed must
  fail with a pack-version hash conflict. Bundled assets are not silently
  overwritten; the bundled pack must bump `pack_version` or the existing DB must
  be repaired explicitly.
- User-created drafts are never overwritten by bundled seed. Drafts should use a
  distinct `pack_version` or a non-bundled pack namespace; collision with a
  bundled `(pack_id, pack_version)` is a validation error even if the hash
  happens to match.
- Activation is explicit per bundled manifest. At most one version per pack is
  active, but versions absent from the current application bundle are not
  automatically deleted or archived.
- Downgrade is non-destructive. If the DB already contains a newer bundled pack
  version than the running application knows about, startup keeps it archived or
  inactive and does not delete it. New runs may only select pack versions whose
  stage/schema assets are present and whose lifecycle is allowed by the current
  runtime.
- Existing runs always read `pack_snapshot_json_zstd` and never require the
  library row to remain active.

Proposed tables:

```text
prompt_packs
prompt_pack_versions
prompt_pack_stage_templates
prompt_pack_schema_assets
```

### `prompt_packs`

Stores stable pack identity.

Important fields:

- `id`
- `pack_id` such as `youtube_summary`
- `display_name`
- `description`
- `is_builtin`
- `created_at`
- `updated_at`

### `prompt_pack_versions`

Stores versioned pack definitions.

Important fields:

- `id`
- `pack_id`
- `pack_version`
- `schema_version`
- `origin_kind`: `bundled` or `user`
- `lifecycle_status` such as `active`, `draft`, `archived`
- `definition_json_zstd`
- `bundled_source_path`
- `content_hash`
- `created_at`
- `activated_at`
- `last_seeded_at`

### `prompt_pack_stage_templates`

Stores stage-level prompt/template assets for a pack version.

Important fields:

- `id`
- `pack_version_id`
- `stage_name`
- `stage_order`
- `provider_family`
- `prompt_template_json_zstd`
- `input_schema_id`
- `output_schema_id`
- `content_hash`

### `prompt_pack_schema_assets`

Stores schema assets associated with a pack version.

Important fields:

- `id`
- `pack_version_id`
- `schema_id`
- `schema_kind`
- `relative_path`
- `schema_json_zstd`
- `content_hash`

## Run Storage

Proposed tables:

```text
prompt_pack_runs
prompt_pack_run_scopes
prompt_pack_run_source_snapshots
prompt_pack_run_source_origins
prompt_pack_run_material_snapshots
prompt_pack_stage_runs
prompt_pack_stage_artifacts
prompt_pack_results
prompt_pack_result_source_refs
prompt_pack_result_claims
prompt_pack_result_evidence
prompt_pack_result_claim_relations
prompt_pack_result_unknowns
prompt_pack_result_verification_tasks
prompt_pack_result_warnings
prompt_pack_result_limitations
prompt_pack_result_quality_flags
prompt_pack_result_audit_refs
prompt_pack_result_ref_edges
prompt_pack_result_validation_findings
prompt_pack_result_audit_events
prompt_pack_result_quarantine_artifacts
prompt_pack_youtube_videos
prompt_pack_youtube_segments
prompt_pack_youtube_key_points
prompt_pack_youtube_quotes
prompt_pack_youtube_action_items
prompt_pack_youtube_open_questions
prompt_pack_youtube_synthesis_items
```

### DB Integrity Contract

The migration should treat this as a relational contract, not only a list of
columns. Canonical JSON remains the source of truth, but DB constraints should
protect run ownership, pack-version consistency, and projection rebuildability.

Naming rules:

- `id` is always a SQLite row id.
- `result_row_id` is the foreign key to `prompt_pack_results.id`.
- Canonical JSON ids keep their semantic names: `result_id`, `source_ref_id`,
  `claim_id`, `evidence_id`, `relation_id`, `video_id`, and nested pack ids.
- Projection and validation tables should not use `result_id` as a DB foreign
  key name because it collides with canonical `result_id`.

Ownership rules:

- Library definition tables cascade from `prompt_packs`.
- Runs should restrict deletion of referenced pack versions, but preserve the
  exact `pack_snapshot_json_zstd` so run reads do not depend on mutable library
  assets.
- All run-owned rows cascade from `prompt_pack_runs` or
  `prompt_pack_results`.
- Links to live Library rows (`sources`, `items`, `youtube_transcript_segments`)
  are best-effort nullable links with `ON DELETE SET NULL`; frozen snapshots are
  authoritative after run creation.
- Compressed JSON/text fields are stored as `BLOB` containing zstd payloads.

Required constraints and indexes:

- `prompt_packs`: `PRIMARY KEY(id)`, `UNIQUE(pack_id)`,
  `CHECK(is_builtin IN (0, 1))`.
- `prompt_pack_versions`: `PRIMARY KEY(id)`, FK `pack_id` to
  `prompt_packs(pack_id) ON DELETE CASCADE`, `UNIQUE(pack_id, pack_version)`,
  `UNIQUE(id, pack_id)`, `CHECK(origin_kind IN ('bundled', 'user'))`,
  `CHECK(lifecycle_status IN ('draft', 'active', 'archived'))`, and at most one
  active version per pack.
- `prompt_pack_stage_templates`: `PRIMARY KEY(id)`, FK `pack_version_id` to
  `prompt_pack_versions(id) ON DELETE CASCADE`,
  `UNIQUE(pack_version_id, stage_name, provider_family)`, index
  `(pack_version_id, stage_order)`.
- `prompt_pack_schema_assets`: `PRIMARY KEY(id)`, FK `pack_version_id` to
  `prompt_pack_versions(id) ON DELETE CASCADE`,
  `UNIQUE(pack_version_id, schema_id)`, `UNIQUE(pack_version_id,
  relative_path)`.
- `prompt_pack_runs`: `PRIMARY KEY(id)`, nullable FK `project_id` to
  `projects(id) ON DELETE CASCADE`, FK `pack_id` to `prompt_packs(pack_id)`,
  composite FK `(pack_version_id, pack_id)` to
  `prompt_pack_versions(id, pack_id) ON DELETE RESTRICT`, status `CHECK`s,
  `result_status` nullable until a result exists, `UNIQUE(id, pack_id,
  pack_version_id, pack_version, schema_version)`, indexes `(project_id,
  created_at DESC)`, `(pack_id, created_at DESC)`, and `(run_status,
  created_at DESC)`.
- `prompt_pack_run_scopes`: `PRIMARY KEY(id)`, FK `run_id` to
  `prompt_pack_runs(id) ON DELETE CASCADE`, nullable FK `source_id` to
  `sources(id) ON DELETE SET NULL`, `UNIQUE(run_id, selected_order)`, index
  `(source_id)`, and `CHECK(scope_kind IN ('youtube_video',
  'youtube_playlist'))`.
- `prompt_pack_run_source_snapshots`: `PRIMARY KEY(id)`, FK `run_id` to
  `prompt_pack_runs(id) ON DELETE CASCADE`, nullable FK `live_source_id` to
  `sources(id) ON DELETE SET NULL`,
  `UNIQUE(run_id, source_ref_id)`, `UNIQUE(run_id, video_id)`, partial unique
  `(run_id, live_source_id)` when `live_source_id IS NOT NULL`, and
  `CHECK(source_type = 'youtube_video')`.
- `prompt_pack_run_source_origins`: `PRIMARY KEY(id)`, FK `run_id` to
  `prompt_pack_runs(id) ON DELETE CASCADE`, nullable FK `source_snapshot_id` to
  `prompt_pack_run_source_snapshots(id) ON DELETE CASCADE`, FK
  `origin_scope_id` to `prompt_pack_run_scopes(id) ON DELETE CASCADE`, nullable
  FKs `playlist_source_id` and `video_source_id` to `sources(id) ON DELETE SET
  NULL`, nullable FK `live_playlist_item_id` to `youtube_playlist_items(id) ON
  DELETE SET NULL`, `UNIQUE(run_id, origin_scope_id, video_id)`, indexes
  `(source_snapshot_id)` and `(run_id, inclusion_status)`, and `CHECK`s for
  `origin_kind IN ('explicit_video', 'playlist_item')` and `inclusion_status IN
  ('included', 'skipped', 'blocking_failure')`.
- `prompt_pack_run_material_snapshots`: `PRIMARY KEY(id)`, FK `run_id` to
  `prompt_pack_runs(id) ON DELETE CASCADE`, FK `source_snapshot_id` to
  `prompt_pack_run_source_snapshots(id) ON DELETE CASCADE`, nullable FK
  `live_item_id` to `items(id) ON DELETE SET NULL`, nullable FK
  `live_segment_id` to `youtube_transcript_segments(id) ON DELETE SET NULL`,
  `UNIQUE(run_id, ref)`, indexes `(source_snapshot_id, document_order)` and
  `(source_snapshot_id, timestamp_start_ms)`, and `CHECK(material_kind IN
  ('youtube_transcript_segment', 'youtube_description', 'youtube_comment'))`.
- `prompt_pack_stage_runs`: `PRIMARY KEY(id)`, FK `run_id` to
  `prompt_pack_runs(id) ON DELETE CASCADE`, nullable FK `source_snapshot_id` to
  `prompt_pack_run_source_snapshots(id) ON DELETE CASCADE`, stage/status
  `CHECK`s, `CHECK((stage_scope_kind = 'run' AND source_snapshot_id IS NULL)
  OR (stage_scope_kind = 'video' AND source_snapshot_id IS NOT NULL))`, partial
  unique `(run_id, stage_name)` for run-scoped stages, partial unique
  `(run_id, stage_name, source_snapshot_id)` for video-scoped stages, indexes
  `(run_id, stage_order)` and `(run_id, stage_status)`.
- `prompt_pack_stage_artifacts`: `PRIMARY KEY(id)`, FK `stage_run_id` to
  `prompt_pack_stage_runs(id) ON DELETE CASCADE`, `attempt_number`,
  `artifact_index`, `redaction_state`, artifact-kind `CHECK`, `CHECK` that at
  least one content column is present, `UNIQUE(stage_run_id, artifact_kind,
  attempt_number, artifact_index)`, index `(stage_run_id, created_at)`.
- `prompt_pack_results`: `PRIMARY KEY(id)`, FK `run_id` to
  `prompt_pack_runs(id) ON DELETE CASCADE`, `UNIQUE(run_id)` for MVP,
  `UNIQUE(run_id, result_id)`, `UNIQUE(id, run_id)`, composite FK `(run_id,
  pack_id, pack_version_id, pack_version, schema_version)` to
  `prompt_pack_runs(id, pack_id, pack_version_id, pack_version,
  schema_version) ON DELETE CASCADE`, result-status `CHECK`, and index
  `(pack_id, created_at DESC)`.
- Core projection tables: FK `result_row_id` to `prompt_pack_results(id) ON
  DELETE CASCADE`, denormalized `run_id`, `UNIQUE(result_row_id, <canonical
  object id>)`, relevant indexes for UI filters, and `raw_object_json_zstd`.
  This applies to source refs, claims, evidence, claim relations, unknowns,
  verification tasks, warnings, limitations, and audit refs. `quality_flags`
  have no canonical id, so use `flag_index` with
  `UNIQUE(result_row_id, flag_index)` plus indexes on `(flag, severity)`.
- `prompt_pack_result_ref_edges`: generic projection of array refs from
  canonical JSON and pack data. Store `from_object_kind`, `from_object_id`,
  `from_object_path`, `ref_kind`, `target_id`, and `ordinal`; use
  `UNIQUE(result_row_id, from_object_path, ref_kind, ordinal)` plus indexes
  `(result_row_id, ref_kind, target_id)` and `(result_row_id,
  from_object_kind, from_object_id)`.
- YouTube projection tables: FK `result_row_id` to `prompt_pack_results(id) ON
  DELETE CASCADE`, denormalized `run_id`, parent video object references, and
  `raw_object_json_zstd`. Use `UNIQUE(result_row_id, video_id)` for videos and
  `UNIQUE(result_row_id, video_id, <nested object id>)` for segments,
  key points, quotes, action items, and open questions. Store synthesis rows in
  `prompt_pack_youtube_synthesis_items` with `synthesis_item_kind` and
  `synthesis_item_id`, unique by `(result_row_id, synthesis_item_kind,
  synthesis_item_id)`.
- `prompt_pack_result_validation_findings`: `PRIMARY KEY(id)`, FK
  `result_row_id` to `prompt_pack_results(id) ON DELETE CASCADE`, nullable FK
  `stage_run_id` to `prompt_pack_stage_runs(id) ON DELETE SET NULL`, severity
  and layer `CHECK`s, indexes `(result_row_id, severity)`, `(rule_id)`, and
  `(stage_run_id)`.
- `prompt_pack_result_audit_events`: `PRIMARY KEY(id)`, FK `run_id` to
  `prompt_pack_runs(id) ON DELETE CASCADE`, nullable FKs `result_row_id` and
  `stage_run_id`, `audit_id`, event type fields, summary, object refs, payload,
  and indexes `(run_id, created_at)`, `(result_row_id, audit_id)`,
  `(stage_run_id)`.
- `prompt_pack_result_quarantine_artifacts`: `PRIMARY KEY(id)`, FK `run_id` to
  `prompt_pack_runs(id) ON DELETE CASCADE`, nullable FKs `result_row_id` and
  `stage_run_id`, `quarantine_id`, reason, object path/type, validation
  findings, invalid object payload, raw artifact payload, `redaction_state`,
  `content_hash`, `UNIQUE(result_row_id, quarantine_id)` when both are present,
  and indexes `(run_id, created_at)`, `(stage_run_id)`.

### `prompt_pack_runs`

The root row for a Prompt Pack run.

Important fields:

- `id`
- `project_id`
- `pack_id`
- `pack_version_id`
- `pack_version`
- `schema_version`
- `run_status`: `queued`, `running`, `complete`, `partial`, `failed`,
  `cancelled`, `interrupted`
- `result_status`: `complete`, `partial`, `error`
- `control_preset`
- `evidence_mode`
- `output_language`
- `include_comments`
- `selected_llm_profile_id`
- `selected_model_override`
- `resolved_provider`
- `resolved_model`
- `resolved_profile_snapshot_json_zstd`
- `preflight_json_zstd`
- `pack_snapshot_json_zstd`
- `progress_current`
- `progress_total`
- `error`
- `created_at`
- `started_at`
- `completed_at`

### `prompt_pack_run_scopes`

Records what the user selected before expansion.

Important fields:

- `id`
- `run_id`
- `scope_kind`: `youtube_video` or `youtube_playlist`
- `source_id`
- `source_subtype`
- `title_snapshot`
- `selected_order`
- `scope_metadata_json_zstd`

Playlist rows record the selected playlist source. Video rows record explicitly
selected video sources.

### `prompt_pack_run_source_snapshots`

Stores frozen canonical per-video source snapshots. Playlist is not a canonical
`source_ref`, and playlist membership/origin context is intentionally not stored
on this table. A video appears at most once per run even if it was selected
directly and also appears in a playlist, or appears in multiple selected
playlists.

Important fields:

- `id`
- `run_id`
- `source_ref_id`
- `live_source_id`
- `source_type`: `youtube_video`
- `video_id`
- `canonical_url`
- `internal_uri`
- `material_id`
- `snapshot_id`
- `title`
- `channel_title`
- `channel_handle`
- `channel_url`
- `duration_seconds`
- `language`
- `published_at`
- `accessed_at`
- `access_status`
- `scraped_at`
- `captions_available`
- `transcript_available`
- `is_live_recording`
- `view_count`
- `like_count`
- `comment_count`
- `comment_collection_status`
- `type_data_json_zstd`
- `raw_metadata_zstd`
- `content_hash`
- `created_at`

### `prompt_pack_run_source_origins`

Stores how each selected scope expanded into included, skipped, or blocking
video candidates. This separates selection/origin context from canonical video
snapshots.

Important fields:

- `id`
- `run_id`
- `origin_scope_id`
- `source_snapshot_id`
- `origin_kind`: `explicit_video` or `playlist_item`
- `inclusion_status`: `included`, `skipped`, or `blocking_failure`
- `skip_reason`
- `live_playlist_item_id`
- `playlist_source_id`
- `playlist_id`
- `playlist_title`
- `playlist_position`
- `video_source_id`
- `video_id`
- `title_snapshot`
- `availability_status_snapshot`
- `estimated_input_tokens`
- `estimated_input_chars`
- `budget_status`
- `origin_metadata_json_zstd`
- `created_at`

Rows with `inclusion_status = "included"` must point to a
`source_snapshot_id`. Skipped and blocking rows may have no source snapshot,
because the live video row can be unavailable, unlinked, missing transcript, or
over budget. For explicit video selections, invalid candidates are normally
`blocking_failure`; for playlist expansions, invalid entries are normally
`skipped` unless no videos remain includable.

### `prompt_pack_run_material_snapshots`

Stores frozen material units used by the run.

Material kinds:

- `youtube_transcript_segment`
- `youtube_description`
- `youtube_comment`

Important fields:

- `id`
- `run_id`
- `source_snapshot_id`
- `material_kind`
- `live_item_id`
- `live_segment_id`
- `ref`
- `external_id`
- `author`
- `published_at`
- `document_order`
- `timestamp_start_ms`
- `timestamp_end_ms`
- `content_zstd`
- `metadata_zstd`
- `content_hash`
- `created_at`

Transcript segments are the primary evidence material for MVP. Description is
included. Comments are included only when `include_comments = true`.

### `prompt_pack_stage_runs`

Stores each planned or executed stage.

MVP stage set:

- `source_ingestion`
- `youtube_summary/transcript_analysis`
- `youtube_summary/segment_extraction`
- `youtube_summary/key_point_extraction`
- `youtube_summary/quote_extraction`
- `youtube_summary/synthesis`
- `final_synthesis`
- `validation`

Important fields:

- `id`
- `run_id`
- `stage_name`
- `stage_order`
- `stage_status`: `pending`, `running`, `succeeded`, `failed`, `skipped`,
  `not_implemented`
- `stage_scope_kind`: `run` or `video`
- `source_snapshot_id`
- `resolved_provider`
- `resolved_model`
- `resolved_profile_snapshot_json_zstd`
- `started_at`
- `completed_at`
- `status_reason`
- `error`

For MVP, `youtube_summary/transcript_analysis` runs once per video.
`youtube_summary/segment_extraction`, `youtube_summary/key_point_extraction`,
and `youtube_summary/quote_extraction` are recorded as `skipped` with
`status_reason = "combined_into_transcript_analysis_mvp"` because the first real
LLM stage returns those objects together. `youtube_summary/synthesis` is
recorded for multi-video runs as `not_implemented` or `skipped`.

### Combined MVP Stage I/O

The combined MVP stage still uses the generic stage I/O rules from
`docs/prompt-packs/stage_io_contracts.md`.

Stage input envelope:

- `stage_io_version = "1.0"`
- `schema_version = "1.0"`
- `stage = "youtube_summary/transcript_analysis"`
- `pack_id = "youtube_summary"`
- `pack_version`
- `run_id`
- `source_ref_id`
- `allowed_source_ref_ids`
- `allowed_material_refs`
- `transcript_segment_registry`
- `description_material_ref`
- `comment_material_refs`
- `control_preset`
- `evidence_mode`
- `output_language`

Closed-world rules:

- LLM output may reference only `source_ref_id` values from
  `allowed_source_ref_ids`.
- LLM output may reference only material `ref` values from
  `allowed_material_refs`.
- The LLM must not assign canonical `claim_id`, `evidence_id`, `source_ref_id`,
  or final pack object ids.
- The validator rejects any candidate object that references a material outside
  the stage input registry.

Expected parsed output shape:

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "youtube_summary/transcript_analysis",
  "video_candidate": {
    "summary_text": "string",
    "segment_candidates": [],
    "key_point_candidates": [],
    "quote_candidates": [],
    "action_item_candidates": [],
    "open_question_candidates": []
  },
  "claim_candidates": [],
  "evidence_fragment_candidates": [],
  "warning_candidates": []
}
```

Mapping to canonical result:

- `evidence_fragment_candidates` become top-level `evidence[]` only after the
  backend assigns `claim_id` and `evidence_id`.
- `claim_candidates` become top-level `claims[]`; backend assigns `claim_id`
  and rebuilds `claim.source_refs`.
- `segment_candidates`, `key_point_candidates`, `quote_candidates`,
  `action_item_candidates`, and `open_question_candidates` become
  pack-specific objects inside one `Video`.
- Backend assigns final nested ids (`segment_id`, `key_point_id`, `quote_id`,
  `action_item_id`, `open_question_id`) and derives traversal refs from the
  accepted canonical claims/evidence.
- The skipped `segment_extraction`, `key_point_extraction`, and
  `quote_extraction` stage rows point to the successful combined stage through
  `status_reason` and audit events; they do not have separate LLM artifacts.

### `prompt_pack_stage_artifacts`

Stores operational stage payloads.

Important fields:

- `id`
- `stage_run_id`
- `artifact_kind`: `prompt_input`, `raw_output`, `parsed_output`,
  `repair_input`, `error`, `metrics`
- `attempt_number`
- `artifact_index`
- `redaction_state`
- `content_json_zstd`
- `content_text_zstd`
- `content_hash`
- `token_input_count`
- `token_output_count`
- `estimated_cost`
- `created_at`

Raw provider output and parsed output are retained for debugging and audit.

### `prompt_pack_results`

Stores the immutable canonical result.

Important fields:

- `id`
- `run_id`
- `result_id`
- `schema_version`
- `pack_id`
- `pack_version_id`
- `pack_version`
- `result_status`
- `canonical_json_zstd`
- `canonical_hash`
- `created_at`
- `projection_updated_at`

The canonical JSON is the contract source of truth. Projection tables are
derived from it and may be rebuilt. Before insertion, the backend validates that
the canonical JSON `run_id`, `pack_id`, `pack_version`, and `schema_version`
match the owning `prompt_pack_runs` row and `pack_snapshot_json_zstd`; DB
constraints then prevent row-level pack identity drift.

### Projection Tables

Projection tables store queryable slices of canonical JSON:

- `prompt_pack_result_source_refs`
- `prompt_pack_result_claims`
- `prompt_pack_result_evidence`
- `prompt_pack_result_claim_relations`
- `prompt_pack_result_unknowns`
- `prompt_pack_result_verification_tasks`
- `prompt_pack_result_warnings`
- `prompt_pack_result_limitations`
- `prompt_pack_result_quality_flags`
- `prompt_pack_result_audit_refs`
- `prompt_pack_result_ref_edges`
- `prompt_pack_youtube_videos`
- `prompt_pack_youtube_segments`
- `prompt_pack_youtube_key_points`
- `prompt_pack_youtube_quotes`
- `prompt_pack_youtube_action_items`
- `prompt_pack_youtube_open_questions`
- `prompt_pack_youtube_synthesis_items`

Every object projection row should store:

- `run_id`
- `result_row_id`
- the object id from canonical JSON;
- denormalized display/search fields;
- relevant refs;
- `raw_object_json_zstd` for lossless projection rebuild and debugging.

`prompt_pack_result_ref_edges` stores normalized reference edges instead of
canonical objects. It exists so UI and validators can query "what points to this
claim/evidence/source/relation" without decompressing canonical JSON.

Projection tables do not replace canonical JSON.

### Validation, Audit, and Quarantine

`prompt_pack_result_validation_findings` stores schema, reference, pipeline, and
QA findings.

Important fields:

- `id`
- `run_id`
- `result_row_id`
- `stage_run_id`
- `rule_id`
- `severity`
- `layer`
- `object_path`
- `message`
- `object_refs_json_zstd`
- `created_at`

`prompt_pack_result_audit_events` stores compact audit events. Large payloads do
not live inline in canonical JSON.

Important fields:

- `id`
- `run_id`
- `result_row_id`
- `stage_run_id`
- `audit_id`
- `event_type`
- `custom_event_type`
- `summary`
- `object_refs_json_zstd`
- `payload_json_zstd`
- `created_at`

`prompt_pack_result_quarantine_artifacts` stores invalid stage or graph objects
that cannot be safely included in canonical JSON.

Important fields:

- `id`
- `run_id`
- `result_row_id`
- `stage_run_id`
- `quarantine_id`
- `reason`
- `object_path`
- `object_type`
- `validation_findings_json_zstd`
- `invalid_object_json_zstd`
- `raw_artifact_json_zstd`
- `redaction_state`
- `content_hash`
- `created_at`

## Runtime Flow

```text
User selects synced video or playlist
  -> preflight
  -> create prompt_pack_run
  -> freeze source and material snapshots
  -> create stage skeleton
  -> run per-video LLM stage
  -> validate stage output
  -> assemble canonical result JSON
  -> write projection tables
  -> mark complete, partial, or failed
```

### Preflight

Preflight validates and estimates the run before it starts.

Rules:

- source must already be synced in Library;
- selected source must be YouTube video or playlist;
- playlist expands through linked, non-removed `youtube_playlist_items`;
- preflight returns three explicit partitions:
  - `included_videos`: linked, available videos with usable transcript and input
    size within the selected model budget;
  - `skipped_videos`: playlist entries excluded before execution, with
    structured reasons such as `unlinked_playlist_item`, `unavailable_video`,
    `no_usable_transcript`, or `input_budget_exceeded`;
  - `blocking_failures`: selected explicit videos that cannot be analyzed,
    invalid source selections, or playlist selections with zero includable
    videos;
- every included video needs a usable transcript;
- preflight never falls back from transcript to description or comments;
- comments are excluded unless explicitly enabled;
- estimated token/cost/chunk info is shown to the user;
- each video's estimated stage input must fit the selected model budget for the
  MVP single-request per-video stage;
- start is allowed when `blocking_failures` is empty and `included_videos` is
  non-empty;
- any `skipped_videos` entry is copied into `prompt_pack_run_source_origins`,
  surfaced in the UI, and later represented as corpus coverage limitation or
  partial result metadata.

### Snapshot

The backend copies source and material data into run-local tables before LLM
execution.

Snapshots include:

- video metadata;
- origin rows for explicit selections, playlist entries, skipped entries, and
  blocking preflight candidates;
- transcript segments with timestamps;
- description;
- optional comments;
- content hashes and metadata needed for traceability.

### LLM Stage

The real MVP LLM stage is `youtube_summary/transcript_analysis`.

It runs once per video. The stage prompt asks for a structured JSON result with:

- readable summary;
- segments;
- key points;
- notable quotes;
- enough local references for backend assembly.

The backend owns canonical IDs and canonical evidence/source refs. LLM output is
treated as a candidate payload. The LLM must use only run-local material refs
provided in the prompt; it must not assign canonical `claim_id`, `evidence_id`,
or `source_ref_id` values.

### Assembly

The backend assembles a minimal canonical Prompt Pack result:

- `source_refs`
- `claims`
- `evidence`
- `outputs.summary`
- `outputs.sections`
- `outputs.pack_data.youtube_summary.videos`
- `outputs.pack_data.youtube_summary.synthesis = null`
- `warnings`, `limitations`, and `quality_flags` as needed
- `audit_refs`

For multi-video runs, `synthesis = null` is allowed in MVP only with an explicit
limitation or quality flag explaining that cross-video synthesis was not
executed.

## Validation

MVP uses a spec-aware validator.

Checks:

- JSON parses;
- required fields are present;
- object IDs are unique inside their scope;
- refs do not dangle;
- evidence timestamps are inside source transcript bounds;
- `video.source_ref_id` points to a `youtube_video` source ref;
- every `Video.source_refs` includes its `source_ref_id`;
- `Video.claim_refs`, `Video.evidence_refs`, and `Video.source_refs` match the
  derived traversal unions for retained nested objects;
- standard key points have non-empty `claim_refs`;
- action items, when present, have non-empty `claim_refs`;
- claims and evidence obey one-claim-per-direct-evidence ownership;
- notable quotes have non-empty `evidence_refs`;
- quote evidence points to top-level evidence with
  `fragment_type = "video_timestamp_range"` and `text_mode = "verbatim"`;
- quote text is at most 50 words;
- quote `word_count`, when present, matches the same word-counting convention;
- segment evidence refs point to evidence whose locator overlaps the segment
  timestamp range;
- multi-video `synthesis = null` produces a warning or limitation.

MVP does not implement the full reference validator, graph healing, retry
repair, or embedding-based fragment deduplication.

## Status Semantics

Single video:

- success -> run `complete`;
- validation failure or LLM failure -> run `failed`;
- cancellation -> run `cancelled`.

Multi-video or playlist:

- all included videos succeed and preflight skipped no selected playlist entries
  -> run `complete`, with synthesis limitation if applicable;
- all included videos succeed but preflight skipped one or more selected
  playlist entries -> run `partial` with `corpus_coverage_limited` or
  `partial_result` quality flag;
- some included videos succeed and some fail during execution -> run `partial`;
- no videos succeed -> run `failed`.

Preflight `skipped_videos`, runtime per-video failures, and quarantined invalid
objects are retained separately. Skipped videos live in source origin rows and
result limitations. Runtime failures live in stage rows, validation findings,
audit, and result quality flags where a partial result exists.

## UI Scope

MVP UI has two entry points:

1. A new YouTube Summary or Prompt Pack runs screen.
2. A `Summarize` action from Library detail for synced YouTube video and
   playlist sources.

### Runs Screen

Shows:

- run list;
- selected scope;
- pack/version;
- status and progress;
- preset/evidence mode;
- provider/model;
- created and completed timestamps;
- error or limitation indicators.

### Result View

Shows per video:

- title, channel, duration;
- summary;
- segments timeline;
- key points;
- notable quotes;
- evidence links with timestamps;
- validation warnings.

For multi-video runs, UI explicitly states that cross-video synthesis is not
executed in MVP.

### Stage Inspector

Shows:

- stage list and status;
- prompt input;
- raw output;
- parsed output;
- validation findings;
- audit events;
- skipped and not-implemented stages;
- token and cost metadata when available.

Pack library UI is read-only or absent in MVP. Full prompt-pack editing is a
future slice.

### Runtime/UI Lifecycle

Prompt Pack runs should have their own runtime state and event stream, separate
from legacy analysis runtime.

Backend state:

- `PromptPackRunState` tracks active run ids and cancel-requested run ids in
  memory.
- DB `run_status` remains authoritative for persisted history.
- On app startup or runtime cleanup, stale `queued` or `running` rows without an
  active task are marked `interrupted`.
- The LLM scheduler may still provide queue position, but Prompt Pack runtime
  owns run status transitions and emitted events.

Event channel:

```text
prompt-pack-run-event
```

`PromptPackRunEvent` fields:

- `run_id`
- `request_id`
- `kind`: `queued`, `started`, `progress`, `stage_started`,
  `stage_completed`, `stage_failed`, `completed`, `partial`, `failed`,
  `cancelled`, `interrupted`
- `run_status`: `queued`, `running`, `complete`, `partial`, `failed`,
  `cancelled`, `interrupted`
- `phase`: `preflight`, `snapshot`, `stage`, `validation`, `projection`,
  `persist`, or `terminal`
- `stage_run_id`
- `stage_name`
- `source_snapshot_id`
- `queue_position`
- `progress_current`
- `progress_total`
- `message`
- `error`

Event rules:

- `start_youtube_summary_run` creates the run, writes preflight partitions,
  creates included source/material snapshots and stage skeleton, registers the
  run as active, and emits `queued` or `started`.
- Every stage status change emits an event with `stage_run_id` and `stage_name`.
- Per-video progress advances over included videos, not skipped entries.
- Terminal events are `completed`, `partial`, `failed`, `cancelled`, or
  `interrupted`; after a terminal event the backend removes the run from active
  state.
- Cancellation is cooperative. `cancel_prompt_pack_run` marks cancel requested,
  cancels queued scheduler work where possible, and stages check cancellation
  between videos and before/after provider calls. If a provider call cannot be
  interrupted, the run becomes `cancelled` after the current call returns.

UI sync rules:

- On mount, UI calls `list_prompt_pack_runs` and
  `list_active_prompt_pack_runs`, then subscribes to `prompt-pack-run-event`.
- Events update in-memory active rows immediately.
- Terminal events trigger a debounced refresh of the run list, active list, run
  detail, stage list, result, validation findings, and audit events for the
  affected run.
- UI keeps a polling fallback while any active run exists, so missed Tauri
  events do not leave the run list stale.
- The bottom queue/runs surface shows queued/running Prompt Pack runs using
  `queue_position`, `progress_current`, and `progress_total`.

## Backend Commands

Initial command/API surface:

```text
list_prompt_pack_runs
list_active_prompt_pack_runs
get_prompt_pack_run
preflight_youtube_summary_run
start_youtube_summary_run
cancel_prompt_pack_run
list_prompt_pack_run_stages
get_prompt_pack_stage_artifact
get_prompt_pack_result
get_prompt_pack_validation_findings
list_prompt_pack_audit_events
```

The implementation can add smaller internal commands or DTOs as needed, but the
UI should not depend on legacy `analysis_runs` commands.

## LLM Routing

MVP uses existing LLM profiles.

The user selects a profile and optional model override at run start. Every stage
stores a resolved provider/model/profile snapshot. This keeps MVP simple while
preserving the future path to per-stage routing.

No automatic provider fallback is allowed. Provider errors produce stage failure,
video failure, partial run, or failed run according to status semantics.

## Testing Strategy

Backend tests should cover:

- prompt-pack library seed idempotency;
- prompt-pack library seed conflict when the same `(pack_id, pack_version)` has
  a different `content_hash`;
- bundled seed not overwriting user drafts;
- downgrade-safe seed behavior preserving unknown newer versions;
- video preflight success;
- playlist expansion into included, skipped, and blocking partitions;
- explicit video transcript-missing preflight blocking failure;
- playlist video transcript-missing preflight skipped entry;
- explicit video oversized transcript preflight blocking failure;
- playlist video oversized transcript preflight skipped entry;
- run snapshot independence from live Library mutations;
- source origin rows preserving playlist membership without duplicating canonical
  video snapshots;
- stage skeleton creation;
- per-video LLM success path using a fake provider;
- per-video LLM failure producing partial multi-video run;
- canonical result storage and projection rebuild;
- canonical result insert rejected when row pack identity and canonical JSON or
  owning run identity disagree;
- spec-aware validator success and failure cases;
- combined `youtube_summary/transcript_analysis` stage output validation against
  `stage_io_version`, allowed source ids, and allowed material refs;
- validator coverage for traversal unions, quote word counts, segment evidence
  ranges, and one-claim-per-evidence ownership;
- stage artifact storage with sanitized provider errors.

Schema tests should verify:

- foreign keys and indexes exist;
- `prompt_pack_runs.pack_id` cannot disagree with `pack_version_id`;
- `prompt_pack_results` cannot disagree with owning run pack identity;
- active pack-version uniqueness is enforced per pack;
- `prompt_pack_run_source_snapshots` is unique by canonical video while
  `prompt_pack_run_source_origins` can store multiple selection origins;
- stage skeleton uniqueness handles run-scoped and video-scoped stages;
- cascade behavior for run-owned data;
- deleting live Library rows nulls live links but keeps run snapshots readable;
- canonical JSON survives compression/decompression;
- projection tables and `prompt_pack_result_ref_edges` can be rebuilt from
  canonical JSON.

UI tests should cover:

- preflight failure display;
- preflight skipped-video display for playlist runs;
- run progress for multi-video;
- partial run display;
- active run event updates, terminal refresh, and polling fallback;
- cancel request behavior for queued and running runs;
- result view timestamp links;
- stage inspector artifact display.

## Future Slices

- Execute `youtube_summary/synthesis` for multi-video runs.
- Split per-video analysis into multiple real stages:
  `fragment_candidate_mining`, `claim_extraction`,
  `canonical_evidence_generation`, `pack_data_generation`.
- Add retry/repair and quarantine workflows.
- Add full reference validator.
- Add prompt-pack editor and version comparison UI.
- Add hybrid search over canonical claims/evidence.
- Add chunked per-video transcript analysis for videos that exceed one model
  request.
- Add URL-to-sync-to-summary workflow after ingest boundaries are settled.
