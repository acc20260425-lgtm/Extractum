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
- `lifecycle_status` such as `active`, `draft`, `archived`
- `definition_json_zstd`
- `bundled_source_path`
- `content_hash`
- `created_at`
- `activated_at`

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
prompt_pack_run_material_snapshots
prompt_pack_stage_runs
prompt_pack_stage_artifacts
prompt_pack_results
prompt_pack_result_source_refs
prompt_pack_result_claims
prompt_pack_result_evidence
prompt_pack_result_claim_relations
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

### `prompt_pack_runs`

The root row for a Prompt Pack run.

Important fields:

- `id`
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

Stores frozen per-video source snapshots. Playlist is not a canonical
`source_ref`, but playlist context is copied into each video source snapshot
when applicable.

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
- `playlist_source_id`
- `playlist_id`
- `playlist_title`
- `playlist_position`
- `type_data_json_zstd`
- `raw_metadata_zstd`
- `content_hash`
- `created_at`

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

### `prompt_pack_stage_artifacts`

Stores operational stage payloads.

Important fields:

- `id`
- `stage_run_id`
- `artifact_kind`: `prompt_input`, `raw_output`, `parsed_output`,
  `repair_input`, `error`, `metrics`
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
- `pack_version`
- `result_status`
- `canonical_json_zstd`
- `canonical_hash`
- `created_at`
- `projection_updated_at`

The canonical JSON is the contract source of truth. Projection tables are
derived from it and may be rebuilt.

### Projection Tables

Projection tables store queryable slices of canonical JSON:

- `prompt_pack_result_source_refs`
- `prompt_pack_result_claims`
- `prompt_pack_result_evidence`
- `prompt_pack_result_claim_relations`
- `prompt_pack_youtube_videos`
- `prompt_pack_youtube_segments`
- `prompt_pack_youtube_key_points`
- `prompt_pack_youtube_quotes`
- `prompt_pack_youtube_action_items`
- `prompt_pack_youtube_open_questions`
- `prompt_pack_youtube_synthesis_items`

Every projection row should store:

- `run_id`
- `result_id`
- the object id from canonical JSON;
- denormalized display/search fields;
- relevant refs;
- `raw_object_json_zstd` for lossless projection rebuild and debugging.

Projection tables do not replace canonical JSON.

### Validation, Audit, and Quarantine

`prompt_pack_result_validation_findings` stores schema, reference, pipeline, and
QA findings.

Important fields:

- `run_id`
- `result_id`
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

`prompt_pack_result_quarantine_artifacts` stores invalid stage or graph objects
that cannot be safely included in canonical JSON.

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
- unavailable or unlinked playlist entries are skipped and reported;
- every selected explicit video and every linked playlist video selected for
  analysis needs usable transcript;
- a linked playlist video without usable transcript fails preflight rather than
  falling back to description or comments;
- comments are excluded unless explicitly enabled;
- estimated token/cost/chunk info is shown to the user;
- each video's estimated stage input must fit the selected model budget for the
  MVP single-request per-video stage;
- if any selected analyzable video lacks transcript or exceeds the MVP input
  budget, preflight fails.

### Snapshot

The backend copies source and material data into run-local tables before LLM
execution.

Snapshots include:

- video metadata;
- playlist context for playlist-origin videos;
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

- all videos succeed -> run `complete`, with synthesis limitation if applicable;
- some videos succeed and some fail -> run `partial`;
- no videos succeed -> run `failed`.

Per-video failures are retained in stage rows, validation findings, audit, and
result quality flags where a partial result exists.

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

## Backend Commands

Initial command/API surface:

```text
list_prompt_pack_runs
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
- video preflight success;
- playlist expansion and skipped unlinked items;
- transcript-missing preflight failure;
- oversized per-video transcript preflight failure;
- run snapshot independence from live Library mutations;
- stage skeleton creation;
- per-video LLM success path using a fake provider;
- per-video LLM failure producing partial multi-video run;
- canonical result storage and projection rebuild;
- spec-aware validator success and failure cases;
- validator coverage for traversal unions, quote word counts, segment evidence
  ranges, and one-claim-per-evidence ownership;
- stage artifact storage with sanitized provider errors.

Schema tests should verify:

- foreign keys and indexes exist;
- cascade behavior for run-owned data;
- canonical JSON survives compression/decompression;
- projection tables can be rebuilt from canonical JSON.

UI tests should cover:

- preflight failure display;
- run progress for multi-video;
- partial run display;
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
