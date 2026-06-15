use sha2::{Digest, Sha384};
use sqlx::SqlitePool;
use tauri::AppHandle;

use super::models::{BuiltinPackAsset, BuiltinSchemaAsset, BuiltinStageTemplateAsset};
use crate::compression::compress_text;
use crate::db::get_pool;
use crate::error::{AppError, AppResult};

const PACK_JSON: &str = include_str!("../../prompt-packs/youtube_summary/1.0.0/pack.json");
const TRANSCRIPT_ANALYSIS_JSON: &str =
    include_str!("../../prompt-packs/youtube_summary/1.0.0/stages/transcript_analysis.json");
const INPUT_SCHEMA_JSON: &str = include_str!(
    "../../prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-input.json"
);
const OUTPUT_SCHEMA_JSON: &str = include_str!(
    "../../prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-output.json"
);
const CANONICAL_RESULT_JSON: &str =
    include_str!("../../prompt-packs/youtube_summary/1.0.0/schemas/canonical-result.json");

pub async fn seed_builtin_prompt_packs(handle: AppHandle) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    seed_builtin_prompt_packs_in_pool(&pool).await
}

pub(crate) async fn seed_builtin_prompt_packs_in_pool(pool: &SqlitePool) -> AppResult<()> {
    let pack: BuiltinPackAsset = serde_json::from_str(PACK_JSON)
        .map_err(|error| AppError::internal(format!("Parse bundled pack.json: {error}")))?;
    let stage: BuiltinStageTemplateAsset = serde_json::from_str(TRANSCRIPT_ANALYSIS_JSON)
        .map_err(|error| AppError::internal(format!("Parse bundled stage template: {error}")))?;
    let schemas = schema_assets();
    let content_hash = bundled_content_hash(&[PACK_JSON, TRANSCRIPT_ANALYSIS_JSON])
        + &schemas
            .iter()
            .map(|schema| bundled_content_hash(&[schema.content]))
            .collect::<String>();
    let content_hash = format!("sha384-{}", sha384_hex(content_hash.as_bytes()));
    let now = unix_timestamp();

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
    .bind("src-tauri/prompt-packs/youtube_summary/1.0.0")
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
            content: INPUT_SCHEMA_JSON,
        },
        BuiltinSchemaAsset {
            schema_id: "stage-io/youtube_summary_transcript_analysis_output",
            schema_kind: "stage_output",
            content: OUTPUT_SCHEMA_JSON,
        },
        BuiltinSchemaAsset {
            schema_id: "canonical-result/youtube_summary",
            schema_kind: "canonical_result",
            content: CANONICAL_RESULT_JSON,
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

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::seed_builtin_prompt_packs_in_pool;
    use crate::migrations::apply_all_migrations_for_test_pool;

    async fn test_pool_with_migrations() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        pool
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
