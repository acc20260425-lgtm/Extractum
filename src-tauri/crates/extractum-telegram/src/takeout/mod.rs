mod export_dc;
mod forum_topics;
mod operations;
mod pagination;
mod raw_parse;
mod transport;
mod types;

use extractum_core::error::AppResult;
use grammers_client::{tl, Client};

use super::{
    dto::{ForumTopicSnapshot, PeerDescriptor},
    session::TelegramSession,
};

const TAKEOUT_FILE_MAX_SIZE: i64 = 8 * 1024 * 1024;

fn takeout_init_request_for_source_subtype(
    source_subtype: &str,
) -> AppResult<tl::functions::account::InitTakeoutSession> {
    let (message_chats, message_megagroups, message_channels) = match source_subtype {
        "group" => (true, false, false),
        "supergroup" => (false, true, false),
        "channel" => (false, false, true),
        other => {
            return Err(extractum_core::error::AppError::validation(format!(
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

pub use self::transport::TakeoutTransport;
pub use self::types::{
    MessageRange, TakeoutAttempt, TakeoutCount, TakeoutFallback, TakeoutFallbackKind,
    TakeoutMessage, TakeoutPage, TakeoutPeer,
};

pub(super) async fn takeout_self_check(client: &Client) -> AppResult<()> {
    operations::takeout_self_check(client).await
}

pub(super) async fn prepare_takeout(
    client: &Client,
    session: &TelegramSession,
) -> AppResult<TakeoutTransport> {
    let session = session.clone_memory_session();
    let (home_dc_id, export_dc_id) = export_dc::prepare_export_dc_alias(&session).await?;
    Ok(TakeoutTransport::new(
        client.clone(),
        session,
        home_dc_id,
        export_dc_id,
    ))
}

pub(super) async fn takeout_forum_topics(
    client: &Client,
    peer: &PeerDescriptor,
) -> AppResult<Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>> {
    forum_topics::takeout_forum_topics(client, peer).await
}

#[cfg(feature = "app-test-support")]
pub fn fallback_fixture(
    kind: TakeoutFallbackKind,
    warning: &str,
    provenance_message: Option<&str>,
) -> TakeoutFallback {
    TakeoutFallback::new(
        kind,
        warning.to_string(),
        provenance_message.map(str::to_string),
    )
}

#[cfg(feature = "app-test-support")]
pub fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {
    TakeoutAttempt::new(home_dc_id, export_dc_id)
}
