# Stage Output Normalization

Status: runtime note.

Compatibility:

- Prompt Pack JSON Contract `schema_version: "1.0"`;
- Stage I/O Contracts `stage_io_version: "1.0"`.

This document records the boundary between provider-authored LLM output and the
internal stage/runtime contracts enforced by JSON Schema and Rust validators.

---

## 1. Purpose

Stage output schemas are executable contracts. They describe the shape the
pipeline validates, persists, and uses for canonical result assembly.

Provider output is less stable. Even with strict prompts, LLMs may return a
semantically correct value in a nearby shape, for example:

- camelCase envelope keys copied from older prompt inputs;
- omitted empty arrays;
- readable text arrays where the runtime expects readable objects.

Normalization is the compatibility layer between those two worlds. It is not a
replacement for validation. It performs small, deterministic shape repairs
before JSON Schema validation and before persisted runtime artifacts when later
pipeline stages need the normalized shape.

---

## 2. Current Runtime Rules

The implementation lives in:

`src-tauri/src/prompt_packs/stage_output_normalization.rs`

Current normalizations:

- `stageIoVersion` -> `stage_io_version`;
- `schemaVersion` -> `schema_version`;
- missing optional readable arrays become `[]` where the runtime contract
  expects an array;
- synthesis readable strings are wrapped as objects:
  - `limitations: ["text"]` -> `[{ "text": "text" }]`;
  - `warning_candidates: ["text"]` -> `[{ "text": "text" }]`;
  - `cross_video_themes: ["text"]` -> `[{ "theme_text": "text" }]`;
  - `common_claims: ["text"]` -> `[{ "summary_text": "text" }]`;
  - `contradictions_across_videos: ["text"]` -> `[{ "description": "text" }]`.

Normalization must stay deterministic and local to representation. It must not:

- invent source, claim, evidence, video, relation, or backend-owned IDs;
- create traversal refs;
- hide unknown or dangling refs;
- weaken pipeline invariants such as evidence ownership or synthesis traversal
  coverage.

Those checks remain validator responsibilities.

---

## 3. Optional Candidate Links

YouTube transcript-analysis output may include optional candidate-to-candidate
links such as `segment_candidate_index` and `quote_candidate_index`.

These links are best-effort provider hints. If an index points to a candidate
that was not retained in the intermediate graph, the runtime drops only that
link and records an `intermediate_entities.warnings` entry such as
`dropped_invalid_segment_candidate_index` or
`dropped_invalid_quote_candidate_index`.

This keeps local linkage noise from failing the whole transcript stage while
preserving stricter validation for malformed indexes, unknown `material_refs`,
backend-owned IDs, dangling final refs, and synthesis traversal invariants.

---

## 4. Persistence Rule

If downstream runtime code consumes a normalized shape, persist the normalized
shape as `parsed_output`.

For example, YouTube synthesis normalizes provider output before validation and
before saving `parsed_output`, because canonical result assembly reads
`synthesis_candidate.common_claims`, `limitations`, and related arrays as object
arrays.

Raw provider output is still preserved separately as `raw_output`.

---

## 5. Testing Expectations

Normalization changes should include tests for both layers:

- validator tests proving provider-friendly shapes are accepted only when the
  normalized result still satisfies schema and custom validation;
- runtime tests proving persisted `parsed_output` contains the normalized shape
  when later pipeline code depends on it.

At least one real single-video and one real multi-video YouTube Summary run
should be checked after changing normalization behavior.
