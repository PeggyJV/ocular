/// Contains models for serializing and deserializing the `chain.json` in a given chain's directory in the registry repository,
/// as well as some methods for getting configuration for a [`ChainClient`]
use crate::{
    chain::{client, config::ChainClientConfig},
    error::{ChainInfoError, RpcError},
    registry::{self, assets::AssetList},
};
use futures::executor;
use rand::{prelude::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use tendermint_rpc::Client;
use url::Url;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ChainInfo {
    pub chain_name: String,
    pub status: String,
    pub network_type: String,
    pub pretty_name: String,
    pub chain_id: String,
    pub bech32_prefix: String,
    pub daemon_name: String,
    pub node_home: String,
    pub slip44: u32,
    pub genesis: Genesis,
    pub codebase: Codebase,
    pub peers: Peers,
    pub apis: Apis,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Genesis {
    pub genesis_url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Codebase {
    pub git_repo: String,
    pub recommended_version: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub compatible_versions: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Peers {
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub seeds: Vec<Seed>,
    pub persistent_peers: Vec<PersistentPeer>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Seed {
    pub id: String,
    pub address: String,
    pub provider: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PersistentPeer {
    pub id: String,
    pub address: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Apis {
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub rpc: Vec<Rpc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub rest: Vec<Rest>,
    pub grpc: Vec<Grpc>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Rpc {
    pub address: String,
    pub provider: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Rest {
    pub address: String,
    pub provider: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Grpc {
    pub address: String,
    pub provider: Option<String>,
}

impl ChainInfo {
    fn get_all_rpc_endpoints(&self) -> Vec<String> {
        self.apis
            .rpc
            .iter()
            .filter_map(|rpc| Url::parse(rpc.address.as_str()).ok())
            .filter_map(|url| {
                if url.scheme().contains("http") {
                    Some(url.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn get_asset_list(&self) -> Result<Option<AssetList>, ChainInfoError> {
        registry::get_assets(self.chain_name.as_str())
            .await
            .map_err(|r| r.into())
    }

    pub fn get_chain_config(&self) -> Result<ChainClientConfig, ChainInfoError> {
        let rpc_address = executor::block_on(async { self.get_random_rpc_endpoint().await })?;

        Ok(ChainClientConfig {
            account_prefix: self.bech32_prefix.clone(),
            chain_id: self.chain_id.clone(),
            gas_adjustment: 1.2,
            rpc_address,
            ..Default::default()
        })
    }

    pub async fn get_random_rpc_endpoint(&self) -> Result<String, ChainInfoError> {
        let endpoints = self.get_rpc_endpoints().await?;
        if let Some(endpoint) = endpoints.choose(&mut thread_rng()) {
            Ok(endpoint.to_string())
        } else {
            Err(RpcError::UnhealthyEndpoint("no available RPC endpoints".to_string()).into())
        }
    }

    pub async fn get_rpc_endpoints(&self) -> Result<Vec<String>, ChainInfoError> {
        let endpoints = self.get_all_rpc_endpoints();
        if endpoints.is_empty() {
            return Err(RpcError::MissingEndpoint(
                "no valid endpoint found. endpoints must use http or https.".to_string(),
            )
            .into());
        }

        // this is not very efficient but i was getting annoyed trying to figure
        // out how to do filtering with an async method
        // for (i, ep) in endpoints.clone().iter().enumerate() {
        //     if is_healthy_rpc(ep.as_str()).await.is_err() {
        //         endpoints.remove(i);
        //     }
        // }
        // if endpoints.is_empty() {
        //     return Err(RpcError::UnhealthyEndpoint(
        //         "no healthy endpoint found (connections could not be established)".to_string(),
        //     )
        //     .into());
        // }

        Ok(endpoints)
    }
}

pub async fn get_cosmoshub_info() -> Result<Option<ChainInfo>, ChainInfoError> {
    registry::get_chain("cosmoshub").await.map_err(|r| r.into())
}

pub async fn is_healthy_rpc(endpoint: &str) -> Result<(), ChainInfoError> {
    let rpc_client = client::new_rpc_http_client(endpoint)?;
    let status = rpc_client
        .status()
        .await
        .map_err(RpcError::TendermintStatus)?;

    if status.sync_info.catching_up {
        return Err(RpcError::UnhealthyEndpoint("node is still syncing.".to_string()).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assay::assay;

    #[assay]
    async fn gets_assets() {
        // as a unit test this shouldn't really rely on other parts
        // of the API but I don't want to get bogged down hardcoding
        // a ChainInfo right now.
        let info = get_cosmoshub_info()
            .await
            .expect("failed to get cosmoshub ChainInfo");
        let assets = info.unwrap().get_asset_list().await;

        assets.unwrap();
    }

    #[assay]
    async fn build_chain_config() {
        // as a unit test this shouldn't really rely on other parts
        // of the API but I don't want to get bogged down hardcoding
        // a ChainInfo right now.
        let info = get_cosmoshub_info()
            .await
            .expect("failed to get cosmoshub ChainInfo");
        let config = info.unwrap().get_chain_config();

        config.unwrap();
    }
}
