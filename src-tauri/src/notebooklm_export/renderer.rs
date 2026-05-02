use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::notebooklm_export::model::{
    NotebookLmExportMessage, NotebookLmExportSource, ParticipantSummary, RenderedMessageBlock,
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
    markdown.push_str("reply_to_id: null\n");
    markdown.push_str("reply_to_author: null\n");
    markdown.push_str("reply_to_snippet: null\n");
    markdown.push_str("forwarded_from: null\n");
    markdown.push_str("thread_id: null\n");
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

    output.push_str(&format!("# Telegram Export: {source_name} / general\n\n"));
    output.push_str("## Document Summary\n\n");
    output.push_str(&format!("- Source: {source_name}\n"));
    output.push_str(&format!("- Source kind: {}\n", source.telegram_source_kind));
    output.push_str("- Topic: general\n");
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

#[cfg(test)]
mod tests {
    use super::{format_unix_rfc3339, render_message_block};
    use crate::media::ItemMediaMetadata;
    use crate::notebooklm_export::model::NotebookLmExportMessage;

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
        });

        assert!(block.markdown.contains("source_id: 2"));
        assert!(block.markdown.contains("**Text:**"));
        assert!(block.markdown.contains("**Links:**"));
    }
}
