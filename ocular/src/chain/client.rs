#![warn(unused_qualifications)]

use crate::{
    chain::{client::cache::Cache, config::ChainClientConfig, registry::get_chain},
    error::{ChainClientError, RpcError},
    keyring::Keyring,
};
use futures::executor;
use tendermint_rpc::{self, WebSocketClient, WebSocketClientDriver};

use super::ChainName;

pub mod authz;
pub mod cache;
pub mod query;
pub mod tx;

type RpcHttpClient = tendermint_rpc::HttpClient;

pub struct ChainClient {
    pub config: ChainClientConfig,
    pub keyring: Keyring,
    pub rpc_client: RpcHttpClient,
    pub cache: Option<Cache>,
}

impl ChainClient {
    pub fn new(chain_name: ChainName) -> Result<Self, ChainClientError> {
        get_client(chain_name.as_str())
    }
}

fn get_client(chain_name: &str) -> Result<ChainClient, ChainClientError> {
    let chain = executor::block_on(async { get_chain(chain_name).await })?;
    let config = chain.get_chain_config()?;
    let keyring = Keyring::new_file_store(None)?;
    let rpc_client = new_rpc_http_client(config.rpc_address.as_str())?;
    // Default to in memory cache
    let cache = Cache::create_memory_cache(None)?;

    Ok(ChainClient {
        config,
        keyring,
        rpc_client,
        cache: Some(cache),
    })
}

pub fn new_rpc_http_client(address: &str) -> Result<RpcHttpClient, RpcError> {
    RpcHttpClient::new(address).map_err(|e| e.into())
}

pub async fn new_rpc_ws_client(
    address: &str,
) -> Result<(WebSocketClient, WebSocketClientDriver), RpcError> {
    WebSocketClient::new(address).await.map_err(|e| e.into())
}
