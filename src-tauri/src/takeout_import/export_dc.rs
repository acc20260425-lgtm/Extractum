use std::{future::Future, sync::Arc};

use grammers_client::{tl, Client};
use grammers_mtsender::InvocationError;
use grammers_session::{storages::MemorySession, Session};

use crate::error::{AppError, AppResult};
use crate::sources::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};

const EXPORT_DC_SHIFT: i32 = 4 * 10_000;
const TAKEOUT_FILE_MAX_SIZE: i64 = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExportDcAlias {
    pub(crate) home_dc_id: i32,
    pub(crate) export_dc_id: i32,
}

#[derive(Default)]
pub(crate) struct ExportDcAttemptState {
    attempted_export_dc_id: Option<i32>,
    fallback_recorded: bool,
}

impl ExportDcAttemptState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn mark_attempted(&mut self, export_dc_id: i32) -> bool {
        if self.attempted_export_dc_id == Some(export_dc_id) {
            return false;
        }
        self.attempted_export_dc_id = Some(export_dc_id);
        true
    }

    pub(crate) fn mark_fallback(&mut self, message: String) -> Option<String> {
        if self.fallback_recorded {
            return None;
        }
        self.fallback_recorded = true;
        Some(message)
    }
}

pub(crate) async fn prepare_export_dc_alias(
    session: &Arc<MemorySession>,
) -> AppResult<ExportDcAlias> {
    let home_dc_id = session.home_dc_id();
    let export_dc_id = export_dc_id_for_home_dc(home_dc_id);
    let mut export_option = session.dc_option(home_dc_id).ok_or_else(|| {
        AppError::internal(format!(
            "Home DC option {home_dc_id} is missing from session"
        ))
    })?;
    export_option.id = export_dc_id;
    session.set_dc_option(&export_option).await;

    Ok(ExportDcAlias {
        home_dc_id,
        export_dc_id,
    })
}

fn export_dc_id_for_home_dc(home_dc_id: i32) -> i32 {
    home_dc_id + EXPORT_DC_SHIFT
}

pub(crate) fn takeout_init_request_for_source_subtype(
    source_subtype: &str,
) -> AppResult<tl::functions::account::InitTakeoutSession> {
    let (message_chats, message_megagroups, message_channels) = match source_subtype {
        TELEGRAM_KIND_GROUP => (true, false, false),
        TELEGRAM_KIND_SUPERGROUP => (false, true, false),
        TELEGRAM_KIND_CHANNEL => (false, false, true),
        other => {
            return Err(AppError::validation(format!(
                "Unsupported Telegram source_subtype '{other}'"
            )));
        }
    };

    Ok(tl::functions::account::InitTakeoutSession {
        contacts: false,
        message_users: false,
        message_chats,
        message_megagroups,
        message_channels,
        files: true,
        file_max_size: Some(TAKEOUT_FILE_MAX_SIZE),
    })
}

pub(crate) async fn export_dc_invoke<R: tl::RemoteCall>(
    client: &Client,
    alias: &ExportDcAlias,
    request: &R,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<R::Return> {
    export_dc_invoke_with(
        alias,
        warnings,
        fallback_used,
        || client.invoke_in_dc(alias.export_dc_id, request),
        || client.invoke(request),
    )
    .await
}

async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
    alias: &ExportDcAlias,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    shifted_invoke: Shifted,
    home_invoke: Home,
) -> AppResult<R>
where
    Shifted: FnOnce() -> ShiftedFuture,
    Home: FnOnce() -> HomeFuture,
    ShiftedFuture: Future<Output = Result<R, InvocationError>>,
    HomeFuture: Future<Output = Result<R, InvocationError>>,
{
    if !*fallback_used {
        match shifted_invoke().await {
            Ok(response) => return Ok(response),
            Err(error) if should_fallback_export_dc_error(&error) => {
                *fallback_used = true;
                warnings.push(format!(
                    "Export DC {} failed with local transport error; falling back to home DC {}: {error}",
                    alias.export_dc_id, alias.home_dc_id
                ));
            }
            Err(error) => return Err(AppError::network(error.to_string())),
        }
    }

    home_invoke()
        .await
        .map_err(|error| AppError::network(error.to_string()))
}

fn should_fallback_export_dc_error(error: &InvocationError) -> bool {
    matches!(
        error,
        InvocationError::InvalidDc
            | InvocationError::Io(_)
            | InvocationError::Transport(_)
            | InvocationError::Authentication(_)
            | InvocationError::Dropped
    )
}

pub(crate) async fn finish_takeout_session(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    success: bool,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<()> {
    export_dc_invoke(
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::account::FinishTakeoutSession { success },
        },
        warnings,
        fallback_used,
    )
    .await
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::{
        export_dc_id_for_home_dc, export_dc_invoke_with, should_fallback_export_dc_error,
        takeout_init_request_for_source_subtype, ExportDcAlias, ExportDcAttemptState,
        TAKEOUT_FILE_MAX_SIZE,
    };
    use crate::error::AppErrorKind;
    use crate::sources::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
    use grammers_mtsender::{InvocationError, RpcError};
    use std::sync::{Arc, Mutex};

    #[test]
    fn export_dc_id_applies_tdesktop_shift() {
        assert_eq!(export_dc_id_for_home_dc(2), 40_002);
    }

    #[test]
    fn export_dc_attempt_state_detects_first_fallback_transition() {
        let mut state = ExportDcAttemptState::new();
        assert!(state.mark_attempted(40002));
        assert!(!state.mark_attempted(40002));
        assert!(state
            .mark_fallback("fallback message".to_string())
            .is_some());
        assert!(state.mark_fallback("second fallback".to_string()).is_none());
    }

    #[test]
    fn takeout_init_request_uses_source_subtype_flags_and_file_limit() {
        let group =
            takeout_init_request_for_source_subtype(TELEGRAM_KIND_GROUP).expect("group flags");
        assert!(group.message_chats);
        assert!(!group.message_megagroups);
        assert!(!group.message_channels);
        assert!(group.files);
        assert_eq!(group.file_max_size, Some(TAKEOUT_FILE_MAX_SIZE));

        let supergroup = takeout_init_request_for_source_subtype(TELEGRAM_KIND_SUPERGROUP)
            .expect("supergroup flags");
        assert!(!supergroup.message_chats);
        assert!(supergroup.message_megagroups);
        assert!(!supergroup.message_channels);

        let channel =
            takeout_init_request_for_source_subtype(TELEGRAM_KIND_CHANNEL).expect("channel flags");
        assert!(!channel.message_chats);
        assert!(!channel.message_megagroups);
        assert!(channel.message_channels);
    }

    #[tokio::test]
    async fn export_dc_invoke_falls_back_to_home_dc_on_local_error() {
        let alias = ExportDcAlias {
            home_dc_id: 2,
            export_dc_id: 40_002,
        };
        let calls = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let shifted_calls = Arc::clone(&calls);
        let home_calls = Arc::clone(&calls);
        let mut warnings = Vec::new();
        let mut fallback_used = false;

        let result = export_dc_invoke_with(
            &alias,
            &mut warnings,
            &mut fallback_used,
            || async move {
                shifted_calls
                    .lock()
                    .expect("lock shifted calls")
                    .push("shifted");
                Err::<i32, InvocationError>(InvocationError::InvalidDc)
            },
            || async move {
                home_calls.lock().expect("lock home calls").push("home");
                Ok(42_i32)
            },
        )
        .await
        .expect("fallback should use home DC");

        assert_eq!(result, 42);
        assert!(fallback_used);
        assert_eq!(*calls.lock().expect("lock calls"), vec!["shifted", "home"]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Export DC 40002 failed"));
        assert!(warnings[0].contains("falling back to home DC 2"));
    }

    #[tokio::test]
    async fn export_dc_invoke_uses_home_dc_directly_after_fallback() {
        let alias = ExportDcAlias {
            home_dc_id: 2,
            export_dc_id: 40_002,
        };
        let calls = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let shifted_calls = Arc::clone(&calls);
        let home_calls = Arc::clone(&calls);
        let mut warnings = Vec::new();
        let mut fallback_used = true;

        let result = export_dc_invoke_with(
            &alias,
            &mut warnings,
            &mut fallback_used,
            || async move {
                shifted_calls
                    .lock()
                    .expect("lock shifted calls")
                    .push("shifted");
                Err::<i32, InvocationError>(InvocationError::InvalidDc)
            },
            || async move {
                home_calls.lock().expect("lock home calls").push("home");
                Ok(7_i32)
            },
        )
        .await
        .expect("already-fallback mode should use home DC");

        assert_eq!(result, 7);
        assert!(fallback_used);
        assert!(warnings.is_empty());
        assert_eq!(*calls.lock().expect("lock calls"), vec!["home"]);
    }

    #[tokio::test]
    async fn export_dc_invoke_does_not_fallback_for_rpc_errors() {
        let alias = ExportDcAlias {
            home_dc_id: 2,
            export_dc_id: 40_002,
        };
        let calls = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let shifted_calls = Arc::clone(&calls);
        let home_calls = Arc::clone(&calls);
        let mut warnings = Vec::new();
        let mut fallback_used = false;

        let error = export_dc_invoke_with(
            &alias,
            &mut warnings,
            &mut fallback_used,
            || async move {
                shifted_calls
                    .lock()
                    .expect("lock shifted calls")
                    .push("shifted");
                Err::<i32, InvocationError>(InvocationError::Rpc(RpcError {
                    code: 400,
                    name: "TAKEOUT_INVALID".to_string(),
                    value: None,
                    caused_by: None,
                }))
            },
            || async move {
                home_calls.lock().expect("lock home calls").push("home");
                Ok(99_i32)
            },
        )
        .await
        .expect_err("RPC errors should not use export-DC fallback");

        assert_eq!(error.kind, AppErrorKind::Network);
        assert!(error.message.contains("TAKEOUT_INVALID"));
        assert!(!fallback_used);
        assert!(warnings.is_empty());
        assert_eq!(*calls.lock().expect("lock calls"), vec!["shifted"]);
    }

    #[test]
    fn export_dc_fallback_is_only_for_local_transport_errors() {
        assert!(should_fallback_export_dc_error(&InvocationError::InvalidDc));
        assert!(should_fallback_export_dc_error(&InvocationError::Dropped));
        assert!(!should_fallback_export_dc_error(&InvocationError::Rpc(
            RpcError {
                code: 400,
                name: "TAKEOUT_INVALID".to_string(),
                value: None,
                caused_by: None,
            }
        )));
    }
}
