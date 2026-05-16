use std::sync::Arc;

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

pub(crate) fn takeout_init_request_for_source_kind(
    telegram_source_subtype: &str,
) -> AppResult<tl::functions::account::InitTakeoutSession> {
    let (message_chats, message_megagroups, message_channels) = match telegram_source_subtype {
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
    if !*fallback_used {
        match client.invoke_in_dc(alias.export_dc_id, request).await {
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

    client
        .invoke(request)
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
        export_dc_id_for_home_dc, should_fallback_export_dc_error,
        takeout_init_request_for_source_kind, TAKEOUT_FILE_MAX_SIZE,
    };
    use crate::sources::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
    use grammers_mtsender::{InvocationError, RpcError};

    #[test]
    fn export_dc_id_applies_tdesktop_shift() {
        assert_eq!(export_dc_id_for_home_dc(2), 40_002);
    }

    #[test]
    fn takeout_init_request_uses_source_kind_flags_and_file_limit() {
        let group = takeout_init_request_for_source_kind(TELEGRAM_KIND_GROUP).expect("group flags");
        assert!(group.message_chats);
        assert!(!group.message_megagroups);
        assert!(!group.message_channels);
        assert!(group.files);
        assert_eq!(group.file_max_size, Some(TAKEOUT_FILE_MAX_SIZE));

        let supergroup = takeout_init_request_for_source_kind(TELEGRAM_KIND_SUPERGROUP)
            .expect("supergroup flags");
        assert!(!supergroup.message_chats);
        assert!(supergroup.message_megagroups);
        assert!(!supergroup.message_channels);

        let channel =
            takeout_init_request_for_source_kind(TELEGRAM_KIND_CHANNEL).expect("channel flags");
        assert!(!channel.message_chats);
        assert!(!channel.message_megagroups);
        assert!(channel.message_channels);
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
