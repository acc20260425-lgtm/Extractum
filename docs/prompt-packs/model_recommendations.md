# Model Recommendations Per Stage

Status: v1 draft.

Compatibility:

- Prompt Pack JSON Contract `schema_version: "1.0"`;
- Stage I/O Contracts `stage_io_version: "1.0"`;
- Provider-neutral stage prompts in `stage_prompt_templates.md`.

This document recommends model classes for prompt-pack execution stages. It is
not a provider SKU list. The goal is to make orchestration decisions explicit
without coupling the contract to a specific model catalog.

---

## 1. Purpose

Model selection should follow stage responsibility:

- cheap, high-throughput extraction for narrow local observations;
- stronger reasoning for claim synthesis and relation linking;
- pack-aware reasoning for structured pack projections;
- strong writing for final readable output;
- cheap repair first, with escalation only when validation keeps failing.

Rules:

- Do not hard-code provider SKU values in pack specs or canonical result
  contracts.
- Store provider-specific model names in runtime configuration or audit metadata,
  not in this document.
- A stage may override the recommended class when corpus risk, language,
  domain difficulty, or validation failure rate justifies it.
- Any escalation policy should preserve the same stage I/O contract and
  closed-world ID boundaries.

---

## 2. Model Class Vocabulary

Recommended `model_class` values:

| model_class | Intended use |
|---|---|
| `cheap_extractor` | Low-cost, high-throughput extraction from bounded windows. |
| `balanced_extractor` | Extraction with better multilingual or noisy-input tolerance. |
| `reasoning` | Claim synthesis, relation detection, and careful semantic mapping. |
| `pack_reasoning` | Pack-specific structured projections over fixed claims/evidence. |
| `writer` | Human-readable final summaries over an already fixed graph. |
| `repair` | Cheap schema/reference repair for local invalid outputs. |
| `strong_repair` | Fallback repair when local repair fails repeatedly. |

These are orchestration labels, not model names. A deployment maps each
`model_class` to provider-specific models in configuration.

---

## 3. Stage Recommendations

| Stage | Recommended model_class | Why |
|---|---|---|
| `fragment_candidate_mining` | `cheap_extractor` | Processes many small deterministic windows; task is local observation selection, not global reasoning. |
| `claim_extraction` | `reasoning` | Converts fragments into atomic claims and must avoid overclaiming beyond evidence. |
| `claim_linking` | `reasoning` | Requires careful comparison of fixed claims and evidence ownership constraints. |
| `pack_data_generation` | `pack_reasoning` | Builds pack-specific projections over a fixed graph while respecting pack rules. |
| `final_synthesis` | `writer` | Produces readable outputs without changing the graph; quality depends on clarity and restraint. |
| `retry_repair` | `repair` first, `strong_repair` on escalation | Most repair is local schema/reference cleanup; repeated failures justify escalation. |

Pipeline-owned stages such as `source_ingestion` and
`canonical_evidence_generation` should not require LLM model selection by
default. If a deployment uses LLM assistance there, it should document that as
an implementation extension, not as a baseline contract requirement.

---

## 4. Stage Notes

### `fragment_candidate_mining`

Default:

- `model_class`: `cheap_extractor`
- input shape: deterministic windows plus source cards
- output: fragment candidates without canonical IDs or locators

Use `balanced_extractor` when:

- ASR/transcript quality is poor;
- content is multilingual;
- windows are dense with technical terms;
- false negatives are more expensive than cost.

Do not use an expensive reasoning model for every window unless the corpus is
small or high-stakes. This stage is usually the dominant token-volume stage.

### `claim_extraction`

Default:

- `model_class`: `reasoning`
- input shape: immutable fragment registry and allowed candidate IDs
- output: claim, unknown, verification task, and warning candidates

The model should prefer `unknown_candidates` or `verification_task_candidates`
over unsupported claims. A cheaper model may be acceptable for low-risk packs,
but validation should monitor overclaiming and dangling reference attempts.

### `claim_linking`

Default:

- `model_class`: `reasoning`
- input shape: fixed claim/evidence registries
- output: relation candidates

This stage benefits from reasoning because relation errors are subtle:
`qualifies`, `supports`, and `contradicts` can be close in wording but different
in graph semantics. Pipeline code still normalizes deterministic fields such as
`contradicts` source/target ordering.

### `pack_data_generation`

Default:

- `model_class`: `pack_reasoning`
- input shape: fixed claim/evidence/source graph
- output: pack-specific candidate structures

This stage should be configured per pack. A simple extraction pack can use a
balanced model, while `technology_watch`-style analysis usually benefits from a
stronger pack-aware reasoning model.

### `final_synthesis`

Default:

- `model_class`: `writer`
- input shape: fixed graph and pack data candidate
- output: readable `outputs_candidate`

The model must not change claims, evidence, source refs, metadata, quality
flags, warnings, limitations, or audit refs. Choose for faithful summarization,
not for graph construction.

### `retry_repair`

Default:

- first attempt: `repair`
- repeated failure or semantic drift: `strong_repair`

Use cheap repair for:

- malformed JSON;
- unknown top-level keys;
- one invalid reference in an otherwise valid object;
- missing optional candidate arrays.

Escalate to `strong_repair` when:

- the same validation class repeats after retry;
- object-isolated repair cannot identify a safe replacement;
- a whole-stage repair keeps changing valid objects;
- the finding indicates semantic overreach rather than a syntax/reference issue.

---

## 5. Escalation Policy

Recommended escalation ladder:

```text
cheap_extractor -> balanced_extractor -> reasoning
repair -> strong_repair -> quarantine/fail
writer -> reasoning writer only when factual faithfulness keeps failing
```

Rules:

- Escalation changes model quality/cost, not stage output shape.
- Escalation must not widen allowed ID arrays.
- Escalation should emit audit events through the graph assembly policy.
- Repeated repair failure should prefer quarantine or hard fail over silent
  mutation of the graph.

Useful routing metrics:

- parse failure rate;
- schema failure rate;
- dangling reference attempts;
- overclaiming warnings;
- quarantine count;
- manual review rate.

---

## 6. Runtime Configuration Shape

Recommended runtime configuration shape:

```json
{
  "model_routing_version": "1.0",
  "default_provider_family": "openai_compatible",
  "model_classes": {
    "cheap_extractor": {
      "provider_model": "configured-at-runtime",
      "temperature": 0.1
    },
    "reasoning": {
      "provider_model": "configured-at-runtime",
      "temperature": 0.2
    },
    "writer": {
      "provider_model": "configured-at-runtime",
      "temperature": 0.3
    },
    "repair": {
      "provider_model": "configured-at-runtime",
      "temperature": 0.0
    }
  },
  "stage_routes": {
    "fragment_candidate_mining": "cheap_extractor",
    "claim_extraction": "reasoning",
    "claim_linking": "reasoning",
    "pack_data_generation": "pack_reasoning",
    "final_synthesis": "writer",
    "retry_repair": "repair"
  }
}
```

This configuration is illustrative. It belongs to runtime orchestration, not to
canonical result JSON.

---

## 7. Open Questions

OQ-MR-01: Should each pack be allowed to define a pack-local override table for
`model_class` routing, or should all routing remain in deployment config?

OQ-MR-02: Should validator output include model-routing health metrics, or
should those remain in pipeline telemetry outside canonical result JSON?
