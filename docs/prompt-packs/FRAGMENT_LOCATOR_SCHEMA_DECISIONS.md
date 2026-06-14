# Fragment Locator Schema Decisions

Дата фиксации: 2026-06-07.

Документ фиксирует принятые решения по `fragment_locator_schemas.md` для
Prompt Pack JSON Contract v1. Это decision log к схемам `locator_data`, а не
сама JSON Schema.

## 1. Назначение

`fragment_locator_schemas.md` описывает структуру `evidence.locator_data`
для стандартных `fragment_type`.

Принятое разделение:

- общий контракт валидирует объект `evidence`;
- `fragment_locator_schemas.md` валидирует `locator_data` по значению
  `evidence.fragment_type`;
- inference evidence не имеет locator: `evidence_type = "inference"`,
  `fragment_type = null`, `locator_data = null`.

Обоснование:

- location semantics отличаются у текста, медиа, документов и изображений;
- общий контракт не должен раздуваться type-specific координатами;
- locator должен оставаться машинно проверяемым и воспроизводимым.

## 2. Строгий `locator_data`

Принято: `locator_data` является строгим объектом без `extra_metadata`.

Правила:

- неизвестные поля запрещены;
- все поля, описанные схемой конкретного locator type, присутствуют;
- неизвестное или неприменимое значение записывается как `null`;
- описательный контекст хранится в `evidence.fragment_text`,
  `evidence.context_text`, `evidence.reasoning_summary` и `provenance`;
- расширение координатной модели требует новой версии схемы.

Обоснование:

- произвольные расширения в координатах быстро ломают валидацию;
- `locator_data` отвечает на вопрос "где находится фрагмент", а не "что он значит";
- текстовый и аналитический контекст уже есть в объекте `evidence`.

## 3. Индексы и диапазоны

Принята смешанная, явно описанная конвенция:

- `paragraph_index`, `comment_index`, `reply_index`, `section_index` — 0-based;
- `page_number` — 1-based;
- `char_start` / `char_end` — 0-based Unicode codepoint offsets;
- text ranges используют `[start, end)` — inclusive start, exclusive end;
- media timestamps используют inclusive/inclusive boundaries;
- image coordinates используют relative `0.0–1.0`, origin top-left.

Обоснование:

- программные индексы естественно 0-based;
- номера страниц являются пользовательской нумерацией;
- Unicode codepoint offsets избегают неоднозначности UTF-8 bytes и UTF-16 code units;
- media ranges должны быть понятны пользователю и медиаплееру.

## 4. Element-level fragments

Для `post`, `comment`, `thread_reply` принято поле:

```json
{ "scope": "full" }
```

Семантика: весь элемент является evidence-фрагментом; sub-locator внутри
элемента в v1 не задаётся.

Для `comment` и `thread_reply`:

- `_id` предпочтителен для воспроизводимости;
- `_index` допустим, если platform ID недоступен;
- nested replies адресуются через platform ID, не через позицию в дереве.

## 5. `aggregate`

`fragment_type = "aggregate"` имеет минимальный `locator_data`:

```json
{
  "schema_version": "1.0",
  "fragment_count": 3
}
```

Реальные ссылки на составные фрагменты находятся в
`evidence.contributing_evidence_refs`.

Правила согласованности `fragment_count` и `contributing_evidence_refs`
являются pipeline-level validation, не standalone JSON Schema rules.

## 6. Покрытые `fragment_type` v1

Стандартные locator schemas v1:

- `text_range`;
- `paragraph`;
- `video_timestamp_range`;
- `audio_timestamp_range`;
- `post`;
- `comment`;
- `thread_reply`;
- `image_region`;
- `document_section`;
- `aggregate`.

## 7. Открытые вопросы

Открытые вопросы не блокируют v1 baseline:

- OCR text внутри `image_region` остаётся в `evidence.fragment_text`;
- multi-page document selection покрывается через `aggregate`;
- nested replies адресуются через platform ID;
- `snapshot_text_id` для `text_range` рекомендован, но не обязателен в v1.

## 8. Итог

`fragment_locator_schemas.md` готов как companion document v1.0 для
Prompt Pack JSON Contract v1.
