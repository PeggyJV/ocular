use std::{str::FromStr, time::Duration};

use crate::{
    config::ChainClientConfig,
    error::{ChainClientError, RpcError},
};
use tendermint_rpc::{self, HttpClient, HttpClientUrl};

pub mod query;

type RpcClient = tendermint_rpc::HttpClient;

pub struct ChainClient {
    pub config: ChainClientConfig,
    // keybase: ?
    // KeyringOptions: Vec<?>
    pub rpc_client: RpcClient,
    // light_provider: ?
    // input:
    // output:
    // codec: ? // needed?
    // logger needed? i think rust does logging differently
}

impl ChainClient {
    pub fn new(config: ChainClientConfig) -> Result<Self, ChainClientError> {
        let rpc_client = new_rpc_client(config.rpc_address.as_str())?;

        Ok(ChainClient {
            config: config,
            rpc_client: rpc_client,
        })
    }
}

pub fn new_rpc_client(address: &str) -> Result<RpcClient, RpcError> {
    tendermint_rpc::HttpClient::new(address).map_err(|e| e.into())
}
