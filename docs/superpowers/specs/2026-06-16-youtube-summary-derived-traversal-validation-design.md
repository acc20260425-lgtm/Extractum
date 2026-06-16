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

### Synthesis Claim Refs

If `outputs.pack_data.youtube_summary.synthesis` is an object,
`synthesis.claim_refs` must exactly match the unique ordered union of nested
`claim_refs` from:

- `synthesis.cross_video_themes[]`;
- `synthesis.common_claims[]`;
- `synthesis.contradictions_across_videos[]`.

Both missing refs and extra refs produce `error` findings.

Recommended rule code: `VR-YS-015`.

### Synthesis Evidence Refs

If synthesis is an object, `synthesis.evidence_refs` must exactly match the
unique ordered union of nested `evidence_refs` from:

- `synthesis.cross_video_themes[]`;
- `synthesis.common_claims[]`;
- `synthesis.contradictions_across_videos[]`.

Both missing refs and extra refs produce `error` findings.

Recommended rule code: `VR-YS-015`.

### Synthesis Source Refs

If synthesis is an object, `synthesis.source_refs` must exactly match the
unique ordered derived source refs from nested synthesis items.

The MVP derivation should include:

1. every string in nested item `source_refs[]`;
2. the `source_ref_id` of every `videos[]` entry referenced by nested item
   `video_refs[]`.

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

If the current validator already checks known relation refs in the future, this
slice must not weaken that behavior.

### Video Source Refs

For each `outputs.pack_data.youtube_summary.videos[]` item:

- `source_ref_id` is already required to reference a known source;
- if `video.source_refs` is present, it must be an array of strings;
- if `video.source_refs` is present, it must include `video.source_ref_id`.

Failure produces an `error` finding.

Recommended rule code: `VR-YS-004`.

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
- order differences should not fail if the set is the same, unless the
  implementation already normalizes order cheaply;
- finding messages should identify missing and extra refs separately when
  practical.

The implementation should avoid noisy cascades. If a relevant parent field is
not an array, the existing shape rule should report that error, and the derived
union check may skip that parent path.

## Finding Model

Use the existing `PromptPackResultValidationFinding` type and result-level
finding persistence behavior.

Suggested messages:

- `synthesis.claim_refs is missing derived refs: claim_1`
- `synthesis.evidence_refs contains refs not present in nested synthesis items: evidence_3`
- `synthesis.source_refs is missing refs derived from nested synthesis video_refs: source_ref_2`
- `video.source_refs must include video.source_ref_id source_ref_1`

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

The existing validator should call these helpers after shape, identity, and
known-ref checks have run. This keeps error reporting predictable and lets the
new checks rely on known source/video/claim/evidence registries when available.

No changes are expected in:

- `youtube_summary::execution`;
- final persistence wrapper;
- projection tables;
- Tauri commands.

## Test Strategy

Add pure validator tests for:

- valid synthesis derived traversal fields produce no `error` findings;
- nested `cross_video_themes[0].claim_refs = ["claim_1"]` with empty
  `synthesis.claim_refs` returns `VR-YS-015`;
- `synthesis.claim_refs = ["claim_1", "claim_2"]` when only nested
  `claim_1` exists returns `VR-YS-015`;
- nested `evidence_refs` missing from top-level `synthesis.evidence_refs`
  returns `VR-YS-015`;
- top-level `synthesis.evidence_refs` containing an extra evidence ref returns
  `VR-YS-015`;
- nested `video_refs = ["video_2"]` with `video_2.source_ref_id =
  "source_ref_2"` and missing `source_ref_2` in `synthesis.source_refs`
  returns `VR-YS-015`;
- nested item `source_refs = ["source_ref_1"]` missing from
  `synthesis.source_refs` returns `VR-YS-015`;
- `videos[0].source_refs = []` while `videos[0].source_ref_id =
  "source_ref_1"` returns `VR-YS-004`;
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
5. Run focused and broad YouTube Summary tests.

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
- Full validator manifest integration.
- Historical result revalidation command.
