CREATE INDEX IF NOT EXISTS idx_analysis_documents_item_id
    ON analysis_documents(item_id)
    WHERE item_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_archive_read_items_item_id
    ON archive_read_items(item_id);
