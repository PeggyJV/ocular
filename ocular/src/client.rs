#![warn(unused_qualifications)]

use std::{any::{TypeId, Any}, collections::HashMap};

use crate::{
    error::{ChainClientError, RpcError},
};
use tendermint_rpc::{self, WebSocketClient, WebSocketClientDriver};

pub mod config;
pub mod query;
pub mod grpc;

type RpcHttpClient = tendermint_rpc::HttpClient;

pub struct ChainClient {
    grpc_endpoint: String,
    grpc_pool: HashMap<TypeId, Box<dyn Any>>,
    rpc_client: RpcHttpClient,
}

impl ChainClient {
    pub fn new(
        rpc_endpoint: &str,
        grpc_endpoint: &str,
    ) -> Result<ChainClient, ChainClientError> {
        let rpc_client = new_rpc_http_client(rpc_endpoint)?;

        Ok(ChainClient {
            grpc_endpoint: String::from(grpc_endpoint),
            grpc_pool: HashMap::new(),
            rpc_client,
        })
    }
}

pub fn new_rpc_http_client(address: &str) -> Result<RpcHttpClient, RpcError> {
    RpcHttpClient::new(address).map_err(|e| e.into())
}

pub async fn new_rpc_ws_client(
    address: &str,
) -> Result<(WebSocketClient, WebSocketClientDriver), RpcError> {
    WebSocketClient::new(address).await.map_err(|e| e.into())
}
