# Prompt Pack JSON Contract — v1 Draft

## Companion Documents

| Документ | Назначение | Версия |
|---|---|---|
| `source_type_schemas.md` | `type_data` schemas для стандартных `source_type` | 1.0 |
| `fragment_locator_schemas.md` | `locator_data` schemas для стандартных `fragment_type` | 1.0 |
| `schemas/README.md` | Machine-readable JSON Schema placement and v1 semantic schema bundle | v1 semantic |
| `execution_model_graph_assembly_policy.md` | Execution model, graph assembly, healing, quarantine, and fragment deduplication policy | v1 draft |
| `stage_io_contracts.md` | Internal stage-level input/output contracts for execution pipeline stages | v1 draft |
| `stage_prompt_templates.md` | Provider-neutral prompt skeletons and output contracts for LLM stages | v1 draft |

Все companion documents версионируются вместе с `schema_version` основного контракта.
Изменение companion document, добавляющее обязательные поля, требует bump версии.

## Pack-specific Specs

| Документ | Назначение | Версия |
|---|---|---|
| `technology_watch_pack_spec.md` | `outputs.pack_data.technology_watch` schema для первого pack | v1 |
| `youtube_summary_pack_spec.md` | `outputs.pack_data.youtube_summary` schema для YouTube summary pack | v1 draft |

Pack-specific specs версионируются независимо от `schema_version` основного
контракта, но должны явно указывать совместимую версию общего контракта.
Если pack spec начинает поддерживать другой `schema_version`, он должен либо
bump-нуть `pack_version`, либо добавить compatibility table. Один `pack_version`
не должен молча означать разные core-contract shapes.

## Validation Documents

| Документ | Назначение | Версия |
|---|---|---|
| `validation_rules.md` | Карта hard errors, warnings, QA и pipeline-level validation rules | v1 draft |
| `validator_manifest.md` | Execution manifest for reference validator phases, inputs, rule groups, fixtures | v1 draft |
| `validator_fixtures.md` | Fixture catalog for reference validator CI and local validation tests | v1 draft |
| `schemas/README.md` | Machine-readable schema bundle placement and loading conventions | v1 draft |
| `stage_io_contracts.md` | Internal stage payloads, closed-world registries, retry/repair payloads | v1 draft |
| `stage_prompt_templates.md` | Prompt skeletons, expected JSON outputs, retry prompts, few-shot rules | v1 draft |
| `prompts/v1/README.md` | Provider-specific prompt file layout and checked-in prompt renders | v1 draft |
| `parser-fixtures/v1/README.md` | Raw provider-response parser fixture layout before parsed stage-output validation | v1 draft |

Provider prompt and parser artifacts are execution documentation, not canonical
result fields. `prompts/v1/` contains provider-specific prompt renderings
derived from `stage_prompt_templates.md`; `parser-fixtures/v1/` contains raw
provider-response fixtures before parsed `stage_output` validation.

Validation documents не добавляют новые поля контракта. Они классифицируют
уже зафиксированные правила для validator implementation и pipeline QA.
`validator_manifest.md` связывает эти правила с execution phases, required
inputs, blocking behavior и CI fixture expectations для reference validator.
`validator_fixtures.md` фиксирует concrete fixture classes, naming conventions
и expected findings для reference validator CI.
`schemas/README.md` фиксирует placement будущих executable JSON Schema files;
сами schema files не являются обязательными полями canonical result JSON.
`execution_model_graph_assembly_policy.md` дополняет validation map: он описывает,
какие поля генерирует LLM, какие поля собирает pipeline, как работают healing,
retry, quarantine, pre-contract fragment registry и fragment deduplication.
`stage_io_contracts.md` фиксирует internal payload boundaries между stage-ами
pipeline; он не добавляет поля в canonical result JSON.
`stage_prompt_templates.md` фиксирует provider-neutral prompt skeletons для
stage payloads; он также не добавляет поля в canonical result JSON.

---

## Раздел 14: Минимальный пример JSON

Один экземпляр каждого типа объекта. Демонстрирует полную цепочку:
`source_ref → evidence → claim → claim_relation → unknown → verification_task → warning → limitation → quality_flag → audit_ref`.

```json
{
  "schema_version": "1.0",
  "result_id": "result_001",
  "parent_result_ids": null,
  "run_id": "run_001",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "stage": "final_result",
  "created_at": "2026-06-06T12:00:00Z",
  "output_language": "ru",

  "metadata": {
    "result_type": "pack_result",
    "result_status": "complete",
    "producer_type": "llm",
    "contains_partial_results": false,
    "contains_unverified_claims": true
  },

  "run_context": {
    "project_id": "project_001",
    "preflight_id": "preflight_001",
    "project_goal": "Оценить технологические тренды в области AI-агентов.",
    "run_goal": "Найти зрелые инструменты для внедрения в 2026 году.",
    "selected_pack": {
      "pack_id": "technology_watch",
      "pack_version": "v1"
    },
    "control_preset": "standard",
    "evidence_mode": "standard",
    "output_language": "ru",
    "source_languages": ["en", "ru"],
    "period": {
      "from": "2026-01-01",
      "to": "2026-06-06"
    },
    "input_corpus": {
      "source_types": ["youtube", "web", "rss"],
      "selected_source_count": 12,
      "selected_material_count": 84,
      "selected_fragment_count": 1320
    },
    "model_selection": [
      {
        "stage": "source_analysis",
        "provider": "openai_compatible",
        "model": "gpt-4.1-mini"
      },
      {
        "stage": "final_synthesis",
        "provider": "openai_compatible",
        "model": "gpt-4.1"
      }
    ]
  },

  "outputs": {
    "summary": {
      "title": "Краткое резюме",
      "summary_text": "В корпусе заметно одно зрелое направление: локальные LLM-агенты переходят из экспериментов в ограниченные пилоты.",
      "claim_refs": ["claim_1"],
      "evidence_refs": ["evidence_1"],
      "source_refs": ["source_ref_1"]
    },
    "sections": [
      {
        "section_id": "section_technology_trends",
        "title": "Технологические тренды",
        "section_type": "trends",
        "custom_section_type": null,
        "items": [
          {
            "item_id": "item_1",
            "title": "Локальные LLM-агенты переходят из экспериментов в пилоты",
            "text": "Несколько источников описывают переход от прототипов к ограниченному внедрению.",
            "claim_refs": ["claim_1"],
            "evidence_refs": ["evidence_1"],
            "source_refs": ["source_ref_1"]
          }
        ]
      }
    ],
    "pack_data": {
      "technology_watch": {
        "technologies": []
      }
    }
  },

  "source_refs": [
    {
      "source_ref_id": "source_ref_1",
      "source_type": "youtube_video",
      "custom_source_type": null,
      "canonical_url": "https://www.youtube.com/watch?v=abc123",
      "internal_uri": "extractum://materials/material_001",
      "source_title": "Local AI Agents in Production",
      "source_id": "source_001",
      "material_id": "material_001",
      "snapshot_id": "snapshot_001",
      "published_at": "2026-04-12T10:00:00Z",
      "accessed_at": "2026-06-06T09:30:00Z",
      "access_status": "cached",
      "type_data": {
        "schema_version": "1.0",
        "video_id": "abc123",
        "duration_seconds": 1840,
        "language": "en",
        "captions_available": true,
        "transcript_available": true,
        "is_live_recording": false,
        "view_count": 45200,
        "like_count": 1830,
        "comment_count": 214,
        "comment_collection_status": "collected",
        "playlist_id": null,
        "playlist_title": null,
        "playlist_position": null,
        "scraped_at": "2026-06-06T09:30:00Z",
        "creator": {
          "creator_type": "channel",
          "custom_creator_type": null,
          "id": "channel_001",
          "platform_specific_id": "UCabc123",
          "display_name": "Example Channel",
          "profile_url": "https://youtube.com/@example"
        },
        "parent_context": {
          "context_type": "youtube_channel",
          "custom_context_type": null,
          "context_id": "channel_001",
          "platform_specific_id": "UCabc123",
          "context_title": "Example Channel",
          "context_url": "https://youtube.com/@example"
        },
        "extra_metadata": {}
      }
    }
  ],

  "claims": [
    {
      "claim_id": "claim_1",
      "claim_type": "factual",
      "custom_claim_type": null,
      "claim_status": "partially_verified",
      "custom_claim_status": null,
      "claim_text": "Локальные LLM-агенты переходят из экспериментов в ограниченные пилоты.",
      "normalized_claim_text": "Local LLM agents are moving from experiments to limited pilots.",
      "normalized_claim_language": "en",
      "scope": {
        "period": {
          "from": "2026-01-01",
          "to": "2026-06-06"
        },
        "geo": null,
        "language": null,
        "applies_to": [
          {
            "label": "local LLM agents",
            "entity_type": "technology",
            "entity_id": null
          }
        ]
      },
      "confidence": {
        "score": 0.78,
        "basis": "multiple_corroborating_sources",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "evidence_refs": ["evidence_1"],
      "source_refs": ["source_ref_1"],
      "relation_refs": [],
      "provenance": {
        "stage": "claim_extraction",
        "provider": "openai_compatible",
        "model": "gpt-4.1-mini",
        "audit_refs": ["audit_1"]
      }
    }
  ],

  "evidence": [
    {
      "evidence_id": "evidence_1",
      "claim_id": "claim_1",
      "source_ref_id": "source_ref_1",
      "evidence_type": "fragment",
      "custom_evidence_type": null,
      "evidence_role": "supports",
      "custom_evidence_role": null,
      "fragment_type": "video_timestamp_range",
      "custom_fragment_type": null,
      "locator_data": {
        "schema_version": "1.0",
        "timestamp_start": 312.5,
        "timestamp_end": 346.0
      },
      "text_mode": "verbatim",
      "fragment_text": "We moved local agents from lab demos into limited customer pilots this quarter.",
      "context_text": "Speaker describes the transition from internal experiments to external pilots.",
      "contributing_evidence_refs": [],
      "reasoning_summary": null,
      "confidence": {
        "score": 0.86,
        "basis": "strong_direct_evidence",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "provenance": {
        "stage": "evidence_linking",
        "provider": "openai_compatible",
        "model": "gpt-4.1-mini",
        "audit_refs": ["audit_2"]
      }
    }
  ],

  "claim_relations": [],

  "unknowns": [
    {
      "unknown_id": "unknown_1",
      "unknown_type": "conflicting",
      "custom_unknown_type": null,
      "title": "Неясно, насколько пилоты перешли в production",
      "description": "Источники подтверждают ограниченные пилоты, но не дают достаточных данных о промышленном внедрении.",
      "why_it_matters": "Без этого нельзя оценить зрелость технологии как готовую к широкому применению.",
      "claim_refs": ["claim_1"],
      "source_refs": ["source_ref_1"],
      "evidence_refs": ["evidence_1"],
      "relation_refs": [],
      "verification_task_refs": ["verification_task_1"],
      "confidence": {
        "score": 0.72,
        "basis": "conflicting_sources",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "provenance": {
        "stage": "quality_check",
        "provider": "openai_compatible",
        "model": "gpt-4.1",
        "audit_refs": ["audit_3"]
      }
    }
  ],

  "verification_tasks": [
    {
      "verification_task_id": "verification_task_1",
      "task_type": "source_search",
      "custom_task_type": null,
      "priority": "high",
      "task": "Найти источники, подтверждающие или опровергающие переход локальных LLM-агентов из пилотов в production.",
      "where_to_check": "Официальные changelog, кейсы внедрения, публичные customer stories, технические блоги компаний.",
      "expected_evidence_type": "Прямое подтверждение production-внедрения или явное указание, что речь идёт только о пилотах.",
      "status_change_condition": "Если найдено два независимых подтверждения production-внедрения, claim можно повысить до verified; если найдено опровержение — создать contradicting claim.",
      "unknown_id": "unknown_1",
      "claim_refs": ["claim_1"],
      "source_refs": [],
      "evidence_refs": []
    }
  ],

  "warnings": [
    {
      "warning_id": "warning_1",
      "warning_type": "single_source_claim",
      "custom_warning_type": null,
      "severity": "medium",
      "message": "claim_1 основан на одном источнике. Уверенность ограничена.",
      "claim_refs": ["claim_1"],
      "source_refs": ["source_ref_1"],
      "evidence_refs": ["evidence_1"],
      "relation_refs": [],
      "section_refs": ["section_technology_trends"]
    }
  ],

  "limitations": [
    {
      "limitation_id": "limitation_1",
      "limitation_type": "corpus_coverage_limited",
      "custom_limitation_type": null,
      "severity": "medium",
      "description": "Корпус содержит в основном англоязычные источники и не отражает локальные рынки.",
      "claim_refs": [],
      "source_refs": [],
      "evidence_refs": [],
      "relation_refs": [],
      "section_refs": []
    },
    {
      "flag": "single_source_claim",
      "custom_flag": null,
      "severity": "medium",
      "message": "claim_1 основан на одном source_ref и требует внимания при downstream review.",
      "claim_refs": ["claim_1"],
      "source_refs": ["source_ref_1"],
      "evidence_refs": ["evidence_1"],
      "relation_refs": [],
      "section_refs": ["section_technology_trends"]
    }
  ],

  "quality_flags": [
    {
      "flag": "unverified_claims_present",
      "custom_flag": null,
      "severity": "medium",
      "message": "Результат содержит claims со статусом partially_verified или unverified.",
      "claim_refs": ["claim_1"],
      "source_refs": [],
      "evidence_refs": [],
      "relation_refs": [],
      "section_refs": []
    }
  ],

  "audit_refs": [
    {
      "audit_id": "audit_1",
      "audit_uri": "extractum://audit/project_audit_log/audit_1",
      "audit_store": "project_audit_log",
      "event_type": "model_call",
      "custom_event_type": null,
      "stage": "claim_extraction",
      "timestamp": "2026-06-06T09:40:00Z",
      "summary": "LLM extracted candidate claims from source-level summaries.",
      "object_refs": {
        "claim_refs": ["claim_1"],
        "evidence_refs": [],
        "relation_refs": [],
        "source_refs": ["source_ref_1"],
        "unknown_refs": [],
        "verification_task_refs": [],
        "warning_refs": [],
        "limitation_refs": []
      }
    },
    {
      "audit_id": "audit_2",
      "audit_uri": "extractum://audit/project_audit_log/audit_2",
      "audit_store": "project_audit_log",
      "event_type": "model_call",
      "custom_event_type": null,
      "stage": "evidence_linking",
      "timestamp": "2026-06-06T09:45:00Z",
      "summary": "LLM linked evidence fragments to extracted claims.",
      "object_refs": {
        "claim_refs": ["claim_1"],
        "evidence_refs": ["evidence_1"],
        "relation_refs": [],
        "source_refs": ["source_ref_1"],
        "unknown_refs": [],
        "verification_task_refs": [],
        "warning_refs": [],
        "limitation_refs": []
      }
    },
    {
      "audit_id": "audit_3",
      "audit_uri": "extractum://audit/project_audit_log/audit_3",
      "audit_store": "project_audit_log",
      "event_type": "validation",
      "custom_event_type": null,
      "stage": "quality_check",
      "timestamp": "2026-06-06T09:55:00Z",
      "summary": "Quality check identified unresolved unknowns and generated verification tasks.",
      "object_refs": {
        "claim_refs": ["claim_1"],
        "evidence_refs": ["evidence_1"],
        "relation_refs": [],
        "source_refs": ["source_ref_1"],
        "unknown_refs": ["unknown_1"],
        "verification_task_refs": ["verification_task_1"],
        "warning_refs": ["warning_1"],
        "limitation_refs": ["limitation_1"]
      }
    }
  ]
}
```

В минимальном примере `claim_1.source_refs` совпадает с union source refs,
достижимых через `claim_1.evidence_refs`: `evidence_1.source_ref_id =
"source_ref_1"`. Если claim ссылается на несколько evidence из разных
материалов, `claim.source_refs` должен содержать их объединение.

---

### Дополнительный пример: `claim_relations`

Минимальный пример выше содержит `claim_relations: []` намеренно — одного claim
недостаточно для демонстрации осмысленного отношения. Ниже изолированные примеры.

**`qualifies`** — направленное отношение, `claim_2` уточняет `claim_1`:

```json
{
  "claim_relations": [
    {
      "relation_id": "rel_1",
      "relation_type": "qualifies",
      "custom_relation_type": null,
      "source_claim_id": "claim_2",
      "target_claim_id": "claim_1",
      "description": "claim_2 уточняет условия применимости claim_1: переход наблюдается только у компаний с выделенной ML-инфраструктурой.",
      "evidence_refs": ["evidence_1"],
      "confidence": {
        "score": 0.74,
        "basis": "inference_chain",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "provenance": {
        "stage": "claim_linking",
        "provider": "openai_compatible",
        "model": "gpt-4.1",
        "audit_refs": []
      }
    }
  ]
}
```

**`contradicts`** — симметричное отношение, порядок claims определяется
natural sort по `claim_id`: `source_claim_id` идёт раньше `target_claim_id`.
В v1 `claim_id` обязан иметь формат `prefix_N`: непустой строковый префикс,
финальный `_` и числовой суффикс. Natural sort сравнивает общий префикс
лексикографически, а числовой суффикс после последнего `_` — как число.
Примеры порядка: `claim_2` идёт раньше `claim_10`,
`claim_abc_2` идёт раньше `claim_abc_10`.

```json
{
  "claim_relations": [
    {
      "relation_id": "rel_2",
      "relation_type": "contradicts",
      "custom_relation_type": null,
      "source_claim_id": "claim_1",
      "target_claim_id": "claim_3",
      "description": "claim_1 фиксирует переход в пилоты; claim_3 утверждает, что внедрение по-прежнему экспериментальное на конец периода.",
      "evidence_refs": ["evidence_1", "evidence_3"],
      "confidence": {
        "score": 0.81,
        "basis": "conflicting_sources",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "provenance": {
        "stage": "contradiction_check",
        "provider": "openai_compatible",
        "model": "gpt-4.1",
        "audit_refs": []
      }
    }
  ]
}
```

`"claim_1"` идёт раньше `"claim_3"` по natural sort — конвенция для
`contradicts` выдержана.

`relation.evidence_refs` может включать evidence обеих сторон противоречия:
например, `evidence_1` принадлежит `claim_1`, а `evidence_3` принадлежит
`claim_3`. Это не нарушает правило one-claim-per-evidence: сами claims
напрямую ссылаются только на evidence со своим `claim_id`.
Если relation опирается на `evidence_type = "inference"`, правило то же:
`evidence.claim_id` должен совпадать с `source_claim_id` или `target_claim_id`.

---

## Раздел 12: Что обязательно для всех packs

Каждый pack обязан возвращать результат, соответствующий следующим требованиям.

### Обязательные поля верхнего уровня

Все поля envelope обязательны как ключи. Массивы могут быть пустыми
только при условиях, явно зафиксированных ниже.

```
schema_version       — всегда "1.0" для v1
result_id            — уникален внутри run_id
parent_result_ids    — null или непустой массив ссылок на parent stage-results
run_id               — ссылка на выполнение pipeline
pack_id              — совпадает с namespace в outputs.pack_data
pack_version         — версия pack
stage                — значение из стандартного namespace (см. раздел stage namespace)
created_at           — ISO 8601
output_language      — BCP 47 language tag
metadata             — объект, все поля обязательны
run_context          — объект, все поля обязательны
outputs              — объект с обязательными блоками summary и sections
source_refs          — для stage = final_result: непустой при result_status = "complete";
                       для intermediate stages: может быть пустым, если соответствует
                       семантике stage и объяснено через outputs или quality_flags
claims               — те же правила, что source_refs
evidence             — пустой массив допустим при result_status = "error",
                       на intermediate stages, или если все claims имеют статус
                       inferred/unverified без фрагментов
claim_relations      — может быть пустым
unknowns             — может быть пустым
verification_tasks   — может быть пустым
warnings             — может быть пустым
limitations          — может быть пустым
quality_flags        — может быть пустым
audit_refs           — может быть пустым
```

### Обязательные правила `outputs`

- `outputs.summary` обязателен; `summary_text` непустой.
- `outputs.sections` обязателен как поле; непустой при `result_status = "complete"`.
- Все `claim_refs` из `outputs.summary` должны встречаться хотя бы
  в одном `outputs.sections[].items[].claim_refs`.
- Каждый значимый проверяемый вывод в `outputs` должен иметь хотя бы
  одну ссылку: `claim_refs`, `evidence_refs` или `source_refs`.

### Обязательные правила трассировки

- Каждый `claim` имеет хотя бы одну ссылку из `evidence_refs`, `source_refs`
  или `relation_refs`, кроме `claim_status = unverified` или `retracted`.
- Каждый `claim.claim_id` имеет формат `prefix_N`: непустой строковый префикс,
  финальный `_` и числовой суффикс. Этот формат нужен для детерминированного
  natural sort в симметричных `claim_relations`.
- Каждая `evidence` запись с `evidence_type = fragment` имеет
  непустые `source_ref_id` и `locator_data`.
- Каждая запись в `source_refs` имеет хотя бы одно заполненное
  из `canonical_url` или `internal_uri`.
- `claim.source_refs` является надмножеством source_refs,
  встречающихся в `evidence`, на которые ссылается `claim.evidence_refs`.

### Обязательные правила консистентности

- Если `metadata.contains_unverified_claims = true`,
  в `quality_flags` должна быть запись с `flag = "unverified_claims_present"`.
- Если `metadata.contains_partial_results = true`,
  в `quality_flags` должна быть запись с `flag = "partial_result"`.
- Если `metadata.result_status = "error"`,
  в `quality_flags` должна быть запись с `flag = "processing_failed"` или аналогом.
- Обратное не требуется: `quality_flags` может содержать локальные или контекстные
  флаги, не отражённые в `metadata` boolean-полях.
- `claim_relations` — источник истины для связей;
  `claim.relation_refs` — производное traversal-поле.
- `unknown.verification_task_refs` — производное traversal-поле;
  источник истины — `verification_tasks[].unknown_id`.

---

## Раздел 13: Что может быть pack-specific

### `outputs.pack_data`

Pack добавляет произвольные структуры в `outputs.pack_data[pack_id]`.
Общий контракт не валидирует содержимое этого блока;
pack-specific schema валидирует только свой namespace-блок.

Требование: если `pack_data` содержит проверяемые выводы,
они должны ссылаться на `claim_refs`, `evidence_refs` или `source_refs`
из верхнего уровня контракта.

### Нестандартные типы через escape-hatch

Pack может вводить собственные значения через `custom_*` поля везде,
где контракт предоставляет escape-hatch:
`section_type`, `claim_type`, `evidence_role`, `source_type`,
`warning_type`, `limitation_type`, `quality_flags.flag`,
`relation_type`, `fragment_type`, `unknown_type`, `task_type`, `event_type`.

Стандартные enum-значения не переопределяются и используются там,
где семантика совпадает.

### `run_context.control_preset` и `evidence_mode`

Допустимые значения определяются pack-спецификацией.
Общий контракт фиксирует только присутствие полей как snapshot-строк.

### Pack-specific stage-имена

Pack может использовать собственные stage-имена в `provenance` и `audit_refs`,
если они выходят за пределы стандартного namespace.
Рекомендуемый формат: `{pack_id}/{stage_name}`,
например `technology_watch/maturity_scoring`.

---

## Раздел 14 (дополнение): Стандартный `stage` namespace — OQ-03 закрыт

Поле `stage` используется в трёх местах контракта:
- `stage` в envelope (позиция result в pipeline)
- `stage` в `provenance` объектов (stage, породивший объект)
- `stage` в `audit_refs` (stage, к которому относится audit-событие)

Все три используют одно пространство имён.

**Стандартные значения v1:**

```
source_ingestion      — загрузка и индексация исходных материалов
source_analysis       — анализ отдельных источников
claim_extraction      — извлечение candidate claims из source-результатов
evidence_linking      — связывание evidence фрагментов с claims
claim_linking         — установление отношений между claims
contradiction_check   — проверка противоречий между claims
quality_check         — оценка качества, генерация unknowns и warnings
final_synthesis       — финальная агрегация и генерация outputs
final_result          — финальный result артефакт (значение для stage в envelope)
```

**Правила:**
- Значение `final_result` используется только в поле `stage` envelope,
  не в `provenance` или `audit_refs`.
- Pack-specific stage-имена используют формат `{pack_id}/{stage_name}`
  и не конфликтуют со стандартными значениями.
- В v1 namespace не является закрытым enum — валидация на уровне конвенции,
  не JSON Schema.

**OQ-03 закрыт.**

---

## Раздел 15: Открытые вопросы

### ~~OQ-01~~ — ЗАКРЫТ: `source_type_schemas.md`

`type_data` schemas для всех стандартных `source_type` v1 определены
в companion document `source_type_schemas.md` (schema_version 1.0).

**Итог:**
- `type_data` — строгий объект; расширения только через `extra_metadata`.
- Все семь стандартных типов покрыты: `youtube_video`, `web_page`, `rss_entry`,
  `telegram_post`, `telegram_channel_snapshot`, `telegram_chat_snapshot`, `forum_thread`.
- Общая обёртка: `schema_version`, `creator`, `parent_context`, `extra_metadata`.
- `creator` и `parent_context` — стандартные объекты с типизацией через
  `creator_type` / `context_type` enum + `custom` escape-hatch.
- Platform-specific timestamps живут в `type_data`; `published_at` / `accessed_at`
  остаются в `source_ref`.
- `source_type_schemas.md` версионируется вместе с `schema_version` контракта.

---

### ~~OQ-02~~ — ЗАКРЫТ: `fragment_locator_schemas.md`

`locator_data` schemas для всех стандартных `fragment_type` v1 определены
в companion document `fragment_locator_schemas.md` (schema_version 1.0).

**Итог:**
- `locator_data` — строгий объект; `extra_metadata` в locator v1 не вводится.
- Все стандартные fragment types покрыты: `text_range`, `paragraph`,
  `video_timestamp_range`, `audio_timestamp_range`, `post`, `comment`,
  `thread_reply`, `image_region`, `document_section`, `aggregate`.
- Text offsets используют Unicode codepoint offsets, 0-based,
  с диапазоном `[start,end)` — inclusive start, exclusive end.
- Индексы используют 0-based convention; page numbers остаются 1-based.
- Media timestamps задаются в секундах от начала media и используют
  inclusive/inclusive границы.
- Для `evidence_type = "inference"` используется `fragment_type = null`
  и `locator_data = null`.
- `comment` и `thread_reply` locator не создают отдельный `source_ref`;
  `evidence.source_ref_id` указывает на parent material.

---

### ~~OQ-04~~ — ЗАКРЫТ: DAG для multi-parent results

В v1 используется `parent_result_ids: null | string[]`.

Правила:
- `null` означает, что у result нет parent stage-result;
- если массив заполнен, он непустой и содержит уникальные `result_id`;
- один parent выражается массивом из одного элемента;
- несколько parent results образуют DAG lineage для `final_synthesis` и других
  aggregate stages.

Это поле введено сразу в v1, чтобы финальные результаты, агрегирующие несколько
stage-результатов, не требовали breaking change.

---

### OQ-05 — `control_preset` и `evidence_mode`: допустимые значения

Сейчас — snapshot-строки. Допустимые значения (`"standard"` и др.) определяются
настройками продукта и pack. Нужен отдельный документ настроек или раздел
в pack-спецификации с явным enum для каждого preset.

---

### OQ-06 — `*_comments` как самостоятельный source_type

`youtube_comments` и `telegram_comments` убраны из enum v1.
В v1 комментарии — это fragment/evidence layer внутри родительского материала
(`fragment_type = comment`).

Если понадобится агрегированный snapshot комментариев как самостоятельный material:
добавить новый стандартный `source_type` с обязательным `type_data.parent_material_id`.

---

### ~~OQ-07~~ — ЗАКРЫТ: `normalized_claim_text`: язык нормализации

В v1 язык нормализации зафиксирован как английский (`"en"`).

Правила:
- если `normalized_claim_text` заполнен, `normalized_claim_language = "en"`;
- если `normalized_claim_text = null`, `normalized_claim_language = null`;
- `claim_text` остаётся на языке результата или исходного материала, как задано
  pack-ом; нормализованная форма нужна для cross-run сравнения и дедупликации.

---

### OQ-08 — Entity нормализация в `claim.scope.applies_to`

`entity_id` опционален. Полноценный entity graph не входит в v1.
Если проект накапливает нормализованные entities — нужен отдельный слой
entity resolution поверх контракта.

---

### OQ-09 — Confidence: доверительные интервалы

`confidence.score` — скаляр. Для v2 может понадобиться `score_min` / `score_max`.
В v1 не открывать.

---

### OQ-10 — `task_status` для `verification_tasks`

В v1 не входит в контракт результата. Статус выполнения задачи живёт в
workflow-слое поверх immutable result. Если нужен — реализовать как
отдельный overlay-документ, ссылающийся на `verification_task_id`.

---

### ~~OQ-11~~ — ЗАКРЫТ: Консистентность `metadata` boolean-полей и `quality_flags`

Зафиксированные правила консистентности:
- `contains_unverified_claims = true` → `quality_flags` содержит `unverified_claims_present`
- `contains_partial_results = true` → `quality_flags` содержит `partial_result`
- `result_status = "partial"` → `quality_flags` содержит `partial_result`
- `result_status = "error"` → `quality_flags` содержит `processing_failed` или аналог
- Обратное не требуется: `quality_flags` может содержать локальные или контекстные
  флаги, не отражённые в `metadata` boolean-полях.

В v1 консистентность не enforced JSON Schema — проверяется на уровне pipeline.
Набор правил зафиксирован в `validation_rules.md` (`VR-CORE-042`–`VR-CORE-046`).
Executable validation module остаётся implementation task поверх документации.

---

### OQ-12 — `n`-арные `claim_relations`

В v1 поддерживаются только бинарные отношения (`source_claim_id` / `target_claim_id`).
`n`-арные отношения (один claim связан с группой) не поддерживаются.
Если понадобятся — добавить отдельное поле `participant_claim_ids: []` без breaking change,
с явным указанием, что `source_claim_id` и `target_claim_id` в этом случае могут быть null.

---

### ~~OQ-13~~ — ЗАКРЫТ: `youtube_playlist` как source контекст

Решено в рамках `source_type_schemas.md`:
playlist-контекст хранится в `type_data.playlist_id` / `playlist_title` / `playlist_position`
внутри `youtube_video` source_ref, не как самостоятельный `source_type`.
Правило: если `playlist_id = null`, то `playlist_title` и `playlist_position` тоже `null`.
