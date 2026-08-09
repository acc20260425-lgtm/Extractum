use sqlx::SqlitePool;

use extractum_core::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RunRuntimeProvider {
    Api,
    GeminiBrowser,
}

impl RunRuntimeProvider {
    fn parse(value: &str) -> AppResult<Self> {
        match value {
            "api" => Ok(Self::Api),
            "gemini_browser" => Ok(Self::GeminiBrowser),
            other => Err(AppError::validation(format!(
                "Unsupported prompt-pack runtime provider: {other}"
            ))),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct RunRuntimeConfig {
    pub(super) runtime_provider: RunRuntimeProvider,
    pub(super) profile_id: Option<String>,
    pub(super) model_override: Option<String>,
    pub(super) browser_provider_config:
        Option<extractum_gemini_browser::GeminiBrowserProviderConfig>,
}

pub(super) async fn load_run_runtime_config(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<RunRuntimeConfig> {
    sqlx::query_as::<_, (Option<String>, Option<String>, String, Option<String>)>(
        "SELECT provider_profile_id, model, runtime_provider, browser_provider_config_json
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
    .and_then(
        |(profile_id, model_override, runtime_provider, browser_config_json)| {
            let browser_provider_config = browser_config_json
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|error| {
                    AppError::internal(format!(
                        "Invalid persisted runtime configuration for run {run_id}: browser provider config: {error}"
                    ))
                })?;
            Ok(RunRuntimeConfig {
                runtime_provider: RunRuntimeProvider::parse(&runtime_provider).map_err(|_| {
                    AppError::validation(format!(
                        "Invalid persisted runtime configuration for run {run_id}: unsupported provider '{runtime_provider}'"
                    ))
                })?,
                profile_id,
                model_override,
                browser_provider_config,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use extractum_core::error::AppErrorKind;

    use super::load_run_runtime_config;
    use crate::seed::seed_builtin_prompt_packs_in_pool;
    use crate::test_schema::prompt_pack_test_pool;

    #[tokio::test]
    async fn invalid_persisted_runtime_configuration_is_reported() {
        let pool = prompt_pack_test_pool().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed prompt pack");
        sqlx::query(
            "INSERT INTO prompt_pack_runs (
                id, project_id, pack_version_id, pack_id, pack_version,
                schema_version, run_status, result_status, runtime_provider,
                browser_provider_config_json, output_language, control_preset,
                evidence_mode, include_comments, latest_message, created_at, updated_at
             ) VALUES
                (4921, NULL, 1, 'youtube_summary', '1.0.0', '1.0',
                 'queued', 'none', 'api', NULL, 'en', 'standard', 'standard', 0,
                 'Queued', '2026-08-09T00:00:00Z', '2026-08-09T00:00:00Z'),
                (4922, NULL, 1, 'youtube_summary', '1.0.0', '1.0',
                 'queued', 'none', 'gemini_browser', '{not-json', 'en', 'standard',
                 'standard', 0, 'Queued', '2026-08-09T00:00:00Z', '2026-08-09T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert persisted runtime configs");
        let mut connection = pool.acquire().await.expect("acquire test connection");
        sqlx::query("PRAGMA ignore_check_constraints = ON")
            .execute(&mut *connection)
            .await
            .expect("allow invalid provider fixture");
        sqlx::query("UPDATE prompt_pack_runs SET runtime_provider = 'unsupported' WHERE id = 4921")
            .execute(&mut *connection)
            .await
            .expect("persist invalid provider");
        drop(connection);

        let provider_error = load_run_runtime_config(&pool, 4921)
            .await
            .expect_err("invalid provider must fail");
        let browser_error = load_run_runtime_config(&pool, 4922)
            .await
            .expect_err("invalid browser JSON must fail");

        assert_eq!(provider_error.kind, AppErrorKind::Validation);
        assert_eq!(
            provider_error.message,
            "Invalid persisted runtime configuration for run 4921: unsupported provider 'unsupported'"
        );
        assert_eq!(browser_error.kind, AppErrorKind::Internal);
        assert!(browser_error.message.starts_with(
            "Invalid persisted runtime configuration for run 4922: browser provider config:"
        ));
        assert!(
            browser_error.message.len()
                > "Invalid persisted runtime configuration for run 4922: browser provider config:"
                    .len()
        );
    }
}
