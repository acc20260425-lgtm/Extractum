# fragment_locator_schemas.md — v1

Версионируется вместе с `schema_version` основного контракта.
Текущая версия: `1.0`.

---

## 1. Назначение

Этот документ определяет структуру поля `locator_data` для каждого стандартного
`fragment_type` из Prompt Pack JSON Contract v1.

`locator_data` отвечает на вопрос: **где именно в material находится evidence-фрагмент**.
Общий контракт валидирует наличие `locator_data`; этот документ валидирует его содержимое
по значению `fragment_type`.

---

## 2. Общие правила `locator_data`

- `locator_data` валидируется по значению `fragment_type`.
- Для `evidence_type = inference` или `fragment_type = null` поле `locator_data = null`.
- `locator_data` — строгий объект: неизвестные поля запрещены.
- `extra_metadata` в `locator_data` не предусмотрен: координаты должны быть
  валидируемыми и предсказуемыми. Описательный контекст фрагмента размещается
  в `evidence.fragment_text`, `evidence.context_text`, `evidence.reasoning_summary`.
- Все поля, описанные схемой типа, обязаны присутствовать.
  Неизвестное или неприменимое значение — `null`, не отсутствие поля.

---

## 3. Общие соглашения

| Тема | Правило |
|---|---|
| `paragraph_index`, `comment_index`, `reply_index`, `section_index` | 0-based |
| `page_number` | 1-based |
| `char_start` / `char_end` | Unicode codepoint offsets, 0-based, `[start, end)` — inclusive start, exclusive end |
| `timestamp_start` / `timestamp_end` | секунды от начала медиа, inclusive/inclusive |
| `image_region` координаты | относительные `0.0–1.0`, origin top-left |
| `scope: "full"` | весь элемент целиком; sub-locator внутри элемента не задаётся |

---

## 4. Правила для `evidence_type = inference`

Если `evidence_type = inference`:
- `fragment_type = null`
- `locator_data = null`
- Источники фрагментов описываются через `contributing_evidence_refs`
  и `reasoning_summary` в объекте `evidence`.

---

## 5. Схемы стандартных `fragment_type`

---

### 5.1 `text_range`

Диапазон символов внутри текстового snapshot материала.

**Обязательные поля (не null):** `schema_version`, `char_start`, `char_end`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `char_start` | integer | да | 0-based, inclusive — начало диапазона |
| `char_end` | integer | да | 0-based, exclusive — конец диапазона `[start, end)` |
| `snapshot_text_id` | string\|null | нет | ID текстового snapshot, к которому привязаны offsets |

**Правила валидации:**
- `char_end > char_start`.
- `char_start >= 0`.
- `char_start` / `char_end` считаются в Unicode codepoints нормализованного текста:
  не UTF-8 byte offsets и не UTF-16 code units/surrogate pairs.
- `snapshot_text_id` рекомендован: без него char offsets могут стать
  невоспроизводимыми при изменении текста материала.

**Пример:**
```json
{
  "schema_version": "1.0",
  "char_start": 1420,
  "char_end": 1587,
  "snapshot_text_id": "snapshot_txt_001"
}
```

---

### 5.2 `paragraph`

Конкретный параграф в тексте материала.

**Обязательные поля (не null):** `schema_version`, `paragraph_index`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `paragraph_index` | integer | да | 0-based позиция параграфа в документе |
| `paragraph_count` | integer\|null | нет | общее число параграфов (для контекста валидации) |

**Правила валидации:**
- `paragraph_index >= 0`.
- Если `paragraph_count` заполнен: `paragraph_index < paragraph_count`.

**Пример:**
```json
{
  "schema_version": "1.0",
  "paragraph_index": 7,
  "paragraph_count": 24
}
```

---

### 5.3 `video_timestamp_range`

Временной диапазон внутри видеоматериала.

**Обязательные поля (не null):** `schema_version`, `timestamp_start`, `timestamp_end`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `timestamp_start` | number | да | секунды от начала видео, inclusive |
| `timestamp_end` | number | да | секунды от начала видео, inclusive |

**Правила валидации:**
- `timestamp_start >= 0`.
- `timestamp_end >= timestamp_start`.

**Пример:**
```json
{
  "schema_version": "1.0",
  "timestamp_start": 312.5,
  "timestamp_end": 346.0
}
```

---

### 5.4 `audio_timestamp_range`

Временной диапазон внутри аудиоматериала. Идентичная структура с `video_timestamp_range`.

**Обязательные поля (не null):** `schema_version`, `timestamp_start`, `timestamp_end`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `timestamp_start` | number | да | секунды от начала аудио, inclusive |
| `timestamp_end` | number | да | секунды от начала аудио, inclusive |

**Правила валидации:** идентичны `video_timestamp_range`.

**Пример:**
```json
{
  "schema_version": "1.0",
  "timestamp_start": 88.0,
  "timestamp_end": 124.5
}
```

---

### 5.5 `post`

Весь пост целиком как единица evidence. Применяется когда источник является
единичным постом (`telegram_post`, целой записью блога и т.д.)
и фрагментирование внутри поста не требуется.

**Обязательные поля (не null):** `schema_version`, `scope`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `scope` | string | да | всегда `"full"` в v1 |

**Правила валидации:**
- `scope` должен быть `"full"` в v1.

**Пример:**
```json
{
  "schema_version": "1.0",
  "scope": "full"
}
```

---

### 5.6 `comment`

Конкретный комментарий внутри material. Применяется для идентификации
отдельного комментария в контексте родительского post/page.

**Обязательные поля (не null):** `schema_version`, `scope`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `scope` | string | да | всегда `"full"` в v1 |
| `comment_id` | string\|null | нет | platform ID комментария |
| `comment_index` | integer\|null | нет | 0-based позиция в плоском порядке верхнеуровневых комментариев |

**Правила валидации:**
- `scope` должен быть `"full"` в v1.
- Хотя бы одно из `comment_id` или `comment_index` должно быть заполнено.
- Если `comment_index` заполнен: `comment_index >= 0`.
- `comment_id` предпочтителен для воспроизводимости; `comment_index` допустим,
  когда platform ID недоступен.
- Вложенные ответы не адресуются через `comment_index`; для них используется
  `comment_id`, если platform ID доступен.

**Пример:**
```json
{
  "schema_version": "1.0",
  "scope": "full",
  "comment_id": "comment_abc123",
  "comment_index": 14
}
```

---

### 5.7 `thread_reply`

Конкретный ответ внутри форумного треда или discussion thread.

**Обязательные поля (не null):** `schema_version`, `scope`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `scope` | string | да | всегда `"full"` в v1 |
| `reply_id` | string\|null | нет | platform ID ответа |
| `reply_index` | integer\|null | нет | 0-based позиция в треде |

**Правила валидации:**
- `scope` должен быть `"full"` в v1.
- Хотя бы одно из `reply_id` или `reply_index` должно быть заполнено.
- Если `reply_index` заполнен: `reply_index >= 0`.
- `reply_id` предпочтителен для воспроизводимости; `reply_index` допустим,
  когда platform ID недоступен.

**Пример:**
```json
{
  "schema_version": "1.0",
  "scope": "full",
  "reply_id": "t1_xyz789",
  "reply_index": 3
}
```

---

### 5.8 `image_region`

Прямоугольная область внутри изображения. Координаты относительные,
независимы от разрешения.

**Обязательные поля (не null):** `schema_version`, `x`, `y`, `width`, `height`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `x` | number | да | левый край, `0.0–1.0`, origin top-left |
| `y` | number | да | верхний край, `0.0–1.0` |
| `width` | number | да | ширина региона, `0.0–1.0` |
| `height` | number | да | высота региона, `0.0–1.0` |
| `snapshot_width_px` | integer\|null | нет | ширина snapshot в пикселях для обратного вычисления |
| `snapshot_height_px` | integer\|null | нет | высота snapshot в пикселях |

**Правила валидации:**
- Все четыре координаты в диапазоне `[0.0, 1.0]`.
- `x >= 0`, `y >= 0`.
- `x + width <= 1.0`.
- `y + height <= 1.0`.
- `width > 0`, `height > 0`.
- Если `snapshot_width_px` заполнен, `snapshot_height_px` тоже должен быть заполнен.

**Пример:**
```json
{
  "schema_version": "1.0",
  "x": 0.12,
  "y": 0.34,
  "width": 0.45,
  "height": 0.18,
  "snapshot_width_px": 1920,
  "snapshot_height_px": 1080
}
```

---

### 5.9 `document_section`

Секция внутри структурированного документа (PDF, Word, HTML-документ).

**Обязательные поля (не null):** `schema_version`.
Хотя бы одно из `section_heading`, `page_number`, `section_index` должно быть заполнено.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `section_heading` | string\|null | нет | заголовок секции |
| `page_number` | integer\|null | нет | 1-based номер страницы |
| `section_index` | integer\|null | нет | 0-based позиция среди секций документа |
| `page_count` | integer\|null | нет | общее число страниц документа |

**Правила валидации:**
- Хотя бы одно из `section_heading`, `page_number`, `section_index` не `null`.
- `page_number >= 1` если заполнен.
- `section_index >= 0` если заполнен.
- Если `page_count` заполнен и `page_number` заполнен: `page_number <= page_count`.

**Пример:**
```json
{
  "schema_version": "1.0",
  "section_heading": "Results and Discussion",
  "page_number": 8,
  "section_index": 3,
  "page_count": 24
}
```

**Пример PDF без структурированных секций:**
```json
{
  "schema_version": "1.0",
  "section_heading": null,
  "page_number": 12,
  "section_index": null,
  "page_count": 48
}
```

---

### 5.10 `aggregate`

Агрегированный фрагмент, составленный из нескольких sub-фрагментов.
Конкретные координаты не задаются — ссылки на составные фрагменты
хранятся в `evidence.contributing_evidence_refs`.

**Обязательные поля (не null):** `schema_version`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `fragment_count` | integer\|null | нет | количество составных фрагментов |

**Правила валидации:**
- Если `fragment_count` заполнен: `fragment_count > 0`.

**Pipeline-level правила консистентности:**
- `evidence.contributing_evidence_refs` должен быть непустым,
  если `fragment_type = "aggregate"`.
- Если `fragment_count` заполнен: он должен совпадать с длиной
  `evidence.contributing_evidence_refs`.
- Эти правила требуют доступа к родительскому объекту `evidence` и проверяются
  pipeline-валидатором результата, а не standalone JSON Schema для `locator_data`.

**Пример:**
```json
{
  "schema_version": "1.0",
  "fragment_count": 3
}
```

---

## 6. Принятые решения

### SD-FL-01 — `locator_data` без `extra_metadata`

`locator_data` — строгий валидируемый объект без escape-hatch.
Описательный контекст фрагмента размещается в полях `evidence`:
`fragment_text`, `context_text`, `reasoning_summary`.
Расширение координатной системы требует новой версии схемы.

### SD-FL-02 — Смешанная конвенция индексов

`paragraph_index`, `comment_index`, `reply_index`, `section_index` — 0-based:
программный traversal по массивам.
`page_number` — 1-based: пользовательская нумерация страниц документа.
Конвенция указывается явно в описании каждого поля.

### SD-FL-03 — Разные boundary conventions по типам

`char_start` / `char_end` — `[start, end)`: стандарт text processing APIs.
`timestamp_start` / `timestamp_end` — inclusive/inclusive: соответствует
интуиции пользователя и convention медиаплееров.

### SD-FL-04 — `scope: "full"` для element-level фрагментов

`post`, `comment`, `thread_reply` используют `scope: "full"` в v1.
Это явное подтверждение: фрагментирование внутри элемента не задаётся,
весь элемент является evidence-единицей.
Расширение до `scope: "partial"` с дополнительными координатами —
возможная v2 feature, не открывается в v1.

### SD-FL-05 — `aggregate` locator_data минимален

Для `fragment_type = "aggregate"` locator_data содержит только
`schema_version` и опциональный `fragment_count`.
Реальные ссылки на составные фрагменты хранятся в
`evidence.contributing_evidence_refs`, не в `locator_data`.
Правила согласованности с `contributing_evidence_refs` являются
pipeline-level проверками, а не standalone constraint схемы `locator_data`.

### SD-FL-06 — `text_range` использует Unicode codepoint offsets

`char_start` / `char_end` считаются в Unicode codepoints нормализованного
текстового snapshot. Это исключает неоднозначность между UTF-8 byte offsets,
UTF-16 code units и surrogate pairs в многоязычных материалах.
JS-потребители должны считать codepoints, а не полагаться на UTF-16 indexing
строк.

---

## 7. Открытые вопросы

### OQ-FL-01 — OCR-регионы: text внутри `image_region`

Если `image_region` содержит текст, распознанный OCR, — где хранится
распознанный текст и его координаты внутри региона? В v1: в `evidence.fragment_text`
(OCR-результат) + `locator_data` для bounding box. Вложенные text coords
внутри image_region не поддерживаются.

### OQ-FL-02 — Multi-page selection в `document_section`

`document_section` предполагает одну секцию. Если evidence охватывает
несколько страниц без явной секционной границы — как задать диапазон?
Вариант: `page_range_start` / `page_range_end` как дополнительный тип
`document_page_range`. В v1 не открывается; такие случаи покрываются
через `aggregate` + несколько `document_section` evidence-записей.

### OQ-FL-03 — Вложенные ответы в `thread_reply`

Форумы (Reddit, Discourse) поддерживают вложенные ответы.
`reply_index` — 0-based позиция в верхнеуровневом треде, но не адресует
вложенный ответ. В v1 не открывается; вложенные ответы описываются через
`reply_id` (platform ID однозначно идентифицирует ответ без знания глубины).

### OQ-FL-04 — `text_range` и версионирование snapshot

`snapshot_text_id` рекомендован, но не обязателен. Если snapshot изменился
после создания locator — char offsets могут указывать на неверный фрагмент.
Вариант для v2: сделать `snapshot_text_id` обязательным для `text_range`.
