use std::sync::Arc;

use grammers_client::Client;
use grammers_mtsender::SenderPool;
use grammers_session::storages::MemorySession;
use tokio::sync::Mutex;
use tauri::AppHandle;

pub struct TelegramState {
    pub client: Mutex<Option<Client>>,
    pub api_id: Mutex<Option<i32>>,
    pub api_hash: Mutex<Option<String>>,
}

impl TelegramState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            api_id: Mutex::new(None),
            api_hash: Mutex::new(None),
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

    // New API: create session, build SenderPool, spawn runner, get Client
    let session = Arc::new(MemorySession::default());
    let pool = SenderPool::new(Arc::clone(&session), api_id);

    // Spawn the pool runner as a background task
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
