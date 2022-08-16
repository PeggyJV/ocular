//! Types and functions pertaining to Tendermint RPC
use eyre::{Context, Result};

pub type RpcHttpClient = tendermint_rpc::HttpClient;

pub fn new_http_client(address: &str) -> Result<RpcHttpClient> {
    RpcHttpClient::new(address).wrap_err("Failed to contruct Tendermint RPC client")
}
