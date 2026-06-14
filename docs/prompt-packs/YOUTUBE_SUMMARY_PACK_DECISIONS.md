# youtube_summary Pack Decisions

Документ фиксирует принятые решения по `youtube_summary_pack_spec.md`.
Это decision log к pack-specific schema для `outputs.pack_data.youtube_summary`.

---

## 1. Назначение pack

`youtube_summary` предназначен для проверяемого summary одного или нескольких
YouTube-видео по transcript, metadata и evidence-фрагментам.

Pack не является:
- механизмом перепубликации transcript-а;
- OSINT-анализом автора или канала;
- playlist-level analysis без video-level anchors.

---

## 2. `videos[]` как основная модель

Pack использует массив:

```json
{
  "youtube_summary": {
    "videos": [],
    "synthesis": null
  }
}
```

Обоснование:
- single-video и multi-video runs имеют одну форму;
- каждый video object может напрямую ссылаться на свой `source_ref`;
- cross-video слой можно добавить без изменения формы `videos[]`.

---

## 3. `synthesis` как cross-video слой

`synthesis` является обязательным полем со значением `null | object`.

Правило:
- при одном видео `synthesis = null`;
- при нескольких видео `synthesis` может быть `null` только в `quick`,
  `narrative_only`, `error` или если отсутствие cross-video synthesis объяснено
  через `quality_flags`;
- в multi-video `standard` или `deep` result с `result_status = "complete"`
  и `evidence_mode != "narrative_only"` `synthesis` должен быть заполнен,
  если отсутствие не объяснено через `quality_flags`.
- если `synthesis = null`, вложенные traversal-поля внутри него отсутствуют;
  `claim_refs: []`, `evidence_refs: []` и другие поля появляются только когда
  `synthesis` является объектом.

Обоснование: multi-video summary не должен смешиваться с per-video object layer.

---

## 4. `segments`, не `chapters`

Используется термин `segments`, потому что не все смысловые блоки видео являются
YouTube chapters.

`creator_defined` фиксирует происхождение:
- `true` — реальный chapter автора видео;
- `false` — LLM/pipeline-generated segment;
- `null` — происхождение неизвестно.

Обоснование: слово `chapters` создало бы ложное ожидание, что структура пришла
из YouTube metadata.

---

## 5. `Video.source_ref_id` обязателен

Каждый `videos[]` object имеет:

```json
{
  "video_id": "video_ys_001",
  "source_ref_id": "source_ref_1",
  "source_refs": ["source_ref_1"]
}
```

`source_ref_id` — primary anchor к top-level `source_refs[]` с
`source_type = "youtube_video"`.

Обоснование: без этого anchor невозможно механически перейти от pack-specific
video object к `youtube_video.type_data`.

`Video.claim_refs` является union traversal field и включает nested `claim_refs`
из `segments`, `key_points`, `notable_quotes`, `action_items`, `open_questions`
и связанных synthesis objects, если они относятся к этому видео.

---

## 6. `control_preset` как глубина извлечения

Для `youtube_summary` значения означают:

- `quick` — summary и key points;
- `standard` — segments, key points, notable quotes;
- `deep` — полный набор, включая action items, open questions и synthesis.

Обоснование: для summary pack ширина анализа менее важна, чем глубина extracted
content objects.

---

## 7. `open_questions` vs top-level `unknowns`

`video.open_questions` — content-layer вопросы, поднятые в видео.

Top-level `unknowns` — pipeline-level пробелы анализа.

Один и тот же фрагмент может породить оба объекта, но они отвечают на разные
вопросы:
- “Что спросил или оставил открытым автор?”
- “Что pipeline не смог установить?”

`raised_by_speaker` имеет три состояния:

- `true` — speaker явно сформулировал вопрос;
- `false` — вопрос выведен pack-ом из содержания, но speaker не сформулировал его явно;
- `null` — происхождение вопроса неизвестно.

---

## 8. `notable_quotes` и copyright boundary

`notable_quotes` используются только как короткий verification/evidence слой.

Правила:
- одна quote не больше 50 слов;
- если `word_count` заполнен, он совпадает с количеством слов в `quote_text`
  по той же convention, что и ограничение 50 слов;
- quote не используется для восстановления transcript-а;
- каждая quote ссылается на top-level evidence через `evidence_refs`;
- top-level evidence для quote имеет `text_mode = "verbatim"` и
  `fragment_type = "video_timestamp_range"`;
- `fragment_type = "text_range"` допустим для transcript-backed `key_points`,
  но не заменяет media locator для `notable_quotes`;
- `speaker_id` зарезервирован как `null | string` для diarized transcripts;
- `speaker_label` остаётся human-readable display string или `null`;
- `quote.timestamp_start/end` — convenience metadata, а `evidence.locator_data`
  остаётся authoritative locator.

Обоснование: pack должен оставаться summary/verification инструментом,
а не механизмом перепубликации исходного материала.

---

## 9. Mapping pack objects к core contract

| Pack object | Core mapping |
|---|---|
| `segments` | Navigation/content structure; claims optional |
| `key_points` | Claims `factual` или `evaluative` |
| `notable_quotes` | Pack_data projection pointing to evidence `verbatim`; not claims by themselves |
| `action_items` | Claims `normative` |
| `open_questions` | Content-layer questions, not top-level `unknowns` |
| `synthesis.common_claims` | Synthesized cross-video claim, or readable grouping traced via videos/evidence |
| `synthesis.contradictions_across_videos` | `claim_relations` with `relation_type = "contradicts"` |

---

## 10. Traversal and authority rules

Rules accepted after review:

- `segment.evidence_refs` must point only to evidence inside the segment timestamp
  range; this is pipeline-level validation.
- If referenced claims have `claim.evidence_refs`, `key_point.evidence_refs`
  should include them for navigation.
- `synthesis.claim_refs`, `synthesis.relation_refs` and `synthesis.evidence_refs`
  are union traversal fields over nested synthesis objects.
- `synthesis.source_refs` uses the pack-specific algorithm: union
  `videos[].source_refs` for all `video_refs` mentioned in nested synthesis
  objects plus source refs reachable through referenced evidence/claims.
- `common_claim.claim_refs` points to a synthesized cross-video claim if one exists;
  it does not mix synthesized and per-video supporting claims.

### SD-YS-11: Speaker identity reserved in v1

`notable_quote` содержит `speaker_id: null | string` и
`speaker_label: null | string`. `speaker_id` стабилен внутри одного `Video`
object и нужен для diarized transcripts. Отдельный `speakers[]` block не
вводится в v1.

---

## 11. Pack-specific stages

Pack-specific stages v1:

- `youtube_summary/transcript_analysis`;
- `youtube_summary/segment_extraction`;
- `youtube_summary/key_point_extraction`;
- `youtube_summary/quote_extraction`;
- `youtube_summary/synthesis`.

Эти значения используются в `provenance.stage` claims, evidence и relations,
которые порождены соответствующими операциями pack-а.

---

## 12. Открытые вопросы

Открытые вопросы v1:

- ~~OQ-YS-01~~ — закрыт через `speaker_id` в `notable_quote`;
- OQ-YS-02 — достаточно ли текущей granularity `synthesis`;
- OQ-YS-03 — нужен ли отдельный playlist-level context.

Все три вопроса non-blocking для первой реализации pack.
