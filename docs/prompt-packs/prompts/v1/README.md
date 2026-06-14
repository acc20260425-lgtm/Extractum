# Provider Prompt Files v1

Status: v1 draft.

This directory contains provider-specific prompt renderings derived from
`stage_prompt_templates.md`.

These files are implementation aids, not new contract sources. The authoritative
stage output shapes remain:

- `stage_prompt_templates.md`;
- `stage_io_contracts.md`.

## Directory layout

```text
prompts/v1/
  openai-compatible/
    fragment_candidate_mining.prompt.json
    claim_extraction.prompt.json
    claim_linking.prompt.json
    pack_data_generation.prompt.json
    final_synthesis.prompt.json
    retry_repair.prompt.json
```

Provider-specific prompt files should preserve:

- closed-world ID boundaries;
- narrow stage output JSON;
- the ban on canonical ID assignment by the model;
- the ban on final canonical result JSON output;
- unknown candidates as the escape hatch for unsupported claims.

## Versioning

Prompt files include `template_version`, `schema_version`, and
`stage_io_version`.

Changing wording without changing output shape does not require a canonical
schema bump. Changing required output keys or parser expectations should bump
`template_version` and may require a `stage_io_version` bump.

## Parser fixtures

Raw provider-response parser fixtures live in:

```text
docs/prompt-packs/parser-fixtures/v1/
```

Parser fixtures are separate from validator fixtures. Validator fixtures test
already parsed `stage_output` artifacts; parser fixtures test extraction from
provider response text into those artifacts.
