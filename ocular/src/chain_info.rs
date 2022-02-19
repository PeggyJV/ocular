use crate::{
    chain_registry::{AssetList, self},
    config::ChainClientConfig,
    error::{ChainInfoError, RpcError},
};
use futures::executor;
use rand::{prelude::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use tendermint_rpc::Client;
use url::Url;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChainInfo {
    #[serde(rename = "$schema")]
    pub schema: String,
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
pub struct Genesis {
    pub genesis_url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Codebase {
    pub git_repo: String,
    pub recommended_version: String,
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub compatible_versions: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Peers {
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub seeds: Vec<Seed>,
    pub persistent_peers: Vec<PersistentPeer>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
pub struct Apis {
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub rpc: Vec<Rpc>,
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub rest: Vec<Rest>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Rpc {
    pub address: String,
    pub provider: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Rest {
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

    pub async fn get_asset_list(&self) -> Result<AssetList, ChainInfoError> {
        chain_registry::get_assets(self.chain_name.as_str())
            .await
            .map_err(|r| r.into())
    }

    pub fn get_chain_config(&self) -> Result<ChainClientConfig, ChainInfoError> {
        let mut gas_prices = String::default();
        let asset_list = executor::block_on(async {
            self.get_asset_list().await
        })?;
        if asset_list.assets.len() > 0 {
            gas_prices = format!("{:.2}{}", 0.01, asset_list.assets[0].base);
        }

        let rpc = executor::block_on(async {
            self.get_random_rpc_endpoint().await
        })?;

        Ok(ChainClientConfig {
            account_prefix: self.bech32_prefix.clone(),
            chain_id: self.chain_id.clone(),
            gas_adjustment: 1.2,
            gas_prices: gas_prices,
            grpc_address: "".to_string(),
            key: "default".to_string(),
            key_directory: "".to_string(),
            keyring_backend: "".to_string(),
            rpc_address: rpc,
        })
    }

    pub async fn get_random_rpc_endpoint(&self) -> Result<String, ChainInfoError> {
        let endpoints = self.get_rpc_endpoints().await?;
        if let Some(endpoint) = endpoints.choose(&mut thread_rng()) {
            return Ok(endpoint.to_string());
        } else {
            Err(RpcError::UnhealthyEndpoint("no available RPC endpoints".to_string()).into())
        }
    }

    pub async fn get_rpc_endpoints(&self) -> Result<Vec<String>, ChainInfoError> {
        let mut endpoints = self.get_all_rpc_endpoints();
        if endpoints.is_empty() {
            return Err(RpcError::UnhealthyEndpoint(
                "no valid endpoint found. endpoints must use http or https.".to_string(),
            )
            .into());
        }

        // this is not very efficient but i was getting annoyed trying to figure
        // out how to do filtering with an async method
        for (i, ep) in endpoints.clone().iter().enumerate() {
            if is_healthy_rpc(ep.as_str()).await.is_err() {
                endpoints.remove(i);
            }
        }
        if endpoints.is_empty() {
            return Err(RpcError::UnhealthyEndpoint(
                "no healthy endpoint found (connections could not be established)".to_string(),
            )
            .into());
        }

        Ok(endpoints)
    }
}

pub async fn get_cosmoshub_info() -> Result<ChainInfo, ChainInfoError> {
    chain_registry::get_chain("cosmoshub")
        .await
        .map_err(|r| r.into())
}

pub async fn is_healthy_rpc(endpoint: &str) -> Result<(), ChainInfoError> {
    let rpc_client = chain_client::new_rpc_client(endpoint)?;
    let status = rpc_client
        .status()
        .await
        .map_err(|e| RpcError::TendermintStatus(e))?;

    if status.sync_info.catching_up {
        return Err(RpcError::UnhealthyEndpoint("node is still syncing.".to_string()).into());
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    // TO-DO
}
