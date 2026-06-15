# Runtime Configuration Policy

Status: v1 draft.

Compatibility:

- Prompt Pack JSON Contract `schema_version: "1.0"`;
- Stage I/O Contracts `stage_io_version: "1.0"`;
- Model recommendations in `model_recommendations.md`.
- Runtime configuration schema
  `schemas/v1/runtime/runtime_configuration.schema.json`.

This document defines which execution settings belong to runtime configuration,
which signals may be summarized in canonical result JSON, and which details
must remain in audit or telemetry systems.

---

## 1. Purpose

Prompt-pack execution needs runtime settings that should not become part of the
canonical result contract. Examples include concrete provider model names,
timeouts, retry counts, feature flags, parser options, and telemetry sinks.

The canonical result JSON should describe what the pipeline produced and how it
can be audited. It should not become a live execution config document.

Core rule:

> Do not copy runtime configuration into canonical result JSON.

Runtime configuration may influence the result. When it does, the result should
carry concise provenance, warnings, quality flags, and `audit_refs`, not the
entire config payload.

---

## 2. Boundary Map

| Area | Runtime configuration | Canonical result JSON | Audit / telemetry |
|---|---|---|---|
| `model_routing` | Provider family, provider model, model class mapping, fallback routes. | Optional model/provider names only inside provenance when already produced by the pipeline. | Full routing decision, fallback reason, latency, cost. |
| `feature_flags` | Enable/disable retry, repair, quarantine, strict validation, parser fallback. | Result effects through `quality_flags`, `warnings`, `limitations`, and `metadata`. | Exact flag values used for the run. |
| `budget_limits` | Max retries, max tokens, timeouts, candidate caps, stage concurrency. | Not copied directly. May surface as `partial_result` or limitation when limits affected output. | Full budget settings and exhaustion events. |
| `retry_policy` | Retry attempts, retryable finding classes, repair escalation ladder. | Retry effects through `audit_refs`, warnings, and quality flags. | Each retry attempt and validation finding. |
| `quarantine_policy` | Quarantine storage location, retention, redaction rules. | Short warning/quality flag plus `audit_refs` pointing to quarantine event. | Full quarantine artifact URI and redaction metadata. |
| `telemetry` | Metrics sinks, sampling, dashboard labels. | Not copied directly. | Parse failure rate, validation failure rate, retry counts, quarantine counts, latency, cost. |

---

## 3. Runtime Configuration Shape

Recommended top-level shape:

```json
{
  "runtime_configuration": {
    "runtime_config_version": "1.0",
    "model_routing": {
      "model_routing_version": "1.0",
      "default_provider_family": "openai_compatible",
      "stage_routes": {
        "fragment_candidate_mining": "cheap_extractor",
        "claim_extraction": "reasoning",
        "claim_linking": "reasoning",
        "pack_data_generation": "pack_reasoning",
        "final_synthesis": "writer",
        "retry_repair": "repair"
      }
    },
    "feature_flags": {
      "retry_enabled": true,
      "object_repair_enabled": true,
      "quarantine_enabled": true,
      "strict_reference_validation": true,
      "parser_fallback_enabled": false
    },
    "budget_limits": {
      "max_retry_attempts": 2,
      "stage_timeout_seconds": 120,
      "max_prompt_tokens": 24000,
      "max_output_tokens": 4000
    },
    "retry_policy": {
      "retryable_layers": ["schema", "reference"],
      "escalate_after_attempts": 1,
      "fallback_model_class": "strong_repair"
    },
    "quarantine_policy": {
      "store": "configured-at-runtime",
      "redact_raw_provider_output": false,
      "retention_days": 30
    },
    "telemetry": {
      "enabled": true,
      "sink": "configured-at-runtime"
    }
  }
}
```

This shape is illustrative. It is not part of canonical result JSON.

The machine-readable schema for this shape lives at:

```text
schemas/v1/runtime/runtime_configuration.schema.json
```

The schema validates implementation configuration artifacts. It does not make
`runtime_configuration` a valid field inside canonical result JSON.

---

## 3.1 Bundled Runtime Assets

Bundled prompt packs may ship runtime configuration artifacts next to pack and
stage assets. Use this path convention for stage-specific runtime settings:

```text
src-tauri/prompt-packs/<pack_id>/<pack_version>/runtime/<stage_name>.json
```

For stage names that contain `/`, use the final route segment as the filename.
For example, the MVP YouTube Summary transcript-analysis stage uses:

```text
src-tauri/prompt-packs/youtube_summary/1.0.0/runtime/transcript_analysis.json
```

This file stores runtime-only execution settings such as
`budget_limits.max_output_tokens`. The current YouTube Summary MVP reads
`runtime_configuration.budget_limits.max_output_tokens = 4096` from this asset
and then clamps it to the selected provider model's `output_token_limit` when
that metadata is available.

Runtime assets are operational configuration. They are not canonical result
schema, and they are not copied into report output. In the current bundled MVP
seed path, runtime assets are intentionally separate from the prompt-pack
version content hash so output budgets can be tuned without changing stage
schemas or prompt templates.

---

## 4. Model Routing

`model_routing` maps `model_class` values from `model_recommendations.md` to
provider-specific runtime choices.

Rules:

- Store concrete provider model names in runtime configuration, not pack specs.
- Stage prompts should receive only the model they are executed with, not the
  full routing table.
- Provider model names may appear in `provenance.model` or audit events after a
  stage has run.
- If a route escalates, emit an audit event with the previous class, new class,
  and reason.

Canonical result impact:

- `provenance.provider` and `provenance.model` may record what was used for a
  claim, evidence item, relation, or unknown.
- Full routing tables stay outside canonical JSON.

---

## 5. Feature Flags

`feature_flags` control execution behavior. They do not change the JSON
contract.

Recommended flags:

| Flag | Meaning |
|---|---|
| `retry_enabled` | Allows retry after retryable parser or validator findings. |
| `object_repair_enabled` | Allows object-isolated replacement repair. |
| `quarantine_enabled` | Allows invalid objects/artifacts to be stored outside canonical JSON. |
| `strict_reference_validation` | Enforces closed-world ID checks before assembly. |
| `parser_fallback_enabled` | Allows implementation-specific parser fallback behavior. |

Rules:

- Turning a feature flag on or off must not create a new canonical schema.
- If a disabled feature affects output completeness, surface that as a
  limitation, warning, or quality flag.
- Do not place the full `feature_flags` object inside canonical result JSON.

---

## 6. Budget Limits

`budget_limits` protect execution cost and latency.

Examples:

- `max_retry_attempts`;
- stage timeout;
- prompt/output token caps;
- max fragment candidates per material;
- max claims per source;
- max relation candidates per run.

Rules:

- Budget limits are runtime-only unless they affected result completeness.
- If a budget limit truncates analysis, canonical JSON should include
  `quality_flags.flag = "partial_result"` or a relevant limitation.
- Audit events should record the exhausted limit and affected stage.

---

## 7. Retry Policy

`retry_policy` controls when the pipeline retries, repairs, escalates, or fails.

Recommended behavior:

- Retry schema/reference failures when the invalid object can be isolated.
- Prefer object-level repair over whole-stage repair when safe.
- Escalate from `repair` to `strong_repair` only after repeated failure or
  semantic drift.
- Do not silently drop invalid objects.

Canonical result impact:

- Successful repair may appear only as an audit event.
- Exhausted retry should produce a warning, quality flag, limitation, or hard
  failure depending on severity.
- Retry attempt details belong in audit and telemetry.

---

## 8. Quarantine Policy

`quarantine_policy` controls where invalid or unsafe artifacts are stored.

Rules:

- Quarantine artifacts live outside canonical result JSON.
- Canonical JSON may point to quarantine through `audit_refs`.
- Quarantine events should not be silent; they should be reflected in warnings
  or quality flags when they affect result completeness.
- Redaction policy is runtime-owned and may vary by deployment.

Recommended audit summary:

```json
{
  "event_type": "quarantine",
  "summary": "One invalid claim candidate was quarantined after retry failure.",
  "object_refs": {
    "claim_refs": [],
    "evidence_refs": [],
    "source_refs": []
  }
}
```

---

## 9. Telemetry

`telemetry` is for operations and monitoring, not for canonical result
semantics.

Recommended metrics:

- parse failure rate;
- schema failure rate;
- reference failure rate;
- retry count;
- repair success rate;
- quarantine count;
- stage latency;
- token usage;
- approximate cost;
- manual review rate.

Canonical result impact:

- Aggregate telemetry should not be copied into canonical JSON.
- If telemetry reveals a result-quality issue during the run, express the issue
  through existing result fields: `warnings`, `limitations`, `quality_flags`,
  `metadata`, and `audit_refs`.

---

## 10. Result Exposure Rules

Allowed in canonical result JSON:

- concise provenance fields;
- provider/model actually used for a produced analytical artifact;
- `audit_refs` to detailed execution records;
- quality flags and warnings that affect interpretation;
- limitations caused by disabled features, budget exhaustion, or unavailable
  runtime capabilities.

Not allowed in canonical result JSON:

- full runtime configuration;
- API keys, endpoint URLs, secrets, or credentials;
- full telemetry streams;
- full quarantine payloads;
- complete model routing tables;
- operational dashboard labels.

---

## 11. Resolved Decisions

RC-01: v1 includes a machine-readable runtime configuration schema under
`schemas/v1/runtime/runtime_configuration.schema.json`.

Rationale: runtime configuration is implementation-owned, but validating its
local shape is useful for orchestrator and CLI integration. Keeping the schema
under `schemas/v1/runtime/` makes it discoverable without placing runtime config
inside canonical result JSON.

---

## 12. Open Questions

OQ-RC-02: Should audit events store a hash of the runtime configuration used
for a run, enabling reproducibility without embedding the full config?
