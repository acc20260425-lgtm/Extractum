use std::collections::HashMap;

use sqlx::FromRow;

use crate::compression::decompress_text;
use crate::error::{AppError, AppResult};
use crate::media::decode_media_metadata;
use crate::notebooklm_export::links::detect_urls;
use crate::notebooklm_export::media::render_media_placeholders;
use crate::notebooklm_export::model::NotebookLmExportMessage;

#[derive(FromRow)]
pub(super) struct ExportMessageRow {
    pub(super) id: i64,
    pub(super) source_id: i64,
    pub(super) external_id: String,
    pub(super) author: Option<String>,
    pub(super) published_at: i64,
    pub(super) content_zstd: Option<Vec<u8>>,
    pub(super) content_kind: String,
    pub(super) has_media: bool,
    pub(super) media_kind: Option<String>,
    pub(super) media_metadata_zstd: Option<Vec<u8>>,
    pub(super) reply_to_msg_id: Option<i64>,
    pub(super) reply_to_peer_kind: Option<String>,
    pub(super) reply_to_peer_id: Option<String>,
    pub(super) reply_to_top_id: Option<i64>,
    pub(super) reaction_count: Option<i64>,
    pub(super) forum_topic_id: Option<i64>,
    pub(super) forum_topic_title: Option<String>,
    pub(super) forum_topic_top_message_id: Option<i64>,
}

#[derive(FromRow)]
pub(super) struct ReplyLookupRow {
    pub(super) external_id: String,
    pub(super) author: Option<String>,
    pub(super) content_zstd: Option<Vec<u8>>,
    pub(super) has_media: bool,
    pub(super) media_kind: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReplyContext {
    pub(super) author: Option<String>,
    pub(super) snippet: String,
}

pub(super) fn map_export_rows(
    rows: Vec<ExportMessageRow>,
    reply_contexts: HashMap<i64, ReplyContext>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    rows.into_iter()
        .map(|row| -> AppResult<NotebookLmExportMessage> {
            let text = row
                .content_zstd
                .as_deref()
                .map(decompress_text)
                .transpose()
                .map_err(AppError::internal)?;
            let urls = text.as_deref().map(detect_urls).unwrap_or_default();
            let media_metadata = decode_media_metadata(row.media_metadata_zstd.as_deref())?;
            let media_placeholders =
                render_media_placeholders(row.media_kind.as_deref(), &media_metadata);
            let reply_context = row
                .reply_to_msg_id
                .and_then(|reply_to_msg_id| reply_contexts.get(&reply_to_msg_id));

            Ok(NotebookLmExportMessage {
                item_id: row.id,
                source_id: row.source_id,
                external_id: row.external_id,
                author: row.author,
                published_at: row.published_at,
                text,
                content_kind: row.content_kind,
                has_media: row.has_media,
                media_kind: row.media_kind,
                media_metadata,
                media_placeholders,
                urls,
                reply_to_msg_id: row.reply_to_msg_id,
                reply_to_author: reply_context.and_then(|context| context.author.clone()),
                reply_to_snippet: row.reply_to_msg_id.map(|_| {
                    reply_context
                        .map(|context| context.snippet.clone())
                        .unwrap_or_else(|| "Original message unavailable".to_string())
                }),
                reply_to_peer_kind: row.reply_to_peer_kind,
                reply_to_peer_id: row.reply_to_peer_id,
                reply_to_top_id: row.reply_to_top_id,
                reaction_count: row.reaction_count,
                forum_topic_id: row.forum_topic_id,
                forum_topic_title: row.forum_topic_title,
                forum_topic_top_message_id: row.forum_topic_top_message_id,
            })
        })
        .collect()
}

pub(super) fn reply_snippet(row: &ReplyLookupRow) -> AppResult<String> {
    let text = row
        .content_zstd
        .as_deref()
        .map(decompress_text)
        .transpose()
        .map_err(AppError::internal)?;

    if let Some(text) = text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Ok(truncate_snippet(&collapse_whitespace(text), 280));
    }

    if row.has_media {
        return Ok(format!(
            "[Media message: {}]",
            row.media_kind.as_deref().unwrap_or("media")
        ));
    }

    Ok("[Message has no text]".to_string())
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_snippet(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let snippet = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{snippet}...")
    } else {
        snippet
    }
}

#[cfg(test)]
mod tests {
    use super::{reply_snippet, ReplyLookupRow};
    use crate::error::AppErrorKind;

    #[test]
    fn reply_snippet_decode_failures_are_typed_internal_errors() {
        let row = ReplyLookupRow {
            external_id: "1".to_string(),
            author: None,
            content_zstd: Some(vec![0x00]),
            has_media: false,
            media_kind: None,
        };

        let error = reply_snippet(&row).expect_err("reject corrupt reply content");

        assert_eq!(error.kind, AppErrorKind::Internal);
    }
}
