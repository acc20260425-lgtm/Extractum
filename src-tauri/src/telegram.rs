use std::collections::HashMap;

use secrecy::{ExposeSecret, SecretString};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::secret_store::{telegram_account_api_hash_secret, SecretStoreState};
use crate::telegram_impl::{
    TelegramApiHash, TelegramClientHandle, TelegramRuntime, TelegramRuntimeStatus,
};
use crate::telegram_session_store;

const STATUS_NOT_INITIALIZED: &str = "not_initialized";
const STATUS_RESTORING: &str = "restoring";
const STATUS_READY: &str = "ready";
const STATUS_REAUTH_REQUIRED: &str = "reauth_required";
const STATUS_RESTORE_FAILED: &str = "restore_failed";
const TELEGRAM_RESTORE_FAILURE_EVENT: &str = "telegram://restore-failure";

fn runtime_status_to_wire(status: TelegramRuntimeStatus) -> &'static str {
    match status {
        TelegramRuntimeStatus::Ready => STATUS_READY,
        TelegramRuntimeStatus::ReauthRequired => STATUS_REAUTH_REQUIRED,
    }
}

#[derive(sqlx::FromRow)]
struct AccountCredentialsRow {
    id: i64,
    api_id: i64,
    api_hash: String,
}

struct AccountCredentials {
    id: i64,
    api_id: i64,
    api_hash: TelegramApiHash,
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
    runtime: TelegramRuntime,
    statuses: Mutex<HashMap<i64, AccountRuntimeStatus>>,
}

impl TelegramState {
    pub fn new() -> Self {
        Self {
            runtime: TelegramRuntime::new(),
            statuses: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn client(&self, account_id: i64) -> AppResult<TelegramClientHandle> {
        self.runtime.client(account_id).await
    }

    pub(crate) async fn authorized_client(
        &self,
        account_id: i64,
    ) -> AppResult<TelegramClientHandle> {
        self.runtime.authorized_client(account_id).await
    }

    pub(crate) async fn diagnostic_status_counts(&self, account_ids: &[i64]) -> Vec<(String, i64)> {
        let statuses = self.statuses.lock().await;
        let mut counts = std::collections::BTreeMap::<String, i64>::new();
        for account_id in account_ids {
            let status = statuses
                .get(account_id)
                .map(|status| status.status.clone())
                .unwrap_or_else(|| STATUS_NOT_INITIALIZED.to_string());
            *counts.entry(status).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    }
}

async fn list_account_credentials(handle: &AppHandle) -> AppResult<Vec<AccountCredentialsRow>> {
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
    let credentials: AccountCredentialsRow =
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
    credentials: AccountCredentialsRow,
) -> AppResult<AccountCredentials> {
    let AccountCredentialsRow {
        id,
        api_id,
        mut api_hash,
    } = credentials;
    let key = telegram_account_api_hash_secret(id);
    if !api_hash.trim().is_empty() {
        let trimmed_end = api_hash.trim_end().len();
        api_hash.truncate(trimmed_end);
        let trimmed_start = api_hash.len() - api_hash.trim_start().len();
        api_hash.drain(..trimmed_start);
        let api_hash = SecretString::new(api_hash);
        secret_store
            .set_secret(key, api_hash.expose_secret())
            .await?;
        sqlx::query("UPDATE accounts SET api_hash = '' WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(AppError::database)?;
        return Ok(AccountCredentials {
            id,
            api_id,
            api_hash: TelegramApiHash::new(api_hash),
        });
    }

    let api_hash = secret_store
        .get_secret(key)
        .await?
        .filter(|value| !value.expose_secret().trim().is_empty())
        .ok_or_else(|| {
            AppError::auth(format!(
                "Telegram API hash for account {} is missing from secure storage. Recreate the account credentials.",
                id
            ))
        })?;
    Ok(AccountCredentials {
        id,
        api_id,
        api_hash: TelegramApiHash::new(api_hash),
    })
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
    set_account_status_with(state, account_id, status, message, |runtime_status| {
        let _ = handle.emit(TELEGRAM_ACCOUNT_STATUS_EVENT, runtime_status);
    })
    .await;
}

async fn set_account_status_with(
    state: &TelegramState,
    account_id: i64,
    status: &str,
    message: Option<String>,
    emit: impl FnOnce(&AccountRuntimeStatus),
) {
    let runtime_status = AccountRuntimeStatus {
        account_id,
        status: status.to_string(),
        message,
    };

    let mut statuses = state.statuses.lock().await;
    statuses.insert(account_id, runtime_status.clone());
    drop(statuses);

    emit(&runtime_status);
}

pub async fn clear_account_runtime(
    handle: &AppHandle,
    state: &TelegramState,
    secret_store: &SecretStoreState,
    account_id: i64,
    sign_out: bool,
) -> AppResult<()> {
    clear_account_runtime_with(
        state.runtime.clear_account(account_id, sign_out),
        telegram_session_store::delete_session(handle, secret_store, account_id),
        set_account_status(handle, state, account_id, STATUS_NOT_INITIALIZED, None),
    )
    .await
}

async fn clear_account_runtime_with<RuntimeFuture, SessionFuture, StatusFuture>(
    clear_runtime: RuntimeFuture,
    delete_session: SessionFuture,
    set_final_status: StatusFuture,
) -> AppResult<()>
where
    RuntimeFuture: std::future::Future<Output = ()>,
    SessionFuture: std::future::Future<Output = AppResult<()>>,
    StatusFuture: std::future::Future<Output = ()>,
{
    clear_runtime.await;
    delete_session.await?;
    set_final_status.await;
    Ok(())
}

async fn init_account_client(
    handle: &AppHandle,
    state: &TelegramState,
    secret_store: &SecretStoreState,
    account_id: i64,
    api_id: i32,
    api_hash: TelegramApiHash,
) -> AppResult<bool> {
    set_account_status(handle, state, account_id, STATUS_RESTORING, None).await;

    let session = telegram_session_store::load_session(handle, secret_store, account_id).await?;
    let runtime_status = state
        .runtime
        .initialize_account(account_id, api_id, api_hash, session)
        .await?;
    set_account_status(
        handle,
        state,
        account_id,
        runtime_status_to_wire(runtime_status),
        None,
    )
    .await;

    Ok(runtime_status == TelegramRuntimeStatus::Ready)
}

fn restore_failure_message(error: impl std::fmt::Display) -> String {
    let error = error.to_string();
    if error.trim().is_empty() {
        "Unknown restore error".to_string()
    } else {
        error
    }
}

enum AccountRestoreFailure {
    Credentials(AppError),
    Initialization(AppError),
}

impl AccountRestoreFailure {
    fn into_message(self) -> String {
        match self {
            Self::Credentials(error) | Self::Initialization(error) => {
                restore_failure_message(error)
            }
        }
    }
}

async fn record_restore_failure_with(
    state: &TelegramState,
    account_id: i64,
    failure: AccountRestoreFailure,
    emit_status: impl FnOnce(&AccountRuntimeStatus),
    emit_failure: impl FnOnce(&RestoreFailureEvent),
) {
    let message = failure.into_message();
    set_account_status_with(
        state,
        account_id,
        STATUS_RESTORE_FAILED,
        Some(message.clone()),
        emit_status,
    )
    .await;
    emit_failure(&RestoreFailureEvent { message });
}

async fn record_restore_failure(
    handle: &AppHandle,
    state: &TelegramState,
    account_id: i64,
    failure: AccountRestoreFailure,
) {
    record_restore_failure_with(
        state,
        account_id,
        failure,
        |status| {
            let _ = handle.emit(TELEGRAM_ACCOUNT_STATUS_EVENT, status);
        },
        |event| {
            let _ = handle.emit(TELEGRAM_RESTORE_FAILURE_EVENT, event);
        },
    )
    .await;
}

async fn restore_account_with<
    ResolveCredentials,
    ResolveFuture,
    Initialize,
    InitializeFuture,
    ClearRuntime,
    ClearFuture,
    RecordFailure,
    FailureFuture,
>(
    credentials: AccountCredentialsRow,
    resolve_credentials: ResolveCredentials,
    initialize: Initialize,
    clear_runtime: ClearRuntime,
    record_failure: RecordFailure,
) where
    ResolveCredentials: FnOnce(AccountCredentialsRow) -> ResolveFuture,
    ResolveFuture: std::future::Future<Output = AppResult<AccountCredentials>>,
    Initialize: FnOnce(i64, i32, TelegramApiHash) -> InitializeFuture,
    InitializeFuture: std::future::Future<Output = AppResult<bool>>,
    ClearRuntime: FnOnce(i64) -> ClearFuture,
    ClearFuture: std::future::Future<Output = ()>,
    RecordFailure: FnOnce(i64, AccountRestoreFailure) -> FailureFuture,
    FailureFuture: std::future::Future<Output = ()>,
{
    let account_id = credentials.id;
    let account = match resolve_credentials(credentials).await {
        Ok(account) => account,
        Err(error) => {
            record_failure(account_id, AccountRestoreFailure::Credentials(error)).await;
            return;
        }
    };
    let api_id = match telegram_api_id(account.api_id) {
        Ok(api_id) => api_id,
        Err(error) => {
            record_failure(account.id, AccountRestoreFailure::Credentials(error)).await;
            return;
        }
    };

    if let Err(error) = initialize(account.id, api_id, account.api_hash).await {
        clear_runtime(account.id).await;
        record_failure(account.id, AccountRestoreFailure::Initialization(error)).await;
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
        if !telegram_session_store::session_exists(&handle, account.id) {
            set_account_status(&handle, &state, account.id, STATUS_NOT_INITIALIZED, None).await;
            continue;
        }

        restore_account_with(
            account,
            |credentials| resolve_account_credentials(&pool, &secret_store, credentials),
            |account_id, api_id, api_hash| {
                init_account_client(&handle, &state, &secret_store, account_id, api_id, api_hash)
            },
            |account_id| state.runtime.clear_account(account_id, false),
            |account_id, failure| record_restore_failure(&handle, &state, account_id, failure),
        )
        .await;
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

    match init_account_client(
        &handle,
        &state,
        &secret_store,
        account_id,
        api_id,
        credentials.api_hash,
    )
    .await
    {
        Ok(is_auth) => Ok(is_auth),
        Err(error) => {
            state.runtime.clear_account(account_id, false).await;
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
    state.runtime.is_authenticated(account_id).await
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
    send_code_with(state.runtime.request_login_code(account_id, phone)).await
}

async fn send_code_with(
    request_login_code: impl std::future::Future<Output = AppResult<()>>,
) -> AppResult<String> {
    request_login_code.await?;
    Ok("Code sent".to_string())
}

#[tauri::command]
pub async fn tg_sign_in(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    secret_store: tauri::State<'_, SecretStoreState>,
    account_id: i64,
    code: String,
) -> AppResult<bool> {
    let session_handle = handle.clone();
    let session_secret_store = secret_store.inner();
    sign_in_with(
        state.runtime.sign_in(account_id, code),
        move |session_to_save| async move {
            telegram_session_store::save_session(
                &session_handle,
                session_secret_store,
                account_id,
                &session_to_save,
            )
            .await
        },
        set_account_status(&handle, &state, account_id, STATUS_READY, None),
    )
    .await
}

async fn sign_in_with<SaveSession, SaveFuture, ReadyFuture>(
    runtime_sign_in: impl std::future::Future<Output = AppResult<crate::telegram_impl::TelegramSession>>,
    save_session: SaveSession,
    set_ready: ReadyFuture,
) -> AppResult<bool>
where
    SaveSession: FnOnce(crate::telegram_impl::TelegramSession) -> SaveFuture,
    SaveFuture: std::future::Future<Output = AppResult<()>>,
    ReadyFuture: std::future::Future<Output = ()>,
{
    let session_to_save = runtime_sign_in.await?;
    save_session(session_to_save).await?;
    set_ready.await;
    Ok(true)
}

#[tauri::command]
pub async fn tg_logout(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    secret_store: tauri::State<'_, SecretStoreState>,
    account_id: i64,
) -> AppResult<bool> {
    logout_with(clear_account_runtime(
        &handle,
        &state,
        &secret_store,
        account_id,
        true,
    ))
    .await
}

async fn logout_with(
    clear_account: impl std::future::Future<Output = AppResult<()>>,
) -> AppResult<bool> {
    clear_account.await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{
        clear_account_runtime_with, get_account_credentials_from_pool, logout_with,
        record_restore_failure_with, restore_account_with, restore_failure_message,
        runtime_status_to_wire, send_code_with, set_account_status_with, sign_in_with,
        telegram_api_id, AccountCredentials, AccountCredentialsRow, AccountRestoreFailure,
        AccountRuntimeStatus, RestoreFailureEvent, TelegramRuntimeStatus, TelegramState,
        STATUS_NOT_INITIALIZED, STATUS_READY, STATUS_REAUTH_REQUIRED, STATUS_RESTORE_FAILED,
        STATUS_RESTORING, TELEGRAM_ACCOUNT_STATUS_EVENT, TELEGRAM_RESTORE_FAILURE_EVENT,
    };
    use crate::error::{AppError, AppErrorKind, AppResult};
    use crate::secret_store::tests::InMemorySecretStore;
    use crate::secret_store::{telegram_account_api_hash_secret, SecretStoreState};
    use secrecy::{ExposeSecret, SecretString};
    use std::sync::{Arc, Mutex as StdMutex};

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

    #[tokio::test]
    async fn restore_emits_failure_event_for_each_failed_account() {
        let state = Arc::new(TelegramState::new());
        let order = Arc::new(StdMutex::new(Vec::new()));

        let resolve_order = Arc::clone(&order);
        let credential_status_order = Arc::clone(&order);
        let credential_failure_order = Arc::clone(&order);
        let credential_state = Arc::clone(&state);
        restore_account_with(
            AccountCredentialsRow {
                id: 7,
                api_id: 12345,
                api_hash: String::new(),
            },
            move |account| async move {
                resolve_order
                    .lock()
                    .expect("record credential resolution")
                    .push(format!("resolve:{}", account.id));
                Err(AppError::auth("missing credentials"))
            },
            |_, _, _| async { panic!("initialization must not run after credential failure") },
            |_| async { panic!("runtime cleanup must not run after credential failure") },
            |account_id, failure| async move {
                assert!(matches!(&failure, AccountRestoreFailure::Credentials(_)));
                record_restore_failure_with(
                    &credential_state,
                    account_id,
                    failure,
                    move |status| {
                        credential_status_order
                            .lock()
                            .expect("record credential status event")
                            .push(format!("status:{}:{}", status.account_id, status.status));
                    },
                    |event| {
                        let stored = credential_state
                            .statuses
                            .try_lock()
                            .expect("status lock released before credential failure event")
                            .get(&account_id)
                            .cloned()
                            .expect("credential failed status stored before failure event");
                        assert_eq!(stored.status, STATUS_RESTORE_FAILED);
                        assert_eq!(stored.message.as_deref(), Some(event.message.as_str()));
                        credential_failure_order
                            .lock()
                            .expect("record credential failure event")
                            .push(format!("failure:{account_id}"));
                    },
                )
                .await;
            },
        )
        .await;

        let resolve_order = Arc::clone(&order);
        let init_order = Arc::clone(&order);
        let clear_order = Arc::clone(&order);
        let init_status_order = Arc::clone(&order);
        let init_failure_order = Arc::clone(&order);
        let init_state = Arc::clone(&state);
        restore_account_with(
            AccountCredentialsRow {
                id: 8,
                api_id: 12345,
                api_hash: "api-hash".to_string(),
            },
            move |account| async move {
                resolve_order
                    .lock()
                    .expect("record credential resolution")
                    .push(format!("resolve:{}", account.id));
                Ok(AccountCredentials {
                    id: account.id,
                    api_id: account.api_id,
                    api_hash: crate::telegram_impl::TelegramApiHash::new(SecretString::new(
                        account.api_hash,
                    )),
                })
            },
            move |account_id, _, _| async move {
                init_order
                    .lock()
                    .expect("record initialization")
                    .push(format!("initialize:{account_id}"));
                Err(AppError::telegram_network("runtime unavailable"))
            },
            move |account_id| async move {
                clear_order
                    .lock()
                    .expect("record runtime cleanup")
                    .push(format!("clear:{account_id}"));
            },
            |account_id, failure| async move {
                assert!(matches!(&failure, AccountRestoreFailure::Initialization(_)));
                record_restore_failure_with(
                    &init_state,
                    account_id,
                    failure,
                    move |status| {
                        init_status_order
                            .lock()
                            .expect("record initialization status event")
                            .push(format!("status:{}:{}", status.account_id, status.status));
                    },
                    |event| {
                        let stored = init_state
                            .statuses
                            .try_lock()
                            .expect("status lock released before initialization failure event")
                            .get(&account_id)
                            .cloned()
                            .expect("initialization failed status stored before failure event");
                        assert_eq!(stored.status, STATUS_RESTORE_FAILED);
                        assert_eq!(stored.message.as_deref(), Some(event.message.as_str()));
                        init_failure_order
                            .lock()
                            .expect("record initialization failure event")
                            .push(format!("failure:{account_id}"));
                    },
                )
                .await;
            },
        )
        .await;

        let statuses = state.statuses.lock().await;
        for (account_id, message) in [
            (7, "missing credentials"),
            (8, "Telegram request failed: runtime unavailable"),
        ] {
            let status = statuses
                .get(&account_id)
                .unwrap_or_else(|| panic!("missing failed status for account {account_id}"));
            assert_eq!(status.status, STATUS_RESTORE_FAILED);
            assert_eq!(status.message.as_deref(), Some(message));
        }
        drop(statuses);

        assert_eq!(
            *order.lock().expect("read restore order"),
            [
                "resolve:7",
                "status:7:restore_failed",
                "failure:7",
                "resolve:8",
                "initialize:8",
                "clear:8",
                "status:8:restore_failed",
                "failure:8",
            ]
        );
    }

    #[tokio::test]
    async fn send_code_success_is_not_auth_error() {
        let calls = Arc::new(StdMutex::new(Vec::new()));
        let runtime_calls = Arc::clone(&calls);

        let result = send_code_with(async move {
            runtime_calls
                .lock()
                .expect("record scripted runtime")
                .push("runtime");
            Ok(())
        })
        .await;

        assert_eq!(result.expect("successful command result"), "Code sent");
        assert_eq!(*calls.lock().expect("read scripted runtime"), ["runtime"]);
    }

    #[tokio::test]
    async fn sign_in_persists_session_before_ready_event() {
        let state = TelegramState::new();
        let order = Arc::new(StdMutex::new(Vec::new()));
        let runtime_order = Arc::clone(&order);
        let session_order = Arc::clone(&order);
        let ready_order = Arc::clone(&order);

        let result = sign_in_with(
            async move {
                runtime_order
                    .lock()
                    .expect("record runtime sign-in")
                    .push("runtime");
                Ok(crate::telegram_impl::TelegramSession::empty())
            },
            move |_session| async move {
                session_order
                    .lock()
                    .expect("record session persistence")
                    .push("session");
                Ok(())
            },
            set_account_status_with(&state, 7, STATUS_READY, None, |status| {
                let stored = state
                    .statuses
                    .try_lock()
                    .expect("status lock released before ready event")
                    .get(&7)
                    .cloned()
                    .expect("ready state stored before event");
                assert_eq!(stored.status, STATUS_READY);
                assert_eq!(stored.status, status.status);
                ready_order
                    .lock()
                    .expect("record ready event")
                    .push("ready");
            }),
        )
        .await;
        order.lock().expect("record command result").push("result");

        assert!(result.expect("successful sign-in command result"));
        assert_eq!(
            *order.lock().expect("read sign-in order"),
            ["runtime", "session", "ready", "result"]
        );
    }

    #[tokio::test]
    async fn logout_returns_true_after_runtime_and_session_cleanup() {
        let state = TelegramState::new();
        let order = Arc::new(StdMutex::new(Vec::new()));
        let runtime_order = Arc::clone(&order);
        let session_order = Arc::clone(&order);
        let status_order = Arc::clone(&order);

        let cleanup = clear_account_runtime_with(
            async move {
                runtime_order
                    .lock()
                    .expect("record runtime logout")
                    .push("runtime");
            },
            async move {
                session_order
                    .lock()
                    .expect("record session deletion")
                    .push("session");
                Ok(())
            },
            set_account_status_with(&state, 7, STATUS_NOT_INITIALIZED, None, |status| {
                let stored = state
                    .statuses
                    .try_lock()
                    .expect("status lock released before final event")
                    .get(&7)
                    .cloned()
                    .expect("final status stored before event");
                assert_eq!(stored.status, STATUS_NOT_INITIALIZED);
                assert_eq!(stored.status, status.status);
                status_order
                    .lock()
                    .expect("record final event")
                    .push("status");
            }),
        );
        let result = logout_with(cleanup).await;
        order.lock().expect("record logout result").push("result");

        assert!(result.expect("successful logout command result"));
        assert_eq!(
            *order.lock().expect("read logout order"),
            ["runtime", "session", "status", "result"]
        );
    }

    #[test]
    fn telegram_api_id_out_of_range_returns_typed_validation_error() {
        let error =
            telegram_api_id(i64::from(i32::MAX) + 1).expect_err("reject out-of-range api id");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Telegram API ID is out of range");
    }

    #[test]
    fn runtime_status_maps_to_existing_wire_strings() {
        assert_eq!(
            runtime_status_to_wire(TelegramRuntimeStatus::Ready),
            STATUS_READY,
            "RED: CP5 app runtime status mapping"
        );
        assert_eq!(
            runtime_status_to_wire(TelegramRuntimeStatus::ReauthRequired),
            STATUS_REAUTH_REQUIRED,
            "RED: CP5 app runtime status mapping"
        );
    }

    #[test]
    fn telegram_status_and_event_payload_contract_is_exact() {
        assert_eq!(
            TELEGRAM_RESTORE_FAILURE_EVENT, "telegram://restore-failure",
            "RED: CP2 Telegram status and event payload"
        );
        assert_eq!(TELEGRAM_ACCOUNT_STATUS_EVENT, "telegram://account-status");
        assert_eq!(
            [
                STATUS_NOT_INITIALIZED,
                STATUS_RESTORING,
                STATUS_READY,
                STATUS_REAUTH_REQUIRED,
                STATUS_RESTORE_FAILED,
            ],
            [
                "not_initialized",
                "restoring",
                "ready",
                "reauth_required",
                "restore_failed",
            ]
        );

        assert_eq!(
            serde_json::to_string(&RestoreFailureEvent {
                message: "restore failed".to_string(),
            })
            .expect("serialize restore failure"),
            r#"{"message":"restore failed"}"#
        );
        assert_eq!(
            serde_json::to_string(&AccountRuntimeStatus {
                account_id: 7,
                status: STATUS_READY.to_string(),
                message: None,
            })
            .expect("serialize account status without message"),
            r#"{"account_id":7,"status":"ready","message":null}"#
        );
        assert_eq!(
            serde_json::to_string(&AccountRuntimeStatus {
                account_id: 7,
                status: STATUS_RESTORE_FAILED.to_string(),
                message: Some("restore failed".to_string()),
            })
            .expect("serialize account status with message"),
            r#"{"account_id":7,"status":"restore_failed","message":"restore failed"}"#
        );
        assert_eq!(restore_failure_message(""), "Unknown restore error");
        assert_eq!(
            restore_failure_message("transport disconnected"),
            "transport disconnected"
        );

        let auth_errors = [
            AppError::auth("Account not initialized"),
            AppError::auth("Call tg_send_code first"),
            AppError::auth("Account 7 not initialized"),
            AppError::auth("Account 7 is not authenticated"),
            AppError::auth(
                "Telegram API hash for account 7 is missing from secure storage. Recreate the account credentials.",
            ),
        ];
        let expected_auth_json = [
            r#"{"kind":"auth","message":"Account not initialized"}"#,
            r#"{"kind":"auth","message":"Call tg_send_code first"}"#,
            r#"{"kind":"auth","message":"Account 7 not initialized"}"#,
            r#"{"kind":"auth","message":"Account 7 is not authenticated"}"#,
            r#"{"kind":"auth","message":"Telegram API hash for account 7 is missing from secure storage. Recreate the account credentials."}"#,
        ];
        for (error, expected_json) in auth_errors.into_iter().zip(expected_auth_json) {
            assert_eq!(error.kind, AppErrorKind::Auth);
            assert_eq!(
                serde_json::to_string(&error).expect("serialize auth error"),
                expected_json
            );
        }

        let network_error = AppError::telegram_network("transport disconnected");
        assert_eq!(network_error.kind, AppErrorKind::Network);
        assert_eq!(
            network_error.message,
            "Telegram request failed: transport disconnected"
        );
        assert_eq!(
            serde_json::to_string(&network_error).expect("serialize network error"),
            r#"{"kind":"network","message":"Telegram request failed: transport disconnected"}"#
        );

        let code_sent_result: AppResult<String> = Ok("Code sent".to_string());
        assert_eq!(
            code_sent_result.expect("successful send-code result"),
            "Code sent"
        );
        let sign_in_result: AppResult<bool> = Ok(true);
        let logout_result: AppResult<bool> = Ok(true);
        assert!(sign_in_result.expect("successful sign-in result"));
        assert!(logout_result.expect("successful logout result"));
    }

    #[tokio::test]
    async fn legacy_api_hash_migrates_to_secret_store_and_blanks_column() {
        let pool = memory_pool().await;
        let (_store, secret_store) = memory_secret_store();
        let account_id = insert_account(&pool, "legacy-hash").await;

        get_account_credentials_from_pool(&pool, &secret_store, account_id)
            .await
            .expect("load credentials");

        assert_eq!(stored_api_hash(&pool, account_id).await, "");
        assert_eq!(
            secret_store
                .get_secret(telegram_account_api_hash_secret(account_id))
                .await
                .expect("read secret")
                .map(|value| value.expose_secret().to_string()),
            Some("legacy-hash".to_string())
        );
    }

    #[tokio::test]
    async fn legacy_api_hash_remains_when_secret_write_fails() {
        let pool = memory_pool().await;
        let (store, secret_store) = memory_secret_store();
        store.fail_set("secure store unavailable");
        let account_id = insert_account(&pool, "legacy-hash").await;

        let error = match get_account_credentials_from_pool(&pool, &secret_store, account_id).await
        {
            Ok(_) => panic!("secret write should fail"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::Internal);
        assert_eq!(error.message, "secure store unavailable");
        assert_eq!(stored_api_hash(&pool, account_id).await, "legacy-hash");
    }

    #[tokio::test]
    async fn missing_secure_api_hash_for_blank_legacy_account_is_auth_error() {
        let pool = memory_pool().await;
        let (_store, secret_store) = memory_secret_store();
        let account_id = insert_account(&pool, "").await;

        let error = match get_account_credentials_from_pool(&pool, &secret_store, account_id).await
        {
            Ok(_) => panic!("missing secret should fail"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::Auth);
        assert_eq!(
            error.message,
            format!(
                "Telegram API hash for account {account_id} is missing from secure storage. Recreate the account credentials."
            )
        );

        secret_store
            .set_secret(telegram_account_api_hash_secret(account_id), " \t\r\n ")
            .await
            .expect("store whitespace-only API hash");
        let whitespace_error =
            match get_account_credentials_from_pool(&pool, &secret_store, account_id).await {
                Ok(_) => panic!("RED: CP5 whitespace-only secure API hash must fail"),
                Err(error) => error,
            };
        assert_eq!(
            whitespace_error.kind,
            AppErrorKind::Auth,
            "RED: CP5 whitespace-only secure API hash must fail"
        );
        assert_eq!(
            whitespace_error.message,
            format!(
                "Telegram API hash for account {account_id} is missing from secure storage. Recreate the account credentials."
            ),
            "RED: CP5 whitespace-only secure API hash must fail"
        );
    }

    #[tokio::test]
    async fn diagnostic_status_counts_do_not_return_account_ids_or_messages() {
        let state = TelegramState::new();
        set_account_status_for_test(&state, 10, "ready", Some("private phone +10000000000")).await;
        set_account_status_for_test(
            &state,
            11,
            "restore_failed",
            Some("C:\\Users\\Dima\\session"),
        )
        .await;

        let counts = state.diagnostic_status_counts(&[10, 11, 12]).await;

        assert_eq!(counts.len(), 3);
        assert!(counts.contains(&("not_initialized".to_string(), 1)));
        assert!(counts.contains(&("ready".to_string(), 1)));
        assert!(counts.contains(&("restore_failed".to_string(), 1)));
        let value = serde_json::to_value(&counts).expect("serialize counts value");
        let entries = value.as_array().expect("counts serialize as array");
        for entry in entries {
            let pair = entry.as_array().expect("status count entry is a tuple");
            assert_eq!(pair.len(), 2);
            assert!(pair[0].as_str().is_some());
            assert!(pair[1].as_i64().is_some());
        }
        let json = serde_json::to_string(&counts).expect("serialize counts");
        assert!(!json.contains("account_id"));
        assert!(!json.contains("\"10\""));
        assert!(!json.contains("\"11\""));
        assert!(!json.contains("\"12\""));
        assert!(!json.contains("+10000000000"));
        assert!(!json.contains("C:\\Users\\Dima"));
    }

    async fn set_account_status_for_test(
        state: &TelegramState,
        account_id: i64,
        status: &str,
        message: Option<&str>,
    ) {
        state.statuses.lock().await.insert(
            account_id,
            AccountRuntimeStatus {
                account_id,
                status: status.to_string(),
                message: message.map(ToString::to_string),
            },
        );
    }
}
