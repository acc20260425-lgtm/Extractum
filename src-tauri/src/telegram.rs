use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;

use grammers_client::{Client, client::LoginToken};
use grammers_mtsender::SenderPool;
use grammers_session::{Session, SessionData, storages::MemorySession};
use grammers_session::types::{DcOption, UpdatesState};
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};

// Serializable snapshot of the session — mirrors SessionData fields
// but with serde derives (SessionData itself doesn't derive serde).
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedSession {
    home_dc: i32,
    dc_options: HashMap<i32, DcOption>,
    updates_state: UpdatesState,
}

pub struct TelegramState {
    pub client: Mutex<Option<Client>>,
    pub session: Mutex<Option<Arc<MemorySession>>>,
    pub api_id: Mutex<Option<i32>>,
    pub api_hash: Mutex<Option<String>>,
    pub phone: Mutex<Option<String>>,
    pub login_token: Mutex<Option<LoginToken>>,
}

impl TelegramState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            session: Mutex::new(None),
            api_id: Mutex::new(None),
            api_hash: Mutex::new(None),
            phone: Mutex::new(None),
            login_token: Mutex::new(None),
        }
    }
}

fn get_session_path(handle: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }
    Ok(app_dir.join("telegram.session.json"))
}

async fn save_session(session: &Arc<MemorySession>, path: &PathBuf) -> Result<(), String> {
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
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tg_init(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    api_id: i32,
    api_hash: String,
) -> Result<bool, String> {
    let mut client_lock = state.client.lock().await;
    let mut session_lock = state.session.lock().await;
    let mut id_lock = state.api_id.lock().await;
    let mut hash_lock = state.api_hash.lock().await;

    *id_lock = Some(api_id);
    *hash_lock = Some(api_hash.clone());

    let session_path = get_session_path(&handle)?;
    let session = if session_path.exists() {
        let json = fs::read_to_string(&session_path).map_err(|e| e.to_string())?;
        let saved: SavedSession = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let session_data = SessionData {
            home_dc: saved.home_dc,
            dc_options: saved.dc_options,
            peer_infos: HashMap::new(),
            updates_state: saved.updates_state,
        };
        Arc::new(MemorySession::from(session_data))
    } else {
        Arc::new(MemorySession::default())
    };

    let pool = SenderPool::new(Arc::clone(&session), api_id);

    tokio::spawn(async move {
        let _ = pool.runner.run().await;
    });

    let client = Client::new(pool.handle);
    let is_auth = client.is_authorized().await.map_err(|e| e.to_string())?;

    *client_lock = Some(client);
    *session_lock = Some(session);

    Ok(is_auth)
}

#[tauri::command]
pub async fn tg_is_authenticated(state: tauri::State<'_, TelegramState>) -> Result<bool, String> {
    let client_lock = state.client.lock().await;
    if let Some(client) = &*client_lock {
        client.is_authorized().await.map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn tg_send_code(
    state: tauri::State<'_, TelegramState>,
    phone: String,
) -> Result<String, String> {
    let client_lock = state.client.lock().await;
    let api_hash_lock = state.api_hash.lock().await;

    let client = client_lock.as_ref().ok_or("Telegram client not initialized")?;
    let api_hash = api_hash_lock.as_ref().ok_or("API Hash not set")?;

    let login_token = client
        .request_login_code(&phone, api_hash)
        .await
        .map_err(|e| e.to_string())?;

    let mut phone_lock = state.phone.lock().await;
    *phone_lock = Some(phone);

    let mut token_lock = state.login_token.lock().await;
    *token_lock = Some(login_token);

    Ok("Code sent".to_string())
}

#[tauri::command]
pub async fn tg_sign_in(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    code: String,
) -> Result<bool, String> {
    let client_lock = state.client.lock().await;
    let token_lock = state.login_token.lock().await;

    let client = client_lock.as_ref().ok_or("Telegram client not initialized")?;
    let token = token_lock.as_ref().ok_or("Login token not found. Call tg_send_code first.")?;

    client
        .sign_in(token, &code)
        .await
        .map_err(|e| e.to_string())?;

    drop(client_lock);
    drop(token_lock);

    // Save session after successful sign-in
    let session_lock = state.session.lock().await;
    if let Some(session) = session_lock.as_ref() {
        let path = get_session_path(&handle)?;
        save_session(session, &path).await?;
    }

    Ok(true)
}

#[tauri::command]
pub async fn tg_logout(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
) -> Result<bool, String> {
    let mut client_lock = state.client.lock().await;
    if let Some(client) = client_lock.take() {
        let _ = client.sign_out().await;
    }

    let mut session_lock = state.session.lock().await;
    *session_lock = None;

    let mut token_lock = state.login_token.lock().await;
    *token_lock = None;

    let mut phone_lock = state.phone.lock().await;
    *phone_lock = None;

    let path = get_session_path(&handle)?;
    if path.exists() {
        let _ = fs::remove_file(path);
    }

    Ok(true)
}
