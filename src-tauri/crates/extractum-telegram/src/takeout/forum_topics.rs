use std::future::Future;

use extractum_core::error::AppResult;

use super::super::{
    dto::{ForumTopicSnapshot, PeerDescriptor},
    live,
};

pub(super) async fn takeout_forum_topics(
    client: &grammers_client::Client,
    peer: &PeerDescriptor,
) -> AppResult<Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>> {
    takeout_forum_topics_with(|| live::fetch_forum_topics(client, peer)).await
}

async fn takeout_forum_topics_with<F, Fut>(
    fetch: F,
) -> AppResult<Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = AppResult<Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>>>,
{
    fetch().await
}

#[cfg(test)]
mod tests {
    use super::super::super::dto::ForumTopicSnapshot;
    use super::takeout_forum_topics_with;

    #[tokio::test]
    async fn forum_topic_operation_returns_owned_snapshots() {
        let expected_topic = ForumTopicSnapshot {
            topic_id: 7,
            top_message_id: 70,
            title: "Owned topic".to_string(),
            icon_color: 0x6fb9f0,
            icon_emoji_id: Some(700),
            is_closed: false,
            is_pinned: true,
            is_hidden: false,
            sort_order: 0,
        };

        let result =
            takeout_forum_topics_with(
                || async move { Ok(Some((vec![expected_topic], vec![8, 9]))) },
            )
            .await
            .expect("takeout forum topic operation")
            .expect("forum topic snapshots");

        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].topic_id, 7);
        assert_eq!(result.0[0].title, "Owned topic");
        assert_eq!(result.0[0].icon_emoji_id, Some(700));
        assert!(result.0[0].is_pinned);
        assert_eq!(result.1, vec![8, 9]);
    }
}
