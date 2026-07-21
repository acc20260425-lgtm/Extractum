use std::future::Future;

use sqlx::SqlitePool;

use super::outputs::execute_transcript_analysis_stage_with_completion_and_metrics_extension;
use super::progress::is_run_cancelled;
use super::transcript_execution::TranscriptStageRow;
use super::{
    estimate_tokens, GemAnalysisInputBudget, GemAnalysisPart, GemAnalysisPartRepairRequest,
    GemAnalysisPartStageExecutionRequest, LlmCompletion, YoutubeSummaryStageExecutionError,
    YoutubeSummaryStageExecutionRequest,
};
use crate::compression::decompress_text;
use crate::error::{AppError, AppResult};
use crate::prompt_packs::source_port::PromptPackTranscriptSegment;
use crate::prompt_packs::stage_io::TranscriptAnalysisStageInput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GemCommentsStatus {
    Present,
    SkippedNoComments,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisMaterialInput {
    pub(crate) transcript_plain: String,
    pub(crate) transcript_timestamped: String,
    pub(crate) timestamps_available: bool,
    pub(crate) comments_text: String,
    pub(crate) comments_status: GemCommentsStatus,
}

const GEM_PASSPORT_PROMPT_BODY: &str = r#"Системная роль:
Вы - ведущий аналитик видеоконтента и эксперт по структурированию знаний.

Цель: создать аналитический паспорт видео на основе только предоставленного transcript material.

Структура: метаданные и контекст, эссенция, optional how-to, адаптивный модуль, упоминания и доступный фактчекинг.

Правила: не использовать комментарии, описание, метаданные источника или результаты других Gem parts; не выдумывать title, URL, автора, подписчиков, длительность, дату публикации, просмотры, ссылки или timestamp; недоступные поля помечать как `Недоступно во входных данных`; timestamp брать только из `[MM:SS]` во входном transcript material; если внешний фактчекинг недоступен, явно указать это ограничение.

Markdown должен начинаться с `###`; не использовать `#`, `##` или leading/trailing `---`."#;

const GEM_COMMENTS_PROMPT_BODY: &str = r#"Системная роль:
Вы - эксперт по анализу общественного мнения, аудитории и сентимент-анализу.

Цель: проанализировать только предоставленный selected comment sample. Это ограниченная выборка комментариев, а не вся аудитория.

Структура: общий сентимент выборки без точных процентов, ключевые темы обсуждения, вопросы и боли аудитории, ценные инсайты и дополнения, конструктивная критика.

Правила: не использовать transcript, описание, метаданные источника или результаты других Gem parts; не цитировать комментарии дословно; не делать выводы репрезентативными для всей аудитории; использовать качественные формулировки вместо точных процентов.

Markdown должен начинаться с `###`; не использовать `#`, `##` или leading/trailing `---`."#;

const GEM_DEEP_RECAP_PROMPT_BODY: &str = r#"Системная роль:
Вы - ведущий аналитик видеоконтента и эксперт по структурированию знаний.

Цель: создать глубокий интерактивный пересказ основного содержания на основе только provided transcript material.

Требования: 800-1000+ слов, если transcript достаточно содержателен; логические главы; каждому ключевому тезису по возможности сопоставить реальный `[MM:SS]` из входа; таблицы для сравнений; LaTeX для формул; fenced code blocks для кода.

Правила: не использовать комментарии, описание, метаданные источника или результаты других Gem parts; не придумывать timestamp; если тезис нельзя связать с входным timestamp, не создавать приблизительный timestamp.

Markdown должен начинаться с `###`; вложенные заголовки через `####`; не использовать `#`, `##` или leading/trailing `---`."#;

const GEM_INPUT_ESTIMATOR_OVERHEAD_TOKENS: i64 = 1_500;
const GEM_LLM_WRAPPER_TEXT_ESTIMATE: &str = "Return strict JSON for one Gem analysis part with part and markdown fields. Use only the provided material and do not add prose outside JSON.";
const GEM_COMMENTS_FAILURE_NOTE: &str =
    "Не выполнено: анализ комментариев завершился ошибкой после повторной попытки.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemPartMetrics {
    part: GemAnalysisPart,
    status: &'static str,
    attempts: i64,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    latency_ms: i64,
    repaired: bool,
    error_message: Option<String>,
}

impl GemPartMetrics {
    fn succeeded(part: GemAnalysisPart, completion: &LlmCompletion) -> Self {
        Self {
            part,
            status: "succeeded",
            attempts: 1,
            input_tokens: completion.input_tokens,
            output_tokens: completion.output_tokens,
            latency_ms: completion.latency_ms,
            repaired: false,
            error_message: None,
        }
    }

    fn failed(part: GemAnalysisPart, error_message: String) -> Self {
        Self {
            part,
            status: "failed",
            attempts: 1,
            input_tokens: None,
            output_tokens: None,
            latency_ms: 0,
            repaired: false,
            error_message: Some(error_message),
        }
    }

    fn add_repair_completion(&mut self, completion: &LlmCompletion) {
        self.attempts += 1;
        self.input_tokens = sum_optional_tokens(self.input_tokens, completion.input_tokens);
        self.output_tokens = sum_optional_tokens(self.output_tokens, completion.output_tokens);
        self.latency_ms += completion.latency_ms;
        self.repaired = true;
    }

    fn to_json(&self) -> serde_json::Value {
        let mut value = serde_json::json!({
            "part": self.part.as_str(),
            "status": self.status,
            "attempts": self.attempts,
            "input_tokens": self.input_tokens,
            "output_tokens": self.output_tokens,
            "latency_ms": self.latency_ms,
            "repaired": self.repaired,
        });
        if let Some(error_message) = &self.error_message {
            value["error_message"] = serde_json::json!(error_message);
        }
        value
    }
}

fn sum_optional_tokens(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GemInputEstimate {
    part: GemAnalysisPart,
    estimated_tokens: i64,
}

pub(crate) async fn load_gem_analysis_materials(
    pool: &SqlitePool,
    stage_run_id: i64,
) -> AppResult<GemAnalysisMaterialInput> {
    let (transcript_zstd, metadata_json_zstd): (Vec<u8>, Option<Vec<u8>>) = sqlx::query_as(
        "SELECT materials.text_zstd, materials.metadata_json_zstd
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_material_snapshots materials
           ON materials.run_id = stages.run_id
          AND materials.source_snapshot_id = stages.source_snapshot_id
         WHERE stages.id = ?
           AND materials.material_kind = 'transcript'
         ORDER BY materials.sequence_index ASC, materials.id ASC
         LIMIT 1",
    )
    .bind(stage_run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let transcript_plain = decompress_text(&transcript_zstd).map_err(AppError::internal)?;
    let transcript_segments = metadata_json_zstd
        .as_deref()
        .and_then(transcript_segments_from_metadata_zstd);
    let (transcript_timestamped, timestamps_available) = match transcript_segments {
        Some(segments) if !segments.is_empty() => (render_timestamped_segments(&segments), true),
        _ => (transcript_plain.clone(), false),
    };

    let comment_rows = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT materials.text_zstd
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_material_snapshots materials
           ON materials.run_id = stages.run_id
          AND materials.source_snapshot_id = stages.source_snapshot_id
         WHERE stages.id = ?
           AND materials.material_kind = 'comment'
         ORDER BY materials.sequence_index ASC, materials.id ASC",
    )
    .bind(stage_run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let comments = comment_rows
        .into_iter()
        .map(|row| decompress_text(&row).map_err(AppError::internal))
        .collect::<AppResult<Vec<_>>>()?
        .into_iter()
        .map(|comment| comment.trim().to_string())
        .filter(|comment| !comment.is_empty())
        .collect::<Vec<_>>();
    let comments_text = comments.join("\n\n");
    let comments_status = if comments_text.trim().is_empty() {
        GemCommentsStatus::SkippedNoComments
    } else {
        GemCommentsStatus::Present
    };

    Ok(GemAnalysisMaterialInput {
        transcript_plain,
        transcript_timestamped,
        timestamps_available,
        comments_text,
        comments_status,
    })
}

fn transcript_segments_from_metadata_zstd(
    metadata_zstd: &[u8],
) -> Option<Vec<PromptPackTranscriptSegment>> {
    let metadata = decompress_text(metadata_zstd).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&metadata).ok()?;
    value
        .get("segments")?
        .as_array()?
        .iter()
        .map(|segment| {
            Some(PromptPackTranscriptSegment::new(
                segment.get("start_ms")?.as_i64()?,
                segment.get("end_ms")?.as_i64()?,
                segment.get("text")?.as_str()?.to_string(),
            ))
        })
        .collect()
}

fn format_timestamp_ms(start_ms: i64) -> String {
    let total_seconds = (start_ms / 1000).max(0);
    format!("[{:02}:{:02}]", total_seconds / 60, total_seconds % 60)
}

fn render_timestamped_segments(segments: &[PromptPackTranscriptSegment]) -> String {
    segments
        .iter()
        .map(|segment| {
            format!(
                "{} {}",
                format_timestamp_ms(segment.start_ms()),
                segment.text()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn build_gem_analysis_part_prompt_input(
    part: GemAnalysisPart,
    source_ref_id: &str,
    materials: &GemAnalysisMaterialInput,
) -> serde_json::Value {
    match part {
        GemAnalysisPart::Passport => serde_json::json!({
            "part": part.as_str(),
            "source_ref_id": source_ref_id,
            "timestamps_available": materials.timestamps_available,
            "input_material": {
                "kind": "transcript",
                "text": materials.transcript_timestamped.as_str(),
            },
            "task": GEM_PASSPORT_PROMPT_BODY,
        }),
        GemAnalysisPart::Comments => serde_json::json!({
            "part": part.as_str(),
            "input_material": {
                "kind": "selected_comment_sample",
                "sample_limit_note": "Analysis is based only on the provided selected comment sample.",
                "text": materials.comments_text.as_str(),
            },
            "task": GEM_COMMENTS_PROMPT_BODY,
        }),
        GemAnalysisPart::DeepRecap => serde_json::json!({
            "part": part.as_str(),
            "source_ref_id": source_ref_id,
            "timestamps_available": materials.timestamps_available,
            "input_material": {
                "kind": "transcript",
                "text": materials.transcript_timestamped.as_str(),
            },
            "task": GEM_DEEP_RECAP_PROMPT_BODY,
        }),
    }
}

pub(crate) fn estimate_gem_prompt_tokens(prompt_input_json: &str, wrapper_text: &str) -> i64 {
    estimate_tokens(prompt_input_json)
        + estimate_tokens(wrapper_text)
        + GEM_INPUT_ESTIMATOR_OVERHEAD_TOKENS
}

pub(crate) fn enforce_gem_input_budget(
    part: GemAnalysisPart,
    estimate: i64,
    budget: GemAnalysisInputBudget,
) -> AppResult<()> {
    if estimate > budget.max_input_tokens {
        return Err(AppError::validation(format!(
            "Gem analysis input for {} exceeds the selected model input budget",
            part.as_str()
        )));
    }
    Ok(())
}

pub(crate) fn assemble_gem_analysis_markdown(
    passport_markdown: &str,
    comments_markdown: Option<&str>,
    deep_recap_markdown: &str,
) -> String {
    let comments = comments_markdown
        .map(normalize_part_markdown)
        .unwrap_or_else(|| "Пропущено: содержательные комментарии отсутствуют.".to_string());
    format!(
        "# Gem-анализ\n\n\
         ## Часть 1. Аналитический паспорт видео\n\n{}\n\n\
         ---\n\n\
         ## Часть 2. Анализ комментариев к видео\n\n{}\n\n\
         ---\n\n\
         ## Часть 3. Глубокий интерактивный пересказ\n\n{}",
        normalize_part_markdown(passport_markdown),
        comments,
        normalize_part_markdown(deep_recap_markdown)
    )
}

pub(crate) fn assemble_gem_analysis_transcript_output(markdown: &str) -> AppResult<String> {
    serde_json::to_string(&serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/transcript_analysis",
        "video_candidate": {
            "summary_text": markdown,
            "segment_candidates": [],
            "key_point_candidates": [],
            "quote_candidates": [],
            "action_item_candidates": [],
            "open_question_candidates": [],
        },
        "claim_candidates": [],
        "evidence_fragment_candidates": [],
        "warning_candidates": [],
    }))
    .map_err(|error| AppError::internal(format!("serialize Gem analysis output: {error}")))
}

fn normalize_part_markdown(markdown: &str) -> String {
    let mut lines = markdown
        .trim()
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>();
    while lines.first().is_some_and(|line| line.trim() == "---") {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim() == "---") {
        lines.pop();
    }
    lines
        .into_iter()
        .map(|line| {
            if let Some(rest) = line.strip_prefix("# ") {
                format!("### {rest}")
            } else if let Some(rest) = line.strip_prefix("## ") {
                format!("### {rest}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisPartOutput {
    pub(crate) part: GemAnalysisPart,
    pub(crate) markdown: String,
}

pub(crate) fn parse_gem_analysis_part_output(
    raw: &str,
    expected_part: GemAnalysisPart,
) -> AppResult<GemAnalysisPartOutput> {
    let value = crate::prompt_packs::stage_io::extract_json_payload(raw)?;
    let part = value
        .get("part")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| AppError::validation("Gem analysis part output is missing part"))?;
    if part != expected_part.as_str() {
        return Err(AppError::validation(format!(
            "Gem analysis part output expected part {} but got {part}",
            expected_part.as_str()
        )));
    }
    let markdown = value
        .get("markdown")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|markdown| !markdown.is_empty())
        .ok_or_else(|| AppError::validation("Gem analysis part output markdown is empty"))?
        .to_string();
    Ok(GemAnalysisPartOutput {
        part: expected_part,
        markdown,
    })
}

pub(crate) async fn execute_gem_analysis_transcript_stage<F, Fut>(
    pool: &SqlitePool,
    run_id: i64,
    stage: &TranscriptStageRow,
    _input: TranscriptAnalysisStageInput,
    input_budget: GemAnalysisInputBudget,
    execute_stage: &mut F,
) -> Result<(), YoutubeSummaryStageExecutionError>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
{
    verify_single_source_snapshot(pool, run_id).await?;

    let materials = load_gem_analysis_materials(pool, stage.stage_run_id)
        .await
        .map_err(YoutubeSummaryStageExecutionError::Failed)?;

    let passport_prompt_input_json =
        build_gem_prompt_input_json(GemAnalysisPart::Passport, &stage.source_ref_id, &materials)?;
    let deep_recap_prompt_input_json =
        build_gem_prompt_input_json(GemAnalysisPart::DeepRecap, &stage.source_ref_id, &materials)?;
    let comments_prompt_input_json = if materials.comments_status == GemCommentsStatus::Present {
        Some(build_gem_prompt_input_json(
            GemAnalysisPart::Comments,
            &stage.source_ref_id,
            &materials,
        )?)
    } else {
        None
    };

    let mut input_estimates = vec![
        GemInputEstimate {
            part: GemAnalysisPart::Passport,
            estimated_tokens: estimate_gem_prompt_tokens(
                &passport_prompt_input_json,
                GEM_LLM_WRAPPER_TEXT_ESTIMATE,
            ),
        },
        GemInputEstimate {
            part: GemAnalysisPart::DeepRecap,
            estimated_tokens: estimate_gem_prompt_tokens(
                &deep_recap_prompt_input_json,
                GEM_LLM_WRAPPER_TEXT_ESTIMATE,
            ),
        },
    ];
    if let Some(prompt_input_json) = &comments_prompt_input_json {
        input_estimates.push(GemInputEstimate {
            part: GemAnalysisPart::Comments,
            estimated_tokens: estimate_gem_prompt_tokens(
                prompt_input_json,
                GEM_LLM_WRAPPER_TEXT_ESTIMATE,
            ),
        });
    }
    for estimate in &input_estimates {
        enforce_gem_input_budget(estimate.part, estimate.estimated_tokens, input_budget)
            .map_err(YoutubeSummaryStageExecutionError::Failed)?;
    }

    ensure_gem_run_not_cancelled(pool, run_id).await?;
    let passport_request = build_gem_part_request(
        run_id,
        stage,
        GemAnalysisPart::Passport,
        passport_prompt_input_json,
    );
    let (passport, passport_metrics) =
        run_gem_part_with_one_repair(execute_stage, passport_request).await?;

    ensure_gem_run_not_cancelled(pool, run_id).await?;
    let (comments_markdown, comments_metrics) =
        if let Some(comments_prompt_input_json) = comments_prompt_input_json {
            let comments_request = build_gem_part_request(
                run_id,
                stage,
                GemAnalysisPart::Comments,
                comments_prompt_input_json,
            );
            match run_gem_part_with_one_repair(execute_stage, comments_request).await {
                Ok((comments, metrics)) => (Some(comments.markdown), metrics),
                Err(YoutubeSummaryStageExecutionError::Cancelled) => {
                    return Err(YoutubeSummaryStageExecutionError::Cancelled)
                }
                Err(YoutubeSummaryStageExecutionError::Failed(error)) => (
                    Some(GEM_COMMENTS_FAILURE_NOTE.to_string()),
                    GemPartMetrics::failed(GemAnalysisPart::Comments, error.message),
                ),
            }
        } else {
            (
                None,
                GemPartMetrics {
                    part: GemAnalysisPart::Comments,
                    status: "skipped_no_comments",
                    attempts: 0,
                    input_tokens: None,
                    output_tokens: None,
                    latency_ms: 0,
                    repaired: false,
                    error_message: None,
                },
            )
        };

    ensure_gem_run_not_cancelled(pool, run_id).await?;
    let deep_recap_request = build_gem_part_request(
        run_id,
        stage,
        GemAnalysisPart::DeepRecap,
        deep_recap_prompt_input_json,
    );
    let (deep_recap, deep_recap_metrics) =
        run_gem_part_with_one_repair(execute_stage, deep_recap_request).await?;

    ensure_gem_run_not_cancelled(pool, run_id).await?;
    let markdown = assemble_gem_analysis_markdown(
        &passport.markdown,
        comments_markdown.as_deref(),
        &deep_recap.markdown,
    );
    let assembled_output_json = assemble_gem_analysis_transcript_output(&markdown)
        .map_err(YoutubeSummaryStageExecutionError::Failed)?;
    let total_latency_ms =
        passport_metrics.latency_ms + comments_metrics.latency_ms + deep_recap_metrics.latency_ms;
    let gem_metrics_json = gem_metrics_json(
        input_budget,
        &input_estimates,
        &[passport_metrics, deep_recap_metrics],
        &comments_metrics,
    );
    let completion = LlmCompletion {
        text: assembled_output_json,
        input_tokens: None,
        output_tokens: None,
        latency_ms: total_latency_ms,
    };
    execute_transcript_analysis_stage_with_completion_and_metrics_extension(
        pool,
        stage.stage_run_id,
        completion,
        Some(gem_metrics_json),
    )
    .await
    .map_err(YoutubeSummaryStageExecutionError::Failed)
}

async fn verify_single_source_snapshot(
    pool: &SqlitePool,
    run_id: i64,
) -> Result<(), YoutubeSummaryStageExecutionError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM prompt_pack_run_source_snapshots
         WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
    .map_err(YoutubeSummaryStageExecutionError::Failed)?;
    if count != 1 {
        return Err(YoutubeSummaryStageExecutionError::Failed(
            AppError::validation("Gem analysis requires exactly one included source snapshot"),
        ));
    }
    Ok(())
}

fn build_gem_prompt_input_json(
    part: GemAnalysisPart,
    source_ref_id: &str,
    materials: &GemAnalysisMaterialInput,
) -> Result<String, YoutubeSummaryStageExecutionError> {
    let value = build_gem_analysis_part_prompt_input(part, source_ref_id, materials);
    serde_json::to_string_pretty(&value)
        .map_err(|error| AppError::internal(format!("serialize Gem prompt input: {error}")))
        .map_err(YoutubeSummaryStageExecutionError::Failed)
}

fn build_gem_part_request(
    run_id: i64,
    stage: &TranscriptStageRow,
    part: GemAnalysisPart,
    prompt_input_json: String,
) -> GemAnalysisPartStageExecutionRequest {
    GemAnalysisPartStageExecutionRequest {
        run_id,
        stage_run_id: stage.stage_run_id,
        source_snapshot_id: stage.source_snapshot_id,
        source_ref_id: stage.source_ref_id.clone(),
        part,
        prompt_input_json,
    }
}

async fn ensure_gem_run_not_cancelled(
    pool: &SqlitePool,
    run_id: i64,
) -> Result<(), YoutubeSummaryStageExecutionError> {
    if is_run_cancelled(pool, run_id)
        .await
        .map_err(YoutubeSummaryStageExecutionError::Failed)?
    {
        return Err(YoutubeSummaryStageExecutionError::Cancelled);
    }
    Ok(())
}

async fn run_gem_part_with_one_repair<F, Fut>(
    execute_stage: &mut F,
    request: GemAnalysisPartStageExecutionRequest,
) -> Result<(GemAnalysisPartOutput, GemPartMetrics), YoutubeSummaryStageExecutionError>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
{
    let part = request.part;
    let completion = execute_stage(YoutubeSummaryStageExecutionRequest::GemAnalysisPart(
        request.clone(),
    ))
    .await?;
    let raw_output = completion.text.clone();
    let mut metrics = GemPartMetrics::succeeded(part, &completion);
    match parse_gem_analysis_part_output(&completion.text, part) {
        Ok(output) => return Ok((output, metrics)),
        Err(error) => {
            let repair_request = GemAnalysisPartRepairRequest {
                run_id: request.run_id,
                stage_run_id: request.stage_run_id,
                source_snapshot_id: request.source_snapshot_id,
                source_ref_id: request.source_ref_id.clone(),
                part,
                attempt_number: 1,
                prompt_input_json: request.prompt_input_json.clone(),
                raw_output,
                error_message: error.message,
            };
            let repair_completion = execute_stage(
                YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(repair_request),
            )
            .await?;
            metrics.add_repair_completion(&repair_completion);
            let output = parse_gem_analysis_part_output(&repair_completion.text, part)
                .map_err(YoutubeSummaryStageExecutionError::Failed)?;
            Ok((output, metrics))
        }
    }
}

fn gem_metrics_json(
    input_budget: GemAnalysisInputBudget,
    input_estimates: &[GemInputEstimate],
    part_metrics: &[GemPartMetrics],
    comments_metrics: &GemPartMetrics,
) -> serde_json::Value {
    serde_json::json!({
        "gem_analysis": {
            "input_budget": {
                "max_input_tokens": input_budget.max_input_tokens,
                "estimates": input_estimates
                    .iter()
                    .map(|estimate| {
                        serde_json::json!({
                            "part": estimate.part.as_str(),
                            "estimated_tokens": estimate.estimated_tokens,
                        })
                    })
                    .collect::<Vec<_>>(),
            },
            "parts": part_metrics
                .iter()
                .map(GemPartMetrics::to_json)
                .collect::<Vec<_>>(),
            "comments_part": comments_metrics.to_json(),
        }
    })
}

#[cfg(test)]
mod gem_analysis_part_tests {
    use super::{
        assemble_gem_analysis_markdown, assemble_gem_analysis_transcript_output,
        build_gem_analysis_part_prompt_input, enforce_gem_input_budget,
        load_gem_analysis_materials, parse_gem_analysis_part_output, GemAnalysisMaterialInput,
        GemCommentsStatus,
    };
    use crate::prompt_packs::youtube_summary::test_support::*;
    use crate::prompt_packs::youtube_summary::{
        start_youtube_summary_run_in_pool, GemAnalysisInputBudget, GemAnalysisPart,
        GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
        YoutubeSummaryStageExecutionRequest,
    };

    #[tokio::test]
    async fn gem_materials_load_formats_timestamped_transcript_from_metadata() {
        let pool = test_pool_with_ready_video().await;
        let request = start_request("req-gem-materials", vec![901]);
        let run = start_youtube_summary_run_in_pool(&pool, request)
            .await
            .expect("start")
            .expect_started("started");
        let stage_id = transcript_analysis_stage_id(&pool, run.run_id).await;

        let materials = load_gem_analysis_materials(&pool, stage_id)
            .await
            .expect("materials");

        assert!(materials.transcript_timestamped.contains("[00:00]"));
        assert!(materials.transcript_plain.contains("Ready transcript"));
        assert!(!materials.transcript_timestamped.contains("youtube_comment"));
        assert!(materials.timestamps_available);
    }

    #[tokio::test]
    async fn gem_materials_load_skips_empty_comment_rows() {
        let pool = test_pool_with_ready_video().await;
        insert_comment(&pool, 901, "empty-comment", 10, "").await;
        let mut request = start_request("req-gem-empty-comments", vec![901]);
        request.include_comments = true;
        let run = start_youtube_summary_run_in_pool(&pool, request)
            .await
            .expect("start")
            .expect_started("started");
        let stage_id = transcript_analysis_stage_id(&pool, run.run_id).await;

        let materials = load_gem_analysis_materials(&pool, stage_id)
            .await
            .expect("materials");

        assert_eq!(
            materials.comments_status,
            GemCommentsStatus::SkippedNoComments
        );
        assert!(materials.comments_text.trim().is_empty());
    }

    #[test]
    fn gem_materials_part_prompt_inputs_are_isolated() {
        let materials = GemAnalysisMaterialInput {
            transcript_plain: "plain transcript".to_string(),
            transcript_timestamped: "[00:00] transcript only".to_string(),
            timestamps_available: true,
            comments_text: "comment only".to_string(),
            comments_status: GemCommentsStatus::Present,
        };

        let passport = build_gem_analysis_part_prompt_input(
            GemAnalysisPart::Passport,
            "source_ref_1",
            &materials,
        );
        let comments = build_gem_analysis_part_prompt_input(
            GemAnalysisPart::Comments,
            "source_ref_1",
            &materials,
        );
        let deep_recap = build_gem_analysis_part_prompt_input(
            GemAnalysisPart::DeepRecap,
            "source_ref_1",
            &materials,
        );

        assert_eq!(passport["input_material"]["kind"], "transcript");
        assert!(passport["input_material"]["text"]
            .as_str()
            .unwrap()
            .contains("transcript only"));
        assert!(!passport.to_string().contains("comment only"));
        assert_eq!(
            comments["input_material"]["kind"],
            "selected_comment_sample"
        );
        assert!(comments["input_material"]["text"]
            .as_str()
            .unwrap()
            .contains("comment only"));
        assert!(!comments.to_string().contains("transcript only"));
        assert_eq!(deep_recap["input_material"]["kind"], "transcript");
        assert!(!deep_recap.to_string().contains("comment only"));
    }

    #[test]
    fn gem_materials_input_budget_rejects_over_cap() {
        let budget = GemAnalysisInputBudget {
            max_input_tokens: 100,
        };
        assert!(super::estimate_gem_prompt_tokens("abcd", "abcd") > 1_500);

        enforce_gem_input_budget(GemAnalysisPart::Passport, 101, budget).expect_err("over cap");
        enforce_gem_input_budget(GemAnalysisPart::Passport, 100, budget).expect("at cap");
    }

    #[test]
    fn assemble_gem_markdown_nests_part_markdown_under_backend_headings() {
        let markdown = assemble_gem_analysis_markdown(
            "### Metadata\nText",
            Some("### Sentiment\nText"),
            "### Recap\nText",
        );

        assert!(markdown.starts_with("# Gem-анализ"));
        assert!(markdown.contains("## Часть 1. Аналитический паспорт видео"));
        assert!(markdown.contains("### Metadata"));
        assert!(!markdown.contains("\n# Metadata"));
    }

    #[test]
    fn assemble_gem_transcript_output_contains_empty_candidate_arrays() {
        let output =
            assemble_gem_analysis_transcript_output("# Gem-анализ\n\nText").expect("output");
        let value: serde_json::Value = serde_json::from_str(&output).expect("json");

        assert_eq!(value["stage"], "youtube_summary/transcript_analysis");
        assert_eq!(value["claim_candidates"], serde_json::json!([]));
        assert_eq!(value["evidence_fragment_candidates"], serde_json::json!([]));
        assert!(value["video_candidate"]["summary_text"]
            .as_str()
            .unwrap()
            .starts_with("# Gem-анализ"));
    }

    #[test]
    fn gem_analysis_part_types_cover_comments_and_stage_variants() {
        let budget = GemAnalysisInputBudget {
            max_input_tokens: 24_000,
        };
        assert_eq!(budget.max_input_tokens, 24_000);
        assert_eq!(GemAnalysisPart::Comments.as_str(), "comments");
        assert_eq!(GemAnalysisPart::Comments.slug(), "comments");

        let part_request = GemAnalysisPartStageExecutionRequest {
            run_id: 1,
            stage_run_id: 2,
            source_snapshot_id: 3,
            source_ref_id: "source_ref_1".to_string(),
            part: GemAnalysisPart::Comments,
            prompt_input_json: "{}".to_string(),
        };
        let stage_request =
            YoutubeSummaryStageExecutionRequest::GemAnalysisPart(part_request.clone());
        assert!(matches!(
            stage_request,
            YoutubeSummaryStageExecutionRequest::GemAnalysisPart(request)
                if request.part == GemAnalysisPart::Comments
        ));

        let repair_request = GemAnalysisPartRepairRequest {
            run_id: 1,
            stage_run_id: 2,
            source_snapshot_id: 3,
            source_ref_id: "source_ref_1".to_string(),
            part: GemAnalysisPart::Comments,
            attempt_number: 2,
            prompt_input_json: "{}".to_string(),
            raw_output: "not json".to_string(),
            error_message: "parse failed".to_string(),
        };
        let stage_request =
            YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(repair_request.clone());
        assert!(matches!(
            stage_request,
            YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(request)
                if request.attempt_number == 2
        ));
    }

    #[test]
    fn parse_part_output_accepts_matching_non_empty_markdown() {
        let raw = serde_json::json!({
            "part": "passport",
            "markdown": "### Section\nText",
        })
        .to_string();

        let parsed =
            parse_gem_analysis_part_output(&raw, GemAnalysisPart::Passport).expect("parse");

        assert_eq!(parsed.part, GemAnalysisPart::Passport);
        assert_eq!(parsed.markdown, "### Section\nText");
    }

    #[test]
    fn parse_part_output_rejects_wrong_part() {
        let raw = serde_json::json!({
            "part": "comments",
            "markdown": "### Section",
        })
        .to_string();

        let error = parse_gem_analysis_part_output(&raw, GemAnalysisPart::Passport)
            .expect_err("wrong part");

        assert!(error.message.contains("expected part passport"));
    }

    #[test]
    fn parse_part_output_rejects_empty_markdown() {
        let error = parse_gem_analysis_part_output(
            r#"{"part":"passport","markdown":"   "}"#,
            GemAnalysisPart::Passport,
        )
        .expect_err("empty markdown");

        assert!(error.message.contains("markdown"));
    }

    #[test]
    fn parse_part_output_accepts_json_fence_with_internal_markdown_code_block() {
        let raw = "```json\n{\"part\":\"deep_recap\",\"markdown\":\"### Code\\n```rust\\nfn main() {}\\n```\\nFormula: $E=mc^2$\"}\n```";

        let parsed = parse_gem_analysis_part_output(raw, GemAnalysisPart::DeepRecap)
            .expect("parse fenced JSON with code block inside markdown string");

        assert!(parsed.markdown.contains("```rust"));
        assert!(parsed.markdown.contains("$E=mc^2$"));
    }
}
