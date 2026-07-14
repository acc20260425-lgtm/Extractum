use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

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
    pub(super) browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
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
                    AppError::internal(format!("parse Browser Provider config snapshot: {error}"))
                })?;
            Ok(RunRuntimeConfig {
                runtime_provider: RunRuntimeProvider::parse(&runtime_provider)?,
                profile_id,
                model_override,
                browser_provider_config,
            })
        },
    )
}
