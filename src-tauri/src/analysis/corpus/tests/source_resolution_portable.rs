use super::super::super::models::{AnalysisSourceKind, ResolvedAnalysisScope};

#[test]
fn resolve_run_source_ids_prefers_snapshot_over_live_group_membership() {
    let frozen_snapshot_source_ids = vec![2, 4];
    let live_group_membership = [77];
    let resolved = ResolvedAnalysisScope::for_source_group(
        9,
        AnalysisSourceKind::Telegram,
        frozen_snapshot_source_ids,
        "Frozen group".to_string(),
    )
    .expect("resolve frozen source-group scope");

    assert_eq!(resolved.source_ids(), &[2, 4]);
    assert_ne!(resolved.source_ids(), live_group_membership);
}
