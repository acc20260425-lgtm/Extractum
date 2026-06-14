# Technology Watch Pack Decisions

Дата фиксации: 2026-06-07.

Документ фиксирует принятые решения по `technology_watch_pack_spec.md`.
Это decision log к pack-specific schema для `outputs.pack_data.technology_watch`.

## 1. Назначение и границы

`technology_watch` предназначен для мониторинга технологических трендов,
инструментов, подходов и сигналов зрелости по корпусу источников.

Pack отвечает на вопросы:

- какие технологии или инструменты появляются и набирают зрелость;
- на каком уровне готовности они находятся;
- какие барьеры и риски мешают внедрению;
- что стоит оценить, пилотировать, внедрять, избегать или выводить из использования.

Не входит в pack:

- анализ конкретных людей или организаций;
- глубокий технический аудит реализаций;
- финансовое или рыночное прогнозирование.

## 2. Две оси `run_context`

Принято разделение:

- `control_preset` управляет шириной анализа;
- `evidence_mode` управляет строгостью доказательной базы.

`control_preset`:

- `quick` — summary, ключевые technologies, maturity;
- `standard` — полный базовый режим;
- `deep` — расширенный охват alternatives, weak signals, barriers, risks,
  tools и unknowns.

`deep` является behavioral guidance для prompt-а pack-а и не добавляет
структурных обязательств сверх `standard`.

`evidence_mode`:

- `narrative_only` — readable result first, structured `technologies` может быть пустым;
- `standard` — structured `technologies` обязателен при `result_status = complete`;
- `strict` — добавляет pipeline-level проверки evidence coverage.

В `quick` mode `technology.signals: []` допустим: maturity трассируется напрямую
через `maturity.claim_refs`.

## 3. Technology-centric `pack_data`

Принята technology-centric модель:

```json
{
  "pack_data": {
    "technology_watch": {
      "technologies": []
    }
  }
}
```

Signal-centric sibling-массивы не используются на верхнем уровне `pack_data`
в v1. Соответствующие аналитические элементы живут внутри конкретной технологии.

Обоснование:

- аналитик принимает решение по технологии, а не по разрозненным спискам;
- maturity, tools, barriers, risks и recommendations должны сохранять общий контекст;
- traversal через `Technology` проще для UI и последующей валидации.

## 4. Technology object

Ключевые поля:

- `technology_id`;
- `name`;
- `normalized_name`;
- `maturity`;
- `signals`;
- `tools`;
- `adoption_barriers`;
- `risks`;
- `recommendations`;
- traversal-поля `claim_refs`, `evidence_refs`, `source_refs`.

`technology_id` уникален внутри `run_id`.

`normalized_name` — best-effort slug для cross-run сравнения:
lowercase ASCII, слова через `_`, без пробелов и специальных символов.
Это не authoritative identity.

Traversal-правила:

- `technology.claim_refs` — union всех `claim_refs` из вложенных объектов;
- `technology.evidence_refs` — union `evidence_refs` из `signals`;
- `technology.source_refs` — union явных `source_refs` из `signals` и `tools`.

Синтезированные объекты не пишут прямые `evidence_refs` на уровне технологии:
они трассируются через свои `claim_refs`.

## 5. Signals, maturity и synthesis

Принято разделение слоёв:

- `signals` — наблюдения из корпуса;
- `maturity` — синтезированная оценка зрелости;
- `adoption_barriers`, `risks`, `recommendations` — synthesized/actionable слой.

`signals` описывают то, что наблюдаемо в source/evidence.

`maturity` не является raw signal. Это синтезированная оценка, которая опирается
на `maturity.claim_refs`.

`adoption_barriers`, `risks`, `recommendations` трассируются через `claim_refs`,
а не напрямую через `signal_refs` в v1.

## 6. Maturity model

Принята практическая шкала зрелости:

```text
experiment
pilot
limited_production
production
deprecated
unknown
```

Обоснование:

- TRL избыточен для software/AI/tool monitoring;
- enum напрямую отвечает на вопрос "что можно делать сейчас";
- `unknown` не подменяет отсутствие данных оценкой;
- `deprecated` позволяет фиксировать уходящие технологии и инструменты.

`tool_maturity` использует тот же enum, но применяется к конкретному инструменту
независимо от maturity родительской технологии.

## 7. ID scope

Вложенные `_id` уникальны только внутри соответствующего массива одной
`Technology`:

- `signal_id` внутри `technology.signals`;
- `tool_id` внутри `technology.tools`;
- `barrier_id` внутри `technology.adoption_barriers`;
- `risk_id` внутри `technology.risks`;
- `recommendation_id` внутри `technology.recommendations`.

Глобальная дедупликация tools между технологиями не входит в контракт v1.

## 8. Sections и obligations

Если `pack_data.technology_watch.technologies` непустой, `assessment` section
должен присутствовать и содержать maturity item.

Отдельной `section_type = barriers` в v1 нет. Барьеры отражаются в `assessment`
или `trends`.

Hard obligations при `result_status = complete`:

- `outputs.sections` непустой;
- `technologies` непустой при `evidence_mode != narrative_only`;
- каждая technology имеет `maturity.level`;
- maturity с `level != unknown` имеет `maturity.claim_refs`;
- top-level `claims` и `source_refs` непустые.

Soft obligations:

- `unknown` для `maturity.level = unknown`;
- `unknown` для `partially_verified` claims;
- `recommendation` для `production` или `limited_production`;
- `avoid` или `deprecate` recommendation для `deprecated`;
- warning для maturity confidence ниже 0.5.

## 9. Pack-specific stages

Pack-specific stages v1:

- `technology_watch/trend_extraction`;
- `technology_watch/maturity_scoring`;
- `technology_watch/adoption_analysis`;
- `technology_watch/tool_identification`.

Все stages опциональны. Зависимости между ними не фиксируются в контракте v1.

## 10. JSON example decisions

Минимальный JSON-пример в `technology_watch_pack_spec.md` проверен как
валидный JSON и согласован с архитектурой общего контракта:

- `contains_unverified_claims = true`, потому что есть `partially_verified` claim;
- присутствуют `quality_flags`: `unverified_claims_present`, `single_source_claim`;
- `claim_1`, `claim_2`, `claim_3` имеют отдельные evidence-записи;
- `evidence.claim_id` не используется как shared evidence для нескольких claims;
- `unknown_1` фиксирует пробел по partially verified barrier claim.

## 11. Принятые решения

### SD-TW-01 — `entity_type` recommended values

`technology_watch` рекомендует pack-local значения:

```text
technology
tool
framework
protocol
platform
concept
component
method
```

Это не строгий enum: неизвестные значения не отклоняются.

### SD-TW-02 — `deprecated` maturity рекомендует `avoid` или `deprecate`

Если `technology.maturity.level = deprecated`, pack по возможности создаёт
recommendation с `recommendation_type = "avoid"` или `"deprecate"`.

Правило мягкое, не hard validation.

## 12. Открытые вопросы

Открытые вопросы не блокируют v1:

- OQ-TW-02 — нужны ли `signal_refs` в barriers/risks;
- OQ-TW-03 — нужны ли `technology_refs` в `sections.items`.

В v1 достаточно traversal через `claim_refs`.

## 13. Итог

`technology_watch_pack_spec.md` является первой завершённой pack-specific schema
для Prompt Pack JSON Contract v1 baseline.
