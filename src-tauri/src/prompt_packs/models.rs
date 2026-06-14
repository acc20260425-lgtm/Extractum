use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BuiltinPackAsset {
    pub pack_id: String,
    pub pack_version: String,
    pub schema_version: String,
    pub display_name: String,
    pub origin_kind: String,
    pub lifecycle_status: String,
    pub default_control_preset: String,
    pub default_evidence_mode: String,
    pub default_include_comments: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BuiltinStageTemplateAsset {
    pub stage_name: String,
    pub stage_order: i64,
    pub provider_family: String,
    pub input_schema_id: String,
    pub output_schema_id: String,
    pub validator_mode: String,
    pub prompt_template: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct BuiltinSchemaAsset {
    pub schema_id: &'static str,
    pub schema_kind: &'static str,
    pub content: &'static str,
}
