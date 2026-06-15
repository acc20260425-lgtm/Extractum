use std::collections::BTreeMap;
use std::time::Duration;

use tokio::process::Command;

use crate::error::AppResult;
use crate::llm::{
    llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key,
    load_provider_diagnostics_from_pool, LlmSchedulerState,
};
use crate::secret_store::SecretStoreState;
use crate::telegram::TelegramState;
use crate::youtube::jobs::SourceJobState;

use super::{
    DiagnosticLlmRequestCount, DiagnosticLlmRequestsInfo, DiagnosticProviderProfileCount,
    DiagnosticProvidersInfo, DiagnosticRuntimeCheck, DiagnosticRuntimeInfo, DiagnosticStatusCount,
    DiagnosticTelegramInfo, DiagnosticYoutubeJobCount, DiagnosticYoutubeJobsInfo,
};

const YTDLP_DIAGNOSTIC_TIMEOUT: Duration = Duration::from_secs(5);
const SECURE_STORAGE_READ_PROBE_KEY: &str = "__extractum_diagnostic_probe__";

pub(crate) async fn load_provider_diagnostics(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secret_store: &SecretStoreState,
) -> AppResult<DiagnosticProvidersInfo> {
    let state = load_provider_diagnostics_from_pool(pool, secret_store).await?;
    Ok(DiagnosticProvidersInfo {
        active_provider: state.active_provider,
        profiles_by_provider: state
            .profiles_by_provider
            .into_iter()
            .map(|count| DiagnosticProviderProfileCount {
                provider: count.provider,
                configured_count: count.configured_count,
                missing_key_count: count.missing_key_count,
            })
            .collect(),
    })
}

pub(crate) async fn load_in_memory_runtime_diagnostics(
    telegram_state: &TelegramState,
    source_job_state: &SourceJobState,
    llm_scheduler: &LlmSchedulerState,
    account_ids: &[i64],
    account_count: i64,
) -> (
    DiagnosticTelegramInfo,
    DiagnosticLlmRequestsInfo,
    DiagnosticYoutubeJobsInfo,
) {
    let runtime_statuses = telegram_state
        .diagnostic_status_counts(account_ids)
        .await
        .into_iter()
        .map(|(status, count)| DiagnosticStatusCount { status, count })
        .collect();

    let llm_requests = group_llm_request_snapshots(llm_scheduler.request_snapshots().await);

    let youtube_jobs = DiagnosticYoutubeJobsInfo {
        counts: source_job_state
            .diagnostic_counts()
            .await
            .into_iter()
            .map(|count| DiagnosticYoutubeJobCount {
                job_type: count.job_type,
                status: count.status,
                warning_state: count.warning_state,
                error_kind: count.error_kind,
                count: count.count,
            })
            .collect(),
    };

    (
        DiagnosticTelegramInfo {
            account_count,
            runtime_statuses,
        },
        llm_requests,
        youtube_jobs,
    )
}

fn group_llm_request_snapshots(
    snapshots: Vec<crate::llm::LlmRequestSnapshot>,
) -> DiagnosticLlmRequestsInfo {
    let mut counts = BTreeMap::<(String, String, String), i64>::new();
    for snapshot in snapshots {
        let kind = llm_request_kind_diagnostic_key(snapshot.kind).to_string();
        let state = llm_request_state_diagnostic_key(snapshot.state).to_string();
        *counts.entry((snapshot.provider, kind, state)).or_insert(0) += 1;
    }
    DiagnosticLlmRequestsInfo {
        counts: counts
            .into_iter()
            .map(
                |((provider, kind, state), count)| DiagnosticLlmRequestCount {
                    provider,
                    kind,
                    state,
                    count,
                },
            )
            .collect(),
    }
}

pub(crate) async fn check_ytdlp_runtime() -> DiagnosticRuntimeCheck {
    // This intentionally does not cache because the first diagnostics slice is
    // called on demand. If a future UI polls this command, add caching above the
    // command boundary instead of spawning yt-dlp repeatedly.
    match tokio::time::timeout(
        YTDLP_DIAGNOSTIC_TIMEOUT,
        Command::new("yt-dlp").arg("--version").output(),
    )
    .await
    {
        Ok(Ok(output)) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DiagnosticRuntimeCheck {
                status: "available".to_string(),
                available: true,
                version: if version.is_empty() {
                    None
                } else {
                    Some(version)
                },
                summary: None,
            }
        }
        Ok(Ok(_)) => failed_runtime_check("yt-dlp check failed"),
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => DiagnosticRuntimeCheck {
            status: "not_found".to_string(),
            available: false,
            version: None,
            summary: Some("yt-dlp is not available on PATH".to_string()),
        },
        // Do not include std::io::Error text here. It can contain local binary
        // paths on Unix-like systems and adds little diagnostic value.
        Ok(Err(_)) => failed_runtime_check("yt-dlp check failed"),
        Err(_) => DiagnosticRuntimeCheck {
            status: "timed_out".to_string(),
            available: false,
            version: None,
            summary: Some("yt-dlp runtime check timed out".to_string()),
        },
    }
}

pub(crate) async fn check_secure_storage(
    secret_store: &SecretStoreState,
) -> DiagnosticRuntimeCheck {
    // Read-only availability probe: Ok(None) means the store responded and the
    // diagnostic key simply does not exist. Do not write a probe key from the
    // diagnostics command; the command must remain read-only.
    match secret_store.get_secret(SECURE_STORAGE_READ_PROBE_KEY).await {
        Ok(Some(_)) | Ok(None) => DiagnosticRuntimeCheck {
            status: "available".to_string(),
            available: true,
            version: None,
            summary: None,
        },
        // Do not include OS/keychain error text here; it can contain local
        // paths or platform account details.
        Err(_) => failed_runtime_check("Secure storage check failed"),
    }
}

fn failed_runtime_check(summary: &str) -> DiagnosticRuntimeCheck {
    DiagnosticRuntimeCheck {
        status: "check_failed".to_string(),
        available: false,
        version: None,
        summary: Some(summary.to_string()),
    }
}

pub(crate) async fn load_runtime_checks(secret_store: &SecretStoreState) -> DiagnosticRuntimeInfo {
    DiagnosticRuntimeInfo {
        ytdlp: check_ytdlp_runtime().await,
        secure_storage: check_secure_storage(secret_store).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_store::tests::InMemorySecretStore;
    use std::sync::Arc;

    #[test]
    fn failed_runtime_check_uses_coarse_summary_without_os_error_text() {
        let check = failed_runtime_check("yt-dlp check failed");

        let json = serde_json::to_string(&check).expect("serialize runtime check");

        assert_eq!(check.status, "check_failed");
        assert_eq!(check.summary.as_deref(), Some("yt-dlp check failed"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/usr/local/bin"));
        assert!(!json.contains("os error"));
    }

    #[tokio::test]
    async fn secure_storage_failure_does_not_expose_store_error_text() {
        let store = Arc::new(InMemorySecretStore::new());
        store.fail_get("keychain failed for /home/user/.local/share/org.ai.extractum/session");
        let secret_store = SecretStoreState::new(store);

        let check = check_secure_storage(&secret_store).await;
        let json = serde_json::to_string(&check).expect("serialize runtime check");

        assert_eq!(check.status, "check_failed");
        assert_eq!(
            check.summary.as_deref(),
            Some("Secure storage check failed")
        );
        assert!(!json.contains("/home/user"));
        assert!(!json.contains("org.ai.extractum/session"));
        assert!(!json.contains("keychain failed"));
    }
}
