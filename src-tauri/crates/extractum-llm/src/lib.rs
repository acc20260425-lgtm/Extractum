mod gemini;
mod openai_compat;
mod provider;
mod runner;
mod scheduler;
mod streaming;
mod types;

pub use provider::{
    list_provider_models, normalize_base_url, resolve_model_input_token_limit,
    resolve_model_output_token_limit, ProviderKind,
};
pub use runner::{
    resolve_effective_model, run_llm_collect_with_profile, run_llm_stream_with_profile,
    validate_request,
};
pub use scheduler::{
    llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key, LlmRequestControl,
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmRequestSnapshot,
    LlmRequestSnapshotState, LlmSchedulerState,
};
pub use types::{
    LlmChatRequest, LlmCompletion, LlmMessage, LlmProviderAccess, LlmProviderModel, LlmUsage,
    ResolvedLlmProfile,
};

#[cfg(test)]
mod public_api_tests {
    use std::fmt::Debug;
    use std::fs;
    use std::path::PathBuf;
    use std::process::{Command, Output};

    use secrecy::{ExposeSecret, SecretString};
    use serde::{de::DeserializeOwned, Serialize};

    use super::*;

    trait AmbiguousIfSerialize<A> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfSerialize<()> for T {}
    impl<T: ?Sized + Serialize> AmbiguousIfSerialize<u8> for T {}

    trait AmbiguousIfDeserialize<A> {
        fn marker() {}
    }
    impl<T> AmbiguousIfDeserialize<()> for T {}
    impl<T: DeserializeOwned> AmbiguousIfDeserialize<u8> for T {}

    trait AmbiguousIfDebug<A> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfDebug<()> for T {}
    impl<T: ?Sized + Debug> AmbiguousIfDebug<u8> for T {}

    trait AmbiguousIfExposesSecret<A> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfExposesSecret<()> for T {}
    impl<T: ?Sized + ExposeSecret<String>> AmbiguousIfExposesSecret<u8> for T {}

    fn assert_type<T>() {}

    fn external_compile_probe(name: &str, source: &str) -> Output {
        let probe_root =
            std::env::temp_dir().join(format!("extractum-llm-public-api-{}", std::process::id()));
        let root = probe_root.join(name);
        let source_dir = root.join("src");
        fs::create_dir_all(&source_dir).expect("create external probe directory");
        let package_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .display()
            .to_string()
            .replace('\\', "/");
        fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"extractum_llm_external_{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n\n[dependencies]\nextractum-llm = {{ path = \"{package_path}\" }}\n"
            ),
        )
        .expect("write external probe manifest");
        fs::write(source_dir.join("main.rs"), source).expect("write external probe source");
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
        let _ = fs::remove_dir_all(
            std::env::temp_dir().join(format!("extractum-llm-public-api-{}", std::process::id())),
        );
    }

    fn assert_external_compile_fail(name: &str, source: &str, diagnostic: &str) {
        let output = external_compile_probe(name, source);
        assert!(!output.status.success(), "{name} unexpectedly compiled");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(diagnostic),
            "{name} failed without expected diagnostic {diagnostic:?}:\n{stderr}"
        );
    }

    #[test]
    fn curated_api_keeps_credentials_non_serializable_and_inaccessible() {
        let external_pass = external_compile_probe(
            "curated_pass",
            r#"
use extractum_llm::{
    list_provider_models, normalize_base_url, resolve_model_input_token_limit,
    resolve_model_output_token_limit, resolve_effective_model, run_llm_collect_with_profile,
    run_llm_stream_with_profile, validate_request, llm_request_kind_diagnostic_key,
    llm_request_state_diagnostic_key, ProviderKind, LlmRequestControl, LlmRequestError,
    LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmRequestSnapshot,
    LlmRequestSnapshotState, LlmSchedulerState, LlmChatRequest, LlmCompletion, LlmMessage,
    LlmProviderAccess, LlmProviderModel, LlmUsage, ResolvedLlmProfile,
};
fn main() {
    let _ = (list_provider_models, normalize_base_url, resolve_model_input_token_limit,
        resolve_model_output_token_limit, resolve_effective_model, run_llm_collect_with_profile,
        run_llm_stream_with_profile::<fn(&str)>, validate_request,
        llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key);
    let _ = std::mem::size_of::<(ProviderKind, LlmRequestControl, LlmRequestError,
        LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmRequestSnapshot,
        LlmRequestSnapshotState, LlmSchedulerState, LlmChatRequest, LlmCompletion, LlmMessage,
        LlmProviderAccess, LlmProviderModel, LlmUsage, ResolvedLlmProfile)>();
}
"#,
        );
        assert!(
            external_pass.status.success(),
            "curated external API failed to compile:\n{}",
            String::from_utf8_lossy(&external_pass.stderr)
        );
        assert_external_compile_fail(
            "private_types_module",
            "use extractum_llm::types::LlmProviderAccess; fn main() {}",
            "module `types` is private",
        );
        assert_external_compile_fail(
            "private_credential_field",
            "fn inspect(value: &extractum_llm::LlmProviderAccess) { let _ = &value.api_key; } fn main() {}",
            "field `api_key` of struct `LlmProviderAccess` is private",
        );
        assert_external_compile_fail(
            "private_credential_accessor",
            "fn inspect(value: &extractum_llm::LlmProviderAccess) { let _ = value.api_key(); } fn main() {}",
            "method `api_key` is private",
        );
        assert_external_compile_fail(
            "private_profile_accessor",
            "fn inspect(value: &extractum_llm::ResolvedLlmProfile) { let _ = value.provider_access(); } fn main() {}",
            "method `provider_access` is private",
        );
        assert_external_compile_fail(
            "non_curated_provider_module",
            "use extractum_llm::provider; fn main() {}",
            "module `provider` is private",
        );
        assert_external_compile_fail(
            "non_curated_gemini_module",
            "use extractum_llm::gemini; fn main() {}",
            "module `gemini` is private",
        );

        let _functions = (
            list_provider_models,
            normalize_base_url,
            resolve_model_input_token_limit,
            resolve_model_output_token_limit,
            resolve_effective_model,
            run_llm_collect_with_profile,
            run_llm_stream_with_profile::<fn(&str)>,
            validate_request,
            llm_request_kind_diagnostic_key,
            llm_request_state_diagnostic_key,
        );
        assert_type::<ProviderKind>();
        assert_type::<LlmRequestControl>();
        assert_type::<LlmRequestError>();
        assert_type::<LlmRequestKind>();
        assert_type::<LlmRequestMetadata>();
        assert_type::<LlmRequestPriority>();
        assert_type::<LlmRequestSnapshot>();
        assert_type::<LlmRequestSnapshotState>();
        assert_type::<LlmSchedulerState>();
        assert_type::<LlmChatRequest>();
        assert_type::<LlmCompletion>();
        assert_type::<LlmMessage>();
        assert_type::<LlmProviderAccess>();
        assert_type::<LlmProviderModel>();
        assert_type::<LlmUsage>();
        assert_type::<ResolvedLlmProfile>();

        let profile = ResolvedLlmProfile::new(
            "profile-441".to_string(),
            "model-441".to_string(),
            LlmProviderAccess::new(
                ProviderKind::OpenAiCompatible,
                SecretString::new("credential-441".to_string()),
                "https://llm.example/v1".to_string(),
            ),
        );
        assert_eq!(profile.profile_id(), "profile-441");
        assert_eq!(profile.provider(), ProviderKind::OpenAiCompatible);
        assert_eq!(profile.default_model(), "model-441");
        assert_eq!(profile.base_url(), "https://llm.example/v1");
        assert!(normalize_base_url(
            ProviderKind::OpenAiCompatible,
            Some("https://user:credential-441@llm.example/v1"),
        )
        .is_err());

        let _ = <LlmProviderAccess as AmbiguousIfSerialize<_>>::marker;
        let _ = <ResolvedLlmProfile as AmbiguousIfSerialize<_>>::marker;
        let _ = <LlmProviderAccess as AmbiguousIfDeserialize<_>>::marker;
        let _ = <ResolvedLlmProfile as AmbiguousIfDeserialize<_>>::marker;
        let _ = <LlmProviderAccess as AmbiguousIfDebug<_>>::marker;
        let _ = <ResolvedLlmProfile as AmbiguousIfDebug<_>>::marker;
        let _ = <LlmProviderAccess as AmbiguousIfExposesSecret<_>>::marker;
        let _ = <ResolvedLlmProfile as AmbiguousIfExposesSecret<_>>::marker;
        clear_external_probe();
    }
}
