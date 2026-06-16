# YouTube Summary Result Validation Design

Date: 2026-06-16

Status: draft. This document defines the backend slice after YouTube Summary
intermediate entities.

## Source Documents

- `docs/superpowers/specs/2026-06-14-youtube-summary-prompt-pack-mvp-design.md`
- `docs/superpowers/specs/2026-06-16-youtube-summary-intermediate-entities-design.md`
- `docs/prompt-packs/youtube_summary_pack_spec.md`
- `docs/prompt-packs/validation_rules.md`
- `docs/prompt-packs/validator_manifest.md`
- `src-tauri/src/prompt_packs/result_builder.rs`
- `src-tauri/src/prompt_packs/projections.rs`
- `src-tauri/src/prompt_packs/validation.rs`
- `src-tauri/src/prompt_packs/youtube_summary/execution.rs`

## Problem

The YouTube Summary backend now builds source-scoped `intermediate_entities`
artifacts, exposes merged `canonical_graph` and `allowed_refs` to synthesis,
validates synthesis refs, and prefers graph claims/evidence in the canonical
result when a complete graph set exists.

The final canonical result is still mostly trusted after
`build_youtube_summary_canonical_result`. Projection persistence assumes the
canonical JSON is internally consistent and writes normalized rows with
best-effort defaults such as empty strings for missing ids. The database already
has `prompt_pack_result_validation_findings`, and the UI/API can read findings,
but the final result path does not yet have a result-level validation gate.

This leaves a gap: stage-level validation can be strict, while final result
assembly can still create duplicate ids, dangling refs, empty required pack
data, or inconsistent synthesis references without surfacing a structured
finding before projections are persisted.

## Goals

- Add a result-level validation pass for backend-built YouTube Summary
  canonical results.
- Store result-level validation findings in the existing
  `prompt_pack_result_validation_findings` table with `stage_run_id = NULL`.
- Treat `error` findings as a hard gate before final result/projection
  persistence.
- Treat `warning` and `info` findings as advisory: persist them and continue
  normal result/projection persistence.
- Keep the first slice focused on the graph/result MVP, not the full reference
  validator.
- Preserve existing stage-level quarantine behavior and stage-level findings.
- Avoid schema migrations, new Tauri commands, and UI changes in this slice.

## Non-Goals

- No full implementation of `validator_manifest.md`.
- No JSON Schema engine integration for canonical result schemas.
- No validation pipeline stage row.
- No new projection tables.
- No UI changes for findings display.
- No repair or healing of invalid canonical results.
- No validation of future pack-specific deep structures that are not emitted by
  the current YouTube Summary result builder.
- No change to synthesis-output validation policy.
- No requirement to detect non-string synthesis ref items after canonical result
  assembly. Raw synthesis-output validation owns that check; the current result
  builder drops non-string ref items while constructing canonical JSON.
- No derived-union consistency validation for synthesis top-level traversal
  arrays in this slice. The validator checks that refs are known, but does not
  require `synthesis.claim_refs`, `synthesis.evidence_refs`,
  `synthesis.source_refs`, or `synthesis.relation_refs` to exactly equal the
  union of nested synthesis item refs.

## Current Backend Shape

Current terminal execution shape:

```text
transcript stages
  -> intermediate_entities artifacts
  -> optional synthesis stage
  -> build_youtube_summary_canonical_result
  -> persist_final_result_transaction
  -> normalized projections
  -> terminal run/result status
```

`persist_final_result_transaction` currently owns final result insertion,
projection rebuild, terminal run status update, and the
`terminal_result_persisted` audit event.

The new validation gate should sit immediately between canonical result building
and final persistence:

```text
build_youtube_summary_canonical_result
  -> validate_youtube_summary_canonical_result
  -> replace result-level findings
  -> if error findings: fail run without result/projection persistence
  -> else: persist result and projections
```

## Validation Finding Model

The implementation should use a small internal finding type that maps directly
to the existing DB and DTO shape:

```rust
pub(crate) struct PromptPackResultValidationFinding {
    pub(crate) severity: String,
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) object_path: Option<String>,
}
```

Allowed severities are `error`, `warning`, and `info`.

`code` is the stable rule identifier stored in the existing DB column. It must
not be an arbitrary free-form label. Use the rule id from
`validation_rules.md` when the finding maps to an existing rule, for example
`VR-YS-001`, `VR-YS-002`, or `VR-YS-005`. For result-validation MVP rules that
do not yet have a manifest id, use a stable local code in the
`RV-RESULT-*` namespace with the same numeric suffix shape as the core
validation-finding schema, such as `RV-RESULT-001` for duplicate ids and
`RV-RESULT-002` for unknown refs.

Result-level findings must be persisted with:

- `run_id = current run`;
- `stage_run_id = NULL`;
- `severity`, `code`, `message`, and `object_path` from the finding;
- `created_at = now`.

Before inserting new result-level findings for a run, delete only previous
result-level rows:

```sql
DELETE FROM prompt_pack_result_validation_findings
WHERE run_id = ? AND stage_run_id IS NULL
```

Stage-level findings must remain untouched.

## Validation Rules

This slice intentionally implements a small set of rules that protects the
current canonical result and graph-backed references.

### Shape Rules

- `schema_version` must be `"1.0"`.
- `pack_id` must be `"youtube_summary"`.
- `run_id` must match the run being persisted.
- `outputs.pack_data.youtube_summary` must be an object.
- `outputs.pack_data.youtube_summary.videos` must be an array.
- `outputs.pack_data.youtube_summary.synthesis` must be present and may be
  `null` or an object.
- `source_refs`, `claims`, `evidence`, `warnings`, `limitations`,
  `quality_flags`, and `audit_refs` must be arrays.

Shape rule failures produce `error` findings.

### Closed-World Identity Rules

- Every `source_refs[].source_ref_id` must be a non-empty string.
- Every `outputs.pack_data.youtube_summary.videos[].video_id` must be a
  non-empty string.
- Every `claims[].claim_id` must be a non-empty string.
- Every `evidence[].evidence_id` must be a non-empty string.
- `source_ref_id`, `video_id`, `claim_id`, and `evidence_id` values must be
  unique within their own arrays.
- If synthesis is an object, every `cross_video_themes[].theme_id`,
  `common_claims[].common_claim_id`, and
  `contradictions_across_videos[].contradiction_id` must be a non-empty string.
- The combined synthesis item id set from `theme_id`, `common_claim_id`, and
  `contradiction_id` must be unique, because projection stores all three item
  kinds in one `prompt_pack_youtube_synthesis_items.synthesis_id` namespace.

Identity rule failures produce `error` findings.

### Reference Rules

- Every `videos[].source_ref_id` must reference a known
  `source_refs[].source_ref_id`.
- Every non-null `claims[].source_ref_id` must reference a known source.
- Every non-null `evidence[].source_ref_id` must reference a known source.
- Every non-null `evidence[].claim_id` must reference a known claim.
- Every synthesis `source_refs[]` value must reference a known source.
- Every synthesis `claim_refs[]` value must reference a known claim.
- Every synthesis `evidence_refs[]` value must reference known evidence.
- Every synthesis item `video_refs[]` value in `cross_video_themes[]`,
  `common_claims[]`, and `contradictions_across_videos[]` must reference a
  known `outputs.pack_data.youtube_summary.videos[].video_id`.
- Synthesis refs must be checked both in the top-level synthesis union arrays
  and in nested synthesis items such as `cross_video_themes[]`,
  `common_claims[]`, and `contradictions_across_videos[]`. A nested
  `claim_refs: ["claim_999"]` must produce an `error` finding even when the
  top-level `synthesis.claim_refs` union array is empty.

Reference rule failures produce `error` findings.

The canonical result validator should not treat non-string synthesis ref items
as a required runtime-detectable case. `result_builder.rs` currently gathers
synthesis refs through `ref_strings`, which filters out non-string values before
canonical JSON is produced. The result-level validator still must catch unknown
string refs such as `claim_999`. Non-string ref items in raw LLM output remain
the responsibility of synthesis-output validation before result building.

### YouTube Pack Rules

- If terminal status is `complete` and the run's `evidence_mode` is not
  `"narrative_only"`, `outputs.pack_data.youtube_summary.videos` must not be
  empty.
- If there is exactly one video, synthesis must be `null`.
- If synthesis is an object, all canonical synthesis fields emitted by the
  current result builder must be present and must be arrays:
  `cross_video_themes`, `common_claims`, `contradictions_across_videos`,
  `claim_refs`, `relation_refs`, `evidence_refs`, and `source_refs`.

YouTube pack rule failures produce `error` findings.

### Advisory Rules

The validator should also surface advisory findings that do not block
persistence:

- If `quality_flags` contains `intermediate_entities_legacy_fallback`, emit a
  `warning` finding explaining that claims/evidence used legacy parsed-output
  assembly.
- If `quality_flags` contains `synthesis_not_applicable_single_video`, emit an
  `info` finding explaining that synthesis was intentionally skipped for a
  single-video run.
- If `quality_flags` contains `synthesis_failed` or
  `synthesis_skipped_insufficient_successes`, emit a `warning` finding.

These advisory findings are intentionally duplicated into the findings table so
diagnostics can be read through one backend API without parsing canonical JSON.
The current result builder stores quality flags as `{ flag, severity }` without
messages. The validator must synthesize finding messages for the known flags
listed above. Unknown quality flags are ignored by this MVP validator rather
than converted into generic findings.

## Persistence Policy

Add a validation-aware final persistence path, for example:

```rust
pub(crate) async fn validate_and_persist_final_result_transaction(
    pool: &SqlitePool,
    run_id: i64,
    canonical_result: serde_json::Value,
    terminal_status: &str,
) -> AppResult<()>
```

The function should:

1. Load run context needed by the validator, at minimum `evidence_mode`.
2. Run `validate_youtube_summary_canonical_result`.
3. Replace result-level findings for the run.
4. If no `error` findings exist, persist the canonical result and projection
   rows through a transaction-aware helper shared with the existing
   `persist_final_result_transaction` wrapper.
5. If any `error` findings exist:
   - delete any existing `prompt_pack_results` row for this `run_id`, relying
     on cascading foreign keys to remove stale projections;
   - do not rebuild projection rows;
   - set `prompt_pack_runs.run_status = 'failed'`;
   - set `prompt_pack_runs.result_status = 'failed'`;
   - set `prompt_pack_runs.latest_message` to a concise validation failure
     message;
   - set `completed_at` and `updated_at` to current time;
   - insert a `prompt_pack_audit_events` row with
     `event_kind = 'terminal_result_validation_failed'`;
   - return an `AppError::validation` with the same concise message.

This hard-gate branch should not write a partial canonical result. A result that
fails result-level validation is treated as not safely projectable.

The validation-aware persistence path must be atomic for finding replacement,
result deletion or result persistence, projection writes, run-status updates,
and audit-event insertion. The current `persist_final_result_transaction` takes
`&SqlitePool`, so it cannot simply be called inside this wrapper while
preserving one SQL transaction. The implementation should extract the shared
result/projection persistence body into a transaction-aware internal helper,
then keep the existing `persist_final_result_transaction` as a thin wrapper
that opens a transaction and calls the helper.

## Execution Integration

`youtube_summary::execution` should call the validation-aware persistence path
after `build_youtube_summary_canonical_result`.

If result validation fails, the executor should return the validation error
after the run has been marked failed and findings/audit have been persisted by
the validation-aware persistence function.

Existing stage-level behavior remains unchanged:

- transcript output validation still owns transcript quarantine;
- intermediate graph builder still owns graph build quarantine;
- synthesis output validation still owns synthesis quarantine;
- result validation does not create quarantine artifacts.

## Projection Repair

`repair_prompt_pack_result_projections` should remain a projection repair tool
for already persisted canonical JSON. It should not run the new result
validation gate in this slice.

If a future slice needs canonical-result revalidation for old persisted runs, it
should add a separate explicit command or maintenance function. That is out of
scope here.

## Test Strategy

Add pure validator tests for:

- a current valid YouTube Summary canonical result has no `error` findings;
- duplicate `source_ref_id` returns an `error` finding;
- missing, blank, or duplicate `videos[].video_id` returns an `error` finding;
- duplicate `claim_id` returns an `error` finding;
- duplicate `evidence_id` returns an `error` finding;
- a video with an unknown `source_ref_id` returns an `error` finding;
- evidence with an unknown `claim_id` returns an `error` finding;
- synthesis with an unknown `claim_refs[]` value returns an `error` finding;
- nested synthesis item refs, for example
  `cross_video_themes[0].claim_refs[0] = "claim_999"`, return an `error`
  finding even when top-level synthesis union refs are empty;
- nested synthesis item `video_refs`, for example
  `cross_video_themes[0].video_refs[0] = "video_missing"`, return an `error`
  finding;
- missing, blank, or duplicate synthesis item ids across `theme_id`,
  `common_claim_id`, and `contradiction_id` return an `error` finding;
- synthesis object missing any required canonical synthesis array returns an
  `error` finding;
- a complete non-`narrative_only` result with empty videos returns an `error`
  finding;
- `intermediate_entities_legacy_fallback` emits a `warning` finding and no
  `error` finding.

Add persistence tests for:

- warning-only findings are persisted and the result/projections are still
  written;
- error findings are persisted with `stage_run_id = NULL`;
- error findings prevent `prompt_pack_results` and projection rows from being
  written;
- error findings mark the run failed and create
  `terminal_result_validation_failed` audit event;
- replacing result-level findings does not delete stage-level findings.
- stale `prompt_pack_results` and projection rows for the same run are removed
  when a later validation attempt produces `error` findings;
- if result persistence or projection insertion fails after validation, the
  findings replacement and partial result writes are rolled back together.

Add YouTube Summary execution regression tests for:

- a valid run still completes and persists final result/projections;
- an intentionally invalid canonical result path records result-level findings
  and marks the run failed.

Run at minimum:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
cargo check --manifest-path src-tauri\Cargo.toml
```

## Rollout

Implement this as one focused backend slice:

1. Add pure result validator and tests.
2. Add result-level finding persistence helpers and tests.
3. Add validation-aware final persistence wrapper and tests.
4. Wire YouTube Summary execution to the wrapper.
5. Run focused and broad Prompt Pack test suites.

No user data migration is required.

## Open Follow-Ups

- Full canonical-result JSON Schema validation.
- Full `validator_manifest.md` implementation.
- Revalidation command for already persisted historical Prompt Pack results.
- UI affordances for result-level finding severity filters.
- Dedicated result validation stage rows if future workflows need validator
  stages in the pipeline graph.
- Derived-union validation for YouTube Summary synthesis traversal fields,
  including exact equality between top-level `synthesis.claim_refs`,
  `synthesis.evidence_refs`, `synthesis.source_refs`, `synthesis.relation_refs`
  and the refs reachable from nested synthesis items.
