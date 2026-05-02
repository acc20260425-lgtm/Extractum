use std::collections::BTreeMap;

use time::{OffsetDateTime, UtcOffset};

use crate::notebooklm_export::filename::sanitize_path_component;
use crate::notebooklm_export::model::{
    ChunkFile, NotebookLmExportMessage, NotebookLmExportSource, RenderedMessageBlock,
};

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
) -> (Vec<ChunkFile>, Vec<String>) {
    let mut warnings = Vec::new();
    let yearly_groups = group_by_period(blocks, PeriodKind::Year);
    let mut chunks = Vec::new();
    let source_slug = sanitize_path_component(
        source.title.as_deref().unwrap_or(&source.external_id),
        "source",
    );

    for yearly in yearly_groups {
        let yearly_words = block_words(&yearly.blocks);
        let yearly_bytes = block_bytes(&yearly.blocks);
        if yearly_words <= max_words && yearly_bytes <= max_bytes {
            chunks.extend(split_period(
                &source_slug,
                yearly.label,
                yearly.filename_prefix,
                yearly.blocks,
                max_words,
                max_bytes,
                &mut warnings,
            ));
            continue;
        }

        for monthly in group_by_period(&yearly.blocks, PeriodKind::Month) {
            chunks.extend(split_period(
                &source_slug,
                monthly.label,
                monthly.filename_prefix,
                monthly.blocks,
                max_words,
                max_bytes,
                &mut warnings,
            ));
        }
    }

    (chunks, warnings)
}

#[derive(Clone, Copy)]
enum PeriodKind {
    Year,
    Month,
}

struct PeriodGroup {
    label: String,
    filename_prefix: String,
    blocks: Vec<RenderedMessageBlock>,
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
    title_period: String,
    filename_prefix: String,
    blocks: Vec<RenderedMessageBlock>,
    max_words: usize,
    max_bytes: usize,
    warnings: &mut Vec<String>,
) -> Vec<ChunkFile> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_words = 0;
    let mut current_bytes = 0;

    for block in blocks {
        let exceeds_alone = block.approximate_word_count > max_words || block.byte_size > max_bytes;
        if exceeds_alone {
            warnings.push(format!(
                "Message {} exceeds configured NotebookLM file limits and was written alone.",
                block.message.external_id
            ));
        }

        let would_exceed = !current.is_empty()
            && (current_words + block.approximate_word_count > max_words
                || current_bytes + block.byte_size > max_bytes);

        if would_exceed {
            chunks.push(make_chunk(
                source_slug,
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
        filename: format!("{filename_prefix}_{source_slug}_general_part-{part_number:03}.md"),
        title_period: title_period.to_string(),
        period_start,
        period_end,
        part_number,
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

#[cfg(test)]
mod tests {
    use super::{build_chunks, should_export_message};
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
        )
        .0;
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].filename.starts_with("1970_"));
    }

    #[test]
    fn falls_back_to_month_when_year_exceeds_limits() {
        let feb_1970 = 31 * 86_400;
        let chunks = build_chunks(
            &source(),
            &[block(1, 0, 60, 10), block(2, feb_1970, 60, 10)],
            100,
            100,
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
        )
        .0;
        assert_eq!(chunks.len(), 2);

        let chunks = build_chunks(
            &source(),
            &[block(1, 0, 10, 60), block(2, 1, 10, 60)],
            100,
            100,
        )
        .0;
        assert_eq!(chunks.len(), 2);
    }
}
