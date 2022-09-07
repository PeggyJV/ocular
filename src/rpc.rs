//! Types and functions pertaining to Tendermint RPC
use eyre::{Context, Result};

/// Convenience alias for Tendermint RPC HTTP client type
pub type RpcHttpClient = crate::cosmrs::rpc::HttpClient;

/// Constructor for RPC client
pub fn new_http_client(address: &str) -> Result<RpcHttpClient> {
    RpcHttpClient::new(address).wrap_err("Failed to contruct Tendermint RPC client")
}
