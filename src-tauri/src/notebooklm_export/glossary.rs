use std::collections::BTreeMap;

use crate::notebooklm_export::model::{NotebookLmExportMessage, ParticipantSummary};

use super::renderer::{approx_word_count, format_unix_rfc3339};

pub(crate) fn aggregate_participants(
    messages: &[NotebookLmExportMessage],
) -> Vec<ParticipantSummary> {
    let mut participants: BTreeMap<String, ParticipantSummary> = BTreeMap::new();

    for message in messages {
        let author = message
            .author
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Unknown")
            .to_string();

        participants
            .entry(author.clone())
            .and_modify(|summary| {
                summary.message_count += 1;
                summary.first_seen = summary.first_seen.min(message.published_at);
                summary.last_seen = summary.last_seen.max(message.published_at);
            })
            .or_insert_with(|| ParticipantSummary {
                author,
                message_count: 1,
                first_seen: message.published_at,
                last_seen: message.published_at,
            });
    }

    let mut summaries = participants.into_values().collect::<Vec<_>>();
    summaries.sort_by(|left, right| {
        right
            .message_count
            .cmp(&left.message_count)
            .then_with(|| left.author.cmp(&right.author))
    });
    summaries
}

pub(crate) fn render_glossary(
    generated_at: i64,
    source_name: &str,
    participants: &[ParticipantSummary],
) -> String {
    let mut output = String::new();
    output.push_str("# Telegram Export Glossary\n\n");
    output.push_str(&format!(
        "Generated at: {}\n\n",
        format_unix_rfc3339(generated_at)
    ));
    output.push_str(&format!("Source: {source_name}\n\n"));
    output.push_str("## Participants\n\n");

    if participants.is_empty() {
        output.push_str("No participants were exported.\n");
        return output;
    }

    for participant in participants {
        output.push_str(&format!("### {}\n\n", participant.author));
        output.push_str("- Telegram ID: null\n");
        output.push_str(&format!("- Message count: {}\n", participant.message_count));
        output.push_str(&format!(
            "- First seen: {}\n",
            format_unix_rfc3339(participant.first_seen)
        ));
        output.push_str(&format!(
            "- Last seen: {}\n",
            format_unix_rfc3339(participant.last_seen)
        ));
        output.push_str("- Detected role: unknown\n\n");
    }

    output
}

pub(crate) fn glossary_word_count(markdown: &str) -> usize {
    approx_word_count(markdown)
}

#[cfg(test)]
mod tests {
    use super::aggregate_participants;
    use crate::media::ItemMediaMetadata;
    use crate::notebooklm_export::model::NotebookLmExportMessage;

    fn message(author: &str, published_at: i64) -> NotebookLmExportMessage {
        NotebookLmExportMessage {
            item_id: published_at,
            source_id: 1,
            external_id: published_at.to_string(),
            author: Some(author.to_string()),
            published_at,
            text: Some("hello".to_string()),
            content_kind: "text_only".to_string(),
            has_media: false,
            media_kind: None,
            media_metadata: ItemMediaMetadata::default(),
            media_placeholders: Vec::new(),
            urls: Vec::new(),
            reply_to_msg_id: None,
            reply_to_author: None,
            reply_to_snippet: None,
            reply_to_peer_kind: None,
            reply_to_peer_id: None,
            reply_to_top_id: None,
            reaction_count: None,
            forum_topic_id: None,
            forum_topic_title: None,
            forum_topic_top_message_id: None,
        }
    }

    #[test]
    fn aggregates_participants_by_author() {
        let participants =
            aggregate_participants(&[message("Ada", 10), message("Bob", 11), message("Ada", 12)]);

        assert_eq!(participants[0].author, "Ada");
        assert_eq!(participants[0].message_count, 2);
        assert_eq!(participants[0].first_seen, 10);
        assert_eq!(participants[0].last_seen, 12);
    }
}
