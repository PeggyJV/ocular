//! Defines [`GrpcClient`] for interacting with Cosmos chains over gRPC
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use async_trait::async_trait;
use eyre::Result;
use tonic::transport::Channel;

pub mod auth;
pub mod authz;
pub mod bank;
pub mod crisis;
pub mod distribution;
pub mod evidence;
pub mod feegrant;
pub mod gov;
pub mod mint;
pub mod params;
pub mod slashing;
pub mod staking;
pub mod tendermint;
pub mod tx;

/// Paging configuration for queries with potentially large result sets
pub type PageRequest = crate::cosmrs::proto::cosmos::base::query::v1beta1::PageRequest;
/// Information for requesting the next page of results
pub type PageResponse = crate::cosmrs::proto::cosmos::base::query::v1beta1::PageResponse;

/// The Cosmos Tx proto client type
pub type TxClient =
    crate::cosmrs::proto::cosmos::tx::v1beta1::service_client::ServiceClient<Channel>;

/// Client for broadcasting [`SignedTx`]
#[derive(Debug)]
pub struct GrpcClient {
    endpoint: String,
    tx_client: TxClient,
    query_client_cache: HashMap<TypeId, Box<dyn Any>>,
}

impl GrpcClient {
    /// Constructor
    pub async fn new(grpc_endpoint: &str) -> Result<GrpcClient> {
        Ok(GrpcClient {
            endpoint: String::from(grpc_endpoint),
            tx_client: TxClient::connect(grpc_endpoint.to_string()).await?,
            query_client_cache: HashMap::default(),
        })
    }

    /// Gets the inner gRPC endpoint
    pub fn grpc_endpoint(&self) -> String {
        self.endpoint.clone()
    }

    /// Gets a query client from the cache, or constructs a new one and caches it if one doesn't exist.
    pub async fn get_client<T: 'static + ConstructClient<T>>(&'_ mut self) -> Result<&'_ mut T> {
        let key = TypeId::of::<T>();
        let endpoint = self.grpc_endpoint();
        let cache = &mut self.query_client_cache;

        Ok(cache
            .entry(key)
            .or_insert(Box::new(T::new_client(endpoint).await?))
            .downcast_mut::<T>()
            .unwrap())
    }
}

/// Structs that implement this trait can be constructed/cached by a [`GrpcClient`]
#[async_trait]
pub trait ConstructClient<T> {
    /// Wraps a client constructor
    async fn new_client(endpoint: String) -> Result<T>;
}
