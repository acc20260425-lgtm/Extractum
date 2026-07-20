use extractum_core::error::{AppError, AppResult};

pub(super) fn find_event_boundary(buffer: &[u8]) -> Option<(usize, usize)> {
    if buffer.len() < 2 {
        return None;
    }

    for index in 0..buffer.len() - 1 {
        if buffer[index] == b'\n' && buffer[index + 1] == b'\n' {
            return Some((index, 2));
        }
        if index + 3 < buffer.len()
            && buffer[index] == b'\r'
            && buffer[index + 1] == b'\n'
            && buffer[index + 2] == b'\r'
            && buffer[index + 3] == b'\n'
        {
            return Some((index, 4));
        }
    }

    None
}

pub(super) fn parse_sse_data(event_bytes: &[u8]) -> AppResult<Option<String>> {
    let text = String::from_utf8(event_bytes.to_vec())
        .map_err(|error| AppError::internal(error.to_string()))?;
    let mut data_lines = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_string());
        }
    }

    if data_lines.is_empty() {
        return Ok(None);
    }

    let data = data_lines.join("\n");
    if data.trim() == "[DONE]" {
        return Ok(None);
    }

    Ok(Some(data))
}

#[cfg(test)]
mod tests {
    use super::{find_event_boundary, parse_sse_data};

    #[test]
    fn sse_data_is_parsed_from_stream_chunks() {
        let frame = b"data: {\"hello\":\"world\"}\n\n";
        let (boundary, delimiter) = find_event_boundary(frame).expect("find boundary");
        assert_eq!(delimiter, 2);
        let payload = parse_sse_data(&frame[..boundary])
            .expect("parse sse")
            .expect("payload");

        assert_eq!(payload, "{\"hello\":\"world\"}");
    }

    #[test]
    fn sse_data_decode_failures_are_typed_internal_errors() {
        let error = parse_sse_data(&[0xff]).expect_err("reject invalid utf-8");

        assert_eq!(error.kind, extractum_core::error::AppErrorKind::Internal);
    }
}
