# telegram_summary Pack Spec — v1 draft

Версионируется независимо от `schema_version` основного контракта.
Текущая версия pack: `v1`.
Совместимость: Prompt Pack JSON Contract `schema_version: "1.0"`.

Если pack spec обновляется под другой `schema_version` core contract, pack должен
либо bump-нуть `pack_version`, либо добавить явную compatibility table. Один
`pack_version` не должен молча означать разные core-contract shapes.

---

## 1. Назначение и границы

`telegram_summary` — pack для проверяемого summary Telegram-каналов, групповых
чатов, supergroup/forum topics и смешанных подборок Telegram-источников за
заданный период.

Pack один для каналов и чатов. Различие фиксируется внутри
`outputs.pack_data.telegram_summary.source_shape`:

- `channel` — преобладают публикации канала, announcement/news поток, пост как
  основная единица анализа;
- `chat` — преобладают сообщения участников, обсуждения, reply chains и темы;
- `mixed` — есть и канал, и обсуждения/чаты, либо несколько Telegram-источников
  разных типов.

**Отвечает на вопросы:**

- Что важного произошло в Telegram-источнике за период?
- Какие темы, события, claims и обсуждения доминировали?
- Какие сообщения стоит прочитать в первую очередь и почему?
- Какие reply chains / forum topics содержат полезную дискуссию?
- Какие утверждения повторяются, спорят друг с другом или требуют проверки?
- Какие forwarded messages повлияли на повестку?

**Не входит в задачу v1:**

- Полный OSINT-анализ авторов, организаций или сетей распространения.
- Глубокий reputation/person profiling участников.
- Полная social graph модель чата.
- Внешняя факт-проверка claims за пределами доступного корпуса.
- OCR/STT/понимание медиа без уже доступных captions, `mediaSummary` или
  текстовых metadata.
- Production monitoring, realtime alerts и anomaly detection как обязательная
  часть pack.

Pack может поставлять сигналы для будущих OSINT, reputation monitoring,
technology_watch или alerting packs, но сам остается source-specific summary
pack, ближе по роли к `youtube_summary`.

---

## 2. `run_context` настройки

### `control_preset`

`control_preset` управляет шириной и глубиной анализа.

| Значение | Поведение |
|---|---|
| `quick` | Короткий digest, timeline, top key messages. Темы и threads допускаются в сокращенной форме |
| `standard` | Полный MVP: digest, timeline, topics, key_messages, claims, threads, forwarded_items, limitations |
| `deep` | Как standard + более подробное ранжирование, противоречия, message_quality_signals и cross-source synthesis |

Правила:

- В `quick` mode `threads`, `claims`, `forwarded_items` и
  `message_quality_signals` могут быть пустыми, если digest и key messages
  покрывают период.
- В `standard` mode при `result_status = "complete"` и
  `evidence_mode != "narrative_only"` ожидаются непустые `digest`,
  `timeline` и `key_messages`, если в корпусе есть usable Telegram messages.
- В `deep` mode для `mixed` input ожидается `cross_source_synthesis`, если
  отсутствие сравнения не объяснено через `quality_flags`.

### `evidence_mode`

`evidence_mode` управляет строгостью traceability, а не типом результата.

| Значение | Поведение |
|---|---|
| `narrative_only` | Readable result first. `pack_data.telegram_summary` может быть частично пустым |
| `standard` | Structured result required. Важные объекты должны ссылаться на evidence или source refs |
| `strict` | Как standard + усиленные pipeline-level проверки покрытия claims, key messages и timeline |

Правила:

- В `narrative_only`, если structured блоки пустые при `result_status =
  "complete"`, нужен `quality_flag` с `flag = "partial_result"` или
  `flag = "corpus_coverage_limited"`.
- В `standard`, каждый `key_message`, `timeline_event`, `topic`, `thread` и
  `claim` должен иметь непустые `evidence_refs` или `message_refs`.
- В `strict`, каждый `claim` должен ссылаться минимум на одно evidence с
  locator до Telegram-сообщения или текстового диапазона внутри сообщения.
- В `strict`, synthesized conclusions должны иметь `claim_refs`, а не только
  prose без трассировки.

### Ожидаемые `source_types`

Основные:

- `telegram_post`;
- `telegram_channel_snapshot`;
- `telegram_chat_snapshot`.

Дополнительные источники могут присутствовать в общем canonical result, но
pack-specific блок `telegram_summary` не должен подменять ими Telegram anchor.
Если mixed run включает web/RSS/YouTube материалы, они могут использоваться как
контекстные source refs, но Telegram corpus остается главным предметом summary.

---

## 3. `outputs.pack_data.telegram_summary`

### Верхний уровень

```json
{
  "telegram_summary": {
    "source_shape": "mixed",
    "sources": [],
    "time_window": null,
    "digest": null,
    "timeline": [],
    "topics": [],
    "key_messages": [],
    "threads": [],
    "claims": [],
    "forwarded_items": [],
    "cross_source_synthesis": null,
    "limitations": []
  }
}
```

Правила:

- `source_shape` обязателен и принимает `channel`, `chat` или `mixed`.
- `sources` обязателен как массив, даже если source один.
- `time_window` обязателен как поле; `null` допустим, если период неизвестен.
- `digest` обязателен как поле; `null` допустим только для error/partial
  result или `narrative_only` с explanation в `limitations`.
- Все массивы обязательны как поля; пустой массив допустим, если блок
  неприменим, не запрошен текущим `control_preset` или корпус недостаточен.

### Объект `TelegramSummarySource`

```json
{
  "summary_source_id": "tg_source_1",
  "source_ref_id": "source_ref_1",
  "source_kind": "channel",
  "display_name": "Example Channel",
  "platform_id": "-1001234567890",
  "message_count": 42,
  "usable_message_count": 40,
  "topic_count": 3,
  "source_refs": ["source_ref_1"]
}
```

`source_kind` принимает:

- `channel`;
- `chat`;
- `supergroup`;
- `forum_topic`;
- `mixed`;
- `unknown`.

Правила:

- `summary_source_id` уникален внутри `telegram_summary.sources`.
- `source_ref_id` должен ссылаться на Telegram snapshot или root Telegram
  context, если он есть в canonical `source_refs`.
- Для multi-source runs каждый pack-level объект должен сохранять namespace
  через `summary_source_id` или `source_refs`, чтобы одинаковые Telegram
  `message_id` из разных источников не смешивались.

### Объект `TimeWindow`

```json
{
  "from": "2026-06-01T00:00:00Z",
  "to": "2026-06-07T23:59:59Z",
  "timezone": "UTC",
  "label": "2026-06-01..2026-06-07"
}
```

`from` и `to` заполняются только вместе. Если период сформирован по
имеющемуся корпусу, а не по пользовательскому запросу, это отражается в
`limitations`.

---

## 4. Основные вложенные объекты

### 4.1 `Digest`

```json
{
  "summary_text": "За период канал сфокусировался на...",
  "key_events": [],
  "major_changes": [],
  "recommended_reads": ["key_message_1", "key_message_3"],
  "claim_refs": [],
  "evidence_refs": [],
  "source_refs": []
}
```

`digest` — короткий readable вход в результат. Он должен быть полезен даже без
чтения всех структурных блоков.

Правила:

- `summary_text` не должен содержать неподтвержденных выводов без связи с
  `claim_refs` или `evidence_refs` при `evidence_mode != "narrative_only"`.
- `recommended_reads` ссылается только на `key_messages[].key_message_id`.
- `key_events` и `major_changes` могут быть prose-массивами или пустыми
  массивами в v1; детальная хронология живет в `timeline`.

### 4.2 `TimelineEvent`

```json
{
  "timeline_event_id": "timeline_event_1",
  "occurred_at": "2026-06-03T12:30:00Z",
  "title": "Запущено обсуждение новой политики",
  "summary_text": "Первый пост вызвал длинную ветку обсуждения...",
  "importance": "high",
  "message_refs": ["msg_ref_1", "msg_ref_2"],
  "claim_refs": ["claim_1"],
  "evidence_refs": ["evidence_1"],
  "source_refs": ["source_ref_1"]
}
```

`importance` принимает `low`, `medium`, `high`, `critical`, `unknown`.

Правила:

- Timeline сортируется по `occurred_at` по возрастанию, если даты известны.
- Если событие синтезировано из нескольких сообщений, `message_refs` включает
  все ключевые сообщения, но не обязан перечислять весь thread.
- В `strict` каждый timeline event должен иметь `evidence_refs`.

### 4.3 `Topic`

```json
{
  "topic_id": "topic_1",
  "title": "Релиз продукта и первые жалобы",
  "summary_text": "Тема объединяет официальный анонс и обсуждение проблем...",
  "topic_source": "reply_chain",
  "topic_scope": "single_source",
  "importance": "high",
  "message_count": 18,
  "key_message_refs": ["key_message_1"],
  "thread_refs": ["thread_1"],
  "claim_refs": ["claim_1", "claim_2"],
  "evidence_refs": ["evidence_1"],
  "source_refs": ["source_ref_1"]
}
```

`topic_source` принимает:

- `forum_topic`;
- `reply_chain`;
- `semantic_cluster`;
- `time_window`;
- `manual_query`;
- `unknown`.

`topic_scope` принимает `single_source`, `cross_source`, `unknown`.

Правила построения тем:

1. Если есть `forumTopicId` / `forum_topic_id`, topic строится вокруг него.
2. Иначе, если есть `reply_to_top_message_id` или root reply chain, topic
   строится вокруг ветки.
3. Иначе применяется semantic grouping по близости тем, entities, links,
   keywords и времени.
4. Multi-source topics не должны объединять сообщения разных источников только
   из-за одинакового Telegram `message_id`; нужен source namespace.

### 4.4 `KeyMessage`

```json
{
  "key_message_id": "key_message_1",
  "summary_source_id": "tg_source_1",
  "message_ref": "msg_ref_1",
  "message_id": "1842",
  "published_at": "2026-06-03T12:30:00Z",
  "author_display": "Example Channel",
  "role": "channel_post",
  "importance_score": 0.82,
  "importance_reasons": ["high_reaction_count", "started_thread", "claim_dense"],
  "summary_text": "Пост сформулировал основной тезис недели...",
  "contains_claims": true,
  "topic_refs": ["topic_1"],
  "thread_refs": ["thread_1"],
  "claim_refs": ["claim_1"],
  "evidence_refs": ["evidence_1"],
  "source_refs": ["source_ref_1"]
}
```

`role` принимает:

- `channel_post`;
- `chat_message`;
- `thread_root`;
- `reply`;
- `answer`;
- `question`;
- `forwarded_message`;
- `moderation_or_service`;
- `unknown`.

`importance_reasons` рекомендуется ограничивать значениями:

- `pinned`;
- `high_reaction_count`;
- `high_reply_count`;
- `high_forward_count`;
- `started_thread`;
- `resolved_question`;
- `claim_dense`;
- `novel_information`;
- `cross_source_repeated`;
- `risk_signal`;
- `useful_instruction`;
- `editorial_pick`;
- `unknown`.

Правила:

- `importance_score` — best-effort число `0..1`, не абсолютная истина.
- При отсутствии `view_count` нельзя штрафовать сообщение автоматически.
- Для чатов полезность сообщения может быть выше популярности: хороший ответ,
  решение проблемы или уточнение claim может ранжироваться выше шумного
  сообщения с реакциями.
- `author_display` можно показывать, если он присутствует в исходных данных и
  важен для анализа. Pack не должен пытаться деанонимизировать автора.

### 4.5 `ThreadSummary`

```json
{
  "thread_id": "thread_1",
  "summary_source_id": "tg_source_1",
  "root_message_ref": "msg_ref_1",
  "thread_kind": "reply_chain",
  "forum_topic_id": null,
  "forum_topic_title": null,
  "message_count": 12,
  "participant_count": 5,
  "summary_text": "Ветка началась с вопроса о...",
  "outcome": "partial_consensus",
  "key_message_refs": ["key_message_1", "key_message_2"],
  "claim_refs": ["claim_1"],
  "evidence_refs": ["evidence_1"],
  "source_refs": ["source_ref_1"]
}
```

`thread_kind` принимает `reply_chain`, `forum_topic`, `comment_discussion`,
`semantic_thread`, `unknown`.

`outcome` принимает:

- `answer_found`;
- `decision_made`;
- `partial_consensus`;
- `disagreement`;
- `unresolved`;
- `announcement_only`;
- `unknown`.

Правила:

- Thread должен сохранять reply chain context, а не просто перечислять похожие
  сообщения.
- Для forum topics `forum_topic_id` и `forum_topic_title` заполняются, если
  доступны.
- Для глубокой ветки pack может суммаризировать только ключевые сообщения, но
  должен указать ограничение в `limitations`, если значительная часть ветки не
  попала в анализ.

### 4.6 `TelegramClaim`

```json
{
  "telegram_claim_id": "telegram_claim_1",
  "claim_ref": "claim_1",
  "claim_text": "Команда заявила, что исправление выйдет до конца недели.",
  "claim_kind": "factual",
  "status_in_corpus": "supported_once",
  "topic_refs": ["topic_1"],
  "key_message_refs": ["key_message_1"],
  "message_refs": ["msg_ref_1"],
  "evidence_refs": ["evidence_1"],
  "source_refs": ["source_ref_1"]
}
```

`claim_kind` принимает:

- `factual`;
- `prediction`;
- `recommendation`;
- `opinion`;
- `rumor`;
- `question`;
- `unknown`.

`status_in_corpus` принимает:

- `supported_once`;
- `repeated`;
- `disputed`;
- `corrected`;
- `unverified`;
- `unknown`.

Правила:

- `telegram_claim_id` — pack-local id. `claim_ref` указывает на top-level
  canonical claim, если claim был вынесен в общий слой.
- Pack не подтверждает истинность claim во внешнем мире; он описывает статус
  claim внутри доступного корпуса.
- В `strict`, спорные claims должны иметь evidence минимум с двух разных
  сообщений или объясняющий `quality_flag`, если данных мало.

### 4.7 `ForwardedItem`

```json
{
  "forwarded_item_id": "forwarded_1",
  "message_ref": "msg_ref_4",
  "forwarded_from_channel_id": "source_channel_1",
  "forwarded_from_channel_title": "External Channel",
  "summary_text": "Пересланный пост принес в чат внешний тезис...",
  "impact": "started_discussion",
  "topic_refs": ["topic_2"],
  "claim_refs": ["claim_3"],
  "evidence_refs": ["evidence_4"],
  "source_refs": ["source_ref_1"]
}
```

`impact` принимает `context_only`, `repeated_claim`, `started_discussion`,
`amplified_topic`, `unknown`.

Правила:

- Forwarded item не должен автоматически считаться независимым подтверждением.
- Если forwarded origin неизвестен, поля origin остаются `null`, а
  `limitations` фиксирует ограничение.

### 4.8 `CrossSourceSynthesis`

```json
{
  "summary_text": "Канал и чат обсуждают одну тему с разными акцентами...",
  "agreements": [],
  "disagreements": [],
  "source_specific_notes": [],
  "claim_refs": [],
  "evidence_refs": [],
  "source_refs": []
}
```

Правила:

- Обязателен в `deep + mixed` complete result, если есть минимум два usable
  Telegram sources.
- Не должен смешивать channel authority и chat consensus: канал может
  публиковать claims, чат может обсуждать их качество, но это разные типы
  сигналов.

### 4.9 `Limitation`

```json
{
  "limitation_id": "limitation_1",
  "severity": "medium",
  "message": "Часть reply chains недоступна в корпусе.",
  "affected_blocks": ["threads", "claims"],
  "source_refs": ["source_ref_1"]
}
```

`severity` принимает `low`, `medium`, `high`.

Типичные limitations:

- отсутствуют views или counters;
- reactions неполные;
- reply chains собраны частично;
- forwarded origin неизвестен;
- migrated history может быть неполной;
- media доступна только через caption/summary;
- corpus coverage ограничен пользовательским фильтром.

---

## 5. Ранжирование важных сообщений

`importance_score` не является универсальной метрикой истины. Это локальный
сигнал для сортировки `key_messages`.

Рекомендуемая модель v1:

```text
importance_score =
  engagement_signal
  + propagation_signal
  + novelty_signal
  + thread_signal
  + claim_density_signal
  + usefulness_signal
  + risk_signal
```

Где:

- `engagement_signal` использует `reaction_count`, `reply_count`, доступные
  комментарии и активность вокруг сообщения;
- `propagation_signal` использует `forward_count`, `is_forwarded`, повторение
  claim в нескольких источниках;
- `novelty_signal` выделяет новую информацию внутри периода;
- `thread_signal` повышает root messages, ответы, решения и поворотные точки
  discussion;
- `claim_density_signal` повышает сообщения с проверяемыми claims;
- `usefulness_signal` повышает практические ответы, инструкции, ссылки,
  решения проблем;
- `risk_signal` повышает сообщения с конфликтом, спорным claim, urgent warning
  или высоким downstream impact.

Правила:

- `view_count` не должен быть обязательным входом, потому что текущий pipeline
  может не иметь reliable views.
- Для каналов важность сильнее опирается на post-level signals.
- Для чатов важность сильнее опирается на thread outcome и usefulness.
- Низкие реакции не означают низкую полезность, если сообщение закрывает
  важный вопрос.

---

## 6. Evidence, message refs и fragment locators

Pack-level `message_refs` — ссылки на Telegram messages в доступном корпусе.
Они могут быть реализованы как evidence IDs, fragment candidates или
pack-local refs, но должны быть однозначно связаны с source namespace и
Telegram message identity.

Рекомендуемый locator для evidence:

```json
{
  "fragment_type": "text_range",
  "source_ref_id": "source_ref_1",
  "locator_data": {
    "start": 0,
    "end": 128
  }
}
```

Дополнительная Telegram identity хранится в `source_ref.type_data.message_id`
или в pipeline metadata, если сообщение представлено как отдельный source item.

Правила:

- Нельзя использовать одно только `message_id` без source namespace в
  multi-source runs.
- Evidence для media-only сообщения допускается только если есть caption,
  `mediaSummary` или другая текстовая metadata.
- Цитаты должны быть короткими и трассируемыми. Длинная републикация сообщений
  не является целью pack.

---

## 7. `outputs.sections` — рекомендуемый readable pattern

Для `telegram_summary` рекомендуется следующий набор `outputs.sections`:

1. `summary` — короткая сводка периода.
2. `timeline` — хронология ключевых событий.
3. `topics` — основные темы и threads.
4. `key_messages` — сообщения, которые стоит открыть.
5. `claims_to_check` — спорные или важные утверждения.
6. `limitations` — ограничения корпуса и анализа.

Правила:

- `sections` должны ссылаться на structured objects через `claim_refs`,
  `evidence_refs` или pack-local refs, когда это возможно.
- В `narrative_only` `sections` могут быть основным результатом, но structured
  pack_data fields все равно должны присутствовать как поля.

---

## 8. Stage names

Pack-specific stage names используют namespace `{pack_id}/{stage_name}`.

Рекомендуемый набор:

| Stage | Назначение |
|---|---|
| `telegram_summary/message_normalization` | Нормализация Telegram messages, source namespaces, migrated history |
| `telegram_summary/thread_grouping` | Группировка forum topics, reply chains и semantic threads |
| `telegram_summary/topic_extraction` | Выделение тем и cross-topic links |
| `telegram_summary/key_message_scoring` | Ранжирование важных сообщений |
| `telegram_summary/claim_extraction` | Извлечение Telegram-specific claims и статуса внутри корпуса |
| `telegram_summary/synthesis` | Финальная сводка, timeline, key messages, limitations |

MVP может объединять эти шаги в один combined stage, если stage output
сохраняет тот же итоговый contract и traceability.

---

## 9. Validation rules draft

Pack-specific validation rules должны быть добавлены в `validation_rules.md`
после утверждения schema.

Предлагаемые правила:

| ID | Severity | Path | Rule |
|---|---|---|---|
| `VR-TS-001` | error | `pack_data.telegram_summary` | Поле присутствует при `pack_id = "telegram_summary"` |
| `VR-TS-002` | error | `source_shape` | Значение одно из `channel`, `chat`, `mixed` |
| `VR-TS-003` | error | `sources` | Непустой массив при complete structured result |
| `VR-TS-004` | error | `key_messages` | При complete `standard/deep` есть хотя бы один key message, если usable messages > 0 |
| `VR-TS-005` | error | refs | `key_messages[].message_ref` не должен ссылаться на другой source namespace |
| `VR-TS-006` | warning | `importance_score` | Есть score без `importance_reasons` |
| `VR-TS-007` | warning | `claims` | Claim имеет `status_in_corpus = "disputed"` без evidence из разных сообщений |
| `VR-TS-008` | warning | `threads` | Thread summary есть без `root_message_ref` и без объяснения в `limitations` |
| `VR-TS-009` | warning | `forwarded_items` | Forwarded item без origin не отражен в `limitations` |
| `VR-TS-010` | qa | `mixed` | Mixed result без `cross_source_synthesis` в `deep` mode |

---

## 10. Fixture strategy

Минимальный набор fixtures:

- `telegram_summary_channel_nominal` — один канал, несколько постов,
  reactions, forwarded post, timeline и key messages.
- `telegram_summary_chat_threads` — group/supergroup chat с reply chains,
  вопросами, ответами, шумом и полезными решениями.
- `telegram_summary_forum_topics` — supergroup с forum topics, где topic
  строится по `forum_topic_id`.
- `telegram_summary_multi_source_clash` — два источника с одинаковыми
  `message_id`, чтобы проверить namespace isolation.
- `telegram_summary_migrated_history` — current + migrated history с
  `historyScope`/migration metadata.
- `telegram_summary_media_caption_only` — media messages, где анализ возможен
  только по caption/summary.

Parser fixtures должны проверять:

- корректный JSON для `pack_data.telegram_summary`;
- отсутствие extra top-level keys;
- восстановление после malformed JSON;
- отказ от refs, не входящих в allowed fragment registry.

Validator fixtures должны проверять:

- valid minimal channel summary;
- valid minimal chat summary;
- invalid missing `source_shape`;
- invalid cross-source message ref;
- warning for disputed claim with weak evidence;
- warning for missing limitation on partial reply-chain coverage.

---

## 11. MVP scope

MVP считается достаточным, если pack умеет:

- принимать один или несколько Telegram sources;
- различать `channel`, `chat`, `mixed`;
- строить короткую сводку периода;
- строить timeline;
- выделять topics;
- ранжировать key messages;
- поддерживать reply chains / forum topics;
- выделять claims со статусом внутри корпуса;
- учитывать forwarded messages;
- явно фиксировать limitations;
- сохранять refs так, чтобы пользователь мог открыть исходные сообщения или
  связанные evidence.

Необязательные для MVP, но совместимые с v1 shape:

- детальная stance/narrative extraction;
- author/source graph;
- anomaly detection;
- scheduled monitoring;
- external fact-checking;
- advanced embeddings/clustering pipeline.

---

## 12. Open questions

### OQ-TS-01 — отдельный `message_quality_signals` блок

Текущий draft хранит качество/полезность через `key_messages.importance_score`
и `importance_reasons`. Если UI или downstream packs потребуют отдельный список
message-quality observations, можно добавить top-level
`message_quality_signals: []` в `pack_data.telegram_summary`.

### OQ-TS-02 — точная форма `message_refs`

Core contract пока не фиксирует отдельный Telegram message ref object в
pack-specific layer. Реализация может использовать evidence IDs или
stage-level fragment IDs. Перед machine-readable schema нужно выбрать
одинаковый ref style.

### OQ-TS-03 — глубина recursive thread summaries

Для очень длинных reply chains может понадобиться recursive summary внутри
thread. В v1 это оставлено как runtime strategy, а не как отдельная schema
форма.

### OQ-TS-04 — adaptive scoring weights

Веса importance scoring должны быть runtime-configurable или prompt-guided.
В v1 spec фиксирует сигналы и правила интерпретации, но не жесткие численные
коэффициенты.

