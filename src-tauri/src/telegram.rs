use std::sync::Arc;

use grammers_client::{Client, client::LoginToken};
use grammers_mtsender::SenderPool;
use grammers_session::storages::MemorySession;
use tokio::sync::Mutex;
use tauri::AppHandle;

pub struct TelegramState {
    pub client: Mutex<Option<Client>>,
    pub api_id: Mutex<Option<i32>>,
    pub api_hash: Mutex<Option<String>>,
    pub phone: Mutex<Option<String>>,
    pub login_token: Mutex<Option<LoginToken>>,
}

impl TelegramState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            api_id: Mutex::new(None),
            api_hash: Mutex::new(None),
            phone: Mutex::new(None),
            login_token: Mutex::new(None),
        }
    }
}

#[tauri::command]
pub async fn tg_init(
    _handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    api_id: i32,
    api_hash: String,
) -> Result<bool, String> {
    let mut client_lock = state.client.lock().await;
    let mut id_lock = state.api_id.lock().await;
    let mut hash_lock = state.api_hash.lock().await;

    *id_lock = Some(api_id);
    *hash_lock = Some(api_hash.clone());

    let session = Arc::new(MemorySession::default());
    let pool = SenderPool::new(Arc::clone(&session), api_id);

    tokio::spawn(async move {
        let _ = pool.runner.run().await;
    });

    let client = Client::new(pool.handle);
    let is_auth = client.is_authorized().await.map_err(|e| e.to_string())?;
    
    *client_lock = Some(client);

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
    let api_id_lock = state.api_id.lock().await;
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

    Ok(true)
}

#[tauri::command]
pub async fn tg_logout(state: tauri::State<'_, TelegramState>) -> Result<bool, String> {
    let mut client_lock = state.client.lock().await;
    if let Some(client) = client_lock.take() {
        let _ = client.sign_out().await;
    }
    
    let mut token_lock = state.login_token.lock().await;
    *token_lock = None;
    
    let mut phone_lock = state.phone.lock().await;
    *phone_lock = None;

    Ok(true)
}
