use grammers_client::{media::Media, tl};

use extractum_core::media_metadata::{media_label, ItemMediaMetadata};

const CONTENT_KIND_TEXT_ONLY: &str = "text_only";
const CONTENT_KIND_TEXT_WITH_MEDIA: &str = "text_with_media";
const CONTENT_KIND_MEDIA_ONLY: &str = "media_only";

#[derive(Clone, Debug, PartialEq)]
pub struct TelegramMediaPayload {
    pub kind: String,
    pub metadata: ItemMediaMetadata,
}

#[derive(Default)]
struct DocumentSignals {
    mime_type: Option<String>,
    has_video: bool,
    has_audio: bool,
    is_voice: bool,
    is_animated: bool,
}

fn trimmed_non_empty(input: &str) -> Option<String> {
    let trimmed = input.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(super) fn derive_content_kind(has_content: bool, has_media: bool) -> &'static str {
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

fn derive_document_media_kind(signals: &DocumentSignals) -> &'static str {
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

pub(super) fn derive_document_media_kind_from_parts(
    mime_type: Option<&str>,
    has_video: bool,
    has_audio: bool,
    is_voice: bool,
    is_animated: bool,
) -> &'static str {
    derive_document_media_kind(&DocumentSignals {
        mime_type: mime_type.map(str::to_string),
        has_video,
        has_audio,
        is_voice,
        is_animated,
    })
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
) -> TelegramMediaPayload {
    let signals = collect_document_signals(document);
    let kind = derive_document_media_kind(&signals).to_string();
    let resolution = document.resolution();

    TelegramMediaPayload {
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

fn extract_media_payload(media: Media) -> TelegramMediaPayload {
    match media {
        Media::Photo(photo) => TelegramMediaPayload {
            kind: "photo".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Photo".to_string()),
                size_bytes: photo.size().and_then(|size| i64::try_from(size).ok()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Document(document) => extract_document_media_payload(&document),
        Media::Sticker(sticker) => TelegramMediaPayload {
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
        Media::Contact(contact) => TelegramMediaPayload {
            kind: "contact".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some(contact_summary(&contact)),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Poll(_) => TelegramMediaPayload {
            kind: "poll".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Poll".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Geo(_) => TelegramMediaPayload {
            kind: "location".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Location".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Dice(_) => TelegramMediaPayload {
            kind: "dice".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Dice".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Venue(venue) => TelegramMediaPayload {
            kind: "venue".to_string(),
            metadata: ItemMediaMetadata {
                summary: trimmed_non_empty(&venue.raw_venue.title)
                    .or_else(|| Some("Venue".to_string())),
                ..ItemMediaMetadata::default()
            },
        },
        Media::GeoLive(_) => TelegramMediaPayload {
            kind: "live_location".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Live location".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::WebPage(_) => TelegramMediaPayload {
            kind: "webpage".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Web page preview".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        _ => TelegramMediaPayload {
            kind: "document".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Media".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
    }
}

fn extract_item_payload_from_parts(
    text: &str,
    media: Option<Media>,
) -> Option<(Option<String>, &'static str, Option<TelegramMediaPayload>)> {
    let content = trimmed_non_empty(text);
    let media = media.map(extract_media_payload);
    let has_content = content.is_some();
    let has_media = media.is_some();

    if !has_content && !has_media {
        return None;
    }

    Some((content, derive_content_kind(has_content, has_media), media))
}

pub(super) fn extract_item_payload(
    message: &grammers_client::message::Message,
) -> Option<(Option<String>, &'static str, Option<TelegramMediaPayload>)> {
    extract_item_payload_from_parts(message.text(), message.media())
}

pub(super) fn extract_raw_item_payload(
    message: &tl::enums::Message,
) -> Option<(Option<String>, &'static str, Option<TelegramMediaPayload>)> {
    let (text, raw_media) = match message {
        tl::enums::Message::Message(message) => (message.message.as_str(), message.media.clone()),
        tl::enums::Message::Empty(_) | tl::enums::Message::Service(_) => ("", None),
    };
    extract_item_payload_from_parts(text, raw_media.and_then(Media::from_raw))
}

#[cfg(test)]
mod tests {
    use super::{
        derive_content_kind, derive_document_media_kind_from_parts, CONTENT_KIND_MEDIA_ONLY,
        CONTENT_KIND_TEXT_ONLY, CONTENT_KIND_TEXT_WITH_MEDIA,
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
        assert_eq!(
            derive_document_media_kind_from_parts(Some("audio/ogg"), false, true, true, false,),
            "voice"
        );
        assert_eq!(
            derive_document_media_kind_from_parts(
                Some("application/octet-stream"),
                true,
                false,
                false,
                false,
            ),
            "video"
        );
        assert_eq!(
            derive_document_media_kind_from_parts(Some("image/png"), false, false, false, false,),
            "image"
        );
    }
}
