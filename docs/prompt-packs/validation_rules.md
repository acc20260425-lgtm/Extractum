# Prompt Pack Validation Rules — v1 Draft

Совместимость: Prompt Pack JSON Contract `schema_version: "1.0"`.

Этот документ собирает правила валидации, которые уже зафиксированы в основном
контракте, companion documents и pack-specific specs. Он не заменяет эти документы:
если правило описано подробнее в исходной spec, там остаётся авторитетная
семантика поля. Здесь фиксируется карта для validator implementation и QA.

---

## 1. Назначение и границы

`validation_rules.md` отвечает на вопросы:

- что считать ошибкой результата;
- что считать warning для downstream routing или review;
- какие проверки может выполнить JSON Schema;
- какие проверки требуют обхода связей между объектами;
- какие правила являются pack-specific.

Документ не задаёт:

- конкретный формат machine-readable JSON Schema;
- реализацию validator-а;
- workflow-статусы задач или human review overlay;
- новые поля контракта.

---

## 2. Таксономия правил

### Severity

| Severity | Семантика |
|---|---|
| `error` | Result невалиден для данного `schema_version` или pack-specific spec |
| `warning` | Result валиден, но требует внимания, routing или human review |
| `info` | Диагностическое сообщение, не влияющее на валидность |

### Validation layer

| Layer | Что проверяет |
|---|---|
| `schema` | Локальная форма объекта: обязательные ключи, типы, enum, nullable |
| `reference` | Существование referenced IDs и локальная область уникальности |
| `pipeline` | Cross-object правила, union-поля, graph checks, consistency rules |
| `qa` | Мягкие рекомендации, полнота, качество и human-review hints |

JSON Schema может покрывать только часть `schema` и простые `reference` проверки,
если validator имеет полный документ. Все `pipeline` правила требуют отдельного
обхода result graph.

Machine-readable JSON Schema failures emitted by the reference validator use a
generic stable rule ID. Specific prose rules such as `VR-CORE-*`, `VR-ST-*`,
`VR-FL-*`, `VR-TW-*`, and `VR-YS-*` remain authoritative for semantics.

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `SCHEMA-VALIDATION-001` | `error` | `schema` | JSON Schema selected object | A machine-readable JSON Schema selected through `schemas/v1/schema_manifest.json` failed against the current artifact or sub-object. The finding message names the logical schema ID, and `object_path` points to the failing property when available. |

Rule IDs являются стабильными идентификаторами правил. Нумерация отражает историю
добавления правила, а не порядок строк в документе; новые правила могут появляться
в уже существующих разделах с более поздним номером.

### Validation finding

Минимальная форма finding для validator v1:

```json
{
  "rule_id": "VR-CORE-006",
  "severity": "error",
  "layer": "pipeline",
  "object_path": "outputs.summary.claim_refs",
  "message": "Each claim_ref from summary.claim_refs must appear in at least one sections.items[].claim_refs entry.",
  "object_refs": {
    "claim_refs": ["claim_5"],
    "evidence_refs": [],
    "source_refs": []
  }
}
```

Поля `rule_id`, `severity`, `layer`, `object_path` и `message` обязательны.
`object_refs` обязателен как объект; массивы внутри него могут быть пустыми.
Реализация validator-а может добавлять дополнительные поля, но не должна менять
семантику этих базовых полей.

`object_path` использует JSON Pointer-like notation или human-readable dotted path.
Точный формат path определяется реализацией validator-а.

---

## 3. Порядок выполнения validator-а

Рекомендуемый порядок:

1. Core shape validation.
2. ID uniqueness и referential integrity.
3. Companion validation: `source_ref.type_data` и `evidence.locator_data`.
4. Core cross-object pipeline rules.
5. Final graph validation pass: DAG/acyclicity, derived refs, and pack graph rules.
6. Pack-specific validation по `pack_id`.
7. QA warnings и soft rules.

Если core shape validation падает с `error`, validator может остановиться раньше,
потому что дальнейшие graph checks могут быть ненадёжны.

---

## 4. Core contract rules

### 4.1 Envelope and required objects

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-001` | `error` | `schema` | envelope | Все top-level поля envelope присутствуют как ключи |
| `VR-CORE-002` | `error` | `schema` | envelope | `schema_version`, `pack_id`, `pack_version`, `run_id`, `result_id`, `stage` непустые |
| `VR-CORE-003` | `error` | `schema` | envelope | `parent_result_ids` присутствует и является `null` или непустым массивом уникальных строк |
| `VR-CORE-055` | `error` | `pipeline` | result lineage | Если validator имеет доступ к run-level result graph, `parent_result_ids` образует ациклический DAG; result не может быть своим прямым или транзитивным parent |
| `VR-CORE-004` | `error` | `schema` | top-level arrays | Все top-level массивы присутствуют как ключи; пустой массив допустим по правилам `stage` и `result_status` |
| `VR-CORE-005` | `error` | `schema` | `metadata`, `run_context`, `outputs` | Объекты `metadata`, `run_context`, `outputs` присутствуют и имеют обязательные поля своей схемы |

### 4.2 Outputs

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-006` | `error` | `schema` | `outputs.summary` | `outputs.summary` обязателен; если `metadata.result_status != "error"`, `summary_text` непустой |
| `VR-CORE-007` | `error` | `pipeline` | `outputs.summary.claim_refs` | Каждый `claim_ref` из `summary.claim_refs` присутствует хотя бы в одном `outputs.sections[].items[].claim_refs` |
| `VR-CORE-008` | `error` | `pipeline` | `outputs.sections` | При `stage = "final_result"` и `result_status = "complete"` массив `sections` непустой; пустой массив допустим только при условиях `VR-CORE-009` |
| `VR-CORE-009` | `error` | `pipeline` | `outputs.sections` | Если применяется exception для пустого `sections` при final complete, `quality_flags` содержит объясняющий код: `insufficient_data`, `processing_failed` или `partial_result` |
| `VR-CORE-010` | `warning` | `qa` | `outputs.sections.items` | `evidence_refs` в item желательно относятся к claims из того же item; exceptions допустимы для contextualizing/qualifying evidence |

### 4.3 Source refs

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-011` | `error` | `schema` | `source_refs[]` | `source_ref_id`, `source_type`, `access_status` присутствуют |
| `VR-CORE-012` | `error` | `schema` | `source_refs[]` | Хотя бы одно из `canonical_url` или `internal_uri` заполнено |
| `VR-CORE-013` | `error` | `schema` | `source_refs[].source_type` | `source_type` входит в enum v1 или равен `custom` с заполненным `custom_source_type` |
| `VR-CORE-014` | `warning` | `qa` | `source_refs[]` | `published_at` может быть `null`; фиктивные даты не должны подставляться |

### 4.4 Claims

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-015` | `error` | `reference` | `claims[].claim_id` | `claim_id` уникален внутри одного `result_id` |
| `VR-CORE-051` | `error` | `schema` | `claims[].claim_id` | `claim_id` имеет формат `prefix_N`: непустой строковый префикс, финальный `_`, числовой суффикс; numeric suffix после последнего `_` используется для natural sort |
| `VR-CORE-016` | `error` | `schema` | `claims[].claim_type` | `claim_type` входит в enum v1 или равен `custom` с заполненным `custom_claim_type` |
| `VR-CORE-017` | `error` | `pipeline` | `claims[].source_refs` | `claim.source_refs` является надмножеством source refs, достижимых через `claim.evidence_refs → evidence.source_ref_id` |
| `VR-CORE-018` | `error` | `pipeline` | `claims[].claim_status` | `claim_status = "inferred"` требует хотя бы одно из `evidence_refs`, `source_refs` или `relation_refs` |
| `VR-CORE-019` | `warning` | `qa` | `claims[].relation_refs` | `claim.relation_refs` является derived traversal field; authoritative source — top-level `claim_relations` |
| `VR-CORE-020` | `error` | `schema` | `claims[].normalized_claim_text` | Если `normalized_claim_text` заполнен, `normalized_claim_language = "en"`; если `normalized_claim_text = null`, `normalized_claim_language = null` |

### 4.5 Evidence

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-021` | `error` | `reference` | `evidence[].evidence_id` | `evidence_id` уникален внутри одного `result_id` |
| `VR-CORE-022` | `error` | `reference` | `evidence[].claim_id` | `evidence.claim_id` ссылается на существующий claim |
| `VR-CORE-023` | `error` | `schema` | `evidence[]` | `evidence_role` присутствует и входит в enum v1 или `custom` |
| `VR-CORE-024` | `error` | `schema` | `evidence[]` | Для fragment evidence `source_ref_id`, `fragment_type`, `locator_data` заполнены |
| `VR-CORE-025` | `error` | `schema` | `evidence[]` | Для inference evidence `fragment_type = null`; `locator_data = null`; `reasoning_summary` заполнен |
| `VR-CORE-026` | `error` | `pipeline` | `evidence[].contributing_evidence_refs` | `contributing_evidence_refs` формирует ациклический граф и не содержит self-reference |
| `VR-CORE-027` | `warning` | `qa` | `evidence[].context_text` | `context_text` без `fragment_text` допустим только для inference или media-like fragment types |
| `VR-CORE-050` | `error` | `pipeline` | `claims[].evidence_refs` | Каждый `evidence_id` в `claim.evidence_refs` указывает на evidence, чей `evidence.claim_id` равен этому `claim.claim_id`; один evidence объект не используется как direct evidence разных claims |

### 4.6 Claim relations

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-028` | `error` | `reference` | `claim_relations[].relation_id` | `relation_id` уникален внутри одного `result_id` |
| `VR-CORE-029` | `error` | `reference` | `claim_relations[]` | `source_claim_id` и `target_claim_id` ссылаются на существующие claims |
| `VR-CORE-030` | `error` | `schema` | `claim_relations[].relation_type` | `relation_type` входит в enum v1 или `custom` с заполненным `custom_relation_type` |
| `VR-CORE-031` | `error` | `pipeline` | `claim_relations[]` | Для `relation_type = "contradicts"` `source_claim_id` идёт раньше `target_claim_id` по natural sort `claim_id`; порядок опирается на формат `prefix_N`, enforced в `VR-CORE-051` |
| `VR-CORE-032` | `error` | `pipeline` | `claim_relations[].evidence_refs` | Relation evidence ссылается только на evidence, чей `claim_id` равен `source_claim_id` или `target_claim_id`; это правило применяется ко всем `evidence_type`, включая `inference`; для `contradicts` relation обычно ссылается на evidence обеих сторон, но каждый claim по-прежнему ссылается напрямую только на свой evidence по `VR-CORE-050` |
| `VR-CORE-033` | `error` | `schema` | `claim_relations[].description` | `description` обязателен для `contradicts` и `qualifies` |
| `VR-CORE-054` | `error` | `pipeline` | `claim_relations[]` | Directed graph по `relation_type = "qualifies"` и `relation_type = "supersedes"` ацикличен; inverse/duplicate `contradicts` предотвращается `VR-CORE-031`; cycles из `supports` являются QA concern, не hard error v1 |

### 4.7 Unknowns, verification tasks, warnings, limitations, quality flags

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-034` | `error` | `schema` | `unknowns[]` | `unknown_type` входит в enum v1 или `custom` |
| `VR-CORE-035` | `error` | `pipeline` | `unknowns[]` | Unknown имеет хотя бы один контекст: непустой `claim_refs`, `source_refs`, `evidence_refs` или `relation_refs`; глобальный пробел результата без локальных ссылок допустим только если он представлен как `limitation` или `quality_flag`, а не как context-free `unknown` |
| `VR-CORE-052` | `info` | `qa` | `unknowns`, `limitations` | Семантическая граница `unknown` vs `limitation` зафиксирована в decisions/spec prose; validator v1 проверяет локальный контекст `unknown`, но не пытается автоматически классифицировать prose как limitation или unknown |
| `VR-CORE-036` | `warning` | `qa` | `unknowns[].confidence` | `confidence.score` означает уверенность в корректности идентификации пробела, не истинность claim |
| `VR-CORE-037` | `error` | `pipeline` | `verification_tasks[]` | Verification task ссылается на `unknown_id` или имеет хотя бы один `claim_refs`/`evidence_refs` |
| `VR-CORE-038` | `error` | `schema` | `warnings[]` | `warning_type` входит в enum warning types или `custom` |
| `VR-CORE-039` | `error` | `schema` | `limitations[]` | `limitation_type` входит в enum limitation types или `custom` |
| `VR-CORE-040` | `error` | `schema` | `quality_flags[]` | `flag` входит в enum quality flags или `custom` |
| `VR-CORE-041` | `warning` | `qa` | `warnings`, `quality_flags` | `warning_type = "single_source_claim"` желательно сопровождается `quality_flags.flag = "single_source_claim"` |

### 4.8 Metadata consistency

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-042` | `error` | `pipeline` | `metadata`, `quality_flags` | `metadata.contains_unverified_claims = true` требует `quality_flags.flag = "unverified_claims_present"` |
| `VR-CORE-043` | `error` | `pipeline` | `metadata`, `quality_flags` | `metadata.contains_partial_results = true` требует `quality_flags.flag = "partial_result"` |
| `VR-CORE-044` | `error` | `pipeline` | `metadata`, `quality_flags` | `metadata.result_status = "partial"` требует `quality_flags.flag = "partial_result"` |
| `VR-CORE-045` | `error` | `pipeline` | `metadata`, `quality_flags` | `metadata.result_status = "error"` требует `quality_flags.flag = "processing_failed"` или pack-specific equivalent |
| `VR-CORE-046` | `info` | `qa` | `quality_flags` | Обратное не требуется: локальный `quality_flags` не обязан иметь metadata boolean |

### 4.9 Audit refs

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-CORE-047` | `error` | `schema` | `audit_refs[]` | `audit_id`, `event_type`, `stage`, `timestamp` присутствуют |
| `VR-CORE-048` | `error` | `schema` | `audit_refs[].event_type` | `event_type` входит в enum v1 или `custom` с заполненным `custom_event_type` |
| `VR-CORE-049` | `warning` | `qa` | `audit_refs[].audit_store` | `audit_store` является project-local locator или human-readable identifier, не universal URL |

---

## 5. Companion document rules

### 5.1 `source_type_schemas.md`

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-ST-001` | `error` | `schema` | `source_ref.type_data` | Для стандартного `source_type` `type_data` присутствует и валидируется по `source_type` |
| `VR-ST-002` | `error` | `schema` | `type_data` | Все описанные поля схемы присутствуют; неизвестное или неприменимое значение записывается как `null` |
| `VR-ST-003` | `error` | `schema` | `type_data.extra_metadata` | `extra_metadata` присутствует как `{}` или содержит только JSON-примитивы/массивы примитивов |
| `VR-ST-004` | `error` | `schema` | `creator` | `creator.creator_type = "custom"` требует `custom_creator_type` |
| `VR-ST-005` | `error` | `schema` | `parent_context` | `parent_context.context_type = "custom"` требует `custom_context_type` |
| `VR-ST-006` | `warning` | `qa` | `parent_context` | `context_type = null` означает root context, не ошибку |
| `VR-ST-YT-001` | `error` | `schema` | `youtube_video.type_data` | `creator.creator_type = "channel"` и `parent_context.context_type = "youtube_channel"` |
| `VR-ST-YT-002` | `error` | `pipeline` | `youtube_video.type_data` | Если `playlist_id = null`, то `playlist_title = null` и `playlist_position = null`; если `playlist_id` заполнен, зависимые поля согласованы |
| `VR-ST-RSS-001` | `error` | `schema` | `rss_entry.type_data` | `content_mode` отражает RSS-specific форму контента и не заменяется collection status |
| `VR-ST-TG-001` | `warning` | `qa` | `telegram_post.type_data` | Если `is_forwarded = true`, `forwarded_from_channel_id` желательно заполнен |
| `VR-ST-SNAP-001` | `error` | `schema` | Telegram snapshot types | `snapshot_period_start` и `snapshot_period_end` заполняются только вместе |
| `VR-ST-FORUM-001` | `warning` | `qa` | `forum_thread.type_data` | `vote_score` является compact cross-platform field; platform-specific vote details живут в `extra_metadata` |

### 5.2 `fragment_locator_schemas.md`

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-FL-001` | `error` | `schema` | `locator_data` | Для стандартного `fragment_type` `locator_data` присутствует и валидируется по `fragment_type` |
| `VR-FL-002` | `error` | `schema` | `locator_data` | `locator_data.schema_version` присутствует |
| `VR-FL-TEXT-001` | `error` | `schema` | `text_range` | `char_start` и `char_end` — Unicode codepoint offsets; `char_end > char_start` |
| `VR-FL-TEXT-002` | `warning` | `qa` | `text_range` | `snapshot_text_id` рекомендован для воспроизводимости offsets |
| `VR-FL-PAR-001` | `error` | `schema` | `paragraph` | `paragraph_index` 0-based; если `paragraph_count` заполнен, index меньше count |
| `VR-FL-TS-001` | `error` | `schema` | media timestamp range | `timestamp_start` и `timestamp_end` inclusive/inclusive; `timestamp_end >= timestamp_start` |
| `VR-FL-FULL-001` | `error` | `schema` | `post`, `comment`, `thread_reply` | `scope = "full"` |
| `VR-FL-COMMENT-001` | `error` | `schema` | `comment` | Хотя бы одно из `comment_id` или `comment_index` заполнено |
| `VR-FL-REPLY-001` | `error` | `schema` | `thread_reply` | Хотя бы одно из `reply_id` или `reply_index` заполнено |
| `VR-FL-ID-001` | `warning` | `qa` | `comment`, `thread_reply` | Platform ID предпочтительнее index для воспроизводимости |
| `VR-FL-IMG-001` | `error` | `schema` | `image_region` | `x`, `y`, `width`, `height` в диапазоне `[0.0, 1.0]`; region не выходит за границы изображения |
| `VR-FL-DOC-001` | `error` | `schema` | `document_section` | Хотя бы одно из `section_heading`, `page_number`, `section_index` заполнено |
| `VR-FL-AGG-001` | `error` | `pipeline` | `aggregate` | `evidence.contributing_evidence_refs` непустой |
| `VR-FL-AGG-002` | `error` | `pipeline` | `aggregate` | Если `fragment_count` заполнен, он равен длине `evidence.contributing_evidence_refs` для того evidence, чей `fragment_type = "aggregate"` |

---

## 6. Pack-specific rules

### 6.1 `technology_watch`

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-TW-001` | `error` | `schema` | `pack_data.technology_watch` | `technologies` присутствует как массив |
| `VR-TW-002` | `error` | `pipeline` | `technology_watch` | При `result_status = "complete"` и `evidence_mode != "narrative_only"` массив `technologies` непустой |
| `VR-TW-003` | `error` | `schema` | `technology.maturity` | Каждая technology имеет заполненный `maturity.level` |
| `VR-TW-004` | `error` | `pipeline` | `technology.maturity` | Если `maturity.level != "unknown"`, `maturity.claim_refs` непустой |
| `VR-TW-005` | `error` | `pipeline` | `technology.signals[]` | Каждый `signal` имеет хотя бы одну ссылку из `claim_refs`, `evidence_refs`, `source_refs` |
| `VR-TW-006` | `error` | `pipeline` | `technology` | `technology.claim_refs`, `evidence_refs`, `source_refs` являются union traversal fields по правилам spec |
| `VR-TW-007` | `error` | `pipeline` | `outputs.sections` | Если `technologies` непустой, `assessment` section присутствует и содержит item про maturity |
| `VR-TW-008` | `error` | `pipeline` | `evidence_mode = "strict"` | Только при `evidence_mode = "strict"` maturity level, отличный от `unknown`, требует минимум 2 независимых claims |
| `VR-TW-009` | `error` | `pipeline` | `evidence_mode = "strict"` | Только при `evidence_mode = "strict"` `limited_production` и `production` требуют coverage минимум из 2 уникальных source refs |
| `VR-TW-010` | `warning` | `qa` | `technology.maturity` | Если `maturity.level = "unknown"`, желательно иметь соответствующий top-level `unknown` |
| `VR-TW-011` | `warning` | `qa` | `claims` | Если есть `partially_verified` claims, желательно иметь соответствующий top-level `unknown` |
| `VR-TW-012` | `warning` | `qa` | `recommendations` | `production` или `limited_production` maturity желательно сопровождается recommendation |
| `VR-TW-013` | `warning` | `qa` | `recommendations` | `deprecated` maturity желательно сопровождается `recommendation_type = "avoid"` или `"deprecate"` |

### 6.2 `youtube_summary`

| Rule ID | Severity | Layer | Scope | Rule |
|---|---|---|---|---|
| `VR-YS-001` | `error` | `schema` | `pack_data.youtube_summary` | `videos` и `synthesis` присутствуют как поля |
| `VR-YS-002` | `error` | `pipeline` | `youtube_summary` | При `result_status = "complete"` и `evidence_mode != "narrative_only"` массив `videos` непустой |
| `VR-YS-003` | `error` | `reference` | `videos[]` | Каждый `Video.source_ref_id` ссылается на top-level `source_ref` с `source_type = "youtube_video"` |
| `VR-YS-004` | `error` | `pipeline` | `videos[]` | `Video.source_refs` включает `Video.source_ref_id` |
| `VR-YS-020` | `error` | `pipeline` | `videos[]` | `Video.claim_refs`, `Video.evidence_refs`, `Video.source_refs` являются union traversal fields по правилам pack spec; `Video.claim_refs` включает nested `claim_refs` из `segments`, `key_points`, `notable_quotes`, `action_items`, `open_questions` и связанных synthesis objects |
| `VR-YS-005` | `error` | `pipeline` | `synthesis` | При одном видео `synthesis = null` |
| `VR-YS-021` | `error` | `schema` | `synthesis` | Если `synthesis = null`, вложенные поля `claim_refs`, `relation_refs`, `evidence_refs`, `source_refs` отсутствуют; `VR-YS-015` применяется только когда `synthesis` является объектом |
| `VR-YS-006` | `error` | `pipeline` | `deep multi-video` | Только в `deep` mode multi-video complete result требует `synthesis`, если отсутствие не объяснено `quality_flags` |
| `VR-YS-018` | `error` | `pipeline` | `standard multi-video` | В `standard` mode multi-video complete result требует `synthesis`, если `evidence_mode != "narrative_only"` и отсутствие не объяснено `quality_flags` |
| `VR-YS-007` | `error` | `pipeline` | `key_points[]` | В `standard` и `strict` каждый `key_point` имеет непустой `claim_refs` |
| `VR-YS-008` | `warning` | `qa` | `key_points[]` | Если referenced claims имеют evidence, `key_point.evidence_refs` желательно включает их union |
| `VR-YS-009` | `error` | `pipeline` | `notable_quotes[]` | Каждая quote имеет непустой `evidence_refs` |
| `VR-YS-010` | `error` | `pipeline` | `notable_quotes[]` | Quote evidence имеет `text_mode = "verbatim"` и `fragment_type = "video_timestamp_range"`; `text_range` transcript evidence допустим для `key_points`, но не заменяет media locator для `notable_quotes` |
| `VR-YS-011` | `error` | `pipeline` | `notable_quotes[]` | `quote_text` не превышает 50 слов |
| `VR-YS-012` | `warning` | `qa` | `notable_quotes[]` | `quote.timestamp_start/end` являются convenience; authoritative locator — `evidence.locator_data` |
| `VR-YS-019` | `error` | `pipeline` | `notable_quotes[].word_count` | Если `word_count` заполнен, он совпадает с количеством слов в `quote_text` по той же word-counting convention, которая используется для `VR-YS-011` |
| `VR-YS-013` | `error` | `pipeline` | `segments[]` | `segment.evidence_refs` ссылается только на evidence внутри timestamp range segment-а; это heavy pipeline check, потому что требует обхода referenced evidence и проверки `locator_data` timestamp-диапазонов |
| `VR-YS-014` | `error` | `pipeline` | `synthesis.common_claims[]` | `common_claim.claim_refs` не смешивает synthesized cross-video claim и supporting per-video claims |
| `VR-YS-015` | `error` | `pipeline` | `synthesis` | `synthesis.claim_refs`, `relation_refs`, `evidence_refs` являются union nested synthesis objects; `synthesis.source_refs` собирается по алгоритму pack spec: `videos[].source_refs` для всех referenced `video_refs` плюс source refs, достижимые через referenced evidence/claims |
| `VR-YS-016` | `warning` | `qa` | `open_questions[]` | `raised_by_speaker = false` означает pack-inferred question; не pipeline unknown |
| `VR-YS-017` | `warning` | `qa` | `standard multi-video` | Если в multi-video `standard` mode `synthesis = null` разрешён через `quality_flags`, и один и тот же `claim_id` встречается в `key_points[].claim_refs` у двух или более разных `Video` объектов, отсутствие `synthesis` даёт warning |

---

## 7. Non-goals v1

Validator v1 не обязан:

- проверять истинность claims;
- проверять copyright beyond quote length rule;
- вычислять доверительные интервалы confidence;
- выполнять media authenticity checks;
- выполнять workflow lifecycle для `verification_tasks`;
- дедуплицировать entities между packs;
- валидировать содержимое `outputs.pack_data[pack_id]` без pack-specific spec.

---

## 8. Открытые вопросы

### ~~OQ-VR-01~~ — CLOSED: Machine-readable schema placement

Machine-readable JSON Schema placement is fixed in `schemas/README.md`.
Executable schema files live under `docs/prompt-packs/schemas/v1/`, with dynamic
loading through `schemas/v1/schema_manifest.json`. The bundle is incremental:
entries marked `semantic` have reviewed local-shape coverage, while skeleton
entries only fix loader paths and `$id` values.

### ~~OQ-VR-02~~ — ЗАКРЫТ: Validation finding format

Минимальная форма validation finding зафиксирована в разделе 2:
`rule_id`, `severity`, `layer`, `object_path`, `message`, `object_refs`.

### OQ-VR-03: Rule versioning

Нужно ли rule-specific versioning поверх `schema_version` и pack version,
если validation logic меняется без изменения структуры контракта.
