# Validator Fixtures

Status: v1 draft.

Compatibility:

- Prompt Pack JSON Contract `schema_version: "1.0"`;
- Stage I/O Contracts `stage_io_version: "1.0"`;
- Validator Manifest `validator_manifest_version: "1.0"`.

This document defines the fixture catalog for the first reference validator. It
does not add validation rules. It translates `validator_manifest.md` into a
concrete set of fixture classes, paths, naming conventions, and expected
findings.

---

## 1. Purpose

`validator_fixtures.md` answers:

- where validator fixtures will live;
- how fixtures are named;
- which fixtures are mandatory for the first validator implementation;
- how expected findings are stored;
- how stage payload, stage output, prompt template, and canonical result
  fixtures differ;
- which fixtures are allowed to be lightweight projections instead of full
  production examples.

The goal is to let validator implementation start with a stable test matrix.

---

## 2. Fixture Directory Layout

Validator fixtures live under:

```text
docs/prompt-packs/fixtures/v1/
```

Planned layout:

```text
fixtures/
  v1/
    fixture_manifest.json
    canonical-result/
      valid/
      invalid-schema/
      invalid-reference/
      invalid-pipeline/
      qa-warning/
    stage-payload/
      valid/
      invalid-registry/
      invalid-version/
    stage-output/
      valid/
      invalid-schema/
      invalid-reference/
      partial-failure/
    prompt-template/
      valid/
      invalid/
    expected-findings/
      canonical-result/
      stage-payload/
      stage-output/
      prompt-template/
```

Rules:

- Fixture JSON files use `.fixture.json`.
- Expected finding files use `.expected.json`.
- Fixture directories are grouped by validation mode and failure class.
- The fixture path itself is not part of canonical result JSON.
- The prose fixture catalog remains authoritative for fixture classes that do
  not yet have checked-in concrete files.

---

## 3. Naming Convention

Fixture filenames use:

```text
<mode>__<class>__<case_id>.fixture.json
```

Expected finding filenames use:

```text
<mode>__<class>__<case_id>.expected.json
```

Examples:

```text
canonical_result__valid__technology_watch_minimal.fixture.json
canonical_result__invalid_reference__dangling_evidence_claim.expected.json
stage_output__partial_failure__one_bad_claim_candidate.fixture.json
prompt_template__invalid__assigns_canonical_ids.expected.json
```

Rules:

- `mode` matches one of the validator modes from `validator_manifest.md`.
- `class` matches the fixture class directory.
- Directory class names use kebab-case; fixture ID class tokens use snake_case.
  They map by replacing `-` with `_`.
- `case_id` is lowercase snake case.
- Fixture and expected-finding files share the same basename before the suffix.

---

## 4. Fixture Manifest

Each fixture set should include `fixture_manifest.json`.

```json
{
  "fixture_manifest_version": "1.0",
  "validator_manifest_version": "1.0",
  "schema_version": "1.0",
  "stage_io_version": "1.0",
  "fixtures": [
    {
      "fixture_id": "canonical_result__valid__technology_watch_minimal",
      "validation_mode": "canonical_result",
      "fixture_path": "canonical-result/valid/canonical_result__valid__technology_watch_minimal.fixture.json",
      "expected_findings_path": "expected-findings/canonical-result/canonical_result__valid__technology_watch_minimal.expected.json",
      "expected_error_count": 0,
      "expected_warning_count": 0,
      "required": true
    }
  ]
}
```

Rules:

- `fixture_id` equals the fixture basename without `.fixture.json`.
- `expected_error_count` and `expected_warning_count` are assertions.
- Required fixtures must pass in CI for the reference validator baseline.
- Optional fixtures may be used for local development and regression coverage.

---

## 5. Expected Findings File

Each fixture has a matching expected findings file.

```json
{
  "fixture_id": "canonical_result__invalid_reference__dangling_evidence_claim",
  "validation_mode": "canonical_result",
  "expected_valid": false,
  "expected_findings": [
    {
      "rule_id": "VR-CORE-022",
      "severity": "error",
      "layer": "reference",
      "object_path": "evidence[0].claim_id",
      "message_contains": "references a missing claim",
      "object_refs": {
        "claim_refs": ["claim_missing"],
        "evidence_refs": ["evidence_1"],
        "source_refs": []
      }
    }
  ]
}
```

Rules:

- `message_contains` is preferred over exact message equality to keep wording
  changes from breaking structural tests.
- `rule_id`, `severity`, `layer`, and `object_path` should be exact matches.
- `object_refs` may contain only the refs relevant to the assertion.
- Extra validator findings are allowed only if the fixture explicitly sets
  `allow_additional_findings: true`.

---

## 6. Mandatory Fixture Matrix

The first reference validator should include these required fixture classes.

| Fixture ID | Mode | Class | Expected result | Primary assertion |
|---|---|---|---|---|
| `canonical_result__valid__technology_watch_minimal` | `canonical_result` | `valid` | valid | No `error` findings |
| `canonical_result__valid__youtube_summary_minimal` | `canonical_result` | `valid` | valid | No `error` findings |
| `canonical_result__valid__telegram_summary_minimal` | `canonical_result` | `valid` | valid | No `error` findings |
| `canonical_result__invalid_schema__missing_outputs` | `canonical_result` | `invalid-schema` | invalid | Required top-level object missing |
| `canonical_result__invalid_reference__dangling_evidence_claim` | `canonical_result` | `invalid-reference` | invalid | `VR-CORE-022` |
| `canonical_result__invalid_pipeline__claim_source_refs_not_superset` | `canonical_result` | `invalid-pipeline` | invalid | `VR-CORE-017` |
| `canonical_result__invalid_pipeline__relation_cycle` | `canonical_result` | `invalid-pipeline` | invalid | `VR-CORE-054` |
| `canonical_result__qa_warning__single_source_claim` | `canonical_result` | `qa-warning` | valid with warning | `VR-CORE-041` |
| `stage_payload__valid__claim_extraction_minimal` | `stage_payload` | `valid` | valid | Stage envelope and allowed IDs valid |
| `stage_payload__invalid_registry__missing_allowed_fragment` | `stage_payload` | `invalid-registry` | invalid | Allowed ID absent from loaded registry |
| `stage_payload__invalid_version__unsupported_stage_io_version` | `stage_payload` | `invalid-version` | invalid | Version gate failure |
| `stage_output__valid__claim_extraction_minimal` | `stage_output` | `valid` | valid | Candidate references allowed fragment |
| `stage_output__invalid_reference__disallowed_fragment_candidate` | `stage_output` | `invalid-reference` | invalid | Candidate references disallowed ID |
| `stage_output__partial_failure__one_bad_claim_candidate` | `stage_output` | `partial-failure` | invalid object, valid siblings | Object-isolated failure |
| `prompt_template__valid__claim_extraction_minimal` | `prompt_template` | `valid` | valid | Contains closed-world instructions and output shape |
| `prompt_template__invalid__assigns_canonical_ids` | `prompt_template` | `invalid` | invalid | Template asks model to assign canonical IDs |

Rules:

- Valid canonical result fixtures may be compact but must be complete enough to
  exercise real graph traversal.
- Invalid fixtures should isolate one primary failure whenever possible.
- Partial-failure fixtures must include at least one valid sibling candidate.
- QA warning fixtures are valid artifacts and should not fail CI unless the
  warning is missing.

---

## 6.1 Checked-In Mandatory Fixture Set

The mandatory fixture matrix is now materialized under `fixtures/v1/`.

| Fixture family | Checked-in coverage |
|---|---|
| Canonical valid | `technology_watch_minimal`, `youtube_summary_minimal`, `telegram_summary_minimal` |
| Canonical invalid schema/reference/pipeline | `missing_outputs`, `dangling_evidence_claim`, `claim_source_refs_not_superset`, `relation_cycle` |
| Canonical QA warning | `single_source_claim` |
| Stage payload | valid `claim_extraction_minimal`, `missing_allowed_fragment`, `unsupported_stage_io_version` |
| Stage output | valid `claim_extraction_minimal`, `disallowed_fragment_candidate`, `one_bad_claim_candidate` |
| Prompt template | valid `claim_extraction_minimal`, invalid `assigns_canonical_ids` |

Rules:

- These files are intended as the first CI baseline for the reference
  validator skeleton.
- Each checked-in fixture has a matching `.expected.json` file.
- Every checked-in fixture is listed in `fixtures/v1/fixture_manifest.json`.
- Future fixtures may expand coverage, but the rows above are the minimum v1
  validator fixture baseline.

### Optional regression fixtures

The manifest may include optional regression fixtures beyond the mandatory
baseline. Optional fixtures use `required: false`, but the reference runner still
validates them when they are listed in `fixture_manifest.json`.

Current optional regression coverage:

| Fixture ID | Primary assertion |
|---|---|
| `canonical_result__invalid_pipeline__claim_evidence_wrong_owner` | `VR-CORE-050`: claim cannot directly reference evidence owned by another claim |
| `canonical_result__invalid_pipeline__technology_watch_stale_traversal` | `VR-TW-006`: stale technology-level traversal fields |
| `canonical_result__invalid_pipeline__technology_watch_strict_single_claim` | `VR-TW-008`: strict maturity needs at least two independent claims |
| `canonical_result__invalid_pipeline__youtube_quote_text_range_evidence` | `VR-YS-010`: notable quote evidence must use media timestamp locator |
| `canonical_result__invalid_pipeline__youtube_segment_evidence_out_of_range` | `VR-YS-013`: segment evidence must stay inside segment timestamp range |
| `canonical_result__invalid_pipeline__evidence_contributing_cycle` | `VR-CORE-026`: evidence contribution graph must be acyclic |
| `canonical_result__invalid_pipeline__contradicts_wrong_natural_sort` | `VR-CORE-031`: `contradicts` relation ordering follows natural sort |
| `canonical_result__invalid_pipeline__technology_watch_strict_single_source_coverage` | `VR-TW-009`: strict production maturity requires at least two source refs |
| `canonical_result__invalid_pipeline__youtube_synthesis_stale_traversal` | `VR-YS-015`: synthesis traversal fields are derived unions |
| `canonical_result__invalid_pipeline__youtube_standard_multi_video_missing_synthesis` | `VR-YS-018`: standard multi-video complete result requires synthesis unless explained |

Current runner status:

- `scripts/prompt_pack_validator` can load this manifest and compare actual
  findings against all checked-in `.expected.json` files.
- The runner currently covers the mandatory fixture baseline plus the optional
  regression fixtures listed above, not the full prose rule set in
  `validation_rules.md`.
- The runner should be invoked with an absolute `--manifest` path in sandboxed
  environments, because relative paths may resolve from an implementation-local
  temporary working directory.

### Fixture set review status

Latest reviewed fixture inventory:

| Metric | Value |
|---|---:|
| Total manifest entries | 25 |
| Required fixtures | 15 |
| Optional regression fixtures | 10 |
| Canonical result fixtures | 17 |
| Stage payload fixtures | 3 |
| Stage output fixtures | 3 |
| Prompt template fixtures | 2 |
| Missing fixture or expected paths | 0 |

Covered expected rule IDs:

```text
PROMPT-SHAPE-001
STAGE-REF-001
STAGE-VERSION-001
VR-CORE-005
VR-CORE-017
VR-CORE-022
VR-CORE-026
VR-CORE-031
VR-CORE-041
VR-CORE-050
VR-CORE-054
VR-TW-006
VR-TW-008
VR-TW-009
VR-YS-015
VR-YS-010
VR-YS-013
VR-YS-018
```

Review result:

- The fixture set is internally consistent: manifest entries, fixture files,
  expected files, fixture IDs, validation modes, and expected counts align.
- The mandatory baseline covers all required validator modes.
- Optional regression fixtures now cover representative high-risk graph rules
  across core, `technology_watch`, and `youtube_summary`.
- The fixture set is not intended to cover every implemented rule one-for-one.
  Unit tests remain the tighter layer for full rule-level behavior.

Recently promoted optional regressions:

| Rule | Fixture |
|---|---|
| `VR-CORE-026` | `canonical_result__invalid_pipeline__evidence_contributing_cycle` |
| `VR-CORE-031` | `canonical_result__invalid_pipeline__contradicts_wrong_natural_sort` |
| `VR-TW-009` | `canonical_result__invalid_pipeline__technology_watch_strict_single_source_coverage` |
| `VR-YS-015` | `canonical_result__invalid_pipeline__youtube_synthesis_stale_traversal` |
| `VR-YS-018` | `canonical_result__invalid_pipeline__youtube_standard_multi_video_missing_synthesis` |

---

## 7. Canonical Result Fixture Rules

Canonical result fixtures validate final or intermediate Prompt Pack JSON
artifacts.

Minimum valid canonical fixture requirements:

- all top-level arrays and objects required by the core contract are present;
- at least one `source_ref`, `claim`, and `evidence` for complete final results;
- `source_ref.type_data.schema_version` is present;
- `evidence.locator_data.schema_version` is present for fragment evidence;
- `claim.source_refs` is consistent with referenced evidence;
- `outputs.summary.claim_refs` is covered by section item claim refs;
- `metadata` and `quality_flags` consistency is satisfied.

Rules:

- Compact fixtures may use one source, one claim, one evidence, and one section
  item when testing schema/reference rules.
- Graph fixtures should intentionally include the minimum graph needed to
  trigger one pipeline rule.
- Pack-specific fixtures should include only the pack namespace under test.

---

## 8. Stage Payload Fixture Rules

Stage payload fixtures validate input sent into one pipeline stage.

Minimum valid `claim_extraction` payload:

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "claim_extraction",
  "run_id": "run_001",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "allowed_fragment_candidate_ids": ["fragcand_1"],
  "fragment_registry": [
    {
      "candidate_id": "fragcand_1",
      "source_ref_id": "source_ref_1",
      "fragment_type": "text_range",
      "locator_data": {
        "schema_version": "1.0",
        "char_start": 0,
        "char_end": 42,
        "snapshot_text_id": "snapshot_text_1"
      },
      "fragment_text": "The tool is running in two internal pilots.",
      "observation_summary": "The tool is in internal pilots."
    }
  ]
}
```

Rules:

- `stage_io_version` mismatch is a version-gate failure.
- Every ID in `allowed_*_ids` must exist in the matching loaded registry.
- Registry URI fixtures must include a loader stub or fixture-local registry
  file for deterministic CI.

---

## 9. Stage Output Fixture Rules

Stage output fixtures validate parsed model output before canonical assembly.

Minimum valid `claim_extraction` output:

```json
{
  "claim_candidates": [
    {
      "claim_text": "The tool is running in two internal pilots.",
      "claim_type": "factual",
      "claim_status": "extracted",
      "fragment_candidate_refs": ["fragcand_1"],
      "confidence": {
        "score": 0.82,
        "basis": "strong_direct_evidence",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      }
    }
  ],
  "unknown_candidates": [],
  "verification_task_candidates": []
}
```

Rules:

- Stage output fixtures do not include canonical `claim_id` or `evidence_id`
  unless the stage contract explicitly provides them.
- Invalid-reference fixtures should use an ID absent from the corresponding
  allowed ID array.
- Stage output fixtures that need allowed ID context may include
  `_fixture_context`. This is fixture-runner metadata, not part of the stage
  output contract.
- Partial-failure fixtures should keep one valid candidate and one invalid
  candidate in the same output.

---

## 10. Prompt Template Fixture Rules

Prompt template fixtures validate static or rendered prompt templates.

Valid prompt fixtures should contain:

- closed-world instruction;
- allowed ID arrays or a rendered allowed ID summary;
- expected output JSON shape;
- prohibition against canonical ID assignment;
- prohibition against final canonical result output.

Invalid prompt fixtures should include one clear prohibited instruction.

Example prohibited instruction:

```text
Assign a new `claim_id` to every claim candidate.
```

Expected finding:

```json
{
  "fixture_id": "prompt_template__invalid__assigns_canonical_ids",
  "validation_mode": "prompt_template",
  "expected_valid": false,
  "expected_findings": [
    {
      "rule_id": "PROMPT-SHAPE-001",
      "severity": "error",
      "layer": "schema",
      "object_path": "$",
      "message_contains": "canonical IDs",
      "object_refs": {
        "claim_refs": [],
        "evidence_refs": [],
        "source_refs": []
      }
    }
  ]
}
```

---

## 11. Fixture Authoring Rules

- Keep fixtures as small as possible while preserving the invariant under test.
- Prefer one primary failure per invalid fixture.
- Do not rely on fixture ordering.
- Do not reuse IDs across unrelated fixtures unless the fixture explicitly tests
  repeated IDs.
- Use deterministic IDs such as `claim_1`, `evidence_1`, and `source_ref_1`.
- Include `schema_version`, `stage_io_version`, and manifest versions wherever
  required by the mode.
- Expected findings should assert rule identity and path more strongly than
  human-readable message wording.
- Fixtures that require external registry files must reference fixture-local
  paths, not production object stores.

---

## 12. Parser Fixture Layer

Raw provider-response parser fixtures live outside the mandatory validator
fixture manifest:

```text
docs/prompt-packs/parser-fixtures/v1/
```

They test the step before `stage_output` validation:

```text
provider response text -> parser -> parsed stage_output JSON
```

Current checked-in parser fixtures:

| Fixture | Purpose |
|---|---|
| `parser_output__valid__fragment_candidate_mining_minimal` | Valid raw assistant content parses to one fragment candidate |
| `parser_output__invalid_parse__fragment_candidate_mining_malformed_json` | Invalid fragment candidate mining JSON yields `STAGE-PARSE-001` |
| `parser_output__invalid_schema__fragment_candidate_mining_unknown_top_level_key` | Unknown fragment candidate mining top-level key yields `STAGE-PARSE-003` |
| `parser_output__invalid_reference__disallowed_window` | Disallowed `window_id` yields `STAGE-REF-001` after parsing |
| `parser_output__valid__claim_extraction_minimal` | Valid raw assistant content parses to one claim candidate |
| `parser_output__invalid_parse__malformed_json` | Invalid JSON yields `STAGE-PARSE-001` |
| `parser_output__invalid_schema__unknown_top_level_key` | Unknown top-level key yields `STAGE-PARSE-003` |
| `parser_output__invalid_reference__disallowed_fragment_candidate` | Disallowed fragment ref yields `STAGE-REF-001` after parsing |
| `parser_output__valid__claim_linking_qualifies_minimal` | Valid raw assistant content parses to one relation candidate |
| `parser_output__invalid_parse__claim_linking_malformed_json` | Invalid claim linking JSON yields `STAGE-PARSE-001` |
| `parser_output__invalid_schema__claim_linking_unknown_top_level_key` | Unknown claim linking top-level key yields `STAGE-PARSE-003` |
| `parser_output__invalid_reference__disallowed_claim` | Disallowed claim ref yields `STAGE-REF-001` after parsing |
| `parser_output__invalid_reference__disallowed_evidence` | Disallowed evidence ref yields `STAGE-REF-001` after parsing |
| `parser_output__valid__pack_data_generation_technology_watch_minimal` | Valid Technology Watch pack data candidate parses to one namespace candidate |
| `parser_output__valid__pack_data_generation_youtube_summary_minimal` | Valid YouTube Summary pack data candidate parses to one namespace candidate |
| `parser_output__invalid_parse__pack_data_generation_malformed_json` | Invalid pack data generation JSON yields `STAGE-PARSE-001` |
| `parser_output__invalid_schema__pack_data_generation_unknown_top_level_key` | Unknown pack data generation top-level key yields `STAGE-PARSE-003` |
| `parser_output__invalid_reference__pack_data_generation_disallowed_claim` | Disallowed recursive `claim_refs` value yields `STAGE-REF-001` after parsing |
| `parser_output__invalid_reference__pack_data_generation_disallowed_evidence` | Disallowed recursive `evidence_refs` value yields `STAGE-REF-001` after parsing |
| `parser_output__invalid_reference__pack_data_generation_disallowed_source` | Disallowed recursive `source_refs` value yields `STAGE-REF-001` after parsing |
| `parser_output__valid__final_synthesis_minimal` | Valid readable outputs candidate parses to one outputs candidate |
| `parser_output__invalid_parse__final_synthesis_malformed_json` | Invalid final synthesis JSON yields `STAGE-PARSE-001` |
| `parser_output__invalid_schema__final_synthesis_unknown_top_level_key` | Unknown final synthesis top-level key yields `STAGE-PARSE-003` |
| `parser_output__invalid_reference__final_synthesis_disallowed_claim` | Disallowed recursive `claim_refs` value yields `STAGE-REF-001` after parsing |
| `parser_output__invalid_reference__final_synthesis_disallowed_evidence` | Disallowed recursive `evidence_refs` value yields `STAGE-REF-001` after parsing |
| `parser_output__invalid_reference__final_synthesis_disallowed_source` | Disallowed recursive `source_refs` value yields `STAGE-REF-001` after parsing |
| `parser_output__invalid_trace__final_synthesis_uncovered_summary_claim` | Uncovered `summary.claim_refs` value yields `STAGE-TRACE-001` after parsing |
| `parser_output__valid__retry_repair_claim_extraction_whole_stage` | Valid whole-stage claim extraction repair parses to one claim candidate |
| `parser_output__valid__retry_repair_claim_extraction_replacement` | Valid object-isolated repair parses to one replacement candidate |
| `parser_output__invalid_parse__retry_repair_malformed_json` | Invalid retry repair JSON yields `STAGE-PARSE-001` |
| `parser_output__invalid_schema__retry_repair_unknown_top_level_key` | Unknown retry repair top-level key yields `STAGE-PARSE-003` |
| `parser_output__invalid_reference__retry_repair_disallowed_fragment_candidate` | Disallowed repaired fragment candidate yields `STAGE-REF-001` |

These fixtures are not listed in `fixtures/v1/fixture_manifest.json`. They are
executed by the separate parser-fixture runner:

```text
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\fragment_candidate_mining
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\claim_extraction
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\claim_linking
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\pack_data_generation
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\final_synthesis
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\retry_repair
```

This keeps the canonical fixture baseline unchanged while still making raw
provider-response parser behavior executable.

---

## 13. Non-Goals v1

This document does not define:

- a production-grade fixture runner beyond the current reference skeleton;
- optional fixture JSON contents beyond the checked-in mandatory baseline;
- performance or load-test fixtures;
- model quality evaluation fixtures;
- production-grade provider-response parser beyond the lightweight reference
  parser-fixture runner.

Those remain implementation or future-version tasks as the reference validator
grows beyond the mandatory baseline.
