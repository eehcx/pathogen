use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NftablesOutput {
    pub nftables: Vec<NftablesItem>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum NftablesItem {
    RuleWrapper { rule: NftRule },
    TableWrapper { table: serde_json::Value },
    ChainWrapper { chain: serde_json::Value },
    MetainfoWrapper { metainfo: serde_json::Value },
    Unknown(serde_json::Value),
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
    pub expr: Vec<serde_json::Value>,
}
