# Stage I/O Contracts

Status: v1 draft.

Compatibility: Prompt Pack JSON Contract `schema_version: "1.0"`.

This document defines stage-level input and output payloads for prompt-pack
pipelines. These payloads are internal execution contracts. They are not the
canonical final result schema.

---

## 1. Purpose

`stage_io_contracts.md` bridges:

- the canonical result contract;
- `execution_model_graph_assembly_policy.md`;
- prompt templates and parser implementations;
- reference validator fixtures.

The goal is to make every LLM stage operate on bounded, immutable registries and
return narrow candidate objects. Final IDs, derived traversal refs, metadata,
audit, and graph healing remain pipeline-owned.

---

## 2. Common Stage Envelope

Every stage payload should include a small execution envelope.

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "claim_extraction",
  "run_id": "run_001",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "input_result_ids": ["result_source_analysis_1"],
  "audit_refs": ["audit_stage_input_1"]
}
```

Rules:

- `stage_io_version` versions this internal stage contract, not the canonical
  result contract.
- The stage runner must reject or route to compatibility handling any payload
  whose `stage_io_version` does not match the version supported by that runner.
- `schema_version` is the compatible Prompt Pack JSON Contract version.
- `stage` uses the shared stage namespace from the core contract or
  `{pack_id}/{stage_name}`.
- `input_result_ids` references immutable upstream stage results when present.
- LLM stages must not alter the stage envelope.

---

## 3. Common Registry Rules

Structural LLM stages receive registries and allowed ID arrays.

```json
{
  "allowed_source_ref_ids": ["source_ref_1"],
  "allowed_fragment_candidate_ids": ["fragcand_1", "fragcand_2"],
  "allowed_claim_ids": ["claim_1", "claim_2"],
  "allowed_evidence_ids": ["evidence_1", "evidence_2"]
}
```

Rules:

- LLM may only reference IDs present in the matching `allowed_*_ids` array.
- New IDs invented by LLM are invalid.
- Registry objects are immutable from the LLM point of view.
- Pipeline code assigns final canonical IDs.
- Derived traversal fields are rebuilt by pipeline code, not trusted from LLM
  output.
- Large registries may be passed by reference instead of inlined. The allowed
  ID arrays remain authoritative in the stage payload; external registry URIs
  are implementation locators, not canonical result fields.

Example external registry reference:

```json
{
  "allowed_fragment_candidate_ids": ["fragcand_1", "fragcand_2"],
  "registry_refs": {
    "fragment_registry_uri": "s3://example-bucket/run_001/fragment_registry.json",
    "claim_registry_uri": null,
    "evidence_registry_uri": null
  }
}
```

Rules for external registries:

- `registry_refs` is optional and internal to stage I/O.
- A stage runner must load and verify external registries before invoking the
  LLM or validator.
- The loaded registry must contain every ID listed in the matching
  `allowed_*_ids` array.
- Registry URIs must not be copied into the canonical result unless referenced
  through an audit or quarantine artifact.

---

## 4. `source_ingestion`

`source_ingestion` is pipeline-owned. It converts raw source manifests and
snapshots into a source registry. LLM may enrich descriptions in later stages,
but source identity and locator fields are deterministic.

### Input

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "source_ingestion",
  "run_id": "run_001",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "raw_material_refs": [
    {
      "material_id": "material_001",
      "source_type": "youtube_video",
      "canonical_url": "https://www.youtube.com/watch?v=abc123",
      "snapshot_id": "snapshot_001"
    }
  ]
}
```

### Output

```json
{
  "source_registry": [
    {
      "source_ref_id": "source_ref_1",
      "source_type": "youtube_video",
      "material_id": "material_001",
      "snapshot_id": "snapshot_001",
      "canonical_url": "https://www.youtube.com/watch?v=abc123",
      "internal_uri": "extractum://materials/material_001",
      "source_title": "Example video",
      "type_data": {
        "schema_version": "1.0"
      }
    }
  ]
}
```

Rules:

- `source_ref_id` is pipeline-owned.
- `type_data` must be completed according to `source_type_schemas.md` before
  canonical result assembly.
- `source_registry` is immutable for downstream LLM stages.

---

## 5. `fragment_candidate_mining`

`fragment_candidate_mining` creates the internal pre-contract fragment registry.
It does not create canonical `evidence[]`.

### Input

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "fragment_candidate_mining",
  "run_id": "run_001",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "source_registry": [
    {
      "source_ref_id": "source_ref_1",
      "source_type": "youtube_video",
      "source_title": "Example video"
    }
  ],
  "material_windows": [
    {
      "window_id": "window_1",
      "source_ref_id": "source_ref_1",
      "fragment_type": "video_timestamp_range",
      "locator_data": {
        "schema_version": "1.0",
        "timestamp_start": 120,
        "timestamp_end": 150
      },
      "window_text": "We are moving from demos to real internal pilots."
    }
  ]
}
```

### LLM Output

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

### Pipeline Output

```json
{
  "fragment_registry": [
    {
      "candidate_id": "fragcand_1",
      "source_ref_id": "source_ref_1",
      "fragment_type": "video_timestamp_range",
      "locator_data": {
        "schema_version": "1.0",
        "timestamp_start": 120,
        "timestamp_end": 150
      },
      "fragment_text": "We are moving from demos to real internal pilots.",
      "observation_summary": "Speaker describes a shift from demos to pilots.",
      "candidate_type": "trend_signal",
      "salience": "high"
    }
  ]
}
```

Rules:

- LLM output does not include `candidate_id`; pipeline assigns it.
- LLM does not invent `locator_data`; it references `window_id`.
- Pipeline validates and normalizes locator data before writing
  `fragment_registry`.

---

## 6. `claim_extraction`

`claim_extraction` converts fragment candidates into claim candidates.

### Input

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
      "fragment_type": "video_timestamp_range",
      "locator_data": {
        "schema_version": "1.0",
        "timestamp_start": 120,
        "timestamp_end": 150
      },
      "fragment_text": "We are moving from demos to real internal pilots.",
      "observation_summary": "Speaker describes a shift from demos to pilots."
    }
  ]
}
```

### LLM Output

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

Rules:

- LLM must use `fragment_candidate_refs`, not final `evidence_refs`.
- LLM must not assign `claim_id`.
- If allowed fragments are insufficient, LLM should produce an
  `unknown_candidate` or `verification_task_candidate`.
- LLM may return `warnings` for local candidate-level quality concerns, such as
  weak or conflicting candidate support. Pipeline decides which warnings become
  canonical result warnings.
- Pipeline validates all referenced `fragment_candidate_refs`.

---

## 7. `canonical_evidence_generation`

`canonical_evidence_generation` is pipeline-owned. It creates canonical
`claims[]` and `evidence[]` from claim candidates and fragment candidates.

### Input

```json
{
  "claim_candidates": [
    {
      "claim_candidate_id": "claimcand_1",
      "claim_text": "Local LLM agents are moving from demos to internal pilots.",
      "claim_type": "factual",
      "fragment_candidate_refs": ["fragcand_1"]
    }
  ],
  "fragment_registry": [
    {
      "candidate_id": "fragcand_1",
      "source_ref_id": "source_ref_1",
      "fragment_type": "video_timestamp_range",
      "locator_data": {
        "schema_version": "1.0",
        "timestamp_start": 120,
        "timestamp_end": 150
      },
      "fragment_text": "We are moving from demos to real internal pilots."
    }
  ]
}
```

### Pipeline Output

```json
{
  "claims": [
    {
      "claim_id": "claim_1",
      "claim_text": "Local LLM agents are moving from demos to internal pilots.",
      "claim_type": "factual",
      "evidence_refs": ["evidence_1"],
      "source_refs": ["source_ref_1"]
    }
  ],
  "evidence": [
    {
      "evidence_id": "evidence_1",
      "claim_id": "claim_1",
      "source_ref_id": "source_ref_1",
      "evidence_type": "fragment",
      "evidence_role": "supports",
      "fragment_type": "video_timestamp_range",
      "locator_data": {
        "schema_version": "1.0",
        "timestamp_start": 120,
        "timestamp_end": 150
      },
      "text_mode": "verbatim",
      "fragment_text": "We are moving from demos to real internal pilots."
    }
  ]
}
```

Rules:

- Pipeline assigns `claim_id` and `evidence_id`.
- One fragment candidate can generate multiple evidence objects if it supports
  multiple claims.
- Each canonical evidence object belongs to exactly one `claim_id`.
- Pipeline rebuilds `claim.source_refs`.

---

## 8. `claim_linking`

`claim_linking` proposes relations between fixed claims.

### Input

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "claim_linking",
  "run_id": "run_001",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "allowed_claim_ids": ["claim_1", "claim_2"],
  "allowed_evidence_ids": ["evidence_1", "evidence_2"],
  "claim_registry": [
    {
      "claim_id": "claim_1",
      "claim_text": "Local LLM agents are moving from demos to internal pilots."
    },
    {
      "claim_id": "claim_2",
      "claim_text": "Local LLM agents remain mostly experimental."
    }
  ]
}
```

### LLM Output

```json
{
  "relation_candidates": [
    {
      "relation_type": "contradicts",
      "source_claim_id": "claim_2",
      "target_claim_id": "claim_1",
      "description": "The claims disagree about whether local LLM agents have moved beyond experiments.",
      "evidence_refs": ["evidence_1", "evidence_2"],
      "confidence": {
        "score": 0.81,
        "basis": "conflicting_sources",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      }
    }
  ]
}
```

Rules:

- LLM may only reference allowed claims and evidence.
- Pipeline assigns `relation_id`.
- Pipeline normalizes `contradicts` source/target order by natural sort.
- Pipeline validates relation evidence ownership.

---

## 9. `pack_data_generation`

`pack_data_generation` creates pack-specific projections over the canonical
graph.

### Input

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "pack_data_generation",
  "run_id": "run_001",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "allowed_claim_ids": ["claim_1"],
  "allowed_evidence_ids": ["evidence_1"],
  "allowed_source_ref_ids": ["source_ref_1"],
  "claim_registry": [
    {
      "claim_id": "claim_1",
      "claim_text": "Local LLM agents are moving from demos to internal pilots.",
      "source_refs": ["source_ref_1"]
    }
  ],
  "evidence_registry": [
    {
      "evidence_id": "evidence_1",
      "claim_id": "claim_1",
      "source_ref_id": "source_ref_1"
    }
  ],
  "source_registry": [
    {
      "source_ref_id": "source_ref_1",
      "source_type": "youtube_video",
      "source_title": "Example video"
    }
  ]
}
```

### LLM Output

```json
{
  "pack_data_candidate": {
    "technology_watch": {
      "technologies": [
        {
          "name": "Local LLM agents",
          "normalized_name": "local_llm_agents",
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

Rules:

- LLM may create pack-specific semantic structures.
- Pipeline assigns pack-specific object IDs where the pack spec requires them.
- Pipeline rebuilds derived traversal refs in pack-specific objects.
- Pack-specific validation runs after pipeline assembly.

---

## 10. `final_synthesis`

`final_synthesis` drafts readable `outputs` over the fixed canonical graph and
pack-specific candidate data. Pipeline code assembles metadata, quality flags,
warnings, limitations, and audit pointers.

### Input

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "final_synthesis",
  "run_id": "run_001",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "claims": [
    {
      "claim_id": "claim_1",
      "claim_text": "Local LLM agents are moving from demos to internal pilots.",
      "evidence_refs": ["evidence_1"],
      "source_refs": ["source_ref_1"]
    }
  ],
  "pack_data": {
    "technology_watch": {
      "technologies": []
    }
  }
}
```

### LLM Output

```json
{
  "outputs_candidate": {
    "summary": {
      "title": "Short summary",
      "summary_text": "Local LLM agents are moving from demos to internal pilots.",
      "claim_refs": ["claim_1"],
      "evidence_refs": ["evidence_1"],
      "source_refs": ["source_ref_1"]
    },
    "sections": [
      {
        "title": "Technology trends",
        "section_type": "trends",
        "items": [
          {
            "title": "Local agents move into pilots",
            "text": "The analyzed source describes movement from demos to pilots.",
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

Rules:

- LLM may draft readable text and section structure.
- LLM does not create `metadata`.
- LLM does not create `quality_flags`.
- LLM does not create `warnings`, `limitations`, or `audit_refs`.
- Pipeline assigns `section_id` and `item_id`.
- Pipeline verifies `summary.claim_refs` coverage by section items.
- Pipeline assembles final `metadata`, `quality_flags`, `warnings`,
  `limitations`, and `audit_refs`.

---

## 11. Retry / Repair Payload

When validation fails in a retryable stage, the pipeline may send a compact
repair payload to the LLM.

The retry payload `stage` is the target stage being repaired, not the
`retry_repair` prompt stage. The provider prompt/render stage is `retry_repair`
and derives its repair target from this payload field.

```json
{
  "stage_io_version": "1.0",
  "stage": "claim_extraction",
  "retry_attempt": 1,
  "max_retry_attempts": 2,
  "repair_scope": "object",
  "previous_output_summary": "One claim candidate referenced a missing fragment candidate.",
  "validation_findings": [
    {
      "rule_id": "STAGE-REF-001",
      "severity": "error",
      "layer": "reference",
      "object_path": "claim_candidates[0].fragment_candidate_refs[0]",
      "message": "fragment_candidate_refs contains an ID not present in allowed_fragment_candidate_ids.",
      "object_refs": {
        "claim_refs": [],
        "evidence_refs": [],
        "source_refs": []
      }
    }
  ],
  "failed_object_paths": ["claim_candidates[0]"],
  "allowed_fragment_candidate_ids": ["fragcand_1", "fragcand_2"]
}
```

Rules:

- Retry payloads should include only the minimal context needed to repair the
  stage output.
- Retry prompts must preserve the same closed-world ID constraints.
- If a stage output contains both valid and invalid candidate objects, the
  pipeline should preserve valid candidates and quarantine or retry only the
  invalid candidates when object-level isolation is possible.
- Whole-stage retry is reserved for malformed payloads, systemic instruction
  failure, or errors that make object-level isolation unsafe.
- If retry fails, the pipeline follows the quarantine protocol from
  `execution_model_graph_assembly_policy.md`.

---

## 12. Stage Output Prohibitions

LLM stage outputs must not:

- assign canonical IDs unless the stage input explicitly provides them;
- invent references outside allowed registries;
- generate derived traversal fields as authoritative data;
- silently omit invalid objects without explaining the omission;
- include raw prompts, model traces, or full source dumps;
- return final canonical result JSON before pipeline assembly.

---

## 13. Implementation Notes

- Stage payloads are implementation contracts. They may evolve independently
  from the canonical result, but changes should bump `stage_io_version`.
- A reference implementation should keep fixtures for every stage input/output
  shape in this document.
- Pack-specific stages may add fields, but must preserve closed-world ID
  constraints.
- `stage_io_contracts.md` is intended to be usable as a source for typed data
  models, such as Python dataclasses or TypeScript interfaces. Generated models
  should preserve the distinction between candidate payloads and canonical
  result objects.
- The canonical result remains governed by
  `prompt_pack_json_contract_v1_draft.md`.
