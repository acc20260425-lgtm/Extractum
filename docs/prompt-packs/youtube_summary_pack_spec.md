# youtube_summary Pack Spec — v1

Версионируется независимо от `schema_version` основного контракта.
Текущая версия pack: `v1`.
Совместимость: Prompt Pack JSON Contract `schema_version: "1.0"`.

Если pack spec обновляется под другой `schema_version` core contract, pack должен
либо bump-нуть `pack_version`, либо добавить явную compatibility table. Один
`pack_version` не должен молча означать разные core-contract shapes.

---

## 1. Назначение и границы

`youtube_summary` — pack для проверяемого summary одного или нескольких YouTube-видео
по транскрипту, metadata и доступным фрагментам видео.

**Отвечает на вопросы:**
- О чём видео и какие ключевые тезисы в нём есть?
- Какие утверждения, выводы, рекомендации или вопросы поднимает автор?
- Какие короткие цитаты помогают проверить наиболее важные тезисы?
- Если видео несколько, какие темы повторяются или противоречат друг другу?

**Не входит в задачу:**
- Републикация полного транскрипта или длинных фрагментов видео.
- Оценка личности автора или OSINT-анализ канала.
- Технический аудит корректности всех утверждений за пределами доступного корпуса.
- Playlist-level analysis как самостоятельный `source_ref`; каждое видео остаётся отдельным
  `source_ref` с `source_type = "youtube_video"`.

---

## 2. `run_context` настройки

### `control_preset`

Для `youtube_summary` `control_preset` управляет глубиной извлечения объектов из видео.
Ось остаётся совместимой с другими packs: `quick` даёт более узкий результат,
`deep` — более полный.

| Значение | Поведение |
|---|---|
| `quick` | Только readable summary и `key_points`; `segments`, `notable_quotes`, `action_items`, `open_questions` не требуются |
| `standard` | `segments`, `key_points`, `notable_quotes`; `action_items` и `open_questions` опциональны |
| `deep` | Полный набор: `segments`, `key_points`, `notable_quotes`, `action_items`, `open_questions`; `synthesis` при нескольких видео |

Правила:
- В `quick` mode `video.segments: []`, `video.notable_quotes: []`,
  `video.action_items: []` и `video.open_questions: []` допустимы.
- В `standard` mode ожидается хотя бы один `segment` и один `key_point`
  для каждого видео при наличии usable transcript.
- В `deep` mode pack должен извлекать content-layer вопросы и action items,
  если они реально присутствуют в видео.
- Для multi-video input `synthesis` обязателен в `deep` mode при
  `result_status = "complete"`, если не объяснён через `quality_flags`.

### `evidence_mode`

`evidence_mode` управляет строгостью traceability, а не глубиной извлечения.

| Значение | Поведение |
|---|---|
| `narrative_only` | Readable result first. `pack_data.youtube_summary.videos` может быть пустым |
| `standard` | Structured result required. `videos` непустой при `result_status = "complete"` |
| `strict` | Как standard + усиленные pipeline-level проверки evidence coverage |

Правила:
- В `narrative_only`, если `videos` пустой при `result_status = "complete"`,
  нужен `quality_flag` с `flag = "partial_result"` или `flag = "corpus_coverage_limited"`.
- В `standard`, каждый `key_point` и `action_item` должен иметь непустой `claim_refs`.
- В `standard`, каждый `notable_quote` должен иметь непустой `evidence_refs`.
- В `standard` и `strict`, каждый `notable_quote.evidence_refs` ссылается на
  top-level evidence с `text_mode = "verbatim"` и
  `fragment_type = "video_timestamp_range"`.
- В `strict`, каждый `key_point.claim_refs` должен вести к claim, у которого есть
  хотя бы одно evidence с `fragment_type = "video_timestamp_range"` или
  `fragment_type = "text_range"` transcript snapshot.
- В `strict`, каждая quote evidence запись должна иметь `text_mode = "verbatim"`.
- В `strict`, cross-video synthesis claims должны ссылаться на claims или evidence
  минимум из двух разных `videos[].source_ref_id`.
- Правила, которые требуют обхода `key_point.claim_refs → claim.evidence_refs → evidence`,
  являются pipeline-level validation, не JSON Schema rules.

### Ожидаемые `source_types`

Основной source type: `youtube_video`.

Каждый объект `videos[]` обязан ссылаться на top-level `source_ref` с
`source_type = "youtube_video"`. Дополнительные источники могут присутствовать
в общем результате, но не заменяют video anchor.

---

## 3. `outputs.pack_data.youtube_summary`

### Верхний уровень

```json
{
  "youtube_summary": {
    "videos": [],
    "synthesis": null
  }
}
```

Правила:
- `videos` обязателен как поле.
- `synthesis` обязателен как поле; значение `null` допустимо.
- При одном видео `synthesis` должен быть `null`.
- При нескольких видео `synthesis` может быть `null` только в `quick`,
  `narrative_only`, `error` или если отсутствие cross-video synthesis объяснено
  через `quality_flags`.
- Для multi-video `standard` или `deep` result с `result_status = "complete"`
  и `evidence_mode != "narrative_only"` отсутствие `synthesis` без объясняющего
  `quality_flags` является validation error.

### Объект `Video`

```json
{
  "video_id": "video_ys_001",
  "source_ref_id": "source_ref_1",
  "segments": [],
  "key_points": [],
  "notable_quotes": [],
  "action_items": [],
  "open_questions": [],
  "claim_refs": [],
  "evidence_refs": [],
  "source_refs": ["source_ref_1"]
}
```

`video_id` уникален внутри `outputs.pack_data.youtube_summary.videos`.

`source_ref_id` обязателен. Это primary anchor к top-level `source_refs[]`,
где хранится `youtube_video.type_data`: `duration_seconds`, `channel_title`,
`transcript_available`, `captions_available`, `comment_collection_status` и другие
platform-specific поля.

`source_refs` — traversal-поле:
- должно включать `source_ref_id`;
- может включать дополнительные источники, если конкретный video-level вывод
  был проверен внешними материалами;
- не является заменой `source_ref_id`.

`claim_refs`, `evidence_refs`, `source_refs` — денормализованные traversal-поля:
- `claim_refs` — union всех `claim_refs` из `segments`, `key_points`,
  `notable_quotes`, `action_items`, `open_questions` и связанных объектов
  synthesis, если они относятся к этому видео;
- `evidence_refs` — union всех `evidence_refs` из `segments`, `key_points`,
  `notable_quotes`, `action_items`, `open_questions`;
- `source_refs` — union source refs, использованных video-level объектами,
  и обязательно `source_ref_id`.

---

## 4. Вложенные объекты `Video`

### ID scope для вложенных объектов

Все `_id` внутри `Video` уникальны только в пределах своего массива:

- `segment_id` уникален внутри `video.segments`;
- `key_point_id` уникален внутри `video.key_points`;
- `quote_id` уникален внутри `video.notable_quotes`;
- `action_item_id` уникален внутри `video.action_items`;
- `open_question_id` уникален внутри `video.open_questions`.

Глобальная уникальность этих ID внутри `run_id` не требуется.

### 4.1 `segments`

`segments` — смысловые блоки видео. Это не обязательно YouTube chapters.

```json
{
  "segment_id": "segment_1",
  "title": "Problem framing",
  "summary_text": "Автор формулирует проблему, которую решает инструмент.",
  "timestamp_start": 0,
  "timestamp_end": 92,
  "creator_defined": false,
  "claim_refs": [],
  "evidence_refs": ["evidence_1"]
}
```

Правила:
- `timestamp_start` и `timestamp_end` задаются в секундах от начала видео.
- Границы timestamp используют ту же convention, что `video_timestamp_range`
  в `fragment_locator_schemas.md`: inclusive/inclusive.
- `segment.evidence_refs` должен ссылаться только на evidence того же video source,
  чьи `locator_data.timestamp_start` или `locator_data.timestamp_end` попадают
  внутрь `[segment.timestamp_start, segment.timestamp_end]`.
- Проверка принадлежности `segment.evidence_refs` к timestamp-диапазону является
  pipeline-level validation, не JSON Schema rule.
- `creator_defined = true` означает, что segment соответствует реальному YouTube chapter,
  заданному автором видео.
- `creator_defined = false` означает LLM-generated или pipeline-generated split.
- `creator_defined = null` означает, что происхождение segment неизвестно.
- Segment — navigation object; он не обязан сам быть claim.

### 4.2 `key_points`

`key_points` — ключевые тезисы видео, выраженные как readable pack-level объекты
и связанные с top-level `claims`.

```json
{
  "key_point_id": "key_point_1",
  "point_text": "Автор утверждает, что локальные агенты становятся практичным вариантом для пилотов.",
  "claim_refs": ["claim_1"],
  "evidence_refs": ["evidence_1"]
}
```

Правила:
- В `standard` и `strict` каждый `key_point` должен иметь непустой `claim_refs`.
- `claim_refs` обычно указывают на claims с `claim_type = "factual"` или
  `claim_type = "evaluative"`.
- `evidence_refs` указывают на фрагменты, где тезис произнесён, показан или обоснован.
- Если claims из `key_point.claim_refs` имеют собственные `claim.evidence_refs`,
  `key_point.evidence_refs` желательно заполнять как их union для удобства навигации.
  Это soft traversal rule, не hard validation в v1.
- `key_point` не заменяет top-level claim; он является readable projection claim-а
  внутри pack-specific структуры.

### 4.3 `notable_quotes`

`notable_quotes` — короткие дословные цитаты для проверки и иллюстрации важных тезисов.
Это evidence layer, не слой перепубликации.

```json
{
  "quote_id": "quote_1",
  "quote_text": "We are moving from demos to real internal pilots.",
  "speaker_id": null,
  "speaker_label": null,
  "word_count": 9,
  "timestamp_start": 120,
  "timestamp_end": 128,
  "claim_refs": ["claim_1"],
  "evidence_refs": ["evidence_2"]
}
```

Правила:
- Одна quote не должна превышать 50 слов.
- Если `word_count` заполнен, он должен совпадать с количеством слов в
  `quote_text` по той же word-counting convention, которая используется для
  ограничения в 50 слов.
- Цитаты предназначены для evidence и verification, не для восстановления transcript-а
  или republication content.
- `notable_quote` является pack_data projection, а не самой top-level evidence записью.
- Каждая quote должна иметь хотя бы одну соответствующую запись в top-level `evidence[]`;
  `notable_quote.evidence_refs` — traversal-ссылка на эту запись.
- Соответствующая evidence запись должна иметь `text_mode = "verbatim"` и
  `fragment_type = "video_timestamp_range"`.
- `fragment_type = "text_range"` допустим для transcript-backed `key_points`,
  но не заменяет media locator для `notable_quotes`.
- `quote.timestamp_start` и `quote.timestamp_end` — convenience metadata для быстрого
  доступа. Авторитетным locator остаётся `evidence.locator_data` referenced evidence;
  при расхождении consumer должен доверять `evidence.locator_data`.
- Quote сама по себе не является claim. Если процитированная фраза содержит проверяемое
  утверждение, оно должно быть вынесено в top-level `claims`, а quote ссылается на него
  через `claim_refs`.
- `speaker_id` зарезервирован для diarized transcripts. В v1 это `null | string`;
  если заполнен, значение стабильно внутри данного `Video` object.
- `speaker_label` может быть `null`, если speaker не определён или в видео один говорящий.

### 4.4 `action_items`

`action_items` — действия, рекомендации или next steps, явно предложенные автором
или выведенные pack-ом из содержания видео.

```json
{
  "action_item_id": "action_1",
  "action_text": "Проверить инструмент на небольшом внутреннем пилоте.",
  "target_audience": "engineering team",
  "priority": "medium",
  "claim_refs": ["claim_2"],
  "evidence_refs": ["evidence_3"]
}
```

`priority` enum v1:
```
high | medium | low | null
```

Правила:
- `action_items` обычно трассируются к claims с `claim_type = "normative"`.
- `target_audience` — свободная строка или `null`.
- Если action item является выводом pack-а, а не прямой рекомендацией автора,
  это должно отражаться в соответствующем claim provenance и confidence.

### 4.5 `open_questions`

`video.open_questions` — вопросы, поднятые внутри видео, но не закрытые самим видео.
Это content layer, не pipeline-gap layer.

```json
{
  "open_question_id": "open_question_1",
  "question_text": "Какие ограничения появятся при масштабировании подхода на production workloads?",
  "raised_by_speaker": true,
  "timestamp_start": 310,
  "timestamp_end": 326,
  "claim_refs": [],
  "evidence_refs": ["evidence_4"]
}
```

Правила:
- `video.open_questions` не дублирует top-level `unknowns`.
- Top-level `unknowns` фиксирует пробел анализа pipeline-а: что результат не смог установить.
- `video.open_questions` фиксирует вопрос как часть содержания видео.
- `raised_by_speaker = true` означает, что вопрос явно сформулирован в видео.
- `raised_by_speaker = false` означает, что вопрос выведен pack-ом из содержания,
  но speaker не сформулировал его явно.
- `raised_by_speaker = null` означает, что происхождение вопроса неизвестно.
- Если вопрос из видео одновременно создаёт аналитический пробел, pack может создать
  и `video.open_question`, и top-level `unknown`, связанный через `evidence_refs`
  или `claim_refs`.

---

## 5. `synthesis`

`synthesis` — cross-video слой для multi-video summaries.

При одном видео `synthesis = null`.
Если `synthesis = null`, вложенные поля `claim_refs`, `relation_refs`,
`evidence_refs` и `source_refs` внутри `synthesis` отсутствуют; это валидно
для `quick`, `narrative_only`, `error` и для multi-video результатов, где
отсутствие cross-video synthesis объяснено через `quality_flags`.

```json
{
  "cross_video_themes": [],
  "common_claims": [],
  "contradictions_across_videos": [],
  "claim_refs": [],
  "relation_refs": [],
  "evidence_refs": [],
  "source_refs": []
}
```

Traversal rules:
- `synthesis.claim_refs` — union всех `claim_refs` из `cross_video_themes`,
  `common_claims` и `contradictions_across_videos`.
- `synthesis.relation_refs` — union всех `relation_refs` из
  `contradictions_across_videos`.
- `synthesis.evidence_refs` — union всех `evidence_refs` из вложенных synthesis objects.
- `synthesis.source_refs` — union `videos[].source_refs` для всех `video_refs`,
  упомянутых во вложенных synthesis objects, плюс source refs, достижимые через
  referenced evidence/claims.

### 5.1 `cross_video_themes`

```json
{
  "theme_id": "theme_1",
  "theme_text": "Несколько видео описывают переход от demo к pilot usage.",
  "video_refs": ["video_ys_001", "video_ys_002"],
  "claim_refs": ["claim_1", "claim_5"],
  "evidence_refs": ["evidence_1", "evidence_8"]
}
```

`video_refs` ссылаются на `videos[].video_id` внутри этого pack_data блока.

### 5.2 `common_claims`

```json
{
  "common_claim_id": "common_claim_1",
  "summary_text": "Оба видео утверждают, что adoption сдерживается качеством tooling.",
  "video_refs": ["video_ys_001", "video_ys_002"],
  "claim_refs": ["claim_cross_1"],
  "evidence_refs": ["evidence_4", "evidence_12"]
}
```

`common_claims` не создают отдельный claim автоматически.

Правила:
- Если pack создаёт отдельный synthesized cross-video claim, `common_claim.claim_refs`
  должен ссылаться на этот top-level claim.
- `common_claim.claim_refs` не должен смешивать synthesized cross-video claim
  и исходные per-video supporting claims.
- Если synthesis остаётся readable observation без отдельного top-level claim,
  `claim_refs: []` допустим; тогда traversal идёт через `video_refs`,
  `evidence_refs` и per-video objects внутри referenced `videos[]`.

### 5.3 `contradictions_across_videos`

```json
{
  "contradiction_id": "cross_contradiction_1",
  "description": "Первое видео описывает подход как pilot-ready, второе — как экспериментальный.",
  "video_refs": ["video_ys_001", "video_ys_002"],
  "relation_refs": ["rel_1"],
  "claim_refs": ["claim_2", "claim_7"],
  "evidence_refs": ["evidence_3", "evidence_10"]
}
```

Правила:
- Семантическим источником истины для противоречий остаётся top-level
  `claim_relations` с `relation_type = "contradicts"`.
- `contradictions_across_videos` — readable traversal layer для multi-video pack_data.
- `relation_refs` должен ссылаться на relation, где оба claim-а относятся к разным
  `videos[].source_ref_id`.

---

## 6. Mapping pack objects к core contract

| Pack object | Core contract mapping |
|---|---|
| `segments` | Navigation/content structure; claims optional |
| `key_points` | Top-level claims with `claim_type = "factual"` or `claim_type = "evaluative"` |
| `notable_quotes` | Pack_data projection pointing to top-level evidence with `text_mode = "verbatim"`; not claims by themselves |
| `action_items` | Top-level claims with `claim_type = "normative"` |
| `open_questions` | Content-layer questions; not top-level `unknowns` unless pipeline gap exists |
| `synthesis.cross_video_themes` | Cross-video grouping over claims/evidence |
| `synthesis.common_claims` | Optional synthesized cross-video claim, or readable grouping traced via videos/evidence |
| `synthesis.contradictions_across_videos` | Top-level `claim_relations` with `relation_type = "contradicts"` |

---

## 7. `outputs.sections` — паттерн для `youtube_summary`

`outputs.sections` остаётся readable слоем результата.

Рекомендуемые section patterns:

| `section_type` | Когда использовать |
|---|---|
| `summary` | Общий summary одного видео или multi-video synthesis |
| `assessment` | Key points, interpretation, content analysis |
| `recommendations` | Action items, если они есть |
| `custom` + `custom_section_type = "segments"` | Readable timeline/segment view, если нужен |
| `custom` + `custom_section_type = "quotes"` | Readable quote list, если нужен |

Правила:
- `sections.items[].claim_refs` и `sections.items[].evidence_refs` должны ссылаться
  на те же top-level claims/evidence, что и pack_data объекты.
- `notable_quotes` не должны попадать в `sections` как длинный transcript-like блок.
- Для multi-video результата `summary` может агрегировать cross-video themes,
  но его `claim_refs` должны быть покрыты `sections.items[].claim_refs`
  по общему правилу контракта.

---

## 8. Обязательства pack v1

### Hard requirements при `result_status = "complete"`

- `outputs.pack_data.youtube_summary.videos` непустой при `evidence_mode != "narrative_only"`.
- Каждый `Video` имеет `source_ref_id`.
- Каждый `Video.source_ref_id` ссылается на top-level `source_refs[]` с
  `source_type = "youtube_video"`.
- Каждый `Video.source_refs` включает свой `source_ref_id`.
- В `standard` и `strict` каждый `key_point` имеет непустой `claim_refs`.
- Каждый `notable_quote`, если он присутствует, имеет непустой `evidence_refs`.
- Каждый `notable_quote.evidence_refs` ссылается на top-level evidence с
  `text_mode = "verbatim"` и `fragment_type = "video_timestamp_range"`.
- Каждая `notable_quote.quote_text` содержит не более 50 слов.
- Если `notable_quote.word_count` заполнен, он совпадает с количеством слов в
  `quote_text` по той же convention, что и правило 50 слов.
- При одном видео `synthesis = null`.
- В multi-video `standard` или `deep` result с `result_status = "complete"`
  и `evidence_mode != "narrative_only"` `synthesis` заполнен, если отсутствие
  не объяснено через `quality_flags`.

### Soft requirements

- В `standard` mode у каждого видео желательно иметь хотя бы один `segment`.
- В `standard` mode у каждого видео желательно иметь хотя бы один `notable_quote`,
  если transcript доступен.
- В `deep` mode, если видео содержит явные рекомендации, желательно иметь
  `action_items`.
- В `deep` mode, если видео явно поднимает вопросы без ответа, желательно иметь
  `open_questions`.
- Если в multi-video `standard` mode `synthesis = null` допустим из-за
  объясняющего `quality_flags`, повтор одного `claim_id` в `key_points[].claim_refs`
  у двух или более разных `Video` объектов является QA warning.

---

## 9. Pack-specific stages

Pack-specific stage names используют namespace `{pack_id}/{stage_name}`.

| Stage | Назначение |
|---|---|
| `youtube_summary/transcript_analysis` | Анализ transcript и доступных caption/transcript фрагментов |
| `youtube_summary/segment_extraction` | Выделение смысловых segments или creator-defined chapters |
| `youtube_summary/key_point_extraction` | Извлечение ключевых тезисов и связанных claims |
| `youtube_summary/quote_extraction` | Выбор коротких notable quotes и evidence |
| `youtube_summary/synthesis` | Cross-video synthesis для multi-video результата |

Все stages опциональны на уровне конкретного run. `youtube_summary/synthesis`
используется только если вход содержит несколько видео или pack явно строит
cross-video view.

MVP может запускать только `youtube_summary/transcript_analysis` как combined
stage. В таком режиме `segment_extraction`, `key_point_extraction` и
`quote_extraction` остаются stage-skeleton rows со статусом `skipped`; их
кандидаты возвращаются внутри parsed output combined stage и проходят тот же
`stage_io_version = "1.0"` closed-world validation по allowed source/material
registries. Pipeline, а не LLM, назначает final canonical IDs и nested pack
object IDs.

MVP validator identity for the combined stage:

- `input_schema_id = "stage-io/youtube_summary_transcript_analysis_input"`
- `output_schema_id = "stage-io/youtube_summary_transcript_analysis_output"`
- `validator_mode = "stage_output"`
- `validator_stage = "youtube_summary/transcript_analysis"`
- these schema assets are seeded through `prompt_pack_schema_assets`, and the
  `youtube_summary/transcript_analysis` stage template references them through
  `input_schema_id` and `output_schema_id`.

---

## 10. Минимальный `pack_data` пример

```json
{
  "youtube_summary": {
    "videos": [
      {
        "video_id": "video_ys_001",
        "source_ref_id": "source_ref_1",
        "segments": [
          {
            "segment_id": "segment_1",
            "title": "Problem framing",
            "summary_text": "Автор описывает переход от демонстраций к внутренним пилотам.",
            "timestamp_start": 0,
            "timestamp_end": 92,
            "creator_defined": false,
            "claim_refs": ["claim_1"],
            "evidence_refs": ["evidence_1"]
          }
        ],
        "key_points": [
          {
            "key_point_id": "key_point_1",
            "point_text": "Локальные агенты начинают использоваться в пилотных сценариях.",
            "claim_refs": ["claim_1"],
            "evidence_refs": ["evidence_1"]
          }
        ],
        "notable_quotes": [
          {
            "quote_id": "quote_1",
            "quote_text": "We are moving from demos to real internal pilots.",
            "speaker_id": null,
            "speaker_label": null,
            "word_count": 9,
            "timestamp_start": 120,
            "timestamp_end": 128,
            "claim_refs": ["claim_1"],
            "evidence_refs": ["evidence_2"]
          }
        ],
        "action_items": [],
        "open_questions": [],
        "claim_refs": ["claim_1"],
        "evidence_refs": ["evidence_1", "evidence_2"],
        "source_refs": ["source_ref_1"]
      }
    ],
    "synthesis": null
  }
}
```

---

## 11. Принятые решения

### SD-YS-01: `videos[]` как атомарная модель

Pack использует `videos[]`, даже если вход содержит одно видео. Это сохраняет
одинаковую форму single-video и multi-video результата.

### SD-YS-02: `synthesis` как top-level cross-video слой

`synthesis` добавлен сразу как `null | object`, чтобы multi-video summary
не требовал breaking change.

### SD-YS-03: `segments` вместо `chapters`

Pack использует `segments`, потому что не каждый смысловой блок является
creator-defined YouTube chapter. Поле `creator_defined` фиксирует происхождение.

### SD-YS-04: `open_questions` не равны top-level `unknowns`

`video.open_questions` описывает вопросы внутри содержания видео. Top-level
`unknowns` описывает пробелы анализа pipeline-а.

### SD-YS-05: `notable_quotes` ограничены evidence use

Quotes предназначены для проверки и иллюстрации, не для перепубликации.
Одна quote ограничена 50 словами.

### SD-YS-06: `Video.source_ref_id` как обязательный anchor

Каждое видео обязано ссылаться на top-level `source_ref` с
`source_type = "youtube_video"`.

### SD-YS-07: `control_preset` как глубина извлечения

Для `youtube_summary` `quick | standard | deep` регулируют полноту extracted
objects: от key points до full extraction.

### SD-YS-08: pack-specific stages

Pack фиксирует пять stage names для provenance:
`transcript_analysis`, `segment_extraction`, `key_point_extraction`,
`quote_extraction`, `synthesis`.

### SD-YS-09: quote как projection top-level evidence

`notable_quote` хранится в pack_data для readable/navigation use, но авторитетная
дословная цитата и locator живут в top-level `evidence[]`.

### SD-YS-10: common_claim не смешивает synthesized и supporting claims

`common_claim.claim_refs` указывает на synthesized cross-video claim, если он создан.
Supporting per-video claims остаются достижимыми через `video_refs`, `evidence_refs`
и per-video objects.

---

## 12. Открытые вопросы

### ~~OQ-YS-01~~ — ЗАКРЫТ: Speaker identity model

В v1 `notable_quote` содержит `speaker_id: null | string` и
`speaker_label: null | string`.

`speaker_id` зарезервирован для diarized transcripts и стабилен внутри одного
`Video` object. `speaker_label` остаётся human-readable display string.
Отдельный `speakers[]` block не вводится в v1; если понадобится профиль speaker-а
или cross-video identity, это отдельное v2 расширение.

### OQ-YS-02: Multi-video synthesis granularity

`synthesis` содержит themes, common claims и contradictions. Практика покажет,
нужны ли отдельные объекты для chronology, consensus score или per-video deltas.

### OQ-YS-03: Playlist-level context

Playlist не является самостоятельным `source_ref` в v1. Если pack начнёт
суммировать playlist как editorial unit, потребуется отдельный контекстный слой,
не заменяющий `videos[]`.

Для MVP run storage selection/origin context не копируется в `youtube_video`
`source_ref`. Canonical video snapshot остаётся уникальным по video, а direct
selection или playlist membership хранится во внешнем run-local origin слое.
Открытым остаётся только будущий вопрос: нужен ли pack-level editorial context
для playlist summary как отдельного readable объекта.
