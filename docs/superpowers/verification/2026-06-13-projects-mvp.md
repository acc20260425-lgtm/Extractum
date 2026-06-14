# Projects MVP Verification

Date: 2026-06-14

## Automated

- `cargo test projects::tests --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test resolve_analysis_sources_ --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test analysis::report --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test library_sources::tests --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test delete_source_is_blocked_when_source_is_used_by_project --manifest-path src-tauri/Cargo.toml`: PASS
- `npm.cmd test -- --run src/lib/api/projects.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/research-projects-route-contract.test.ts`: PASS
- `npm.cmd test -- --run src/lib/analysis-utils.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/source-browser-model.test.ts src/lib/analysis-source-readers.test.ts`: PASS
- `npm.cmd run check`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS

## Manual Smoke

- Opened `/projects` through the Tauri MCP bridge: PASS
- Created project `MVP Smoke Project` and confirmed it appeared in `ProjectRail`: PASS
- Opened Library connection sheet and added one YouTube source: PASS
- Reopened Library connection sheet and confirmed the same YouTube source was disabled as `Already in project`: PASS
- Confirmed Project Inspector showed one source and enabled `Run project analysis`: PASS
- Added one Telegram source to the same project and confirmed run was disabled with `Mixed-provider project runs are not supported yet.`: PASS
- Removed the Telegram source and confirmed `Run project analysis` was enabled again: PASS
- Opened Run Project Analysis dialog and verified period, prompt, output language, and YouTube corpus fields: PASS
- Deleted the project and confirmed the YouTube and Telegram Library sources remained with `project_count: 0`: PASS
