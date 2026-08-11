use std::{future::Future, sync::Arc};

use extractum_core::error::{AppError, AppResult};
use grammers_client::tl;
use grammers_mtsender::InvocationError;
use grammers_session::{storages::MemorySession, Session};

use crate::session::session_error;

use super::super::error::should_fallback_export_dc_error;
use super::{
    transport::TakeoutTransport,
    types::{TakeoutAttempt, TakeoutFallback, TakeoutFallbackKind},
};

const EXPORT_DC_SHIFT: i32 = 4 * 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExportDcAlias {
    home_dc_id: i32,
    export_dc_id: i32,
}

pub(super) async fn prepare_export_dc_alias(session: &Arc<MemorySession>) -> AppResult<(i32, i32)> {
    let home_dc_id = session.home_dc_id().map_err(session_error)?;
    let export_dc_id = export_dc_id_for_home_dc(home_dc_id);
    let mut export_option = session
        .dc_option(home_dc_id)
        .map_err(session_error)?
        .ok_or_else(|| {
            AppError::internal(format!(
                "Home DC option {home_dc_id} is missing from session"
            ))
        })?;
    export_option.id = export_dc_id;
    session
        .set_dc_option(&export_option)
        .await
        .map_err(session_error)?;

    let alias = ExportDcAlias {
        home_dc_id,
        export_dc_id,
    };
    Ok((alias.home_dc_id, alias.export_dc_id))
}

fn export_dc_id_for_home_dc(home_dc_id: i32) -> i32 {
    home_dc_id + EXPORT_DC_SHIFT
}

pub(super) async fn export_dc_invoke<R, Shifted, Home, ShiftedFuture, HomeFuture, OnFallback>(
    attempt: TakeoutAttempt,
    use_shifted: bool,
    shifted_invoke: Shifted,
    home_invoke: Home,
    on_fallback: OnFallback,
) -> AppResult<R>
where
    Shifted: FnOnce() -> ShiftedFuture,
    Home: FnOnce() -> HomeFuture,
    ShiftedFuture: Future<Output = Result<R, InvocationError>>,
    HomeFuture: Future<Output = Result<R, InvocationError>>,
    OnFallback: FnOnce(String),
{
    if use_shifted {
        match shifted_invoke().await {
            Ok(response) => return Ok(response),
            Err(error) if should_fallback_export_dc_error(&error) => {
                on_fallback(format!(
                    "Export DC {} failed with local transport error; falling back to home DC {}: {error}",
                    attempt.export_dc_id(),
                    attempt.home_dc_id()
                ));
            }
            Err(error) => return Err(AppError::network(error.to_string())),
        }
    }

    home_invoke()
        .await
        .map_err(|error| AppError::network(error.to_string()))
}

#[cfg(test)]
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
    let use_shifted = !*fallback_used;
    export_dc_invoke(
        TakeoutAttempt::new(alias.home_dc_id, alias.export_dc_id),
        use_shifted,
        shifted_invoke,
        home_invoke,
        |warning| {
            *fallback_used = true;
            warnings.push(warning);
        },
    )
    .await
}

pub(super) async fn finish_takeout_session(
    transport: &mut TakeoutTransport,
    takeout_id: i64,
    success: bool,
) -> AppResult<()> {
    let client = transport.client().clone();
    let attempt = transport.export_dc_attempt();
    let use_shifted = transport.export_dc_id().is_some();
    let request = tl::functions::InvokeWithTakeout {
        takeout_id,
        query: tl::functions::account::FinishTakeoutSession { success },
    };
    export_dc_invoke(
        attempt,
        use_shifted,
        || client.invoke_in_dc(attempt.export_dc_id(), &request),
        || client.invoke(&request),
        |warning| {
            transport.queue_fallback(TakeoutFallback::new(
                TakeoutFallbackKind::ExportDc,
                warning.clone(),
                Some(warning),
            ));
        },
    )
    .await
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use extractum_core::error::AppErrorKind;
    use grammers_mtsender::{InvocationError, RpcError};

    use super::super::{
        takeout_init_request_for_source_subtype,
        types::{TakeoutAttempt, TakeoutFallback, TakeoutFallbackKind},
        TAKEOUT_FILE_MAX_SIZE,
    };
    use super::{
        export_dc_id_for_home_dc, export_dc_invoke_with, should_fallback_export_dc_error,
        ExportDcAlias,
    };

    #[test]
    fn export_dc_id_applies_tdesktop_shift() {
        assert_eq!(export_dc_id_for_home_dc(2), 40_002);
    }

    #[test]
    fn export_dc_attempt_state_detects_first_fallback_transition() {
        let attempt = TakeoutAttempt::new(2, 40_002);
        assert_eq!(attempt.home_dc_id(), 2);
        assert_eq!(attempt.export_dc_id(), 40_002);

        let fallback = TakeoutFallback::new(
            TakeoutFallbackKind::ExportDc,
            "fallback message".to_string(),
            Some("fallback message".to_string()),
        );
        assert_eq!(fallback.kind(), TakeoutFallbackKind::ExportDc);
        assert_eq!(fallback.warning(), "fallback message");
        assert_eq!(fallback.provenance_message(), Some("fallback message"));
    }

    #[test]
    fn takeout_init_request_uses_source_subtype_flags_and_file_limit() {
        let group = takeout_init_request_for_source_subtype("group").expect("group flags");
        assert!(group.message_chats);
        assert!(!group.message_megagroups);
        assert!(!group.message_channels);
        assert!(group.files);
        assert_eq!(group.file_max_size, Some(TAKEOUT_FILE_MAX_SIZE));

        let supergroup =
            takeout_init_request_for_source_subtype("supergroup").expect("supergroup flags");
        assert!(!supergroup.message_chats);
        assert!(supergroup.message_megagroups);
        assert!(!supergroup.message_channels);

        let channel = takeout_init_request_for_source_subtype("channel").expect("channel flags");
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
        assert!(!should_fallback_export_dc_error(&InvocationError::Session(
            Box::new(std::io::Error::other("session failure")),
        )));
    }
}
