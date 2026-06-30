use crate::error::{database_error, AppResult};
use sqlx::{Pool, Sqlite};

pub(crate) type SqlitePoolConnection = sqlx::pool::PoolConnection<Sqlite>;

pub(crate) async fn begin_immediate(pool: &Pool<Sqlite>) -> AppResult<SqlitePoolConnection> {
    let mut conn = pool.acquire().await.map_err(database_error)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(database_error)?;
    Ok(conn)
}

pub(crate) async fn commit(conn: &mut SqlitePoolConnection) -> AppResult<()> {
    sqlx::query("COMMIT")
        .execute(&mut **conn)
        .await
        .map_err(database_error)?;
    Ok(())
}

pub(crate) async fn rollback(conn: &mut SqlitePoolConnection) -> AppResult<()> {
    sqlx::query("ROLLBACK")
        .execute(&mut **conn)
        .await
        .map_err(database_error)?;
    Ok(())
}

pub(crate) async fn finish_manual_transaction<T>(
    conn: &mut SqlitePoolConnection,
    result: AppResult<T>,
) -> AppResult<T> {
    match result {
        Ok(value) => {
            commit(conn).await?;
            Ok(value)
        }
        Err(error) => {
            let _ = rollback(conn).await;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{begin_immediate, commit, finish_manual_transaction, rollback};
    use crate::error::{AppError, AppErrorKind, AppResult};

    #[tokio::test]
    async fn begin_immediate_commit_persists_changes() {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("connect in-memory db");
        sqlx::query("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create table");

        let mut conn = begin_immediate(&pool).await.expect("begin immediate");
        sqlx::query("INSERT INTO records (name) VALUES ('committed')")
            .execute(&mut *conn)
            .await
            .expect("insert row");
        commit(&mut conn).await.expect("commit");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM records")
            .fetch_one(&pool)
            .await
            .expect("count rows");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn begin_immediate_rollback_discards_changes() {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("connect in-memory db");
        sqlx::query("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create table");

        let mut conn = begin_immediate(&pool).await.expect("begin immediate");
        sqlx::query("INSERT INTO records (name) VALUES ('rolled back')")
            .execute(&mut *conn)
            .await
            .expect("insert row");
        rollback(&mut conn).await.expect("rollback");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM records")
            .fetch_one(&pool)
            .await
            .expect("count rows");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn finish_manual_transaction_commits_success_result() {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("connect in-memory db");
        sqlx::query("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create table");

        let mut conn = begin_immediate(&pool).await.expect("begin immediate");
        let result: AppResult<i64> = async {
            sqlx::query("INSERT INTO records (name) VALUES ('committed')")
                .execute(&mut *conn)
                .await
                .expect("insert row");
            Ok(42)
        }
        .await;

        let value = finish_manual_transaction(&mut conn, result)
            .await
            .expect("finish transaction");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM records")
            .fetch_one(&pool)
            .await
            .expect("count rows");
        assert_eq!(value, 42);
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn finish_manual_transaction_rolls_back_error_result() {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("connect in-memory db");
        sqlx::query("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create table");

        let mut conn = begin_immediate(&pool).await.expect("begin immediate");
        let result: AppResult<i64> = async {
            sqlx::query("INSERT INTO records (name) VALUES ('rolled back')")
                .execute(&mut *conn)
                .await
                .expect("insert row");
            Err(AppError::validation("stop here"))
        }
        .await;

        let error = finish_manual_transaction(&mut conn, result)
            .await
            .expect_err("finish transaction returns original error");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM records")
            .fetch_one(&pool)
            .await
            .expect("count rows");
        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "stop here");
        assert_eq!(count, 0);
    }

    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::time::Duration;

    async fn file_pool(db_path: &std::path::Path, busy_timeout: Duration) -> sqlx::SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(busy_timeout);
        SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .expect("connect file pool")
    }

    // Reproduces the first-launch failure: a DEFERRED transaction that reads, then
    // writes, fails with SQLITE_BUSY_SNAPSHOT (extended code 517, "database is locked")
    // when another connection commits between the read and the write. busy_timeout does
    // NOT rescue a snapshot conflict, so the failure is immediate.
    #[tokio::test]
    async fn deferred_read_then_write_hits_busy_snapshot_under_concurrent_writer() {
        let dir = tempfile::tempdir().expect("temp dir");
        let pool = file_pool(&dir.path().join("snap.db"), Duration::from_millis(200)).await;
        sqlx::query("CREATE TABLE t (id INTEGER PRIMARY KEY, n INTEGER NOT NULL)")
            .execute(&pool)
            .await
            .expect("create table");
        sqlx::query("INSERT INTO t (n) VALUES (1)")
            .execute(&pool)
            .await
            .expect("seed row");

        // Connection A: deferred BEGIN, then read -> takes a fixed WAL read snapshot.
        let mut a = pool.acquire().await.expect("acquire A");
        sqlx::query("BEGIN")
            .execute(&mut *a)
            .await
            .expect("begin deferred");
        let _read: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM t")
            .fetch_one(&mut *a)
            .await
            .expect("read snapshot");

        // Connection B: write + commit, advancing the WAL past A's snapshot.
        {
            let mut b = pool.acquire().await.expect("acquire B");
            sqlx::query("INSERT INTO t (n) VALUES (2)")
                .execute(&mut *b)
                .await
                .expect("concurrent write commits");
        }

        // Connection A upgrades read -> write and must fail with a busy snapshot.
        let err = sqlx::query("INSERT INTO t (n) VALUES (3)")
            .execute(&mut *a)
            .await
            .expect_err("deferred read->write upgrade must fail under a concurrent writer");
        assert!(
            err.to_string().to_lowercase().contains("locked"),
            "expected a 'database is locked' busy snapshot, got: {err}"
        );
    }

    // Proves the fix: BEGIN IMMEDIATE takes the write lock up front, so the same
    // read-then-write cycle completes even while another connection contends for writes.
    #[tokio::test]
    async fn begin_immediate_read_then_write_survives_concurrent_writer() {
        let dir = tempfile::tempdir().expect("temp dir");
        let pool = file_pool(&dir.path().join("immediate.db"), Duration::from_secs(5)).await;
        sqlx::query("CREATE TABLE t (id INTEGER PRIMARY KEY, n INTEGER NOT NULL)")
            .execute(&pool)
            .await
            .expect("create table");
        sqlx::query("INSERT INTO t (n) VALUES (1)")
            .execute(&pool)
            .await
            .expect("seed row");

        // Connection A: BEGIN IMMEDIATE acquires the write lock before reading.
        let mut a = begin_immediate(&pool).await.expect("begin immediate");
        let _read: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM t")
            .fetch_one(&mut *a)
            .await
            .expect("read under immediate");

        // A concurrent writer contends for the write lock; it must wait, not corrupt A.
        let pool_b = pool.clone();
        let writer = tokio::spawn(async move {
            sqlx::query("INSERT INTO t (n) VALUES (99)")
                .execute(&pool_b)
                .await
        });

        // A's read -> write cycle succeeds; there is no snapshot to invalidate.
        sqlx::query("INSERT INTO t (n) VALUES (2)")
            .execute(&mut *a)
            .await
            .expect("write under immediate must succeed");
        commit(&mut a).await.expect("commit");

        writer
            .await
            .expect("join writer")
            .expect("contending writer eventually succeeds");
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM t")
            .fetch_one(&pool)
            .await
            .expect("count rows");
        assert_eq!(count, 3);
    }
}
