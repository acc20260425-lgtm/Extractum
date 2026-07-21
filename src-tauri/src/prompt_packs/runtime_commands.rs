use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use super::browser_adapter::TauriGeminiBrowserPort;
use super::dto::{
    ListPromptPackRunsRequest, PreflightYoutubeSummaryRunRequest, PromptPackRunSummaryDto,
    PromptPackRuntimeProvider, PromptPackStageRunDto, StartYoutubeSummaryRunOutcomeDto,
    StartYoutubeSummaryRunRequest, YoutubeSummaryPreflightResponse,
};
use super::event_adapter::TauriPromptPackEventSink;
use super::events::PromptPackEventSink;
use super::runtime::{
    cancel_prompt_pack_run_in_pool, cleanup_interrupted_prompt_pack_runs_in_pool,
    delete_prompt_pack_run_in_pool, execute_prepared_api_run, execute_prepared_browser_run,
    fail_run_execution, list_active_prompt_pack_runs_in_pool, list_prompt_pack_run_stages_in_pool,
    list_prompt_pack_runs_in_pool,
    preflight_youtube_summary_run as preflight_youtube_summary_run_service, prepare_run_execution,
    start_youtube_summary_run_service, update_prompt_pack_run_in_pool, PreparedRunExecution,
    PromptPackRunState, RunExecutionTicket,
};
#[cfg(dev)]
use super::runtime::{
    clear_prompt_pack_cancellation_smoke_fixture_in_pool,
    seed_prompt_pack_cancellation_smoke_fixture_in_pool,
};
use super::source_adapter::AppPromptPackSourceReader;
use crate::db::get_pool;
use crate::error::AppResult;
use crate::llm::{resolve_profile_for_backend, LlmSchedulerState};

type ExecutionTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

fn dispatch_execution_ticket<T, Build, Spawn>(ticket: T, build: Build, spawn: Spawn)
where
    Build: FnOnce(T) -> ExecutionTask,
    Spawn: FnOnce(ExecutionTask),
{
    spawn(build(ticket));
}

#[tauri::command]
pub async fn preflight_youtube_summary_run(
    handle: AppHandle,
    project_id: Option<i64>,
    source_ids: Vec<i64>,
    profile_id: Option<String>,
    model_override: Option<String>,
    runtime_provider: Option<PromptPackRuntimeProvider>,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    output_language: String,
    control_preset: String,
    evidence_mode: String,
    include_comments: bool,
) -> AppResult<YoutubeSummaryPreflightResponse> {
    let pool = get_pool(&handle).await?;
    let source = AppPromptPackSourceReader::new(pool);
    preflight_youtube_summary_run_service(
        &source,
        PreflightYoutubeSummaryRunRequest::new(
            project_id,
            source_ids,
            profile_id,
            model_override,
            runtime_provider.unwrap_or_default(),
            browser_provider_config,
            output_language,
            control_preset,
            evidence_mode,
            include_comments,
        ),
    )
    .await
}

#[tauri::command]
pub async fn start_youtube_summary_run(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
    client_request_id: String,
    project_id: Option<i64>,
    source_ids: Vec<i64>,
    profile_id: Option<String>,
    model_override: Option<String>,
    runtime_provider: Option<PromptPackRuntimeProvider>,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    output_language: String,
    control_preset: String,
    evidence_mode: String,
    include_comments: bool,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    let pool = get_pool(&handle).await?;
    let source = AppPromptPackSourceReader::new(pool.clone());
    let browser = TauriGeminiBrowserPort::new(handle.clone());
    let events = TauriPromptPackEventSink::new(handle.clone());
    let outcome = start_youtube_summary_run_service(
        &pool,
        state.inner(),
        &source,
        &browser,
        &events,
        StartYoutubeSummaryRunRequest::new(
            client_request_id,
            project_id,
            source_ids,
            profile_id,
            model_override,
            runtime_provider.unwrap_or_default(),
            browser_provider_config,
            output_language,
            control_preset,
            evidence_mode,
            include_comments,
        ),
    )
    .await?;
    if let Some(ticket) = outcome.execution_ticket {
        spawn_youtube_summary_execution(handle, ticket);
    }
    Ok(outcome.response)
}

fn spawn_youtube_summary_execution(handle: AppHandle, ticket: RunExecutionTicket) {
    let build_handle = handle.clone();
    dispatch_execution_ticket(
        ticket,
        move |ticket| build_youtube_summary_execution_task(build_handle, ticket),
        |task| {
            tauri::async_runtime::spawn(task);
            // resolve_profile_for_backend runs only when the spawned task is polled.
        },
    );
}

fn build_youtube_summary_execution_task(
    handle: AppHandle,
    ticket: RunExecutionTicket,
) -> ExecutionTask {
    Box::pin(async move {
        let pool = match get_pool(&handle).await {
            Ok(pool) => pool,
            Err(error) => {
                eprintln!(
                    "Prompt Pack run {} could not acquire the application pool: {error}",
                    ticket.run_id()
                );
                return;
            }
        };
        let state = handle.state::<PromptPackRunState>();
        let events: Arc<dyn PromptPackEventSink> =
            Arc::new(TauriPromptPackEventSink::new(handle.clone()));
        let prepared = match prepare_run_execution(&pool, &ticket).await {
            Ok(value) => value,
            Err(error) => {
                if let Err(failure_error) =
                    fail_run_execution(&pool, state.inner(), events, &ticket, &error).await
                {
                    eprintln!(
                        "Prompt Pack run {} failed and could not be marked failed: {failure_error}",
                        ticket.run_id()
                    );
                }
                return;
            }
        };
        let result = match prepared {
            PreparedRunExecution::Api(api) => {
                match resolve_profile_for_backend(&handle, api.profile_id()).await {
                    Ok(profile) => {
                        execute_prepared_api_run(
                            &pool,
                            state.inner(),
                            handle.state::<LlmSchedulerState>().inner(),
                            events.clone(),
                            api,
                            profile,
                        )
                        .await
                    }
                    Err(error) => Err(error),
                }
            }
            PreparedRunExecution::GeminiBrowser(browser_run) => {
                execute_prepared_browser_run(
                    &pool,
                    state.inner(),
                    Arc::new(TauriGeminiBrowserPort::new(handle.clone())),
                    events.clone(),
                    browser_run,
                )
                .await
            }
        };
        if let Err(error) = result {
            if let Err(failure_error) =
                fail_run_execution(&pool, state.inner(), events, &ticket, &error).await
            {
                eprintln!(
                    "Prompt Pack run {} failed and could not be marked failed: {failure_error}",
                    ticket.run_id()
                );
            }
        }
    })
}

#[tauri::command]
pub async fn cancel_prompt_pack_run(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
    scheduler: State<'_, LlmSchedulerState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let events = TauriPromptPackEventSink::new(handle);
    cancel_prompt_pack_run_in_pool(&pool, state.inner(), scheduler.inner(), &events, run_id).await
}

#[tauri::command]
pub async fn update_prompt_pack_run(
    handle: AppHandle,
    run_id: i64,
    run_label: Option<String>,
) -> AppResult<PromptPackRunSummaryDto> {
    let pool = get_pool(&handle).await?;
    update_prompt_pack_run_in_pool(&pool, run_id, run_label).await
}

#[tauri::command]
pub async fn delete_prompt_pack_run(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_prompt_pack_run_in_pool(&pool, state.inner(), run_id).await
}

#[tauri::command]
pub async fn list_prompt_pack_runs(
    handle: AppHandle,
    project_id: Option<i64>,
    limit: Option<i64>,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let pool = get_pool(&handle).await?;
    list_prompt_pack_runs_in_pool(&pool, ListPromptPackRunsRequest::new(project_id, limit)).await
}

#[tauri::command]
pub async fn list_active_prompt_pack_runs(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let pool = get_pool(&handle).await?;
    list_active_prompt_pack_runs_in_pool(&pool, state.inner()).await
}

#[tauri::command]
pub async fn list_prompt_pack_run_stages(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Vec<PromptPackStageRunDto>> {
    let pool = get_pool(&handle).await?;
    list_prompt_pack_run_stages_in_pool(&pool, run_id).await
}

pub async fn cleanup_interrupted_prompt_pack_runs(handle: AppHandle) {
    match get_pool(&handle).await {
        Ok(pool) => {
            let state = handle.state::<PromptPackRunState>();
            if let Err(error) =
                cleanup_interrupted_prompt_pack_runs_in_pool(&pool, state.inner()).await
            {
                eprintln!("Prompt Pack cleanup failed: {error}");
            }
        }
        Err(error) => eprintln!("Prompt Pack cleanup skipped: {error}"),
    }
}

#[cfg(dev)]
#[tauri::command]
pub async fn seed_prompt_pack_cancellation_smoke_fixture(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<PromptPackRunSummaryDto> {
    let pool = get_pool(&handle).await?;
    seed_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, state.inner()).await
}

#[cfg(dev)]
#[tauri::command]
pub async fn clear_prompt_pack_cancellation_smoke_fixture(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<i64> {
    let pool = get_pool(&handle).await?;
    clear_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, state.inner()).await
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::{dispatch_execution_ticket, ExecutionTask};

    #[tokio::test]
    async fn execution_adapter_resolves_api_profile_only_inside_spawned_task() {
        let resolutions = Arc::new(AtomicUsize::new(0));
        let resolutions_for_task = resolutions.clone();
        let captured = Arc::new(Mutex::new(None::<ExecutionTask>));
        let captured_for_spawn = captured.clone();

        dispatch_execution_ticket(
            "opaque-ticket",
            move |_| {
                Box::pin(async move {
                    resolutions_for_task.fetch_add(1, Ordering::SeqCst);
                }) as Pin<Box<dyn Future<Output = ()> + Send>>
            },
            move |task| {
                *captured_for_spawn.lock().expect("captured task") = Some(task);
            },
        );

        assert_eq!(resolutions.load(Ordering::SeqCst), 0);
        let task = captured
            .lock()
            .expect("captured task")
            .take()
            .expect("spawned task");
        task.await;
        assert_eq!(resolutions.load(Ordering::SeqCst), 1);

        let source = include_str!("runtime_commands.rs");
        let task_body = source
            .split("fn build_youtube_summary_execution_task")
            .nth(1)
            .expect("execution task builder");
        assert!(
            task_body.find("Box::pin(async move").expect("async task")
                < task_body
                    .find("resolve_profile_for_backend")
                    .expect("profile resolution")
        );
    }

    #[test]
    fn execution_adapter_spawns_exactly_once_per_ticket() {
        let builds = Arc::new(AtomicUsize::new(0));
        let spawns = Arc::new(AtomicUsize::new(0));
        let builds_for_task = builds.clone();
        let spawns_for_adapter = spawns.clone();

        dispatch_execution_ticket(
            "opaque-ticket",
            move |_| {
                builds_for_task.fetch_add(1, Ordering::SeqCst);
                Box::pin(async {}) as ExecutionTask
            },
            move |_| {
                spawns_for_adapter.fetch_add(1, Ordering::SeqCst);
            },
        );

        assert_eq!(builds.load(Ordering::SeqCst), 1);
        assert_eq!(spawns.load(Ordering::SeqCst), 1);
    }
}
