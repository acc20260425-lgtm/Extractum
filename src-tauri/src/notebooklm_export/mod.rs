mod chunker;
mod filename;
mod glossary;
mod links;
mod media;
mod message_mapping;
mod model;
mod query;
mod renderer;

use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use time::{format_description, OffsetDateTime, UtcOffset};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{require_source_identity_ready, SourceIdentityRepairState};
use crate::time::now_secs;

use chunker::{build_chunks, should_export_message};
use filename::{ensure_child_path, ensure_child_relative_path, sanitize_path_component};
use glossary::{aggregate_participants, glossary_word_count, render_glossary};
use model::{
    ChunkFile, NotebookLmExportConfig, NotebookLmExportFile, NotebookLmExportMessage,
    NotebookLmExportRequest, NotebookLmExportResult, NotebookLmExportScope, ParticipantSummary,
    DEFAULT_MAX_BYTES_PER_FILE, DEFAULT_MAX_WORDS_PER_FILE, DEFAULT_MIN_MESSAGE_LENGTH,
};
use query::{
    load_export_messages, load_export_source, load_export_source_group, ExportHistoryScope,
    NotebookLmExportSourceGroup,
};
use renderer::{
    approx_word_count, render_document, render_document_overhead, render_message_block,
    DocumentRenderContext,
};

const EXPORT_MARKER_FILE: &str = ".extractum-notebooklm-export.json";
const NOTEBOOKLM_EXPORT_EVENT: &str = "notebooklm://export";
const MIGRATED_HISTORY_EMPTY_WARNING: &str =
    "Migrated small-group history was included, but no migrated messages matched the export range.";

struct ExportSection {
    heading: Option<&'static str>,
    filename_prefix: Option<&'static str>,
    empty_warning: Option<&'static str>,
    messages: Vec<NotebookLmExportMessage>,
}

struct RenderedExportSection {
    heading: Option<&'static str>,
    participants: Vec<ParticipantSummary>,
    chunks: Vec<ChunkFile>,
}

struct SourceExportInput {
    source: model::NotebookLmExportSource,
    current_messages: Vec<NotebookLmExportMessage>,
    migrated_messages: Vec<NotebookLmExportMessage>,
}

struct RenderedSourceExport {
    source: model::NotebookLmExportSource,
    rendered_sections: Vec<RenderedExportSection>,
    exported_messages: Vec<NotebookLmExportMessage>,
    skipped_message_count: usize,
    warnings: Vec<String>,
}

struct RenderedGroupMemberExport {
    member_index: usize,
    rendered: RenderedSourceExport,
    skipped_reason: Option<String>,
}

struct RenderedSourceGroupExport {
    rendered_members: Vec<RenderedGroupMemberExport>,
    exported_messages: Vec<NotebookLmExportMessage>,
    skipped_message_count: usize,
    warnings: Vec<String>,
}

#[derive(Clone)]
struct NotebookLmExportProgress {
    handle: AppHandle,
    export_id: String,
    source_id: i64,
}

impl NotebookLmExportProgress {
    fn new(handle: AppHandle, export_id: String, source_id: i64) -> Self {
        Self {
            handle,
            export_id,
            source_id,
        }
    }

    fn emit_started(
        &self,
        phase: &str,
        message: &str,
        progress_current: Option<usize>,
        progress_total: Option<usize>,
    ) {
        self.emit(NotebookLmExportEvent {
            export_id: self.export_id.clone(),
            source_id: self.source_id,
            kind: "started".to_string(),
            phase: phase.to_string(),
            message: Some(message.to_string()),
            progress_current,
            progress_total,
            file_path: None,
            error: None,
        });
    }

    fn emit_progress(
        &self,
        phase: &str,
        message: &str,
        progress_current: Option<usize>,
        progress_total: Option<usize>,
        file_path: Option<&str>,
    ) {
        self.emit(NotebookLmExportEvent {
            export_id: self.export_id.clone(),
            source_id: self.source_id,
            kind: "progress".to_string(),
            phase: phase.to_string(),
            message: Some(message.to_string()),
            progress_current,
            progress_total,
            file_path: file_path.map(str::to_string),
            error: None,
        });
    }

    fn emit_completed(
        &self,
        phase: &str,
        message: &str,
        progress_current: Option<usize>,
        progress_total: Option<usize>,
    ) {
        self.emit(NotebookLmExportEvent {
            export_id: self.export_id.clone(),
            source_id: self.source_id,
            kind: "completed".to_string(),
            phase: phase.to_string(),
            message: Some(message.to_string()),
            progress_current,
            progress_total,
            file_path: None,
            error: None,
        });
    }

    fn emit_failed(&self, phase: &str, error: &AppError) {
        self.emit(NotebookLmExportEvent {
            export_id: self.export_id.clone(),
            source_id: self.source_id,
            kind: "failed".to_string(),
            phase: phase.to_string(),
            message: None,
            progress_current: None,
            progress_total: None,
            file_path: None,
            error: Some(error.to_string()),
        });
    }

    fn emit(&self, event: NotebookLmExportEvent) {
        let _ = self.handle.emit(NOTEBOOKLM_EXPORT_EVENT, event);
    }
}

#[derive(Clone, Serialize)]
struct NotebookLmExportEvent {
    export_id: String,
    source_id: i64,
    kind: String,
    phase: String,
    message: Option<String>,
    progress_current: Option<usize>,
    progress_total: Option<usize>,
    file_path: Option<String>,
    error: Option<String>,
}

#[tauri::command]
pub async fn export_source_to_notebooklm(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    request: NotebookLmExportRequest,
) -> AppResult<NotebookLmExportResult> {
    require_source_identity_ready(repair_state.inner()).await?;
    let config = validate_request(request)?;
    let generated_at = now_secs();
    let progress = NotebookLmExportProgress::new(
        handle.clone(),
        config
            .export_id
            .clone()
            .unwrap_or_else(|| format!("notebooklm-{}-{generated_at}", config.event_scope_id())),
        config.event_scope_id(),
    );

    match config.scope.clone() {
        NotebookLmExportScope::Source { source_id } => {
            export_single_source_to_notebooklm(handle, progress, config, generated_at, source_id)
                .await
        }
        NotebookLmExportScope::SourceGroup { source_group_id } => {
            export_source_group_to_notebooklm(
                handle,
                progress,
                config,
                generated_at,
                source_group_id,
            )
            .await
        }
    }
}

async fn export_single_source_to_notebooklm(
    handle: AppHandle,
    progress: NotebookLmExportProgress,
    config: NotebookLmExportConfig,
    generated_at: i64,
    source_id: i64,
) -> AppResult<NotebookLmExportResult> {
    progress.emit_started("loading", "Loading source and synced messages.", None, None);

    let pool = match get_pool(&handle).await {
        Ok(pool) => pool,
        Err(error) => {
            progress.emit_failed("loading", &error);
            return Err(error);
        }
    };
    let repair_state = handle.state::<SourceIdentityRepairState>();
    if let Err(error) = require_source_identity_ready(repair_state.inner()).await {
        progress.emit_failed("loading", &error);
        return Err(error);
    }
    let source = match load_export_source(&pool, source_id).await {
        Ok(source) => source,
        Err(error) => {
            progress.emit_failed("loading", &error);
            return Err(error);
        }
    };
    let current_messages = match load_export_messages(
        &pool,
        source_id,
        config.period_from,
        config.period_to,
        ExportHistoryScope::Current,
    )
    .await
    {
        Ok(messages) => messages,
        Err(error) => {
            progress.emit_failed("loading", &error);
            return Err(error);
        }
    };
    let migrated_messages = if config.include_migrated_history {
        match load_export_messages(
            &pool,
            source_id,
            config.period_from,
            config.period_to,
            ExportHistoryScope::Migrated,
        )
        .await
        {
            Ok(messages) => messages,
            Err(error) => {
                progress.emit_failed("loading", &error);
                return Err(error);
            }
        }
    } else {
        Vec::new()
    };
    let message_count = current_messages.len() + migrated_messages.len();

    progress.emit_progress(
        "filtering",
        "Filtering and rendering message blocks.",
        Some(0),
        Some(message_count),
        None,
    );

    let task_progress = progress.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        task_progress.emit_progress(
            "chunking",
            "Grouping messages into NotebookLM-sized Markdown files.",
            None,
            None,
            None,
        );

        let rendered = render_source_export(
            SourceExportInput {
                source,
                current_messages,
                migrated_messages,
            },
            &config,
            generated_at,
            |filename| filename.to_string(),
            |current, total| {
                task_progress.emit_progress(
                    "filtering",
                    "Filtering and rendering message blocks.",
                    Some(current),
                    Some(total),
                    None,
                );
            },
        );
        let RenderedSourceExport {
            source,
            rendered_sections,
            exported_messages,
            skipped_message_count,
            warnings,
        } = rendered;

        task_progress.emit_progress(
            "preparing_output",
            "Preparing the export folder.",
            None,
            None,
            None,
        );
        let output_root = prepare_output_root(&config, &source, generated_at)?;
        let mut generated_file_names = vec!["glossary.md".to_string()];
        let participants = aggregate_participants(&exported_messages);
        let glossary_markdown = render_glossary(
            generated_at,
            source.title.as_deref().unwrap_or(&source.external_id),
            &participants,
        );
        let write_total = rendered_sections
            .iter()
            .map(|section| section.chunks.len())
            .sum::<usize>()
            + 1;
        task_progress.emit_progress(
            "writing",
            "Writing glossary.md.",
            Some(0),
            Some(write_total),
            Some("glossary.md"),
        );
        let glossary_path = write_export_file(&output_root, "glossary.md", &glossary_markdown)?;
        task_progress.emit_progress(
            "writing",
            "Writing glossary.md.",
            Some(1),
            Some(write_total),
            Some("glossary.md"),
        );

        let mut files = Vec::new();
        let mut written_count = 1;
        for section in rendered_sections {
            for chunk in section.chunks {
                generated_file_names.push(chunk.filename.clone());
                let context = DocumentRenderContext {
                    source: &source,
                    topic: &chunk.topic,
                    history_scope_heading: section.heading,
                    generated_at,
                    title_period: &chunk.title_period,
                    period_start: chunk.period_start,
                    period_end: chunk.period_end,
                    participants: &section.participants,
                    message_count: chunk.blocks.len(),
                    is_continuation: chunk.part_number > 1,
                };
                let markdown = render_document(&context, &chunk.blocks);
                task_progress.emit_progress(
                    "writing",
                    &format!("Writing {}.", chunk.filename),
                    Some(written_count),
                    Some(write_total),
                    Some(&chunk.filename),
                );
                let path = write_export_file(&output_root, &chunk.filename, &markdown)?;
                files.push(NotebookLmExportFile {
                    path: path_to_string(path),
                    message_count: chunk.blocks.len(),
                    byte_size: markdown.len(),
                    approximate_word_count: approx_word_count(&markdown),
                });
                written_count += 1;
                task_progress.emit_progress(
                    "writing",
                    &format!("Writing {}.", chunk.filename),
                    Some(written_count),
                    Some(write_total),
                    Some(&chunk.filename),
                );
            }
        }

        task_progress.emit_progress("manifest", "Writing export manifest.", None, None, None);
        write_marker(
            &output_root,
            &NotebookLmExportManifest {
                generated_at,
                scope: Some("source".to_string()),
                source_id: Some(source.id),
                source_external_id: Some(source.external_id.clone()),
                source_title: source.title.clone(),
                source_group_id: None,
                source_group_name: None,
                file_count: files.len(),
                exported_message_count: exported_messages.len(),
                skipped_message_count,
                warning_count: warnings.len(),
                warnings: warnings.clone(),
                generated_files: generated_file_names,
                members: Vec::new(),
            },
        )?;

        let glossary_file = NotebookLmExportFile {
            path: path_to_string(glossary_path.clone()),
            message_count: participants
                .iter()
                .map(|summary| summary.message_count)
                .sum(),
            byte_size: glossary_markdown.len(),
            approximate_word_count: glossary_word_count(&glossary_markdown),
        };

        Ok(NotebookLmExportResult {
            output_dir: path_to_string(output_root),
            files,
            glossary_file: Some(glossary_file.path),
            exported_message_count: exported_messages.len(),
            skipped_message_count,
            warning_count: warnings.len(),
            warnings,
        })
    })
    .await
    .map_err(|e| AppError::internal(format!("NotebookLM export task failed: {e}")))?;

    match result {
        Ok(result) => {
            progress.emit_completed(
                "completed",
                "NotebookLM export complete.",
                Some(result.files.len()),
                Some(result.files.len()),
            );
            Ok(result)
        }
        Err(error) => {
            progress.emit_failed("failed", &error);
            Err(error)
        }
    }
}

async fn load_group_export_inputs(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_group_id: i64,
    config: &NotebookLmExportConfig,
) -> AppResult<(
    NotebookLmExportSourceGroup,
    Vec<SourceExportInput>,
    Vec<String>,
)> {
    let group = load_export_source_group(pool, source_group_id).await?;
    if group.source_type != "telegram" {
        return Err(AppError::validation(
            "YouTube source-group NotebookLM export is not implemented yet.",
        ));
    }

    let mut warnings = Vec::new();
    let mut inputs = Vec::new();
    for member in &group.members {
        if member.source_type != "telegram" {
            warnings.push(format!(
                "Source {} was skipped because it is not a Telegram source.",
                member.source_id
            ));
            continue;
        }
        let source = load_export_source(pool, member.source_id).await?;
        let current_messages = load_export_messages(
            pool,
            member.source_id,
            config.period_from,
            config.period_to,
            ExportHistoryScope::Current,
        )
        .await?;
        let migrated_messages = if config.include_migrated_history {
            load_export_messages(
                pool,
                member.source_id,
                config.period_from,
                config.period_to,
                ExportHistoryScope::Migrated,
            )
            .await?
        } else {
            Vec::new()
        };
        inputs.push(SourceExportInput {
            source,
            current_messages,
            migrated_messages,
        });
    }

    if inputs.is_empty() {
        return Err(AppError::validation(
            "No Telegram sources found in this source group.",
        ));
    }

    Ok((group, inputs, warnings))
}

async fn export_source_group_to_notebooklm(
    handle: AppHandle,
    progress: NotebookLmExportProgress,
    config: NotebookLmExportConfig,
    generated_at: i64,
    source_group_id: i64,
) -> AppResult<NotebookLmExportResult> {
    progress.emit_started(
        "loading",
        "Loading source group and synced messages.",
        None,
        None,
    );
    let pool = match get_pool(&handle).await {
        Ok(pool) => pool,
        Err(error) => {
            progress.emit_failed("loading", &error);
            return Err(error);
        }
    };
    let (group, inputs, load_warnings) =
        match load_group_export_inputs(&pool, source_group_id, &config).await {
            Ok(value) => value,
            Err(error) => {
                progress.emit_failed("loading", &error);
                return Err(error);
            }
        };

    let result = tauri::async_runtime::spawn_blocking(move || {
        let rendered = render_source_group_export(inputs, &config, generated_at, load_warnings)?;
        write_group_export_package(
            &config,
            &group,
            generated_at,
            rendered.rendered_members,
            rendered.exported_messages,
            rendered.skipped_message_count,
            rendered.warnings,
        )
    })
    .await
    .map_err(|e| AppError::internal(format!("NotebookLM export task failed: {e}")))?;

    match result {
        Ok(result) => {
            progress.emit_completed(
                "completed",
                "NotebookLM export complete.",
                Some(result.files.len()),
                Some(result.files.len()),
            );
            Ok(result)
        }
        Err(error) => {
            progress.emit_failed("failed", &error);
            Err(error)
        }
    }
}

fn render_source_group_export(
    inputs: Vec<SourceExportInput>,
    config: &NotebookLmExportConfig,
    generated_at: i64,
    load_warnings: Vec<String>,
) -> AppResult<RenderedSourceGroupExport> {
    let mut warnings = load_warnings;
    let mut rendered_members = Vec::new();
    let mut exported_messages = Vec::new();
    let mut skipped_message_count = 0;

    for (index, input) in inputs.into_iter().enumerate() {
        let member_index = index + 1;
        let prefix_source = input.source.clone();
        let prefix = source_member_file_prefix(member_index, &prefix_source);
        let rendered = render_source_export(
            input,
            config,
            generated_at,
            |filename| prefix_chunk_filename(&prefix, filename),
            |_, _| {},
        );
        let source_label = rendered
            .source
            .title
            .as_deref()
            .unwrap_or(&rendered.source.external_id)
            .to_string();
        let mut member_warnings = rendered
            .warnings
            .iter()
            .map(|warning| format!("{source_label}: {warning}"))
            .collect::<Vec<_>>();
        let skipped_reason = if rendered.exported_messages.is_empty() {
            let reason =
                format!("{source_label}: no exportable messages matched the export settings.");
            member_warnings.push(reason.clone());
            Some(reason)
        } else {
            None
        };
        warnings.extend(member_warnings.clone());
        skipped_message_count += rendered.skipped_message_count;
        exported_messages.extend(rendered.exported_messages.iter().cloned());
        rendered_members.push(RenderedGroupMemberExport {
            member_index,
            rendered: RenderedSourceExport {
                warnings: member_warnings,
                ..rendered
            },
            skipped_reason,
        });
    }

    if exported_messages.is_empty() {
        return Err(AppError::validation(
            "No exportable Telegram messages found for this source group.",
        ));
    }

    Ok(RenderedSourceGroupExport {
        rendered_members,
        exported_messages,
        skipped_message_count,
        warnings,
    })
}

fn validate_request(request: NotebookLmExportRequest) -> AppResult<NotebookLmExportConfig> {
    let output_dir = request.output_dir.trim();
    if output_dir.is_empty() {
        return Err(AppError::validation("Output directory is required"));
    }
    if let (Some(from), Some(to)) = (request.period_from, request.period_to) {
        if from > to {
            return Err(AppError::validation(
                "Export period start must be before export period end",
            ));
        }
    }

    let scope = match (request.source_id, request.source_group_id) {
        (Some(source_id), None) => NotebookLmExportScope::Source { source_id },
        (None, Some(source_group_id)) => NotebookLmExportScope::SourceGroup { source_group_id },
        (None, None) => {
            return Err(AppError::validation(
                "Select a source or source group before exporting",
            ));
        }
        (Some(_), Some(_)) => {
            return Err(AppError::validation(
                "Select either a source or source group, not both",
            ));
        }
    };

    Ok(NotebookLmExportConfig {
        export_id: request
            .export_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        scope,
        output_dir: output_dir.to_string(),
        period_from: request.period_from,
        period_to: request.period_to,
        include_media_placeholders: request.include_media_placeholders,
        include_migrated_history: request.include_migrated_history,
        min_message_length: validate_positive_usize(
            request.min_message_length,
            "min_message_length",
            DEFAULT_MIN_MESSAGE_LENGTH,
        )?,
        max_words_per_file: validate_positive_usize(
            request.max_words_per_file,
            "max_words_per_file",
            DEFAULT_MAX_WORDS_PER_FILE,
        )?,
        max_bytes_per_file: validate_positive_usize(
            request.max_bytes_per_file,
            "max_bytes_per_file",
            DEFAULT_MAX_BYTES_PER_FILE,
        )?,
        overwrite_existing: request.overwrite_existing,
    })
}

fn validate_positive_usize(value: i64, label: &str, default_value: usize) -> AppResult<usize> {
    let value = if value == 0 {
        default_value as i64
    } else {
        value
    };
    if value < 0 {
        return Err(AppError::validation(format!("{label} must be positive")));
    }
    usize::try_from(value).map_err(|_| AppError::validation(format!("{label} is too large")))
}

fn progress_step(total: usize) -> usize {
    (total / 100).max(1)
}

fn should_emit_progress(current: usize, total: usize, step: usize) -> bool {
    current == total || current.is_multiple_of(step)
}

fn source_member_file_prefix(
    member_index: usize,
    source: &model::NotebookLmExportSource,
) -> String {
    let fallback = format!("source_{}", source.id);
    let slug = sanitize_path_component(
        source.title.as_deref().unwrap_or(&source.external_id),
        &fallback,
    );
    format!("{member_index:03}-source-{}-{slug}", source.id)
}

fn prefix_chunk_filename(prefix: &str, filename: &str) -> String {
    format!("sources/{prefix}-{filename}")
}

fn render_source_export(
    input: SourceExportInput,
    config: &NotebookLmExportConfig,
    generated_at: i64,
    filename_mapper: impl Fn(&str) -> String,
    mut on_filter_progress: impl FnMut(usize, usize),
) -> RenderedSourceExport {
    let SourceExportInput {
        source,
        current_messages,
        migrated_messages,
    } = input;
    let mut warnings = Vec::new();
    let mut skipped_message_count = 0;
    let filter_total = current_messages.len() + migrated_messages.len();
    let filter_step = progress_step(filter_total);
    let sections = if config.include_migrated_history {
        vec![
            ExportSection {
                heading: Some("Current supergroup history"),
                filename_prefix: Some("current-supergroup-history"),
                empty_warning: None,
                messages: current_messages,
            },
            ExportSection {
                heading: Some("Migrated small-group history"),
                filename_prefix: Some("migrated-small-group-history"),
                empty_warning: Some(MIGRATED_HISTORY_EMPTY_WARNING),
                messages: migrated_messages,
            },
        ]
    } else {
        vec![ExportSection {
            heading: None,
            filename_prefix: None,
            empty_warning: None,
            messages: current_messages,
        }]
    };

    let mut rendered_sections = Vec::new();
    let mut exported_messages = Vec::new();
    let mut filter_current = 0;

    for section in sections {
        if section.messages.is_empty() {
            if let Some(warning) = section.empty_warning {
                warnings.push(warning.to_string());
            }
        }

        let mut blocks = Vec::new();
        for message in &section.messages {
            if should_export_message(
                message,
                config.min_message_length,
                config.include_media_placeholders,
            ) {
                let mut message = message.clone();
                if !config.include_media_placeholders {
                    message.media_placeholders.clear();
                }
                blocks.push(render_message_block(&message));
            } else {
                skipped_message_count += 1;
            }

            filter_current += 1;
            if should_emit_progress(filter_current, filter_total, filter_step) {
                on_filter_progress(filter_current, filter_total);
            }
        }

        let section_messages = blocks
            .iter()
            .map(|block| block.message.clone())
            .collect::<Vec<_>>();
        exported_messages.extend(section_messages.iter().cloned());
        let participants = aggregate_participants(&section_messages);
        let (mut chunks, chunk_warnings) = build_chunks(
            &source,
            &blocks,
            config.max_words_per_file,
            config.max_bytes_per_file,
            |topic, title_period, period_start, period_end, is_continuation, message_count| {
                let context = DocumentRenderContext {
                    source: &source,
                    topic,
                    history_scope_heading: section.heading,
                    generated_at,
                    title_period,
                    period_start,
                    period_end,
                    participants: &participants,
                    message_count,
                    is_continuation,
                };
                render_document_overhead(&context)
            },
        );
        if let Some(filename_prefix) = section.filename_prefix {
            for chunk in &mut chunks {
                chunk.filename = format!("{filename_prefix}-{}", chunk.filename);
            }
        }
        for chunk in &mut chunks {
            chunk.filename = filename_mapper(&chunk.filename);
        }
        warnings.extend(chunk_warnings);
        rendered_sections.push(RenderedExportSection {
            heading: section.heading,
            participants,
            chunks,
        });
    }

    RenderedSourceExport {
        source,
        rendered_sections,
        exported_messages,
        skipped_message_count,
        warnings,
    }
}

fn write_group_export_package(
    config: &NotebookLmExportConfig,
    group: &NotebookLmExportSourceGroup,
    generated_at: i64,
    mut rendered_members: Vec<RenderedGroupMemberExport>,
    exported_messages: Vec<NotebookLmExportMessage>,
    skipped_message_count: usize,
    warnings: Vec<String>,
) -> AppResult<NotebookLmExportResult> {
    rendered_members.sort_by_key(|member| member.member_index);
    let output_root = prepare_output_root_for_label(config, &group.name, generated_at)?;
    let mut generated_file_names = vec!["glossary.md".to_string()];
    let participants = aggregate_participants(&exported_messages);
    let glossary_markdown = render_glossary(generated_at, &group.name, &participants);
    let glossary_path = write_export_file(&output_root, "glossary.md", &glossary_markdown)?;

    let mut files = Vec::new();
    for member in &rendered_members {
        for section in &member.rendered.rendered_sections {
            for chunk in &section.chunks {
                generated_file_names.push(chunk.filename.clone());
                let context = DocumentRenderContext {
                    source: &member.rendered.source,
                    topic: &chunk.topic,
                    history_scope_heading: section.heading,
                    generated_at,
                    title_period: &chunk.title_period,
                    period_start: chunk.period_start,
                    period_end: chunk.period_end,
                    participants: &section.participants,
                    message_count: chunk.blocks.len(),
                    is_continuation: chunk.part_number > 1,
                };
                let markdown = render_document(&context, &chunk.blocks);
                let path = write_export_file(&output_root, &chunk.filename, &markdown)?;
                files.push(NotebookLmExportFile {
                    path: path_to_string(path),
                    message_count: chunk.blocks.len(),
                    byte_size: markdown.len(),
                    approximate_word_count: approx_word_count(&markdown),
                });
            }
        }
    }

    let members = rendered_members
        .into_iter()
        .map(|member| NotebookLmExportManifestMember {
            source_id: member.rendered.source.id,
            source_title: member.rendered.source.title.clone(),
            source_subtype: Some(member.rendered.source.source_subtype.clone()),
            exported_message_count: member.rendered.exported_messages.len(),
            skipped_message_count: member.rendered.skipped_message_count,
            generated_files: member
                .rendered
                .rendered_sections
                .iter()
                .flat_map(|section| section.chunks.iter().map(|chunk| chunk.filename.clone()))
                .collect(),
            warnings: member.rendered.warnings.clone(),
            skipped_reason: member.skipped_reason,
        })
        .collect();

    write_marker(
        &output_root,
        &NotebookLmExportManifest {
            generated_at,
            scope: Some("source_group".to_string()),
            source_id: None,
            source_external_id: None,
            source_title: None,
            source_group_id: Some(group.id),
            source_group_name: Some(group.name.clone()),
            file_count: files.len(),
            exported_message_count: exported_messages.len(),
            skipped_message_count,
            warning_count: warnings.len(),
            warnings: warnings.clone(),
            generated_files: generated_file_names,
            members,
        },
    )?;

    let glossary_file = NotebookLmExportFile {
        path: path_to_string(glossary_path),
        message_count: participants
            .iter()
            .map(|summary| summary.message_count)
            .sum(),
        byte_size: glossary_markdown.len(),
        approximate_word_count: glossary_word_count(&glossary_markdown),
    };

    Ok(NotebookLmExportResult {
        output_dir: path_to_string(output_root),
        files,
        glossary_file: Some(glossary_file.path),
        exported_message_count: exported_messages.len(),
        skipped_message_count,
        warning_count: warnings.len(),
        warnings,
    })
}

fn prepare_output_root(
    config: &NotebookLmExportConfig,
    source: &model::NotebookLmExportSource,
    generated_at: i64,
) -> AppResult<PathBuf> {
    prepare_output_root_for_label_with_fallback(
        config,
        source.title.as_deref().unwrap_or(&source.external_id),
        "source",
        generated_at,
    )
}

fn prepare_output_root_for_label(
    config: &NotebookLmExportConfig,
    label: &str,
    generated_at: i64,
) -> AppResult<PathBuf> {
    prepare_output_root_for_label_with_fallback(config, label, "source_group", generated_at)
}

fn prepare_output_root_for_label_with_fallback(
    config: &NotebookLmExportConfig,
    label: &str,
    fallback_slug: &str,
    generated_at: i64,
) -> AppResult<PathBuf> {
    let base = PathBuf::from(&config.output_dir);
    if base.exists() && !base.is_dir() {
        return Err(AppError::validation("Output path is not a directory"));
    }
    fs::create_dir_all(&base).map_err(map_create_dir_error)?;
    let base = base
        .canonicalize()
        .map_err(|e| AppError::validation(format!("Could not resolve output directory: {e}")))?;

    let source_slug = sanitize_path_component(label, fallback_slug);
    let folder = if config.overwrite_existing {
        format!("notebooklm_export_{source_slug}")
    } else {
        format!(
            "notebooklm_export_{source_slug}_{}",
            timestamp_for_folder(generated_at)
        )
    };
    let output_root = ensure_child_path(&base, &folder)
        .ok_or_else(|| AppError::validation("Generated export folder name is invalid"))?;

    if output_root.exists() {
        if !config.overwrite_existing {
            return Err(AppError::conflict(
                "Timestamped NotebookLM export folder already exists",
            ));
        }
        let output_root = validate_existing_export_root(&base, &output_root)?;
        let marker = output_root.join(EXPORT_MARKER_FILE);
        if !marker.exists() {
            return Err(AppError::conflict(
                "Deterministic export folder exists without Extractum export marker",
            ));
        }
        remove_generated_files(&output_root)?;
        return Ok(output_root);
    } else {
        fs::create_dir(&output_root).map_err(map_create_dir_error)?;
    }

    let output_root = output_root
        .canonicalize()
        .map_err(|e| AppError::validation(format!("Could not resolve export folder: {e}")))?;
    if !output_root.starts_with(&base) {
        return Err(AppError::conflict(
            "Export folder resolves outside the selected output directory",
        ));
    }

    Ok(output_root)
}

fn remove_generated_files(output_root: &Path) -> AppResult<()> {
    let manifest = read_manifest(output_root)?;
    if manifest.generated_files.is_empty() {
        return Err(AppError::conflict(
            "Existing export manifest does not list generated files",
        ));
    }

    for file_name in manifest.generated_files {
        let path = ensure_child_relative_path(output_root, &file_name).ok_or_else(|| {
            AppError::conflict("Existing export manifest contains an invalid file path")
        })?;
        validate_generated_path_for_io(output_root, &path)?;
        if !path.exists() {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| AppError::conflict(format!("Could not inspect export file: {e}")))?;
        if !metadata.is_file() {
            return Err(AppError::conflict(
                "Existing export manifest references a non-file path",
            ));
        }
        fs::remove_file(&path)
            .map_err(|e| AppError::conflict(format!("Could not replace export file: {e}")))?;
    }
    Ok(())
}

fn validate_existing_export_root(base: &Path, output_root: &Path) -> AppResult<PathBuf> {
    let metadata = fs::symlink_metadata(output_root)
        .map_err(|e| AppError::validation(format!("Could not inspect export folder: {e}")))?;
    if metadata_is_link_or_reparse(&metadata) {
        return Err(AppError::conflict(
            "Export folder cannot be a symbolic link or reparse point",
        ));
    }
    if !metadata.is_dir() {
        return Err(AppError::validation("Export path is not a directory"));
    }

    let output_root = output_root
        .canonicalize()
        .map_err(|e| AppError::validation(format!("Could not resolve export folder: {e}")))?;
    if !output_root.starts_with(base) {
        return Err(AppError::conflict(
            "Export folder resolves outside the selected output directory",
        ));
    }

    Ok(output_root)
}

fn write_export_file(output_root: &Path, filename: &str, content: &str) -> AppResult<PathBuf> {
    let path = ensure_child_relative_path(output_root, filename).ok_or_else(|| {
        AppError::validation(format!("Generated filename '{filename}' is invalid"))
    })?;
    validate_generated_path_for_io(output_root, &path)?;
    if let Some(parent) = path.parent() {
        if parent != output_root {
            fs::create_dir_all(parent).map_err(map_create_dir_error)?;
        }
    }
    validate_generated_path_for_io(output_root, &path)?;
    fs::write(&path, content)
        .map_err(|e| AppError::internal(format!("Could not write export file: {e}")))?;
    Ok(path)
}

fn validate_generated_path_for_io(output_root: &Path, path: &Path) -> AppResult<()> {
    let root_metadata = fs::symlink_metadata(output_root)
        .map_err(|e| AppError::conflict(format!("Could not inspect export folder: {e}")))?;
    if metadata_is_link_or_reparse(&root_metadata) {
        return Err(AppError::conflict(
            "Generated export path contains a symbolic link or reparse point",
        ));
    }
    if !root_metadata.is_dir() {
        return Err(AppError::conflict("Export path is not a directory"));
    }

    let relative_path = path
        .strip_prefix(output_root)
        .map_err(|_| AppError::conflict("Generated export path is outside the export directory"))?;
    let components = relative_path.components().collect::<Vec<_>>();
    let mut current = output_root.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(value) = component else {
            return Err(AppError::conflict(
                "Generated export path contains an invalid component",
            ));
        };
        current.push(value);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata_is_link_or_reparse(&metadata) {
                    return Err(AppError::conflict(
                        "Generated export path contains a symbolic link or reparse point",
                    ));
                }
                if index + 1 < components.len() && !metadata.is_dir() {
                    return Err(AppError::conflict(
                        "Generated export path contains a non-directory ancestor",
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(AppError::conflict(format!(
                    "Could not inspect generated export path: {error}"
                )));
            }
        }
    }

    Ok(())
}

fn metadata_is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }

    #[cfg(not(windows))]
    {
        false
    }
}

fn write_marker(output_root: &Path, manifest: &NotebookLmExportManifest) -> AppResult<()> {
    let path = ensure_child_path(output_root, EXPORT_MARKER_FILE)
        .ok_or_else(|| AppError::validation("Export marker filename is invalid"))?;
    validate_generated_path_for_io(output_root, &path)?;
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| AppError::internal(format!("Could not serialize export manifest: {e}")))?;
    fs::write(path, json)
        .map_err(|e| AppError::internal(format!("Could not write export manifest: {e}")))
}

fn read_manifest(output_root: &Path) -> AppResult<NotebookLmExportManifest> {
    let path = ensure_child_path(output_root, EXPORT_MARKER_FILE)
        .ok_or_else(|| AppError::validation("Export marker filename is invalid"))?;
    validate_generated_path_for_io(output_root, &path)?;
    let json = fs::read_to_string(path)
        .map_err(|e| AppError::conflict(format!("Could not read export manifest: {e}")))?;
    serde_json::from_str(&json)
        .map_err(|e| AppError::conflict(format!("Could not parse export manifest: {e}")))
}

fn map_create_dir_error(error: std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::AlreadyExists => AppError::conflict(error.to_string()),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
            AppError::validation(error.to_string())
        }
        _ => AppError::internal(error.to_string()),
    }
}

fn timestamp_for_folder(unix: i64) -> String {
    let format = format_description::parse("[year][month][day]-[hour][minute][second]")
        .expect("timestamp format");
    OffsetDateTime::from_unix_timestamp(unix)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .to_offset(UtcOffset::UTC)
        .format(&format)
        .unwrap_or_else(|_| "19700101-000000".to_string())
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

#[derive(Deserialize, Serialize)]
struct NotebookLmExportManifest {
    generated_at: i64,
    #[serde(default = "default_manifest_scope")]
    scope: Option<String>,
    #[serde(default)]
    source_id: Option<i64>,
    #[serde(default)]
    source_external_id: Option<String>,
    #[serde(default)]
    source_title: Option<String>,
    #[serde(default)]
    source_group_id: Option<i64>,
    #[serde(default)]
    source_group_name: Option<String>,
    file_count: usize,
    exported_message_count: usize,
    #[serde(default)]
    skipped_message_count: usize,
    #[serde(default)]
    warning_count: usize,
    #[serde(default)]
    warnings: Vec<String>,
    generated_files: Vec<String>,
    #[serde(default)]
    members: Vec<NotebookLmExportManifestMember>,
}

#[derive(Deserialize, Serialize)]
struct NotebookLmExportManifestMember {
    source_id: i64,
    source_title: Option<String>,
    source_subtype: Option<String>,
    exported_message_count: usize,
    skipped_message_count: usize,
    generated_files: Vec<String>,
    warnings: Vec<String>,
    skipped_reason: Option<String>,
}

fn default_manifest_scope() -> Option<String> {
    Some("source".to_string())
}

#[cfg(test)]
mod tests {
    use super::query::NotebookLmExportSourceGroup;
    use super::{
        load_group_export_inputs, prefix_chunk_filename, read_manifest, remove_generated_files,
        render_source_export, render_source_group_export, source_member_file_prefix,
        timestamp_for_folder, validate_request, write_export_file, write_group_export_package,
        write_marker, NotebookLmExportManifest, NotebookLmExportManifestMember,
        RenderedGroupMemberExport, SourceExportInput, EXPORT_MARKER_FILE,
        MIGRATED_HISTORY_EMPTY_WARNING,
    };
    use crate::error::AppError;
    use crate::media::ItemMediaMetadata;
    use crate::notebooklm_export::model::{
        NotebookLmExportConfig, NotebookLmExportMessage, NotebookLmExportRequest,
        NotebookLmExportScope, NotebookLmExportSource,
    };
    use crate::sources::NOTEBOOKLM_HISTORY_SCOPE_CURRENT_SUPERGROUP;
    use std::io;

    fn request() -> NotebookLmExportRequest {
        NotebookLmExportRequest {
            export_id: None,
            source_id: Some(1),
            source_group_id: None,
            output_dir: ".".to_string(),
            period_from: None,
            period_to: None,
            include_media_placeholders: true,
            include_migrated_history: false,
            min_message_length: 3,
            max_words_per_file: 300_000,
            max_bytes_per_file: 50_000_000,
            overwrite_existing: false,
        }
    }

    fn test_source() -> NotebookLmExportSource {
        NotebookLmExportSource {
            id: 42,
            source_type: "telegram".to_string(),
            source_subtype: "channel".to_string(),
            external_id: "source-42".to_string(),
            title: Some("Alpha Source".to_string()),
        }
    }

    fn export_config() -> NotebookLmExportConfig {
        NotebookLmExportConfig {
            export_id: None,
            scope: NotebookLmExportScope::Source { source_id: 42 },
            output_dir: ".".to_string(),
            period_from: None,
            period_to: None,
            include_media_placeholders: true,
            include_migrated_history: false,
            min_message_length: 3,
            max_words_per_file: 300_000,
            max_bytes_per_file: 50_000_000,
            overwrite_existing: false,
        }
    }

    fn export_message(item_id: i64, text: &str) -> NotebookLmExportMessage {
        NotebookLmExportMessage {
            item_id,
            source_id: 42,
            external_id: format!("message-{item_id}"),
            author: Some("Ada".to_string()),
            published_at: 0,
            text: Some(text.to_string()),
            content_kind: "text_only".to_string(),
            has_media: false,
            media_kind: None,
            media_metadata: ItemMediaMetadata::default(),
            media_placeholders: Vec::new(),
            urls: Vec::new(),
            reply_to_msg_id: None,
            reply_to_author: None,
            reply_to_snippet: None,
            reply_to_peer_kind: None,
            reply_to_peer_id: None,
            reply_to_top_id: None,
            reaction_count: None,
            forum_topic_id: None,
            forum_topic_title: None,
            forum_topic_top_message_id: None,
            history_scope: NOTEBOOKLM_HISTORY_SCOPE_CURRENT_SUPERGROUP.to_string(),
            migration_domain: None,
        }
    }

    async fn source_group_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_type TEXT NOT NULL,
                source_subtype TEXT,
                external_id TEXT NOT NULL,
                title TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create sources");
        sqlx::query(
            r#"
            CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL DEFAULT 'telegram',
                created_at INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create analysis_source_groups");
        sqlx::query(
            r#"
            CREATE TABLE analysis_source_group_members (
                group_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (group_id, source_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create analysis_source_group_members");
        pool
    }

    #[test]
    fn source_member_file_prefix_includes_index_id_and_slug() {
        let source = crate::notebooklm_export::model::NotebookLmExportSource {
            id: 42,
            source_type: "telegram".to_string(),
            source_subtype: "channel".to_string(),
            external_id: "external".to_string(),
            title: Some("Alpha Source".to_string()),
        };

        assert_eq!(
            source_member_file_prefix(1, &source),
            "001-source-42-alpha_source"
        );
    }

    #[test]
    fn source_member_file_prefix_uses_fallback_slug_for_unsafe_title() {
        let source = crate::notebooklm_export::model::NotebookLmExportSource {
            id: 77,
            source_type: "telegram".to_string(),
            source_subtype: "channel".to_string(),
            external_id: "external".to_string(),
            title: Some("..".to_string()),
        };

        assert_eq!(
            source_member_file_prefix(2, &source),
            "002-source-77-source_77"
        );
    }

    #[test]
    fn prefix_chunk_filename_adds_sources_directory_and_prefix() {
        assert_eq!(
            prefix_chunk_filename("001-source-42-alpha", "part-001.md"),
            "sources/001-source-42-alpha-part-001.md"
        );
    }

    #[test]
    fn group_member_manifest_records_source_scoped_generated_files() {
        let member = NotebookLmExportManifestMember {
            source_id: 42,
            source_title: Some("Alpha".to_string()),
            source_subtype: Some("channel".to_string()),
            exported_message_count: 2,
            skipped_message_count: 1,
            generated_files: vec![
                "sources/001-source-42-alpha-1970_alpha_unrecognized_topic_part-001.md".to_string(),
            ],
            warnings: vec!["Alpha: skipped 1 short message.".to_string()],
            skipped_reason: None,
        };
        let manifest = NotebookLmExportManifest {
            generated_at: 1,
            scope: Some("source_group".to_string()),
            source_id: None,
            source_external_id: None,
            source_title: None,
            source_group_id: Some(9),
            source_group_name: Some("Notebook Group".to_string()),
            file_count: 1,
            exported_message_count: 2,
            skipped_message_count: 1,
            warning_count: 1,
            warnings: vec!["Alpha: skipped 1 short message.".to_string()],
            generated_files: vec![
                "glossary.md".to_string(),
                "sources/001-source-42-alpha-1970_alpha_unrecognized_topic_part-001.md".to_string(),
            ],
            members: vec![member],
        };

        let json = serde_json::to_string(&manifest).expect("serialize manifest");

        assert!(json.contains(r#""scope":"source_group""#));
        assert!(json.contains(r#""source_group_id":9"#));
        assert!(json.contains("sources/001-source-42-alpha"));
    }

    #[test]
    fn render_source_export_filters_messages_and_clears_media_placeholders() {
        let mut config = export_config();
        config.include_media_placeholders = false;
        config.min_message_length = 10;

        let mut exported = export_message(1, "this message is long enough to export");
        exported.content_kind = "text_with_media".to_string();
        exported.has_media = true;
        exported.media_kind = Some("photo".to_string());
        exported.media_placeholders = vec!["photo: image.jpg".to_string()];
        let skipped = export_message(2, "short");
        let mut progress = Vec::new();

        let rendered = render_source_export(
            SourceExportInput {
                source: test_source(),
                current_messages: vec![exported, skipped],
                migrated_messages: Vec::new(),
            },
            &config,
            0,
            str::to_string,
            |current, total| progress.push((current, total)),
        );

        assert_eq!(rendered.skipped_message_count, 1);
        assert_eq!(rendered.exported_messages.len(), 1);
        assert!(rendered.exported_messages[0].media_placeholders.is_empty());
        assert_eq!(rendered.rendered_sections.len(), 1);
        assert_eq!(
            rendered.rendered_sections[0].chunks[0].filename,
            "1970_alpha_source_unrecognized_topic_part-001.md"
        );
        assert_eq!(progress.last().copied(), Some((2, 2)));
    }

    #[test]
    fn render_source_export_tracks_empty_migrated_history_warning_and_section_prefix() {
        let mut config = export_config();
        config.include_migrated_history = true;

        let rendered = render_source_export(
            SourceExportInput {
                source: test_source(),
                current_messages: vec![export_message(1, "current history message")],
                migrated_messages: Vec::new(),
            },
            &config,
            0,
            str::to_string,
            |_, _| {},
        );

        assert_eq!(
            rendered.warnings,
            vec![MIGRATED_HISTORY_EMPTY_WARNING.to_string()]
        );
        assert_eq!(rendered.rendered_sections.len(), 2);
        assert_eq!(
            rendered.rendered_sections[0].heading,
            Some("Current supergroup history")
        );
        assert_eq!(
            rendered.rendered_sections[1].heading,
            Some("Migrated small-group history")
        );
        assert_eq!(
            rendered.rendered_sections[0].chunks[0].filename,
            "current-supergroup-history-1970_alpha_source_unrecognized_topic_part-001.md"
        );
    }

    #[test]
    fn write_group_export_package_records_group_manifest_and_source_files() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-notebooklm-group-package-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).expect("create temp");
        let mut config = export_config();
        config.output_dir = temp.to_string_lossy().into_owned();
        config.scope = NotebookLmExportScope::SourceGroup { source_group_id: 9 };

        let group = NotebookLmExportSourceGroup {
            id: 9,
            name: "Notebook Group".to_string(),
            source_type: "telegram".to_string(),
            members: Vec::new(),
        };
        let input = SourceExportInput {
            source: test_source(),
            current_messages: vec![export_message(1, "current history message")],
            migrated_messages: Vec::new(),
        };
        let rendered = render_source_export(
            input,
            &config,
            0,
            |filename| prefix_chunk_filename("001-source-42-alpha_source", filename),
            |_, _| {},
        );
        let exported_messages = rendered.exported_messages.clone();

        let result = write_group_export_package(
            &config,
            &group,
            0,
            vec![RenderedGroupMemberExport {
                member_index: 1,
                rendered,
                skipped_reason: None,
            }],
            exported_messages,
            0,
            Vec::new(),
        )
        .expect("write group package");

        assert_eq!(result.files.len(), 1);
        assert!(result.glossary_file.is_some());
        let manifest =
            read_manifest(std::path::Path::new(&result.output_dir)).expect("read group manifest");
        assert_eq!(manifest.scope.as_deref(), Some("source_group"));
        assert_eq!(manifest.source_group_id, Some(9));
        assert_eq!(
            manifest.source_group_name.as_deref(),
            Some("Notebook Group")
        );
        assert_eq!(manifest.members.len(), 1);
        assert_eq!(
            manifest.members[0].generated_files,
            vec!["sources/001-source-42-alpha_source-1970_alpha_source_unrecognized_topic_part-001.md"]
        );

        std::fs::remove_dir_all(&temp).expect("cleanup temp");
    }

    #[test]
    fn validates_exactly_one_export_scope() {
        let mut missing = request();
        missing.source_id = None;
        missing.source_group_id = None;
        let error = validate_request(missing).expect_err("missing scope is invalid");
        assert!(error.message.contains("Select a source or source group"));

        let mut both = request();
        both.source_id = Some(1);
        both.source_group_id = Some(9);
        let error = validate_request(both).expect_err("two scopes are invalid");
        assert!(error
            .message
            .contains("Select either a source or source group"));
    }

    #[test]
    fn validates_single_source_scope() {
        let config = validate_request(request()).expect("valid source request");
        assert_eq!(config.scope, NotebookLmExportScope::Source { source_id: 1 });
        assert_eq!(config.event_scope_id(), 1);
    }

    #[test]
    fn validates_source_group_scope() {
        let mut request = request();
        request.source_id = None;
        request.source_group_id = Some(9);

        let config = validate_request(request).expect("valid group request");

        assert_eq!(
            config.scope,
            NotebookLmExportScope::SourceGroup { source_group_id: 9 }
        );
        assert_eq!(config.event_scope_id(), 9);
    }

    #[test]
    fn group_export_returns_no_exportable_messages_copy_for_empty_rendered_members() {
        let error =
            AppError::validation("No exportable Telegram messages found for this source group.");
        assert!(error
            .message
            .contains("No exportable Telegram messages found for this source group."));
    }

    #[test]
    fn render_source_group_export_errors_when_all_members_empty_after_filters() {
        let mut config = export_config();
        config.min_message_length = 100;

        let error = match render_source_group_export(
            vec![SourceExportInput {
                source: test_source(),
                current_messages: vec![export_message(1, "short")],
                migrated_messages: Vec::new(),
            }],
            &config,
            0,
            Vec::new(),
        ) {
            Ok(_) => panic!("all-empty rendered group should fail"),
            Err(error) => error,
        };

        assert!(error
            .message
            .contains("No exportable Telegram messages found for this source group."));
    }

    #[test]
    fn group_export_returns_no_telegram_sources_copy_for_empty_valid_members() {
        let error = AppError::validation("No Telegram sources found in this source group.");
        assert!(error
            .message
            .contains("No Telegram sources found in this source group."));
    }

    #[tokio::test]
    async fn load_group_export_inputs_rejects_youtube_group_for_hard_validation() {
        let pool = source_group_pool().await;
        sqlx::query(
            "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
             VALUES (9, 'YouTube Group', 'youtube', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert group");

        let error = match load_group_export_inputs(&pool, 9, &export_config()).await {
            Ok(_) => panic!("youtube groups are not implemented"),
            Err(error) => error,
        };

        assert!(error
            .message
            .contains("YouTube source-group NotebookLM export is not implemented yet."));
    }

    #[tokio::test]
    async fn load_group_export_inputs_rejects_group_without_telegram_members() {
        let pool = source_group_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (2, 'youtube', 'video', 'youtube-1', 'YouTube')",
        )
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
             VALUES (9, 'Notebook Group', 'telegram', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert group");
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (9, 2, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert member");

        let error = match load_group_export_inputs(&pool, 9, &export_config()).await {
            Ok(_) => panic!("group without telegram members is rejected"),
            Err(error) => error,
        };

        assert!(error
            .message
            .contains("No Telegram sources found in this source group."));
    }

    #[test]
    fn validates_period_order() {
        let mut request = request();
        request.period_from = Some(2);
        request.period_to = Some(1);
        assert!(validate_request(request).is_err());
    }

    #[test]
    fn formats_timestamp_folder_suffix() {
        assert_eq!(timestamp_for_folder(0), "19700101-000000");
    }

    #[test]
    fn trims_optional_export_id() {
        let mut request = request();
        request.export_id = Some("  export-123  ".to_string());
        let config = validate_request(request).expect("valid request");
        assert_eq!(config.export_id.as_deref(), Some("export-123"));
    }

    #[test]
    fn keeps_migrated_history_opt_in_in_validated_config() {
        let mut request = request();
        request.include_migrated_history = true;
        let config = validate_request(request).expect("valid request");
        assert!(config.include_migrated_history);
    }

    #[test]
    fn treats_blank_export_id_as_missing() {
        let mut request = request();
        request.export_id = Some("   ".to_string());
        let config = validate_request(request).expect("valid request");
        assert_eq!(config.export_id, None);
    }

    #[test]
    fn reads_legacy_single_source_manifest_after_manifest_expansion() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-legacy-notebooklm-manifest-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).expect("create temp");
        std::fs::write(
            temp.join(EXPORT_MARKER_FILE),
            r#"{
          "generated_at": 1,
          "source_id": 7,
          "source_external_id": "source-7",
          "source_title": "Source 7",
          "file_count": 1,
          "exported_message_count": 2,
          "generated_files": ["glossary.md", "source.md"]
        }"#,
        )
        .expect("write old manifest");

        let manifest = read_manifest(&temp).expect("read manifest");

        assert_eq!(manifest.source_id, Some(7));
        assert_eq!(manifest.scope.as_deref(), Some("source"));
        assert!(manifest.members.is_empty());

        std::fs::remove_dir_all(&temp).expect("cleanup temp");
    }

    #[test]
    fn marker_read_and_write_reject_existing_symlink_file() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-notebooklm-marker-link-{}",
            std::process::id()
        ));
        let outside = std::env::temp_dir().join(format!(
            "extractum-notebooklm-marker-link-outside-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::remove_dir_all(&outside);
        std::fs::create_dir_all(&temp).expect("create temp");
        std::fs::create_dir_all(&outside).expect("create outside");
        let outside_marker = outside.join("manifest.json");
        let outside_json = r#"{
          "generated_at": 1,
          "scope": "source",
          "source_id": 7,
          "source_external_id": "source-7",
          "source_title": "Source 7",
          "file_count": 1,
          "exported_message_count": 2,
          "generated_files": ["glossary.md", "source.md"]
        }"#;
        std::fs::write(&outside_marker, outside_json).expect("write outside marker");
        if let Err(error) = symlink_file(&outside_marker, temp.join(EXPORT_MARKER_FILE)) {
            if should_skip_symlink_test(&error) {
                let _ = std::fs::remove_dir_all(&temp);
                let _ = std::fs::remove_dir_all(&outside);
                return;
            }
            panic!("create symlink: {error}");
        }

        let read_error = match read_manifest(&temp) {
            Ok(_) => panic!("symlink marker read is rejected"),
            Err(error) => error,
        };
        assert!(read_error.message.contains("symbolic link"));

        let write_error = write_marker(
            &temp,
            &NotebookLmExportManifest {
                generated_at: 2,
                scope: Some("source".to_string()),
                source_id: Some(8),
                source_external_id: Some("source-8".to_string()),
                source_title: Some("Source 8".to_string()),
                source_group_id: None,
                source_group_name: None,
                file_count: 1,
                exported_message_count: 3,
                skipped_message_count: 0,
                warning_count: 0,
                warnings: Vec::new(),
                generated_files: vec!["glossary.md".to_string(), "source.md".to_string()],
                members: Vec::new(),
            },
        )
        .expect_err("symlink marker write is rejected");
        assert!(write_error.message.contains("symbolic link"));
        assert_eq!(
            std::fs::read_to_string(&outside_marker).expect("read outside marker"),
            outside_json
        );

        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn marker_read_and_write_accept_normal_file() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-notebooklm-marker-normal-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).expect("create temp");

        write_marker(
            &temp,
            &NotebookLmExportManifest {
                generated_at: 2,
                scope: Some("source".to_string()),
                source_id: Some(8),
                source_external_id: Some("source-8".to_string()),
                source_title: Some("Source 8".to_string()),
                source_group_id: None,
                source_group_name: None,
                file_count: 1,
                exported_message_count: 3,
                skipped_message_count: 1,
                warning_count: 1,
                warnings: vec!["warning".to_string()],
                generated_files: vec!["glossary.md".to_string(), "source.md".to_string()],
                members: Vec::new(),
            },
        )
        .expect("write normal marker");

        let manifest = read_manifest(&temp).expect("read normal marker");

        assert_eq!(manifest.source_id, Some(8));
        assert_eq!(manifest.exported_message_count, 3);
        assert_eq!(manifest.warnings, vec!["warning".to_string()]);

        std::fs::remove_dir_all(&temp).expect("cleanup temp");
    }

    #[test]
    fn removes_generated_files_in_sources_subdirectory() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-group-notebooklm-cleanup-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(temp.join("sources")).expect("create sources dir");
        std::fs::write(temp.join("glossary.md"), "glossary").expect("write glossary");
        std::fs::write(
            temp.join("sources").join("001-source-7-alpha-part-001.md"),
            "chunk",
        )
        .expect("write chunk");
        write_marker(
            &temp,
            &NotebookLmExportManifest {
                generated_at: 1,
                scope: Some("source_group".to_string()),
                source_id: None,
                source_external_id: None,
                source_title: None,
                source_group_id: Some(9),
                source_group_name: Some("Group".to_string()),
                file_count: 1,
                exported_message_count: 1,
                skipped_message_count: 0,
                warning_count: 0,
                warnings: Vec::new(),
                generated_files: vec![
                    "glossary.md".to_string(),
                    "sources/001-source-7-alpha-part-001.md".to_string(),
                ],
                members: vec![NotebookLmExportManifestMember {
                    source_id: 7,
                    source_title: Some("Alpha".to_string()),
                    source_subtype: Some("channel".to_string()),
                    exported_message_count: 1,
                    skipped_message_count: 0,
                    generated_files: vec!["sources/001-source-7-alpha-part-001.md".to_string()],
                    warnings: Vec::new(),
                    skipped_reason: None,
                }],
            },
        )
        .expect("write marker");

        remove_generated_files(&temp).expect("remove generated files");

        assert!(!temp.join("glossary.md").exists());
        assert!(!temp
            .join("sources")
            .join("001-source-7-alpha-part-001.md")
            .exists());

        std::fs::remove_dir_all(&temp).expect("cleanup temp");
    }

    #[test]
    fn remove_generated_files_rejects_invalid_manifest_relative_path() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-group-notebooklm-invalid-manifest-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(temp.join("sources")).expect("create sources dir");
        std::fs::write(temp.join("sources").join("source.md"), "chunk").expect("write chunk");
        write_marker(
            &temp,
            &NotebookLmExportManifest {
                generated_at: 1,
                scope: Some("source_group".to_string()),
                source_id: None,
                source_external_id: None,
                source_title: None,
                source_group_id: Some(9),
                source_group_name: Some("Group".to_string()),
                file_count: 1,
                exported_message_count: 1,
                skipped_message_count: 0,
                warning_count: 0,
                warnings: Vec::new(),
                generated_files: vec!["sources/nul.md".to_string()],
                members: Vec::new(),
            },
        )
        .expect("write marker");

        let error = remove_generated_files(&temp).expect_err("invalid path is rejected");

        assert!(error.message.contains("invalid file path"));
        assert!(temp.join("sources").join("source.md").exists());

        std::fs::remove_dir_all(&temp).expect("cleanup temp");
    }

    #[test]
    fn write_export_file_creates_sources_parent_directory() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-group-notebooklm-write-sources-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).expect("create temp");

        let path = write_export_file(&temp, "sources/example.md", "example").expect("write file");

        assert_eq!(path, temp.join("sources").join("example.md"));
        assert_eq!(
            std::fs::read_to_string(temp.join("sources").join("example.md"))
                .expect("read written file"),
            "example"
        );

        std::fs::remove_dir_all(&temp).expect("cleanup temp");
    }

    #[test]
    fn write_export_file_rejects_symlink_parent_directory() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-group-notebooklm-write-link-{}",
            std::process::id()
        ));
        let outside = std::env::temp_dir().join(format!(
            "extractum-group-notebooklm-write-link-outside-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::remove_dir_all(&outside);
        std::fs::create_dir_all(&temp).expect("create temp");
        std::fs::create_dir_all(&outside).expect("create outside");
        if let Err(error) = symlink_dir(&outside, temp.join("sources")) {
            if should_skip_symlink_test(&error) {
                let _ = std::fs::remove_dir_all(&temp);
                let _ = std::fs::remove_dir_all(&outside);
                return;
            }
            panic!("create symlink: {error}");
        }

        let error = write_export_file(&temp, "sources/example.md", "example")
            .expect_err("symlink parent is rejected");

        assert!(error.message.contains("symbolic link"));
        assert!(!outside.join("example.md").exists());

        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn remove_generated_files_rejects_symlink_parent_directory() {
        let temp = std::env::temp_dir().join(format!(
            "extractum-group-notebooklm-remove-link-{}",
            std::process::id()
        ));
        let outside = std::env::temp_dir().join(format!(
            "extractum-group-notebooklm-remove-link-outside-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::remove_dir_all(&outside);
        std::fs::create_dir_all(&temp).expect("create temp");
        std::fs::create_dir_all(&outside).expect("create outside");
        std::fs::write(outside.join("victim.md"), "outside").expect("write outside file");
        if let Err(error) = symlink_dir(&outside, temp.join("sources")) {
            if should_skip_symlink_test(&error) {
                let _ = std::fs::remove_dir_all(&temp);
                let _ = std::fs::remove_dir_all(&outside);
                return;
            }
            panic!("create symlink: {error}");
        }
        write_marker(
            &temp,
            &NotebookLmExportManifest {
                generated_at: 1,
                scope: Some("source_group".to_string()),
                source_id: None,
                source_external_id: None,
                source_title: None,
                source_group_id: Some(9),
                source_group_name: Some("Group".to_string()),
                file_count: 1,
                exported_message_count: 1,
                skipped_message_count: 0,
                warning_count: 0,
                warnings: Vec::new(),
                generated_files: vec!["sources/victim.md".to_string()],
                members: Vec::new(),
            },
        )
        .expect("write marker");

        let error =
            remove_generated_files(&temp).expect_err("symlink parent is rejected before removal");

        assert!(error.message.contains("symbolic link"));
        assert!(outside.join("victim.md").exists());

        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[cfg(unix)]
    fn symlink_dir(
        original: impl AsRef<std::path::Path>,
        link: impl AsRef<std::path::Path>,
    ) -> io::Result<()> {
        std::os::unix::fs::symlink(original, link)
    }

    #[cfg(windows)]
    fn symlink_dir(
        original: impl AsRef<std::path::Path>,
        link: impl AsRef<std::path::Path>,
    ) -> io::Result<()> {
        std::os::windows::fs::symlink_dir(original, link)
    }

    #[cfg(unix)]
    fn symlink_file(
        original: impl AsRef<std::path::Path>,
        link: impl AsRef<std::path::Path>,
    ) -> io::Result<()> {
        std::os::unix::fs::symlink(original, link)
    }

    #[cfg(windows)]
    fn symlink_file(
        original: impl AsRef<std::path::Path>,
        link: impl AsRef<std::path::Path>,
    ) -> io::Result<()> {
        std::os::windows::fs::symlink_file(original, link)
    }

    fn should_skip_symlink_test(error: &io::Error) -> bool {
        matches!(
            error.kind(),
            io::ErrorKind::PermissionDenied
                | io::ErrorKind::Unsupported
                | io::ErrorKind::InvalidInput
        ) || error.raw_os_error() == Some(1314)
    }
}
