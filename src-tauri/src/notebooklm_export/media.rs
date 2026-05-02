use crate::media::{media_label, ItemMediaMetadata};

pub(crate) fn render_media_placeholders(
    media_kind: Option<&str>,
    metadata: &ItemMediaMetadata,
) -> Vec<String> {
    if media_kind.is_none()
        && metadata.summary.is_none()
        && metadata.file_name.is_none()
        && metadata.mime_type.is_none()
    {
        return Vec::new();
    }

    let kind = media_kind.unwrap_or("media");
    let mut details = Vec::new();
    let label = media_label(kind);
    details.push(label.to_string());

    if let Some(file_name) = metadata
        .file_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        details.push(file_name.trim().to_string());
    }
    if let Some(mime_type) = metadata
        .mime_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        details.push(mime_type.trim().to_string());
    }
    if let Some(summary) = metadata
        .summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        if !details.iter().any(|detail| detail == summary.trim()) {
            details.push(summary.trim().to_string());
        }
    }
    if let Some(duration) = metadata.duration_seconds {
        details.push(format!("duration {:.0}s", duration));
    }
    if let (Some(width), Some(height)) = (metadata.width, metadata.height) {
        details.push(format!("{width}x{height}"));
    }
    if let Some(size) = metadata.size_bytes {
        details.push(format!("{size} bytes"));
    }

    vec![format!("[Attachment: {}]", details.join(" - "))]
}

#[cfg(test)]
mod tests {
    use super::render_media_placeholders;
    use crate::media::ItemMediaMetadata;

    #[test]
    fn renders_useful_media_placeholder_parts() {
        let metadata = ItemMediaMetadata {
            summary: Some("Video".to_string()),
            file_name: Some("clip.mp4".to_string()),
            mime_type: Some("video/mp4".to_string()),
            duration_seconds: Some(12.0),
            ..ItemMediaMetadata::default()
        };

        assert_eq!(
            render_media_placeholders(Some("video"), &metadata),
            vec!["[Attachment: Video - clip.mp4 - video/mp4 - duration 12s]"]
        );
    }
}
