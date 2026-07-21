use extractum_core::compression::compress_text;
use extractum_core::error::{AppError, AppResult};
use extractum_core::time::now_secs;
use sha2::{Digest, Sha384};
use sqlx::SqlitePool;

use super::assets::{
    BUNDLED_SOURCE_PATH, CANONICAL_RESULT_SCHEMA_JSON, PACK_JSON, TRANSCRIPT_INPUT_SCHEMA_JSON,
    TRANSCRIPT_OUTPUT_SCHEMA_JSON, TRANSCRIPT_STAGE_JSON,
};
use super::models::{BuiltinPackAsset, BuiltinSchemaAsset, BuiltinStageTemplateAsset};

pub async fn seed_builtin_prompt_packs_in_pool(pool: &SqlitePool) -> AppResult<()> {
    let pack: BuiltinPackAsset = serde_json::from_str(PACK_JSON)
        .map_err(|error| AppError::internal(format!("Parse bundled pack.json: {error}")))?;
    let stage: BuiltinStageTemplateAsset = serde_json::from_str(TRANSCRIPT_STAGE_JSON)
        .map_err(|error| AppError::internal(format!("Parse bundled stage template: {error}")))?;
    let schemas = schema_assets();
    let content_hash = bundled_content_hash(&[PACK_JSON, TRANSCRIPT_STAGE_JSON])
        + &schemas
            .iter()
            .map(|schema| bundled_content_hash(&[schema.content]))
            .collect::<String>();
    let content_hash = format!("sha384-{}", sha384_hex(content_hash.as_bytes()));
    let now = now_secs();

    let existing = sqlx::query_as::<_, (String, String)>(
        "SELECT origin_kind, content_hash FROM prompt_pack_versions WHERE pack_id = ? AND pack_version = ?",
    )
    .bind(&pack.pack_id)
    .bind(&pack.pack_version)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    if let Some((origin_kind, existing_hash)) = existing {
        if origin_kind == "user" {
            return Err(AppError::validation(format!(
                "Prompt pack collision for {}@{}",
                pack.pack_id, pack.pack_version
            )));
        }
        if existing_hash != content_hash {
            return Err(AppError::validation(format!(
                "Bundled prompt pack hash conflict for {}@{}",
                pack.pack_id, pack.pack_version
            )));
        }
    }

    sqlx::query(
        "INSERT INTO prompt_packs (pack_id, display_name, is_builtin, created_at, updated_at)
         VALUES (?, ?, 1, ?, ?)
         ON CONFLICT(pack_id) DO UPDATE SET
             display_name = excluded.display_name,
             is_builtin = 1,
             updated_at = excluded.updated_at",
    )
    .bind(&pack.pack_id)
    .bind(&pack.display_name)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        "INSERT INTO prompt_pack_versions (
            pack_id, pack_version, schema_version, origin_kind, lifecycle_status,
            content_hash, bundled_source_path, default_control_preset,
            default_evidence_mode, default_include_comments, seeded_at,
            last_seeded_at, created_at, updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(pack_id, pack_version) DO UPDATE SET
            last_seeded_at = excluded.last_seeded_at,
            updated_at = excluded.updated_at",
    )
    .bind(&pack.pack_id)
    .bind(&pack.pack_version)
    .bind(&pack.schema_version)
    .bind(&pack.origin_kind)
    .bind(&pack.lifecycle_status)
    .bind(&content_hash)
    .bind(BUNDLED_SOURCE_PATH)
    .bind(&pack.default_control_preset)
    .bind(&pack.default_evidence_mode)
    .bind(pack.default_include_comments)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    let pack_version_id =
        super::store::require_prompt_pack_version_id(pool, &pack.pack_id, &pack.pack_version)
            .await?;

    let prompt_template_json = serde_json::to_string(&stage.prompt_template)
        .map_err(|error| AppError::internal(format!("Serialize prompt template: {error}")))?;
    sqlx::query(
        "INSERT INTO prompt_pack_stage_templates (
            pack_version_id, pack_id, pack_version, schema_version, stage_name,
            stage_order, provider_family, input_schema_id, output_schema_id,
            validator_mode, prompt_template_json_zstd, content_hash, created_at, updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(pack_version_id, stage_name) DO UPDATE SET
            stage_order = excluded.stage_order,
            provider_family = excluded.provider_family,
            input_schema_id = excluded.input_schema_id,
            output_schema_id = excluded.output_schema_id,
            validator_mode = excluded.validator_mode,
            prompt_template_json_zstd = excluded.prompt_template_json_zstd,
            content_hash = excluded.content_hash,
            updated_at = excluded.updated_at",
    )
    .bind(pack_version_id)
    .bind(&pack.pack_id)
    .bind(&pack.pack_version)
    .bind(&pack.schema_version)
    .bind(&stage.stage_name)
    .bind(stage.stage_order)
    .bind(&stage.provider_family)
    .bind(&stage.input_schema_id)
    .bind(&stage.output_schema_id)
    .bind(&stage.validator_mode)
    .bind(compress_text(&prompt_template_json).map_err(AppError::internal)?)
    .bind(format!(
        "sha384-{}",
        sha384_hex(prompt_template_json.as_bytes())
    ))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    for schema in schemas {
        sqlx::query(
            "INSERT INTO prompt_pack_schema_assets (
                pack_version_id, pack_id, pack_version, schema_version, schema_id,
                schema_kind, content_hash, content_json_zstd, created_at, updated_at
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(pack_version_id, schema_id) DO UPDATE SET
                schema_kind = excluded.schema_kind,
                content_hash = excluded.content_hash,
                content_json_zstd = excluded.content_json_zstd,
                updated_at = excluded.updated_at",
        )
        .bind(pack_version_id)
        .bind(&pack.pack_id)
        .bind(&pack.pack_version)
        .bind(&pack.schema_version)
        .bind(schema.schema_id)
        .bind(schema.schema_kind)
        .bind(format!("sha384-{}", sha384_hex(schema.content.as_bytes())))
        .bind(compress_text(schema.content).map_err(AppError::internal)?)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    }

    Ok(())
}

fn schema_assets() -> Vec<BuiltinSchemaAsset> {
    vec![
        BuiltinSchemaAsset {
            schema_id: "stage-io/youtube_summary_transcript_analysis_input",
            schema_kind: "stage_input",
            content: TRANSCRIPT_INPUT_SCHEMA_JSON,
        },
        BuiltinSchemaAsset {
            schema_id: "stage-io/youtube_summary_transcript_analysis_output",
            schema_kind: "stage_output",
            content: TRANSCRIPT_OUTPUT_SCHEMA_JSON,
        },
        BuiltinSchemaAsset {
            schema_id: "canonical-result/youtube_summary",
            schema_kind: "canonical_result",
            content: CANONICAL_RESULT_SCHEMA_JSON,
        },
    ]
}

fn bundled_content_hash(parts: &[&str]) -> String {
    let mut normalized = String::new();
    for part in parts {
        normalized.push_str(part.trim());
        normalized.push('\n');
    }
    sha384_hex(normalized.as_bytes())
}

fn sha384_hex(bytes: &[u8]) -> String {
    Sha384::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::{seed_builtin_prompt_packs_in_pool, sha384_hex};
    use crate::assets::{
        CANONICAL_RESULT_SCHEMA_JSON, PACK_JSON, SYNTHESIS_OUTPUT_SCHEMA_JSON,
        SYNTHESIS_RUNTIME_JSON, TRANSCRIPT_INPUT_SCHEMA_JSON, TRANSCRIPT_OUTPUT_SCHEMA_JSON,
        TRANSCRIPT_RUNTIME_JSON, TRANSCRIPT_STAGE_JSON,
    };
    use crate::test_schema::prompt_pack_test_pool;
    use std::path::{Path, PathBuf};

    fn prompt_pack_domain_root() -> PathBuf {
        let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let prepared_root = manifest_root.join("src/prompt_packs");
        if prepared_root.is_dir() {
            prepared_root
        } else {
            manifest_root.join("src")
        }
    }

    fn collect_rust_sources(root: &Path, sources: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(root)
            .unwrap_or_else(|error| panic!("read {}: {error}", root.display()))
        {
            let path = entry.expect("read Prompt Pack source entry").path();
            if path.is_dir() {
                collect_rust_sources(&path, sources);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                sources.push(path);
            }
        }
    }

    fn asset_source_owners(asset_path: &str) -> Vec<String> {
        let root = prompt_pack_domain_root();
        let needle = format!("prompt-packs/youtube_summary/1.0.0/{asset_path}");
        let mut sources = Vec::new();
        collect_rust_sources(&root, &mut sources);
        let mut owners = sources
            .into_iter()
            .filter(|path| {
                std::fs::read_to_string(path)
                    .map(|source| source.contains(&needle))
                    .unwrap_or(false)
            })
            .map(|path| {
                path.strip_prefix(&root)
                    .expect("Prompt Pack source under root")
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect::<Vec<_>>();
        owners.sort();
        owners
    }

    async fn test_pool_with_migrations() -> sqlx::SqlitePool {
        prompt_pack_test_pool().await
    }

    #[tokio::test]
    async fn bundled_assets_hashes_and_source_path_match_canonical_bytes() {
        let assets = [
            (
                "pack.json",
                PACK_JSON,
                "21d0e7803f25474bb761cbe5c9fe6e45ef363cf5d9c7f030f7c84ee02ef9b7d8dd3664dfed782a3e8c607b7a0f37cf06",
            ),
            (
                "runtime/synthesis.json",
                SYNTHESIS_RUNTIME_JSON,
                "36b1c4653bc4befdcd168b482929f3b34980c58d9179cb0e0e3db9ac4d3760f9e66dc834ad6a799df6df62618b28d367",
            ),
            (
                "runtime/transcript_analysis.json",
                TRANSCRIPT_RUNTIME_JSON,
                "a9ba63c8ff582429866042aad354693cf9a583f5fc05f319189f44266d9eec6871b0ceb40758719a4b0d95dc8f25ee8f",
            ),
            (
                "schemas/canonical-result.json",
                CANONICAL_RESULT_SCHEMA_JSON,
                "067ac18d452b6ec6ca2000899d3e7d8df87ace30e4676c7f88080a59cc4731887032943c7ea961ac39b69ab17e9697fd",
            ),
            (
                "schemas/stage-io-youtube-summary-synthesis-output.json",
                SYNTHESIS_OUTPUT_SCHEMA_JSON,
                "ff518213fba16805dfbde2c6c55f8d3ca204ca7f772fb2348cfc375e83070289bfd29623ea4af1b78044504e92a22dac",
            ),
            (
                "schemas/stage-io-youtube-summary-transcript-analysis-input.json",
                TRANSCRIPT_INPUT_SCHEMA_JSON,
                "bb75aad9fd645912f723ad470a715f7b43c3af964ee4ea74cd84bebb635a1d3bc5bb0ac5460c9608e15eabee07b74419",
            ),
            (
                "schemas/stage-io-youtube-summary-transcript-analysis-output.json",
                TRANSCRIPT_OUTPUT_SCHEMA_JSON,
                "9d3d32cf7b7bfd00fdc5ae6d74dac8ad06f488b05e31e52866553aeaa1cd836c1d6599d5dd21c2228abf51e4bcc5f693",
            ),
            (
                "stages/transcript_analysis.json",
                TRANSCRIPT_STAGE_JSON,
                "1b4f18dc3b1baf4b01389a6187d54b96ed689dc044aefd6338a2a176779f433359b0bdc77364fec1ef2ccb58a9088793",
            ),
        ];

        for (path, content, expected_hash) in assets {
            assert_eq!(sha384_hex(content.as_bytes()), expected_hash, "{path}");
            assert_eq!(
                asset_source_owners(path),
                vec!["assets.rs".to_string()],
                "{path} must have exactly one compile-time owner"
            );
        }

        let pool = test_pool_with_migrations().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed bundled pack");
        let bundled_source_path = sqlx::query_scalar::<_, String>(
            "SELECT bundled_source_path FROM prompt_pack_versions
             WHERE pack_id = 'youtube_summary' AND pack_version = '1.0.0'",
        )
        .fetch_one(&pool)
        .await
        .expect("read bundled source path");

        assert_eq!(
            bundled_source_path,
            "src-tauri/prompt-packs/youtube_summary/1.0.0"
        );
    }

    #[tokio::test]
    async fn seed_youtube_summary_pack_is_idempotent() {
        let pool = test_pool_with_migrations().await;

        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("first seed");
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("second seed");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_versions WHERE pack_id = 'youtube_summary'",
        )
        .fetch_one(&pool)
        .await
        .expect("count pack versions");

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn seed_youtube_summary_pack_writes_required_schema_assets() {
        let pool = test_pool_with_migrations().await;

        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed");

        let schema_ids = sqlx::query_scalar::<_, String>(
            "SELECT schema_id FROM prompt_pack_schema_assets WHERE schema_id LIKE 'stage-io/%' ORDER BY schema_id",
        )
        .fetch_all(&pool)
        .await
        .expect("schema ids");

        assert_eq!(
            schema_ids,
            vec![
                "stage-io/youtube_summary_transcript_analysis_input".to_string(),
                "stage-io/youtube_summary_transcript_analysis_output".to_string(),
            ],
        );
    }

    #[tokio::test]
    async fn seed_youtube_summary_pack_rejects_bundled_hash_conflict() {
        let pool = test_pool_with_migrations().await;

        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed");

        sqlx::query(
            "UPDATE prompt_pack_versions SET content_hash = 'sha384-conflict' WHERE pack_id = 'youtube_summary' AND pack_version = '1.0.0'",
        )
        .execute(&pool)
        .await
        .expect("mutate content hash");

        let error = seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect_err("hash conflict rejected");

        assert!(error.to_string().contains("hash conflict"));
    }

    #[tokio::test]
    async fn seed_youtube_summary_pack_rejects_user_collision() {
        let pool = test_pool_with_migrations().await;

        sqlx::query(
            r#"
            INSERT INTO prompt_packs (pack_id, display_name, is_builtin, created_at, updated_at)
            VALUES ('youtube_summary', 'User YouTube Summary', 0, 1, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert colliding pack");

        sqlx::query(
            r#"
            INSERT INTO prompt_pack_versions (
                pack_id, pack_version, schema_version, origin_kind, lifecycle_status,
                content_hash, bundled_source_path, created_at, updated_at
            )
            VALUES (
                'youtube_summary', '1.0.0', '1.0', 'user', 'draft',
                'sha384-user-draft', NULL, 1, 1
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert colliding user draft");

        let error = seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect_err("user collision rejected");

        assert!(error.to_string().contains("collision"));
    }

    #[tokio::test]
    async fn seed_youtube_summary_pack_preserves_unknown_newer_bundled_version() {
        let pool = test_pool_with_migrations().await;

        sqlx::query(
            r#"
            INSERT INTO prompt_packs (pack_id, display_name, is_builtin, created_at, updated_at)
            VALUES ('youtube_summary', 'YouTube Summary', 1, 1, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert pack");

        sqlx::query(
            r#"
            INSERT INTO prompt_pack_versions (
                pack_id, pack_version, schema_version, origin_kind, lifecycle_status,
                content_hash, bundled_source_path, created_at, updated_at
            )
            VALUES (
                'youtube_summary', '9.9.9', '1.0', 'bundled', 'archived',
                'sha384-future', 'future-bundle', 1, 1
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert future bundled version");

        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed current bundle");

        let future_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_versions WHERE pack_id = 'youtube_summary' AND pack_version = '9.9.9'",
        )
        .fetch_one(&pool)
        .await
        .expect("future version count");

        assert_eq!(future_count, 1);
    }
}
