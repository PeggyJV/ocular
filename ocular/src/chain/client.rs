#![warn(unused_qualifications)]
//! This module defines the [`ChainClient`], it's constructors, and submodules that define all of its capabilities
use crate::{
    chain::{client::cache::Cache, config::ChainClientConfig},
    error::{ChainClientError, ChainRegistryError, RpcError},
    keyring::Keyring,
    registry::get_chain,
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

/// The core client type capable of configuring itself from the chain registry and managing its own
/// set of healthy gRPC endpoints
pub struct ChainClient {
    pub config: ChainClientConfig,
    pub keyring: Keyring,
    pub rpc_client: RpcHttpClient,
    pub cache: Option<Cache>,
    pub connection_retry_attempts: u8,
}

impl ChainClient {
    /// Constructor for an automatically configured [`ChainClient`] via the chain registry. Be aware that the
    /// endpoint health checks are currently not very thorough; you can manually set your desired RPC and/or
    /// gRPC endpoint for the client to use by setting them in the client's [`ChainClientConfig`].
    pub fn create(chain_name: ChainName) -> Result<Self, ChainClientError> {
        get_client(chain_name.as_str())
    }

    /// Constructor for manual composition
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

/// A basic builder that can be used to construct a [`ChainClient`] with a combination of automatic
/// and manual configuration. A [`Keyring`], [`Cache`], and gRPC/RPC endpoint selection can be set
/// via the builder's setters, while the rest of the chain's context will be retrieved from the
/// registry when the `build()` is called.
pub struct ChainClientBuilder {
    chain_name: ChainName,
    grpc_endpoint: Option<String>,
    rpc_endpoint: Option<String>,
    keyring: Option<Keyring>,
    cache: Option<Cache>,
    connection_retry_attempts: Option<u8>,
}

impl ChainClientBuilder {
    /// Builder constructor
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

    /// Retrieve the chain's context from the registry and construct the [`ChainClient`] using whatever
    /// values were manually provided.
    pub async fn build(self) -> Result<ChainClient, ChainClientError> {
        let info = match get_chain(self.chain_name.as_str()).await? {
            Some(c) => c,
            None => {
                return Err(ChainRegistryError::UnsupportedChain(format!(
                    "chain info for {} not found (no chain.json present)",
                    &self.chain_name
                ))
                .into())
            }
        };

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

    /// Set a [`Keyring`] to be used by the client
    pub fn with_keyring(mut self, keyring: Keyring) -> ChainClientBuilder {
        self.keyring = Some(keyring);
        self
    }

    /// Set a default gRPC endpoint to be used by the client
    pub fn with_grpc_endpoint(mut self, endpoint: &str) -> ChainClientBuilder {
        self.grpc_endpoint = Some(endpoint.to_string());
        self
    }

    /// Set a default RPC endpoint to be used by the client
    pub fn with_rpc_endpoint(mut self, endpoint: &str) -> ChainClientBuilder {
        self.rpc_endpoint = Some(endpoint.to_string());
        self
    }

    /// Set a [`Cache`] to be used by the client
    pub fn with_cache(mut self, cache: Cache) -> ChainClientBuilder {
        self.cache = Some(cache);
        self
    }
}

fn get_client(chain_name: &str) -> Result<ChainClient, ChainClientError> {
    let chain = match executor::block_on(async { get_chain(chain_name).await })? {
        Some(c) => c,
        None => {
            return Err(ChainRegistryError::UnsupportedChain(format!(
                "chain info for {} not found (no chain.json present)",
                chain_name
            ))
            .into())
        }
    };

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

/// Construct a new Tendermint RPC HTTP client
pub fn new_rpc_http_client(address: &str) -> Result<RpcHttpClient, RpcError> {
    RpcHttpClient::new(address).map_err(|e| e.into())
}

/// Constructs a new Tendermint RPC Websocket client
pub async fn new_rpc_ws_client(
    address: &str,
) -> Result<(WebSocketClient, WebSocketClientDriver), RpcError> {
    WebSocketClient::new(address).await.map_err(|e| e.into())
}
