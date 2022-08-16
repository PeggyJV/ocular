#![warn(unused_qualifications)]
//! This module contains RPC and gRPC queries, and query clients from each Cosmos SDK modules' respective proto definitions. Convenience methods are provided for some queries. For others, you can use the query client definition directly.
//!
//! # Examples
//!
//! ```no_run
//! use cosmos_sdk_proto::cosmos::auth::v1beta1::{BaseAccount, QueryAccountsRequest};
//! use ocular::{query::auth::AuthQueryClient, QueryClient};
//! use prost::Message;
//!
//! async fn get_accounts_example() {
//!     // with ocular's `QueryClient`
//!     let mut qclient = QueryClient::new("", "http://some.grpc:9090").unwrap();
//!     let accounts = qclient.all_accounts(None).await;
//!
//!     //or
//!
//!     // with proto query client
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
use eyre::Result;

pub use self::{
    auth::*, authz::*, bank::*, distribution::*, evidence::*, gov::*, mint::*, params::*,
    slashing::*, staking::*,
};
use crate::rpc::new_http_client;
pub use crate::rpc::RpcHttpClient;

pub mod auth;
pub mod authz;
pub mod bank;
pub mod distribution;
pub mod evidence;
pub mod gov;
pub mod mint;
pub mod params;
pub mod rpc;
pub mod slashing;
pub mod staking;

pub type PageRequest = cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;

pub struct QueryClient {
    grpc_endpoint: String,
    grpc_pool: HashMap<TypeId, Box<dyn Any>>,
    rpc_client: RpcHttpClient,
}

impl QueryClient {
    pub fn new(rpc_endpoint: &str, grpc_endpoint: &str) -> Result<QueryClient> {
        let rpc_client = new_http_client(rpc_endpoint)?;

        Ok(QueryClient {
            grpc_endpoint: String::from(grpc_endpoint),
            grpc_pool: HashMap::new(),
            rpc_client,
        })
    }

    pub fn has_grpc_client<T: 'static>(&self) -> bool {
        let key = TypeId::of::<T>();
        self.grpc_pool.contains_key(&key)
    }

    async fn get_grpc_query_client<T: 'static + Any + GrpcClient>(&mut self) -> Result<&mut T> {
        let key = TypeId::of::<T>();

        Ok(self.grpc_pool.entry(key).or_insert(
            Box::new(new_grpc_query_client::<T>(&self.grpc_endpoint).await?),
        )
        .downcast_mut::<T>()
        .unwrap())
    }
}

#[async_trait]
/// A marker trait for query client types in the Cosmos SDK proto
pub trait GrpcClient {
    type ClientType;

    async fn make_client(endpoint: String) -> Result<Self::ClientType>;
}

/// Constructor for query clients.
pub async fn new_grpc_query_client<T>(endpoint: &str) -> Result<T::ClientType>
where
    T: GrpcClient,
{
    T::make_client(endpoint.to_string()).await
}
