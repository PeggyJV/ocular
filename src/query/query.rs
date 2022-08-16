//! This module contains RPC and gRPC queries, and query clients from each Cosmos SDK modules' respective proto definitions. Convenience methods are provided for some queries. For others, you can use the query client definition directly.
//!
//! # Examples
//!
//! ```no_run
//! use cosmos_sdk_proto::cosmos::auth::v1beta1::{BaseAccount, QueryAccountsRequest};
//! use ocular::{chain::{COSMOSHUB, client::{ChainClient, query::*}}, error::ChainClientError};
//! use prost::Message;
//!
//! async fn get_accounts_example() {
//!     // with ocular's `ChainClient`
//!     let mut client = ChainClient::new("", "http://some.grpc:9090").unwrap();
//!     let accounts = client.query_accounts(None).await;
//!
//!     //or
//!
//!     // with proto query client
//!     let mut client = AuthQueryClient::connect("http://some.grpc:9090").await.unwrap();
//!     let request = QueryAccountsRequest { pagination: None };
//!     let accounts: Vec<BaseAccount> = client
//!         .accounts(request)
//!         .await
//!         .unwrap()
//!         .into_inner()
//!         .accounts
//!         .iter()
//!         .map(|any| BaseAccount::decode(&any.value as &[u8]).unwrap())
//!         .collect();
//! }
//! ```
use std::any::{TypeId, Any};

use async_trait::async_trait;
use eyre::Result;
use tendermint_rpc::{Client as RpcClient, HttpClient};

pub use self::{
    auth::*, authz::*, bank::*, distribution::*, evidence::*, gov::*, mint::*, params::*,
    slashing::*, staking::*,
};

use super::QueryClient;

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

pub type PageRequest = cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;

impl QueryClient {
    async fn get_grpc_query_client<T: 'static + Any + GrpcClient>(&mut self) -> Result<&mut T>
    {
        let key = TypeId::of::<T>();
        if !self.grpc_pool.contains_key(&key) {
            let _ = self.grpc_pool.insert(key, Box::new(new_grpc_query_client::<T>(&self.grpc_endpoint).await?));
        }

        return Ok(self.grpc_pool
            .get_mut(&key)
            .unwrap()
            .downcast_mut::<T>()
            .unwrap())
    }
}

#[async_trait]
/// A marker trait for query client types in the Cosmos SDK proto
pub trait GrpcClient
{
}

#[async_trait]
pub trait Connect
where
    Self: Sized
{
    async fn connect(endpoint: String) -> Result<Self>;
}

#[async_trait]
impl<T> Connect for T
where
    T: GrpcClient
{
    async fn connect(endpoint: String) -> Result<Self> {
        T::connect(endpoint).await
    }
}

/// A generic factory for query clients defined in the Cosmos SDK proto definitions
pub struct GrpcClientFactory;

impl GrpcClientFactory {
    pub async fn connect<T>(endpoint: String) -> Result<T>
    where
        T: GrpcClient + Connect,
    {
        T::connect(endpoint).await
    }
}

/// Constructor for query clients.
pub async fn new_grpc_query_client<T>(endpoint: &str) -> Result<T>
where
    T: GrpcClient,
{
    Ok(GrpcClientFactory::connect::<T>(endpoint.to_string()).await?)
}

/// RPC query for latest block height
pub async fn latest_height(rpc_client: &HttpClient) -> Result<u64> {
    let status = rpc_client
        .status()
        .await?;
    Ok(status.sync_info.latest_block_height.value())
}
