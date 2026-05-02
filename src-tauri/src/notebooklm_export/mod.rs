mod chunker;
mod filename;
mod glossary;
mod links;
mod media;
mod model;
mod query;
mod renderer;

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::AppHandle;
use time::{format_description, OffsetDateTime, UtcOffset};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};

use chunker::{build_chunks, should_export_message};
use filename::{ensure_child_path, sanitize_path_component};
use glossary::{aggregate_participants, glossary_word_count, render_glossary};
use model::{
    NotebookLmExportConfig, NotebookLmExportFile, NotebookLmExportRequest, NotebookLmExportResult,
    DEFAULT_MAX_BYTES_PER_FILE, DEFAULT_MAX_WORDS_PER_FILE, DEFAULT_MIN_MESSAGE_LENGTH,
};
use query::{load_export_messages, load_export_source};
use renderer::{approx_word_count, render_document, render_message_block};

const EXPORT_MARKER_FILE: &str = ".extractum-notebooklm-export.json";

#[tauri::command]
pub async fn export_source_to_notebooklm(
    handle: AppHandle,
    request: NotebookLmExportRequest,
) -> AppResult<NotebookLmExportResult> {
    let config = validate_request(request)?;
    let pool = get_pool(&handle).await?;
    let source = load_export_source(&pool, config.source_id).await?;
    let messages = load_export_messages(
        &pool,
        config.source_id,
        config.period_from,
        config.period_to,
    )
    .await?;
    let generated_at = now_secs();

    tauri::async_runtime::spawn_blocking(move || {
        let mut warnings = Vec::new();
        let mut skipped_message_count = 0;
        let blocks = messages
            .iter()
            .filter_map(|message| {
                if !should_export_message(
                    message,
                    config.min_message_length,
                    config.include_media_placeholders,
                ) {
                    skipped_message_count += 1;
                    return None;
                }

                let mut message = message.clone();
                if !config.include_media_placeholders {
                    message.media_placeholders.clear();
                }
                Some(render_message_block(&message))
            })
            .collect::<Vec<_>>();

        let exported_messages = blocks
            .iter()
            .map(|block| block.message.clone())
            .collect::<Vec<_>>();
        let participants = aggregate_participants(&exported_messages);
        let (chunks, chunk_warnings) = build_chunks(
            &source,
            &blocks,
            config.max_words_per_file,
            config.max_bytes_per_file,
        );
        warnings.extend(chunk_warnings);

        let output_root = prepare_output_root(&config, &source, generated_at)?;
        let glossary_markdown = render_glossary(
            generated_at,
            source.title.as_deref().unwrap_or(&source.external_id),
            &participants,
        );
        let glossary_path = write_export_file(&output_root, "glossary.md", &glossary_markdown)?;

        let mut files = Vec::new();
        for chunk in chunks {
            let markdown = render_document(
                &source,
                generated_at,
                &chunk.title_period,
                chunk.period_start,
                chunk.period_end,
                &participants,
                &chunk.blocks,
                chunk.part_number > 1,
            );
            let path = write_export_file(&output_root, &chunk.filename, &markdown)?;
            files.push(NotebookLmExportFile {
                path: path_to_string(path),
                message_count: chunk.blocks.len(),
                byte_size: markdown.len(),
                approximate_word_count: approx_word_count(&markdown),
            });
        }

        write_marker(
            &output_root,
            &NotebookLmExportManifest {
                generated_at,
                source_id: source.id,
                source_external_id: source.external_id.clone(),
                source_title: source.title.clone(),
                file_count: files.len(),
                exported_message_count: blocks.len(),
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
    .map_err(|e| AppError::internal(format!("NotebookLM export task failed: {e}")))?
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
        let marker = output_root.join(EXPORT_MARKER_FILE);
        if !marker.exists() {
            return Err(AppError::conflict(
                "Deterministic export folder exists without Extractum export marker",
            ));
        }
        remove_generated_files(&output_root)?;
    } else {
        fs::create_dir(&output_root).map_err(map_create_dir_error)?;
    }

    Ok(output_root)
}

fn remove_generated_files(output_root: &Path) -> AppResult<()> {
    for entry in fs::read_dir(output_root)
        .map_err(|e| AppError::internal(format!("Could not read export folder: {e}")))?
    {
        let entry = entry.map_err(|e| AppError::internal(format!("Could not read file: {e}")))?;
        let path = entry.path();
        let is_generated_markdown = path.extension().and_then(|ext| ext.to_str()) == Some("md");
        let is_marker = path.file_name().and_then(|name| name.to_str()) == Some(EXPORT_MARKER_FILE);
        if is_generated_markdown || is_marker {
            fs::remove_file(&path)
                .map_err(|e| AppError::conflict(format!("Could not replace export file: {e}")))?;
        }
    }
    Ok(())
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

fn now_secs() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

#[derive(Serialize)]
struct NotebookLmExportManifest {
    generated_at: i64,
    source_id: i64,
    source_external_id: String,
    source_title: Option<String>,
    file_count: usize,
    exported_message_count: usize,
}

#[cfg(test)]
mod tests {
    use super::{timestamp_for_folder, validate_request};
    use crate::notebooklm_export::model::NotebookLmExportRequest;

    fn request() -> NotebookLmExportRequest {
        NotebookLmExportRequest {
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
}
