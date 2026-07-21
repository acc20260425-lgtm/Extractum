use std::sync::Arc;

use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use super::completion_transport::{RunCompletionRuntime, StageCompletionRequest};
use super::events::PromptPackEventSink;
use super::json_repair::JsonRepairStageExecutionRequest;
use super::stage_request_policy::{
    build_gem_analysis_part_llm_request, build_gem_analysis_part_repair_llm_request,
    build_json_repair_llm_request, build_synthesis_llm_request,
    build_transcript_analysis_llm_request, gem_analysis_part_max_output_tokens,
    gem_part_repair_request_suffix, gem_part_request_suffix,
    synthesis_stage_max_output_token_budget, transcript_analysis_control_preset,
    transcript_analysis_max_output_tokens, transcript_analysis_stage_max_output_token_budget,
    transcript_analysis_stage_max_output_token_budget_for_control_preset,
};
use super::youtube_summary::{
    GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
    LlmCompletion as PromptPackLlmCompletion, SynthesisStageExecutionRequest,
    TranscriptAnalysisStageExecutionRequest, YoutubeSummaryStageExecutionError,
};
use crate::llm::LlmSchedulerState;

pub(super) async fn run_transcript_analysis_stage_request(
    pool: &SqlitePool,
    scheduler: Option<&LlmSchedulerState>,
    events: Arc<dyn PromptPackEventSink>,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: TranscriptAnalysisStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let control_preset = transcript_analysis_control_preset(&stage_request.prompt_input_json);
    let stage_output_budget =
        transcript_analysis_stage_max_output_token_budget_for_control_preset(&control_preset)?;
    let max_output_tokens = transcript_analysis_max_output_tokens(
        stage_output_budget,
        model_context.model_output_limit,
    );
    let llm_request = build_transcript_analysis_llm_request(
        &stage_request,
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );

    completion_runtime
        .execute(
            pool,
            scheduler,
            events,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: Some(stage_request.source_snapshot_id),
                stage_name: "youtube_summary/transcript_analysis".to_string(),
                phase: "transcript_analysis",
                started_message: "Analyzing transcript",
                repair_attempt_number: None,
                request_discriminator: None,
                run_cancellation_token,
            },
        )
        .await
}

pub(super) async fn run_synthesis_stage_request(
    pool: &SqlitePool,
    scheduler: Option<&LlmSchedulerState>,
    events: Arc<dyn PromptPackEventSink>,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: SynthesisStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let stage_output_budget = synthesis_stage_max_output_token_budget()?;
    let max_output_tokens = transcript_analysis_max_output_tokens(
        stage_output_budget,
        model_context.model_output_limit,
    );
    let llm_request = build_synthesis_llm_request(
        stage_request.run_id,
        stage_request.stage_run_id,
        stage_request.prompt_input_json.clone(),
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );

    completion_runtime
        .execute(
            pool,
            scheduler,
            events,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: None,
                stage_name: "youtube_summary/synthesis".to_string(),
                phase: "synthesis",
                started_message: "Synthesizing videos",
                repair_attempt_number: None,
                request_discriminator: None,
                run_cancellation_token,
            },
        )
        .await
}

pub(super) async fn run_json_repair_stage_request(
    pool: &SqlitePool,
    scheduler: Option<&LlmSchedulerState>,
    events: Arc<dyn PromptPackEventSink>,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: JsonRepairStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let stage_output_budget = if stage_request.stage_name == "youtube_summary/synthesis" {
        synthesis_stage_max_output_token_budget()?
    } else if stage_request.stage_name == "youtube_summary/transcript_analysis" {
        let control_preset = transcript_analysis_control_preset(&stage_request.prompt_input_json);
        transcript_analysis_stage_max_output_token_budget_for_control_preset(&control_preset)?
    } else {
        transcript_analysis_stage_max_output_token_budget()?
    };
    let max_output_tokens = transcript_analysis_max_output_tokens(
        stage_output_budget,
        model_context.model_output_limit,
    );
    let llm_request = build_json_repair_llm_request(
        &stage_request,
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );

    completion_runtime
        .execute(
            pool,
            scheduler,
            events,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: None,
                stage_name: stage_request.stage_name.clone(),
                phase: "repair",
                started_message: "Repairing provider JSON",
                repair_attempt_number: Some(stage_request.attempt_number),
                request_discriminator: None,
                run_cancellation_token,
            },
        )
        .await
}

pub(super) async fn run_gem_analysis_part_stage_request(
    pool: &SqlitePool,
    scheduler: Option<&LlmSchedulerState>,
    events: Arc<dyn PromptPackEventSink>,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: GemAnalysisPartStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let max_output_tokens =
        gem_analysis_part_max_output_tokens(stage_request.part, model_context.model_output_limit);
    let llm_request = build_gem_analysis_part_llm_request(
        &stage_request,
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );
    let phase = gem_part_phase(stage_request.part);
    let started_message = gem_part_started_message(stage_request.part);
    let request_discriminator = Some(gem_part_request_suffix(stage_request.part));

    completion_runtime
        .execute(
            pool,
            scheduler,
            events,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: Some(stage_request.source_snapshot_id),
                stage_name: "youtube_summary/transcript_analysis".to_string(),
                phase,
                started_message,
                repair_attempt_number: None,
                request_discriminator,
                run_cancellation_token,
            },
        )
        .await
}

pub(super) async fn run_gem_analysis_part_repair_request(
    pool: &SqlitePool,
    scheduler: Option<&LlmSchedulerState>,
    events: Arc<dyn PromptPackEventSink>,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: GemAnalysisPartRepairRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let max_output_tokens =
        gem_analysis_part_max_output_tokens(stage_request.part, model_context.model_output_limit);
    let llm_request = build_gem_analysis_part_repair_llm_request(
        &stage_request,
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );
    let request_discriminator = Some(gem_part_repair_request_suffix(
        stage_request.part,
        stage_request.attempt_number,
    ));

    completion_runtime
        .execute(
            pool,
            scheduler,
            events,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: Some(stage_request.source_snapshot_id),
                stage_name: "youtube_summary/transcript_analysis".to_string(),
                phase: "gem_part_repair",
                started_message: "Gem analysis: repairing part JSON",
                repair_attempt_number: None,
                request_discriminator,
                run_cancellation_token,
            },
        )
        .await
}

fn gem_part_phase(part: GemAnalysisPart) -> &'static str {
    match part {
        GemAnalysisPart::Passport => "gem_passport",
        GemAnalysisPart::Comments => "gem_comments",
        GemAnalysisPart::DeepRecap => "gem_deep_recap",
    }
}

fn gem_part_started_message(part: GemAnalysisPart) -> &'static str {
    match part {
        GemAnalysisPart::Passport => "Gem analysis: building analytical passport",
        GemAnalysisPart::Comments => "Gem analysis: analyzing comments",
        GemAnalysisPart::DeepRecap => "Gem analysis: writing deep recap",
    }
}
