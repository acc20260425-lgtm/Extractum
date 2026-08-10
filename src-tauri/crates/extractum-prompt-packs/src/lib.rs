mod assets;
mod browser_port;
mod completion_transport;
mod dto;
mod events;
mod gemini_browser_stage;
mod json_repair;
mod library;
mod models;
mod projections;
mod result_builder;
mod result_service;
mod run_control;
mod run_store;
mod runtime;
mod runtime_config;
mod seed;
mod source_port;
mod stage_execution;
mod stage_io;
mod stage_output_normalization;
mod stage_request_policy;
mod store;
#[cfg(test)]
mod test_schema;
mod validation;
mod youtube_summary;

pub use browser_port::{
    PromptPackBrowserCancelRequest, PromptPackBrowserExecutor, PromptPackBrowserFuture,
    PromptPackBrowserRunRequest, PromptPackBrowserStatusRequest,
};
pub use dto::{
    ListPromptPackRunsRequest, PreflightYoutubeSummaryRunRequest, PromptPackAuditEventDto,
    PromptPackResultDto, PromptPackRunSummaryDto, PromptPackRuntimeProvider,
    PromptPackStageArtifactDto, PromptPackStageArtifactSummaryDto, PromptPackStageRunDto,
    PromptPackValidationFindingDto, StartYoutubeSummaryRunOutcomeDto,
    StartYoutubeSummaryRunRequest, YoutubeSummaryPreflightFailure, YoutubeSummaryPreflightResponse,
    YoutubeSummaryPreflightSkippedVideo, YoutubeSummaryPreflightVideo,
};
pub use events::{PromptPackEvent, PromptPackEventSink};
pub use library::{
    get_prompt_pack_library_in_pool, PromptPackDto, PromptPackLibraryDto, PromptPackSchemaAssetDto,
    PromptPackStageTemplateDto, PromptPackVersionDto,
};
pub use result_service::{
    get_prompt_pack_result_in_pool, get_prompt_pack_stage_artifact_in_pool,
    get_prompt_pack_validation_findings_in_pool, list_prompt_pack_audit_events_in_pool,
    list_prompt_pack_stage_artifacts_in_pool,
};
pub use run_control::PromptPackRunState;
pub use runtime::{
    cancel_prompt_pack_run_in_pool, cleanup_interrupted_prompt_pack_runs_in_pool,
    delete_prompt_pack_run_in_pool, dispatch_run_execution_ticket, execute_prepared_api_run,
    execute_prepared_browser_run, fail_run_execution, list_active_prompt_pack_runs_in_pool,
    list_prompt_pack_run_stages_in_pool, list_prompt_pack_runs_in_pool,
    preflight_youtube_summary_run, prepare_run_execution, start_youtube_summary_run_service,
    update_prompt_pack_run_in_pool, PreparedApiRunExecution, PreparedBrowserRunExecution,
    PreparedRunExecution, RunExecutionTicket, StartServiceOutcome,
};
#[cfg(any(test, feature = "dev-fixtures"))]
pub use runtime::{
    clear_prompt_pack_cancellation_smoke_fixture_in_pool,
    seed_prompt_pack_cancellation_smoke_fixture_in_pool,
};
pub use seed::seed_builtin_prompt_packs_in_pool;
pub use source_port::{
    CommentBodyReadRequest, CommentCandidateReadRequest, PromptPackCommentCandidate,
    PromptPackPlaylistItemRecord, PromptPackPortFuture, PromptPackSourceReader,
    PromptPackSourceRecord, PromptPackTranscriptSegment, PromptPackYoutubeVideoRecord,
    YoutubeVideoReadRequest,
};
pub use youtube_summary::YoutubeSummaryRunExecutionOutcome;

#[cfg(test)]
mod public_api_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::{Command, Output};

    use super::runtime::{
        clear_prompt_pack_cancellation_smoke_fixture_in_pool,
        seed_prompt_pack_cancellation_smoke_fixture_in_pool,
    };
    use super::{seed_builtin_prompt_packs_in_pool, PromptPackRunState};
    use crate::test_schema::prompt_pack_test_pool;

    fn external_fixture_probe(name: &str, feature_on: bool) -> Output {
        let probe_root = std::env::temp_dir().join(format!(
            "extractum-prompt-packs-public-api-{}",
            std::process::id()
        ));
        let root = probe_root.join(name);
        fs::create_dir_all(root.join("src")).expect("create external probe directory");
        let package_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .display()
            .to_string()
            .replace('\\', "/");
        let feature = if feature_on {
            ", features = [\"dev-fixtures\"]"
        } else {
            ""
        };
        fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"extractum_prompt_packs_external_{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n\n[dependencies]\nextractum-prompt-packs = {{ path = \"{package_path}\"{feature} }}\n"
            ),
        )
        .expect("write external probe manifest");
        fs::write(
            root.join("src/main.rs"),
            r#"
use extractum_prompt_packs::{
    clear_prompt_pack_cancellation_smoke_fixture_in_pool,
    seed_prompt_pack_cancellation_smoke_fixture_in_pool,
};
fn main() {
    let _ = (
        clear_prompt_pack_cancellation_smoke_fixture_in_pool,
        seed_prompt_pack_cancellation_smoke_fixture_in_pool,
    );
}
"#,
        )
        .expect("write external probe source");
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args([
                "check",
                "--quiet",
                "--offline",
                "--manifest-path",
                root.join("Cargo.toml").to_str().expect("utf-8 probe path"),
            ])
            .env("CARGO_TARGET_DIR", probe_root.join("target"))
            .output()
            .expect("run external Cargo probe")
    }

    fn clear_external_probe() {
        let _ = fs::remove_dir_all(std::env::temp_dir().join(format!(
            "extractum-prompt-packs-public-api-{}",
            std::process::id()
        )));
    }

    #[tokio::test]
    async fn cancellation_smoke_services_remain_test_only() {
        assert!(cfg!(test));

        let feature_off = external_fixture_probe("feature_off", false);
        assert!(
            !feature_off.status.success(),
            "cancellation smoke services leaked into the feature-off API"
        );
        let feature_off_stderr = String::from_utf8_lossy(&feature_off.stderr);
        assert!(
            feature_off_stderr.contains("unresolved imports"),
            "feature-off probe failed without the expected visibility diagnostic:\n{feature_off_stderr}"
        );
        let feature_on = external_fixture_probe("feature_on", true);
        assert!(
            feature_on.status.success(),
            "dev-fixtures consumer could not import cancellation smoke services:\n{}",
            String::from_utf8_lossy(&feature_on.stderr)
        );

        let pool = prompt_pack_test_pool().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed prompt-pack library");
        let state = PromptPackRunState::new();
        let run = seed_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, &state)
            .await
            .expect("seed cancellation smoke fixture");

        assert_eq!(state.active_run_ids().await, [run.run_id]);
        assert_eq!(
            clear_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, &state)
                .await
                .expect("clear cancellation smoke fixture"),
            1
        );
        assert!(state.active_run_ids().await.is_empty());
        clear_external_probe();
    }
}
