use crate::error::{database_error, AppResult};
use sqlx::{Pool, Sqlite, SqliteConnection};

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

#[allow(dead_code)]
pub(crate) async fn begin_immediate_on_connection(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(database_error)?;
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn commit_connection(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::query("COMMIT")
        .execute(&mut *conn)
        .await
        .map_err(database_error)?;
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn rollback_connection(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::query("ROLLBACK")
        .execute(&mut *conn)
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

#[allow(dead_code)]
pub(crate) async fn finish_connection_transaction<T>(
    conn: &mut SqliteConnection,
    result: AppResult<T>,
) -> AppResult<T> {
    match result {
        Ok(value) => {
            commit_connection(conn).await?;
            Ok(value)
        }
        Err(error) => {
            let _ = rollback_connection(conn).await;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        begin_immediate, begin_immediate_on_connection, commit, finish_connection_transaction,
        finish_manual_transaction, rollback,
    };
    use crate::error::{AppError, AppErrorKind, AppResult};
    use sqlx::Connection;

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

    #[tokio::test]
    async fn finish_connection_transaction_commits_success_result() {
        let mut conn = sqlx::SqliteConnection::connect(":memory:")
            .await
            .expect("connect in-memory db");
        sqlx::query("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&mut conn)
            .await
            .expect("create table");

        begin_immediate_on_connection(&mut conn)
            .await
            .expect("begin immediate");
        let result: AppResult<i64> = async {
            sqlx::query("INSERT INTO records (name) VALUES ('committed')")
                .execute(&mut conn)
                .await
                .expect("insert row");
            Ok(42)
        }
        .await;

        let value = finish_connection_transaction(&mut conn, result)
            .await
            .expect("finish transaction");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM records")
            .fetch_one(&mut conn)
            .await
            .expect("count rows");
        assert_eq!(value, 42);
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn finish_connection_transaction_rolls_back_error_result() {
        let mut conn = sqlx::SqliteConnection::connect(":memory:")
            .await
            .expect("connect in-memory db");
        sqlx::query("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&mut conn)
            .await
            .expect("create table");

        begin_immediate_on_connection(&mut conn)
            .await
            .expect("begin immediate");
        let result: AppResult<i64> = async {
            sqlx::query("INSERT INTO records (name) VALUES ('rolled back')")
                .execute(&mut conn)
                .await
                .expect("insert row");
            Err(AppError::validation("stop here"))
        }
        .await;

        let error = finish_connection_transaction(&mut conn, result)
            .await
            .expect_err("finish transaction returns original error");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM records")
            .fetch_one(&mut conn)
            .await
            .expect("count rows");
        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "stop here");
        assert_eq!(count, 0);
    }
}
