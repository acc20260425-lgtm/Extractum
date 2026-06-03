# Takeout Forum Topic Refresh Policy - Historical Note

> Status: shipped and archived.

## Decision

Completed supergroup Takeout imports can refresh forum topic metadata. Topic
refresh failures are recorded as sanitized warnings rather than blocking the
entire import.

## Rationale

- Topic names and activity improve source browsing and NotebookLM context.
- A Takeout import can still be valuable when topic refresh is partial.
- Warning codes keep failure visibility without exposing raw provider data.

## Preserved Contract

- Only completed supergroup Takeout imports attempt the refresh path.
- `forum_topic_refresh_failed` remains a recoverable warning category.
- Topic refresh should not mutate unrelated source identity.
