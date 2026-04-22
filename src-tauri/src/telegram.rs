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

#[derive(sqlx::FromRow)]
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

fn session_path(handle: &AppHandle, account_id: i64) -> Result<PathBuf, String> {
    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
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
) -> Result<(), String> {
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
    let json = serde_json::to_string(&saved).map_err(|e| e.to_string())?;
    let path = session_path(handle, account_id)?;
    fs::write(path, json).map_err(|e| e.to_string())
}

async fn list_account_credentials(handle: &AppHandle) -> Result<Vec<AccountCredentials>, String> {
    let pool = get_pool(handle).await?;
    sqlx::query_as("SELECT id, api_id, api_hash FROM accounts ORDER BY created_at ASC")
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
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
) -> Result<bool, String> {
    set_account_status(handle, state, account_id, STATUS_RESTORING, None).await;

    let session = load_session(handle, account_id).await;
    let pool = SenderPool::new(Arc::clone(&session), api_id);

    tokio::spawn(async move {
        let _ = pool.runner.run().await;
    });

    let client = Client::new(pool.handle);
    let is_auth = client.is_authorized().await.map_err(|e| e.to_string())?;

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

fn restore_failure_message(error: String) -> String {
    if error.trim().is_empty() {
        "Unknown restore error".to_string()
    } else {
        error
    }
}

pub async fn restore_telegram_accounts(handle: AppHandle) {
    let state = handle.state::<TelegramState>();
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

        let init_result = init_account_client(
            &handle,
            &state,
            account.id,
            account.api_id as i32,
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
    account_id: i64,
    api_id: i32,
    api_hash: String,
) -> Result<bool, String> {
    match init_account_client(&handle, &state, account_id, api_id, api_hash).await {
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
                Some(restore_failure_message(error.clone())),
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
) -> Result<bool, String> {
    let client = {
        let accounts = state.accounts.lock().await;
        accounts.get(&account_id).map(|ac| ac.client.clone())
    };

    if let Some(client) = client {
        client.is_authorized().await.map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn tg_get_account_statuses(
    state: tauri::State<'_, TelegramState>,
    account_ids: Vec<i64>,
) -> Result<Vec<AccountRuntimeStatus>, String> {
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
) -> Result<String, String> {
    let mut accounts = state.accounts.lock().await;
    let ac = accounts
        .get_mut(&account_id)
        .ok_or("Account not initialized")?;

    let token = ac
        .client
        .request_login_code(&phone, &ac.api_hash.clone())
        .await
        .map_err(|e| e.to_string())?;

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
) -> Result<bool, String> {
    let session_to_save = {
        let mut accounts = state.accounts.lock().await;
        let ac = accounts
            .get_mut(&account_id)
            .ok_or("Account not initialized")?;
        let token = ac.login_token.as_ref().ok_or("Call tg_send_code first")?;

        ac.client
            .sign_in(token, &code)
            .await
            .map_err(|e| e.to_string())?;
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
) -> Result<bool, String> {
    clear_account_runtime(&handle, &state, account_id, true).await;
    Ok(true)
}

/// Returns the Client for a given account_id (for use in other modules).
/// Caller must hold the lock.
pub async fn get_client<'a>(
    accounts: &'a HashMap<i64, AccountClient>,
    account_id: i64,
) -> Result<&'a Client, String> {
    accounts
        .get(&account_id)
        .map(|ac| &ac.client)
        .ok_or_else(|| format!("Account {account_id} not initialized"))
}
