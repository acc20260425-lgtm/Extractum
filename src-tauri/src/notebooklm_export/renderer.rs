use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::notebooklm_export::model::{
    ExportTopicDescriptor, NotebookLmExportMessage, NotebookLmExportSource, ParticipantSummary,
    RenderedMessageBlock,
};

pub(crate) fn format_unix_rfc3339(value: i64) -> String {
    OffsetDateTime::from_unix_timestamp(value)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub(crate) fn approx_word_count(markdown: &str) -> usize {
    markdown.split_whitespace().count()
}

pub(crate) fn render_message_block(message: &NotebookLmExportMessage) -> RenderedMessageBlock {
    let mut markdown = String::new();
    let message_id = if message.external_id.trim().is_empty() {
        message.item_id.to_string()
    } else {
        message.external_id.clone()
    };
    let author = yaml_string(
        message
            .author
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Unknown"),
    );
    let message_type = if message.has_media {
        message.media_kind.as_deref().unwrap_or("media")
    } else {
        "text"
    };

    markdown.push_str("---\n");
    markdown.push_str(&format!("### Message #{message_id}\n\n"));
    markdown.push_str("```yaml\n");
    markdown.push_str(&format!("source_id: {}\n", message.source_id));
    markdown.push_str(&format!("item_id: {}\n", message.item_id));
    markdown.push_str(&format!(
        "telegram_message_id: {}\n",
        yaml_string(&message.external_id)
    ));
    markdown.push_str(&format!(
        "date: {}\n",
        yaml_string(&format_unix_rfc3339(message.published_at))
    ));
    markdown.push_str(&format!("author: {author}\n"));
    markdown.push_str(&format!("type: {}\n", yaml_string(message_type)));
    markdown.push_str(&format!(
        "reply_to_id: {}\n",
        yaml_optional_i64(message.reply_to_msg_id)
    ));
    markdown.push_str(&format!(
        "reply_to_author: {}\n",
        yaml_optional_string(message.reply_to_author.as_deref())
    ));
    markdown.push_str(&format!(
        "reply_to_snippet: {}\n",
        yaml_optional_string(message.reply_to_snippet.as_deref())
    ));
    markdown.push_str(&format!(
        "reply_to_peer_kind: {}\n",
        yaml_optional_string(message.reply_to_peer_kind.as_deref())
    ));
    markdown.push_str(&format!(
        "reply_to_peer_id: {}\n",
        yaml_optional_string(message.reply_to_peer_id.as_deref())
    ));
    markdown.push_str("forwarded_from: null\n");
    markdown.push_str(&format!(
        "thread_id: {}\n",
        yaml_optional_i64(message.reply_to_top_id)
    ));
    markdown.push_str(&format!(
        "topic_id: {}\n",
        yaml_optional_i64(message.forum_topic_id)
    ));
    markdown.push_str(&format!(
        "topic_title: {}\n",
        yaml_optional_string(message.forum_topic_title.as_deref())
    ));
    markdown.push_str(&format!(
        "topic_top_message_id: {}\n",
        yaml_optional_i64(message.forum_topic_top_message_id)
    ));
    markdown.push_str(&format!(
        "reaction_count: {}\n",
        yaml_optional_i64(message.reaction_count)
    ));
    markdown.push_str(&format!(
        "content_kind: {}\n",
        yaml_string(&message.content_kind)
    ));
    markdown.push_str("```\n\n");

    if let Some(text) = message
        .text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        markdown.push_str("**Text:**\n");
        markdown.push_str(text);
        markdown.push_str("\n\n");
    }

    if !message.urls.is_empty() {
        markdown.push_str("**Links:**\n");
        for url in &message.urls {
            markdown.push_str(&format!("- {url}\n"));
        }
        markdown.push('\n');
    }

    if !message.media_placeholders.is_empty() {
        markdown.push_str("**Attachments:**\n");
        for placeholder in &message.media_placeholders {
            markdown.push_str(&format!("- {placeholder}\n"));
        }
        markdown.push('\n');
    }

    if let Some(reaction_count) = message.reaction_count.filter(|count| *count > 0) {
        markdown.push_str("**Reactions:**\n");
        markdown.push_str(&format!("- Total: {reaction_count}\n\n"));
    }

    let byte_size = markdown.len();
    let approximate_word_count = approx_word_count(&markdown);
    RenderedMessageBlock {
        message: message.clone(),
        markdown,
        approximate_word_count,
        byte_size,
    }
}

pub(crate) fn render_document(
    source: &NotebookLmExportSource,
    topic: &ExportTopicDescriptor,
    generated_at: i64,
    title_period: &str,
    period_start: i64,
    period_end: i64,
    participants: &[ParticipantSummary],
    blocks: &[RenderedMessageBlock],
    is_continuation: bool,
) -> String {
    let mut output = render_document_header(
        source,
        topic,
        generated_at,
        title_period,
        period_start,
        period_end,
        participants,
        blocks.len(),
        is_continuation,
    );

    for block in blocks {
        output.push_str(&block.markdown);
    }

    output
}

pub(crate) fn render_document_overhead(
    source: &NotebookLmExportSource,
    topic: &ExportTopicDescriptor,
    generated_at: i64,
    title_period: &str,
    period_start: i64,
    period_end: i64,
    participants: &[ParticipantSummary],
    message_count: usize,
    is_continuation: bool,
) -> (usize, usize) {
    let markdown = render_document_header(
        source,
        topic,
        generated_at,
        title_period,
        period_start,
        period_end,
        participants,
        message_count,
        is_continuation,
    );

    (approx_word_count(&markdown), markdown.len())
}

fn render_document_header(
    source: &NotebookLmExportSource,
    topic: &ExportTopicDescriptor,
    generated_at: i64,
    title_period: &str,
    period_start: i64,
    period_end: i64,
    participants: &[ParticipantSummary],
    message_count: usize,
    is_continuation: bool,
) -> String {
    let source_name = source.title.as_deref().unwrap_or(&source.external_id);
    let mut output = String::new();

    output.push_str(&format!(
        "# Telegram Export: {source_name} / {}\n\n",
        topic.title
    ));
    output.push_str("## Document Summary\n\n");
    output.push_str(&format!("- Source: {source_name}\n"));
    output.push_str(&format!("- Source kind: {}\n", source.telegram_source_kind));
    output.push_str(&format!("- Topic: {}\n", topic.title));
    output.push_str(&format!(
        "- Topic id: {}\n",
        yaml_optional_i64(topic.topic_id)
    ));
    output.push_str(&format!(
        "- Topic top message id: {}\n",
        yaml_optional_i64(topic.top_message_id)
    ));
    output.push_str(&format!(
        "- Export period: {} - {}\n",
        format_unix_rfc3339(period_start),
        format_unix_rfc3339(period_end)
    ));
    output.push_str(&format!(
        "- Generated at: {}\n",
        format_unix_rfc3339(generated_at)
    ));
    output.push_str(&format!("- Message count: {message_count}\n"));
    output.push_str(&format!("- Active participants: {}\n", participants.len()));
    output.push_str("- Source system: Extractum local SQLite\n");
    output.push_str("- Intended use: Google NotebookLM analysis\n");
    output.push_str(&format!("- Chunk period: {title_period}\n\n"));

    output.push_str("## Participants\n\n");
    for participant in participants.iter().take(50) {
        output.push_str(&format!(
            "- {} - {} messages\n",
            participant.author, participant.message_count
        ));
    }
    if participants.is_empty() {
        output.push_str("- Unknown - 0 messages\n");
    }
    output.push('\n');

    output.push_str("## Conversation\n\n");
    if is_continuation {
        output.push_str("> This file continues the export from the previous part.\n\n");
    }

    output
}

fn yaml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn yaml_optional_string(value: Option<&str>) -> String {
    value.map(yaml_string).unwrap_or_else(|| "null".to_string())
}

fn yaml_optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

#[cfg(test)]
mod tests {
    use super::{format_unix_rfc3339, render_document, render_message_block};
    use crate::media::ItemMediaMetadata;
    use crate::notebooklm_export::model::{
        ExportTopicDescriptor, NotebookLmExportMessage, NotebookLmExportSource,
    };

    #[test]
    fn formats_metadata_as_rfc3339() {
        assert_eq!(format_unix_rfc3339(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn renders_message_metadata_and_text() {
        let block = render_message_block(&NotebookLmExportMessage {
            item_id: 1,
            source_id: 2,
            external_id: "3".to_string(),
            author: Some("Ada".to_string()),
            published_at: 0,
            text: Some("Hello https://example.com".to_string()),
            content_kind: "text_only".to_string(),
            has_media: false,
            media_kind: None,
            media_metadata: ItemMediaMetadata::default(),
            media_placeholders: Vec::new(),
            urls: vec!["https://example.com".to_string()],
            reply_to_msg_id: None,
            reply_to_author: None,
            reply_to_snippet: None,
            reply_to_peer_kind: None,
            reply_to_peer_id: None,
            reply_to_top_id: None,
            reaction_count: None,
            forum_topic_id: Some(200),
            forum_topic_title: Some("Roadmap".to_string()),
            forum_topic_top_message_id: Some(700),
        });

        assert!(block.markdown.contains("source_id: 2"));
        assert!(block.markdown.contains("**Text:**"));
        assert!(block.markdown.contains("**Links:**"));
        assert!(block.markdown.contains("topic_id: 200"));
        assert!(block.markdown.contains("topic_title: \"Roadmap\""));
    }

    #[test]
    fn renders_reply_thread_and_reaction_metadata() {
        let block = render_message_block(&NotebookLmExportMessage {
            item_id: 1,
            source_id: 2,
            external_id: "4".to_string(),
            author: Some("Ada".to_string()),
            published_at: 0,
            text: Some("Reply".to_string()),
            content_kind: "text_only".to_string(),
            has_media: false,
            media_kind: None,
            media_metadata: ItemMediaMetadata::default(),
            media_placeholders: Vec::new(),
            urls: Vec::new(),
            reply_to_msg_id: Some(3),
            reply_to_author: Some("Bob".to_string()),
            reply_to_snippet: Some("Original text".to_string()),
            reply_to_peer_kind: Some("channel".to_string()),
            reply_to_peer_id: Some("42".to_string()),
            reply_to_top_id: Some(3),
            reaction_count: Some(2),
            forum_topic_id: Some(200),
            forum_topic_title: Some("Roadmap".to_string()),
            forum_topic_top_message_id: Some(700),
        });

        assert!(block.markdown.contains("reply_to_id: 3"));
        assert!(block.markdown.contains("reply_to_author: \"Bob\""));
        assert!(block.markdown.contains("reply_to_peer_kind: \"channel\""));
        assert!(block.markdown.contains("thread_id: 3"));
        assert!(block.markdown.contains("reaction_count: 2"));
        assert!(block.markdown.contains("**Reactions:**"));
    }

    #[test]
    fn renders_topic_aware_document_header() {
        let source = NotebookLmExportSource {
            id: 1,
            source_type: "telegram".to_string(),
            telegram_source_kind: "supergroup".to_string(),
            external_id: "123".to_string(),
            title: Some("Forum".to_string()),
        };
        let topic = ExportTopicDescriptor {
            key: "topic_200".to_string(),
            slug: "roadmap".to_string(),
            title: "Roadmap".to_string(),
            topic_id: Some(200),
            top_message_id: Some(700),
        };
        let markdown = render_document(&source, &topic, 100, "2024-01", 0, 100, &[], &[], false);

        assert!(markdown.contains("# Telegram Export: Forum / Roadmap"));
        assert!(markdown.contains("- Topic: Roadmap"));
        assert!(markdown.contains("- Topic id: 200"));
        assert!(markdown.contains("- Topic top message id: 700"));
    }
}
