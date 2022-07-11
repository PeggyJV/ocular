use crate::{error::ChainRegistryError, github::Content};
use http::Method;
use serde::de::DeserializeOwned;

pub use self::{assets::*, chain::*, paths::*};

#[cfg(all(feature = "registry-cache"))]
pub use self::cache::*;

pub mod assets;
#[cfg(feature = "registry-cache")]
pub mod cache;
pub mod chain;
pub mod paths;

const VERSION: &str = env!("CARGO_PKG_VERSION");
// In the future we may want to provide a way for a user to set the desired ref for the registry
// module to use when querying.
const GIT_REF: &str = "d063b0fd6d1c20d6476880e5ea2212ade009f69e";
const RAW_FILE_REPO_URL: &str = "https://raw.githubusercontent.com/cosmos/chain-registry";
const REPO_URL: &str = "https://api.github.com/repos/cosmos/chain-registry/contents";

async fn get(url: String) -> Result<String, ChainRegistryError> {
    let client = reqwest::Client::new();
    let req = client
        .request(Method::GET, url)
        .header("User-Agent", format!("ocular/{}", VERSION))
        .build()?;
    Ok(client.execute(req).await?.text().await?)
}

/// Gets a list of chain names from the registry
pub async fn list_chains() -> Result<Vec<String>, ChainRegistryError> {
    let url = format!("{}?ref={}", REPO_URL, GIT_REF,);
    let json = get(url).await?;
    let contents: Vec<Content> = serde_json::from_str(json.as_str())?;

    Ok(contents
        .iter()
        .filter(|c| c.type_field == "dir" && !c.name.starts_with('_') && c.name != ".github")
        .map(|c| c.clone().name)
        .collect())
}

/// Gets a list of path names from the registry in the form <chain_a>-<chain_b>
pub async fn list_paths() -> Result<Vec<String>, ChainRegistryError> {
    let url = format!("{}/_IBC?ref={}", REPO_URL, GIT_REF,);
    let json = get(url).await?;
    let contents: Vec<Content> = serde_json::from_str(json.as_str())?;

    Ok(contents
        .iter()
        .filter(|c| c.type_field == "file" && !c.name.starts_with('_') && c.name.ends_with(".json"))
        .map(|c| c.name[..c.name.len() - ".json".len()].to_string())
        .collect())
}

/// Retrieves the deserialized `assets.json` for a given chain. The result will contain
/// `None` if the there is no `assets.json` present.
///
/// # Arguments
///
/// * `name` - The chain name. Must match the name of the chain's folder in the root directory of the
/// [chain registry](https://github.com/cosmos/chain-registry).
pub async fn get_assets(name: &str) -> Result<Option<AssetList>, ChainRegistryError> {
    let path = format!("{}/assetlist.json", name);
    let data = get_file_content(GIT_REF, &path).await?;

    Ok(parse_json(data).await)
}

/// Retrieves the deserialized `chain.json` for a given chain. The result will contain
/// `None` if the there is no `chain.json` present.
///
/// # Arguments
///
/// * `name` - The chain name. Must match the name of the chain's folder in the root directory of the
/// [chain registry](https://github.com/cosmos/chain-registry).
pub async fn get_chain(name: &str) -> Result<Option<ChainInfo>, ChainRegistryError> {
    let path = format!("{}/chain.json", name);
    let data = get_file_content(GIT_REF, &path).await?;

    Ok(parse_json(data).await)
}

/// Retrieves the deserialized IBC path json for a given pair of chains. The result will contain
/// `None` if the there is no path present.
///
/// # Arguments
///
/// * `name` - The chain name. Must match the name of the chain's folder in the root directory of the
/// [chain registry](https://github.com/cosmos/chain-registry).
pub async fn get_path(chain_a: &str, chain_b: &str) -> Result<Option<IBCPath>, ChainRegistryError> {
    // path names order the chain names alphabetically
    let path = format!(
        "_IBC/{}-{}.json",
        chain_a.min(chain_b),
        chain_a.max(chain_b)
    );
    let data = get_file_content(GIT_REF, &path).await?;

    Ok(parse_json(data).await)
}

async fn get_file_content(r#ref: &str, path: &str) -> Result<String, ChainRegistryError> {
    let url = format!("{}/{}/{}", RAW_FILE_REPO_URL, r#ref, path);
    Ok(reqwest::get(url).await?.text().await?)
}

async fn parse_json<T>(data: String) -> Option<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(&data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assay::assay;

    #[assay]
    async fn gets_content_from_registry() {
        let result = get_file_content(GIT_REF, "cosmoshub/chain.json").await;

        result.unwrap();
    }

    #[assay]
    async fn parses_chain_info() {
        let result = get_file_content(GIT_REF, "cosmoshub/chain.json")
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

    #[assay]
    async fn lists_chains() {
        list_chains().await.unwrap();
    }

    #[assay]
    async fn lists_paths() {
        let paths = list_paths().await.unwrap();
        assert!(paths.len() > 0);
        paths
            .iter()
            .for_each(|path| assert!(!path.ends_with(".json")))
    }

    #[assay]
    async fn gets_path_in_order() {
        let chain_a = "cosmoshub";
        let chain_b = "osmosis";
        let result = get_path(chain_a, chain_b).await.unwrap().unwrap();
        assert_eq!(result.chain_1.chain_name, "cosmoshub");
        assert_eq!(result.chain_2.chain_name, "osmosis");
    }

    #[assay]
    async fn gets_path_out_of_order() {
        let chain_a = "cosmoshub";
        let chain_b = "osmosis";
        let result = get_path(chain_b, chain_a).await.unwrap().unwrap();
        assert_eq!(result.chain_1.chain_name, "cosmoshub");
        assert_eq!(result.chain_2.chain_name, "osmosis");
    }

    #[assay]
    async fn get_path_not_present_returns_none() {
        let chain_a = "fake";
        let chain_b = "osmosis";
        let result = get_path(chain_b, chain_a).await.unwrap();
        assert!(result.is_none())
    }
}
