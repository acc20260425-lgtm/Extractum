use extractum_core::error::{AppError, AppResult};
use grammers_client::{tl, Client};
use grammers_session::types::PeerRef;

use super::super::dto::ForumTopicSnapshot;
use super::super::error::is_non_forum_topic_refresh_error;

trait ForumTopicBackend {
    async fn get_forum_topics(
        &mut self,
        request: tl::functions::messages::GetForumTopics,
    ) -> AppResult<tl::enums::messages::ForumTopics>;
}

struct LiveForumTopicBackend<'a> {
    client: &'a Client,
}

impl ForumTopicBackend for LiveForumTopicBackend<'_> {
    async fn get_forum_topics(
        &mut self,
        request: tl::functions::messages::GetForumTopics,
    ) -> AppResult<tl::enums::messages::ForumTopics> {
        self.client
            .invoke(&request)
            .await
            .map_err(|error| AppError::network(error.to_string()))
    }
}

pub(super) async fn fetch_forum_topics(
    client: &Client,
    peer: PeerRef,
) -> AppResult<Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>> {
    fetch_forum_topics_with(&mut LiveForumTopicBackend { client }, peer).await
}

async fn fetch_forum_topics_with<B: ForumTopicBackend>(
    backend: &mut B,
    peer: PeerRef,
) -> AppResult<Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>> {
    let mut topics = Vec::new();
    let mut deleted_topic_ids = Vec::new();
    let mut offset_date = 0_i32;
    let mut offset_id = 0_i32;
    let mut offset_topic = 0_i32;
    let mut sort_order = 0_i64;

    loop {
        let response = match backend
            .get_forum_topics(tl::functions::messages::GetForumTopics {
                peer: peer.into(),
                q: None,
                offset_date,
                offset_id,
                offset_topic,
                limit: 100,
            })
            .await
        {
            Ok(response) => response,
            Err(error) if is_non_forum_topic_refresh_error(&error.message) => return Ok(None),
            Err(error) => return Err(error),
        };

        let tl::enums::messages::ForumTopics::Topics(forum_topics) = response;
        if forum_topics.topics.is_empty() {
            break;
        }

        let last_cursor = forum_topic_page_cursor(&forum_topics);
        for topic in forum_topics.topics {
            match topic {
                tl::enums::ForumTopic::Topic(topic) => {
                    topics.push(ForumTopicSnapshot {
                        topic_id: i64::from(topic.id),
                        top_message_id: i64::from(topic.top_message),
                        title: topic.title,
                        icon_color: i64::from(topic.icon_color),
                        icon_emoji_id: topic.icon_emoji_id,
                        is_closed: topic.closed,
                        is_pinned: topic.pinned,
                        is_hidden: topic.hidden,
                        sort_order,
                    });
                    sort_order += 1;
                }
                tl::enums::ForumTopic::Deleted(topic) => {
                    deleted_topic_ids.push(i64::from(topic.id));
                }
            }
        }

        let Some((next_offset_date, next_offset_id, next_offset_topic)) = last_cursor else {
            break;
        };
        if (next_offset_date, next_offset_id, next_offset_topic)
            == (offset_date, offset_id, offset_topic)
        {
            break;
        }
        offset_date = next_offset_date;
        offset_id = next_offset_id;
        offset_topic = next_offset_topic;
    }

    Ok(Some((topics, deleted_topic_ids)))
}

fn forum_topic_page_cursor(
    forum_topics: &tl::types::messages::ForumTopics,
) -> Option<(i32, i32, i32)> {
    let last_topic = forum_topics
        .topics
        .iter()
        .rev()
        .find_map(|topic| match topic {
            tl::enums::ForumTopic::Topic(topic) => Some(topic),
            tl::enums::ForumTopic::Deleted(_) => None,
        })?;
    let offset_date = forum_topics
        .messages
        .iter()
        .find(|message| message.id() == last_topic.top_message)
        .and_then(forum_topic_message_date)
        .unwrap_or(last_topic.date);

    Some((offset_date, last_topic.top_message, last_topic.id))
}

fn forum_topic_message_date(message: &tl::enums::Message) -> Option<i32> {
    match message {
        tl::enums::Message::Empty(_) => None,
        tl::enums::Message::Message(message) => Some(message.date),
        tl::enums::Message::Service(message) => Some(message.date),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use extractum_core::error::AppError;
    use grammers_session::types::{PeerAuth, PeerId};

    use super::*;

    struct TestForumTopicBackend {
        responses: VecDeque<AppResult<tl::enums::messages::ForumTopics>>,
        requests: Vec<tl::functions::messages::GetForumTopics>,
    }

    impl ForumTopicBackend for TestForumTopicBackend {
        async fn get_forum_topics(
            &mut self,
            request: tl::functions::messages::GetForumTopics,
        ) -> AppResult<tl::enums::messages::ForumTopics> {
            self.requests.push(request);
            self.responses
                .pop_front()
                .unwrap_or_else(|| Err(AppError::internal("missing test GetForumTopics response")))
        }
    }

    fn peer() -> PeerRef {
        PeerRef {
            id: PeerId::channel(900).expect("valid forum channel"),
            auth: PeerAuth::from_hash(9_000),
        }
    }

    fn notify_settings() -> tl::enums::PeerNotifySettings {
        tl::types::PeerNotifySettings {
            show_previews: None,
            silent: None,
            mute_until: None,
            ios_sound: None,
            android_sound: None,
            other_sound: None,
            stories_muted: None,
            stories_hide_sender: None,
            stories_ios_sound: None,
            stories_android_sound: None,
            stories_other_sound: None,
        }
        .into()
    }

    fn topic(id: i32, top_message: i32, title: &str) -> tl::enums::ForumTopic {
        tl::types::ForumTopic {
            my: false,
            closed: id == 1,
            pinned: id == 3,
            short: false,
            hidden: id == 3,
            title_missing: false,
            id,
            date: 700 + id,
            peer: tl::types::PeerChannel { channel_id: 900 }.into(),
            title: title.to_string(),
            icon_color: 10 + id,
            icon_emoji_id: Some(i64::from(1_000 + id)),
            top_message,
            read_inbox_max_id: 0,
            read_outbox_max_id: 0,
            unread_count: 0,
            unread_mentions_count: 0,
            unread_reactions_count: 0,
            unread_poll_votes_count: 0,
            from_id: tl::types::PeerUser { user_id: 101 }.into(),
            notify_settings: notify_settings(),
            draft: None,
        }
        .into()
    }

    fn deleted(id: i32) -> tl::enums::ForumTopic {
        tl::types::ForumTopicDeleted { id }.into()
    }

    fn cursor_message(id: i32, date: i32) -> tl::enums::Message {
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

    fn page(
        topics: Vec<tl::enums::ForumTopic>,
        messages: Vec<tl::enums::Message>,
    ) -> tl::enums::messages::ForumTopics {
        tl::types::messages::ForumTopics {
            order_by_create_date: false,
            count: topics.len() as i32,
            topics,
            messages,
            chats: Vec::new(),
            users: Vec::new(),
            pts: 0,
        }
        .into()
    }

    #[tokio::test]
    async fn forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor() {
        let mut backend = TestForumTopicBackend {
            responses: VecDeque::from([
                Ok(page(
                    vec![topic(1, 101, "One"), deleted(2), topic(3, 103, "Three")],
                    vec![cursor_message(101, 1_001), cursor_message(103, 1_003)],
                )),
                Ok(page(
                    vec![deleted(4), topic(3, 103, "Three repeated")],
                    vec![cursor_message(103, 1_003)],
                )),
            ]),
            requests: Vec::new(),
        };

        let result = fetch_forum_topics_with(&mut backend, peer())
            .await
            .expect("fetch forum topics")
            .expect("forum topic response");

        assert_eq!(
            backend.requests.len(),
            2,
            "non-advancing cursor is terminal"
        );
        assert_eq!(
            backend
                .requests
                .iter()
                .map(|request| (
                    request.offset_date,
                    request.offset_id,
                    request.offset_topic,
                    request.limit
                ))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0, 100), (1_003, 103, 3, 100)]
        );
        assert_eq!(
            result
                .0
                .iter()
                .map(|topic| (topic.topic_id, topic.title.as_str(), topic.sort_order))
                .collect::<Vec<_>>(),
            vec![(1, "One", 0), (3, "Three", 1), (3, "Three repeated", 2)]
        );
        assert_eq!(result.1, vec![2, 4]);
        assert!(result.0[0].is_closed);
        assert!(result.0[1].is_pinned);
        assert!(result.0[1].is_hidden);

        let mut non_forum = TestForumTopicBackend {
            responses: VecDeque::from([Err(AppError::network(
                "Rpc error 400: CHANNEL_FORUM_MISSING",
            ))]),
            requests: Vec::new(),
        };
        assert!(fetch_forum_topics_with(&mut non_forum, peer())
            .await
            .expect("classified non-forum response")
            .is_none());
        assert_eq!(non_forum.requests.len(), 1);

        let mut private = TestForumTopicBackend {
            responses: VecDeque::from([Err(AppError::network("Rpc error 400: CHANNEL_PRIVATE"))]),
            requests: Vec::new(),
        };
        let error = fetch_forum_topics_with(&mut private, peer())
            .await
            .expect_err("unclassified RPC error remains an error");
        assert_eq!(error.message, "Rpc error 400: CHANNEL_PRIVATE");
        assert_eq!(private.requests.len(), 1);
    }
}
