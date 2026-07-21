use extractum_core::error::{AppError, AppResult};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackLibraryDto {
    packs: Vec<PromptPackDto>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackDto {
    pack_id: String,
    display_name: String,
    active_version: Option<PromptPackVersionDto>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackVersionDto {
    pack_version_id: i64,
    pack_version: String,
    schema_version: String,
    lifecycle_status: String,
    default_control_preset: String,
    default_evidence_mode: String,
    default_include_comments: bool,
    stages: Vec<PromptPackStageTemplateDto>,
    schema_assets: Vec<PromptPackSchemaAssetDto>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackStageTemplateDto {
    stage_name: String,
    stage_order: i64,
    provider_family: String,
    input_schema_id: String,
    output_schema_id: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackSchemaAssetDto {
    schema_id: String,
    schema_kind: String,
    content_hash: String,
}

pub(crate) async fn get_prompt_pack_library_in_pool(
    pool: &SqlitePool,
) -> AppResult<PromptPackLibraryDto> {
    let packs = sqlx::query_as::<_, (String, String)>(
        "SELECT pack_id, display_name FROM prompt_packs ORDER BY display_name ASC, pack_id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut dtos = Vec::with_capacity(packs.len());
    for (pack_id, display_name) in packs {
        let active = sqlx::query_as::<_, (i64, String, String, String, String, String, bool)>(
            "SELECT id, pack_version, schema_version, lifecycle_status,
                    default_control_preset, default_evidence_mode, default_include_comments
             FROM prompt_pack_versions
             WHERE pack_id = ? AND lifecycle_status = 'active'
             ORDER BY id DESC
             LIMIT 1",
        )
        .bind(&pack_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;

        let active_version = if let Some((
            pack_version_id,
            pack_version,
            schema_version,
            lifecycle_status,
            default_control_preset,
            default_evidence_mode,
            default_include_comments,
        )) = active
        {
            let stages = sqlx::query_as::<_, (String, i64, String, String, String)>(
                "SELECT stage_name, stage_order, provider_family, input_schema_id, output_schema_id
                 FROM prompt_pack_stage_templates
                 WHERE pack_version_id = ?
                 ORDER BY stage_order ASC, stage_name ASC",
            )
            .bind(pack_version_id)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?
            .into_iter()
            .map(
                |(stage_name, stage_order, provider_family, input_schema_id, output_schema_id)| {
                    PromptPackStageTemplateDto {
                        stage_name,
                        stage_order,
                        provider_family,
                        input_schema_id,
                        output_schema_id,
                    }
                },
            )
            .collect::<Vec<_>>();

            let schema_assets = sqlx::query_as::<_, (String, String, String)>(
                "SELECT schema_id, schema_kind, content_hash
                 FROM prompt_pack_schema_assets
                 WHERE pack_version_id = ?
                 ORDER BY schema_id ASC",
            )
            .bind(pack_version_id)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?
            .into_iter()
            .map(
                |(schema_id, schema_kind, content_hash)| PromptPackSchemaAssetDto {
                    schema_id,
                    schema_kind,
                    content_hash,
                },
            )
            .collect::<Vec<_>>();

            Some(PromptPackVersionDto {
                pack_version_id,
                pack_version,
                schema_version,
                lifecycle_status,
                default_control_preset,
                default_evidence_mode,
                default_include_comments,
                stages,
                schema_assets,
            })
        } else {
            None
        };

        dtos.push(PromptPackDto {
            pack_id,
            display_name,
            active_version,
        });
    }

    Ok(PromptPackLibraryDto { packs: dtos })
}

#[cfg(test)]
mod tests {
    use super::get_prompt_pack_library_in_pool;
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::test_schema::prompt_pack_test_pool;
    use std::path::PathBuf;

    fn prompt_pack_domain_root() -> PathBuf {
        let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let prepared_root = manifest_root.join("src/prompt_packs");
        if prepared_root.is_dir() {
            prepared_root
        } else {
            manifest_root.join("src")
        }
    }

    fn assert_defines_pool_service(source_file: &str, function_name: &str) {
        let path = prompt_pack_domain_root().join(source_file);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        assert!(
            source.contains(&format!("fn {function_name}(")),
            "{source_file} must define {function_name}"
        );
    }

    fn assert_portable_pool_service_allowlist() {
        assert_defines_pool_service("library.rs", "get_prompt_pack_library_in_pool");
        assert_defines_pool_service("seed.rs", "seed_builtin_prompt_packs_in_pool");

        for function_name in [
            "cancel_prompt_pack_run_in_pool",
            "update_prompt_pack_run_in_pool",
            "delete_prompt_pack_run_in_pool",
            "list_prompt_pack_runs_in_pool",
            "list_active_prompt_pack_runs_in_pool",
            "list_prompt_pack_run_stages_in_pool",
            "cleanup_interrupted_prompt_pack_runs_in_pool",
            "seed_prompt_pack_cancellation_smoke_fixture_in_pool",
            "clear_prompt_pack_cancellation_smoke_fixture_in_pool",
        ] {
            assert_defines_pool_service("runtime.rs", function_name);
        }

        for function_name in [
            "get_prompt_pack_result_in_pool",
            "list_prompt_pack_stage_artifacts_in_pool",
            "get_prompt_pack_stage_artifact_in_pool",
            "get_prompt_pack_validation_findings_in_pool",
            "list_prompt_pack_audit_events_in_pool",
        ] {
            assert_defines_pool_service("result_service.rs", function_name);
        }
    }

    async fn seeded_pool() -> sqlx::SqlitePool {
        let pool = prompt_pack_test_pool().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed");
        pool
    }

    #[tokio::test]
    async fn get_prompt_pack_library_returns_active_youtube_summary_pack() {
        assert_portable_pool_service_allowlist();

        let pool = seeded_pool().await;

        let library = get_prompt_pack_library_in_pool(&pool)
            .await
            .expect("library");

        let pack = library
            .packs
            .iter()
            .find(|pack| pack.pack_id == "youtube_summary")
            .expect("youtube summary pack");
        assert_eq!(pack.display_name, "YouTube Summary");

        let version = pack.active_version.as_ref().expect("active version");
        assert_eq!(version.pack_version, "1.0.0");
        assert_eq!(version.schema_version, "1.0");
        assert_eq!(version.lifecycle_status, "active");
        assert_eq!(version.default_control_preset, "standard");
        assert_eq!(version.default_evidence_mode, "standard");
        assert!(!version.default_include_comments);

        assert_eq!(version.stages.len(), 1);
        assert_eq!(
            version.stages[0].stage_name,
            "youtube_summary/transcript_analysis"
        );
        assert_eq!(
            version.stages[0].input_schema_id,
            "stage-io/youtube_summary_transcript_analysis_input"
        );
        assert_eq!(
            version.stages[0].output_schema_id,
            "stage-io/youtube_summary_transcript_analysis_output"
        );

        let schema_ids = version
            .schema_assets
            .iter()
            .map(|schema| (schema.schema_id.as_str(), schema.schema_kind.as_str()))
            .collect::<Vec<_>>();
        assert!(schema_ids.contains(&(
            "stage-io/youtube_summary_transcript_analysis_input",
            "stage_input",
        )));
        assert!(schema_ids.contains(&(
            "stage-io/youtube_summary_transcript_analysis_output",
            "stage_output",
        )));
        assert!(schema_ids.contains(&("canonical-result/youtube_summary", "canonical_result")));
    }
}
