use std::collections::BTreeMap;

use time::{OffsetDateTime, UtcOffset};

use crate::notebooklm_export::filename::sanitize_path_component;
use crate::notebooklm_export::model::{
    ChunkFile, ExportTopicDescriptor, NotebookLmExportMessage, NotebookLmExportSource,
    RenderedMessageBlock,
};

const GENERAL_UNCATEGORIZED_SLUG: &str = "general_uncategorized";
const GENERAL_UNCATEGORIZED_TITLE: &str = "General / Uncategorized";

pub(crate) fn should_export_message(
    message: &NotebookLmExportMessage,
    min_message_length: usize,
    include_media_placeholders: bool,
) -> bool {
    let text_len = message
        .text
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .chars()
        .count();
    let has_meaningful_text = text_len >= min_message_length;
    let has_url = !message.urls.is_empty();
    let has_media = message.has_media && !message.media_placeholders.is_empty();
    let text_bearing_media = message.content_kind == "text_with_media" && text_len > 0;
    let meaningful_non_text =
        include_media_placeholders && message.content_kind == "media_only" && has_media;

    has_meaningful_text || has_url || text_bearing_media || meaningful_non_text
}

pub(crate) fn build_chunks(
    source: &NotebookLmExportSource,
    blocks: &[RenderedMessageBlock],
    max_words: usize,
    max_bytes: usize,
    document_overhead: impl Fn(&ExportTopicDescriptor, &str, i64, i64, bool, usize) -> (usize, usize),
) -> (Vec<ChunkFile>, Vec<String>) {
    let mut warnings = Vec::new();
    let mut chunks = Vec::new();
    let source_slug = sanitize_path_component(
        source.title.as_deref().unwrap_or(&source.external_id),
        "source",
    );

    for topic_group in group_by_topic(blocks) {
        let yearly_groups = group_by_period(&topic_group.blocks, PeriodKind::Year);
        for yearly in yearly_groups {
            let yearly_words = block_words(&yearly.blocks);
            let yearly_bytes = block_bytes(&yearly.blocks);
            let (overhead_words, overhead_bytes) =
                period_overhead(&topic_group.topic, &yearly, false, &document_overhead);
            if yearly_words + overhead_words <= max_words
                && yearly_bytes + overhead_bytes <= max_bytes
            {
                chunks.extend(split_period(
                    &source_slug,
                    &topic_group.topic,
                    yearly.label,
                    yearly.filename_prefix,
                    yearly.blocks,
                    max_words,
                    max_bytes,
                    &document_overhead,
                    &mut warnings,
                ));
                continue;
            }

            for monthly in group_by_period(&yearly.blocks, PeriodKind::Month) {
                chunks.extend(split_period(
                    &source_slug,
                    &topic_group.topic,
                    monthly.label,
                    monthly.filename_prefix,
                    monthly.blocks,
                    max_words,
                    max_bytes,
                    &document_overhead,
                    &mut warnings,
                ));
            }
        }
    }

    (chunks, warnings)
}

#[derive(Clone, Copy)]
enum PeriodKind {
    Year,
    Month,
}

struct TopicGroup {
    topic: ExportTopicDescriptor,
    blocks: Vec<RenderedMessageBlock>,
}

struct PeriodGroup {
    label: String,
    filename_prefix: String,
    blocks: Vec<RenderedMessageBlock>,
}

fn group_by_topic(blocks: &[RenderedMessageBlock]) -> Vec<TopicGroup> {
    let mut grouped: BTreeMap<String, (ExportTopicDescriptor, Vec<RenderedMessageBlock>)> =
        BTreeMap::new();

    for block in blocks {
        let topic = topic_descriptor(&block.message);
        grouped
            .entry(topic.key.clone())
            .or_insert_with(|| (topic.clone(), Vec::new()))
            .1
            .push(block.clone());
    }

    grouped
        .into_iter()
        .map(|(_, (topic, blocks))| TopicGroup { topic, blocks })
        .collect()
}

fn topic_descriptor(message: &NotebookLmExportMessage) -> ExportTopicDescriptor {
    if let Some(topic_id) = message.forum_topic_id {
        let title = message
            .forum_topic_title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Untitled Topic")
            .to_string();
        let slug = sanitize_path_component(&title, &format!("topic_{topic_id}"));
        return ExportTopicDescriptor {
            key: format!("topic_{topic_id}"),
            slug,
            title,
            topic_id: Some(topic_id),
            top_message_id: message.forum_topic_top_message_id,
        };
    }

    ExportTopicDescriptor {
        key: GENERAL_UNCATEGORIZED_SLUG.to_string(),
        slug: GENERAL_UNCATEGORIZED_SLUG.to_string(),
        title: GENERAL_UNCATEGORIZED_TITLE.to_string(),
        topic_id: None,
        top_message_id: None,
    }
}

fn group_by_period(blocks: &[RenderedMessageBlock], kind: PeriodKind) -> Vec<PeriodGroup> {
    let mut grouped: BTreeMap<String, Vec<RenderedMessageBlock>> = BTreeMap::new();
    for block in blocks {
        grouped
            .entry(period_key(block.message.published_at, kind))
            .or_default()
            .push(block.clone());
    }

    grouped
        .into_iter()
        .map(|(key, blocks)| PeriodGroup {
            label: key.clone(),
            filename_prefix: key,
            blocks,
        })
        .collect()
}

fn period_key(unix: i64, kind: PeriodKind) -> String {
    let value = OffsetDateTime::from_unix_timestamp(unix)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .to_offset(UtcOffset::UTC);
    match kind {
        PeriodKind::Year => format!("{:04}", value.year()),
        PeriodKind::Month => format!("{:04}-{:02}", value.year(), u8::from(value.month())),
    }
}

fn split_period(
    source_slug: &str,
    topic: &ExportTopicDescriptor,
    title_period: String,
    filename_prefix: String,
    blocks: Vec<RenderedMessageBlock>,
    max_words: usize,
    max_bytes: usize,
    document_overhead: &impl Fn(&ExportTopicDescriptor, &str, i64, i64, bool, usize) -> (usize, usize),
    warnings: &mut Vec<String>,
) -> Vec<ChunkFile> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_words = 0;
    let mut current_bytes = 0;

    for block in blocks {
        let is_next_continuation = !chunks.is_empty();
        let (single_overhead_words, single_overhead_bytes) = document_overhead(
            topic,
            &title_period,
            block.message.published_at,
            block.message.published_at,
            is_next_continuation,
            1,
        );
        let exceeds_alone = block.approximate_word_count + single_overhead_words > max_words
            || block.byte_size + single_overhead_bytes > max_bytes;
        if exceeds_alone {
            warnings.push(format!(
                "Message {} exceeds configured NotebookLM file limits and was written alone.",
                block.message.external_id
            ));
        }

        let would_exceed = if current.is_empty() {
            false
        } else {
            let period_start = current
                .first()
                .map(|block: &RenderedMessageBlock| block.message.published_at)
                .unwrap_or(block.message.published_at);
            let period_end = block.message.published_at;
            let (overhead_words, overhead_bytes) = document_overhead(
                topic,
                &title_period,
                period_start,
                period_end,
                !chunks.is_empty(),
                current.len() + 1,
            );

            current_words + block.approximate_word_count + overhead_words > max_words
                || current_bytes + block.byte_size + overhead_bytes > max_bytes
        };

        if would_exceed {
            chunks.push(make_chunk(
                source_slug,
                topic,
                &title_period,
                &filename_prefix,
                chunks.len() + 1,
                std::mem::take(&mut current),
            ));
            current_words = 0;
            current_bytes = 0;
        }

        current_words += block.approximate_word_count;
        current_bytes += block.byte_size;
        current.push(block);
    }

    if !current.is_empty() {
        chunks.push(make_chunk(
            source_slug,
            topic,
            &title_period,
            &filename_prefix,
            chunks.len() + 1,
            current,
        ));
    }

    chunks
}

fn make_chunk(
    source_slug: &str,
    topic: &ExportTopicDescriptor,
    title_period: &str,
    filename_prefix: &str,
    part_number: usize,
    blocks: Vec<RenderedMessageBlock>,
) -> ChunkFile {
    let period_start = blocks
        .first()
        .map(|block| block.message.published_at)
        .unwrap_or(0);
    let period_end = blocks
        .last()
        .map(|block| block.message.published_at)
        .unwrap_or(0);
    ChunkFile {
        filename: format!(
            "{filename_prefix}_{source_slug}_{}_part-{part_number:03}.md",
            topic.slug
        ),
        title_period: title_period.to_string(),
        period_start,
        period_end,
        part_number,
        topic: topic.clone(),
        blocks,
    }
}

fn block_words(blocks: &[RenderedMessageBlock]) -> usize {
    blocks
        .iter()
        .map(|block| block.approximate_word_count)
        .sum()
}

fn block_bytes(blocks: &[RenderedMessageBlock]) -> usize {
    blocks.iter().map(|block| block.byte_size).sum()
}

fn period_overhead(
    topic: &ExportTopicDescriptor,
    group: &PeriodGroup,
    is_continuation: bool,
    document_overhead: &impl Fn(&ExportTopicDescriptor, &str, i64, i64, bool, usize) -> (usize, usize),
) -> (usize, usize) {
    let period_start = group
        .blocks
        .first()
        .map(|block| block.message.published_at)
        .unwrap_or(0);
    let period_end = group
        .blocks
        .last()
        .map(|block| block.message.published_at)
        .unwrap_or(0);
    document_overhead(
        topic,
        &group.label,
        period_start,
        period_end,
        is_continuation,
        group.blocks.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::{build_chunks, should_export_message, GENERAL_UNCATEGORIZED_SLUG};
    use crate::media::ItemMediaMetadata;
    use crate::notebooklm_export::model::{
        NotebookLmExportMessage, NotebookLmExportSource, RenderedMessageBlock,
    };

    fn block(id: i64, published_at: i64, words: usize, bytes: usize) -> RenderedMessageBlock {
        RenderedMessageBlock {
            message: NotebookLmExportMessage {
                item_id: id,
                source_id: 1,
                external_id: id.to_string(),
                author: Some("Ada".to_string()),
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
            },
            markdown: "hello".to_string(),
            approximate_word_count: words,
            byte_size: bytes,
        }
    }

    fn source() -> NotebookLmExportSource {
        NotebookLmExportSource {
            id: 1,
            source_type: "telegram".to_string(),
            telegram_source_kind: "channel".to_string(),
            external_id: "123".to_string(),
            title: Some("My Source".to_string()),
        }
    }

    #[test]
    fn filters_short_text_without_other_signal() {
        let mut message = block(1, 0, 1, 1).message;
        message.text = Some("ok".to_string());
        assert!(!should_export_message(&message, 3, true));
        message.urls = vec!["https://example.com".to_string()];
        assert!(should_export_message(&message, 3, true));
    }

    #[test]
    fn keeps_yearly_group_when_within_limits() {
        let chunks = build_chunks(
            &source(),
            &[block(1, 0, 10, 10), block(2, 10, 10, 10)],
            100,
            100,
            |_, _, _, _, _, _| (0, 0),
        )
        .0;
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].filename.starts_with("1970_"));
        assert!(chunks[0].filename.contains(GENERAL_UNCATEGORIZED_SLUG));
    }

    #[test]
    fn falls_back_to_month_when_year_exceeds_limits() {
        let feb_1970 = 31 * 86_400;
        let chunks = build_chunks(
            &source(),
            &[block(1, 0, 60, 10), block(2, feb_1970, 60, 10)],
            100,
            100,
            |_, _, _, _, _, _| (0, 0),
        )
        .0;
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].filename.starts_with("1970-01_"));
        assert!(chunks[1].filename.starts_with("1970-02_"));
    }

    #[test]
    fn splits_by_word_and_byte_limits() {
        let chunks = build_chunks(
            &source(),
            &[block(1, 0, 60, 10), block(2, 1, 60, 10)],
            100,
            100,
            |_, _, _, _, _, _| (0, 0),
        )
        .0;
        assert_eq!(chunks.len(), 2);

        let chunks = build_chunks(
            &source(),
            &[block(1, 0, 10, 60), block(2, 1, 10, 60)],
            100,
            100,
            |_, _, _, _, _, _| (0, 0),
        )
        .0;
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn accounts_for_document_overhead_when_splitting() {
        let chunks = build_chunks(
            &source(),
            &[block(1, 0, 40, 10), block(2, 1, 40, 10)],
            100,
            100,
            |_, _, _, _, _, _| (30, 0),
        )
        .0;

        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn groups_chunks_by_topic_slug() {
        let mut topic_a = block(1, 0, 10, 10);
        topic_a.message.forum_topic_id = Some(200);
        topic_a.message.forum_topic_title = Some("Roadmap".to_string());
        topic_a.message.forum_topic_top_message_id = Some(700);

        let mut topic_b = block(2, 10, 10, 10);
        topic_b.message.forum_topic_id = Some(201);
        topic_b.message.forum_topic_title = Some("Bugs".to_string());
        topic_b.message.forum_topic_top_message_id = Some(701);

        let uncategorized = block(3, 20, 10, 10);

        let chunks = build_chunks(
            &source(),
            &[topic_a, topic_b, uncategorized],
            100,
            100,
            |_, _, _, _, _, _| (0, 0),
        )
        .0;

        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().any(|chunk| chunk.filename.contains("_roadmap_")));
        assert!(chunks.iter().any(|chunk| chunk.filename.contains("_bugs_")));
        assert!(chunks
            .iter()
            .any(|chunk| chunk.filename.contains(GENERAL_UNCATEGORIZED_SLUG)));
    }

    #[test]
    fn falls_back_to_topic_id_when_topic_title_slug_is_invalid() {
        let mut topic = block(1, 0, 10, 10);
        topic.message.forum_topic_id = Some(200);
        topic.message.forum_topic_title = Some("..".to_string());
        topic.message.forum_topic_top_message_id = Some(700);

        let chunks = build_chunks(&source(), &[topic], 100, 100, |_, _, _, _, _, _| (0, 0)).0;

        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].filename.contains("_topic_200_"));
        assert_eq!(chunks[0].topic.slug, "topic_200");
        assert_eq!(chunks[0].topic.title, "..");
    }
}
