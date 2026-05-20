use grammers_client::{media::Media, tl};
use serde::{Deserialize, Serialize};

use crate::compression::{compress_json_bytes, decompress_bytes};
use crate::error::{AppError, AppResult};

pub(crate) const CONTENT_KIND_TEXT_ONLY: &str = "text_only";
pub(crate) const CONTENT_KIND_TEXT_WITH_MEDIA: &str = "text_with_media";
pub(crate) const CONTENT_KIND_MEDIA_ONLY: &str = "media_only";

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct ItemMediaMetadata {
    pub(crate) summary: Option<String>,
    pub(crate) file_name: Option<String>,
    pub(crate) mime_type: Option<String>,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) width: Option<i32>,
    pub(crate) height: Option<i32>,
    pub(crate) duration_seconds: Option<f64>,
}

pub(crate) struct ExtractedMediaPayload {
    pub(crate) kind: String,
    pub(crate) metadata: ItemMediaMetadata,
}

pub(crate) struct ExtractedItemPayload {
    pub(crate) content: Option<String>,
    pub(crate) content_kind: &'static str,
    pub(crate) media: Option<ExtractedMediaPayload>,
}

pub(crate) fn encode_media_metadata(metadata: &ItemMediaMetadata) -> AppResult<Vec<u8>> {
    let json =
        serde_json::to_vec(metadata).map_err(|error| AppError::internal(error.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

pub(crate) fn decode_media_metadata(bytes: Option<&[u8]>) -> AppResult<ItemMediaMetadata> {
    let Some(bytes) = bytes else {
        return Ok(ItemMediaMetadata::default());
    };
    let decoded = decompress_bytes(bytes).map_err(AppError::internal)?;
    serde_json::from_slice(&decoded).map_err(|error| AppError::internal(error.to_string()))
}

#[derive(Default)]
pub(crate) struct DocumentSignals {
    pub(crate) mime_type: Option<String>,
    pub(crate) has_video: bool,
    pub(crate) has_audio: bool,
    pub(crate) is_voice: bool,
    pub(crate) is_animated: bool,
}

fn trimmed_non_empty(input: &str) -> Option<String> {
    let trimmed = input.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(crate) fn media_label(kind: &str) -> &'static str {
    match kind {
        "photo" => "Photo",
        "video" => "Video",
        "audio" => "Audio",
        "voice" => "Voice message",
        "image" => "Image",
        "animation" => "Animation",
        "sticker" => "Sticker",
        "contact" => "Contact card",
        "poll" => "Poll",
        "location" => "Location",
        "live_location" => "Live location",
        "venue" => "Venue",
        "webpage" => "Web page preview",
        "dice" => "Dice",
        _ => "Document",
    }
}

pub(crate) fn derive_content_kind(has_content: bool, has_media: bool) -> &'static str {
    match (has_content, has_media) {
        (true, true) => CONTENT_KIND_TEXT_WITH_MEDIA,
        (false, true) => CONTENT_KIND_MEDIA_ONLY,
        _ => CONTENT_KIND_TEXT_ONLY,
    }
}

fn collect_document_signals(document: &grammers_client::media::Document) -> DocumentSignals {
    let mut signals = DocumentSignals {
        mime_type: document.mime_type().map(str::to_string),
        is_animated: document.is_animated(),
        ..DocumentSignals::default()
    };

    if let Some(tl::enums::Document::Document(raw_document)) = document.raw.document.as_ref() {
        for attribute in &raw_document.attributes {
            match attribute {
                tl::enums::DocumentAttribute::Video(_) => signals.has_video = true,
                tl::enums::DocumentAttribute::Audio(audio) => {
                    signals.has_audio = true;
                    signals.is_voice = audio.voice;
                }
                _ => {}
            }
        }
    }

    signals
}

pub(crate) fn derive_document_media_kind(signals: &DocumentSignals) -> &'static str {
    let mime_type = signals.mime_type.as_deref().unwrap_or("");

    if signals.has_video || mime_type.starts_with("video/") {
        return "video";
    }
    if signals.is_voice {
        return "voice";
    }
    if signals.has_audio || mime_type.starts_with("audio/") {
        return "audio";
    }
    if signals.is_animated {
        return "animation";
    }
    if mime_type.starts_with("image/") {
        return "image";
    }
    "document"
}

fn contact_summary(contact: &grammers_client::media::Contact) -> String {
    let display_name = [contact.first_name(), contact.last_name()]
        .into_iter()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if !display_name.is_empty() {
        return format!("Contact: {display_name}");
    }

    if !contact.phone_number().trim().is_empty() {
        return format!("Contact: {}", contact.phone_number().trim());
    }

    "Contact card".to_string()
}

fn extract_document_media_payload(
    document: &grammers_client::media::Document,
) -> ExtractedMediaPayload {
    let signals = collect_document_signals(document);
    let kind = derive_document_media_kind(&signals).to_string();
    let resolution = document.resolution();

    ExtractedMediaPayload {
        kind: kind.clone(),
        metadata: ItemMediaMetadata {
            summary: Some(media_label(&kind).to_string()),
            file_name: document.name().and_then(trimmed_non_empty),
            mime_type: document.mime_type().map(str::to_string),
            size_bytes: document.size().and_then(|size| i64::try_from(size).ok()),
            width: resolution.map(|(width, _)| width),
            height: resolution.map(|(_, height)| height),
            duration_seconds: document.duration(),
        },
    }
}

pub(crate) fn extract_media_payload(media: Media) -> ExtractedMediaPayload {
    match media {
        Media::Photo(photo) => ExtractedMediaPayload {
            kind: "photo".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Photo".to_string()),
                size_bytes: photo.size().and_then(|size| i64::try_from(size).ok()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Document(document) => extract_document_media_payload(&document),
        Media::Sticker(sticker) => ExtractedMediaPayload {
            kind: "sticker".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some(if sticker.emoji().trim().is_empty() {
                    "Sticker".to_string()
                } else {
                    format!("Sticker {}", sticker.emoji().trim())
                }),
                file_name: sticker.document.name().and_then(trimmed_non_empty),
                mime_type: sticker.document.mime_type().map(str::to_string),
                size_bytes: sticker
                    .document
                    .size()
                    .and_then(|size| i64::try_from(size).ok()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Contact(contact) => ExtractedMediaPayload {
            kind: "contact".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some(contact_summary(&contact)),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Poll(_) => ExtractedMediaPayload {
            kind: "poll".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Poll".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Geo(_) => ExtractedMediaPayload {
            kind: "location".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Location".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Dice(_) => ExtractedMediaPayload {
            kind: "dice".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Dice".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Venue(venue) => ExtractedMediaPayload {
            kind: "venue".to_string(),
            metadata: ItemMediaMetadata {
                summary: trimmed_non_empty(&venue.raw_venue.title)
                    .or_else(|| Some("Venue".to_string())),
                ..ItemMediaMetadata::default()
            },
        },
        Media::GeoLive(_) => ExtractedMediaPayload {
            kind: "live_location".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Live location".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::WebPage(_) => ExtractedMediaPayload {
            kind: "webpage".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Web page preview".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        _ => ExtractedMediaPayload {
            kind: "document".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Media".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
    }
}

pub(crate) fn extract_item_payload(
    message: &grammers_client::message::Message,
) -> Option<ExtractedItemPayload> {
    let content = trimmed_non_empty(message.text());
    let media = message.media().map(extract_media_payload);
    let has_content = content.is_some();
    let has_media = media.is_some();

    if !has_content && !has_media {
        return None;
    }

    Some(ExtractedItemPayload {
        content,
        content_kind: derive_content_kind(has_content, has_media),
        media,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        decode_media_metadata, derive_content_kind, derive_document_media_kind,
        encode_media_metadata, media_label, DocumentSignals, ItemMediaMetadata,
        CONTENT_KIND_MEDIA_ONLY, CONTENT_KIND_TEXT_ONLY, CONTENT_KIND_TEXT_WITH_MEDIA,
    };

    #[test]
    fn derive_content_kind_tracks_text_and_media_presence() {
        assert_eq!(derive_content_kind(true, false), CONTENT_KIND_TEXT_ONLY);
        assert_eq!(
            derive_content_kind(true, true),
            CONTENT_KIND_TEXT_WITH_MEDIA
        );
        assert_eq!(derive_content_kind(false, true), CONTENT_KIND_MEDIA_ONLY);
    }

    #[test]
    fn derive_document_media_kind_prefers_specific_signals() {
        let voice = DocumentSignals {
            mime_type: Some("audio/ogg".to_string()),
            has_audio: true,
            is_voice: true,
            ..DocumentSignals::default()
        };
        assert_eq!(derive_document_media_kind(&voice), "voice");

        let video = DocumentSignals {
            mime_type: Some("application/octet-stream".to_string()),
            has_video: true,
            ..DocumentSignals::default()
        };
        assert_eq!(derive_document_media_kind(&video), "video");

        let image = DocumentSignals {
            mime_type: Some("image/png".to_string()),
            ..DocumentSignals::default()
        };
        assert_eq!(derive_document_media_kind(&image), "image");
    }

    #[test]
    fn media_label_covers_known_and_fallback_kinds() {
        assert_eq!(media_label("photo"), "Photo");
        assert_eq!(media_label("live_location"), "Live location");
        assert_eq!(media_label("unknown"), "Document");
    }

    #[test]
    fn media_metadata_roundtrip_through_zstd() {
        let original = ItemMediaMetadata {
            summary: Some("Video".to_string()),
            file_name: Some("clip.mp4".to_string()),
            mime_type: Some("video/mp4".to_string()),
            size_bytes: Some(42),
            width: Some(1920),
            height: Some(1080),
            duration_seconds: Some(12.5),
        };

        let encoded = encode_media_metadata(&original).expect("encode");
        let decoded = decode_media_metadata(Some(&encoded)).expect("decode");

        assert_eq!(decoded, original);
    }

    #[test]
    fn media_metadata_decode_failures_are_typed_internal_errors() {
        let error = decode_media_metadata(Some(&[0x00])).expect_err("reject corrupt metadata");

        assert_eq!(error.kind, crate::error::AppErrorKind::Internal);
    }
}
