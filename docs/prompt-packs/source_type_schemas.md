# source_type_schemas.md — v1

Версионируется вместе с `schema_version` основного контракта.
Текущая версия: `1.0`.

---

## 1. Назначение

Этот документ определяет структуру поля `type_data` для каждого стандартного
`source_type` из Prompt Pack JSON Contract v1.

Общий контракт валидирует ядро `source_ref`.
Этот документ валидирует содержимое `type_data` по значению `source_type`.

---

## 2. Общие правила `type_data`

- `type_data` присутствует для всех стандартных `source_type`.
- Для `source_type = custom` поле `type_data` опционально.
- Структура `type_data` определяется `source_type`; неизвестные поля запрещены.
- Расширения допустимы только через `extra_metadata`.
- Все поля, описанные схемой типа, обязаны присутствовать в объекте.
  Отсутствующее стандартное поле — ошибка валидации.
  Неизвестное или неприменимое значение — `null`, не отсутствие поля.

---

## 3. Общие соглашения

| Тема | Правило |
|---|---|
| ID | строки, не числа |
| Даты и время | ISO 8601 UTC (`2026-04-12T10:00:00Z`) |
| Длительности | целое число секунд |
| Счётчики | целое число или `null` если недоступно |
| URL | только публичные; внутренние ссылки через `internal_uri` в `source_ref` |
| Булевы | `true` / `false` / `null` (null = неизвестно) |
| Строки | `null` если неизвестно или неприменимо |

Platform-specific timestamps (кроме `published_at` и `accessed_at`, которые
живут в `source_ref`) размещаются в `type_data`. Стандартные имена:

```
last_edited_at    — последнее редактирование материала
scraped_at        — когда был произведён сбор данных пайплайном
snapshot_from     — начало периода snapshot (для агрегированных типов)
snapshot_to       — конец периода snapshot
```

---

### Корневой контекст: `parent_context.context_type = null`

Если материал является корневым контекстом и не имеет родителя
(например, snapshot канала или чата) — `context_type = null`.
В этом случае все остальные поля `parent_context` также `null`:
`custom_context_type`, `context_id`, `platform_specific_id`,
`context_title`, `context_url`.

---

## 4. Общая обёртка `type_data`

Каждая схема строится поверх следующего каркаса.
Type-specific поля добавляются рядом с полями обёртки, не вложенными в неё.

```json
{
  "schema_version": "1.0",

  "creator": {
    "creator_type": "channel",
    "custom_creator_type": null,
    "id": null,
    "platform_specific_id": null,
    "display_name": null,
    "profile_url": null
  },

  "parent_context": {
    "context_type": null,
    "custom_context_type": null,
    "context_id": null,
    "platform_specific_id": null,
    "context_title": null,
    "context_url": null
  },

  "extra_metadata": {}
}
```

### `creator`

Всегда присутствует. Если автор неизвестен — `creator_type = "unknown"`,
остальные поля `null`.

`creator_type` enum v1:
```
channel | user | organization | publication | website | forum | unknown | custom
```
Для `custom`: `custom_creator_type` обязателен.

### `parent_context`

Всегда присутствует. Если родительский контекст неприменим —
`context_type = null`, остальные поля `null`.

`context_type` enum v1:
```
youtube_channel | youtube_playlist | website | rss_feed |
telegram_channel | telegram_chat | forum | forum_category | custom
```
Для `custom`: `custom_context_type` обязателен.

### `extra_metadata`

Всегда присутствует, минимум `{}`. Правила значений:
- только JSON-примитивы (`string`, `number`, `boolean`, `null`)
  или массивы примитивов;
- вложенные объекты запрещены;
- не должен дублировать стандартные поля схемы.

---

## 5. Схемы стандартных `source_type`

---

### 5.1 `youtube_video`

Конкретное видео на YouTube.

**Обязательные поля (не null):**
```
schema_version
video_id
```

**Поля:**

**Примечание:** `channel_id` не выносится отдельным полем —
он представлен через `creator.platform_specific_id`
и `parent_context.platform_specific_id`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | всегда `"1.0"` |
| `video_id` | string | да | YouTube video ID |
| `duration_seconds` | integer\|null | нет | длительность видео |
| `language` | string\|null | нет | BCP 47, язык видео |
| `captions_available` | boolean\|null | нет | есть ли субтитры любого типа |
| `transcript_available` | boolean\|null | нет | собран ли и пригоден ли транскрипт для LLM-анализа |
| `is_live_recording` | boolean\|null | нет | запись live-трансляции |
| `view_count` | integer\|null | нет | счётчик просмотров на момент сбора |
| `like_count` | integer\|null | нет | счётчик лайков |
| `comment_count` | integer\|null | нет | счётчик комментариев |
| `comment_collection_status` | string\|null | нет | статус сбора комментариев (см. ниже) |
| `playlist_id` | string\|null | нет | ID плейлиста, если видео в нём |
| `playlist_title` | string\|null | нет | название плейлиста |
| `playlist_position` | integer\|null | нет | позиция видео в плейлисте |
| `scraped_at` | string\|null | нет | ISO 8601, когда собраны данные |
| `creator` | object | да | см. обёртку; `creator_type = "channel"` |
| `parent_context` | object | да | `context_type = "youtube_channel"` |
| `extra_metadata` | object | да | минимум `{}` |

`comment_collection_status` — свободная нормализованная строка.
Рекомендованные значения: `not_requested`, `collected`, `partial`, `unavailable`.
Отличается от `comment_count`: описывает, участвовал ли comments-layer в pipeline,
а не сколько комментариев у видео на платформе.

**Пример:**

```json
{
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
  "playlist_id": "PLxyz",
  "playlist_title": "AI Agents Course 2026",
  "playlist_position": 3,
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
```

**Правила валидации:**
- Если `playlist_id = null`, то `playlist_title` и `playlist_position` тоже `null`.
- Если `playlist_position` заполнен, `playlist_id` должен быть заполнен.
- `creator.creator_type` должен быть `"channel"`.
- `parent_context.context_type` должен быть `"youtube_channel"`.

---

### 5.2 `web_page`

Конкретная веб-страница (статья, документация, пост в блоге, landing page).

**Обязательные поля (не null):** `schema_version`.

**Примечание:** canonical URL страницы хранится только в `source_ref.canonical_url`,
не дублируется в `web_page.type_data`.

`page_type` — свободная нормализованная строка, не enum.
Рекомендованные значения: `article`, `documentation`, `homepage`,
`blog_post`, `landing`, `forum_page`, `other`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `page_type` | string\|null | нет | тип страницы (см. выше) |
| `site_name` | string\|null | нет | название сайта/издания |
| `language` | string\|null | нет | BCP 47 |
| `word_count` | integer\|null | нет | приблизительное количество слов |
| `content_extraction_status` | string\|null | нет | статус извлечения основного текста (см. ниже) |
| `comment_collection_status` | string\|null | нет | статус сбора комментариев (см. ниже) |
| `last_modified_at` | string\|null | нет | ISO 8601, из HTTP header или meta |
| `scraped_at` | string\|null | нет | ISO 8601 |
| `creator` | object | да | `creator_type`: `"user"`, `"organization"`, `"publication"` или `"website"` |
| `parent_context` | object | да | `context_type = "website"` или `context_type = null` если неприменимо |
| `extra_metadata` | object | да | минимум `{}` |

`content_extraction_status` — свободная нормализованная строка.
Рекомендованные значения: `not_attempted`, `extracted`, `partial`, `failed`.
Страница могла быть доступна, но основной текст извлечён частично или не извлечён
(пейволл, JS-рендеринг, нестандартная вёрстка).

`comment_collection_status` — свободная нормализованная строка.
Рекомендованные значения: `not_requested`, `collected`, `partial`, `unavailable`.
Применяется только если комментарии на той же странице и сбор был запрошен.

**Пример:**

```json
{
  "schema_version": "1.0",
  "page_type": "article",
  "site_name": "The Pragmatic Engineer",
  "language": "en",
  "word_count": 3200,
  "content_extraction_status": "extracted",
  "comment_collection_status": "not_requested",
  "last_modified_at": null,
  "scraped_at": "2026-06-05T14:10:00Z",
  "creator": {
    "creator_type": "user",
    "custom_creator_type": null,
    "id": null,
    "platform_specific_id": null,
    "display_name": "Gergely Orosz",
    "profile_url": "https://newsletter.pragmaticengineer.com"
  },
  "parent_context": {
    "context_type": "website",
    "custom_context_type": null,
    "context_id": null,
    "platform_specific_id": null,
    "context_title": "The Pragmatic Engineer",
    "context_url": "https://newsletter.pragmaticengineer.com"
  },
  "extra_metadata": {}
}
```

---

### 5.3 `rss_entry`

Конкретная запись из RSS/Atom-ленты.

**Обязательные поля (не null):** `schema_version`, `entry_id`.

**Примечание:** `collection_status` не нужен для `rss_entry` в v1.
RSS entry представлен как полученная запись фида: либо получена целиком, либо нет.
Неполнота полного текста выражается через `content_mode`;
если нужен полный текст страницы — это отдельный `web_page` source_ref
с дедупликацией через `material_id`.

`content_mode` — свободная нормализованная строка, не enum.
Рекомендованные значения: `summary_only`, `full_content`, `link_only`, `unknown`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `entry_id` | string | да | GUID/ID записи из фида |
| `entry_url` | string\|null | нет | URL, объявленный самим фидом для этой записи |
| `feed_url` | string\|null | нет | URL самого RSS/Atom фида |
| `feed_title` | string\|null | нет | название фида |
| `content_mode` | string\|null | нет | что содержит запись фида (см. выше) |
| `categories` | array\|null | нет | массив строк-тегов/категорий из фида |
| `word_count` | integer\|null | нет | слов в тексте записи фида |
| `language` | string\|null | нет | BCP 47 |
| `scraped_at` | string\|null | нет | ISO 8601 |
| `creator` | object | да | `creator_type`: `"user"`, `"organization"`, `"publication"` или `"website"` |
| `parent_context` | object | да | `context_type = "rss_feed"` |
| `extra_metadata` | object | да | минимум `{}` |

**Правила валидации:**
- Если `categories` не `null`, элементы массива — строки.
- `entry_url` и `source_ref.canonical_url` могут совпадать;
  `entry_url` хранит URL из фида, `canonical_url` — canonical locator материала.
- `parent_context.context_url` желательно совпадает с `feed_url`.

**Пример:**

```json
{
  "schema_version": "1.0",
  "entry_id": "tag:example.com,2026:entry-42",
  "entry_url": "https://example.com/blog/entry-42",
  "feed_url": "https://example.com/rss.xml",
  "feed_title": "Example Tech Blog",
  "content_mode": "summary_only",
  "categories": ["AI", "agents", "local-models"],
  "word_count": 320,
  "language": "en",
  "scraped_at": "2026-06-04T08:00:00Z",
  "creator": {
    "creator_type": "user",
    "custom_creator_type": null,
    "id": null,
    "platform_specific_id": null,
    "display_name": "Jane Smith",
    "profile_url": null
  },
  "parent_context": {
    "context_type": "rss_feed",
    "custom_context_type": null,
    "context_id": null,
    "platform_specific_id": null,
    "context_title": "Example Tech Blog",
    "context_url": "https://example.com/rss.xml"
  },
  "extra_metadata": {}
}
```

---

### 5.4 `telegram_post`

Конкретное сообщение в Telegram-канале.

**Обязательные поля (не null):** `schema_version`, `message_id`.

`post_type` — свободная нормализованная строка, не enum.
Telegram-медиа и служебные сообщения разнообразнее любого закрытого списка.
Рекомендованные значения: `text`, `photo`, `video`, `document`, `poll`,
`forwarded`, `sticker`, `voice`, `service`, `other`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `message_id` | string | да | ID сообщения внутри канала |
| `post_type` | string\|null | нет | тип сообщения (см. выше) |
| `has_media` | boolean\|null | нет | есть ли медиавложение |
| `view_count` | integer\|null | нет | счётчик просмотров на платформе |
| `forward_count` | integer\|null | нет | сколько раз переслано |
| `reply_count` | integer\|null | нет | счётчик replies на платформе |
| `reaction_count` | integer\|null | нет | суммарное количество реакций |
| `discussion_collection_status` | string\|null | нет | статус сбора слоя обсуждения (см. ниже) |
| `discussion_message_count` | integer\|null | нет | сообщений обсуждения, попавших в pipeline |
| `is_pinned` | boolean\|null | нет | |
| `is_forwarded` | boolean\|null | нет | это пересланное сообщение |
| `forwarded_from_channel_id` | string\|null | нет | ID канала-источника при пересылке |
| `forwarded_from_channel_title` | string\|null | нет | название канала-источника |
| `last_edited_at` | string\|null | нет | ISO 8601 |
| `scraped_at` | string\|null | нет | ISO 8601 |
| `creator` | object | да | `creator_type`: `"channel"` или `"user"` |
| `parent_context` | object | да | `context_type = "telegram_channel"` |
| `extra_metadata` | object | да | минимум `{}` |

`discussion_collection_status` — свободная нормализованная строка.
Рекомендованные значения: `not_requested`, `collected`, `partial`, `unavailable`.
Отличается от `reply_count`: описывает участие discussion-layer в pipeline,
а не счётчик replies на платформе. Аналог `comment_collection_status`
в `youtube_video`, но для Telegram-обсуждений.

**Правила валидации:**
- Если `is_forwarded = true`, `forwarded_from_channel_id` желательно заполнен.
- Если `is_forwarded = false`, `forwarded_from_channel_id`
  и `forwarded_from_channel_title` должны быть `null`.
- `creator` и `parent_context` могут ссылаться на один и тот же канал
  (если пост опубликован от имени канала).

**Пример:**

```json
{
  "schema_version": "1.0",
  "message_id": "1842",
  "post_type": "text",
  "has_media": false,
  "view_count": 12400,
  "forward_count": 87,
  "reply_count": null,
  "reaction_count": 430,
  "discussion_collection_status": "not_requested",
  "discussion_message_count": null,
  "is_pinned": false,
  "is_forwarded": false,
  "forwarded_from_channel_id": null,
  "forwarded_from_channel_title": null,
  "last_edited_at": null,
  "scraped_at": "2026-06-06T09:30:00Z",
  "creator": {
    "creator_type": "channel",
    "custom_creator_type": null,
    "id": "channel_001",
    "platform_specific_id": "-1001234567890",
    "display_name": "Example Tech Channel",
    "profile_url": "https://t.me/examplechannel"
  },
  "parent_context": {
    "context_type": "telegram_channel",
    "custom_context_type": null,
    "context_id": "channel_001",
    "platform_specific_id": "-1001234567890",
    "context_title": "Example Tech Channel",
    "context_url": "https://t.me/examplechannel"
  },
  "extra_metadata": {}
}
```

---

### 5.5 `telegram_channel_snapshot`

Агрегированный snapshot публичного Telegram-канала за период.
Не отдельный пост, а срез состояния канала.

**Обязательные поля (не null):** `schema_version`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `channel_platform_id` | string\|null | нет | Telegram internal channel ID |
| `subscriber_count` | integer\|null | нет | на момент snapshot |
| `post_count_in_snapshot` | integer\|null | нет | постов в периоде snapshot |
| `avg_views_per_post` | number\|null | нет | среднее просмотров на пост |
| `avg_posts_per_day` | number\|null | нет | среднее постов в день за период |
| `snapshot_from` | string\|null | нет | ISO 8601, начало периода |
| `snapshot_to` | string\|null | нет | ISO 8601, конец периода |
| `scraped_at` | string\|null | нет | ISO 8601 |
| `creator` | object | да | `creator_type = "channel"` |
| `parent_context` | object | да | `context_type = null` (нет родителя) |
| `extra_metadata` | object | да | минимум `{}` |

`avg_reactions_per_post` не добавляется в v1: реакции неполны и нестандартизованы
между каналами; детальные реакции доступны на уровне `telegram_post.reaction_count`.
Для агрегированной картины активности достаточно `avg_views_per_post` + `avg_posts_per_day`.

**Правила валидации:**
- `snapshot_from` и `snapshot_to` заполняются только вместе:
  если одно заполнено — второе тоже обязательно; если одно `null` — второе тоже `null`.
- `parent_context.context_type = null`; остальные поля `parent_context = null`.

**Пример:**

```json
{
  "schema_version": "1.0",
  "channel_platform_id": "-1001234567890",
  "subscriber_count": 84200,
  "post_count_in_snapshot": 47,
  "avg_views_per_post": 9300.5,
  "avg_posts_per_day": 0.26,
  "snapshot_from": "2026-01-01T00:00:00Z",
  "snapshot_to": "2026-06-06T00:00:00Z",
  "scraped_at": "2026-06-06T10:00:00Z",
  "creator": {
    "creator_type": "channel",
    "custom_creator_type": null,
    "id": "channel_001",
    "platform_specific_id": "-1001234567890",
    "display_name": "Example Tech Channel",
    "profile_url": "https://t.me/examplechannel"
  },
  "parent_context": {
    "context_type": null,
    "custom_context_type": null,
    "context_id": null,
    "platform_specific_id": null,
    "context_title": null,
    "context_url": null
  },
  "extra_metadata": {}
}
```

---

### 5.6 `telegram_chat_snapshot`

Агрегированный snapshot публичного или доступного Telegram-чата (группы) за период.

**Обязательные поля (не null):** `schema_version`.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `chat_platform_id` | string\|null | нет | Telegram internal chat ID |
| `member_count` | integer\|null | нет | участников на момент snapshot |
| `message_count_in_snapshot` | integer\|null | нет | сообщений в периоде |
| `unique_author_count` | integer\|null | нет | уникальных авторов в периоде |
| `snapshot_from` | string\|null | нет | ISO 8601 |
| `snapshot_to` | string\|null | нет | ISO 8601 |
| `scraped_at` | string\|null | нет | ISO 8601 |
| `creator` | object | да | `creator_type = "unknown"` (группа, не индивидуальный автор) |
| `parent_context` | object | да | `context_type = null` |
| `extra_metadata` | object | да | минимум `{}` |

**Правила валидации:**
- `snapshot_from` и `snapshot_to` заполняются только вместе:
  если одно заполнено — второе тоже обязательно; если одно `null` — второе тоже `null`.
- `creator.creator_type = "unknown"` — у чата нет единственного автора.

**Пример:**

```json
{
  "schema_version": "1.0",
  "chat_platform_id": "-1009876543210",
  "member_count": 3200,
  "message_count_in_snapshot": 840,
  "unique_author_count": 187,
  "snapshot_from": "2026-03-01T00:00:00Z",
  "snapshot_to": "2026-06-06T00:00:00Z",
  "scraped_at": "2026-06-06T10:15:00Z",
  "creator": {
    "creator_type": "unknown",
    "custom_creator_type": null,
    "id": null,
    "platform_specific_id": null,
    "display_name": "Example Community Chat",
    "profile_url": null
  },
  "parent_context": {
    "context_type": null,
    "custom_context_type": null,
    "context_id": null,
    "platform_specific_id": null,
    "context_title": null,
    "context_url": null
  },
  "extra_metadata": {}
}
```

---

### 5.7 `forum_thread`

Тред на форуме или в дискуссионной платформе (Reddit, HN, Stack Overflow, Discourse и т.д.).

**Обязательные поля (не null):** `schema_version`, `thread_id`.

**`platform`** — свободная нормализованная строка, не enum.
Рекомендованные значения: `reddit`, `hackernews`, `stackoverflow`, `discourse`, `other`.
Уникальность треда определяется комбинацией `platform + thread_id` плюс `parent_context`.

**`vote_score`** — кросс-платформенный агрегат (upvotes, score, net votes).
Раздельные `upvote_count` / `downvote_count` не добавляются в v1: platform-specific
и не везде доступны. При наличии деталей — в `extra_metadata`.

**`reply_collection_status`** не добавляется в v1.
Replies внутри треда — fragment/evidence layer, не отдельный collection layer.
Частичный сбор выражается через `warning` или `quality_flag` на уровне результата.

| Поле | Тип | Обязательный | Описание |
|---|---|---|---|
| `schema_version` | string | да | `"1.0"` |
| `thread_id` | string | да | ID треда на платформе |
| `platform` | string\|null | нет | платформа (см. выше) |
| `board_id` | string\|null | нет | ID доски/subreddit/категории |
| `board_name` | string\|null | нет | название доски |
| `reply_count` | integer\|null | нет | количество ответов на платформе |
| `participant_count` | integer\|null | нет | уникальных участников треда |
| `view_count` | integer\|null | нет | |
| `vote_score` | integer\|null | нет | агрегированный score/upvotes (см. выше) |
| `is_locked` | boolean\|null | нет | закрыт ли тред для новых ответов |
| `is_pinned` | boolean\|null | нет | |
| `last_reply_at` | string\|null | нет | ISO 8601 |
| `scraped_at` | string\|null | нет | ISO 8601 |
| `creator` | object | да | `creator_type = "user"` (автор первого поста) |
| `parent_context` | object | да | `context_type`: `"forum"` или `"forum_category"` |
| `extra_metadata` | object | да | минимум `{}` |

**Пример:**

```json
{
  "schema_version": "1.0",
  "thread_id": "t3_abc123",
  "platform": "reddit",
  "board_id": "LocalLLaMA",
  "board_name": "r/LocalLLaMA",
  "reply_count": 147,
  "participant_count": 43,
  "view_count": null,
  "vote_score": 842,
  "is_locked": false,
  "is_pinned": false,
  "last_reply_at": "2026-06-05T18:42:00Z",
  "scraped_at": "2026-06-06T09:00:00Z",
  "creator": {
    "creator_type": "user",
    "custom_creator_type": null,
    "id": null,
    "platform_specific_id": "u_example_user",
    "display_name": "example_user",
    "profile_url": "https://reddit.com/u/example_user"
  },
  "parent_context": {
    "context_type": "forum_category",
    "custom_context_type": null,
    "context_id": "LocalLLaMA",
    "platform_specific_id": null,
    "context_title": "r/LocalLLaMA",
    "context_url": "https://reddit.com/r/LocalLLaMA"
  },
  "extra_metadata": {}
}
```

---

## 6. Принятые решения

### SD-ST-01 — `rss_entry` vs `web_page`: два `source_ref`, дедупликация через `material_id`

Если RSS entry и web page указывают на один и тот же материал,
они представляются как два отдельных `source_ref` с разными `source_type`.
Дедупликация решается на уровне `material_id` и pipeline snapshot,
не внутри `type_data`. Общий контракт не требует их объединения.

### SD-ST-02 — `web_page.page_type`: свободная нормализованная строка, не enum

`page_type` не закрывается enum-ом: типов веб-страниц слишком много,
закрытый список быстро стал бы неполным. Используются рекомендованные значения
без validation constraint. Тот же паттерн, что `post_type` в `telegram_post`
и `platform` в `forum_thread`.

### SD-ST-03 — `rss_entry.content_mode` и отсутствие collection status

- `rss_entry` не получает отдельный `*_collection_status` в v1.
- RSS entry считается полученной записью фида; разделения на collection layer нет.
- Полнота содержимого entry выражается через `content_mode`
  (свободная нормализованная строка; recommended values:
  `summary_only`, `full_content`, `link_only`, `unknown`).
- Если нужен анализ полного текста страницы по ссылке из RSS,
  создаётся отдельный `web_page` source_ref.
- Связь и дедупликация между `rss_entry` и `web_page` решается
  через `material_id` / pipeline snapshot, не внутри `type_data`.

Обоснование: RSS отличается от comments/discussions — это не дополнительный
collection layer вокруг материала, а сама запись фида с разной полнотой контента.

### SD-ST-04 — `forum_thread`: компактная кросс-платформенная модель

- `vote_score` — единственный агрегированный счётчик голосов; раздельные
  `upvote_count` / `downvote_count` не добавляются в v1 как platform-specific.
  При наличии деталей — в `extra_metadata`.
- `reply_collection_status` не добавляется: replies — fragment/evidence layer,
  частичный сбор выражается через `warning` / `quality_flag` на уровне результата.
- `participant_count` добавлен как общий кросс-платформенный сигнал состава
  обсуждения: насколько вывод поддержан одним человеком или несколькими участниками.

---

## 7. Открытые вопросы

### OQ-ST-01 — Глубина описания авторов

Текущий `creator` объект хранит только `display_name` и `profile_url`.
Для OSINT pack может потребоваться расширенный профиль:
подписчики, верификация, дата регистрации, связанные аккаунты.

Рекомендация: расширять через `extra_metadata` в v1;
для OSINT-specific данных — pack-specific `type_data` extension через `custom`.

---

### OQ-ST-02 — `telegram_post`: автор в каналах vs группах

В каналах посты публикуются от имени канала, автор-человек скрыт.
В группах автор известен. Текущая схема это покрывает через
`creator_type = "channel"` vs `"user"`, но нет поля для подписи редактора
(подпись автора в channel с нескольким редакторами).

Если понадобится: добавить `editor_signature: null | string` в `extra_metadata`.

---

### OQ-ST-03 — `forum_thread` и multi-platform ID

`thread_id` — ID внутри платформы, не глобальный.
Reddit и Discourse могут использовать одинаковые строки.
Уникальность обеспечивается комбинацией `platform + thread_id`,
но сейчас это явно не зафиксировано как composite key.

---

---


