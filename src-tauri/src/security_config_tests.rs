use serde_json::Value;

fn base_config() -> Value {
    serde_json::from_str(include_str!("../tauri.conf.json")).expect("parse committed Tauri config")
}

fn mcp_config() -> Value {
    serde_json::from_str(include_str!("../tauri.mcp.conf.json"))
        .expect("parse committed MCP config")
}

fn default_capability() -> Value {
    serde_json::from_str(include_str!("../capabilities/default.json"))
        .expect("parse committed default capability")
}

#[test]
fn base_tauri_configuration_is_production_restrictive() {
    let config = base_config();
    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("string CSP");

    assert_eq!(config["app"]["withGlobalTauri"], false);
    assert!(!config["build"]["features"]
        .as_array()
        .is_some_and(|features| features
            .iter()
            .any(|feature| feature == "prompt-pack-dev-fixtures")));
    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("connect-src 'self' ipc: http://ipc.localhost"));
    assert!(csp.contains("img-src 'self' asset: http://asset.localhost data: blob:"));
    assert!(csp.contains("style-src 'self' 'unsafe-inline'"));
    assert!(csp.contains("font-src 'self' data:"));
    assert!(csp.contains("script-src 'self'"));
    assert_eq!(
        config["app"]["security"]["dangerousDisableAssetCspModification"],
        false
    );
}

#[test]
fn production_image_csp_rejects_remote_origins() {
    let config = base_config();
    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("string CSP");
    let image_sources = csp
        .split(';')
        .map(str::trim)
        .find_map(|directive| directive.strip_prefix("img-src "))
        .expect("image CSP directive");
    let remote = image_sources
        .split_whitespace()
        .filter(|source| source.starts_with("http://") || source.starts_with("https://"))
        .filter(|source| *source != "http://asset.localhost")
        .collect::<Vec<_>>();

    assert!(remote.is_empty(), "remote image origins: {remote:?}");
}

#[test]
fn frontend_capabilities_contain_no_sql_permission() {
    let capability = default_capability();
    let permissions = capability["permissions"]
        .as_array()
        .expect("capability permissions");
    assert!(permissions
        .iter()
        .filter_map(Value::as_str)
        .all(|permission| !permission.starts_with("sql:")));
}

#[test]
fn mcp_and_fixture_commands_are_localhost_dev_only() {
    macro_rules! registered_command_names {
        ([$($before:ident,)*], [$($(#[$after_attribute:meta])* $after:ident $(=> ($implementation:path; (($($parameter:ident : $parameter_type:ty),* $(,)?) -> $result:ty; [$($wire:literal),* $(,)?])))?),* $(,)?]; $($telegram:ident),* $(,)?) => {
            [
                $(stringify!($before),)*
                $(stringify!($telegram),)*
                $($(#[$after_attribute])* stringify!($after),)*
            ]
        };
    }
    let config = mcp_config();
    let registered = crate::application_command_inventory!(registered_command_names);
    assert_eq!(
        config["build"]["features"],
        serde_json::json!(["prompt-pack-dev-fixtures"])
    );
    assert_eq!(config["app"]["withGlobalTauri"], true);
    assert_eq!(crate::security_config::MCP_BIND_ADDRESS, "127.0.0.1");
    assert!(crate::security_config::require_local_dev_command(
        crate::security_config::MCP_BIND_ADDRESS
    )
    .is_ok());
    assert!(crate::security_config::require_local_dev_command("0.0.0.0").is_err());
    for fixture_command in [
        "seed_takeout_cancellation_smoke_fixture",
        "clear_takeout_cancellation_smoke_fixture",
        "seed_analysis_redesign_fixtures",
        "clear_analysis_redesign_fixture_active_runs",
        "clear_analysis_redesign_fixtures",
        "seed_source_job_cancellation_smoke_fixture",
        "clear_source_job_cancellation_smoke_fixture",
    ] {
        assert!(
            registered.contains(&fixture_command),
            "fixture command bypassed the production inventory: {fixture_command}"
        );
    }
    for fixture_command in [
        "seed_prompt_pack_cancellation_smoke_fixture",
        "clear_prompt_pack_cancellation_smoke_fixture",
    ] {
        assert_eq!(
            registered.contains(&fixture_command),
            cfg!(feature = "prompt-pack-dev-fixtures"),
            "prompt-pack fixture registration policy drifted: {fixture_command}"
        );
    }
}

#[test]
fn production_devtools_require_csp_verification() {
    assert!(!crate::security_config::production_devtools_allowed(false));
    assert_eq!(
        crate::security_config::production_devtools_allowed(true),
        cfg!(feature = "csp-verification")
    );
}
