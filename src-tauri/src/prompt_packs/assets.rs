pub(crate) const PACK_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prompt-packs/youtube_summary/1.0.0/pack.json"
));
pub(crate) const TRANSCRIPT_RUNTIME_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prompt-packs/youtube_summary/1.0.0/runtime/transcript_analysis.json"
));
pub(crate) const SYNTHESIS_RUNTIME_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json"
));
pub(crate) const CANONICAL_RESULT_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prompt-packs/youtube_summary/1.0.0/schemas/canonical-result.json"
));
pub(crate) const TRANSCRIPT_INPUT_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-input.json"
));
pub(crate) const TRANSCRIPT_OUTPUT_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-output.json"
));
pub(crate) const SYNTHESIS_OUTPUT_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-synthesis-output.json"
));
pub(crate) const TRANSCRIPT_STAGE_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prompt-packs/youtube_summary/1.0.0/stages/transcript_analysis.json"
));

pub(crate) const BUNDLED_SOURCE_PATH: &str = "src-tauri/prompt-packs/youtube_summary/1.0.0";
