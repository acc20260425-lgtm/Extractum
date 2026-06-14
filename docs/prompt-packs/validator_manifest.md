# Validator Manifest

Status: v1 draft.

Compatibility:

- Prompt Pack JSON Contract `schema_version: "1.0"`;
- Stage I/O Contracts `stage_io_version: "1.0"`;
- Validation Rules `validation_rules.md` v1 draft.

This document defines the execution manifest for a first reference validator.
It does not add validation semantics that are absent from the contract docs. It
binds existing rules to execution phases, required inputs, blocking behavior,
and CI fixture expectations.

---

## 1. Purpose

`validator_manifest.md` answers implementation questions that
`validation_rules.md` intentionally leaves open:

- which rule groups run in which order;
- which input artifacts are required for each validation mode;
- which failures are blocking;
- how canonical result validation differs from stage payload validation;
- how findings are emitted;
- how partial results, repair, and quarantine interact with validation;
- what fixture coverage is expected in CI.

The manifest is the bridge between prose rules and executable validator code.

---

## 2. Manifest Header

Reference validator implementations should declare the manifest they support.

```json
{
  "validator_manifest_version": "1.0",
  "compatible_schema_version": "1.0",
  "compatible_stage_io_version": "1.0",
  "rule_sources": [
    "prompt_pack_json_contract_v1_draft.md",
    "source_type_schemas.md",
    "fragment_locator_schemas.md",
    "validation_rules.md",
    "execution_model_graph_assembly_policy.md",
    "stage_io_contracts.md",
    "stage_prompt_templates.md",
    "prompts/v1/README.md",
    "parser-fixtures/v1/README.md",
    "schemas/README.md",
    "technology_watch_pack_spec.md",
    "youtube_summary_pack_spec.md"
  ],
  "finding_format": "validation_rules.md#validation-finding"
}
```

Rules:

- `validator_manifest_version` versions this manifest, not the canonical result.
- `compatible_schema_version` is the canonical result schema version supported
  by the validator.
- `compatible_stage_io_version` is required only for stage payload and prompt
  output validation.
- A validator must reject, skip, or route to compatibility handling any artifact
  whose version is not supported by the active manifest.

---

## 3. Validation Modes

The v1 reference validator supports four modes.

| Mode | Target artifact | Primary purpose |
|---|---|---|
| `canonical_result` | Final or intermediate Prompt Pack JSON result | Validate contract, graph, pack data, and QA rules |
| `stage_payload` | Input payload for one pipeline stage | Validate stage envelope, registries, allowed ID boundaries |
| `stage_output` | Raw parsed output from one LLM stage | Validate narrow candidate JSON before assembly |
| `prompt_template` | Static prompt template or generated prompt fixture | Validate template variables, prohibited instructions, output-shape alignment |

Parser fixtures under `parser-fixtures/v1/` are executable through a separate
parser-fixture runner, but they are not a fifth mandatory manifest validation
mode in v1. They test raw provider-response parsing before the parsed object is
handed to `stage_output` validation.

Mode request shape:

```json
{
  "validation_mode": "canonical_result",
  "validator_manifest_version": "1.0",
  "schema_version": "1.0",
  "stage_io_version": null,
  "pack_id": "technology_watch",
  "artifact_ref": {
    "artifact_id": "result_001",
    "artifact_uri": "artifact://runs/run_001/results/result_001.json"
  }
}
```

Rules:

- `schema_version` is required for `canonical_result`.
- `stage_io_version` is required for `stage_payload` and `stage_output`.
- `pack_id` is required when pack-specific rules are enabled.
- `artifact_uri` is an implementation locator and must not be copied into
  canonical result JSON unless referenced through audit/quarantine artifacts.

---

## 4. Required Input Artifacts

### `canonical_result`

Required:

- canonical result JSON;
- selected pack spec for `pack_id`;
- companion schemas referenced by `source_refs[].source_type` and
  `evidence[].fragment_type`;
- `validation_rules.md`.

Optional:

- run-level result graph for `parent_result_ids` DAG validation;
- external audit/quarantine store access for diagnostic enrichment;
- previous stage artifacts for debugging only.

### `stage_payload`

Required:

- stage payload JSON;
- `stage_io_contracts.md`;
- expected `stage_io_version`;
- registry contents or verified `registry_refs`;
- allowed ID arrays.

### `stage_output`

Required:

- parsed model output JSON;
- stage name;
- expected stage output shape from `stage_io_contracts.md`;
- allowed ID arrays from the corresponding stage payload.

Optional:

- original raw model output for quarantine;
- previous output summary for retry/repair diagnostics.

### `prompt_template`

Required:

- prompt template text;
- expected stage name;
- expected output shape from `stage_prompt_templates.md`.

Optional:

- rendered fixture with sample registries;
- parser fixture for the expected provider response format.

### `parser_fixture`

Parser fixtures are optional implementation inputs for parser runners.

Required when used:

- raw provider response content;
- stage name;
- parser options such as wrapper-text handling and unknown-key policy;
- allowed ID arrays from the matching stage payload;
- expected parse result.

The mandatory fixture manifest starts after parsing, at `stage_output`. The
separate parser-fixture runner validates raw provider responses.

---

## 5. Execution Phases

The reference validator runs phases in this order.

```json
{
  "phases": [
    {
      "phase": "version_gate",
      "blocking": true,
      "applies_to": ["canonical_result", "stage_payload", "stage_output", "prompt_template"]
    },
    {
      "phase": "local_schema",
      "blocking": true,
      "applies_to": ["canonical_result", "stage_payload", "stage_output"]
    },
    {
      "phase": "reference_integrity",
      "blocking": true,
      "applies_to": ["canonical_result", "stage_payload", "stage_output"]
    },
    {
      "phase": "companion_schema",
      "blocking": true,
      "applies_to": ["canonical_result"]
    },
    {
      "phase": "pipeline_graph",
      "blocking": true,
      "applies_to": ["canonical_result"]
    },
    {
      "phase": "pack_specific",
      "blocking": true,
      "applies_to": ["canonical_result", "stage_output"]
    },
    {
      "phase": "prompt_template_static",
      "blocking": true,
      "applies_to": ["prompt_template"]
    },
    {
      "phase": "qa",
      "blocking": false,
      "applies_to": ["canonical_result", "stage_payload", "stage_output", "prompt_template"]
    }
  ]
}
```

Rules:

- A blocking phase with `error` findings marks the artifact invalid.
- Non-blocking `warning` and `info` findings do not invalidate the artifact.
- If `local_schema` fails, graph phases may be skipped because object traversal
  may be unreliable.
- `qa` always runs when the artifact is parseable enough to inspect.

---

## 6. Rule Group Map

| Group | Source | Applies to | Layer | Blocking |
|---|---|---|---|---|
| `core_schema` | `VR-CORE-*` with `layer = schema` | `canonical_result` | `schema` | yes |
| `core_reference` | `VR-CORE-*` with `layer = reference` | `canonical_result` | `reference` | yes |
| `core_pipeline` | `VR-CORE-*` with `layer = pipeline` | `canonical_result` | `pipeline` | yes |
| `core_qa` | `VR-CORE-*` with `layer = qa` | `canonical_result` | `qa` | no |
| `source_type_schema` | `VR-ST-*` | `canonical_result` | schema/reference/pipeline | yes for `error` |
| `fragment_locator_schema` | `VR-FL-*` | `canonical_result` | schema/reference/pipeline | yes for `error` |
| `technology_watch_pack` | `VR-TW-*` | `canonical_result`, `stage_output` | schema/pipeline/qa | yes for `error` |
| `youtube_summary_pack` | `VR-YS-*` | `canonical_result`, `stage_output` | schema/pipeline/qa | yes for `error` |
| `stage_io_shape` | `stage_io_contracts.md` | `stage_payload`, `stage_output` | schema/reference | yes |
| `prompt_template_shape` | `stage_prompt_templates.md` | `prompt_template` | schema/qa | yes for schema |
| `execution_policy` | `execution_model_graph_assembly_policy.md` | all modes | pipeline/qa | yes for hard policy |

Rules:

- Rule IDs from `validation_rules.md` remain stable identifiers.
- Stage I/O and prompt-template checks may use implementation-local rule IDs
  such as `STAGE-REF-001` or `PROMPT-SHAPE-001` until they are promoted into
  `validation_rules.md`.
- Implementation-local rule IDs must still emit the same finding format.

---

## 7. Finding Output

All modes emit the v1 finding shape from `validation_rules.md`.

```json
{
  "rule_id": "VR-CORE-022",
  "severity": "error",
  "layer": "reference",
  "object_path": "evidence[0].claim_id",
  "message": "evidence.claim_id references a missing claim.",
  "object_refs": {
    "claim_refs": ["claim_missing"],
    "evidence_refs": ["evidence_1"],
    "source_refs": []
  }
}
```

Additional implementation fields are allowed if they do not change these base
semantics. Recommended optional fields:

```json
{
  "stage": "claim_extraction",
  "validation_mode": "stage_output",
  "artifact_id": "stage_output_001",
  "repairable": true,
  "quarantine_recommended": false
}
```

Rules:

- `severity`, `layer`, and `rule_id` must match the source rule when the
  finding maps to `validation_rules.md`.
- `object_refs` arrays may be empty, but the `object_refs` object is present.
- Repair/quarantine hints are advisory. The execution policy remains
  authoritative for retry and quarantine behavior.

---

## 8. Partial Result and Quarantine Behavior

Validator output should distinguish artifact-level failure from object-level
failure.

| Situation | Validator behavior | Pipeline behavior |
|---|---|---|
| Malformed JSON | One artifact-level `error` finding | Whole-stage retry or quarantine |
| Missing required top-level object | Artifact-level `error` finding | Whole-stage retry or fail |
| One invalid candidate in otherwise valid stage output | Object-level `error` finding | Preserve valid candidates, repair/quarantine invalid object |
| Dangling canonical reference | Object-level or graph-level `error` finding | Retry/heal only if policy allows; otherwise quarantine/fail |
| QA warning | `warning` finding | Keep artifact valid; route to review if configured |

Rules:

- The validator must not silently delete invalid objects.
- The validator may mark an object as `repairable`, but repair is performed by
  the stage runner or graph assembly pipeline.
- If object-level isolation is unsafe, the validator should emit an
  artifact-level finding explaining why.

---

## 9. CI Fixture Expectations

Reference validator CI should include fixtures for each validation mode.
The concrete fixture catalog is defined in `validator_fixtures.md`.

Required fixture classes:

| Fixture class | Minimum examples |
|---|---|
| `canonical_result_valid` | One valid `technology_watch`, one valid `youtube_summary` |
| `canonical_result_invalid_schema` | Missing required top-level field |
| `canonical_result_invalid_reference` | Dangling `claim_id` or `evidence_id` |
| `canonical_result_invalid_pipeline` | Broken derived traversal field or cycle |
| `canonical_result_qa_warning` | Valid artifact that should emit a QA warning |
| `stage_payload_valid` | One valid `claim_extraction` payload |
| `stage_payload_invalid_registry` | `allowed_*_ids` references object absent from loaded registry |
| `stage_payload_invalid_version` | Unsupported `stage_io_version` |
| `stage_output_valid` | One valid `claim_extraction` LLM output |
| `stage_output_invalid_reference` | Candidate references disallowed ID |
| `stage_output_partial_failure` | One invalid candidate with valid sibling candidates |
| `prompt_template_valid` | One rendered prompt matching `stage_prompt_templates.md` |
| `prompt_template_invalid` | Prompt asks model to assign canonical IDs |
| `parser_fixture_valid` | Optional: raw provider responses that parse to valid stage outputs |
| `parser_fixture_invalid` | Optional: malformed JSON, unknown top-level key, or disallowed reference |

Recommended CI assertions:

- every invalid fixture emits at least one expected `rule_id`;
- every valid fixture emits no `error` findings;
- QA-only fixtures emit warnings without invalidating the artifact;
- finding JSON itself validates against the finding shape;
- object-isolated failure preserves valid sibling candidates in the runner test.

---

## 10. First Reference Validator Scope

The first implementation should cover:

- machine-readable JSON Schema local-shape validation for `core/result`,
  selected source `type_data`, selected fragment `locator_data`, selected
  pack-specific `pack_data`, `stage_payload` inputs, and normalized
  `stage_output` payloads;
- JSON parse and top-level object shape;
- version gate for `schema_version` and `stage_io_version`;
- ID uniqueness and referential integrity;
- `source_ref.type_data.schema_version`;
- `evidence.locator_data.schema_version`;
- derived traversal ref checks for claims, technologies, videos, and synthesis;
- `claim_relations` evidence ownership and `contradicts` ordering;
- `contributing_evidence_refs` acyclicity;
- `parent_result_ids` DAG check when run-level graph is supplied;
- pack-specific hard rules for `technology_watch` and `youtube_summary`;
- stage payload allowed-ID boundary checks;
- stage output candidate reference checks;
- prompt template checks for prohibited canonical ID assignment.

The first implementation may defer:

- semantic prose classification such as whether a text is a limitation or
  unknown;
- source credibility scoring;
- confidence calibration;
- full JSON Schema validation for parser fixtures beyond the current lightweight
  parser-fixture runner.

---

## 11. Machine-Readable Schema Placement

Machine-readable JSON Schemas live under:

```text
docs/prompt-packs/schemas/v1/
```

The placement contract is documented in `schemas/README.md`.

The reference validator should load schemas through
`schemas/v1/schema_manifest.json`.

Loading order:

1. core schemas;
2. companion schemas selected by actual `source_type` and `fragment_type`;
3. stage I/O schemas for `stage_payload` and normalized `stage_output`
   artifacts;
4. pack-specific schemas selected by `pack_id`.

For `stage_output`, the reference skeleton validates the normalized stage
artifact shape: standard stage envelope fields plus the stage-specific output
fields. Raw provider responses remain covered by the parser-fixture runner,
not by `validate_artifact("stage_output", ...)`.

Rules:

- Missing schema files are validator setup errors, not validation findings
  against an artifact.
- JSON Schema covers local shape and simple references only; graph and
  pipeline-level rules remain implemented as code.
- The checked-in v1 schemas are an incremental bundle. Entries marked
  `semantic` in `schema_manifest.json` have reviewed local-shape coverage;
  entries without that status remain loader skeletons.

---

## 12. Validator Implementation Affordances

The manifest is designed to support generated validator skeletons and CLI
tooling without mandating a specific language.

Recommended CLI shape:

```text
prompt-pack-validator validate --mode canonical_result --file result.json
prompt-pack-validator validate --mode stage_output --stage claim_extraction --file output.json
prompt-pack-validator validate-parser-fixtures --fixtures-dir parser-fixtures/v1/fragment_candidate_mining
prompt-pack-validator validate-parser-fixtures --fixtures-dir parser-fixtures/v1/claim_extraction
prompt-pack-validator validate-parser-fixtures --fixtures-dir parser-fixtures/v1/claim_linking
prompt-pack-validator validate-parser-fixtures --fixtures-dir parser-fixtures/v1/pack_data_generation
prompt-pack-validator validate-parser-fixtures --fixtures-dir parser-fixtures/v1/final_synthesis
prompt-pack-validator validate-parser-fixtures --fixtures-dir parser-fixtures/v1/retry_repair
prompt-pack-validator explain VR-CORE-055
prompt-pack-validator list-rules --group core_pipeline
```

Current reference skeleton:

```text
scripts/prompt_pack_validator/
scripts/prompt_pack_validator_tests.py
```

Current fixture runner command from `research-control-deck\scripts`:

```text
python -m prompt_pack_validator validate-fixtures --manifest F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\fixtures\v1\fixture_manifest.json
```

Current parser-fixture runner command from `research-control-deck\scripts`:

```text
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\fragment_candidate_mining
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\claim_extraction
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\claim_linking
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\pack_data_generation
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\final_synthesis
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\retry_repair
```

Implemented in the current skeleton:

| Area | Implemented checks |
|---|---|
| Fixture runner | Loads `fixture_manifest.json`, fixture files, `.expected.json`, compares rule IDs, severities, paths and expected counts |
| JSON Schema loader | `SCHEMA-VALIDATION-001` for `core/result`, selected source-type `type_data`, selected fragment-locator `locator_data`, selected pack-specific `pack_data`, `stage_payload` input schemas, and normalized `stage_output` schemas loaded through `schemas/v1/schema_manifest.json` |
| Core schema/reference | `VR-CORE-004`, `VR-CORE-005`, `VR-CORE-006`, `VR-CORE-011`, `VR-CORE-012`, `VR-CORE-013`, `VR-CORE-015`, `VR-CORE-016`, `VR-CORE-020`, `VR-CORE-021`, `VR-CORE-022`, `VR-CORE-023`, `VR-CORE-024`, `VR-CORE-025`, `VR-CORE-028`, `VR-CORE-029`, `VR-CORE-030`, `VR-CORE-033`, `VR-CORE-051` |
| Core graph/pipeline | `VR-CORE-007`, `VR-CORE-008`, `VR-CORE-009`, `VR-CORE-017`, `VR-CORE-018`, `VR-CORE-026`, `VR-CORE-031`, `VR-CORE-032`, `VR-CORE-035`, `VR-CORE-037`, `VR-CORE-050`, `VR-CORE-054` |
| Metadata/QA | `VR-CORE-036`, `VR-CORE-038`, `VR-CORE-039`, `VR-CORE-040`, `VR-CORE-041`, `VR-CORE-042`, `VR-CORE-043`, `VR-CORE-044`, `VR-CORE-045`; opt-in info mode: `VR-CORE-046`, `VR-CORE-052` |
| Companion presence | `VR-ST-001`, `VR-FL-002` |
| Technology Watch pack | `VR-TW-001`, `VR-TW-002`, `VR-TW-003`, `VR-TW-004`, `VR-TW-005`, `VR-TW-006`, `VR-TW-007`, `VR-TW-008`, `VR-TW-009`, `VR-TW-010`, `VR-TW-011`, `VR-TW-012`, `VR-TW-013` |
| YouTube Summary pack | `VR-YS-001`, `VR-YS-002`, `VR-YS-003`, `VR-YS-004`, `VR-YS-005`, `VR-YS-006`, `VR-YS-007`, `VR-YS-008`, `VR-YS-009`, `VR-YS-010`, `VR-YS-011`, `VR-YS-012`, `VR-YS-013`, `VR-YS-014`, `VR-YS-015`, `VR-YS-016`, `VR-YS-017`, `VR-YS-018`, `VR-YS-019`, `VR-YS-020`, `VR-YS-021` |
| Stage and prompt checks | `STAGE-REF-001`, `STAGE-TRACE-001`, `STAGE-VERSION-001`, `PROMPT-SHAPE-001` |
| Parser fixture runner | `STAGE-PARSE-001`, `STAGE-PARSE-002`, `STAGE-PARSE-003`, `STAGE-REF-001`, `STAGE-TRACE-001` for raw provider-response parser fixtures, including retry/repair outputs |

The current skeleton covers the mandatory fixture baseline plus selected
optional regression fixtures, including machine-readable JSON Schema local-shape
checks where the schema/fixture boundary is normalized. It is useful for
implementation feedback and regression checks, but the prose contract and rule
documents remain authoritative.

Recommended generated skeleton concepts:

- one phase runner per execution phase;
- one rule function or method per promoted `rule_id`;
- one schema loader keyed by `schema_manifest.json`;
- one finding formatter shared across all modes;
- one `explain` registry generated from `validation_rules.md`.

Rules:

- CLI output is an implementation concern, but machine-readable JSON finding
  output must preserve the base finding shape from `validation_rules.md`.
- `explain <rule_id>` should show the rule text, severity, layer, scope, source
  document, and remediation hint when available.
- Generated code is a convenience layer. The docs remain the source of truth
  until code and docs are versioned together.

---

## 13. Non-Goals v1

The v1 manifest does not define:

- a production packaging format or distribution channel for the validator;
- a storage backend for fixtures, registries, audit logs, or quarantine
  artifacts;
- automatic correction of invalid artifacts;
- human-review workflow states;
- model quality evaluation metrics beyond structural validation.

These are implementation or future-version concerns. The manifest defines the
minimum executable boundary needed to start a reference validator without
guessing rule order or input requirements.
