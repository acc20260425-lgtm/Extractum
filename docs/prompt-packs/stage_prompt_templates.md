# Stage Prompt Templates

Status: v1 draft.

Compatibility:

- Prompt Pack JSON Contract `schema_version: "1.0"`;
- Stage I/O Contracts `stage_io_version: "1.0"`.

This document defines stable prompt skeletons for LLM-powered prompt-pack
pipeline stages. It does not define canonical result fields. It sits between
`stage_io_contracts.md` and actual provider-specific prompts.

---

## 1. Purpose

`stage_prompt_templates.md` defines:

- what each LLM stage is allowed to see;
- which registry IDs the model may reference;
- the narrow JSON object the model must return;
- stage-specific prohibitions;
- retry/repair prompt shape;
- few-shot example rules.

The templates are intentionally provider-neutral. A runtime may wrap them with
provider-specific system/developer/user message structure, but the closed-world
ID rules and output JSON shapes remain unchanged.

---

## 2. Global Prompt Contract

Every LLM stage prompt should be built from the same five blocks:

```text
1. Role and task boundary.
2. Immutable input registry or compact registry URI summary.
3. Allowed ID arrays.
4. Output JSON schema for this stage.
5. Prohibitions and validation failure behavior.
```

Global instruction:

```text
You are operating inside a closed-world extraction pipeline.

Use only IDs present in the allowed ID arrays.
Do not invent IDs.
Do not assign canonical result IDs.
Do not create fields that are not requested by the output schema.
If evidence is insufficient, return an unknown or verification task candidate
instead of inventing support.
Return only valid JSON matching the requested output object.
```

Rules:

- Prompts may include natural-language guidance, but parser expectations must
  be expressed as JSON shape examples.
- Prompt templates must not ask the model to rebuild derived traversal fields.
- Prompt templates must not ask the model to output canonical result JSON.
- Any model output outside the requested JSON object is ignored or treated as a
  validation failure by the stage runner.

---

## 3. Shared Template Variables

Common variables:

| Variable | Meaning |
|---|---|
| `{{stage}}` | Stage name from the stage I/O envelope. |
| `{{pack_id}}` | Pack ID, for example `technology_watch`. |
| `{{pack_version}}` | Pack version. |
| `{{stage_io_version}}` | Stage I/O contract version expected by the runner. |
| `{{schema_version}}` | Compatible canonical contract version. |
| `{{stage_payload_json}}` | Full compact input payload or payload summary plus registry refs. |
| `{{allowed_id_summary}}` | Human-readable summary of allowed IDs. |
| `{{output_schema_json}}` | Minimal JSON object shape expected from the model. |

Registry variables:

| Variable | Meaning |
|---|---|
| `{{source_registry}}` | Inlined source registry or short source cards. |
| `{{fragment_registry}}` | Inlined pre-contract fragment candidates or short candidate cards. |
| `{{claim_registry}}` | Inlined claim cards with fixed `claim_id`. |
| `{{evidence_registry}}` | Inlined canonical evidence cards with fixed `evidence_id`. |
| `{{relation_registry}}` | Existing relation cards when the stage needs them. |

Rules:

- If a registry is passed by URI, the prompt should still include the allowed
  ID arrays and a compact summary of what the registry contains.
- The model is not allowed to dereference registry URIs. URI dereferencing is a
  stage-runner responsibility.
- Allowed ID arrays are authoritative even when a registry summary appears to
  mention more objects.

---

## 4. `fragment_candidate_mining`

Goal: identify significant observations inside deterministic material windows.

The model does not create canonical `evidence[]`, does not assign
`candidate_id`, and does not author `locator_data`.

### Prompt Skeleton

```text
Stage: fragment_candidate_mining

Task:
Find concise observation candidates in the provided material windows.
Each candidate must point to one existing `window_id`.

Input:
{{stage_payload_json}}

Allowed source refs:
{{allowed_source_ref_ids}}

Output:
Return only this JSON object:
{{output_schema_json}}

Rules:
- Use only `source_ref_id` and `window_id` values present in the input.
- Do not create `candidate_id`.
- Do not create or modify `locator_data`.
- Keep `fragment_text` as narrow as possible.
- `observation_summary` should describe the signal, not repeat the whole text.
- If no useful observation exists, return `"fragment_candidates": []`.
```

### Output Shape

```json
{
  "fragment_candidates": [
    {
      "source_ref_id": "source_ref_1",
      "window_id": "window_1",
      "fragment_text": "We are moving from demos to real internal pilots.",
      "observation_summary": "Speaker describes a shift from demos to pilots.",
      "candidate_type": "trend_signal",
      "salience": "high"
    }
  ]
}
```

### Few-Shot Rule

Input window:

```text
window_id: window_1
text: "We tried this as a demo last year. This quarter it is running in two internal pilots."
```

Expected candidate:

```json
{
  "fragment_candidates": [
    {
      "source_ref_id": "source_ref_1",
      "window_id": "window_1",
      "fragment_text": "This quarter it is running in two internal pilots.",
      "observation_summary": "The item has moved from demo to internal pilots.",
      "candidate_type": "adoption_signal",
      "salience": "high"
    }
  ]
}
```

The model should prefer the narrow sentence over the full surrounding paragraph.

---

## 5. `claim_extraction`

Goal: synthesize claim candidates from pre-contract fragment candidates.

The model uses `fragment_candidate_refs`, not final `evidence_refs`.
Canonical `claim_id` and `evidence_id` values are assigned later by pipeline
code.

### Prompt Skeleton

```text
Stage: claim_extraction

Task:
Create claim candidates supported by the provided fragment candidates.

Input fragment registry:
{{fragment_registry}}

Allowed fragment candidate IDs:
{{allowed_fragment_candidate_ids}}

Output:
Return only this JSON object:
{{output_schema_json}}

Rules:
- Use only IDs from `allowed_fragment_candidate_ids`.
- Do not create `claim_id`.
- Do not create `evidence_refs`.
- Each claim must cite at least one `fragment_candidate_refs` entry unless it is
  explicitly returned as an unknown candidate.
- If support is weak or missing, return an `unknown_candidates` entry instead
  of inventing a claim.
- Keep `claim_text` atomic: one checkable assertion per claim.
```

### Output Shape

```json
{
  "claim_candidates": [
    {
      "claim_text": "Local LLM agents are moving from demos to internal pilots.",
      "claim_type": "factual",
      "claim_status": "extracted",
      "fragment_candidate_refs": ["fragcand_1"],
      "confidence": {
        "score": 0.78,
        "basis": "strong_direct_evidence",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      }
    }
  ],
  "unknown_candidates": [],
  "verification_task_candidates": [],
  "warnings": []
}
```

### Few-Shot Rule

Candidate card:

```text
fragcand_1: "This quarter it is running in two internal pilots."
summary: The item has moved from demo to internal pilots.
```

Expected claim:

```json
{
  "claim_candidates": [
    {
      "claim_text": "The item is running in two internal pilots this quarter.",
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
  "verification_task_candidates": [],
  "warnings": []
}
```

Counterexample:

```text
Do not produce: "The item is production-ready."
Reason: the fragment only supports internal pilots, not production readiness.
```

---

## 6. `claim_linking`

Goal: create relation candidates between already fixed claims.

The model does not assign `relation_id`. The pipeline normalizes symmetric
relations such as `contradicts`.

### Prompt Skeleton

```text
Stage: claim_linking

Task:
Find meaningful relations between the provided claims.

Claim registry:
{{claim_registry}}

Evidence registry:
{{evidence_registry}}

Allowed claim IDs:
{{allowed_claim_ids}}

Allowed evidence IDs:
{{allowed_evidence_ids}}

Output:
Return only this JSON object:
{{output_schema_json}}

Rules:
- Use only IDs from `allowed_claim_ids` and `allowed_evidence_ids`.
- Do not create `relation_id`.
- `evidence_refs` must refer only to evidence belonging to one of the two
  related claims.
- Use `description` for `contradicts` and `qualifies`.
- Do not duplicate an existing relation if it is present in the input.
```

### Output Shape

```json
{
  "relation_candidates": [
    {
      "relation_type": "qualifies",
      "custom_relation_type": null,
      "source_claim_id": "claim_2",
      "target_claim_id": "claim_1",
      "description": "The second claim narrows when the first claim applies.",
      "evidence_refs": ["evidence_2"],
      "confidence": {
        "score": 0.74,
        "basis": "inference_chain",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      }
    }
  ]
}
```

### Few-Shot Rule

Claims:

```text
claim_1: "The tool is used in production by several teams."
claim_2: "The same tool is limited to an internal pilot in the cited source."
```

Expected relation candidate:

```json
{
  "relation_candidates": [
    {
      "relation_type": "contradicts",
      "custom_relation_type": null,
      "source_claim_id": "claim_1",
      "target_claim_id": "claim_2",
      "description": "The claims disagree on whether usage is production or only an internal pilot.",
      "evidence_refs": ["evidence_1", "evidence_2"],
      "confidence": {
        "score": 0.8,
        "basis": "conflicting_sources",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      }
    }
  ]
}
```

The pipeline may reorder `source_claim_id` and `target_claim_id` for
`contradicts` according to the canonical natural-sort convention.

---

## 7. `pack_data_generation`

Goal: create pack-specific candidate structures over the fixed claim/evidence
graph.

The model drafts semantic pack objects. The pipeline assigns pack object IDs
when the pack spec requires them and rebuilds traversal refs.

### Prompt Skeleton

```text
Stage: pack_data_generation

Task:
Create pack-specific structured candidates using only the fixed graph objects.

Pack:
{{pack_id}} {{pack_version}}

Claim registry:
{{claim_registry}}

Evidence registry:
{{evidence_registry}}

Allowed claim IDs:
{{allowed_claim_ids}}

Allowed evidence IDs:
{{allowed_evidence_ids}}

Output:
Return only this JSON object:
{{output_schema_json}}

Rules:
- Use only allowed claim and evidence IDs.
- Do not invent source refs.
- Do not generate derived traversal fields as authoritative data.
- Follow the pack-specific spec for object names and required fields.
- If the pack-specific conclusion is unsupported, return a weaker object or an
  unknown candidate instead.
```

### Technology Watch Output Shape

```json
{
  "pack_data_candidate": {
    "technology_watch": {
      "technologies": [
        {
          "name": "Local LLM agents",
          "normalized_name": "local_llm_agents",
          "maturity": {
            "level": "pilot",
            "rationale": "The cited evidence describes internal pilots.",
            "claim_refs": ["claim_1"]
          },
          "signals": [],
          "tools": [],
          "adoption_barriers": [],
          "risks": [],
          "recommendations": [],
          "claim_refs": ["claim_1"],
          "evidence_refs": ["evidence_1"],
          "source_refs": ["source_ref_1"]
        }
      ]
    }
  },
  "unknown_candidates": [],
  "warning_candidates": []
}
```

### YouTube Summary Output Shape

```json
{
  "pack_data_candidate": {
    "youtube_summary": {
      "videos": [
        {
          "source_ref_id": "source_ref_1",
          "segments": [],
          "key_points": [
            {
              "text": "The speaker says the work has moved into internal pilots.",
              "claim_refs": ["claim_1"],
              "evidence_refs": ["evidence_1"]
            }
          ],
          "notable_quotes": [],
          "action_items": [],
          "open_questions": [],
          "claim_refs": ["claim_1"],
          "evidence_refs": ["evidence_1"],
          "source_refs": ["source_ref_1"]
        }
      ],
      "synthesis": null
    }
  },
  "unknown_candidates": [],
  "warning_candidates": []
}
```

Rules:

- Pack-specific examples are illustrative. The authoritative object schemas
  remain in the corresponding pack specs.
- If a pack object includes traversal refs, the pipeline may recompute and
  overwrite them during assembly.

---

## 8. `final_synthesis`

Goal: draft readable `outputs.summary` and `outputs.sections` over the fixed
canonical graph and pack-specific candidate data.

The model does not alter claims, evidence, relations, source refs, metadata, or
audit refs.

### Prompt Skeleton

```text
Stage: final_synthesis

Task:
Draft readable outputs over the fixed graph and pack data.

Canonical graph summary:
{{canonical_graph_summary}}

Pack data candidate:
{{pack_data_candidate}}

Allowed claim IDs:
{{allowed_claim_ids}}

Allowed evidence IDs:
{{allowed_evidence_ids}}

Allowed source ref IDs:
{{allowed_source_ref_ids}}

Output:
Return only this JSON object:
{{output_schema_json}}

Rules:
- Use only allowed IDs.
- Do not create or rewrite claims.
- `summary.claim_refs` must be covered by section item claim refs.
- Every section item that makes a substantive statement should include
  `claim_refs`.
- Do not create `metadata`, `quality_flags`, `warnings`, `limitations`, or
  `audit_refs`; the pipeline assembles those fields.
```

### Output Shape

```json
{
  "outputs_candidate": {
    "summary": {
      "summary_text": "Local LLM agents appear to be moving from demos into internal pilots.",
      "claim_refs": ["claim_1"],
      "evidence_refs": ["evidence_1"],
      "source_refs": ["source_ref_1"]
    },
    "sections": [
      {
        "section_type": "trends",
        "custom_section_type": null,
        "title": "Trend",
        "items": [
          {
            "text": "The cited video describes internal pilots rather than only demos.",
            "claim_refs": ["claim_1"],
            "evidence_refs": ["evidence_1"],
            "source_refs": ["source_ref_1"]
          }
        ]
      }
    ]
  }
}
```

---

## 9. Retry / Repair Prompt

Retry prompts should be compact and local. They should not resend the full raw
corpus unless the validation error proves that the model lacked necessary
context.

### Prompt Skeleton

```text
Stage retry: retry_repair
Repair target stage: {{repair_target_stage}}
Repair scope: {{repair_scope}}

The previous output failed validation.

Validation findings:
{{validation_findings}}

Allowed IDs remain unchanged:
{{allowed_id_summary}}

Previous output summary:
{{previous_output_summary}}

Retry payload:
{{retry_repair_payload_json}}

Task:
Return a corrected JSON object for the target stage or replacement candidates
for the invalid objects.

Rules:
- Do not add IDs outside the allowed arrays.
- Fix only the invalid candidate objects or invalid fields.
- Preserve valid candidate objects when possible.
- If `repair_scope = "object"`, return only `replacement_candidates`.
- If `repair_scope = "whole_stage"`, return the exact output shape of
  `{{repair_target_stage}}`.
- Do not mix `replacement_candidates` with whole-stage output keys.
- Do not add unknown top-level keys.
- Return only valid JSON.
```

### Repair Output Shape

Whole-stage repair uses the same shape as the target stage output.

For object-isolated repair, the runner may ask only for replacement objects:

```json
{
  "replacement_candidates": [
    {
      "replacement_for_path": "claim_candidates[0]",
      "claim_text": "The item is running in two internal pilots this quarter.",
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
  ]
}
```

Rules:

- Object-isolated repair is preferred when the failed object can be isolated.
- Whole-stage repair is reserved for malformed JSON, schema-wide failure, or
  instruction drift.
- Retry repair is itself a stage (`retry_repair`), but its output is validated
  against either `replacement_candidates` or the requested target-stage shape.

---

## 10. Few-Shot Example Rules

Few-shot examples should be small and adversarially useful.

Rules:

- Include one positive example for the desired output shape.
- Include one counterexample for common overreach, such as converting weak
  evidence into a strong claim.
- Use only IDs that appear in the example input.
- Keep examples shorter than production payloads.
- Do not include examples that violate current validation rules.
- Do not reuse stale IDs from unrelated stages.
- Prefer examples that show refusal or unknown generation when support is
  insufficient.

Counterexample pattern:

```text
Bad output:
The model creates `evidence_99`.

Why invalid:
`evidence_99` is not present in `allowed_evidence_ids`.

Correct behavior:
Use an allowed evidence ID, return an unknown candidate, or omit the unsupported
object.
```

---

## 11. Parser Handoff

Every prompt template should have a matching parser contract.

Parser responsibilities:

- strip provider wrapper text if the provider returns it;
- parse the first valid JSON object only if the runner explicitly allows this;
- reject payloads with unknown top-level keys unless the stage parser allows
  extensions;
- validate references against allowed ID arrays before pipeline assembly;
- keep raw invalid outputs in quarantine artifacts, not in canonical JSON.

Recommended parser result shape:

```json
{
  "parse_status": "valid",
  "stage": "claim_extraction",
  "candidate_count": 1,
  "validation_findings": []
}
```

If parsing fails:

```json
{
  "parse_status": "invalid",
  "stage": "claim_extraction",
  "candidate_count": 0,
  "validation_findings": [
    {
      "rule_id": "STAGE-PARSE-001",
      "severity": "error",
      "layer": "schema",
      "object_path": "$",
      "message": "Model output is not valid JSON.",
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

## 12. Implementation Notes

- Provider-specific prompt files may be generated from this document, but must
  preserve the stage output shapes and closed-world ID rules.
- Checked-in provider-specific renders currently live at
  `prompts/v1/openai-compatible/`:
  - `fragment_candidate_mining.prompt.json`
  - `claim_extraction.prompt.json`
  - `claim_linking.prompt.json`
  - `pack_data_generation.prompt.json`
  - `final_synthesis.prompt.json`
  - `retry_repair.prompt.json`
- Template variables should be filled by code, not by string concatenation from
  untrusted content.
- For large registries, prompt builders should include compact cards and rely
  on the stage runner to load external registry URIs.
- Template tests should include JSON parsing, unknown-key handling, forbidden ID
  injection, and object-isolated retry.
- Raw provider-response parser fixtures live under `parser-fixtures/v1/` and
  validate behavior before parsed `stage_output` fixtures are produced.
- Prompt template changes that alter required output fields should bump the
  template version used by the stage runner.
