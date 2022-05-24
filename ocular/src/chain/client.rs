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
pub mod automated_tx_handler;
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
    pub fn create(chain_name: ChainName) -> Result<Self, ChainClientError> {
        get_client(chain_name.as_str())
    }

    pub fn new(
        config: ChainClientConfig,
        keyring: Keyring,
        cache: Option<Cache>,
    ) -> Result<ChainClient, ChainClientError> {
        let rpc_client = new_rpc_http_client(config.rpc_address.as_str())?;

        Ok(ChainClient {
            config,
            keyring,
            rpc_client,
            cache,
        })
    }
}

pub struct ChainClientBuilder {
    chain_name: ChainName,
    grpc_endpoint: Option<String>,
    rpc_endpoint: Option<String>,
    keyring: Option<Keyring>,
    cache: Option<Cache>,
}

impl ChainClientBuilder {
    pub fn new(chain_name: ChainName) -> ChainClientBuilder {
        ChainClientBuilder {
            chain_name,
            grpc_endpoint: None,
            rpc_endpoint: None,
            keyring: None,
            cache: None,
        }
    }

    pub async fn build(self) -> Result<ChainClient, ChainClientError> {
        let info = get_chain(self.chain_name.as_str()).await?;
        let mut config = info.get_chain_config()?;

        if self.grpc_endpoint.is_some() {
            config.grpc_address = self.grpc_endpoint.unwrap();
        }
        if self.rpc_endpoint.is_some() {
            config.rpc_address = self.rpc_endpoint.unwrap();
        }
        let keyring = match self.keyring {
            Some(kr) => kr,
            None => Keyring::new_file_store(None)?,
        };
        let cache = match self.cache {
            Some(c) => c,
            None => Cache::create_memory_cache(None)?,
        };
        let rpc_client = new_rpc_http_client(config.rpc_address.as_str())?;

        Ok(ChainClient {
            config,
            keyring,
            rpc_client,
            cache: Some(cache),
        })
    }

    pub fn with_keyring(mut self, keyring: Keyring) -> ChainClientBuilder {
        self.keyring = Some(keyring);
        self
    }

    pub fn with_grpc_endpoint(mut self, endpoint: &str) -> ChainClientBuilder {
        self.grpc_endpoint = Some(endpoint.to_string());
        self
    }

    pub fn with_rpc_endpoint(mut self, endpoint: &str) -> ChainClientBuilder {
        self.rpc_endpoint = Some(endpoint.to_string());
        self
    }

    pub fn with_cache(mut self, cache: Cache) -> ChainClientBuilder {
        self.cache = Some(cache);
        self
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
