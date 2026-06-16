# YouTube Summary Derived Traversal Validation Design

Date: 2026-06-16

Status: draft. This document defines the backend slice after YouTube Summary
result-level validation.

## Source Documents

- `docs/superpowers/specs/2026-06-14-youtube-summary-prompt-pack-mvp-design.md`
- `docs/superpowers/specs/2026-06-16-youtube-summary-intermediate-entities-design.md`
- `docs/superpowers/specs/2026-06-16-youtube-summary-result-validation-design.md`
- `docs/prompt-packs/youtube_summary_pack_spec.md`
- `docs/prompt-packs/validation_rules.md`
- `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`
- `src-tauri/src/prompt_packs/result_builder.rs`

## Problem

The current YouTube Summary result validator now checks canonical result shape,
backend-owned identity uniqueness, known references, advisory quality flags, and
the final persistence hard gate. It catches a dangling reference such as
`claim_999`.

It still does not validate that derived traversal arrays accurately summarize
the nested data they represent. A canonical result can therefore contain only
known refs but still be internally inconsistent, for example:

```json
{
  "synthesis": {
    "cross_video_themes": [
      { "claim_refs": ["claim_1"], "evidence_refs": ["evidence_1"] }
    ],
    "claim_refs": [],
    "evidence_refs": []
  }
}
```

Here every nested ref may be known, but the top-level synthesis traversal arrays
are wrong. This makes downstream navigation, UI projections, diagnostics, and
future validator-manifest work less trustworthy.

## Goals

- Extend the existing result-level validator with derived traversal consistency
  checks for the current YouTube Summary canonical result.
- Validate exact derived unions for `synthesis.claim_refs`,
  `synthesis.evidence_refs`, and `synthesis.source_refs`.
- Add minimal `videos[]` traversal validation that is safe for the fields the
  current canonical result may emit.
- Reuse existing result-level findings and persistence behavior.
- Keep this as a backend-only slice with no migrations and no UI changes.
- Keep the implementation focused on rules that can be derived from the current
  canonical JSON without a full validator manifest or schema engine.

## Non-Goals

- No full implementation of `validator_manifest.md`.
- No JSON Schema engine integration.
- No UI changes for validation findings.
- No canonical result repair or healing.
- No revalidation command for historical persisted results.
- No deep/strict pack rules such as quote word counts, timestamp-range checks,
  or multi-source evidence coverage.
- No exact `synthesis.relation_refs` derived-union validation in this slice.
  The current canonical result does not expose a top-level `claim_relations`
  registry, so relation validation remains limited to shape and known-ref
  checks where possible.
- No full `VR-YS-020` validation for every nested video item. The validator may
  validate fields that are already present, but it should not require traversal
  fields that the current canonical builder does not emit.

## Current Backend Shape

The current terminal path is:

```text
build_youtube_summary_canonical_result
  -> validate_youtube_summary_canonical_result
  -> replace result-level findings
  -> if error findings: fail run without result/projection persistence
  -> else: persist result and projections
```

This slice stays inside `validate_youtube_summary_canonical_result`. It should
not change the persistence wrapper, execution wiring, database schema, or Tauri
commands.

## Validation Rules

Rule-code note: `VR-YS-015` is intentionally shared by claim, evidence, and
source traversal violations because the current validation rule registry defines
it as the single YouTube Summary synthesis traversal-union rule. Implementations
should distinguish the concrete failure by `object_path` and message, for
example `$.outputs.pack_data.youtube_summary.synthesis.claim_refs` versus
`$.outputs.pack_data.youtube_summary.synthesis.source_refs`, instead of
inventing unregistered subcodes.

### Synthesis Claim Refs

If `outputs.pack_data.youtube_summary.synthesis` is an object,
`synthesis.claim_refs` must exactly match the unique ordered union of nested
`claim_refs` from:

- `synthesis.cross_video_themes[]`;
- `synthesis.common_claims[]`;
- `synthesis.contradictions_across_videos[]`.

The expected union uses deterministic first-seen order:

1. iterate `cross_video_themes`;
2. iterate `common_claims`;
3. iterate `contradictions_across_videos`;
4. for each item, collect nested `claim_refs` in array order.

Both missing refs and extra refs produce `error` findings.

Recommended rule code: `VR-YS-015`.

### Synthesis Evidence Refs

If synthesis is an object, `synthesis.evidence_refs` must exactly match the
unique ordered union of nested `evidence_refs` from:

- `synthesis.cross_video_themes[]`;
- `synthesis.common_claims[]`;
- `synthesis.contradictions_across_videos[]`.

The expected union uses deterministic first-seen order:

1. iterate `cross_video_themes`;
2. iterate `common_claims`;
3. iterate `contradictions_across_videos`;
4. for each item, collect nested `evidence_refs` in array order.

Both missing refs and extra refs produce `error` findings.

Recommended rule code: `VR-YS-015`.

### Synthesis Source Refs

If synthesis is an object, `synthesis.source_refs` must exactly match the
unique ordered derived source refs from nested synthesis items.

The MVP derivation should include:

1. every string in nested item `source_refs[]`;
2. the `source_ref_id` of every `videos[]` entry referenced by nested item
   `video_refs[]`.

If a `video_refs[]` value does not resolve to a known `videos[].video_id`, this
derivation should skip that value rather than adding a second cascading
traversal finding. The existing known-ref validation layer owns the unknown
video-ref error.

The validator should preserve deterministic first-seen order when building the
expected union:

1. iterate `cross_video_themes`;
2. iterate `common_claims`;
3. iterate `contradictions_across_videos`;
4. for each item, collect nested `source_refs` first, then source refs derived
   from `video_refs`.

Both missing refs and extra refs produce `error` findings.

Recommended rule code: `VR-YS-015`.

This intentionally does not add source refs reachable through referenced claims
or evidence in the first implementation. That deeper pack-spec derivation can
be added later once the validator owns broader graph traversal helpers.

### Synthesis Relation Refs

`synthesis.relation_refs` remains shape-validated as an array by the previous
result-validation slice. Exact derived-union validation for relation refs is out
of scope until the canonical result exposes authoritative relation objects.

The existing shape check for `relation_refs` (`VR-YS-001` via
`validate_synthesis_shape`) must not be removed or weakened by this slice.

### Video Source Refs

For each `outputs.pack_data.youtube_summary.videos[]` item:

- `source_ref_id` is already required to reference a known source;
- if `video.source_refs` is present, it must be an array of strings;
- if `video.source_refs` is present, every string value must reference a known
  top-level `source_refs[].source_ref_id`;
- if `video.source_refs` is present, it must include `video.source_ref_id`.

Failure produces an `error` finding.

Recommended rule codes:

- malformed `video.source_refs` shape uses `VR-YS-020`;
- unknown string refs inside `video.source_refs` use the same existing
  unknown-ref code used by the result validator for other unknown refs;
- missing `video.source_ref_id` in `video.source_refs` uses `VR-YS-004`.

### Video Claim And Evidence Refs

If `video.claim_refs` or `video.evidence_refs` is present, the field must be an
array of strings whose values are known refs. This is a guarded extension of the
existing known-ref validation and should not require the fields to exist.

Exact derivation from nested video items is out of scope for this slice because
the current canonical builder does not yet guarantee all nested item traversal
fields needed for full `VR-YS-020`.

Recommended rule code for malformed traversal field shape: `VR-YS-020`.
Unknown string refs may keep using the existing unknown-ref code used by the
result validator.

## Comparison Semantics

Derived traversal arrays should be compared as ordered unique sets:

- duplicate values in an actual traversal array should be treated as
  inconsistent, because canonical traversal arrays are expected to be unique;
- duplicate findings should use the path of the traversal field itself, not the
  individual duplicate index, for example
  `$.outputs.pack_data.youtube_summary.synthesis.claim_refs`;
- expected values for claim, evidence, and source traversal arrays are collected
  in the deterministic first-seen order defined above;
- order differences do not fail if the set is the same. The first-seen order is
  the canonical reference for expected unions;
- finding messages should identify missing and extra refs separately when
  practical.

The implementation should avoid noisy cascades. If a relevant parent field is
not an array, the existing shape rule should report that error, and the derived
union check may skip that parent path.

## Finding Model

Use the existing `PromptPackResultValidationFinding` type and result-level
finding persistence behavior.

Suggested messages:

- `synthesis.claim_refs missing: ["claim_1"]`
- `synthesis.evidence_refs extra: ["evidence_3"]`
- `synthesis.source_refs missing: ["source_ref_2"]`
- `video.source_refs must include self source_ref_id "source_ref_1"`

Object paths should point to the traversal field being validated, for example:

- `$.outputs.pack_data.youtube_summary.synthesis.claim_refs`
- `$.outputs.pack_data.youtube_summary.synthesis.source_refs`
- `$.outputs.pack_data.youtube_summary.videos[0].source_refs`

## Implementation Shape

Add focused helpers inside `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`:

- `validate_synthesis_derived_traversal_refs(...)`
- `derive_synthesis_claim_refs(...)`
- `derive_synthesis_evidence_refs(...)`
- `derive_synthesis_source_refs(...)`
- `validate_video_traversal_refs(...)`
- small reusable helpers for ordered unique ref collection and comparison.

### Rust Interface

Recommended internal signatures:

```rust
fn validate_synthesis_derived_traversal_refs(
    synthesis: &serde_json::Map<String, serde_json::Value>,
    video_source_by_id: &std::collections::HashMap<String, String>,
    findings: &mut Vec<PromptPackResultValidationFinding>,
);

fn derive_synthesis_claim_refs(
    synthesis: &serde_json::Map<String, serde_json::Value>,
) -> Vec<String>;

fn derive_synthesis_evidence_refs(
    synthesis: &serde_json::Map<String, serde_json::Value>,
) -> Vec<String>;

fn derive_synthesis_source_refs(
    synthesis: &serde_json::Map<String, serde_json::Value>,
    video_source_by_id: &std::collections::HashMap<String, String>,
) -> Vec<String>;

fn validate_video_traversal_refs(
    videos: Option<&Vec<serde_json::Value>>,
    source_ids: &std::collections::HashSet<String>,
    claim_ids: &std::collections::HashSet<String>,
    evidence_ids: &std::collections::HashSet<String>,
    findings: &mut Vec<PromptPackResultValidationFinding>,
);
```

`video_source_by_id` maps `video_id -> source_ref_id` for videos with a
non-empty `video_id` and a `source_ref_id` that was accepted by the identity
check on canonical `source_refs[]` (`source_ref_id in source_ids`). It is
derived from the canonical `videos[]` array after existing identity and
known-ref checks have collected `video_ids` and `source_ids`, so
`derive_synthesis_source_refs(...)` can skip unknown `video_refs[]` without
creating cascade errors. Exact visibility may stay private to the module.

The existing validator should call these helpers after shape, identity, and
known-ref checks have run. This keeps error reporting predictable and lets the
new checks rely on known source/video/claim/evidence registries when available.
`validate_result_refs(...)` remains responsible for existing object refs such as
`videos[].source_ref_id` and synthesis nested refs. The new
`validate_video_traversal_refs(...)` should cover only the guarded traversal
arrays introduced or tightened by this slice: `videos[].source_refs`,
`videos[].claim_refs`, and `videos[].evidence_refs`. It may reuse the same
known-ref helper functions and registries, but it should not duplicate the base
iteration solely to re-check `videos[].source_ref_id`.

No changes are expected in:

- `youtube_summary::execution`;
- final persistence wrapper;
- projection tables;
- Tauri commands.

## Test Strategy

Unless a test is explicitly described as a helper-level unit test, these tests
should exercise `validate_youtube_summary_canonical_result(...)` using the
existing test-support canonical fixture style. Helper-level tests may set
`source_ids`, `claim_ids`, `evidence_ids`, or `video_source_by_id` directly when
that is the behavior under test.

Add pure validator tests for:

- valid synthesis derived traversal fields produce no `error` findings;
- `synthesis = null` produces no traversal error findings;
- nested `cross_video_themes[0].claim_refs = ["claim_1"]` with empty
  `synthesis.claim_refs` returns `VR-YS-015`;
- `synthesis.claim_refs = ["claim_1", "claim_2"]` when only nested
  `claim_1` exists returns `VR-YS-015`;
- `synthesis.claim_refs = ["claim_1", "claim_1"]` returns an error finding for
  duplicate actual traversal refs;
- nested `evidence_refs` missing from top-level `synthesis.evidence_refs`
  returns `VR-YS-015`;
- top-level `synthesis.evidence_refs` containing an extra evidence ref returns
  `VR-YS-015`;
- nested `video_refs = ["video_2"]` with `video_2.source_ref_id =
  "source_ref_2"`, where `source_ref_2` exists in canonical `source_refs[]` but
  is missing from `synthesis.source_refs`, returns `VR-YS-015`;
- nested item `source_refs = ["source_ref_1"]` missing from
  `synthesis.source_refs` returns `VR-YS-015`;
- `videos[0].source_refs = []` while `videos[0].source_ref_id =
  "source_ref_1"` returns `VR-YS-004`;
- `videos[0].source_refs = "not_an_array"` returns `VR-YS-020`;
- full-validator test with `videos[0].source_refs = ["source_ref_missing"]`
  returns an unknown-ref error;
- missing `video.source_refs` is allowed in this slice;
- present `video.claim_refs` with an unknown claim returns an error;
- present `video.evidence_refs` with an unknown evidence ref returns an error.

Run at minimum:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
cargo check --manifest-path src-tauri\Cargo.toml
git diff --check
```

## Rollout

Implement this as one focused result-validator slice:

1. Add pure tests for synthesis derived unions.
2. Add ordered-union helpers and synthesis traversal validation.
3. Add pure tests for guarded video traversal validation.
4. Add video traversal validation.
5. Run all commands listed in Test Strategy.

No data migration is required. Existing persisted results are not revalidated
until a future explicit revalidation command exists.

## Open Follow-Ups

- Full `synthesis.relation_refs` validation once canonical claim relations are
  present.
- Full `VR-YS-020` derivation from nested video `segments`, `key_points`,
  `notable_quotes`, `action_items`, `open_questions`, and related synthesis
  objects.
- Source refs reachable through referenced claims and evidence for the complete
  `VR-YS-015` source-ref derivation algorithm.
- Future canonical traversal-order normalization, if needed, should use the
  same first-seen order for all three synthesis traversal fields.
- Full validator manifest integration.
- Historical result revalidation command.
