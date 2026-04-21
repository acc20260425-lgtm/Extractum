# Database Schema Design (SQLite + ZSTD Compression) v3

## 1. Storage Architecture

SQLite is used as the single local storage layer for the MVP.

The database acts as a compact and reliable local store for:
- source metadata;
- collected content items;
- application settings;
- fast filtered selection by metadata such as source, date, and author.

Database-level full-text search is not part of the MVP.  
The application selects relevant records through standard SQL queries and sends the resulting context to the LLM for analysis.

To reduce the on-disk database size, heavy text fields are compressed in the Rust backend using **Zstandard (zstd)** and stored as `BLOB` values.

## 2. Design Principles

The schema is designed around the following principles:
- one generic table for data sources;
- one generic table for collected content items;
- one simple table for application settings;
- optimized reads for the most common UI and LLM workflows;
- no vectorization lifecycle, no embedding queue, no semantic index.

This keeps the storage model simple, portable, and aligned with the MVP scope.

## 3. Tables

### 3.1 `sources`
Stores configured data sources such as Telegram channels.

| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | INTEGER | Primary key, autoincrement |
| `source_type` | TEXT | Source type, for example `telegram_channel` |
| `external_id` | TEXT | External identifier in the target system |
| `title` | TEXT | Human-readable source title |
| `metadata_zstd` | BLOB | ZSTD-compressed JSON metadata |
| `last_sync_state` | INTEGER | Sync cursor or checkpoint: `message_id` of the last synced message |
| `is_active` | BOOLEAN | Whether the source participates in sync |
| `is_member` | BOOLEAN | Whether the user is subscribed to this source |
| `created_at` | INTEGER | Unix Timestamp, UTC |

### 3.2 `items`
Stores collected content records such as Telegram messages.

| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | INTEGER | Primary key, autoincrement |
| `source_id` | INTEGER | Foreign key to `sources.id` (`ON DELETE CASCADE`) |
| `external_id` | TEXT | External item identifier in the source system |
| `author` | TEXT | Optional author or sender name |
| `published_at` | INTEGER | Original publication timestamp |
| `ingested_at` | INTEGER | Unix Timestamp, UTC (когда попало в базу) |
| `content_zstd` | BLOB | ZSTD-compressed normalized text content |
| `raw_data_zstd` | BLOB | ZSTD-compressed raw API payload |

### 3.3 `app_settings`
Stores local application settings as simple key-value pairs.

| Column | Type | Description |
| :--- | :--- | :--- |
| `key` | TEXT | Primary key |
| `value` | TEXT | Setting value |

## 4. Constraints and Indexes

The schema should enforce uniqueness for both sources and collected items, and should optimize the primary read paths used by the UI and the LLM preparation flow.

```sql
-- Sources
CREATE UNIQUE INDEX idx_sources_ext
ON sources(source_type, external_id);

-- Items
CREATE UNIQUE INDEX idx_items_ext
ON items(source_id, external_id);

CREATE INDEX idx_items_source_date
ON items(source_id, published_at DESC);

CREATE INDEX idx_items_author
ON items(author);
```


### 4.1 Why these indexes

- `idx_sources_ext` prevents duplicate registration of the same external source.
- `idx_items_ext` prevents duplicate storage of the same message inside one source.
- `idx_items_source_date` supports the main browsing and filtering scenario by source and time range.
- `idx_items_author` supports optional filtering by sender or author.

No index for embeddings or semantic retrieval is needed, because the MVP does not contain a vector-processing pipeline.

<h2>5. Compression Strategy</h2>

The backend compresses large fields before writing them into SQLite:

- normalized text content goes into `content_zstd`;
- original API payload goes into `raw_data_zstd`;
- optional source metadata goes into `metadata_zstd`.

This approach gives three advantages:

- smaller local database size;
- preservation of both normalized and raw forms of the data;
- flexibility for future reprocessing without needing to re-fetch everything from Telegram.


<h2>6. Backend Interaction Model</h2>

<h3>6.1 Insert Flow</h3>

<ol>
<li>Backend receives data from Telegram MTProto.</li>
<li>Backend normalizes fields needed for UI and analysis.</li>
<li>Backend serializes raw payloads when needed.</li>
<li>Backend compresses large payloads with `zstd`.</li>
<li>Backend inserts rows into `sources` and `items`.</li>
</ol>

Conceptually:

```sql
INSERT INTO items (
  source_id,
  external_id,
  author,
  published_at,
  content_zstd,
  raw_data_zstd
) VALUES (?, ?, ?, ?, ?, ?);
```


<h3>6.2 Select Flow</h3>

<ol>
<li>Frontend requests records using filters.</li>
<li>Backend executes parameterized SQL queries.</li>
<li>Backend decompresses `content_zstd` and, if needed, `raw_data_zstd`.</li>
<li>Backend returns ready-to-use records to the frontend or directly to the LLM request pipeline.</li>
</ol>

Conceptually:

```sql
SELECT
  id,
  source_id,
  external_id,
  author,
  published_at,
  content_zstd,
  raw_data_zstd
FROM items
WHERE source_id = ?
  AND published_at >= ?
  AND published_at <= ?
ORDER BY published_at DESC
LIMIT ?;
```


<h2>7. Relationship to LLM Analysis</h2>

This schema is intentionally built for a **SQL-first analysis flow**:

<ul>
<li>records are stored locally in SQLite;</li>
<li>frontend or backend selects relevant rows with ordinary SQL;</li>
<li>decompressed text is assembled into context blocks;</li>
<li>the resulting context is sent to a configured LLM provider.</li>
</ul>

This means the schema is optimized for deterministic filtering and structured retrieval, not for embedding-based nearest-neighbor search.

<h2>8. Example Query Scenarios</h2>

Typical queries supported by this schema include:

<ul>
<li>latest messages from a source;</li>
<li>messages from a source in a date range;</li>
<li>messages by author;</li>
<li>limited record batches for UI pages;</li>
<li>selected subsets of records for LLM analysis.</li>
</ul>

Examples:

```sql
-- Latest items for one source
SELECT id, published_at, content_zstd
FROM items
WHERE source_id = ?
ORDER BY published_at DESC
LIMIT 100;
```

```sql
-- Items for one source in a date range
SELECT id, author, published_at, content_zstd
FROM items
WHERE source_id = ?
  AND published_at BETWEEN ? AND ?
ORDER BY published_at DESC;
```

```sql
-- Items by author inside one source
SELECT id, published_at, content_zstd
FROM items
WHERE source_id = ?
  AND author = ?
ORDER BY published_at DESC;
```


<h2>9. Future Extensions</h2>

The schema is intentionally generic enough to support later expansion:

<ul>
<li>additional <code>source_type</code> values;</li>
<li>richer metadata in compressed JSON fields;</li>
<li>optional tagging or classification tables;</li>
<li>optional cached analysis results.</li>
</ul>

These extensions can be added later without changing the MVP storage principle that SQLite remains the core local store.

<h2>10. Summary of MVP Storage Boundaries</h2>

The MVP schema includes:

<ul>
<li><code>sources</code></li>
<li><code>items</code></li>
<li><code>app_settings</code></li>
<li>compressed storage of large payloads</li>
<li>SQL-based filtered retrieval.</li>
</ul>

The MVP schema does not include:

<ul>
<li>embeddings;</li>
<li>vector indexes;</li>
<li>semantic retrieval queues;</li>
<li>vector database synchronization state.</li>
</ul>
