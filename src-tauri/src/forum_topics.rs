pub(crate) const FORUM_TOPIC_UNCATEGORIZED_KEY: &str = "unrecognized_topic";
pub(crate) const FORUM_TOPIC_UNCATEGORIZED_TITLE: &str = "Unrecognized topic";

pub(crate) struct ResolvedTopicAliases<'a> {
    pub(crate) item: &'a str,
    pub(crate) topic: &'a str,
    pub(crate) matched_topic: &'a str,
}

pub(crate) fn resolved_topic_join(aliases: &ResolvedTopicAliases<'_>) -> String {
    format!(
        "LEFT JOIN telegram_forum_topics AS {topic}\n  ON {}",
        resolved_topic_predicate(aliases),
        topic = aliases.topic
    )
}

pub(crate) fn resolved_topic_predicate(aliases: &ResolvedTopicAliases<'_>) -> String {
    let item = aliases.item;
    let topic = aliases.topic;
    let matched_topic = aliases.matched_topic;

    format!(
        r#"{topic}.source_id = {item}.source_id
 AND (
        {item}.reply_to_top_id = {topic}.topic_id
        OR (
            {item}.reply_to_top_id IS NULL
            AND {item}.external_id <> ''
            AND {item}.external_id NOT GLOB '*[^0-9]*'
            AND CAST({item}.external_id AS INTEGER) = {topic}.top_message_id
        )
        OR (
            {item}.reply_to_top_id IS NULL
            AND {item}.reply_to_msg_id = {topic}.topic_id
        )
        OR (
            {item}.reply_to_top_id IS NULL
            AND {topic}.topic_id = 1
            AND NOT EXISTS (
                SELECT 1
                FROM telegram_forum_topics AS {matched_topic}
                WHERE {matched_topic}.source_id = {item}.source_id
                  AND (
                        (
                            {item}.external_id <> ''
                            AND {item}.external_id NOT GLOB '*[^0-9]*'
                            AND CAST({item}.external_id AS INTEGER) = {matched_topic}.top_message_id
                        )
                        OR {item}.reply_to_msg_id = {matched_topic}.topic_id
                  )
            )
        )
    )"#
    )
}

#[cfg(test)]
mod tests {
    use super::{resolved_topic_join, ResolvedTopicAliases};

    #[test]
    fn resolved_topic_join_uses_supplied_aliases() {
        let join = resolved_topic_join(&ResolvedTopicAliases {
            item: "items",
            topic: "forum_topics",
            matched_topic: "matched_topics",
        });

        assert!(join.contains("LEFT JOIN telegram_forum_topics AS forum_topics"));
        assert!(join.contains("forum_topics.source_id = items.source_id"));
        assert!(join.contains("items.reply_to_top_id = forum_topics.topic_id"));
        assert!(join.contains("FROM telegram_forum_topics AS matched_topics"));
    }
}
