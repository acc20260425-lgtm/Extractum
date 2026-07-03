#[test]
fn analysis_report_workflow_file_has_no_tauri_command_adapters() {
    let source = std::fs::read_to_string("src/analysis/report.rs").expect("read report.rs");
    let command_attribute = ["#[tauri", "::command]"].join("");

    assert!(
        !source.contains(&command_attribute),
        "Analysis report command adapters should live outside src/analysis/report.rs"
    );
}
