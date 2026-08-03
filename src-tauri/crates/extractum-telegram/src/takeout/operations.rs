use std::{future::Future, pin::Pin};

use extractum_core::error::{AppError, AppResult};
use grammers_client::{tl, Client};

use super::super::error::is_channel_private_error;
use super::{
    export_dc::{export_dc_invoke, finish_takeout_session},
    pagination::{
        parse_takeout_page, select_history_splits, takeout_page_request, TakeoutPaginationCursor,
        TakeoutPaginationProfile,
    },
    raw_parse::messages_response_count,
    takeout_init_request_for_source_subtype,
    transport::TakeoutTransport,
    types::{
        MessageRange, TakeoutCount, TakeoutFallback, TakeoutFallbackKind, TakeoutPage, TakeoutPeer,
    },
};

const ONLY_MY_WARNING: &str =
    "Channel history is private; falling back to messages.search(from_id=self).";
const ONLY_MY_PROVENANCE: &str =
    "Channel history is private; importing only messages visible through from_id=self fallback.";

#[derive(Clone, Debug, PartialEq)]
enum RawCall {
    SelfCheck,
    Init {
        request: tl::functions::account::InitTakeoutSession,
    },
    MessageRanges {
        takeout_id: i64,
    },
    Validate {
        takeout_id: i64,
        peer: TakeoutPeer,
        source_subtype: String,
    },
    DetectMigration {
        takeout_id: i64,
        peer: TakeoutPeer,
    },
    RevalidateMigration {
        takeout_id: i64,
        peer: TakeoutPeer,
    },
    History {
        takeout_id: i64,
        peer: TakeoutPeer,
        range: MessageRange,
        offset_id: i32,
        add_offset: i32,
        limit: i32,
        search_my: bool,
    },
    Finish {
        takeout_id: i64,
        success: bool,
    },
}

enum RawFixture {
    Unit,
    Started { takeout_id: i64 },
    Ranges(Vec<tl::enums::MessageRange>),
    Migration(Option<(i64, tl::enums::Peer)>),
    Messages(tl::enums::messages::Messages),
}

type BackendFuture<'a> = Pin<Box<dyn Future<Output = AppResult<RawFixture>> + Send + 'a>>;

trait OperationsBackend {
    fn invoke(&mut self, call: RawCall) -> BackendFuture<'_>;
    fn queue_fallback(&mut self, fallback: TakeoutFallback);
}

struct RealOperationsBackend<'a> {
    transport: &'a mut TakeoutTransport,
}

impl OperationsBackend for RealOperationsBackend<'_> {
    fn invoke(&mut self, call: RawCall) -> BackendFuture<'_> {
        Box::pin(async move {
            match call {
                RawCall::SelfCheck => {
                    takeout_self_check(self.transport.client()).await?;
                    Ok(RawFixture::Unit)
                }
                RawCall::Init { request } => {
                    let takeout = invoke_export_dc(self.transport, &request).await?;
                    let tl::enums::account::Takeout::Takeout(takeout) = takeout;
                    Ok(RawFixture::Started {
                        takeout_id: takeout.id,
                    })
                }
                RawCall::MessageRanges { takeout_id } => {
                    let response = invoke_export_dc(
                        self.transport,
                        &tl::functions::InvokeWithTakeout {
                            takeout_id,
                            query: tl::functions::messages::GetSplitRanges {},
                        },
                    )
                    .await?;
                    Ok(RawFixture::Ranges(response))
                }
                RawCall::Validate {
                    takeout_id,
                    peer,
                    source_subtype,
                } => {
                    match source_subtype.as_str() {
                        "channel" | "supergroup" => {
                            let channel = input_channel(&peer)?;
                            invoke_export_dc(
                                self.transport,
                                &tl::functions::InvokeWithTakeout {
                                    takeout_id,
                                    query: tl::functions::channels::GetChannels {
                                        id: vec![channel],
                                    },
                                },
                            )
                            .await?;
                        }
                        "group" => {
                            invoke_export_dc(
                                self.transport,
                                &tl::functions::InvokeWithTakeout {
                                    takeout_id,
                                    query: tl::functions::messages::GetChats {
                                        id: vec![peer.peer_id()],
                                    },
                                },
                            )
                            .await?;
                        }
                        other => return Err(unsupported_source_subtype(other)),
                    }
                    Ok(RawFixture::Unit)
                }
                RawCall::DetectMigration { takeout_id, peer }
                | RawCall::RevalidateMigration { takeout_id, peer } => {
                    let response = invoke_export_dc(
                        self.transport,
                        &tl::functions::InvokeWithTakeout {
                            takeout_id,
                            query: tl::functions::channels::GetFullChannel {
                                channel: input_channel(&peer)?,
                            },
                        },
                    )
                    .await?;
                    Ok(RawFixture::Migration(migrated_chat_from_full(response)))
                }
                RawCall::History {
                    takeout_id,
                    peer,
                    range,
                    offset_id,
                    add_offset,
                    limit,
                    search_my,
                } => {
                    let input_peer = input_peer(&peer)?;
                    let range = raw_range(&range);
                    let response = if search_my {
                        invoke_export_dc(
                            self.transport,
                            &tl::functions::InvokeWithTakeout {
                                takeout_id,
                                query: tl::functions::InvokeWithMessagesRange {
                                    range,
                                    query: tl::functions::messages::Search {
                                        peer: input_peer,
                                        q: String::new(),
                                        from_id: Some(tl::enums::InputPeer::PeerSelf),
                                        saved_peer_id: None,
                                        saved_reaction: None,
                                        top_msg_id: None,
                                        filter: tl::enums::MessagesFilter::InputMessagesFilterEmpty,
                                        min_date: 0,
                                        max_date: 0,
                                        offset_id,
                                        add_offset,
                                        limit,
                                        max_id: 0,
                                        min_id: 0,
                                        hash: 0,
                                    },
                                },
                            },
                        )
                        .await?
                    } else {
                        invoke_export_dc(
                            self.transport,
                            &tl::functions::InvokeWithTakeout {
                                takeout_id,
                                query: tl::functions::InvokeWithMessagesRange {
                                    range,
                                    query: tl::functions::messages::GetHistory {
                                        peer: input_peer,
                                        offset_id,
                                        offset_date: 0,
                                        add_offset,
                                        limit,
                                        max_id: 0,
                                        min_id: 0,
                                        hash: 0,
                                    },
                                },
                            },
                        )
                        .await?
                    };
                    Ok(RawFixture::Messages(response))
                }
                RawCall::Finish {
                    takeout_id,
                    success,
                } => {
                    finish_takeout_session(self.transport, takeout_id, success).await?;
                    Ok(RawFixture::Unit)
                }
            }
        })
    }

    fn queue_fallback(&mut self, fallback: TakeoutFallback) {
        self.transport.queue_fallback(fallback);
    }
}

pub(super) async fn takeout_self_check(client: &Client) -> AppResult<()> {
    client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![tl::enums::InputUser::UserSelf],
        })
        .await
        .map_err(|error| AppError::network(format!("Telegram self check failed: {error}")))?;
    Ok(())
}

impl TakeoutTransport {
    pub async fn init(&mut self, source_subtype: &str) -> AppResult<i64> {
        init_with_backend(
            &mut RealOperationsBackend { transport: self },
            source_subtype,
        )
        .await
    }

    pub async fn message_ranges(
        &mut self,
        takeout_id: i64,
        source_subtype: &str,
    ) -> AppResult<(i64, Vec<MessageRange>)> {
        message_ranges_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            source_subtype,
        )
        .await
    }

    pub async fn validate_peer(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        source_subtype: &str,
    ) -> AppResult<()> {
        validate_peer_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            peer,
            source_subtype,
        )
        .await
    }

    pub async fn detect_supergroup_migration(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        source_subtype: &str,
    ) -> AppResult<Option<i64>> {
        detect_migration_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            peer,
            source_subtype,
        )
        .await
    }

    pub async fn revalidate_migrated_peer(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
    ) -> AppResult<Option<(i64, TakeoutPeer)>> {
        revalidate_migration_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            peer,
        )
        .await
    }

    pub async fn history_count(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        source_subtype: &str,
    ) -> AppResult<TakeoutCount> {
        history_count_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            peer,
            range,
            source_subtype,
            false,
        )
        .await
    }

    pub async fn search_my_history_count(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
    ) -> AppResult<TakeoutCount> {
        history_count_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            peer,
            range,
            peer.source_subtype(),
            true,
        )
        .await
    }

    pub async fn history_page(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        count: &TakeoutCount,
        previous: Option<&TakeoutPage>,
    ) -> AppResult<TakeoutPage> {
        history_page_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            peer,
            range,
            count,
            previous,
            false,
        )
        .await
    }

    pub async fn search_my_history_page(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        count: &TakeoutCount,
        previous: Option<&TakeoutPage>,
    ) -> AppResult<TakeoutPage> {
        history_page_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            peer,
            range,
            count,
            previous,
            true,
        )
        .await
    }

    pub async fn finish(&mut self, takeout_id: i64, success: bool) -> AppResult<()> {
        finish_with_backend(
            &mut RealOperationsBackend { transport: self },
            takeout_id,
            success,
        )
        .await
    }
}

async fn invoke_export_dc<R: tl::RemoteCall>(
    transport: &mut TakeoutTransport,
    request: &R,
) -> AppResult<R::Return> {
    let client = transport.client().clone();
    let attempt = transport.export_dc_attempt();
    let use_shifted = transport.export_dc_id().is_some();
    export_dc_invoke(
        attempt,
        use_shifted,
        || client.invoke_in_dc(attempt.export_dc_id(), request),
        || client.invoke(request),
        |warning| {
            transport.queue_fallback(TakeoutFallback::new(
                TakeoutFallbackKind::ExportDc,
                warning.clone(),
                Some(warning),
            ));
        },
    )
    .await
}

async fn start_takeout_with_backend<B: OperationsBackend>(
    backend: &mut B,
    source_subtype: &str,
) -> AppResult<(i64, i64, Vec<MessageRange>)> {
    expect_unit(backend.invoke(RawCall::SelfCheck).await?)?;
    let takeout_id = init_with_backend(backend, source_subtype).await?;
    let (split_count, ranges) =
        message_ranges_with_backend(backend, takeout_id, source_subtype).await?;
    Ok((takeout_id, split_count, ranges))
}

async fn init_with_backend<B: OperationsBackend>(
    backend: &mut B,
    source_subtype: &str,
) -> AppResult<i64> {
    match backend
        .invoke(RawCall::Init {
            request: takeout_init_request_for_source_subtype(source_subtype)?,
        })
        .await?
    {
        RawFixture::Started { takeout_id } => Ok(takeout_id),
        _ => Err(unexpected_response("Takeout init")),
    }
}

async fn message_ranges_with_backend<B: OperationsBackend>(
    backend: &mut B,
    takeout_id: i64,
    source_subtype: &str,
) -> AppResult<(i64, Vec<MessageRange>)> {
    let response = backend
        .invoke(RawCall::MessageRanges { takeout_id })
        .await?;
    let RawFixture::Ranges(ranges) = response else {
        return Err(unexpected_response("Takeout split ranges"));
    };
    let split_count = ranges.len() as i64;
    Ok((split_count, select_history_splits(source_subtype, ranges)?))
}

async fn validate_peer_with_backend<B: OperationsBackend>(
    backend: &mut B,
    takeout_id: i64,
    peer: &TakeoutPeer,
    source_subtype: &str,
) -> AppResult<()> {
    let result = backend
        .invoke(RawCall::Validate {
            takeout_id,
            peer: peer.clone(),
            source_subtype: source_subtype.to_string(),
        })
        .await;
    match result {
        Ok(response) => expect_unit(response),
        Err(error)
            if supports_only_my_messages_fallback(source_subtype)
                && is_channel_private_error(&error) =>
        {
            queue_only_my_fallback(backend);
            Ok(())
        }
        Err(error) => Err(error),
    }
}

async fn detect_migration_with_backend<B: OperationsBackend>(
    backend: &mut B,
    takeout_id: i64,
    peer: &TakeoutPeer,
    source_subtype: &str,
) -> AppResult<Option<i64>> {
    if source_subtype != "supergroup" {
        return Ok(None);
    }
    let response = backend
        .invoke(RawCall::DetectMigration {
            takeout_id,
            peer: peer.clone(),
        })
        .await?;
    let RawFixture::Migration(migration) = response else {
        return Err(unexpected_response("Takeout migration probe"));
    };
    Ok(migration.map(|(chat_id, _)| chat_id))
}

async fn revalidate_migration_with_backend<B: OperationsBackend>(
    backend: &mut B,
    takeout_id: i64,
    peer: &TakeoutPeer,
) -> AppResult<Option<(i64, TakeoutPeer)>> {
    let response = backend
        .invoke(RawCall::RevalidateMigration {
            takeout_id,
            peer: peer.clone(),
        })
        .await?;
    let RawFixture::Migration(migration) = response else {
        return Err(unexpected_response("Takeout migration revalidation"));
    };
    migration.map(migrated_peer).transpose()
}

async fn migration_probe_with_backend<B: OperationsBackend>(
    backend: &mut B,
    takeout_id: i64,
    peer: &TakeoutPeer,
) -> AppResult<Option<(i64, TakeoutPeer)>> {
    let Some(expected_chat_id) =
        detect_migration_with_backend(backend, takeout_id, peer, "supergroup").await?
    else {
        return Ok(None);
    };
    let revalidated = revalidate_migration_with_backend(backend, takeout_id, peer).await?;
    Ok(revalidated.filter(|(chat_id, _)| *chat_id == expected_chat_id))
}

async fn history_count_with_backend<B: OperationsBackend>(
    backend: &mut B,
    takeout_id: i64,
    peer: &TakeoutPeer,
    range: &MessageRange,
    source_subtype: &str,
    search_my: bool,
) -> AppResult<TakeoutCount> {
    let result = backend
        .invoke(RawCall::History {
            takeout_id,
            peer: peer.clone(),
            range: range.clone(),
            offset_id: 0,
            add_offset: 0,
            limit: 1,
            search_my,
        })
        .await;
    let response = match result {
        Ok(RawFixture::Messages(response)) => response,
        Ok(_) => return Err(unexpected_response("Takeout history count")),
        Err(error)
            if !search_my
                && supports_only_my_messages_fallback(source_subtype)
                && is_channel_private_error(&error) =>
        {
            queue_only_my_fallback(backend);
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    Ok(TakeoutCount::new(
        messages_response_count(response)?,
        search_my,
    ))
}

async fn history_page_with_backend<B: OperationsBackend>(
    backend: &mut B,
    takeout_id: i64,
    peer: &TakeoutPeer,
    range: &MessageRange,
    count: &TakeoutCount,
    previous: Option<&TakeoutPage>,
    search_my: bool,
) -> AppResult<TakeoutPage> {
    let (profile, cursor, page_index) =
        previous
            .map(TakeoutPage::pagination_state)
            .unwrap_or_else(|| {
                let profile = TakeoutPaginationProfile::TDesktop;
                (profile, TakeoutPaginationCursor::new(profile, range), 0)
            });
    let request = takeout_page_request(cursor);
    let result = backend
        .invoke(RawCall::History {
            takeout_id,
            peer: peer.clone(),
            range: range.clone(),
            offset_id: request.offset_id,
            add_offset: request.add_offset,
            limit: request.limit,
            search_my,
        })
        .await;
    let response = match result {
        Ok(RawFixture::Messages(response)) => response,
        Ok(_) => return Err(unexpected_response("Takeout history page")),
        Err(error)
            if !search_my
                && supports_only_my_messages_fallback(peer.source_subtype())
                && is_channel_private_error(&error) =>
        {
            queue_only_my_fallback(backend);
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    parse_takeout_page(
        response,
        profile,
        cursor,
        range,
        count.count(),
        page_index,
        search_my,
    )
}

async fn finish_with_backend<B: OperationsBackend>(
    backend: &mut B,
    takeout_id: i64,
    success: bool,
) -> AppResult<()> {
    expect_unit(
        backend
            .invoke(RawCall::Finish {
                takeout_id,
                success,
            })
            .await?,
    )
}

fn queue_only_my_fallback(backend: &mut impl OperationsBackend) {
    backend.queue_fallback(TakeoutFallback::new(
        TakeoutFallbackKind::OnlyMyMessages,
        ONLY_MY_WARNING.to_string(),
        Some(ONLY_MY_PROVENANCE.to_string()),
    ));
}

fn supports_only_my_messages_fallback(source_subtype: &str) -> bool {
    matches!(source_subtype, "channel" | "supergroup")
}

fn input_peer(peer: &TakeoutPeer) -> AppResult<tl::enums::InputPeer> {
    match peer.peer_kind() {
        "channel" => Ok(tl::types::InputPeerChannel {
            channel_id: peer.peer_id(),
            access_hash: peer.access_hash().ok_or_else(|| {
                AppError::validation(format!(
                    "Telegram {} {} is missing an access hash",
                    peer.source_subtype(),
                    peer.peer_id()
                ))
            })?,
        }
        .into()),
        "chat" => Ok(tl::types::InputPeerChat {
            chat_id: peer.peer_id(),
        }
        .into()),
        other => Err(AppError::validation(format!(
            "Unsupported Telegram Takeout peer kind '{other}'"
        ))),
    }
}

fn input_channel(peer: &TakeoutPeer) -> AppResult<tl::enums::InputChannel> {
    match input_peer(peer)? {
        tl::enums::InputPeer::Channel(channel) => Ok(tl::types::InputChannel {
            channel_id: channel.channel_id,
            access_hash: channel.access_hash,
        }
        .into()),
        _ => Err(AppError::validation(
            "Telegram Takeout channel operation requires a channel peer",
        )),
    }
}

fn raw_range(range: &MessageRange) -> tl::enums::MessageRange {
    tl::types::MessageRange {
        min_id: range.min_id(),
        max_id: range.max_id(),
    }
    .into()
}

fn migrated_chat_from_full(
    response: tl::enums::messages::ChatFull,
) -> Option<(i64, tl::enums::Peer)> {
    let tl::enums::messages::ChatFull::Full(response) = response;
    let tl::enums::ChatFull::ChannelFull(full) = response.full_chat else {
        return None;
    };
    full.migrated_from_chat_id
        .map(|chat_id| (chat_id, tl::types::PeerChat { chat_id }.into()))
}

fn migrated_peer((chat_id, peer): (i64, tl::enums::Peer)) -> AppResult<(i64, TakeoutPeer)> {
    match peer {
        tl::enums::Peer::Chat(peer) if peer.chat_id == chat_id => Ok((
            chat_id,
            TakeoutPeer::new("group".to_string(), "chat", chat_id, None),
        )),
        _ => Err(AppError::validation(
            "Telegram migrated Takeout peer did not resolve to the expected chat",
        )),
    }
}

fn expect_unit(response: RawFixture) -> AppResult<()> {
    match response {
        RawFixture::Unit => Ok(()),
        _ => Err(unexpected_response("Takeout unit operation")),
    }
}

fn unexpected_response(operation: &str) -> AppError {
    AppError::internal(format!("Unexpected raw response for {operation}"))
}

fn unsupported_source_subtype(source_subtype: &str) -> AppError {
    AppError::validation(format!(
        "Unsupported Telegram source_subtype '{source_subtype}'"
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use extractum_core::error::{AppError, AppErrorKind, AppResult};
    use grammers_client::tl;

    use super::super::{types::TakeoutMessage, TAKEOUT_FILE_MAX_SIZE};
    use super::*;

    struct FakeOperationsBackend {
        calls: Vec<RawCall>,
        responses: VecDeque<AppResult<RawFixture>>,
        fallbacks: Vec<TakeoutFallback>,
    }

    impl FakeOperationsBackend {
        fn new(responses: impl IntoIterator<Item = AppResult<RawFixture>>) -> Self {
            Self {
                calls: Vec::new(),
                responses: responses.into_iter().collect(),
                fallbacks: Vec::new(),
            }
        }

        fn drain_fallbacks(&mut self) -> Vec<TakeoutFallback> {
            std::mem::take(&mut self.fallbacks)
        }
    }

    #[tokio::test]
    async fn checkpoint_six_lifecycle_preserves_fallback_cancellation_and_finalize_order() {
        fn check_cancellation(
            lifecycle: &mut Vec<String>,
            cancellation_checks: &mut usize,
            point: &'static str,
        ) -> bool {
            *cancellation_checks += 1;
            lifecycle.push(format!("cancel:{point}"));
            false
        }

        let peer = TakeoutPeer::new("supergroup".to_string(), "channel", 12_345, Some(99));
        let mut backend = FakeOperationsBackend::new([
            Err(AppError::network("Rpc error 400: CHANNEL_PRIVATE")),
            Ok(RawFixture::Migration(None)),
            Ok(RawFixture::Ranges(vec![raw_range(10, 500)])),
            Err(AppError::network("Rpc error 400: CHANNEL_PRIVATE")),
            Ok(RawFixture::Messages(messages_response(vec![42]))),
            Ok(RawFixture::Messages(messages_response(Vec::new()))),
            Ok(RawFixture::Messages(messages_response(vec![42]))),
            Ok(RawFixture::Unit),
        ]);
        let mut lifecycle = Vec::new();
        let mut cancellation_checks = 0;

        assert!(!check_cancellation(
            &mut lifecycle,
            &mut cancellation_checks,
            "before_validate"
        ));
        lifecycle.push("validate".to_string());
        validate_peer_with_backend(&mut backend, 77, &peer, "supergroup")
            .await
            .expect("private validation falls back");
        for fallback in backend.drain_fallbacks() {
            lifecycle.push(format!("fallback:{:?}", fallback.kind()));
            lifecycle.push(format!(
                "provenance:{}",
                fallback.provenance_message().expect("fallback provenance")
            ));
        }

        lifecycle.push("migration".to_string());
        let migration = detect_migration_with_backend(&mut backend, 77, &peer, "supergroup")
            .await
            .expect("migration probe");
        assert_eq!(migration, None);

        lifecycle.push("split".to_string());
        let (split_count, ranges) = message_ranges_with_backend(&mut backend, 77, "supergroup")
            .await
            .expect("split selection");
        assert_eq!(split_count, 1);
        assert_eq!(ranges, vec![MessageRange::new(10, 500)]);
        let range = &ranges[0];

        assert!(!check_cancellation(
            &mut lifecycle,
            &mut cancellation_checks,
            "before_count"
        ));
        lifecycle.push("count:history".to_string());
        let history_count =
            history_count_with_backend(&mut backend, 77, &peer, range, "supergroup", false).await;
        assert!(history_count.is_err());
        for fallback in backend.drain_fallbacks() {
            lifecycle.push(format!("fallback:{:?}", fallback.kind()));
            lifecycle.push(format!(
                "provenance:{}",
                fallback.provenance_message().expect("fallback provenance")
            ));
        }
        lifecycle.push("count:search".to_string());
        let count = history_count_with_backend(&mut backend, 77, &peer, range, "supergroup", true)
            .await
            .expect("private-channel search count");
        assert_eq!(count, TakeoutCount::new(1, true));

        assert!(!check_cancellation(
            &mut lifecycle,
            &mut cancellation_checks,
            "before_import"
        ));
        lifecycle.push("import:tdesktop".to_string());
        let mut first_page =
            history_page_with_backend(&mut backend, 77, &peer, range, &count, None, true)
                .await
                .expect("TDesktop page");
        assert!(first_page.take_messages().is_empty());
        let pagination_warning = first_page
            .take_pagination_fallback_warning()
            .expect("descending fallback warning");
        lifecycle.push("fallback:Descending".to_string());
        lifecycle.push(format!("provenance:{pagination_warning}"));

        assert!(!check_cancellation(
            &mut lifecycle,
            &mut cancellation_checks,
            "before_import"
        ));
        lifecycle.push("import:descending".to_string());
        let mut descending_page = history_page_with_backend(
            &mut backend,
            77,
            &peer,
            range,
            &count,
            Some(&first_page),
            true,
        )
        .await
        .expect("descending page");
        assert_eq!(message_ids(&descending_page.take_messages()), vec![42]);

        assert!(!check_cancellation(
            &mut lifecycle,
            &mut cancellation_checks,
            "before_finish"
        ));
        lifecycle.push("finish".to_string());
        finish_with_backend(&mut backend, 77, true)
            .await
            .expect("finish Takeout");
        lifecycle.push("finalize:completed".to_string());

        assert_eq!(cancellation_checks, 5);
        assert_eq!(
            lifecycle,
            vec![
                "cancel:before_validate",
                "validate",
                "fallback:OnlyMyMessages",
                "provenance:Channel history is private; importing only messages visible through from_id=self fallback.",
                "migration",
                "split",
                "cancel:before_count",
                "count:history",
                "fallback:OnlyMyMessages",
                "provenance:Channel history is private; importing only messages visible through from_id=self fallback.",
                "count:search",
                "cancel:before_import",
                "import:tdesktop",
                "fallback:Descending",
                "provenance:TDesktop Takeout pagination returned an empty first page for split 10..500; retrying this split with Extractum descending fallback.",
                "cancel:before_import",
                "import:descending",
                "cancel:before_finish",
                "finish",
                "finalize:completed",
            ]
        );
        assert_eq!(
            backend.calls,
            vec![
                RawCall::Validate {
                    takeout_id: 77,
                    peer: peer.clone(),
                    source_subtype: "supergroup".to_string(),
                },
                RawCall::DetectMigration {
                    takeout_id: 77,
                    peer: peer.clone(),
                },
                RawCall::MessageRanges { takeout_id: 77 },
                RawCall::History {
                    takeout_id: 77,
                    peer: peer.clone(),
                    range: range.clone(),
                    offset_id: 0,
                    add_offset: 0,
                    limit: 1,
                    search_my: false,
                },
                RawCall::History {
                    takeout_id: 77,
                    peer: peer.clone(),
                    range: range.clone(),
                    offset_id: 0,
                    add_offset: 0,
                    limit: 1,
                    search_my: true,
                },
                RawCall::History {
                    takeout_id: 77,
                    peer: peer.clone(),
                    range: range.clone(),
                    offset_id: 1,
                    add_offset: -100,
                    limit: 100,
                    search_my: true,
                },
                RawCall::History {
                    takeout_id: 77,
                    peer,
                    range: range.clone(),
                    offset_id: 500,
                    add_offset: 0,
                    limit: 100,
                    search_my: true,
                },
                RawCall::Finish {
                    takeout_id: 77,
                    success: true,
                },
            ]
        );
    }

    impl OperationsBackend for FakeOperationsBackend {
        fn invoke(&mut self, call: RawCall) -> BackendFuture<'_> {
            Box::pin(async move {
                self.calls.push(call);
                self.responses.pop_front().expect("fake Takeout response")
            })
        }

        fn queue_fallback(&mut self, fallback: TakeoutFallback) {
            self.fallbacks.push(fallback);
        }
    }

    #[tokio::test]
    async fn start_takeout_returns_owned_session_and_selected_ranges() {
        let mut backend = FakeOperationsBackend::new([
            Ok(RawFixture::Unit),
            Ok(RawFixture::Started { takeout_id: 77 }),
            Ok(RawFixture::Ranges(vec![
                raw_range(1, 100),
                raw_range(101, 200),
            ])),
        ]);

        let (takeout_id, raw_split_count, ranges) =
            start_takeout_with_backend(&mut backend, "supergroup")
                .await
                .expect("start Takeout");

        assert_eq!(takeout_id, 77);
        assert_eq!(raw_split_count, 2);
        assert_eq!(ranges, vec![MessageRange::new(101, 200)]);
        let init_request =
            takeout_init_request_for_source_subtype("supergroup").expect("init request");
        assert!(!init_request.contacts);
        assert!(!init_request.message_users);
        assert!(!init_request.message_chats);
        assert!(init_request.message_megagroups);
        assert!(!init_request.message_channels);
        assert!(init_request.files);
        assert_eq!(init_request.file_max_size, Some(TAKEOUT_FILE_MAX_SIZE));
        assert_eq!(
            backend.calls,
            vec![
                RawCall::SelfCheck,
                RawCall::Init {
                    request: init_request,
                },
                RawCall::MessageRanges { takeout_id: 77 },
            ]
        );
    }

    #[tokio::test]
    async fn migration_probe_and_revalidation_return_owned_chat_identity() {
        let current_peer = TakeoutPeer::new("supergroup".to_string(), "channel", 12_345, Some(99));
        let mut backend = FakeOperationsBackend::new([
            Ok(RawFixture::Migration(Some((
                777,
                tl::types::PeerChat { chat_id: 777 }.into(),
            )))),
            Ok(RawFixture::Migration(Some((
                777,
                tl::types::PeerChat { chat_id: 777 }.into(),
            )))),
        ]);

        let migrated = migration_probe_with_backend(&mut backend, 77, &current_peer)
            .await
            .expect("migration operations")
            .expect("migrated peer");

        assert_eq!(migrated.0, 777);
        assert_eq!(migrated.1.peer_kind(), "chat");
        assert_eq!(migrated.1.peer_id(), 777);
        assert_eq!(
            backend.calls,
            vec![
                RawCall::DetectMigration {
                    takeout_id: 77,
                    peer: current_peer.clone(),
                },
                RawCall::RevalidateMigration {
                    takeout_id: 77,
                    peer: current_peer,
                },
            ]
        );
    }

    #[tokio::test]
    async fn history_count_preserves_channel_private_fallback_outcome() {
        let peer = TakeoutPeer::new("channel".to_string(), "channel", 12_345, Some(99));
        let range = MessageRange::new(1, 200);

        let mut validation_backend =
            FakeOperationsBackend::new([Err(AppError::network("Rpc error 400: CHANNEL_PRIVATE"))]);
        validate_peer_with_backend(&mut validation_backend, 77, &peer, "channel")
            .await
            .expect("channel-private validation is metadata-only");
        assert_eq!(validation_backend.fallbacks.len(), 1);
        assert_eq!(
            validation_backend.fallbacks[0].kind(),
            TakeoutFallbackKind::OnlyMyMessages
        );
        assert_eq!(
            validation_backend.calls,
            vec![RawCall::Validate {
                takeout_id: 77,
                peer: peer.clone(),
                source_subtype: "channel".to_string(),
            }]
        );

        let mut private_backend = FakeOperationsBackend::new([
            Err(AppError::network("Rpc error 400: CHANNEL_PRIVATE")),
            Ok(RawFixture::Messages(messages_response(vec![42]))),
        ]);

        let private_result =
            history_count_with_backend(&mut private_backend, 77, &peer, &range, "channel", false)
                .await;
        assert!(private_result.is_err());
        let fallbacks = &private_backend.fallbacks;
        assert_eq!(fallbacks.len(), 1);
        assert_eq!(fallbacks[0].kind(), TakeoutFallbackKind::OnlyMyMessages);
        assert_eq!(
            fallbacks[0].warning(),
            "Channel history is private; falling back to messages.search(from_id=self)."
        );
        assert_eq!(
            fallbacks[0].provenance_message(),
            Some(
                "Channel history is private; importing only messages visible through from_id=self fallback."
            )
        );
        let only_my_count =
            history_count_with_backend(&mut private_backend, 77, &peer, &range, "channel", true)
                .await
                .expect("only-my count continuation");
        assert_eq!(only_my_count, TakeoutCount::new(1, true));
        assert_eq!(
            private_backend.calls,
            vec![
                RawCall::History {
                    takeout_id: 77,
                    peer: peer.clone(),
                    range: range.clone(),
                    offset_id: 0,
                    add_offset: 0,
                    limit: 1,
                    search_my: false,
                },
                RawCall::History {
                    takeout_id: 77,
                    peer: peer.clone(),
                    range: range.clone(),
                    offset_id: 0,
                    add_offset: 0,
                    limit: 1,
                    search_my: true,
                },
            ]
        );

        let mut not_modified_backend = FakeOperationsBackend::new([Ok(RawFixture::Messages(
            tl::types::messages::MessagesNotModified { count: 0 }.into(),
        ))]);
        let result = history_count_with_backend(
            &mut not_modified_backend,
            77,
            &peer,
            &range,
            "channel",
            false,
        )
        .await;
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("messagesNotModified must be rejected"),
        };
        assert_eq!(error.kind, AppErrorKind::Network);
        assert_eq!(
            error.message,
            "Telegram returned messagesNotModified for Takeout history count probe"
        );
        assert!(not_modified_backend.fallbacks.is_empty());
        assert_eq!(
            not_modified_backend.calls,
            vec![RawCall::History {
                takeout_id: 77,
                peer: peer.clone(),
                range: range.clone(),
                offset_id: 0,
                add_offset: 0,
                limit: 1,
                search_my: false,
            }]
        );
    }

    #[tokio::test]
    async fn history_page_and_search_return_owned_takeout_messages() {
        let peer = TakeoutPeer::new("channel".to_string(), "channel", 12_345, Some(99));
        let range = MessageRange::new(1, 300);
        let count = TakeoutCount::new(2, false);
        let mut backend = FakeOperationsBackend::new([
            Ok(RawFixture::Messages(messages_response(vec![200, 150]))),
            Ok(RawFixture::Messages(messages_response(vec![250]))),
        ]);

        let mut history =
            history_page_with_backend(&mut backend, 77, &peer, &range, &count, None, false)
                .await
                .expect("history page");
        assert_eq!(message_ids(&history.take_messages()), vec![150, 200]);
        assert!(!history.only_my_messages());

        let only_my_count = TakeoutCount::new(1, true);
        let mut search = history_page_with_backend(
            &mut backend,
            77,
            &peer,
            &range,
            &only_my_count,
            Some(&history),
            true,
        )
        .await
        .expect("search page");
        assert_eq!(message_ids(&search.take_messages()), vec![250]);
        assert!(search.only_my_messages());
        assert_eq!(
            backend.calls,
            vec![
                RawCall::History {
                    takeout_id: 77,
                    peer: peer.clone(),
                    range: range.clone(),
                    offset_id: 1,
                    add_offset: -100,
                    limit: 100,
                    search_my: false,
                },
                RawCall::History {
                    takeout_id: 77,
                    peer,
                    range,
                    offset_id: 201,
                    add_offset: -100,
                    limit: 100,
                    search_my: true,
                },
            ]
        );
        assert!(backend.fallbacks.is_empty());
    }

    #[test]
    fn only_my_messages_fallback_is_limited_to_channels() {
        assert!(supports_only_my_messages_fallback("channel"));
        assert!(supports_only_my_messages_fallback("supergroup"));
        assert!(!supports_only_my_messages_fallback("group"));
    }

    #[tokio::test]
    async fn finish_takeout_preserves_success_and_error_mapping() {
        let mut success_backend = FakeOperationsBackend::new([Ok(RawFixture::Unit)]);
        finish_with_backend(&mut success_backend, 77, true)
            .await
            .expect("finish success");
        assert_eq!(
            success_backend.calls,
            vec![RawCall::Finish {
                takeout_id: 77,
                success: true,
            }]
        );

        let mut error_backend =
            FakeOperationsBackend::new([Err(AppError::network("TAKEOUT_INVALID"))]);
        let error = finish_with_backend(&mut error_backend, 77, false)
            .await
            .expect_err("finish error");
        assert_eq!(error.kind, AppErrorKind::Network);
        assert_eq!(error.message, "TAKEOUT_INVALID");
        assert_eq!(
            error_backend.calls,
            vec![RawCall::Finish {
                takeout_id: 77,
                success: false,
            }]
        );
    }

    fn raw_range(min_id: i32, max_id: i32) -> tl::enums::MessageRange {
        tl::types::MessageRange { min_id, max_id }.into()
    }

    fn messages_response(ids: Vec<i32>) -> tl::enums::messages::Messages {
        tl::types::messages::MessagesSlice {
            inexact: false,
            count: ids.len() as i32,
            next_rate: None,
            offset_id_offset: None,
            search_flood: None,
            messages: ids
                .into_iter()
                .map(raw_message)
                .map(tl::enums::Message::Message)
                .collect(),
            topics: Vec::new(),
            chats: Vec::new(),
            users: Vec::new(),
        }
        .into()
    }

    fn raw_message(id: i32) -> tl::types::Message {
        tl::types::Message {
            out: false,
            mentioned: false,
            media_unread: false,
            silent: false,
            post: false,
            from_scheduled: false,
            legacy: false,
            edit_hide: false,
            pinned: false,
            noforwards: false,
            invert_media: false,
            offline: false,
            video_processing_pending: false,
            paid_suggested_post_stars: false,
            paid_suggested_post_ton: false,
            id,
            from_id: None,
            from_boosts_applied: None,
            from_rank: None,
            peer_id: tl::types::PeerChannel { channel_id: 12_345 }.into(),
            saved_peer_id: None,
            fwd_from: None,
            via_bot_id: None,
            via_business_bot_id: None,
            guestchat_via_from: None,
            reply_to: None,
            date: 1234,
            message: format!("message {id}"),
            media: None,
            reply_markup: None,
            entities: None,
            views: None,
            forwards: None,
            replies: None,
            edit_date: None,
            post_author: None,
            grouped_id: None,
            reactions: None,
            restriction_reason: None,
            ttl_period: None,
            quick_reply_shortcut_id: None,
            effect: None,
            factcheck: None,
            report_delivery_until_date: None,
            paid_message_stars: None,
            suggested_post: None,
            schedule_repeat_period: None,
            summary_from_language: None,
        }
    }

    fn message_ids(messages: &[TakeoutMessage]) -> Vec<i64> {
        messages.iter().map(TakeoutMessage::message_id).collect()
    }
}
