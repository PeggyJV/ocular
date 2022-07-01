//! This module contains RPC and gRPC queries, and query clients from each Cosmos SDK modules' respective proto definitions. Convenience methods are provided for some queries. For others, you can use the query client definition directly.
//!
//! # Examples
//!
//! ```no_run
//! use cosmos_sdk_proto::cosmos::auth::QueryAccountsRequest;
//! use ocular::chain::{ChainName::COSMOSHUB, client::ChainClient, query::*};
//!
//! // with ocular's `ChainClient`
//! let client = ChainClient::create(COSMOSHUB).await.unwrap();
//! let accounts = client.query_accounts(None).await.unwrap();
//!
//! // with proto query client
//! let client = AuthQueryClient::connect("http://some-grpc-endpoint.com:9090").await.unwrap();
//! let request = QueryAccountsRequest { pagination: None };
//! let accounts = client
//!     .accounts()
//!     .await?
//!     .into_inner()
//!     .accounts
//!     .iter()
//!     .map(|any| auth::BaseAccount::decode(&any.value as &[u8]).unwrap())
//!     .collect()
//! ```
use async_trait::async_trait;
use tendermint_rpc::Client as RpcClient;
use tonic::transport::Channel;

use crate::error::{ChainClientError, GrpcError, RpcError, TxError};

use super::ChainClient;

pub use self::{
    auth::*, authz::*, bank::*, distribution::*, evidence::*, gov::*, mint::*, params::*,
    slashing::*, staking::*,
};

pub mod auth;
pub mod authz;
pub mod bank;
pub mod distribution;
pub mod evidence;
pub mod gov;
pub mod mint;
pub mod params;
pub mod slashing;
pub mod staking;

#[async_trait]
/// A marker trait for query client types in the Cosmos SDK proto
pub trait QueryClient
where
    Self: Sized
{
    type Transport;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error>;
}

pub struct ClientFactory;

impl ClientFactory {
    pub async fn connect<T>(endpoint: String) -> Result<T, tonic::transport::Error>
    where
        T: Sized + QueryClient<Transport = Channel>
    {
        T::connect(endpoint).await
    }
}

impl ChainClient {
    /// Constructor for query clients.
    pub async fn get_query_client<T>(&mut self) -> Result<T, ChainClientError>
    where
        T: QueryClient<Transport = Channel>
    {
        let mut result: Result<T, ChainClientError> =
            Err(TxError::Broadcast(String::from("Client connection never attempted.")).into());

        // Get grpc address randomly each time; shuffles on failures
        for _i in 0u8..self.connection_retry_attempts + 1 {
            // Attempt to use last healthy (or manually set) endpoint if it exists (in config)
            let endpoint: String = if !self.config.grpc_address.is_empty() {
                self.config.grpc_address.clone()
            } else {
                // Get a random endpoint from the cache
                match self.get_random_grpc_endpoint().await {
                    Ok(endpt) => endpt,
                    Err(err) => return Err(GrpcError::MissingEndpoint(err.to_string()).into()),
                }
            };

            result = ClientFactory::connect::<T>(endpoint.clone())
                .await
                .map_err(|e| GrpcError::Connection(e).into());

            // Return if result is valid client, or increment failure in cache if being used
            if result.is_ok() {
                // Reset consecutive failed connections to 0
                self.cache
                    .as_mut()
                    .unwrap()
                    .grpc_endpoint_cache
                    .add_item(endpoint.clone(), 0)?;

                // Update config to last healthy grpc connection address
                self.config.grpc_address = endpoint.clone();

                break;
            } else if result.is_err() && self.cache.is_some() {
                // Don't bother updating config grpc address if it fails, it'll be overriden upon a successful connection
                let _res = self
                    .cache
                    .as_mut()
                    .unwrap()
                    .grpc_endpoint_cache
                    .increment_failed_connections(endpoint)?;
            }
        }

        result
    }

    /// RPC query for latest block height
    pub async fn query_latest_height(&self) -> Result<u64, ChainClientError> {
        let status = self
            .rpc_client
            .status()
            .await
            .map_err(RpcError::TendermintStatus)?;
        Ok(status.sync_info.latest_block_height.value())
    }
}
