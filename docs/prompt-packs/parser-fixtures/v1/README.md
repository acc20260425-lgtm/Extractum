# Parser Fixtures v1

Status: v1 draft.

Parser fixtures test raw provider responses before they become `stage_output`
fixtures.

They are intentionally separate from:

- `fixtures/v1/stage-output/*`, which validates already parsed model output;
- `fixtures/v1/prompt-template/*`, which validates static prompt wording.

## Scope

Parser fixtures cover:

- extraction of assistant message content from provider response wrappers;
- strict JSON parsing;
- unknown top-level key rejection;
- reference checks against allowed ID arrays after parsing;
- expected parser result reporting.

They do not define canonical Prompt Pack JSON. Parsed valid content is handed to
the `stage_output` validator.

## Fixture shape

Each parser fixture is a JSON object:

```json
{
  "fixture_id": "parser_output__valid__claim_extraction_minimal",
  "parser_fixture_version": "1.0",
  "provider_family": "openai_compatible",
  "stage": "claim_extraction",
  "parser_options": {
    "allow_wrapper_text": false,
    "reject_unknown_top_level_keys": true
  },
  "allowed_fragment_candidate_ids": ["fragcand_1"],
  "raw_message_content": "{ ... raw assistant text ... }",
  "expected_parse_result": {
    "parse_status": "valid",
    "stage": "claim_extraction",
    "candidate_count": 1,
    "validation_findings": []
  },
  "expected_stage_output": {}
}
```

## Local parser rule IDs

Parser fixtures may use implementation-local rule IDs until promoted to
`validation_rules.md`:

| Rule ID | Meaning |
|---|---|
| `STAGE-PARSE-001` | Raw assistant content is not valid JSON. |
| `STAGE-PARSE-002` | Parsed JSON is not an object. |
| `STAGE-PARSE-003` | Parsed object contains unknown top-level keys. |
| `STAGE-REF-001` | Parsed object references an ID outside the allowed arrays. |
| `STAGE-TRACE-001` | Parsed object violates a stage-local traversal or coverage rule. |

## Current fixture directories

```text
parser-fixtures/v1/
  fragment_candidate_mining/
  claim_extraction/
  claim_linking/
  pack_data_generation/
  final_synthesis/
  retry_repair/
```

`fragment_candidate_mining` fixtures validate:

- strict JSON parsing;
- unknown top-level key rejection;
- `source_ref_id` and `window_id` references against allowed arrays.

`claim_extraction` fixtures validate:

- strict JSON parsing;
- unknown top-level key rejection;
- `fragment_candidate_refs` against `allowed_fragment_candidate_ids`.

`claim_linking` fixtures validate:

- strict JSON parsing;
- unknown top-level key rejection;
- `source_claim_id` and `target_claim_id` against `allowed_claim_ids`;
- `evidence_refs` against `allowed_evidence_ids`.

`pack_data_generation` fixtures validate:

- strict JSON parsing;
- unknown top-level key rejection;
- recursive `claim_refs` against `allowed_claim_ids`;
- recursive `evidence_refs` against `allowed_evidence_ids`;
- recursive `source_refs` and `source_ref_id` against `allowed_source_ref_ids`.

`final_synthesis` fixtures validate:

- strict JSON parsing;
- unknown top-level key rejection;
- recursive `claim_refs` against `allowed_claim_ids`;
- recursive `evidence_refs` against `allowed_evidence_ids`;
- recursive `source_refs` against `allowed_source_ref_ids`;
- `summary.claim_refs` coverage by `sections[].items[].claim_refs`.

`retry_repair` fixtures validate:

- strict JSON parsing of repaired provider responses;
- unknown top-level key rejection against the target stage output shape or
  `replacement_candidates`;
- whole-stage repaired output for `claim_extraction`;
- object-isolated `replacement_candidates` repair;
- `fragment_candidate_refs` against `allowed_fragment_candidate_ids`.

## Execution

These fixtures are executable through the reference parser-fixture runner, but
they are not part of the mandatory `fixture_manifest.json` baseline.

Run from `research-control-deck\scripts`:

```text
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\fragment_candidate_mining
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\claim_extraction
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\claim_linking
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\pack_data_generation
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\final_synthesis
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\retry_repair
```

The standard fixture manifest still starts at parsed `stage_output` artifacts.
