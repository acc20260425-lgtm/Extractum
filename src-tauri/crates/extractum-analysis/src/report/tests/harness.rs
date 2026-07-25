use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::JoinHandle;

use super::super::super::corpus::AnalysisCorpusMessage;
use super::super::super::models::{AnalysisPromptTemplate, ChunkSummary};
use extractum_llm::{LlmProviderAccess, ProviderKind, ResolvedLlmProfile};
use sqlx::SqlitePool;

pub(super) const SAMPLE_JSON: &str = r#"{"summary":"Brief","topics":["sync"],"notable_points":["Point"],"candidate_refs":["s1-i2"]}"#;

pub(super) fn sample_chunk_summary(label: &str) -> ChunkSummary {
    ChunkSummary {
        summary: label.to_string(),
        topics: vec![format!("{label}-topic")],
        notable_points: vec![format!("{label}-point")],
        candidate_refs: vec![format!("{label}-ref")],
    }
}

pub(super) fn sample_prompt_template() -> AnalysisPromptTemplate {
    AnalysisPromptTemplate {
        id: 7,
        name: "Report".to_string(),
        template_kind: "report".to_string(),
        body: "Write a concise report.".to_string(),
        version: 3,
        is_builtin: false,
        created_at: 1,
        updated_at: 1,
    }
}

pub(super) fn sample_corpus_message() -> AnalysisCorpusMessage {
    AnalysisCorpusMessage::new(
        1,
        2,
        "42".to_string(),
        1_700_000_000,
        Some("analyst".to_string()),
        "Important update from the source".to_string(),
        "s2-i1".to_string(),
        Some("telegram_message".to_string()),
        Some("telegram".to_string()),
        Some("channel".to_string()),
        None,
    )
}

pub(super) fn sample_resolved_profile() -> ResolvedLlmProfile {
    ResolvedLlmProfile::new(
        "research".to_string(),
        "gemini-2.5-flash".to_string(),
        LlmProviderAccess::new(
            ProviderKind::Gemini,
            "secret-key".to_string().into(),
            String::new(),
        ),
    )
}

fn read_http_request(stream: &mut TcpStream) {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut chunk).expect("read completion request");
        assert!(read > 0, "completion request ended before headers");
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    let body_read = buffer.len().saturating_sub(header_end);
    if content_length > body_read {
        let mut body_tail = vec![0_u8; content_length - body_read];
        stream
            .read_exact(&mut body_tail)
            .expect("read completion request body");
    }
}

pub(super) fn start_openai_compat_completion_server(
    completions: Vec<String>,
) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind completion server");
    let base_url = format!(
        "http://{}/v1",
        listener.local_addr().expect("completion server address")
    );
    let server = std::thread::spawn(move || {
        for completion in completions {
            let (mut stream, _) = listener.accept().expect("accept completion request");
            read_http_request(&mut stream);
            let payload = serde_json::json!({
                "choices": [{"delta": {"content": completion}}],
            });
            let body = format!("data: {payload}\n\ndata: [DONE]\n\n");
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            stream
                .write_all(response.as_bytes())
                .expect("write completion response");
        }
    });
    (base_url, server)
}

pub(super) async fn report_capture_pool(run_id: i64) -> SqlitePool {
    let pool = super::super::super::test_schema::analysis_test_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status, created_at
         ) VALUES (
            ?, 'report', 'single_source', 1, 2, 'English',
            1, 'research', 'gemini', 'gemini-2.5-flash', 'running', 1
         )",
    )
    .bind(run_id)
    .execute(&pool)
    .await
    .expect("insert report capture run");
    pool
}

pub(super) async fn request_cancel_pool_with_runs() -> SqlitePool {
    super::super::super::test_schema::analysis_test_pool().await
}

pub(super) async fn insert_cancel_request_run(pool: &SqlitePool, run_id: i64, status: &str) {
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, status, period_from, period_to, output_language,
            prompt_template_id, prompt_template_version, provider_profile, provider, model,
            youtube_corpus_mode, created_at
        ) VALUES (
            ?, 'report', 'single_source', ?, 1, 2, 'English', 1, 1,
            'research', 'gemini', 'gemini-2.5-flash', 'transcript_description', 1
        )",
    )
    .bind(run_id)
    .bind(status)
    .execute(pool)
    .await
    .expect("insert analysis run");
}
