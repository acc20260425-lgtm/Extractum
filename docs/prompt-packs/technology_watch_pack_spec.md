# technology_watch Pack Spec — v1

Версионируется независимо от `schema_version` основного контракта.
Текущая версия pack: `v1`.
Совместимость: Prompt Pack JSON Contract `schema_version: "1.0"`.

Если pack spec обновляется под другой `schema_version` core contract, pack должен
либо bump-нуть `pack_version`, либо добавить явную compatibility table. Один
`pack_version` не должен молча означать разные core-contract shapes.

---

## 1. Назначение и границы

`technology_watch` — pack для мониторинга и оценки технологических трендов
по корпусу источников: YouTube, web, RSS, Telegram, форумы.

**Отвечает на вопросы:**
- Какие технологии / инструменты / подходы появляются или набирают зрелость?
- На каком уровне готовности они находятся сейчас?
- Какие барьеры мешают внедрению?
- Что стоит оценить, пилотировать или внедрять прямо сейчас?

**Не входит в задачу:**
- Анализ конкретных людей или организаций (это OSINT pack).
- Глубокий технический аудит реализаций (это Engineering Analysis pack).
- Прогнозирование рынка или финансовых показателей.

---

## 2. `run_context` настройки

### `control_preset`

`control_preset` управляет шириной анализа: какие аспекты pack пытается покрыть.
Он не задаёт строгость доказательной базы — это зона `evidence_mode`.

| Значение | Поведение |
|---|---|
| `quick` | Минимальный Technology Watch: summary, ключевые technologies, maturity |
| `standard` | Базовый полный режим: technologies, maturity, signals, tools, adoption_barriers, risks, recommendations where applicable |
| `deep` | Как standard + расширенный охват alternatives, weak signals, barriers, risks, tools и unknowns |

Правила:
- `quick` не требует `signals`, `tools`, `adoption_barriers`, `risks` или `recommendations`.
- В `quick` mode `technology.signals: []` допустим; maturity трассируется напрямую через
  `maturity.claim_refs`.
- `deep` — behavioral guidance для prompt-а pack-а: он расширяет охват анализа,
  но не добавляет структурных обязательств сверх `standard`.

### `evidence_mode`

`evidence_mode` управляет строгостью доказательной базы: какие traceability
и evidence требования считаются hard или QA-level правилами.

| Значение | Поведение |
|---|---|
| `narrative_only` | Readable result first. `sections` обязательны; `pack_data.technology_watch.technologies` может быть пустым |
| `standard` | Structured result required. `technologies` обязателен при `result_status = complete` |
| `strict` | Как standard + усиленные pipeline-level проверки evidence coverage |

Правила:
- В `narrative_only`, если `technologies` пустой при `result_status = complete`,
  нужен `quality_flag` с `flag = "partial_result"` или `flag = "corpus_coverage_limited"`.
- В `standard`, каждый проверяемый вложенный объект должен иметь ссылки по правилам
  своей схемы (`claim_refs`, `evidence_refs` или `source_refs`).
- В `strict`, `technology.maturity.level != unknown` требует минимум 2 независимых
  `claim_refs`.
- В `strict`, `maturity.level = limited_production` или `production` требует coverage
  минимум из 2 уникальных `source_refs`, полученных как union по
  `maturity.claim_refs → claim.source_refs`.
- В `strict`, `single_source_claim` для maturity не запрещён, но должен породить
  `warning` с `warning_type = "single_source_claim"`.
- В `strict`, recommendations типа `adopt` или `deprecate` требуют минимум 2 `claim_refs`.

Определения:
- Два `claim_refs` считаются независимыми, если соответствующие claims не имеют
  пересекающихся `source_refs`.
- Все правила, которые требуют обхода `maturity.claim_refs → claim.source_refs`,
  являются pipeline-level validation, не JSON Schema rules.

### Сочетания `control_preset` и `evidence_mode`

Все комбинации `control_preset × evidence_mode` допустимы. Оси независимы.

| Комбинация | Значение |
|---|---|
| `quick + strict` | Узкий отчёт со строгой доказательной базой |
| `deep + narrative_only` | Широкий prose-отчёт без обязательного structured `pack_data` |
| `deep + strict` | Максимальная ширина анализа и максимальная строгость traceability |

### Ожидаемые `source_types`

Рекомендованные: `youtube_video`, `web_page`, `rss_entry`, `forum_thread`.
Дополнительные: `telegram_post`, `telegram_channel_snapshot`.
Pack не требует конкретного набора — анализирует то, что доступно в корпусе.

---

## 3. `outputs.pack_data.technology_watch`

### Верхний уровень

```json
"pack_data": {
  "technology_watch": {
    "technologies": []
  }
}
```

При `evidence_mode = narrative_only` или `result_status = error`:
`"technologies": []` допустим; должен сопровождаться `quality_flag`.

### Объект `Technology`

```json
{
  "technology_id": "tech_local_llm_agents",
  "name": "Local LLM agents",
  "normalized_name": "local_llm_agents",
  "maturity": {},
  "signals": [],
  "tools": [],
  "adoption_barriers": [],
  "risks": [],
  "recommendations": [],
  "claim_refs": [],
  "evidence_refs": [],
  "source_refs": []
}
```

`technology_id` уникален внутри `run_id`.
`normalized_name` — slug для cross-run сравнений; опционален.
Рекомендуемая форма `normalized_name`: lowercase ASCII, слова разделены `_`,
без пробелов и специальных символов. Это best-effort поле, не authoritative
entity identity.

Структура `maturity` описана в разделе 4.1. Остальные вложенные объекты
описаны в разделах 4.2–4.6.

`claim_refs`, `evidence_refs`, `source_refs` — денормализованные traversal-поля:
- `claim_refs` — union всех `claim_refs` из `maturity`, `signals`, `tools`,
  `adoption_barriers`, `risks`, `recommendations`;
- `evidence_refs` — union всех `evidence_refs` из `signals` этой технологии;
- `source_refs` — union всех явных `source_refs` из `signals` и `tools`.

Синтезированные объекты (`adoption_barriers`, `risks`, `recommendations`)
не пишут прямые `evidence_refs` в technology-level traversal: они трассируются
через свои `claim_refs`.

---

## 3.5 Signals vs synthesized objects: граница слоёв

`technology_watch` разделяет аналитические объекты на два слоя:

**Слой наблюдений** — `signals`:
- Что именно сказано в корпусе.
- Прямая связь с конкретным evidence и source.
- Не содержит синтетических выводов — только то, что наблюдаемо.
- Пример: «Компания X описывает переход от internal demo к customer pilot».

**Слой синтеза** — `adoption_barriers`, `risks`, `recommendations`:
- Вывод, построенный из нескольких наблюдений.
- Трассируется через `claim_refs`, не напрямую через `evidence_refs`.
- Может обобщать паттерн из нескольких signals без ссылки на конкретный.
- Пример: «Отсутствие стандартизованного интерфейса — барьер внедрения»
  (синтез из нескольких signals про vendor lock-in и несовместимость API).

**Maturity** — синтезированная оценка зрелости технологии:
- Стоит между слоями наблюдений и синтезированных actionable-выводов.
- Не является raw signal.
- Опирается на `maturity.claim_refs`, которые трассируются через claims
  к evidence и source.
- Может использовать signals как аналитическое основание, но не требует
  прямой ссылки `maturity → signal` в v1.

**Следствие для трассировки:**
- Если нужно найти, откуда взялся вывод — путь: `barrier.claim_refs` →
  `claim.evidence_refs` → `evidence.locator_data` → `source_ref`.
- Прямой путь `barrier → signal` не предусмотрен в v1.
  Если этот traversal окажется нужным на практике — добавить
  `signal_refs: []` в barrier/risk как опциональное traversal-поле.

## 4. Вложенные объекты `Technology`

### ID scope для вложенных объектов

Все `_id` внутри `Technology` уникальны только в пределах своего массива:

- `signal_id` уникален внутри `technology.signals`;
- `tool_id` уникален внутри `technology.tools`;
- `barrier_id` уникален внутри `technology.adoption_barriers`;
- `risk_id` уникален внутри `technology.risks`;
- `recommendation_id` уникален внутри `technology.recommendations`.

Глобальная уникальность этих ID внутри `run_id` не требуется. Один и тот же
инструмент может появиться в нескольких технологиях с одинаковым или разным
`tool_id`; контракт не выполняет cross-technology дедупликацию tools.

### 4.1 `maturity`

```json
{
  "level": "pilot",
  "confidence": {
    "score": 0.78,
    "basis": "multiple_corroborating_sources",
    "custom_basis": null,
    "method": "llm_assessment",
    "custom_method": null
  },
  "rationale": "Несколько источников описывают переход от прототипов к ограниченным пилотам.",
  "claim_refs": ["claim_1", "claim_2"]
}
```

`level` enum v1:
```
experiment | pilot | limited_production | production | deprecated | unknown
```

Правила:
- `maturity.claim_refs` непустой при любом `level`, кроме `unknown`.
- При `level = unknown` обязателен соответствующий `unknown` объект
  на верхнем уровне результата с `claim_refs` этой технологии.
- В `strict` mode: `level` (кроме `unknown`) требует минимум 2 `claim_refs`.

### 4.2 `signals`

Наблюдаемые сигналы из корпуса — сырой аналитический слой перед синтезом.
Каждый сигнал — конкретное наблюдение, которое вносит вклад в оценку технологии.

```json
{
  "signal_id": "signal_1",
  "signal_type": "adoption_evidence",
  "custom_signal_type": null,
  "description": "Компания X описывает переход от internal demo к customer pilot.",
  "claim_refs": ["claim_1"],
  "evidence_refs": ["evidence_1"],
  "source_refs": ["source_ref_1"]
}
```

`signal_type` enum v1:
```
adoption_evidence    — наблюдение об использовании/внедрении
maturity_indicator   — сигнал уровня зрелости
community_activity   — активность сообщества (обсуждения, pull requests, issues)
vendor_movement      — движение вендоров (релизы, партнёрства, прекращение поддержки)
research_publication — академические или исследовательские публикации
tool_release         — выход конкретного инструмента или версии
custom
```

Правило: каждый `signal` должен иметь хотя бы одну ссылку из
`claim_refs`, `evidence_refs` или `source_refs`.

`signal_type = tool_release` и объект в `tools` находятся на разных слоях:
signal описывает наблюдаемое событие из корпуса, а `tools` — структурированную
запись об инструменте в контексте технологии. Один релиз может породить и
signal, и запись/обновление в `tools`.

### 4.3 `tools`

Конкретные инструменты, библиотеки, платформы, упоминаемые в контексте технологии.

```json
{
  "tool_id": "tool_ollama",
  "name": "Ollama",
  "canonical_url": "https://ollama.ai",
  "description": "Local model runner for desktop and server deployment.",
  "tool_maturity": "production",
  "claim_refs": ["claim_5"],
  "source_refs": ["source_ref_2"]
}
```

`tool_maturity` использует тот же enum, что `technology.maturity.level`.
Семантика применяется к конкретному инструменту независимо от maturity
родительской технологии: технология может быть `pilot`, а отдельный инструмент
внутри неё — `production` или `deprecated`.
Правило: хотя бы одно из `claim_refs` или `source_refs` непустое.

### 4.4 `adoption_barriers`

Синтезированные барьеры, мешающие внедрению технологии.

```json
{
  "barrier_id": "barrier_1",
  "barrier_type": "technical",
  "custom_barrier_type": null,
  "description": "Отсутствие стандартизованного интерфейса для локальных моделей.",
  "severity": "high",
  "claim_refs": ["claim_3"]
}
```

`barrier_type` enum v1:
```
technical | organizational | cost | regulatory | knowledge | ecosystem | custom
```

`severity` enum: `high | medium | low`.
Правило: `claim_refs` непустой.

Барьер описывает то, что уже мешает внедрению или ограничивает применимость
технологии сейчас. Например, `barrier_type = ecosystem` означает нехватку
интеграций, инструментов, документации или совместимых компонентов.

### 4.5 `risks`

Риски, связанные с технологией или её внедрением.

```json
{
  "risk_id": "risk_1",
  "risk_type": "security",
  "custom_risk_type": null,
  "description": "Локальные модели могут использоваться для генерации вредоносного контента без API-контроля.",
  "severity": "medium",
  "claim_refs": ["claim_4"]
}
```

`risk_type` enum v1:
```
technical | security | adoption | strategic | regulatory | vendor_dependency | custom
```

Правило: `claim_refs` непустой.

Риск описывает то, что может пойти не так в процессе внедрения или после него.
Например, `risk_type = adoption` — риск низкого принятия пользователями или
командами, даже если текущие барьеры частично сняты.

### 4.6 `recommendations`

Синтезированные рекомендации по отношению к технологии.

```json
{
  "recommendation_id": "rec_1",
  "recommendation_type": "pilot",
  "custom_recommendation_type": null,
  "description": "Запустить внутренний пилот на одном use case с явными критериями успеха.",
  "priority": "high",
  "target_audience": "engineering_teams",
  "claim_refs": ["claim_1", "claim_2"]
}
```

`recommendation_type` enum v1:
```
evaluate | pilot | adopt | monitor | avoid | deprecate | custom
```

`priority` enum: `high | medium | low`.
`target_audience` — опциональная свободная строка.
Правило: `claim_refs` непустой.

---

## 5. `outputs.sections` — паттерн для `technology_watch`

| `section_type` | Когда присутствует | Типичное содержимое |
|---|---|---|
| `trends` | Всегда при `result_status = complete` | Ключевые тренды корпуса |
| `assessment` | При наличии ≥1 `technology` | Maturity по технологиям |
| `risks` | При наличии risks у ≥1 технологии | Сводка рисков |
| `recommendations` | При `control_preset ≠ quick` | Приоритетные рекомендации |

Если `pack_data.technology_watch.technologies` непустой, `assessment` section
должен присутствовать и содержать хотя бы один item, отражающий maturity-оценку.

Отдельной `section_type = barriers` в v1 нет. Барьеры отражаются в `assessment`
items или, если они важны для общей динамики корпуса, в отдельных `trends` items.

Правило трассировки: каждый `item` в `sections` ссылается на `claim_refs`
из объектов `pack_data.technology_watch.technologies`.
Обратное не обязательно: технология может существовать в `pack_data`
без отдельного `sections` item (если она упомянута, но не достаточно значима
для readable output).

---

## 6. Обязательства pack при `result_status = complete`

### Жёсткие (hard)

- `outputs.sections` непустой.
- `pack_data.technology_watch.technologies` непустой при `evidence_mode ≠ narrative_only`.
- Каждая `technology` имеет заполненный `maturity.level`.
- Каждая `technology` с `maturity.level ≠ unknown` имеет непустой `maturity.claim_refs`.
- Верхнеуровневый `claims` непустой.
- Верхнеуровневый `source_refs` непустой.

### Мягкие (желательно)

- Если `maturity.level` любой технологии `unknown` — присутствует соответствующий `unknown`.
- Если есть `partially_verified` claims — присутствует соответствующий `unknown`.
- Если `maturity.level = production` или `limited_production` — присутствует
  хотя бы одна `recommendation`.
- Если `maturity.level = deprecated` — рекомендуется `recommendation_type = "avoid"`
  или `recommendation_type = "deprecate"`.
- Если `confidence.score < 0.5` у maturity — присутствует `warning` с `warning_type = low_confidence_output`.

---

## 7. Pack-specific stages

Все опциональны. Зависимости между ними не фиксируются в контракте.

| Stage | Назначение |
|---|---|
| `technology_watch/trend_extraction` | Извлечение технологических трендов из source-результатов |
| `technology_watch/maturity_scoring` | Оценка уровня зрелости каждой технологии |
| `technology_watch/adoption_analysis` | Анализ барьеров, рисков, сигналов внедрения |
| `technology_watch/tool_identification` | Идентификация конкретных инструментов |

Используются в `provenance.stage` и `audit_refs[].stage`.

---

## 8. Минимальный JSON-пример

Одна технология, один сигнал, один инструмент, один барьер, одна рекомендация.
Сокращённые claim/evidence/source объекты показывают только ключевые поля.

```json
{
  "schema_version": "1.0",
  "result_id": "result_tw_001",
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
    "project_goal": "Оценить готовность локальных LLM-агентов к внедрению.",
    "run_goal": "Найти зрелые инструменты для пилота в 2026 году.",
    "selected_pack": { "pack_id": "technology_watch", "pack_version": "v1" },
    "control_preset": "standard",
    "evidence_mode": "standard",
    "output_language": "ru",
    "source_languages": ["en", "ru"],
    "period": { "from": "2026-01-01", "to": "2026-06-06" },
    "input_corpus": {
      "source_types": ["youtube_video", "web_page", "rss_entry"],
      "selected_source_count": 8,
      "selected_material_count": 42,
      "selected_fragment_count": 610
    },
    "model_selection": [
      { "stage": "technology_watch/trend_extraction", "provider": "openai_compatible", "model": "gpt-4.1-mini" },
      { "stage": "technology_watch/maturity_scoring", "provider": "openai_compatible", "model": "gpt-4.1" }
    ]
  },

  "outputs": {
    "summary": {
      "title": "Локальные LLM-агенты: переход в пилоты",
      "summary_text": "Корпус показывает устойчивый переход локальных LLM-агентов из экспериментов в ограниченные пилоты. Основной барьер — отсутствие стандартизованного интерфейса.",
      "claim_refs": ["claim_1", "claim_3"],
      "evidence_refs": ["evidence_1", "evidence_3"],
      "source_refs": ["source_ref_1"]
    },
    "sections": [
      {
        "section_id": "section_trends",
        "title": "Технологические тренды",
        "section_type": "trends",
        "custom_section_type": null,
        "items": [
          {
            "item_id": "item_1",
            "title": "Локальные LLM-агенты переходят в пилоты",
            "text": "Несколько источников фиксируют переход от прототипов к ограниченному внедрению у корпоративных клиентов.",
            "claim_refs": ["claim_1"],
            "evidence_refs": ["evidence_1"],
            "source_refs": ["source_ref_1"]
          }
        ]
      },
      {
        "section_id": "section_assessment",
        "title": "Оценка зрелости",
        "section_type": "assessment",
        "custom_section_type": null,
        "items": [
          {
            "item_id": "item_2",
            "title": "Local LLM agents — уровень: pilot",
            "text": "Технология находится на стадии pilot: есть реальные клиентские внедрения, но не достигнут масштаб production. Основной барьер — отсутствие стандартизованного интерфейса.",
            "claim_refs": ["claim_1", "claim_2", "claim_3"],
            "evidence_refs": ["evidence_1", "evidence_2", "evidence_3"],
            "source_refs": ["source_ref_1"]
          }
        ]
      },
      {
        "section_id": "section_recommendations",
        "title": "Рекомендации",
        "section_type": "recommendations",
        "custom_section_type": null,
        "items": [
          {
            "item_id": "item_3",
            "title": "Запустить внутренний пилот",
            "text": "Технология готова к пилоту на одном use case. Рекомендуется выбрать изолированный сценарий с явными критериями успеха.",
            "claim_refs": ["claim_1", "claim_2"],
            "evidence_refs": ["evidence_1"],
            "source_refs": ["source_ref_1"]
          }
        ]
      }
    ],
    "pack_data": {
      "technology_watch": {
        "technologies": [
          {
            "technology_id": "tech_local_llm_agents",
            "name": "Local LLM agents",
            "normalized_name": "local_llm_agents",
            "maturity": {
              "level": "pilot",
              "confidence": {
                "score": 0.78,
                "basis": "multiple_corroborating_sources",
                "custom_basis": null,
                "method": "llm_assessment",
                "custom_method": null
              },
              "rationale": "Несколько независимых источников описывают переход от прототипов к ограниченным пилотам у корпоративных клиентов.",
              "claim_refs": ["claim_1", "claim_2"]
            },
            "signals": [
              {
                "signal_id": "signal_1",
                "signal_type": "adoption_evidence",
                "custom_signal_type": null,
                "description": "Компании описывают переход от internal demo к customer-facing pilot в Q1–Q2 2026.",
                "claim_refs": ["claim_1"],
                "evidence_refs": ["evidence_1"],
                "source_refs": ["source_ref_1"]
              }
            ],
            "tools": [
              {
                "tool_id": "tool_ollama",
                "name": "Ollama",
                "canonical_url": "https://ollama.ai",
                "description": "Local model runner для desktop и server deployment.",
                "tool_maturity": "production",
                "claim_refs": ["claim_2"],
                "source_refs": ["source_ref_1"]
              }
            ],
            "adoption_barriers": [
              {
                "barrier_id": "barrier_1",
                "barrier_type": "technical",
                "custom_barrier_type": null,
                "description": "Отсутствие стандартизованного интерфейса для локальных моделей затрудняет смену провайдера.",
                "severity": "high",
                "claim_refs": ["claim_3"]
              }
            ],
            "risks": [],
            "recommendations": [
              {
                "recommendation_id": "rec_1",
                "recommendation_type": "pilot",
                "custom_recommendation_type": null,
                "description": "Запустить внутренний пилот на одном изолированном use case с явными критериями успеха.",
                "priority": "high",
                "target_audience": "engineering_teams",
                "claim_refs": ["claim_1", "claim_2"]
              }
            ],
            "claim_refs": ["claim_1", "claim_2", "claim_3"],
            "evidence_refs": ["evidence_1"],
            "source_refs": ["source_ref_1"]
          }
        ]
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
        "comment_collection_status": "not_requested",
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
      "claim_status": "verified",
      "custom_claim_status": null,
      "claim_text": "Локальные LLM-агенты переходят из экспериментов в ограниченные пилоты.",
      "normalized_claim_text": "Local LLM agents are moving from experiments to limited pilots.",
      "normalized_claim_language": "en",
      "scope": {
        "period": { "from": "2026-01-01", "to": "2026-06-06" },
        "geo": null,
        "language": null,
        "applies_to": [{ "label": "local LLM agents", "entity_type": "technology", "entity_id": null }]
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
        "stage": "technology_watch/trend_extraction",
        "provider": "openai_compatible",
        "model": "gpt-4.1-mini",
        "audit_refs": []
      }
    },
    {
      "claim_id": "claim_2",
      "claim_type": "evaluative",
      "custom_claim_type": null,
      "claim_status": "verified",
      "custom_claim_status": null,
      "claim_text": "Ollama достиг production-уровня зрелости как local model runner.",
      "normalized_claim_text": "Ollama has reached production-level maturity as a local model runner.",
      "normalized_claim_language": "en",
      "scope": {
        "period": { "from": "2026-01-01", "to": "2026-06-06" },
        "geo": null,
        "language": null,
        "applies_to": [{ "label": "Ollama", "entity_type": "tool", "entity_id": null }]
      },
      "confidence": {
        "score": 0.85,
        "basis": "strong_direct_evidence",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "evidence_refs": ["evidence_2"],
      "source_refs": ["source_ref_1"],
      "relation_refs": [],
      "provenance": {
        "stage": "technology_watch/maturity_scoring",
        "provider": "openai_compatible",
        "model": "gpt-4.1",
        "audit_refs": []
      }
    },
    {
      "claim_id": "claim_3",
      "claim_type": "evaluative",
      "custom_claim_type": null,
      "claim_status": "partially_verified",
      "custom_claim_status": null,
      "claim_text": "Отсутствие стандартизованного интерфейса является основным барьером внедрения.",
      "normalized_claim_text": "Lack of a standardized interface is the primary adoption barrier.",
      "normalized_claim_language": "en",
      "scope": {
        "period": { "from": "2026-01-01", "to": "2026-06-06" },
        "geo": null,
        "language": null,
        "applies_to": [{ "label": "local LLM agents", "entity_type": "technology", "entity_id": null }]
      },
      "confidence": {
        "score": 0.65,
        "basis": "single_source",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "evidence_refs": ["evidence_3"],
      "source_refs": ["source_ref_1"],
      "relation_refs": [],
      "provenance": {
        "stage": "technology_watch/adoption_analysis",
        "provider": "openai_compatible",
        "model": "gpt-4.1",
        "audit_refs": []
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
      "context_text": "Speaker describes transition from internal experiments to customer-facing pilots.",
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
        "stage": "technology_watch/trend_extraction",
        "provider": "openai_compatible",
        "model": "gpt-4.1-mini",
        "audit_refs": []
      }
    },
    {
      "evidence_id": "evidence_2",
      "claim_id": "claim_2",
      "source_ref_id": "source_ref_1",
      "evidence_type": "fragment",
      "custom_evidence_type": null,
      "evidence_role": "supports",
      "custom_evidence_role": null,
      "fragment_type": "video_timestamp_range",
      "custom_fragment_type": null,
      "locator_data": {
        "schema_version": "1.0",
        "timestamp_start": 620.0,
        "timestamp_end": 645.0
      },
      "text_mode": "paraphrase",
      "fragment_text": "The source describes Ollama as stable enough for production use as a local model runner.",
      "context_text": "Speaker discusses local model runners that are already used in production deployment stacks.",
      "contributing_evidence_refs": [],
      "reasoning_summary": null,
      "confidence": {
        "score": 0.82,
        "basis": "strong_direct_evidence",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "provenance": {
        "stage": "technology_watch/maturity_scoring",
        "provider": "openai_compatible",
        "model": "gpt-4.1",
        "audit_refs": []
      }
    },
    {
      "evidence_id": "evidence_3",
      "claim_id": "claim_3",
      "source_ref_id": "source_ref_1",
      "evidence_type": "fragment",
      "custom_evidence_type": null,
      "evidence_role": "supports",
      "custom_evidence_role": null,
      "fragment_type": "video_timestamp_range",
      "custom_fragment_type": null,
      "locator_data": {
        "schema_version": "1.0",
        "timestamp_start": 901.0,
        "timestamp_end": 930.0
      },
      "text_mode": "paraphrase",
      "fragment_text": "The source identifies missing standardized interfaces between local model runners as a barrier to switching providers.",
      "context_text": "Speaker describes integration friction and provider lock-in around local model runner APIs.",
      "contributing_evidence_refs": [],
      "reasoning_summary": null,
      "confidence": {
        "score": 0.66,
        "basis": "single_source",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "provenance": {
        "stage": "technology_watch/adoption_analysis",
        "provider": "openai_compatible",
        "model": "gpt-4.1",
        "audit_refs": []
      }
    }
  ],

  "claim_relations": [],
  "unknowns": [
    {
      "unknown_id": "unknown_1",
      "unknown_type": "missing_data",
      "custom_unknown_type": null,
      "title": "Недостаточно независимых подтверждений барьера стандартизации",
      "description": "claim_3 основан на одном источнике; нужны дополнительные источники, чтобы подтвердить, что отсутствие стандартизованного интерфейса является основным барьером внедрения.",
      "why_it_matters": "Без подтверждения из независимых источников барьер нельзя считать устойчивым паттерном для всей технологии.",
      "claim_refs": ["claim_3"],
      "source_refs": ["source_ref_1"],
      "evidence_refs": ["evidence_3"],
      "relation_refs": [],
      "verification_task_refs": [],
      "confidence": {
        "score": 0.74,
        "basis": "single_source",
        "custom_basis": null,
        "method": "llm_assessment",
        "custom_method": null
      },
      "provenance": {
        "stage": "technology_watch/adoption_analysis",
        "provider": "openai_compatible",
        "model": "gpt-4.1",
        "audit_refs": []
      }
    }
  ],
  "verification_tasks": [],
  "warnings": [
    {
      "warning_id": "warning_1",
      "warning_type": "single_source_claim",
      "custom_warning_type": null,
      "severity": "medium",
      "message": "claim_3 (барьер стандартизации) основан на одном источнике.",
      "claim_refs": ["claim_3"],
      "source_refs": ["source_ref_1"],
      "evidence_refs": ["evidence_3"],
      "relation_refs": [],
      "section_refs": []
    }
  ],
  "limitations": [
    {
      "limitation_id": "limitation_1",
      "limitation_type": "corpus_coverage_limited",
      "custom_limitation_type": null,
      "severity": "low",
      "description": "Корпус содержит преимущественно англоязычные источники.",
      "claim_refs": [],
      "source_refs": [],
      "evidence_refs": [],
      "relation_refs": [],
      "section_refs": []
    }
  ],
  "quality_flags": [
    {
      "flag": "unverified_claims_present",
      "custom_flag": null,
      "severity": "medium",
      "message": "Результат содержит partially_verified claim.",
      "claim_refs": ["claim_3"],
      "source_refs": ["source_ref_1"],
      "evidence_refs": ["evidence_3"],
      "relation_refs": [],
      "section_refs": []
    },
    {
      "flag": "single_source_claim",
      "custom_flag": null,
      "severity": "medium",
      "message": "claim_3 основан на одном источнике.",
      "claim_refs": ["claim_3"],
      "source_refs": ["source_ref_1"],
      "evidence_refs": ["evidence_3"],
      "relation_refs": [],
      "section_refs": []
    }
  ],
  "audit_refs": []
}
```

---

## 9. Принятые решения

### SD-TW-01 — `entity_type` в `claim.scope.applies_to`: pack-local recommended values

Базовый контракт оставляет `entity_type` свободной строкой.
`technology_watch` фиксирует рекомендованный набор:

```
technology    — технология, подход, парадигма
tool          — конкретный инструмент, утилита, CLI
framework     — фреймворк или библиотека
protocol      — протокол или стандарт
platform      — платформа или инфраструктурный сервис
concept       — концептуальный термин или идея
component     — компонент системы или архитектурный блок
method        — метод, алгоритм или техника
```

Значения рекомендованные, не строгий enum: pack не отклоняет неизвестные значения,
но генерация должна придерживаться этого словаря для cross-run сравнений.

### SD-TW-02 — `deprecated` maturity рекомендует `avoid` или `deprecate`

Если `technology.maturity.level = deprecated`, pack должен по возможности
создать рекомендацию с `recommendation_type = "avoid"` или
`recommendation_type = "deprecate"`.

Это мягкое правило v1, не hard validation: иногда deprecated-технология может
быть упомянута только как исторический контекст или как comparison baseline.

---

## 10. Открытые вопросы

### OQ-TW-02 — Связь между `signals` и `adoption_barriers` / `risks`

`signals` — сырые наблюдения, `adoption_barriers` / `risks` — синтез.
Сейчас нет явной ссылки из `adoption_barrier` на исходный `signal`.
Если нужна трассируемость синтеза: добавить `signal_refs: []` в barrier/risk.
Для v1 достаточно `claim_refs`.

### OQ-TW-03 — Несколько технологий в одном `sections` item

Рекомендация или тренд может охватывать несколько технологий одновременно.
Сейчас `item.claim_refs` может ссылаться на claims разных технологий — это допустимо.
Но нет явного поля `technology_refs` в `sections.items` для быстрого traversal.
Для v1 достаточно traversal через `claim_refs` → `technology.claim_refs`.
