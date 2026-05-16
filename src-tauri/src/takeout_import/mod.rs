#![allow(clippy::needless_borrow, clippy::too_many_arguments)]

use grammers_client::{tl, Client};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::sources::{
    finalize_sync, insert_source_item, load_source, require_source_identity_ready,
    resolve_and_refresh_peer, SourceIdentityRepairState, TelegramSourceKind, TELEGRAM_KIND_CHANNEL,
    TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
};
use crate::telegram::{get_authorized_runtime, AuthorizedTelegramRuntime, TelegramState};

mod export_dc;
mod pagination;
#[allow(dead_code)]
mod raw_parse;
mod state;

use export_dc::{
    export_dc_invoke, finish_takeout_session, prepare_export_dc_alias,
    takeout_init_request_for_source_kind, ExportDcAlias,
};
use pagination::{
    message_range_min_id, next_takeout_cursor, parse_takeout_page, select_history_splits,
    should_restart_with_descending_fallback, takeout_page_request,
    takeout_pagination_fallback_warning, TakeoutPageRequest, TakeoutPaginationCursor,
    TakeoutPaginationProfile,
};
pub use state::TakeoutImportState;
use state::{
    emit_takeout_import_event, update_and_emit, CancelTakeoutImportResponse,
    StartTakeoutImportResponse, TakeoutImportJobRecord, PHASE_CANCELLED, PHASE_COMPLETED,
    PHASE_COUNTING, PHASE_FAILED, PHASE_FINISHING_TAKEOUT, PHASE_IMPORTING_HISTORY,
    PHASE_LOADING_SPLITS, PHASE_RESOLVING_SOURCE, PHASE_STARTING_TAKEOUT, PHASE_VALIDATING_PEER,
    STATUS_CANCELLED, STATUS_COMPLETED, STATUS_FAILED, STATUS_RUNNING,
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutExportDcSpikeResult {
    pub(crate) source_id: i64,
    pub(crate) account_id: i64,
    pub(crate) telegram_source_kind: String,
    pub(crate) home_dc_id: i32,
    pub(crate) export_dc_id: i32,
    pub(crate) used_export_dc: bool,
    pub(crate) fallback_used: bool,
    pub(crate) takeout_id: i64,
    pub(crate) split_count: usize,
    pub(crate) warnings: Vec<String>,
}

#[tauri::command]
pub async fn start_takeout_source_import(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, TakeoutImportState>,
    source_id: i64,
) -> AppResult<StartTakeoutImportResponse> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;
    let record = state.create_job(source_id, account_id).await?;
    emit_takeout_import_event(&handle, &record);

    let job_id = record.job_id.clone();
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_takeout_import_job(task_handle, job_id).await;
    });

    Ok(StartTakeoutImportResponse {
        job_id: record.job_id,
    })
}

#[tauri::command]
pub async fn cancel_takeout_source_import(
    handle: AppHandle,
    state: tauri::State<'_, TakeoutImportState>,
    job_id: String,
) -> AppResult<CancelTakeoutImportResponse> {
    let Some(record) = state.request_cancel(&job_id).await else {
        return Ok(CancelTakeoutImportResponse { cancelled: false });
    };
    emit_takeout_import_event(&handle, &record);
    Ok(CancelTakeoutImportResponse { cancelled: true })
}

#[tauri::command]
pub async fn list_takeout_source_import_jobs(
    state: tauri::State<'_, TakeoutImportState>,
) -> AppResult<Vec<TakeoutImportJobRecord>> {
    Ok(state.list_jobs().await)
}

#[tauri::command]
pub async fn run_takeout_export_dc_spike(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, TelegramState>,
    source_id: i64,
) -> AppResult<TakeoutExportDcSpikeResult> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;
    let runtime = get_authorized_runtime(&state, account_id).await?;

    run_export_dc_spike_for_runtime(source.id, account_id, &source.telegram_source_kind, runtime)
        .await
}

async fn run_export_dc_spike_for_runtime(
    source_id: i64,
    account_id: i64,
    telegram_source_kind: &str,
    runtime: AuthorizedTelegramRuntime,
) -> AppResult<TakeoutExportDcSpikeResult> {
    let client = runtime.client;
    client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![tl::enums::InputUser::UserSelf],
        })
        .await
        .map_err(|e| AppError::network(format!("Telegram self check failed: {e}")))?;

    let alias = prepare_export_dc_alias(&runtime.session).await?;
    let init_request = takeout_init_request_for_source_kind(telegram_source_kind)?;
    let mut warnings = Vec::new();
    let mut fallback_used = false;

    let takeout = export_dc_invoke(
        &client,
        &alias,
        &init_request,
        &mut warnings,
        &mut fallback_used,
    )
    .await?;
    let tl::enums::account::Takeout::Takeout(takeout) = takeout;
    let takeout_id = takeout.id;

    let split_ranges = export_dc_invoke(
        &client,
        &alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::messages::GetSplitRanges {},
        },
        &mut warnings,
        &mut fallback_used,
    )
    .await?;

    export_dc_invoke(
        &client,
        &alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::account::FinishTakeoutSession { success: true },
        },
        &mut warnings,
        &mut fallback_used,
    )
    .await?;

    Ok(TakeoutExportDcSpikeResult {
        source_id,
        account_id,
        telegram_source_kind: telegram_source_kind.to_string(),
        home_dc_id: alias.home_dc_id,
        export_dc_id: alias.export_dc_id,
        used_export_dc: !fallback_used,
        fallback_used,
        takeout_id,
        split_count: split_ranges.len(),
        warnings,
    })
}

async fn run_takeout_import_job(handle: AppHandle, job_id: String) {
    let takeout_state = handle.state::<TakeoutImportState>();
    let ingest_locks = handle.state::<SourceIngestLocks>();

    let Some(running_record) = takeout_state
        .update_job(&job_id, |job| {
            job.status = STATUS_RUNNING.to_string();
            job.phase = PHASE_RESOLVING_SOURCE.to_string();
            job.message = Some("Preparing Takeout import.".to_string());
        })
        .await
    else {
        return;
    };
    emit_takeout_import_event(&handle, &running_record);

    let ingest_guard = match ingest_locks
        .try_acquire(running_record.source_id, SourceIngestKind::TakeoutImport)
        .await
    {
        Ok(guard) => guard,
        Err(error) => {
            if let Some(record) = takeout_state
                .finish_job(&job_id, |job| {
                    job.status = STATUS_FAILED.to_string();
                    job.phase = PHASE_FAILED.to_string();
                    job.message = None;
                    job.error = Some(error.to_string());
                })
                .await
            {
                emit_takeout_import_event(&handle, &record);
            }
            return;
        }
    };

    if takeout_state.is_cancel_requested(&job_id).await {
        if let Some(record) = takeout_state
            .finish_job(&job_id, |job| {
                job.status = STATUS_CANCELLED.to_string();
                job.phase = PHASE_CANCELLED.to_string();
                job.message = Some("Takeout import cancelled.".to_string());
            })
            .await
        {
            emit_takeout_import_event(&handle, &record);
        }
        drop(ingest_guard);
        return;
    }

    match run_takeout_source_import(&handle, &job_id).await {
        Ok(outcome) => {
            if let Some(record) = takeout_state
                .finish_job(&job_id, |job| {
                    job.status = STATUS_COMPLETED.to_string();
                    job.phase = PHASE_COMPLETED.to_string();
                    job.message = Some(format!(
                        "Takeout import completed. Inserted {}, skipped {}.",
                        outcome.inserted, outcome.skipped
                    ));
                    job.inserted = outcome.inserted;
                    job.skipped = outcome.skipped;
                    job.progress_current = outcome.progress_total;
                    job.progress_total = outcome.progress_total;
                    job.warnings = outcome.warnings;
                })
                .await
            {
                emit_takeout_import_event(&handle, &record);
            }
        }
        Err(error) => {
            if takeout_state.is_cancel_requested(&job_id).await {
                if let Some(record) = takeout_state
                    .finish_job(&job_id, |job| {
                        job.status = STATUS_CANCELLED.to_string();
                        job.phase = PHASE_CANCELLED.to_string();
                        job.message = Some("Takeout import cancelled.".to_string());
                    })
                    .await
                {
                    emit_takeout_import_event(&handle, &record);
                }
            } else if let Some(record) = takeout_state
                .finish_job(&job_id, |job| {
                    job.status = STATUS_FAILED.to_string();
                    job.phase = PHASE_FAILED.to_string();
                    job.message = None;
                    job.error = Some(error.to_string());
                })
                .await
            {
                emit_takeout_import_event(&handle, &record);
            }
        }
    }
    drop(ingest_guard);
}

struct TakeoutImportOutcome {
    inserted: i64,
    skipped: i64,
    progress_total: Option<i64>,
    warnings: Vec<String>,
}

async fn run_takeout_source_import(
    handle: &AppHandle,
    job_id: &str,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let telegram_state = handle.state::<TelegramState>();
    let repair_state = handle.state::<SourceIdentityRepairState>();
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(handle).await?;
    let source_id = takeout_state
        .update_job(job_id, |_| {})
        .await
        .ok_or_else(|| AppError::internal(format!("Takeout job {job_id} not found")))?
        .source_id;
    let source = load_source(&pool, source_id).await?;
    ensure_supported_takeout_source_kind(&source.telegram_source_kind)?;

    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {} is not linked to an account", source.id))
    })?;
    let runtime = get_authorized_runtime(&telegram_state, account_id).await?;
    let client = runtime.client;
    let session = runtime.session;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_RESOLVING_SOURCE.to_string();
        job.message = Some("Resolving Telegram source.".to_string());
    })
    .await;
    let resolved_peer =
        resolve_and_refresh_peer(handle, &pool, &client, &source, account_id).await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_STARTING_TAKEOUT.to_string();
        job.message = Some("Starting Takeout session.".to_string());
    })
    .await;
    client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![tl::enums::InputUser::UserSelf],
        })
        .await
        .map_err(|e| AppError::network(format!("Telegram self check failed: {e}")))?;
    let alias = prepare_export_dc_alias(&session).await?;
    let init_request = takeout_init_request_for_source_kind(&source.telegram_source_kind)?;
    let mut warnings = Vec::new();
    let mut fallback_used = false;
    let takeout = export_dc_invoke(
        &client,
        &alias,
        &init_request,
        &mut warnings,
        &mut fallback_used,
    )
    .await?;
    let tl::enums::account::Takeout::Takeout(takeout) = takeout;
    let takeout_id = takeout.id;

    let started_result = run_started_takeout_source_import(
        handle,
        job_id,
        &pool,
        &source,
        resolved_peer,
        &client,
        &alias,
        takeout_id,
        warnings,
        fallback_used,
    )
    .await;

    match started_result {
        Ok(outcome) => Ok(outcome),
        Err((error, mut warnings, mut fallback_used)) => {
            if let Err(finish_error) = finish_takeout_session(
                &client,
                &alias,
                takeout_id,
                false,
                &mut warnings,
                &mut fallback_used,
            )
            .await
            {
                warnings.push(format!(
                    "Failed to finish Takeout session after error: {finish_error}"
                ));
            }
            update_and_emit(handle, &takeout_state, job_id, |job| {
                job.warnings = warnings;
            })
            .await;
            Err(error)
        }
    }
}

async fn run_started_takeout_source_import(
    handle: &AppHandle,
    job_id: &str,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &crate::sources::SourceSyncTarget,
    resolved_peer: crate::sources::ResolvedSyncPeer,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    mut warnings: Vec<String>,
    mut fallback_used: bool,
) -> Result<TakeoutImportOutcome, (AppError, Vec<String>, bool)> {
    match run_started_takeout_source_import_inner(
        handle,
        job_id,
        pool,
        source,
        resolved_peer,
        client,
        alias,
        takeout_id,
        &mut warnings,
        &mut fallback_used,
    )
    .await
    {
        Ok(outcome) => Ok(outcome),
        Err(error) => Err((error, warnings, fallback_used)),
    }
}

async fn run_started_takeout_source_import_inner(
    handle: &AppHandle,
    job_id: &str,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &crate::sources::SourceSyncTarget,
    resolved_peer: crate::sources::ResolvedSyncPeer,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let input_peer: tl::enums::InputPeer = resolved_peer.peer.into();

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_VALIDATING_PEER.to_string();
        job.message = Some("Validating Telegram source.".to_string());
        job.warnings.extend(warnings.clone());
    })
    .await;
    validate_takeout_peer(
        &client,
        &alias,
        takeout_id,
        &source.telegram_source_kind,
        resolved_peer.peer,
        warnings,
        fallback_used,
    )
    .await?;
    detect_supergroup_migration(
        client,
        alias,
        takeout_id,
        &source.telegram_source_kind,
        resolved_peer.peer,
        warnings,
        fallback_used,
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_LOADING_SPLITS.to_string();
        job.message = Some("Loading Takeout message ranges.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    let split_ranges = export_dc_invoke(
        &client,
        &alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::messages::GetSplitRanges {},
        },
        warnings,
        fallback_used,
    )
    .await?;
    let selected_ranges = select_history_splits(&source.telegram_source_kind, split_ranges)?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_COUNTING.to_string();
        job.message = Some("Counting messages.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    let mut counted_ranges = Vec::new();
    let mut total = 0_i64;
    for range in selected_ranges {
        let probe = takeout_history_count_probe(
            &client,
            &alias,
            takeout_id,
            input_peer.clone(),
            range.clone(),
            &source.telegram_source_kind,
            warnings,
            fallback_used,
        )
        .await?;
        total += probe.count;
        counted_ranges.push(CountedMessageRange {
            range,
            count: probe.count,
            only_my_messages: probe.only_my_messages,
        });
    }

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_IMPORTING_HISTORY.to_string();
        job.message = Some("Importing history.".to_string());
        job.progress_current = Some(0);
        job.progress_total = Some(total);
        job.warnings = warnings.to_vec();
    })
    .await;
    let import = import_takeout_history_ranges(
        handle,
        job_id,
        &client,
        &alias,
        takeout_id,
        input_peer,
        counted_ranges,
        &source,
        total,
        &source.telegram_source_kind,
        warnings,
        fallback_used,
    )
    .await?;

    if takeout_state.is_cancel_requested(job_id).await {
        return Err(AppError::validation("Takeout import cancelled"));
    }

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_FINISHING_TAKEOUT.to_string();
        job.message = Some("Finishing Takeout session.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    finish_takeout_session(client, alias, takeout_id, true, warnings, fallback_used).await?;
    finalize_sync(
        &pool,
        &source,
        source.last_sync_state.unwrap_or(0),
        import.max_message_id,
        resolved_peer.refreshed_avatar_cache_key,
    )
    .await?;

    Ok(TakeoutImportOutcome {
        inserted: import.inserted,
        skipped: import.skipped,
        progress_total: Some(total),
        warnings: warnings.to_vec(),
    })
}

struct TakeoutHistoryImport {
    inserted: i64,
    skipped: i64,
    max_message_id: i64,
}

struct CountedMessageRange {
    range: tl::enums::MessageRange,
    count: i64,
    only_my_messages: bool,
}

struct TakeoutHistoryProbe {
    count: i64,
    only_my_messages: bool,
}

fn ensure_supported_takeout_source_kind(telegram_source_kind: &str) -> AppResult<()> {
    TelegramSourceKind::parse(telegram_source_kind).map(|_| ())
}

async fn validate_takeout_peer(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    telegram_source_kind: &str,
    peer: grammers_session::types::PeerRef,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<()> {
    match telegram_source_kind {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP => {
            let input_channel: tl::enums::InputChannel = peer.into();
            export_dc_invoke(
                client,
                alias,
                &tl::functions::InvokeWithTakeout {
                    takeout_id,
                    query: tl::functions::channels::GetChannels {
                        id: vec![input_channel],
                    },
                },
                warnings,
                fallback_used,
            )
            .await?;
        }
        TELEGRAM_KIND_GROUP => {
            export_dc_invoke(
                client,
                alias,
                &tl::functions::InvokeWithTakeout {
                    takeout_id,
                    query: tl::functions::messages::GetChats {
                        id: vec![peer.id.bare_id()],
                    },
                },
                warnings,
                fallback_used,
            )
            .await?;
        }
        other => {
            return Err(AppError::validation(format!(
                "Unsupported telegram_source_kind '{other}'"
            )));
        }
    }

    Ok(())
}

async fn detect_supergroup_migration(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    telegram_source_kind: &str,
    peer: grammers_session::types::PeerRef,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<()> {
    if telegram_source_kind != TELEGRAM_KIND_SUPERGROUP {
        return Ok(());
    }

    let input_channel: tl::enums::InputChannel = peer.into();
    let chat_full = export_dc_invoke(
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::channels::GetFullChannel {
                channel: input_channel,
            },
        },
        warnings,
        fallback_used,
    )
    .await?;

    let tl::enums::messages::ChatFull::Full(chat_full) = chat_full;
    if let tl::enums::ChatFull::ChannelFull(full) = chat_full.full_chat {
        if let Some(migrated_from_chat_id) = full.migrated_from_chat_id {
            warnings.push(format!(
                "Supergroup migrated_from_chat_id {migrated_from_chat_id} detected; migrated history import is deferred to avoid source item id collisions."
            ));
        }
    }

    Ok(())
}

async fn takeout_history_count_probe(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    telegram_source_kind: &str,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<TakeoutHistoryProbe> {
    let response = takeout_get_history(
        client,
        alias,
        takeout_id,
        input_peer.clone(),
        range.clone(),
        0,
        0,
        1,
        warnings,
        fallback_used,
    )
    .await;

    let response = match response {
        Ok(response) => response,
        Err(error)
            if supports_only_my_messages_fallback(telegram_source_kind)
                && is_channel_private_error(&error) =>
        {
            push_warning_once(
                warnings,
                "Channel history is private; falling back to messages.search(from_id=self).",
            );
            let search_response = takeout_search_my_messages(
                client,
                alias,
                takeout_id,
                input_peer,
                range,
                0,
                0,
                1,
                warnings,
                fallback_used,
            )
            .await?;
            return Ok(TakeoutHistoryProbe {
                count: messages_response_count(search_response)?,
                only_my_messages: true,
            });
        }
        Err(error) => return Err(error),
    };

    Ok(TakeoutHistoryProbe {
        count: messages_response_count(response)?,
        only_my_messages: false,
    })
}

async fn import_takeout_history_ranges(
    handle: &AppHandle,
    job_id: &str,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    ranges: Vec<CountedMessageRange>,
    source: &crate::sources::SourceSyncTarget,
    total: i64,
    telegram_source_kind: &str,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<TakeoutHistoryImport> {
    let mut imported = TakeoutHistoryImport {
        inserted: 0,
        skipped: 0,
        max_message_id: source.last_sync_state.unwrap_or(0),
    };

    for counted_range in ranges {
        imported = import_takeout_history_pages(
            handle,
            job_id,
            client,
            alias,
            takeout_id,
            input_peer.clone(),
            counted_range,
            source,
            total,
            telegram_source_kind,
            imported,
            warnings,
            fallback_used,
        )
        .await?;
    }

    Ok(imported)
}

async fn import_takeout_history_pages(
    handle: &AppHandle,
    job_id: &str,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    counted_range: CountedMessageRange,
    source: &crate::sources::SourceSyncTarget,
    total: i64,
    telegram_source_kind: &str,
    mut imported: TakeoutHistoryImport,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<TakeoutHistoryImport> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let pool = get_pool(handle).await?;
    let range = counted_range.range;
    let split_count = counted_range.count;
    let mut only_my_messages = counted_range.only_my_messages;
    let mut profile = TakeoutPaginationProfile::TDesktop;
    let mut cursor = TakeoutPaginationCursor::new(profile, &range);
    let mut page_index = 0_usize;

    loop {
        if takeout_state.is_cancel_requested(job_id).await {
            return Err(AppError::validation("Takeout import cancelled"));
        }

        let request = takeout_page_request(cursor);
        let response = takeout_history_page_response(
            client,
            alias,
            takeout_id,
            input_peer.clone(),
            range.clone(),
            request,
            telegram_source_kind,
            &mut only_my_messages,
            warnings,
            fallback_used,
        )
        .await?;
        let page = parse_takeout_page(response, profile)?;
        let advance = next_takeout_cursor(cursor, &page, &range);

        if let Some(reason) = should_restart_with_descending_fallback(
            profile,
            split_count,
            page_index,
            &page,
            advance,
        ) {
            push_warning_once(
                warnings,
                takeout_pagination_fallback_warning(reason, &range),
            );
            update_and_emit(handle, &takeout_state, job_id, |job| {
                job.warnings = warnings.clone();
            })
            .await;
            profile = TakeoutPaginationProfile::DescendingFallback;
            cursor = TakeoutPaginationCursor::new(profile, &range);
            page_index = 0;
            continue;
        }

        if page.messages.is_empty() {
            break;
        }

        for message in page.messages {
            let message_id = message.id;
            if message_id <= message_range_min_id(&range) {
                continue;
            }
            imported.max_message_id = imported.max_message_id.max(i64::from(message_id));
            match raw_parse::parse_raw_message(&source.title, message) {
                Ok(Some(item)) => {
                    if insert_source_item(&pool, source.id, item).await? {
                        imported.inserted += 1;
                    } else {
                        imported.skipped += 1;
                    }
                }
                Ok(None) => imported.skipped += 1,
                Err(error) => return Err(AppError::internal(error)),
            }
        }

        update_and_emit(handle, &takeout_state, job_id, |job| {
            job.inserted = imported.inserted;
            job.skipped = imported.skipped;
            job.progress_current = Some((imported.inserted + imported.skipped).min(total));
            job.progress_total = Some(total);
            job.warnings = warnings.clone();
        })
        .await;

        if takeout_state.is_cancel_requested(job_id).await {
            return Err(AppError::validation("Takeout import cancelled"));
        }

        if page.is_terminal_response || !advance.advanced || advance.reached_range_start {
            break;
        }
        cursor = advance.cursor;
        page_index += 1;
    }

    Ok(imported)
}

async fn takeout_history_page_response(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    request: TakeoutPageRequest,
    telegram_source_kind: &str,
    only_my_messages: &mut bool,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<tl::enums::messages::Messages> {
    if *only_my_messages {
        return takeout_search_my_messages(
            client,
            alias,
            takeout_id,
            input_peer,
            range,
            request.offset_id,
            request.add_offset,
            request.limit,
            warnings,
            fallback_used,
        )
        .await;
    }

    match takeout_get_history(
        client,
        alias,
        takeout_id,
        input_peer.clone(),
        range.clone(),
        request.offset_id,
        request.add_offset,
        request.limit,
        warnings,
        fallback_used,
    )
    .await
    {
        Ok(response) => Ok(response),
        Err(error)
            if supports_only_my_messages_fallback(telegram_source_kind)
                && is_channel_private_error(&error) =>
        {
            push_warning_once(
                warnings,
                "Channel history is private; falling back to messages.search(from_id=self).",
            );
            *only_my_messages = true;
            takeout_search_my_messages(
                client,
                alias,
                takeout_id,
                input_peer,
                range,
                request.offset_id,
                request.add_offset,
                request.limit,
                warnings,
                fallback_used,
            )
            .await
        }
        Err(error) => Err(error),
    }
}

fn push_warning_once(warnings: &mut Vec<String>, warning: impl Into<String>) {
    let warning = warning.into();
    if !warnings.iter().any(|existing| existing == &warning) {
        warnings.push(warning);
    }
}

async fn takeout_get_history(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    offset_id: i32,
    add_offset: i32,
    limit: i32,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<tl::enums::messages::Messages> {
    export_dc_invoke(
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::InvokeWithMessagesRange {
                range,
                query: tl::functions::messages::GetHistory {
                    peer: input_peer,
                    offset_id,
                    offset_date: 0,
                    add_offset,
                    limit,
                    max_id: 0,
                    min_id: 0,
                    hash: 0,
                },
            },
        },
        warnings,
        fallback_used,
    )
    .await
}

async fn takeout_search_my_messages(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    offset_id: i32,
    add_offset: i32,
    limit: i32,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<tl::enums::messages::Messages> {
    export_dc_invoke(
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::InvokeWithMessagesRange {
                range,
                query: tl::functions::messages::Search {
                    peer: input_peer,
                    q: String::new(),
                    from_id: Some(tl::enums::InputPeer::PeerSelf),
                    saved_peer_id: None,
                    saved_reaction: None,
                    top_msg_id: None,
                    filter: tl::enums::MessagesFilter::InputMessagesFilterEmpty,
                    min_date: 0,
                    max_date: 0,
                    offset_id,
                    add_offset,
                    limit,
                    max_id: 0,
                    min_id: 0,
                    hash: 0,
                },
            },
        },
        warnings,
        fallback_used,
    )
    .await
}

fn supports_only_my_messages_fallback(telegram_source_kind: &str) -> bool {
    matches!(
        telegram_source_kind,
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP
    )
}

fn is_channel_private_error(error: &AppError) -> bool {
    error
        .message
        .to_ascii_uppercase()
        .contains("CHANNEL_PRIVATE")
}

fn messages_response_count(response: tl::enums::messages::Messages) -> AppResult<i64> {
    match response {
        tl::enums::messages::Messages::Messages(messages) => Ok(messages.messages.len() as i64),
        tl::enums::messages::Messages::Slice(messages) => Ok(i64::from(messages.count)),
        tl::enums::messages::Messages::ChannelMessages(messages) => Ok(i64::from(messages.count)),
        tl::enums::messages::Messages::NotModified(_) => Err(AppError::network(
            "Telegram returned messagesNotModified for Takeout history count probe",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_channel_private_error, supports_only_my_messages_fallback, TELEGRAM_KIND_CHANNEL,
        TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
    };
    use crate::error::AppError;

    #[test]
    fn only_my_messages_fallback_is_limited_to_channels() {
        assert!(supports_only_my_messages_fallback(TELEGRAM_KIND_CHANNEL));
        assert!(supports_only_my_messages_fallback(TELEGRAM_KIND_SUPERGROUP));
        assert!(!supports_only_my_messages_fallback(TELEGRAM_KIND_GROUP));
    }

    #[test]
    fn channel_private_detection_reads_rpc_name_from_error_message() {
        assert!(is_channel_private_error(&AppError::network(
            "Rpc error 400: CHANNEL_PRIVATE"
        )));
        assert!(!is_channel_private_error(&AppError::network(
            "Rpc error 400: TAKEOUT_INVALID"
        )));
    }
}
