use std::sync::Arc;

use extractum_analysis::{
    clear_analysis_chat_messages as clear_analysis_chat_messages_in_pool, complete_analysis_chat,
    execute_analysis_chat, list_analysis_chat_messages as list_analysis_chat_messages_in_pool,
    prepare_analysis_chat, publish_analysis_chat_execution_error,
    publish_analysis_chat_persistence_error, AnalysisChatMessage, AnalysisExecutionError,
    AskAnalysisRunQuestionRequest,
};
use extractum_core::error::AppResult;
use extractum_llm::LlmSchedulerState;
use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::llm::resolve_profile_for_backend;

use super::events::TauriAnalysisEventSink;
use super::store::resolve_legacy_analysis_chat_run_in_pool;

#[tauri::command]
pub async fn list_analysis_chat_messages(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Vec<AnalysisChatMessage>> {
    let pool = get_pool(&handle).await?;
    list_analysis_chat_messages_in_pool(&pool, run_id).await
}

#[tauri::command]
pub async fn clear_analysis_chat_messages(handle: AppHandle, run_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    clear_analysis_chat_messages_in_pool(&pool, run_id).await
}

#[tauri::command]
pub async fn ask_analysis_run_question(
    handle: AppHandle,
    run_id: i64,
    question: String,
    model_override: Option<String>,
    profile_id: Option<String>,
) -> AppResult<String> {
    let request = AskAnalysisRunQuestionRequest::new(run_id, question, model_override, profile_id)?;
    let pool = get_pool(&handle).await?;
    let run = resolve_legacy_analysis_chat_run_in_pool(&pool, run_id).await?;
    let ticket = prepare_analysis_chat(&pool, request, run).await?;
    let request_id = ticket.request_id().to_string();
    let profile_id = ticket.profile_id().to_string();
    let emitted_request_id = request_id.clone();
    let app_handle = handle.clone();
    tokio::spawn(async move {
        let sink = Arc::new(TauriAnalysisEventSink::new(app_handle.clone()));
        let resolved_profile =
            match resolve_profile_for_backend(&app_handle, Some(profile_id.as_str())).await {
                Ok(profile) => profile,
                Err(error) => {
                    let error = AnalysisExecutionError::Failed(String::from(error));
                    publish_analysis_chat_execution_error(
                        sink.as_ref(),
                        &emitted_request_id,
                        run_id,
                        &error,
                    );
                    return;
                }
            };
        let scheduler = app_handle.state::<Arc<LlmSchedulerState>>().inner().clone();
        let completion =
            match execute_analysis_chat(scheduler, sink.clone(), ticket, resolved_profile).await {
                Ok(completion) => completion,
                Err(error) => {
                    publish_analysis_chat_execution_error(
                        sink.as_ref(),
                        &emitted_request_id,
                        run_id,
                        &error,
                    );
                    return;
                }
            };
        let pool = match get_pool(&app_handle).await {
            Ok(pool) => pool,
            Err(error) => {
                publish_analysis_chat_persistence_error(
                    sink.as_ref(),
                    &emitted_request_id,
                    run_id,
                    &error,
                );
                return;
            }
        };
        if let Err(error) = complete_analysis_chat(&pool, sink.as_ref(), completion).await {
            publish_analysis_chat_persistence_error(
                sink.as_ref(),
                &emitted_request_id,
                run_id,
                &error,
            );
        }
    });

    Ok(request_id)
}
