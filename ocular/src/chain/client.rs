#![warn(unused_qualifications)]

use crate::{
    chain::{client::cache::Cache, config::ChainClientConfig, registry::get_chain},
    error::{ChainClientError, RpcError},
    keyring::Keyring,
};
use futures::executor;
use tendermint_rpc::{self, WebSocketClient, WebSocketClientDriver};

use super::ChainName;

pub mod airdrop;
pub mod cache;
pub mod grpc;
pub mod query;
pub mod tx;

type RpcHttpClient = tendermint_rpc::HttpClient;

pub struct ChainClient {
    pub config: ChainClientConfig,
    pub keyring: Keyring,
    pub rpc_client: RpcHttpClient,
    pub cache: Option<Cache>,
    pub connection_retry_attempts: u8,
}

impl ChainClient {
    pub fn create(chain_name: ChainName) -> Result<Self, ChainClientError> {
        get_client(chain_name.as_str())
    }

    pub fn new(
        config: ChainClientConfig,
        keyring: Keyring,
        cache: Option<Cache>,
        connection_retry_attempts: u8,
    ) -> Result<ChainClient, ChainClientError> {
        let rpc_client = new_rpc_http_client(config.rpc_address.as_str())?;

        Ok(ChainClient {
            config,
            keyring,
            rpc_client,
            cache,
            connection_retry_attempts,
        })
    }
}

pub struct ChainClientBuilder {
    chain_name: ChainName,
    grpc_endpoint: Option<String>,
    rpc_endpoint: Option<String>,
    keyring: Option<Keyring>,
    cache: Option<Cache>,
    connection_retry_attempts: Option<u8>,
}

impl ChainClientBuilder {
    pub fn new(chain_name: ChainName) -> ChainClientBuilder {
        ChainClientBuilder {
            chain_name,
            grpc_endpoint: None,
            rpc_endpoint: None,
            keyring: None,
            cache: None,
            connection_retry_attempts: None,
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
        let keyring = self.keyring.unwrap_or(Keyring::new_file_store(None)?);
        let connection_retry_attempts = self.connection_retry_attempts.unwrap_or(5);
        let cache = self
            .cache
            .unwrap_or(Cache::create_memory_cache(None, connection_retry_attempts)?);
        let rpc_client = new_rpc_http_client(config.rpc_address.as_str())?;

        Ok(ChainClient {
            config,
            keyring,
            rpc_client,
            cache: Some(cache),
            connection_retry_attempts,
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
    // Default to in memory cache
    let cache = Cache::create_memory_cache(None, 3)?;
    let config = chain.get_chain_config()?;
    let keyring = Keyring::new_file_store(None)?;
    let rpc_client = new_rpc_http_client(config.rpc_address.as_str())?;

    Ok(ChainClient {
        config,
        keyring,
        rpc_client,
        cache: Some(cache),
        connection_retry_attempts: 5,
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
