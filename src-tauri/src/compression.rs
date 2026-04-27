use std::io::Cursor;

pub(crate) fn compress_text(input: &str) -> Result<Vec<u8>, String> {
    compress_json_bytes(input.as_bytes())
}

pub(crate) fn compress_json_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    zstd::encode_all(Cursor::new(bytes), 3).map_err(|e| e.to_string())
}

pub(crate) fn decompress_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    zstd::decode_all(Cursor::new(bytes)).map_err(|e| e.to_string())
}

pub(crate) fn decompress_text(bytes: &[u8]) -> Result<String, String> {
    let decoded = decompress_bytes(bytes)?;
    String::from_utf8(decoded).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{compress_json_bytes, compress_text, decompress_bytes, decompress_text};

    #[test]
    fn text_roundtrip_through_zstd() {
        let original = "hello from extractum";
        let compressed = compress_text(original).expect("compress");
        let decompressed = decompress_text(&compressed).expect("decompress");
        assert_eq!(decompressed, original);
    }

    #[test]
    fn json_bytes_roundtrip_through_zstd() {
        let original = br#"{"hello":"world"}"#;
        let compressed = compress_json_bytes(original).expect("compress");
        let decompressed = decompress_bytes(&compressed).expect("decompress");
        assert_eq!(decompressed, original);
    }

    #[test]
    fn decompress_text_rejects_invalid_utf8() {
        let compressed = compress_json_bytes(&[0xff, 0xfe, 0xfd]).expect("compress");
        let error = decompress_text(&compressed).expect_err("invalid utf8 should fail");
        assert!(error.to_ascii_lowercase().contains("utf"));
    }
}
