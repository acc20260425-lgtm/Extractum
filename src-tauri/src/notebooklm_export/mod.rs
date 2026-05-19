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
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use time::{format_description, OffsetDateTime, UtcOffset};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{require_source_identity_ready, SourceIdentityRepairState};
use crate::time::now_secs;

use chunker::{build_chunks, should_export_message};
use filename::{ensure_child_path, sanitize_path_component};
use glossary::{aggregate_participants, glossary_word_count, render_glossary};
use model::{
    NotebookLmExportConfig, NotebookLmExportFile, NotebookLmExportRequest, NotebookLmExportResult,
    DEFAULT_MAX_BYTES_PER_FILE, DEFAULT_MAX_WORDS_PER_FILE, DEFAULT_MIN_MESSAGE_LENGTH,
};
use query::{load_export_messages, load_export_source};
use renderer::{
    approx_word_count, render_document, render_document_overhead, render_message_block,
    DocumentRenderContext,
};

const EXPORT_MARKER_FILE: &str = ".extractum-notebooklm-export.json";
const NOTEBOOKLM_EXPORT_EVENT: &str = "notebooklm://export";

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
            .unwrap_or_else(|| format!("notebooklm-{}-{generated_at}", config.source_id)),
        config.source_id,
    );

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
    let source = match load_export_source(&pool, config.source_id).await {
        Ok(source) => source,
        Err(error) => {
            progress.emit_failed("loading", &error);
            return Err(error);
        }
    };
    let messages = match load_export_messages(
        &pool,
        config.source_id,
        config.period_from,
        config.period_to,
    )
    .await
    {
        Ok(messages) => messages,
        Err(error) => {
            progress.emit_failed("loading", &error);
            return Err(error);
        }
    };

    progress.emit_progress(
        "filtering",
        "Filtering and rendering message blocks.",
        Some(0),
        Some(messages.len()),
        None,
    );

    let task_progress = progress.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut warnings = Vec::new();
        let mut skipped_message_count = 0;
        let mut blocks = Vec::new();
        let filter_total = messages.len();
        let filter_step = progress_step(filter_total);

        for (index, message) in messages.iter().enumerate() {
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

            let current = index + 1;
            if should_emit_progress(current, filter_total, filter_step) {
                task_progress.emit_progress(
                    "filtering",
                    "Filtering and rendering message blocks.",
                    Some(current),
                    Some(filter_total),
                    None,
                );
            }
        }

        let exported_messages = blocks
            .iter()
            .map(|block| block.message.clone())
            .collect::<Vec<_>>();
        let participants = aggregate_participants(&exported_messages);
        task_progress.emit_progress(
            "chunking",
            "Grouping messages into NotebookLM-sized Markdown files.",
            None,
            None,
            None,
        );
        let (chunks, chunk_warnings) = build_chunks(
            &source,
            &blocks,
            config.max_words_per_file,
            config.max_bytes_per_file,
            |topic, title_period, period_start, period_end, is_continuation, message_count| {
                let context = DocumentRenderContext {
                    source: &source,
                    topic,
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
        warnings.extend(chunk_warnings);

        task_progress.emit_progress(
            "preparing_output",
            "Preparing the export folder.",
            None,
            None,
            None,
        );
        let output_root = prepare_output_root(&config, &source, generated_at)?;
        let mut generated_file_names = vec!["glossary.md".to_string()];
        let glossary_markdown = render_glossary(
            generated_at,
            source.title.as_deref().unwrap_or(&source.external_id),
            &participants,
        );
        let write_total = chunks.len() + 1;
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
        for (index, chunk) in chunks.into_iter().enumerate() {
            generated_file_names.push(chunk.filename.clone());
            let context = DocumentRenderContext {
                source: &source,
                topic: &chunk.topic,
                generated_at,
                title_period: &chunk.title_period,
                period_start: chunk.period_start,
                period_end: chunk.period_end,
                participants: &participants,
                message_count: chunk.blocks.len(),
                is_continuation: chunk.part_number > 1,
            };
            let markdown = render_document(&context, &chunk.blocks);
            task_progress.emit_progress(
                "writing",
                &format!("Writing {}.", chunk.filename),
                Some(index + 1),
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
            task_progress.emit_progress(
                "writing",
                &format!("Writing {}.", chunk.filename),
                Some(index + 2),
                Some(write_total),
                Some(&chunk.filename),
            );
        }

        task_progress.emit_progress("manifest", "Writing export manifest.", None, None, None);
        write_marker(
            &output_root,
            &NotebookLmExportManifest {
                generated_at,
                source_id: source.id,
                source_external_id: source.external_id.clone(),
                source_title: source.title.clone(),
                file_count: files.len(),
                exported_message_count: blocks.len(),
                generated_files: generated_file_names,
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
            exported_message_count: blocks.len(),
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

    Ok(NotebookLmExportConfig {
        export_id: request
            .export_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        source_id: request.source_id,
        output_dir: output_dir.to_string(),
        period_from: request.period_from,
        period_to: request.period_to,
        include_media_placeholders: request.include_media_placeholders,
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

fn prepare_output_root(
    config: &NotebookLmExportConfig,
    source: &model::NotebookLmExportSource,
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

    let source_slug = sanitize_path_component(
        source.title.as_deref().unwrap_or(&source.external_id),
        "source",
    );
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
        let path = ensure_child_path(output_root, &file_name).ok_or_else(|| {
            AppError::conflict("Existing export manifest contains an invalid file path")
        })?;
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
    if metadata.file_type().is_symlink() {
        return Err(AppError::conflict(
            "Export folder cannot be a symbolic link",
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
    let path = ensure_child_path(output_root, filename).ok_or_else(|| {
        AppError::validation(format!("Generated filename '{filename}' is invalid"))
    })?;
    fs::write(&path, content)
        .map_err(|e| AppError::internal(format!("Could not write export file: {e}")))?;
    Ok(path)
}

fn write_marker(output_root: &Path, manifest: &NotebookLmExportManifest) -> AppResult<()> {
    let path = output_root.join(EXPORT_MARKER_FILE);
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| AppError::internal(format!("Could not serialize export manifest: {e}")))?;
    fs::write(path, json)
        .map_err(|e| AppError::internal(format!("Could not write export manifest: {e}")))
}

fn read_manifest(output_root: &Path) -> AppResult<NotebookLmExportManifest> {
    let path = ensure_child_path(output_root, EXPORT_MARKER_FILE)
        .ok_or_else(|| AppError::validation("Export marker filename is invalid"))?;
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
    source_id: i64,
    source_external_id: String,
    source_title: Option<String>,
    file_count: usize,
    exported_message_count: usize,
    generated_files: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{timestamp_for_folder, validate_request};
    use crate::notebooklm_export::model::NotebookLmExportRequest;

    fn request() -> NotebookLmExportRequest {
        NotebookLmExportRequest {
            export_id: None,
            source_id: 1,
            output_dir: ".".to_string(),
            period_from: None,
            period_to: None,
            include_media_placeholders: true,
            min_message_length: 3,
            max_words_per_file: 300_000,
            max_bytes_per_file: 50_000_000,
            overwrite_existing: false,
        }
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
    fn treats_blank_export_id_as_missing() {
        let mut request = request();
        request.export_id = Some("   ".to_string());
        let config = validate_request(request).expect("valid request");
        assert_eq!(config.export_id, None);
    }
}
