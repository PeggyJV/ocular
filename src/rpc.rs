use eyre::{Result, Context};

pub type RpcHttpClient = tendermint_rpc::HttpClient;

pub fn new_http_client(address: &str) -> Result<RpcHttpClient> {
    RpcHttpClient::new(address)
        .wrap_err("Failed to contruct Tendermint RPC client")
}
