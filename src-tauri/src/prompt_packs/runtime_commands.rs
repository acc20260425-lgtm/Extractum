use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use super::browser_adapter::TauriGeminiBrowserPort;
use super::event_adapter::TauriPromptPackEventSink;
use super::source_adapter::AppPromptPackSourceReader;
use crate::db::get_pool;
use crate::error::AppResult;
use crate::llm::{resolve_profile_for_backend, LlmSchedulerState};
use extractum_llm::ResolvedLlmProfile;
use extractum_prompt_packs::{
    cancel_prompt_pack_run_in_pool, cleanup_interrupted_prompt_pack_runs_in_pool,
    delete_prompt_pack_run_in_pool, dispatch_run_execution_ticket, execute_prepared_api_run,
    execute_prepared_browser_run, fail_run_execution, list_active_prompt_pack_runs_in_pool,
    list_prompt_pack_run_stages_in_pool, list_prompt_pack_runs_in_pool,
    preflight_youtube_summary_run as preflight_youtube_summary_run_service, prepare_run_execution,
    start_youtube_summary_run_service, update_prompt_pack_run_in_pool, ListPromptPackRunsRequest,
    PreflightYoutubeSummaryRunRequest, PreparedRunExecution, PromptPackEventSink,
    PromptPackRunState, PromptPackRunSummaryDto, PromptPackRuntimeProvider, PromptPackStageRunDto,
    RunExecutionTicket, StartYoutubeSummaryRunOutcomeDto, StartYoutubeSummaryRunRequest,
    YoutubeSummaryPreflightResponse,
};
#[cfg(all(dev, feature = "prompt-pack-dev-fixtures"))]
use extractum_prompt_packs::{
    clear_prompt_pack_cancellation_smoke_fixture_in_pool,
    seed_prompt_pack_cancellation_smoke_fixture_in_pool,
};

type ExecutionTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
type ExecutionPortFuture<'a, T> = Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

trait YoutubeSummaryExecutionPort: Send + Sync + 'static {
    fn resolve_profile<'a>(
        &'a self,
        profile_id: Option<&'a str>,
    ) -> ExecutionPortFuture<'a, ResolvedLlmProfile>;
    fn execute_api<'a>(
        &'a self,
        pool: &'a SqlitePool,
        events: Arc<dyn PromptPackEventSink>,
        prepared: extractum_prompt_packs::PreparedApiRunExecution,
        profile: ResolvedLlmProfile,
    ) -> ExecutionPortFuture<'a, ()>;
    fn execute_browser<'a>(
        &'a self,
        pool: &'a SqlitePool,
        events: Arc<dyn PromptPackEventSink>,
        prepared: extractum_prompt_packs::PreparedBrowserRunExecution,
    ) -> ExecutionPortFuture<'a, ()>;
    fn fail<'a>(
        &'a self,
        pool: &'a SqlitePool,
        events: Arc<dyn PromptPackEventSink>,
        ticket: &'a RunExecutionTicket,
        error: &'a crate::error::AppError,
    ) -> ExecutionPortFuture<'a, ()>;
    fn events(&self) -> Arc<dyn PromptPackEventSink>;
}

struct TauriYoutubeSummaryExecutionPort {
    handle: AppHandle,
}

impl YoutubeSummaryExecutionPort for TauriYoutubeSummaryExecutionPort {
    fn resolve_profile<'a>(
        &'a self,
        profile_id: Option<&'a str>,
    ) -> ExecutionPortFuture<'a, ResolvedLlmProfile> {
        Box::pin(resolve_profile_for_backend(&self.handle, profile_id))
    }

    fn execute_api<'a>(
        &'a self,
        pool: &'a SqlitePool,
        events: Arc<dyn PromptPackEventSink>,
        prepared: extractum_prompt_packs::PreparedApiRunExecution,
        profile: ResolvedLlmProfile,
    ) -> ExecutionPortFuture<'a, ()> {
        Box::pin(async move {
            execute_prepared_api_run(
                pool,
                self.handle.state::<PromptPackRunState>().inner(),
                self.handle
                    .state::<Arc<LlmSchedulerState>>()
                    .inner()
                    .as_ref(),
                events,
                prepared,
                profile,
            )
            .await
            .map(|_| ())
        })
    }

    fn execute_browser<'a>(
        &'a self,
        pool: &'a SqlitePool,
        events: Arc<dyn PromptPackEventSink>,
        prepared: extractum_prompt_packs::PreparedBrowserRunExecution,
    ) -> ExecutionPortFuture<'a, ()> {
        Box::pin(async move {
            execute_prepared_browser_run(
                pool,
                self.handle.state::<PromptPackRunState>().inner(),
                Arc::new(TauriGeminiBrowserPort::new(self.handle.clone())),
                events,
                prepared,
            )
            .await
            .map(|_| ())
        })
    }

    fn fail<'a>(
        &'a self,
        pool: &'a SqlitePool,
        events: Arc<dyn PromptPackEventSink>,
        ticket: &'a RunExecutionTicket,
        error: &'a crate::error::AppError,
    ) -> ExecutionPortFuture<'a, ()> {
        Box::pin(fail_run_execution(
            pool,
            self.handle.state::<PromptPackRunState>().inner(),
            events,
            ticket,
            error,
        ))
    }

    fn events(&self) -> Arc<dyn PromptPackEventSink> {
        Arc::new(TauriPromptPackEventSink::new(self.handle.clone()))
    }
}

#[tauri::command]
// Preserve the established IPC request shape rather than introducing a Wave 0 wire change.
#[allow(clippy::too_many_arguments)]
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
// Preserve the established IPC request shape rather than introducing a Wave 0 wire change.
#[allow(clippy::too_many_arguments)]
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
        spawn_youtube_summary_execution(handle, pool, ticket);
    }
    Ok(outcome.response)
}

fn spawn_youtube_summary_execution(
    handle: AppHandle,
    pool: SqlitePool,
    ticket: RunExecutionTicket,
) {
    let execution_port = Arc::new(TauriYoutubeSummaryExecutionPort { handle });
    dispatch_run_execution_ticket(
        ticket,
        move |ticket| build_youtube_summary_execution_task(pool, ticket, execution_port),
        |task| {
            tauri::async_runtime::spawn(task);
            // resolve_profile_for_backend runs only when the spawned task is polled.
        },
    );
}

fn build_youtube_summary_execution_task(
    pool: SqlitePool,
    ticket: RunExecutionTicket,
    execution_port: Arc<dyn YoutubeSummaryExecutionPort>,
) -> ExecutionTask {
    Box::pin(async move {
        let events = execution_port.events();
        let prepared = match prepare_run_execution(&pool, &ticket).await {
            Ok(value) => value,
            Err(error) => {
                if let Err(failure_error) =
                    execution_port.fail(&pool, events, &ticket, &error).await
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
                match execution_port.resolve_profile(api.profile_id()).await {
                    Ok(profile) => {
                        execution_port
                            .execute_api(&pool, events.clone(), api, profile)
                            .await
                    }
                    Err(error) => Err(error),
                }
            }
            PreparedRunExecution::GeminiBrowser(browser_run) => {
                execution_port
                    .execute_browser(&pool, events.clone(), browser_run)
                    .await
            }
        };
        if let Err(error) = result {
            if let Err(failure_error) = execution_port.fail(&pool, events, &ticket, &error).await {
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
    scheduler: State<'_, Arc<LlmSchedulerState>>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let events = TauriPromptPackEventSink::new(handle);
    cancel_prompt_pack_run_in_pool(
        &pool,
        state.inner(),
        scheduler.inner().as_ref(),
        &events,
        run_id,
    )
    .await
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

#[cfg(all(dev, feature = "prompt-pack-dev-fixtures"))]
#[tauri::command]
pub async fn seed_prompt_pack_cancellation_smoke_fixture(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<PromptPackRunSummaryDto> {
    crate::security_config::require_local_dev_command(crate::security_config::MCP_BIND_ADDRESS)?;
    let pool = get_pool(&handle).await?;
    seed_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, state.inner()).await
}

#[cfg(all(dev, feature = "prompt-pack-dev-fixtures"))]
#[tauri::command]
pub async fn clear_prompt_pack_cancellation_smoke_fixture(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<i64> {
    crate::security_config::require_local_dev_command(crate::security_config::MCP_BIND_ADDRESS)?;
    let pool = get_pool(&handle).await?;
    clear_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, state.inner()).await
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::{
        build_youtube_summary_execution_task, ExecutionPortFuture, ExecutionTask,
        YoutubeSummaryExecutionPort,
    };
    use extractum_core::error::AppError;
    use extractum_llm::ResolvedLlmProfile;
    use extractum_prompt_packs::{
        dispatch_run_execution_ticket, PreparedApiRunExecution, PreparedBrowserRunExecution,
        PromptPackEvent, PromptPackEventSink, RunExecutionTicket,
    };
    use sqlx::SqlitePool;

    #[derive(Default)]
    struct RecordingExecutionPort {
        resolutions: AtomicUsize,
    }

    impl PromptPackEventSink for RecordingExecutionPort {
        fn emit(&self, _event: PromptPackEvent) {}
    }

    impl YoutubeSummaryExecutionPort for RecordingExecutionPort {
        fn resolve_profile<'a>(
            &'a self,
            profile_id: Option<&'a str>,
        ) -> ExecutionPortFuture<'a, ResolvedLlmProfile> {
            assert_eq!(profile_id, Some("profile-91"));
            self.resolutions.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err(AppError::internal("recording resolver")) })
        }

        fn execute_api<'a>(
            &'a self,
            _pool: &'a SqlitePool,
            _events: Arc<dyn PromptPackEventSink>,
            _prepared: PreparedApiRunExecution,
            _profile: ResolvedLlmProfile,
        ) -> ExecutionPortFuture<'a, ()> {
            panic!("recording resolver must stop before API execution")
        }

        fn execute_browser<'a>(
            &'a self,
            _pool: &'a SqlitePool,
            _events: Arc<dyn PromptPackEventSink>,
            _prepared: PreparedBrowserRunExecution,
        ) -> ExecutionPortFuture<'a, ()> {
            panic!("API ticket must not execute the browser path")
        }

        fn fail<'a>(
            &'a self,
            _pool: &'a SqlitePool,
            _events: Arc<dyn PromptPackEventSink>,
            _ticket: &'a RunExecutionTicket,
            _error: &'a AppError,
        ) -> ExecutionPortFuture<'a, ()> {
            Box::pin(async { Ok(()) })
        }

        fn events(&self) -> Arc<dyn PromptPackEventSink> {
            Arc::new(RecordingExecutionPort::default())
        }
    }

    #[tokio::test]
    async fn execution_adapter_resolves_api_profile_only_inside_spawned_task() {
        let resolutions = Arc::new(AtomicUsize::new(0));
        let resolutions_for_task = resolutions.clone();
        let captured = Arc::new(Mutex::new(None::<ExecutionTask>));
        let captured_for_spawn = captured.clone();

        dispatch_run_execution_ticket(
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
    }

    #[tokio::test]
    async fn build_youtube_summary_execution_task_defers_profile_resolution_until_spawned_future_is_polled(
    ) {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("test pool");
        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("prompt-pack schema");
        sqlx::query("INSERT INTO prompt_packs (pack_id, display_name, created_at, updated_at) VALUES ('youtube-summary', 'YouTube Summary', 1, 1)")
            .execute(&pool)
            .await
            .expect("prompt pack");
        sqlx::query("INSERT INTO prompt_pack_versions (id, pack_id, pack_version, schema_version, origin_kind, lifecycle_status, content_hash, created_at, updated_at) VALUES (1, 'youtube-summary', '1', '1', 'bundled', 'active', 'hash', 1, 1)")
            .execute(&pool)
            .await
            .expect("prompt pack version");
        sqlx::query("INSERT INTO prompt_pack_runs (id, pack_version_id, pack_id, pack_version, schema_version, run_status, provider_profile_id, output_language, control_preset, evidence_mode, created_at, updated_at, runtime_provider) VALUES (91, 1, 'youtube-summary', '1', '1', 'queued', 'profile-91', 'en', 'standard', 'standard', '2026-08-08T00:00:00Z', '2026-08-08T00:00:00Z', 'api')")
            .execute(&pool)
            .await
            .expect("prompt pack run");

        let execution_port = Arc::new(RecordingExecutionPort::default());
        let observed_port = execution_port.clone();
        let captured = Arc::new(Mutex::new(None::<ExecutionTask>));
        let captured_for_spawn = captured.clone();

        dispatch_run_execution_ticket(
            RunExecutionTicket::for_app_test(91),
            move |ticket| build_youtube_summary_execution_task(pool, ticket, execution_port),
            move |task| {
                *captured_for_spawn.lock().expect("captured task") = Some(task);
            },
        );

        assert_eq!(observed_port.resolutions.load(Ordering::SeqCst), 0);
        let task = captured
            .lock()
            .expect("captured task")
            .take()
            .expect("spawned task");
        task.await;
        assert_eq!(observed_port.resolutions.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn execution_adapter_spawns_exactly_once_per_ticket() {
        let builds = Arc::new(AtomicUsize::new(0));
        let spawns = Arc::new(AtomicUsize::new(0));
        let builds_for_task = builds.clone();
        let spawns_for_adapter = spawns.clone();

        dispatch_run_execution_ticket(
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

    #[test]
    fn execution_task_reuses_start_pool_without_reacquisition() {
        let source = include_str!("runtime_commands.rs");
        let start_body = source
            .split("pub async fn start_youtube_summary_run")
            .nth(1)
            .expect("start command")
            .split("fn spawn_youtube_summary_execution")
            .next()
            .expect("start command body");
        assert!(start_body.contains("spawn_youtube_summary_execution(handle, pool, ticket);"));

        let task_body = source
            .split("fn build_youtube_summary_execution_task")
            .nth(1)
            .expect("execution task builder")
            .split("#[tauri::command]")
            .next()
            .expect("execution task body");
        assert!(
            !task_body.contains("get_pool("),
            "the spawned task must reuse the pool acquired by the start command"
        );
    }
}
