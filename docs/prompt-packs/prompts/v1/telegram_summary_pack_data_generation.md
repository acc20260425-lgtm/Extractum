# Telegram Summary Pack Data Generation Prompt Guidance

Status: v1 draft.

This document is pack-specific guidance for rendering the generic
`pack_data_generation` prompt when `pack_id = "telegram_summary"`.

The provider prompt file remains:

```text
docs/prompt-packs/prompts/v1/openai-compatible/pack_data_generation.prompt.json
```

This file defines the additional Telegram-specific payload context and wording
that should be injected into `{{stage_payload_json}}` or the surrounding prompt
context.

## Purpose

The `telegram_summary` pack turns a fixed Telegram message graph into:

- a short digest for the selected period;
- a timeline of important events;
- key messages with reasons for importance;
- topic summaries;
- reply chain and thread summaries;
- claim summaries grounded in canonical claims/evidence;
- forwarded-message notes;
- optional message quality signals.

The model does not create canonical result IDs. It only creates a
`pack_data_candidate.telegram_summary` object using allowed IDs.

## Required Runtime Inputs

The stage payload should include or make available:

- `pack_id = "telegram_summary"`;
- `source_shape`: `channel`, `chat`, or `mixed`;
- `allowed_claim_ids`;
- `allowed_evidence_ids`;
- `allowed_source_ref_ids`;
- `allowed_message_ref_ids`;
- `source_registry`;
- `claim_registry`;
- `evidence_registry`;
- `telegram_message_registry`;
- optional `telegram_thread_hints`;
- optional `telegram_topic_hints`;
- optional `time_window`;
- optional runtime controls for `summary_depth`, `include_quality_signals`,
  and `importance_scoring_profile`.

`telegram_message_registry` is the authoritative closed-world registry for
Telegram-local message references. Its items should expose at least:

- `message_ref_id`;
- `summary_source_id`;
- `source_ref_id`;
- `message_id`;
- `published_at`;
- `author_display`;
- `message_kind`;
- reply/thread links when known;
- forwarded-message metadata when known;
- reaction/reply/forward counts when available;
- linked `claim_refs`, `evidence_refs`, and `source_refs`.

## Closed-World Rules

The model must follow these rules:

- Use only IDs from `allowed_claim_ids`, `allowed_evidence_ids`,
  `allowed_source_ref_ids`, and `allowed_message_ref_ids`.
- Do not invent `message_ref_id`, `claim_id`, `evidence_id`, `source_ref_id`,
  `topic_id`, `thread_id`, or canonical result IDs.
- Do not output final canonical result JSON.
- Do not output top-level `claims`, `evidence`, `source_refs`,
  `claim_relations`, `outputs`, `result_id`, or `audit_refs`.
- Do not treat forwarded messages as independent confirmation unless a separate
  source/message supports the same claim.
- Preserve reply chains and forum topic context when they help explain why a
  message matters.
- If a conclusion cannot be grounded in allowed IDs, put the gap into
  `unknown_candidates` or `warning_candidates` instead of inventing support.

## Output Shape

Return only this JSON object:

```json
{
  "pack_data_candidate": {
    "telegram_summary": {
      "source_shape": "channel",
      "sources": [],
      "time_window": null,
      "message_refs": [],
      "digest": null,
      "timeline": [],
      "topics": [],
      "key_messages": [],
      "threads": [],
      "claims": [],
      "forwarded_items": [],
      "message_quality_signals": [],
      "cross_source_synthesis": null,
      "limitations": []
    }
  },
  "unknown_candidates": [],
  "warning_candidates": []
}
```

The shape inside `telegram_summary` follows
`docs/prompt-packs/telegram_summary_pack_spec.md`.

## Source Shape Guidance

Use `source_shape = "channel"` when all messages come from broadcast-style
channel snapshots. Emphasize timeline, topic shifts, important posts, and
forwarded material.

Use `source_shape = "chat"` when messages come from group/chat snapshots.
Emphasize reply chains, question/answer pairs, useful user messages, consensus,
disagreement, and unresolved threads.

Use `source_shape = "mixed"` when multiple Telegram sources or source kinds are
summarized together. Include `cross_source_synthesis` when the depth/runtime
profile requests cross-source synthesis.

## Message Importance Guidance

A message can be important because it:

- contains a central claim;
- starts or resolves a reply chain;
- receives unusually high reactions/replies/forwards;
- changes the topic or timeline;
- provides an answer, correction, decision, warning, or useful resource;
- is a forwarded item that introduces an external source or claim.

When `importance_score` or a quality score is used, include short reason codes
such as:

- `claim_dense`;
- `useful_answer`;
- `thread_root`;
- `high_reaction_count`;
- `high_forward_count`;
- `decision_or_action`;
- `correction`;
- `source_link`.

## Template Text Smoke Contract

A valid Telegram pack-data-generation prompt template should include:

- closed-world language;
- `allowed_message_ref_ids`;
- `telegram_message_registry`;
- the exact output namespace `pack_data_candidate.telegram_summary`;
- a ban on final canonical result JSON;
- a ban on inventing Telegram message refs;
- instructions for reply chains / threads;
- instructions for forwarded messages.

The fixture
`prompt_template__valid__telegram_summary_pack_data_generation` captures this
minimum smoke contract.
