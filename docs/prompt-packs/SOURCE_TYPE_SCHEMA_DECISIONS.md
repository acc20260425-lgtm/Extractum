# Source Type Schema Decisions

Дата фиксации: 2026-06-06.

Документ фиксирует принятые решения по `source_type_schemas.md` для Prompt Pack JSON Contract v1.
Это decision log к схемам `type_data`, а не сама JSON Schema.

## 1. Назначение `source_type_schemas.md`

`source_type_schemas.md` описывает структуру `type_data` для стандартных `source_type` из общего Prompt Pack JSON Contract v1.

Принятое разделение:

- общий контракт валидирует ядро `source_ref`;
- `source_type_schemas.md` валидирует `source_ref.type_data` по значению `source_type`;
- pack-specific schemas валидируют только pack-specific блоки, например `outputs.pack_data[pack_id]`.

Обоснование:

- `source_ref` должен оставаться единым material-level указателем;
- platform-specific детали не должны раздувать общий контракт;
- type-specific validation нужен, чтобы `type_data` не стал непроверяемым набором произвольных полей.

## 2. Строгий `type_data` и `extra_metadata`

Принято: `type_data` является строгим объектом.

Правила:

- `type_data` присутствует для всех стандартных `source_type`;
- для `source_type = custom` `type_data` опционален;
- все поля, описанные схемой конкретного типа, должны присутствовать;
- неизвестное или неприменимое значение записывается как `null`;
- отсутствующее стандартное поле является ошибкой валидации;
- неизвестные поля запрещены;
- расширения допустимы только через `extra_metadata`;
- `extra_metadata` всегда присутствует, минимум `{}`.

`extra_metadata`:

- свободный map;
- значения только JSON-примитивы или массивы примитивов;
- вложенные объекты запрещены;
- стандартные поля схемы не дублируются.

Обоснование:

- строгий объект даёт предсказуемую валидацию;
- nullable-поля сохраняют стабильную форму JSON;
- `extra_metadata` оставляет escape-hatch без превращения схемы в произвольный объект.

## 3. Общие соглашения

Приняты общие правила для всех `type_data`:

- ID всегда строки, не числа;
- даты и время — ISO 8601 UTC;
- длительности — целое число секунд;
- счётчики — integer или `null`;
- URL — только публичные;
- внутренние ссылки живут в `source_ref.internal_uri`;
- булевы поля: `true`, `false` или `null`;
- строки: строка или `null`.

Обоснование:

- эти правила совпадают с уже принятыми правилами общего JSON-контракта;
- они убирают неоднозначность между "значение неизвестно" и "поля нет";
- внутренние locator-ы не смешиваются с публичными URL.

## 4. Platform-specific timestamps

Принято:

- `source_ref.published_at` и `source_ref.accessed_at` остаются универсальными полями;
- platform-specific timestamps живут в `type_data`.

Стандартные имена:

```text
last_edited_at
scraped_at
snapshot_from
snapshot_to
```

Обоснование:

- `published_at` и `accessed_at` нужны всем source types и принадлежат `source_ref`;
- timestamps вроде `last_edited_at`, `snapshot_from`, `snapshot_to` имеют смысл только для отдельных платформ или агрегированных snapshot-типов.

## 5. Стандартный `creator`

Принято: все standard `type_data` используют общий объект `creator`.

```json
{
  "creator": {
    "creator_type": "channel",
    "custom_creator_type": null,
    "id": null,
    "platform_specific_id": null,
    "display_name": null,
    "profile_url": null
  }
}
```

`creator_type` enum v1:

```text
channel
user
organization
publication
website
forum
unknown
custom
```

Правила:

- `creator` всегда присутствует;
- если автор неизвестен, `creator_type = "unknown"`, остальные поля `null`;
- если `creator_type = custom`, `custom_creator_type` обязателен.

Обоснование:

- единый `creator` даёт cross-type traversal по авторам/каналам/изданиям;
- type-specific поля вроде `channel_id`, `author`, `username` быстро расходятся по именованию;
- `platform_specific_id` покрывает платформенные ID без дублирования.

## 6. Стандартный `parent_context`

Принято: все standard `type_data` используют общий объект `parent_context`.

```json
{
  "parent_context": {
    "context_type": null,
    "custom_context_type": null,
    "context_id": null,
    "platform_specific_id": null,
    "context_title": null,
    "context_url": null
  }
}
```

`context_type` enum v1:

```text
youtube_channel
youtube_playlist
website
rss_feed
telegram_channel
telegram_chat
forum
forum_category
custom
```

Правила:

- `parent_context` всегда присутствует;
- если родительский контекст неприменим, `context_type = null`, остальные поля `null`;
- если `context_type = custom`, `custom_context_type` обязателен.

Обоснование:

- Telegram posts, forum threads, RSS entries and YouTube videos all have context;
- единый объект предотвращает разные схемы для одной и той же идеи "родителя";
- parent context полезен для source grouping и аналитической трассировки.

## 7. Корневой контекст

Принято: `parent_context.context_type = null` означает, что материал является корневым контекстом и родителя нет.

В этом случае все остальные поля `parent_context` тоже `null`:

```text
custom_context_type
context_id
platform_specific_id
context_title
context_url
```

Примеры:

- `telegram_channel_snapshot`;
- `telegram_chat_snapshot`.

Обоснование:

- snapshot канала или чата сам является верхним уровнем иерархии;
- отдельный sentinel вроде `"root"` не нужен;
- `null` уже используется в контракте как "неприменимо".

## 8. `rss_entry` vs `web_page`

Принятое решение `SD-ST-01`:

Если RSS entry и web page указывают на один материал, это всё равно два разных `source_ref`.
Дедупликация решается на уровне `material_id` / pipeline snapshot, а не внутри `type_data`.

Обоснование:

- RSS entry и web page имеют разные provenance и разные metadata;
- RSS может содержать GUID, feed URL, categories и feed-level context;
- web page может иметь HTML/snapshot/extraction metadata;
- объединение внутри `type_data` сломало бы material-level прозрачность.

## 8.1 `web_page`

Принятые решения:

- `page_type` — свободная нормализованная строка с recommended values, не закрытый enum.
- Canonical URL страницы хранится только в `source_ref.canonical_url`, не в `web_page.type_data`.
- `content_extraction_status` добавлен как свободная нормализованная строка.
- `comment_collection_status` добавлен как свободная нормализованная строка.
- `content_extraction_status` описывает, удалось ли pipeline извлечь основной текст страницы.
- `comment_collection_status` описывает, собирался ли слой комментариев на самой странице.
- Web comments не являются отдельным `source_ref` в v1.

Recommended values для `content_extraction_status`:

```text
not_attempted
extracted
partial
failed
```

Recommended values для `comment_collection_status`:

```text
not_requested
collected
partial
unavailable
```

Обоснование:

- web extraction часто бывает partial или failed из-за paywall, JS, boilerplate и структуры страницы;
- `page_type` невозможно стабильно закрыть enum-ом, потому что web-жанры расширяются быстрее схемы;
- comments на web-странице являются fragment/evidence layer, если пользователь включил их сбор.

## 8.2 `rss_entry`

Принятые решения:

- `entry_url` добавлен как URL, объявленный самой RSS/Atom записью.
- `source_ref.canonical_url` остаётся canonical locator material.
- `entry_url` и `source_ref.canonical_url` могут совпадать, но имеют разную семантику.
- `content_mode` добавлен как свободная нормализованная строка.
- `rss_entry` не получает отдельный `*_collection_status` в v1.
- Если нужен анализ полного текста страницы по ссылке из RSS, создаётся отдельный `web_page` `source_ref`.

Recommended values для `content_mode`:

```text
summary_only
full_content
link_only
unknown
```

Обоснование:

- RSS entry является самой записью фида, а не collection layer вокруг другого material;
- полнота RSS content выражается через `content_mode`;
- связь RSS entry и web page решается через `material_id`/pipeline snapshot.

## 9. `youtube_video`

Принятые решения:

- `video_id` является обязательным не-null полем.
- `creator.creator_type = "channel"`.
- `parent_context.context_type = "youtube_channel"`.
- `channel_id` не вводится отдельным полем `type_data`.
- YouTube channel ID хранится в `creator.platform_specific_id` и `parent_context.platform_specific_id`.
- `captions_available` и `transcript_available` разделены.
- `captions_available` описывает доступность субтитров на платформе.
- `transcript_available` описывает, что pipeline реально собрал и может использовать transcript.
- `comment_collection_status` добавлен как свободная нормализованная строка.
- `comment_count` остаётся платформенным счётчиком.
- `comment_collection_status` описывает участие comments-layer в pipeline.
- `playlist_id`, `playlist_title`, `playlist_position` образуют связанный блок.
- Если `playlist_id = null`, то `playlist_title = null` и `playlist_position = null`.
- Если `playlist_position` заполнен, `playlist_id` должен быть заполнен.

Recommended values для `comment_collection_status`:

```text
not_requested
collected
partial
unavailable
```

Обоснование:

- `youtube_playlist` не является отдельным `source_ref` в v1, но playlist context важен для видео;
- transcript availability критична для LLM-анализа и не равна наличию captions;
- comments-layer не должен становиться отдельным material-level source_ref, но должен быть отражён в metadata;
- отказ от отдельного `channel_id` предотвращает дублирование одного и того же ID.

## 10. `telegram_post`

Принятые решения:

- `message_id` является обязательным не-null полем.
- `creator.creator_type` может быть `"channel"` или `"user"`.
- `parent_context.context_type = "telegram_channel"`.
- `creator` и `parent_context` могут ссылаться на один и тот же канал, если пост опубликован от имени канала.
- `post_type` — свободная нормализованная строка с recommended values, не закрытый enum.
- `reply_count` остаётся платформенным счётчиком.
- `discussion_message_count` показывает, сколько сообщений обсуждения реально попало в snapshot/pipeline.
- `discussion_collection_status` добавлен как свободная нормализованная строка.
- `discussion_collection_status` использует ту же логику, что `youtube_video.comment_collection_status`, но для Telegram discussion layer.
- Если `is_forwarded = false`, `forwarded_from_channel_id = null` и `forwarded_from_channel_title = null`.
- Если `is_forwarded = true`, `forwarded_from_channel_id` желательно заполнен.

Recommended values для `discussion_collection_status`:

```text
not_requested
collected
partial
unavailable
```

Обоснование:

- Telegram discussion/comments не являются отдельным `source_ref` в v1;
- platform reply count и реально собранный discussion corpus могут расходиться;
- свободный `post_type` устойчивее закрытого enum, потому что Telegram media/service-message cases разнообразны;
- forward metadata должна быть строго null, когда пост не является пересланным.

## 11. Collection status pattern

Принят общий паттерн для слоёв, которые не являются отдельными `source_ref`, но могут участвовать в pipeline:

```text
*_collection_status
```

Примеры:

- `youtube_video.comment_collection_status`;
- `telegram_post.discussion_collection_status`;
- `web_page.comment_collection_status`.

Recommended values:

```text
not_requested
collected
partial
unavailable
```

Обоснование:

- comments/replies/discussions являются fragment/evidence layer, а не material-level source_ref;
- статус сбора нужен для интерпретации полноты результата;
- свободная нормализованная строка устойчивее закрытого enum для evolving pipeline states.

## 12. Platform counter vs pipeline counter

Принят паттерн разделения:

- платформенный счётчик показывает значение, сообщённое платформой;
- pipeline-счётчик показывает, сколько объектов реально попало в snapshot или analysis corpus.

Примеры:

- `youtube_video.comment_count` — платформенный счётчик комментариев;
- `youtube_video.comment_collection_status` — статус сбора comments-layer;
- `telegram_post.reply_count` — платформенный счётчик replies;
- `telegram_post.discussion_message_count` — число сообщений обсуждения, реально собранных pipeline;
- `forum_thread.reply_count` — платформенный счётчик replies;
- `forum_thread.participant_count` — число уникальных участников thread snapshot, если доступно.

Обоснование:

- платформа может показывать количество, но pipeline может собрать только часть;
- смешение этих двух значений создаёт ложное ощущение полноты корпуса.

## 13. `forum_thread.platform`

Принято: `forum_thread.platform` остаётся свободной нормализованной строкой с recommended values, а не закрытым enum.

Recommended values:

```text
reddit
hackernews
stackoverflow
discourse
other
```

Правила:

- `thread_id` не является глобально уникальным;
- уникальность обеспечивается комбинацией `platform + thread_id` и, при наличии, `parent_context`.

Обоснование:

- форумных платформ слишком много для закрытого enum;
- закрытый enum быстро потребовал бы `custom` почти в каждом реальном проекте;
- normalized free string даёт стабильный минимум без искусственного ограничения.

## 13.1 `forum_thread`

Принятые решения:

- `vote_score` остаётся единым кросс-платформенным агрегатом.
- `upvote_count` и `downvote_count` не добавляются в v1.
- Platform-specific vote details уходят в `extra_metadata`.
- `reply_collection_status` не добавляется в v1.
- `participant_count` добавлен как общий сигнал состава обсуждения.

Обоснование:

- Reddit, Hacker News, Stack Overflow и Discourse по-разному представляют votes;
- единый `vote_score` покрывает минимальный общий слой;
- forum thread является material-level веткой обсуждения, а replies внутри него являются fragment/evidence layer;
- `participant_count` полезен для оценки широты подтверждения или спора внутри thread.

## 13.2 Telegram snapshots

Принятые решения:

- `telegram_channel_snapshot` и `telegram_chat_snapshot` являются корневыми контекстами.
- У обоих типов `parent_context.context_type = null`.
- `snapshot_from` и `snapshot_to` заполняются только вместе.
- `telegram_channel_snapshot.avg_posts_per_day` добавлен как snapshot-метрика активности.
- `telegram_channel_snapshot.avg_reactions_per_post` не добавляется в v1.
- `telegram_chat_snapshot` дополнительных полей не требует: `member_count`,
  `message_count_in_snapshot` и `unique_author_count` достаточно для v1.

Обоснование:

- channel/chat snapshot сам является верхним уровнем иерархии;
- симметричное правило периода исключает неполные snapshot ranges;
- `avg_posts_per_day` вычислим, но удобен для сравнения каналов за разные периоды;
- reactions в Telegram могут быть неполными и уже представлены на уровне `telegram_post.reaction_count`.

## 14. Открытые вопросы

Открытые темы, которые не закрыты этим decision log:

1. Глубина описания авторов.
2. `telegram_post`: подпись редактора в каналах с несколькими редакторами.
3. `forum_thread` и multi-platform ID beyond `platform + thread_id`.

## 15. Итог

Принятый подход:

```text
type_data строгий и валидируемый;
creator и parent_context едины для всех типов;
extra_metadata является контролируемым escape-hatch;
source-specific layers вроде comments/discussions остаются fragment/evidence layer,
а не отдельными source_ref.
```

Это сохраняет единый material-level контракт и позволяет добавлять platform-specific детали без размывания общей схемы.
