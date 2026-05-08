use serde::Serialize;
use sqlx::{Pool, Sqlite};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::secret_store::{telegram_account_api_hash_secret, SecretStoreState};
use crate::telegram::{clear_account_runtime, TelegramState};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AccountRecord {
    pub id: i64,
    pub label: String,
    pub api_id: i64,
    pub phone: Option<String>,
    pub created_at: i64,
}

#[tauri::command]
pub async fn list_accounts(handle: AppHandle) -> AppResult<Vec<AccountRecord>> {
    let pool = get_pool(&handle).await?;
    Ok(sqlx::query_as(
        "SELECT id, label, api_id, phone, created_at FROM accounts ORDER BY created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::database)?)
}

#[tauri::command]
pub async fn get_account(handle: AppHandle, account_id: i64) -> AppResult<Option<AccountRecord>> {
    let pool = get_pool(&handle).await?;
    Ok(
        sqlx::query_as("SELECT id, label, api_id, phone, created_at FROM accounts WHERE id = ?")
            .bind(account_id)
            .fetch_optional(&pool)
            .await
            .map_err(AppError::database)?,
    )
}

#[tauri::command]
pub async fn create_account(
    handle: AppHandle,
    secret_store: tauri::State<'_, SecretStoreState>,
    label: String,
    api_id: i64,
    api_hash: String,
) -> AppResult<AccountRecord> {
    let pool = get_pool(&handle).await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    create_account_in_pool(&pool, &secret_store, label, api_id, api_hash, now).await
}

pub(crate) async fn create_account_in_pool(
    pool: &Pool<Sqlite>,
    secret_store: &SecretStoreState,
    label: String,
    api_id: i64,
    api_hash: String,
    created_at: i64,
) -> AppResult<AccountRecord> {
    let label = label.trim().to_string();
    if label.is_empty() {
        return Err(AppError::validation("Account label cannot be empty"));
    }
    if api_hash.trim().is_empty() {
        return Err(AppError::validation("Telegram API hash cannot be empty"));
    }
    i32::try_from(api_id).map_err(|_| AppError::validation("Telegram API ID is out of range"))?;

    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let account: AccountRecord = sqlx::query_as(
        "INSERT INTO accounts (label, api_id, api_hash, created_at) VALUES (?, ?, '', ?) RETURNING id, label, api_id, phone, created_at",
    )
    .bind(&label)
    .bind(api_id)
    .bind(created_at)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::database)?;

    let secret_key = telegram_account_api_hash_secret(account.id);
    if let Err(error) = secret_store.set_secret(secret_key.clone(), api_hash.trim()).await {
        let _ = tx.rollback().await;
        return Err(error);
    }

    if let Err(error) = tx.commit().await {
        let _ = secret_store.delete_secret(secret_key).await;
        return Err(AppError::database(error));
    }

    Ok(account)
}

#[tauri::command]
pub async fn set_account_phone(handle: AppHandle, account_id: i64, phone: String) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    sqlx::query("UPDATE accounts SET phone = ? WHERE id = ?")
        .bind(&phone)
        .bind(account_id)
        .execute(&pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

#[tauri::command]
pub async fn clear_account_phone(handle: AppHandle, account_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    sqlx::query("UPDATE accounts SET phone = NULL WHERE id = ?")
        .bind(account_id)
        .execute(&pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_account(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    secret_store: tauri::State<'_, SecretStoreState>,
    account_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_account_row_from_pool(&pool, account_id).await?;
    clear_account_runtime(&handle, &state, account_id, true).await;
    secret_store
        .delete_secret(telegram_account_api_hash_secret(account_id))
        .await
}

#[cfg(test)]
pub(crate) async fn delete_account_from_pool(
    pool: &Pool<Sqlite>,
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<()> {
    delete_account_row_from_pool(pool, account_id).await?;
    secret_store
        .delete_secret(telegram_account_api_hash_secret(account_id))
        .await
}

async fn delete_account_row_from_pool(pool: &Pool<Sqlite>, account_id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{create_account_in_pool, delete_account_from_pool};
    use crate::error::AppErrorKind;
    use crate::secret_store::tests::InMemorySecretStore;
    use crate::secret_store::{telegram_account_api_hash_secret, SecretStoreState};
    use std::sync::Arc;

    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                label TEXT NOT NULL,
                api_id INTEGER NOT NULL,
                api_hash TEXT NOT NULL,
                phone TEXT,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create accounts");
        pool
    }

    fn memory_secret_store() -> (Arc<InMemorySecretStore>, SecretStoreState) {
        let store = Arc::new(InMemorySecretStore::new());
        let state = SecretStoreState::new(store.clone());
        (store, state)
    }

    async fn stored_api_hash(pool: &sqlx::SqlitePool, account_id: i64) -> Option<String> {
        sqlx::query_scalar::<_, String>("SELECT api_hash FROM accounts WHERE id = ?")
            .bind(account_id)
            .fetch_optional(pool)
            .await
            .expect("read api_hash")
    }

    async fn account_count(pool: &sqlx::SqlitePool) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM accounts")
            .fetch_one(pool)
            .await
            .expect("count accounts")
    }

    #[tokio::test]
    async fn creating_account_writes_api_hash_to_secret_store_only() {
        let pool = memory_pool().await;
        let (_store, secret_store) = memory_secret_store();

        let account = create_account_in_pool(
            &pool,
            &secret_store,
            "Personal".to_string(),
            12345,
            "api-hash".to_string(),
            1000,
        )
        .await
        .expect("create account");

        assert_eq!(account.label, "Personal");
        assert_eq!(stored_api_hash(&pool, account.id).await, Some("".to_string()));
        assert_eq!(
            secret_store
                .get_secret(telegram_account_api_hash_secret(account.id))
                .await
                .expect("read secret"),
            Some("api-hash".to_string())
        );
    }

    #[tokio::test]
    async fn creating_account_rolls_back_when_secret_write_fails() {
        let pool = memory_pool().await;
        let (store, secret_store) = memory_secret_store();
        store.fail_set("secure store unavailable");

        let error = create_account_in_pool(
            &pool,
            &secret_store,
            "Personal".to_string(),
            12345,
            "api-hash".to_string(),
            1000,
        )
        .await
        .expect_err("secret write should fail");

        assert_eq!(error.kind, AppErrorKind::Internal);
        assert_eq!(error.message, "secure store unavailable");
        assert_eq!(account_count(&pool).await, 0);
    }

    #[tokio::test]
    async fn deleting_account_removes_secret_after_database_row() {
        let pool = memory_pool().await;
        let (_store, secret_store) = memory_secret_store();
        let account = create_account_in_pool(
            &pool,
            &secret_store,
            "Personal".to_string(),
            12345,
            "api-hash".to_string(),
            1000,
        )
        .await
        .expect("create account");

        delete_account_from_pool(&pool, &secret_store, account.id)
            .await
            .expect("delete account");

        assert_eq!(account_count(&pool).await, 0);
        assert_eq!(
            secret_store
                .get_secret(telegram_account_api_hash_secret(account.id))
                .await
                .expect("read secret"),
            None
        );
    }
}
