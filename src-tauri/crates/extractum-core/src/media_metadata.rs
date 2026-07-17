use serde::{Deserialize, Serialize};

use crate::compression::{compress_json_bytes, decompress_bytes};
use crate::error::{AppError, AppResult};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct ItemMediaMetadata {
    pub summary: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_seconds: Option<f64>,
}

pub fn encode_media_metadata(metadata: &ItemMediaMetadata) -> AppResult<Vec<u8>> {
    let json =
        serde_json::to_vec(metadata).map_err(|error| AppError::internal(error.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

pub fn decode_media_metadata(bytes: Option<&[u8]>) -> AppResult<ItemMediaMetadata> {
    let Some(bytes) = bytes else {
        return Ok(ItemMediaMetadata::default());
    };
    let decoded = decompress_bytes(bytes).map_err(AppError::internal)?;
    serde_json::from_slice(&decoded).map_err(|error| AppError::internal(error.to_string()))
}

pub fn media_label(kind: &str) -> &'static str {
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

#[cfg(test)]
mod tests {
    use crate::error::AppErrorKind;

    use super::{decode_media_metadata, encode_media_metadata, media_label, ItemMediaMetadata};

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

        assert_eq!(error.kind, AppErrorKind::Internal);
    }

    #[test]
    fn absent_media_metadata_decodes_to_default() {
        let decoded = decode_media_metadata(None).expect("decode absent metadata");

        assert_eq!(decoded, ItemMediaMetadata::default());
    }
}
