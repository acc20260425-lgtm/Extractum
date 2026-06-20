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
        create_queued_run, finish_run, list_runs, mark_running, GeminiBrowserArtifactRefs,
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
    }
}
