#[test]
fn analysis_report_workflow_file_has_no_tauri_command_adapters() {
    let source = include_str!("../../report_engine.rs");
    let command_attribute = ["#[tauri", "::command]"].join("");

    assert!(
        !source.contains(&command_attribute),
        "Analysis report command adapters should live outside src/analysis/report.rs"
    );
    assert!(
        source.contains("pub async fn execute_analysis_report"),
        "Portable report workflow should remain the architecture-test subject"
    );
    assert!(!source.contains("AppHandle"));
    assert!(!source.contains("get_pool"));
}
