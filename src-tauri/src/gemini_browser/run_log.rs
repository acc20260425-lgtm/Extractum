use std::fs;
use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use crate::error::{AppError, AppResult};

use super::paths::safe_run_id;
use super::{
    GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunResult,
    GeminiBrowserRunStatus,
};

const RUN_FILE: &str = "result.json";
const PROMPT_PREVIEW_CHARS: usize = 120;

fn now_string() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn run_file_path(runs_dir: &Path, run_id: &str) -> AppResult<PathBuf> {
    Ok(runs_dir.join(safe_run_id(run_id)?).join(RUN_FILE))
}

fn prompt_preview(prompt: &str) -> String {
    let mut chars = prompt.trim().chars();
    let preview = chars.by_ref().take(PROMPT_PREVIEW_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
}

pub(crate) fn create_queued_run(
    runs_dir: &Path,
    run_id: &str,
    source: &str,
    prompt: &str,
) -> AppResult<GeminiBrowserRun> {
    let run_dir = runs_dir.join(safe_run_id(run_id)?);
    fs::create_dir_all(&run_dir).map_err(|error| AppError::internal(error.to_string()))?;
    let now = now_string();
    let run = GeminiBrowserRun {
        run_id: run_id.to_string(),
        source: source.to_string(),
        status: GeminiBrowserRunStatus::Queued,
        prompt_preview: prompt_preview(prompt),
        created_at: now.clone(),
        updated_at: now,
        result: None,
    };
    write_run(&run_dir.join(RUN_FILE), &run)?;
    Ok(run)
}

pub(crate) fn mark_running(runs_dir: &Path, run_id: &str) -> AppResult<GeminiBrowserRun> {
    let mut run = read_run_file(&run_file_path(runs_dir, run_id)?)?;
    run.status = GeminiBrowserRunStatus::Running;
    run.updated_at = now_string();
    write_run(&run_file_path(runs_dir, run_id)?, &run)?;
    Ok(run)
}

pub(crate) fn finish_run(
    runs_dir: &Path,
    run_id: &str,
    result: GeminiBrowserRunResult,
) -> AppResult<GeminiBrowserRun> {
    let mut run = read_run_file(&run_file_path(runs_dir, run_id)?)?;
    run.status = result.status.clone();
    run.updated_at = now_string();
    run.result = Some(result);
    write_run(&run_file_path(runs_dir, run_id)?, &run)?;
    Ok(run)
}

pub(crate) fn list_runs(runs_dir: &Path, limit: usize) -> AppResult<GeminiBrowserRunLogSummary> {
    if !runs_dir.exists() {
        return Ok(GeminiBrowserRunLogSummary { runs: Vec::new() });
    }
    let mut runs = Vec::new();
    for entry in fs::read_dir(runs_dir).map_err(|error| AppError::internal(error.to_string()))? {
        let path = entry
            .map_err(|error| AppError::internal(error.to_string()))?
            .path()
            .join(RUN_FILE);
        if path.exists() {
            runs.push(read_run_file(&path)?);
        }
    }
    runs.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    runs.truncate(limit);
    Ok(GeminiBrowserRunLogSummary { runs })
}

pub(crate) fn recorded_run_dir(runs_dir: &Path, run_id: &str) -> AppResult<PathBuf> {
    let safe_id = safe_run_id(run_id)?;
    let dir = runs_dir.join(&safe_id);
    let result_path = dir.join(RUN_FILE);
    if !result_path.exists() {
        return Err(AppError::validation("Gemini browser run was not found"));
    }
    let run = read_run_file(&result_path)?;
    let _recorded_run_dir = run
        .result
        .as_ref()
        .and_then(|result| result.artifacts.run_dir.as_deref())
        .ok_or_else(|| AppError::validation("Gemini browser run folder is not available"))?;

    dir.canonicalize().map_err(|error| {
        AppError::internal(format!(
            "Failed to resolve Gemini browser run folder: {error}"
        ))
    })
}

fn read_run_file(path: &Path) -> AppResult<GeminiBrowserRun> {
    let content = fs::read_to_string(path).map_err(|error| AppError::internal(error.to_string()))?;
    serde_json::from_str(&content).map_err(|error| AppError::internal(error.to_string()))
}

fn write_run(path: &Path, run: &GeminiBrowserRun) -> AppResult<()> {
    let content =
        serde_json::to_string_pretty(run).map_err(|error| AppError::internal(error.to_string()))?;
    fs::write(path, content).map_err(|error| AppError::internal(error.to_string()))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::gemini_browser::{
        create_queued_run, finish_run, list_runs, mark_running,
        GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs,
        GeminiBrowserDebugErrorStage, GeminiBrowserProviderMode, GeminiBrowserRunDebugSummary,
        GeminiBrowserRunResult, GeminiBrowserRunStatus,
    };

    #[test]
    fn run_log_persists_queued_running_and_terminal_result() {
        let temp = tempdir().expect("tempdir");
        let runs_dir = temp.path();

        let queued = create_queued_run(runs_dir, "run-1", "settings_test", "hello Gemini")
            .expect("create queued run");
        assert_eq!(queued.status, GeminiBrowserRunStatus::Queued);

        let running = mark_running(runs_dir, "run-1").expect("mark running");
        assert_eq!(running.status, GeminiBrowserRunStatus::Running);

        let result = GeminiBrowserRunResult {
            run_id: "run-1".to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 25,
            debug_summary: Some(GeminiBrowserRunDebugSummary {
                mode: GeminiBrowserProviderMode::Managed,
                composer_found: true,
                send_button_found: true,
                generation_busy_observed: false,
                answer_found: true,
                answer_selector: Some("message-content".to_string()),
                waited_for_send_ms: 0,
                waited_for_answer_ms: 8_000,
                answer_stable_ms: 8_000,
                answer_completion_reason: GeminiBrowserAnswerCompletionReason::Stable,
                final_text_length: 6,
                error_stage: Some(GeminiBrowserDebugErrorStage::Answer),
            }),
        };
        let finished = finish_run(runs_dir, "run-1", result).expect("finish run");
        assert_eq!(finished.status, GeminiBrowserRunStatus::Ok);
        assert_eq!(
            finished.result.expect("result").text,
            Some("answer".to_string())
        );

        let listed = list_runs(runs_dir, 10).expect("list runs");
        assert_eq!(listed.runs.len(), 1);
        assert_eq!(listed.runs[0].run_id, "run-1");
        assert_eq!(
            listed.runs[0]
                .result
                .as_ref()
                .and_then(|result| result.debug_summary.as_ref())
                .and_then(|summary| summary.answer_selector.as_deref()),
            Some("message-content")
        );
    }

    #[test]
    fn recorded_run_dir_requires_result_artifact_flag_and_returns_computed_dir() {
        let temp = tempdir().expect("tempdir");
        let runs_dir = temp.path();
        create_queued_run(runs_dir, "run-1", "settings_test", "hello Gemini")
            .expect("create queued run");
        assert!(super::recorded_run_dir(runs_dir, "run-1").is_err());

        let run_dir = runs_dir.join("run-1");
        let result = GeminiBrowserRunResult {
            run_id: "run-1".to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs {
                run_dir: Some(run_dir.to_string_lossy().to_string()),
                ..Default::default()
            },
            elapsed_ms: 25,
            debug_summary: None,
        };
        finish_run(runs_dir, "run-1", result).expect("finish run");

        let dir = super::recorded_run_dir(runs_dir, "run-1").expect("recorded run dir");
        assert_eq!(dir.file_name().and_then(|name| name.to_str()), Some("run-1"));

        create_queued_run(runs_dir, "run-2", "settings_test", "hello Gemini")
            .expect("create queued run");
        let outside = temp.path().join("outside-run-dir");
        std::fs::create_dir_all(&outside).expect("outside dir");
        let mismatched = GeminiBrowserRunResult {
            run_id: "run-2".to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs {
                run_dir: Some(outside.to_string_lossy().to_string()),
                ..Default::default()
            },
            elapsed_ms: 25,
            debug_summary: None,
        };
        finish_run(runs_dir, "run-2", mismatched).expect("finish run");
        let dir = super::recorded_run_dir(runs_dir, "run-2").expect("recorded run dir");
        assert_eq!(dir.file_name().and_then(|name| name.to_str()), Some("run-2"));
        assert_ne!(dir, outside.canonicalize().expect("outside canonicalize"));

        create_queued_run(runs_dir, "run-3", "settings_test", "hello Gemini")
            .expect("create queued run");
        let no_artifact = GeminiBrowserRunResult {
            run_id: "run-3".to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 25,
            debug_summary: None,
        };
        finish_run(runs_dir, "run-3", no_artifact).expect("finish run");
        assert!(super::recorded_run_dir(runs_dir, "run-3").is_err());
        assert!(super::recorded_run_dir(runs_dir, "../bad").is_err());
        assert!(super::recorded_run_dir(runs_dir, "missing-run").is_err());
    }
}
