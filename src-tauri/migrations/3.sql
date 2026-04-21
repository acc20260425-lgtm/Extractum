-- Add accounts table and link sources to accounts

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT NOT NULL,
    api_id INTEGER NOT NULL,
    api_hash TEXT NOT NULL,
    phone TEXT,
    created_at INTEGER NOT NULL
);

-- Add account_id to sources (nullable for backward compat, will be set on use)
ALTER TABLE sources ADD COLUMN account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE;
