use std::time::Duration;

use serde_json::Value;

use crate::error::{AppError, AppResult};

use super::dto::{YoutubeComment, YoutubeVideoMetadata};
use super::ytdlp::{run_ytdlp_with_options, YtdlpRunOptions, YTDLP_PREVIEW_TIMEOUT};

pub(crate) const DEFAULT_MAX_COMMENTS_PER_VIDEO: usize = 1_000;
pub(crate) const YOUTUBE_COMMENTS_FETCH_TIMEOUT: Duration = YTDLP_PREVIEW_TIMEOUT;

pub(crate) struct YoutubeCommentsIngest {
    pub(crate) comments: Vec<YoutubeComment>,
    pub(crate) warnings: Vec<String>,
}

pub(crate) async fn fetch_comments_for_video(
    metadata: &YoutubeVideoMetadata,
    max_comments: usize,
    sync_started_at: i64,
    cookies: Option<String>,
) -> AppResult<YoutubeCommentsIngest> {
    let output = run_ytdlp_with_options(
        &comments_fetch_args(&metadata.canonical_url, max_comments),
        YtdlpRunOptions {
            timeout: YOUTUBE_COMMENTS_FETCH_TIMEOUT,
            cookies,
        },
    )
    .await?;
    let json = ytdlp_stdout_json(&output.stdout)?;
    normalize_comments_from_ytdlp(json, max_comments, sync_started_at)
}

pub(crate) fn comments_fetch_args(canonical_url: &str, max_comments: usize) -> Vec<String> {
    vec![
        "--dump-single-json".to_string(),
        "--write-comments".to_string(),
        "--skip-download".to_string(),
        "--extractor-args".to_string(),
        format!("youtube:max_comments={max_comments}"),
        canonical_url.to_string(),
    ]
}

pub(crate) fn normalize_comments_from_ytdlp(
    value: Value,
    max_comments: usize,
    sync_started_at: i64,
) -> AppResult<YoutubeCommentsIngest> {
    let fallback_timestamp = video_published_at(&value).unwrap_or(sync_started_at);
    let raw_comments = value
        .get("comments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let raw_total = raw_comments.len();
    let mut warnings = Vec::new();
    if raw_total > max_comments {
        warnings.push(format!(
            "Comment sync truncated at {max_comments} comments."
        ));
    }

    let mut comments = Vec::new();
    for raw_comment in raw_comments.into_iter().take(max_comments) {
        normalize_comment_tree(
            raw_comment,
            None,
            fallback_timestamp,
            &mut warnings,
            &mut comments,
        );
    }

    Ok(YoutubeCommentsIngest { comments, warnings })
}

pub(crate) fn comment_published_at(raw: &Value, fallback_timestamp: i64) -> i64 {
    raw.get("timestamp")
        .and_then(timestamp_value)
        .unwrap_or(fallback_timestamp)
}

fn normalize_comment_tree(
    raw: Value,
    parent_comment_id: Option<&str>,
    fallback_timestamp: i64,
    warnings: &mut Vec<String>,
    comments: &mut Vec<YoutubeComment>,
) {
    let Some(comment_id) = first_string_field(&raw, &["id", "comment_id"]) else {
        return;
    };
    let Some(text) = first_string_field(&raw, &["text", "text_plain", "content"]) else {
        return;
    };

    if raw.get("timestamp").and_then(timestamp_value).is_none() {
        warnings.push(format!(
            "Comment {comment_id} timestamp missing or invalid; used fallback timestamp."
        ));
    }
    let replies = raw
        .get("replies")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let is_reply = parent_comment_id.is_some()
        || first_string_field(&raw, &["parent", "parent_id", "parent_comment_id"])
            .as_deref()
            .is_some_and(|parent| parent != "root");
    let parent = parent_comment_id
        .map(str::to_string)
        .or_else(|| first_string_field(&raw, &["parent_comment_id", "parent_id", "parent"]))
        .filter(|value| value != "root");

    let comment = YoutubeComment {
        comment_id: comment_id.clone(),
        parent_comment_id: parent,
        is_reply,
        author: first_string_field(&raw, &["author", "author_name"]),
        author_channel_id: first_string_field(
            &raw,
            &["author_id", "author_channel_id", "channel_id"],
        ),
        author_channel_url: first_string_field(
            &raw,
            &["author_url", "author_channel_url", "channel_url"],
        ),
        published_at: comment_published_at(&raw, fallback_timestamp),
        text,
        like_count: first_i64_field(&raw, &["like_count", "likes"]),
        is_pinned: first_bool_field(&raw, &["is_pinned", "pinned"]),
        is_hearted: first_bool_field(&raw, &["is_favorited", "is_hearted", "creator_heart"]),
        raw_payload: raw,
    };
    comments.push(comment);

    for reply in replies {
        normalize_comment_tree(
            reply,
            Some(&comment_id),
            fallback_timestamp,
            warnings,
            comments,
        );
    }
}

fn video_published_at(value: &Value) -> Option<i64> {
    value
        .get("timestamp")
        .and_then(timestamp_value)
        .or_else(|| value.get("release_timestamp").and_then(timestamp_value))
        .or_else(|| {
            first_string_field(value, &["upload_date", "release_date"])
                .as_deref()
                .and_then(ymd_to_unix_midnight)
        })
}

fn ytdlp_stdout_json(stdout: &str) -> AppResult<Value> {
    serde_json::from_str(stdout.trim())
        .map_err(|error| AppError::validation(format!("yt-dlp returned invalid JSON: {error}")))
}

fn first_string_field(value: &Value, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| string_field(value, field))
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(value_to_string)
}

fn value_to_string(value: &Value) -> Option<String> {
    if let Some(raw) = value.as_str() {
        let trimmed = raw.trim();
        return (!trimmed.is_empty()).then(|| trimmed.to_string());
    }

    value
        .as_i64()
        .map(|number| number.to_string())
        .or_else(|| value.as_u64().map(|number| number.to_string()))
}

fn first_i64_field(value: &Value, fields: &[&str]) -> Option<i64> {
    fields
        .iter()
        .find_map(|field| value.get(field).and_then(i64_value))
}

fn i64_value(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|number| i64::try_from(number).ok()))
        .or_else(|| value.as_str()?.trim().parse::<i64>().ok())
}

fn timestamp_value(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_str()?.trim().parse::<i64>().ok())
}

fn first_bool_field(value: &Value, fields: &[&str]) -> Option<bool> {
    fields
        .iter()
        .find_map(|field| value.get(field).and_then(bool_value))
}

fn bool_value(value: &Value) -> Option<bool> {
    value.as_bool().or_else(|| {
        let raw = value.as_str()?.trim().to_ascii_lowercase();
        match raw.as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        }
    })
}

fn ymd_to_unix_midnight(value: &str) -> Option<i64> {
    let compact = value.trim();
    let normalized = if compact.len() == 8 && compact.chars().all(|ch| ch.is_ascii_digit()) {
        format!("{}-{}-{}", &compact[0..4], &compact[4..6], &compact[6..8])
    } else {
        compact.to_string()
    };
    let mut parts = normalized.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        comment_published_at, comments_fetch_args, normalize_comments_from_ytdlp,
        DEFAULT_MAX_COMMENTS_PER_VIDEO,
    };

    #[test]
    fn comments_fetch_args_include_bounded_extractor_args() {
        let args = comments_fetch_args("https://www.youtube.com/watch?v=abc123", 250);

        assert_eq!(
            args,
            vec![
                "--dump-single-json",
                "--write-comments",
                "--skip-download",
                "--extractor-args",
                "youtube:max_comments=250",
                "https://www.youtube.com/watch?v=abc123"
            ]
        );
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--extractor-args", "youtube:max_comments=250"]));
    }

    #[test]
    fn default_comment_limit_is_bounded() {
        assert_eq!(DEFAULT_MAX_COMMENTS_PER_VIDEO, 1_000);
    }

    #[test]
    fn comment_published_at_accepts_numbers_strings_and_fallback() {
        assert_eq!(
            comment_published_at(&json!({ "timestamp": 1_700_000_000 }), 42),
            1_700_000_000
        );
        assert_eq!(
            comment_published_at(&json!({ "timestamp": "1700000001" }), 42),
            1_700_000_001
        );
        assert_eq!(comment_published_at(&json!({}), 42), 42);
        assert_eq!(
            comment_published_at(&json!({ "timestamp": "soon" }), 42),
            42
        );
    }

    #[test]
    fn normalize_comments_flattens_replies_and_warns_for_timestamp_fallbacks() {
        let fixture = json!({
            "id": "abc123",
            "timestamp": 1_690_000_000,
            "comments": [
                {
                    "id": "top1",
                    "text": "Top comment",
                    "author": "Alice",
                    "author_id": "UCalice",
                    "author_url": "https://www.youtube.com/@alice",
                    "timestamp": "1700000000",
                    "like_count": 7,
                    "is_pinned": true,
                    "is_favorited": true,
                    "replies": [
                        {
                            "id": "reply1",
                            "text": "Reply text",
                            "author": "Bob",
                            "timestamp": "not-a-timestamp",
                            "like_count": 2
                        }
                    ]
                },
                {
                    "id": "top2",
                    "text": "Second top",
                    "author": "Carol"
                }
            ]
        });

        let result =
            normalize_comments_from_ytdlp(fixture, 10, 1_680_000_000).expect("normalize comments");

        assert_eq!(
            result
                .comments
                .iter()
                .map(|comment| comment.comment_id.as_str())
                .collect::<Vec<_>>(),
            vec!["top1", "reply1", "top2"]
        );
        assert_eq!(result.comments[0].parent_comment_id, None);
        assert!(!result.comments[0].is_reply);
        assert_eq!(result.comments[0].published_at, 1_700_000_000);
        assert_eq!(result.comments[0].like_count, Some(7));
        assert_eq!(result.comments[0].is_pinned, Some(true));
        assert_eq!(result.comments[0].is_hearted, Some(true));
        assert_eq!(
            result.comments[0].author_channel_id.as_deref(),
            Some("UCalice")
        );
        assert_eq!(
            result.comments[0].author_channel_url.as_deref(),
            Some("https://www.youtube.com/@alice")
        );

        assert_eq!(
            result.comments[1].parent_comment_id.as_deref(),
            Some("top1")
        );
        assert!(result.comments[1].is_reply);
        assert_eq!(result.comments[1].published_at, 1_690_000_000);

        assert_eq!(result.comments[2].published_at, 1_690_000_000);
        assert_eq!(
            result.warnings,
            vec![
                "Comment reply1 timestamp missing or invalid; used fallback timestamp.",
                "Comment top2 timestamp missing or invalid; used fallback timestamp.",
            ]
        );
    }

    #[test]
    fn normalize_comments_truncates_raw_comment_array_before_normalization() {
        let fixture = json!({
            "id": "abc123",
            "timestamp": 1_690_000_000,
            "comments": [
                { "id": "one", "text": "One", "timestamp": 1 },
                { "id": "two", "text": "Two", "timestamp": 2 }
            ]
        });

        let result =
            normalize_comments_from_ytdlp(fixture, 1, 1_680_000_000).expect("normalize comments");

        assert_eq!(result.comments.len(), 1);
        assert_eq!(result.comments[0].comment_id, "one");
        assert_eq!(
            result.warnings,
            vec!["Comment sync truncated at 1 comments."]
        );
    }
}
