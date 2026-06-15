use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub(crate) struct SourceRow {
    pub(crate) id: i64,
    pub(crate) source_type: String,
    pub(crate) source_subtype: Option<String>,
    pub(crate) title: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct VideoCandidate {
    pub(crate) source_id: i64,
    pub(crate) video_id: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) is_playlist_child: bool,
}

pub(crate) enum PlaylistCandidate {
    Linked(VideoCandidate),
    Unlinked {
        video_id: String,
        title: Option<String>,
    },
}

pub(crate) async fn load_source(pool: &SqlitePool, source_id: i64) -> AppResult<Option<SourceRow>> {
    sqlx::query_as::<_, (i64, String, Option<String>, Option<String>)>(
        "SELECT id, source_type, source_subtype, title FROM sources WHERE id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|(id, source_type, source_subtype, title)| SourceRow {
            id,
            source_type,
            source_subtype,
            title,
        })
    })
    .map_err(AppError::database)
}

pub(crate) async fn load_video_candidate(
    pool: &SqlitePool,
    source_id: i64,
    is_playlist_child: bool,
) -> AppResult<Option<VideoCandidate>> {
    sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT video_id, title, description FROM youtube_video_sources WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|(video_id, title, description)| VideoCandidate {
            source_id,
            title: title.unwrap_or_else(|| video_id.clone()),
            video_id,
            description,
            is_playlist_child,
        })
    })
    .map_err(AppError::database)
}

pub(crate) async fn load_playlist_candidates(
    pool: &SqlitePool,
    playlist_source_id: i64,
) -> AppResult<Vec<PlaylistCandidate>> {
    let rows = sqlx::query_as::<_, (Option<i64>, String, Option<String>)>(
        "SELECT video_source_id, video_id, title_snapshot
         FROM youtube_playlist_items
         WHERE playlist_source_id = ? AND is_removed_from_playlist = 0
         ORDER BY position ASC, id ASC",
    )
    .bind(playlist_source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut candidates = Vec::with_capacity(rows.len());
    for (video_source_id, video_id, title) in rows {
        if let Some(source_id) = video_source_id {
            if let Some(video) = load_video_candidate(pool, source_id, true).await? {
                candidates.push(PlaylistCandidate::Linked(video));
            } else {
                candidates.push(PlaylistCandidate::Unlinked { video_id, title });
            }
        } else {
            candidates.push(PlaylistCandidate::Unlinked { video_id, title });
        }
    }
    Ok(candidates)
}

pub(crate) async fn transcript_text_for_source(
    pool: &SqlitePool,
    source_id: i64,
) -> AppResult<String> {
    let segments = sqlx::query_scalar::<_, String>(
        "SELECT text
         FROM youtube_transcript_segments
         WHERE source_id = ?
         ORDER BY segment_index ASC, id ASC",
    )
    .bind(source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    Ok(segments.join("\n"))
}
