use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

use super::domain_error::{GeminiBrowserError, GeminiBrowserResult};
use super::run_id::safe_run_id;
use super::{
    GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

const RUN_FILE: &str = "result.json";
const PROMPT_PREVIEW_CHARS: usize = 120;
const RUN_RETENTION_HOURS: i64 = 24;

fn now_string() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn run_file_path(runs_dir: &Path, run_id: &str) -> GeminiBrowserResult<PathBuf> {
    Ok(runs_dir.join(safe_run_id(run_id)?).join(RUN_FILE))
}

fn prompt_preview(prompt: &str) -> String {
    let mut chars = prompt.trim().chars();
    let preview = chars
        .by_ref()
        .take(PROMPT_PREVIEW_CHARS)
        .collect::<String>();
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
) -> GeminiBrowserResult<GeminiBrowserRun> {
    prune_expired_runs(runs_dir)?;
    let run_dir = runs_dir.join(safe_run_id(run_id)?);
    fs::create_dir_all(&run_dir)
        .map_err(|error| GeminiBrowserError::persistence(error.to_string()))?;
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

pub(crate) fn mark_running(runs_dir: &Path, run_id: &str) -> GeminiBrowserResult<GeminiBrowserRun> {
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
) -> GeminiBrowserResult<GeminiBrowserRun> {
    let mut run = read_run_file(&run_file_path(runs_dir, run_id)?)?;
    run.status = result.status.clone();
    run.updated_at = now_string();
    run.result = Some(result);
    write_run(&run_file_path(runs_dir, run_id)?, &run)?;
    Ok(run)
}

pub(crate) fn list_runs(
    runs_dir: &Path,
    limit: usize,
) -> GeminiBrowserResult<GeminiBrowserRunLogSummary> {
    prune_expired_runs(runs_dir)?;
    if !runs_dir.exists() {
        return Ok(GeminiBrowserRunLogSummary { runs: Vec::new() });
    }
    let mut runs = Vec::new();
    for entry in fs::read_dir(runs_dir)
        .map_err(|error| GeminiBrowserError::persistence(error.to_string()))?
    {
        let path = entry
            .map_err(|error| GeminiBrowserError::persistence(error.to_string()))?
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

pub(crate) fn read_run(runs_dir: &Path, run_id: &str) -> GeminiBrowserResult<GeminiBrowserRun> {
    prune_expired_runs(runs_dir)?;
    let path = run_file_path(runs_dir, run_id)?;
    if !path.exists() {
        return Err(GeminiBrowserError::not_found(
            "Gemini browser run was not found",
        ));
    }
    read_run_file(&path)
}

pub(crate) fn recorded_run_dir(runs_dir: &Path, run_id: &str) -> GeminiBrowserResult<PathBuf> {
    prune_expired_runs(runs_dir)?;
    let safe_id = safe_run_id(run_id)?;
    let dir = runs_dir.join(&safe_id);
    let result_path = dir.join(RUN_FILE);
    if !result_path.exists() {
        return Err(GeminiBrowserError::validation(
            "Gemini browser run was not found",
        ));
    }
    let run = read_run_file(&result_path)?;
    let _recorded_run_dir = run
        .result
        .as_ref()
        .and_then(|result| result.artifacts.run_dir.as_deref())
        .ok_or_else(|| {
            GeminiBrowserError::validation("Gemini browser run folder is not available")
        })?;

    dir.canonicalize().map_err(|error| {
        GeminiBrowserError::persistence(format!(
            "Failed to resolve Gemini browser run folder: {error}"
        ))
    })
}

fn read_run_file(path: &Path) -> GeminiBrowserResult<GeminiBrowserRun> {
    let content = fs::read_to_string(path)
        .map_err(|error| GeminiBrowserError::persistence(error.to_string()))?;
    serde_json::from_str(&content)
        .map_err(|error| GeminiBrowserError::persistence(error.to_string()))
}

fn write_run(path: &Path, run: &GeminiBrowserRun) -> GeminiBrowserResult<()> {
    let content = serde_json::to_string_pretty(run)
        .map_err(|error| GeminiBrowserError::persistence(error.to_string()))?;
    fs::write(path, content).map_err(|error| GeminiBrowserError::persistence(error.to_string()))
}

fn prune_expired_runs(runs_dir: &Path) -> GeminiBrowserResult<()> {
    prune_expired_runs_at(runs_dir, OffsetDateTime::now_utc())
}

fn prune_expired_runs_at(runs_dir: &Path, now: OffsetDateTime) -> GeminiBrowserResult<()> {
    if !runs_dir.exists() {
        return Ok(());
    }

    let cutoff = now - Duration::hours(RUN_RETENTION_HOURS);
    for entry in fs::read_dir(runs_dir)
        .map_err(|error| GeminiBrowserError::persistence(error.to_string()))?
    {
        let run_dir = entry
            .map_err(|error| GeminiBrowserError::persistence(error.to_string()))?
            .path();
        if !run_dir.is_dir() {
            continue;
        }

        let result_path = run_dir.join(RUN_FILE);
        if !result_path.exists() {
            continue;
        }

        let Some(updated_at) = run_updated_at_or_modified_at(&result_path)? else {
            continue;
        };
        if updated_at < cutoff {
            remove_run_dir(&run_dir)?;
        }
    }

    Ok(())
}

fn run_updated_at_or_modified_at(
    result_path: &Path,
) -> GeminiBrowserResult<Option<OffsetDateTime>> {
    if let Ok(content) = fs::read_to_string(result_path) {
        if let Ok(run) = serde_json::from_str::<GeminiBrowserRun>(&content) {
            if let Ok(updated_at) = OffsetDateTime::parse(&run.updated_at, &Rfc3339) {
                return Ok(Some(updated_at));
            }
        }
    }

    file_modified_at(result_path)
}

fn file_modified_at(path: &Path) -> GeminiBrowserResult<Option<OffsetDateTime>> {
    let modified = match fs::metadata(path).and_then(|metadata| metadata.modified()) {
        Ok(modified) => modified,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(GeminiBrowserError::persistence(error.to_string())),
    };
    Ok(system_time_to_offset_datetime(modified))
}

fn system_time_to_offset_datetime(value: SystemTime) -> Option<OffsetDateTime> {
    let duration = value.duration_since(UNIX_EPOCH).ok()?;
    OffsetDateTime::from_unix_timestamp(duration.as_secs() as i64).ok()
}

fn remove_run_dir(run_dir: &Path) -> GeminiBrowserResult<()> {
    match fs::remove_dir_all(run_dir) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(GeminiBrowserError::persistence(error.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

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
                extraction: None,
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
    fn read_run_returns_exact_run_by_id() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let runs_dir = temp.path();

        create_queued_run(runs_dir, "run-detail", "settings_test", "hello")
            .expect("create queued run");

        let run = super::read_run(runs_dir, "run-detail").expect("read run");

        assert_eq!(run.run_id, "run-detail");
        assert_eq!(run.status, GeminiBrowserRunStatus::Queued);
    }

    #[test]
    fn get_run_core_returns_exact_run_from_log() {
        let temp = tempfile::tempdir().expect("create temp dir");
        create_queued_run(temp.path(), "run-core", "settings_test", "hello")
            .expect("create queued run");
        let run = super::read_run(temp.path(), "run-core").expect("read exact run");
        assert_eq!(run.run_id, "run-core");
        assert_eq!(run.status, GeminiBrowserRunStatus::Queued);
    }

    #[test]
    fn read_run_returns_validation_error_for_missing_run() {
        let temp = tempfile::tempdir().expect("create temp dir");

        let error = super::read_run(temp.path(), "missing-run").expect_err("missing run errors");

        assert_eq!(
            error.kind(),
            crate::gemini_browser::domain_error::GeminiBrowserErrorKind::NotFound
        );
        assert_eq!(error.message(), "Gemini browser run was not found");
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
        assert_eq!(
            dir.file_name().and_then(|name| name.to_str()),
            Some("run-1")
        );

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
        assert_eq!(
            dir.file_name().and_then(|name| name.to_str()),
            Some("run-2")
        );
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

    const EXPIRED_RUN_AGE_HOURS: i64 = super::RUN_RETENTION_HOURS + 1;
    const FRESH_RUN_AGE_HOURS: i64 = super::RUN_RETENTION_HOURS - 1;

    #[test]
    fn list_runs_deletes_run_directories_outside_retention_window() {
        let temp = tempdir().expect("tempdir");
        let runs_dir = temp.path();

        create_queued_run(runs_dir, "old-run", "settings_test", "old prompt")
            .expect("create old run");
        create_queued_run(runs_dir, "fresh-run", "settings_test", "fresh prompt")
            .expect("create fresh run");
        set_run_updated_at(runs_dir, "old-run", hours_ago(EXPIRED_RUN_AGE_HOURS));
        set_run_updated_at(runs_dir, "fresh-run", hours_ago(FRESH_RUN_AGE_HOURS));
        fs::write(
            runs_dir.join("old-run").join("page.html"),
            "<html>debug</html>",
        )
        .expect("write old artifact");

        let listed = list_runs(runs_dir, 10).expect("list runs");

        assert_eq!(listed.runs.len(), 1);
        assert_eq!(listed.runs[0].run_id, "fresh-run");
        assert!(!runs_dir.join("old-run").exists());
        assert!(runs_dir.join("fresh-run").exists());
    }

    #[test]
    fn create_queued_run_prunes_expired_runs_before_writing_new_run() {
        let temp = tempdir().expect("tempdir");
        let runs_dir = temp.path();

        create_queued_run(runs_dir, "old-run", "settings_test", "old prompt")
            .expect("create old run");
        set_run_updated_at(runs_dir, "old-run", hours_ago(EXPIRED_RUN_AGE_HOURS));

        create_queued_run(runs_dir, "new-run", "settings_test", "new prompt")
            .expect("create new run");

        assert!(!runs_dir.join("old-run").exists());
        assert!(runs_dir.join("new-run").join("result.json").exists());
    }

    #[test]
    fn recorded_run_dir_prunes_expired_run_before_opening_artifacts() {
        let temp = tempdir().expect("tempdir");
        let runs_dir = temp.path();

        create_queued_run(runs_dir, "old-run", "settings_test", "old prompt")
            .expect("create old run");
        let old_run_dir = runs_dir.join("old-run");
        finish_run(
            runs_dir,
            "old-run",
            GeminiBrowserRunResult {
                run_id: "old-run".to_string(),
                status: GeminiBrowserRunStatus::Ok,
                text: Some("answer".to_string()),
                message: None,
                manual_action: None,
                artifacts: GeminiBrowserArtifactRefs {
                    run_dir: Some(old_run_dir.to_string_lossy().to_string()),
                    ..Default::default()
                },
                elapsed_ms: 25,
                debug_summary: None,
            },
        )
        .expect("finish old run");
        set_run_updated_at(runs_dir, "old-run", hours_ago(EXPIRED_RUN_AGE_HOURS));

        assert!(super::recorded_run_dir(runs_dir, "old-run").is_err());
        assert!(!old_run_dir.exists());
    }

    fn hours_ago(hours: i64) -> String {
        (OffsetDateTime::now_utc() - Duration::hours(hours))
            .format(&Rfc3339)
            .expect("format timestamp")
    }

    fn set_run_updated_at(runs_dir: &std::path::Path, run_id: &str, updated_at: String) {
        let path = runs_dir.join(run_id).join("result.json");
        let mut run = super::read_run_file(&path).expect("read run");
        run.updated_at = updated_at;
        super::write_run(&path, &run).expect("write run");
    }
}
