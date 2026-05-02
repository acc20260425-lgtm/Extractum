CREATE TABLE IF NOT EXISTS telegram_forum_topics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    topic_id INTEGER NOT NULL,
    top_message_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    icon_color INTEGER,
    icon_emoji_id INTEGER,
    is_closed BOOLEAN NOT NULL DEFAULT 0,
    is_pinned BOOLEAN NOT NULL DEFAULT 0,
    is_hidden BOOLEAN NOT NULL DEFAULT 0,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    sort_order INTEGER,
    last_seen_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_telegram_forum_topics_source_topic
    ON telegram_forum_topics(source_id, topic_id);

CREATE INDEX IF NOT EXISTS idx_telegram_forum_topics_source_top_message
    ON telegram_forum_topics(source_id, top_message_id);

CREATE INDEX IF NOT EXISTS idx_items_source_reply_to_top
    ON items(source_id, reply_to_top_id);
