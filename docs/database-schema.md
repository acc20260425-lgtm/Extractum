# Database Schema Design (SQLite + ZSTD Compression)

## Архитектурное решение (Storage)
База данных используется исключительно как надежное и компактное хранилище (Key-Value/Document Store) с возможностью быстрых выборок по метаданным (дата, автор, источник). Полнотекстовый поиск средствами БД не используется.

Для минимизации размера файла базы данных на диске пользователя, тяжелые текстовые поля (`content`, `raw_data`) сжимаются на стороне Rust-бэкенда с использованием алгоритма **Zstandard (zstd)** перед записью и сохраняются в бинарном формате `BLOB`.

## Таблицы

### 1. `sources` (Источники данных)
Хранит информацию о каналах (например, Telegram).

| Колонка | Тип | Описание | Индекс |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | Первичный ключ (PK), автоинкремент | PK |
| `source_type` | TEXT | Тип источника (`'telegram_channel'`, и т.д.) | |
| `external_id` | TEXT | Уникальный ID в целевой системе (ID канала) | Уникальный (вместе с source_type) |
| `title` | TEXT | Название источника | |
| `metadata` | BLOB | Сжатый JSON (zstd) с доп. данными (аватар и т.д.) | |
| `last_sync_state` | TEXT | Состояние синхронизации (ID последнего сообщения) | |
| `is_active` | BOOLEAN | Флаг фоновой синхронизации (1/0) | |
| `created_at` | DATETIME | Дата добавления источника | |

### 2. `items` (Сообщения / Контент)
Хранилище собранных данных. Оптимизировано для быстрых выборок по времени и источнику.

| Колонка | Тип | Описание | Индекс |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | Первичный ключ (PK), автоинкремент | PK |
| `source_id` | INTEGER | Внешний ключ (FK) на `sources.id` | Да |
| `external_id` | TEXT | ID сообщения в целевой системе | Уникальный (вместе с source_id) |
| `author` | TEXT | Имя автора / отправителя (опционально) | Да |
| `published_at` | DATETIME | Оригинальная дата публикации (Unix Timestamp) | Да (Для выборок по диапазону дат) |
| `content_zstd` | BLOB | Сжатый (zstd) основной текст сообщения | Нет |
| `raw_data_zstd` | BLOB | Сжатый (zstd) полный сырой ответ API (JSON) | Нет |
| `is_embedded` | BOOLEAN | Флаг: обработано ли для векторной БД (1/0) | Да |

### 3. `app_settings` (Настройки приложения)
Хранение конфигурации.

| Колонка | Тип | Описание |
| :--- | :--- | :--- |
| `key` | TEXT | Уникальный ключ настройки (PK) |
| `value` | TEXT | Значение |

## Индексы (Оптимизация выборок)
Для обеспечения мгновенных `SELECT` запросов по заданным сценариям:

```sql
-- Уникальность сообщений и источников
CREATE UNIQUE INDEX idx_sources_ext ON sources(source_type, external_id);
CREATE UNIQUE INDEX idx_items_ext ON items(source_id, external_id);

-- Быстрые выборки сообщений по источнику и дате (основной сценарий)
CREATE INDEX idx_items_source_date ON items(source_id, published_at DESC);

-- Быстрые выборки по автору
CREATE INDEX idx_items_author ON items(author);

-- Поиск необработанных сообщений для векторатора
CREATE INDEX idx_items_embedded ON items(is_embedded) WHERE is_embedded = 0;
```

## Взаимодействие с Rust Backend
1. **Запись (Insert):** Данные из Telegram API -> Сериализация в JSON (если нужно) -> `zstd::encode` -> `INSERT INTO items (..., content_zstd, raw_data_zstd) VALUES (..., ?, ?)`.
2. **Чтение (Select):** `SELECT content_zstd FROM items WHERE source_id = ? AND published_at > ?` -> `zstd::decode` -> Десериализация/Отдача на Frontend или LLM.
