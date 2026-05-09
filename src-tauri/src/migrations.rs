#![allow(clippy::items_after_test_module)]

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
        repair_line_ending_migration_checksums(&pool).await;

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
        Migration {
            version: 15,
            description: "add provider source subtype",
            sql: include_str!("../migrations/15.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 16,
            description: "add youtube source foundation",
            sql: include_str!("../migrations/16.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{build_migrations, checksum_matches_line_ending_variant};
    use sha2::{Digest, Sha384};

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

    #[test]
    fn includes_provider_source_subtype_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 15)
            .expect("version 15 migration is registered");

        for fragment in [
            "ALTER TABLE sources ADD COLUMN source_subtype TEXT",
            "SET source_subtype = telegram_source_kind",
            "WHERE source_type = 'telegram'",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn includes_youtube_source_foundation_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 16)
            .expect("version 16 migration is registered");

        for fragment in [
            "ALTER TABLE items ADD COLUMN item_kind TEXT NOT NULL DEFAULT 'telegram_message'",
            "CREATE TABLE IF NOT EXISTS youtube_playlist_items",
            "CHECK (availability_status IN",
            "CREATE TABLE IF NOT EXISTS youtube_transcript_segments",
            "ALTER TABLE analysis_run_messages ADD COLUMN item_kind TEXT",
            "ALTER TABLE analysis_run_messages ADD COLUMN source_type TEXT",
            "ALTER TABLE analysis_run_messages ADD COLUMN source_subtype TEXT",
            "ALTER TABLE analysis_run_messages ADD COLUMN metadata_zstd BLOB",
            "ALTER TABLE analysis_source_groups ADD COLUMN source_type TEXT NOT NULL DEFAULT 'telegram'",
            "idx_sources_unique_youtube_video",
            "idx_sources_unique_youtube_playlist",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn checksum_match_accepts_line_ending_only_differences() {
        let lf_sql = "ALTER TABLE sources ADD COLUMN source_subtype TEXT;\n\n";
        let crlf_sql = lf_sql.replace('\n', "\r\n");
        let applied_checksum = Sha384::digest(lf_sql.as_bytes()).to_vec();

        assert!(checksum_matches_line_ending_variant(
            &applied_checksum,
            crlf_sql.as_str()
        ));
    }
}

fn sha384_bytes(value: &str) -> Vec<u8> {
    Sha384::digest(value.as_bytes()).to_vec()
}

fn normalize_sql_lf(sql: &str) -> String {
    sql.replace("\r\n", "\n")
}

fn normalize_sql_crlf(sql: &str) -> String {
    normalize_sql_lf(sql).replace('\n', "\r\n")
}

fn checksum_matches_line_ending_variant(applied_checksum: &[u8], sql: &str) -> bool {
    let current_checksum = sha384_bytes(sql);
    if applied_checksum == current_checksum {
        return true;
    }

    applied_checksum == sha384_bytes(&normalize_sql_lf(sql))
        || applied_checksum == sha384_bytes(&normalize_sql_crlf(sql))
}

async fn repair_line_ending_migration_checksums(pool: &sqlx::SqlitePool) {
    let migrations = build_migrations();

    for migration in migrations {
        let current_checksum = sha384_bytes(migration.sql);
        let applied_checksum = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT checksum FROM _sqlx_migrations WHERE version = ?",
        )
        .bind(migration.version)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let Some(applied_checksum) = applied_checksum else {
            continue;
        };

        if applied_checksum == current_checksum
            || !checksum_matches_line_ending_variant(&applied_checksum, migration.sql)
        {
            continue;
        }

        let _ = sqlx::query(
            "UPDATE _sqlx_migrations
             SET description = ?, success = 1, checksum = ?
             WHERE version = ?",
        )
        .bind(migration.description)
        .bind(&current_checksum)
        .bind(migration.version)
        .execute(pool)
        .await;
    }
}
