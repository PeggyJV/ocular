use crate::{chain::info::ChainInfo, error::ChainRegistryError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AssetList {
    #[serde(rename = "$schema")]
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
    #[serde(rename = "logo_URIs")]
    pub logo_uris: LogoURIs,
    pub coingecko_id: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u16,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LogoURIs {
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
    let path = format!("{}/assetlist.json", name);
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
        .map_err(|e| e.into())
}

async fn parse_json<T>(data: reqwest::Response) -> Result<T, ChainRegistryError>
where
    T: DeserializeOwned,
{
    let content = data.text().await.unwrap();
    serde_json::from_str(content.as_str()).map_err(|r| r.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assay::assay;

    #[assay]
    async fn gets_content_from_registry() {
        let result = get_content("cosmoshub/chain.json".to_string()).await;

        result.unwrap();
    }

    #[assay]
    async fn parses_chain_info() {
        let result = get_content("cosmoshub/chain.json".to_string())
            .await
            .unwrap();
        let result = parse_json::<ChainInfo>(result).await;

        result.unwrap();
    }

    #[assay]
    async fn gets_chain() {
        let result = get_chain("cosmoshub").await;

        result.unwrap();
    }
}
