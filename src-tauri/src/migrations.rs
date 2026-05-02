use sha2::{Digest, Sha384};
use std::path::{Path, PathBuf};
use tauri_plugin_sql::{Migration, MigrationKind};

const APP_IDENTIFIER: &str = "org.ai.extractum";
const DB_FILENAME: &str = "extractum.db";

/// Before the sql plugin runs, remove stale migration records whose SQL has changed.
/// This allows us to update migration files without deleting the database.
async fn patch_migrations(db_path: &Path) {
    use sqlx::SqlitePool;

    if !db_path.exists() {
        return;
    }

    let url = format!("sqlite:{}", db_path.to_string_lossy());
    if let Ok(pool) = SqlitePool::connect(&url).await {
        let expected_checksum =
            Sha384::digest(include_str!("../migrations/2.sql").as_bytes()).to_vec();
        let has_v3 = sqlx::query_scalar::<_, i64>(
            "SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version = 3)",
        )
        .fetch_one(&pool)
        .await
        .map(|exists| exists != 0)
        .unwrap_or(false);

        let v2_checksum = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT checksum FROM _sqlx_migrations WHERE version = 2",
        )
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();

        match v2_checksum {
            Some(checksum) if checksum != expected_checksum => {
                if has_v3 {
                    // Once later migrations are applied, deleting v2 leaves a gap that sqlx will not backfill.
                    // Update the metadata in place so startup validation passes without replaying schema changes.
                    let _ = sqlx::query(
                        "UPDATE _sqlx_migrations
                         SET description = ?, success = 1, checksum = ?
                         WHERE version = 2",
                    )
                    .bind("add is_member to sources")
                    .bind(&expected_checksum)
                    .execute(&pool)
                    .await;
                } else {
                    // Safe only before later migrations exist: let sqlx replay the no-op v2 with the new checksum.
                    let _ = sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 2")
                        .execute(&pool)
                        .await;
                }
            }
            None if has_v3 => {
                // Repair older upgraded databases that lost v2 metadata after the previous patch strategy.
                let _ = sqlx::query(
                    "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
                     VALUES (?, ?, 1, ?, 0)",
                )
                .bind(2_i64)
                .bind("add is_member to sources")
                .bind(&expected_checksum)
                .execute(&pool)
                .await;
            }
            _ => {}
        }

        pool.close().await;
    }
}

fn app_config_db_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(APP_IDENTIFIER).join(DB_FILENAME))
}

pub fn prepare_database() {
    if let Some(db_path) = app_config_db_path() {
        tauri::async_runtime::block_on(patch_migrations(&db_path));
    }
}

pub fn build_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "initialize storage",
            sql: include_str!("../migrations/1.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add is_member to sources",
            sql: include_str!("../migrations/2.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "add accounts table",
            sql: include_str!("../migrations/3.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "add last synced at to sources",
            sql: include_str!("../migrations/4.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "add analysis storage",
            sql: include_str!("../migrations/5.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "add analysis source groups",
            sql: include_str!("../migrations/6.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "add source group id to analysis runs",
            sql: include_str!("../migrations/7.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 8,
            description: "add analysis chat history",
            sql: include_str!("../migrations/8.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 9,
            description: "add media aware item metadata",
            sql: include_str!("../migrations/9.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 10,
            description: "add analysis run snapshots",
            sql: include_str!("../migrations/10.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 11,
            description: "add telegram source kind",
            sql: include_str!("../migrations/11.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 12,
            description: "scope source uniqueness by account",
            sql: include_str!("../migrations/12.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 13,
            description: "add telegram item context metadata",
            sql: include_str!("../migrations/13.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 14,
            description: "add telegram forum topics",
            sql: include_str!("../migrations/14.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::build_migrations;

    #[test]
    fn includes_telegram_item_context_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 13)
            .expect("version 13 migration is registered");

        for column in [
            "reply_to_msg_id",
            "reply_to_peer_kind",
            "reply_to_peer_id",
            "reply_to_top_id",
            "reaction_count",
        ] {
            assert!(migration.sql.contains(column), "missing column {column}");
        }
    }

    #[test]
    fn includes_telegram_forum_topics_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 14)
            .expect("version 14 migration is registered");

        for fragment in [
            "CREATE TABLE IF NOT EXISTS telegram_forum_topics",
            "topic_id INTEGER NOT NULL",
            "top_message_id INTEGER NOT NULL",
            "FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE",
            "idx_telegram_forum_topics_source_topic",
            "idx_telegram_forum_topics_source_top_message",
            "idx_items_source_reply_to_top",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }
}
