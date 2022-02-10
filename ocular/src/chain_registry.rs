use crate::{chain_info::ChainInfo, error::ChainRegistryError};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const REGISTRY_SOURCE: &str = "https://github.com/cosmos/chain-registry";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AssetList {
    pub schema: String,
    pub chain_id: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Asset {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub name: String,
    pub display: String,
    pub symbol: String,
    logo_uris: Vec<LogoURI>,
    pub coingecko_id: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u16,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LogoURI {
    pub png: String,
    pub svg: String,
}

// TO-DO this needs to list chains added (in ~/.ocular/config.toml) and
// print in JSON format, not a new-line seperated list of names.
pub async fn list_chains() -> Result<Vec<String>, ChainRegistryError> {
    let mut content = octocrab::instance()
        .repos("cosmos", "chain-registry")
        .get_content()
        .send()
        .await?;

    Ok(content
        .take_items()
        .into_iter()
        .filter(|item| item.r#type == "dir" && item.name != ".github")
        .map(|item| item.name)
        .collect())
}

pub async fn get_assets(name: &str) -> Result<AssetList, ChainRegistryError> {
    let path = format!("{}/chain.json", name);
    let data = get_content(path).await?;

    parse_json(data).await
}

pub async fn get_chain(name: &str) -> Result<ChainInfo, ChainRegistryError> {
    let path = format!("{}/chain.json", name);
    let data = get_content(path).await?;

    parse_json(data).await
}

async fn get_content(path: String) -> Result<reqwest::Response, ChainRegistryError> {
    octocrab::instance()
        .repos("cosmos", "chain-registry")
        .raw_file("HEAD".to_string(), path)
        .await
        .map_err(|r| r.into())
}

pub async fn is_healthy_rpc(endpoint: &str) -> bool {
    // TO-DO
    true
}

pub fn get_source() -> &'static str {
    REGISTRY_SOURCE
}

async fn parse_json<T>(data: reqwest::Response) -> Result<T, ChainRegistryError>
where T: DeserializeOwned {
    let content = data.text().await.unwrap();
    serde_json::from_str(content.as_str()).map_err(|r| {r.into()})
}
