use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct NftablesOutput {
    pub nftables: Vec<NftablesItem>,
}

/// Variantes con contenido - el campo "rule" contiene el NftRule
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NftablesItem {
    #[serde(rename = "rule")]
    Rule {
        #[serde(flatten)]
        rule: NftRule,
    },
    #[serde(rename = "table")]
    Table(Value),
    #[serde(rename = "chain")]
    Chain(Value),
    #[serde(rename = "metainfo")]
    Metainfo(Value),
    #[serde(rename = "ct helper")]
    CtHelper(Value),
    /// Fallback para cualquier otra key
    Unknown(Value),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct NftRule {
    pub family: String,
    pub table: String,
    pub chain: String,
    pub handle: u64,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub expr: Vec<Value>,
}
