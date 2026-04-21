use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use grammers_client::{Client, client::LoginToken};
use grammers_mtsender::SenderPool;
use grammers_session::{Session, SessionData, storages::MemorySession};
use grammers_session::types::{DcOption, UpdatesState};
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedSession {
    home_dc: i32,
    dc_options: HashMap<i32, DcOption>,
    updates_state: UpdatesState,
}

pub struct AccountClient {
    pub client: Client,
    pub session: Arc<MemorySession>,
    pub api_hash: String,
    pub login_token: Option<LoginToken>,
    pub phone: Option<String>,
}

/// Global state: map of account_id → active client
pub struct TelegramState {
    pub accounts: Mutex<HashMap<i64, AccountClient>>,
}

impl TelegramState {
    pub fn new() -> Self {
        Self {
            accounts: Mutex::new(HashMap::new()),
        }
    }
}

fn session_path(handle: &AppHandle, account_id: i64) -> Result<PathBuf, String> {
    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    Ok(app_dir.join(format!("telegram_{account_id}.session.json")))
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

async fn save_session(handle: &AppHandle, account_id: i64, session: &Arc<MemorySession>) -> Result<(), String> {
    let home_dc = session.home_dc_id();
    let updates_state = session.updates_state().await;
    let mut dc_options = HashMap::new();
    for dc_id in 1..=5i32 {
        if let Some(dc) = session.dc_option(dc_id) {
            dc_options.insert(dc_id, dc);
        }
    }
    let saved = SavedSession { home_dc, dc_options, updates_state };
    let json = serde_json::to_string(&saved).map_err(|e| e.to_string())?;
    let path = session_path(handle, account_id)?;
    fs::write(path, json).map_err(|e| e.to_string())
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
    let session = load_session(&handle, account_id).await;
    let pool = SenderPool::new(Arc::clone(&session), api_id);

    tokio::spawn(async move {
        let _ = pool.runner.run().await;
    });

    let client = Client::new(pool.handle);
    let is_auth = client.is_authorized().await.map_err(|e| e.to_string())?;

    let mut accounts = state.accounts.lock().await;
    accounts.insert(account_id, AccountClient {
        client,
        session,
        api_hash,
        login_token: None,
        phone: None,
    });

    Ok(is_auth)
}

#[tauri::command]
pub async fn tg_is_authenticated(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> Result<bool, String> {
    let accounts = state.accounts.lock().await;
    if let Some(ac) = accounts.get(&account_id) {
        ac.client.is_authorized().await.map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn tg_send_code(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
    phone: String,
) -> Result<String, String> {
    let mut accounts = state.accounts.lock().await;
    let ac = accounts.get_mut(&account_id).ok_or("Account not initialized")?;

    let token = ac.client
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
    let mut accounts = state.accounts.lock().await;
    let ac = accounts.get_mut(&account_id).ok_or("Account not initialized")?;
    let token = ac.login_token.as_ref().ok_or("Call tg_send_code first")?;

    ac.client.sign_in(token, &code).await.map_err(|e| e.to_string())?;
    ac.login_token = None;

    save_session(&handle, account_id, &ac.session).await?;

    Ok(true)
}

#[tauri::command]
pub async fn tg_logout(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> Result<bool, String> {
    let mut accounts = state.accounts.lock().await;
    if let Some(ac) = accounts.remove(&account_id) {
        let _ = ac.client.sign_out().await;
    }

    if let Ok(path) = session_path(&handle, account_id) {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    Ok(true)
}

/// Returns the Client for a given account_id (for use in other modules).
/// Caller must hold the lock.
pub async fn get_client<'a>(
    accounts: &'a HashMap<i64, AccountClient>,
    account_id: i64,
) -> Result<&'a Client, String> {
    accounts.get(&account_id)
        .map(|ac| &ac.client)
        .ok_or_else(|| format!("Account {account_id} not initialized"))
}
