# Prompt Pack JSON Contract Decisions

Дата фиксации: 2026-06-06.

Документ фиксирует принятые решения по единому минимальному JSON-контракту для всех prompt packs в `Research Control Deck`.
Это спецификация для обсуждения, а не готовая JSON Schema. Задача документа - сохранить архитектурные решения, их обоснование и инварианты, чтобы дальше можно было проектировать schemas, templates и examples без потери контекста.

## 1. Цели и границы

Контракт разделяет:

- `outputs` - форму результата для чтения, UI и отчетов;
- `claims`, `evidence`, `source_refs` - проверяемость, доказательную трассировку и связи с материалами.

Обоснование:

- разные prompt packs должны возвращать разные аналитические формы;
- при этом любой значимый вывод должен быть трассируем до claims, evidence или source refs;
- readable output не должен становиться единственным источником истины.

Контракт описывает immutable результат конкретного pipeline/run stage. Workflow-состояния, например статус выполнения verification task, не входят в v1.

## 2. Верхнеуровневый envelope

Принята плоская структура верхнего уровня:

```json
{
  "schema_version": "1.0",
  "result_id": "result_...",
  "parent_result_ids": null,
  "run_id": "run_...",
  "pack_id": "technology_watch",
  "pack_version": "v1",
  "stage": "final_result",
  "created_at": "2026-06-06T12:00:00Z",
  "output_language": "ru",
  "metadata": {},
  "run_context": {},
  "outputs": {},
  "source_refs": [],
  "claims": [],
  "evidence": [],
  "claim_relations": [],
  "unknowns": [],
  "verification_tasks": [],
  "warnings": [],
  "limitations": [],
  "quality_flags": [],
  "audit_refs": []
}
```

Решения:

- `parent_result_ids` добавлен для связи stage-результатов. В v1 это `null | string[]`;
  массив используется для одного или нескольких parent results и сразу поддерживает
  DAG lineage для aggregate/final_synthesis stages.
- `output_language` означает язык результата, а не язык исходных материалов.
- `source_languages` живет в `run_context`.
- ID стабильны и уникальны внутри `run_id`. Для ссылок между runs нужно использовать `run_id + local_id`.
- `claim_relations` используется вместо отдельного `contradictions`.

Обоснование:

- плоские массивы упрощают traversal и валидацию;
- `stage` остается явным routing-полем;
- противоречие является отношением между claims, а не самостоятельным типом вывода.

## 3. Metadata и run context

`metadata` описывает сам артефакт результата.
`run_context` описывает условия исполнения.

```json
{
  "metadata": {
    "result_type": "pack_result",
    "result_status": "complete",
    "producer_type": "llm",
    "contains_partial_results": false,
    "contains_unverified_claims": true
  },
  "run_context": {
    "project_id": "project_...",
    "preflight_id": "preflight_...",
    "project_goal": "Оценить технологические тренды...",
    "run_goal": "Найти зрелые инструменты для внедрения в 2026 году",
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
      }
    ]
  }
}
```

Решения:

- `result_type` не кодирует позицию в pipeline. Позицию описывает `stage`.
- `completeness` не вводим, чтобы не дублировать `result_status`.
- `created_from_stage_results` не вводим, чтобы не открывать DAG.
- `model_selection` - массив, потому что stage-имена зависят от pack.
- `input_corpus` описывает выбранный корпус, а не только процитированные источники.
- Pack specs версионируются независимо от core `schema_version`, но каждый pack
  обязан явно объявлять compatible core `schema_version`. Если совместимость
  меняется, pack должен bump-нуть `pack_version` или добавить compatibility table.

Обоснование:

- `metadata` нужен для быстрого routing;
- `run_context` является snapshot условий preflight/run;
- полные технические детали остаются в audit log.

## 4. Outputs

Принята смешанная модель: минимальный общий readable слой плюс pack-specific namespace.

```json
{
  "outputs": {
    "summary": {
      "title": "Краткое резюме",
      "summary_text": "В корпусе заметны три зрелых технологических направления...",
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
  }
}
```

Решения:

- `claim_refs`, `evidence_refs`, `source_refs` всегда являются массивами ID.
- Все `claim_refs` из `outputs.summary` должны встречаться хотя бы в одном `outputs.sections[].items[].claim_refs`.
- Обратное не требуется.
- Дублирование ссылок между summary и items допустимо.
- `sections` обязателен как поле.
- Для `result_status = "complete"` `sections` должен быть непустым.
- Пустой `sections` допустим только при `result_status = "error"` или при явных `quality_flags`.
- `pack_data` опционален и namespace-ится по `pack_id`.
- Общий контракт не валидирует содержимое `pack_data`.
- Для `technology_watch` текущая pack-specific schema использует technology-centric
  форму `pack_data.technology_watch.technologies`.

`section_type` enum v1:

```text
summary
trends
assessment
risks
recommendations
custom
```

Обоснование:

- ссылки на уровне item дают достаточную точность без превращения prose в гипертекст;
- `pack_data` сохраняет гибкость packs;
- общие `summary` и `sections` нужны для UI и отчетов.

## 5. Source refs

`source_refs` - material-level ссылки на материалы, реально использованные или процитированные результатом. Это подмножество `run_context.input_corpus`, а не полный corpus.

```json
{
  "source_ref_id": "source_ref_1",
  "source_type": "youtube_video",
  "custom_source_type": null,
  "canonical_url": "https://www.youtube.com/watch?v=...",
  "internal_uri": "extractum://materials/material_...",
  "source_title": "Local AI Agents in Production",
  "source_id": "source_...",
  "material_id": "material_...",
  "snapshot_id": "snapshot_...",
  "published_at": "2026-04-12T10:00:00Z",
  "accessed_at": "2026-06-06T09:30:00Z",
  "access_status": "cached",
  "type_data": {
    "video_id": "...",
    "channel_id": "...",
    "channel_title": "Example Channel",
    "playlist_id": "...",
    "playlist_title": "AI Agents Course",
    "duration_seconds": 1840
  }
}
```

Обязательные поля:

```text
source_ref_id
source_type
access_status
canonical_url или internal_uri
```

`source_type` enum v1:

```text
youtube_video
web_page
rss_entry
telegram_post
telegram_channel_snapshot
telegram_chat_snapshot
forum_thread
custom
```

`access_status` enum v1:

```text
live
cached
unavailable
unknown
```

Решения:

- `canonical_url` - только публичная или внешне проверяемая ссылка.
- `internal_uri` - внутренний стабильный locator.
- Хотя бы одно из `canonical_url` или `internal_uri` должно быть заполнено.
- `source_id`, `material_id`, `snapshot_id` - nullable internal pipeline references.
- `published_at` nullable/optional. Фиктивные даты запрещены.
- `access_status` и `snapshot_id` независимы.
- `youtube_playlist`, `youtube_comments`, `telegram_comments` не являются самостоятельными `source_ref` в v1.
- Playlist хранится как контекст в `type_data` конкретного видео.
- Comments живут на уровне evidence fragments.
- Надежность источника не хранится глобально в `source_ref`.
- `type_data` standard schemas вынесены в отдельный `source_type_schemas.md`
  и версионируются вместе с `schema_version`.

Обоснование:

- material-level ссылка сохраняет точность;
- fragment-level точность принадлежит `evidence`;
- разделение `canonical_url` и `internal_uri` убирает двусмысленность внешней верифицируемости.

## 6. Claims

Claims - центральный слой утверждений.

```json
{
  "claim_id": "claim_1",
  "claim_type": "factual",
  "custom_claim_type": null,
  "claim_status": "verified",
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
  "relation_refs": ["rel_1"],
  "provenance": {
    "stage": "claim_extraction",
    "provider": "openai_compatible",
    "model": "gpt-4.1-mini",
    "audit_refs": ["audit_1"]
  }
}
```

`claim_type` enum v1:

```text
factual
evaluative
predictive
causal
normative
hypothesis
custom
```

`claim_status` enum v1:

```text
extracted
inferred
verified
partially_verified
unverified
retracted
custom
```

Решения:

- `claim_text` в v1 является свободным текстом.
- `claim_id` в v1 имеет формат `prefix_N`: непустой строковый префикс,
  финальный `_` и числовой суффикс. Этот формат нужен для детерминированного
  natural sort в `claim_relations` с `relation_type = "contradicts"`.
- `normalized_claim_text` опционален; если заполнен, `normalized_claim_language = "en"`.
  Если `normalized_claim_text = null`, `normalized_claim_language = null`.
- `contradicted` не является ручным `claim_status`; это вычисляется через `claim_relations`.
- `source_refs` в claim - денормализованное traversal-поле.
- `claim.source_refs` должен быть надмножеством всех `source_ref_id`, встречающихся в evidence, на которые ссылается claim.
- `relation_refs` - traversal-поле; источник истины - top-level `claim_relations`.
- `claim_status` отражает состояние на момент создания result. История восстанавливается через `parent_result_ids` и audit.
- `inferred` claim требует хотя бы одного из `evidence_refs`, `source_refs`, `relation_refs`.

Обоснование:

- свободный `claim_text` проще и надежнее для v1, чем subject/predicate нормализация;
- claim type нужен, потому что факты, оценки, прогнозы и причинные связи проверяются по-разному;
- provenance нужен для трассировки stage/model, породивших claim.

## 7. Confidence

`confidence` единый для claims, evidence и claim relations.

```json
{
  "score": 0.78,
  "basis": "multiple_corroborating_sources",
  "custom_basis": null,
  "method": "llm_assessment",
  "custom_method": null
}
```

Также допустимо:

```json
null
```

`basis` enum v1:

```text
multiple_corroborating_sources
single_source
strong_direct_evidence
weak_or_indirect_evidence
conflicting_sources
inference_chain
expert_consensus
llm_prior_only
no_evidence
custom
```

`method` enum v1:

```text
llm_assessment
rule_based
human_review
aggregated
inherited
custom
```

Решения:

- `confidence = null` означает, что уверенность не оценивалась.
- `score` находится в диапазоне `0.0..1.0`.
- `score` означает уверенность в корректности утверждения, доказательной роли или связи как они сформулированы.
- `score: 0.0` не означает автоматически "утверждение ложно".
- Ложность или опровержение выражаются через `claim_relations` и evidence roles.

Обоснование:

- объектная форма объясняет, откуда взялась оценка;
- nullable значение отличает "не оценивалось" от низкой уверенности;
- один universal enum проще для v1, чем отдельные confidence-типы.

## 8. Evidence

Одна запись `evidence` - это fragment + role относительно одного claim.

```json
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
  "fragment_text": "We moved local agents from lab demos into limited customer pilots...",
  "context_text": "Speaker describes the transition from internal experiments to pilots.",
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
```

`evidence_type` enum v1:

```text
fragment
inference
custom
```

`evidence_role` enum v1:

```text
supports
contradicts
qualifies
contextualizes
custom
```

`fragment_type` enum v1:

```text
text_range
paragraph
video_timestamp_range
audio_timestamp_range
post
comment
thread_reply
image_region
document_section
aggregate
custom
```

`text_mode` enum v1:

```text
verbatim
paraphrase
summary
description
```

Решения:

- evidence всегда относится к одному `claim_id`.
- один фрагмент может породить несколько evidence-записей для разных claims.
- `source_refs` в evidence не храним; для direct fragment используется `source_ref_id`.
- `aggregate` является `fragment_type`, а не `evidence_type`.
- `media` не является `evidence_type`; media выражается через `fragment_type`.
- `fragment_type` может быть `null` только при `evidence_type = inference`.
- `text_mode` может быть `null`, если текстовый слой отсутствует или неприменим.
- Для `evidence_type = fragment` поле `source_ref_id` обязательно.
- Для `evidence_type = inference` `source_ref_id` может быть `null`, но нужны `contributing_evidence_refs` или `reasoning_summary`.
- `contributing_evidence_refs` должен формировать ациклический граф.
- `fragment_text` обязателен при `text_mode = verbatim`.
- `context_text` без `fragment_text` допустим только для inference или media-фрагментов, где текстовый слой отсутствует.

Обоснование:

- role на уровне evidence делает доказательную функцию явной;
- небольшое дублирование фрагментов лучше, чем неявная роль;
- inference evidence нужно для claim-ов, полученных из цепочки рассуждения, а не одной цитаты.

## 9. Claim relations

`claim_relations` описывает направленные отношения между claims.

```json
{
  "relation_id": "rel_1",
  "relation_type": "qualifies",
  "custom_relation_type": null,
  "source_claim_id": "claim_2",
  "target_claim_id": "claim_1",
  "description": "claim_2 уточняет условия применимости claim_1.",
  "evidence_refs": ["evidence_1"],
  "confidence": {
    "score": 0.78,
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
```

`relation_type` enum v1:

```text
contradicts
supports
qualifies
supersedes
custom
```

Решения:

- используется `source_claim_id` / `target_claim_id`, а не unordered `claim_ids`.
- для `contradicts` порядок детерминирован: `source_claim_id` идёт раньше `target_claim_id` по natural sort `claim_id`;
  в v1 `claim_id` обязан иметь формат `prefix_N`, где общий префикс сравнивается лексикографически,
  а числовой суффикс после последнего `_` сравнивается как число (`claim_2` идёт раньше `claim_10`,
  `claim_abc_2` идёт раньше `claim_abc_10`);
- `evidence_refs` могут ссылаться только на evidence, чей `claim_id` совпадает с `source_claim_id` или `target_claim_id`; это относится и к `evidence_type = "inference"`, потому что evidence в v1 всегда принадлежит одному claim.
- Для `contradicts` relation-level `evidence_refs` обычно включает evidence обеих сторон отношения; это не нарушает one-claim-per-evidence, потому что каждый claim напрямую ссылается только на evidence со своим `claim_id`.
- `description` обязателен для `contradicts` и `qualifies`, опционален для `supports` и `supersedes`.
- `provenance` обязателен.
- `claim_relations` - источник истины; `claim.relation_refs` является производным traversal-полем.
- Directed graph по `relation_type = "qualifies"` и `relation_type = "supersedes"`
  должен быть ацикличным. Циклы `supports` в v1 считаются QA concern, не hard error.

Обоснование:

- направленная модель нужна для `supports`, `qualifies`, `supersedes`;
- natural sort конвенция предотвращает дубли для симметричных contradictions
  без ложных ошибок на ID вида `claim_2` / `claim_10`, при этом ID generation
  должен придерживаться формата с числовым суффиксом;
- relation-level provenance показывает, какая stage/model установила связь.

## 10. Unknowns и verification tasks

`unknowns` - констатация пробела в знании.
`verification_tasks` - предписание действия, чтобы закрыть пробел.

### Unknowns

```json
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
  "relation_refs": ["rel_1"],
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
```

`unknown_type` enum v1:

```text
missing_data
unresolvable
conflicting
out_of_scope
requires_expertise
custom
```

Решения:

- хотя бы один из `claim_refs`, `source_refs`, `evidence_refs`, `relation_refs` должен быть непустым.
- `unknown.confidence` означает уверенность pipeline в корректности идентификации пробела, а не истинность claim.
- `provenance` опционален, но желателен для LLM-generated unknowns.
- `verification_task_refs` - производное traversal-поле.
- источник истины для связи - `verification_tasks[].unknown_id`.

### Verification tasks

```json
{
  "verification_task_id": "verification_task_1",
  "task_type": "source_search",
  "custom_task_type": null,
  "priority": "high",
  "task": "Найти дополнительные источники, подтверждающие или опровергающие переход локальных LLM-агентов из пилотов в production.",
  "where_to_check": "Официальные changelog, кейсы внедрения, публичные customer stories, технические блоги компаний.",
  "expected_evidence_type": "Прямое подтверждение production-внедрения или явное указание, что речь идет только о пилотах.",
  "status_change_condition": "Если найдено два независимых подтверждения production-внедрения, claim можно повысить до verified.",
  "unknown_id": "unknown_1",
  "claim_refs": ["claim_1"],
  "source_refs": [],
  "evidence_refs": []
}
```

`task_type` enum v1:

```text
manual_check
source_search
expert_review
re_run_with_data
cross_reference
custom
```

`priority` enum v1:

```text
high
medium
low
```

Решения:

- `unknown_id` опционален.
- если `unknown_id = null`, хотя бы одно из `claim_refs`, `source_refs`, `evidence_refs` должно быть непустым.
- `task_status` не входит в v1, потому что это workflow-состояние поверх immutable result.
- `status_change_condition` - human-readable hint, не машиноисполняемое правило.

Обоснование:

- пробел и действие нельзя смешивать;
- status задач меняется после генерации результата и не должен превращать result contract в живой task tracker.

## 11. Warnings, limitations, quality flags

Семантическая граница:

- `warnings` - конкретные проблемы данного результата, которые могли повлиять на качество вывода;
- `limitations` - структурные ограничения задачи, корпуса или метода;
- `quality_flags` - машиночитаемые коды для routing, фильтрации и валидации.

Общие контекстные поля:

```text
claim_refs
source_refs
evidence_refs
relation_refs
section_refs
```

`severity` enum v1:

```text
low
medium
high
critical
```

### Warnings

```json
{
  "warning_id": "warning_1",
  "warning_type": "single_source_claim",
  "custom_warning_type": null,
  "severity": "medium",
  "message": "Один из ключевых claims основан только на одном источнике.",
  "claim_refs": ["claim_3"],
  "source_refs": ["source_ref_4"],
  "evidence_refs": ["evidence_7"],
  "relation_refs": [],
  "section_refs": ["section_technology_trends"]
}
```

`warning_type` enum v1:

```text
single_source_claim
low_confidence_output
unresolved_contradiction
missing_evidence
conflicting_sources
temporal_gap
access_unavailable
custom
```

### Limitations

```json
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
}
```

`limitation_type` enum v1:

```text
corpus_coverage_limited
temporal_scope_narrow
language_bias
methodology_constraint
out_of_scope_topic
data_freshness
custom
```

### Quality flags

```json
{
  "flag": "unverified_claims_present",
  "custom_flag": null,
  "severity": "medium",
  "message": "В результате есть claims со статусом unverified.",
  "claim_refs": ["claim_8"],
  "source_refs": [],
  "evidence_refs": [],
  "relation_refs": [],
  "section_refs": []
}
```

`quality_flags.flag` enum v1:

```text
insufficient_data
processing_failed
single_source_claim
low_confidence_result
unverified_claims_present
corpus_coverage_limited
conflicting_sources_unresolved
out_of_scope_content_present
partial_result
custom
```

Решения:

- enum-ы `warning_type`, `limitation_type` и `quality_flags.flag` разделены.
- смысловые пересечения допустимы, но это разные словари.
- если есть `warning_type = single_source_claim`, наличие `quality_flags.flag = single_source_claim` желательно, но не обязательно.
- `metadata` boolean-поля остаются быстрыми флагами, но должны быть консистентны с `quality_flags`.

Правила консистентности:

- если `metadata.contains_unverified_claims = true`, должен быть `quality_flags.flag = "unverified_claims_present"`;
- если `metadata.contains_partial_results = true`, должен быть `quality_flags.flag = "partial_result"`;
- если `metadata.result_status = "partial"`, должен быть `quality_flags.flag = "partial_result"`;
- если `metadata.result_status = "error"`, должен быть `quality_flags.flag = "processing_failed"` или другой более конкретный error-like flag.

Обоснование:

- warnings/limitations нужны человеку;
- quality flags нужны машине;
- общий enum создавал бы семантическое смешение и ошибки генерации.

## 12. Audit refs

`audit_refs` - lightweight inline pointers на внешние audit-записи.

```json
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
}
```

Обязательные поля:

```text
audit_id
event_type
stage
timestamp
```

`event_type` enum v1:

```text
stage_start
stage_end
model_call
validation
repair
error
human_review
system_event
custom
```

Решения:

- top-level `audit_refs` и `provenance.audit_refs` используют одну схему.
- top-level `audit_refs` относится ко всему result/pipeline.
- `provenance.audit_refs` относится к операции, породившей или проверившей конкретный объект.
- top-level `audit_refs` обязателен как поле, но может быть пустым массивом.
- `audit_uri` - предпочтительный внутренний locator в формате `extractum://audit/<store>/<audit_id>`.
- `audit_store` - human-readable идентификатор хранилища в контексте проекта, не универсальный locator.
- `object_refs` является backlink, не источником истины.
- `stage` использует то же пространство имен, что envelope `stage` и `provenance.stage`. В v1 это не закрытый enum.
- полные prompts, inputs/outputs, токены, стоимость, ошибки и repair-запросы остаются во внешнем audit log.

Обоснование:

- pointer-only был бы слишком непрозрачным без audit-системы;
- full inline audit раздул бы JSON;
- lightweight pointer дает навигацию и контекст без дублирования полного audit.

## 13. Минимальный пример результата

```json
{
  "schema_version": "1.0",
  "result_id": "result_1",
  "parent_result_ids": null,
  "run_id": "run_1",
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
    "contains_unverified_claims": false
  },
  "run_context": {
    "project_id": "project_1",
    "preflight_id": "preflight_1",
    "project_goal": "Оценить технологические тренды",
    "run_goal": "Найти зрелые инструменты для внедрения",
    "selected_pack": {
      "pack_id": "technology_watch",
      "pack_version": "v1"
    },
    "control_preset": "standard",
    "evidence_mode": "standard",
    "output_language": "ru",
    "source_languages": ["en"],
    "period": {
      "from": "2026-01-01",
      "to": "2026-06-06"
    },
    "input_corpus": {
      "source_types": ["youtube"],
      "selected_source_count": 1,
      "selected_material_count": 1,
      "selected_fragment_count": 3
    },
    "model_selection": [
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
      "summary_text": "Локальные LLM-агенты переходят из экспериментов в ограниченные пилоты.",
      "claim_refs": ["claim_1"],
      "evidence_refs": ["evidence_1"],
      "source_refs": ["source_ref_1"]
    },
    "sections": [
      {
        "section_id": "section_1",
        "title": "Технологическая зрелость",
        "section_type": "assessment",
        "custom_section_type": null,
        "items": [
          {
            "item_id": "item_1",
            "title": "Переход к пилотам",
            "text": "Источник описывает переход от демонстраций к ограниченным клиентским пилотам.",
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
      "canonical_url": "https://www.youtube.com/watch?v=example",
      "internal_uri": "extractum://materials/material_1",
      "source_title": "Local AI Agents in Production",
      "source_id": "source_1",
      "material_id": "material_1",
      "snapshot_id": "snapshot_1",
      "published_at": "2026-04-12T10:00:00Z",
      "accessed_at": "2026-06-06T09:30:00Z",
      "access_status": "cached",
      "type_data": {
        "video_id": "example",
        "channel_id": "channel_1",
        "channel_title": "Example Channel",
        "duration_seconds": 1840
      }
    }
  ],
  "claims": [
    {
      "claim_id": "claim_1",
      "claim_type": "factual",
      "custom_claim_type": null,
      "claim_status": "verified",
      "custom_claim_status": null,
      "claim_text": "Локальные LLM-агенты переходят из экспериментов в ограниченные пилоты.",
      "normalized_claim_text": null,
      "normalized_claim_language": null,
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
        "basis": "single_source",
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
        "model": "gpt-4.1",
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
      "fragment_text": "We moved local agents from lab demos into limited customer pilots.",
      "context_text": null,
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
        "model": "gpt-4.1",
        "audit_refs": ["audit_1"]
      }
    }
  ],
  "claim_relations": [],
  "unknowns": [],
  "verification_tasks": [],
  "warnings": [],
  "limitations": [],
  "quality_flags": [],
  "audit_refs": [
    {
      "audit_id": "audit_1",
      "audit_uri": "extractum://audit/project_audit_log/audit_1",
      "audit_store": "project_audit_log",
      "event_type": "model_call",
      "custom_event_type": null,
      "stage": "final_synthesis",
      "timestamp": "2026-06-06T09:40:00Z",
      "summary": "LLM generated the final Technology Watch result.",
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
    }
  ]
}
```

## 14. Открытые вопросы

Открытые темы для следующего этапа:

1. Решить, где физически хранить JSON Schemas: общий контракт, source type schemas, fragment locator schemas, pack-specific schemas.
2. Определить правила few-shot examples для каждого pack.
3. Определить структуру stage templates внутри prompt packs.
4. Определить рекомендации моделей по stage для каждого стартового pack.

Закрыто после первой версии decision log:

- `source_type_schemas.md` описывает `type_data` для стандартных `source_type`.
- `fragment_locator_schemas.md` описывает `locator_data` для стандартных `fragment_type`.
- `technology_watch_pack_spec.md` описывает первую pack-specific schema
  для `outputs.pack_data.technology_watch`.
- `validation_rules.md` классифицирует hard errors, warnings, QA и pipeline-level
  validation rules для core, companion и pack-specific документов.

## 15. Baseline v1

Prompt Pack JSON Contract v1 считается baseline-ready для первого pack при
следующем составе документов:

- `prompt_pack_json_contract_v1_draft.md`;
- `source_type_schemas.md`;
- `fragment_locator_schemas.md`;
- `schemas/README.md`;
- `validation_rules.md`;
- `validator_manifest.md`;
- `validator_fixtures.md`;
- `execution_model_graph_assembly_policy.md`;
- `stage_io_contracts.md`;
- `stage_prompt_templates.md`;
- `prompts/v1/README.md`;
- `parser-fixtures/v1/README.md`;
- `technology_watch_pack_spec.md`.

Decision logs:

- `PROMPT_PACK_JSON_CONTRACT_DECISIONS.md`;
- `SOURCE_TYPE_SCHEMA_DECISIONS.md`;
- `FRAGMENT_LOCATOR_SCHEMA_DECISIONS.md`;
- `TECHNOLOGY_WATCH_PACK_DECISIONS.md`.

Execution policy:

- `execution_model_graph_assembly_policy.md` fixes the runtime boundary between
  LLM-authored semantic content and pipeline-owned graph assembly.
- `validator_manifest.md` fixes reference-validator execution phases,
  required input artifacts, rule groups, blocking behavior, finding output, and
  CI fixture expectations.
- `validator_fixtures.md` fixes the first validator fixture catalog: directory
  layout, naming convention, required fixture matrix, expected findings, and
  fixture authoring rules.
- `schemas/README.md` closes machine-readable schema placement:
  JSON Schemas live under `docs/prompt-packs/schemas/v1/` and are indexed by
  `schemas/v1/schema_manifest.json`. The checked-in v1 bundle has reviewed
  semantic local-shape coverage for every schema:
  `core/result.schema.json`,
  `core/audit_ref.schema.json`, `core/confidence.schema.json`,
  `core/validation_finding.schema.json`,
  `source-types/youtube_video.schema.json`,
  `source-types/web_page.schema.json`,
  `source-types/rss_entry.schema.json`,
  `source-types/telegram_post.schema.json`,
  `source-types/telegram_channel_snapshot.schema.json`,
  `source-types/telegram_chat_snapshot.schema.json`,
  `source-types/forum_thread.schema.json`,
  `packs/technology_watch/pack_data.schema.json`,
  `packs/youtube_summary/pack_data.schema.json`,
  `fragment-locators/video_timestamp_range.schema.json`,
  `fragment-locators/audio_timestamp_range.schema.json`,
  `fragment-locators/text_range.schema.json`,
  `fragment-locators/image_region.schema.json`,
  `fragment-locators/document_section.schema.json`,
  `fragment-locators/post.schema.json`,
  `fragment-locators/comment.schema.json`,
  `fragment-locators/thread_reply.schema.json`, and
  `fragment-locators/aggregate.schema.json`, and
  `stage-io/source_ingestion.schema.json`, and
  `stage-io/fragment_candidate_mining.schema.json`, and
  `stage-io/claim_extraction.schema.json`, and
  `stage-io/canonical_evidence_generation.schema.json`, and
  `stage-io/claim_linking.schema.json`, and
  `stage-io/pack_data_generation.schema.json`, and
  `stage-io/final_synthesis.schema.json`, and
  `stage-io/retry_repair_payload.schema.json` are marked `semantic`.
  Prose specs and `validation_rules.md` remain authoritative for cross-object
  graph, pipeline, and QA semantics. Locator cross-field comparisons such as
  `timestamp_end >= timestamp_start`, and aggregate checks that require parent
  evidence access, remain pipeline/code checks.
  `core/result.schema.json` has semantic local-shape coverage for the canonical
  result envelope and core graph object shells: metadata, run context, readable
  outputs, material-level source refs, claims, evidence, claim relations,
  unknowns, verification tasks, warnings, limitations, quality flags, and audit
  refs. Cross-object referential integrity, traversal unions,
  metadata/quality-flag consistency, pack-specific `pack_data`, source
  `type_data`, and fragment `locator_data` semantics remain validator/pipeline
  or companion-schema checks.
  All ten v1 fragment locator schemas now have semantic local-shape coverage.
  `source-types/youtube_video.schema.json` is the first source type schema
  promoted to semantic local-shape coverage; it enforces the common
  `type_data` wrapper, YouTube video fields, playlist dependencies, and the
  free-string collection-status convention.
  `source-types/web_page.schema.json` also has semantic local-shape coverage;
  it enforces the common wrapper, keeps canonical URLs on `source_ref`, leaves
  page/extraction/comment status fields as free normalized strings, and supports
  `parent_context.context_type = null` for root pages.
  `source-types/rss_entry.schema.json` also has semantic local-shape coverage;
  it enforces the common wrapper, keeps feed-declared `entry_url` separate from
  `source_ref.canonical_url`, uses `parent_context.context_type = "rss_feed"`,
  and intentionally has no `collection_status` field in v1.
  `source-types/telegram_post.schema.json` also has semantic local-shape
  coverage; it enforces the common wrapper, Telegram post counters,
  discussion-layer fields, forwarded-message metadata, and
  `parent_context.context_type = "telegram_channel"`.
  `source-types/telegram_channel_snapshot.schema.json` also has semantic
  local-shape coverage; it enforces aggregate channel activity metrics,
  paired `snapshot_from` / `snapshot_to` fields, root `parent_context = null`,
  and the v1 omission of `avg_reactions_per_post`.
  `source-types/telegram_chat_snapshot.schema.json` also has semantic
  local-shape coverage; it enforces aggregate chat metrics, paired
  `snapshot_from` / `snapshot_to` fields, root `parent_context = null`, and
  `creator_type = "unknown"` for group/chat snapshots.
  `source-types/forum_thread.schema.json` also has semantic local-shape
  coverage; it enforces the compact cross-platform forum thread model,
  free-string `platform`, aggregate `vote_score`, participant/reply counters,
  and `parent_context.context_type = "forum" | "forum_category"`. All seven
  standard v1 source type schemas now have semantic local-shape coverage.
  `packs/technology_watch/pack_data.schema.json` also has semantic local-shape
  coverage; it enforces `technologies[]`, the `Technology` object, maturity,
  signals, tools, barriers, risks, recommendations, and enum/custom-field
  conventions. Pack traversal and strict-mode obligations remain
  validator/pipeline checks.
  `packs/youtube_summary/pack_data.schema.json` also has semantic local-shape
  coverage; it enforces `videos[]`, per-video segments, key points, notable
  quotes, action items, open questions, nullable/object synthesis, and
  cross-video synthesis object shapes. Source anchors, traversal unions, quote
  evidence authority, word-count equality, segment timestamp membership, and
  multi-video synthesis obligations remain validator/pipeline checks.
  `stage-io/source_ingestion.schema.json` also has semantic local-shape
  coverage; it enforces the ingestion input envelope with `raw_material_refs`
  and the pipeline-owned output `source_registry`. Full standard `type_data`
  validation remains delegated to source-type schemas, and source graph
  consistency remains a validator/pipeline check.
  `stage-io/fragment_candidate_mining.schema.json` also has semantic
  local-shape coverage; it enforces the pre-contract mining input with
  `source_registry` and `material_windows`, the LLM output
  `fragment_candidates`, and the pipeline output `fragment_registry`.
  Allowed-ID checks, full locator validation, candidate deduplication, and
  registry normalization remain validator/pipeline checks.
  `stage-io/claim_extraction.schema.json` also has semantic local-shape
  coverage; it enforces the closed-world input with
  `allowed_fragment_candidate_ids` and `fragment_registry`, and the LLM output
  with `claim_candidates`, `unknown_candidates`,
  `verification_task_candidates`, and optional `warnings`. Canonical IDs,
  allowed-ID enforcement, and final claim/evidence assembly remain
  validator/pipeline checks.
  `stage-io/canonical_evidence_generation.schema.json` also has semantic
  local-shape coverage; it enforces the pipeline-owned canonical assembly input
  with `claim_candidates` and `fragment_registry`, and the output with
  canonical `claims` and `evidence`. Evidence ownership, traversal rebuilding,
  source-ref superset rules, and fragment/inference consistency remain
  validator/pipeline checks.
  `stage-io/claim_linking.schema.json` also has semantic local-shape coverage;
  it enforces the closed-world relation-candidate input with
  `allowed_claim_ids`, `allowed_evidence_ids`, `claim_registry`, and optional
  `evidence_registry`, plus the LLM output `relation_candidates`. Relation ID
  assignment, allowed-ID enforcement, `contradicts` natural-sort normalization,
  and relation evidence ownership remain validator/pipeline checks.
  `stage-io/pack_data_generation.schema.json` also has semantic local-shape
  coverage; it enforces the pack-specific projection input with allowed
  claim/evidence/source IDs and immutable claim/evidence/source registries,
  plus the LLM output `pack_data_candidate`, `unknown_candidates`, and
  `warning_candidates`. Pack-specific object IDs, traversal rebuilding,
  allowed-ID enforcement, and full pack-specific schema validation remain
  validator/pipeline checks.
  `stage-io/final_synthesis.schema.json` also has semantic local-shape
  coverage; it enforces the readable-output synthesis input with allowed
  claim/evidence/source IDs, canonical claims, pack data, and optional graph
  registries, plus the LLM output `outputs_candidate.summary` and
  `outputs_candidate.sections`. Section/item ID assignment, summary claim
  coverage, metadata assembly, quality flags, warnings, limitations, and audit
  refs remain validator/pipeline checks.
  `stage-io/retry_repair_payload.schema.json` also has semantic local-shape
  coverage; it enforces compact repair prompts for retryable LLM stages with
  retry counters, retryable stage names, validation findings, failed object
  paths, and optional allowed-ID context arrays. The repaired response keeps
  using the original stage output schema, or an implementation-specific
  object-isolated replacement wrapper. All eight v1 stage I/O schemas now have
  semantic local-shape coverage.
- `stage_io_contracts.md` fixes internal stage payload boundaries, closed-world
  allowed-ID registries, retry/repair payloads, and stage output prohibitions.
- `stage_prompt_templates.md` fixes provider-neutral prompt skeletons, expected
  narrow JSON outputs, retry prompts, parser handoff, and few-shot rules for
  LLM stages.
- `prompts/v1/openai-compatible/fragment_candidate_mining.prompt.json`,
  `prompts/v1/openai-compatible/claim_extraction.prompt.json`,
  `prompts/v1/openai-compatible/claim_linking.prompt.json`, and
  `prompts/v1/openai-compatible/pack_data_generation.prompt.json`, and
  `prompts/v1/openai-compatible/final_synthesis.prompt.json` are the first
  checked-in provider-specific prompt renders for the baseline stage route.
  They are derived from
  `stage_prompt_templates.md` and do not add canonical result fields.
- `parser-fixtures/v1/fragment_candidate_mining/`,
  `parser-fixtures/v1/claim_extraction/`,
  `parser-fixtures/v1/claim_linking/`, and
  `parser-fixtures/v1/pack_data_generation/`, and
  `parser-fixtures/v1/final_synthesis/` contain the first raw provider-response
  parser fixtures for the baseline stage route. They document parser behavior
  and make it executable before parsed `stage_output` validation, while
  remaining separate from the mandatory validator fixture manifest baseline.
- `stage_io_version` is an execution payload version and is separate from the
  canonical result `schema_version`.
- Large stage registries may be passed through internal registry URIs, while
  `allowed_*_ids` arrays remain the authoritative closed-world boundary.
- Stage repair is object-isolated when possible: valid candidates continue
  forward, while invalid candidates are quarantined or retried.
- LLM stages select from immutable registries; pipeline owns final IDs, derived
  traversal refs, healing, quarantine, audit, and fragment deduplication.
- Pre-contract fragment registry is internal to the pipeline. Canonical
  `evidence[]` is created only after final `claim_id` assignment.
- Fragment candidate minimum shape is fixed for pipeline handoff:
  `candidate_id`, `source_ref_id`, `fragment_type`, `locator_data`, and at
  least one of `fragment_text` or `observation_summary`.
- Fragment deduplication follows "false merge is worse than false split";
  overlapping locators are not merged unless they represent the same semantic
  observation.
- Healing and quarantine audit events use `event_type = "custom"` with
  `custom_event_type = "healing"` or `"quarantine"`, so the core audit enum
  does not need to expand in v1.
- Quarantine artifacts live outside canonical JSON and are referenced through
  `audit_uri`; canonical JSON preserves the failure through audit pointers,
  warnings, quality flags, and metadata status.
- Canonical JSON remains compact: full healing diffs, raw invalid objects,
  prompts, model traces, and quarantine dumps live in external stores.
- Pipeline health metrics are tracked outside core `metadata` in v1. They may be
  summarized through `audit_refs` with `custom_event_type = "pipeline_health"`;
  a future `metadata.pipeline_health` would require a schema-versioned object.
- Final graph validation runs after healing and before final audit. It includes
  referential integrity, DAG checks for `parent_result_ids`, acyclicity checks
  for evidence and relation graphs, derived traversal recomputation, and
  pack-specific graph rules.

Проверенный pack-specific пример:

- использует technology-centric `pack_data.technology_watch.technologies`;
- использует полную `youtube_video.type_data` форму из `source_type_schemas.md`;
- использует `video_timestamp_range.locator_data` из `fragment_locator_schemas.md`;
- соблюдает правило `evidence.claim_id`: один evidence объект принадлежит одному claim;
- содержит `quality_flags` для `partially_verified` и single-source claim.

## 16. Итог

Принятый контракт строится вокруг одного принципа:

```text
outputs описывает форму результата,
claims/evidence/source_refs/claim_relations описывают проверяемость,
а audit_refs связывает результат с историей выполнения.
```

Это сохраняет гибкость prompt packs, но не позволяет значимым выводам отрываться от доказательной базы.
