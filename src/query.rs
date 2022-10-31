#![warn(unused_qualifications)]
//! Defines the [`QueryClient`] and convenience methods for SDK module queries
//!
//! # Examples
//!
//! You can use ocular's [`QueryClient`] for querying:
//!
//! ```ignore
//! use ocular::QueryClient;
//!
//! async fn get_accounts_with_ocular_client() {
//!     let mut qclient = QueryClient::new("", "http://some.grpc:9090").unwrap();
//!     let accounts = qclient.all_accounts(None).await.accounts;
//! }
//! ```
//!
//! Or you can use the proto's query client directly, which is much more verbose:
//!
//! ```ignore
//! use ocular::cosmrs::proto::cosmos::auth::v1beta1::{BaseAccount, QueryAccountsRequest}, query::auth::AuthQueryClient;
//! use prost::Message;
//!
//! async fn get_accounts_with_proto_client() {
//!     let mut auth_qclient = AuthQueryClient::connect("http://some.grpc:9090").await.unwrap();
//!     let request = QueryAccountsRequest { pagination: None };
//!     let accounts: Vec<BaseAccount> = auth_qclient
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
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use async_trait::async_trait;
use cosmrs::proto::cosmos::tx::v1beta1::{GetTxResponse, GetTxRequest};
use eyre::{Result, Context};

use crate::tx::TxClient;

pub use self::{
    auth::*, authz::*, bank::*, distribution::*, evidence::*, gov::*, mint::*, params::*,
    slashing::*, staking::*,
};

pub mod auth;
pub mod authz;
pub mod bank;
pub mod distribution;
pub mod evidence;
pub mod feegrant;
pub mod gov;
pub mod mint;
pub mod params;
pub mod slashing;
pub mod staking;
pub mod tendermint;

/// Paging configuration for queries with potentially large result sets
pub type PageRequest = crate::cosmrs::proto::cosmos::base::query::v1beta1::PageRequest;
/// Information for requesting the next page of results
pub type PageResponse = crate::cosmrs::proto::cosmos::base::query::v1beta1::PageResponse;

#[async_trait]
impl GrpcClient for TxClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        TxClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

/// A convencience wrapper for querying Cosmos SDK module gRPC endpoints. It creates a Tendermint RPC client at construction
/// time. gRPC clients are created on demand because each module has it's own query client proto definition. [`QueryClient`]
/// keeps a cache of these gRPC clients, and will reuse them for subsequent queries to the same SDK module.
#[derive(Debug)]
pub struct QueryClient {
    grpc_endpoint: String,
    grpc_cache: HashMap<TypeId, Box<dyn Any>>,
}

impl QueryClient {
    /// Constructor
    ///
    /// ```ignore
    /// // Note that because of how prost generates the proto definitions,
    /// // it is necessary to bind the client as mutable in order to use it.
    /// let mut client = QueryClient::new(rpc, grpc)?;
    /// ```
    pub fn new(grpc_endpoint: &str) -> QueryClient {
        QueryClient {
            grpc_endpoint: String::from(grpc_endpoint),
            grpc_cache: HashMap::new(),
        }
    }

    /// Checks if the internal gRPC pool contains a client of the given module type. Primarily used for testing.
    pub fn has_grpc_client<T: 'static>(&self) -> bool {
        let key = TypeId::of::<T>();
        self.grpc_cache.contains_key(&key)
    }

    /// Retrieves a gRPC query client of the given type. If one exists in the pool it is used, otherwise one is added
    /// and returned.
    async fn get_grpc_query_client<T: 'static + Any + GrpcClient>(&mut self) -> Result<&mut T> {
        let key = TypeId::of::<T>();

        Ok(self
            .grpc_cache
            .entry(key)
            .or_insert(Box::new(
                new_grpc_query_client::<T>(&self.grpc_endpoint).await?,
            ))
            .downcast_mut::<T>()
            .unwrap())
    }

    /// Gets the inner gRPC endpoint
    pub fn grpc_endpoint(&self) -> &str {
        &self.grpc_endpoint
    }

    /// Gets a tx by its hash
    pub async fn tx_by_hash(&mut self, hash: &str) -> Result<GetTxResponse> {
        let client = self.get_grpc_query_client::<TxClient>().await?;
        let request = GetTxRequest { hash: hash.to_string() };

        Ok(client.get_tx(request).await?.into_inner())
    }
}

#[async_trait]
/// A marker trait for query client types in the Cosmos SDK proto
pub trait GrpcClient {
    /// The particular QueryClient type for the given module
    type ClientType;

    /// Constructor wrapper
    async fn make_client(endpoint: String) -> Result<Self::ClientType>;
}

/// Generalized constructor for query clients
pub async fn new_grpc_query_client<T>(endpoint: &str) -> Result<T::ClientType>
where
    T: GrpcClient,
{
    T::make_client(endpoint.to_string()).await
}
