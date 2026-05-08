use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use grammers_client::{client::LoginToken, Client};
use grammers_mtsender::SenderPool;
use grammers_session::types::{DcOption, UpdatesState};
use grammers_session::{storages::MemorySession, Session, SessionData};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::secret_store::{telegram_account_api_hash_secret, SecretStoreState};

const STATUS_NOT_INITIALIZED: &str = "not_initialized";
const STATUS_RESTORING: &str = "restoring";
const STATUS_READY: &str = "ready";
const STATUS_REAUTH_REQUIRED: &str = "reauth_required";
const STATUS_RESTORE_FAILED: &str = "restore_failed";
const TELEGRAM_RESTORE_FAILURE_EVENT: &str = "telegram://restore-failure";

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedSession {
    home_dc: i32,
    dc_options: HashMap<i32, DcOption>,
    updates_state: UpdatesState,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountCredentials {
    id: i64,
    api_id: i64,
    api_hash: String,
}

pub struct AccountClient {
    pub client: Client,
    pub session: Arc<MemorySession>,
    pub api_hash: String,
    pub login_token: Option<LoginToken>,
    pub phone: Option<String>,
}

#[derive(Clone)]
pub(crate) struct AuthorizedTelegramRuntime {
    pub(crate) client: Client,
    #[allow(dead_code)]
    pub(crate) session: Arc<MemorySession>,
}

#[derive(Clone, serde::Serialize)]
pub struct AccountRuntimeStatus {
    pub account_id: i64,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Clone, serde::Serialize)]
pub struct RestoreFailureEvent {
    pub message: String,
}

/// Global state: map of account_id -> active client and runtime readiness
pub struct TelegramState {
    pub accounts: Mutex<HashMap<i64, AccountClient>>,
    pub statuses: Mutex<HashMap<i64, AccountRuntimeStatus>>,
}

impl TelegramState {
    pub fn new() -> Self {
        Self {
            accounts: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
        }
    }
}

fn session_path(handle: &AppHandle, account_id: i64) -> AppResult<PathBuf> {
    let app_dir = handle
        .path()
        .app_data_dir()
        .map_err(|error| AppError::internal(error.to_string()))?;
    fs::create_dir_all(&app_dir).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(app_dir.join(format!("telegram_{account_id}.session.json")))
}

fn session_exists(handle: &AppHandle, account_id: i64) -> bool {
    session_path(handle, account_id)
        .map(|path| path.exists())
        .unwrap_or(false)
}

async fn load_session(handle: &AppHandle, account_id: i64) -> Arc<MemorySession> {
    if let Ok(path) = session_path(handle, account_id) {
        if let Ok(json) = fs::read_to_string(&path) {
            if let Ok(saved) = serde_json::from_str::<SavedSession>(&json) {
                let session_data = SessionData {
                    home_dc: saved.home_dc,
                    dc_options: saved.dc_options,
                    peer_infos: HashMap::new(),
                    updates_state: saved.updates_state,
                };
                return Arc::new(MemorySession::from(session_data));
            }
        }
    }
    Arc::new(MemorySession::default())
}

async fn save_session(
    handle: &AppHandle,
    account_id: i64,
    session: &Arc<MemorySession>,
) -> AppResult<()> {
    let home_dc = session.home_dc_id();
    let updates_state = session.updates_state().await;
    let mut dc_options = HashMap::new();
    for dc_id in 1..=5i32 {
        if let Some(dc) = session.dc_option(dc_id) {
            dc_options.insert(dc_id, dc);
        }
    }
    let saved = SavedSession {
        home_dc,
        dc_options,
        updates_state,
    };
    let json = serde_json::to_string(&saved)
        .map_err(|error| AppError::internal(error.to_string()))?;
    let path = session_path(handle, account_id)?;
    fs::write(path, json).map_err(|error| AppError::internal(error.to_string()))
}

async fn list_account_credentials(handle: &AppHandle) -> AppResult<Vec<AccountCredentials>> {
    let pool = get_pool(handle).await?;
    sqlx::query_as("SELECT id, api_id, api_hash FROM accounts ORDER BY created_at ASC")
        .fetch_all(&pool)
        .await
        .map_err(AppError::database)
}

async fn get_account_credentials(
    handle: &AppHandle,
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<AccountCredentials> {
    let pool = get_pool(handle).await?;
    get_account_credentials_from_pool(&pool, secret_store, account_id).await
}

async fn get_account_credentials_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secret_store: &SecretStoreState,
    account_id: i64,
) -> AppResult<AccountCredentials> {
    let credentials: AccountCredentials =
    sqlx::query_as("SELECT id, api_id, api_hash FROM accounts WHERE id = ?")
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?
        .ok_or_else(|| AppError::not_found(format!("Account {account_id} not found")))?;
    resolve_account_credentials(pool, secret_store, credentials).await
}

async fn resolve_account_credentials(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secret_store: &SecretStoreState,
    mut credentials: AccountCredentials,
) -> AppResult<AccountCredentials> {
    let key = telegram_account_api_hash_secret(credentials.id);
    if !credentials.api_hash.trim().is_empty() {
        let api_hash = credentials.api_hash.trim().to_string();
        secret_store.set_secret(key, api_hash.clone()).await?;
        sqlx::query("UPDATE accounts SET api_hash = '' WHERE id = ?")
            .bind(credentials.id)
            .execute(pool)
            .await
            .map_err(AppError::database)?;
        credentials.api_hash = api_hash;
        return Ok(credentials);
    }

    let api_hash = secret_store
        .get_secret(key)
        .await?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AppError::auth(format!(
                "Telegram API hash for account {} is missing from secure storage. Recreate the account credentials.",
                credentials.id
            ))
        })?;
    credentials.api_hash = api_hash;
    Ok(credentials)
}

fn telegram_api_id(api_id: i64) -> AppResult<i32> {
    i32::try_from(api_id).map_err(|_| AppError::validation("Telegram API ID is out of range"))
}

const TELEGRAM_ACCOUNT_STATUS_EVENT: &str = "telegram://account-status";

async fn set_account_status(
    handle: &AppHandle,
    state: &TelegramState,
    account_id: i64,
    status: &str,
    message: Option<String>,
) {
    let runtime_status = AccountRuntimeStatus {
        account_id,
        status: status.to_string(),
        message,
    };

    let mut statuses = state.statuses.lock().await;
    statuses.insert(account_id, runtime_status.clone());
    drop(statuses);

    let _ = handle.emit(TELEGRAM_ACCOUNT_STATUS_EVENT, &runtime_status);
}

pub async fn clear_account_runtime(
    handle: &AppHandle,
    state: &TelegramState,
    account_id: i64,
    sign_out: bool,
) {
    let mut accounts = state.accounts.lock().await;
    if let Some(ac) = accounts.remove(&account_id) {
        if sign_out {
            let _ = ac.client.sign_out().await;
        }
    }
    drop(accounts);

    if let Ok(path) = session_path(handle, account_id) {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    set_account_status(handle, state, account_id, STATUS_NOT_INITIALIZED, None).await;
}

async fn init_account_client(
    handle: &AppHandle,
    state: &TelegramState,
    account_id: i64,
    api_id: i32,
    api_hash: String,
) -> AppResult<bool> {
    set_account_status(handle, state, account_id, STATUS_RESTORING, None).await;

    let session = load_session(handle, account_id).await;
    let pool = SenderPool::new(Arc::clone(&session), api_id);

    tokio::spawn(async move {
        let _ = pool.runner.run().await;
    });

    let client = Client::new(pool.handle);
    let is_auth = client
        .is_authorized()
        .await
        .map_err(AppError::telegram_network)?;

    let mut accounts = state.accounts.lock().await;
    accounts.insert(
        account_id,
        AccountClient {
            client,
            session,
            api_hash,
            login_token: None,
            phone: None,
        },
    );
    drop(accounts);

    let status = if is_auth {
        STATUS_READY
    } else {
        STATUS_REAUTH_REQUIRED
    };
    set_account_status(handle, state, account_id, status, None).await;

    Ok(is_auth)
}

fn restore_failure_message(error: impl std::fmt::Display) -> String {
    let error = error.to_string();
    if error.trim().is_empty() {
        "Unknown restore error".to_string()
    } else {
        error
    }
}

pub async fn restore_telegram_accounts(handle: AppHandle) {
    let state = handle.state::<TelegramState>();
    let secret_store = handle.state::<SecretStoreState>();
    let pool = match get_pool(&handle).await {
        Ok(pool) => pool,
        Err(error) => {
            let message = format!("Failed to load accounts for Telegram restore: {error}");
            eprintln!("{message}");
            let _ = handle.emit(
                TELEGRAM_RESTORE_FAILURE_EVENT,
                &RestoreFailureEvent { message },
            );
            return;
        }
    };
    let accounts = match list_account_credentials(&handle).await {
        Ok(accounts) => accounts,
        Err(error) => {
            let message = format!("Failed to load accounts for Telegram restore: {error}");
            eprintln!("{message}");
            let _ = handle.emit(
                TELEGRAM_RESTORE_FAILURE_EVENT,
                &RestoreFailureEvent { message },
            );
            return;
        }
    };

    for account in accounts {
        if !session_exists(&handle, account.id) {
            set_account_status(&handle, &state, account.id, STATUS_NOT_INITIALIZED, None).await;
            continue;
        }

        let account_id = account.id;
        let account = match resolve_account_credentials(&pool, &secret_store, account).await {
            Ok(account) => account,
            Err(error) => {
                set_account_status(
                    &handle,
                    &state,
                    account_id,
                    STATUS_RESTORE_FAILED,
                    Some(restore_failure_message(error)),
                )
                .await;
                continue;
            }
        };

        let init_result = init_account_client(
            &handle,
            &state,
            account.id,
            match telegram_api_id(account.api_id) {
                Ok(api_id) => api_id,
                Err(error) => {
                    set_account_status(
                        &handle,
                        &state,
                        account.id,
                        STATUS_RESTORE_FAILED,
                        Some(restore_failure_message(error)),
                    )
                    .await;
                    continue;
                }
            },
            account.api_hash,
        )
        .await;

        if let Err(error) = init_result {
            {
                let mut clients = state.accounts.lock().await;
                clients.remove(&account.id);
            }
            set_account_status(
                &handle,
                &state,
                account.id,
                STATUS_RESTORE_FAILED,
                Some(restore_failure_message(error)),
            )
            .await;
        }
    }
}

/// Initialize (or re-initialize) a Telegram client for the given account.
/// Returns true if already authorized.
#[tauri::command]
pub async fn tg_init(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    secret_store: tauri::State<'_, SecretStoreState>,
    account_id: i64,
) -> AppResult<bool> {
    let credentials = get_account_credentials(&handle, &secret_store, account_id).await?;
    let api_id = telegram_api_id(credentials.api_id)?;

    match init_account_client(&handle, &state, account_id, api_id, credentials.api_hash).await {
        Ok(is_auth) => Ok(is_auth),
        Err(error) => {
            let mut accounts = state.accounts.lock().await;
            accounts.remove(&account_id);
            drop(accounts);
            set_account_status(
                &handle,
                &state,
                account_id,
                STATUS_RESTORE_FAILED,
                Some(restore_failure_message(&error)),
            )
            .await;
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn tg_is_authenticated(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> AppResult<bool> {
    let client = {
        let accounts = state.accounts.lock().await;
        accounts.get(&account_id).map(|ac| ac.client.clone())
    };

    if let Some(client) = client {
        Ok(client
            .is_authorized()
            .await
            .map_err(AppError::telegram_network)?)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn tg_get_account_statuses(
    state: tauri::State<'_, TelegramState>,
    account_ids: Vec<i64>,
) -> AppResult<Vec<AccountRuntimeStatus>> {
    let statuses = state.statuses.lock().await;
    Ok(account_ids
        .into_iter()
        .map(|account_id| {
            statuses
                .get(&account_id)
                .cloned()
                .unwrap_or(AccountRuntimeStatus {
                    account_id,
                    status: STATUS_NOT_INITIALIZED.to_string(),
                    message: None,
                })
        })
        .collect())
}

#[tauri::command]
pub async fn tg_send_code(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
    phone: String,
) -> AppResult<String> {
    let mut accounts = state.accounts.lock().await;
    let ac = accounts
        .get_mut(&account_id)
        .ok_or_else(|| AppError::auth("Account not initialized"))?;

    let token = ac
        .client
        .request_login_code(&phone, &ac.api_hash.clone())
        .await
        .map_err(AppError::telegram_network)?;

    ac.phone = Some(phone);
    ac.login_token = Some(token);

    Ok("Code sent".to_string())
}

#[tauri::command]
pub async fn tg_sign_in(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
    code: String,
) -> AppResult<bool> {
    let session_to_save = {
        let mut accounts = state.accounts.lock().await;
        let ac = accounts
            .get_mut(&account_id)
            .ok_or_else(|| AppError::auth("Account not initialized"))?;
        let token = ac
            .login_token
            .as_ref()
            .ok_or_else(|| AppError::auth("Call tg_send_code first"))?;

        ac.client
            .sign_in(token, &code)
            .await
            .map_err(AppError::telegram_network)?;
        ac.login_token = None;
        Arc::clone(&ac.session)
    };

    save_session(&handle, account_id, &session_to_save).await?;
    set_account_status(&handle, &state, account_id, STATUS_READY, None).await;

    Ok(true)
}

#[tauri::command]
pub async fn tg_logout(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> AppResult<bool> {
    clear_account_runtime(&handle, &state, account_id, true).await;
    Ok(true)
}

/// Returns the Client for a given account_id (for use in other modules).
/// Caller must hold the lock.
pub async fn get_client(
    accounts: &HashMap<i64, AccountClient>,
    account_id: i64,
) -> AppResult<&Client> {
    accounts
        .get(&account_id)
        .map(|ac| &ac.client)
        .ok_or_else(|| AppError::auth(format!("Account {account_id} not initialized")))
}

pub(crate) async fn get_authorized_runtime(
    state: &TelegramState,
    account_id: i64,
) -> AppResult<AuthorizedTelegramRuntime> {
    let runtime = {
        let accounts = state.accounts.lock().await;
        let account = accounts
            .get(&account_id)
            .ok_or_else(|| AppError::auth(format!("Account {account_id} not initialized")))?;

        AuthorizedTelegramRuntime {
            client: account.client.clone(),
            session: Arc::clone(&account.session),
        }
    };

    if !runtime
        .client
        .is_authorized()
        .await
        .map_err(AppError::telegram_network)?
    {
        return Err(AppError::auth(format!(
            "Account {account_id} is not authenticated"
        )));
    }

    Ok(runtime)
}

#[cfg(test)]
mod tests {
    use super::{get_account_credentials_from_pool, telegram_api_id};
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

    async fn insert_account(pool: &sqlx::SqlitePool, api_hash: &str) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO accounts (label, api_id, api_hash, created_at) VALUES ('Personal', 12345, ?, 1000) RETURNING id",
        )
        .bind(api_hash)
        .fetch_one(pool)
        .await
        .expect("insert account")
    }

    async fn stored_api_hash(pool: &sqlx::SqlitePool, account_id: i64) -> String {
        sqlx::query_scalar::<_, String>("SELECT api_hash FROM accounts WHERE id = ?")
            .bind(account_id)
            .fetch_one(pool)
            .await
            .expect("read api_hash")
    }

    #[test]
    fn telegram_api_id_out_of_range_returns_typed_validation_error() {
        let error = telegram_api_id(i64::from(i32::MAX) + 1)
            .expect_err("reject out-of-range api id");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Telegram API ID is out of range");
    }

    #[tokio::test]
    async fn legacy_api_hash_migrates_to_secret_store_and_blanks_column() {
        let pool = memory_pool().await;
        let (_store, secret_store) = memory_secret_store();
        let account_id = insert_account(&pool, "legacy-hash").await;

        let credentials = get_account_credentials_from_pool(&pool, &secret_store, account_id)
            .await
            .expect("load credentials");

        assert_eq!(credentials.api_hash, "legacy-hash");
        assert_eq!(stored_api_hash(&pool, account_id).await, "");
        assert_eq!(
            secret_store
                .get_secret(telegram_account_api_hash_secret(account_id))
                .await
                .expect("read secret"),
            Some("legacy-hash".to_string())
        );
    }

    #[tokio::test]
    async fn legacy_api_hash_remains_when_secret_write_fails() {
        let pool = memory_pool().await;
        let (store, secret_store) = memory_secret_store();
        store.fail_set("secure store unavailable");
        let account_id = insert_account(&pool, "legacy-hash").await;

        let error = get_account_credentials_from_pool(&pool, &secret_store, account_id)
            .await
            .expect_err("secret write should fail");

        assert_eq!(error.kind, AppErrorKind::Internal);
        assert_eq!(error.message, "secure store unavailable");
        assert_eq!(stored_api_hash(&pool, account_id).await, "legacy-hash");
    }

    #[tokio::test]
    async fn missing_secure_api_hash_for_blank_legacy_account_is_auth_error() {
        let pool = memory_pool().await;
        let (_store, secret_store) = memory_secret_store();
        let account_id = insert_account(&pool, "").await;

        let error = get_account_credentials_from_pool(&pool, &secret_store, account_id)
            .await
            .expect_err("missing secret should fail");

        assert_eq!(error.kind, AppErrorKind::Auth);
        assert_eq!(
            error.message,
            format!(
                "Telegram API hash for account {account_id} is missing from secure storage. Recreate the account credentials."
            )
        );
    }
}
