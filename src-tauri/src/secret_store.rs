use std::sync::Arc;

use crate::error::{AppError, AppResult};

pub(crate) const SECRET_SERVICE_NAME: &str = "org.ai.extractum";

pub(crate) fn llm_profile_api_key_secret(profile_id: &str) -> String {
    format!("llm.profile.{profile_id}.api_key")
}

pub(crate) fn telegram_account_api_hash_secret(account_id: i64) -> String {
    format!("telegram.account.{account_id}.api_hash")
}

pub(crate) fn telegram_account_session_key_secret(account_id: i64) -> String {
    format!("telegram.account.{account_id}.session_key")
}

pub(crate) fn youtube_default_cookies_secret() -> String {
    "youtube.auth.default.cookies".to_string()
}

pub(crate) trait SecretStore: Send + Sync {
    fn get_secret(&self, key: &str) -> AppResult<Option<String>>;
    fn set_secret(&self, key: &str, value: &str) -> AppResult<()>;
    fn delete_secret(&self, key: &str) -> AppResult<()>;
}

#[derive(Clone)]
pub(crate) struct SecretStoreState {
    store: Arc<dyn SecretStore>,
}

impl SecretStoreState {
    pub(crate) fn new(store: Arc<dyn SecretStore>) -> Self {
        Self { store }
    }

    pub(crate) fn system() -> Self {
        Self::new(Arc::new(SystemSecretStore))
    }

    pub(crate) async fn get_secret(&self, key: impl Into<String>) -> AppResult<Option<String>> {
        let store = Arc::clone(&self.store);
        let key = key.into();
        tauri::async_runtime::spawn_blocking(move || store.get_secret(&key))
            .await
            .map_err(|error| AppError::internal(format!("Secure storage task failed: {error}")))?
    }

    pub(crate) async fn set_secret(
        &self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> AppResult<()> {
        let store = Arc::clone(&self.store);
        let key = key.into();
        let value = value.into();
        tauri::async_runtime::spawn_blocking(move || store.set_secret(&key, &value))
            .await
            .map_err(|error| AppError::internal(format!("Secure storage task failed: {error}")))?
    }

    pub(crate) async fn delete_secret(&self, key: impl Into<String>) -> AppResult<()> {
        let store = Arc::clone(&self.store);
        let key = key.into();
        tauri::async_runtime::spawn_blocking(move || store.delete_secret(&key))
            .await
            .map_err(|error| AppError::internal(format!("Secure storage task failed: {error}")))?
    }
}

pub(crate) struct SystemSecretStore;

impl SecretStore for SystemSecretStore {
    fn get_secret(&self, key: &str) -> AppResult<Option<String>> {
        let entry = keyring::Entry::new(SECRET_SERVICE_NAME, key).map_err(map_keyring_error)?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(map_keyring_error(error)),
        }
    }

    fn set_secret(&self, key: &str, value: &str) -> AppResult<()> {
        let entry = keyring::Entry::new(SECRET_SERVICE_NAME, key).map_err(map_keyring_error)?;
        entry.set_password(value).map_err(map_keyring_error)
    }

    fn delete_secret(&self, key: &str) -> AppResult<()> {
        let entry = keyring::Entry::new(SECRET_SERVICE_NAME, key).map_err(map_keyring_error)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(map_keyring_error(error)),
        }
    }
}

fn map_keyring_error(error: keyring::Error) -> AppError {
    AppError::internal(format!("Secure storage error: {error}"))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::error::AppErrorKind;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    pub(crate) struct InMemorySecretStore {
        secrets: Mutex<HashMap<String, String>>,
        fail_get: Mutex<Option<String>>,
        fail_set: Mutex<Option<String>>,
        fail_delete: Mutex<Option<String>>,
    }

    impl InMemorySecretStore {
        pub(crate) fn new() -> Self {
            Self::default()
        }

        pub(crate) fn fail_get(&self, message: impl Into<String>) {
            *self.fail_get.lock().unwrap() = Some(message.into());
        }

        pub(crate) fn fail_set(&self, message: impl Into<String>) {
            *self.fail_set.lock().unwrap() = Some(message.into());
        }

        pub(crate) fn fail_delete(&self, message: impl Into<String>) {
            *self.fail_delete.lock().unwrap() = Some(message.into());
        }
    }

    impl SecretStore for InMemorySecretStore {
        fn get_secret(&self, key: &str) -> AppResult<Option<String>> {
            if let Some(message) = self.fail_get.lock().unwrap().clone() {
                return Err(AppError::internal(message));
            }
            Ok(self.secrets.lock().unwrap().get(key).cloned())
        }

        fn set_secret(&self, key: &str, value: &str) -> AppResult<()> {
            if let Some(message) = self.fail_set.lock().unwrap().clone() {
                return Err(AppError::internal(message));
            }
            self.secrets
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn delete_secret(&self, key: &str) -> AppResult<()> {
            if let Some(message) = self.fail_delete.lock().unwrap().clone() {
                return Err(AppError::internal(message));
            }
            self.secrets.lock().unwrap().remove(key);
            Ok(())
        }
    }

    #[test]
    fn secret_ids_are_stable() {
        assert_eq!(
            llm_profile_api_key_secret("default"),
            "llm.profile.default.api_key"
        );
        assert_eq!(
            telegram_account_api_hash_secret(42),
            "telegram.account.42.api_hash"
        );
        assert_eq!(
            telegram_account_session_key_secret(42),
            "telegram.account.42.session_key"
        );
        assert_eq!(
            youtube_default_cookies_secret(),
            "youtube.auth.default.cookies"
        );
    }

    #[tokio::test]
    async fn state_reads_writes_and_deletes_secrets() {
        let store = Arc::new(InMemorySecretStore::new());
        let state = SecretStoreState::new(store);

        assert_eq!(state.get_secret("alpha").await.unwrap(), None);

        state.set_secret("alpha", "secret-value").await.unwrap();
        assert_eq!(
            state.get_secret("alpha").await.unwrap(),
            Some("secret-value".to_string())
        );

        state.delete_secret("alpha").await.unwrap();
        assert_eq!(state.get_secret("alpha").await.unwrap(), None);
    }

    #[tokio::test]
    async fn in_memory_store_can_fail_each_operation() {
        let store = Arc::new(InMemorySecretStore::new());
        store.fail_get("get failed");
        store.fail_set("set failed");
        store.fail_delete("delete failed");
        let state = SecretStoreState::new(store);

        let get_error = state.get_secret("alpha").await.unwrap_err();
        assert_eq!(get_error.kind, AppErrorKind::Internal);
        assert_eq!(get_error.message, "get failed");

        let set_error = state.set_secret("alpha", "value").await.unwrap_err();
        assert_eq!(set_error.message, "set failed");

        let delete_error = state.delete_secret("alpha").await.unwrap_err();
        assert_eq!(delete_error.message, "delete failed");
    }
}
