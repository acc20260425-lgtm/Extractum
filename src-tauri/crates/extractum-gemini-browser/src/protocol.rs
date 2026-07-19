use serde::Deserialize;

use super::{
    error::{GeminiBrowserError, GeminiBrowserResult},
    GeminiBrowserProviderStatus, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
    GeminiBrowserSidecarResponse,
};

#[derive(Deserialize)]
struct SidecarLine {
    id: String,
    response: GeminiBrowserSidecarResponse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResumeSidecarOutcome {
    Status(GeminiBrowserProviderStatus),
    LegacyAck,
}

#[derive(Debug, Default)]
pub struct GeminiBrowserJsonlCodec {
    buffer: Vec<u8>,
}

impl GeminiBrowserJsonlCodec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encode_request(
        &self,
        id: &str,
        command: &GeminiBrowserSidecarCommand,
    ) -> GeminiBrowserResult<Vec<u8>> {
        let envelope = GeminiBrowserSidecarEnvelope {
            id: id.to_string(),
            command: command.clone(),
        };
        let mut encoded = serde_json::to_vec(&envelope)
            .map_err(|error| GeminiBrowserError::protocol(error.to_string()))?;
        encoded.push(b'\n');
        Ok(encoded)
    }

    pub fn push_response_bytes(
        &mut self,
        expected_id: &str,
        chunk: &[u8],
    ) -> GeminiBrowserResult<Option<GeminiBrowserSidecarResponse>> {
        self.buffer.extend_from_slice(chunk);
        loop {
            let Some(newline_index) = self.buffer.iter().position(|byte| *byte == b'\n') else {
                return Ok(None);
            };
            let line = self.buffer.drain(..=newline_index).collect::<Vec<_>>();
            let line = trim_ascii_whitespace(&line);
            if line.is_empty() {
                continue;
            }
            let response: SidecarLine = serde_json::from_slice(line).map_err(|error| {
                GeminiBrowserError::protocol(format!("Invalid Gemini sidecar response: {error}"))
            })?;
            if response.id == expected_id {
                return Ok(Some(response.response));
            }
        }
    }
}

fn trim_ascii_whitespace(value: &[u8]) -> &[u8] {
    let start = value
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    &value[start..end]
}

pub fn classify_resume_response(
    response: GeminiBrowserSidecarResponse,
) -> GeminiBrowserResult<ResumeSidecarOutcome> {
    match response {
        GeminiBrowserSidecarResponse::Status { status } => Ok(ResumeSidecarOutcome::Status(status)),
        GeminiBrowserSidecarResponse::Ack => Ok(ResumeSidecarOutcome::LegacyAck),
        _ => Err(GeminiBrowserError::protocol(
            "Unexpected Gemini sidecar resume response",
        )),
    }
}

pub(crate) fn sidecar_unavailable_result(
    request: GeminiBrowserRunRequest,
) -> GeminiBrowserRunResult {
    GeminiBrowserRunResult {
        run_id: request.run_id,
        status: GeminiBrowserRunStatus::Failed,
        text: None,
        message: Some("Gemini browser sidecar is unavailable.".to_string()),
        manual_action: None,
        artifacts: Default::default(),
        elapsed_ms: 0,
        debug_summary: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_sidecar_line(
        id: &str,
        response_line: &str,
    ) -> GeminiBrowserResult<GeminiBrowserSidecarResponse> {
        let mut codec = GeminiBrowserJsonlCodec::new();
        let mut line = response_line.as_bytes().to_vec();
        line.push(b'\n');
        codec.push_response_bytes(id, &line)?.ok_or_else(|| {
            GeminiBrowserError::protocol("Gemini browser sidecar response id mismatch")
        })
    }

    #[test]
    fn decode_sidecar_line_rejects_mismatched_ids() {
        let error = decode_sidecar_line("expected", r#"{"id":"other","response":{"type":"ack"}}"#)
            .unwrap_err();
        assert!(error.to_string().contains("response id mismatch"));
    }

    #[test]
    fn decode_sidecar_line_accepts_ack_for_matching_id() {
        let response =
            decode_sidecar_line("expected", r#"{"id":"expected","response":{"type":"ack"}}"#)
                .expect("decode response");
        assert!(matches!(response, GeminiBrowserSidecarResponse::Ack));
    }

    #[test]
    fn decode_sidecar_line_for_request_skips_stale_response_ids() {
        let mut codec = GeminiBrowserJsonlCodec::new();
        assert!(codec
            .push_response_bytes(
                "expected",
                b"{\"id\":\"previous\",\"response\":{\"type\":\"ack\"}}\n"
            )
            .expect("decode stale response")
            .is_none());
        assert!(matches!(
            codec
                .push_response_bytes(
                    "expected",
                    b"{\"id\":\"expected\",\"response\":{\"type\":\"ack\"}}\n"
                )
                .expect("decode expected response"),
            Some(GeminiBrowserSidecarResponse::Ack)
        ));
    }

    #[test]
    fn take_complete_jsonl_lines_handles_partial_and_multiple_chunks() {
        let mut codec = GeminiBrowserJsonlCodec::new();
        assert!(codec
            .push_response_bytes("one", b"{\"id\":\"one\"")
            .expect("partial chunk")
            .is_none());
        assert!(matches!(
            codec
                .push_response_bytes("one", b",\"response\":{\"type\":\"ack\"}}\n\n{\"id\":\"two\",\"response\":{\"type\":\"ack\"}}\n")
                .expect("multiple frames"),
            Some(GeminiBrowserSidecarResponse::Ack)
        ));
        assert!(matches!(
            codec
                .push_response_bytes("two", b"")
                .expect("retained frame"),
            Some(GeminiBrowserSidecarResponse::Ack)
        ));
    }

    #[test]
    fn jsonl_transport_round_trips_a_duplex_request() {
        let mut codec = GeminiBrowserJsonlCodec::new();
        assert_eq!(
            String::from_utf8(
                codec
                    .encode_request("gemini-sidecar-1", &GeminiBrowserSidecarCommand::Stop)
                    .expect("encode request")
            )
            .expect("utf8 request"),
            "{\"id\":\"gemini-sidecar-1\",\"command\":{\"type\":\"stop\"}}\n"
        );
        assert!(codec
            .push_response_bytes("gemini-sidecar-1", b"{\"id\":\"st")
            .expect("partial stale frame")
            .is_none());
        let response = codec
            .push_response_bytes("gemini-sidecar-1", b"ale\",\"response\":{\"type\":\"ack\"}}\n{\"id\":\"gemini-sidecar-1\",\"response\":{\"type\":\"ack\"}}\n{\"id\":\"next\",\"response\":{\"type\":\"a")
            .expect("stale, matching, and read-ahead frames")
            .expect("matching response");
        assert!(matches!(response, GeminiBrowserSidecarResponse::Ack));
        assert!(matches!(
            codec
                .push_response_bytes("next", b"ck\"}}\n")
                .expect("finish retained prefix"),
            Some(GeminiBrowserSidecarResponse::Ack)
        ));
    }

    #[test]
    fn resume_response_classifies_legacy_ack_for_retry() {
        let outcome = classify_resume_response(GeminiBrowserSidecarResponse::Ack)
            .expect("classify resume response");
        assert!(matches!(outcome, ResumeSidecarOutcome::LegacyAck));
    }
}
