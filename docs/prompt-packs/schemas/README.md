# Prompt Pack JSON Schemas

Status: v1 semantic local-shape schema bundle.

This directory is the canonical location for machine-readable JSON Schema files
used by the prompt-pack reference validator.

The checked-in `schemas/v1/` files all have reviewed semantic local-shape
coverage. They fix stable paths, `$id` values, draft version, grouping, and
manifest loading conventions. They do not replace the prose specs or
`validation_rules.md` for cross-object graph, pipeline, and QA semantics.

---

## 1. Directory Layout

Machine-readable schemas for Prompt Pack JSON Contract v1 live under:

```text
docs/prompt-packs/schemas/v1/
```

Current layout:

```text
schemas/
  README.md
  v1/
    schema_manifest.json
    core/
      result.schema.json
      validation_finding.schema.json
      audit_ref.schema.json
      confidence.schema.json
    source-types/
      youtube_video.schema.json
      web_page.schema.json
      rss_entry.schema.json
      telegram_post.schema.json
      telegram_channel_snapshot.schema.json
      telegram_chat_snapshot.schema.json
      forum_thread.schema.json
    fragment-locators/
      video_timestamp_range.schema.json
      audio_timestamp_range.schema.json
      text_range.schema.json
      image_region.schema.json
      document_section.schema.json
      post.schema.json
      comment.schema.json
      thread_reply.schema.json
      aggregate.schema.json
    stage-io/
      source_ingestion.schema.json
      fragment_candidate_mining.schema.json
      claim_extraction.schema.json
      canonical_evidence_generation.schema.json
      claim_linking.schema.json
      pack_data_generation.schema.json
      final_synthesis.schema.json
      retry_repair_payload.schema.json
    runtime/
      runtime_configuration.schema.json
    packs/
      technology_watch/
        pack_data.schema.json
      youtube_summary/
        pack_data.schema.json
```

Rules:

- Schema files use the suffix `.schema.json`.
- Schema files use JSON Schema draft 2020-12 unless a later manifest says
  otherwise.
- `schemas/v1/schema_manifest.json` is the machine-readable index for dynamic
  loading.
- The prose specs and `validation_rules.md` remain authoritative for
  cross-object graph checks, pipeline ownership rules, and QA semantics.

Current semantic coverage:

- `core/result.schema.json`;
- `core/audit_ref.schema.json`;
- `core/confidence.schema.json`;
- `core/validation_finding.schema.json`;
- `source-types/youtube_video.schema.json`;
- `source-types/web_page.schema.json`;
- `source-types/rss_entry.schema.json`;
- `source-types/telegram_post.schema.json`;
- `source-types/telegram_channel_snapshot.schema.json`;
- `source-types/telegram_chat_snapshot.schema.json`;
- `source-types/forum_thread.schema.json`;
- `packs/technology_watch/pack_data.schema.json`;
- `packs/youtube_summary/pack_data.schema.json`;
- `fragment-locators/video_timestamp_range.schema.json`;
- `fragment-locators/audio_timestamp_range.schema.json`;
- `fragment-locators/text_range.schema.json`;
- `fragment-locators/image_region.schema.json`;
- `fragment-locators/document_section.schema.json`;
- `fragment-locators/post.schema.json`;
- `fragment-locators/comment.schema.json`;
- `fragment-locators/thread_reply.schema.json`;
- `fragment-locators/aggregate.schema.json`;
- `stage-io/source_ingestion.schema.json`;
- `stage-io/fragment_candidate_mining.schema.json`;
- `stage-io/claim_extraction.schema.json`;
- `stage-io/canonical_evidence_generation.schema.json`;
- `stage-io/claim_linking.schema.json`;
- `stage-io/pack_data_generation.schema.json`;
- `stage-io/final_synthesis.schema.json`;
- `stage-io/retry_repair_payload.schema.json`.
- `runtime/runtime_configuration.schema.json`.

Some accepted locator rules remain pipeline/code checks because standard JSON
Schema draft 2020-12 does not support `$data`-style comparisons. Examples:
`timestamp_end >= timestamp_start`, `char_end > char_start`,
`x + width <= 1.0`, `y + height <= 1.0`, and aggregate consistency with
`evidence.contributing_evidence_refs`. The `document_section` cross-field rule
`page_number <= page_count` is also a pipeline/code check.

The `core/result` schema covers the reviewed local shape of the canonical
result envelope and core graph object shells: metadata, run context, readable
outputs, material-level source refs, claims, evidence, claim relations,
unknowns, verification tasks, warnings, limitations, quality flags, and audit
refs. Cross-object referential integrity, traversal unions, metadata/flag
consistency, pack-specific `pack_data`, source `type_data`, and fragment
`locator_data` semantics remain validator/pipeline or companion-schema checks.

The `youtube_video` source type schema covers the reviewed local `type_data`
shape, including the common wrapper, playlist field dependencies, and the
free-string `comment_collection_status` convention. Graph-level source
reference consistency remains a pipeline/code check.

The `web_page` source type schema covers the reviewed local `type_data` shape,
including the common wrapper, the no-duplicated-URL convention, free-string
page/extraction/comment status fields, and nullable root `parent_context`.

The `rss_entry` source type schema covers the reviewed local `type_data` shape,
including the feed-declared `entry_url`, `rss_feed` parent context,
string-category arrays, and the free-string `content_mode` convention. It
intentionally has no `collection_status` field in v1.

The `telegram_post` source type schema covers the reviewed local `type_data`
shape, including platform counters, discussion-layer collection fields,
forwarded-message metadata, `telegram_channel` parent context, and the
free-string `post_type` / `discussion_collection_status` conventions.

The `telegram_channel_snapshot` source type schema covers the reviewed local
`type_data` shape for aggregate channel snapshots, including channel activity
metrics, paired snapshot period fields, root `parent_context = null`, and the
intentional absence of `avg_reactions_per_post` in v1.

The `telegram_chat_snapshot` source type schema covers the reviewed local
`type_data` shape for aggregate chat snapshots, including member/message/author
counts, paired snapshot period fields, root `parent_context = null`, and
`creator_type = "unknown"` because a chat has no single author.

The `forum_thread` source type schema covers the reviewed local `type_data`
shape for forum and discussion-platform threads, including free-string
`platform`, cross-platform `vote_score`, participant/reply counters,
`forum | forum_category` parent context, and the intentional absence of
`reply_collection_status`, `upvote_count`, and `downvote_count` in v1.

The `technology_watch` pack data schema covers the reviewed local shape of
`pack_data.technology_watch`, including `technologies[]`, the `Technology`
object, maturity levels, signals, tools, adoption barriers, risks,
recommendations, and enum/custom-field conventions. Cross-object traversal,
strict-mode coverage, and pack obligation rules remain validator/pipeline
checks.

The `youtube_summary` pack data schema covers the reviewed local shape of
`pack_data.youtube_summary`, including `videos[]`, per-video segments, key
points, notable quotes, action items, open questions, and nullable/object
cross-video synthesis. Source anchor validation, traversal unions, quote
evidence authority, word-count equality, segment timestamp membership, and
multi-video synthesis obligations remain validator/pipeline checks.

The `source_ingestion` stage I/O schema covers the reviewed local shape of the
pipeline-owned ingestion boundary: input payloads with the common stage
envelope and `raw_material_refs`, and output payloads with a `source_registry`.
Full standard `type_data` validation is delegated to the source-type schemas,
and canonical source graph consistency remains a validator/pipeline check.

The `fragment_candidate_mining` stage I/O schema covers the reviewed local
shape of the pre-contract fragment mining boundary: input payloads with
`source_registry` and `material_windows`, LLM output payloads with
`fragment_candidates`, and pipeline output payloads with `fragment_registry`.
Allowed-ID checks, full `locator_data` validation, candidate deduplication, and
registry normalization remain validator/pipeline checks.

The `claim_extraction` stage I/O schema covers the reviewed local shape of the
closed-world claim candidate extraction boundary: input payloads with
`allowed_fragment_candidate_ids` and an immutable `fragment_registry`, and LLM
output payloads with `claim_candidates`, `unknown_candidates`,
`verification_task_candidates`, and optional `warnings`. Canonical IDs,
allowed-ID enforcement, unknown/task promotion, and final claim/evidence
assembly remain validator/pipeline checks.

The `canonical_evidence_generation` stage I/O schema covers the reviewed local
shape of the pipeline-owned canonical assembly boundary: input payloads with
`claim_candidates` and `fragment_registry`, and output payloads with canonical
`claims` and `evidence`. Evidence ownership, traversal rebuilding,
source-ref superset rules, and fragment/inference consistency remain
validator/pipeline checks.

The `claim_linking` stage I/O schema covers the reviewed local shape of the
closed-world relation candidate boundary: input payloads with
`allowed_claim_ids`, `allowed_evidence_ids`, and immutable claim/evidence
registries, and LLM output payloads with `relation_candidates`. Allowed-ID
enforcement, `contradicts` natural-sort normalization, relation ID assignment,
and relation evidence ownership remain validator/pipeline checks.

The `pack_data_generation` stage I/O schema covers the reviewed local shape of
the pack-specific projection boundary: input payloads with allowed
claim/evidence/source IDs and immutable claim/evidence/source registries, and
LLM output payloads with a single-namespace `pack_data_candidate`,
`unknown_candidates`, and `warning_candidates`. Pack-specific object IDs,
derived traversal fields, allowed-ID enforcement, and full pack-specific schema
validation remain validator/pipeline checks.

The `final_synthesis` stage I/O schema covers the reviewed local shape of the
readable-output synthesis boundary: input payloads with allowed
claim/evidence/source IDs, canonical claims, pack data, and optional graph
registries, and LLM output payloads with `outputs_candidate.summary` and
`outputs_candidate.sections`. Section/item ID assignment, summary claim
coverage, metadata assembly, quality flags, warnings, limitations, and audit
refs remain validator/pipeline checks.

The `retry_repair_payload` stage I/O schema covers the reviewed local shape of
compact repair prompts for retryable LLM stages. It enforces retry counters,
retryable stage names, validation findings, failed object paths, and optional
allowed-ID context arrays. The repaired response itself keeps using the
original stage output schema, or an implementation-specific object-isolated
replacement wrapper when the runner can safely isolate failed objects.

All eight v1 stage I/O schemas now have semantic local-shape coverage.

The `runtime_configuration` schema covers the reviewed local shape of
implementation-owned prompt-pack runtime configuration: model routing, feature
flags, budget limits, retry policy, quarantine policy, and telemetry settings.
This schema validates orchestration configuration artifacts; it does not make
runtime configuration part of canonical result JSON.

---

## 2. Schema IDs

Schema `$id` values should use a stable internal URI convention:

```text
extractum://prompt-packs/schemas/v1/<group>/<name>.schema.json
```

Examples:

```text
extractum://prompt-packs/schemas/v1/core/result.schema.json
extractum://prompt-packs/schemas/v1/source-types/youtube_video.schema.json
extractum://prompt-packs/schemas/v1/packs/technology_watch/pack_data.schema.json
```

Rules:

- `$id` is stable inside a schema version.
- Moving a file without changing `$id` is allowed only if
  `schema_manifest.json` maps the old logical ID to the new path.
- Changing required fields or enum meaning requires a schema version bump.

---

## 3. Dynamic Loading

The reference validator loads schemas in this order:

1. `schemas/v1/schema_manifest.json`;
2. core schemas;
3. companion schemas required by actual `source_type` and `fragment_type`
   values;
4. stage I/O schemas when validating `stage_payload` inputs and normalized
   `stage_output` artifacts;
5. runtime schemas when validating implementation configuration artifacts;
6. pack-specific schemas selected by `pack_id`.

Current reference-validator enforcement emits `SCHEMA-VALIDATION-001` findings
for canonical results, selected companion schemas, selected pack-specific
schemas, `stage_payload` input schemas, and normalized `stage_output` artifact
schemas. Raw provider responses remain covered by parser fixtures before they
are promoted into normalized stage outputs.

Rules:

- Missing required schema files are validator setup errors, not artifact
  validation errors.
- Unknown `source_type`, `fragment_type`, or `pack_id` falls back to the
  corresponding `custom` rules in the prose contract when available.
- Pipeline-level graph checks remain code rules even when local JSON Schema
  validation passes.

---

## 4. Promotion Policy

Future skeleton schemas are stable loader targets. They should be expanded into
semantic schemas incrementally and reviewed before enforcement. The v1 baseline
bundle currently has semantic local-shape coverage for every checked-in schema.

Recommended flow:

1. Select one schema group or artifact type.
2. Expand the skeleton from prose specs and `validator_manifest.md`.
3. Review the expanded schema against examples and validation fixtures.
4. Update `schemas/v1/schema_manifest.json` only if path, `$id`, status, or
   grouping changes.
5. Run validator fixtures in CI.

Rules:

- Expanded schemas must not silently add required fields absent from the prose
  spec.
- Expanded schemas must preserve nullable-vs-present conventions.
- Expanded schemas should include `$comment` fields referencing source
  documents and rule IDs where useful.
