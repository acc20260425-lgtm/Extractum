use std::collections::HashMap;
#[cfg(test)]
use std::sync::Arc;

use extractum_core::error::AppError;
use grammers_client::client::LoginToken;
use grammers_mtsender::SenderPool;
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::{dto::PeerDescriptor, live, session::TelegramSession, DialogListing};

#[derive(Clone)]
pub struct TelegramApiHash(SecretString);

impl TelegramApiHash {
    pub fn new(value: SecretString) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramRuntimeStatus {
    Ready,
    ReauthRequired,
}

#[derive(Clone)]
enum TelegramClientInner {
    Grammers(grammers_client::Client),
    #[cfg(test)]
    Test {
        account_id: i64,
    },
}

#[derive(Clone)]
pub struct TelegramClientHandle {
    client: TelegramClientInner,
    session: TelegramSession,
}

impl TelegramClientHandle {
    pub fn dialog_listing(&self, avatar_budget_ms: u64) -> DialogListing {
        match &self.client {
            TelegramClientInner::Grammers(client) => live::dialog_listing(client, avatar_budget_ms),
            #[cfg(test)]
            TelegramClientInner::Test { .. } => {
                unreachable!("test Telegram clients cannot list live dialogs")
            }
        }
    }

    pub async fn resolve_dialog_peer(
        &self,
        peer_id: i64,
        expected_subtype: Option<&str>,
    ) -> extractum_core::error::AppResult<PeerDescriptor> {
        match &self.client {
            TelegramClientInner::Grammers(client) => {
                live::resolve_dialog_peer(client, peer_id, expected_subtype).await
            }
            #[cfg(test)]
            TelegramClientInner::Test { .. } => {
                unreachable!("test Telegram clients cannot resolve live dialogs")
            }
        }
    }

    pub async fn resolve_username(
        &self,
        username: &str,
        expected_subtype: Option<&str>,
    ) -> extractum_core::error::AppResult<Option<PeerDescriptor>> {
        match &self.client {
            TelegramClientInner::Grammers(client) => {
                live::resolve_username(client, username, expected_subtype).await
            }
            #[cfg(test)]
            TelegramClientInner::Test { .. } => {
                unreachable!("test Telegram clients cannot resolve live usernames")
            }
        }
    }

    pub async fn peer_avatar_bytes(&self, peer: &PeerDescriptor) -> Option<Vec<u8>> {
        match &self.client {
            TelegramClientInner::Grammers(client) => live::peer_avatar_bytes(client, peer).await,
            #[cfg(test)]
            TelegramClientInner::Test { .. } => {
                unreachable!("test Telegram clients cannot download live avatars")
            }
        }
    }

    pub(crate) fn raw_client(&self) -> &grammers_client::Client {
        match &self.client {
            TelegramClientInner::Grammers(client) => client,
            #[cfg(test)]
            TelegramClientInner::Test { .. } => {
                unreachable!("test Telegram clients have no raw Grammers handle")
            }
        }
    }

    pub(crate) fn raw_session(&self) -> &std::sync::Arc<grammers_session::storages::MemorySession> {
        self.session.raw_memory_session()
    }
}

enum TelegramLoginAttemptToken {
    Grammers(LoginToken),
    #[cfg(test)]
    Test(u64),
}

pub struct TelegramLoginAttempt {
    token: TelegramLoginAttemptToken,
    phone: String,
}

struct TelegramRuntimeAccount {
    handle: TelegramClientHandle,
    api_hash: TelegramApiHash,
    login_attempt: Option<TelegramLoginAttempt>,
    runner: JoinHandle<()>,
}

fn detach_replaced<T>(replaced: Option<T>) {
    drop(replaced);
}

pub struct TelegramRuntime {
    accounts: Mutex<HashMap<i64, TelegramRuntimeAccount>>,
    #[cfg(test)]
    test_callbacks: Option<Arc<TelegramRuntimeTestCallbacks>>,
}

impl TelegramRuntime {
    pub fn new() -> Self {
        Self {
            accounts: Mutex::new(HashMap::new()),
            #[cfg(test)]
            test_callbacks: None,
        }
    }

    pub async fn initialize_account(
        &self,
        account_id: i64,
        api_id: i32,
        api_hash: TelegramApiHash,
        session: TelegramSession,
    ) -> extractum_core::error::AppResult<TelegramRuntimeStatus> {
        let (client, runner, is_authorized);

        #[cfg(test)]
        {
            if let Some(callbacks) = self.test_callbacks.as_ref() {
                runner = callbacks.spawn_pending_runner(account_id);
                client = TelegramClientInner::Test { account_id };
                is_authorized = (callbacks.is_authorized)(account_id).await?;
            } else {
                (client, runner, is_authorized) =
                    initialize_grammers_client(api_id, &session).await?;
            }
        }

        #[cfg(not(test))]
        {
            (client, runner, is_authorized) = initialize_grammers_client(api_id, &session).await?;
        }

        let handle = TelegramClientHandle { client, session };
        // Dropping a Tokio JoinHandle detaches its task. Explicit cancellation is
        // reserved for clear_account, while replacement preserves the old runner.
        detach_replaced(self.accounts.lock().await.insert(
            account_id,
            TelegramRuntimeAccount {
                handle,
                api_hash,
                login_attempt: None,
                runner,
            },
        ));

        Ok(if is_authorized {
            TelegramRuntimeStatus::Ready
        } else {
            TelegramRuntimeStatus::ReauthRequired
        })
    }

    pub async fn is_authenticated(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<bool> {
        let handle = {
            let accounts = self.accounts.lock().await;
            let Some(account) = accounts.get(&account_id) else {
                return Ok(false);
            };
            account.handle.clone()
        };

        self.handle_is_authorized(&handle).await
    }

    pub async fn request_login_code(
        &self,
        account_id: i64,
        phone: String,
    ) -> extractum_core::error::AppResult<()> {
        let mut accounts = self.accounts.lock().await;
        let account = accounts
            .get_mut(&account_id)
            .ok_or_else(|| AppError::auth("Account not initialized"))?;

        let token = match &account.handle.client {
            TelegramClientInner::Grammers(client) => TelegramLoginAttemptToken::Grammers(
                client
                    .request_login_code(&phone, account.api_hash.0.expose_secret())
                    .await
                    .map_err(AppError::telegram_network)?,
            ),
            #[cfg(test)]
            TelegramClientInner::Test { account_id } => TelegramLoginAttemptToken::Test(
                (self
                    .test_callbacks
                    .as_ref()
                    .expect("test Telegram handle requires callbacks")
                    .request_login_code)(*account_id, phone.clone())
                .await?,
            ),
        };

        account.login_attempt = Some(TelegramLoginAttempt { token, phone });
        Ok(())
    }

    pub async fn sign_in(
        &self,
        account_id: i64,
        code: String,
    ) -> extractum_core::error::AppResult<TelegramSession> {
        let mut accounts = self.accounts.lock().await;
        let account = accounts
            .get_mut(&account_id)
            .ok_or_else(|| AppError::auth("Account not initialized"))?;
        let attempt = account
            .login_attempt
            .as_ref()
            .ok_or_else(|| AppError::auth("Call tg_send_code first"))?;

        let result = match (&account.handle.client, &attempt.token) {
            (TelegramClientInner::Grammers(client), TelegramLoginAttemptToken::Grammers(token)) => {
                client
                    .sign_in(token, &code)
                    .await
                    .map(|_| ())
                    .map_err(AppError::telegram_network)
            }
            #[cfg(test)]
            (TelegramClientInner::Test { account_id }, TelegramLoginAttemptToken::Test(marker)) => {
                (self
                    .test_callbacks
                    .as_ref()
                    .expect("test Telegram handle requires callbacks")
                    .sign_in)(
                    *account_id, *marker, attempt.phone.clone(), code.clone()
                )
                .await
            }
            #[cfg(test)]
            _ => unreachable!("Telegram client and login token variants must match"),
        };
        result?;

        account.login_attempt = None;
        Ok(account.handle.session.clone())
    }

    pub async fn authorized_client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle> {
        let handle = self.initialized_client(account_id).await?;
        if !self.handle_is_authorized(&handle).await? {
            return Err(AppError::auth(format!(
                "Account {account_id} is not authenticated"
            )));
        }
        Ok(handle)
    }

    pub async fn client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle> {
        self.initialized_client(account_id).await
    }

    async fn initialized_client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle> {
        self.accounts
            .lock()
            .await
            .get(&account_id)
            .map(|account| account.handle.clone())
            .ok_or_else(|| AppError::auth(format!("Account {account_id} not initialized")))
    }

    pub async fn clear_account(&self, account_id: i64, sign_out: bool) {
        let mut accounts = self.accounts.lock().await;
        if let Some(account) = accounts.remove(&account_id) {
            if sign_out {
                match &account.handle.client {
                    TelegramClientInner::Grammers(client) => {
                        let _ = client.sign_out().await;
                    }
                    #[cfg(test)]
                    TelegramClientInner::Test { account_id } => {
                        let _ = (self
                            .test_callbacks
                            .as_ref()
                            .expect("test Telegram handle requires callbacks")
                            .sign_out)(*account_id)
                        .await;
                    }
                }
            }
            account.runner.abort();
        }
    }

    async fn handle_is_authorized(
        &self,
        handle: &TelegramClientHandle,
    ) -> extractum_core::error::AppResult<bool> {
        match &handle.client {
            TelegramClientInner::Grammers(client) => client
                .is_authorized()
                .await
                .map_err(AppError::telegram_network),
            #[cfg(test)]
            TelegramClientInner::Test { account_id } => {
                (self
                    .test_callbacks
                    .as_ref()
                    .expect("test Telegram handle requires callbacks")
                    .is_authorized)(*account_id)
                .await
            }
        }
    }
}

async fn initialize_grammers_client(
    api_id: i32,
    session: &TelegramSession,
) -> extractum_core::error::AppResult<(TelegramClientInner, JoinHandle<()>, bool)> {
    let configuration = grammers_client::client::ClientConfiguration::default();
    if !configuration.auto_cache_peers {
        return Err(AppError::internal(
            "Grammers client configuration must enable auto_cache_peers",
        ));
    }
    let pool = SenderPool::new(session.clone_memory_session(), api_id);
    let runner = tokio::spawn(async move {
        let _ = pool.runner.run().await;
    });
    let client = grammers_client::Client::with_configuration(pool.handle, configuration);
    let is_authorized = client
        .is_authorized()
        .await
        .map_err(AppError::telegram_network)?;
    Ok((TelegramClientInner::Grammers(client), runner, is_authorized))
}

#[cfg(test)]
type TelegramRuntimeTestFuture<T> = std::pin::Pin<
    Box<dyn std::future::Future<Output = extractum_core::error::AppResult<T>> + Send + 'static>,
>;

#[cfg(test)]
#[derive(Clone)]
struct TelegramRuntimeTestCallbacks {
    is_authorized: Arc<dyn Fn(i64) -> TelegramRuntimeTestFuture<bool> + Send + Sync>,
    request_login_code: Arc<dyn Fn(i64, String) -> TelegramRuntimeTestFuture<u64> + Send + Sync>,
    sign_in: Arc<dyn Fn(i64, u64, String, String) -> TelegramRuntimeTestFuture<()> + Send + Sync>,
    sign_out: Arc<dyn Fn(i64) -> TelegramRuntimeTestFuture<()> + Send + Sync>,
    on_runner_drop: Arc<dyn Fn(i64) + Send + Sync>,
}

#[cfg(test)]
struct TelegramRuntimeTestRunnerDropProbe {
    account_id: i64,
    on_drop: Arc<dyn Fn(i64) + Send + Sync>,
}

#[cfg(test)]
impl Drop for TelegramRuntimeTestRunnerDropProbe {
    fn drop(&mut self) {
        (self.on_drop)(self.account_id);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex as StdMutex};

    use extractum_core::error::{AppErrorKind, AppResult};
    use secrecy::SecretString;
    use tokio::sync::oneshot;

    use super::*;

    fn test_future<T: Send + 'static>(
        future: impl std::future::Future<Output = AppResult<T>> + Send + 'static,
    ) -> TelegramRuntimeTestFuture<T> {
        Box::pin(future)
    }

    fn default_test_callbacks() -> TelegramRuntimeTestCallbacks {
        TelegramRuntimeTestCallbacks {
            is_authorized: Arc::new(|_| test_future(async { Ok(true) })),
            request_login_code: Arc::new(|_, _| test_future(async { Ok(1) })),
            sign_in: Arc::new(|_, _, _, _| test_future(async { Ok(()) })),
            sign_out: Arc::new(|_| test_future(async { Ok(()) })),
            on_runner_drop: Arc::new(|_| {}),
        }
    }

    fn test_api_hash(value: &str) -> TelegramApiHash {
        TelegramApiHash::new(SecretString::new(value.to_string()))
    }

    async fn initialize_test_account(
        runtime: &TelegramRuntime,
        account_id: i64,
        session: TelegramSession,
    ) {
        assert_eq!(
            runtime
                .initialize_account(account_id, 12345, test_api_hash("api-hash"), session)
                .await
                .expect("initialize test account"),
            TelegramRuntimeStatus::Ready
        );
    }

    fn assert_initialization_uses_drop_only_replacement_path() {
        let source = include_str!("runtime.rs").replace("\r\n", "\n");
        assert!(
            source.contains(
                "fn detach_replaced<T>(replaced: Option<T>) {\n    drop(replaced);\n}"
            ),
            "RED: CP5 initialization ordering: the replacement helper must remain a generic drop-only path"
        );

        let initialize_source = source
            .split_once("    pub async fn initialize_account(")
            .expect("initialize_account source")
            .1
            .split_once("    pub async fn is_authenticated(")
            .expect("is_authenticated source boundary")
            .0;
        assert_eq!(
            initialize_source.matches("detach_replaced(").count(),
            1,
            "RED: CP5 initialization ordering: replacement must use the single drop-only path"
        );
        let normalized_initialize_source = initialize_source.split_whitespace().collect::<String>();
        assert!(
            normalized_initialize_source
                .contains("detach_replaced(self.accounts.lock().await.insert("),
            "RED: CP5 initialization ordering: accounts.insert must feed the drop-only replacement path directly"
        );
        assert!(
            !initialize_source.contains(".abort"),
            "RED: CP5 initialization ordering: initialization must not abort a replaced runner"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner(
    ) {
        assert_initialization_uses_drop_only_replacement_path();

        let (first_entered_tx, first_entered_rx) = oneshot::channel();
        let (first_release_tx, first_release_rx) = oneshot::channel();
        let (second_entered_tx, second_entered_rx) = oneshot::channel();
        let (second_release_tx, second_release_rx) = oneshot::channel();
        let authorization_steps = Arc::new(StdMutex::new(VecDeque::from([
            (first_entered_tx, first_release_rx, true),
            (second_entered_tx, second_release_rx, false),
        ])));
        let runner_drops = Arc::new(StdMutex::new(Vec::new()));

        let callbacks = TelegramRuntimeTestCallbacks {
            is_authorized: {
                let authorization_steps = Arc::clone(&authorization_steps);
                Arc::new(move |account_id| {
                    let (entered, release, authorized) = authorization_steps
                        .lock()
                        .expect("authorization step lock")
                        .pop_front()
                        .expect("prepared authorization step");
                    test_future(async move {
                        let _ = entered.send(account_id);
                        let _ = release.await;
                        Ok(authorized)
                    })
                })
            },
            on_runner_drop: {
                let runner_drops = Arc::clone(&runner_drops);
                Arc::new(move |account_id| {
                    runner_drops
                        .lock()
                        .expect("runner-drop ledger lock")
                        .push(account_id);
                })
            },
            ..default_test_callbacks()
        };
        let runtime = Arc::new(TelegramRuntime::with_test_callbacks(callbacks));
        let first_session = TelegramSession::empty();
        let second_session = TelegramSession::empty();

        let mut first_init = {
            let runtime = Arc::clone(&runtime);
            let session = first_session.clone();
            tokio::spawn(async move {
                runtime
                    .initialize_account(7, 12345, test_api_hash("first"), session)
                    .await
            })
        };
        tokio::select! {
            entered = first_entered_rx => {
                assert_eq!(entered.expect("first initialization entered"), 7);
            }
            result = &mut first_init => {
                panic!("RED: CP5 initialization ordering: first initialization returned before callback: {result:?}");
            }
        }

        let mut second_init = {
            let runtime = Arc::clone(&runtime);
            let session = second_session.clone();
            tokio::spawn(async move {
                runtime
                    .initialize_account(7, 12345, test_api_hash("second"), session)
                    .await
            })
        };
        tokio::select! {
            entered = second_entered_rx => {
                assert_eq!(entered.expect("second initialization entered"), 7);
            }
            result = &mut second_init => {
                panic!("RED: CP5 initialization ordering: second initialization returned before callback: {result:?}");
            }
        }

        second_release_tx
            .send(())
            .expect("release second initialization");
        assert_eq!(
            second_init
                .await
                .expect("second initialization task")
                .expect("second initialization result"),
            TelegramRuntimeStatus::ReauthRequired,
            "RED: CP5 initialization ordering"
        );
        first_release_tx
            .send(())
            .expect("release first initialization");
        assert_eq!(
            first_init
                .await
                .expect("first initialization task")
                .expect("first initialization result"),
            TelegramRuntimeStatus::Ready,
            "RED: CP5 initialization ordering"
        );

        let winning_handle = runtime
            .initialized_client(7)
            .await
            .expect("initialized winning account");
        let winning_session = winning_handle.session.clone_memory_session();
        let first_memory_session = first_session.clone_memory_session();
        assert!(
            Arc::ptr_eq(&winning_session, &first_memory_session),
            "RED: CP5 initialization ordering: last accounts.insert must win"
        );
        assert!(
            runner_drops.lock().expect("runner-drop ledger lock").is_empty(),
            "RED: CP5 initialization ordering: no replaced-runner drop was observed in the completed-initialization assertion window"
        );

        let missing = match runtime.initialized_client(99).await {
            Ok(_) => panic!("RED: CP5 initialization ordering: missing lookup must fail"),
            Err(error) => error,
        };
        assert_eq!(missing.kind, AppErrorKind::Auth);
        assert_eq!(missing.message, "Account 99 not initialized");
    }

    #[tokio::test]
    async fn missing_account_authentication_is_false() {
        let runtime = TelegramRuntime::with_test_callbacks(default_test_callbacks());

        assert!(
            !runtime
                .is_authenticated(99)
                .await
                .expect("RED: CP5 missing account authentication"),
            "RED: CP5 missing account authentication"
        );
    }

    #[tokio::test]
    async fn request_login_code_serializes_queued_requests_and_later_success_replaces_attempt() {
        let (a1_entered_tx, a1_entered_rx) = oneshot::channel();
        let (a1_release_tx, a1_release_rx) = oneshot::channel();
        let (b1_entered_tx, mut b1_entered_rx) = oneshot::channel();
        let (b1_release_tx, b1_release_rx) = oneshot::channel();
        let (failed_entered_tx, failed_entered_rx) = oneshot::channel();
        let (failed_release_tx, failed_release_rx) = oneshot::channel();
        let (a2_entered_tx, a2_entered_rx) = oneshot::channel();
        let (a2_release_tx, a2_release_rx) = oneshot::channel();
        let request_steps = Arc::new(StdMutex::new(VecDeque::from([
            (7, "+1001".to_string(), a1_entered_tx, a1_release_rx, Ok(11)),
            (8, "+2001".to_string(), b1_entered_tx, b1_release_rx, Ok(21)),
            (
                7,
                "+failed".to_string(),
                failed_entered_tx,
                failed_release_rx,
                Err(AppError::telegram_network("replacement request failed")),
            ),
            (7, "+1002".to_string(), a2_entered_tx, a2_release_rx, Ok(12)),
        ])));
        let sign_in_calls = Arc::new(StdMutex::new(Vec::new()));
        let sign_in_results = Arc::new(StdMutex::new(VecDeque::from([
            Err(AppError::telegram_network("old attempt probe failed")),
            Ok(()),
        ])));
        let callbacks = TelegramRuntimeTestCallbacks {
            request_login_code: {
                let request_steps = Arc::clone(&request_steps);
                Arc::new(move |account_id, phone| {
                    let (expected_id, expected_phone, entered, release, result) = request_steps
                        .lock()
                        .expect("request-step lock")
                        .pop_front()
                        .expect("prepared request step");
                    test_future(async move {
                        assert_eq!(
                            (account_id, phone.as_str()),
                            (expected_id, expected_phone.as_str()),
                            "RED: CP5 serialized login attempts"
                        );
                        let _ = entered.send(());
                        let _ = release.await;
                        result
                    })
                })
            },
            sign_in: {
                let sign_in_calls = Arc::clone(&sign_in_calls);
                let sign_in_results = Arc::clone(&sign_in_results);
                Arc::new(move |account_id, marker, phone, code| {
                    sign_in_calls
                        .lock()
                        .expect("sign-in ledger lock")
                        .push((account_id, marker, phone, code));
                    let result = sign_in_results
                        .lock()
                        .expect("sign-in result lock")
                        .pop_front()
                        .expect("prepared sign-in result");
                    test_future(async move { result })
                })
            },
            ..default_test_callbacks()
        };
        let runtime = Arc::new(TelegramRuntime::with_test_callbacks(callbacks));
        initialize_test_account(&runtime, 7, TelegramSession::empty()).await;
        initialize_test_account(&runtime, 8, TelegramSession::empty()).await;

        let mut a1_request = {
            let runtime = Arc::clone(&runtime);
            tokio::spawn(async move { runtime.request_login_code(7, "+1001".to_string()).await })
        };
        tokio::select! {
            entered = a1_entered_rx => {
                entered.expect("A1 request entered");
            }
            result = &mut a1_request => {
                panic!("RED: CP5 serialized login attempts: A1 returned before callback: {result:?}");
            }
        }

        let (b1_started_tx, b1_started_rx) = oneshot::channel();
        let b1_request = {
            let runtime = Arc::clone(&runtime);
            tokio::spawn(async move {
                let _ = b1_started_tx.send(());
                runtime.request_login_code(8, "+2001".to_string()).await
            })
        };
        b1_started_rx.await.expect("B1 task started");
        tokio::select! {
            biased;
            entered = &mut b1_entered_rx => {
                panic!("RED: CP5 serialized login attempts: B1 entered before A1 released: {entered:?}");
            }
            _ = tokio::task::yield_now() => {}
        }

        a1_release_tx.send(()).expect("release A1 request");
        a1_request
            .await
            .expect("A1 request task")
            .expect("RED: CP5 serialized login attempts");
        b1_entered_rx.await.expect("B1 request entered after A1");
        b1_release_tx.send(()).expect("release B1 request");
        b1_request
            .await
            .expect("B1 request task")
            .expect("RED: CP5 serialized login attempts");

        failed_release_tx
            .send(())
            .expect("release failed replacement request");
        let failed_request = runtime
            .request_login_code(7, "+failed".to_string())
            .await
            .expect_err("RED: CP5 serialized login attempts");
        failed_entered_rx
            .await
            .expect("failed replacement request entered");
        assert_eq!(
            failed_request.kind,
            AppErrorKind::Network,
            "RED: CP5 serialized login attempts"
        );
        assert_eq!(
            failed_request.message, "Telegram request failed: replacement request failed",
            "RED: CP5 serialized login attempts"
        );
        let old_attempt_probe = match runtime.sign_in(7, "probe-old".to_string()).await {
            Ok(_) => panic!("RED: CP5 serialized login attempts"),
            Err(error) => error,
        };
        assert_eq!(
            old_attempt_probe.kind,
            AppErrorKind::Network,
            "RED: CP5 serialized login attempts"
        );
        assert_eq!(
            old_attempt_probe.message, "Telegram request failed: old attempt probe failed",
            "RED: CP5 serialized login attempts"
        );
        assert_eq!(
            *sign_in_calls.lock().expect("sign-in ledger lock"),
            vec![(7, 11, "+1001".to_string(), "probe-old".to_string())],
            "RED: CP5 serialized login attempts"
        );

        let mut a2_request = {
            let runtime = Arc::clone(&runtime);
            tokio::spawn(async move { runtime.request_login_code(7, "+1002".to_string()).await })
        };
        tokio::select! {
            entered = a2_entered_rx => {
                entered.expect("A2 request entered");
            }
            result = &mut a2_request => {
                panic!("RED: CP5 serialized login attempts: A2 returned before callback: {result:?}");
            }
        }
        a2_release_tx.send(()).expect("release A2 request");
        a2_request
            .await
            .expect("A2 request task")
            .expect("RED: CP5 serialized login attempts");

        runtime
            .sign_in(7, "9999".to_string())
            .await
            .expect("RED: CP5 serialized login attempts");
        assert_eq!(
            *sign_in_calls.lock().expect("sign-in ledger lock"),
            vec![
                (7, 11, "+1001".to_string(), "probe-old".to_string()),
                (7, 12, "+1002".to_string(), "9999".to_string()),
            ],
            "RED: CP5 serialized login attempts"
        );
    }

    #[tokio::test]
    async fn sign_in_without_code_request_preserves_auth_error() {
        let sign_in_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let callbacks = TelegramRuntimeTestCallbacks {
            sign_in: {
                let sign_in_calls = Arc::clone(&sign_in_calls);
                Arc::new(move |_, _, _, _| {
                    sign_in_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    test_future(async { Ok(()) })
                })
            },
            ..default_test_callbacks()
        };
        let runtime = TelegramRuntime::with_test_callbacks(callbacks);
        initialize_test_account(&runtime, 7, TelegramSession::empty()).await;

        let error = match runtime.sign_in(7, "12345".to_string()).await {
            Ok(_) => panic!("RED: CP5 missing login attempt"),
            Err(error) => error,
        };

        assert_eq!(
            error.kind,
            AppErrorKind::Auth,
            "RED: CP5 missing login attempt"
        );
        assert_eq!(
            error.message, "Call tg_send_code first",
            "RED: CP5 missing login attempt"
        );
        assert_eq!(
            sign_in_calls.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "RED: CP5 missing login attempt"
        );
    }

    #[tokio::test]
    async fn failed_sign_in_retains_pending_attempt() {
        let sign_in_calls = Arc::new(StdMutex::new(Vec::new()));
        let sign_in_results = Arc::new(StdMutex::new(VecDeque::from([
            Err(AppError::telegram_network("invalid code")),
            Ok(()),
        ])));
        let callbacks = TelegramRuntimeTestCallbacks {
            request_login_code: Arc::new(|account_id, phone| {
                test_future(async move {
                    assert_eq!(
                        (account_id, phone.as_str()),
                        (7, "+1001"),
                        "RED: CP5 retain failed login attempt"
                    );
                    Ok(41)
                })
            }),
            sign_in: {
                let sign_in_calls = Arc::clone(&sign_in_calls);
                let sign_in_results = Arc::clone(&sign_in_results);
                Arc::new(move |account_id, marker, phone, code| {
                    sign_in_calls
                        .lock()
                        .expect("sign-in ledger lock")
                        .push((account_id, marker, phone, code));
                    let result = sign_in_results
                        .lock()
                        .expect("sign-in result lock")
                        .pop_front()
                        .expect("prepared sign-in result");
                    test_future(async move { result })
                })
            },
            ..default_test_callbacks()
        };
        let runtime = TelegramRuntime::with_test_callbacks(callbacks);
        let session = TelegramSession::empty();
        initialize_test_account(&runtime, 7, session.clone()).await;
        runtime
            .request_login_code(7, "+1001".to_string())
            .await
            .expect("prepare pending login attempt");

        let first_error = match runtime.sign_in(7, "bad".to_string()).await {
            Ok(_) => panic!("RED: CP5 retain failed login attempt"),
            Err(error) => error,
        };
        assert_eq!(
            first_error.kind,
            AppErrorKind::Network,
            "RED: CP5 retain failed login attempt"
        );
        assert_eq!(
            first_error.message, "Telegram request failed: invalid code",
            "RED: CP5 retain failed login attempt"
        );

        let returned = runtime
            .sign_in(7, "good".to_string())
            .await
            .expect("RED: CP5 retain failed login attempt");
        let returned_memory_session = returned.clone_memory_session();
        let expected_memory_session = session.clone_memory_session();
        assert!(
            Arc::ptr_eq(&returned_memory_session, &expected_memory_session),
            "RED: CP5 retain failed login attempt"
        );
        assert_eq!(
            *sign_in_calls.lock().expect("sign-in ledger lock"),
            vec![
                (7, 41, "+1001".to_string(), "bad".to_string()),
                (7, 41, "+1001".to_string(), "good".to_string()),
            ],
            "RED: CP5 retain failed login attempt"
        );
    }

    #[tokio::test]
    async fn successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt() {
        let direct_runtime = TelegramRuntime::with_test_callbacks(default_test_callbacks());
        let direct_session = TelegramSession::empty();
        initialize_test_account(&direct_runtime, 7, direct_session.clone()).await;
        direct_runtime
            .request_login_code(7, "+1001".to_string())
            .await
            .expect("prepare direct sign-in");

        let returned = direct_runtime
            .sign_in(7, "12345".to_string())
            .await
            .expect("direct sign-in succeeds");
        let returned_memory_session = returned.clone_memory_session();
        let expected_memory_session = direct_session.clone_memory_session();
        assert!(
            Arc::ptr_eq(&returned_memory_session, &expected_memory_session),
            "RED: CP5 serialized successful sign in"
        );
        let second_error = match direct_runtime.sign_in(7, "12345".to_string()).await {
            Ok(_) => panic!("RED: CP5 serialized successful sign in"),
            Err(error) => error,
        };
        assert_eq!(
            second_error.kind,
            AppErrorKind::Auth,
            "RED: CP5 serialized successful sign in"
        );
        assert_eq!(
            second_error.message, "Call tg_send_code first",
            "RED: CP5 serialized successful sign in"
        );

        let (sign_in_entered_tx, sign_in_entered_rx) = oneshot::channel();
        let (sign_in_release_tx, sign_in_release_rx) = oneshot::channel();
        let sign_in_step = Arc::new(StdMutex::new(Some((
            sign_in_entered_tx,
            sign_in_release_rx,
        ))));
        let (sign_out_entered_tx, mut sign_out_entered_rx) = oneshot::channel();
        let (sign_out_release_tx, sign_out_release_rx) = oneshot::channel();
        let sign_out_step = Arc::new(StdMutex::new(Some((
            sign_out_entered_tx,
            sign_out_release_rx,
        ))));
        let (runner_drop_tx, mut runner_drop_rx) = tokio::sync::mpsc::unbounded_channel();
        let callbacks = TelegramRuntimeTestCallbacks {
            sign_in: {
                let sign_in_step = Arc::clone(&sign_in_step);
                Arc::new(move |_, _, _, _| {
                    let (entered, release) = sign_in_step
                        .lock()
                        .expect("sign-in step lock")
                        .take()
                        .expect("prepared sign-in step");
                    test_future(async move {
                        let _ = entered.send(());
                        let _ = release.await;
                        Ok(())
                    })
                })
            },
            sign_out: {
                let sign_out_step = Arc::clone(&sign_out_step);
                Arc::new(move |account_id| {
                    let (entered, release) = sign_out_step
                        .lock()
                        .expect("sign-out step lock")
                        .take()
                        .expect("prepared sign-out step");
                    test_future(async move {
                        assert_eq!(account_id, 8, "RED: CP5 serialized successful sign in");
                        let _ = entered.send(());
                        let _ = release.await;
                        Ok(())
                    })
                })
            },
            on_runner_drop: Arc::new(move |account_id| {
                let _ = runner_drop_tx.send(account_id);
            }),
            ..default_test_callbacks()
        };
        let runtime = Arc::new(TelegramRuntime::with_test_callbacks(callbacks));
        let session = TelegramSession::empty();
        initialize_test_account(&runtime, 8, session).await;
        runtime
            .request_login_code(8, "+2001".to_string())
            .await
            .expect("prepare blocked sign-in");

        let mut sign_in_task = {
            let runtime = Arc::clone(&runtime);
            tokio::spawn(async move { runtime.sign_in(8, "54321".to_string()).await })
        };
        tokio::select! {
            entered = sign_in_entered_rx => {
                entered.expect("blocked sign-in entered");
            }
            _ = &mut sign_in_task => {
                panic!("RED: CP5 serialized successful sign in: sign-in returned before release");
            }
        }

        let (clear_started_tx, clear_started_rx) = oneshot::channel();
        let mut clear_task = {
            let runtime = Arc::clone(&runtime);
            tokio::spawn(async move {
                let _ = clear_started_tx.send(());
                runtime.clear_account(8, true).await;
            })
        };
        clear_started_rx.await.expect("clear task started");
        tokio::select! {
            biased;
            entered = &mut sign_out_entered_rx => {
                panic!("RED: CP5 serialized successful sign in: clear signed out before sign-in release: {entered:?}");
            }
            _ = tokio::task::yield_now() => {}
        }
        assert!(
            runner_drop_rx.try_recv().is_err(),
            "RED: CP5 serialized successful sign in: clear aborted runner before sign-in release"
        );

        sign_in_release_tx
            .send(())
            .expect("release blocked sign-in");
        sign_in_task
            .await
            .expect("blocked sign-in task")
            .expect("RED: CP5 serialized successful sign in");
        tokio::select! {
            entered = &mut sign_out_entered_rx => {
                entered.expect("clear sign-out entered");
            }
            result = &mut clear_task => {
                panic!("RED: CP5 serialized successful sign in: clear returned without sign-out: {result:?}");
            }
        }
        assert!(
            runner_drop_rx.try_recv().is_err(),
            "RED: CP5 serialized successful sign in: runner aborted before sign-out completed"
        );
        sign_out_release_tx
            .send(())
            .expect("release clear sign-out");
        clear_task.await.expect("clear task");
        assert_eq!(
            runner_drop_rx.recv().await,
            Some(8),
            "RED: CP5 serialized successful sign in"
        );

        let missing = match runtime.initialized_client(8).await {
            Ok(_) => panic!("RED: CP5 serialized successful sign in"),
            Err(error) => error,
        };
        assert_eq!(missing.kind, AppErrorKind::Auth);
        assert_eq!(missing.message, "Account 8 not initialized");
    }

    #[tokio::test]
    async fn clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure(
    ) {
        let (request_entered_tx, request_entered_rx) = oneshot::channel();
        let (request_release_tx, request_release_rx) = oneshot::channel();
        let request_step = Arc::new(StdMutex::new(Some((
            request_entered_tx,
            request_release_rx,
        ))));
        let (sign_out_entered_tx, mut sign_out_entered_rx) = oneshot::channel();
        let (sign_out_release_tx, sign_out_release_rx) = oneshot::channel();
        let sign_out_step = Arc::new(StdMutex::new(Some((
            sign_out_entered_tx,
            sign_out_release_rx,
        ))));
        let sign_out_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (runner_drop_tx, mut runner_drop_rx) = tokio::sync::mpsc::unbounded_channel();
        let callbacks = TelegramRuntimeTestCallbacks {
            request_login_code: {
                let request_step = Arc::clone(&request_step);
                Arc::new(move |account_id, phone| {
                    let (entered, release) = request_step
                        .lock()
                        .expect("request step lock")
                        .take()
                        .expect("only the first request reaches its callback");
                    test_future(async move {
                        assert_eq!(
                            (account_id, phone.as_str()),
                            (7, "+1001"),
                            "RED: CP5 runtime clear ordering"
                        );
                        let _ = entered.send(());
                        let _ = release.await;
                        Ok(51)
                    })
                })
            },
            sign_out: {
                let sign_out_step = Arc::clone(&sign_out_step);
                let sign_out_calls = Arc::clone(&sign_out_calls);
                Arc::new(move |account_id| {
                    sign_out_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let (entered, release) = sign_out_step
                        .lock()
                        .expect("sign-out step lock")
                        .take()
                        .expect("only initialized account signs out");
                    test_future(async move {
                        assert_eq!(account_id, 7, "RED: CP5 runtime clear ordering");
                        let _ = entered.send(());
                        let _ = release.await;
                        Err(AppError::telegram_network("sign out failed"))
                    })
                })
            },
            on_runner_drop: Arc::new(move |account_id| {
                let _ = runner_drop_tx.send(account_id);
            }),
            ..default_test_callbacks()
        };
        let runtime = Arc::new(TelegramRuntime::with_test_callbacks(callbacks));
        initialize_test_account(&runtime, 7, TelegramSession::empty()).await;

        let mut request_task = {
            let runtime = Arc::clone(&runtime);
            tokio::spawn(async move { runtime.request_login_code(7, "+1001".to_string()).await })
        };
        tokio::select! {
            entered = request_entered_rx => {
                entered.expect("blocked request entered");
            }
            result = &mut request_task => {
                panic!("RED: CP5 runtime clear ordering: request returned before release: {result:?}");
            }
        }

        let (clear_started_tx, clear_started_rx) = oneshot::channel();
        let mut clear_task = {
            let runtime = Arc::clone(&runtime);
            tokio::spawn(async move {
                let _ = clear_started_tx.send(());
                runtime.clear_account(7, true).await;
            })
        };
        clear_started_rx.await.expect("clear task started");
        tokio::select! {
            biased;
            entered = &mut sign_out_entered_rx => {
                panic!("RED: CP5 runtime clear ordering: sign-out entered before request release: {entered:?}");
            }
            _ = tokio::task::yield_now() => {}
        }
        assert!(
            runner_drop_rx.try_recv().is_err(),
            "RED: CP5 runtime clear ordering: runner dropped before request release"
        );

        request_release_tx.send(()).expect("release request");
        request_task
            .await
            .expect("request task")
            .expect("RED: CP5 runtime clear ordering");
        tokio::select! {
            entered = &mut sign_out_entered_rx => {
                entered.expect("sign-out entered after request");
            }
            result = &mut clear_task => {
                panic!("RED: CP5 runtime clear ordering: clear returned before sign-out: {result:?}");
            }
        }
        assert!(
            runner_drop_rx.try_recv().is_err(),
            "RED: CP5 runtime clear ordering: runner dropped before sign-out completion"
        );

        sign_out_release_tx.send(()).expect("release sign-out");
        clear_task.await.expect("RED: CP5 runtime clear ordering");
        assert_eq!(
            runner_drop_rx.recv().await,
            Some(7),
            "RED: CP5 runtime clear ordering"
        );

        let post_clear = runtime
            .request_login_code(7, "+1002".to_string())
            .await
            .expect_err("RED: CP5 runtime clear ordering");
        assert_eq!(post_clear.kind, AppErrorKind::Auth);
        assert_eq!(post_clear.message, "Account not initialized");

        runtime.clear_account(99, true).await;
        assert_eq!(
            sign_out_calls.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "RED: CP5 runtime clear ordering"
        );
        assert!(
            runner_drop_rx.try_recv().is_err(),
            "RED: CP5 runtime clear ordering: missing clear must not drop a runner"
        );
    }

    #[tokio::test]
    async fn client_preserves_missing_account_error_without_authorization_check() {
        let authorization_checks = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let callbacks = TelegramRuntimeTestCallbacks {
            is_authorized: {
                let authorization_checks = Arc::clone(&authorization_checks);
                Arc::new(move |account_id| {
                    authorization_checks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    test_future(async move {
                        assert_eq!(account_id, 7);
                        Ok(false)
                    })
                })
            },
            ..default_test_callbacks()
        };
        let runtime = TelegramRuntime::with_test_callbacks(callbacks);

        assert_eq!(
            runtime
                .initialize_account(
                    7,
                    12345,
                    test_api_hash("api-hash"),
                    TelegramSession::empty(),
                )
                .await
                .expect("initialize unauthenticated account"),
            TelegramRuntimeStatus::ReauthRequired
        );

        let missing = match runtime.client(99).await {
            Ok(_) => panic!("missing account lookup must fail"),
            Err(error) => error,
        };
        assert_eq!(missing.kind, AppErrorKind::Auth);
        assert_eq!(missing.message, "Account 99 not initialized");

        let initialized = runtime.client(7).await;
        assert!(initialized.is_ok(), "PHASE8B_RED_RUNTIME_CLIENT_LOOKUP");
        assert_eq!(
            authorization_checks.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
    }

    #[tokio::test]
    async fn authorized_client_preserves_missing_and_unauthenticated_errors() {
        let (probe_entered_tx, probe_entered_rx) = oneshot::channel();
        let (probe_release_tx, probe_release_rx) = oneshot::channel();
        let authorization_steps = Arc::new(StdMutex::new(VecDeque::from([
            (None, None, false),
            (Some(probe_entered_tx), Some(probe_release_rx), false),
        ])));
        let callbacks = TelegramRuntimeTestCallbacks {
            is_authorized: {
                let authorization_steps = Arc::clone(&authorization_steps);
                Arc::new(move |account_id| {
                    let (entered, release, authorized) = authorization_steps
                        .lock()
                        .expect("authorization step lock")
                        .pop_front()
                        .expect("prepared authorization step");
                    test_future(async move {
                        assert_eq!(account_id, 7, "RED: CP5 authorized client");
                        if let Some(entered) = entered {
                            let _ = entered.send(());
                        }
                        if let Some(release) = release {
                            let _ = release.await;
                        }
                        Ok(authorized)
                    })
                })
            },
            ..default_test_callbacks()
        };
        let runtime = Arc::new(TelegramRuntime::with_test_callbacks(callbacks));

        let missing = match runtime.authorized_client(99).await {
            Ok(_) => panic!("RED: CP5 authorized client"),
            Err(error) => error,
        };
        assert_eq!(
            missing.kind,
            AppErrorKind::Auth,
            "RED: CP5 authorized client"
        );
        assert_eq!(
            missing.message, "Account 99 not initialized",
            "RED: CP5 authorized client"
        );

        assert_eq!(
            runtime
                .initialize_account(
                    7,
                    12345,
                    test_api_hash("api-hash"),
                    TelegramSession::empty(),
                )
                .await
                .expect("initialize unauthenticated account"),
            TelegramRuntimeStatus::ReauthRequired
        );

        let mut authorized_task = {
            let runtime = Arc::clone(&runtime);
            tokio::spawn(async move { runtime.authorized_client(7).await })
        };
        tokio::select! {
            entered = probe_entered_rx => {
                entered.expect("authorized probe entered");
            }
            _ = &mut authorized_task => {
                panic!("RED: CP5 authorized client: lookup returned before authorization callback");
            }
        }

        runtime
            .initialized_client(7)
            .await
            .expect("initialized lookup must complete while authorization probe is blocked");
        probe_release_tx
            .send(())
            .expect("release authorization probe");
        let unauthenticated = match authorized_task.await.expect("authorized lookup task") {
            Ok(_) => panic!("RED: CP5 authorized client"),
            Err(error) => error,
        };
        assert_eq!(
            unauthenticated.kind,
            AppErrorKind::Auth,
            "RED: CP5 authorized client"
        );
        assert_eq!(
            unauthenticated.message, "Account 7 is not authenticated",
            "RED: CP5 authorized client"
        );
    }
}

#[cfg(test)]
impl TelegramRuntimeTestCallbacks {
    fn spawn_pending_runner(&self, account_id: i64) -> JoinHandle<()> {
        let probe = TelegramRuntimeTestRunnerDropProbe {
            account_id,
            on_drop: Arc::clone(&self.on_runner_drop),
        };
        tokio::spawn(async move {
            let _probe = probe;
            std::future::pending::<()>().await;
        })
    }
}

#[cfg(test)]
impl TelegramRuntime {
    fn with_test_callbacks(callbacks: TelegramRuntimeTestCallbacks) -> Self {
        Self {
            accounts: Mutex::new(HashMap::new()),
            test_callbacks: Some(Arc::new(callbacks)),
        }
    }
}
