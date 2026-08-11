use std::collections::HashMap;
use std::sync::Arc;

use extractum_core::error::{AppError, AppResult};
use grammers_client::{peer::Peer, tl, Client};
use grammers_session::types::{PeerId, PeerInfo, PeerKind, PeerRef};
use serde_json::json;

use super::super::dto::{
    TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,
};
use super::super::media::extract_raw_item_payload;
use super::super::session::TelegramSession;

pub struct LiveMessageBatch {
    messages: Vec<LiveMessage>,
    is_terminal: bool,
    next_offset_id: i32,
    next_offset_date: i32,
}

pub struct LiveMessage {
    raw: tl::enums::Message,
    fetched_in: PeerRef,
    peers: Arc<HashMap<PeerId, Peer>>,
}

impl LiveMessageBatch {
    pub fn take_messages(&mut self) -> Vec<LiveMessage> {
        std::mem::take(&mut self.messages)
    }

    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    pub fn next_offset_id(&self) -> i32 {
        self.next_offset_id
    }

    pub fn next_offset_date(&self) -> i32 {
        self.next_offset_date
    }
}

impl LiveMessage {
    pub fn message_id(&self) -> i64 {
        i64::from(self.raw.id())
    }

    pub fn published_at(&self) -> i64 {
        i64::from(raw_message_date(&self.raw))
    }

    pub fn into_draft(self, source_title: Option<&str>) -> AppResult<Option<TelegramMessageDraft>> {
        let Some((content, content_kind, media)) = extract_raw_item_payload(&self.raw) else {
            return Ok(None);
        };
        let author = raw_message_author(&self);
        let telegram_context = extract_telegram_context(&self.raw);
        let raw_data = build_raw_payload(
            &self,
            source_title,
            &author,
            content.as_deref(),
            content_kind,
            media.as_ref(),
        )?;
        let telegram_identity = Some(fallback_message_identity(
            self.fetched_in,
            self.message_id(),
        )?);

        Ok(Some(TelegramMessageDraft {
            telegram_identity,
            telegram_context,
            content,
            content_kind,
            author,
            published_at: self.published_at(),
            raw_data,
            item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
            media,
        }))
    }
}

fn raw_message_date(message: &tl::enums::Message) -> i32 {
    match message {
        tl::enums::Message::Empty(_) => 0,
        tl::enums::Message::Message(message) => message.date,
        tl::enums::Message::Service(message) => message.date,
    }
}

fn raw_message_peer_id(message: &LiveMessage) -> PeerId {
    match &message.raw {
        tl::enums::Message::Empty(_) => message.fetched_in.id,
        tl::enums::Message::Message(message) => PeerId::from(&message.peer_id),
        tl::enums::Message::Service(message) => PeerId::from(&message.peer_id),
    }
}

fn raw_message_sender_id(message: &LiveMessage) -> Option<PeerId> {
    let (from_id, outgoing) = match &message.raw {
        tl::enums::Message::Empty(_) => return None,
        tl::enums::Message::Message(message) => (message.from_id.as_ref(), message.out),
        tl::enums::Message::Service(message) => (message.from_id.as_ref(), message.out),
    };
    from_id.map(PeerId::from).or_else(|| {
        let peer_id = raw_message_peer_id(message);
        matches!(peer_id.kind(), PeerKind::User).then(|| {
            if outgoing {
                PeerId::self_user()
            } else {
                peer_id
            }
        })
    })
}

fn raw_message_author(message: &LiveMessage) -> Option<String> {
    if let tl::enums::Message::Message(raw) = &message.raw {
        if let Some(post_author) = raw.post_author.as_ref() {
            return Some(post_author.clone());
        }
    }

    raw_message_sender_id(message)
        .and_then(|sender_id| message.peers.get(&sender_id))
        .and_then(|sender| {
            sender
                .name()
                .map(str::to_string)
                .or_else(|| sender.username().map(|username| format!("@{username}")))
        })
}

fn extract_telegram_context(message: &tl::enums::Message) -> TelegramItemContext {
    let (reply_header, reactions) = match message {
        tl::enums::Message::Empty(_) => (None, None),
        tl::enums::Message::Message(message) => {
            (message.reply_to.as_ref(), message.reactions.as_ref())
        }
        tl::enums::Message::Service(message) => (message.reply_to.as_ref(), None),
    };
    let mut context = TelegramItemContext {
        reaction_count: reactions.and_then(reaction_count),
        ..TelegramItemContext::default()
    };

    if let Some(tl::enums::MessageReplyHeader::Header(header)) = reply_header {
        context.reply_to_msg_id = header.reply_to_msg_id.map(i64::from);
        context.reply_to_top_id = header.reply_to_top_id.map(i64::from);
        if let Some((kind, id)) = reply_peer_context(header.reply_to_peer_id.as_ref()) {
            context.reply_to_peer_kind = Some(kind.to_string());
            context.reply_to_peer_id = Some(id);
        }
    }

    context
}

fn reaction_count(reactions: &tl::enums::MessageReactions) -> Option<i64> {
    let tl::enums::MessageReactions::Reactions(reactions) = reactions;
    let count = reactions
        .results
        .iter()
        .map(|reaction| {
            let tl::enums::ReactionCount::Count(reaction) = reaction;
            reaction.count
        })
        .sum::<i32>();
    Some(i64::from(count))
}

fn reply_peer_context(peer: Option<&tl::enums::Peer>) -> Option<(&'static str, String)> {
    match peer? {
        tl::enums::Peer::User(peer) => Some(("user", peer.user_id.to_string())),
        tl::enums::Peer::Chat(peer) => Some(("chat", peer.chat_id.to_string())),
        tl::enums::Peer::Channel(peer) => Some(("channel", peer.channel_id.to_string())),
    }
}

fn build_raw_payload(
    message: &LiveMessage,
    source_title: Option<&str>,
    author: &Option<String>,
    content: Option<&str>,
    content_kind: &'static str,
    media: Option<&super::super::media::TelegramMediaPayload>,
) -> AppResult<Vec<u8>> {
    serde_json::to_vec(&json!({
        "id": message.message_id(),
        "peer_id": raw_message_peer_id(message).to_string(),
        "sender_id": raw_message_sender_id(message).map(|id| id.to_string()),
        "published_at": message.published_at(),
        "text": content,
        "content_kind": content_kind,
        "has_media": media.is_some(),
        "media_kind": media.map(|media| &media.kind),
        "media_metadata": media.map(|media| &media.metadata),
        "post_author": match &message.raw {
            tl::enums::Message::Message(message) => message.post_author.as_deref(),
            tl::enums::Message::Empty(_) | tl::enums::Message::Service(_) => None,
        },
        "source_title": source_title,
        "author": author,
    }))
    .map_err(|error| AppError::internal(error.to_string()))
}

fn fallback_message_identity(
    fallback_peer: PeerRef,
    telegram_message_id: i64,
) -> AppResult<TelegramMessageIdentity> {
    let history_peer_kind = match fallback_peer.id.kind() {
        PeerKind::User => "user",
        PeerKind::Chat => "chat",
        PeerKind::Channel => "channel",
    }
    .to_string();
    let history_peer_id = fallback_peer.id.bare_id().ok_or_else(|| {
        AppError::validation("Telegram self-user peer cannot be used as message history peer")
    })?;

    Ok(TelegramMessageIdentity {
        history_peer_kind,
        history_peer_id,
        telegram_message_id,
        migration_domain: None,
        is_migrated_history: false,
    })
}

trait MessageBatchBackend {
    fn client(&self) -> &Client;

    async fn get_history(
        &mut self,
        request: tl::functions::messages::GetHistory,
    ) -> AppResult<tl::enums::messages::Messages>;
}

struct LiveMessageBatchBackend<'a> {
    client: &'a Client,
}

impl MessageBatchBackend for LiveMessageBatchBackend<'_> {
    fn client(&self) -> &Client {
        self.client
    }

    async fn get_history(
        &mut self,
        request: tl::functions::messages::GetHistory,
    ) -> AppResult<tl::enums::messages::Messages> {
        self.client
            .invoke(&request)
            .await
            .map_err(|error| AppError::network(error.to_string()))
    }
}

pub(super) async fn fetch_message_batch(
    client: &Client,
    session: &TelegramSession,
    peer: PeerRef,
    offset_id: i32,
    offset_date: i32,
    limit: usize,
) -> AppResult<LiveMessageBatch> {
    fetch_message_batch_with(
        &mut LiveMessageBatchBackend { client },
        session,
        peer,
        offset_id,
        offset_date,
        limit,
    )
    .await
}

async fn fetch_message_batch_with<B: MessageBatchBackend>(
    backend: &mut B,
    session: &TelegramSession,
    peer: PeerRef,
    offset_id: i32,
    offset_date: i32,
    limit: usize,
) -> AppResult<LiveMessageBatch> {
    if !(1..=100).contains(&limit) {
        return Err(AppError::validation(
            "Telegram live history batch limit must be between 1 and 100",
        ));
    }
    let response = backend
        .get_history(tl::functions::messages::GetHistory {
            peer: peer.into(),
            offset_id,
            offset_date,
            add_offset: 0,
            limit: limit as i32,
            max_id: 0,
            min_id: 0,
            hash: 0,
        })
        .await?;

    let (messages, users, chats, is_terminal) = match response {
        tl::enums::messages::Messages::NotModified(_) => {
            return Err(AppError::network(
                "Telegram returned messagesNotModified for live history batch",
            ));
        }
        tl::enums::messages::Messages::Messages(response) => {
            (response.messages, response.users, response.chats, true)
        }
        tl::enums::messages::Messages::Slice(response) => {
            let is_terminal =
                response.messages.is_empty() || response.messages[0].id() <= limit as i32;
            (
                response.messages,
                response.users,
                response.chats,
                is_terminal,
            )
        }
        tl::enums::messages::Messages::ChannelMessages(response) => {
            let is_terminal =
                response.messages.is_empty() || response.messages[0].id() <= limit as i32;
            (
                response.messages,
                response.users,
                response.chats,
                is_terminal,
            )
        }
    };

    let client = backend.client();
    let peers = users
        .into_iter()
        .map(|user| Peer::User(grammers_client::peer::User::from_raw(client, user)))
        .chain(chats.into_iter().map(|chat| Peer::from_raw(client, chat)))
        .map(|peer| (peer.id(), peer))
        .collect::<HashMap<_, _>>();
    let peer_infos = peers.values().map(PeerInfo::from).collect::<Vec<_>>();
    session.cache_peer_infos(&peer_infos).await?;

    let (next_offset_id, next_offset_date) = if !is_terminal {
        messages
            .last()
            .map(|message| (message.id(), raw_message_date(message)))
            .unwrap_or((offset_id, offset_date))
    } else {
        (offset_id, offset_date)
    };
    let peers = Arc::new(peers);
    let messages = messages
        .into_iter()
        .map(|raw| LiveMessage {
            raw,
            fetched_in: peer,
            peers: Arc::clone(&peers),
        })
        .collect();

    Ok(LiveMessageBatch {
        messages,
        is_terminal,
        next_offset_id,
        next_offset_date,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use grammers_mtsender::SenderPool;
    use grammers_session::types::{PeerAuth, PeerId, PeerRef};
    use grammers_session::Session;

    use super::*;

    struct TestMessageBackend {
        client: Client,
        responses: VecDeque<tl::enums::messages::Messages>,
        requests: Vec<tl::functions::messages::GetHistory>,
    }

    impl MessageBatchBackend for TestMessageBackend {
        fn client(&self) -> &Client {
            &self.client
        }

        async fn get_history(
            &mut self,
            request: tl::functions::messages::GetHistory,
        ) -> AppResult<tl::enums::messages::Messages> {
            self.requests.push(request);
            self.responses
                .pop_front()
                .ok_or_else(|| AppError::internal("missing test GetHistory response"))
        }
    }

    fn client_for_session(session: &TelegramSession) -> Client {
        let pool = SenderPool::new(session.clone_memory_session(), 12345);
        Client::new(pool.handle)
    }

    fn history_peer() -> PeerRef {
        PeerRef {
            id: PeerId::channel(900).expect("valid history channel"),
            auth: PeerAuth::from_hash(9_000),
        }
    }

    fn raw_message(id: i32, date: i32) -> tl::enums::Message {
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
            rich_message: None,
            peer_id: tl::types::PeerChannel { channel_id: 900 }.into(),
            saved_peer_id: None,
            fwd_from: None,
            via_bot_id: None,
            via_business_bot_id: None,
            guestchat_via_from: None,
            reply_to: None,
            date,
            message: String::new(),
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
        .into()
    }

    fn raw_user(id: i64, access_hash: i64, min: bool) -> tl::enums::User {
        tl::types::User {
            is_self: false,
            contact: false,
            mutual_contact: false,
            deleted: false,
            bot: false,
            bot_guard: false,
            bot_chat_history: false,
            bot_nochats: false,
            verified: false,
            restricted: false,
            min,
            bot_inline_geo: false,
            support: false,
            scam: false,
            apply_min_photo: false,
            fake: false,
            bot_attach_menu: false,
            premium: false,
            attach_menu_enabled: false,
            bot_can_edit: false,
            close_friend: false,
            stories_hidden: false,
            stories_unavailable: false,
            contact_require_premium: false,
            bot_business: false,
            bot_has_main_app: false,
            bot_forum_view: false,
            bot_forum_can_manage_topics: false,
            bot_can_manage_bots: false,
            bot_guestchat: false,
            id,
            access_hash: Some(access_hash),
            first_name: Some(format!("User {id}")),
            last_name: None,
            username: Some(format!("user{id}")),
            phone: None,
            photo: None,
            status: None,
            bot_info_version: None,
            restriction_reason: None,
            bot_inline_placeholder: None,
            lang_code: None,
            emoji_status: None,
            usernames: None,
            stories_max_id: None,
            color: None,
            profile_color: None,
            bot_active_users: None,
            bot_verification_icon: None,
            send_paid_messages_stars: None,
        }
        .into()
    }

    fn raw_chat(id: i64) -> tl::enums::Chat {
        tl::types::Chat {
            creator: false,
            left: false,
            deactivated: false,
            call_active: false,
            call_not_empty: false,
            noforwards: false,
            id,
            title: format!("Chat {id}"),
            photo: tl::enums::ChatPhoto::Empty,
            participants_count: 2,
            date: 0,
            version: 1,
            migrated_to: None,
            admin_rights: None,
            default_banned_rights: None,
        }
        .into()
    }

    fn raw_channel(id: i64, access_hash: i64, min: bool) -> tl::enums::Chat {
        tl::types::Channel {
            creator: false,
            left: false,
            broadcast: true,
            verified: false,
            megagroup: false,
            restricted: false,
            signatures: false,
            min,
            scam: false,
            has_link: false,
            has_geo: false,
            slowmode_enabled: false,
            call_active: false,
            call_not_empty: false,
            fake: false,
            gigagroup: false,
            noforwards: false,
            join_to_send: false,
            join_request: false,
            forum: false,
            stories_hidden: false,
            stories_hidden_min: false,
            stories_unavailable: false,
            signature_profiles: false,
            autotranslation: false,
            broadcast_messages_allowed: false,
            monoforum: false,
            forum_tabs: false,
            id,
            access_hash: Some(access_hash),
            title: format!("Channel {id}"),
            username: Some(format!("channel{id}")),
            photo: tl::enums::ChatPhoto::Empty,
            date: 0,
            restriction_reason: None,
            admin_rights: None,
            banned_rights: None,
            default_banned_rights: None,
            participants_count: None,
            usernames: None,
            stories_max_id: None,
            color: None,
            profile_color: None,
            emoji_status: None,
            level: None,
            subscription_until_date: None,
            bot_verification_icon: None,
            send_paid_messages_stars: None,
            linked_monoforum_id: None,
        }
        .into()
    }

    fn slice_response(
        messages: Vec<tl::enums::Message>,
        users: Vec<tl::enums::User>,
        chats: Vec<tl::enums::Chat>,
    ) -> tl::enums::messages::Messages {
        tl::types::messages::MessagesSlice {
            inexact: false,
            count: messages.len() as i32,
            next_rate: None,
            offset_id_offset: None,
            search_flood: None,
            messages,
            topics: Vec::new(),
            chats,
            users,
        }
        .into()
    }

    fn messages_response(messages: Vec<tl::enums::Message>) -> tl::enums::messages::Messages {
        tl::types::messages::Messages {
            messages,
            topics: Vec::new(),
            chats: Vec::new(),
            users: Vec::new(),
        }
        .into()
    }

    fn channel_response(messages: Vec<tl::enums::Message>) -> tl::enums::messages::Messages {
        tl::types::messages::ChannelMessages {
            inexact: false,
            pts: 0,
            count: messages.len() as i32,
            offset_id_offset: None,
            messages,
            topics: Vec::new(),
            chats: Vec::new(),
            users: Vec::new(),
        }
        .into()
    }

    #[tokio::test]
    async fn message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule() {
        let session = TelegramSession::empty();
        let memory_session = session.clone_memory_session();
        let mut backend = TestMessageBackend {
            client: client_for_session(&session),
            responses: VecDeque::from([slice_response(
                vec![raw_message(50, 500), raw_message(49, 490)],
                vec![raw_user(101, 1_001, false), raw_user(102, 1_002, true)],
                vec![
                    raw_chat(201),
                    raw_channel(301, 3_001, false),
                    raw_channel(302, 3_002, true),
                ],
            )]),
            requests: Vec::new(),
        };

        let mut batch = fetch_message_batch_with(&mut backend, &session, history_peer(), 77, 88, 2)
            .await
            .expect("fetch live batch");

        assert_eq!(backend.requests.len(), 1, "exactly one raw invoke");
        let request = &backend.requests[0];
        assert_eq!(
            (
                request.offset_id,
                request.offset_date,
                request.add_offset,
                request.limit,
                request.max_id,
                request.min_id,
                request.hash,
            ),
            (77, 88, 0, 2, 0, 0, 0)
        );
        let tl::enums::InputPeer::Channel(request_peer) = &request.peer else {
            panic!("history request must use the supplied channel peer")
        };
        assert_eq!(
            (request_peer.channel_id, request_peer.access_hash),
            (900, 9_000)
        );
        assert_eq!(
            batch
                .take_messages()
                .iter()
                .map(LiveMessage::message_id)
                .collect::<Vec<_>>(),
            vec![50, 49]
        );
        assert!(!batch.is_terminal());
        assert_eq!(
            (batch.next_offset_id(), batch.next_offset_date()),
            (49, 490)
        );

        assert!(
            Session::peer(memory_session.as_ref(), PeerId::user(101).unwrap())
                .await
                .expect("read cached peer")
                .is_some()
        );
        assert!(
            Session::peer(memory_session.as_ref(), PeerId::user(102).unwrap())
                .await
                .expect("read cached peer")
                .is_none()
        );
        assert!(
            Session::peer(memory_session.as_ref(), PeerId::chat(201).unwrap())
                .await
                .expect("read cached peer")
                .is_some()
        );
        assert!(
            Session::peer(memory_session.as_ref(), PeerId::channel(301).unwrap())
                .await
                .expect("read cached peer")
                .is_some()
        );
        assert!(
            Session::peer(memory_session.as_ref(), PeerId::channel(302).unwrap())
                .await
                .expect("read cached peer")
                .is_none()
        );

        let validation_session = TelegramSession::empty();
        let mut validation_backend = TestMessageBackend {
            client: client_for_session(&validation_session),
            responses: VecDeque::new(),
            requests: Vec::new(),
        };
        for invalid_limit in [0, 101] {
            let error = fetch_message_batch_with(
                &mut validation_backend,
                &validation_session,
                history_peer(),
                0,
                0,
                invalid_limit,
            )
            .await
            .err()
            .expect("out-of-range limit is rejected");
            assert_eq!(error.kind, extractum_core::error::AppErrorKind::Validation);
        }
        assert!(
            validation_backend.requests.is_empty(),
            "invalid limits fail before invoke"
        );

        let terminal_session = TelegramSession::empty();
        let mut terminal_backend = TestMessageBackend {
            client: client_for_session(&terminal_session),
            responses: VecDeque::from([
                messages_response(vec![raw_message(80, 800)]),
                slice_response(vec![raw_message(1, 10)], Vec::new(), Vec::new()),
                channel_response(Vec::new()),
                channel_response(vec![raw_message(100, 1_000)]),
                channel_response(vec![raw_message(101, 1_010)]),
            ]),
            requests: Vec::new(),
        };
        let messages_terminal = fetch_message_batch_with(
            &mut terminal_backend,
            &terminal_session,
            history_peer(),
            70,
            700,
            1,
        )
        .await
        .expect("Messages response");
        assert!(messages_terminal.is_terminal());
        assert_eq!(
            (
                messages_terminal.next_offset_id(),
                messages_terminal.next_offset_date()
            ),
            (70, 700),
            "terminal Messages preserves input offsets"
        );
        let slice_terminal = fetch_message_batch_with(
            &mut terminal_backend,
            &terminal_session,
            history_peer(),
            60,
            600,
            1,
        )
        .await
        .expect("terminal Slice response");
        assert!(slice_terminal.is_terminal());
        assert_eq!(
            (
                slice_terminal.next_offset_id(),
                slice_terminal.next_offset_date()
            ),
            (60, 600)
        );
        let channel_empty = fetch_message_batch_with(
            &mut terminal_backend,
            &terminal_session,
            history_peer(),
            50,
            500,
            100,
        )
        .await
        .expect("empty ChannelMessages response");
        assert!(channel_empty.is_terminal());
        assert_eq!(
            (
                channel_empty.next_offset_id(),
                channel_empty.next_offset_date()
            ),
            (50, 500)
        );
        let channel_boundary = fetch_message_batch_with(
            &mut terminal_backend,
            &terminal_session,
            history_peer(),
            45,
            450,
            100,
        )
        .await
        .expect("boundary ChannelMessages response");
        assert!(channel_boundary.is_terminal());
        assert_eq!(
            (
                channel_boundary.next_offset_id(),
                channel_boundary.next_offset_date()
            ),
            (45, 450),
            "first message id equal to limit is terminal"
        );
        let channel_nonterminal = fetch_message_batch_with(
            &mut terminal_backend,
            &terminal_session,
            history_peer(),
            40,
            400,
            100,
        )
        .await
        .expect("nonterminal ChannelMessages response");
        assert!(!channel_nonterminal.is_terminal());
        assert_eq!(
            (
                channel_nonterminal.next_offset_id(),
                channel_nonterminal.next_offset_date()
            ),
            (101, 1_010)
        );
        assert_eq!(
            terminal_backend
                .requests
                .iter()
                .map(|request| request.limit)
                .collect::<Vec<_>>(),
            vec![1, 1, 100, 100, 100],
            "both inclusive limit endpoints invoke successfully"
        );

        let not_modified_session = TelegramSession::empty();
        let not_modified_memory = not_modified_session.clone_memory_session();
        let mut not_modified_backend = TestMessageBackend {
            client: client_for_session(&not_modified_session),
            responses: VecDeque::from([
                tl::types::messages::MessagesNotModified { count: 0 }.into()
            ]),
            requests: Vec::new(),
        };
        let error = fetch_message_batch_with(
            &mut not_modified_backend,
            &not_modified_session,
            history_peer(),
            0,
            0,
            100,
        )
        .await
        .err()
        .expect("messagesNotModified is a typed error");
        assert_eq!(error.kind, extractum_core::error::AppErrorKind::Network);
        assert_eq!(
            error.message,
            "Telegram returned messagesNotModified for live history batch"
        );
        assert_eq!(not_modified_backend.requests.len(), 1);
        assert!(Session::peer(
            not_modified_memory.as_ref(),
            PeerId::channel(900).expect("history peer id")
        )
        .await
        .expect("read cached peer")
        .is_none());
    }

    #[tokio::test]
    async fn live_message_maps_owned_draft_and_skips_empty_payload() {
        let session = TelegramSession::empty();
        let client = client_for_session(&session);
        let sender = raw_user(101, 1_001, false);
        let sender_peer = Peer::User(grammers_client::peer::User::from_raw(
            &client,
            sender.clone(),
        ));
        let peers = Arc::new(HashMap::from([(sender_peer.id(), sender_peer)]));
        let tl::enums::Message::Message(mut raw) = raw_message(42, 1_700_000_042) else {
            unreachable!("regular message fixture")
        };
        raw.message = "  hello owned batch  ".to_string();
        raw.from_id = Some(tl::types::PeerUser { user_id: 101 }.into());
        raw.reply_to = Some(
            tl::types::MessageReplyHeader {
                reply_to_scheduled: false,
                forum_topic: true,
                quote: false,
                reply_to_ephemeral: false,
                reply_to_msg_id: Some(41),
                reply_to_peer_id: Some(tl::types::PeerChannel { channel_id: 900 }.into()),
                reply_from: None,
                reply_media: None,
                reply_to_top_id: Some(40),
                quote_text: None,
                quote_entities: None,
                quote_offset: None,
                todo_item_id: None,
                poll_option: None,
            }
            .into(),
        );
        raw.reactions = Some(
            tl::types::MessageReactions {
                min: false,
                can_see_list: false,
                reactions_as_tags: false,
                results: vec![tl::types::ReactionCount {
                    chosen_order: None,
                    reaction: tl::types::ReactionEmoji {
                        emoticon: "ok".to_string(),
                    }
                    .into(),
                    count: 3,
                }
                .into()],
                recent_reactions: None,
                top_reactors: None,
            }
            .into(),
        );
        raw.media = Some(
            tl::types::MessageMediaContact {
                phone_number: "+10000000000".to_string(),
                first_name: "Ada".to_string(),
                last_name: "Lovelace".to_string(),
                vcard: String::new(),
                user_id: 101,
            }
            .into(),
        );

        let draft = LiveMessage {
            raw: raw.into(),
            fetched_in: history_peer(),
            peers: Arc::clone(&peers),
        }
        .into_draft(Some("Owned Source"))
        .expect("map live message")
        .expect("non-empty payload");

        let identity = draft.telegram_identity.expect("fallback identity");
        assert_eq!(
            (
                identity.history_peer_kind.as_str(),
                identity.history_peer_id,
                identity.telegram_message_id,
                identity.migration_domain,
                identity.is_migrated_history,
            ),
            ("channel", 900, 42, None, false)
        );
        assert_eq!(draft.content.as_deref(), Some("hello owned batch"));
        assert_eq!(draft.content_kind, "text_with_media");
        assert_eq!(draft.author.as_deref(), Some("User 101"));
        assert_eq!(draft.published_at, 1_700_000_042);
        assert_eq!(draft.telegram_context.reply_to_msg_id, Some(41));
        assert_eq!(
            draft.telegram_context.reply_to_peer_kind.as_deref(),
            Some("channel")
        );
        assert_eq!(
            draft.telegram_context.reply_to_peer_id.as_deref(),
            Some("900")
        );
        assert_eq!(draft.telegram_context.reply_to_top_id, Some(40));
        assert_eq!(draft.telegram_context.reaction_count, Some(3));
        assert_eq!(
            draft.media.as_ref().map(|media| media.kind.as_str()),
            Some("contact")
        );
        let raw_data: serde_json::Value =
            serde_json::from_slice(&draft.raw_data).expect("raw payload json");
        assert_eq!(
            raw_data,
            serde_json::json!({
                "id": 42,
                "peer_id": "-1000000000900",
                "sender_id": "101",
                "published_at": 1_700_000_042,
                "text": "hello owned batch",
                "content_kind": "text_with_media",
                "has_media": true,
                "media_kind": "contact",
                "media_metadata": {
                    "summary": "Contact: Ada Lovelace",
                    "file_name": null,
                    "mime_type": null,
                    "size_bytes": null,
                    "width": null,
                    "height": null,
                    "duration_seconds": null,
                },
                "post_author": null,
                "source_title": "Owned Source",
                "author": "User 101",
            })
        );

        let tl::enums::Message::Message(mut incoming_private_raw) = raw_message(43, 1_700_000_043)
        else {
            unreachable!("regular message fixture")
        };
        incoming_private_raw.peer_id = tl::types::PeerUser { user_id: 101 }.into();
        incoming_private_raw.message = "private incoming".to_string();
        let private_draft = LiveMessage {
            raw: incoming_private_raw.into(),
            fetched_in: PeerRef {
                id: PeerId::user(101).expect("valid private user"),
                auth: PeerAuth::from_hash(1_001),
            },
            peers: Arc::clone(&peers),
        }
        .into_draft(Some("Owned Source"))
        .expect("map private incoming message")
        .expect("private message payload");
        assert_eq!(private_draft.author.as_deref(), Some("User 101"));
        let private_identity = private_draft
            .telegram_identity
            .expect("private fallback identity");
        assert_eq!(
            (
                private_identity.history_peer_kind.as_str(),
                private_identity.history_peer_id
            ),
            ("user", 101)
        );
        let private_raw_data: serde_json::Value =
            serde_json::from_slice(&private_draft.raw_data).expect("private raw payload json");
        assert_eq!(private_raw_data["peer_id"], "101");
        assert_eq!(private_raw_data["sender_id"], "101");
        assert_eq!(private_raw_data["author"], "User 101");

        let empty = LiveMessage {
            raw: raw_message(41, 1_700_000_041),
            fetched_in: PeerRef {
                id: PeerId::self_user(),
                auth: PeerAuth::default(),
            },
            peers,
        }
        .into_draft(Some("Owned Source"))
        .expect("map empty message");
        assert!(empty.is_none(), "empty raw payload is skipped");
    }

    #[test]
    fn fallback_peer_identity_uses_telegram_history_peer_vocabulary() {
        let identity = fallback_message_identity(history_peer(), 42).expect("valid fallback peer");

        assert_eq!(identity.history_peer_kind, "channel");
        assert_eq!(identity.history_peer_id, 900);
        assert_eq!(identity.telegram_message_id, 42);
        assert_eq!(identity.migration_domain, None);
        assert!(!identity.is_migrated_history);
    }

    #[test]
    fn reply_peer_context_uses_telegram_peer_kinds() {
        assert_eq!(
            reply_peer_context(Some(&tl::enums::Peer::User(tl::types::PeerUser {
                user_id: 11
            }))),
            Some(("user", "11".to_string()))
        );
        assert_eq!(
            reply_peer_context(Some(&tl::enums::Peer::Chat(tl::types::PeerChat {
                chat_id: 22
            }))),
            Some(("chat", "22".to_string()))
        );
        assert_eq!(
            reply_peer_context(Some(&tl::enums::Peer::Channel(tl::types::PeerChannel {
                channel_id: 33
            }))),
            Some(("channel", "33".to_string()))
        );
        assert_eq!(reply_peer_context(None), None);
    }
}
