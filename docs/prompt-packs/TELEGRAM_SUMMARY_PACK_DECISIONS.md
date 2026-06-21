# telegram_summary Pack Decisions

Документ фиксирует принятые решения по `telegram_summary_pack_spec.md`.
Это decision log к pack-specific schema для
`outputs.pack_data.telegram_summary`.

---

## 1. Назначение pack

`telegram_summary` предназначен для проверяемого summary Telegram-каналов,
групповых чатов, supergroups/forum topics и смешанных Telegram-подборок за
период.

Pack не является:

- полноценным OSINT-анализом авторов, организаций или сетей распространения;
- reputation/person profiling участников;
- полной social graph моделью чата;
- внешней факт-проверкой claims;
- OCR/STT/media-understanding pipeline без уже доступных captions или
  `mediaSummary`;
- production monitoring или realtime alerting system.

Обоснование: Telegram Summary должен быть source-specific summary pack, ближе
по роли к `youtube_summary`, а не umbrella-pack для OSINT, reputation и
monitoring workflows.

---

## 2. Один pack для каналов и чатов

Принято: каналы, чаты, supergroups/forum topics и смешанные Telegram-подборки
покрываются одним pack:

```json
{
  "telegram_summary": {
    "source_shape": "mixed"
  }
}
```

`source_shape` принимает:

- `channel`;
- `chat`;
- `mixed`.

Обоснование:

- Каналы и чаты используют общую Telegram message substrate.
- Пользовательские сценарии пересекаются: digest за период, поиск важных
  сообщений, claims, topics, reply chains.
- Разделение на два pack-а привело бы к дублированию schema и prompt logic.
- Отличия каналов и чатов лучше выражать через `source_shape`,
  `source_kind`, `message_kind`, thread/topic grouping и scoring rules.

---

## 3. `message_refs` как pack-local индекс сообщений

Принято: `telegram_summary` вводит обязательный массив
`telegram_summary.message_refs`.

```json
{
  "message_refs": [
    {
      "message_ref_id": "msg_ref_1",
      "summary_source_id": "tg_source_1",
      "message_id": "1842"
    }
  ]
}
```

Остальные объекты используют:

- `message_ref` для одного primary anchor;
- `message_refs` для нескольких сообщений.

Обоснование:

- `message_id` уникален только внутри Telegram source namespace.
- Chat/supergroup messages не всегда являются отдельными `source_ref`.
- Snapshot source types описывают root context и период, а не каждое сообщение.
- Pack-level key messages, topics, threads, claims и forwarded items должны
  ссылаться на сообщения единым способом.
- `evidence_refs` указывают на проверяемые фрагменты текста/metadata, а
  `message_refs` — на message-level identity. Эти слои не заменяют друг друга.

---

## 4. Snapshot source types остаются root context

Принято: `telegram_channel_snapshot` и `telegram_chat_snapshot` остаются root
context source types. Конкретные сообщения, включая chat/supergroup messages,
представляются через `telegram_summary.message_refs` и связанные
evidence/fragment records.

Обоснование:

- Не нужно раздувать canonical `source_refs` тысячами отдельных chat messages.
- Можно анализировать noisy chats и reply chains без изменения текущего
  companion schema layer.
- Machine-readable schema сможет проверять pack-level refs независимо от того,
  пришло сообщение как `telegram_post`, internal `SourceItem` или evidence
  fragment.

---

## 5. Темы строятся гибридно

Принято: topic grouping использует приоритет:

1. `forum_topic_id`, если Telegram forum topic доступен.
2. Root reply chain через `reply_to_top_message_ref`.
3. Semantic grouping по entities, links, keywords и временной близости.

Обоснование:

- Forum topic — самый сильный структурный сигнал.
- Reply chain сохраняет разговорный контекст.
- Semantic grouping нужен для каналов и неструктурированных чатов.
- Только LLM-summary "всего подряд" плохо работает на больших noisy chats.

---

## 6. Канал, чат и mixed result имеют разные аналитические акценты

Для `source_shape = "channel"` основной unit — пост:

- timeline;
- key posts;
- forwarded items;
- claims;
- post-level engagement signals.

Для `source_shape = "chat"` основной unit — discussion/thread:

- reply chains;
- вопросы и ответы;
- useful answers;
- disputes;
- consensus/outcome.

Для `source_shape = "mixed"` pack должен различать:

- channel authority;
- chat consensus;
- discussion quality;
- cross-source repeated или disputed claims.

Обоснование: канал, чат и mixed corpus имеют разную доказательную ценность.
Официальный пост, пересланное сообщение и реплика в чате не должны смешиваться
как однотипные сигналы.

---

## 7. `message_quality_signals`

Принято: `message_quality_signals` входит в top-level
`telegram_summary` как обязательное поле-массив, но может быть пустым.

Обоснование:

- Пользовательский сценарий "найти важные сообщения" требует объяснимого
  scoring.
- Для чатов полезность сообщения часто важнее popularity metrics.
- Низкий score относится к сообщению в контексте задачи, а не к личности
  автора. Pack не делает скрытый profiling участников.

---

## 8. Importance scoring — best effort, не абсолютная метрика

Принято: `importance_score` хранится как best-effort `0..1` и обязательно
сопровождается `importance_reasons`, если score используется для key messages.

Сигналы v1:

- engagement;
- propagation;
- novelty;
- thread role/outcome;
- claim density;
- usefulness;
- risk.

Обоснование:

- Telegram counters неполны и неодинаково доступны.
- `view_count` не является обязательным входом.
- Для каналов важнее post-level signals.
- Для чатов важнее usefulness, answer quality и thread outcome.

---

## 9. Forwarded messages не считаются независимым подтверждением

Принято: forwarded items выделяются отдельно, но forwarded message не становится
автоматически независимым source confirmation.

Обоснование:

- Forward может усиливать тему или запускать обсуждение.
- Origin может быть неизвестен или неполон.
- Повторение forwarded текста не равно независимой проверке claim.

---

## 10. Migrated history должна сохранять namespace

Принято: message refs имеют `history_scope`, `is_migrated_history` и
`migration_domain`.

Обоснование:

- Telegram history может быть migrated/current.
- Старые и новые сообщения могут иметь пересекающиеся IDs или разные
  контексты происхождения.
- Pack не должен смешивать migrated и current history без явного marker.

---

## 11. Pack-specific stages

Pack-specific stages v1:

- `telegram_summary/message_normalization`;
- `telegram_summary/thread_grouping`;
- `telegram_summary/topic_extraction`;
- `telegram_summary/key_message_scoring`;
- `telegram_summary/claim_extraction`;
- `telegram_summary/synthesis`.

MVP может объединять несколько стадий в один combined stage, если итоговый
contract и traceability сохраняются.

---

## 12. Открытые вопросы

Открытые вопросы v1:

- OQ-TS-01 — нужна ли отдельная schema-форма для recursive summaries очень
  длинных reply chains;
- OQ-TS-02 — должны ли веса importance scoring быть runtime-configurable,
  prompt-guided или фиксироваться в pack runtime profile.

Оставшиеся вопросы non-blocking для первой реализации pack.

